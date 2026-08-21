//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use common::compile_alfa_src;
use common::get_nth_policy;
use pretty_assertions::assert_eq;
mod common;

// Integration tests for imports.
//
// This primarily handles sections 4.5 (namespaces) of
// alfa-for-xacml-v1.0-csd01

// Rule #1: child match: start from the source_namespace,
// append symbol, and check for match. (can only be one)

/// Rule from child match.
#[test]
fn rule_child_match() {
    let x = compile_alfa_src(
        r"
namespace main {
  rule R1 { permit }
  policy {
    apply firstApplicable
    R1
  }
}
",
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 1);
}

// Rule #2: root match: attempt to match the full symbol.

/// Rule from fully qualified name in same namespace
#[test]
fn rule_root_match_same_ns() {
    let x = compile_alfa_src(
        r"
namespace main {
  rule R1 { permit }
  policy {
    apply firstApplicable
    main.R1
  }
}
",
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 1);
}

/// Rule from fully qualified name in sibling
#[test]
fn rule_root_match_sibling_ns() {
    let x = compile_alfa_src(
        r"
namespace sibling { rule R1 { permit } }
namespace main {
  policy {
    apply firstApplicable
    sibling.R1
  }
}
",
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 1);
}

/// Rule from fully qualified name in sibling multi-level
#[test]
fn rule_root_match_sibling_multi_ns() {
    let x = compile_alfa_src(
        r"
namespace root.sibling { rule R1 { permit } }
namespace main {
  policy {
    apply firstApplicable
    root.sibling.R1
  }
}
",
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 1);
}

// R3: Static relative imports

/// Import a rule from a child namespace using relative name
#[test]
fn rule_relative_import() {
    let x = compile_alfa_src(
        r"
namespace main {
  import child.R1
  policy {
    apply firstApplicable
    R1
  }
  namespace child {
    rule R1 { permit }
  }
}
",
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 1);
}

/// Import a rule from a child namespace using relative name two
/// levels down
#[test]
fn rule_relative_import_multi() {
    let x = compile_alfa_src(
        r"
namespace main {
  import child.child.R1
  policy {
    apply firstApplicable
    R1
  }
  namespace child.child {
    rule R1 { permit }
  }
}
",
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 1);
}

// R4: Static absolute imports

/// Import a rule from a child namespace using an absolute name
#[test]
fn rule_absolute_import() {
    let x = compile_alfa_src(
        r"
namespace main {
  import main.child.R1
  policy {
    apply firstApplicable
    R1
  }
  namespace child {
    rule R1 { permit }
  }
}
",
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 1);
}

/// Import a rule from a sibling namespace using an absolute name
#[test]
fn rule_absolute_import_sibling() {
    let x = compile_alfa_src(
        r"
namespace root.child { rule R1 { permit } }
namespace main {
  import root.child.R1
  policy {
    apply firstApplicable
    R1
  }
}
",
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 1);
}

// R5: Wildcard relative imports

/// Import a rule from a child namespace using a wildcard
#[test]
fn rule_wildcard_relative_import() {
    let x = compile_alfa_src(
        r"
namespace main {
  import child.*
  policy {
    apply firstApplicable
    R1
  }
  namespace child {
    rule R1 { permit }
  }
}
",
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 1);
}

// R6: Wildcard absolute imports

/// Import a rule from a child namespace using a wildcard (absolute)
#[test]
fn rule_wildcard_absolute_import() {
    let x = compile_alfa_src(
        r"
namespace main {
  import main.child.*
  policy {
    apply firstApplicable
    R1
  }
  namespace child {
    rule R1 { permit }
  }
}
",
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 1);
}

/// Import a rule from a sibling namespace using a wildcard (absolute)
#[test]
fn rule_wildcard_absolute_sibling_import() {
    let x = compile_alfa_src(
        r"
namespace sibling { rule R1 { permit } }
namespace main {
  import sibling.*
  policy {
    apply firstApplicable
    R1
  }
}
",
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 1);
}

// Ambiguous imports

/// Static Relative Imports with multiple options
/// Import a rule from a child namespace using relative name
#[test]
#[should_panic(expected = "compile failed")]
fn rule_static_relative_import_ambiguous() {
    // two imports that resolve to the same name.
    let x = compile_alfa_src(
        r"
namespace main {
  import child_one.R1
  import child_two.R1
  policy {
    apply firstApplicable
    R1
  }
  namespace child_one {
    rule R1 { permit }
  }
  namespace child_two {
    rule R1 { permit }
  }
}
",
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 1);
}

/// Static Absolute Imports with multiple options
#[test]
#[should_panic(expected = "compile failed")]
fn rule_static_absolute_import_ambiguous() {
    // two imports that resolve to the same name.
    let x = compile_alfa_src(
        r"
namespace ambig1 { rule R1 { permit } }
namespace ambig2 { rule R1 { permit } }

namespace main {
  import ambig1.R1
  import ambig2.R1
  policy {
    apply firstApplicable
    R1
  }
}
",
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 1);
}

/// Wildcard Relative Imports with multiple options
#[test]
#[should_panic(expected = "compile failed")]
fn rule_wildcard_relative_import_ambiguous() {
    // two imports that resolve to the same name.
    let x = compile_alfa_src(
        r"
namespace main {
  import ambig1.*
  import ambig2.*
  policy {
    apply firstApplicable
    R1
  }
  namespace ambig1 { rule R1 { permit } }
  namespace ambig2 { rule R1 { permit } }
}
",
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 1);
}

/// Wildcard Absolute Imports with multiple options
#[test]
#[should_panic(expected = "compile failed")]
fn rule_wildcard_absolute_import_ambiguous() {
    // two imports that resolve to the same name.
    let x = compile_alfa_src(
        r"
namespace ambig1 { rule R1 { permit } }
namespace ambig2 { rule R1 { permit } }
namespace main {
  import ambig1.*
  import ambig2.*
  policy {
    apply firstApplicable
    R1
  }
}
",
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 1);
}

// Other tests

/// Import a non-existent namespace
#[test]
fn ignore_invalid_namespace() {
    // import doesn't resolve to anything, but that doesn't cause
    // compilation failure since the import is not needed.
    let x = compile_alfa_src(
        r"
namespace main {
  import madeup.R1
  policy {
    apply firstApplicable
    rule R1 { permit }
  }
}
",
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 1);
}
