use ansi_to_html::{convert, Converter};
use html5ever::{
    local_name, tendril,
    tokenizer::{
        BufferQueue, Tag, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerResult,
    },
    Attribute, QualName,
};

use std::{cell::RefCell, collections::BTreeSet, mem, str::FromStr};

/// Ensures that our optimized HTML output is semantically equivalent to the unoptimized output
/// (along with verifying some other properties like our HTML being reasonably well-formed)
pub fn assert_opt_equiv_to_no_opt(ansi_text: &str) {
    let Ok(htmlified) = Converter::new().skip_optimize(true).convert(ansi_text) else {
        return;
    };
    let full_text = normalize_output(interpret_html(&htmlified));
    let opt_text = normalize_output(interpret_html(&convert(ansi_text).unwrap()));

    assert_eq!(
        full_text, opt_text,
        "Text should be semantically equivalent\nNo-Opt: {full_text:#?}\nOpt: {opt_text:#?}"
    );
}

fn normalize_output(texts: Vec<StylizedText>) -> Vec<StylizedText> {
    texts
        .into_iter()
        // Filter out any empty spans of text
        .filter(|t| !t.text.is_empty())
        .fold(Vec::new(), |mut acc, text| {
            match acc.last_mut() {
                // Coalesce text with consecutive runs of the same style
                Some(top) if top.styles == text.styles => top.text.push_str(&text.text),
                _ => acc.push(text),
            }
            acc
        })
}

/// Convert HTML to runs of stylized text
pub fn interpret_html(text: &str) -> Vec<StylizedText> {
    let tokenizer = Tokenizer::new(HtmlInterpreter::default(), Default::default());
    let queue = BufferQueue::default();
    let text = tendril::Tendril::from_str(text).unwrap();
    queue.push_back(text);
    let res = tokenizer.feed(&queue);
    assert!(matches!(res, TokenizerResult::Done));
    tokenizer.end();
    tokenizer.sink.finish()
}

#[derive(Default)]
struct HtmlInterpreter(RefCell<Inner>);

impl HtmlInterpreter {
    fn finish(self) -> Vec<StylizedText> {
        let mut inner = self.0.into_inner();
        inner.emit_pending_text();
        assert!(inner.state.raw_styles.is_empty(), "Start tags with no end");
        inner.output
    }
}

impl TokenSink for HtmlInterpreter {
    type Handle = ();

    fn process_token(&self, token: Token, _line: u64) -> TokenSinkResult<Self::Handle> {
        match token {
            Token::TagToken(tag) => {
                let mut interpreter = self.0.borrow_mut();
                // Handle the new tag
                let (raw_style, tag_kind) = RawStyle::new(&tag);
                match tag_kind {
                    TagKind::StartTag => interpreter.apply_start_style(raw_style),
                    TagKind::EndTag => interpreter.apply_end_style(raw_style),
                }
            }
            Token::CharacterTokens(s) => {
                let mut interpreter = self.0.borrow_mut();
                interpreter.push_pending_text(&s);
            }
            Token::NullCharacterToken | Token::ParseError(_) | Token::EOFToken => {}
            unknown => panic!("Missing implementation for {unknown:#?}"),
        }

        TokenSinkResult::Continue
    }
}

#[derive(Default)]
struct Inner {
    state: State,
    output: Vec<StylizedText>,
}

impl Inner {
    fn apply_start_style(&mut self, style: RawStyle) {
        self.emit_pending_text();

        if let Some(last_last) = self.state.pending_style.replace(style) {
            self.state.raw_styles.push(last_last);
        }
    }

    fn apply_end_style(&mut self, style: RawStyle) {
        self.emit_pending_text();

        let mut start_tag = match self.state.pending_style.take() {
            // We just pushed an opening tag without pushing text, so this is an empty span
            Some(style) => {
                let mut styles = Styles::new(&self.state.raw_styles);
                styles = styles.apply(style.clone());
                self.output.push(StylizedText::empty(styles));
                style
            }
            None => self.state.raw_styles.pop().unwrap(),
        };

        // The end tag won't have any of the attrs that the start had
        if let RawStyle::Span(attrs) | RawStyle::Underlined(attrs) = &mut start_tag {
            let _ = mem::take(attrs);
        }

        assert_eq!(start_tag, style, "Start tag should have a matching end");
    }

    fn push_pending_text(&mut self, s: &str) {
        if let Some(pending) = self.state.pending_style.take() {
            self.state.raw_styles.push(pending);
        }

        let pending_text = self.state.pending_text.get_or_insert_default();
        pending_text.push_str(s);
    }

    fn emit_pending_text(&mut self) {
        // `pending_style` gets cleared out when text gets pushed, so it being `Some` means there's
        // no pending text
        if self.state.pending_style.is_some() {
            return;
        }

        if let Some(text) = self.state.pending_text.take() {
            let styles = Styles::new(&self.state.raw_styles);
            let stylized_text = StylizedText::new(styles, text);
            self.output.push(stylized_text);
        }
    }
}

