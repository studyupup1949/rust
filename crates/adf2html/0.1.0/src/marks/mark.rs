use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{
    color::Color, 
    link::Link, 
    subsup::Subsup, 
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Mark {
    BackgroundColor(Color),
    Code,
    Em,
    Link(Link),
    Strike,
    Strong,
    Subsup(Subsup),
    TextColor(Color),
    Underline,
}

struct Style {
    name: String,
    value: String,
}

impl Style {
    fn new(name: &str, value: &str) -> Self {
        Self { name: name.to_string(), value: value.to_string() }
    }
    fn new_vec(name: &str, value: &str) -> Vec<Self> {
        vec![Self::new(name, value)]
    }
}

impl Mark {
    fn get_styles(&self) -> Vec<Style> {
        match self {
            Mark::BackgroundColor(color) => Style::new_vec("background-color", &color.attributes.color),
            Mark::Code => vec![
                Style::new("background-color", "#F4F5F7"),
                Style::new("color", "#172B4D"),
                Style::new("font-family", "SFMono-Medium, 'SF Mono', 'Segoe UI Mono', 'Roboto Mono', 'Ubuntu Mono', Menlo, Consolas, Courier, monospace"),
                Style::new("padding", "3px"),
            ],
            Mark::Em => Style::new_vec("font-style", "italic"),
            Mark::Link(_) => Vec::new(),
            Mark::Strike => Style::new_vec("text-decoration", "line-through"),
            Mark::Strong => Style::new_vec("font-weight", "bold"),
            Mark::Subsup(subsup) => vec![
                Style::new("vertical-align", match subsup.is_sub() { true => "sub", false => "super"}),
                Style::new("font-size", "smaller"),
        
            ],
            Mark::TextColor(color) => Style::new_vec("color", &color.attributes.color),
            Mark::Underline => Style::new_vec("text-decoration", "underline"),
        }
    }
}

pub(crate) trait MarkVecToHtml {
    fn get_styles(&self) -> String;
}

impl MarkVecToHtml for Vec<Mark> {
    fn get_styles(&self) -> String {
        let mut style_map = HashMap::new();

        for mark in self {
            for style in mark.get_styles() {
                style_map
                    .entry(style.name)
                    .and_modify(|v| *v = format!("{} {}", v, style.value))
                    .or_insert(style.value);
            }
        }

        style_map
            .into_iter()
            .map(|(k, v)| format!("{k}: {v};"))
            .collect()
    }
}
