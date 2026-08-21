use std::{borrow::Borrow, collections::HashMap, hash::Hash};

/// Extends the [`HashMap`] functionality.
pub trait HashmapExt<K, V> {
    /// Updates the value of a given key in the map.
    ///
    /// If the map does not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the updated value is returned.
    fn update<Q, F>(&mut self, k: &Q, f: F) -> Option<V>
    where
        Q: ?Sized,
        K: Borrow<Q> + PartialEq + Eq + Hash,
        Q: Hash + Eq,
        V: Clone,
        F: FnMut(&mut V);

    /// Updates the value of a given key in the map.
    ///
    /// If the map does not have this key present, the provided value is inserted.
    ///
    /// If the map did have this key present, the updated value is returned.
    fn update_or_insert<F>(&mut self, k: K, f: F, def: V) -> Option<V>
    where
        K: PartialEq + Eq + Hash,
        V: Clone,
        F: FnMut(&mut V);
}

impl<K, V> HashmapExt<K, V> for HashMap<K, V> {
    fn update<Q, F>(&mut self, k: &Q, mut f: F) -> Option<V>
    where
        Q: ?Sized,
        K: Borrow<Q> + PartialEq + Eq + Hash,
        Q: Hash + Eq,
        V: Clone,
        F: FnMut(&mut V),
    {
        if let Some(totals) = self.get_mut(k) {
            f(totals);
            Some(totals.clone())
        } else {
            None
        }
    }

    fn update_or_insert<F>(&mut self, k: K, f: F, def: V) -> Option<V>
    where
        K: PartialEq + Eq + Hash,
        V: Clone,
        F: FnMut(&mut V),
    {
        self.update(&k, f).or_else(|| {
            self.insert(k, def.clone());
            Some(def)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_exist_update() {
        let mut map: HashMap<usize, u8> = HashMap::new();
        let res = map.update(&1usize, |u| *u += 1);
        assert!(res.is_none())
    }

    #[test]
    fn exist_update() {
        let mut map: HashMap<usize, u8> = HashMap::new();
        let res = map.insert(1usize, 10);
        assert!(res.is_none());

        let res = map.update(&1usize, |u| *u += 1);
        assert!(res.is_some());
        assert_eq!(Some(11), res);
    }

    #[test]
    fn not_exist_continuation() {
        let mut map: HashMap<usize, u8> = HashMap::new();

        let res = map
            .update(&1usize, |u| *u += 1)
            .or_else(|| map.insert(1usize, 1).or(Some(1)));
        assert_eq!(res, Some(1));
    }

    #[test]
    fn exist_continuation() {
        let mut map: HashMap<usize, u8> = HashMap::new();
        let res = map.insert(1usize, 10);
        assert!(res.is_none());

        let res = map
            .update(&1usize, |u| *u += 1)
            .or_else(|| map.insert(1usize, 1).or(Some(1)));
        assert_eq!(res, Some(11));
    }

    #[test]
    fn not_exist_update_or_insert() {
        let mut map: HashMap<usize, u8> = HashMap::new();
        let res = map.update_or_insert(1usize, |u| *u += 1, 1);
        assert_eq!(res, Some(1));
    }

    #[test]
    fn exist_update_or_insert() {
        let mut map: HashMap<usize, u8> = HashMap::new();
        let res = map.insert(1usize, 10);
        assert!(res.is_none());

        let res = map.update_or_insert(1usize, |u| *u += 1, 1);
        assert_eq!(res, Some(11));
    }
}
