use my_mod::MyStruct;

mod my_mod {
    use accessors_rs::Accessors;

    #[derive(Default, Accessors)]
    #[accessors(get, get_mut, set)]
    pub struct MyStruct {
        #[accessors(get_copy)]
        a: u8,
        b: String,
    }
}

#[test]
fn test_example() {
    let mut my_struct = MyStruct::default();
    
    assert_eq!(0, my_struct.a());
    assert_eq!("", my_struct.b());
    
    my_struct.set_a(42);
    my_struct.set_b(String::from("hello_world"));
    assert_eq!(42, my_struct.a());
    assert_eq!("hello_world", my_struct.b());
    
    *my_struct.a_mut() =  100;
    *my_struct.b_mut() = String::from("allo");
    assert_eq!(100, my_struct.a());
    assert_eq!("allo", my_struct.b());
}
