use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "full")] {
        use alloc::vec::Vec;

        mod vector;
        mod mut_vector;

        pub struct Vector<T> {
            values: Vec<T>,
        }

        pub struct MutVector<T> {
            values: Vec<T>,
        }
    }
}

cfg_if! {
    if #[cfg(feature = "no_std")]  {
        mod vector_slice;
        mod mut_vector_slice;

        pub struct VectorSlice<'v, T> {
            values: &'v [T]
        }

        pub struct MutVectorSlice<'v, T> {
            values: &'v mut [T]
        }
    }
}

// implement addition

mod no_std_only {
    #![cfg(all(feature = "no_std", not(feature = "full")))]

    use super::*;
    use adv_linalg_proc_macro::{impl_dot_product, impl_vector_add, impl_vector_sub};
    use core::ops::{Add, Mul, Sub};

    impl_vector_add!(
        #[no_std] #[mut_left]
        impl<'lhs, 'rhs, T: Clone + Add<Output = T>>
            Add for
                [MutVectorSlice<'lhs, T>] + [VectorSlice<'rhs, T>];

        #[no_std] #[mut_both]
        impl<'lhs, 'rhs, T: Clone + Add<Output = T>>
            Add for
                [MutVectorSlice<'lhs, T>] + [MutVectorSlice<'rhs, T>]
    );

    impl_dot_product!(
        #[mut_left]
        impl<'lhs, 'rhs, T: Clone + Default + Add<Output = T> + Mul<Output = T>>
            Mul for
                [MutVectorSlice<'lhs, T>] * [VectorSlice<'rhs, T>];

        #[mut_both]
        impl<'lhs, 'rhs, T: Clone + Default + Add<Output = T> + Mul<Output = T>>
            Mul for
                [MutVectorSlice<'lhs, T>] * [MutVectorSlice<'rhs, T>]
    );

    impl_vector_sub!(
        #[no_std] #[mut_left]
        impl<'lhs, 'rhs, T: Clone + Sub<Output = T>>
            Sub for
                [MutVectorSlice<'lhs, T>] - [VectorSlice<'rhs, T>];

        #[no_std] #[mut_both]
        impl<'lhs, 'rhs, T: Clone + Sub<Output = T>>
            Sub for
                [MutVectorSlice<'lhs, T>] - [MutVectorSlice<'rhs, T>]
    );
}

mod full_only {
    #![cfg(feature = "full")]

    use super::*;
    use adv_linalg_proc_macro::{impl_dot_product, impl_vector_add, impl_vector_sub};
    use core::ops::{Add, Mul, Sub};

