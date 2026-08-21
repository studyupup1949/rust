#[derive(Debug, Clone)]
/// Internal display-independent element so we can translate e.g. to raw svg or leptos component
pub enum AdicEl {
    /// Draw a circle element
    Circle(CircleEl),
    /// Draw a path element
    Path(PathEl),
    /// Draw a text element
    Text(TextEl),
}


#[derive(Debug, Clone)]
pub struct PathEl {
    pub class: Option<String>,
    pub d: Vec<PathDInstruction>,
}

#[derive(Debug, Clone, Copy)]
pub enum PathDInstruction {
    Move((f64, f64)),
    Line((f64, f64)),
}

impl From<PathDInstruction> for String {
    fn from(instruction: PathDInstruction) -> Self {
        match instruction {
            PathDInstruction::Move(m) => format!("M {} {}", m.0, m.1),
            PathDInstruction::Line(l) => format!("L {} {}", l.0, l.1),
        }
    }
}


#[derive(Debug, Clone)]
pub struct CircleEl {
    pub class: Option<String>,
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
}


#[derive(Debug, Clone)]
pub struct TextEl {
    pub content: String,
    pub class: Option<String>,
    pub style: Option<String>,
    pub x: f64,
    pub y: f64,
    pub dx: f64,
    pub dy: f64,
}
