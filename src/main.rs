use ansi_to_html::Esc;
use clap::{App, AppSettings, Arg, ArgMatches};
use std::{borrow::Cow, error::Error, fmt::Write, path::PathBuf};

pub mod cmd;
mod lexer;

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
                .help("The command(s) to execute")
                .required(true),
            Arg::with_name("highlight")
                .long("highlight")
                .short("l")
                .help(
                    "Programs that have subcommands (which should be highlighted). \
                    Multiple arguments are separated with a comma, e.g.\n\
                    to-html -l git,cargo,npm 'git checkout main'",
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
                .short("n")
                .help("Don't run the commands, just emit the HTML for the command prompt"),
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
        .map(|s| format!("{}-", Esc(s)))
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

    let mut buf = String::new();
    writeln!(buf, "<pre class=\"{}terminal\">", args.prefix)?;

    for &command in &args.commands {
        if args.no_run {
            fmt_command_prompt(&mut buf, command, &args)?;
        } else {
            fmt_command(&mut buf, command, &args)?;
        }
    }

    if !args.no_run {
        shell_prompt(&mut buf, &args)?;
        writeln!(buf, "<span class=\"{p}caret\"> </span>", p = args.prefix)?;
    }
    write!(buf, "</pre>")?;

    println!("{}", buf);

    Ok(())
}

fn fmt_command(buf: &mut String, command: &str, args: &Args) -> Result<(), Box<dyn Error>> {
    fmt_command_prompt(buf, command, args)?;

    let (cmd_out, cmd_err, _) = cmd::run(&command)?;
    if !cmd_out.is_empty() {
        let html = ansi_to_html::convert_escaped(&cmd_out)?;
        write!(buf, "{}", html)?;
    }
    if !cmd_err.is_empty() {
        let html = ansi_to_html::convert_escaped(&cmd_err)?;
        write!(buf, "{}", html)?;
    }

    Ok(())
}

fn fmt_command_prompt(buf: &mut String, command: &str, args: &Args) -> Result<(), Box<dyn Error>> {
    shell_prompt(buf, args)?;
    lexer::colorize(buf, command, args)?;
    writeln!(buf)?;

    Ok(())
}

fn shell_prompt(buf: &mut String, args: &Args) -> Result<(), Box<dyn Error>> {
    match &args.shell {
        Shell::Arrow => {
            write!(buf, "<span class=\"{}shell\">&gt; </span>", args.prefix)?;
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
                "<span class=\"{p}cwd\">{} </span><span class=\"{p}shell\">$ </span>",
                Esc(&cwd),
                p = args.prefix
            )?;
        }
    }
    Ok(())
}
