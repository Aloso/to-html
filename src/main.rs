use ansi_to_html::Esc;
use clap::{App, AppSettings, Arg, ArgMatches};
use std::{borrow::Cow, error, fmt::Write, path::PathBuf};

pub mod cmd;
mod lexer;

pub type StdError = Box<dyn error::Error>;

fn clap_app<'a, 'b>() -> App<'a, 'b> {
    App::new(env!("CARGO_PKG_NAME"))
        .about(
            "Terminal wrapper that generates HTML from ANSI escape sequences\n\
            This requires that `bash` and `ansi-to-html` are installed.",
        )
        .version(env!("CARGO_PKG_VERSION"))
        .author("Ludwig Stecher <ludwig.stecher@gmx.de>")
        .global_setting(AppSettings::ColoredHelp)
        .args(&[
            Arg::with_name("command")
                .index(1)
                .multiple(true)
                .help("The command(s) to execute")
                .required(true),
            Arg::with_name("shell")
                .long("shell")
                .short("s")
                .takes_value(true)
                .help(
                    "The shell to run the command in. \
                    On macOS and FreeBSD, the shell has to support `-c <command>`",
                ),
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
            Arg::with_name("doc")
                .long("doc")
                .short("d")
                .help("Output a complete HTML document, not just a <pre>"),
        ])
}

#[derive(Debug)]
struct Args<'a> {
    commands: Vec<&'a str>,
    shell: Option<&'a str>,
    highlight: Vec<&'a str>,
    prefix: String,
    no_run: bool,
    prompt: ShellPrompt,
    doc: bool,
}

#[derive(Debug)]
enum ShellPrompt {
    Arrow,
    Cwd { home: Option<PathBuf> },
}

fn parse_args<'a>(matches: &'a ArgMatches) -> Result<Args<'a>, StdError> {
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

    let shell = matches.value_of("shell");

    let no_run = matches.is_present("no-run");
    let prompt = if matches.is_present("cwd") {
        ShellPrompt::Cwd {
            home: dirs_next::home_dir(),
        }
    } else {
        ShellPrompt::Arrow
    };

    let doc = matches.is_present("doc");

    Ok(Args {
        commands,
        shell,
        highlight,
        prefix,
        no_run,
        prompt,
        doc,
    })
}

fn main() {
    match main_inner() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

fn main_inner() -> Result<(), StdError> {
    let matches = clap_app().get_matches();
    let args = parse_args(&matches)?;

    let mut buf = String::new();

    if args.doc {
        let lang = std::env::var("LANG")
            .ok()
            .and_then(|s| s.split('.').next().map(|s| s.replace('_', "-")));

        if let Some(lang) = lang {
            writeln!(buf, "<html lang=\"{}\">", Esc(lang))?;
        } else {
            writeln!(buf, "<html>")?;
        }

        let mut title = args
            .commands
            .iter()
            .flat_map(|s| s.chars().chain(", ".chars()))
            .collect::<String>();
        title.truncate(title.len() - 2);

        writeln!(
            buf,
            "<head>
<meta charset=\"utf-8\">
<title>{}</title>
<style>{}</style>
</head>
<body>",
            Esc(title),
            make_style(&args.prefix),
        )?;
    }

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

    if args.doc {
        writeln!(buf, "</body>\n</html>")?;
    }

    println!("{}", buf);

    Ok(())
}

fn fmt_command(buf: &mut String, command: &str, args: &Args) -> Result<(), StdError> {
    fmt_command_prompt(buf, command, args)?;

    let (cmd_out, cmd_err, _) = cmd::run(&command, args.shell)?;
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

fn fmt_command_prompt(buf: &mut String, command: &str, args: &Args) -> Result<(), StdError> {
    shell_prompt(buf, args)?;
    lexer::colorize(buf, command, args)?;
    writeln!(buf)?;

    Ok(())
}

fn shell_prompt(buf: &mut String, args: &Args) -> Result<(), StdError> {
    match &args.prompt {
        ShellPrompt::Arrow => {
            write!(buf, "<span class=\"{}shell\">&gt; </span>", args.prefix)?;
        }
        ShellPrompt::Cwd { home } => {
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

fn make_style(prefix: &str) -> String {
    format!(
        "
body {{
  background-color: #141414;
  color: white;
}}
.{p}terminal {{
  overflow: auto;
  line-height: 120%;
}}

.{p}terminal .{p}shell {{
  color: #32d132;
  user-select: none;
  pointer-events: none;
}}
.{p}terminal .{p}cmd {{
  color: #419df3;
}}
.{p}terminal .{p}hl {{
  color: #00ffff;
  font-weight: bold;
}}
.{p}terminal .{p}arg {{
  color: white;
}}
.{p}terminal .{p}str {{
  color: #ffba24;
}}
.{p}terminal .{p}pipe, .{p}terminal .{p}punct {{
  color: #a2be00;
}}
.{p}terminal .{p}flag {{
  color: #ff7167;
}}
.{p}terminal .{p}esc {{
  color: #d558f5;
  font-weight: bold;
}}
.{p}terminal .{p}caret {{
  background-color: white;
  user-select: none;
}}
",
        p = prefix,
    )
}
