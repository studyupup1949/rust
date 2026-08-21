mod no_std {
    #![cfg(feature = "no_std")]
    
    use core::ops::{Range, Index, IndexMut};
    use crate::vectors::{VectorSlice, MutVectorSlice};

    impl<'v, T> MutVectorSlice<'v, T> {
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

    impl<'v, T> MutVectorSlice<'v, T> {
        pub fn lambda_mut<F>(&'v mut self, f: F) -> &'v mut Self
        where
            F: Fn(&mut T) {
            self.values.iter_mut().for_each(|value| f(value));
            self
        }

        pub fn lambda_index_mut<F>(&'v mut self, f: F) -> &'v mut Self
        where
            F: Fn(usize) {
            self.values.iter_mut().enumerate().for_each(|(index, _)| f(index));
            self
        }

        pub fn lambda_enumerate_mut<F>(&'v mut self, f: F) -> &'v mut Self
        where
            F: Fn(usize, &mut T) {
            self.values.iter_mut().enumerate().for_each(|(index, value)| f(index, value));
            self
        }
    }

    impl<'v, T, U> From<U> for MutVectorSlice<'v, T>
    where
        U: Into<&'v mut [T]>
    {
        fn from(values: U) -> Self {
            MutVectorSlice { values: values.into() }
        }
    }
    impl<'v, T> Index<usize> for MutVectorSlice<'v, T>
    where
        T: Clone {
        type Output = T;

        fn index(&self, index: usize) -> &Self::Output {
            &self.values[index]
        }
    }
    impl<'v, T> IndexMut<usize> for MutVectorSlice<'v, T>
    where
        T: Clone,
    {
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            &mut self.values[index]
        }
    }
}

mod full {
    #![cfg(feature = "full")]

    use alloc::vec::Vec;
    use crate::vectors::{Vector, MutVectorSlice};

    impl<'v, T> MutVectorSlice<'v, T> {
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