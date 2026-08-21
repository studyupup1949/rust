use syn::punctuated::Punctuated;

pub trait PunctuatedConstructExt<T, P>
where
    P: Default,
{
    fn from_value(value: T) -> Punctuated<T, P>;
}

impl<T, P> PunctuatedConstructExt<T, P> for Punctuated<T, P>
where
    P: Default,
{
    fn from_value(value: T) -> Punctuated<T, P> {
        let mut punctuated = Punctuated::new();
        punctuated.push(value);
        punctuated
    }
}
