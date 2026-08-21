//! Expression evaluator and statement interpreter.
//!
//! To interpret a piece of AbleScript code, you first need to
//! construct an [ExecEnv], which is responsible for storing the stack
//! of local variable and function definitions accessible from an
//! AbleScript snippet. You can then call [ExecEnv::eval_stmts] to
//! evaluate or execute any number of expressions or statements.

#![deny(missing_docs)]

use crate::{
    ast::{Assignable, AssignableKind, Block, Expr, Spanned, Stmt},
    consts::ablescript_consts,
    error::{Error, ErrorKind},
    host_interface::HostInterface,
    value::{Functio, Value, ValueRef, Variable},
};
use rand::random;
use std::{
    cmp::Ordering,
    collections::{HashMap, VecDeque},
    mem::take,
    ops::Range,
};

/// An environment for executing AbleScript code.
pub struct ExecEnv<H> {
    /// The stack, ordered such that `stack[stack.len() - 1]` is the
    /// top-most (newest) stack frame, and `stack[0]` is the
    /// bottom-most (oldest) stack frame.
    stack: Vec<Scope>,

    /// The `read` statement maintains a buffer of up to 7 bits,
    /// because input comes from the operating system 8 bits at a time
    /// (via stdin) but gets delivered to AbleScript 3 bits at a time
    /// (via the `read` statement). We store each of those bits as
    /// booleans to facilitate easy manipulation.
    read_buf: VecDeque<bool>,

    /// Interface to interact with the host interface
    host_interface: H,

    /// Vector of blocks to be executed at the end of the program
    finalisers: Vec<Block>,
}

/// A set of visible variable and function definitions in a single
/// stack frame.
struct Scope {
    /// The mapping from variable names to values.
    variables: HashMap<String, Variable>,
}

impl<H> Default for ExecEnv<H>
where
    H: Default + HostInterface,
{
    fn default() -> Self {
        Self::with_host_interface(H::default())
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            variables: ablescript_consts(),
        }
    }
}

/// The reason a successful series of statements halted.
enum HaltStatus {
    /// We ran out of statements to execute.
    Finished,

    /// An `enough` statement occurred at the given span, and was not
    /// caught by a `loop` statement up to this point.
    Enough(Range<usize>),

    /// A `and again` statement occurred at the given span, and was not
    /// caught by a `loop` statement up to this point.
    AndAgain(Range<usize>),
}

/// The number of bits the `read` statement reads at once from
/// standard input.
pub const READ_BITS: u8 = 3;

impl<H: HostInterface> ExecEnv<H> {
    /// Create a new Scope with no predefined variable definitions or
    /// other information.
    pub fn with_host_interface(host_interface: H) -> Self {
        Self {
            stack: vec![Default::default()],
            read_buf: Default::default(),
            finalisers: vec![],
            host_interface,
        }
    }

    /// Create a new Scope with predefined variables
    pub fn new_with_vars<I>(host_interface: H, vars: I) -> Self
    where
        I: IntoIterator<Item = (String, Variable)>,
    {
        let scope = Scope {
            variables: ablescript_consts().into_iter().chain(vars).collect(),
        };

        Self {
            stack: vec![scope],
            read_buf: Default::default(),
            finalisers: vec![],
            host_interface,
        }
    }

    /// Execute a set of Statements in the root stack frame. Return an
    /// error if one or more of the Stmts failed to evaluate, or if a
    /// `enough` or `and again` statement occurred at the top level.
    pub fn eval_stmts(&mut self, stmts: &[Spanned<Stmt>]) -> Result<(), Error> {
        match self.eval_stmts_hs(stmts, false)? {
            HaltStatus::Finished => Ok(()),
            HaltStatus::Enough(span) | HaltStatus::AndAgain(span) => Err(Error {
                // It's an error to issue a `enough` outside of a
                // `loop` statement.
                kind: ErrorKind::LoopOpOutsideLoop,
                span,
            }),
        }?;

        for block in std::mem::take(&mut self.finalisers) {
            self.eval_stmts_hs(&block, true)?;
        }

        Ok(())
    }

