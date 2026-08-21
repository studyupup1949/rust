use crate::error::AamlError;
use crate::types::Type;

pub enum MathTypes {
    Vector2,
    Vector3,
    Vector4,
    Quaternion,
    Matrix3x3,
    Matrix4x4,
}

impl Type for MathTypes {
    fn from_name(name: &str) -> Result<Self, crate::error::AamlError>
    where
        Self: Sized
    {
        match name {
            "vector2" => Ok(MathTypes::Vector2),
            "vector3" => Ok(MathTypes::Vector3),
            "vector4" => Ok(MathTypes::Vector4),
            "quaternion" => Ok(MathTypes::Quaternion),
            "matrix3x3" => Ok(MathTypes::Matrix3x3),
            "matrix4x4" => Ok(MathTypes::Matrix4x4),
            _ => Err(crate::error::AamlError::NotFound(name.to_string())),
        }
    }

    fn base_type(&self) -> crate::types::primitive_type::PrimitiveType {
        crate::types::primitive_type::PrimitiveType::F64
    }

    fn validate(&self, value: &str) -> Result<(), AamlError> {
        let parts: Vec<&str> = value.split(',').map(|s| s.trim()).collect();
        let expected_len = match self {
            MathTypes::Vector2 => 2,
            MathTypes::Vector3 => 3,
            MathTypes::Vector4 | MathTypes::Quaternion => 4,
            MathTypes::Matrix3x3 => 9,
            MathTypes::Matrix4x4 => 16,
        };

        if parts.len() != expected_len {
            return Err(AamlError::InvalidValue(format!("Expected {} components, got {}", expected_len, parts.len())));
        }

        for part in parts {
            if part.parse::<f64>().is_err() {
                return Err(AamlError::InvalidValue(format!("Invalid number: {}", part)));
            }
        }

        Ok(())
    }
}