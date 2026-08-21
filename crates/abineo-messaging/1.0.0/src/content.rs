use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum Content {
    Title(String),
    Subtitle(String),
    Text(String),
    Secret(String),
}

impl Content {
    pub fn builder() -> ContentBuilder {
        ContentBuilder::default()
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct ContentBuilder {
    items: Vec<Content>,
}

impl ContentBuilder {
    pub fn title<T: Into<String>>(mut self, value: T) -> Self {
        self.items.push(Content::Title(value.into()));
        self
    }

    pub fn subtitle<T: Into<String>>(mut self, value: T) -> Self {
        self.items.push(Content::Subtitle(value.into()));
        self
    }

    pub fn text<T: Into<String>>(mut self, value: T) -> Self {
        self.items.push(Content::Text(value.into()));
        self
    }

    pub fn secret<T: Into<String>>(mut self, value: T) -> Self {
        self.items.push(Content::Secret(value.into()));
        self
    }

    pub fn build(self) -> Vec<Content> {
        self.items
    }
}

impl Into<Vec<Content>> for ContentBuilder {
    fn into(self) -> Vec<Content> {
        self.build()
    }
}
