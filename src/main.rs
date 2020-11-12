use clap::{App, AppSettings, Arg, ArgMatches};
use std::{
    error::Error,
    process::{Command, Stdio},
};

mod cmd;
mod html;

fn clap_app<'a, 'b>() -> App<'a, 'b> {
    App::new("to-html")
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
}

struct Args<'a> {
    commands: Vec<&'a str>,
    highlight: Vec<&'a str>,
    prefix: String,
}

fn parse_args<'a>(matches: &'a ArgMatches) -> Result<Args<'a>, Box<dyn Error>> {
    let highlight = matches
        .values_of("highlight")
        .map(Iterator::collect)
        .unwrap_or_default();

    let prefix = matches
        .value_of("prefix")
        .map(|s| s.to_string() + "-")
        .unwrap_or_default();

    let commands = matches
        .values_of("command")
        .ok_or("command missing")?
        .collect();

    Ok(Args {
        commands,
        highlight,
        prefix,
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap_app().get_matches();
    let Args {
        commands,
        highlight,
        prefix,
    } = parse_args(&matches)?;

    let commands: Vec<&[&str]> = commands.split(|&s| s == "--").collect();

    let mut result = format!("<pre class=\"{}terminal\">\n", prefix);

    for command_parts in commands {
        result.push_str(&command_to_html(command_parts, &highlight, &prefix)?);
    }

    result.push_str(&format!(
        "<span class=\"{p}arrow\">&gt;</span> <span class=\"{p}caret\"> </span>",
        p = prefix
    ));
    result.push_str("\n</pre>");

    println!("{}", result);

    Ok(())
}

fn command_to_html(
    command_parts: &[&str],
    highlight: &[&str],
    prefix: &str,
) -> Result<String, Box<dyn Error>> {
    let mut result = format!("<span class=\"{}arrow\">&gt;</span> ", prefix);

    let command = cmd::concat(command_parts);

    let (cmd_out, cmd_err) = cmd::run(&command)?;
    let stdout = run_ansi_to_html(&html::dimmed_to_html(&cmd_out, prefix))?;

    result.push_str(&command_line_to_html(command_parts, &highlight, prefix));
    result.push('\n');
    if !cmd_err.is_empty() {
        result.push_str(&cmd_err);
        result.push('\n');
    }
    result.push_str(&stdout);
    Ok(result)
}

fn command_line_to_html(command_parts: &[&str], highlight: &[&str], prefix: &str) -> String {
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
        .map(move |part| {
            let part_esc = &*html::esc_html(part);

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
                format!("<span class=\"{}op\">{}</span>\n ", prefix, part_esc)
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

fn run_ansi_to_html(input: &str) -> Result<String, Box<dyn Error>> {
    let process = Command::new("ansi-to-html")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let output = cmd::input(process, input)?.wait_with_output()?;

    Ok(cmd::stdout(&output)?)
}
