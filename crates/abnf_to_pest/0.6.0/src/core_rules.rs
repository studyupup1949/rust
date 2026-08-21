//! The predefined ABNF "core rules" of [RFC 5234 §B.1][core], made available
//! to `abnf_to_pest` as `PestyRule`s and auto-injected into rendered grammars
//! where referenced.
//!
//! [core]: https://tools.ietf.org/html/rfc5234#appendix-B.1

use abnf::types::{Node, Repeat, TerminalValues};
use std::collections::{HashMap, HashSet};

use crate::{collect_rulenames, PestyRule};

/// The predefined ABNF "core rules" of [RFC 5234 §B.1][core], as `PestyRule`s
/// ready to be merged into a pest grammar.
///
/// These are rules that ABNF defines once and for all (e.g. `ALPHA`, `DIGIT`,
/// `SP`, `WSP`) and that grammars may freely reference without redefining
/// them. Callers that want them unconditionally can merge the entries returned
/// here into their rule map before rendering.
///
/// The rule names are already pest identifiers — they never need escaping —
/// and are returned as-is.
///
/// [core]: https://tools.ietf.org/html/rfc5234#appendix-B.1
pub fn abnf_core_rules() -> impl Iterator<Item = (&'static str, PestyRule)> {
    abnf_core_rule_definitions()
        .into_iter()
        .map(|(name, node)| {
            (
                name,
                PestyRule {
                    silent: false,
                    node,
                },
            )
        })
}

/// Among the core rules, return those referenced by the grammar but not
/// defined by it.
///
/// Includes core rules pulled in transitively: pulling in `CRLF` also pulls in
/// `CR` and `LF`, pulling in `LWSP` also pulls in `WSP`, `CRLF`, `CR` and `LF`.
/// The result is ordered like the [RFC 5234 §B.1][core] core-rules table
/// (pest allows forward references, so order is purely cosmetic).
///
/// [core]: https://tools.ietf.org/html/rfc5234#appendix-B.1
pub(crate) fn referenced_core_rules(
    referenced: &HashSet<String>,
    defined: &HashSet<String>,
) -> Vec<(String, PestyRule)> {
    let core_list = abnf_core_rule_definitions();
    let core_map: HashMap<&'static str, &Node> =
        core_list.iter().map(|(name, node)| (*name, node)).collect();

    // Worklist: pop a referenced name; if it's a core rule that the grammar
    // doesn't define itself, emit it, and the rulenames it references become
    // referenced in turn.  Emitted rules are immediately added to `defined`,
    // so each core rule is processed at most once.
    let mut emitted: HashSet<&'static str> = HashSet::new();
    let mut defined = defined.clone();
    let mut needed = referenced.clone();
    while let Some(name) = needed.iter().next().cloned() {
        needed.remove(&name);
        if defined.contains(&name) {
            continue;
        }
        if let Some((&key, &node)) = core_map.get_key_value(name.as_str()) {
            emitted.insert(key);
            defined.insert(name);
            collect_rulenames(node, &mut needed);
        }
    }

    core_list
        .iter()
        .filter(|(name, _)| emitted.contains(name))
        .map(|(name, node)| {
            (
                name.to_string(),
                PestyRule {
                    silent: false,
                    node: node.clone(),
                },
            )
        })
        .collect()
}

/// The [RFC 5234 §B.1][core] core rules as `(name, Node)` pairs.
///
/// **Namespace contract:** the names are pest identifiers, i.e. already
/// escaped with [`escape_rulename`](crate::escape_rulename) semantics (`-`
/// becomes `_`, Rust/pest reserved words get a trailing `_`).  All current
/// core rule names are uppercase ASCII and therefore escape to themselves,
/// but future entries must uphold this contract, so that consumers can
/// compare and emit the names without any further escaping.
///
/// The `Node`s, on the other hand, hold raw ABNF rulenames: each entry
/// mirrors what the `abnf` crate would produce parsing the rulelist, so the
/// shared pretty-printer renders it the same way as a parsed rule would be.
/// Rulenames inside the nodes are escaped when collected
/// (`collect_rulenames`) and when rendered, which is where the two
/// namespaces meet.
///
/// [core]: https://tools.ietf.org/html/rfc5234#appendix-B.1
fn abnf_core_rule_definitions() -> Vec<(&'static str, Node)> {
    vec![
        // ALPHA = %x41-5A / %x61-7A  (pest built-in ASCII_ALPHA)
        ("ALPHA", Node::rulename("ASCII_ALPHA")),
        // BIT = "0" / "1"  (pest built-in ASCII_BIN_DIGIT)
        ("BIT", Node::rulename("ASCII_BIN_DIGIT")),
        // CHAR = %x01-7F
        (
            "CHAR",
            Node::terminal_values(TerminalValues::range(0x01, 0x7F)),
        ),
        // CR = %x0D
        (
            "CR",
            Node::terminal_values(TerminalValues::range(0x0D, 0x0D)),
        ),
        // CRLF = CR LF
        (
            "CRLF",
            Node::concatenation(&[Node::rulename("CR"), Node::rulename("LF")]),
        ),
        // DIGIT = %x30-39  (pest built-in ASCII_DIGIT)
        ("DIGIT", Node::rulename("ASCII_DIGIT")),
        // DQUOTE = %x22
        (
            "DQUOTE",
            Node::terminal_values(TerminalValues::range(0x22, 0x22)),
        ),
        // HEXDIG = DIGIT / "A" / ... / "F"  (pest built-in ASCII_HEX_DIGIT;
        // ABNF strings are case-insensitive, so lowercase a-f match too.)
        ("HEXDIG", Node::rulename("ASCII_HEX_DIGIT")),
        // HTAB = %x09
        (
            "HTAB",
            Node::terminal_values(TerminalValues::range(0x09, 0x09)),
        ),
        // LF = %x0A
        (
            "LF",
            Node::terminal_values(TerminalValues::range(0x0A, 0x0A)),
        ),
        // LWSP = *(WSP / CRLF WSP)
        (
            "LWSP",
            Node::repetition(
                Repeat::unbounded(),
                Node::group(Node::alternatives(&[
                    Node::rulename("WSP"),
                    Node::concatenation(&[Node::rulename("CRLF"), Node::rulename("WSP")]),
                ])),
            ),
        ),
        // OCTET = %x00-FF
        (
            "OCTET",
            Node::terminal_values(TerminalValues::range(0x00, 0xFF)),
        ),
        // SP = %x20
        (
            "SP",
            Node::terminal_values(TerminalValues::range(0x20, 0x20)),
        ),
        // VCHAR = %x21-7E
        (
            "VCHAR",
            Node::terminal_values(TerminalValues::range(0x21, 0x7E)),
        ),
        // WSP = SP / HTAB
        (
            "WSP",
            Node::alternatives(&[Node::rulename("SP"), Node::rulename("HTAB")]),
        ),
    ]
}