    /// The same as `eval_stmts`, but report "enough" and "and again"
    /// exit codes as normal conditions in a HaltStatus enum, and
    /// create a new stack frame if `stackframe` is true.
    ///
    /// `interpret`-internal code should typically prefer this
    /// function over `eval_stmts`.
    fn eval_stmts_hs(
        &mut self,
        stmts: &[Spanned<Stmt>],
        stackframe: bool,
    ) -> Result<HaltStatus, Error> {
        let init_depth = self.stack.len();

        if stackframe {
            self.stack.push(Default::default());
        }

        let mut final_result = Ok(HaltStatus::Finished);
        for stmt in stmts {
            final_result = self.eval_stmt(stmt);
            if !matches!(final_result, Ok(HaltStatus::Finished)) {
                break;
            }
        }

        if stackframe {
            self.stack.pop();
        }

        // Invariant: stack size must have net 0 change.
        debug_assert_eq!(self.stack.len(), init_depth);
        final_result
    }

    /// Evaluate an Expr, returning its value or an error.
    fn eval_expr(&self, expr: &Spanned<Expr>) -> Result<Value, Error> {
        use crate::ast::BinOpKind::*;
        use crate::ast::Expr::*;

        Ok(match &expr.item {
            BinOp { lhs, rhs, kind } => {
                let lhs = self.eval_expr(lhs)?;
                let rhs = self.eval_expr(rhs)?;
                match kind {
                    Add => lhs + rhs,
                    Subtract => lhs - rhs,
                    Multiply => lhs * rhs,
                    Divide => lhs / rhs,
                    Greater => Value::Abool((lhs > rhs).into()),
                    Less => Value::Abool((lhs < rhs).into()),
                    Equal => Value::Abool((lhs == rhs).into()),
                    NotEqual => Value::Abool((lhs != rhs).into()),
                }
            }
            Aint(expr) => !self.eval_expr(expr)?,
            Literal(lit) => lit.clone().into(),
            Expr::Cart(members) => Value::Cart(
                members
                    .iter()
                    .map(|(value, key)| {
                        self.eval_expr(value).and_then(|value| {
                            self.eval_expr(key).map(|key| (key, ValueRef::new(value)))
                        })
                    })
                    .collect::<Result<HashMap<_, _>, _>>()?,
            ),
            Index { expr, index } => {
                let value = self.eval_expr(expr)?;
                let index = self.eval_expr(index)?;

                value
                    .into_cart()
                    .get(&index)
                    .map(|x| x.borrow().clone())
                    .unwrap_or(Value::Nul)
            }
            Len(expr) => Value::Int(self.eval_expr(expr)?.length()),
            Keys(expr) => Value::Cart(
                self.eval_expr(expr)?
                    .into_cart()
                    .into_keys()
                    .enumerate()
                    .map(|(i, k)| (Value::Int(i as isize + 1), ValueRef::new(k)))
                    .collect(),
            ),
            // TODO: not too happy with constructing an artificial
            // Ident here.
            Variable(name) => {
                self.get_var_value(&Spanned::new(name.to_owned(), expr.span.clone()))?
            }
        })
    }

