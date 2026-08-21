use crate::{Access, AccessMut};

impl<'a, T: Copy> Access for &'a T {
    type Target = T;

    fn get(&self) -> Self::Target {
        **self
    }
}

impl<'a, T: Copy> Access for &'a mut T {
    type Target = T;

    fn get(&self) -> Self::Target {
        **self
    }
}

impl<'a, T: Copy> AccessMut for &'a mut T {
    fn set(&mut self, value: T) {
        **self = value;
    }
}
