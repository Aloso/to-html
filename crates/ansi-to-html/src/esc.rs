use std::fmt;

/// A formatting wrapper for escaping HTML in a string.
///
/// The `Display` implementation replaces
///   - `&` with `&amp;`
///   - `<` with `&lt;`
///   - `>` with `&gt;`
///   - `"` with `&quot;`
///   - `'` with `&#39;`
///
/// `Esc` is lazy: If you don't use it, it does nothing. Also, it
/// doesn't allocate a `String` unless you call `.to_string()`.
///
/// ## Examples
///
/// In a `format!`-like macro:
///
/// ```
/// # use ansi_to_html::Esc;
/// assert_eq!(format!("{}", Esc("<h1>")).as_str(), "&lt;h1&gt;");
/// ```
///
/// Convert it to a String directly:
///
/// ```
/// # use ansi_to_html::Esc;
/// assert_eq!(Esc("<h1>").to_string().as_str(), "&lt;h1&gt;");
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Esc<T: AsRef<str>>(pub T);

impl<T: AsRef<str>> fmt::Display for Esc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.0.as_ref().chars() {
            match c {
                '&' => fmt::Display::fmt("&amp;", f)?,
                '<' => fmt::Display::fmt("&lt;", f)?,
                '>' => fmt::Display::fmt("&gt;", f)?,
                '"' => fmt::Display::fmt("&quot;", f)?,
                '\'' => fmt::Display::fmt("&#39;", f)?,
                c => fmt::Display::fmt(&c, f)?,
            }
        }
        Ok(())
    }
}
