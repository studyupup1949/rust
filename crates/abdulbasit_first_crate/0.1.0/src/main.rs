mod abc {
    pub mod ui{
        pub fn id1(){
            println!("ID1");
        }
    }
}


mod bcd {
    pub fn aaa(){
        println!("Anum");
    }
}

fn main() {
    println!("Hello, world!");
    bcd::aaa();
    abc::ui::id1();    
}
