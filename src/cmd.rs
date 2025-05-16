use std::{
    io::{self, Write},
    path::Path,
    process::{Child, ExitStatus},
};

use crate::StdError;

pub fn run(args: &str, shell: Option<&str>) -> Result<(String, String, ExitStatus), StdError> {
    let cwd_before = std::env::current_dir()?;

    let output = fake_tty::command(args, shell)?.output()?;

    let stdout = fake_tty::get_stdout(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    let status = output.status;

    let output = stdout.trim_end();

    if !cmp_paths(std::env::current_dir()?, &cwd_before) {
        std::env::set_current_dir(cwd_before)?;
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

#[cfg(not(target_os = "windows"))]
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
fn test_run() {
    let (stdout, stderr, status) = run("ls -Verbose", None).unwrap();
    assert!(
        status.success(),
        "Running `ls -Verbose` was unsuccessful (stdout: {:?}, stderr: {:?})",
        stdout,
        stderr
    );
}