    /// Perform the action indicated by a statement.
    fn eval_stmt(&mut self, stmt: &Spanned<Stmt>) -> Result<HaltStatus, Error> {
        match &stmt.item {
            Stmt::Print { expr, newline } => {
                let value = self.eval_expr(expr)?;
                self.host_interface
                    .print(&value.to_string(), *newline)
                    .map_err(|e| Error::new(e.into(), stmt.span.clone()))?;
            }
            Stmt::Dim { ident, init } => {
                let init = match init {
                    Some(e) => self.eval_expr(e)?,
                    None => Value::Nul,
                };

                self.decl_var(&ident.item, init);
            }
            Stmt::Functio {
                ident,
                params,
                body,
            } => {
                self.decl_var(
                    &ident.item,
                    Value::Functio(Functio::Able {
                        params: params.iter().map(|ident| ident.item.to_owned()).collect(),
                        body: body.to_owned(),
                    }),
                );
            }
            Stmt::BfFunctio {
                ident,
                tape_len,
                code,
            } => {
                self.decl_var(
                    &ident.item,
                    Value::Functio(Functio::Bf {
                        instructions: code.to_owned(),
                        tape_len: tape_len
                            .as_ref()
                            .map(|tape_len| {
                                self.eval_expr(tape_len).map(|v| v.into_isize() as usize)
                            })
                            .unwrap_or(Ok(crate::brian::DEFAULT_TAPE_SIZE_LIMIT))?,
                    }),
                );
            }
            Stmt::Unless { cond, body } => {
                if !self.eval_expr(cond)?.into_abool().to_bool() {
                    return self.eval_stmts_hs(body, true);
                }
            }
            Stmt::Call { expr, args } => {
                let func = self.eval_expr(expr)?.into_functio();
                return self.fn_call(func, args, &stmt.span);
            }
            Stmt::Loop { body } => loop {
                let res = self.eval_stmts_hs(body, true)?;
                match res {
                    HaltStatus::Finished => (),
                    HaltStatus::Enough(_) => break,
                    HaltStatus::AndAgain(_) => continue,
                }
            },
            Stmt::Assign { assignable, value } => {
                self.assign(assignable, self.eval_expr(value)?)?;
            }
            Stmt::Enough => {
                return Ok(HaltStatus::Enough(stmt.span.clone()));
            }
            Stmt::AndAgain => {
                return Ok(HaltStatus::AndAgain(stmt.span.clone()));
            }
            Stmt::Melo(ident) => match self.get_var_mut(ident)? {
                var @ Variable::Ref(_) => *var = Variable::Melo,
                Variable::Melo => {
                    for s in &mut self.stack {
                        if s.variables.remove(&ident.item).is_some() {
                            break;
                        }
                    }
                }
            },
            Stmt::Finally(block) => self.finalisers.push(block.clone()),
            Stmt::Rlyeh => {
                // Maybe print a creepy error message or something
                // here at some point. ~~Alex
                let code = random();
                self.host_interface.exit(code);
                return Err(Error::new(
                    ErrorKind::NonExitingRlyeh(code),
                    stmt.span.clone(),
                ));
            }
            Stmt::Rickroll => {
                self.host_interface
                    .print(include_str!("rickroll"), false)
                    .map_err(|e| Error::new(e.into(), stmt.span.clone()))?;
            }
            Stmt::Read(assignable) => {
                let mut value = 0;
                for _ in 0..READ_BITS {
                    value <<= 1;
                    value += self
                        .get_bit()
                        .map_err(|e| Error::new(e, stmt.span.clone()))?
                        as isize;
                }

                self.assign(assignable, Value::Int(value))?;
            }
        }

        Ok(HaltStatus::Finished)
    }

    /// Assign a value to an Assignable.
    fn assign(&mut self, dest: &Assignable, value: Value) -> Result<(), Error> {
        match dest.kind {
            AssignableKind::Variable => {
                self.get_var_rc_mut(&dest.ident)?.replace(value);
            }
            AssignableKind::Index { ref indices } => {
                let mut cell = self.get_var_rc_mut(&dest.ident)?.clone();
                for index in indices {
                    let index = self.eval_expr(index)?;

                    let next_cell = match &mut *cell.borrow_mut() {
                        Value::Cart(c) => {
                            // cell is a cart, so we can do simple
                            // indexing.
                            if let Some(x) = c.get(&index) {
                                // cell[index] exists, get a shared
                                // reference to it.
                                ValueRef::clone(x)
                            } else {
                                // cell[index] does not exist, so we
                                // insert an empty cart by default
                                // instead.
                                let next_cell = ValueRef::new(Value::Cart(Default::default()));
                                c.insert(index, ValueRef::clone(&next_cell));
                                next_cell
                            }
                        }
                        x => {
                            // cell is not a cart; `take` it, convert
                            // it into a cart, and write the result
                            // back into it.
                            let mut cart = take(x).into_cart();
                            let next_cell = ValueRef::new(Value::Cart(Default::default()));
                            cart.insert(index, ValueRef::clone(&next_cell));
                            *x = Value::Cart(cart);
                            next_cell
                        }
                    };
                    cell = next_cell;
                }
                cell.replace(value);
            }
        }

        Ok(())
    }

