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
/// assert_eq!(&format!("{}", Esc("<h1>")), "&lt;h1&gt;");
/// ```
///
/// Convert it to a String directly:
///
/// ```
/// # use ansi_to_html::Esc;
/// assert_eq!(&Esc("<h1>").to_string(), "&lt;h1&gt;");
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Esc<T: AsRef<str>>(pub T);

impl<T: AsRef<str>> fmt::Display for Esc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = self.0.as_ref();
        while let Some(pos) = s
            .as_bytes()
            .iter()
            .position(|b| [b'&', b'<', b'>', b'"', b'\''].contains(&b))
        {
            f.write_str(&s[..pos])?;
            let escaped = match s.as_bytes()[pos] {
                b'&' => "&amp;",
                b'<' => "&lt;",
                b'>' => "&gt;",
                b'"' => "&quot;",
                b'\'' => "&#39;",
                _ => unreachable!("We covered all patterns from `.ends_with(SPECIAL_CHARS)`"),
            };
            f.write_str(escaped)?;
            s = &s[pos + 1..];
        }
        f.write_str(s)
    }
}
