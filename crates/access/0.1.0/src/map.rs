use crate::{Access, AccessMut};

pub struct MapAccess<A, F> {
    access: A,
    from: F,
}

impl<'a, A, F> MapAccess<A, F> {
    pub(crate) fn new(access: A, from: F) -> Self {
        Self { access, from }
    }
}

impl<T, B, A, F> Access for MapAccess<A, F>
where
    A: Access<Target = T>,
    F: Fn(T) -> B,
{
    type Target = B;

    fn get(&self) -> B {
        (self.from)(self.access.get())
    }
}

pub struct MapAccessMut<A, F, I> {
    access: A,
    from: F,
    into: I,
}

impl<'a, A, F, I> MapAccessMut<A, F, I> {
    pub(crate) fn new(access: A, from: F, into: I) -> Self {
        Self { access, from, into }
    }
}

impl<T, B, A, F, I> Access for MapAccessMut<A, F, I>
where
    A: Access<Target = T>,
    F: Fn(T) -> B,
{
    type Target = B;

    fn get(&self) -> B {
        (self.from)(self.access.get())
    }
}

impl<T, B, A, F, I> AccessMut for MapAccessMut<A, F, I>
where
    A: AccessMut<Target = T>,
    F: Fn(T) -> B,
    I: Fn(B) -> T,
{
    fn set(&mut self, value: B) {
        self.access.set((self.into)(value));
    }
}
