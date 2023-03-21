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
    storage: Vec<(String, Vec<Ansi>)>,
}

impl Default for Minifier {
    fn default() -> Self {
        Self {
            storage: vec![(String::new(), Vec::new())],
        }
    }
}

impl Minifier {
    pub fn clear_styles(&mut self) {
        self.push_ansi_code(Ansi::Reset);
    }

    pub fn push_ansi_code(&mut self, ansi: Ansi) {
        self.storage
            .last_mut()
            .expect("Starts with an entry")
            .1
            .push(ansi);
    }

    pub fn push_str(&mut self, s: &str) {
        let top = self.storage.last_mut().expect("Starts with an entry");
        if top.1.is_empty() {
            top.0.push_str(s);
        } else {
            self.storage.push((s.to_owned(), Vec::new()));
        }
    }

    pub fn into_html(self) -> String {
        let Self { storage } = self;

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