    /// Call a function with the given arguments (i.e., actual
    /// parameters). If the function invocation fails for some reason,
    /// report the error at `span`.
    fn fn_call(
        &mut self,
        func: Functio,
        args: &[Spanned<Expr>],
        span: &Range<usize>,
    ) -> Result<HaltStatus, Error> {
        // Arguments that are ExprKind::Variable are pass by
        // reference; all other expressions are pass by value.
        let args = args
            .iter()
            .map(|arg| {
                if let Expr::Variable(name) = &arg.item {
                    self.get_var_rc_mut(&Spanned::new(name.to_owned(), arg.span.clone()))
                        .cloned()
                } else {
                    self.eval_expr(arg).map(ValueRef::new)
                }
            })
            .collect::<Result<Vec<_>, Error>>()?;

        self.fn_call_with_values(func, &args, span)
    }

    fn fn_call_with_values(
        &mut self,
        func: Functio,
        args: &[ValueRef],
        span: &Range<usize>,
    ) -> Result<HaltStatus, Error> {
        match func {
            Functio::Bf {
                instructions,
                tape_len,
            } => {
                let mut input: Vec<u8> = vec![];
                for arg in args {
                    arg.borrow().bf_write(&mut input);
                }

                let mut output = vec![];

                crate::brian::Interpreter::from_ascii_with_tape_limit(
                    &instructions,
                    &input as &[_],
                    tape_len,
                )
                .interpret_with_output(&mut output)
                .map_err(|e| Error {
                    kind: ErrorKind::Brian(e),
                    span: span.to_owned(),
                })?;

                match String::from_utf8(output) {
                    Ok(string) => self.host_interface.print(&string, false),
                    Err(e) => self
                        .host_interface
                        .print(&format!("{:?}", e.as_bytes()), true),
                }
                .map_err(|e| Error::new(e.into(), span.clone()))?;

                Ok(HaltStatus::Finished)
            }
            Functio::Able { params, body } => {
                self.stack.push(Default::default());

                for (param, arg) in params.iter().zip(args.iter()) {
                    self.decl_var_shared(param, arg.to_owned());
                }

                let res = self.eval_stmts_hs(&body, false);

                self.stack.pop();
                res
            }
            Functio::Builtin(b) => {
                b.call(args).map_err(|e| Error::new(e, span.clone()))?;
                Ok(HaltStatus::Finished)
            }
            Functio::Chain { functios, kind } => {
                use crate::value::functio::FunctioChainKind;
                let (left_functio, right_functio) = *functios;
                Ok(
                    match match kind {
                        FunctioChainKind::Equal => {
                            let (l, r) = args.split_at(args.len() / 2);

                            (
                                self.fn_call_with_values(left_functio, l, span)?,
                                self.fn_call_with_values(right_functio, r, span)?,
                            )
                        }
                        FunctioChainKind::ByArity => {
                            let (l, r) = Self::deinterlace(
                                args,
                                (left_functio.arity(), right_functio.arity()),
                            );

                            (
                                self.fn_call_with_values(left_functio, &l, span)?,
                                self.fn_call_with_values(right_functio, &r, span)?,
                            )
                        }
                    } {
                        (s, HaltStatus::Finished) => s,
                        (HaltStatus::Finished, s) => s,
                        (_, r) => r,
                    },
                )
            }
            Functio::Eval(code) => self.eval_stmts_hs(&crate::parser::parse(&code)?, false),
        }
    }

