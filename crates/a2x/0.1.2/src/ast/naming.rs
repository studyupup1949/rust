// Dynamic tracking of names.
//
// For anonymous policysets/policies, we need to output these as part
// of the ID, but we won't know these till serialization time.  This
// module gives us slots to create these and share them.

// each element of the Vec is a Cell that can be updated as the name
// is finally determined.

// the way this will work is that the first container will create a
// GenName struct and will store the Cell within its own struct.

use std::default::Default;
use std::{cell::RefCell, rc::Rc};

pub type NameSlot = Rc<RefCell<Option<String>>>;

#[derive(Default, Debug, PartialEq, Clone)]
pub struct GenName {
    name_path: Vec<NameSlot>,
}

impl GenName {
    pub fn push_name(&mut self, elem: NameSlot) {
        self.name_path.push(elem);
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.name_path.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.name_path.is_empty()
    }

    pub fn push_name_at_index(&mut self, elem: NameSlot, idx: usize) {
        self.name_path.insert(idx, elem);
    }
    // Check if the name is resolvable
    #[must_use]
    pub fn is_resolvable(&self) -> bool {
        self.name_path.iter().all(|x| !x.borrow().is_none())
    }
    // Get the full path ("." separated), if resolveable
    #[must_use]
    pub fn build_path(&self, sep: &str) -> Option<String> {
        self.name_path
            .iter()
            .map(|x| x.borrow().clone())
            .collect::<Option<Vec<String>>>()
            .map(|parts| parts.join(sep))
    }

    // Get reference to the last element
    #[must_use]
    pub fn last_elem(&self) -> Option<NameSlot> {
        self.name_path.last().cloned()
    }

    // Policy definition level
    #[must_use]
    pub fn policy_level(&self) -> usize {
        self.name_path.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let _n = GenName::default();
    }

    #[test]
    fn test_push() {
        let mut n = GenName::default();
        let x = Rc::new(RefCell::new(None));
        n.push_name(x);
        // get the name back
    }
    #[test]
    fn test_resolvable() {
        let mut n = GenName::default();
        // we can resolve an empty name
        assert!(n.is_resolvable());
        let x = Rc::new(RefCell::new(None));
        n.push_name(x.clone());
        // once a empty name is added, resolution fails.
        assert!(!n.is_resolvable());
        // Resolve the name that we inserted.
        x.replace(Some("a-name".to_owned()));
        // Now, we can resolve
        assert!(n.is_resolvable());
    }
}
