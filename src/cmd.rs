use std::{error, io, io::Write, path::Path, process};

pub fn run(args: &str) -> Result<(String, String, process::ExitStatus), Box<dyn error::Error>> {
    let output = fake_tty::bash_command(&format!("{}; echo \"\n$PWD\"", args)).output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    let status = output.status;

    let stdout = stdout.trim_end();
    let lb = stdout.rfind(|c| matches!(c, '\n' | '\r')).unwrap();
    let (mut output, cwd) = stdout.split_at(lb);
    let cwd = cwd.trim_start();
    if !cmp_paths(std::env::current_dir()?, cwd) {
        std::env::set_current_dir(cwd)?;
    }
    if output.ends_with('\r') {
        output = &output[..output.len() - 1];
    }
    Ok((output.to_string(), stderr, status))
}

fn cmp_paths(p1: impl AsRef<Path>, p2: impl AsRef<Path>) -> bool {
    p1.as_ref() == p2.as_ref()
}

pub fn input(child: process::Child, input: impl AsRef<str>) -> io::Result<process::Child> {
    child
        .stdin
        .as_ref()
        .unwrap()
        .write_all(input.as_ref().as_bytes())?;
    Ok(child)
}
