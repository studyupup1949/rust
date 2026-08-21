use std::ops::AddAssign;
use crate::core::tape::Tape;

pub(crate) type BinaryFn<T> = fn(T, T) -> T;
pub(crate) type UnaryFn<T> = fn(T) -> T;

pub(crate) enum Operation {
    UnaryFn(UnaryFnPayload),
    BinaryFn(BinaryFnPayload),
}

impl Operation {
    pub(crate) fn backward(&self, tape: &Tape) {
        match self {
            Operation::UnaryFn(payload) => unsafe {
                let values = &*tape.values.get();
                let x = *values.get_unchecked(payload.x);
                let grads = &mut *tape.grads.get();
                let grad = *grads.get_unchecked(payload.y);
                grads.get_unchecked_mut(payload.x).add_assign((payload.dfdx)(x) * grad);
            },
            Operation::BinaryFn(payload) => unsafe {
                let values = &*tape.values.get();
                let x = *values.get_unchecked(payload.x);
                let y = *values.get_unchecked(payload.y);

                let grads = &mut *tape.grads.get();
                let grad = *grads.get_unchecked(payload.z);

                grads.get_unchecked_mut(payload.x).add_assign((payload.dfdx)(x, y) * grad);
                grads.get_unchecked_mut(payload.y).add_assign((payload.dfdy)(x, y) * grad);
            },
        }
    }
}

pub(crate) struct UnaryFnPayload {
    pub x: usize,
    pub y: usize,
    pub dfdx: UnaryFn<f64>,
}

pub(crate) struct BinaryFnPayload {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    pub dfdx: BinaryFn<f64>,
    pub dfdy: BinaryFn<f64>,
}