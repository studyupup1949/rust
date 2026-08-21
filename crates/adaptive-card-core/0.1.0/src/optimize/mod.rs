//! Card optimizations: accessibility, performance, and modernization.

pub mod accessibility;
pub mod modernize;
pub mod performance;

use crate::host::adapt::adapt_for_host;
use crate::types::{OptimizeOpts, TransformReport};
use serde_json::Value;

/// Apply optimizations selectively based on [`OptimizeOpts`] flags.
#[must_use]
pub fn optimize_card(mut card: Value, opts: &OptimizeOpts) -> TransformReport {
    let mut removed: Vec<String> = Vec::new();
    let mut downgraded: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    if opts.accessibility {
        downgraded.extend(accessibility::fix(&mut card));
    }

    if opts.performance {
        downgraded.extend(performance::fix(&mut card));
    }

    if opts.modernize {
        downgraded.extend(modernize::fix(&mut card, opts.target_host));
    }

    if let Some(host) = opts.target_host {
        let report = adapt_for_host(card, host);
        card = report.card;
        removed.extend(report.removed);
        downgraded.extend(report.downgraded);
        warnings.extend(report.warnings);
    }

    TransformReport {
        card,
        removed,
        downgraded,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accessibility_optimization_adds_speak() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [{ "type": "TextBlock", "text": "Hi" }]
        });
        let report = optimize_card(
            card,
            &OptimizeOpts {
                accessibility: true,
                ..Default::default()
            },
        );
        assert!(report.card.get("speak").is_some());
    }

    #[test]
    fn performance_flattens_and_dedupes() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [
                { "type": "Container", "items": [{ "type": "TextBlock", "text": "A", "id": "t1" }] },
                { "type": "TextBlock", "text": "B", "id": "t1" }
            ]
        });
        let report = optimize_card(
            card,
            &OptimizeOpts {
                performance: true,
                ..Default::default()
            },
        );
        // Flattened: first child is now TextBlock directly
        assert_eq!(report.card["body"][0]["type"], "TextBlock");
    }
}
