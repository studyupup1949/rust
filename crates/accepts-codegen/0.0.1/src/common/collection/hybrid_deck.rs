use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HybridDeck<T, const C: usize> {
    inline: [T; C],
    extras: Vec<T>,
}

#[allow(dead_code)]
impl<T, const C: usize> HybridDeck<T, C> {
    pub fn from_inline(inline: [T; C]) -> Self {
        Self {
            inline,
            extras: Vec::new(),
        }
    }

    pub fn from_parts(inline: [T; C], extras: Vec<T>) -> Self {
        Self { inline, extras }
    }

    pub fn into_parts(self) -> ([T; C], Vec<T>) {
        (self.inline, self.extras)
    }

    pub fn inline(&self) -> &[T; C] {
        &self.inline
    }
    pub fn inline_mut(&mut self) -> &mut [T; C] {
        &mut self.inline
    }

    pub fn extras(&self) -> &Vec<T> {
        &self.extras
    }
    pub fn extras_mut(&mut self) -> &mut Vec<T> {
        &mut self.extras
    }

    pub fn push_extra(&mut self, v: T) {
        self.extras.push(v);
    }

    pub fn len_total(&self) -> usize {
        C + self.extras.len()
    }

    pub fn get(&self, i: usize) -> Option<&T> {
        if i < C {
            Some(&self.inline[i])
        } else {
            self.extras.get(i - C)
        }
    }
}

impl<T, const C: usize> IntoIterator for HybridDeck<T, C> {
    type Item = T;
    type IntoIter = std::iter::Chain<std::array::IntoIter<T, C>, std::vec::IntoIter<T>>;
    fn into_iter(self) -> Self::IntoIter {
        (self.inline.into_iter()).chain(self.extras.into_iter())
    }
}

impl<T, const C: usize> Index<usize> for HybridDeck<T, C> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        let inline_len = self.inline.len();
        if index < inline_len {
            &self.inline[index]
        } else {
            &self.extras[index - inline_len]
        }
    }
}
impl<T, const C: usize> IndexMut<usize> for HybridDeck<T, C> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let inline_len = self.inline.len();
        if index < inline_len {
            &mut self.inline[index]
        } else {
            &mut self.extras[index - inline_len]
        }
    }
}

impl<T, const C: usize> From<[T; C]> for HybridDeck<T, C> {
    fn from(inline: [T; C]) -> Self {
        Self::from_inline(inline)
    }
}
impl<T, const C: usize> From<([T; C], Vec<T>)> for HybridDeck<T, C> {
    fn from(p: ([T; C], Vec<T>)) -> Self {
        Self::from_parts(p.0, p.1)
    }
}

impl<T, const C: usize> Extend<T> for HybridDeck<T, C> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.extras.extend(iter);
    }
}