    fn deinterlace(args: &[ValueRef], arities: (usize, usize)) -> (Vec<ValueRef>, Vec<ValueRef>) {
        let n_alternations = usize::min(arities.0, arities.1);
        let (extra_l, extra_r) = match Ord::cmp(&arities.0, &arities.1) {
            Ordering::Less => (0, arities.1 - arities.0),
            Ordering::Equal => (0, 0),
            Ordering::Greater => (arities.0 - arities.1, 0),
        };

        (
            args.chunks(2)
                .take(n_alternations)
                .map(|chunk| ValueRef::clone(&chunk[0]))
                .chain(
                    args[2 * n_alternations..]
                        .iter()
                        .map(ValueRef::clone)
                        .take(extra_l),
                )
                .collect(),
            args.chunks(2)
                .take(n_alternations)
                .map(|chunk| ValueRef::clone(&chunk[1]))
                .chain(
                    args[2 * n_alternations..]
                        .iter()
                        .map(ValueRef::clone)
                        .take(extra_r),
                )
                .collect(),
        )
    }

    /// Get a single bit from the bit buffer, or refill it from
    /// standard input if it is empty.
    fn get_bit(&mut self) -> Result<bool, ErrorKind> {
        const BITS_PER_BYTE: u8 = 8;

        if self.read_buf.is_empty() {
            let byte = self.host_interface.read_byte()?;

            for n in (0..BITS_PER_BYTE).rev() {
                self.read_buf.push_back(((byte >> n) & 1) != 0);
            }
        }

        Ok(self
            .read_buf
            .pop_front()
            .expect("We just pushed to the buffer if it was empty"))
    }

    /// Get the value of a variable. Throw an error if the variable is
    /// inaccessible or banned.
    fn get_var_value(&self, name: &Spanned<String>) -> Result<Value, Error> {
        // Search for the name in the stack from top to bottom.
        match self
            .stack
            .iter()
            .rev()
            .find_map(|scope| scope.variables.get(&name.item))
        {
            Some(Variable::Ref(r)) => Ok(r.borrow().clone()),
            Some(Variable::Melo) => Err(Error {
                kind: ErrorKind::MeloVariable(name.item.to_owned()),
                span: name.span.clone(),
            }),
            None => Ok(Value::Undefined),
        }
    }

    /// Get a mutable reference to a variable.
    fn get_var_mut(&mut self, name: &Spanned<String>) -> Result<&mut Variable, Error> {
        // This function has a lot of duplicated code with `get_var`,
        // which I feel like is a bad sign...
        match self
            .stack
            .iter_mut()
            .rev()
            .find_map(|scope| scope.variables.get_mut(&name.item))
        {
            Some(var) => Ok(var),
            None => Err(Error {
                kind: ErrorKind::UnknownVariable(name.item.to_owned()),
                span: name.span.clone(),
            }),
        }
    }

    /// Get an reference to an Rc'd pointer to the value of a variable. Throw an error
    /// if the variable is inaccessible or banned.
    fn get_var_rc_mut(&mut self, name: &Spanned<String>) -> Result<&mut ValueRef, Error> {
        match self.get_var_mut(name)? {
            Variable::Ref(r) => Ok(r),
            Variable::Melo => Err(Error {
                kind: ErrorKind::MeloVariable(name.item.to_owned()),
                span: name.span.clone(),
            }),
        }
    }

    /// Declare a new variable, with the given initial value.
    fn decl_var(&mut self, name: &str, value: Value) {
        self.decl_var_shared(name, ValueRef::new(value));
    }

