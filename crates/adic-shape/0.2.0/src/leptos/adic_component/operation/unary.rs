use adic::{normed::UltraNormed, traits::HasApproximateDigits, QAdic, ZAdic};

use crate::error::{AdicShapeError, AdicShapeResult};


#[derive(Debug, Clone, PartialEq, Eq)]
pub (super) enum UnaryOp {
    Neg,
    Square,
    Sqrt,
}

impl UnaryOp {
    pub fn call(&self, input: QAdic<ZAdic>) -> AdicShapeResult<QAdic<ZAdic>> {
        let output = match self {
            UnaryOp::Neg => -input,
            UnaryOp::Square => input.clone() * input,
            UnaryOp::Sqrt => {
                // Square certainty is certainty + valuation; sqrt certainty is certainty - valuation
                let input_valuation = input.valuation().finite().unwrap_or(100);
                let input_certainty = input.certainty().finite().unwrap_or(100);
                let output_certainty = input_certainty - input_valuation;
                input.nth_root(2, output_certainty)?.into_roots().next().ok_or(AdicShapeError::Math("No sqrt".to_string()))?
            },
        };
        Ok(output)
    }
}


impl std::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UnaryOp::Neg => write!(f, "neg"),
            UnaryOp::Square => write!(f, "square"),
            UnaryOp::Sqrt => write!(f, "sqrt"),
        }
    }
}

impl std::str::FromStr for UnaryOp {
    type Err = AdicShapeError;
    fn from_str(s: &str) -> AdicShapeResult<Self> {
        match s {
            "neg" => Ok(UnaryOp::Neg),
            "square" => Ok(UnaryOp::Square),
            "sqrt" => Ok(UnaryOp::Sqrt),
            other => Err(AdicShapeError::Parse(format!("Adic operation parse error :\"{other}\""))),
        }
    }
}
