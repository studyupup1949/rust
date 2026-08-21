use crate::Access;
use std::ops::{Deref, DerefMut};

pub struct RefAccess<'a, T>(&'a mut T);

impl<'a, T> RefAccess<'a, T> {
    pub fn new(value: &'a mut T) -> Self {
        Self(value)
    }
}

impl<'a, T> From<&'a mut T> for RefAccess<'a, T> {
    fn from(value: &'a mut T) -> Self {
        Self::new(value)
    }
}

impl<T> Deref for RefAccess<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<T> DerefMut for RefAccess<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<T: Clone> Access<T> for RefAccess<'_, T> {
    fn get(&self) -> T {
        self.0.clone()
    }

    fn set(&mut self, new_value: T) {
        *self.0 = new_value;
    }
}
