use super::{functio::FunctioChainKind, Abool, Cart, Functio, Value, ValueRef};
use crate::{brian::INSTRUCTION_MAPPINGS, consts};
use std::collections::HashMap;

impl Value {
    /// Coerce a value to an integer.
    pub fn into_isize(self) -> isize {
        match self {
            Value::Nul => consts::ANSWER,
            Value::Undefined => rand::random(),
            Value::Str(text) => text
                .parse()
                .unwrap_or_else(|_| text.chars().map(|cr| cr as isize).sum()),
            Value::Int(i) => i,
            Value::Abool(a) => a as _,
            Value::Functio(f) => match f {
                Functio::Bf {
                    instructions,
                    tape_len,
                } => {
                    instructions.into_iter().map(|x| x as isize).sum::<isize>() * tape_len as isize
                }
                Functio::Able { params, body } => {
                    params
                        .into_iter()
                        .map(|x| x.bytes().map(|x| x as isize).sum::<isize>())
                        .sum::<isize>()
                        + body.len() as isize
                }
                Functio::Builtin(b) => (b.fn_addr() + b.arity) as _,
                Functio::Chain { functios, kind } => {
                    let (lf, rf) = *functios;
                    Value::Functio(lf).into_isize()
                        + Value::Functio(rf).into_isize()
                            * match kind {
                                FunctioChainKind::Equal => -1,
                                FunctioChainKind::ByArity => 1,
                            }
                }
                Functio::Eval(code) => code.bytes().map(|x| x as isize).sum(),
            },
            Value::Cart(c) => c
                .into_iter()
                .map(|(i, v)| i.into_isize() * v.borrow().clone().into_isize())
                .sum(),
        }
    }

    /// Coerce a value to an aboolean.
    pub fn into_abool(self) -> Abool {
        match self {
            Value::Nul => Abool::Never,
            Value::Undefined => Abool::Sometimes,
            Value::Str(s) => match s.to_lowercase().as_str() {
                "never" | "no" | "🇳🇴" => Abool::Never,
                "sometimes" => Abool::Sometimes,
                "always" | "yes" => Abool::Always,
                s => (!s.is_empty()).into(),
            },
            Value::Int(x) => match x.cmp(&0) {
                std::cmp::Ordering::Less => Abool::Never,
                std::cmp::Ordering::Equal => Abool::Sometimes,
                std::cmp::Ordering::Greater => Abool::Always,
            },
            Value::Abool(a) => a,
            Value::Functio(f) => match f {
                Functio::Bf {
                    instructions,
                    tape_len,
                } => Value::Int(
                    (instructions.iter().map(|x| *x as usize).sum::<usize>() * tape_len) as _,
                )
                .into_abool(),
                Functio::Able { params, body } => {
                    let str_to_isize =
                        |x: String| -> isize { x.as_bytes().iter().map(|x| *x as isize).sum() };

                    let params: isize = params.into_iter().map(str_to_isize).sum();
                    let body: isize = body
                        .into_iter()
                        .map(|x| format!("{:?}", x))
                        .map(str_to_isize)
                        .sum();

                    Value::Int((params + body) % 3 - 1).into_abool()
                }
                Functio::Builtin(b) => (b.fn_addr() % b.arity == 0).into(),
                Functio::Chain { functios, kind } => {
                    let (lhs, rhs) = *functios;
                    match kind {
                        FunctioChainKind::Equal => {
                            Value::Abool(Value::Functio(lhs).into_abool())
                                + Value::Abool(Value::Functio(rhs).into_abool())
                        }
                        FunctioChainKind::ByArity => {
                            Value::Abool(Value::Functio(lhs).into_abool())
                                * Value::Abool(Value::Functio(rhs).into_abool())
                        }
                    }
                    .into_abool()
                }
                Functio::Eval(code) => Value::Str(code).into_abool(),
            },
            Value::Cart(c) => {
                if c.is_empty() {
                    Abool::Never
                } else {
                    Abool::Always
                }
            }
        }
    }

