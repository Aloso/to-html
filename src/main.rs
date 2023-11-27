use ansi_to_html::Esc;
use std::{borrow::Cow, error, fmt::Write};

pub mod cmd;
mod lexer;
mod opts;

use opts::{Opts, ShellPrompt};

pub type StdError = Box<dyn error::Error>;

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
    let opts = opts::Opts::load()?;

    let mut buf = String::new();

    if opts.doc {
        let lang = std::env::var("LANG")
            .ok()
            .and_then(|s| s.split('.').next().map(|s| s.replace('_', "-")));

        if let Some(lang) = lang {
            writeln!(buf, "<html lang=\"{}\">", Esc(lang))?;
        } else {
            writeln!(buf, "<html>")?;
        }

        let mut title = opts
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
            make_style(&opts.prefix),
        )?;
    }

    writeln!(buf, "<pre class=\"{}terminal\">", opts.prefix)?;

    for command in &opts.commands {
        if opts.no_run {
            fmt_command_prompt(&mut buf, command, &opts)?;
        } else {
            fmt_command(&mut buf, command, &opts)?;
        }
    }

    if !opts.no_run {
        shell_prompt(&mut buf, &opts)?;
        writeln!(buf, "<span class='{p}caret'> </span>", p = opts.prefix)?;
    }
    write!(buf, "</pre>")?;

    if opts.doc {
        writeln!(buf, "</body>\n</html>")?;
    }

    println!("{}", buf);

    Ok(())
}

fn fmt_command(buf: &mut String, command: &str, opts: &Opts) -> Result<(), StdError> {
    if !opts.hide_prompt {
        fmt_command_prompt(buf, command, opts)?;
    }

    let var_prefix = if opts.prefix.is_empty() {
        None
    } else {
        Some(opts.prefix.to_owned())
    };
    let convert_opts = ansi_to_html::Opts::default().four_bit_var_prefix(var_prefix);

    let (cmd_out, cmd_err, _) = cmd::run(command, opts.shell.as_deref())?;
    if !cmd_out.is_empty() {
        let html = ansi_to_html::convert_with_opts(&cmd_out, &convert_opts)?;
        write!(buf, "{}", html)?;
    }
    if !cmd_err.is_empty() {
        let html = ansi_to_html::convert_with_opts(&cmd_err, &convert_opts)?;
        write!(buf, "{}", html)?;
    }

    Ok(())
}

fn fmt_command_prompt(buf: &mut String, command: &str, opts: &Opts) -> Result<(), StdError> {
    shell_prompt(buf, opts)?;
    lexer::colorize(buf, command, opts)?;
    writeln!(buf)?;

    Ok(())
}

fn shell_prompt(buf: &mut String, opts: &Opts) -> Result<(), StdError> {
    match &opts.prompt {
        ShellPrompt::Arrow => {
            write!(buf, "<span class='{}shell'>&gt; </span>", opts.prefix)?;
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
                p = opts.prefix
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
