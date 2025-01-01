use crate::{
    html::{AnsiConverter, AnsiSink, UnderlineStyle},
    Ansi, Color, Theme,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CurrentStyling {
    fg: Option<Color>,
    bg: Option<Color>,
    bold: bool,
    faint: bool,
    italic: bool,
    underline: Option<UnderlineStyle>,
    crossed_out: bool,
    inverted: bool,
}

impl CurrentStyling {
    fn apply(&mut self, ansi: Ansi) {
        match ansi {
            Ansi::Noop => {}
            Ansi::Reset => *self = Self::default(),
            Ansi::Bold => self.bold = true,
            Ansi::Faint => self.faint = true,
            Ansi::Italic => self.italic = true,
            Ansi::Underline => self.underline = Some(UnderlineStyle::Default),
            Ansi::DoubleUnderline => self.underline = Some(UnderlineStyle::Double),
            Ansi::Invert => self.inverted = true,
            Ansi::CrossedOut => self.crossed_out = true,
            Ansi::BoldAndFaintOff => {
                self.bold = false;
                self.faint = false;
            }
            Ansi::ItalicOff => self.italic = false,
            Ansi::UnderlineOff => self.underline = None,
            Ansi::InvertOff => self.inverted = false,
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
    pub(crate) fn new(var_prefix: Option<String>, theme: Theme) -> Self {
        Self {
            converter: AnsiConverter::new(var_prefix, theme),
            ..Self::default()
        }
    }

    /// Apply buffered ansi codes while ignoring ansi codes that repeat the previously used style
    fn apply_ansi_codes(&mut self) {
        let prev_styling = self.current_styling;
        for &code in &self.code_buffer {
            self.current_styling.apply(code);
        }
        if prev_styling != self.current_styling {
            for &code in &self.code_buffer {
                self.converter.push_ansi_code(code);
            }
        }
        self.code_buffer.clear();
    }
}

impl AnsiSink for Minifier {
    fn clear_styles(&mut self) {
        self.push_ansi_code(Ansi::Reset);
    }

    fn push_ansi_code(&mut self, ansi: Ansi) {
        self.code_buffer.push(ansi);
    }

    fn push_str(&mut self, text: &str) {
        self.apply_ansi_codes();
        self.converter.push_str(text);
    }

    fn to_html(&mut self) -> String {
        self.apply_ansi_codes();
        self.converter.to_html()
    }
}
