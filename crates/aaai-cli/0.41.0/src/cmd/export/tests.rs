//! RFC 103 F1 — CSV/TSV formula-injection regression test.

use super::*;

#[test]
fn formula_trigger_characters_are_neutralized_with_a_leading_apostrophe() {
    for trigger in ['=', '+', '-', '@', '\t', '\r'] {
        let payload = format!("{trigger}cmd|'/c calc'!A1");
        let escaped = csv_escape(&payload, ',');
        assert!(
            escaped.starts_with('\'') || escaped.starts_with("\"'"),
            "leading {trigger:?} must be neutralized with an apostrophe: {escaped:?}"
        );
    }
}

#[test]
fn apostrophe_neutralization_lands_inside_rfc4180_quoting() {
    // Contains the separator, so RFC 4180 quoting also applies. The
    // apostrophe must be inside the quotes, not before them — a leading
    // apostrophe outside the quotes would not be part of the cell value.
    let escaped = csv_escape("=SUM(A1,A2)", ',');
    assert_eq!(escaped, "\"'=SUM(A1,A2)\"");
}

#[test]
fn bare_cr_triggers_quoting_alongside_newline() {
    let escaped = csv_escape("line1\rline2", ',');
    assert!(
        escaped.starts_with('"') && escaped.ends_with('"'),
        "a bare CR must trigger RFC 4180 quoting, same as a newline: {escaped:?}"
    );
}

#[test]
fn ordinary_values_are_unaffected() {
    assert_eq!(csv_escape("ordinary value", ','), "ordinary value");
    assert_eq!(csv_escape("has space", ','), "has space");
}

#[test]
fn separator_and_quote_quoting_is_unchanged() {
    assert_eq!(csv_escape("a,b", ','), "\"a,b\"");
    assert_eq!(csv_escape("a\"b", ','), "\"a\"\"b\"");
    assert_eq!(csv_escape("a\tb", '\t'), "\"a\tb\"");
}
