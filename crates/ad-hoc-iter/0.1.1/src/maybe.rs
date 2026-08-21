//! Iterator types that can be 0, 1, or more.
//!
//! This exists to prevent needless heap allocations.

#![allow(dead_code)]

/// An iterator that can yield `0,1,2+` items
#[derive(Debug, Clone)]
pub enum MaybeMany<T,U>
{
    None,
    One(T),
    Many(U),
}

impl<T,U> MaybeMany<T,U>   
{
    /// Try to guess the size.
    ///
    /// * `None` => `Some(0)`
    /// * `One(_)` => `Some(1)`
    /// * `Many(_) => `None`
    #[inline] pub const fn size_hint(&self) -> Option<usize>
    {
	match self {
	    Self::None => Some(0),
	    Self::One(_) => Some(1),
	    Self::Many(_) => None
	}
    }
    /// Is this an empty instance
    #[inline] pub const fn is_none(&self) -> bool
    {
	if let Self::None = self {
	    true
	} else {
	    false
	}
    }
    /// Is this a single value instance?
    #[inline] pub const fn is_single(&self) -> bool
    {
	if let Self::One(_) = self {
	    true
	} else {
	    false
	}
    }
    /// Is this a 2+ value instance?
    #[inline] pub const fn is_many(&self) -> bool
    {
	if let Self::Many(_) = self {
	    true
	} else {
	    false
	}
    }
    /// Map the single value with this function
    #[inline] pub fn map_many<F, A>(self, fun: F) -> MaybeMany<T, A>
    where F: FnOnce(U) -> A,
	  A: IntoIterator<Item=T>
    {
	match self {
	    Self::One(t) => MaybeMany::One(t),
	    Self::Many(m) => MaybeMany::Many(fun(m)),
	    Self::None => MaybeMany::None,
	}
    }
    
    /// Map the single value with this function
    #[inline] fn map_one<F>(self, fun: F) -> MaybeMany<T,U>
    where F: FnOnce(T) -> T
    {
	match self {
	    Self::One(t) => MaybeMany::One(fun(t)),
	    Self::Many(m) => MaybeMany::Many(m),
	    Self::None => MaybeMany::None,
	}
    }

    /// Map both the single and many results 
    fn map<Fo, Fm, A,B>(self, one: Fo, many: Fm) -> MaybeMany<A,B>
    where Fo: FnOnce(T) -> A,
	  Fm: FnOnce(U) -> B,
	  B: IntoIterator<Item=A>
    {
	match self {
	    Self::One(o) => MaybeMany::One(one(o)),
	    Self::Many(m) => MaybeMany::Many(many(m)),
	    Self::None => MaybeMany::None,
	}
    }
    /// Take the value from this instance and replace it with nothing.
    #[inline] pub fn take(&mut self) -> Self
    {
	std::mem::replace(self, Self::None)
    }
}

/// An iterator for `MaybeMany` instances.
#[non_exhaustive] #[derive(Debug, Clone)]
pub enum MaybeManyIter<T,U>
{
    None,
    One(std::iter::Once<T>),
    Many(std::iter::Fuse<U>),
}

impl<T,U> Iterator for MaybeManyIter<T,U>
where U: Iterator<Item=T>
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item>
    {
	match self {
	    Self::None => None,
	    Self::One(one) => one.next(),
	    Self::Many(many) => many.next(),
	}
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
	match self {
	    Self::None => (0, Some(0)),
	    Self::One(_) => (1, Some(1)),
	    Self::Many(many) => many.size_hint(),
	}
    }
}
impl<T,U: Iterator<Item=T>> std::iter::FusedIterator for MaybeManyIter<T,U>{}
impl<T,U: Iterator<Item=T>> std::iter::ExactSizeIterator for MaybeManyIter<T,U>
where U: ExactSizeIterator{}

impl<T, U: IntoIterator<Item=T>> IntoIterator for MaybeMany<T, U>
{
    type Item= T;
    type IntoIter = MaybeManyIter<T, <U as IntoIterator>::IntoIter>;

    fn into_iter(self) -> Self::IntoIter
    {
	match self {
	    Self::None => MaybeManyIter::None,
	    Self::One(one) => MaybeManyIter::One(std::iter::once(one)),
	    Self::Many(many) => MaybeManyIter::Many(many.into_iter().fuse())
	}
    }
}
