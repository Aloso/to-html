const ESCAPE: u8 = 0x1b;

#[must_use]
pub(crate) struct AnsiParser<'text> {
    text: &'text str,
    index: usize,
}

impl<'text> AnsiParser<'text> {
    pub(crate) fn new(text: &'text str) -> Self {
        let index = 0;
        Self { text, index }
    }

    fn current_byte(&self) -> Option<u8> {
        self.text.as_bytes().get(self.index).copied()
    }

    fn next_byte(&mut self) -> Option<u8> {
        self.inc();
        self.current_byte()
    }

    fn inc(&mut self) {
        self.index += 1;
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum AnsiFragment<'text> {
    Sequence(&'text str),
    Text(&'text str),
}

impl<'text> Iterator for AnsiParser<'text> {
    type Item = AnsiFragment<'text>;

    fn next(&mut self) -> Option<Self::Item> {
        let start_idx = self.index;
        // All ANSI codes start with ESCAPE, so check if we're parsing an ANSI code or plain text
        // with this iteration
        if self.current_byte()? == ESCAPE {
            let mut state = State::default();
            loop {
                let Some(b) = self.next_byte() else {
                    break Some(AnsiFragment::Text(&self.text[start_idx..self.index]));
                };

                state.munch(b);
                match state.into() {
                    Status::InSequence => {}
                    Status::Accept => {
                        self.inc();
                        break Some(AnsiFragment::Sequence(&self.text[start_idx..self.index]));
                    }
                    // NOTE(cosmic): niche case, but behavior is diverging from the regex here
                    // which would return invalid ansi codes _along with_ any surround text whereas
                    // we're emitting them separately atm
                    Status::RejectAsText => {
                        // Fortunately the starting ESC of an ANSI sequence can't appear within
                        // the sequence itself, so we don't need to worry about any backtracking or
                        // reparsing
                        break Some(AnsiFragment::Text(&self.text[start_idx..self.index]));
                    }
                }
            }
        } else {
            // Increment past the byte we just checked
            self.inc();
            // Find the next ESCAPE if there is one and adjust our index accordingly
            match memchr::memchr(ESCAPE, &self.text.as_bytes()[start_idx..]) {
                Some(end_offset) => self.index += end_offset - 1,
                None => self.index = self.text.len(),
            };
            Some(AnsiFragment::Text(&self.text[start_idx..self.index]))
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
    /// Operating System Command (OSC) - Starts a OSC command ended by a string terminator (ST)
    InOsc,
    StartSt,
}

impl State {
    fn munch(&mut self, b: u8) {
        *self = match (*self, b) {
            // Weird `<ESC>(B` ansi code
            (Self::Escape, b'(') => Self::EscapeOpenParen,
            (Self::EscapeOpenParen, b'B') => Self::Accept,
            // CSI-related codes
            (Self::Escape, b'[') => Self::Csi,
            (Self::Csi | Self::Digit | Self::SemiColon, b'0'..=b'9') => Self::Digit,
            (Self::Digit, b';') => Self::SemiColon,
            (
                Self::Csi | Self::Digit | Self::SemiColon,
                b'A'..=b'H'
                | b'J'..=b'K'
                | b'S'..=b'T'
                | b'f'
                | b'h'..=b'i'
                | b'l'..=b'n'
                | b's'
                | b'u',
            ) => Self::Accept,
            // OSC sequence
            (Self::Escape, b']') => Self::InOsc,
            // OSC sequence ends on a string terminator consisting of either the lone BEL byte
            // (0x07) or ESC followed by `\` (0x1b5c)
            (Self::InOsc | Self::StartSt, 0x07) => Self::Accept,
            (Self::InOsc | Self::StartSt, 0x1b) => Self::StartSt,
            (Self::StartSt, b'\\') => Self::Accept,
            (Self::InOsc | Self::StartSt, _) => Self::InOsc,
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
            | State::SemiColon
            | State::InOsc
            | State::StartSt => Self::InSequence,
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

    #[test]
    fn basic_osc() {
        let parser = AnsiParser::new("Before\x1b]8;;https://example.com\x07After");
        let fragments: Vec<_> = parser.into_iter().collect();
        insta::assert_debug_snapshot!(fragments, @r#"
        [
            Text(
                "Before",
            ),
            Sequence(
                "\u{1b}]8;;https://example.com\u{7}",
            ),
            Text(
                "After",
            ),
        ]
        "#);
    }

    #[track_caller]
    fn assert_lone_sequence(seq: &str) {
        let fragments: Vec<_> = AnsiParser::new(seq).into_iter().collect();
        assert_eq!(fragments, [AnsiFragment::Sequence(seq)]);
    }

    #[test]
    fn osc_st_variety() {
        assert_lone_sequence("\x1b]0;custom window title\x07");
        assert_lone_sequence("\x1b]8;;https://example.com\x1b\\");
    }

    #[test]
    fn osc_st_edgecases() {
        assert_lone_sequence("\x1b]\x1b <-- esc not for st \x07");
        assert_lone_sequence("\x1b] esc not for st right before bel st --> \x1b\x07");
        assert_lone_sequence("\x1b]\x1b <-- both esc not for st --> \x1b\x07");
        assert_lone_sequence("\x1b] st start before valid st \x1b\x1b\\");
    }

    #[test]
    fn osc_no_st() {
        let input = "\x1b] osc with no string terminator";
        let fragments: Vec<_> = AnsiParser::new(input).into_iter().collect();
        assert_eq!(fragments, [AnsiFragment::Text(input)]);
    }
}
