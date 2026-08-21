#![allow(dead_code)]

use std::rc::Rc;
use itertools::Itertools;
use super::Sequence;


#[derive(Clone)]
/// Enum for the possible bounding behaviors
pub (crate) enum BoundingBehavior<'t, T>
where T: 't {
    Bounded(Rc<dyn Sequence<Term=T> + 't>,),
    Unbounded,
    Unknown,
}


impl<T> std::fmt::Debug for BoundingBehavior<'_, T>
where T: std::fmt::Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoundingBehavior::Bounded(s) => {
                let term_str = match s.approx_num_terms() {
                    Some(n) if n <= 5 => s.terms()
                        .enumerate()
                        .map(|(idx, t)| format!("{t:?} * k_{idx}"))
                        .join(" + "),
                    _ => s.terms()
                        .enumerate()
                        .take(5)
                        .map(|(idx, t)| format!("{t:?} * k_{idx}"))
                        .chain(std::iter::once("...".to_string()))
                        .join(" + "),
                };
                write!(f, "Bounded({term_str}")
            },
            BoundingBehavior::Unbounded => write!(f, "Unbounded"),
            BoundingBehavior::Unknown => write!(f, "Unknown"),
        }
    }
}
