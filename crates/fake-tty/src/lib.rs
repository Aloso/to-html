//! Run a command in bash, pretending to be a tty.
//!
//! This means that the command will assume that terminal colors and
//! other terminal features are available.
//!
//! ## Example
//!
//! ```
//! #[cfg(not(target_os = "windows"))]
//! let cmd = fake_tty::bash_command("ls");
//! #[cfg(target_os = "windows")]
//! let cmd = fake_tty::command("ls", None);
//! let output = cmd.unwrap()
//!     .output().unwrap();
//! assert!(output.status.success());
//!
//! let _stdout: String = fake_tty::get_stdout(output.stdout).unwrap();
//! ```

use std::{
    io,
    process::{Command, Stdio},
    string::FromUtf8Error,
};

/// Creates a command that is executed by bash, pretending to be a tty.
///
/// This means that the command will assume that terminal colors and
/// other terminal features are available.
pub fn bash_command(command: &str) -> io::Result<Command> {
    let mut command = make_script_command(command, Some("bash"))?;
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    Ok(command)
}

/// Creates a command that is executed by a shell, pretending to be a tty.
///
/// This means that the command will assume that terminal colors and
/// other terminal features are available.
pub fn command(command: &str, shell: Option<&str>) -> io::Result<Command> {
    let mut command = make_script_command(command, shell)?;
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    Ok(command)
}

/// Wraps the command in the `script` command that can execute it
/// pretending to be a tty.
///
/// - [Linux docs](https://man7.org/linux/man-pages/man1/script.1.html)
/// - [FreeBSD docs](https://www.freebsd.org/cgi/man.cgi?query=script&sektion=0&manpath=FreeBSD+12.2-RELEASE+and+Ports&arch=default&format=html)
/// - [Apple docs](https://opensource.apple.com/source/shell_cmds/shell_cmds-170/script/script.1.auto.html)
///
/// ## Examples
///
/// ```
/// use std::process::{Command, Stdio};
/// use fake_tty::make_script_command;
///
/// #[cfg(not(target_os = "windows"))]
/// let shell = Some("bash");
/// #[cfg(target_os = "windows")]
/// let shell = None;
/// let output = make_script_command("ls", shell).unwrap()
///     .stdout(Stdio::piped())
///     .stderr(Stdio::piped())
///     .output().unwrap();
///
/// assert!(output.status.success());
/// ```
pub fn make_script_command(c: &str, shell: Option<&str>) -> io::Result<Command> {
    let shell = {
        #[cfg(not(target_os = "windows"))]
        let shell_default = shell.unwrap_or("bash");
        #[cfg(target_os = "windows")]
        let shell_default = shell.unwrap_or("pwsh");

        which_shell(shell_default)?
    };

    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        let mut command = Command::new("script");
        command.args(["-qec", c, "/dev/null"]);
        command.env("SHELL", shell.trim());

        Ok(command)
    }

    #[cfg(any(target_os = "macos", target_os = "freebsd"))]
    {
        let mut command = Command::new("script");
        command.args(&["-q", "/dev/null", shell.trim(), "-c", c]);
        Ok(command)
    }

    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new(shell.trim());
        command.args(&["-Command", c]);
        Ok(command)
    }

    #[cfg(not(any(
        target_os = "android",
        target_os = "linux",
        target_os = "macos",
        target_os = "freebsd",
        target_os = "windows"
    )))]
    compile_error!("This platform is not supported. See https://github.com/Aloso/to-html/issues/3")
}

/// Returns the standard output of the command.
pub fn get_stdout(stdout: Vec<u8>) -> Result<String, FromUtf8Error> {
    let out = String::from_utf8(stdout)?;

    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        Ok(out.replace("\r\n", "\n"))
    }

    #[cfg(any(target_os = "macos", target_os = "freebsd"))]
    {
        let mut out = out.replace("\r\n", "\n");
        if out.starts_with("^D\u{8}\u{8}") {
            out = out["^D\u{8}\u{8}".len()..].to_string()
        }
        Ok(out)
    }

    #[cfg(target_os = "windows")]
    {
        Ok(out)
    }
}

fn which_shell(shell: &str) -> io::Result<String> {
    #[cfg(not(target_os = "windows"))]
    {
        let which = Command::new("which")
            .arg(shell)
            .stdout(Stdio::piped())
            .output()?;

        if which.status.success() {
            Ok(String::from_utf8(which.stdout).unwrap())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8(which.stderr).unwrap(),
            ))
        }
    }

    #[cfg(target_os = "windows")]
    {
        let get_command = Command::new("pwsh")
            .args(&["-Command", &format!("(Get-Command {shell}).Path")])
            .stdout(Stdio::piped())
            .output()?;

        // pwsh returns 0 when the subcommand fails, so we can't just
        // use `get_command.status.succes()`
        let output = String::from_utf8(get_command.stdout).unwrap();
        if !output.trim().is_empty() {
            Ok(output)
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8(get_command.stderr).unwrap(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    fn run(s: &str) -> String {
        #[cfg(not(target_os = "windows"))]
        let output = crate::bash_command(s).unwrap().output().unwrap();
        #[cfg(target_os = "windows")]
        let output = crate::command(s, None).unwrap().output().unwrap();
        let s1 = crate::get_stdout(output.stdout).unwrap();

        if crate::which_shell("zsh").is_ok() {
            let output = crate::command(s, Some("zsh")).unwrap().output().unwrap();
            let s2 = crate::get_stdout(output.stdout).unwrap();

            assert_eq!(s1, s2);
        }

        s1
    }

    #[cfg(not(target_os = "windows"))]
    mod not_windows {
        use super::run;

        #[test]
        fn echo() {
            assert_eq!(run("echo hello world"), "hello world\n");
        }

        #[test]
        fn seq() {
            assert_eq!(run("seq 3"), "1\n2\n3\n");
        }

        #[test]
        fn echo_quotes() {
            assert_eq!(run(r#"echo "Hello \$\`' world!""#), "Hello $`' world!\n");
        }

        #[test]
        fn echo_and_cat() {
            assert_eq!(
                run("echo 'look, bash support!' | cat"),
                "look, bash support!\n"
            );
        }
    }

    #[cfg(target_os = "windows")]
    mod windows {
        use super::run;

        #[test]
        fn echo() {
            assert_eq!(run("echo hello world"), "hello\r\nworld\r\n");
        }

        #[test]
        fn seq() {
            assert_eq!(run("1..3"), "1\r\n2\r\n3\r\n");
        }

        #[test]
        fn echo_quotes() {
            // In powershell, backtick is used to escape the next character.
            assert_eq!(run(r#"echo "Hello `$``' world!""#), "Hello $`' world!\r\n");
        }

        #[test]
        fn echo_and_pipe() {
            assert_eq!(
                run("echo 'look, pipe support!' | % { Write-Host $_ }"),
                "look, pipe support!\r\n"
            );
        }
    }
}
