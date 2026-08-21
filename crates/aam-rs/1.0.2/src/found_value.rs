use std::fmt::Display;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct  FoundValue {
    inner: String,
}

impl FoundValue {
    pub fn new(value: &str) -> FoundValue {
        FoundValue {
            inner: value.to_string(),
        }
    }

    pub fn remove(&mut self, target: &str) -> &mut Self {
        self.inner = self.inner.replace(target, "");
        self
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl From<String> for FoundValue {
    fn from(value: String) -> Self {
        FoundValue { inner: value }
    }
}

impl PartialEq<&str> for FoundValue {
    fn eq(&self, other: &&str) -> bool {
        self.inner == *other
    }
}

impl Display for FoundValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.clone())
    }
}

impl Deref for FoundValue {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}