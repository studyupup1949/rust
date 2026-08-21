// adminx-core/src/menu.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    pub title: String,
    pub path: String,
    pub children: Option<Vec<MenuItem>>,
    pub icon: Option<String>,
    pub order: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuAction {
    List,
    View,
    Create,
    Edit,
    Delete,
}

impl MenuAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            MenuAction::List => "list",
            MenuAction::View => "view",
            MenuAction::Create => "create",
            MenuAction::Edit => "edit",
            MenuAction::Delete => "delete",
        }
    }
}
