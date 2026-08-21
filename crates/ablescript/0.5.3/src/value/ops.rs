use super::{
    functio::{BuiltinFunctio, FunctioChainKind},
    Abool, Functio, Value, ValueRef,
};
use crate::consts;
use rand::Rng;
use std::{
    collections::HashMap,
    ops::{Add, Div, Mul, Not, Sub},
};

impl Add for Value {
    type Output = Value;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Value::Nul | Value::Undefined => match rhs {
                Value::Nul => Value::Nul,
                Value::Undefined => Value::Undefined,
                Value::Str(_) => Value::Str(self.to_string()) + rhs,
                Value::Int(_) => Value::Int(self.into_isize()) + rhs,
                Value::Abool(_) => Value::Abool(self.into_abool()) + rhs,
                Value::Functio(_) => Value::Functio(self.into_functio()) + rhs,
                Value::Cart(_) => Value::Cart(self.into_cart()) + rhs,
            },
            Value::Str(s) => Value::Str(format!("{s}{rhs}")),
            Value::Int(i) => Value::Int(i.wrapping_add(rhs.into_isize())),
            Value::Abool(_) => {
                Value::Abool(Value::Int(self.into_isize().max(rhs.into_isize())).into_abool())
            }
            Value::Functio(f) => Value::Functio(Functio::Chain {
                functios: Box::new((f, rhs.into_functio())),
                kind: FunctioChainKind::Equal,
            }),
            Value::Cart(c) => {
                Value::Cart(c.into_iter().chain(rhs.into_cart().into_iter()).collect())
            }
        }
    }
}

impl Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            Value::Nul | Value::Undefined => match rhs {
                Value::Nul => Value::Nul,
                Value::Undefined => Value::Undefined,
                Value::Str(_) => Value::Str(self.to_string()) - rhs,
                Value::Int(_) => Value::Int(self.into_isize()) - rhs,
                Value::Abool(_) => Value::Abool(self.into_abool()) - rhs,
                Value::Functio(_) => Value::Functio(self.into_functio()) - rhs,
                Value::Cart(_) => Value::Cart(self.into_cart()) - rhs,
            },
            Value::Str(s) => Value::Str(s.replace(&rhs.to_string(), "")),
            Value::Int(i) => Value::Int(i.wrapping_sub(rhs.into_isize())),
            Value::Abool(_) => (self.clone() + rhs.clone()) * !(self * rhs),
            Value::Functio(f) => Value::Functio(match f {
                Functio::Bf {
                    instructions: lhs_ins,
                    tape_len: lhs_tl,
                } => match rhs.into_functio() {
                    Functio::Bf {
                        instructions: rhs_ins,
                        tape_len: rhs_tl,
                    } => Functio::Bf {
                        instructions: lhs_ins
                            .into_iter()
                            .zip(rhs_ins.into_iter())
                            .filter_map(|(l, r)| if l != r { Some(l) } else { None })
                            .collect(),
                        tape_len: lhs_tl - rhs_tl,
                    },
                    rhs => Functio::Bf {
                        instructions: lhs_ins
                            .into_iter()
                            .zip(Value::Functio(rhs).to_string().bytes())
                            .filter_map(|(l, r)| if l != r { Some(l) } else { None })
                            .collect(),
                        tape_len: lhs_tl,
                    },
                },
                Functio::Able {
                    params: lhs_params,
                    body: lhs_body,
                } => match rhs.into_functio() {
                    Functio::Able {
                        params: rhs_params,
                        body: rhs_body,
                    } => Functio::Able {
                        params: lhs_params
                            .into_iter()
                            .zip(rhs_params.into_iter())
                            .filter_map(|(l, r)| if l != r { Some(l) } else { None })
                            .collect(),
                        body: lhs_body
                            .into_iter()
                            .zip(rhs_body.into_iter())
                            .filter_map(|(l, r)| if l != r { Some(l) } else { None })
                            .collect(),
                    },
                    rhs => Value::Int(
                        Value::Functio(Functio::Able {
                            params: lhs_params,
                            body: lhs_body,
                        })
                        .into_isize()
                            - Value::Functio(rhs).into_isize(),
                    )
                    .into_functio(),
                },
                Functio::Builtin(b) => {
                    let arity = b.arity;
                    let resulting_arity = arity.saturating_sub(rhs.into_isize() as usize);

                    Functio::Builtin(BuiltinFunctio::new(
                        move |args| {
                            b.call(
                                &args
                                    .iter()
                                    .take(resulting_arity)
                                    .cloned()
                                    .chain(std::iter::repeat_with(|| ValueRef::new(Value::Nul)))
                                    .take(arity)
                                    .collect::<Vec<_>>(),
                            )
                        },
                        resulting_arity,
                    ))
                }
                Functio::Chain { functios, .. } => {
                    let rhs = rhs.into_functio();
                    let (a, b) = *functios;

                    match (a == rhs, b == rhs) {
                        (_, true) => a,
                        (true, _) => b,
                        (_, _) => (Value::Functio(a) - Value::Functio(rhs)).into_functio(),
                    }
                }
                Functio::Eval(lhs_code) => Functio::Eval(lhs_code.replace(
                    &match rhs.into_functio() {
                        Functio::Eval(code) => code,
                        rhs => Value::Functio(rhs).to_string(),
                    },
                    "",
                )),
            }),
            Value::Cart(c) => Value::Cart({
                let rhs_cart = rhs.into_cart();
                c.into_iter()
                    .filter(|(k, v)| rhs_cart.get(k) != Some(v))
                    .collect()
            }),
        }
    }
}