    /// Declare a new variable, with the given shared initial value.
    fn decl_var_shared(&mut self, name: &str, value: ValueRef) {
        self.stack
            .iter_mut()
            .last()
            .expect("Declaring variable on empty stack")
            .variables
            .insert(name.to_owned(), Variable::Ref(value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::{Expr, Literal},
        host_interface::Standard,
    };

    #[test]
    fn basic_expression_test() {
        // Check that 2 + 2 = 4.
        let env = ExecEnv::<Standard>::default();
        assert_eq!(
            env.eval_expr(&Spanned {
                item: Expr::BinOp {
                    lhs: Box::new(Spanned {
                        item: Expr::Literal(Literal::Int(2)),
                        span: 1..1,
                    }),
                    rhs: Box::new(Spanned {
                        item: Expr::Literal(Literal::Int(2)),
                        span: 1..1,
                    }),
                    kind: crate::ast::BinOpKind::Add,
                },
                span: 1..1
            })
            .unwrap(),
            Value::Int(4)
        )
    }

    #[test]
    fn type_coercions() {
        // The sum of an integer and an aboolean causes an aboolean
        // coercion.

        let env = ExecEnv::<Standard>::default();
        assert_eq!(
            env.eval_expr(&Spanned {
                item: Expr::BinOp {
                    lhs: Box::new(Spanned {
                        item: Expr::Literal(Literal::Int(2)),
                        span: 1..1,
                    }),
                    rhs: Box::new(Spanned {
                        item: Expr::Variable("always".to_owned()),
                        span: 1..1,
                    }),
                    kind: crate::ast::BinOpKind::Add,
                },
                span: 1..1
            })
            .unwrap(),
            Value::Int(3)
        );
    }

    #[test]
    fn overflow_should_not_panic() {
        // Integer overflow should throw a recoverable error instead
        // of panicking.
        let env = ExecEnv::<Standard>::default();
        assert_eq!(
            env.eval_expr(&Spanned {
                item: Expr::BinOp {
                    lhs: Box::new(Spanned {
                        item: Expr::Literal(Literal::Int(isize::MAX)),
                        span: 1..1,
                    }),
                    rhs: Box::new(Spanned {
                        item: Expr::Literal(Literal::Int(1)),
                        span: 1..1,
                    }),
                    kind: crate::ast::BinOpKind::Add,
                },
                span: 1..1
            })
            .unwrap(),
            Value::Int(-9223372036854775808)
        );

        // And the same for divide by zero.
        assert_eq!(
            env.eval_expr(&Spanned {
                item: Expr::BinOp {
                    lhs: Box::new(Spanned {
                        item: Expr::Literal(Literal::Int(84)),
                        span: 1..1,
                    }),
                    rhs: Box::new(Spanned {
                        item: Expr::Literal(Literal::Int(0)),
                        span: 1..1,
                    }),
                    kind: crate::ast::BinOpKind::Divide,
                },
                span: 1..1
            })
            .unwrap(),
            Value::Int(2)
        );
    }

    // From here on out, I'll use this function to parse and run
    // expressions, because writing out abstract syntax trees by hand
    // takes forever and is error-prone.
    fn eval(env: &mut ExecEnv<Standard>, src: &str) -> Result<Value, Error> {
        // We can assume there won't be any syntax errors in the
        // interpreter tests.
        let ast = crate::parser::parse(src).unwrap();
        env.eval_stmts(&ast).map(|()| Value::Nul)
    }

    #[test]
    fn variable_decl_and_assignment() {
        // Functions have no return values, so use some
        // pass-by-reference hacks to detect the correct
        // functionality.
        let mut env = ExecEnv::<Standard>::default();

        // Declaring and reading from a variable.
        eval(&mut env, "foo dim 32; bar dim foo + 1;").unwrap();
        assert_eq!(
            env.get_var_value(&Spanned {
                item: "bar".to_owned(),
                span: 1..1,
            })
            .unwrap(),
            Value::Int(33)
        );

        // Assigning an existing variable.
        eval(&mut env, "/*hi*/ =: foo;").unwrap();
        assert_eq!(
            env.get_var_value(&Spanned {
                item: "foo".to_owned(),
                span: 1..1,
            })
            .unwrap(),
            Value::Str("hi".to_owned())
        );

        // But variable assignment should be illegal when the variable
        // hasn't been declared in advance.
        eval(&mut env, "bar + 1 =: invalid;").unwrap_err();
    }

    #[test]
    fn scope_visibility_rules() {
        // Declaration and assignment of variables declared in an `if`
        // statement should have no effect on those declared outside
        // of it.
        let mut env = ExecEnv::<Standard>::default();
        eval(
            &mut env,
            "foo dim 1; 2 =: foo; unless (never) { foo dim 3; 4 =: foo; }",
        )
        .unwrap();

        assert_eq!(
            env.get_var_value(&Spanned {
                item: "foo".to_owned(),
                span: 1..1,
            })
            .unwrap(),
            Value::Int(2)
        );
    }
}
