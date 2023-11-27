//! Handles all of the configuration for `to-html`
//!
//! `to-html` can be configured either through CLI flags or through a config file. Options passed
//! on the command line take priority over those set in the config file when merging
//!
//! The flow is represented by `cli::Args` and `config::Config` being consolidated into the final
//! `Opts` that is used through the rest of the application

use std::path::PathBuf;

use ansi_to_html::Esc;

mod cli;
mod config;

#[derive(Debug)]
pub struct Opts {
    pub commands: Vec<String>,
    pub shell: Option<String>,
    pub highlight: Vec<String>,
    pub prefix: String,
    pub no_run: bool,
    pub prompt: ShellPrompt,
    pub doc: bool,
    pub hide_prompt: bool,
}

impl Opts {
    pub fn load() -> Result<Self, crate::StdError> {
        let config::Config {
            shell: config::Shell {
                program: config_shell,
            },
            output:
                config::Output {
                    cwd: config_cwd,
                    full_document: config_doc,
                    highlight: config_highlight,
                    css_prefix: config_prefix,
                },
        } = config::load()?;

        let cli::Cli {
            commands: cli_commands,
            shell: cli_shell,
            highlight: cli_highlight,
            prefix: cli_prefix,
            no_run: cli_no_run,
            cwd: cli_cwd,
            doc: cli_doc,
            hide_prompt: cli_hide_prompt,
        } = cli::parse();

        let prompt = if cli_cwd || config_cwd {
            ShellPrompt::Cwd {
                home: dirs_next::home_dir(),
            }
        } else {
            ShellPrompt::Arrow
        };
        let prefix = cli_prefix
            .or(config_prefix)
            .map(|s| format!("{}-", Esc(s)))
            .unwrap_or_default();

        Ok(Self {
            commands: cli_commands,
            shell: cli_shell.or(config_shell),
            highlight: cli_highlight.unwrap_or(config_highlight),
            prefix,
            no_run: cli_no_run,
            prompt,
            doc: cli_doc || config_doc,
            hide_prompt: cli_hide_prompt,
        })
    }
}

#[derive(Debug)]
pub enum ShellPrompt {
    Arrow,
    Cwd { home: Option<PathBuf> },
}
