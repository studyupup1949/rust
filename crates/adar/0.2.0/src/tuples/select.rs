/// Gets a reference to a tuple element defined by its type.
pub trait TupleSelect<T, GFT> {
    /// Gets a reference to a tuple element defined by its type. \
    /// Your code will not compile if the tuple doesn't have exactly 1 element of the chosen type.
    fn select<J>(&self) -> &J
    where
        J: SelectFromTuple<T, GFT>;
}

///  Gets a mutable reference to a tuple element defined by its type.
pub trait TupleSelectMut<T, GFT> {
    /// Gets a mutable reference to a tuple element defined by its type. \
    /// Your code will not compile if the tuple doesn't have exactly 1 element of the chosen type.
    fn select_mut<J>(&mut self) -> &mut J
    where
        J: SelectFromTupleMut<T, GFT>;
}

impl<T, GFT> TupleSelect<Self, GFT> for T {
    #[inline(always)]
    fn select<J>(&self) -> &J
    where
        J: SelectFromTuple<Self, GFT>,
    {
        J::get_from(self)
    }
}

impl<T, GFT> TupleSelectMut<Self, GFT> for T {
    #[inline(always)]
    fn select_mut<J>(&mut self) -> &mut J
    where
        J: SelectFromTupleMut<Self, GFT>,
    {
        J::get_from_mut(self)
    }
}

/// Helper tuple for [`super::TupleSelect`].
pub trait SelectFromTuple<T, GFT> {
    fn get_from(tuple: &T) -> &Self;
}

/// Helper tuple for [`super::TupleSelectMut`].
pub trait SelectFromTupleMut<T, GFT> {
    fn get_from_mut(tuple: &mut T) -> &mut Self;
}

macro_rules! impl_get_from_tuple {
    ($gft_type:ident, $target:ident, $index:tt, $($types:ident),+) => {
        #[doc(hidden)]
        pub struct $gft_type;
        impl<$($types),+> SelectFromTuple<($($types,)+), $gft_type> for $target {
            #[inline(always)]
            fn get_from(tuple: &($($types,)+)) -> &Self {
                &tuple.$index
            }
        }

        impl<$($types),+> SelectFromTupleMut<($($types,)+), $gft_type> for $target {
            #[inline(always)]
            fn get_from_mut(tuple: &mut ($($types,)+)) -> &mut Self {
                &mut tuple.$index
            }
        }
    };
}

impl_get_from_tuple!(GFT0, AX, 0, AX);
impl_get_from_tuple!(GFT1, AX, 0, AX, B);
impl_get_from_tuple!(GFT2, BX, 1, A, BX);

impl_get_from_tuple!(GFT3, AX, 0, AX, B, C);
impl_get_from_tuple!(GFT4, BX, 1, A, BX, C);
impl_get_from_tuple!(GFT5, CX, 2, A, B, CX);

impl_get_from_tuple!(GFT6, AX, 0, AX, B, C, D);
impl_get_from_tuple!(GFT7, BX, 1, A, BX, C, D);
impl_get_from_tuple!(GFT8, CX, 2, A, B, CX, D);
impl_get_from_tuple!(GFT9, DX, 3, A, B, C, DX);

impl_get_from_tuple!(GFT10, AX, 0, AX, B, C, D, E);
impl_get_from_tuple!(GFT11, BX, 1, A, BX, C, D, E);
impl_get_from_tuple!(GFT12, CX, 2, A, B, CX, D, E);
impl_get_from_tuple!(GFT13, DX, 3, A, B, C, DX, E);
impl_get_from_tuple!(GFT14, EX, 4, A, B, C, D, EX);

impl_get_from_tuple!(GFT15, AX, 0, AX, B, C, D, E, F);
impl_get_from_tuple!(GFT16, BX, 1, A, BX, C, D, E, F);
impl_get_from_tuple!(GFT17, CX, 2, A, B, CX, D, E, F);
impl_get_from_tuple!(GFT18, DX, 3, A, B, C, DX, E, F);
impl_get_from_tuple!(GFT19, EX, 4, A, B, C, D, EX, F);
impl_get_from_tuple!(GFT20, FX, 5, A, B, C, D, E, FX);

impl_get_from_tuple!(GFT21, AX, 0, AX, B, C, D, E, F, G);
impl_get_from_tuple!(GFT22, BX, 1, A, BX, C, D, E, F, G);
impl_get_from_tuple!(GFT23, CX, 2, A, B, CX, D, E, F, G);
impl_get_from_tuple!(GFT24, DX, 3, A, B, C, DX, E, F, G);
impl_get_from_tuple!(GFT25, EX, 4, A, B, C, D, EX, F, G);
impl_get_from_tuple!(GFT26, FX, 5, A, B, C, D, E, FX, G);
impl_get_from_tuple!(GFT27, GX, 6, A, B, C, D, E, F, GX);

impl_get_from_tuple!(GFT28, AX, 0, AX, B, C, D, E, F, G, H);
impl_get_from_tuple!(GFT29, BX, 1, A, BX, C, D, E, F, G, H);
impl_get_from_tuple!(GFT30, CX, 2, A, B, CX, D, E, F, G, H);
impl_get_from_tuple!(GFT31, DX, 3, A, B, C, DX, E, F, G, H);
impl_get_from_tuple!(GFT32, EX, 4, A, B, C, D, EX, F, G, H);
impl_get_from_tuple!(GFT33, FX, 5, A, B, C, D, E, FX, G, H);
impl_get_from_tuple!(GFT34, GX, 6, A, B, C, D, E, F, GX, H);
impl_get_from_tuple!(GFT35, HX, 7, A, B, C, D, E, F, G, HX);

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Eq, PartialEq, Debug)]
    struct Tut;

    #[test]
    fn test_tuple_select() {
        assert_eq!((false,).select::<bool>(), &false);
        assert_eq!((22_i32, true).select::<bool>(), &true);
        assert_eq!((22_i32, true).select::<i32>(), &22);
        assert_eq!((22_i32, true, 2.2_f32).select::<f32>(), &2.2);
        assert_eq!((22_i32, Tut, 2.2_f32).select::<Tut>(), &Tut);
    }

    #[test]
    fn test_tuple_select_mut() {
        assert_eq!((false,).select_mut::<bool>(), &false);
        assert_eq!((22_i32, true).select_mut::<bool>(), &true);
        assert_eq!((22_i32, true).select_mut::<i32>(), &22);
        assert_eq!((22_i32, true, 2.2_f32).select_mut::<f32>(), &2.2);
        assert_eq!((22_i32, Tut, 2.2_f32).select_mut::<Tut>(), &Tut);

        let mut tuple = (64_i32, "Hello", 2.2_f32);
        *tuple.select_mut::<i32>() += 10;
        assert_eq!(tuple.0, 74);
        assert_eq!(tuple.0, *tuple.select::<i32>());
    }
}
