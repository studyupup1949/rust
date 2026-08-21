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

fn main() {
    let mut my_struct = MyStruct::default();
    
    println!();
    println!("Print default MyStruct: a:{}, b:{:?}", my_struct.a(), my_struct.b());
    
    println!();
    println!("Set a to 42");
    my_struct.set_a(42);
    println!("Set b to hello_world");
    my_struct.set_b(String::from("hello_world"));
    println!("Print MyStruct after setting values: a:{}, b:{:?}", my_struct.a(), my_struct.b());

    println!();
    println!("mutate a with get_mut to 100");
    *my_struct.a_mut() =  100;
    println!("mutate b with get_mut to allo");
    *my_struct.b_mut() = String::from("allo");
    println!("Print MyStruct after mutate values: a:{}, b:{:?}", my_struct.a(), my_struct.b());
}
