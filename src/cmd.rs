use std::{
    io::{self, Write},
    path::Path,
    process::{Child, ExitStatus},
};

use crate::StdError;

pub fn run(args: &str, shell: Option<&str>) -> Result<(String, String, ExitStatus), StdError> {
    let output =
        fake_tty::command(&format!("{args}; printf \"~~////~~\"; pwd"), shell)?.output()?;

    let stdout = fake_tty::get_stdout(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    let status = output.status;

    let stdout = stdout.trim_end();
    let lb = stdout
        .rfind("~~////~~")
        .ok_or_else(|| format!("Delimiter not found in the string {stdout:?}"))
        .unwrap();

    let (output, cwd) = stdout.split_at(lb);
    let cwd = cwd.trim_start_matches("~~////~~").trim_start();

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
        "Running `ls -l` was unsuccessful (stdout: {stdout:?}, stderr: {stderr:?})"
    );
}
