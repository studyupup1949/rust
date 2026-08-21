use crate::error::AdicShapeError;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Interactive shape controls, e.g. `depth` and `shape_type`
pub struct ShapeControls {
    /// Enables or disables depth control
    pub enable_depth_control: bool,
    /// Depth of clocks and trees
    pub depth: isize,
    /// E.g. clock, tree, euclidean
    pub shape_type: ShapeType,
}

impl Default for ShapeControls {
    fn default() -> Self {
        ShapeControls{
            enable_depth_control: true,
            depth: 10,
            shape_type: ShapeType::Clock,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Shape type for display
pub enum ShapeType {
    /// Clock shape type
    Clock,
    /// Tree shape type
    Tree,
    /// Euclidean shape type
    Euclidean,
}

impl std::fmt::Display for ShapeType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ShapeType::Clock => write!(f, "Clock"),
            ShapeType::Tree => write!(f, "Tree"),
            ShapeType::Euclidean => write!(f, "Euclidean"),
        }
    }
}

impl std::str::FromStr for ShapeType {
    type Err = AdicShapeError;
    fn from_str(s: &str) -> Result<Self, AdicShapeError> {
        match s {
            "Clock" => Ok(ShapeType::Clock),
            "Tree" => Ok(ShapeType::Tree),
            "Euclidean" => Ok(ShapeType::Euclidean),
            _ => Err(AdicShapeError::Parse("Shape type parse error".to_string()))
        }
    }
}
