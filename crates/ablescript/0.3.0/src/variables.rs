use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    fmt::Display,
    hash::Hash,
    io::Write,
    mem::discriminant,
    ops,
    rc::Rc,
    vec,
};

use rand::Rng;

use crate::{ast::Block, brian::INSTRUCTION_MAPPINGS, consts};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Abool {
    Never = -1,
    Sometimes = 0,
    Always = 1,
}

impl Abool {
    pub fn to_bool(&self) -> bool {
        match self {
            Self::Always => true,
            Self::Sometimes if rand::random() => true,
            _ => false,
        }
    }
}
impl Display for Abool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Abool::Never => write!(f, "never"),
            Abool::Sometimes => write!(f, "sometimes"),
            Abool::Always => write!(f, "always"),
        }
    }
}

impl From<bool> for Abool {
    fn from(b: bool) -> Self {
        if b {
            Abool::Always
        } else {
            Abool::Never
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum Functio {
    Bf {
        instructions: Vec<u8>,
        tape_len: usize,
    },
    Able {
        params: Vec<String>,
        body: Block,
    },
    Builtin(BuiltinFunctio),
    Chain {
        functios: Box<(Functio, Functio)>,
        kind: FunctioChainKind,
    },
    Eval(String),
}

impl Functio {
    pub fn arity(&self) -> usize {
        match self {
            Functio::Bf {
                instructions: _,
                tape_len: _,
            } => 0,
            Functio::Able { params, body: _ } => params.len(),
            Functio::Builtin(b) => b.arity,
            Functio::Chain { functios, kind: _ } => functios.0.arity() + functios.1.arity(),
            Functio::Eval(_) => 0,
        }
    }
}

#[derive(Clone)]
pub struct BuiltinFunctio {
    function: Rc<dyn Fn(&[ValueRef]) -> Result<(), crate::error::ErrorKind>>,
    arity: usize,
}

impl BuiltinFunctio {
    pub fn new<F>(f: F, arity: usize) -> Self
    where
        F: Fn(&[ValueRef]) -> Result<(), crate::error::ErrorKind> + 'static,
    {
        Self {
            function: Rc::new(f),
            arity,
        }
    }

    pub fn call(&self, args: &[ValueRef]) -> Result<(), crate::error::ErrorKind> {
        (self.function)(args)
    }

    pub fn fn_addr(&self) -> usize {
        Rc::as_ptr(&self.function) as *const () as _
    }
}

impl std::fmt::Debug for BuiltinFunctio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuiltinFunctio")
            .field("function", &"built-in")
            .field("arity", &self.arity)
            .finish()
    }
}

impl PartialEq for BuiltinFunctio {
    fn eq(&self, other: &Self) -> bool {
        self.fn_addr() == other.fn_addr() && self.arity == other.arity
    }
}

impl Hash for BuiltinFunctio {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.fn_addr().hash(state);
        self.arity.hash(state);
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Hash)]
pub enum FunctioChainKind {
    Equal,
    ByArity,
}

pub type Cart = HashMap<Value, ValueRef>;

#[derive(Debug, Clone)]
pub enum Value {
    Nul,
    Str(String),
    Int(isize),
    Abool(Abool),
    Functio(Functio),
    Cart(Cart),
}

impl Default for Value {
    fn default() -> Self {
        Self::Nul
    }
}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        discriminant(self).hash(state);
        match self {
            Value::Nul => (),
            Value::Str(v) => v.hash(state),
            Value::Int(v) => v.hash(state),
            Value::Abool(v) => v.to_string().hash(state),
            Value::Functio(statements) => statements.hash(state),
            Value::Cart(_) => self.to_string().hash(state),
        }
    }
}

impl Value {
    /// Write an AbleScript value to a Brainfuck input stream by
    /// coercing the value to an integer, then truncating that integer
    /// to a single byte, then writing that byte. This should only be
    /// called on `Write`rs that cannot fail, e.g., `Vec<u8>`, because
    /// any IO errors will cause a panic.
    pub fn bf_write(&self, stream: &mut impl Write) {
        stream
            .write_all(&[self.clone().into_isize() as u8])
            .expect("Failed to write to Brainfuck input");
    }

