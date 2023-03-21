use std::mem;

use crate::{html::AnsiConverter, Ansi, Color};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CurrentStyling {
    fg: Option<Color>,
    bg: Option<Color>,
    bold: bool,
    faint: bool,
    italic: bool,
    underline: bool,
    crossed_out: bool,
}

impl CurrentStyling {
    fn apply(&mut self, ansi: Ansi) {
        match ansi {
            Ansi::Noop => {}
            Ansi::Reset => *self = Self::default(),
            Ansi::Bold => self.bold = true,
            Ansi::Faint => self.faint = true,
            Ansi::Italic => self.italic = true,
            Ansi::Underline => self.underline = true,
            Ansi::CrossedOut => self.crossed_out = true,
            Ansi::BoldOff => self.bold = false,
            Ansi::BoldAndFaintOff => {
                self.bold = false;
                self.faint = false;
            }
            Ansi::ItalicOff => self.italic = false,
            Ansi::UnderlineOff => self.underline = false,
            Ansi::CrossedOutOff => self.crossed_out = false,
            Ansi::ForgroundColor(c) => self.fg = Some(c),
            Ansi::DefaultForegroundColor => self.fg = None,
            Ansi::BackgroundColor(c) => self.bg = Some(c),
            Ansi::DefaultBackgroundColor => self.bg = None,
        }
    }
}

/// Collects runs of text and ANSI codes in an organized layout to allow for easy minifying before
/// converting to HTML
#[derive(Debug)]
pub(crate) struct Minifier {
    top: Top,
    storage: Vec<(String, Vec<Ansi>)>,
}

impl Default for Minifier {
    fn default() -> Self {
        Self {
            top: Top::default(),
            storage: vec![(String::new(), Vec::new())],
        }
    }
}

#[derive(Debug)]
enum Top {
    Text(String),
    Codes(Vec<Ansi>),
}

impl Default for Top {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

impl Minifier {
    pub fn clear_styles(&mut self) {
        self.push_ansi_code(Ansi::Reset);
    }

    pub fn push_ansi_code(&mut self, ansi: Ansi) {
        let prev_top = mem::take(&mut self.top);
        self.top = match prev_top {
            Top::Codes(mut codes) => {
                codes.push(ansi);
                Top::Codes(codes)
            }
            Top::Text(text) => {
                if !text.is_empty() {
                    self.storage.push((text, Vec::new()));
                }
                Top::Codes(vec![ansi])
            }
        };
    }

    pub fn push_str(&mut self, s: &str) {
        let prev_top = mem::take(&mut self.top);
        self.top = match prev_top {
            Top::Codes(codes) => {
                let last = self.storage.last_mut().expect("Starts with an entry");
                last.1.extend_from_slice(&codes);
                Top::Text(s.to_owned())
            }
            Top::Text(mut text) => {
                text.push_str(s);
                Top::Text(text)
            }
        };
    }

    pub fn into_html(self) -> String {
        let Self { top, mut storage } = self;

        // Consume lingering `top` to finish off `storage`
        match top {
            Top::Codes(codes) => storage
                .last_mut()
                .expect("Starts with an entry")
                .1
                .extend_from_slice(&codes),
            Top::Text(text) => {
                if !text.is_empty() {
                    let (last_text, last_codes) = storage.last_mut().expect("Starts with an entry");
                    if last_codes.is_empty() {
                        last_text.push_str(&text);
                    } else {
                        storage.push((text, Vec::new()));
                    }
                }
            }
        }

        // Iterate over each pair ignoring ansi codes that repeat the previously used style before
        // text is used. E.g.
        // Blue - "foo" - Reset, Blue - "bar" - Reset
        // becomes
        // Blue - "foo" - "bar" - Reset
        let mut styling = CurrentStyling::default();
        let mut converter = AnsiConverter::default();
        for (text, codes) in storage {
            converter.push_str(&text);

            // Only consume ansi codes if the styling would be different
            let prev_styling = styling;
            for &code in &codes {
                styling.apply(code);
            }
            if prev_styling != styling {
                for &code in &codes {
                    converter.consume_ansi_code(code);
                }
            }
        }

        converter.result()
    }
}
