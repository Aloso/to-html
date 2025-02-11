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
        fn fmt_special_terminated_text(f: &mut fmt::Formatter<'_>, text: &str) -> fmt::Result {
            debug_assert!(text.chars().filter(|c| SPECIAL_CHARS.contains(c)).count() == 1);
            let special = text
                .chars()
                .last()
                .expect("Must be called with text ending on a special char");
            f.write_str(&text[..text.len() - special.len_utf8()])?;
            let escaped = match special {
                '&' => "&amp;",
                '<' => "&lt;",
                '>' => "&gt;",
                '"' => "&quot;",
                '\'' => "&#39;",
                _ => unreachable!("We covered all patterns from `.ends_with(SPECIAL_CHARS)`"),
            };
            f.write_str(escaped)?;
            Ok(())
        }

        const SPECIAL_CHARS: [char; 5] = ['&', '<', '>', '"', '\''];

        let mut chunk_iter = self.0.as_ref().split_inclusive(SPECIAL_CHARS);
        // All chunks aside from the last are guaranteed to be terminated with a special char
        // > Differs from the iterator produced by `split` in that `split_inclusive` leaves the
        // > matched part as the terminator of the substring.
        let last_chunk = chunk_iter.next_back();
        for chunk in chunk_iter {
            fmt_special_terminated_text(f, chunk)?;
        }
        if let Some(chunk) = last_chunk {
            // The last chunk might end with a special char
            // > If the last element of the string is matched, that element will be considered the
            // > terminator of the preceding substring. That substring will be the last item
            // > returned by the iterator.
            if chunk.ends_with(SPECIAL_CHARS) {
                fmt_special_terminated_text(f, chunk)?;
            } else {
                f.write_str(chunk)?;
            }
        }

        Ok(())
    }
}
