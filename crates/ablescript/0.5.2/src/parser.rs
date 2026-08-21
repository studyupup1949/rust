//! AbleScript Parser
//!
//! Type of this parser is recursive descent

use crate::ast::*;
use crate::error::{Error, ErrorKind};
use crate::lexer::Token;
use logos::{Lexer, Logos};

/// Parser structure which holds lexer and metadata
///
/// Make one using [`Parser::new`] function
struct Parser<'source> {
    lexer: Lexer<'source, Token>,
    tdark: bool,
}

impl<'source> Parser<'source> {
    /// Create a new parser from source code
    fn new(source: &'source str) -> Self {
        Self {
            lexer: Token::lexer(source),
            tdark: false,
        }
    }

    /// Start parsing tokens
    ///
    /// Loops trough lexer, parses statements, returns AST
    fn parse(&mut self) -> Result<Block, Error> {
        let mut ast = vec![];
        while let Some(token) = self.lexer.next() {
            match token {
                // Ignore comments
                Token::Comment => continue,

                // Invalid token
                Token::Error => return Err(Error::new(ErrorKind::InvalidToken, self.lexer.span())),

                // T-Dark block (replace `lang` with `script`)
                Token::TDark => ast.extend(self.tdark_flow()?),
                token => ast.push(self.parse_stmt(token)?),
            }
        }
        Ok(ast)
    }

    /// Get next item
    ///
    /// If EOF, return Error instead of None
    fn checked_next(&mut self) -> Result<Token, Error> {
        loop {
            match self
                .lexer
                .next()
                .ok_or_else(|| Error::unexpected_eoi(self.lexer.span().start))?
            {
                Token::Comment => (),
                Token::Error => break Err(Error::new(ErrorKind::InvalidToken, self.lexer.span())),
                token => break Ok(token),
            }
        }
    }

    /// Parse a token
    ///
    /// This function will route to corresponding flow functions
    /// which may advance the lexer iterator
    fn parse_stmt(&mut self, token: Token) -> Result<Spanned<Stmt>, Error> {
        let start = self.lexer.span().start;

        match token {
            Token::Unless => self.unless_flow(),
            Token::Functio => self.functio_flow(),
            Token::Bff => self.bff_flow(),
            Token::Melo => self.melo_flow(),
            Token::Loop => self.get_block().map(|body| Stmt::Loop { body }),
            Token::Enough => self.semicolon_terminated(Stmt::Enough),
            Token::AndAgain => self.semicolon_terminated(Stmt::AndAgain),
            Token::Finally => self.get_block().map(Stmt::Finally),
            Token::Rlyeh => self.semicolon_terminated(Stmt::Rlyeh),
            Token::Rickroll => self.semicolon_terminated(Stmt::Rickroll),

            Token::Identifier(_)
            | Token::String(_)
            | Token::Integer(_)
            | Token::Char(_)
            | Token::Aint
            | Token::LeftBracket
            | Token::LeftParen => self.value_flow(token),

            t => Err(Error {
                kind: ErrorKind::UnexpectedToken(t),
                span: start..self.lexer.span().end,
            }),
        }
        .map(|stmt| Spanned::new(stmt, start..self.lexer.span().end))
    }

    /// Require statement to be semicolon terminated
    ///
    /// Utility function for short statements
    fn semicolon_terminated(&mut self, stmt_kind: Stmt) -> Result<Stmt, Error> {
        self.require(Token::Semicolon)?;
        Ok(stmt_kind)
    }

    /// Require next item to be equal with expected one
    fn require(&mut self, required: Token) -> Result<(), Error> {
        match self.checked_next()? {
            t if t == required => Ok(()),
            t => Err(Error::new(ErrorKind::UnexpectedToken(t), self.lexer.span())),
        }
    }

    /// Get an Identifier
    fn get_ident(&mut self) -> Result<Spanned<String>, Error> {
        match self.checked_next()? {
            Token::Identifier(ident) => {
                Ok(Spanned::new(self.tdark_subst(ident), self.lexer.span()))
            }
            t => Err(Error::new(ErrorKind::UnexpectedToken(t), self.lexer.span())),
        }
    }

