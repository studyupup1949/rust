pub trait AccessorPair<Struct, Field> {
    type Getter: Fn(&Struct) -> &Field;
    type Setter: Fn(&mut Struct) -> &mut Field;

    fn get<'a>(&self, on: &'a Struct) -> &'a Field ;

    fn set<'a>(&self, on: &'a mut Struct) -> &'a mut Field ;

}

impl<Struct, Field, Getter, Setter> AccessorPair<Struct, Field> for (Getter, Setter)
where
    Getter: for<'a> Fn(&'a Struct) -> &'a Field,
    Setter: for<'a> Fn(&'a mut Struct) -> &'a mut Field,
{
    type Getter = Getter;
    type Setter = Setter;

    fn get<'a>(&self, on: &'a Struct) -> &'a Field {
        self.0(on)
    }

    fn set<'a>(&self, on: &'a mut Struct) -> &'a mut Field {
        self.1(on)
    }
}


macro_rules! field_accessors {
    ($struct:ident, $field:ident) => {
        {
            let accessor_pair:(fn(&$struct) -> &_, fn(&mut $struct) -> &mut _) = (|s: &$struct| &s.$field, |s: &mut $struct| &mut s.$field);
            accessor_pair
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fern {
        species: String,
        healthy: bool,
    }

    #[test]
    fn test() {
        let mut fern = Fern {
            species: "Horsetails".to_string(),
            healthy: true,
        };
        let species_accessor: _ = field_accessors!(Fern, species);

        fn test_getter<T:AccessorPair<Fern, String>>(
            accessor: T,
            fern: &Fern
        ) {
            assert_eq!(accessor.get(&fern), "Horsetails");
        }
        test_getter(species_accessor, &fern);

        fn test_setter<T: AccessorPair<Fern, String>>(
            accessor: T,
            fern: &mut Fern,
        ) {
            *accessor.set(fern) = String::from("Equisetum");
        }
        test_setter(species_accessor, &mut fern);

        assert_ne!(fern.species, "Horsetails");
    }
}
