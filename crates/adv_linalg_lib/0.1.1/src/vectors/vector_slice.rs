mod no_std {
    #![cfg(feature = "no_std")]

    use core::ops::{Range, Index};
    use crate::vectors::VectorSlice;

    impl<'v, T> VectorSlice<'v, T> {
        pub fn len(&self) -> usize {
            self.values.len()
        }

        pub fn as_slice(&'v self, range: Range<usize>) -> VectorSlice<'v, T> {
            VectorSlice {
                values: self.values
                            .split_at(range.start).1
                            .split_at(range.len()).0
            }
        }
    }
    impl<'v, T, U> From<U> for VectorSlice<'v, T>
    where
        U: Into<&'v [T]>
    {
        fn from(values: U) -> Self {
            VectorSlice { values: values.into() }
        }
    }
    impl<'v, T> Index<usize> for VectorSlice<'v, T>
    where
        T: Clone {
        type Output = T;

        fn index(&self, index: usize) -> &Self::Output {
            &self.values[index]
        }
    }
}

mod full {
    #![cfg(feature = "full")]

    use alloc::vec::Vec;
    use crate::vectors::{Vector, VectorSlice};

    impl<'v, T> VectorSlice<'v, T> {
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
}