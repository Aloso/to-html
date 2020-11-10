use clap::{App, AppSettings, Arg};
use std::{
    error::Error,
    io::{Read, Write},
    iter,
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
        ])
        .get_matches();

    let no_env = matches.is_present("no-env");

    let commands: Vec<&str> = matches
        .values_of("command")
        .ok_or("command missing")?
        .collect();
    let commands: Vec<&[&str]> = commands.split(|&s| s == "--").collect();

    let mut result = String::from("<pre class=\"terminal-text\">\n");

    for command in commands {
        result.push_str("<span class=\"terminal-arrow\">&gt;</span> ");

        let printed_command = concat_command(command, no_env);
        let command = concat_command(command, false);

        let cmd_out = run_command(&command)?;
        let ansi_html_out = run_ansi_to_html(&dimmed_to_html(&cmd_out))?;

        result.push_str(&command_to_html(&printed_command));
        result.push('\n');
        result.push_str(&ansi_html_out);
    }

    result.push_str("<span class=\"terminal-arrow\">&gt;</span> <span class=\"caret\"> </span>");
    result.push_str("\n</pre>");

    println!("{}", result);

    Ok(())
}

fn concat_command(command: &[&str], no_env: bool) -> String {
    let mut result: String = command
        .iter()
        .copied()
        .filter(|&s| !no_env || !is_environment_variable(s))
        .map(|s| -> String {
            if s.contains(|c: char| c.is_ascii_whitespace() || matches!(c, '"' | '\'')) {
                iter::once('"')
                    .chain(s.chars().flat_map(|c| {
                        Some('\\')
                            .filter(|_| matches!(c, '"' | '\''))
                            .into_iter()
                            .chain(iter::once(c))
                    }))
                    .chain("\" ".chars())
                    .collect()
            } else {
                s.chars().chain(iter::once(' ')).collect()
            }
        })
        .collect();

    result.truncate(result.len() - 1);
    result
}

fn dimmed_to_html(input: &str) -> String {
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
                    result.push_str(r#"<span class="dimmed">"#);
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

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::html::styled_line_to_highlighted_html;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

fn command_to_html(input: &str) -> String {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ps.find_syntax_by_extension("sh").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    let mut result = String::new();

    for line in LinesWithEndings::from(&input) {
        let ranges: Vec<(Style, &str)> = h.highlight(line, &ps);
        let escaped =
            styled_line_to_highlighted_html(&ranges, syntect::html::IncludeBackground::No);
        result.push_str(&escaped);
    }
    result
        .replace("<span style=\"color:#c0c5ce;\">", "<span class=\"quote\">")
        .replace("<span style=\"color:#a3be8c;\">", "<span class=\"string\">")
        .replacen(
            "<span style=\"color:#8fa1b3;\">",
            "<span class=\"terminal-command\">",
            1,
        )
}

fn run_command(args: &str) -> Result<String, Box<dyn Error>> {
    let mut output = String::new();

    let process = Command::new("bash")
        // .args(&["-c", args])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut args = args.to_string();
    args.push_str("\nexit");

    process.stdin.unwrap().write_all(args.as_bytes())?;
    process.stdout.unwrap().read_to_string(&mut output)?;

    Ok(output)
}

fn run_ansi_to_html(input: &str) -> Result<String, Box<dyn Error>> {
    let mut output = String::new();

    let process = Command::new("ansi-to-html")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    process.stdin.unwrap().write_all(input.as_bytes())?;
    process.stdout.unwrap().read_to_string(&mut output)?;

    Ok(output)
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
