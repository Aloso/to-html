use std::fmt;

pub struct Esc<'a>(pub &'a str);

impl fmt::Display for Esc<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.0.chars() {
            match c {
                '&' => fmt::Display::fmt("&amp;", f)?,
                '<' => fmt::Display::fmt("&lt;", f)?,
                '>' => fmt::Display::fmt("&gt;", f)?,
                c => fmt::Display::fmt(&c, f)?,
            }
        }
        Ok(())
    }
}

/// Convert "dimmed" ANSI sequences to HTML.
///
/// TODO: Eventually, this function should cover **all** ANSI sequences, since
/// the `ansi-to-html` program is buggy, and using it is a hack.
///
/// https://stackoverflow.com/questions/4842424/list-of-ansi-color-escape-sequences
pub fn dimmed_to_html(mut input: &str, prefix: &str) -> String {
    let mut open_tags = 0;
    let mut result = String::new();

    loop {
        match input.find('\x1b') {
            Some(i) => {
                if i > 0 {
                    let (before, after) = input.split_at(i);
                    result.push_str(before);
                    input = after;
                }
                if input.starts_with("\x1b[2m") {
                    result.push_str(&format!(r#"<span class="{}dim">"#, prefix));
                    input = &input[4..];
                    open_tags += 1;
                } else if input.starts_with("\x1b[0m") && open_tags > 0 {
                    result.push_str("</span>");
                    input = &input[4..];
                    open_tags -= 1;
                } else {
                    result.push('\x1b');
                    input = &input[1..];
                }
            }
            None => {
                result.push_str(input);
                break;
            }
        }
    }

    result
}
