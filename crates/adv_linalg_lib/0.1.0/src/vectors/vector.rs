#![cfg(feature = "full")]

use alloc::vec::Vec;
use core::ops::{Range, Index};
use super::{Vector, VectorSlice};

impl<T> Vector<T> {
    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn as_slice<'v>(&'v self, range: Range<usize>) -> VectorSlice<'v, T> {
        VectorSlice {
            values: self.values
                        .as_slice()
                        .split_at(range.start).1
                        .split_at(range.len()).0
        }
    }
}
impl<T> Vector<T> {
    pub fn lambda<F>(&self, f: F) -> Vector<T>
    where
        F: Fn(&T) -> T {
        Vector::from(
            self.values
                .iter()
                .map(|value| f(value))
                .collect::<Vec<T>>()
        )
    }

    pub fn lambda_index<F>(&self, f: F) -> Vector<T>
    where
        F: Fn(usize) -> T {
        Vector::from(
            self.values
                .iter()
                .enumerate()
                .map(|(index, _)| f(index))
                .collect::<Vec<T>>()
        )
    }

    pub fn lambda_enumerate<F>(&self, f: F) -> Vector<T>
    where
        F: Fn(usize, &T) -> T {
            Vector::from(
                self.values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| f(index, value))
                    .collect::<Vec<T>>()
            )
    }
}
impl<T, U> From<U> for Vector<T>
where
    U: Into<Vec<T>>
{
    fn from(values: U) -> Self {
        Vector { values: values.into() }
    }
}
impl<'v, T> Index<usize> for Vector<T>
where
    T: Clone {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}