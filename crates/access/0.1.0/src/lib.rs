mod map;
mod reference;

pub use map::{MapAccess, MapAccessMut};

pub trait Access {
    type Target;

    fn get(&self) -> Self::Target;

    fn map<T, B, F>(self, from: F) -> MapAccess<Self, F>
    where
        Self: Sized + Access<Target = T>,
        F: Fn(T) -> B,
    {
        MapAccess::new(self, from)
    }
}

pub trait AccessMut: Access {
    fn set(&mut self, value: Self::Target);

    fn map_mut<T, B, F, I>(self, from: F, into: I) -> MapAccessMut<Self, F, I>
    where
        Self: Sized + Access<Target = T>,
        F: Fn(T) -> B,
        I: Fn(B) -> T,
    {
        MapAccessMut::new(self, from, into)
    }
}
