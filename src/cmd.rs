use std::{
    io::{self, Write},
    path::Path,
    process::{Child, ExitStatus},
};

use crate::StdError;

pub fn run(args: &str, shell: Option<&str>) -> Result<(String, String, ExitStatus), StdError> {
    let is_powershell = shell
        .map(str::to_lowercase)
        .map(|shell_lowercase| {
            shell_lowercase.contains("pwsh") || shell_lowercase.contains("powershell")
        })
        .unwrap_or(false);
    let cmd_and_pwd = if is_powershell {
        format!("{}; echo \"~~////~~\"; (pwd).Path", args)
    } else {
        format!("{}; echo \"~~////~~\"; pwd", args)
    };

    let output = fake_tty::command(&cmd_and_pwd, shell)?.output()?;

    let stdout = fake_tty::get_stdout(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    let status = output.status;

    let stdout = stdout.trim_end();
    let lb = stdout
        .rfind("~~////~~")
        .ok_or_else(|| format!("Delimiter not found in the string {:?}", stdout))
        .unwrap();

    let (output, cwd) = stdout.split_at(lb);
    let cwd = cwd.trim_start_matches("~~////~~").trim_start();

    // On Windows, `std::env::set_current_dir` needs to be in the form:
    //
    // ```text
    // C:\Some\Path
    // ```
    //
    // `pwd` outputs on different shells:
    //
    // * `git-bash`: `/c/Some/Path`
    // * WSL `bash`: `/mnt/c/Some/Path`
    #[cfg(target_os = "windows")]
    let cwd = &cwd
        .trim_start_matches("/mnt")
        .trim_start_matches('/') // Remove the leading slash
        .replacen('/', ":\\", 1) // Add a `:` for the drive letter.
        .replace('/', "\\"); // Use Windows directory separator.

    if !cmp_paths(std::env::current_dir()?, cwd) {
        std::env::set_current_dir(cwd)?;
    }
    Ok((output.to_string(), stderr, status))
}

fn cmp_paths(p1: impl AsRef<Path>, p2: impl AsRef<Path>) -> bool {
    p1.as_ref() == p2.as_ref()
}

pub fn input(mut child: Child, input: impl AsRef<str>) -> io::Result<Child> {
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_ref().as_bytes())?;
    Ok(child)
}

#[test]
fn test_run() {
    let (stdout, stderr, status) = run("ls -l", None).unwrap();
    assert!(
        status.success(),
        "Running `ls -l` was unsuccessful (stdout: {:?}, stderr: {:?})",
        stdout,
        stderr
    );
}

#[cfg(target_os = "windows")]
#[test]
fn test_run_pwsh() {
    let (stdout, stderr, status) = run("ls -Verbose", Some("pwsh")).unwrap();
    assert!(
        status.success(),
        "Running `ls -Verbose` was unsuccessful (stdout: {:?}, stderr: {:?})",
        stdout,
        stderr
    );
}
