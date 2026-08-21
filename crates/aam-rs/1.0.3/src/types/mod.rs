use crate::error::AamlError;
use crate::types::primitive_type::PrimitiveType;

pub(crate) mod physics;
pub(crate) mod primitive_type;
mod math;
mod time;

pub trait Type {
    fn from_name(name: &str) -> Result<Self, AamlError> where Self: Sized;
    fn base_type(&self) -> PrimitiveType;
    fn validate(&self, value: &str) -> Result<(), AamlError>;
}

pub fn resolve_builtin(path: &str) -> Result<Box<dyn Type>, AamlError> {
    let parts: Vec<&str> = path.splitn(2, "::").collect();

    match parts.as_slice() {
        ["math", name] => Ok(Box::new(math::MathTypes::from_name(name)?)),
        ["time", name] => Ok(Box::new(time::TimeTypes::from_name(name)?)),
        ["physics", name] => Ok(Box::new(physics::PhysicsTypes::from_name(name)?)),
        [name] => Ok(Box::new(primitive_type::PrimitiveType::from_name(name)?)),
        _ => Err(AamlError::NotFound(path.to_string())),
    }
}