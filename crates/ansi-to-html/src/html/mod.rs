use regex::Regex;

use crate::{Ansi, AnsiIter, Color, Error};

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
    skip_optimize: bool,
) -> Result<String, Error> {
    let mut ansi_sink: Box<dyn AnsiSink> = if skip_optimize {
        Box::new(AnsiConverter::new(four_bit_var_prefix))
    } else {
        Box::new(minifier::Minifier::new(four_bit_var_prefix))
    };

    loop {
        match ansi_regex.find(input) {
            Some(m) => {
                if m.start() > 0 {
                    let (before, after) = input.split_at(m.start());
                    ansi_sink.push_str(before);
                    input = after;
                }

                let len = m.range().len();
                input = &input[len..];

                if !m.as_str().ends_with('m') {
                    continue;
                }

                if len == 3 {
                    ansi_sink.clear_styles();
                    continue;
                }

                let nums = &m.as_str()[2..len - 1];
                let norm_nums = nums.strip_suffix(';').unwrap_or(nums);
                let norm_nums = norm_nums.split(';').map(|n| n.parse::<u8>());

                for ansi in AnsiIter::new(norm_nums) {
                    ansi_sink.push_ansi_code(ansi?);
                }
            }
            None => {
                ansi_sink.push_str(input);
                break;
            }
        }
    }
    ansi_sink.push_ansi_code(Ansi::Reset); // make sure all tags are closed

    Ok(ansi_sink.to_html())
}

trait AnsiSink {
    fn clear_styles(&mut self);
    fn push_ansi_code(&mut self, ansi: Ansi);
    fn push_str(&mut self, text: &str);
    fn to_html(&mut self) -> String;
}

#[derive(Debug, Default)]
struct AnsiConverter {
    styles: Vec<Style>,
    styles_to_apply: Vec<Style>,
    result: String,
    four_bit_var_prefix: Option<String>,
}

impl AnsiConverter {
    fn new(four_bit_var_prefix: Option<String>) -> Self {
        Self {
            four_bit_var_prefix,
            ..Self::default()
        }
    }

    fn set_style(&mut self, s: Style) {
        s.apply(&mut self.result, self.four_bit_var_prefix.as_deref());
        self.styles.push(s);
    }

    fn clear_style(&mut self, cond: impl Fn(&Style) -> bool) {
        let Some((i, _)) = self.styles.iter().enumerate().find(|&(_, s)| cond(s)) else {
            return;
        };
        // Unwind the stack of styles past the style being cleared
        for style in self.styles.drain(i..).rev() {
            style.clear(&mut self.result);
            if !cond(&style) {
                self.styles_to_apply.push(style);
            }
        }
        // Re-wind back styles that are still set
        for style in self.styles_to_apply.drain(..).rev() {
            style.apply(&mut self.result, self.four_bit_var_prefix.as_deref());
            self.styles.push(style);
        }
    }
}

impl AnsiSink for AnsiConverter {
    fn clear_styles(&mut self) {
        self.clear_style(|_| true);
    }

    fn push_ansi_code(&mut self, ansi: Ansi) {
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

    fn push_str(&mut self, text: &str) {
        self.result.push_str(text);
    }

    fn to_html(&mut self) -> String {
        self.result.clone()
    }
}
