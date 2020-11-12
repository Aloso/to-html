//! Run a command in bash, pretending to be a tty.
//!
//! This means that the command will assume that terminal colors and
//! other terminal features are available.
//!
//! ## Example
//!
//! ```
//! let output = faketty::bash_command("ls --color=auto").output().unwrap();
//! assert!(output.status.success());
//!
//! let _stdout: String = String::from_utf8(output.stdout.to_vec())
//!     .expect("Invalid UTF-8");
//! ```

use std::{iter, process};

/// Creates a command that is executed by bash, pretending to be a tty.
///
/// This means that the command will assume that terminal colors and
/// other terminal features are available.
pub fn bash_command(command: &str) -> process::Command {
    let escaped = escape_bash_string(command);

    #[cfg(target_os = "linux")]
    let script_command = format!("script -qec \"{}\" /dev/null", escaped);

    #[cfg(target_os = "macos")]
    let script_command = format!("script -q /dev/null \"{}\"", escaped);

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    panic!("This platform is not supported");

    let mut command = process::Command::new("bash");
    command
        .args(&["-c", &script_command])
        .stdout(process::Stdio::piped());

    command
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

#[test]
fn test_escaping() {
    assert_eq!(
        escape_bash_string(r#"Hello $`"' world!"#).as_str(),
        r#"Hello \$\`\"' world!"#
    );
}
