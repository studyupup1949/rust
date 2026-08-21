/// Type of data displayed in a table column.
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum ColumnType {
    Text,
    Number,
    Boolean,
    Date,
}

/// Configuration for a column in the admin list view.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Column {
    pub key: String,
    pub label: String,
    pub sortable: bool,
    pub column_type: ColumnType,
}

impl Column {
    pub fn text(key: &str, label: &str) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            sortable: true,
            column_type: ColumnType::Text,
        }
    }

    pub fn number(key: &str, label: &str) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            sortable: true,
            column_type: ColumnType::Number,
        }
    }

    pub fn boolean(key: &str, label: &str) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            sortable: true,
            column_type: ColumnType::Boolean,
        }
    }

    pub fn date(key: &str, label: &str) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            sortable: true,
            column_type: ColumnType::Date,
        }
    }
}
