use std::io::Read;

use clap::{App, Arg, SubCommand};

fn main() {
    let matches = App::new("colo-utils")
        .about("Utilities for generating HTML documentation for colo")
        .version("0.1")
        .author("Ludwig Stecher <ludwig.stecher@gmx.de>")
        .subcommand(
            SubCommand::with_name("dim")
                .about("Replace ANSI sequences for dimming colors with HTML")
                .arg(Arg::with_name("text").index(1).help("Input text")),
        )
        .get_matches();

    match matches.subcommand() {
        ("dim", Some(matches)) => {
            let text = matches
                .value_of("text")
                .map(ToString::to_string)
                .unwrap_or_else(|| {
                    let mut text = Vec::new();
                    std::io::stdin().read_to_end(&mut text).unwrap();
                    let mut text = String::from_utf8(text).unwrap();
                    if text.ends_with('\n') {
                        text.truncate(text.len() - 1);
                    }
                    text
                });

            println!("{}", dimmed_to_html(&text));
        }
        _ => {
            unreachable!("No subcommand entered");
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum CharPos {
    None,
    Esc,
    Bracket,
    Two,
    Zero,
}

fn dimmed_to_html(input: &str) -> String {
    let mut pos = CharPos::None;
    let mut open_tags = 0;
    let mut result = String::new();

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