impl Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            Value::Nul | Value::Undefined => match rhs {
                Value::Nul => Value::Nul,
                Value::Undefined => Value::Undefined,
                Value::Str(_) => Value::Str(self.to_string()) * rhs,
                Value::Int(_) => Value::Int(self.into_isize()) * rhs,
                Value::Abool(_) => Value::Abool(self.into_abool()) * rhs,
                Value::Functio(_) => Value::Functio(self.into_functio()) * rhs,
                Value::Cart(_) => Value::Cart(self.into_cart()) * rhs,
            },
            Value::Str(s) => Value::Str(s.repeat(rhs.into_isize() as usize)),
            Value::Int(i) => Value::Int(i.wrapping_mul(rhs.into_isize())),
            Value::Abool(_) => {
                Value::Abool(Value::Int(self.into_isize().min(rhs.into_isize())).into_abool())
            }
            Value::Functio(f) => Value::Functio(Functio::Chain {
                functios: Box::new((f, rhs.into_functio())),
                kind: FunctioChainKind::ByArity,
            }),
            Value::Cart(c) => {
                let rhsc = rhs.into_cart();

                Value::Cart(
                    c.into_iter()
                        .map(|(k, v)| {
                            if let Some(k) = rhsc.get(&k) {
                                (k.borrow().clone(), v)
                            } else {
                                (k, v)
                            }
                        })
                        .collect(),
                )
            }
        }
    }
}

impl Div for Value {
    type Output = Value;

    fn div(self, rhs: Self) -> Self::Output {
        match self {
            Value::Nul | Value::Undefined => match rhs {
                Value::Nul => Value::Nul,
                Value::Undefined => Value::Undefined,
                Value::Str(_) => Value::Str(self.to_string()) / rhs,
                Value::Int(_) => Value::Int(self.into_isize()) / rhs,
                Value::Abool(_) => Value::Abool(self.into_abool()) / rhs,
                Value::Functio(_) => Value::Functio(self.into_functio()) / rhs,
                Value::Cart(_) => Value::Cart(self.into_cart()) / rhs,
            },
            Value::Str(s) => Value::Cart(
                s.split(&rhs.to_string())
                    .enumerate()
                    .map(|(i, x)| {
                        (
                            Value::Int(i as isize + 1),
                            ValueRef::new(Value::Str(x.to_owned())),
                        )
                    })
                    .collect(),
            ),
            Value::Int(i) => Value::Int(i.wrapping_div(match rhs.into_isize() {
                0 => consts::ANSWER,
                x => x,
            })),
            Value::Abool(_) => !self + rhs,
            Value::Functio(f) => Value::Functio(match f {
                Functio::Bf {
                    instructions,
                    tape_len,
                } => {
                    let fraction = 1.0 / rhs.into_isize() as f64;
                    let len = instructions.len();
                    Functio::Bf {
                        instructions: instructions
                            .into_iter()
                            .take((len as f64 * fraction) as usize)
                            .collect(),
                        tape_len,
                    }
                }
                Functio::Able { params, body } => {
                    let fraction = 1.0 / rhs.into_isize() as f64;
                    let len = body.len();
                    Functio::Able {
                        params,
                        body: body
                            .into_iter()
                            .take((len as f64 * fraction) as usize)
                            .collect(),
                    }
                }
                Functio::Builtin(b) => Functio::Builtin(BuiltinFunctio {
                    arity: b.arity + rhs.into_isize() as usize,
                    ..b
                }),
                Functio::Chain { functios, kind } => {
                    let functios = *functios;
                    Functio::Chain {
                        functios: Box::new((
                            (Value::Functio(functios.0) / rhs.clone()).into_functio(),
                            (Value::Functio(functios.1) / rhs).into_functio(),
                        )),
                        kind,
                    }
                }
                Functio::Eval(s) => {
                    let fraction = 1.0 / rhs.into_isize() as f64;
                    let len = s.len();
                    Functio::Eval(s.chars().take((len as f64 * fraction) as usize).collect())
                }
            }),
            Value::Cart(c) => {
                let cart_len = match c.len() {
                    0 => return Value::Cart(HashMap::new()),
                    l => l,
                };

                let chunk_len = match rhs.into_isize() as usize {
                    0 => rand::thread_rng().gen_range(1..=cart_len),
                    l => l,
                };

                Value::Cart(
                    c.into_iter()
                        .collect::<Vec<_>>()
                        .chunks(cart_len / chunk_len + (cart_len % chunk_len != 0) as usize)
                        .enumerate()
                        .map(|(k, v)| {
                            (
                                Value::Int(k as isize + 1),
                                ValueRef::new(Value::Cart(v.iter().cloned().collect())),
                            )
                        })
                        .collect(),
                )
            }
        }
    }
}

