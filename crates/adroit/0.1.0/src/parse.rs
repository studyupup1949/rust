use std::{
    iter::Peekable,
    num::{ParseFloatError, ParseIntError},
    str::FromStr,
};

use indexmap::{IndexMap, IndexSet};
use logos::{Logos, Span, SpannedIter};

use crate::{
    Block, BlockId, Exit, Expr, FieldId, FloatBin, FuncId, Function, Instr, IntBin, IntComp,
    Module, Type, TypeId, Typedef, VarId, VariantId,
};

#[derive(Clone, Copy, Logos)]
#[logos(skip r"(\s|;[^\n]*)+")]
enum Token<'input> {
    #[regex(r"[^\s;()]+")]
    Atom(&'input str),

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,
}

struct Atom<'input> {
    val: &'input str,
    span: Span,
}

#[derive(Debug)]
pub enum ParseError {
    Lex { span: Span },
    Eof,
    Unexpected { span: Span },
    ExpectedAtom { span: Span },
    ExpectedList { span: Span },
    ModuleHead { span: Span },
    DefHead { span: Span },
    TypedefHead { span: Span },
    FieldHead { span: Span },
    VariantHead { span: Span },
    FuncPartHead { span: Span },
    BlockPartHead { span: Span },
    ExprName { span: Span },
    IntParse { span: Span, err: ParseIntError },
    FloatParse { span: Span, err: ParseFloatError },
    ThenHead { span: Span },
    ElseHead { span: Span },
    CaseHead { span: Span },
}

type ParseResult<T> = Result<T, ParseError>;

struct Lexer<'input> {
    tokens: Peekable<SpannedIter<'input, Token<'input>>>,
}

impl<'input> Lexer<'input> {
    fn new(input: &'input str) -> Self {
        Self {
            tokens: Token::lexer(input).spanned().peekable(),
        }
    }

    fn peek(&mut self) -> ParseResult<Option<(Token<'input>, Span)>> {
        match self.tokens.peek() {
            None => Ok(None),
            Some((res, span)) => {
                let span = span.clone();
                match res {
                    Err(()) => Err(ParseError::Lex { span }),
                    &Ok(token) => Ok(Some((token, span))),
                }
            }
        }
    }

    fn next(&mut self) -> ParseResult<(Token<'input>, Span)> {
        match self.tokens.next() {
            None => Err(ParseError::Eof),
            Some((res, span)) => match res {
                Err(()) => Err(ParseError::Lex { span }),
                Ok(token) => Ok((token, span)),
            },
        }
    }

    fn maybe_atom(&mut self) -> ParseResult<Option<Atom<'input>>> {
        match self.peek()? {
            None | Some((Token::LParen | Token::RParen, _)) => Ok(None),
            Some((Token::Atom(val), span)) => {
                self.next().unwrap();
                Ok(Some(Atom { val, span }))
            }
        }
    }

    fn atom(&mut self) -> ParseResult<Atom<'input>> {
        match self.next()? {
            (Token::LParen | Token::RParen, span) => Err(ParseError::ExpectedAtom { span }),
            (Token::Atom(val), span) => Ok(Atom { val, span }),
        }
    }

    fn maybe_list<T>(
        &mut self,
        f: impl FnOnce(Atom<'input>, &mut Self) -> ParseResult<T>,
    ) -> ParseResult<Option<T>> {
        match self.peek()? {
            None | Some((Token::RParen, _)) => return Ok(None),
            Some((Token::Atom(_), span)) => return Err(ParseError::ExpectedList { span }),
            Some((Token::LParen, _)) => {}
        };
        self.next().unwrap();
        let head = self.atom()?;
        let x = f(head, self)?;
        match self.next()? {
            (Token::Atom(_) | Token::LParen, span) => Err(ParseError::Unexpected { span }),
            (Token::RParen, _) => Ok(Some(x)),
        }
    }

    fn list<T>(
        &mut self,
        f: impl FnOnce(Atom<'input>, &mut Self) -> ParseResult<T>,
    ) -> ParseResult<T> {
        self.maybe_list(f)
            .transpose()
            .unwrap_or(Err(ParseError::Eof))
    }
}

impl FromStr for Module {
    type Err = ParseError;

    fn from_str(s: &str) -> ParseResult<Self> {
        Parser::new().parse_module(&mut Lexer::new(s))
    }
}

struct BlockNames<'input> {
    names: IndexSet<&'input str>,
}

