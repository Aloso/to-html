use std::str::CharIndices;

const ESCAPE: char = '\u{1b}';

#[must_use]
pub(crate) struct AnsiParser<'text> {
    text: &'text str,
    chars: CharIndices<'text>,
    /// Potentially stores the character that broke us out of the last `.next()` call
    ///
    /// In the case of an invalid/unrecognized ANSI code or when iterating over plain text portions
    /// the character that breaks us out of the current token that we're parsing (e.g. when we see
    /// an escape while parsing plain text) will be stored in this field and taken into account on
    /// the start of the next iteration. This removes the need to peek each character to avoid
    /// advancing the iterator by retaining the character for next iteration instead
    ended_last_iter_on: Option<char>,
}

impl<'text> AnsiParser<'text> {
    pub(crate) fn new(text: &'text str) -> Self {
        let chars = text.char_indices();
        let ended_last_iter_on = None;
        Self {
            text,
            chars,
            ended_last_iter_on,
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
        let ended_last_iter_on = self.ended_last_iter_on.take();
        let start_idx = self.chars.offset() - ended_last_iter_on.map(char::len_utf8).unwrap_or(0);
        let c = ended_last_iter_on.or_else(|| self.chars.next().map(|(_, c)| c))?;
        // All ANSI codes start with ESCAPE, so check if we're parsing an ANSI code or plain text
        // with this iteration
        if c == ESCAPE {
            let mut state = State::default();
            loop {
                let Some((_, c)) = self.chars.next() else {
                    break Some(AnsiFragment::Text(
                        &self.text[start_idx..self.chars.offset()],
                    ));
                };

                state.munch(c);
                match state.into() {
                    Status::InSequence => {}
                    Status::Accept => {
                        break Some(AnsiFragment::Sequence(
                            &self.text[start_idx..self.chars.offset()],
                        ));
                    }
                    // NOTE(cosmic): niche case, but behavior is diverging from the regex here
                    // which would return invalid ansi codes _along with_ any surround text whereas
                    // we're emitting them separately atm
                    Status::RejectAsText => {
                        self.ended_last_iter_on = Some(c);
                        // Fortunately the starting ESC of an ANSI sequence can't appear within
                        // the sequence itself, so we don't need to worry about any backtracking or
                        // reparsing
                        break Some(AnsiFragment::Text(
                            &self.text[start_idx..self.chars.offset() - c.len_utf8()],
                        ));
                    }
                }
            }
        } else {
            while let Some((_, c)) = self.chars.next() {
                if c == ESCAPE {
                    self.ended_last_iter_on = Some(c);
                    break;
                }
            }
            let end_offset = self.ended_last_iter_on.map(char::len_utf8).unwrap_or(0);
            let end_idx = self.chars.offset() - end_offset;
            Some(AnsiFragment::Text(&self.text[start_idx..end_idx]))
        }
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
