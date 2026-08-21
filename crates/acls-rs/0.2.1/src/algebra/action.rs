//! Monoid action algebraic structure.
//!
//! A monoid action allows elements of a monoid to act on elements of another type,
//! with the action respecting the monoid structure.

use super::Monoid;

/// A monoid action: a way for a monoid to transform values of another type.
///
/// # Algebraic Laws
///
/// For a type to implement `MonoidAction<M, T>`, it must satisfy:
///
/// - **Identity**: `act(M::identity(), t) = t` for all `t`
/// - **Compatibility**: `act(m1.combine(m2), t) = act(m1, act(m2, t))` for all `m1, m2, t`
///
/// # Type Parameters
///
/// - `M`: The monoid type that acts on `T`
/// - `T`: The target type being transformed
///
/// # Examples
///
/// ```
/// use acls_rs::algebra::MonoidAction;
/// use acls_rs::permission::PermissionDelta;
/// use acls_rs::Subject;
///
/// // PermissionDelta acts on Subject
/// let subject = Subject::new("alice");
/// let delta = PermissionDelta::builder()
///     .grant_str("file", "read")
///     .build();
///
/// let updated = PermissionDelta::act(delta, subject);
/// // subject now has file:read permission
/// ```
pub trait MonoidAction<M: Monoid, T> {
    /// Apply the monoid element to the target value.
    ///
    /// This operation must satisfy:
    /// - **Identity**: `act(M::identity(), t) == t`
    /// - **Compatibility**: `act(m1.combine(m2), t) == act(m1, act(m2, t))`
    ///
    /// # Arguments
    ///
    /// - `monoid`: The monoid element to apply
    /// - `target`: The value to transform
    ///
    /// # Returns
    ///
    /// The transformed value.
    fn act(monoid: M, target: T) -> T;

    /// Preview the action without consuming the target.
    ///
    /// This is a convenience method that clones the inputs before applying the action.
    ///
    /// # Arguments
    ///
    /// - `monoid`: The monoid element to preview
    /// - `target`: The value to preview the transformation on
    ///
    /// # Returns
    ///
    /// The result of applying the action (original target is unchanged).
    fn preview_act(monoid: &M, target: &T) -> T
    where
        M: Clone,
        T: Clone,
    {
        Self::act(monoid.clone(), target.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    // Example monoid action: adding elements to a set
    struct SetAddAction;

    impl<T: Ord + Clone> MonoidAction<BTreeSet<T>, BTreeSet<T>> for SetAddAction {
        fn act(monoid: BTreeSet<T>, target: BTreeSet<T>) -> BTreeSet<T> {
            target.union(&monoid).cloned().collect()
        }
    }

    #[test]
    fn test_action_identity() {
        let target: BTreeSet<i32> = [1, 2, 3].iter().cloned().collect();
        let identity = BTreeSet::identity();

        let result = SetAddAction::act(identity, target.clone());
        assert_eq!(result, target);
    }

    #[test]
    fn test_action_compatibility() {
        use crate::algebra::Semigroup;

        let m1: BTreeSet<i32> = [4, 5].iter().cloned().collect();
        let m2: BTreeSet<i32> = [6, 7].iter().cloned().collect();
        let target: BTreeSet<i32> = [1, 2, 3].iter().cloned().collect();

        let left = SetAddAction::act(m1.clone().combine(m2.clone()), target.clone());
        let right = SetAddAction::act(m1, SetAddAction::act(m2, target));

        assert_eq!(left, right);
    }

    #[test]
    fn test_preview_act() {
        let monoid: BTreeSet<i32> = [4, 5].iter().cloned().collect();
        let target: BTreeSet<i32> = [1, 2, 3].iter().cloned().collect();

        let result = SetAddAction::preview_act(&monoid, &target);

        // Original target should be unchanged
        assert_eq!(target, [1, 2, 3].iter().cloned().collect());
        // Result should have union
        assert_eq!(result, [1, 2, 3, 4, 5].iter().cloned().collect());
    }
}
