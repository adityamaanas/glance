//! Newline-delimited local IPC: Unix sockets and Windows named pipes.
use interprocess::local_socket::{prelude::*, GenericFilePath, Stream};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::time::{Duration, Instant};

pub struct Connection {
    reader: BufReader<Stream>,
    /// Bytes of an incomplete frame, kept across timeouts so no frame is lost.
    pending: Vec<u8>,
}

const MAX_FRAME: usize = 4 * 1024 * 1024;
const SEND_TIMEOUT: Duration = Duration::from_secs(5);

impl Connection {
    pub fn connect(path: &Path) -> io::Result<Self> {
        let stream = Stream::connect(path.to_fs_name::<GenericFilePath>()?)?;
        // Unix sockets block with OS-level timeouts, so an idle reader sleeps in the kernel.
        // Windows named pipes have no timeouts; they use nonblocking reads with backoff.
        #[cfg(windows)]
        stream.set_nonblocking(true)?;
        #[cfg(not(windows))]
        stream.set_send_timeout(Some(SEND_TIMEOUT))?;
        Ok(Self {
            reader: BufReader::new(stream),
            pending: Vec::new(),
        })
    }

    pub fn send(&mut self, bytes: &[u8]) -> io::Result<()> {
        let deadline = Instant::now() + SEND_TIMEOUT;
        let mut pending = bytes;
        while !pending.is_empty() {
            match self.reader.get_mut().write(pending) {
                Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
                Ok(n) => pending = &pending[n..],
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) if e.kind() == io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Read one newline-terminated frame. A timeout keeps any partial frame for the next call.
    pub fn line(&mut self, timeout: Duration) -> io::Result<String> {
        let deadline = Instant::now() + timeout;
        let mut idle = Duration::from_millis(5);
        loop {
            let now = Instant::now();
            if now >= deadline {
                return Err(io::ErrorKind::TimedOut.into());
            }
            #[cfg(not(windows))]
            self.reader
                .get_ref()
                .set_recv_timeout(Some((deadline - now).max(Duration::from_millis(1))))?;
            match self.reader.fill_buf() {
                // Byte-mode Windows pipes in NOWAIT mode return zero when temporarily empty.
                Ok([]) if cfg!(windows) => backoff(&mut idle, deadline),
                Ok([]) => return Err(io::ErrorKind::UnexpectedEof.into()),
                Ok(buf) => {
                    idle = Duration::from_millis(5);
                    let end = buf.iter().position(|b| *b == b'\n');
                    let n = end.map(|i| i + 1).unwrap_or(buf.len());
                    if self.pending.len() + n > MAX_FRAME {
                        self.pending.clear();
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "IPC frame exceeds 4 MiB",
                        ));
                    }
                    self.pending.extend_from_slice(&buf[..n]);
                    self.reader.consume(n);
                    if end.is_some() {
                        return String::from_utf8(std::mem::take(&mut self.pending))
                            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e));
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) =>
                {
                    if cfg!(windows) {
                        backoff(&mut idle, deadline);
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }
}

/// Sleep with exponential backoff (5 ms to 250 ms), never past the deadline.
fn backoff(idle: &mut Duration, deadline: Instant) {
    std::thread::sleep((*idle).min(deadline.saturating_duration_since(Instant::now())));
    *idle = (*idle * 2).min(Duration::from_millis(250));
}

#[cfg(test)]
mod tests {
    use super::*;
    use interprocess::local_socket::ListenerOptions;

    #[test]
    fn fragmented_frame_and_idle_deadline() {
        let dir = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        let path = dir.path().join("glance.sock");
        #[cfg(windows)]
        let path = std::path::PathBuf::from(format!(
            r"\\.\pipe\glance-test-{}-{}",
            std::process::id(),
            dir.path().file_name().unwrap().to_string_lossy()
        ));
        let listener = ListenerOptions::new()
            .name(path.as_path().to_fs_name::<GenericFilePath>().unwrap())
            .create_sync()
            .unwrap();
        let server = std::thread::spawn(move || {
            let mut stream = listener.accept().unwrap();
            stream.write_all(b"{\"ok\":").unwrap();
            std::thread::sleep(Duration::from_millis(30));
            stream.write_all(b"true}\n").unwrap();
            std::thread::sleep(Duration::from_millis(200));
            stream.write_all(b"{\"split\":").unwrap();
            std::thread::sleep(Duration::from_millis(400));
            stream.write_all(b"1}\n").unwrap();
            std::thread::sleep(Duration::from_millis(100));
        });
        let mut client = Connection::connect(&path).unwrap();
        assert_eq!(
            client.line(Duration::from_secs(2)).unwrap(),
            "{\"ok\":true}\n"
        );
        assert_eq!(
            client.line(Duration::from_millis(50)).unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
        // A frame split across a timeout is completed by the next read, not dropped.
        let timed_out = client.line(Duration::from_millis(300)).unwrap_err();
        assert_eq!(timed_out.kind(), io::ErrorKind::TimedOut);
        assert_eq!(
            client.line(Duration::from_secs(2)).unwrap(),
            "{\"split\":1}\n"
        );
        server.join().unwrap();
    }
}
