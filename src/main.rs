use clap::{App, AppSettings, Arg};
use std::{
    error::Error,
    io::{Read, Write},
    iter,
    process::{Command, Stdio},
};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("colo-utils")
        .about(
            "Utility for generating HTML documentation for colo\n\
            This requires the `ansi-to-html` NPM package",
        )
        .version("0.1")
        .author("Ludwig Stecher <ludwig.stecher@gmx.de>")
        .global_setting(AppSettings::ColoredHelp)
        .arg(
            Arg::with_name("command")
                .index(1)
                .multiple(true)
                .help("The colo command")
                .required(true),
        )
        .arg(
            Arg::with_name("cargo")
                .long("cargo")
                .short("c")
                .help("Use `cargo run` instead of `colo`"),
        )
        .get_matches();

    let cargo_flag = matches.is_present("cargo");

    let commands: Vec<&str> = matches
        .values_of("command")
        .ok_or("command missing")?
        .collect();
    let commands: Vec<&[&str]> = commands.split(|&s| s == "--").collect();

    let mut result = String::from(
        "<pre class=\"terminal-text\">\n\
        <span class=\"terminal-arrow\">&gt;</span> <span class=\"terminal-command\">colo</span> ",
    );

    for command_args in commands {
        let colo_out = run_colo(command_args, cargo_flag)?;
        let ansi_html_out = run_ansi_to_html(&dimmed_to_html(&colo_out))?;

        result.push_str(&command_to_html(&concat_commands(command_args)));
        result.push('\n');
        result.push_str(&ansi_html_out);
    }

    result.push_str(
        "<span class=\"terminal-arrow\">&gt;</span> <span style=\"background-color: white\"> </span>",
    );
    result.push_str("\n</pre>");

    println!("{}", result);

    Ok(())
}

fn concat_commands(input: &[&str]) -> String {
    let mut result: String = input
        .iter()
        .map(|&s| -> String {
            if s.contains(|c: char| c.is_ascii_whitespace()) {
                iter::once('"')
                    .chain(s.chars())
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
    // let s = "~ $ colo show orange 'hsl(30, 100%, 50%)' --out hex";
    let mut result = String::new();

    for line in LinesWithEndings::from(&input) {
        let ranges: Vec<(Style, &str)> = h.highlight(line, &ps);
        let escaped =
            styled_line_to_highlighted_html(&ranges, syntect::html::IncludeBackground::No);
        result.push_str(&escaped);
    }
    result
}

fn run_colo(command_args: &[&str], cargo_flag: bool) -> Result<String, Box<dyn Error>> {
    let output = if cargo_flag {
        Command::new("cargo")
            .args(["run", "-q", "--"].iter().chain(command_args))
            .env("FORCE_ANSI_OUTPUT", "1")
            .output()
    } else {
        Command::new("colo")
            .args(command_args)
            .env("FORCE_ANSI_OUTPUT", "1")
            .output()
    }?;
    Ok(String::from_utf8(output.stdout)?)
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
