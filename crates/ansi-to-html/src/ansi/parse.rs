use std::{mem, ops::Range, str::CharIndices};

const ESCAPE: char = '\u{1b}';

#[must_use]
pub(crate) struct AnsiParser<'text> {
    text: &'text str,
    chars: CharIndices<'text>,
    /// Stores whether the last iteration of `.next()` ended on an escape character
    ///
    /// The parser sifts through text to find ANSI codes which **always** start with an escape
    /// character. We can exploit this to speed up the hot path of iterating over plain text by
    /// unconditionally iterating over characters and storing whether the character that broke us
    /// out was an escape in this field. This avoids the need to peek each character to see if it's
    /// the start of an ANSI code by re-framing the ansi code parsing to start _after_ the intial
    /// escape and adjusting the starting index to still include it
    was_on_esc: bool,
}

impl<'text> AnsiParser<'text> {
    pub(crate) fn new(text: &'text str) -> Self {
        let chars = text.char_indices();
        let was_on_esc = false;
        Self {
            text,
            chars,
            was_on_esc,
        }
    }

    fn parse_ansi_code_after_esc(&mut self) -> Option<AnsiFragment<'text>> {
        let mut fsm = AnsiFsm::new(&mut self.chars);
        loop {
            let Some(status) = fsm.peek() else {
                break Some(AnsiFragment::Text(&self.text[fsm.span()]));
            };

            match status {
                Status::InSequence => _ = fsm.next(),
                Status::Accept => {
                    fsm.next();
                    break Some(AnsiFragment::Sequence(&self.text[fsm.span()]));
                }
                // NOTE(cosmic): niche case, but behavior is diverging from the regex here
                // which would return invalid ansi codes _along with_ any surround text whereas
                // we're emitting them separately atm
                Status::RejectAsText => {
                    // Fortunately the starting ESC of an ANSI sequence can't appear within
                    // the sequence itself, so we don't need to worry about any backtracking or
                    // reparsing
                    break Some(AnsiFragment::Text(&self.text[fsm.span()]));
                }
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum AnsiFragment<'text> {
    Sequence(&'text str),
    Text(&'text str),
}

impl<'text> Iterator for AnsiParser<'text> {
    type Item = AnsiFragment<'text>;

    fn next(&mut self) -> Option<Self::Item> {
        if mem::take(&mut self.was_on_esc) {
            self.parse_ansi_code_after_esc()
        } else {
            let start_idx = self.chars.offset();
            let (_, c) = self.chars.next()?;
            if c == ESCAPE {
                return self.parse_ansi_code_after_esc();
            }
            while let Some((_, c)) = self.chars.next() {
                if c == ESCAPE {
                    self.was_on_esc = true;
                    break;
                }
            }
            let end_offset = if self.was_on_esc {
                ESCAPE.len_utf8()
            } else {
                0
            };
            let end_idx = self.chars.offset() - end_offset;
            Some(AnsiFragment::Text(&self.text[start_idx..end_idx]))
        }
    }
}

/// A small finite state machine that parses ANSI codes after the initial ESC
struct AnsiFsm<'short, 'text: 'short> {
    chars: &'short mut CharIndices<'text>,
    state: State,
    start_idx: usize,
}

impl<'short, 'text> AnsiFsm<'short, 'text> {
    fn new(chars: &'short mut CharIndices<'text>) -> Self {
        let state = State::default();
        let start_idx = chars.offset() - ESCAPE.len_utf8();
        Self {
            chars,
            state,
            start_idx,
        }
    }

    fn peek(&self) -> Option<Status> {
        let mut chars_lookahead = self.chars.to_owned();
        let mut state_lookahead = self.state.to_owned();
        let (_, c) = chars_lookahead.next()?;
        state_lookahead.munch(c);
        Some(state_lookahead.into())
    }

    fn next(&mut self) -> Option<Status> {
        let (_, c) = self.chars.next()?;
        self.state.munch(c);
        Some(self.state.into())
    }

    fn span(self) -> Range<usize> {
        self.start_idx..self.chars.offset()
    }
}

#[derive(Clone, Copy, Default)]
enum State {
    #[default]
    Escape,
    Trap,
    Accept,
    EscapeOpenParen,
    /// Control Sequence Introducer (CSI) - Indicates the start of an ansi sequence
    Csi,
    Digit,
    SemiColon,
}

impl State {
    fn munch(&mut self, c: char) {
        *self = match (*self, c) {
            // Weird `<ESC>(B` ansi code
            (Self::Escape, '(') => Self::EscapeOpenParen,
            (Self::EscapeOpenParen, 'B') => Self::Accept,
            // CSI-related codes
            (Self::Escape, '[') => Self::Csi,
            (Self::Csi | Self::Digit | Self::SemiColon, '0'..='9') => Self::Digit,
            (Self::Digit, ';') => Self::SemiColon,
            (
                Self::Csi | Self::Digit | Self::SemiColon,
                'A'..='H' | 'J' | 'K' | 'S' | 'T' | 'f' | 'h' | 'i' | 'l' | 'm' | 'n' | 's' | 'u',
            ) => Self::Accept,
            // Anything else is invalid
            _ => Self::Trap,
        };
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    InSequence,
    Accept,
    RejectAsText,
}

impl From<State> for Status {
    fn from(state: State) -> Self {
        match state {
            State::Trap => Self::RejectAsText,
            State::Accept => Self::Accept,
            State::Escape
            | State::EscapeOpenParen
            | State::Csi
            | State::Digit
            | State::SemiColon => Self::InSequence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain() {
        let parser = AnsiParser::new("Hello World!");
        let fragments: Vec<_> = parser.into_iter().collect();
        insta::assert_debug_snapshot!(fragments, @r#"
        [
            Text(
                "Hello World!",
            ),
        ]
        "#);
    }

    #[test]
    fn variety() {
        let parser = AnsiParser::new("\u{1b}(BHello \u{1b}[4m\u{1b}[1;21mWorld!\u{1b}[0;m");
        let fragments: Vec<_> = parser.into_iter().collect();
        insta::assert_debug_snapshot!(fragments, @r###"
        [
            Sequence(
                "\u{1b}(B",
            ),
            Text(
                "Hello ",
            ),
            Sequence(
                "\u{1b}[4m",
            ),
            Sequence(
                "\u{1b}[1;21m",
            ),
            Text(
                "World!",
            ),
            Sequence(
                "\u{1b}[0;m",
            ),
        ]
        "###);
    }

    #[test]
    fn invalid_right_before_valid() {
        let parser = AnsiParser::new("Before\u{1b}[4;\u{1b}[mAfter");
        let fragments: Vec<_> = parser.into_iter().collect();
        insta::assert_debug_snapshot!(fragments, @r###"
        [
            Text(
                "Before",
            ),
            Text(
                "\u{1b}[4;",
            ),
            Sequence(
                "\u{1b}[m",
            ),
            Text(
                "After",
            ),
        ]
        "###);
    }
}
