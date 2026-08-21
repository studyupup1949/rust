use logos::{Lexer, Logos};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(skip r"owo .*")]
#[rustfmt::skip]
pub enum Token {
    // Symbols
    #[token("(")] LeftParen,
    #[token(")")] RightParen,
    #[token("[")] LeftBracket,
    #[token("]")] RightBracket,
    #[token("{")] LeftCurly,
    #[token("}")] RightCurly,
    #[token(";")] Semicolon,
    #[token(",")] Comma,

    // Operators
    #[token("+")] Plus,
    #[token("-")] Minus,
    #[token("*")] Star,
    #[token("/")] FwdSlash,
    #[token("=:")] Assign,
    #[token("<=")] Arrow,

    // Logical operators
    #[token("<")] LessThan,
    #[token(">")] GreaterThan,
    #[token("=")] Equals,
    #[token("ain't")] Aint,

    // Keywords
    #[token("functio")] Functio,
    #[token("bff")] Bff,
    #[token("dim")] Dim,
    #[token("print")] Print,
    #[token("read")] Read,
    #[token("melo")] Melo,
    #[token("T-Dark")] TDark,

    // Control flow keywords
    #[token("unless")] Unless,
    #[token("loop")] Loop,
    #[token("enough")] Enough,
    #[token("and again")] AndAgain,
    #[token("finally")] Finally,
    #[token("rlyeh")] Rlyeh,

    #[token("rickroll")] Rickroll,

    // Literals
    #[token("/*", get_string)] String(String),
    #[regex(r"-?[0-9]+", get_value)] Integer(isize),
    #[regex(r"\p{XID_Start}", get_value)] Char(char),
    #[regex(r"\p{XID_Start}[\p{XID_Continue}]+", get_ident)]
    #[token("and ", |_| "and".to_owned())]
    Identifier(String),
}

fn get_value<T: std::str::FromStr>(lexer: &mut Lexer<Token>) -> Option<T> {
    lexer.slice().parse().ok()
}

fn get_string(lexer: &mut Lexer<Token>) -> Option<String> {
    lexer.bump(lexer.remainder().find("*/")?);

    let mut string = String::new();
    let mut slice = &lexer.slice()[2..];
    while let Some(escape_start) = slice.find('"') {
        // Push predeceasing string
        string.push_str(slice.get(..escape_start)?);

        // Move slice behind escape start delimiter
        slice = slice.get(escape_start + 1..)?;

        // Get escape end delimiter position and parse string before it to
        // a character from it's unicode value (base-12) and push it to string
        let escape_end = slice.find('"')?;
        string.push(
            u32::from_str_radix(slice.get(..escape_end)?, 12)
                .ok()
                .and_then(char::from_u32)?,
        );

        // Move slice behind escape end delimiter
        slice = slice.get(escape_end + 1..)?;
    }

    // Push remaining string
    string.push_str(slice);
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
        let code = "functio test() { dim var 3; unless (var ain't 3) { var print } }";
        let expected = &[
            Functio,
            Identifier("test".to_owned()),
            LeftParen,
            RightParen,
            LeftCurly,
            Dim,
            Identifier("var".to_owned()),
            Integer(3),
            Semicolon,
            Unless,
            LeftParen,
            Identifier("var".to_owned()),
            Aint,
            Integer(3),
            RightParen,
            LeftCurly,
            Identifier("var".to_owned()),
            Print,
            RightCurly,
            RightCurly,
        ];

        let result: Vec<_> = Token::lexer(code).collect::<Result<_, _>>().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn escapes() {
        let code = r#"/*»"720B""722B""7195"«*/"#;
        let expected = &[Token::String("»にゃぁ«".to_owned())];

        let result: Vec<_> = Token::lexer(code).collect::<Result<_, _>>().unwrap();
        assert_eq!(result, expected);
    }
}
