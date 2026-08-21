#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct EnumVariant<T> {
    pub name: &'static str,
    pub value: Option<T>,
}

impl<T> EnumVariant<T> {
    pub const fn new(name: &'static str, value: Option<T>) -> Self {
        Self { name, value }
    }
}

pub trait ReflectEnum: Sized {
    type Type;
    fn variants() -> &'static [EnumVariant<Self>];
    fn count() -> usize;
    fn name(&self) -> &'static str;
}

#[cfg(test)]
mod test {
    use crate::{self as adar, prelude::*};
    use std::any::TypeId;

    #[ReflectEnum]
    #[derive(Debug, Eq, PartialEq)]
    enum MixedTestEnum {
        Elem1,
        Elem2(u32),
        Elem3 { a: u32, b: u32 },
    }

    #[test]
    fn test_empty_enum() {
        #[ReflectEnum]
        #[derive(Debug, Eq, PartialEq)]
        enum TestEnum {}

        let elements = TestEnum::variants();
        assert_eq!(elements.iter().next(), None);
        assert_eq!(TestEnum::count(), 0);
    }

    #[test]
    fn test_enum_iter() {
        let elements = MixedTestEnum::variants();
        let mut i = elements.iter();
        assert_eq!(
            i.next(),
            Some(&EnumVariant::new("Elem1", Some(MixedTestEnum::Elem1))),
        );
        assert_eq!(i.next(), Some(&EnumVariant::new("Elem2", None)));
        assert_eq!(i.next(), Some(&EnumVariant::new("Elem3", None)));
        assert_eq!(i.next(), None);
        assert_eq!(MixedTestEnum::count(), 3);
    }

    #[test]
    fn test_enum_name() {
        assert_eq!(MixedTestEnum::Elem1.name(), "Elem1");
        assert_eq!(MixedTestEnum::Elem2(0).name(), "Elem2");
        assert_eq!(MixedTestEnum::Elem3 { a: 0, b: 0 }.name(), "Elem3");
    }

    #[test]
    fn test_enum_repr() {
        #[ReflectEnum]
        #[derive(Debug, Eq, PartialEq)]
        enum TestEnum {
            E1,
        }

        assert_eq!(
            TypeId::of::<<TestEnum as ReflectEnum>::Type>(),
            TypeId::of::<u32>()
        );

        #[derive(Debug, Eq, PartialEq)]
        #[repr(u8)]
        #[ReflectEnum]
        enum TestEnum2 {
            E1,
        }

        assert_eq!(
            TypeId::of::<<TestEnum2 as ReflectEnum>::Type>(),
            TypeId::of::<u8>()
        );
    }
}