impl<'input> BlockNames<'input> {
    fn new() -> Self {
        Self {
            names: IndexSet::new(),
        }
    }

    fn id(&mut self, name: &'input str) -> BlockId {
        let (i, _) = self.names.insert_full(name);
        BlockId(i)
    }
}

struct VarNames<'input> {
    names: IndexSet<&'input str>,
}

impl<'input> VarNames<'input> {
    fn new() -> Self {
        Self {
            names: IndexSet::new(),
        }
    }

    fn id(&mut self, name: &'input str) -> VarId {
        let (i, _) = self.names.insert_full(name);
        VarId(i)
    }
}

struct Parser<'input> {
    types: IndexMap<&'input str, IndexMap<&'input str, Type>>,
    funcs: IndexSet<&'input str>,
}

impl<'input> Parser<'input> {
    fn new() -> Self {
        Self {
            types: IndexMap::new(),
            funcs: IndexSet::new(),
        }
    }

    fn new_type(&mut self, name: &'input str, parts: IndexMap<&'input str, Type>) -> TypeId {
        let (i, _) = self.types.insert_full(name, parts);
        TypeId(i)
    }

    fn type_id(&mut self, name: &'input str) -> TypeId {
        let i = self.types.get_index_of(name).unwrap();
        TypeId(i)
    }

    fn element(&self, id: TypeId) -> Type {
        let (_, &t) = self.types[id.0].get_index(0).unwrap();
        t
    }

    fn field(&self, id: TypeId, name: &'input str) -> (FieldId, Type) {
        let (i, _, &t) = self.types[id.0].get_full(name).unwrap();
        (FieldId(i), t)
    }

    fn variant(&self, id: TypeId, name: &'input str) -> (VariantId, Type) {
        let (i, _, &t) = self.types[id.0].get_full(name).unwrap();
        (VariantId(i), t)
    }

    fn func_id(&mut self, name: &'input str) -> FuncId {
        let (i, _) = self.funcs.insert_full(name);
        FuncId(i)
    }

