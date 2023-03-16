//! Run a command in bash, pretending to be a tty.
//!
//! This means that the command will assume that terminal colors and
//! other terminal features are available.
//!
//! ## Example
//!
//! ```
//! let output = fake_tty::bash_command("ls").unwrap()
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
/// let output = make_script_command("ls", Some("bash")).unwrap()
///     .stdout(Stdio::piped())
///     .stderr(Stdio::piped())
///     .output().unwrap();
///
/// assert!(output.status.success());
/// ```
pub fn make_script_command(c: &str, shell: Option<&str>) -> io::Result<Command> {
    let shell = which_shell(shell.unwrap_or("bash"))?;

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

    #[cfg(not(any(
        target_os = "android",
        target_os = "linux",
        target_os = "macos",
        target_os = "freebsd"
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
}

fn which_shell(shell: &str) -> io::Result<String> {
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

#[cfg(test)]
mod tests {
    fn run(s: &str) -> String {
        let output = crate::bash_command(s).unwrap().output().unwrap();
        let s1 = crate::get_stdout(output.stdout).unwrap();

        if crate::which_shell("zsh").is_ok() {
            let output = crate::command(s, Some("zsh")).unwrap().output().unwrap();
            let s2 = crate::get_stdout(output.stdout).unwrap();

            assert_eq!(s1, s2);
        }

        s1
    }

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
