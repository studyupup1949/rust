//! AbleScript Parser
//!
//! Type of this parser is recursive descent

use crate::ast::*;
use crate::error::{Error, ErrorKind};
use crate::lexer::Token;
use crate::variables::Value;
use logos::{Lexer, Logos};

/// Parser structure which holds lexer and metadata
///
/// Make one using [`Parser::new`] function
pub struct Parser<'source> {
    lexer: Lexer<'source, Token>,
    tdark: bool,
}

impl<'source> Parser<'source> {
    /// Create a new parser from source code
    pub fn new(source: &'source str) -> Self {
        Self {
            lexer: Token::lexer(source),
            tdark: false,
        }
    }

    /// Start parsing tokens
    ///
    /// Loops trough lexer, parses statements, returns AST
    pub fn init(&mut self) -> Result<Block, Error> {
        let mut ast = vec![];
        while let Some(token) = self.lexer.next() {
            match token {
                // Ignore comments
                Token::Comment => continue,

                // T-Dark block (replace `lang` with `script`)
                Token::TDark => ast.extend(self.tdark_flow()?),
                token => ast.push(self.parse(token)?),
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
                .ok_or_else(|| Error::unexpected_eof(self.lexer.span().start))?
            {
                Token::Comment => (),
                token => break Ok(token),
            }
        }
    }

    /// Parse a token
    ///
    /// This function will route to corresponding flow functions
    /// which may advance the lexer iterator
    fn parse(&mut self, token: Token) -> Result<Spanned<Stmt>, Error> {
        let start = self.lexer.span().start;

        match token {
            Token::If => Ok(Spanned::new(self.if_flow()?, start..self.lexer.span().end)),
            Token::Functio => Ok(Spanned::new(
                self.functio_flow()?,
                start..self.lexer.span().end,
            )),
            Token::Bff => Ok(Spanned::new(self.bff_flow()?, start..self.lexer.span().end)),
            Token::Var => Ok(Spanned::new(self.var_flow()?, start..self.lexer.span().end)),
            Token::Melo => Ok(Spanned::new(
                self.melo_flow()?,
                start..self.lexer.span().end,
            )),
            Token::Loop => Ok(Spanned::new(
                self.loop_flow()?,
                start..self.lexer.span().end,
            )),
            Token::Break => Ok(Spanned::new(
                self.semicolon_terminated(Stmt::Break)?,
                start..self.lexer.span().end,
            )),
            Token::HopBack => Ok(Spanned::new(
                self.semicolon_terminated(Stmt::HopBack)?,
                start..self.lexer.span().end,
            )),
            Token::Rlyeh => Ok(Spanned::new(
                self.semicolon_terminated(Stmt::Rlyeh)?,
                start..self.lexer.span().end,
            )),
            Token::Rickroll => Ok(Spanned::new(
                self.semicolon_terminated(Stmt::Rickroll)?,
                start..self.lexer.span().end,
            )),

            Token::Identifier(_)
            | Token::String(_)
            | Token::Integer(_)
            | Token::Aint
            | Token::LeftBracket
            | Token::LeftParen => Ok(Spanned::new(
                self.value_flow(token)?,
                start..self.lexer.span().end,
            )),

            t => Err(Error {
                kind: ErrorKind::UnexpectedToken(t),
                span: start..self.lexer.span().end,
            }),
        }
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
            Token::Identifier(ident) => Ok(Spanned::new(
                if self.tdark {
                    ident.replace("lang", "script")
                } else {
                    ident
                },
                self.lexer.span(),
            )),
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
            Token::Identifier(i) => Ok(Spanned::new(
                Expr::Variable(if self.tdark {
                    i.replace("lang", "script")
                } else {
                    i
                }),
                start..self.lexer.span().end,
            )),
            Token::Integer(i) => Ok(Spanned::new(
                Expr::Literal(Value::Int(i)),
                start..self.lexer.span().end,
            )),
            Token::String(s) => Ok(Spanned::new(
                Expr::Literal(Value::Str(if self.tdark {
                    s.replace("lang", "script")
                } else {
                    s
                })),
                start..self.lexer.span().end,
            )),

            Token::LeftBracket => match buf.take() {
                Some(buf) => Ok(Spanned::new(
                    self.index_flow(buf)?,
                    start..self.lexer.span().end,
                )),
                None => Ok(Spanned::new(
                    self.cart_flow()?,
                    start..self.lexer.span().end,
                )),
            },

            // Operations
            Token::Aint if buf.is_none() => Ok(Spanned::new(
                {
                    let next = self.checked_next()?;
                    Expr::Aint(Box::new(self.parse_expr(next, buf)?))
                },
                start..self.lexer.span().end,
            )),

            Token::Plus
            | Token::Minus
            | Token::Star
            | Token::FwdSlash
            | Token::EqualEqual
            | Token::LessThan
            | Token::GreaterThan
            | Token::Aint => Ok(Spanned::new(
                self.binop_flow(
                    BinOpKind::from_token(token).map_err(|e| Error::new(e, self.lexer.span()))?,
                    buf,
                )?,
                start..self.lexer.span().end,
            )),

            Token::LeftParen => self.expr_flow(Token::RightParen),
            t => Err(Error::new(ErrorKind::UnexpectedToken(t), self.lexer.span())),
        }
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
                                        Error::unexpected_eof(self.lexer.span().start)
                                    })?,
                                ));

                                break 'cart;
                            }
                            Token::Comma => break buf.take(),
                            t => buf = Some(self.parse_expr(t, &mut buf)?),
                        }
                    }
                    .ok_or_else(|| Error::unexpected_eof(self.lexer.span().start))?;

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
                let next = self
                    .lexer
                    .next()
                    .ok_or_else(|| Error::unexpected_eof(self.lexer.span().start))?;
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
                t => block.push(self.parse(t)?),
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
        let r = loop {
            match self.checked_next()? {
                // Print to stdout
                Token::Print => {
                    let stmt = Stmt::Print(buf.take().ok_or_else(|| {
                        Error::new(ErrorKind::UnexpectedToken(Token::Print), self.lexer.span())
                    })?);
                    break self.semicolon_terminated(stmt)?;
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

                // Variable Assignment
                Token::Equal => {
                    if let Some(Ok(assignable)) = buf.take().map(Assignable::from_expr) {
                        break Stmt::Assign {
                            assignable,
                            value: self.expr_flow(Token::Semicolon)?,
                        };
                    } else {
                        return Err(Error::new(
                            ErrorKind::UnexpectedToken(Token::Equal),
                            self.lexer.span(),
                        ));
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
        };

        Ok(r)
    }

    /// Parse If flow
    ///
    /// Consists of condition and block, there is no else
    fn if_flow(&mut self) -> Result<Stmt, Error> {
        self.require(Token::LeftParen)?;

        let cond = self.expr_flow(Token::RightParen)?;

        let body = self.get_block()?;

        Ok(Stmt::If { cond, body })
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

    /// Parse variable declaration
    fn var_flow(&mut self) -> Result<Stmt, Error> {
        let ident = self.get_ident()?;
        let init = match self.checked_next()? {
            Token::Equal => Some(self.expr_flow(Token::Semicolon)?),
            Token::Semicolon => None,
            t => return Err(Error::new(ErrorKind::UnexpectedToken(t), self.lexer.span())),
        };

        Ok(Stmt::Var { ident, init })
    }

    /// Parse Melo flow
    fn melo_flow(&mut self) -> Result<Stmt, Error> {
        let ident = self.get_ident()?;
        self.semicolon_terminated(Stmt::Melo(ident))
    }

    /// Parse loop flow
    ///
    /// `loop` is an infinite loop, no condition, only body
    fn loop_flow(&mut self) -> Result<Stmt, Error> {
        Ok(Stmt::Loop {
            body: self.get_block()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_math() {
        let code = "1 * (a + 3) / 666 print;";
        let expected = &[Spanned {
            item: Stmt::Print(Spanned {
                item: Expr::BinOp {
                    lhs: Box::new(Spanned {
                        item: Expr::BinOp {
                            lhs: Box::new(Spanned {
                                item: Expr::Literal(Value::Int(1)),
                                span: 0..1,
                            }),
                            rhs: Box::new(Spanned {
                                item: Expr::BinOp {
                                    lhs: Box::new(Spanned {
                                        item: Expr::Variable("a".to_owned()),
                                        span: 5..6,
                                    }),
                                    rhs: Box::new(Spanned {
                                        item: Expr::Literal(Value::Int(3)),
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
                        item: Expr::Literal(Value::Int(666)),
                        span: 14..17,
                    }),
                    kind: BinOpKind::Divide,
                },
                span: 0..17,
            }),
            span: 0..24,
        }];

        let ast = Parser::new(code).init().unwrap();
        assert_eq!(ast, expected);
    }

    #[test]
    fn variable_declaration() {
        let code = "var a = 42;";
        let expected = &[Spanned {
            item: Stmt::Var {
                ident: Spanned {
                    item: "a".to_owned(),
                    span: 4..5,
                },
                init: Some(Spanned {
                    item: Expr::Literal(Value::Int(42)),
                    span: 8..10,
                }),
            },
            span: 0..11,
        }];

        let ast = Parser::new(code).init().unwrap();
        assert_eq!(ast, expected);
    }

    #[test]
    fn if_flow() {
        let code = "if (a == always) { /*Buy Able products!*/ print; }";
        let expected = &[Spanned {
            item: Stmt::If {
                cond: Spanned {
                    item: Expr::BinOp {
                        lhs: Box::new(Spanned {
                            item: Expr::Variable("a".to_owned()),
                            span: 4..5,
                        }),
                        rhs: Box::new(Spanned {
                            item: Expr::Variable("always".to_owned()),
                            span: 9..15,
                        }),
                        kind: BinOpKind::Equal,
                    },
                    span: 4..15,
                },
                body: vec![Spanned {
                    item: Stmt::Print(Spanned {
                        item: Expr::Literal(Value::Str("Buy Able products!".to_owned())),
                        span: 19..39,
                    }),
                    span: 19..46,
                }],
            },
            span: 0..48,
        }];

        let ast = Parser::new(code).init().unwrap();
        assert_eq!(ast, expected);
    }

    #[test]
    fn tdark() {
        let code = "T-Dark { var lang = /*lang*/ + lang; }";
        let expected = &[Spanned {
            item: Stmt::Var {
                ident: Spanned {
                    item: "script".to_owned(),
                    span: 13..17,
                },
                init: Some(Spanned {
                    item: Expr::BinOp {
                        lhs: Box::new(Spanned {
                            item: Expr::Literal(Value::Str("script".to_owned())),
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

        let ast = Parser::new(code).init().unwrap();
        assert_eq!(ast, expected);
    }

    #[test]
    fn cart_construction() {
        let code = "[/*able*/ <= 1, /*script*/ <= 3 - 1] print;";
        let expected = &[Spanned {
            item: Stmt::Print(Spanned {
                item: Expr::Cart(vec![
                    (
                        Spanned {
                            item: Expr::Literal(Value::Str("able".to_owned())),
                            span: 1..7,
                        },
                        Spanned {
                            item: Expr::Literal(Value::Int(1)),
                            span: 11..12,
                        },
                    ),
                    (
                        Spanned {
                            item: Expr::Literal(Value::Str("script".to_owned())),
                            span: 14..22,
                        },
                        Spanned {
                            item: Expr::BinOp {
                                kind: BinOpKind::Subtract,
                                lhs: Box::new(Spanned {
                                    item: Expr::Literal(Value::Int(3)),
                                    span: 26..27,
                                }),
                                rhs: Box::new(Spanned {
                                    item: Expr::Literal(Value::Int(1)),
                                    span: 30..31,
                                }),
                            },
                            span: 26..31,
                        },
                    ),
                ]),
                span: 0..32,
            }),
            span: 0..39,
        }];

        let ast = Parser::new(code).init().unwrap();
        assert_eq!(ast, expected);
    }

    #[test]
    fn cart_index() {
        let code = "[/*able*/ <= /*ablecorp*/][/*ablecorp*/] print;";
        let expected = &[Spanned {
            item: Stmt::Print(Spanned {
                item: Expr::Index {
                    expr: Box::new(Spanned {
                        item: Expr::Cart(vec![(
                            Spanned {
                                item: Expr::Literal(Value::Str("able".to_owned())),
                                span: 1..7,
                            },
                            Spanned {
                                item: Expr::Literal(Value::Str("ablecorp".to_owned())),
                                span: 11..21,
                            },
                        )]),
                        span: 0..22,
                    }),
                    index: Box::new(Spanned {
                        item: Expr::Literal(Value::Str("ablecorp".to_owned())),
                        span: 23..33,
                    }),
                },
                span: 0..34,
            }),
            span: 0..41,
        }];

        let ast = Parser::new(code).init().unwrap();
        assert_eq!(ast, expected);
    }
}
