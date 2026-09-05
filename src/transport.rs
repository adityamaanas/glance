//! Newline-delimited local IPC: Unix sockets and Windows named pipes.
use interprocess::local_socket::{prelude::*, GenericFilePath, Stream};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::time::{Duration, Instant};

pub struct Connection {
    reader: BufReader<Stream>,
}

impl Connection {
    pub fn connect(path: &Path) -> io::Result<Self> {
        let stream = Stream::connect(path.to_fs_name::<GenericFilePath>()?)?;
        stream.set_nonblocking(true)?;
        Ok(Self {
            reader: BufReader::new(stream),
        })
    }

    pub fn send(&mut self, bytes: &[u8]) -> io::Result<()> {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut pending = bytes;
        while !pending.is_empty() {
            match self.reader.get_mut().write(pending) {
                Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
                Ok(n) => pending = &pending[n..],
                Err(e) if e.kind() == io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    pub fn line(&mut self, timeout: Duration) -> io::Result<String> {
        let deadline = Instant::now() + timeout;
        let mut bytes = Vec::new();
        loop {
            if Instant::now() >= deadline {
                return Err(io::ErrorKind::TimedOut.into());
            }
            match self.reader.fill_buf() {
                // Byte-mode Windows pipes in NOWAIT mode return zero when temporarily empty.
                Ok([]) if cfg!(windows) => std::thread::sleep(Duration::from_millis(10)),
                Ok([]) => return Err(io::ErrorKind::UnexpectedEof.into()),
                Ok(buf) => {
                    let end = buf.iter().position(|b| *b == b'\n');
                    let n = end.map(|i| i + 1).unwrap_or(buf.len());
                    if bytes.len() + n > 4 * 1024 * 1024 {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "IPC frame exceeds 4 MiB",
                        ));
                    }
                    bytes.extend_from_slice(&buf[..n]);
                    self.reader.consume(n);
                    if end.is_some() {
                        return String::from_utf8(bytes)
                            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e));
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                Err(e) => return Err(e),
            }
        }
    }
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
        server.join().unwrap();
    }
}
