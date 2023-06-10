use crate::{html::AnsiConverter, Ansi, Color, FourBitColorType};

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
    pub fn new(color_type: FourBitColorType) -> Self {
        Self {
            converter: AnsiConverter::new(color_type),
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
        for &code in &self.code_buffer {
            self.current_styling.apply(code);
        }
        if prev_styling != self.current_styling {
            for &code in &self.code_buffer {
                self.converter.consume_ansi_code(code);
            }
        }
        self.code_buffer.clear();
    }

    pub fn push_str(&mut self, text: &str) {
        self.apply_ansi_codes();
        self.converter.push_str(text);
    }

    pub fn into_html(mut self) -> String {
        self.apply_ansi_codes();
        self.converter.result()
    }
}
