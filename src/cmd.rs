use std::{borrow::Cow, error, io, io::Write, process, string};

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
    let output = faketty::bash_command(args).spawn()?.wait_with_output()?;

    Ok((stdout(&output)?, stderr(&output)?))
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
