pub trait AcceptsInfo: Clone + Copy {
    fn accepts_name(&self) -> &'static str;

    fn accept_fn_name(&self) -> &'static str;

    fn accept_lifetime_name(&self) -> Option<&'static str>;

    fn accept_is_async(&self) -> bool;
}

impl<T: AcceptsInfo + ?Sized> AcceptsInfo for &T {
    fn accepts_name(&self) -> &'static str {
        (**self).accepts_name()
    }

    fn accept_fn_name(&self) -> &'static str {
        (**self).accept_fn_name()
    }

    fn accept_lifetime_name(&self) -> Option<&'static str> {
        (**self).accept_lifetime_name()
    }

    fn accept_is_async(&self) -> bool {
        (**self).accept_is_async()
    }
}
