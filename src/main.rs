use ansi_to_html::{Esc, Theme};
use std::{borrow::Cow, error, fmt::Write};

pub mod cmd;
mod lexer;
mod opts;
mod process;

use opts::{Opts, ShellPrompt};

pub type StdError = Box<dyn error::Error>;

fn main() {
    match main_inner() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{e}");
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
            make_style(&opts.prefix, opts.theme),
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

    if !opts.no_run && !opts.no_prompt {
        shell_prompt(&mut buf, &opts)?;
        writeln!(buf, "<span class='{p}caret'> </span>", p = opts.prefix)?;
    }
    write!(buf, "</pre>")?;

    if opts.doc {
        writeln!(buf, "</body>\n</html>")?;
    }

    println!("{buf}");

    Ok(())
}

fn fmt_command(buf: &mut String, command: &str, opts: &Opts) -> Result<(), StdError> {
    if !opts.no_prompt {
        fmt_command_prompt(buf, command, opts)?;
    }

    let var_prefix = if opts.prefix.is_empty() {
        None
    } else {
        Some(opts.prefix.to_owned())
    };
    let converter = ansi_to_html::Converter::new()
        .four_bit_var_prefix(var_prefix)
        .theme(opts.theme);

    let mut cmd = String::new();
    let shell = opts.shell.as_deref().or_else(|| {
        cmd = process::get_ancestor_process_cmd(1)?;
        Some(cmd.as_str()).filter(|&n| {
            matches!(
                n.rsplit('/').next(),
                Some("bash" | "sh" | "fish" | "zsh" | "csh" | "ksh" | "elvish")
            )
        })
    });

    let (cmd_out, cmd_err, _) = cmd::run(command, shell)?;
    if !cmd_out.is_empty() {
        let html = converter.convert(&cmd_out)?;
        write!(buf, "{html}")?;
    }
    if !cmd_err.is_empty() {
        let html = converter.convert(&cmd_err)?;
        write!(buf, "{html}")?;
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

fn make_style(prefix: &str, theme: Theme) -> String {
    macro_rules! format_colors {
        ($s:literal $(, $name:ident)* $(,)?) => {
            format!($s, p = prefix, $( $name = get_color(Color::$name, theme) ),*)
        };
    }

    format_colors!(
        "
body {{
  background-color: {Bg};
  color: {Fg};
}}
.{p}terminal {{
  overflow: auto;
  line-height: 120%;
}}

.{p}terminal .{p}shell {{
  color: {Shell};
  user-select: none;
  pointer-events: none;
}}
.{p}terminal .{p}cmd {{
  color: {Cmd};
}}
.{p}terminal .{p}hl {{
  color: {Hl};
  font-weight: bold;
}}
.{p}terminal .{p}arg {{
  color: {Arg};
}}
.{p}terminal .{p}str {{
  color: {Str};
}}
.{p}terminal .{p}pipe, .{p}terminal .{p}punct {{
  color: {Punct};
}}
.{p}terminal .{p}flag {{
  color: {Flag};
}}
.{p}terminal .{p}esc {{
  color: {Esc};
  font-weight: bold;
}}
.{p}terminal .{p}caret {{
  background-color: {CaretBg};
  user-select: none;
}}
",
        Bg,
        Fg,
        Shell,
        Cmd,
        Hl,
        Arg,
        Str,
        Punct,
        Flag,
        Esc,
        CaretBg,
    )
}

enum Color {
    Bg,
    Fg,
    Shell,
    Cmd,
    Hl,
    Arg,
    Str,
    Punct,
    Flag,
    Esc,
    CaretBg,
}

fn get_color(color: Color, theme: Theme) -> &'static str {
    match theme {
        Theme::Dark => match color {
            Color::Bg => "#141414",
            Color::Fg => "white",
            Color::Shell => "#32d132",
            Color::Cmd => "#419df3",
            Color::Hl => "#00ffff",
            Color::Arg => "white",
            Color::Str => "#ffba24",
            Color::Punct => "#a2be00",
            Color::Flag => "#ff7167",
            Color::Esc => "#d558f5",
            Color::CaretBg => "white",
        },
        Theme::Light => match color {
            Color::Bg => "#eeeeee",
            Color::Fg => "black",
            Color::Shell => "#1fa21f",
            Color::Cmd => "#1a71c1",
            Color::Hl => "#00c4c4",
            Color::Arg => "black",
            Color::Str => "#ce6a00",
            Color::Punct => "#819700",
            Color::Flag => "#b33742",
            Color::Esc => "#9f1adb",
            Color::CaretBg => "black",
        },
    }
}