    /// Coerce a value to an integer.
    pub fn into_isize(self) -> isize {
        match self {
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
            Value::Int(i) => i,
            Value::Nul => consts::ANSWER,
            Value::Str(text) => text.parse().unwrap_or(consts::ANSWER),
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
            Value::Nul => Functio::Able {
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

    /// Get a length of a value
    pub fn length(&self) -> isize {
        match self {
            Value::Nul => 0,
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

impl ops::Add for Value {
    type Output = Value;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Value::Nul => match rhs {
                Value::Nul => Value::Nul,
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

impl ops::Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            Value::Nul => match rhs {
                Value::Nul => Value::Nul,
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

impl ops::Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            Value::Nul => match rhs {
                Value::Nul => Value::Nul,
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

impl ops::Div for Value {
    type Output = Value;

    fn div(self, rhs: Self) -> Self::Output {
        match self {
            Value::Nul => match rhs {
                Value::Nul => Value::Nul,
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
                let cart_len = c.len();
                let chunk_len = rhs.into_isize() as usize;

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

impl ops::Not for Value {
    type Output = Value;

    fn not(self) -> Self::Output {
        match self {
            Value::Nul => Value::Nul,
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
            Value::Nul => {
                if other == Value::Nul {
                    Some(Equal)
                } else {
                    None
                }
            }
            Value::Str(s) => Some(s.cmp(&other.to_string())),
            Value::Int(i) => Some(i.cmp(&other.into_isize())),
            Value::Abool(a) => a.partial_cmp(&other.into_abool()),
            Value::Functio(_) => self.clone().into_isize().partial_cmp(&other.into_isize()),
            Value::Cart(c) => Some(c.len().cmp(&other.into_cart().len())),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nul => write!(f, "nul"),
            Value::Str(v) => write!(f, "{}", v),
            Value::Int(v) => write!(f, "{}", v),
            Value::Abool(v) => write!(f, "{}", v),
            Value::Functio(v) => match v {
                Functio::Bf {
                    instructions,
                    tape_len,
                } => {
                    write!(
                        f,
                        "({}) {}",
                        tape_len,
                        String::from_utf8(instructions.to_owned())
                            .expect("Brainfuck functio source should be UTF-8")
                    )
                }
                Functio::Able { params, body } => {
                    write!(
                        f,
                        "({}) -> {:?}",
                        params.join(", "),
                        // Maybe we should have a pretty-printer for
                        // statement blocks at some point?
                        body,
                    )
                }
                Functio::Builtin(b) => write!(f, "builtin @ {}", b.fn_addr()),
                Functio::Chain { functios, kind } => {
                    let (a, b) = *functios.clone();
                    write!(
                        f,
                        "{} {} {} ",
                        Value::Functio(a),
                        match kind {
                            FunctioChainKind::Equal => '+',
                            FunctioChainKind::ByArity => '*',
                        },
                        Value::Functio(b)
                    )
                }
                Functio::Eval(s) => write!(f, "{}", s),
            },
            Value::Cart(c) => {
                write!(f, "[")?;
                let mut cart_vec = c.iter().collect::<Vec<_>>();
                cart_vec.sort_by(|x, y| x.0.partial_cmp(y.0).unwrap_or(std::cmp::Ordering::Less));

                for (idx, (key, value)) in cart_vec.into_iter().enumerate() {
                    write!(
                        f,
                        "{}{} <= {}",
                        if idx != 0 { ", " } else { "" },
                        value.borrow(),
                        key
                    )?;
                }

                write!(f, "]")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueRef(Rc<RefCell<Value>>);

impl ValueRef {
    pub fn new(v: Value) -> Self {
        Self(Rc::new(RefCell::new(v)))
    }

    pub fn borrow(&self) -> Ref<Value> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<Value> {
        self.0.borrow_mut()
    }

    pub fn replace(&self, v: Value) -> Value {
        self.0.replace(v)
    }
}

#[derive(Debug)]
pub struct Variable {
    pub melo: bool,

    // Multiple Variables can reference the same underlying Value when
    // pass-by-reference is used, therefore we use Rc here.
    pub value: ValueRef,
}

impl Variable {
    pub fn from_value(value: Value) -> Self {
        Self {
            melo: false,
            value: ValueRef::new(value),
        }
    }
}
