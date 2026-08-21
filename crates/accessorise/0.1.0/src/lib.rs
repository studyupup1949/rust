#![doc = include_str!("../README.md")]

mod macros;

#[cfg(test)]
mod tests
{

    use super::*;

    use paste::paste;

    trait TestTrait
    {

        trait_get!(a_number, i32);

        trait_set!(a_number, i32);

        trait_get_ref!(a_string, String);

        trait_get_mut!(a_string, String);

        trait_get!(a_number_doc, i8, "This is a getter declaration, presumably for a number field.");

        trait_set!(a_number_doc, i8, "This is a setter declaration, presumably for a number field.");

        trait_get_ref!(a_string_doc, String, "This is a getter declaration, presumably for a String field.");

        trait_get_mut!(a_string_doc, String, "This is a setter declaration, presumably for a String field.");

    }

    #[derive(Default)]
    struct TestStruct
    {

        a_number: i32,
        a_string: String,
        a_number_doc: i8,
        a_string_doc: String,
        some_numbers: Vec<i32>

    }

    impl TestStruct
    {

        pub fn new() -> Self
        {

            Self::default()

        }

        impl_get!(a_number, i32);

        impl_set!(a_number, i32);

        impl_get_clone!(a_string, String);

        impl_get_clone!(a_string_doc, String, "Returns a cloned String.");

        impl_get_ref!(some_numbers, Vec<i32>, "Returns some numbers by reference.");

        impl_get_mut!(some_numbers, Vec<i32>, "Returns some numbers by mutable reference.");

        impl_get_clone!(some_numbers, Vec<i32>);

        impl_set_clone!(some_numbers, Vec<i32>);

    }

    impl TestTrait for TestStruct
    {

        impl_trait_get!(a_number, i32);

        impl_trait_set!(a_number, i32);

        impl_trait_get_ref!(a_string, String);

        impl_trait_get_mut!(a_string, String);

        impl_trait_get!(a_number_doc, i8, "This is a getter implementation for a number field.");

        impl_trait_set!(a_number_doc, i8, "This is a setter implementation for a number field.");

        impl_trait_get_ref!(a_string_doc, String, "This is a getter implementation for a String field.");

        impl_trait_get_mut!(a_string_doc, String, "This is a setter implementation for a String field.");
        
    }

    #[test]
    fn it_works()
    {

        //use super::tests::TestStruct;
        
        let mut test_struct = TestStruct::new();

        let _number = test_struct.a_number();

        test_struct.set_a_number(5);

        //test_struct.

    }

}

