//! Convert a string that can contain
//! [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code) to HTML.
//!
//! This crate currently supports SGR parameters (text style and colors).
//! The supported styles are:
//!
//! - bold
//! - italic
//! - underlined
//! - crossed out
//! - faint
//! - foreground and background colors: 3-bit, 4-bit, 8-bit, truecolor (24-bit)
//!
//! **Not** supported SGR parameters (note that most of these are niche features
//! and rarely supported by terminals):
//!
//! - slow/rapid blink
//! - reverse video
//! - conceal
//! - alternative fonts
//! - fraktur
//! - doubly underlined
//! - proportional spacing
//! - framed
//! - encircled
//! - overlined
//! - underline color (not in standard)
//! - ideogram attributes
//! - superscript, subscript (not in standard)
//! - bright foreground/background color (not in standard)
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
//!     "&lt;h1&gt; <b>Hello <span style='color:#a00'>world! &lt;/h1&gt;</span></b>"
//! );
//! ```
//!
//! ## Features
//!
//! Enable the `lazy-init` feature to initialize a few things lazily, which is faster if you're
//! converting many strings.
#![deny(unsafe_code)]

mod ansi;
mod color;
mod error;
mod esc;
mod html;

use ansi::{Ansi, AnsiIter};
use color::Color;

pub use error::Error;
pub use esc::Esc;

#[cfg(feature = "once_cell")]
use once_cell::sync::Lazy;
use regex::Regex;

/// Converts a string containing ANSI escape codes to HTML.
///
/// Special html characters (`<>&'"`) are escaped prior to the conversion.
///
/// This function attempts to minimize the number of generated HTML tags.
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
///     "&lt;h1&gt; <b>Hello <span style='color:#a00'>world! &lt;/h1&gt;</span></b>",
/// );
/// ```
pub fn convert(ansi_string: &str) -> Result<String, Error> {
    convert_with_opts(ansi_string, &Opts::default())
}

/// Dictates use of `<span class='{name}'>` or `<span style='color:{hex}'>` for 4-bit colors
///
/// See [`Opts::four_bit_color_type()`] for how to set this.
#[derive(Clone, Copy, Debug, Default)]
pub enum FourBitColorType {
    /// Uses `<span style='color:{hex}'>` with hardcoded values for each color.
    #[default]
    Hardcoded,
    /// Uses `<span class='{name}'>` with a unique name for each color.
    ///
    /// The `{name}` is set as follows:
    ///
    /// 1. The name for the color e.g. `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`,
    ///    and `white`.
    /// 2. If the color is the bright variant then it will be prefixed with `bright-`.
    /// 3. If it is a background color then it will be prefixed with `bg-`.
    ///
    /// ## Example
    ///
    /// - Background bright red - `bg-bright-red`
    /// - Background blue - `bg-blue`
    /// - Plain Yellow - `yellow`
    Class,
}

/// Customizes the behavior of [`convert_with_opts()`]
///
/// By default this will:
///
/// - Escape special HTML characters (`<>&'"`) prior to conversion.
/// - Optimizes to minimize the number of generated HTML tags.
/// - Uses hardcoded colors.
#[derive(Clone, Debug, Default)]
pub struct Opts {
    skip_escape: bool,
    skip_optimize: bool,
    four_bit: FourBitColorType,
}

impl Opts {
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

    /// Configures how color span's attributes will have the color applied.
    ///
    /// Defaults to using hardcoded hex colors.
    pub fn four_bit_color_type(mut self, color_type: FourBitColorType) -> Self {
        self.four_bit = color_type;
        self
    }
}

/// Converts a string containing ANSI escape codes to HTML with customized behavior.
///
/// If `escaped` is `true`, then special html characters (`<>&'"`) are escaped prior
/// to the conversion.
///
/// If `optimized` is `true`, this function attempts to minimize the number of
/// generated HTML tags. Set it to `false` if you want optimal performance.
///
/// ## Example
///
/// ```
/// use ansi_to_html::{convert_with_opts, FourBitColorType, Opts};
///
/// let opts = Opts::default()
///     .skip_escape(true)
///     .skip_optimize(true)
///     .four_bit_color_type(FourBitColorType::Class);
/// let bold = "\x1b[1m";
/// let red = "\x1b[31m";
/// let reset = "\x1b[0m";
/// let input = format!("<h1> <i></i> {bold}Hello {red}world!{reset} </h1>");
/// let converted = convert_with_opts(&input, &opts).unwrap();
///
/// assert_eq!(
///     converted,
///     // The `<h1>` and `</h1>` aren't escaped, useless `<i></i>` is kept, and
///     // `<span class='red'>` is used instead of `<span style='color:#a00'>`
///     "<h1> <i></i> <b>Hello <span class='red'>world!</span></b> </h1>",
/// );
/// ```
pub fn convert_with_opts(input: &str, opts: &Opts) -> Result<String, Error> {
    let Opts {
        skip_escape,
        skip_optimize,
        four_bit,
    } = opts;

    let html = if *skip_escape {
        html::ansi_to_html(input, &ansi_regex(), *four_bit)?
    } else {
        let input = Esc(input).to_string();
        html::ansi_to_html(&input, &ansi_regex(), *four_bit)?
    };

    let html = if *skip_optimize {
        html
    } else {
        optimize(&html)
    };

    Ok(html)
}

const ANSI_REGEX: &str = "\x1b(\\[[0-9;?]*[A-HJKSTfhilmnsu]|\\(B)";
const OPT_REGEX_1: &str = r"<span \w+='[^']*'></span>|<b></b>|<i></i>|<u></u>|<s></s>";
const OPT_REGEX_2: &str = "</b><b>|</i><i>|</u><u>|</s><s>";

#[cfg(not(feature = "once_cell"))]
fn ansi_regex() -> Regex {
    Regex::new(ANSI_REGEX).unwrap()
}

#[cfg(feature = "once_cell")]
fn ansi_regex() -> &'static Regex {
    static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(ANSI_REGEX).unwrap());
    &*REGEX
}

#[cfg(not(feature = "once_cell"))]
fn optimize(html: &str) -> String {
    let html = Regex::new(OPT_REGEX_1).unwrap().replace_all(html, "");
    let html = Regex::new(OPT_REGEX_2).unwrap().replace_all(&html, "");

    html.to_string()
}

#[cfg(feature = "once_cell")]
fn optimize(html: &str) -> String {
    static REGEXES: Lazy<(Regex, Regex)> = Lazy::new(|| {
        (
            Regex::new(OPT_REGEX_1).unwrap(),
            Regex::new(OPT_REGEX_2).unwrap(),
        )
    });
    let (regex1, regex2) = &*REGEXES;

    let html = regex1.replace_all(html, "");
    let html = regex2.replace_all(&html, "");

    html.to_string()
}