#[derive(Default)]
struct State {
    /// The style that was just pushed on before any text
    ///
    /// We buffer the styles that we push, so that we can identify and retain empty pairs of
    /// elements as empty text
    pending_style: Option<RawStyle>,
    /// The stack of styles that are currently active except for `pending_style`
    raw_styles: Vec<RawStyle>,
    /// The current run of text
    pending_text: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum RawStyle {
    Bold,
    Italic,
    Underlined(Vec<Attr>),
    CrossedOut,
    Span(Vec<Attr>),
}

impl RawStyle {
    /// Convert a generic HTML tag to one our known styles that we emit, panicking otherwise
    fn new(tag: &Tag) -> (Self, TagKind) {
        let Tag {
            kind,
            name,
            self_closing,
            attrs,
        } = tag;
        assert!(!self_closing, "Unexpected self-closing tag");

        let raw_style = match name {
            &local_name!("b") => Self::Bold,
            &local_name!("i") => Self::Italic,
            &local_name!("u") => Self::Underlined(attrs.iter().map(Attr::new).collect()),
            &local_name!("s") => Self::CrossedOut,
            &local_name!("span") => Self::Span(attrs.iter().map(Attr::new).collect()),
            unknown => panic!("Unexpected HTML tag kind: {unknown}"),
        };

        if !matches!(raw_style, Self::Span(_) | Self::Underlined(_)) {
            assert!(attrs.is_empty(), "Unexpected attrs for tag: {tag:#?}");
        }

        (raw_style, *kind)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct StylizedText {
    pub styles: Styles,
    pub text: String,
}

impl StylizedText {
    fn new(styles: Styles, text: String) -> Self {
        Self { styles, text }
    }

    fn empty(styles: Styles) -> Self {
        let text = String::new();
        Self { styles, text }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Styles {
    bold: bool,
    italic: bool,
    underlined: Option<UnderlineStyle>,
    crossed_out: bool,
    spans: BTreeSet<Vec<Attr>>,
}

impl Styles {
    fn new(raw_styles: &[RawStyle]) -> Self {
        raw_styles
            .iter()
            .cloned()
            .fold(Styles::default(), |styles, s| styles.apply(s))
    }

    #[must_use]
    fn apply(mut self, raw_style: RawStyle) -> Self {
        match raw_style {
            RawStyle::Bold => self.bold = true,
            RawStyle::Italic => self.italic = true,
            RawStyle::Underlined(mut attrs) => {
                assert!(
                    attrs.len() <= 1,
                    "Unexpected number of attrs for <u> {attrs:#?}"
                );
                self.underlined = Some(attrs.pop().into());
            }
            RawStyle::CrossedOut => self.crossed_out = true,
            RawStyle::Span(span) => _ = self.spans.insert(span),
        }
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Attr {
    name: String,
    value: String,
}

impl Attr {
    fn new(attr: &Attribute) -> Self {
        let Attribute {
            name: QualName { prefix, ns, local },
            value,
        } = attr;
        assert!(prefix.is_none());
        assert_eq!(ns, "");

        let name = local.to_string();
        let value = value.to_string();
        Self { name, value }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnderlineStyle {
    Default,
    Double,
}

impl From<Option<Attr>> for UnderlineStyle {
    fn from(maybe_attr: Option<Attr>) -> Self {
        match maybe_attr {
            None => Self::Default,
            Some(attr) if attr.name == "style" && attr.value == "text-decoration-style:double" => {
                Self::Double
            }
            Some(unknown) => panic!("Unknown attr for <u>: {unknown:#?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        let ansi_text = "\x1b[1mBold\x1b[31mRed and Bold";
        let htmlified = ansi_to_html::convert(ansi_text).unwrap();
        insta::assert_debug_snapshot!(interpret_html(&htmlified), @r#"
        [
            StylizedText {
                styles: Styles {
                    bold: true,
                    italic: false,
                    underlined: None,
                    crossed_out: false,
                    spans: {},
                },
                text: "Bold",
            },
            StylizedText {
                styles: Styles {
                    bold: true,
                    italic: false,
                    underlined: None,
                    crossed_out: false,
                    spans: {
                        [
                            Attr {
                                name: "style",
                                value: "color:var(--red,#a00)",
                            },
                        ],
                    },
                },
                text: "Red and Bold",
            },
        ]
        "#);
    }

    #[test]
    fn input_blue_red_text_red_text() {
        // Input: blue -> red -> "Red" -> red -> " Still Red"
        let ansi_text = "\x1b[34;31mRed\x1b[31m Still Red";
        assert_opt_equiv_to_no_opt(ansi_text);
        let htmlified = ansi_to_html::convert(ansi_text).unwrap();
        insta::assert_snapshot!(
            htmlified,
            @"<span style='color:var(--red,#a00)'>Red Still Red</span>"
        );
    }

    #[test]
    fn input_red_text_blue_red_text() {
        // Input: red -> "Red" -> blue -> red -> " Still Red"
        let ansi_text = "\x1b[31mRed\x1b[34;31m Still Red";
        assert_opt_equiv_to_no_opt(ansi_text);
        let htmlified = ansi_to_html::convert(ansi_text).unwrap();
        insta::assert_snapshot!(
            htmlified,
            @"<span style='color:var(--red,#a00)'>Red Still Red</span>"
        );
    }

    #[test]
    fn input_uline_blue_red_ulineoff_text_red_text() {
        // Input: underline -> blue -> red -> underline off -> "Red" -> red -> " Still Red"
        let ansi_text = "\x1b[4;34;31;24mRed\x1b[31m Still Red";
        assert_opt_equiv_to_no_opt(ansi_text);
        let htmlified = ansi_to_html::convert(ansi_text).unwrap();
        insta::assert_snapshot!(
            htmlified,
            @"<u></u><span style='color:var(--red,#a00)'>Red Still Red</span>"
        );
    }
}
