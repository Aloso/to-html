use std::fmt::Write;

use crate::{color::FourBitColor, Ansi, AnsiFragment, AnsiIter, AnsiParser, Color, Error, Theme};

mod minifier;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Style {
    Bold,
    Faint,
    Italic,
    Underline(UnderlineStyle),
    CrossedOut,
    ForegroundColor(Color),
    BackgroundColor(Color),
    Inverted,
}

impl Style {
    fn apply(&self, buf: &mut String, var_prefix: Option<&str>, styles: &[Style], theme: Theme) {
        let str = match self {
            Style::Bold => "<b>",
            Style::Faint => "<span style='opacity:0.67'>",
            Style::Italic => "<i>",
            Style::Underline(UnderlineStyle::Default) => "<u>",
            Style::Underline(UnderlineStyle::Double) => "<u style='text-decoration-style:double'>",
            Style::CrossedOut => "<s>",
            Style::ForegroundColor(c) => {
                let color = c.into_color_css(var_prefix);
                let inverted = styles.contains(&Style::Inverted);
                let property = Self::get_property(!inverted);
                let _ = buf.write_fmt(format_args!("<span style='{property}:{color}'>"));
                return;
            }
            Style::BackgroundColor(c) => {
                let color = c.into_color_css(var_prefix);
                let inverted = styles.contains(&Style::Inverted);
                let property = Self::get_property(inverted);
                let _ = buf.write_fmt(format_args!("<span style='{property}:{color}'>"));
                return;
            }
            Style::Inverted => {
                let (fg, bg) = Self::get_fg_and_bg(styles, theme);
                let fg = fg.into_color_css(var_prefix);
                let bg = bg.into_color_css(var_prefix);
                let _ = buf.write_fmt(format_args!("<span style='color:{fg};background:{bg}'>"));
                return;
            }
        };
        buf.push_str(str);
    }

    fn get_property(is_foreground: bool) -> &'static str {
        if is_foreground {
            "color"
        } else {
            "background"
        }
    }

    fn get_fg_and_bg(styles: &[Style], theme: Theme) -> (Color, Color) {
        let mut fg = None;
        let mut bg = None;
        for style in styles.iter().rev() {
            match style {
                Style::ForegroundColor(fg) => bg = Some(*fg),
                Style::BackgroundColor(bg) => fg = Some(*bg),
                _ => {}
            }
            if let (Some(_), Some(_)) = (fg, bg) {
                break;
            }
        }

        // Default inverted fg/bg if missing
        let white = Color::FourBit(FourBitColor::BrightWhite);
        let black = Color::FourBit(FourBitColor::Black);
        let dark_theme = theme == Theme::Dark;

        let fg = fg.unwrap_or(if dark_theme { black } else { white });
        let bg = bg.unwrap_or(if dark_theme { white } else { black });
        (fg, bg)
    }

    fn clear(&self, buf: &mut String) {
        buf.push_str(match self {
            Style::Bold => "</b>",
            Style::Italic => "</i>",
            Style::Underline(_) => "</u>",
            Style::CrossedOut => "</s>",
            Style::Faint
            | Style::ForegroundColor(_)
            | Style::BackgroundColor(_)
            | Style::Inverted => "</span>",
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum UnderlineStyle {
    Default,
    Double,
}

/// Convert ANSI sequences to html. This does NOT escape html characters such as `<` and `&`.
pub fn ansi_to_html(
    input: &str,
    four_bit_var_prefix: Option<String>,
    theme: Theme,
    skip_optimize: bool,
) -> Result<String, Error> {
    let mut ansi_sink: Box<dyn AnsiSink> = if skip_optimize {
        Box::new(AnsiConverter::new(four_bit_var_prefix, theme))
    } else {
        Box::new(minifier::Minifier::new(four_bit_var_prefix, theme))
    };

    for fragment in AnsiParser::new(input) {
        match fragment {
            AnsiFragment::Sequence(ansi_codes) => {
                if !ansi_codes.ends_with('m') {
                    continue;
                }

                let len = ansi_codes.len();
                if len == 3 {
                    ansi_sink.clear_styles();
                    continue;
                }

                let nums = &ansi_codes[2..len - 1];
                let norm_nums = nums.strip_suffix(';').unwrap_or(nums);
                let norm_nums = norm_nums.split(';').map(|n| n.parse::<u8>());

                for ansi in AnsiIter::new(norm_nums) {
                    ansi_sink.push_ansi_code(ansi?);
                }
            }
            AnsiFragment::Text(text) => ansi_sink.push_str(text),
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
    theme: Theme,
}

impl AnsiConverter {
    fn new(four_bit_var_prefix: Option<String>, theme: Theme) -> Self {
        Self {
            four_bit_var_prefix,
            theme,
            ..Self::default()
        }
    }

    fn set_style(&mut self, s: Style) {
        let var_prefix = self.four_bit_var_prefix.as_deref();
        s.apply(&mut self.result, var_prefix, &self.styles, self.theme);
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
            let var_prefix = self.four_bit_var_prefix.as_deref();
            style.apply(&mut self.result, var_prefix, &self.styles, self.theme);
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
            Ansi::Underline => self.set_style(Style::Underline(UnderlineStyle::Default)),
            Ansi::Invert => self.set_style(Style::Inverted),
            Ansi::DoubleUnderline => self.set_style(Style::Underline(UnderlineStyle::Double)),
            Ansi::CrossedOut => self.set_style(Style::CrossedOut),
            Ansi::BoldAndFaintOff => self.clear_style(|&s| s == Style::Bold || s == Style::Faint),
            Ansi::ItalicOff => self.clear_style(|&s| s == Style::Italic),
            Ansi::UnderlineOff => self.clear_style(|&s| matches!(s, Style::Underline(_))),
            Ansi::InvertOff => self.clear_style(|&s| s == Style::Inverted),
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
