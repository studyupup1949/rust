/// Type of input field in the admin form.
#[derive(Debug, Clone, serde::Serialize)]
pub enum FieldType {
    Text,
    TextArea { rows: u8 },
    Number,
    Email,
    Password,
    Boolean,
    Date,
    Select { options: Vec<(String, String)> },
}

/// Configuration for a field in the admin create/edit form.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FormField {
    pub key: String,
    pub label: String,
    pub field_type: FieldType,
    pub required: bool,
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
}

impl FormField {
    pub fn text(key: &str, label: &str) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            field_type: FieldType::Text,
            required: false,
            placeholder: None,
            help_text: None,
        }
    }

    pub fn number(key: &str, label: &str) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            field_type: FieldType::Number,
            required: false,
            placeholder: None,
            help_text: None,
        }
    }

    pub fn email(key: &str, label: &str) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            field_type: FieldType::Email,
            required: false,
            placeholder: None,
            help_text: None,
        }
    }

    pub fn password(key: &str, label: &str) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            field_type: FieldType::Password,
            required: false,
            placeholder: None,
            help_text: None,
        }
    }

    pub fn textarea(key: &str, label: &str, rows: u8) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            field_type: FieldType::TextArea { rows },
            required: false,
            placeholder: None,
            help_text: None,
        }
    }

    pub fn boolean(key: &str, label: &str) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            field_type: FieldType::Boolean,
            required: false,
            placeholder: None,
            help_text: None,
        }
    }

    pub fn select(key: &str, label: &str, options: Vec<(&str, &str)>) -> Self {
        let options = options
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Self {
            key: key.to_string(),
            label: label.to_string(),
            field_type: FieldType::Select { options },
            required: false,
            placeholder: None,
            help_text: None,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn placeholder(mut self, p: &str) -> Self {
        self.placeholder = Some(p.to_string());
        self
    }

    pub fn help(mut self, h: &str) -> Self {
        self.help_text = Some(h.to_string());
        self
    }
}