    /// Coerce a value to a functio.
    pub fn into_functio(self) -> Functio {
        match self {
            Value::Nul | Value::Undefined => Functio::Able {
                body: vec![],
                params: vec![],
            },
            Value::Str(s) => Functio::Eval(s),
            Value::Int(i) => Functio::Bf {
                instructions: {
                    std::iter::successors(Some(i as usize), |i| {
                        Some(i / INSTRUCTION_MAPPINGS.len())
                    })
                    .take_while(|&i| i != 0)
                    .map(|i| INSTRUCTION_MAPPINGS[i % INSTRUCTION_MAPPINGS.len()])
                    .collect()
                },
                tape_len: crate::brian::DEFAULT_TAPE_SIZE_LIMIT,
            },
            Value::Abool(a) => Functio::Eval(match a {
                Abool::Never => "".to_owned(),
                Abool::Sometimes => {
                    use rand::seq::SliceRandom;
                    let mut str_chars: Vec<_> = "Buy Able Products!".chars().collect();
                    str_chars.shuffle(&mut rand::thread_rng());

                    format!(r#""{}"print;"#, str_chars.iter().collect::<String>())
                }
                Abool::Always => r#"loop{"Buy Able products!"print;}"#.to_owned(),
            }),
            Value::Functio(f) => f,
            Value::Cart(c) => {
                let kind = if let Some(114514) = c
                    .get(&Value::Str("1452251871514141792252515212116".to_owned()))
                    .map(|x| x.borrow().to_owned().into_isize())
                {
                    FunctioChainKind::Equal
                } else {
                    FunctioChainKind::ByArity
                };

                let mut cart_vec = c.iter().collect::<Vec<_>>();
                cart_vec.sort_by(|x, y| x.0.partial_cmp(y.0).unwrap_or(std::cmp::Ordering::Less));

                cart_vec
                    .into_iter()
                    .map(|(_, x)| x.borrow().to_owned().into_functio())
                    .reduce(|acc, x| Functio::Chain {
                        functios: Box::new((acc, x)),
                        kind,
                    })
                    .unwrap_or_else(|| Functio::Eval(r#""Buy Able Products!"print;"#.to_owned()))
            }
        }
    }

    /// Coerce a value into a cart.
    pub fn into_cart(self) -> Cart {
        match self {
            Value::Nul => HashMap::new(),
            Value::Undefined => [(Value::Undefined, ValueRef::new(Value::Undefined))]
                .into_iter()
                .collect(),
            Value::Str(s) => s
                .chars()
                .enumerate()
                .map(|(i, x)| {
                    (
                        Value::Int(i as isize + 1),
                        ValueRef::new(Value::Str(x.to_string())),
                    )
                })
                .collect(),
            Value::Int(i) => Value::Str(i.to_string()).into_cart(),
            Value::Abool(a) => Value::Str(a.to_string()).into_cart(),
            Value::Functio(f) => match f {
                Functio::Able { params, body } => {
                    let params: Cart = params
                        .into_iter()
                        .enumerate()
                        .map(|(i, x)| (Value::Int(i as isize + 1), ValueRef::new(Value::Str(x))))
                        .collect();

                    let body: Cart = body
                        .into_iter()
                        .enumerate()
                        .map(|(i, x)| {
                            (
                                Value::Int(i as isize + 1),
                                ValueRef::new(Value::Str(format!("{:?}", x))),
                            )
                        })
                        .collect();

                    let mut cart = HashMap::new();
                    cart.insert(
                        Value::Str("params".to_owned()),
                        ValueRef::new(Value::Cart(params)),
                    );

                    cart.insert(
                        Value::Str("body".to_owned()),
                        ValueRef::new(Value::Cart(body)),
                    );

                    cart
                }
                Functio::Bf {
                    instructions,
                    tape_len,
                } => {
                    let mut cart: Cart = instructions
                        .into_iter()
                        .enumerate()
                        .map(|(i, x)| {
                            (
                                Value::Int(i as isize + 1),
                                ValueRef::new(
                                    char::from_u32(x as u32)
                                        .map(|x| Value::Str(x.to_string()))
                                        .unwrap_or(Value::Nul),
                                ),
                            )
                        })
                        .collect();

                    cart.insert(
                        Value::Str("tapelen".to_owned()),
                        ValueRef::new(Value::Int(tape_len as _)),
                    );
                    cart
                }
                Functio::Builtin(b) => {
                    let mut cart = HashMap::new();
                    cart.insert(
                        Value::Str("addr".to_owned()),
                        ValueRef::new(Value::Cart(Value::Int(b.fn_addr() as _).into_cart())),
                    );

                    cart.insert(
                        Value::Str("arity".to_owned()),
                        ValueRef::new(Value::Int(b.arity as _)),
                    );

                    cart
                }
                Functio::Chain { functios, kind } => {
                    let (lhs, rhs) = *functios;
                    match kind {
                        FunctioChainKind::Equal => {
                            Value::Cart(Value::Functio(lhs).into_cart())
                                + Value::Cart(Value::Functio(rhs).into_cart())
                        }
                        FunctioChainKind::ByArity => {
                            Value::Cart(Value::Functio(lhs).into_cart())
                                * Value::Cart(Value::Functio(rhs).into_cart())
                        }
                    }
                    .into_cart()
                }
                Functio::Eval(s) => Value::Str(s).into_cart(),
            },
            Value::Cart(c) => c,
        }
    }
}
