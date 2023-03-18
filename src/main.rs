use ansi_to_html::Esc;
use clap::Parser;
use std::{borrow::Cow, error, fmt::Write, path::PathBuf};

pub mod cmd;
mod lexer;

pub type StdError = Box<dyn error::Error>;

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    /// The command(s) to execute
    #[clap(required = true)]
    command: Vec<String>,
    /// The shell to run the command in. On macOS and FreeBSD, the shell has to support
    /// `-c <command>`
    #[clap(short, long)]
    shell: Option<String>,
    /// Programs that have subcommands (which should be highlighted). Multiple arguments are
    /// separated with a comma, e.g.
    /// to-html -l git,cargo,npm 'git checkout main'
    #[clap(short = 'l', long, require_value_delimiter = true)]
    highlight: Vec<String>,
    /// Prefix for CSS classes. For example, with the 'to-html' prefix, the 'arg' class becomes
    /// 'to-html-arg'",
    #[clap(short, long)]
    prefix: Option<String>,
    /// Don't run the commands, just emit the HTML for the command prompt
    #[clap(short, long)]
    no_run: bool,
    /// Print the (abbreviated) current working directory in the command prompt
    #[clap(short, long)]
    cwd: bool,
    /// Output a complete HTML document, not just a <pre>
    #[clap(short, long)]
    doc: bool,
}

#[derive(Debug)]
struct Args {
    commands: Vec<String>,
    shell: Option<String>,
    highlight: Vec<String>,
    prefix: String,
    no_run: bool,
    prompt: ShellPrompt,
    doc: bool,
}

impl From<Cli> for Args {
    fn from(cli: Cli) -> Self {
        let Cli {
            shell,
            highlight,
            prefix,
            no_run,
            cwd,
            doc,
            command: commands,
        } = cli;

        let prefix = prefix.map(|s| format!("{}-", Esc(s))).unwrap_or_default();
        let prompt = if cwd {
            ShellPrompt::Cwd {
                home: dirs_next::home_dir(),
            }
        } else {
            ShellPrompt::Arrow
        };

        Self {
            commands,
            shell,
            highlight,
            prefix,
            no_run,
            prompt,
            doc,
        }
    }
}

#[derive(Debug)]
enum ShellPrompt {
    Arrow,
    Cwd { home: Option<PathBuf> },
}

fn parse_args() -> Args {
    let cli = Cli::parse();
    Args::from(cli)
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
    let args = parse_args();

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

    for command in &args.commands {
        if args.no_run {
            fmt_command_prompt(&mut buf, command, &args)?;
        } else {
            fmt_command(&mut buf, command, &args)?;
        }
    }

    if !args.no_run {
        shell_prompt(&mut buf, &args)?;
        writeln!(buf, "<span class='{p}caret'> </span>", p = args.prefix)?;
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

    let (cmd_out, cmd_err, _) = cmd::run(command, args.shell.as_deref())?;
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
            write!(buf, "<span class='{}shell'>&gt; </span>", args.prefix)?;
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
                "<span class='{p}cwd'>{} </span><span class='{p}shell'>$ </span>",
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
