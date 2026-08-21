//! Card transformation — version downgrades and host adaptation.

pub mod downgrade;

use crate::error::{Error, Result};
use crate::host::adapt::adapt_for_host;
use crate::types::{CardVersion, TransformReport, TransformTarget};
use serde_json::Value;

/// Apply a transform target (version downgrade and/or host adaptation) to a card.
pub fn transform_card(mut card: Value, target: &TransformTarget) -> Result<TransformReport> {
    let mut removed: Vec<String> = Vec::new();
    let mut downgraded: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Determine the effective version target (explicit version or host's max).
    let version_target: Option<CardVersion> = target
        .version
        .or_else(|| target.host.map(|h| h.max_version()));

    if let Some(v) = version_target {
        let current = card
            .get("version")
            .and_then(Value::as_str)
            .and_then(CardVersion::parse);
        if let Some(c) = current {
            if c > v {
                let (r, d) = downgrade::downgrade_to(&mut card, v);
                removed.extend(r);
                downgraded.extend(d);
            }
        } else {
            warnings.push("Card has no parseable version; cannot downgrade".to_string());
        }
    }

    if let Some(h) = target.host {
        let report = adapt_for_host(card, h);
        card = report.card;
        removed.extend(report.removed);
        downgraded.extend(report.downgraded);
        warnings.extend(report.warnings);
    }

    if target.strict && !removed.is_empty() {
        return Err(Error::TransformLossy(format!(
            "strict mode: {} removals not allowed",
            removed.len()
        )));
    }

    Ok(TransformReport {
        card,
        removed,
        downgraded,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Host;
    use serde_json::json;

    #[test]
    fn transform_to_outlook_downgrades_and_adapts() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [{ "type": "TextBlock", "text": "Hi" }],
            "actions": [{ "type": "Action.Execute", "title": "OK" }]
        });
        let report = transform_card(
            card,
            &TransformTarget {
                version: None,
                host: Some(Host::Outlook),
                strict: false,
            },
        )
        .unwrap();
        assert_eq!(report.card["version"], "1.4");
        assert_eq!(report.card["actions"][0]["type"], "Action.Submit");
    }

    #[test]
    fn strict_transform_errors_on_removal() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.3",
            "body": [{ "type": "Table", "columns": [] }]
        });
        let result = transform_card(
            card,
            &TransformTarget {
                version: None,
                host: Some(Host::Webex),
                strict: true,
            },
        );
        assert!(matches!(result, Err(Error::TransformLossy(_))));
    }
}