    /// Parse an expression
    ///
    /// AbleScript strongly separates expressions from statements.
    /// Expressions do not have any side effects and the are
    /// only mathematial and logical operations or values.
    fn parse_expr(
        &mut self,
        token: Token,
        buf: &mut Option<Spanned<Expr>>,
    ) -> Result<Spanned<Expr>, Error> {
        let start = match buf {
            Some(e) => e.span.start,
            None => self.lexer.span().start,
        };

        match token {
            // Values
            Token::Identifier(i) => Ok(Expr::Variable(self.tdark_subst(i))),
            Token::Integer(i) => Ok(Expr::Literal(Literal::Int(i))),
            Token::String(s) => Ok(Expr::Literal(Literal::Str(self.tdark_subst(s)))),
            Token::Char(c) => Ok(Expr::Literal(Literal::Char(c))),
            Token::LeftBracket => match buf.take() {
                Some(buf) => self.index_flow(buf),
                None => self.cart_flow(),
            },

            // Operations
            Token::Aint if buf.is_none() => {
                let next = self.checked_next()?;
                Ok(Expr::Aint(Box::new(self.parse_expr(next, buf)?)))
            }

            Token::Plus
            | Token::Minus
            | Token::Star
            | Token::FwdSlash
            | Token::Equals
            | Token::LessThan
            | Token::GreaterThan
            | Token::Aint => self.binop_flow(
                BinOpKind::from_token(token).map_err(|e| Error::new(e, self.lexer.span()))?,
                buf,
            ),

            Token::LeftParen => return self.expr_flow(Token::RightParen),
            t => Err(Error::new(ErrorKind::UnexpectedToken(t), self.lexer.span())),
        }
        .map(|expr| Spanned::new(expr, start..self.lexer.span().end))
    }

    /// Flow for creating carts
    fn cart_flow(&mut self) -> Result<Expr, Error> {
        let mut cart = vec![];
        let mut buf = None;

        match self.checked_next()? {
            Token::RightBracket => (),
            t => {
                buf = Some(self.parse_expr(t, &mut buf)?);
                'cart: loop {
                    let value = loop {
                        match self.checked_next()? {
                            Token::Arrow => break buf.take(),
                            t => buf = Some(self.parse_expr(t, &mut buf)?),
                        }
                    }
                    .ok_or_else(|| {
                        Error::new(ErrorKind::UnexpectedToken(Token::Arrow), self.lexer.span())
                    })?;

                    let key = loop {
                        match self.checked_next()? {
                            Token::RightBracket => {
                                cart.push((
                                    value,
                                    buf.take().ok_or_else(|| {
                                        Error::unexpected_eoi(self.lexer.span().start)
                                    })?,
                                ));

                                break 'cart;
                            }
                            Token::Comma => break buf.take(),
                            t => buf = Some(self.parse_expr(t, &mut buf)?),
                        }
                    }
                    .ok_or_else(|| Error::unexpected_eoi(self.lexer.span().start))?;

                    cart.push((value, key));
                }
            }
        }

