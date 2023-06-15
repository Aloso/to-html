use std::{fs, io};

use serde::Deserialize;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed detecting configuration directory")]
    ConfigDetection,
    #[error("I/O error while trying to read config file: {0}")]
    Io(#[from] io::Error),
    #[error("Config file has invalid format: {0}")]
    Parsing(#[from] basic_toml::Error),
}

pub fn load() -> Result<Config, Error> {
    let to_html_config = dirs_next::config_dir()
        .ok_or(Error::ConfigDetection)?
        .join("to-html")
        .join("config.toml");

    match fs::read_to_string(&to_html_config) {
        Ok(contents) => match basic_toml::from_str(&contents) {
            Ok(config) => Ok(config),
            Err(e) => Err(e.into()),
        },
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => Ok(Config::default()),
            _ => Err(e.into()),
        },
    }
}

#[derive(Deserialize, Default)]
pub struct Config {
    pub shell: Shell,
    pub output: Output,
}

#[derive(Deserialize, Default)]
pub struct Shell {
    pub program: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct Output {
    pub cwd: bool,
    pub full_document: bool,
    pub highlight: Vec<String>,
    pub css_prefix: Option<String>,
}
