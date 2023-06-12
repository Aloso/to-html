use regex::Regex;

use crate::{Ansi, AnsiIter, Color, Error, FourBitColorType};

mod minifier;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Style {
    Bold,
    Faint,
    Italic,
    Underline,
    CrossedOut,
    ForegroundColor(Color),
    BackgroundColor(Color),
}

impl Style {
    fn apply(&self, buf: &mut String, color_type: &FourBitColorType) {
        let s;
        buf.push_str(match self {
            Style::Bold => "<b>",
            Style::Faint => "<span style='opacity:0.67'>",
            Style::Italic => "<i>",
            Style::Underline => "<u>",
            Style::CrossedOut => "<s>",
            Style::ForegroundColor(c) => {
                s = c.into_opening_fg_span(color_type);
                &s
            }
            Style::BackgroundColor(c) => {
                s = c.into_opening_bg_span(color_type);
                &s
            }
        });
    }

    fn clear(&self, buf: &mut String) {
        buf.push_str(match self {
            Style::Bold => "</b>",
            Style::Faint => "</span>",
            Style::Italic => "</i>",
            Style::Underline => "</u>",
            Style::CrossedOut => "</s>",
            Style::ForegroundColor(_) => "</span>",
            Style::BackgroundColor(_) => "</span>",
        })
    }
}

/// Convert ANSI sequences to html. This does NOT escape html characters such as `<` and `&`.
pub fn ansi_to_html(
    mut input: &str,
    ansi_regex: &Regex,
    color_type: FourBitColorType,
) -> Result<String, Error> {
    let mut minifier = minifier::Minifier::new(color_type);

    loop {
        match ansi_regex.find(input) {
            Some(m) => {
                if m.start() > 0 {
                    let (before, after) = input.split_at(m.start());
                    minifier.push_str(before);
                    input = after;
                }

                let len = m.range().len();
                input = &input[len..];

                if !m.as_str().ends_with('m') {
                    continue;
                }

                if len == 3 {
                    minifier.clear_styles();
                    continue;
                }

                let nums = &m.as_str()[2..len - 1];
                let nums = nums.split(';').map(|n| n.parse::<u8>());

                for ansi in AnsiIter::new(nums) {
                    minifier.push_ansi_code(ansi?);
                }
            }
            None => {
                minifier.push_str(input);
                break;
            }
        }
    }
    minifier.push_ansi_code(Ansi::Reset); // make sure all tags are closed

    Ok(minifier.into_html())
}

#[derive(Debug, Default)]
struct AnsiConverter {
    styles: Vec<Style>,
    styles_to_apply: Vec<Style>,
    result: String,
    four_bit: FourBitColorType,
}

impl AnsiConverter {
    fn new(four_bit: FourBitColorType) -> Self {
        Self {
            four_bit,
            ..Self::default()
        }
    }

    fn consume_ansi_code(&mut self, ansi: Ansi) {
        match ansi {
            Ansi::Noop => {}
            Ansi::Reset => self.clear_style(|_| true),
            Ansi::Bold => self.set_style(Style::Bold),
            Ansi::Faint => self.set_style(Style::Faint),
            Ansi::Italic => self.set_style(Style::Italic),
            Ansi::Underline => self.set_style(Style::Underline),
            Ansi::CrossedOut => self.set_style(Style::CrossedOut),
            Ansi::BoldOff => self.clear_style(|&s| s == Style::Bold),
            Ansi::BoldAndFaintOff => self.clear_style(|&s| s == Style::Bold || s == Style::Faint),
            Ansi::ItalicOff => self.clear_style(|&s| s == Style::Italic),
            Ansi::UnderlineOff => self.clear_style(|&s| s == Style::Underline),
            Ansi::CrossedOutOff => self.clear_style(|&s| s == Style::CrossedOut),
            Ansi::ForgroundColor(c) => self.set_style(Style::ForegroundColor(c)),
            Ansi::DefaultForegroundColor => {
                self.clear_style(|&s| matches!(s, Style::ForegroundColor(_)))
            }
            Ansi::BackgroundColor(c) => self.set_style(Style::BackgroundColor(c)),
            Ansi::DefaultBackgroundColor => {
                self.clear_style(|&s| matches!(s, Style::BackgroundColor(_)))
            }
        }
    }

    fn set_style(&mut self, s: Style) {
        if !self.styles.contains(&s) {
            s.apply(&mut self.result, &self.four_bit);
            self.styles.push(s);
        }
    }

    fn clear_style(&mut self, cond: impl Fn(&Style) -> bool) {
        if let Some((i, _)) = self.styles.iter().enumerate().find(|&(_, s)| cond(s)) {
            while self.styles.len() > i {
                let style = self.styles.pop().unwrap();
                style.clear(&mut self.result);
                if !cond(&style) {
                    self.styles_to_apply.push(style);
                }
            }
        }
        for &style in &self.styles_to_apply {
            style.apply(&mut self.result, &self.four_bit);
            self.styles.push(style);
        }
        self.styles_to_apply.clear();
    }

    fn push_str(&mut self, s: &str) {
        self.result.push_str(s);
    }

    fn result(self) -> String {
        self.result
    }
}
