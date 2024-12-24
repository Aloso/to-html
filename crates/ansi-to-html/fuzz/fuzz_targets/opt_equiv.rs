#![no_main]

use ansi_to_html::{convert, Converter};
use html_interpreter::{interpret_html, StylizedText};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|ansi_text: &str| {
    assert_opt_equiv_to_no_opt(ansi_text);
});

/// Ensures that our optimized HTML output is semantically equivalent to the unoptimized output
/// (along with verifying some other properties like our HTML being reasonably well-formed)
pub fn assert_opt_equiv_to_no_opt(ansi_text: &str) {
    let Ok(htmlified) = Converter::new().skip_optimize(true).convert(ansi_text) else {
        return;
    };
    let full_text = normalize_output(interpret_html(&htmlified));
    let opt_text = normalize_output(interpret_html(&convert(ansi_text).unwrap()));

    assert_eq!(
        full_text, opt_text,
        "Optimized text should be semantically equivalent"
    );
}

fn normalize_output(texts: Vec<StylizedText>) -> Vec<StylizedText> {
    texts
        .into_iter()
        // Filter out any empty spans of text
        .filter(|t| !t.text.is_empty())
        .fold(Vec::new(), |mut acc, text| {
            match acc.last_mut() {
                // Coalesce text with consecutive runs of the same style
                Some(top) if top.styles == text.styles => top.text.push_str(&text.text),
                _ => acc.push(text),
            }
            acc
        })
}
