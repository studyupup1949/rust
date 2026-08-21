macro_rules! impl_pointer {
    (
        $trait:ident::$method:ident $(<$($gen:tt),*>)? (
            &$($lt:lifetime)? self $(, $arg:ident : $arg_ty:ty)*
        ) $(-> $ret:ty)? $(where $($where:tt)*)?

    ) => {
        impl<T: $trait<Value> + ?Sized, Value> $trait<Value> for &T {
            fn $method $(<$($gen),*>)? (&$($lt)? self $(, $arg: $arg_ty)* ) $(-> $ret)? $(where $($where)*)?{
                (**self).$method($($arg),*)
            }
        }

        impl<T: $trait<Value> + ?Sized, Value> $trait<Value> for &mut T {
            fn $method $(<$($gen),*>)? (&$($lt)? self $(, $arg: $arg_ty)* ) $(-> $ret)? $(where $($where)*)?{
                (**self).$method($($arg),*)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $trait<Value> + ?Sized, Value> $trait<Value> for crate::__internal::alloc::boxed::Box<T> {
            fn $method $(<$($gen),*>)? (&$($lt)? self $(, $arg: $arg_ty)* ) $(-> $ret)? $(where $($where)*)?{
                (**self).$method($($arg),*)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $trait<Value> + ?Sized, Value> $trait<Value> for crate::__internal::alloc::rc::Rc<T> {
            fn $method $(<$($gen),*>)? (&$($lt)? self $(, $arg: $arg_ty)* ) $(-> $ret)? $(where $($where)*)?{
                (**self).$method($($arg),*)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $trait<Value> + ?Sized, Value> $trait<Value> for crate::__internal::alloc::sync::Arc<T> {
            fn $method $(<$($gen),*>)? (&$($lt)? self $(, $arg: $arg_ty)* ) $(-> $ret)? $(where $($where)*)?{
                (**self).$method($($arg),*)
            }
        }
    };
}
pub(super) use impl_pointer;
