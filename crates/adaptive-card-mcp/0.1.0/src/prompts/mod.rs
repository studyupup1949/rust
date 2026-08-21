//! MCP prompt templates — guided workflow scripts.
//!
//! Each prompt renders a user-visible script guiding the LLM through a
//! multi-step workflow using this server's tools.

#![allow(dead_code, reason = "wired into the rmcp prompt router in Task 35")]

pub mod refine_host;
pub mod review_card;
pub mod templatize;

/// Static metadata for an MCP prompt exposed by this server.
#[derive(Debug, Clone, Copy)]
pub struct PromptDef {
    pub name: &'static str,
    pub description: &'static str,
    pub arguments: &'static [PromptArg],
}

/// Static metadata for a single prompt argument.
#[derive(Debug, Clone, Copy)]
pub struct PromptArg {
    pub name: &'static str,
    pub description: &'static str,
    pub required: bool,
}

pub const REVIEW_CARD: PromptDef = PromptDef {
    name: "review-adaptive-card",
    description: "Review a card: validate, optimize accessibility, and return a before/after report",
    arguments: &[
        PromptArg {
            name: "card",
            description: "Card JSON to review",
            required: true,
        },
        PromptArg {
            name: "host",
            description: "Optional target host (teams, outlook, webchat, ...)",
            required: false,
        },
    ],
};

pub const REFINE_HOST: PromptDef = PromptDef {
    name: "refine-for-host",
    description: "Refine a card to fit a target host: validate, transform, re-validate",
    arguments: &[
        PromptArg {
            name: "card",
            description: "Card JSON to refine",
            required: true,
        },
        PromptArg {
            name: "target_host",
            description: "Target host name (e.g. teams, outlook, webex)",
            required: true,
        },
    ],
};

pub const TEMPLATIZE: PromptDef = PromptDef {
    name: "templatize-card",
    description: "Convert a static card into a template with sample data",
    arguments: &[PromptArg {
        name: "card",
        description: "Card JSON to templatize",
        required: true,
    }],
};

/// Return every prompt exposed by this server, in listing order.
#[must_use]
pub fn all() -> &'static [PromptDef] {
    &[REVIEW_CARD, REFINE_HOST, TEMPLATIZE]
}

/// Render a prompt by name into a user-visible script.
#[must_use]
pub fn render(name: &str, args: &serde_json::Value) -> Option<String> {
    match name {
        "review-adaptive-card" => Some(review_card::render(args)),
        "refine-for-host" => Some(refine_host::render(args)),
        "templatize-card" => Some(templatize::render(args)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn all_has_three_prompts() {
        assert_eq!(all().len(), 3);
    }

    #[test]
    fn render_review_card() {
        let out = render(
            "review-adaptive-card",
            &json!({ "card": { "type": "AdaptiveCard" }, "host": "teams" }),
        )
        .unwrap();
        assert!(out.contains("validate_card"));
        assert!(out.contains("teams"));
    }

    #[test]
    fn render_refine_host() {
        let out = render(
            "refine-for-host",
            &json!({ "card": { "type": "AdaptiveCard" }, "target_host": "outlook" }),
        )
        .unwrap();
        assert!(out.contains("transform_card"));
        assert!(out.contains("outlook"));
    }

    #[test]
    fn render_templatize() {
        let out = render(
            "templatize-card",
            &json!({ "card": { "type": "AdaptiveCard" } }),
        )
        .unwrap();
        assert!(out.contains("template_card"));
    }

    #[test]
    fn render_unknown_prompt_is_none() {
        assert!(render("nope", &json!({})).is_none());
    }
}
