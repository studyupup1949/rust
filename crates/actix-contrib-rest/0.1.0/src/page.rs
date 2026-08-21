//! Map page responses.

use serde::{Deserialize, Serialize};

/// Struct used to serialize and deserialize paginated results.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Page<T> {
    /// The data in the page, an empty `[]` vector
    /// if there is no results.
    pub data: Vec<T>,
    /// the offset from the full results, normally
    /// zero indexed.
    pub offset: i64,
    /// The size of the current page result, that could
    /// be <= to the size requested depending of how many
    /// results you get.
    pub page_size: i64,
    /// The total results count including the ones included
    /// in this page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,

    /// A message that might be presented to the user along
    /// the result, e.g. a hint of how to improve the
    /// query to get more results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// A warning message that might be presented to the user along
    /// the results, e.g. a user using a source of information
    /// that is deprecated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

impl<T> From<Vec<T>> for Page<T> {
    /// Creates a page with the vector passe. The `page_size` and
    /// `total` will be equivalent to the size of the `vec`.
    fn from(vec: Vec<T>) -> Self {
        let len: i64 = vec.len() as i64;
        Page {
            data: vec,
            offset: 0,
            page_size: len,
            total: Some(len),
            message: None,
            warning: None,
        }
    }
}

impl<T> Page<T> {
    /// Create empty page.
    pub fn empty() -> Page<T> {
        Page {
            data: Vec::new(),
            offset: 0,
            page_size: 0,
            total: Some(0),
            message: None,
            warning: None,
        }
    }

    /// Create page with the data, total and offset passed.
    pub fn with_data(data: Vec<T>, total: Option<i64>, offset: i64) -> Self {
        let page_size: i64 = data.len() as i64;
        Page {
            data,
            total,
            offset,
            page_size,
            message: None,
            warning: None,
        }
    }
}
