//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

/// An operator identifier.
///
/// This records the name of an operator, exactly as it was declared
/// in the Alfa text.  The name is broken out into the namespace
/// components, which may be empty, and the operator name itself,
/// which must be composed of symbols.
///
/// Operators only appear in targets and conditions, so they need to
/// be resolved based on the namespace that their target/condition is
/// defined in.

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Operator {
    pub ns: Vec<String>,  // qualified name, not including operator
    pub operator: String, // operator name (symbols)
}

impl Operator {
    /// Returns the fully qualified operator name
    #[must_use]
    pub fn qualified_name(&self) -> String {
        if self.ns.is_empty() {
            self.operator.clone()
        } else {
            format!("{}.{}", self.ns.join("."), self.operator)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qualified_name_no_ns() {
        let o = Operator {
            ns: vec![],
            operator: "++".to_string(),
        };
        assert_eq!(o.qualified_name(), "++".to_string());
    }

    #[test]
    fn test_qualified_name_with_ns() {
        let o = Operator {
            ns: vec!["foo".to_string(), "bar".to_string()],
            operator: "++".to_string(),
        };
        assert_eq!(o.qualified_name(), "foo.bar.++".to_string());
    }
}