    fn parse_module(&mut self, l: &mut Lexer<'input>) -> ParseResult<Module> {
        l.list(|head, l| match head.val {
            "module" => {
                let mut types = vec![];
                let mut funcs = vec![];
                while let Some(()) = l.maybe_list(|head, l| match head.val {
                    "type" => {
                        let name = l.atom()?.val;
                        let (def, parts) = self.parse_typedef(l)?;
                        let id = self.new_type(name, parts);
                        types.push((id, def));
                        Ok(())
                    }
                    "func" => {
                        let id = self.func_id(l.atom()?.val);
                        let mut dom = vec![];
                        let mut cod = vec![];
                        let mut entry = None;
                        let mut var_names = VarNames::new();
                        let mut vars = vec![];
                        let mut block_names = BlockNames::new();
                        let mut blocks = vec![];
                        while let Some(()) = l.maybe_list(|head, l| match head.val {
                            "param" => {
                                dom.push(self.parse_type(l)?);
                                Ok(())
                            }
                            "result" => {
                                cod.push(self.parse_type(l)?);
                                Ok(())
                            }
                            "entry" => {
                                entry = Some(block_names.id(l.atom()?.val));
                                Ok(())
                            }
                            "block" => {
                                let id = block_names.id(l.atom()?.val);
                                let mut params = vec![];
                                let mut instrs = vec![];
                                let mut exit = None;
                                while let Some(()) = l.maybe_list(|head, l| match head.val {
                                    "param" => {
                                        let id = var_names.id(l.atom()?.val);
                                        vars.push((id, self.parse_type(l)?));
                                        params.push(id);
                                        Ok(())
                                    }
                                    "let" => {
                                        let mut ids = vec![];
                                        while let Some(atom) = l.maybe_atom()? {
                                            ids.push(var_names.id(atom.val));
                                        }
                                        let (types, expr) = self.parse_expr(l, &mut var_names)?;
                                        for (&x, t) in ids.iter().zip(types) {
                                            vars.push((x, t));
                                        }
                                        instrs.push(Instr { vars: ids, expr });
                                        Ok(())
                                    }
                                    "jump" => {
                                        let to = block_names.id(l.atom()?.val);
                                        let mut values = vec![];
                                        while let Some(atom) = l.maybe_atom()? {
                                            values.push(var_names.id(atom.val));
                                        }
                                        exit = Some(Exit::Jump { to, values });
                                        Ok(())
                                    }
                                    "if" => {
                                        let cond = var_names.id(l.atom()?.val);
                                        let (true_to, true_values) =
                                            l.list(|head, l| match head.val {
                                                "then" => {
                                                    let to = block_names.id(l.atom()?.val);
                                                    let mut values = vec![];
                                                    while let Some(atom) = l.maybe_atom()? {
                                                        values.push(var_names.id(atom.val));
                                                    }
                                                    Ok((to, values))
                                                }
                                                _ => Err(ParseError::ThenHead { span: head.span }),
                                            })?;
                                        let (false_to, false_values) =
                                            l.list(|head, l| match head.val {
                                                "else" => {
                                                    let to = block_names.id(l.atom()?.val);
                                                    let mut values = vec![];
                                                    while let Some(atom) = l.maybe_atom()? {
                                                        values.push(var_names.id(atom.val));
                                                    }
                                                    Ok((to, values))
                                                }
                                                _ => Err(ParseError::ElseHead { span: head.span }),
                                            })?;
                                        exit = Some(Exit::If {
                                            cond,
                                            true_to,
                                            true_values,
                                            false_to,
                                            false_values,
                                        });
                                        Ok(())
                                    }
                                    "match" => {
                                        let on = var_names.id(l.atom()?.val);
                                        let mut cases = vec![];
                                        while let Some(()) =
                                            l.maybe_list(|head, l| match head.val {
                                                "case" => {
                                                    cases.push(block_names.id(l.atom()?.val));
                                                    Ok(())
                                                }
                                                _ => Err(ParseError::CaseHead { span: head.span }),
                                            })?
                                        {}
                                        exit = Some(Exit::Match { on, cases });
                                        Ok(())
                                    }
                                    "return" => {
                                        let mut values = vec![];
                                        while let Some(atom) = l.maybe_atom()? {
                                            values.push(var_names.id(atom.val));
                                        }
                                        exit = Some(Exit::Return { values });
                                        Ok(())
                                    }
                                    _ => Err(ParseError::BlockPartHead { span: head.span }),
                                })? {}
                                let block = Block {
                                    params,
                                    instrs,
                                    exit: exit.unwrap(),
                                };
                                blocks.push((id, block));
                                Ok(())
                            }
                            _ => Err(ParseError::FuncPartHead { span: head.span }),
                        })? {}
                        vars.sort_by_key(|&(id, _)| id);
                        for (i, (id, _)) in vars.iter().enumerate() {
                            assert_eq!(id.0, i);
                        }
                        blocks.sort_by_key(|&(id, _)| id);
                        for (i, (id, _)) in blocks.iter().enumerate() {
                            assert_eq!(id.0, i);
                        }
                        let func = Function {
                            dom,
                            cod,
                            vars: vars.into_iter().map(|(_, t)| t).collect(),
                            blocks: blocks.into_iter().map(|(_, block)| block).collect(),
                            entry: entry.unwrap(),
                        };
                        funcs.push((id, func));
                        Ok(())
                    }
                    _ => Err(ParseError::DefHead { span: head.span }),
                })? {}
                types.sort_by_key(|&(id, _)| id);
                for (i, (id, _)) in types.iter().enumerate() {
                    assert_eq!(id.0, i);
                }
                funcs.sort_by_key(|&(id, _)| id);
                for (i, (id, _)) in funcs.iter().enumerate() {
                    assert_eq!(id.0, i);
                }
                Ok(Module {
                    types: types.into_iter().map(|(_, def)| def).collect(),
                    funcs: funcs.into_iter().map(|(_, func)| func).collect(),
                })
            }
            _ => Err(ParseError::ModuleHead { span: head.span }),
        })
    }

    fn parse_typedef(
        &mut self,
        l: &mut Lexer<'input>,
    ) -> ParseResult<(Typedef, IndexMap<&'input str, Type>)> {
        l.list(|head, l| match head.val {
            "array" => {
                let mut parts = IndexMap::new();
                let element = self.parse_type(l)?;
                parts.insert("", element);
                Ok((Typedef::Array { element }, parts))
            }
            "struct" => {
                let mut fields = vec![];
                let mut parts = IndexMap::new();
                while let Some(()) = l.maybe_list(|head, l| match head.val {
                    "field" => {
                        let name = l.atom()?.val;
                        let t = self.parse_type(l)?;
                        fields.push(t);
                        parts.insert(name, t);
                        Ok(())
                    }
                    _ => Err(ParseError::FieldHead { span: head.span }),
                })? {}
                Ok((Typedef::Struct { fields }, parts))
            }
            "enum" => {
                let mut variants = vec![];
                let mut parts = IndexMap::new();
                while let Some(()) = l.maybe_list(|head, l| match head.val {
                    "variant" => {
                        let name = l.atom()?.val;
                        let t = self.parse_type(l)?;
                        variants.push(t);
                        parts.insert(name, t);
                        Ok(())
                    }
                    _ => Err(ParseError::VariantHead { span: head.span }),
                })? {}
                Ok((Typedef::Enum { variants }, parts))
            }
            _ => Err(ParseError::TypedefHead { span: head.span }),
        })
    }

