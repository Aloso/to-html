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
//! // \x1b[1m : bold   \x1b[31m : red
//! let input = "<h1> \x1b[1m Hello \x1b[31m world! </h1>";
//! let converted = ansi_to_html::convert_escaped(input).unwrap();
//! assert_eq!(
//!     converted.as_str(),
//!     "&lt;h1&gt; <b> Hello <span style='color:#a00'> world! &lt;/h1&gt;</span></b>"
//! );
//! ```
//!
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
/// Special html characters (`<>&'"`) are escaped prior to the conversion.
///
/// This function attempts to minimize the number of generated HTML tags.
///
/// ## Example
///
/// ```
/// // \x1b[1m : bold   \x1b[31m : red
/// let input = "<h1> \x1b[1m Hello \x1b[31m world! </h1>";
/// let converted = ansi_to_html::convert_escaped(input).unwrap();
///
/// assert_eq!(
///     converted.as_str(),
///     "&lt;h1&gt; <b> Hello <span style='color:#a00'> world! &lt;/h1&gt;</span></b>"
/// );
/// ```
pub fn convert_escaped(ansi_string: &str) -> Result<String, Error> {
    let input = Esc(ansi_string).to_string();
    let html = html::ansi_to_html(&input, &ansi_regex())?;
    Ok(optimize(&html))
}

/// Converts a string containing ANSI escape codes to HTML.
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
/// // \x1b[1m : bold   \x1b[31m : red   \x1b[22m : bold off
/// let input = "\x1b[1m Hello \x1b[31m world \x1b[22m!";
/// let converted = ansi_to_html::convert_escaped(input).unwrap();
///
/// assert_eq!(
///     converted.as_str(),
///     "<b> Hello <span style='color:#a00'> world </span></b><span style='color:#a00'>!</span>"
/// );
/// ```
pub fn convert(input: &str, escaped: bool, optimized: bool) -> Result<String, Error> {
    let html = if escaped {
        let input = Esc(input).to_string();
        html::ansi_to_html(&input, &ansi_regex())?
    } else {
        html::ansi_to_html(&input, &ansi_regex())?
    };

    let html = if optimized { optimize(&html) } else { html };
    Ok(html)
}

const ANSI_REGEX: &str = "\x1b(\\[[0-9;?]*[A-HJKSTfhilmnsu]|\\(B)";
const OPT_REGEX_1: &str = "<span [~>]*></span>|<b></b>|<i></i>|<u></u>|<s></s>";
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
    let html = Regex::new(OPT_REGEX_1).unwrap().replace_all(&html, "");
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

    let html = regex1.replace_all(&html, "");
    let html = regex2.replace_all(&html, "");

    html.to_string()
}
