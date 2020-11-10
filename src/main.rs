use clap::{App, AppSettings, Arg};
use std::{
    borrow::Cow,
    error::Error,
    io::{Read, Write},
    process::{Command, Stdio},
};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("to-html")
        .about(
            "Terminal wrapper that generates HTML from ANSI escape sequences\n\
            This requires that `bash` and `ansi-to-html` are installed.",
        )
        .version("0.1")
        .author("Ludwig Stecher <ludwig.stecher@gmx.de>")
        .global_setting(AppSettings::ColoredHelp)
        .args(&[
            Arg::with_name("command")
                .index(1)
                .multiple(true)
                .help("The command(s) to execute. Multiple commands are separated with '--'")
                .required(true),
            Arg::with_name("no-env")
                .long("no-env")
                .short("e")
                .help("Don't print environment variables passed as arguments"),
            Arg::with_name("highlight")
                .long("highlight")
                .short("l")
                .help(
                    "Arguments and subcommands that should be highlighted differently. \
                    Multiple arguments are delimited with a comma.",
                )
                .multiple(true)
                .require_delimiter(true),
            Arg::with_name("prefix")
                .long("prefix")
                .short("p")
                .takes_value(true)
                .help(
                    "Prefix for CSS classes. For example, with the 'to-html' prefix, \
                    the 'arg' class becomes 'to-html-arg'",
                ),
        ])
        .get_matches();

    let no_env = matches.is_present("no-env");
    let highlight: Vec<&str> = matches
        .values_of("highlight")
        .map(Iterator::collect)
        .unwrap_or_default();

    let prefix = matches.value_of("prefix").map(|s| s.to_string() + "-");
    let prefix = prefix.as_deref().unwrap_or_default();

    let commands: Vec<&str> = matches
        .values_of("command")
        .ok_or("command missing")?
        .collect();
    let commands: Vec<&[&str]> = commands.split(|&s| s == "--").collect();

    let mut result = String::from("<pre class=\"terminal-text\">\n");

    for command_parts in commands {
        result.push_str("<span class=\"terminal-arrow\">&gt;</span> ");

        let command = concat_command(command_parts);

        let cmd_out = run_command(&command)?;
        let (stdout, stderr) = run_ansi_to_html(&dimmed_to_html(&cmd_out, prefix))?;

        result.push_str(&command_to_html(command_parts, no_env, &highlight, prefix));
        result.push('\n');
        result.push_str(&stdout);
        if let Some(stderr) = stderr {
            result.push_str(&stderr);
        }
    }

    result.push_str("<span class=\"terminal-arrow\">&gt;</span> <span class=\"caret\"> </span>");
    result.push_str("\n</pre>");

    println!("{}", result);

    Ok(())
}

