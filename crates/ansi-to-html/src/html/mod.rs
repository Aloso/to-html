use regex::Regex;

use crate::{color::FourBitColor, Ansi, AnsiIter, Color, Error};

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
    fn apply(&self, buf: &mut String, var_prefix: Option<&str>) {
        let s;
        buf.push_str(match self {
            Style::Bold => "<b>",
            Style::Faint => "<span style='opacity:0.67'>",
            Style::Italic => "<i>",
            Style::Underline => "<u>",
            Style::CrossedOut => "<s>",
            Style::ForegroundColor(c) => {
                s = c.into_opening_fg_span(var_prefix);
                &s
            }
            Style::BackgroundColor(c) => {
                s = c.into_opening_bg_span(var_prefix);
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
    four_bit_var_prefix: Option<String>,
) -> Result<String, Error> {
    let mut minifier = minifier::Minifier::new(four_bit_var_prefix);

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
    four_bit_var_prefix: Option<String>,
    inverted: bool,
}

impl AnsiConverter {
    fn new(four_bit_var_prefix: Option<String>) -> Self {
        Self {
            four_bit_var_prefix,
            ..Self::default()
        }
    }

    fn consume_ansi_code(&mut self, ansi: Ansi) {
        match ansi {
            Ansi::Noop => {}
            Ansi::Reset => {
                self.clear_style(|_| true);
                self.inverted = false;
            }
            Ansi::Bold => self.set_style(Style::Bold),
            Ansi::Faint => self.set_style(Style::Faint),
            Ansi::Italic => self.set_style(Style::Italic),
            Ansi::Invert if self.inverted => {}
            Ansi::Invert => {
                self.inverted = true;
                self.invert_fg_bg(FourBitColor::White.into(), FourBitColor::Black.into());
            }
            Ansi::Underline => self.set_style(Style::Underline),
            Ansi::CrossedOut => self.set_style(Style::CrossedOut),
            Ansi::BoldOff => self.clear_style(|&s| s == Style::Bold),
            Ansi::BoldAndFaintOff => self.clear_style(|&s| s == Style::Bold || s == Style::Faint),
            Ansi::ItalicOff => self.clear_style(|&s| s == Style::Italic),
            Ansi::UnderlineOff => self.clear_style(|&s| s == Style::Underline),
            Ansi::InvertOff if !self.inverted => {}
            Ansi::InvertOff => {
                self.inverted = false;
                self.invert_fg_bg(FourBitColor::Black.into(), FourBitColor::White.into());
            }
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

    fn invert_fg_bg(&mut self, default_fg: Color, default_bg: Color) {
        let mut new_fg = None;
        let mut new_bg = None;
        for style in self.styles.iter().rev() {
            match (style, new_fg, new_bg) {
                (_, Some(_), Some(_)) => break,
                (Style::ForegroundColor(fg), None, _) => new_bg = Some(*fg),
                (Style::BackgroundColor(bg), _, None) => new_fg = Some(*bg),
                _ => {}
            }
        }

        // Default the inverted fg/bg if missing
        let new_fg = new_fg.unwrap_or(default_fg);
        let new_bg = new_bg.unwrap_or(default_bg);

        // Actually swap them
        self.set_style(Style::ForegroundColor(new_fg));
        self.set_style(Style::BackgroundColor(new_bg));
    }

    fn set_style(&mut self, s: Style) {
        if !self.styles.contains(&s) {
            s.apply(&mut self.result, self.four_bit_var_prefix.as_deref());
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
            style.apply(&mut self.result, self.four_bit_var_prefix.as_deref());
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
