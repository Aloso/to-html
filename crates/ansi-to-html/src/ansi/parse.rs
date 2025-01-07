use std::{ops::Range, str::CharIndices};

#[must_use]
pub(crate) struct AnsiParser<'text> {
    text: &'text str,
    chars: CharIndices<'text>,
}

impl<'text> AnsiParser<'text> {
    pub(crate) fn new(text: &'text str) -> Self {
        let chars = text.char_indices();
        Self { text, chars }
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
        let mut dfa = Dfa::new(&mut self.chars).unwrap();

        match dfa.next()? {
            // Walk through the current run of text
            Status::Waiting => {
                while dfa.peek() == Some(Status::Waiting) {
                    dfa.next();
                }
                Some(AnsiFragment::Text(&self.text[dfa.span()]))
            }
            // Walk through the ansi sequence
            Status::InSequence => loop {
                let Some(status) = dfa.peek() else {
                    break Some(AnsiFragment::Text(&self.text[dfa.span()]));
                };

                match status {
                    Status::Waiting => unreachable!("We're already past `Waiting`"),
                    Status::InSequence => _ = dfa.next(),
                    Status::Accept => {
                        dfa.next();
                        break Some(AnsiFragment::Sequence(&self.text[dfa.span()]));
                    }
                    // TODO(cosmic): niche case, but behavior is diverging from the regex here
                    // which would return invalid ansi codes _along with_ any surround text whereas
                    // we're emitting them separately atm
                    Status::RejectAsText => {
                        // Fortunately the starting ESC of an ANSI sequence can't appear within
                        // the sequence itself, so we don't need to worry about any backtracking or
                        // reparsing
                        break Some(AnsiFragment::Text(&self.text[dfa.span()]));
                    }
                }
            },
            _ => unreachable!("No other possible statuses after just one char"),
        }
    }
}

/// A small DFA that detects and emits ANSI codes from an underlying `CharIndices`
struct Dfa<'short, 'text: 'short> {
    chars: &'short mut CharIndices<'text>,
    state: State,
    start_idx: usize,
}

impl<'short, 'text> Dfa<'short, 'text> {
    fn new(chars: &'short mut CharIndices<'text>) -> Option<Self> {
        let state = State::default();
        let start_idx = chars.offset();
        Some(Self {
            chars,
            state,
            start_idx,
        })
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
    /// Waiting to see an escape
    #[default]
    Init,
    Trap,
    Accept,
    Escape,
    EscapeOpenParen,
    /// Control Sequence Introducer (CSI) - Indicates the start of an ansi sequence
    Csi,
    Digit,
    SemiColon,
}

impl State {
    fn munch(&mut self, c: char) {
        *self = match (*self, c) {
            // Waiting for ESC
            (Self::Init, '\u{1b}') => Self::Escape,
            (Self::Init, _) => Self::Init,
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
    Waiting,
    InSequence,
    Accept,
    RejectAsText,
}

impl From<State> for Status {
    fn from(state: State) -> Self {
        match state {
            State::Init => Self::Waiting,
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
