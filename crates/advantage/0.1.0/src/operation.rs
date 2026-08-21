use num::traits::Float;

/// Enum of possible elementary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Nop,
    Const,
    Add,
    Sub,
    Mul,
    Div,
    Sin,
    Cos,
    Tan,
    Abs,
    Exp,
    Ln,
    Asin,
    Acos,
    Atan,
    Powf,
}

pub(crate) fn zero_order_value<S: Float>(opcode: OpCode, arg1: S, arg2: Option<S>) -> S {
    match opcode {
        OpCode::Add => arg1 + arg2.unwrap(),
        OpCode::Sub => arg1 - arg2.unwrap(),
        OpCode::Mul => arg1 * arg2.unwrap(),
        OpCode::Div => arg1 / arg2.unwrap(),
        OpCode::Sin => arg1.sin(),
        OpCode::Cos => arg1.cos(),
        OpCode::Tan => arg1.tan(),
        OpCode::Abs => arg1.abs(),
        OpCode::Exp => arg1.exp(),
        OpCode::Ln => arg1.ln(),
        OpCode::Asin => arg1.asin(),
        OpCode::Acos => arg1.acos(),
        OpCode::Atan => arg1.atan(),
        OpCode::Powf => arg1.powf(arg2.unwrap()),
        _ => panic!("Invalid opcode in zero_order_value"),
    }
}

pub(crate) fn first_order_value<S: Float>(
    opcode: OpCode,
    arg1: S,
    arg2: Option<S>,
    darg1: S,
    darg2: Option<S>,
) -> S {
    match opcode {
        OpCode::Add => darg1 + darg2.unwrap(),
        OpCode::Sub => darg1 - darg2.unwrap(),
        OpCode::Mul => darg1 * arg2.unwrap() + arg1 * darg2.unwrap(),
        OpCode::Div => (darg1 * arg2.unwrap() - arg1 * darg2.unwrap()) / arg2.unwrap().powi(2),
        OpCode::Sin => darg1 * arg1.cos(),
        OpCode::Cos => -darg1 * arg1.sin(),
        OpCode::Tan => darg1 * (S::one() / arg1.cos().powi(2)),
        OpCode::Abs => (arg1 + darg1).abs() - arg1.abs(),
        OpCode::Exp => darg1 * arg1.exp(),
        OpCode::Ln => darg1 * (S::one() / arg1),
        OpCode::Asin => darg1 / (S::one() - arg1.powi(2)).sqrt(),
        OpCode::Acos => -darg1 / (S::one() - arg1.powi(2)).sqrt(),
        OpCode::Atan => darg1 / (S::one() + arg1.powi(2)),
        OpCode::Powf => {
            let x = arg1;
            let y = arg2.unwrap();
            let dx = darg1;
            let dy = darg2.unwrap();
            y * x.powf(y - S::one()) * dx + x.ln() * x.powf(y) * dy
        }
        _ => panic!("Invalid opcode in first_order_value"),
    }
}

/// Representation of a single elementary operation and inputs and output
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Operation {
    /// Op code of the operation
    pub opcode: OpCode,
    /// Result id
    pub vid: usize,
    /// Argument 1 id
    pub arg1: Option<usize>,
    /// Argument 2 id
    pub arg2: Option<usize>,
}

impl Operation {
    pub fn nop() -> Self {
        Self {
            opcode: OpCode::Nop,
            vid: 0,
            arg1: None,
            arg2: None,
        }
    }

    pub fn constant(vid: usize) -> Self {
        Self {
            opcode: OpCode::Const,
            vid,
            arg1: None,
            arg2: None,
        }
    }

    pub fn add(vid: usize, lhs: usize, rhs: usize) -> Self {
        Self {
            opcode: OpCode::Add,
            vid,
            arg1: Some(lhs),
            arg2: Some(rhs),
        }
    }

    pub fn sub(vid: usize, lhs: usize, rhs: usize) -> Self {
        Self {
            opcode: OpCode::Sub,
            vid,
            arg1: Some(lhs),
            arg2: Some(rhs),
        }
    }

    pub fn mul(vid: usize, lhs: usize, rhs: usize) -> Self {
        Self {
            opcode: OpCode::Mul,
            vid,
            arg1: Some(lhs),
            arg2: Some(rhs),
        }
    }

    pub fn div(vid: usize, lhs: usize, rhs: usize) -> Self {
        Self {
            opcode: OpCode::Div,
            vid,
            arg1: Some(lhs),
            arg2: Some(rhs),
        }
    }

    pub fn sin(vid: usize, idx: usize) -> Self {
        Self {
            opcode: OpCode::Sin,
            vid,
            arg1: Some(idx),
            arg2: None,
        }
    }

    pub fn cos(vid: usize, idx: usize) -> Self {
        Self {
            opcode: OpCode::Cos,
            vid,
            arg1: Some(idx),
            arg2: None,
        }
    }

    pub fn tan(vid: usize, idx: usize) -> Self {
        Self {
            opcode: OpCode::Tan,
            vid,
            arg1: Some(idx),
            arg2: None,
        }
    }

    pub fn abs(vid: usize, idx: usize) -> Self {
        Self {
            opcode: OpCode::Abs,
            vid,
            arg1: Some(idx),
            arg2: None,
        }
    }

