use crate::Access;

pub struct MapAccess<A, F, I> {
    access: A,
    from: F,
    into: I,
}

impl<A, F, I> MapAccess<A, F, I> {
    pub fn new(access: A, from: F, into: I) -> Self {
        Self { access, from, into }
    }
}

impl<A, F, I, T, B> Access<B> for MapAccess<A, F, I>
where
    A: Access<T>,
    F: Fn(T) -> B,
    I: Fn(B) -> T,
{
    fn get(&self) -> B {
        (self.from)(self.access.get())
    }

    fn set(&mut self, new_value: B) {
        self.access.set((self.into)(new_value));
    }
}
