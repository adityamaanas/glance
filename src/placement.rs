//! Launch a panel with argument vectors in supported terminal multiplexers.
use anyhow::{bail, Result};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, clap::ValueEnum)]
pub enum Backend {
    Auto,
    Herdr,
    Tmux,
    Zellij,
}

pub fn resolve(backend: Backend) -> Result<Backend> {
    if backend != Backend::Auto {
        return Ok(backend);
    }
    if std::env::var_os("HERDR_PANE_ID").is_some() {
        return Ok(Backend::Herdr);
    }
    if std::env::var_os("TMUX").is_some() {
        return Ok(Backend::Tmux);
    }
    if std::env::var_os("ZELLIJ").is_some() {
        return Ok(Backend::Zellij);
    }
    bail!("no supported multiplexer detected; open a split and run glance-panel --cwd .")
}

fn command(
    backend: Backend,
    exe: &Path,
    cwd: &Path,
    session: &str,
    ratio: f64,
    no_model: bool,
) -> Result<Command> {
    crate::transcript::validate_session_id(session)?;
    if !ratio.is_finite() || ratio <= 0.0 || ratio >= 1.0 {
        bail!("ratio must be greater than 0 and less than 1");
    }
    let mut cmd = match backend {
        Backend::Tmux => {
            let mut cmd = Command::new("tmux");
            cmd.args([
                "split-window",
                "-h",
                "-l",
                &format!("{}%", (ratio * 100.0).round().clamp(1.0, 99.0)),
            ])
            .arg("-c")
            .arg(cwd);
            for key in ["GLANCE_HOME", "CLAUDE_CONFIG_DIR", "GLANCE_MODEL"] {
                if let Some(value) = std::env::var_os(key) {
                    let mut pair = std::ffi::OsString::from(format!("{key}="));
                    pair.push(value);
                    cmd.arg("-e").arg(pair);
                }
            }
            cmd.arg("--");
            cmd
        }
        Backend::Zellij => {
            let mut cmd = Command::new("zellij");
            cmd.args([
                "action",
                "new-pane",
                "--direction",
                "right",
                "--name",
                "glance",
                "--cwd",
            ])
            .arg(cwd)
            .arg("--");
            cmd.arg("env");
            for key in ["GLANCE_HOME", "CLAUDE_CONFIG_DIR", "GLANCE_MODEL"] {
                if let Some(value) = std::env::var_os(key) {
                    let mut pair = std::ffi::OsString::from(format!("{key}="));
                    pair.push(value);
                    cmd.arg(pair);
                }
            }
            cmd
        }
        _ => bail!("unsupported split backend"),
    };
    cmd.arg(exe).args(["--session", session]);
    if no_model {
        cmd.arg("--no-model");
    }
    Ok(cmd)
}

pub fn attach(
    backend: Backend,
    session: Option<String>,
    cwd: Option<&Path>,
    ratio: f64,
    no_model: bool,
) -> Result<()> {
    let cwd = cwd
        .map(Path::to_path_buf)
        .unwrap_or(std::env::current_dir()?);
    let session = match session {
        Some(id) => id,
        None => crate::discovery::pick(Some(&cwd), None, true)?,
    };
    let mut cmd = command(
        backend,
        &std::env::current_exe()?,
        &cwd,
        &session,
        ratio,
        no_model,
    )?;
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    crate::summary::run_process(&mut cmd, vec![], std::time::Duration::from_secs(15))?;
    if backend == Backend::Zellij {
        println!("glance: panel opened; Zellij controls the tiled split size");
    }
    Ok(())
}

pub fn shell_command(exe: &str, pane: &str, shell: &str) -> Result<String> {
    if exe.chars().chain(pane.chars()).any(char::is_control) {
        bail!("invalid control character in pane command");
    }
    match shell
        .trim_start_matches('-')
        .trim_end_matches(".exe")
        .to_lowercase()
        .as_str()
    {
        "pwsh" | "powershell" => Ok(format!(
            "& '{}' --pane '{}'",
            exe.replace('\'', "''"),
            pane.replace('\'', "''")
        )),
        "cmd" => {
            if exe.contains(['%', '!', '"']) || pane.contains(['%', '!', '"', '&', '|', '<', '>']) {
                bail!("this path requires PowerShell or a POSIX shell for automatic attachment");
            }
            Ok(format!("\"{exe}\" --pane \"{pane}\""))
        }
        _ => Ok(format!(
            "{} --pane {}",
            shell_words::quote(exe),
            shell_words::quote(pane)
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn split_preserves_arguments_and_shell_quoting_matches_the_shell() {
        for backend in [Backend::Tmux, Backend::Zellij] {
            let cmd = command(
                backend,
                Path::new("/a path/it's/glance-panel"),
                Path::new("/project space"),
                "session",
                0.3,
                true,
            )
            .unwrap();
            let args: Vec<_> = cmd
                .get_args()
                .map(|s| s.to_string_lossy().to_string())
                .collect();
            assert!(args.contains(&"/a path/it's/glance-panel".into()));
            assert!(args.contains(&"/project space".into()));
            assert_eq!(args.last().unwrap(), "--no-model");
        }
        assert_eq!(
            shell_command("C:\\it's here\\glance-panel.exe", "w1:p1", "pwsh.exe").unwrap(),
            "& 'C:\\it''s here\\glance-panel.exe' --pane 'w1:p1'"
        );
        assert!(shell_command("C:\\%user%\\glance-panel.exe", "w1:p1", "cmd.exe").is_err());
    }
}
