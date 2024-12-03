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

/// Basic minifier that avoids reapplying the same style to consecutive runs of text
///
/// E.g.
/// Blue - "foo" - Reset, Blue - "bar" - Reset
/// becomes
/// Blue - "foo" - "bar" - Reset
#[derive(Debug, Default)]
pub(crate) struct Minifier {
    code_buffer: Vec<Ansi>,
    current_styling: CurrentStyling,
    converter: AnsiConverter,
}

impl Minifier {
    pub(crate) fn new(var_prefix: Option<String>) -> Self {
        Self {
            converter: AnsiConverter::new(var_prefix),
            ..Self::default()
        }
    }

    pub fn clear_styles(&mut self) {
        self.push_ansi_code(Ansi::Reset);
    }

    pub fn push_ansi_code(&mut self, ansi: Ansi) {
        self.code_buffer.push(ansi);
    }

    /// Apply buffered ansi codes while ignoring ansi codes that repeat the previously used style
    fn apply_ansi_codes(&mut self) {
        let prev_styling = self.current_styling;
        let mut to_apply = &self.code_buffer[..];
        for (index, &code) in self.code_buffer.iter().enumerate() {
            self.current_styling.apply(code);
            if self.current_styling == prev_styling {
                to_apply = &self.code_buffer.get(index + 1..).unwrap_or_default();
            }
        }
        for &code in to_apply {
            self.converter.consume_ansi_code(code);
        }
        self.code_buffer.clear();
    }

    pub fn push_str(&mut self, text: &str) {
        // No point in applying styles to nothing
        if text.is_empty() {
            return;
        }

        self.apply_ansi_codes();
        self.converter.push_str(text);
    }

    pub fn into_html(mut self) -> String {
        // End of text, so flush out any styles
        self.converter.consume_ansi_code(Ansi::Reset);

        self.converter.result()
    }
}
