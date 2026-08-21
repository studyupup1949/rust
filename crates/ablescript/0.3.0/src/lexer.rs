use logos::{Lexer, Logos};

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // Symbols
    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    #[token("[")]
    LeftBracket,

    #[token("]")]
    RightBracket,

    #[token("{")]
    LeftCurly,

    #[token("}")]
    RightCurly,

    #[token(";")]
    Semicolon,

    #[token(",")]
    Comma,

    // Operators
    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    FwdSlash,

    #[token("=")]
    Equal,

    #[token("<=")]
    Arrow,

    // Logical operators
    #[token("<")]
    LessThan,

    #[token(">")]
    GreaterThan,

    #[token("==")]
    EqualEqual,

    #[token("ain't")]
    Aint,

    // Keywords
    #[token("functio")]
    Functio,

    /// Brain fuck FFI
    #[token("bff")]
    Bff,

    /// Variable bro
    #[token("var")]
    Var,

    /// Prints the preceding things
    #[token("print")]
    Print,

    /// Read input into preceding variable
    #[token("read")]
    Read,

    /// Ban the following variable from ever being used again
    #[token("melo")]
    Melo,

    #[token("T-Dark")]
    TDark,

    // Control flow keywords
    #[token("if")]
    If,

    #[token("loop")]
    Loop,

    #[token("break")]
    Break,

    /// HopBack hops on the back of loop - like `continue`
    #[token("hopback")]
    HopBack,

    /// Crash with random error (see discussion #17)
    #[token("rlyeh")]
    Rlyeh,

    #[token("rickroll")]
    Rickroll,

    // Literals
    /// String
    #[token("/*", get_string)]
    String(String),

    /// Integer
    #[regex(r"-?[0-9]+", get_value)]
    Integer(isize),

    /// An identifier
    #[regex(r"\p{XID_Start}[\p{XID_Continue}]*", get_ident)]
    Identifier(String),

    #[regex(r"owo .*")]
    Comment,

    #[regex(r"[ \t\n\f]+", logos::skip)]
    #[error]
    Error,
}

fn get_value<T: std::str::FromStr>(lexer: &mut Lexer<Token>) -> Option<T> {
    lexer.slice().parse().ok()
}

fn get_string(lexer: &mut Lexer<Token>) -> Option<String> {
    lexer.bump(lexer.remainder().find("*/")?);
    let string = lexer.slice()[2..].to_owned();
    lexer.bump(2);

    Some(string)
}

fn get_ident(lexer: &mut Lexer<Token>) -> String {
    lexer.slice().to_owned()
}

#[cfg(test)]
mod tests {
    use super::Token;
    use super::Token::*;
    use logos::Logos;

    #[test]
    fn simple_fn() {
        let code = "functio test() { var a = 3; if a == 3 { a print } }";
        let expected = &[
            Functio,
            Identifier("test".to_owned()),
            LeftParen,
            RightParen,
            LeftCurly,
            Var,
            Identifier("a".to_owned()),
            Equal,
            Integer(3),
            Semicolon,
            If,
            Identifier("a".to_owned()),
            EqualEqual,
            Integer(3),
            LeftCurly,
            Identifier("a".to_owned()),
            Print,
            RightCurly,
            RightCurly,
        ];
        let lexer = Token::lexer(code);
        let result: Vec<Token> = lexer.collect();
        assert_eq!(result, expected);
    }
}
