use clap::{App, AppSettings, Arg, ArgMatches};
use std::{borrow::Cow, error::Error, fmt::Write, path::PathBuf};
use to_html::{cmd, html, to_html};

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
            Arg::with_name("cwd")
                .long("cwd")
                .short("c")
                .help("Print the (abbreviated) current working directory in the command prompt"),
        ])
}

#[derive(Debug)]
struct Args<'a> {
    commands: Vec<&'a str>,
    highlight: Vec<&'a str>,
    prefix: String,
    no_run: bool,
    shell: Shell,
}

#[derive(Debug)]
enum Shell {
    Arrow,
    Cwd { home: Option<PathBuf> },
}

fn parse_args<'a>(matches: &'a ArgMatches) -> Result<Args<'a>, Box<dyn Error>> {
    let highlight = matches
        .values_of("highlight")
        .map(Iterator::collect)
        .unwrap_or_default();

    let prefix = matches
        .value_of("prefix")
        .map(|s| format!("{}-", html::Esc(s)))
        .unwrap_or_default();

    let commands = matches
        .values_of("command")
        .ok_or("command missing")?
        .collect();

    let no_run = matches.is_present("no-run");
    let shell = if matches.is_present("cwd") {
        Shell::Cwd {
            home: dirs_next::home_dir(),
        }
    } else {
        Shell::Arrow
    };

    Ok(Args {
        commands,
        highlight,
        prefix,
        no_run,
        shell,
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
        shell_prompt(&mut buf, &args)?;
        writeln!(buf, " <span class=\"{p}caret\"> </span>", p = args.prefix)?;
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

    command_prompt_to_html(buf, command_parts, args)?;

    let (cmd_out, cmd_err, _) = cmd::run(&command)?;
    if !cmd_out.is_empty() {
        let html = to_html(&cmd_out, &args.prefix)?;
        write!(buf, "{}", html)?;
    }
    if !cmd_err.is_empty() {
        let html = to_html(&cmd_err, &args.prefix)?;
        write!(buf, "{}", html)?;
    }

    Ok(())
}

fn command_prompt_to_html(
    buf: &mut String,
    command_parts: &[&str],
    args: &Args,
) -> Result<(), Box<dyn Error>> {
    shell_prompt(buf, args)?;

    let mut next = State::Start;

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    enum State {
        Default,
        Start,
        Pipe,
    }

    for &part in command_parts {
        let part_esc = html::Esc(part);
        let prefix = &args.prefix;

        if part.contains(|c: char| c.is_ascii_whitespace()) {
            next = State::Default;
            write!(
                buf,
                " <span class=\"{}str\">\"{}\"</span>",
                prefix,
                part_esc.to_string().escape_debug(),
            )?;
        } else if next == State::Pipe {
            next = State::Default;
            write!(buf, " <span class=\"{}pipe\">{}</span>", prefix, part_esc)?;
        } else if next == State::Start {
            next = State::Default;
            write!(buf, " <span class=\"{}cmd\">{}</span>", prefix, part_esc)?;
        } else if part == "|" {
            next = State::Start;
            write!(buf, " <span class=\"{}pipe\">{}</span>", prefix, part_esc)?;
        } else if part == "<" || part == ">" {
            next = State::Pipe;
            write!(buf, " <span class=\"{}pipe\">{}</span>", prefix, part_esc)?;
        } else if part == "&&" {
            next = State::Start;
            write!(buf, " <span class=\"{}op\">{}</span>\n ", prefix, part_esc)?;
        } else if part.starts_with('-') {
            if let Some((i, _)) = part.char_indices().find(|&(_, c)| c == '=') {
                let (p1, p2) = part.split_at(i);
                let (p1, p2) = (html::Esc(p1), html::Esc(p2));

                write!(buf, " <span class=\"{}flag\">{}</span>", prefix, p1)?;
                write!(buf, "<span class=\"{}arg\">{}</span>", prefix, p2)?;
            } else {
                write!(buf, " <span class=\"{}flag\">{}</span>", prefix, part_esc)?;
            }
        } else if args.highlight.contains(&part) {
            write!(buf, " <span class=\"{}hl\">{}</span>", prefix, part_esc)?;
        } else {
            write!(buf, " <span class=\"{}arg\">{}</span>", prefix, part_esc)?;
        }
    }
    writeln!(buf)?;
    Ok(())
}

fn shell_prompt(buf: &mut String, args: &Args) -> Result<(), Box<dyn Error>> {
    match &args.shell {
        Shell::Arrow => {
            write!(buf, "<span class=\"{}shell\">&gt;</span>", args.prefix)?;
        }
        Shell::Cwd { home } => {
            let cwd = std::env::current_dir()?;
            let cwd = cwd.to_str().ok_or("invalid UTF-8 in cwd")?;
            let cwd = match home {
                Some(home) => {
                    let home = home.to_str().ok_or("invalid UTF-8 in home dir")?;
                    Cow::Owned(cwd.replace(home, "~"))
                }
                None => Cow::Borrowed(cwd),
            };

            write!(
                buf,
                "<span class=\"{p}cwd\">{}</span> <span class=\"{p}shell\">$</span>",
                html::Esc(&cwd),
                p = args.prefix
            )?;
        }
    }
    Ok(())
}