impl Not for Value {
    type Output = Value;

    fn not(self) -> Self::Output {
        match self {
            Value::Nul => Value::Undefined,
            Value::Undefined => Value::Nul,
            Value::Str(s) => Value::Str(s.chars().rev().collect()),
            Value::Int(i) => Value::Int(i.swap_bytes()),
            Value::Abool(a) => Value::Abool(match a {
                Abool::Never => Abool::Always,
                Abool::Sometimes => Abool::Sometimes,
                Abool::Always => Abool::Never,
            }),
            Value::Functio(f) => Value::Functio(match f {
                Functio::Bf {
                    mut instructions,
                    tape_len,
                } => {
                    instructions.reverse();

                    Functio::Bf {
                        instructions,
                        tape_len,
                    }
                }
                Functio::Able {
                    mut params,
                    mut body,
                } => {
                    params.reverse();
                    body.reverse();

                    Functio::Able { params, body }
                }
                Functio::Builtin(b) => {
                    let arity = b.arity;
                    Functio::Builtin(BuiltinFunctio::new(
                        move |args| b.call(&args.iter().cloned().rev().collect::<Vec<_>>()),
                        arity,
                    ))
                }
                Functio::Chain { functios, kind } => {
                    let (a, b) = *functios;
                    Functio::Chain {
                        functios: Box::new((
                            (!Value::Functio(b)).into_functio(),
                            (!Value::Functio(a)).into_functio(),
                        )),
                        kind,
                    }
                }
                Functio::Eval(code) => Functio::Eval(code.chars().rev().collect()),
            }),
            Value::Cart(c) => Value::Cart(
                c.into_iter()
                    .map(|(k, v)| (v.borrow().clone(), ValueRef::new(k)))
                    .collect(),
            ),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        let other = other.clone();

        match self {
            Value::Nul => matches!(other, Value::Nul),
            Value::Undefined => matches!(other, Value::Undefined),
            Value::Str(s) => *s == other.to_string(),
            Value::Int(i) => *i == other.into_isize(),
            Value::Abool(a) => *a == other.into_abool(),
            Value::Functio(f) => *f == other.into_functio(),
            Value::Cart(c) => *c == other.into_cart(),
        }
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering::*;
        let other = other.clone();

        match self {
            Value::Nul if other == Value::Nul => Some(Equal),
            Value::Undefined if other == Value::Undefined => Some(Equal),
            Value::Str(s) => Some(s.cmp(&other.to_string())),
            Value::Int(i) => Some(i.cmp(&other.into_isize())),
            Value::Abool(a) => a.partial_cmp(&other.into_abool()),
            Value::Functio(_) => self.clone().into_isize().partial_cmp(&other.into_isize()),
            Value::Cart(c) => Some(c.len().cmp(&other.into_cart().len())),
            Value::Nul | Value::Undefined => None,
        }
    }
}

impl Value {
    /// Get a length of a value
    pub fn length(&self) -> isize {
        match self {
            Value::Nul => 0,
            Value::Undefined => -1,
            Value::Str(s) => s.len() as _,
            Value::Int(i) => i.count_zeros() as _,
            Value::Abool(a) => match a {
                Abool::Never => -3,
                Abool::Sometimes => {
                    if rand::thread_rng().gen() {
                        3
                    } else {
                        -3
                    }
                }
                Abool::Always => 3,
            },
            Value::Functio(f) => match f {
                // Compares lengths of functions:
                // BfFunctio - Sum of lengths of instructions and length of tape
                // AbleFunctio - Sum of argument count and body length
                // Eval - Length of input code
                Functio::Bf {
                    instructions,
                    tape_len,
                } => (instructions.len() + tape_len) as _,
                Functio::Able { params, body } => (params.len() + format!("{:?}", body).len()) as _,
                Functio::Builtin(b) => (std::mem::size_of_val(b.function.as_ref()) + b.arity) as _,
                Functio::Chain { functios, kind } => {
                    let (lhs, rhs) = *functios.clone();
                    match kind {
                        FunctioChainKind::Equal => {
                            Value::Int(Value::Functio(lhs).into_isize())
                                + Value::Int(Value::Functio(rhs).into_isize())
                        }
                        FunctioChainKind::ByArity => {
                            Value::Int(Value::Functio(lhs).into_isize())
                                * Value::Int(Value::Functio(rhs).into_isize())
                        }
                    }
                    .into_isize()
                }
                Functio::Eval(s) => s.len() as _,
            },
            Value::Cart(c) => c.len() as _,
        }
    }
}
