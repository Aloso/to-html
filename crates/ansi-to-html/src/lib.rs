//! Convert a string that can contain
//! [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code) to HTML.
//!
//! This crate currently supports SGR parameters (text style and colors).
//! The supported styles are:
//!
//! - bold
//! - italic
//! - underlined
//! - doubly underlined
//! - reverse video
//! - crossed out
//! - faint
//! - foreground and background colors: 3-bit, 4-bit, 8-bit, truecolor (24-bit)
//!
//! **Not** supported SGR parameters (note that most of these are niche features
//! and rarely supported by terminals):
//!
//! - slow/rapid blink
//! - conceal
//! - alternative fonts
//! - fraktur
//! - proportional spacing
//! - framed
//! - encircled
//! - overlined
//! - ideogram attributes
//! - non-standard extensions
//!   - underline color
//!   - superscript, subscript
//!   - bright foreground/background color
//!
//! All unsupported ANSI escape codes are stripped from the output.
//!
//! It should be easy to add support for more styles, if there's a straightforward HTML
//! representation. If you need a different style (e.g. doubly underlined), file an issue.
//!
//!
//! ## Example
//! ```
//! let bold = "\x1b[1m";
//! let red = "\x1b[31m";
//! let input = format!("<h1> {bold}Hello {red}world! </h1>");
//! let converted = ansi_to_html::convert(&input).unwrap();
//! assert_eq!(
//!     converted,
//!     "&lt;h1&gt; <b>Hello <span style='color:var(--red,#a00)'>world! &lt;/h1&gt;</span></b>"
//! );
//! ```
//!
//! Use the [`Converter`] builder for customization options.
#![deny(unsafe_code)]

use std::sync::OnceLock;

mod ansi;
mod color;
mod error;
mod esc;
mod html;

use ansi::{
    parse::{AnsiFragment, AnsiParser},
    Ansi, AnsiIter,
};
use color::Color;

pub use error::Error;
pub use esc::Esc;

use regex::Regex;

/// Converts a string containing ANSI escape codes to HTML.
///
/// Special html characters (`<>&'"`) are escaped prior to the conversion.
/// The number of generated HTML tags is minimized.
///
/// This behaviour can be customized by using the [`Converter`] builder.
///
/// ## Example
///
/// ```
/// let bold = "\x1b[1m";
/// let red = "\x1b[31m";
/// let input = format!("<h1> {bold}Hello {red}world! </h1>");
/// let converted = ansi_to_html::convert(&input).unwrap();
///
/// assert_eq!(
///     converted,
///     "&lt;h1&gt; <b>Hello <span style='color:var(--red,#a00)'>world! &lt;/h1&gt;</span></b>",
/// );
/// ```
pub fn convert(ansi_string: &str) -> Result<String, Error> {
    Converter::new().convert(ansi_string)
}

/// A builder for converting a string containing ANSI escape codes to HTML.
///
/// By default this will:
///
/// - Escape special HTML characters (`<>&'"`) prior to conversion.
/// - Apply optimizations to minimize the number of generated HTML tags.
/// - Use hardcoded colors.
/// - Uses a dark theme (assumes white text on a dark background).
///
/// ## Example
///
/// This skips HTML escaping and optimization, and sets a prefix for the CSS
/// variables to customize 4-bit colors.
///
/// ```
/// use ansi_to_html::Converter;
///
/// let converter = Converter::new()
///     .skip_escape(true)
///     .skip_optimize(true)
///     .four_bit_var_prefix(Some("custom-".to_owned()));
///
/// let bold = "\x1b[1m";
/// let red = "\x1b[31m";
/// let reset = "\x1b[0m";
/// let input = format!("<h1> <i></i> {bold}Hello {red}world!{reset} </h1>");
/// let converted = converter.convert(&input).unwrap();
///
/// assert_eq!(
///     converted,
///     // The `<h1>` and `</h1>` aren't escaped, useless `<i></i>` is kept, and
///     // `<span class='red'>` is used instead of `<span style='color:#a00'>`
///     "<h1> <i></i> <b>Hello <span style='color:var(--custom-red,#a00)'>world!</span></b> </h1>",
/// );
/// ```
#[derive(Clone, Debug, Default)]
pub struct Converter {
    skip_escape: bool,
    skip_optimize: bool,
    four_bit_var_prefix: Option<String>,
    theme: Theme,
}

#[deprecated(note = "this is now a type alias for the `Converter` builder")]
pub type Opts = Converter;

impl Converter {
    /// Creates a new set of default options.
    pub fn new() -> Self {
        Converter::default()
    }

    /// Avoids escaping special HTML characters prior to conversion.
    pub fn skip_escape(mut self, skip: bool) -> Self {
        self.skip_escape = skip;
        self
    }

    /// Skips removing some useless HTML tags.
    pub fn skip_optimize(mut self, skip: bool) -> Self {
        self.skip_optimize = skip;
        self
    }

    /// Adds a custom prefix for the CSS variables used for all the 4-bit colors.
    pub fn four_bit_var_prefix(mut self, prefix: Option<String>) -> Self {
        self.four_bit_var_prefix = prefix;
        self
    }

    /// Sets the color theme of the terminal.
    ///
    /// This is needed to decide how text with the "reverse video" ANSI code is displayed.
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Converts a string containing ANSI escape codes to HTML.
    pub fn convert(&self, input: &str) -> Result<String, Error> {
        let Converter {
            skip_escape,
            skip_optimize,
            ref four_bit_var_prefix,
            theme,
        } = *self;

        let four_bit_var_prefix = four_bit_var_prefix.to_owned();
        let html = if skip_escape {
            html::ansi_to_html(input, four_bit_var_prefix, theme, skip_optimize)?
        } else {
            let input = Esc(input).to_string();
            html::ansi_to_html(&input, four_bit_var_prefix, theme, skip_optimize)?
        };

        let html = if skip_optimize { html } else { optimize(&html) };

        Ok(html)
    }
}

/// The terminal's color theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Theme {
    Light,
    #[default]
    Dark,
}

#[deprecated(note = "Use the `convert` method of the `Converter` builder")]
pub fn convert_with_opts(input: &str, converter: &Converter) -> Result<String, Error> {
    converter.convert(input)
}

const OPT_REGEX_1: &str = r"<span \w+='[^']*'></span>|<b></b>|<i></i>|<u></u>|<s></s>";
const OPT_REGEX_2: &str = "</b><b>|</i><i>|</s><s>";

fn optimize(html: &str) -> String {
    static REGEXES: OnceLock<(Regex, Regex)> = OnceLock::new();
    let (regex1, regex2) = REGEXES.get_or_init(|| {
        (
            Regex::new(OPT_REGEX_1).unwrap(),
            Regex::new(OPT_REGEX_2).unwrap(),
        )
    });

    let html = regex1.replace_all(html, "");
    let html = regex2.replace_all(&html, "");

    html.to_string()
}