    fn parse_type(&mut self, l: &mut Lexer<'input>) -> ParseResult<Type> {
        match l.atom()?.val {
            "bool" => Ok(Type::Bool),
            "int" => Ok(Type::Int),
            "float" => Ok(Type::Float),
            name => Ok(Type::Ref {
                id: self.type_id(name),
            }),
        }
    }

    fn parse_expr(
        &mut self,
        l: &mut Lexer<'input>,
        var_names: &mut VarNames<'input>,
    ) -> ParseResult<(Vec<Type>, Expr)> {
        l.list(|head, l| {
            let op = head.val;
            match op {
                "int.const" => Ok((
                    vec![Type::Int],
                    Expr::IntConst {
                        val: l.atom()?.val.parse().map_err(|err| ParseError::IntParse {
                            span: head.span,
                            err,
                        })?,
                    },
                )),
                "int.lt" => Ok((
                    vec![Type::Bool],
                    Expr::IntComp {
                        op: match op {
                            "int.lt" => IntComp::Lt,
                            _ => unreachable!(),
                        },
                        x: var_names.id(l.atom()?.val),
                        y: var_names.id(l.atom()?.val),
                    },
                )),
                "int.add" => Ok((
                    vec![Type::Int],
                    Expr::IntBin {
                        op: match op {
                            "int.add" => IntBin::Add,
                            _ => unreachable!(),
                        },
                        x: var_names.id(l.atom()?.val),
                        y: var_names.id(l.atom()?.val),
                    },
                )),
                "float.const" => Ok((
                    vec![Type::Float],
                    Expr::FloatConst {
                        val: l
                            .atom()?
                            .val
                            .parse()
                            .map_err(|err| ParseError::FloatParse {
                                span: head.span,
                                err,
                            })?,
                    },
                )),
                "float.add" | "float.sub" => Ok((
                    vec![Type::Float],
                    Expr::FloatBin {
                        op: match op {
                            "float.add" => FloatBin::Add,
                            "float.sub" => FloatBin::Sub,
                            _ => unreachable!(),
                        },
                        x: var_names.id(l.atom()?.val),
                        y: var_names.id(l.atom()?.val),
                    },
                )),
                "array.len" => Ok((
                    vec![Type::Int],
                    Expr::ArrayLen {
                        id: self.type_id(l.atom()?.val),
                        var: var_names.id(l.atom()?.val),
                    },
                )),
                "array.get" => Ok({
                    let id = self.type_id(l.atom()?.val);
                    let t = self.element(id);
                    let var = var_names.id(l.atom()?.val);
                    let index = var_names.id(l.atom()?.val);
                    (vec![t], Expr::ArrayGet { id, var, index })
                }),
                "struct.make" => Ok({
                    let id = self.type_id(l.atom()?.val);
                    let mut fields = vec![];
                    while let Some(atom) = l.maybe_atom()? {
                        fields.push(var_names.id(atom.val));
                    }
                    (vec![Type::Ref { id }], Expr::StructMake { id, fields })
                }),
                "struct.get" => Ok({
                    let id = self.type_id(l.atom()?.val);
                    let (field, t) = self.field(id, l.atom()?.val);
                    let var = var_names.id(l.atom()?.val);
                    (vec![t], Expr::StructGet { id, field, var })
                }),
                "enum.make" => Ok({
                    let id = self.type_id(l.atom()?.val);
                    let (variant, _) = self.variant(id, l.atom()?.val);
                    let inner = var_names.id(l.atom()?.val);
                    (
                        vec![Type::Ref { id }],
                        Expr::EnumMake { id, variant, inner },
                    )
                }),
                _ => Err(ParseError::ExprName { span: head.span }),
            }
        })
    }
}
