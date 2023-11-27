use clap::Parser;

pub fn parse() -> Cli {
    Cli::parse()
}

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    max_term_width = 100,
    help_template = "\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}"
)]
pub struct Cli {
    /// The command(s) to execute. Must be wrapped in quotes.
    #[arg(required = true)]
    pub commands: Vec<String>,
    /// The shell to run the command in. On macOS and FreeBSD, the shell has to support
    /// `-c <command>`
    #[arg(short, long)]
    pub shell: Option<String>,
    /// Programs that have subcommands (which should be highlighted). Multiple arguments are
    /// separated with a comma, e.g. `to-html -l git,cargo,npm "git checkout main"`
    #[arg(short = 'l', long, value_delimiter = ',')]
    pub highlight: Option<Vec<String>>,
    /// Prefix for CSS classes. For example, with the `to-html` prefix, the `arg` class becomes
    /// `to-html-arg`
    #[arg(short, long)]
    pub prefix: Option<String>,
    /// Don't run the commands, just emit the HTML for the command prompt
    #[arg(short, long)]
    pub no_run: bool,
    /// Print the (abbreviated) current working directory in the command prompt
    #[arg(short, long)]
    pub cwd: bool,
    /// Output a complete HTML document, not just a `<pre>`
    #[arg(short, long)]
    pub doc: bool,
    /// Do not show the command prompt
    #[arg(short = 'H', long)]
    pub hide_prompt: bool,
}