fn concat_command(command: &[&str]) -> String {
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

fn dimmed_to_html(input: &str, prefix: &str) -> String {
    let mut pos = CharPos::None;
    let mut open_tags = 0;
    let mut result = String::new();

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    enum CharPos {
        None,
        Esc,
        Bracket,
        Two,
        Zero,
    }

    for c in input.chars() {
        match pos {
            CharPos::None if c == '\x1b' => {
                pos = CharPos::Esc;
            }
            CharPos::Esc => {
                if c == '[' {
                    pos = CharPos::Bracket;
                } else {
                    result.push('\x1b');
                    result.push(c);
                    pos = CharPos::None;
                }
            }
            CharPos::Bracket => {
                if c == '2' {
                    pos = CharPos::Two;
                } else if c == '0' {
                    pos = CharPos::Zero;
                } else {
                    result.push_str("\x1b[");
                    result.push(c);
                    pos = CharPos::None;
                }
            }
            CharPos::Two => {
                if c == 'm' {
                    pos = CharPos::None;
                    result.push_str(&format!(r#"<span class="{}dim">"#, prefix));
                    open_tags += 1;
                } else {
                    result.push_str("\x1b[2");
                    result.push(c);
                    pos = CharPos::None;
                }
            }
            CharPos::Zero => {
                if open_tags > 0 && c == 'm' {
                    pos = CharPos::None;
                    result.push_str("</span>");
                    open_tags -= 1;
                } else {
                    result.push_str("\x1b[0");
                    result.push(c);
                    pos = CharPos::None;
                }
            }
            _ => {
                result.push(c);
            }
        }
    }

    result
}

fn command_to_html(
    command_parts: &[&str],
    no_env: bool,
    highlight: &[&str],
    prefix: &str,
) -> String {
    let mut next = State::Start;

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    enum State {
        Default,
        Start,
        Pipe,
    }

    command_parts
        .iter()
        .copied()
        .filter(|&part| !no_env || next != State::Start || !is_environment_variable(part))
        .map(move |part| {
            let part_esc = &*esc_html(part);

            if part.contains(|c: char| c.is_ascii_whitespace()) {
                next = State::Default;
                format!(
                    "<span class=\"{}str\">\"{}\"</span>",
                    prefix,
                    part_esc.escape_debug(),
                )
            } else if next == State::Pipe {
                next = State::Default;
                format!("<span class=\"{}pipe\">{}</span>", prefix, part_esc)
            } else if next == State::Start {
                next = State::Default;
                format!("<span class=\"{}cmd\">{}</span>", prefix, part_esc)
            } else if part == "|" {
                next = State::Start;
                format!("<span class=\"{}pipe\">{}</span>", prefix, part_esc)
            } else if part == "<" || part == ">" {
                next = State::Pipe;
                format!("<span class=\"{}pipe\">{}</span>", prefix, part_esc)
            } else if part == "&&" {
                next = State::Start;
                format!("<span class=\"{}op\">{}</span>", prefix, part_esc)
            } else if part.starts_with('-') {
                if let Some((i, _)) = part_esc.char_indices().find(|&(_, c)| c == '=') {
                    let (p1, p2) = part_esc.split_at(i);
                    format!(
                        "<span class=\"{}flag\">{}</span><span class=\"{}arg\">{}</span>",
                        prefix, p1, prefix, p2,
                    )
                } else {
                    format!("<span class=\"{}flag\">{}</span>", prefix, part_esc)
                }
            } else if highlight.contains(&part) {
                format!("<span class=\"{}hl\">{}</span>", prefix, part_esc)
            } else {
                format!("<span class=\"{}arg\">{}</span>", prefix, part_esc)
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn run_command(args: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new("bash").args(&["-c", args]).output()?;
    let result = output.stdout.to_vec();

    Ok(String::from_utf8(result)?)
}

fn run_ansi_to_html(input: &str) -> Result<(String, Option<String>), Box<dyn Error>> {
    let (mut output1, mut output2) = (String::new(), String::new());

    let process = Command::new("ansi-to-html")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    process.stdin.unwrap().write_all(input.as_bytes())?;
    process.stdout.unwrap().read_to_string(&mut output1)?;

    if let Some(mut stderr) = process.stderr {
        stderr.read_to_string(&mut output2)?;
        Ok((output1, Some(output2)))
    } else {
        Ok((output1, None))
    }
}

fn is_environment_variable(s: &str) -> bool {
    if let Some((i, _)) = s.char_indices().find(|&(_, c)| c == '=') {
        let var_name = &s[..i];
        var_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    } else {
        false
    }
}

fn esc_html(input: &str) -> Cow<str> {
    if input.contains(|c: char| matches!(c, '&' | '<' | '>')) {
        Cow::Owned(input.chars().fold(String::new(), |mut acc, c| match c {
            '&' => acc + "&amp;",
            '<' => acc + "&lt;",
            '>' => acc + "&gt;",
            c => {
                acc.push(c);
                acc
            }
        }))
    } else {
        Cow::Borrowed(input)
    }
}
