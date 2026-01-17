use std::{fmt::Display, ops::Deref, sync::Arc};

use line_column::line_column;
use logos::{Logos, Source, Span};
use peg::{Parse, ParseLiteral, RuleResult};
use unescaper::unescape;

pub struct TokenStream {
    input: Arc<String>,
    tokens: Vec<(Token, Span)>,
}

impl Deref for TokenStream {
    type Target = Vec<(Token, Span)>;
    fn deref(&self) -> &Self::Target {
        &self.tokens
    }
}

impl TokenStream {
    pub fn new(input: Arc<String>) -> Result<Self, LexingErrors> {
        let mut errors = Vec::new();
        let tokens = Token::lexer(&input)
            .spanned()
            .filter_map(|(token, span)| match token {
                Ok(token) => Some((token, span)),
                Err(err) => {
                    errors.push(err);
                    None
                }
            })
            .collect();

        if !errors.is_empty() {
            return Err(LexingErrors(errors));
        }

        Ok(Self { input, tokens })
    }
}

#[derive(Debug, Clone)]
pub struct Sp(Span, Arc<String>);

impl Display for Sp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start = self.0.start;
        let start = line_column(&self.1, start);

        write!(f, "{}:{}", start.0, start.1)
    }
}

impl Parse for TokenStream {
    type PositionRepr = Sp;

    fn start(&self) -> usize {
        0
    }

    fn is_eof(&self, p: usize) -> bool {
        p >= self.tokens.len()
    }

    fn position_repr(&self, p: usize) -> Self::PositionRepr {
        Sp(
            self.tokens
                .get(p)
                .map_or_else(|| 0..0, |(_, span)| span.clone()),
            Arc::clone(&self.input),
        )
    }
}

impl ParseLiteral for TokenStream {
    fn parse_string_literal(&self, pos: usize, literal: &str) -> RuleResult<()> {
        let Some((token, span)) = self.tokens.get(pos) else {
            return RuleResult::Failed;
        };
        match token {
            Token::RawIdent(ident) if ident == literal => RuleResult::Matched(pos + 1, ()),
            _ if self.input.slice(span.clone()).unwrap() == literal => {
                RuleResult::Matched(pos + 1, ())
            }
            _ => RuleResult::Failed,
        }
    }
}

#[derive(Debug)]
pub struct LexingErrors(Vec<LexingError>);

impl Display for LexingErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for error in &self.0 {
            writeln!(f, "{:?}", error)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LexingError {
    Unexpected(char),
    #[default]
    Other,
}

impl LexingError {
    fn from_lexer(lex: &mut logos::Lexer<'_, Token>) -> Self {
        LexingError::Unexpected(lex.slice().chars().next().unwrap())
    }
}

impl std::error::Error for LexingErrors {}

#[derive(Logos, Debug, PartialEq, Eq)]
#[logos(skip(r"[ \t\r\n\f]+"))]
#[logos(skip(r"\/\/[^\n]*"))]
#[logos(skip(r"\/\* [^\*\/]* \*\/"))]
#[logos(error(LexingError, LexingError::from_lexer))]
pub enum Token {
    #[regex(r"\d+[iu]size", |lex| lex.slice().to_string())]
    #[regex(r"\d+[iu][1-9]\d*", |lex| lex.slice().to_string())]
    #[regex(r"\d+", |lex| lex.slice().to_string())]
    Number(String),
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),
    #[regex(r#"\@\"[^\"]*\""#, |lex| lex.slice().to_string())]
    RawIdent(String),
    #[regex(r#"\"[^\"]*\""#, |lex| {
        let s = lex.slice().trim_start_matches('"').trim_end_matches('"');
        unescape(s).unwrap()
    })]
    String(String),

    #[token(";")]
    Semi,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token(":")]
    Colon,
    #[token("=")]
    Eq,
    #[token("!")]
    Bang,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("&")]
    And,
    #[token("|")]
    Or,
}
