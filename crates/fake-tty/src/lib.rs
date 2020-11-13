//! Run a command in bash, pretending to be a tty.
//!
//! This means that the command will assume that terminal colors and
//! other terminal features are available.
//!
//! ## Example
//!
//! ```
//! let output = fake_tty::bash_command("ls --color=auto").output().unwrap();
//! assert!(output.status.success());
//!
//! let _stdout: String = String::from_utf8(output.stdout).unwrap();
//! ```

use std::{iter, process};

/// Creates a command that is executed by bash, pretending to be a tty.
///
/// This means that the command will assume that terminal colors and
/// other terminal features are available.
pub fn bash_command(command: &str) -> process::Command {
    let script_command = make_script_command(&command);

    let mut command = process::Command::new("bash");
    command
        .args(&["-c", &script_command])
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped());

    command
}

/// Wraps the command in the `script` command that can execute it
/// pretending to be a tty.
///
/// This can be executed in two ways:
///  - by passing it to `bash -c <command>`
///    (you can use the `bash_command` function for this) or
///  - by running `bash` and piping the command to stdin,
///    followed with the command `exit`
///
/// ## Examples
///
/// Pass it to `bash -c <command>`:
///
/// ```
/// use std::process::{Command, Stdio};
/// use fake_tty::make_script_command;
///
/// let script_command = make_script_command("ls");
///
/// let output = Command::new("bash")
///     .args(&["-c", &script_command])
///     .stdout(Stdio::piped())
///     .stderr(Stdio::piped())
///     .output().unwrap();
///
/// assert!(output.status.success());
/// ```
///
/// Pipe it to `bash`:
///
/// ```
/// use std::process::{Command, Stdio};
/// use std::io::Write;
/// use fake_tty::make_script_command;
///
/// let mut script_command = make_script_command("ls");
/// script_command.push_str("\nexit");
///
/// let process = Command::new("bash")
///     .stdin(Stdio::piped())
///     .stdout(Stdio::piped())
///     .stderr(Stdio::piped())
///     .spawn()
///     .unwrap();
///
/// let mut stdin = process.stdin.as_ref().unwrap();
/// stdin.write_all(script_command.as_bytes()).unwrap();
/// let output = process.wait_with_output().unwrap();
///
/// assert!(output.status.success());
/// ```
pub fn make_script_command(command: &str) -> String {
    let escaped = escape_bash_string(command);

    #[cfg(target_os = "linux")]
    let script_command = format!("script -qec \"{}\" /dev/null", escaped);

    #[cfg(target_os = "macos")]
    let script_command = format!("script -q /dev/null \"{}\"", escaped);

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    panic!("This platform is not supported");

    script_command
}

fn escape_bash_string(s: &str) -> String {
    s.chars().flat_map(escape_bash_string_char).collect()
}

fn escape_bash_string_char(c: char) -> impl Iterator<Item = char> {
    Some('\\')
        .filter(|_| matches!(c, '"' | '$' | '`' | '\\'))
        .into_iter()
        .chain(iter::once(c))
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::process;

    use crate::{bash_command, escape_bash_string, make_script_command};

    #[test]
    fn test_escaping() {
        assert_eq!(
            escape_bash_string(r#"Hello $`"' world!"#).as_str(),
            r#"Hello \$\`\"' world!"#
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_script_wrapping() {
        assert_eq!(
            make_script_command(r#"echo "Hello $`' world!""#).as_str(),
            r#"script -qec "echo \"Hello \$\`' world!\"" /dev/null"#
        );
    }

    #[test]
    fn test_echo() {
        let output = bash_command(r#"echo "Hello \$\`' world!""#)
            .output()
            .unwrap();
        assert_eq!(to_string(output.stdout).trim_end(), r#"Hello $`' world!"#);
    }

    #[test]
    fn test_echo_custom() {
        let process = process::Command::new("bash")
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .spawn()
            .unwrap();

        let mut stdin = process.stdin.as_ref().unwrap();
        stdin.write_all(br#"echo "Hello \$\`' world!""#).unwrap();
        let output = process.wait_with_output().unwrap();

        assert_eq!(to_string(output.stdout).trim_end(), r#"Hello $`' world!"#);
        assert_eq!(to_string(output.stderr), "");
    }

    fn to_string(v: Vec<u8>) -> String {
        String::from_utf8(v).unwrap()
    }
}
