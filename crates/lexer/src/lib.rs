use logos::{Logos, Span};

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

pub fn tokenize(input: &str) -> (Vec<(Token, Span)>, Vec<<Token as Logos<'_>>::Error>) {
    let tokens = Token::lexer(input).spanned();
    let mut errors = Vec::new();
    let tokens = tokens.filter_map(|(token, span)| match token {
        Ok(token) => Some((token, span)),
        Err(err) => {
            errors.push(err);
            None
        }
    });
    (tokens.collect(), errors)
}

#[derive(Logos, Debug, PartialEq, Eq)]
#[logos(subpattern digit = r"[0-9]")]
#[logos(error(LexingError, LexingError::from_lexer))]
pub enum Token {
    #[regex(r"\/\/[^\n]*", allow_greedy = true, priority = 1)]
    LineComment,
    #[regex(r"\/\* [^\*\/]* \*\/", allow_greedy = true, priority = 2)]
    BlockComment,

    #[regex(r"[ \t\r\n\f]+")]
    Whitespace,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,
    #[regex(r#"\@\"[^\"]*\""#)]
    RawIdent,

    #[regex(r"(?&digit)+[i|u(?&digit)+]?")]
    Number,
    #[regex(r#"\"[^\"]*\""#)]
    String,

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

    #[token("const")]
    Const,
    #[token("let")]
    Let,
    #[token("mut")]
    Mut,
    #[token("fn")]
    Fn,
    #[token("proto")]
    Proto,
    #[token("extern")]
    Extern,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("loop")]
    Loop,
    #[token("return")]
    Return,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("asm")]
    Asm,
    #[token("as")]
    As,
    #[token("struct")]
    Struct,
}
