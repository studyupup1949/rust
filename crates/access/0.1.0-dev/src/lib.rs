mod map_access;
mod ref_access;

pub use map_access::MapAccess;
pub use ref_access::RefAccess;

pub trait Access<T> {
    fn get(&self) -> T;
    fn set(&mut self, new_value: T);

    fn map<B, F, I>(self, from: F, into: I) -> MapAccess<Self, F, I>
    where
        Self: Sized,
        F: Fn(T) -> B,
        I: Fn(B) -> T,
    {
        MapAccess::new(self, from, into)
    }
}
