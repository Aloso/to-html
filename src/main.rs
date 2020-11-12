use clap::{App, AppSettings, Arg, ArgMatches};
use std::{
    error::Error,
    fmt::Write,
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
            Arg::with_name("no-run")
                .long("no-run")
                .help("Don't run the commands, just emit the HTML for the command line"),
        ])
}

#[derive(Debug)]
struct Args<'a> {
    commands: Vec<&'a str>,
    highlight: Vec<&'a str>,
    prefix: String,
    no_run: bool,
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

    let no_run = matches.is_present("no-run");

    Ok(Args {
        commands,
        highlight,
        prefix,
        no_run,
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap_app().get_matches();
    let args = parse_args(&matches)?;

    let commands: Vec<&[&str]> = args.commands.split(|&s| s == "--").collect();

    let mut buf = String::new();
    writeln!(buf, "<pre class=\"{}terminal\">", args.prefix)?;

    for command_parts in commands {
        if args.no_run {
            command_prompt_to_html(&mut buf, command_parts, &args)?;
        } else {
            command_to_html(&mut buf, command_parts, &args)?;
        }
    }

    if !args.no_run {
        writeln!(
            buf,
            "<span class=\"{p}arrow\">&gt;</span> <span class=\"{p}caret\"> </span>",
            p = args.prefix
        )?;
    }
    write!(buf, "</pre>")?;

    println!("{}", buf);

    Ok(())
}

fn command_to_html(
    buf: &mut String,
    command_parts: &[&str],
    args: &Args,
) -> Result<(), Box<dyn Error>> {
    let command = cmd::concat(command_parts);

    let (cmd_out, cmd_err) = cmd::run(&command)?;
    let stdout = run_ansi_to_html(&html::dimmed_to_html(&cmd_out, &args.prefix))?;

    command_prompt_to_html(buf, command_parts, args)?;

    if !cmd_err.is_empty() {
        writeln!(buf, "{}", cmd_err)?;
    }
    write!(buf, "{}", stdout)?;
    Ok(())
}

fn command_prompt_to_html(
    buf: &mut String,
    command_parts: &[&str],
    args: &Args,
) -> Result<(), Box<dyn Error>> {
    let prefix = &args.prefix;
    write!(buf, "<span class=\"{}arrow\">&gt;</span>", prefix)?;

    let mut next = State::Start;

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    enum State {
        Default,
        Start,
        Pipe,
    }

    for &part in command_parts {
        let part_esc = &*html::esc_html(part);

        if part.contains(|c: char| c.is_ascii_whitespace()) {
            next = State::Default;
            write!(
                buf,
                "<span class=\"{}str\">\"{}\"</span>",
                prefix,
                part_esc.escape_debug(),
            )?;
        } else if next == State::Pipe {
            next = State::Default;
            write!(buf, "<span class=\"{}pipe\">{}</span>", prefix, part_esc)?;
        } else if next == State::Start {
            next = State::Default;
            write!(buf, "<span class=\"{}cmd\">{}</span>", prefix, part_esc)?;
        } else if part == "|" {
            next = State::Start;
            write!(buf, "<span class=\"{}pipe\">{}</span>", prefix, part_esc)?;
        } else if part == "<" || part == ">" {
            next = State::Pipe;
            write!(buf, "<span class=\"{}pipe\">{}</span>", prefix, part_esc)?;
        } else if part == "&&" {
            next = State::Start;
            write!(buf, "<span class=\"{}op\">{}</span>\n ", prefix, part_esc)?;
        } else if part.starts_with('-') {
            if let Some((i, _)) = part_esc.char_indices().find(|&(_, c)| c == '=') {
                let (p1, p2) = part_esc.split_at(i);
                write!(
                    buf,
                    "<span class=\"{}flag\">{}</span><span class=\"{}arg\">{}</span>",
                    prefix, p1, prefix, p2,
                )?;
            } else {
                write!(buf, "<span class=\"{}flag\">{}</span>", prefix, part_esc)?;
            }
        } else if args.highlight.contains(&part) {
            write!(buf, "<span class=\"{}hl\">{}</span>", prefix, part_esc)?;
        } else {
            write!(buf, "<span class=\"{}arg\">{}</span>", prefix, part_esc)?;
        }
    }
    writeln!(buf)?;
    Ok(())
}

fn run_ansi_to_html(input: &str) -> Result<String, Box<dyn Error>> {
    let process = Command::new("ansi-to-html")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let output = cmd::input(process, input)?.wait_with_output()?;

    Ok(cmd::stdout(&output)?)
}
