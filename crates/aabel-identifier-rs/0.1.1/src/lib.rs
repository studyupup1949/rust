/// Identity trait represents data which can be used
/// as identifiers. Given two identifiers we just need
/// to determine if they are equal (represent the same entity)
/// or not.
///
/// # Example
///
/// ```
/// use aabel_identifier_rs::*;
///
/// fn test_identifier(_id: impl Identifier) {
///     assert!(true);
/// }
///
/// let id = 10_u8;
/// test_identifier(id);
/// ```
pub trait Identifier: Eq {}
impl<T> Identifier for T where T: Eq {}

/// The PartialOrdIdentity trait represents data which can be
/// used for concurrent actions. Given two such identifiers, we
/// can tell if they are equal and we could potentially compare them.
pub trait PartialOrdIdentifier: Identifier + PartialOrd {}
impl<T> PartialOrdIdentifier for T where T: Eq + PartialOrd {}

/// The IntoIdentityIterator trait allows to create a sequence of
/// identifiers starting with a given identifier value and using a
/// given function which computes a new identifier.
///
/// # Example
///
/// ```
/// use aabel_identifier_rs::*;
///
/// let id = 10_u8;
/// let mut iter = id.into_ids_iterator(|id| id + 1);
/// assert_eq!(iter.next(), Some(10));
/// assert_eq!(iter.next(), Some(11));
/// ```
pub trait IntoIdentifierIterator {
    type Item: Identifier;

    fn into_ids_iterator<F>(self, next: F) -> impl Iterator<Item = Self::Item>
    where
        F: Fn(&Self::Item) -> Self::Item;
}

/// Internal implementation of the [`IntoIdentityIterator`] trait.
/// The iterator will return an [`Identifier`] instance. The identifier
/// type must also implement [`Copy`] trait.
struct IdentifierIterator<F, I>
where
    F: Fn(&I) -> I,
    I: Copy,
{
    current_id: I,
    get_next_id: F,
}

impl<F, I> Iterator for IdentifierIterator<F, I>
where
    F: Fn(&I) -> I,
    I: Copy,
{
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        let nxt = self.current_id;
        self.current_id = (self.get_next_id)(&self.current_id);
        Some(nxt)
    }
}

impl<T> IntoIdentifierIterator for T
where
    T: Copy + Identifier,
{
    type Item = Self;

    fn into_ids_iterator<F>(self, next: F) -> impl Iterator<Item = Self::Item>
    where
        F: Fn(&Self::Item) -> Self::Item,
    {
        IdentifierIterator {
            current_id: self,
            get_next_id: next,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_identifier(_id: impl Identifier) {
        assert!(true);
    }

    fn test_partialord_identifier(_id: impl PartialOrdIdentifier) {
        assert!(true);
    }

    #[test]
    fn identifier_u8() {
        let id = 10_u8;
        test_identifier(id)
    }

    #[test]
    fn identifier_u64() {
        let id = 10_u64;
        test_identifier(id)
    }

    #[test]
    fn identifier_str() {
        let id = "test";
        test_identifier(id)
    }

    #[test]
    fn identifier_arr() {
        let id = [1, 2, 3];
        test_identifier(id)
    }

    #[test]
    fn identifier_vec() {
        let id = vec![1, 2, 3];
        test_identifier(id)
    }

    #[test]
    fn identifier_eq() {
        fn cmp_identifiers<I>(a: I, b: I) -> bool
        where
            I: Identifier,
        {
            a == b
        }

        assert!(cmp_identifiers(10_u16, 10_u16));
        assert!(!cmp_identifiers(10_u16, 20_u16));
    }

    #[test]
    fn partialord_identifier_u8() {
        let id = 10_u8;
        test_partialord_identifier(id)
    }

    #[test]
    fn partialord_identifier_u64() {
        let id = 10_u64;
        test_partialord_identifier(id)
    }

    #[test]
    fn partialord_identity_str() {
        let id = "test";
        test_partialord_identifier(id)
    }

    #[test]
    fn partialord_identifier_arr() {
        let id = [1, 2, 3];
        test_partialord_identifier(id)
    }

    #[test]
    fn partialord_identifier_vec() {
        let id = vec![1, 2, 3];
        test_partialord_identifier(id)
    }

    #[test]
    fn partialord_idenitifier_cmp() {
        fn cmp_identifiers<I>(a: I, b: &I) -> Option<std::cmp::Ordering>
        where
            I: PartialOrdIdentifier,
        {
            a.partial_cmp(b)
        }

        assert!(cmp_identifiers(10_u8, &10_u8).is_some());
        assert!(cmp_identifiers("az", &"za").is_some());
    }

    #[test]
    fn iter_u8() {
        let id = 10_u8;
        let mut iter = id.into_ids_iterator(|id| id + 1);
        assert_eq!(iter.next(), Some(10));
        assert_eq!(iter.next(), Some(11));
    }

    #[test]
    fn iter_u64() {
        let id = 10_u64;
        let mut iter = id.into_ids_iterator(|id| id + 1);
        assert_eq!(iter.next(), Some(10));
        assert_eq!(iter.next(), Some(11));
    }

    #[test]
    fn iter_str() {
        let id = "a";
        let mut iter = id.into_ids_iterator(|id| id);
        assert_eq!(iter.next(), Some("a"));
        assert_eq!(iter.next(), Some("a"));
    }

    #[test]
    fn iter_string() {
        let id = "a".to_string();
        let mut iter = id.into_ids_iterator(|id| id);
        assert_eq!(iter.next(), Some(&"a".into()));
        assert_eq!(iter.next(), Some(&"a".into()));
    }
}
