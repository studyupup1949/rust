use serde::{Deserialize, Serialize};
use std::cmp;
use std::fmt;

use crate::syntax::*;

use std::hash::Hash;

/// AclBlock representes a ACL block of CaitSith policy template.
/// An ACL block is composed of one `AclHeader` and several `AclRule`s.
///
#[derive(Clone, Serialize, Deserialize)]
pub struct AclBlock {
    pub header: AclHeader,
    pub rule: Vec<AclRule>,
}

impl fmt::Debug for AclBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AclBlock")
            .field(&self.header)
            .field(&self.rule)
            .finish()
    }
}

impl Drop for AclBlock {
    fn drop(&mut self) {
        self.clear();
    }
}

impl AclBlock {
    pub fn clear(&mut self) {
        self.header.clear();
        for i in 0..self.rule.len() {
            self.rule[i].clear();
        }
    }
    pub fn as_ref(&self) -> &Self {
        self
    }
}


/// AclHeader represents a header declaration which heads to Rules in an
/// ACL block.
///
/// It holds acl priority, target operation, optional attributes and
/// rule entities consiting an ACL block.
///
#[derive(Hash, Clone, Serialize, Deserialize)]
pub struct AclHeader {
    pub priority: u32,
    pub op: Op,
    pub attr: Vec<(Resource, Cond, String)>,
}

impl Drop for AclHeader {
    fn drop(&mut self) {
        self.clear();
    }
}

impl AclHeader {
    pub fn new() -> Self {
        AclHeader {
            priority: 0,
            op: Op::Read,
            attr: vec![],
        }
    }

    pub fn clear(&mut self) {
        self.priority = 0;
        self.op = Op::Error;
        for i in 0..self.attr.len() {
            self.attr[i].0 = Resource::Error;
            self.attr[i].1 = Cond::Error;
            self.attr[i].2 = String::new();
        }
    }
}

impl cmp::PartialEq for AclHeader {
    fn eq(&self, other: &Self) -> bool {
        if self.attr.len() != other.attr.len() {
            return false;
        }
        if self.priority != other.priority || self.op != other.op {
            return false;
        }
        for i in 0..self.attr.len() {
            if self.attr[i].0 != other.attr[i].0 {
                return false;
            } else if self.attr[i].1 != other.attr[i].1 {
                return false;
            }
        }
        true
    }
}

impl cmp::Eq for AclHeader {}

impl fmt::Display for AclHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut attr = String::new();
        for i in 0..self.attr.len() {
            attr += format!(
                " {}{}{}",
                self.attr[i].0.as_str(),
                self.attr[i].1.as_str(),
                self.attr[i].2
            )
            .as_str();
        }
        write!(f, "{} acl {}{}", self.priority, self.op.as_str(), attr)
    }
}

impl fmt::Debug for AclHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AclHeader")
            .field(&self.priority)
            .field(&self.op)
            .field(&self.attr)
            .finish()
    }
}

/// AclRule represents a rule declaration in an ACL block.
/// It holds priority, verb and optional attributes for the verb.
///
///
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AclRule {
    pub priority: u32,
    pub verb: Verb,
    pub attr: Vec<(Resource, Cond, String)>,
}

impl Drop for AclRule {
    fn drop(&mut self) {
        self.clear();
    }
}

impl AclRule {
    pub fn clear(&mut self) {
        self.priority = 0;
        self.verb = Verb::Error;
        for i in 0..self.attr.len() {
            self.attr[i].0 = Resource::Error;
            self.attr[i].1 = Cond::Error;
            self.attr[i].2 = String::new();
        }
    }
}

impl fmt::Display for AclRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut attr = String::new();
        for i in 0..self.attr.len() {
            attr += format!(
                " {}{}{}",
                self.attr[i].0.as_str(),
                self.attr[i].1.as_str(),
                self.attr[i].2
            )
            .as_str();
        }

        if self.verb != Verb::Audit {
            write!(f, "    {} {}{}", self.priority, self.verb.as_str(), attr)
        } else {
            write!(f, "    audit {}", self.priority)
        }
    }
}

impl fmt::Debug for AclRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AclRule")
            .field(&self.priority)
            .field(&self.verb)
            .field(&self.attr)
            .finish()
    }
}
