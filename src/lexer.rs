//! Bash lexer and parser for syntax highlighting.
//!
//! Note that this parser is imprecise and possibly incorrect for
//! more complicated expressions.

use std::{borrow::Cow, fmt::Write};

use ansi_to_html::Esc;
use logos::{Lexer, Logos};

use crate::Args;

type StdError = Box<dyn std::error::Error>;

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("Backticks in heredoc delimiters are not supported")]
    BackticksInHeredocDelimiter,
    #[error("Parentheses in heredoc delimiters are not supported")]
    ParensInHeredocDelimiter,
    #[error("Invalid heredoc")]
    InvalidHeredoc,

    #[error("Unexpected token {0:?} found")]
    UnexpectedToken(&'static str),

    #[error("Unknown error occurred")]
    Unknown,
}

/// List of regular tokens
#[derive(Debug)]
pub(crate) struct Tokens<'a>(Vec<Token<'a>>);

/// Regular token
#[derive(Debug)]
pub(crate) enum Token<'a> {
    /// `"# this is a comment"`
    Comment(&'a str),

    /// `"\\033"`, `"\\@"`
    EscapeSequence(&'a str),

    /// `"|"`, `">"` `"2>&1"`
    Pipe(&'a str),

    /// `"   "`
    Whitespace(&'a str),

    /// `"--foo"`
    Word(&'a str),

    /// ```js
    /// "Hello `echo $world`!"
    /// ```
    /// is represented as (simplified):
    /// ```js
    /// ["Hello ", Backticks(["echo ", Variable("$world")]), "!"]
    /// ```
    DString(DString<'a>),

    /// `"'Hello world'"`
    SString(&'a str),

    /// ```js
    /// `echo 'Hello world'`
    /// ```
    /// is represented as (simplified):
    /// ```js
    /// Backticks(["echo", " ", "'Hello world'"])
    /// ```
    Backticks(Tokens<'a>),

    /// ```js
    /// [ ! foo ]
    /// ```
    /// is represented as (simplified):
    /// ```js
    /// Brackets([" ", "!", " ", "foo", " "])
    /// ```
    Brackets(Tokens<'a>),

    /// ```js
    /// ( echo foo )
    /// ```
    /// is represented as (simplified):
    /// ```js
    /// Parens([" ", "echo", " ", "foo", " "])
    /// ```
    Parens(Tokens<'a>),

    /// ```js
    /// $( echo foo )
    /// ```
    /// is represented as (simplified):
    /// ```js
    /// DollarParens([" ", "echo", " ", "foo", " "])
    /// ```
    DollarParens(Tokens<'a>),

    /// ```js
    /// { echo foo }
    /// ```
    /// is represented as (simplified):
    /// ```js
    /// Braces([" ", "echo", " ", "foo", " "])
    /// ```
    Braces(Tokens<'a>),

    /// ```bash
    /// <<EOF > somefile.txt
    /// Hello world!
    /// EOF
    /// ```
    /// is represented as (simplified):
    /// ```ignore
    /// Heredoc {
    ///     first_line: ["EOF", " ", ">", " ", "somefile.txt"],
    ///     content: ["Hello world!"],
    ///     last: "EOF",
    /// }
    /// ```
    Heredoc(Heredoc<'a>),

    /// `"$@"`, `"$HELLO_WORLD"`
    Variable(&'a str),
}

/// Double quoted string
#[derive(Debug)]
pub(crate) struct DString<'a>(Vec<DStringToken<'a>>);

/// Token in a double quoted string
#[derive(Debug)]
pub(crate) enum DStringToken<'a> {
    Content(&'a str),
    Variable(&'a str),
    Escaped(&'a str),
    Backticks(Tokens<'a>),
    Parens(Tokens<'a>),
}

/// Heredoc string
#[derive(Debug)]
pub(crate) struct Heredoc<'a> {
    first_line: Tokens<'a>,
    content: Vec<&'a str>,
    last: String,
}

#[derive(Logos, Debug, PartialEq, Copy, Clone)]
pub(crate) enum TokenKind {
    #[regex("#.*")]
    Comment,

    #[regex(r#"\\\d\d\d"#)]
    #[regex(r#"\\[^\d]"#)]
    EscapeSequence,

    #[token("|")]
    #[token("<")]
    #[token(";")]
    #[token("&&")]
    #[regex(">>?")]
    #[regex("[012&]>>?")]
    #[regex("[012]>>?&[012]")]
    Pipe,

    #[token("\n")]
    LineBreak,

    #[regex(r"\s+")]
    Whitespace,

    #[regex(r#"[^\s"'\\\|#<>;`\[\]\{\}\(\)\$]+"#, priority = 0)]
    #[token("$", priority = 0)]
    Word,

    #[token("\"")]
    DoubleQuote,

    #[token("<<")]
    HeredocStart,

    #[token("`")]
    Backtick,

    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,

    #[token("(")]
    OpenParen,
    #[token("$(")]
    OpenDollarParen,
    #[token(")")]
    CloseParen,

    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,

    #[regex("'[^']*'")]
    SingleQuoteString,

    #[regex(r"\$[\d#\-\$*?!@]|\$[\w_][\w\d_]*")]
    #[regex(r#"\$\{(\\\\|\\\}|[^\\}])*\}"#)]
    Variable,

    #[error]
    Error,
}

#[derive(Logos, Debug, PartialEq, Copy, Clone)]
pub(crate) enum DStringTokenKind {
    #[token("\"", priority = 2)]
    DoubleQuote,

    #[regex(r"\$[\d#\-\$*?!@]|\$[\w_][\w\d_]*", priority = 3)]
    #[regex(r#"\$\{(\\\\|\\\}|[^\\}])*\}"#, priority = 3)]
    Variable,

    #[token("`", priority = 2)]
    Backtick,
    #[token("$(", priority = 3)]
    OpenDollarParen,

    #[regex(r#"\\[`\$\\"]"#, priority = 2)]
    Escaped,

    #[regex(r#"[^\\\$`"]+"#, priority = 1)]
    #[token("$", priority = 2)]
    Content,

    #[error]
    Error,
}

#[derive(Logos, Debug, PartialEq, Copy, Clone)]
pub(crate) enum HeredocTokenKind {
    #[regex("[^\n]+\n?", priority = 1)]
    Line,

    #[error]
    Error,
}

impl Tokens<'_> {
    fn heredoc_start_tokens(&self) -> Result<String, Error> {
        self.0
            .iter()
            .take_while(|&t| {
                matches!(
                    t,
                    Token::Word(_)
                        | Token::SString(_)
                        | Token::Variable(_)
                        | Token::EscapeSequence(_)
                        | Token::DString(_)
                )
            })
            .map(|t| {
                Ok(match t {
                    &Token::Word(s) => Cow::Borrowed(s),
                    &Token::SString(d) => Cow::Borrowed(&d[1..d.len() - 1]),
                    &Token::Variable(s) => Cow::Borrowed(s),
                    &Token::EscapeSequence(s) => Cow::Borrowed(&s[1..]),
                    Token::DString(d) => Cow::Owned(d.to_string_heredoc()?),
                    _ => unreachable!("Invalid token"),
                })
            })
            .collect()
    }
}

impl DString<'_> {
    fn to_string_heredoc(&self) -> Result<String, Error> {
        self.0
            .iter()
            .map(|t| {
                Ok(match t {
                    &DStringToken::Content(c) => Cow::Borrowed(c),
                    &DStringToken::Variable(v) => Cow::Borrowed(v),
                    &DStringToken::Escaped(e) => Cow::Borrowed(&e[1..]),
                    DStringToken::Backticks(_) => return Err(Error::BackticksInHeredocDelimiter),
                    DStringToken::Parens(_) => return Err(Error::ParensInHeredocDelimiter),
                })
            })
            .collect()
    }
}

pub(crate) fn parse_tokens(
    mut lex: Lexer<TokenKind>,
    until: fn(&TokenKind) -> bool,
) -> Result<(Tokens, Lexer<TokenKind>), Error> {
    let mut tokens = Vec::new();

    while let Some(token) = lex.next() {
        if until(&token) {
            break;
        }
        match token {
            TokenKind::Comment => {
                tokens.push(Token::Comment(lex.slice()));
            }
            TokenKind::EscapeSequence => {
                tokens.push(Token::EscapeSequence(lex.slice()));
            }
            TokenKind::Pipe => {
                tokens.push(Token::Pipe(lex.slice()));
            }
            TokenKind::Whitespace => {
                tokens.push(Token::Whitespace(lex.slice()));
            }
            TokenKind::LineBreak => {
                tokens.push(Token::Whitespace(lex.slice()));
            }
            TokenKind::Word => {
                tokens.push(Token::Word(lex.slice()));
            }
            TokenKind::DoubleQuote => {
                let (d_string, lex2) = parse_d_string(lex.morph())?;
                lex = lex2.morph();
                tokens.push(Token::DString(d_string));
            }
            TokenKind::SingleQuoteString => {
                tokens.push(Token::SString(lex.slice()));
            }
            TokenKind::HeredocStart => {
                let (first_line, lex2) = parse_tokens(lex, |&t| t == TokenKind::LineBreak)?;

                let start_tokens = first_line.heredoc_start_tokens()?;
                if start_tokens.is_empty() {
                    return Err(Error::InvalidHeredoc);
                }
                let mut heredoc = Heredoc {
                    first_line,
                    content: Vec::new(),
                    last: start_tokens,
                };

                let mut lex2 = lex2.morph::<HeredocTokenKind>();
                while let Some(_) = lex2.next() {
                    let mut s: &str = lex2.slice();
                    if s.ends_with('\n') {
                        s = &s[..s.len() - 1];
                    }
                    if s == heredoc.last {
                        break;
                    }
                    heredoc.content.push(s);
                }
                lex = lex2.morph();
                tokens.push(Token::Heredoc(heredoc));
            }
            TokenKind::Backtick => {
                let (backticks, lex2) = parse_tokens(lex, |&t| t == TokenKind::Backtick)?;
                lex = lex2;
                tokens.push(Token::Backticks(backticks));
            }
            TokenKind::OpenBracket => {
                let (token, lex2) = parse_tokens(lex, |&t| t == TokenKind::CloseBracket)?;
                lex = lex2;
                tokens.push(Token::Brackets(token));
            }
            TokenKind::CloseBracket => {
                return Err(Error::UnexpectedToken("]"));
            }
            TokenKind::OpenBrace => {
                let (token, lex2) = parse_tokens(lex, |&t| t == TokenKind::CloseBrace)?;
                lex = lex2;
                tokens.push(Token::Braces(token));
            }
            TokenKind::CloseBrace => {
                return Err(Error::UnexpectedToken("}"));
            }
            TokenKind::OpenParen => {
                let (token, lex2) = parse_tokens(lex, |&t| t == TokenKind::CloseParen)?;
                lex = lex2;
                tokens.push(Token::Parens(token));
            }
            TokenKind::OpenDollarParen => {
                let (token, lex2) = parse_tokens(lex, |&t| t == TokenKind::CloseParen)?;
                lex = lex2;
                tokens.push(Token::DollarParens(token));
            }
            TokenKind::CloseParen => {
                return Err(Error::UnexpectedToken(")"));
            }
            TokenKind::Variable => {
                tokens.push(Token::Variable(lex.slice()));
            }
            TokenKind::Error => return Err(Error::Unknown),
        }
    }

    Ok((Tokens(tokens), lex))
}

fn parse_d_string(
    mut lex: Lexer<DStringTokenKind>,
) -> Result<(DString, Lexer<DStringTokenKind>), Error> {
    let mut tokens = Vec::new();
    while let Some(token) = lex.next() {
        match token {
            DStringTokenKind::DoubleQuote => {
                break;
            }
            DStringTokenKind::Variable => {
                tokens.push(DStringToken::Variable(lex.slice()));
            }
            DStringTokenKind::Backtick => {
                let (backticks, lex2) = parse_tokens(lex.morph(), |&t| t == TokenKind::Backtick)?;
                lex = lex2.morph();
                tokens.push(DStringToken::Backticks(backticks));
            }
            DStringTokenKind::OpenDollarParen => {
                let (parens, lex2) = parse_tokens(lex.morph(), |&t| t == TokenKind::CloseParen)?;
                lex = lex2.morph();
                tokens.push(DStringToken::Parens(parens));
            }
            DStringTokenKind::Escaped => {
                tokens.push(DStringToken::Escaped(lex.slice()));
            }
            DStringTokenKind::Content => {
                tokens.push(DStringToken::Content(lex.slice()));
            }
            DStringTokenKind::Error => return Err(Error::Unknown),
        }
    }

    Ok((DString(tokens), lex))
}

pub(crate) fn colorize(buf: &mut String, command: &str, args: &Args) -> Result<(), StdError> {
    let lex = TokenKind::lexer(command);
    let (tokens, _) = parse_tokens(lex, |_| false)?;

    tokens.colorize(buf, args, true)?;

    Ok(())
}

impl Tokens<'_> {
    fn colorize(&self, buf: &mut String, args: &Args, as_command: bool) -> Result<(), StdError> {
        let mut next = if as_command {
            State::Start
        } else {
            State::Default
        };
        let prefix = args.prefix.as_str();

        #[derive(Debug, Copy, Clone, Eq, PartialEq)]
        enum State {
            Default,
            Start,
            Pipe,
        }

        let mut hl_subcommand = false;

        for token in &self.0 {
            match token {
                &Token::Comment(c) => {
                    write!(buf, "<span class='{}com'>{}</span>", prefix, Esc(c))?;
                }
                &Token::EscapeSequence(e) => {
                    write!(buf, "<span class='{}esc'>{}</span>", prefix, Esc(e))?;
                    if e == "\\\n" {
                        continue;
                    }
                }
                &Token::Pipe(p) => {
                    if let ";" | "&&" = p {
                        write!(buf, "<span class='{}punct'>{}</span>", prefix, Esc(p))?;
                    } else {
                        write!(buf, "<span class='{}pipe'>{}</span>", prefix, Esc(p))?;
                    }

                    hl_subcommand = false;
                    if let "|" | ";" | "&&" = p {
                        next = State::Start;
                        continue;
                    } else {
                        next = State::Pipe;
                        continue;
                    }
                }
                &Token::Whitespace(w) => {
                    write!(buf, "{}", w)?;
                    continue;
                }
                &Token::Word(w) => {
                    if next == State::Start {
                        if w.contains('=') {
                            write!(buf, "<span class=\"{}env\">{}</span>", prefix, Esc(w))?;
                        } else {
                            next = State::Default;
                            write!(buf, "<span class=\"{}cmd\">{}</span>", prefix, Esc(w))?;
                            if args.highlight.contains(&w) {
                                hl_subcommand = true;
                                continue;
                            }
                        }
                    } else if next == State::Pipe {
                        write!(buf, "<span class='{}pipe'>{}</span>", prefix, Esc(w))?;
                    } else if w.starts_with('-') {
                        if let Some((i, _)) = w.char_indices().find(|&(_, c)| c == '=') {
                            let (p1, p2) = w.split_at(i);

                            write!(buf, "<span class=\"{}flag\">{}</span>", prefix, Esc(p1))?;
                            write!(buf, "<span class=\"{}arg\">{}</span>", prefix, Esc(p2))?;
                        } else {
                            write!(buf, "<span class=\"{}flag\">{}</span>", prefix, Esc(w))?;
                        }
                    } else if hl_subcommand {
                        write!(buf, "<span class=\"{}hl\">{}</span>", prefix, Esc(w))?;
                    } else {
                        write!(buf, "<span class=\"{}arg\">{}</span>", prefix, Esc(w))?;
                    }
                }
                Token::DString(d) => {
                    d.colorize(buf, args)?;
                }
                &Token::SString(s) => {
                    write!(buf, "<span class='{}str'>{}</span>", prefix, Esc(s))?;
                }
                Token::Backticks(t) => {
                    write!(buf, "<span class='{}punct'>`</span>", prefix)?;
                    t.colorize(buf, args, true)?;
                    write!(buf, "<span class='{}punct'>`</span>", prefix)?;
                }
                Token::Brackets(t) => {
                    write!(buf, "<span class='{}punct'>[</span>", prefix)?;
                    t.colorize(buf, args, false)?;
                    write!(buf, "<span class='{}punct'>]</span>", prefix)?;
                }
                Token::Parens(t) => {
                    write!(buf, "<span class='{}punct'>(</span>", prefix)?;
                    t.colorize(buf, args, false)?;
                    write!(buf, "<span class='{}punct'>)</span>", prefix)?;
                }
                Token::DollarParens(t) => {
                    write!(buf, "<span class='{}punct'>$(</span>", prefix)?;
                    t.colorize(buf, args, true)?;
                    write!(buf, "<span class='{}punct'>)</span>", prefix)?;
                }
                Token::Braces(t) => {
                    write!(buf, "<span class='{}punct'>{{</span>", prefix)?;
                    t.colorize(buf, args, true)?;
                    write!(buf, "<span class='{}punct'>}}</span>", prefix)?;
                }
                Token::Heredoc(h) => {
                    h.colorize(buf, args)?;
                }
                &Token::Variable(v) => {
                    write!(buf, "<span class='{}var'>{}</span>", prefix, Esc(v))?;
                }
            }
            hl_subcommand = false;
            next = State::Default;
        }
        Ok(())
    }
}

impl DString<'_> {
    fn colorize(&self, buf: &mut String, args: &Args) -> Result<(), StdError> {
        let prefix = args.prefix.as_str();
        write!(buf, "<span class='{}str'>\"", prefix)?;

        for token in &self.0 {
            match token {
                &DStringToken::Content(c) => {
                    write!(buf, "{}", Esc(c))?;
                }
                &DStringToken::Variable(v) => {
                    write!(buf, "<span class='{}var'>{}</span>", prefix, Esc(v))?;
                }
                &DStringToken::Escaped(e) => {
                    write!(buf, "<span class='{}esc'>{}</span>", prefix, Esc(e))?;
                }
                DStringToken::Backticks(t) => {
                    write!(buf, "`</span>")?;
                    t.colorize(buf, args, true)?;
                    write!(buf, "<span class='{}str'>`", prefix)?;
                }
                DStringToken::Parens(t) => {
                    write!(buf, "$(</span>")?;
                    t.colorize(buf, args, true)?;
                    write!(buf, "<span class='{}str'>)", prefix)?;
                }
            }
        }

        write!(buf, "\"</span>")?;
        Ok(())
    }
}

impl Heredoc<'_> {
    fn colorize(&self, buf: &mut String, args: &Args) -> Result<(), StdError> {
        let prefix = args.prefix.as_str();

        write!(buf, "&lt;&lt;")?;
        self.first_line.colorize(buf, args, false)?;
        writeln!(buf)?;

        write!(buf, "<span class='{}str'>", prefix)?;
        for &line in &self.content {
            writeln!(buf, "{}", Esc(line))?;
        }
        writeln!(buf, "</span>{}", Esc(&self.last))?;

        Ok(())
    }
}
