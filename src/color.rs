use std::{error::Error, fmt, num::ParseIntError};

/// An ANSI color.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Color {
    FourBit(FourBitColor),
    EightBit(EightBitColor),
    Rgb(RgbColor),
}

impl Color {
    pub(crate) fn parse_4bit(code: u8) -> Result<Self, Box<dyn Error>> {
        Ok(Color::FourBit(match code {
            0 => FourBitColor::Black,
            1 => FourBitColor::Red,
            2 => FourBitColor::Green,
            3 => FourBitColor::Yellow,
            4 => FourBitColor::Blue,
            5 => FourBitColor::Magenta,
            6 => FourBitColor::Cyan,
            7 => FourBitColor::White,
            _ => unreachable!("4 bit colors only"),
        }))
    }

    pub(crate) fn parse_4bit_bright(code: u8) -> Result<Self, Box<dyn Error>> {
        Ok(Color::FourBit(match code {
            0 => FourBitColor::BrightBlack,
            1 => FourBitColor::BrightRed,
            2 => FourBitColor::BrightGreen,
            3 => FourBitColor::BrightYellow,
            4 => FourBitColor::BrightBlue,
            5 => FourBitColor::BrightMagenta,
            6 => FourBitColor::BrightCyan,
            7 => FourBitColor::BrightWhite,
            _ => unreachable!("4 bit colors only"),
        }))
    }

    pub(crate) fn parse_better<I>(mut iter: I) -> Result<Self, Box<dyn Error>>
    where
        I: Iterator<Item = Result<u8, ParseIntError>>,
    {
        let code = iter.next().transpose()?.ok_or("Missing ANSI code")?;
        Ok(match code {
            5 => {
                let color = iter.next().transpose()?.ok_or("Missing ANSI 8-bit color")?;
                Color::EightBit(EightBitColor::new(color))
            }
            2 => {
                let r = iter.next().transpose()?.ok_or("Missing ANSI red")?;
                let g = iter.next().transpose()?.ok_or("Missing ANSI green")?;
                let b = iter.next().transpose()?.ok_or("Missing ANSI blue")?;
                Color::Rgb(RgbColor { r, g, b })
            }
            _ => return Err("Invalid ANSI color code".into()),
        })
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
        if self.code < 16 {
            const COLORS: [&str; 16] = [
                "#000", "#a00", "#0a0", "#a50", "#00a", "#a0a", "#0aa", "#aaa", "#555", "#f55",
                "#5f5", "#ff5", "#55f", "#f5f", "#5ff", "#fff",
            ];
            f.write_str(COLORS[self.code as usize])
        } else {
            todo!()
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct RgbColor {
    r: u8,
    g: u8,
    b: u8,
}
