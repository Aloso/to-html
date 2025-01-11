//! Handles all of the configuration for `to-html`
//!
//! `to-html` can be configured either through CLI flags or through a config file. Options passed
//! on the command line take priority over those set in the config file when merging
//!
//! The flow is represented by `cli::Args` and `config::Config` being consolidated into the final
//! `Opts` that is used through the rest of the application

use std::path::PathBuf;

use ansi_to_html::{Esc, Theme};

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
    pub no_prompt: bool,
    pub theme: Theme,
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
                    theme: config_theme,
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
            no_prompt: cli_no_prompt,
            theme,
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
            no_prompt: cli_no_prompt,
            theme: theme.map(Into::into).unwrap_or(config_theme.into()),
        })
    }
}

#[derive(Debug)]
pub enum ShellPrompt {
    Arrow,
    Cwd { home: Option<PathBuf> },
}

impl From<cli::Theme> for Theme {
    fn from(value: cli::Theme) -> Self {
        match value {
            cli::Theme::Light => Theme::Light,
            cli::Theme::Dark => Theme::Dark,
        }
    }
}

impl From<config::Theme> for Theme {
    fn from(value: config::Theme) -> Self {
        match value {
            config::Theme::Light => Theme::Light,
            config::Theme::Dark => Theme::Dark,
        }
    }
}
