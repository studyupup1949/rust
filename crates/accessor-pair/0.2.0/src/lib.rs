/// Note that `Field: 'static'` is only used for `compose` atm, so may have to be removed
pub trait AccessorPair<Struct, Field: 'static> {
    fn get<'a>(&self, on: &'a Struct) -> &'a Field ;

    fn set<'a>(&self, on: &'a mut Struct) -> &'a mut Field ;

    fn compose<'a, Next: 'static, Other: AccessorPair<Field, Next> + Clone + 'a>(&'a self, other: Other) -> (impl Fn(&Struct) -> &Next, impl Fn(&mut Struct) -> &mut Next) {
        let other_for_get = other.clone();
        (
            move |s: &Struct| other_for_get.get(self.get(s)),
            move |s: &mut Struct| other.set(self.set(s)),
        )
    }
}

impl<Struct, Field: 'static, Getter, Setter> AccessorPair<Struct, Field> for (Getter, Setter)
where
    Getter: for<'a> Fn(&'a Struct) -> &'a Field,
    Setter: for<'a> Fn(&'a mut Struct) -> &'a mut Field,
{

    fn get<'a>(&self, on: &'a Struct) -> &'a Field {
        self.0(on)
    }

    fn set<'a>(&self, on: &'a mut Struct) -> &'a mut Field {
        self.1(on)
    }
}

macro_rules! field_accessors {
    (| $_1:ident : $struct:ty | $_2:ident . $field:ident) => {{
        let accessor_pair: (fn(&$struct) -> &_, fn(&mut $struct) -> &mut _) =
            (|s: &$struct| &s.$field, |s: &mut $struct| &mut s.$field);
        accessor_pair
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PottedFern {
        fern: Fern,
        pot_name: String,
    }

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

        let species_accessor: _ = field_accessors!(|fern:Fern| fern.species);

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

    #[test]
    fn test_compose() {
        let mut potted_fern = PottedFern{
            fern: Fern { species: "Horsetails".to_string(), healthy: false },
            pot_name: "Medium terracotta pot".to_string(),
        };

        let fern_accessor: _ = field_accessors!(|potted_fern: PottedFern| potted_fern.fern);
        let species_accessor: _ = field_accessors!(|fern: Fern| fern.species);
        let composed = fern_accessor.compose(species_accessor);

        assert_eq!(composed.get(&potted_fern), "Horsetails");
        *composed.set(&mut potted_fern) = "Equisetum".to_string();
        assert_ne!(composed.get(&potted_fern), "Horsetails");

    }
}
