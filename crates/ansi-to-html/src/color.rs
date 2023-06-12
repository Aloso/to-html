use std::{fmt, num::ParseIntError};

use crate::{Error, FourBitColorType};

/// An ANSI color.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Color {
    FourBit(FourBitColor),
    EightBit(EightBitColor),
    Rgb(RgbColor),
}

impl Color {
    pub(crate) fn parse_4bit(code: u8) -> Result<Self, Error> {
        Ok(Color::FourBit(match code {
            0 => FourBitColor::Black,
            1 => FourBitColor::Red,
            2 => FourBitColor::Green,
            3 => FourBitColor::Yellow,
            4 => FourBitColor::Blue,
            5 => FourBitColor::Magenta,
            6 => FourBitColor::Cyan,
            7 => FourBitColor::White,
            _ => {
                return Err(Error::InvalidAnsi {
                    msg: format!("unexpected integer {} parsing 4-bit color", code),
                })
            }
        }))
    }

    pub(crate) fn parse_4bit_bright(code: u8) -> Result<Self, Error> {
        Ok(Color::FourBit(match code {
            0 => FourBitColor::BrightBlack,
            1 => FourBitColor::BrightRed,
            2 => FourBitColor::BrightGreen,
            3 => FourBitColor::BrightYellow,
            4 => FourBitColor::BrightBlue,
            5 => FourBitColor::BrightMagenta,
            6 => FourBitColor::BrightCyan,
            7 => FourBitColor::BrightWhite,
            _ => {
                return Err(Error::InvalidAnsi {
                    msg: format!("unexpected integer {} parsing bright 4-bit color", code),
                })
            }
        }))
    }

    pub(crate) fn parse_8bit_or_rgb<I>(mut iter: I) -> Result<Self, Error>
    where
        I: Iterator<Item = Result<u8, ParseIntError>>,
    {
        let code = iter
            .next()
            .transpose()?
            .ok_or_else(Error::invalid_ansi("Missing 2 or 5"))?;
        Ok(match code {
            5 => {
                let color = iter
                    .next()
                    .transpose()?
                    .ok_or_else(Error::invalid_ansi("Missing 8-bit color"))?;
                Color::EightBit(EightBitColor::new(color))
            }
            2 => {
                let r = iter.next().transpose()?;
                let g = iter.next().transpose()?;
                let b = iter.next().transpose()?;

                let r = r.ok_or_else(Error::invalid_ansi("Missing ANSI red"))?;
                let g = g.ok_or_else(Error::invalid_ansi("Missing ANSI green"))?;
                let b = b.ok_or_else(Error::invalid_ansi("Missing ANSI blue"))?;

                Color::Rgb(RgbColor { r, g, b })
            }
            _ => {
                return Err(Error::InvalidAnsi {
                    msg: format!("Expected 2 or 5, got {}", code),
                })
            }
        })
    }

    pub(crate) fn into_opening_fg_span(self, color_type: &FourBitColorType) -> String {
        self.into_opening_span(color_type, true)
    }

    pub(crate) fn into_opening_bg_span(self, color_type: &FourBitColorType) -> String {
        self.into_opening_span(color_type, false)
    }