        Ok(Expr::Cart(cart))
    }

    /// Flow for indexing operations
    ///
    /// Indexing with empty index resolves to length of expression, else it indexes
    fn index_flow(&mut self, expr: Spanned<Expr>) -> Result<Expr, Error> {
        let mut buf = None;
        Ok(loop {
            match self.checked_next()? {
                Token::RightBracket => match buf {
                    Some(index) => {
                        break Expr::Index {
                            expr: Box::new(expr),
                            index: Box::new(index),
                        }
                    }
                    None => break Expr::Len(Box::new(expr)),
                },
                Token::GreaterThan if buf.is_none() => {
                    self.require(Token::RightBracket)?;
                    break Expr::Keys(Box::new(expr));
                }
                token => buf = Some(self.parse_expr(token, &mut buf)?),
            }
        })
    }

    /// Flow for operators
    ///
    /// Generates operation from LHS buffer and next expression as RHS
    ///
    /// This is unaware of precedence, as AbleScript do not have it
    fn binop_flow(
        &mut self,
        kind: BinOpKind,
        lhs: &mut Option<Spanned<Expr>>,
    ) -> Result<Expr, Error> {
        Ok(Expr::BinOp {
            lhs: Box::new(
                lhs.take()
                    .ok_or_else(|| Error::new(ErrorKind::MissingLhs, self.lexer.span()))?,
            ),
            rhs: {
                let next = self.checked_next()?;
                Box::new(self.parse_expr(next, &mut None)?)
            },
            kind,
        })
    }

    /// Parse expressions until terminate token
    fn expr_flow(&mut self, terminate: Token) -> Result<Spanned<Expr>, Error> {
        let mut buf = None;
        Ok(loop {
            match self.checked_next()? {
                t if t == terminate => {
                    break buf.take().ok_or_else(|| {
                        Error::new(ErrorKind::UnexpectedToken(t), self.lexer.span())
                    })?
                }
                t => buf = Some(self.parse_expr(t, &mut buf)?),
            }
        })
    }

    /// Parse a list of statements between curly braces
    fn get_block(&mut self) -> Result<Block, Error> {
        self.require(Token::LeftCurly)?;
        let mut block = vec![];

        loop {
            match self.checked_next()? {
                Token::RightCurly => break,
                Token::TDark => block.extend(self.tdark_flow()?),
                t => block.push(self.parse_stmt(t)?),
            }
        }
        Ok(block)
    }

    /// Parse T-Dark block
    fn tdark_flow(&mut self) -> Result<Block, Error> {
        self.tdark = true;
        let block = self.get_block();
        self.tdark = false;
        block
    }

    /// If Statement parser gets any kind of value (Identifier or Literal)
    /// It cannot parse it as it do not parse expressions. Instead of it it
    /// will parse it to function call or print statement.
    fn value_flow(&mut self, init: Token) -> Result<Stmt, Error> {
        let mut buf = Some(self.parse_expr(init, &mut None)?);
        Ok(loop {
            match self.checked_next()? {
                // Print to stdout
                Token::Print => {
                    break Stmt::Print {
                        expr: buf.take().ok_or_else(|| {
                            Error::new(ErrorKind::UnexpectedToken(Token::Print), self.lexer.span())
                        })?,
                        newline: match self.checked_next()? {
                            Token::Semicolon => true,
                            Token::Minus => {
                                self.require(Token::Semicolon)?;
                                false
                            }
                            token => {
                                return Err(Error::new(
                                    ErrorKind::UnexpectedToken(token),
                                    self.lexer.span(),
                                ));
                            }
                        },
                    };
                }

                // Functio call
                Token::LeftParen => {
                    break self.functio_call_flow(buf.take().ok_or_else(|| {
                        Error::new(
                            ErrorKind::UnexpectedToken(Token::LeftParen),
                            self.lexer.span(),
                        )
                    })?)?;
                }

                // Variable declaration
                Token::Dim => {
                    return match buf.take() {
                        Some(Spanned {
                            item: Expr::Variable(ident),
                            span,
                        }) => Ok(Stmt::Dim {
                            ident: Spanned::new(ident, span),
                            init: {
                                let mut init = None;
                                loop {
                                    match self.checked_next()? {
                                        Token::Semicolon => break init,
                                        token => init = Some(self.parse_expr(token, &mut init)?),
                                    }
                                }
                            },
                        }),
                        _ => Err(Error::new(
                            ErrorKind::UnexpectedToken(Token::Dim),
                            self.lexer.span(),
                        )),
                    }
                }

                // Variable assignment
                Token::Assign => {
                    return match buf.take() {
                        Some(expr) => self.assignment_flow(expr),
                        None => Err(Error::new(
                            ErrorKind::UnexpectedToken(Token::Assign),
                            self.lexer.span(),
                        )),
                    }
                }

                // Read input
                Token::Read => {
                    if let Some(Ok(assignable)) = buf.take().map(Assignable::from_expr) {
                        self.require(Token::Semicolon)?;
                        break Stmt::Read(assignable);
                    } else {
                        return Err(Error::new(
                            ErrorKind::UnexpectedToken(Token::Read),
                            self.lexer.span(),
                        ));
                    }
                }

                t => buf = Some(self.parse_expr(t, &mut buf)?),
            }
        })
    }

    /// Parse Unless flow
    ///
    /// Consists of condition and block, there is no else
    fn unless_flow(&mut self) -> Result<Stmt, Error> {
        self.require(Token::LeftParen)?;
        Ok(Stmt::Unless {
            cond: self.expr_flow(Token::RightParen)?,
            body: self.get_block()?,
        })
    }

    /// Parse functio flow
    ///
    /// functio $ident (a, b, c) { ... }
    fn functio_flow(&mut self) -> Result<Stmt, Error> {
        let ident = self.get_ident()?;
        self.require(Token::LeftParen)?;

        let mut params = vec![];
        loop {
            match self.checked_next()? {
                Token::RightParen => break,
                Token::Identifier(i) => {
                    params.push(Spanned::new(i, self.lexer.span()));

                    // Require comma (next) or right paren (end) after identifier
                    match self.checked_next()? {
                        Token::Comma => continue,
                        Token::RightParen => break,
                        t => {
                            return Err(Error::new(
                                ErrorKind::UnexpectedToken(t),
                                self.lexer.span(),
                            ))
                        }
                    }
                }
                t => return Err(Error::new(ErrorKind::UnexpectedToken(t), self.lexer.span())),
            }
        }

        let body = self.get_block()?;

        Ok(Stmt::Functio {
            ident,
            params,
            body,
        })
    }

    /// Parse BF function declaration
    ///
    /// `bff $ident ([tapelen]) { ... }`
    fn bff_flow(&mut self) -> Result<Stmt, Error> {
        let ident = self.get_ident()?;

        let tape_len = match self.checked_next()? {
            Token::LeftParen => {
                let len = Some(self.expr_flow(Token::RightParen)?);
                self.require(Token::LeftCurly)?;
                len
            }
            Token::LeftCurly => None,
            token => {
                return Err(Error::new(
                    ErrorKind::UnexpectedToken(token),
                    self.lexer.span(),
                ))
            }
        };

        let mut code: Vec<u8> = vec![];
        loop {
            match self.checked_next()? {
                Token::Plus
                | Token::Minus
                | Token::Comma
                | Token::LeftBracket
                | Token::RightBracket
                | Token::LessThan
                | Token::GreaterThan => code.push(self.lexer.slice().as_bytes()[0]),
                Token::RightCurly => break,
                _ => (),
            }
        }

        Ok(Stmt::BfFunctio {
            ident,
            tape_len,
            code,
        })
    }

    /// Parse functio call flow
    fn functio_call_flow(&mut self, expr: Spanned<Expr>) -> Result<Stmt, Error> {
        let mut args = vec![];
        let mut buf = None;
        loop {
            match self.checked_next()? {
                // End of argument list
                Token::RightParen => {
                    if let Some(expr) = buf.take() {
                        args.push(expr)
                    }
                    break;
                }

                // Next argument
                Token::Comma => match buf.take() {
                    Some(expr) => args.push(expr),
                    // Comma alone
                    None => {
                        return Err(Error::new(
                            ErrorKind::UnexpectedToken(Token::Comma),
                            self.lexer.span(),
                        ))
                    }
                },
                t => buf = Some(self.parse_expr(t, &mut buf)?),
            }
        }

        self.require(Token::Semicolon)?;
        Ok(Stmt::Call { expr, args })
    }

    /// Parse assignment to assignable
    fn assignment_flow(&mut self, value: Spanned<Expr>) -> Result<Stmt, Error> {
        let ident = self.get_ident()?;
        let kind = match self.checked_next()? {
            Token::Semicolon => AssignableKind::Variable,
            Token::LeftBracket => {
                let mut indices = vec![];
                loop {
                    indices.push(self.expr_flow(Token::RightBracket)?);
                    match self.checked_next()? {
                        Token::Semicolon => break AssignableKind::Index { indices },
                        Token::LeftBracket => (),
                        t => {
                            return Err(Error::new(
                                ErrorKind::UnexpectedToken(t),
                                self.lexer.span(),
                            ))
                        }
                    }
                }
            }
            t => return Err(Error::new(ErrorKind::UnexpectedToken(t), self.lexer.span())),
        };

        Ok(Stmt::Assign {
            assignable: Assignable { ident, kind },
            value,
        })
    }

    /// Parse Melo flow
    fn melo_flow(&mut self) -> Result<Stmt, Error> {
        let ident = self.get_ident()?;
        self.semicolon_terminated(Stmt::Melo(ident))
    }

    /// Perform lang -> script substitution if in T-Dark block
    fn tdark_subst(&self, mut string: String) -> String {
        if self.tdark {
            if let Some(pos) = string.to_lowercase().find("lang") {
                let range = pos..pos + 4;
                let mut count_upper = 0_u8;
                string.replace_range(
                    range.clone(),
                    &(string[range]
                        .chars()
                        .zip("scri".chars())
                        .map(|(lc, sc)| {
                            if lc.is_uppercase() {
                                count_upper += 1;
                                sc.to_ascii_uppercase()
                            } else {
                                sc.to_ascii_lowercase()
                            }
                        })
                        .collect::<String>()
                        + match count_upper {
                            0 | 1 => "pt",
                            2 if rand::random() => "Pt",
                            2 => "pT",
                            _ => "PT",
                        }),
                )
            }
        }

        string
    }
}

