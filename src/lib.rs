use std::{
    error::Error,
    process::{Command, Stdio},
};

pub mod cmd;
pub mod html;

pub fn to_html(ansi_string: &str, css_prefix: &str) -> Result<String, Box<dyn Error>> {
    let input = html::Esc(ansi_string).to_string();
    let stdout = run_ansi_to_html(&html::dimmed_to_html(&input, css_prefix))?;
    Ok(stdout)
}

fn run_ansi_to_html(input: &str) -> Result<String, Box<dyn Error>> {
    let process = Command::new("ansi-to-html")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let output = cmd::input(process, input)?.wait_with_output()?;

    Ok(cmd::stdout(&output)?)
}