    pub(crate) fn into_opening_span(self, color_type: &FourBitColorType, is_fg: bool) -> String {
        if let (Self::FourBit(four_bit), FourBitColorType::Class { prefix }) = (self, color_type) {
            let mut s = "<span class='".to_owned();
            if let Some(prefix) = prefix {
                s.push_str(prefix);
            }

            if is_fg {
                four_bit.write_fg_class(&mut s);
            } else {
                four_bit.write_bg_class(&mut s);
            }
            s.push_str("'>");
            s
        } else if is_fg {
            format!("<span style='color:{self}'>")
        } else {
            format!("<span style='background:{self}'>")
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Color::FourBit(color) => fmt::Display::fmt(&EightBitColor { code: color as u8 }, f),
            Color::EightBit(color) => fmt::Display::fmt(&color, f),
            Color::Rgb(RgbColor { r, g, b }) => write!(f, "#{:02x}{:02x}{:02x}", r, g, b),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub(crate) enum FourBitColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl FourBitColor {
    pub(crate) fn is_bright(self) -> bool {
        matches!(
            self,
            Self::BrightBlack
                | Self::BrightRed
                | Self::BrightGreen
                | Self::BrightYellow
                | Self::BrightBlue
                | Self::BrightMagenta
                | Self::BrightCyan
                | Self::BrightWhite,
        )
    }

    pub(crate) fn write_fg_class(self, s: &mut String) {
        if self.is_bright() {
            s.push_str("bright-");
        }

        s.push_str(match self {
            Self::Black | Self::BrightBlack => "black",
            Self::Red | Self::BrightRed => "red",
            Self::Green | Self::BrightGreen => "green",
            Self::Yellow | Self::BrightYellow => "yellow",
            Self::Blue | Self::BrightBlue => "blue",
            Self::Magenta | Self::BrightMagenta => "magenta",
            Self::Cyan | Self::BrightCyan => "cyan",
            Self::White | Self::BrightWhite => "white",
        });
    }

    pub(crate) fn write_bg_class(self, s: &mut String) {
        s.push_str("bg-");
        self.write_fg_class(s);
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct EightBitColor {
    code: u8,
}

impl EightBitColor {
    pub(crate) fn new(code: u8) -> Self {
        Self { code }
    }
}

impl fmt::Display for EightBitColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const COLORS: [&str; 256] = [
            "#000", "#a00", "#0a0", "#a60", "#00a", "#a0a", "#0aa", "#aaa", "#555", "#f55", "#5f5",
            "#ff5", "#55f", "#f5f", "#5ff", "#fff", "#000", "#00005f", "#000087", "#0000af",
            "#0000d7", "#00f", "#005f00", "#005f5f", "#005f87", "#005faf", "#005fd7", "#005fff",
            "#008700", "#00875f", "#008787", "#0087af", "#0087d7", "#0087ff", "#00af00", "#00af5f",
            "#00af87", "#00afaf", "#00afd7", "#00afff", "#00d700", "#00d75f", "#00d787", "#00d7af",
            "#00d7d7", "#00d7ff", "#0f0", "#00ff5f", "#00ff87", "#00ffaf", "#00ffd7", "#0ff",
            "#5f0000", "#5f005f", "#5f0087", "#5f00af", "#5f00d7", "#5f00ff", "#5f5f00", "#5f5f5f",
            "#5f5f87", "#5f5faf", "#5f5fd7", "#5f5fff", "#5f8700", "#5f875f", "#5f8787", "#5f87af",
            "#5f87d7", "#5f87ff", "#5faf00", "#5faf5f", "#5faf87", "#5fafaf", "#5fafd7", "#5fafff",
            "#5fd700", "#5fd75f", "#5fd787", "#5fd7af", "#5fd7d7", "#5fd7ff", "#5fff00", "#5fff5f",
            "#5fff87", "#5fffaf", "#5fffd7", "#5fffff", "#870000", "#87005f", "#870087", "#8700af",
            "#8700d7", "#8700ff", "#875f00", "#875f5f", "#875f87", "#875faf", "#875fd7", "#875fff",
            "#878700", "#87875f", "#878787", "#8787af", "#8787d7", "#8787ff", "#87af00", "#87af5f",
            "#87af87", "#87afaf", "#87afd7", "#87afff", "#87d700", "#87d75f", "#87d787", "#87d7af",
            "#87d7d7", "#87d7ff", "#87ff00", "#87ff5f", "#87ff87", "#87ffaf", "#87ffd7", "#87ffff",
            "#af0000", "#af005f", "#af0087", "#af00af", "#af00d7", "#af00ff", "#af5f00", "#af5f5f",
            "#af5f87", "#af5faf", "#af5fd7", "#af5fff", "#af8700", "#af875f", "#af8787", "#af87af",
            "#af87d7", "#af87ff", "#afaf00", "#afaf5f", "#afaf87", "#afafaf", "#afafd7", "#afafff",
            "#afd700", "#afd75f", "#afd787", "#afd7af", "#afd7d7", "#afd7ff", "#afff00", "#afff5f",
            "#afff87", "#afffaf", "#afffd7", "#afffff", "#d70000", "#d7005f", "#d70087", "#d700af",
            "#d700d7", "#d700ff", "#d75f00", "#d75f5f", "#d75f87", "#d75faf", "#d75fd7", "#d75fff",
            "#d78700", "#d7875f", "#d78787", "#d787af", "#d787d7", "#d787ff", "#d7af00", "#d7af5f",
            "#d7af87", "#d7afaf", "#d7afd7", "#d7afff", "#d7d700", "#d7d75f", "#d7d787", "#d7d7af",
            "#d7d7d7", "#d7d7ff", "#d7ff00", "#d7ff5f", "#d7ff87", "#d7ffaf", "#d7ffd7", "#d7ffff",
            "#f00", "#ff005f", "#ff0087", "#ff00af", "#ff00d7", "#f0f", "#ff5f00", "#ff5f5f",
            "#ff5f87", "#ff5faf", "#ff5fd7", "#ff5fff", "#ff8700", "#ff875f", "#ff8787", "#ff87af",
            "#ff87d7", "#ff87ff", "#ffaf00", "#ffaf5f", "#ffaf87", "#ffafaf", "#ffafd7", "#ffafff",
            "#ffd700", "#ffd75f", "#ffd787", "#ffd7af", "#ffd7d7", "#ffd7ff", "#ff0", "#ffff5f",
            "#ffff87", "#ffffaf", "#ffffd7", "#fff", "#080808", "#121212", "#1c1c1c", "#262626",
            "#303030", "#3a3a3a", "#444", "#4e4e4e", "#585858", "#626262", "#6c6c6c", "#767676",
            "#808080", "#8a8a8a", "#949494", "#9e9e9e", "#a8a8a8", "#b2b2b2", "#bcbcbc", "#c6c6c6",
            "#d0d0d0", "#dadada", "#e4e4e4", "#eee",
        ];
        f.write_str(COLORS[self.code as usize])
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct RgbColor {
    r: u8,
    g: u8,
    b: u8,
}
