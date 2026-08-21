use std::ops::AddAssign;

pub(crate) struct UnaryFnPayload {
    pub x: usize,
    pub y: usize,
    pub dfdx: f64,
}

pub(crate) struct BinaryFnPayload {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    pub dfdx: f64,
    pub dfdy: f64,
}

pub(crate) enum Operation {
    Unary(UnaryFnPayload),
    Binary(BinaryFnPayload),
}

impl Operation {
    pub(crate) fn backward(&self, grads: &mut [f64]) {
        unsafe {
            match self {
                Operation::Unary(payload) => {
                    let grad = *grads.get_unchecked(payload.y);

                    grads
                        .get_unchecked_mut(payload.x)
                        .add_assign(payload.dfdx * grad);
                }
                Operation::Binary(payload) => {
                    let grad = *grads.get_unchecked(payload.z);

                    grads
                        .get_unchecked_mut(payload.x)
                        .add_assign(payload.dfdx * grad);
                    grads
                        .get_unchecked_mut(payload.y)
                        .add_assign(payload.dfdy * grad);
                }
            }
        }
    }
}
