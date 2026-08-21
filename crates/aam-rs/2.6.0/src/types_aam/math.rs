use crate::error::{AamlError, ErrorDiagnostics};
use crate::pipeline::ExecutionContext;
use crate::types_aam::TypeAAM;
use crate::types_aam::primitive_type::PrimitiveType;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MathTypes {
    Vector2,
    Vector3,
    Vector4,
    Quaternion,
    Matrix3x3,
    Matrix4x4,
}

impl TypeAAM for MathTypes {
    fn from_name(name: &str) -> Result<Self, AamlError>
    where
        Self: Sized,
    {
        match name {
            "vector2" => Ok(Self::Vector2),
            "vector3" => Ok(Self::Vector3),
            "vector4" => Ok(Self::Vector4),
            "quaternion" => Ok(Self::Quaternion),
            "matrix3x3" => Ok(Self::Matrix3x3),
            "matrix4x4" => Ok(Self::Matrix4x4),
            _ => Err(AamlError::NotFound {
                key: name.to_string(),
                context: "math types".to_string(),
                diagnostics: Some(Box::new(ErrorDiagnostics::new(
                    "Unknown math type",
                    format!("Math type '{name}' is not recognized"),
                    "Valid types: vector2, vector3, vector4, quaternion, matrix3x3, matrix4x4",
                ))),
            }),
        }
    }

    fn base_type(&self) -> PrimitiveType {
        PrimitiveType::F64
    }

    fn validate(&self, value: &str, _context: &ExecutionContext) -> Result<(), AamlError> {
        let expected_len = match self {
            Self::Vector2 => 2,
            Self::Vector3 => 3,
            Self::Vector4 | Self::Quaternion => 4,
            Self::Matrix3x3 => 9,
            Self::Matrix4x4 => 16,
        };

        let mut count = 0usize;
        for part in value.split(',') {
            let part = part.trim();
            if part.parse::<f64>().is_err() {
                return Err(AamlError::InvalidValue {
                    details: format!("'{part}' is not a valid number"),
                    expected: "floating-point number (f64)".to_string(),
                    diagnostics: Some(Box::new(ErrorDiagnostics::new(
                        "Invalid number in vector/matrix",
                        format!("Component '{part}' could not be parsed as a number"),
                        "Use decimal notation: 1.0, 2.5, -3.14, etc.",
                    ))),
                });
            }
            count += 1;
        }

        if count != expected_len {
            let type_name = match self {
                Self::Vector2 => "vector2",
                Self::Vector3 => "vector3",
                Self::Vector4 => "vector4",
                Self::Quaternion => "quaternion",
                Self::Matrix3x3 => "matrix3x3",
                Self::Matrix4x4 => "matrix4x4",
            };
            return Err(AamlError::InvalidValue {
                details: format!("Expected {expected_len} components, got {count}"),
                expected: format!("{expected_len} comma-separated numbers"),
                diagnostics: Some(Box::new(ErrorDiagnostics::new(
                    "Wrong number of components",
                    format!(
                        "Type '{type_name}' requires {expected_len} values, but {count} were provided"
                    ),
                    format!("Provide exactly {expected_len} numbers separated by commas"),
                ))),
            });
        }

        Ok(())
    }
}
