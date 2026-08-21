use indexmap::IndexSet;
use std::{
    collections::HashSet,
    hash::{BuildHasher, Hash},
};
use syn::punctuated::Punctuated;

pub fn extend_unique_by_key<T, K, P>(
    dst: &mut Punctuated<T, P>,
    src: impl IntoIterator<Item = T>,
    mut key: impl FnMut(&T) -> K,
) where
    P: Default,
    K: Eq + Hash,
{
    let mut seen: HashSet<K> = dst.iter().map(|t| key(t)).collect();
    for item in src {
        let k = key(&item);
        if seen.insert(k) {
            dst.push(item);
        }
    }
}

pub fn merge_unique_punctuated<T, P, S>(
    punctuated: Punctuated<T, P>,
    other_punctuateds: impl IntoIterator<Item = Punctuated<T, P>>,
) -> Punctuated<T, P>
where
    T: Eq + Hash,
    P: Default,
    S: BuildHasher + Default,
{
    let mut set: IndexSet<T, S> = punctuated.into_iter().collect();

    for param_list in other_punctuateds {
        for captured in param_list {
            set.insert(captured);
        }
    }

    Punctuated::<T, P>::from_iter(set)
}