/// Parse AbleScript code into AST
pub fn parse(source: &str) -> Result<Block, Error> {
    Parser::new(source).parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_math() {
        let code = "1 * (num + 3) / 666 print;";
        let expected = &[Spanned {
            item: Stmt::Print {
                expr: Spanned {
                    item: Expr::BinOp {
                        lhs: Box::new(Spanned {
                            item: Expr::BinOp {
                                lhs: Box::new(Spanned {
                                    item: Expr::Literal(Literal::Int(1)),
                                    span: 0..1,
                                }),
                                rhs: Box::new(Spanned {
                                    item: Expr::BinOp {
                                        lhs: Box::new(Spanned {
                                            item: Expr::Variable("num".to_owned()),
                                            span: 5..6,
                                        }),
                                        rhs: Box::new(Spanned {
                                            item: Expr::Literal(Literal::Int(3)),
                                            span: 9..10,
                                        }),
                                        kind: BinOpKind::Add,
                                    },
                                    span: 5..10,
                                }),
                                kind: BinOpKind::Multiply,
                            },
                            span: 0..11,
                        }),
                        rhs: Box::new(Spanned {
                            item: Expr::Literal(Literal::Int(666)),
                            span: 14..17,
                        }),
                        kind: BinOpKind::Divide,
                    },
                    span: 0..17,
                },
                newline: true,
            },
            span: 0..24,
        }];

        let ast = Parser::new(code).parse().unwrap();
        assert_eq!(ast, expected);
    }

    #[test]
    fn variable_declaration() {
        let code = "var dim 42;";
        let expected = &[Spanned {
            item: Stmt::Dim {
                ident: Spanned {
                    item: "var".to_owned(),
                    span: 0..3,
                },
                init: Some(Spanned {
                    item: Expr::Literal(Literal::Int(42)),
                    span: 4..6,
                }),
            },
            span: 0..11,
        }];

        let ast = Parser::new(code).parse().unwrap();
        assert_eq!(ast, expected);
    }

    #[test]
    fn unless_flow() {
        let code = "unless (never + never) { /*Buy Able products!*/ print; }";
        let expected = &[Spanned {
            item: Stmt::Unless {
                cond: Spanned {
                    item: Expr::BinOp {
                        lhs: Box::new(Spanned {
                            item: Expr::Variable("never".to_owned()),
                            span: 8..13,
                        }),
                        rhs: Box::new(Spanned {
                            item: Expr::Variable("never".to_owned()),
                            span: 16..21,
                        }),
                        kind: BinOpKind::Add,
                    },
                    span: 8..21,
                },
                body: vec![Spanned {
                    item: Stmt::Print {
                        expr: Spanned {
                            item: Expr::Literal(Literal::Str("Buy Able products!".to_owned())),
                            span: 25..47,
                        },
                        newline: true,
                    },
                    span: 25..54,
                }],
            },
            span: 0..56,
        }];

        let ast = Parser::new(code).parse().unwrap();
        assert_eq!(ast, expected);
    }

    #[test]
    fn tdark() {
        let code = "T-Dark { lang dim /*lang*/ + lang; }";
        let expected = &[Spanned {
            item: Stmt::Dim {
                ident: Spanned {
                    item: "script".to_owned(),
                    span: 9..15,
                },
                init: Some(Spanned {
                    item: Expr::BinOp {
                        lhs: Box::new(Spanned {
                            item: Expr::Literal(Literal::Str("script".to_owned())),
                            span: 20..26,
                        }),
                        rhs: Box::new(Spanned {
                            item: Expr::Variable("script".to_owned()),
                            span: 29..33,
                        }),
                        kind: BinOpKind::Add,
                    },
                    span: 20..33,
                }),
            },
            span: 9..34,
        }];

        let ast = Parser::new(code).parse().unwrap();
        assert_eq!(ast, expected);
    }

    #[test]
    fn cart_construction() {
        let code = "[/*able*/ <= 1, /*script*/ <= 3 - 1] print;";
        let expected = &[Spanned {
            item: Stmt::Print {
                expr: Spanned {
                    item: Expr::Cart(vec![
                        (
                            Spanned {
                                item: Expr::Literal(Literal::Str("able".to_owned())),
                                span: 1..7,
                            },
                            Spanned {
                                item: Expr::Literal(Literal::Int(1)),
                                span: 11..12,
                            },
                        ),
                        (
                            Spanned {
                                item: Expr::Literal(Literal::Str("script".to_owned())),
                                span: 14..22,
                            },
                            Spanned {
                                item: Expr::BinOp {
                                    kind: BinOpKind::Subtract,
                                    lhs: Box::new(Spanned {
                                        item: Expr::Literal(Literal::Int(3)),
                                        span: 26..27,
                                    }),
                                    rhs: Box::new(Spanned {
                                        item: Expr::Literal(Literal::Int(1)),
                                        span: 30..31,
                                    }),
                                },
                                span: 26..31,
                            },
                        ),
                    ]),
                    span: 0..32,
                },
                newline: true,
            },
            span: 0..39,
        }];

        let ast = Parser::new(code).parse().unwrap();
        assert_eq!(ast, expected);
    }

    #[test]
    fn cart_index() {
        let code = "[/*able*/ <= /*ablecorp*/][/*ablecorp*/] print;";
        let expected = &[Spanned {
            item: Stmt::Print {
                expr: Spanned {
                    item: Expr::Index {
                        expr: Box::new(Spanned {
                            item: Expr::Cart(vec![(
                                Spanned {
                                    item: Expr::Literal(Literal::Str("able".to_owned())),
                                    span: 1..7,
                                },
                                Spanned {
                                    item: Expr::Literal(Literal::Str("ablecorp".to_owned())),
                                    span: 11..21,
                                },
                            )]),
                            span: 0..22,
                        }),
                        index: Box::new(Spanned {
                            item: Expr::Literal(Literal::Str("ablecorp".to_owned())),
                            span: 23..33,
                        }),
                    },
                    span: 0..34,
                },
                newline: true,
            },
            span: 0..41,
        }];

        let ast = Parser::new(code).parse().unwrap();
        assert_eq!(ast, expected);
    }
}
