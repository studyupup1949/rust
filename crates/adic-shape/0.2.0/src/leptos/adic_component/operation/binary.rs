use adic::{error::AdicError, traits::AdicPrimitive, QAdic, ZAdic};

use crate::error::{AdicShapeError, AdicShapeResult};


#[derive(Debug, Clone, PartialEq, Eq)]
pub (super) enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}


impl BinaryOp {
    pub fn call(&self, input0: QAdic<ZAdic>, input1: QAdic<ZAdic>) -> AdicShapeResult<QAdic<ZAdic>> {
        if input0.p() != input1.p() {
            Err(AdicShapeError::AdicError(AdicError::MixedCharacteristic))?;
        }
        let output = match self {
            BinaryOp::Add => input0 + input1,
            BinaryOp::Sub => input0 - input1,
            BinaryOp::Mul => input0 * input1,
            BinaryOp::Div => input0 / input1,
        };
        Ok(output)
    }

}


impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "add"),
            BinaryOp::Sub => write!(f, "sub"),
            BinaryOp::Mul => write!(f, "mul"),
            BinaryOp::Div => write!(f, "div"),
        }
    }
}

impl std::str::FromStr for BinaryOp {
    type Err = AdicShapeError;
    fn from_str(s: &str) -> AdicShapeResult<Self> {
        match s {
            "add" => Ok(BinaryOp::Add),
            "sub" => Ok(BinaryOp::Sub),
            "mul" => Ok(BinaryOp::Mul),
            "div" => Ok(BinaryOp::Div),
            other => Err(AdicShapeError::Parse(format!("Adic operation parse error :\"{other}\""))),
        }
    }
}