    pub fn exp(vid: usize, idx: usize) -> Self {
        Self {
            opcode: OpCode::Exp,
            vid,
            arg1: Some(idx),
            arg2: None,
        }
    }

    pub fn ln(vid: usize, idx: usize) -> Self {
        Self {
            opcode: OpCode::Ln,
            vid,
            arg1: Some(idx),
            arg2: None,
        }
    }

    pub fn asin(vid: usize, idx: usize) -> Self {
        Self {
            opcode: OpCode::Asin,
            vid,
            arg1: Some(idx),
            arg2: None,
        }
    }

    pub fn acos(vid: usize, idx: usize) -> Self {
        Self {
            opcode: OpCode::Acos,
            vid,
            arg1: Some(idx),
            arg2: None,
        }
    }

    pub fn atan(vid: usize, idx: usize) -> Self {
        Self {
            opcode: OpCode::Atan,
            vid,
            arg1: Some(idx),
            arg2: None,
        }
    }

    pub fn powf(vid: usize, base: usize, exponent: usize) -> Self {
        Self {
            opcode: OpCode::Powf,
            vid,
            arg1: Some(base),
            arg2: Some(exponent),
        }
    }

    pub fn zero_order(self, v: &mut [f64]) {
        match self.opcode {
            OpCode::Nop => {}
            OpCode::Const => {}
            _ => {
                v[self.vid] =
                    zero_order_value(self.opcode, v[self.arg1.unwrap()], self.arg2.map(|i| v[i]))
            }
        }
    }

    pub fn first_order(self, v: &[f64], dv: &mut [f64]) {
        match self.opcode {
            OpCode::Nop => {}
            OpCode::Const => {
                dv[self.vid] = 0.0;
            }
            _ => {
                dv[self.vid] = first_order_value(
                    self.opcode,
                    v[self.arg1.unwrap()],
                    self.arg2.map(|i| v[i]),
                    dv[self.arg1.unwrap()],
                    self.arg2.map(|i| dv[i]),
                );
            }
        }
    }

    pub fn first_order_reverse(self, v: &[f64], vbar: &mut [f64]) {
        // ∂s/∂v_i = sum_j ∂s/∂v_j * ∂v_j/∂v_i  + ...
        // vbar_i := ∂s/∂v_i
        // => vbar_i = sum_j vbar_j * ∂v_j/∂v_i
        match self.opcode {
            OpCode::Nop => {}
            OpCode::Const => {}
            OpCode::Add => {
                // v_i = v_j + v_k
                // =>
                // vbar_j += vbar_i * ∂v_i/∂v_j = vbar_i
                // vbar_k += vbar_i * ∂v_i/∂v_k = vbar_i
                vbar[self.arg1.unwrap()] += vbar[self.vid];
                vbar[self.arg2.unwrap()] += vbar[self.vid];
            }
            OpCode::Sub => {
                // v_i = v_j - v_k
                // =>
                // vbar_j += vbar_i * ∂v_i/∂v_j = vbar_i
                // vbar_k += vbar_i * ∂v_i/∂v_k = -vbar_i
                vbar[self.arg1.unwrap()] += vbar[self.vid];
                vbar[self.arg2.unwrap()] += -vbar[self.vid];
            }
            OpCode::Mul => {
                // v_i = v_j * v_k
                // =>
                // vbar_j += vbar_i * ∂v_i/∂v_j = vbar_i * v_k
                // vbar_k += vbar_i * ∂v_i/∂v_k = vbar_i * v_j
                vbar[self.arg1.unwrap()] += vbar[self.vid] * v[self.arg2.unwrap()];
                vbar[self.arg2.unwrap()] += vbar[self.vid] * v[self.arg1.unwrap()];
            }
            OpCode::Div => {
                // v_i = v_j / v_k
                // =>
                // vbar_j += vbar_i * ∂v_i/∂v_j = vbar_i * 1/v_k
                // vbar_k += vbar_i * ∂v_i/∂v_k = vbar_i * -v_j/(v_k^2)
                vbar[self.arg1.unwrap()] += vbar[self.vid] * 1.0 / v[self.arg2.unwrap()];
                vbar[self.arg2.unwrap()] +=
                    vbar[self.vid] * (-v[self.arg1.unwrap()] / v[self.arg2.unwrap()].powi(2));
            }
            OpCode::Powf => {
                // v_i = v_j^v_k
                // =>
                // vbar_j += vbar_i * ∂v_i/∂v_j = vbar_i * v_k * v_j.powf(v_k - 1)
                // vbar_k += vbar_i * ∂v_i/∂v_k = vbar_i * v_j.ln() * v_j.powf(v_k)
                let x = v[self.arg1.unwrap()];
                let y = v[self.arg2.unwrap()];
                vbar[self.arg1.unwrap()] += vbar[self.vid] * y * x.powf(y - 1.0);
                vbar[self.arg2.unwrap()] += vbar[self.vid] * x.ln() * x.powf(y);
            }
            OpCode::Abs => {
                panic!("Abs-function encountered in first_order_reverse");
            }
            _ => {
                // Unary function
                // vbar_j += vbar_i * ∂v_i/∂v_j
                vbar[self.arg1.unwrap()] += vbar[self.vid]
                    * first_order_value(self.opcode, v[self.arg1.unwrap()], None, 1.0, None);
            }
        }
    }
}
