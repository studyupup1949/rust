use crate::tuples::*;
use core::marker::PhantomData;

pub trait TupleAtTrait<T: ?Sized> {
    fn at_trait<'a>(&'a self, index: usize) -> Option<&'a T>;
}

pub struct TupleTraitIterator<'a, T: ?Sized, U: ?Sized> {
    tuple: &'a T,
    index: usize,
    phantom: PhantomData<&'a U>,
}

impl<'a, T, U: ?Sized> Iterator for TupleTraitIterator<'a, T, U>
where
    T: TupleAtTrait<U> + ?Sized,
{
    type Item = &'a U;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.tuple.at_trait(self.index);
        self.index += 1;
        result
    }
}

pub trait TupleTraitIter {
    fn iter_trait<U>(&self) -> impl Iterator<Item = &U>
    where
        U: ?Sized + 'static,
        Self: TupleAtTrait<U>;
}

impl<J> TupleTraitIter for J {
    fn iter_trait<U>(&self) -> impl Iterator<Item = &U>
    where
        U: ?Sized + 'static,
        Self: TupleAtTrait<U>,
    {
        TupleTraitIterator::<Self, U> {
            tuple: self,
            index: 0,
            phantom: PhantomData,
        }
    }
}

macro_rules! impl_tuple_trait {
    ($len:tt, ( $($idx:tt => $T:ident),+ $(,)? )) => {
        impl<T: ?Sized, $($T,)+> TupleAtTrait<T> for ( $($T,)+ )
        where
            $( T: AsTraitRef<$T>, )+
            $( $T: 'static ),+
        {
            fn at_trait<'a>(&'a self, index: usize) -> Option<&'a T> {
                match index {
                    $(
                        $idx => Some(T::as_trait_ref(&self.$idx)),
                    )+
                    _ => None,
                }
            }
        }

        impl<T: ?Sized, $($T,)+> TupleAtTraitMut<T> for ( $($T,)+ )
        where
            $( T: AsTraitMut<$T>, )+
            $( $T: 'static ),+
        {
            fn at_trait_mut<'a>(&'a mut self, index: usize) -> Option<&'a mut T> {
                match index {
                    $( $idx => {
                        let r: &mut $T = &mut self.$idx;
                        Some(T::as_trait_mut(r))
                    }, )+
                    _ => None,
                }
            }
        }
    };
}

pub trait TupleAtTraitMut<T: ?Sized> {
    fn at_trait_mut<'a>(&'a mut self, index: usize) -> Option<&'a mut T>;
}

pub struct TupleTraitIteratorMut<'a, T: ?Sized, U: ?Sized> {
    tuple: &'a mut T,
    index: usize,
    phantom: PhantomData<&'a mut U>,
}

impl<'a, T, U: ?Sized> Iterator for TupleTraitIteratorMut<'a, T, U>
where
    T: TupleAtTraitMut<U> + ?Sized,
{
    type Item = &'a mut U;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.index;
        self.index += 1;
        // SAFETY: unique borrow guaranteed by iteration
        unsafe {
            let ptr = self.tuple as *mut T;
            (*ptr).at_trait_mut(i)
        }
    }
}

pub trait TupleTraitIterMut {
    fn iter_trait_mut<U>(&mut self) -> impl Iterator<Item = &mut U>
    where
        U: ?Sized + 'static,
        Self: TupleAtTraitMut<U>;
}

impl<J> TupleTraitIterMut for J {
    fn iter_trait_mut<U>(&mut self) -> impl Iterator<Item = &mut U>
    where
        U: ?Sized + 'static,
        Self: TupleAtTraitMut<U>,
    {
        TupleTraitIteratorMut::<Self, U> {
            tuple: self,
            index: 0,
            phantom: PhantomData,
        }
    }
}

impl_tuple_trait!(1, (0 => A));
impl_tuple_trait!(2, (0 => A, 1 => B));
impl_tuple_trait!(3, (0 => A, 1 => B, 2 => C));
impl_tuple_trait!(4, (0 => A, 1 => B, 2 => C, 3 => D));
impl_tuple_trait!(5, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E));
impl_tuple_trait!(6, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F));
impl_tuple_trait!(7, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G));
impl_tuple_trait!(8, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H));
impl_tuple_trait!(9, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I));
impl_tuple_trait!(10, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J));
impl_tuple_trait!(11, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K));
impl_tuple_trait!(12, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L));
impl_tuple_trait!(13, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M));
impl_tuple_trait!(14, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M, 13 => N));
impl_tuple_trait!(15, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M, 13 => N, 14 => O));
impl_tuple_trait!(16, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M, 13 => N, 14 => O, 15 => P));

#[cfg(test)]
mod test {
    use crate::{self as adar, prelude::*};
    use num_traits::One;
    use std::ops::Add;

    #[test]
    fn test_tuple_trait_iter_to_string() {
        let tuple = ("0", true, 2.0, 3);
        let mut iter = tuple.iter_trait::<dyn ToString>();
        assert_eq!(iter.next().map(|v| v.to_string()), Some("0".into()));
        assert_eq!(iter.next().map(|v| v.to_string()), Some("true".into()));
        assert_eq!(iter.next().map(|v| v.to_string()), Some("2".into()));
        assert_eq!(iter.next().map(|v| v.to_string()), Some("3".into()));
        assert_eq!(iter.next().map(|v| v.to_string()), None);
    }

    #[test]
    fn test_tuple_trait_iter16() {
        let tuple = (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let mut iter = tuple.iter_trait::<dyn ToString>();
        for i in 0..16 {
            assert_eq!(iter.next().map(|v| v.to_string()), Some(i.to_string()));
        }
        assert_eq!(iter.next().map(|v| v.to_string()), None);
    }

    #[test]
    fn test_tuple_trait_iter_mut16() {
        let mut tuple = (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let mut iter = tuple.iter_trait_mut::<dyn ToString>();
        for i in 0..16 {
            assert_eq!(iter.next().map(|v| v.to_string()), Some(i.to_string()));
        }
        assert_eq!(iter.next().map(|v| v.to_string()), None);
    }

    #[test]
    fn test_tuple_trait_iter_mut_custom() {
        #[TraitRef]
        pub trait DynIncrement: Send + Sync {
            fn inc(&mut self);
        }

        impl<T> DynIncrement for T
        where
            T: Copy + Add<Output = T> + One + Send + Sync,
        {
            fn inc(&mut self) {
                *self = *self + T::one();
            }
        }
        let mut tuple = (0_i32, 1.0_f32, 2_u8);
        tuple
            .iter_trait_mut::<dyn DynIncrement>()
            .for_each(|v| v.inc());

        assert_eq!(tuple, (1_i32, 2.0_f32, 3_u8));
    }
}