    impl_vector_add!(
        impl<T: Clone + Add<Output = T>> Add for [Vector<T>] + [Vector<T>];
        impl<'lhs, T: Clone + Add<Output = T>> Add for [VectorSlice<'lhs, T>] + [Vector<T>];
        impl<'rhs, T: Clone + Add<Output = T>> Add for [Vector<T>] + [VectorSlice<'rhs, T>];
        impl<'lhs, 'rhs, T: Clone + Add<Output = T>> Add for [VectorSlice<'lhs, T>] + [VectorSlice<'rhs, T>];

        #[mut_left] impl<T: Clone + Add<Output = T>> Add for [MutVector<T>] + [Vector<T>];
        #[mut_left] impl<'lhs, T: Clone + Add<Output = T>> Add for [MutVectorSlice<'lhs, T>] + [Vector<T>];
        #[mut_left] impl<'rhs, T: Clone + Add<Output = T>> Add for [MutVector<T>] + [VectorSlice<'rhs, T>];
        #[mut_left] impl<'lhs, 'rhs, T: Clone + Add<Output = T>> Add for [MutVectorSlice<'lhs, T>] + [VectorSlice<'rhs, T>];

        #[mut_right] impl<T: Clone + Add<Output = T>> Add for [Vector<T>] + [MutVector<T>];
        #[mut_right] impl<'lhs, T: Clone + Add<Output = T>> Add for [VectorSlice<'lhs, T>] + [MutVector<T>];
        #[mut_right] impl<'rhs, T: Clone + Add<Output = T>> Add for [Vector<T>] + [MutVectorSlice<'rhs, T>];
        #[mut_right] impl<'lhs, 'rhs, T: Clone + Add<Output = T>> Add for [VectorSlice<'lhs, T>] + [MutVectorSlice<'rhs, T>];

        #[mut_both] impl<T: Clone + Add<Output = T>> Add for [MutVector<T>] + [MutVector<T>];
        #[mut_both] impl<'lhs, T: Clone + Add<Output = T>> Add for [MutVectorSlice<'lhs, T>] + [MutVector<T>];
        #[mut_both] impl<'rhs, T: Clone + Add<Output = T>> Add for [MutVector<T>] + [MutVectorSlice<'rhs, T>];
        #[mut_both] impl<'lhs, 'rhs, T: Clone + Add<Output = T>> Add for [MutVectorSlice<'lhs, T>] + [MutVectorSlice<'rhs, T>]
    );

    impl_dot_product!(
        impl<T: Clone + Default + Add<Output = T> + Mul<Output = T>>
            Mul for
                [Vector<T>] * [Vector<T>];
        impl<'lhs, T: Clone + Default + Add<Output = T> + Mul<Output = T>>
            Mul for
                [VectorSlice<'lhs, T>] * [Vector<T>];
        impl<'rhs, T: Clone + Default + Add<Output = T> + Mul<Output = T>>
            Mul for
                [Vector<T>] * [VectorSlice<'rhs, T>];
        impl<'lhs, 'rhs, T: Clone + Default + Add<Output = T> + Mul<Output = T>>
            Mul for
                [VectorSlice<'lhs, T>] * [VectorSlice<'rhs, T>];

        #[mut_left]
        impl<T: Clone + Default + Add<Output = T> + Mul<Output = T>>
            Mul for
                [MutVector<T>] * [Vector<T>];
        #[mut_left]
        impl<'lhs, T: Clone + Default + Add<Output = T> + Mul<Output = T>>
            Mul for
                [MutVectorSlice<'lhs, T>] * [Vector<T>];
        #[mut_left]
        impl<'rhs, T: Clone + Default + Add<Output = T> + Mul<Output = T>>
            Mul for
                [MutVector<T>] * [VectorSlice<'rhs, T>];
        #[mut_left]
        impl<'lhs, 'rhs, T: Clone + Default + Add<Output = T> + Mul<Output = T>>
            Mul for
                [MutVectorSlice<'lhs, T>] * [VectorSlice<'rhs, T>];

        #[mut_right]
        impl<T: Clone + Default + Add<Output = T> + Mul<Output = T>>
            Mul for
                [Vector<T>] * [MutVector<T>];
        #[mut_right]
        impl<'lhs, T: Clone + Default + Add<Output = T> + Mul<Output = T>>
            Mul for
                [VectorSlice<'lhs, T>] * [MutVector<T>];
        #[mut_right]
        impl<'rhs, T: Clone + Default + Add<Output = T> + Mul<Output = T>>
            Mul for
                [Vector<T>] * [MutVectorSlice<'rhs, T>];
        #[mut_right]
        impl<'lhs, 'rhs, T: Clone + Default + Add<Output = T> + Mul<Output = T>>
            Mul for
                [VectorSlice<'lhs, T>] * [MutVectorSlice<'rhs, T>];

        #[mut_both]
        impl<T: Clone + Default + Add<Output = T> + Mul<Output = T>>
            Mul for
                [MutVector<T>] * [MutVector<T>];
        #[mut_both]
        impl<'lhs, T: Clone + Default + Add<Output = T> + Mul<Output = T>>
            Mul for
                [MutVectorSlice<'lhs, T>] * [MutVector<T>];
        #[mut_both]
        impl<'rhs, T: Clone + Default + Add<Output = T> + Mul<Output = T>>
            Mul for [MutVector<T>] * [MutVectorSlice<'rhs, T>];
        #[mut_both]
        impl<'lhs, 'rhs, T: Clone + Default + Add<Output = T> + Mul<Output = T>>
            Mul for
                [MutVectorSlice<'lhs, T>] * [MutVectorSlice<'rhs, T>]
    );

    impl_vector_sub!(
        impl<T: Clone + Sub<Output = T>> Sub for [Vector<T>] - [Vector<T>];
        impl<'lhs, T: Clone + Sub<Output = T>> Sub for [VectorSlice<'lhs, T>] - [Vector<T>];
        impl<'rhs, T: Clone + Sub<Output = T>> Sub for [Vector<T>] - [VectorSlice<'rhs, T>];
        impl<'lhs, 'rhs, T: Clone + Sub<Output = T>> Sub for [VectorSlice<'lhs, T>] - [VectorSlice<'rhs, T>];

        #[mut_left] impl<T: Clone + Sub<Output = T>> Sub for [MutVector<T>] - [Vector<T>];
        #[mut_left] impl<'lhs, T: Clone + Sub<Output = T>> Sub for [MutVectorSlice<'lhs, T>] - [Vector<T>];
        #[mut_left] impl<'rhs, T: Clone + Sub<Output = T>> Sub for [MutVector<T>] - [VectorSlice<'rhs, T>];
        #[mut_left] impl<'lhs, 'rhs, T: Clone + Sub<Output = T>> Sub for [MutVectorSlice<'lhs, T>] - [VectorSlice<'rhs, T>];

        #[mut_right] impl<T: Clone + Sub<Output = T>> Sub for [Vector<T>] - [MutVector<T>];
        #[mut_right] impl<'lhs, T: Clone + Sub<Output = T>> Sub for [VectorSlice<'lhs, T>] - [MutVector<T>];
        #[mut_right] impl<'rhs, T: Clone + Sub<Output = T>> Sub for [Vector<T>] - [MutVectorSlice<'rhs, T>];
        #[mut_right] impl<'lhs, 'rhs, T: Clone + Sub<Output = T>> Sub for [VectorSlice<'lhs, T>] - [MutVectorSlice<'rhs, T>];

        #[mut_both] impl<T: Clone + Sub<Output = T>> Sub for [MutVector<T>] - [MutVector<T>];
        #[mut_both] impl<'lhs, T: Clone + Sub<Output = T>> Sub for [MutVectorSlice<'lhs, T>] - [MutVector<T>];
        #[mut_both] impl<'rhs, T: Clone + Sub<Output = T>> Sub for [MutVector<T>] - [MutVectorSlice<'rhs, T>];
        #[mut_both] impl<'lhs, 'rhs, T: Clone + Sub<Output = T>> Sub for [MutVectorSlice<'lhs, T>] - [MutVectorSlice<'rhs, T>]
    );
}
