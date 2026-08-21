use crate::aaml::AAML;
use crate::error::{AamlError, ErrorDiagnostics};
use crate::types::Type;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MathTypes {
    Vector2,
    Vector3,
    Vector4,
    Quaternion,
    Matrix3x3,
    Matrix4x4,
}

impl Type for MathTypes {
    fn from_name(name: &str) -> Result<Self, AamlError>
    where
        Self: Sized,
    {
        match name {
            "vector2" => Ok(MathTypes::Vector2),
            "vector3" => Ok(MathTypes::Vector3),
            "vector4" => Ok(MathTypes::Vector4),
            "quaternion" => Ok(MathTypes::Quaternion),
            "matrix3x3" => Ok(MathTypes::Matrix3x3),
            "matrix4x4" => Ok(MathTypes::Matrix4x4),
            _ => Err(AamlError::NotFound {
                key: name.to_string(),
                context: "math types".to_string(),
                diagnostics: Some(ErrorDiagnostics::new(
                    "Unknown math type",
                    format!("Math type '{}' is not recognized", name),
                    "Valid types: vector2, vector3, vector4, quaternion, matrix3x3, matrix4x4",
                )),
            }),
        }
    }

    fn base_type(&self) -> crate::types::primitive_type::PrimitiveType {
        crate::types::primitive_type::PrimitiveType::F64
    }

    fn validate(&self, value: &str, _aaml: &AAML) -> Result<(), AamlError> {
        let expected_len = match self {
            MathTypes::Vector2 => 2,
            MathTypes::Vector3 => 3,
            MathTypes::Vector4 | MathTypes::Quaternion => 4,
            MathTypes::Matrix3x3 => 9,
            MathTypes::Matrix4x4 => 16,
        };

        let mut count = 0usize;
        for part in value.split(',') {
            let part = part.trim();
            if part.parse::<f64>().is_err() {
                return Err(AamlError::InvalidValue {
                    details: format!("'{}' is not a valid number", part),
                    expected: "floating-point number (f64)".to_string(),
                    diagnostics: Some(ErrorDiagnostics::new(
                        "Invalid number in vector/matrix",
                        format!("Component '{}' could not be parsed as a number", part),
                        "Use decimal notation: 1.0, 2.5, -3.14, etc.",
                    )),
                });
            }
            count += 1;
        }

        if count != expected_len {
            let type_name = match self {
                MathTypes::Vector2 => "vector2",
                MathTypes::Vector3 => "vector3",
                MathTypes::Vector4 => "vector4",
                MathTypes::Quaternion => "quaternion",
                MathTypes::Matrix3x3 => "matrix3x3",
                MathTypes::Matrix4x4 => "matrix4x4",
            };
            return Err(AamlError::InvalidValue {
                details: format!("Expected {} components, got {}", expected_len, count),
                expected: format!("{} comma-separated numbers", expected_len),
                diagnostics: Some(ErrorDiagnostics::new(
                    "Wrong number of components",
                    format!(
                        "Type '{}' requires {} values, but {} were provided",
                        type_name, expected_len, count
                    ),
                    format!(
                        "Provide exactly {} numbers separated by commas",
                        expected_len
                    ),
                )),
            });
        }

        Ok(())
    }
}
