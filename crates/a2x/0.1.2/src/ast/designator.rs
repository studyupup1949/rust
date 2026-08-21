use std::fmt;

/// An attribute designator
#[derive(Debug, Default, Clone, PartialEq)]
pub struct AttributeDesignator {
    // TODO: rename this to identifier for consistency
    pub attribute: Vec<String>, //qualified name
    /// attribute issuer
    pub issuer: Option<String>,
    /// is the attribute required
    pub mustbepresent: bool,
}

impl AttributeDesignator {
    #[must_use]
    pub fn fully_qualified_name(&self) -> String {
        self.attribute.join(".")
    }
}

impl fmt::Display for AttributeDesignator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut options = vec![];
        if self.mustbepresent {
            options.push("mustbepresent".to_string());
        }
        if let Some(i) = &self.issuer {
            options.push(format!("issuer=\"{}\"", i.clone()));
        }
        if options.is_empty() {
            // simple, no options
            write!(f, "{}", self.attribute.join("."))
        } else {
            write!(f, "{}[{}]", self.attribute.join("."), options.join(" "))
        }
    }
}
