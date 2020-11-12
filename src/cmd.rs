use std::{borrow::Cow, error, io, io::Write, path::Path, process, string};

pub fn concat(command: &[&str]) -> String {
    command
        .iter()
        .copied()
        .map(|s| -> Cow<str> {
            if s.contains(|c: char| c.is_ascii_whitespace() || matches!(c, '"' | '\'')) {
                let mut result = String::from("\"");
                for c in s.chars() {
                    if matches!(c, '"' | '\'') {
                        result.push('\\');
                    }
                    result.push(c);
                }
                result.push('"');
                result.push(' ');
                Cow::Owned(result)
            } else {
                Cow::Borrowed(s)
            }
        })
        .collect::<Vec<Cow<str>>>()
        .join(" ")
}

pub fn run(args: &str) -> Result<(String, String), Box<dyn error::Error>> {
    let output = faketty::bash_command(&format!("{}; echo \"\n$PWD\"", args))
        .spawn()?
        .wait_with_output()?;

    let stderr = stderr(&output)?;
    let stdout = stdout(&output)?;

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
    Ok((output.to_string(), stderr))
}

fn cmp_paths(p1: impl AsRef<Path>, p2: impl AsRef<Path>) -> bool {
    p1.as_ref() == p2.as_ref()
}

pub fn stdout(output: &process::Output) -> Result<String, string::FromUtf8Error> {
    String::from_utf8(output.stdout.to_vec())
}

pub fn stderr(output: &process::Output) -> Result<String, string::FromUtf8Error> {
    String::from_utf8(output.stderr.to_vec())
}

pub fn input(child: process::Child, input: impl AsRef<str>) -> io::Result<process::Child> {
    child
        .stdin
        .as_ref()
        .unwrap()
        .write_all(input.as_ref().as_bytes())?;
    Ok(child)
}
