// use crossterm::terminal::Clear;

use crate::utils::func::ident::{Identifier, Status};

pub fn section_1_8() -> Identifier {
    println!("\n\n\n\n\n\n ###### Start of udemy::section_1_8() ######");
    struct Person {
        name: String,
        age: u32,
    }
    impl Person {
        pub fn new(name: String, age: u32) -> Self {
            Person { name, age }
        }
        pub fn greet(&self) -> String {
            format!("Hi my name is {}, and my age is {}", self.name, self.age)
        }
        pub fn age_up(&mut self, a: u32) {
            self.age += a;
        }
        pub fn new_make(mut self) -> Self {
            // note taht self is capilat cauz its returning typ
            // self
            self.age_up(20);
            self
        }
        pub fn drop(self) {}
    }
    let mut p = Person::new("Walter".to_owned(), 20);
    println!("P's values = {}", p.greet());
    p.age_up(20);
    println!("p's age increased");
    println!("P's values = {}", p.greet());
    let mut m = p.new_make();
    println!("p was consumed and converted to m");
    println!("m's values = {}", m.greet());
    let a = get_age(&m);
    println!("{a}");
    m.age_up(20);
    //     let a = get_age(&m); u have todo smt like that
    //     println!("{a}"); cant do this while a mut borrrw occurs
    //     m.drop(); cant do this cauz we are using a longer then it is lived
    // we have todo smt like:
    m.drop();
    // p.age_up() -> error cauz p is consumed
    //     p.drop();
    //     println!("P's values = {}", p.greet()); error cauz we droped 'p'
    pub fn get_age(s: &Person) -> &u32 {
        &s.age
    }
    use crate::utils::func::ident::{Identifier, Status};

    let mut id1 = Identifier {
        name: "section_1_8".to_string(),
        id: "ud.1.8".to_string(),
        pid: None,
        location: "udemy::section_1_8".to_string(),
        description: Some("So basically no c_func for htis ".to_string()),
        return_type: None,
        return_value: None,
        args_type: None,
        number_of_args: None,
        args: None,
        source: Some("udemy::section_1_8::section_1_8(){}".to_string()),
        source_call: Some("call.ud.1.8".to_string()),
        cid: None,
        called_by: Some(vec!["main.rs::main()".to_string()]),
        status: Status {
            status_title: crate::utils::func::ident::Status_T::Working(Some(
                "udemy.rs".to_string(),
            )),
            status_code: 523,
        },
        validate: false,
        func_pointer: Some(section_1_8 as fn() -> Identifier),
    };
    id1.print_s();
    id1.generate_pid()
        .validate();
    id1.print_s();
    let pid = id1
        .pid
        .expect("NO PID AHAH");
    println!("id1 is {}", id1.validate.clone());
    println!("PID = {}, {:?}", std::process::id(), pid);
    println!("Calling from udemy::section_1_8::section_1_8");
    println!("###### End of udemy::section_1_8() ######\n\n\n\n\n");
    println!("{:?}", id1.clone());
    id1.s_status(Some("Free to be used".to_string()), Some(200));
    id1.print_s();

    id1
}
//
//
//
//##########################################################################
//

pub fn section_1_9() -> Identifier {
    println!("\n\n\n\n\n\n ###### Start of udemy::section_1_9() ######");
    let mut id1 = Identifier {
        name: "section_1_9".to_string(),
        id: "ud1.9".to_string(),
        pid: None,
        location: "udemy::section_1_9()".to_string(),
        description: Some("Section 1.9 Exlains the box shi".to_string()),
        return_type: None,
        return_value: None,
        args_type: None,
        number_of_args: None,
        args: None,
        source: Some("udemy::section_1_9()::section_1_9(){}".to_string()),
        source_call: Some("call.ud1.9".to_string()),
        cid: None,
        called_by: Some(vec!["main.rs::main()".to_string()]),
        status: Status {
            status_title: crate::utils::func::ident::Status_T::Working(Some(
                "udemy.rs".to_string(),
            )),
            status_code: 523,
        },
        validate: false,
        func_pointer: Some(section_1_9 as fn() -> Identifier),
    };
    #[derive(Debug)]
    pub struct LinkedList<T> {
        data: T,
        next: Option<Box<LinkedList<T>>>,
    }
    impl<T: std::ops::AddAssign> LinkedList<T> {
        pub fn add_up(&mut self, n: T) {
            self.data += n;
        }
        pub fn len(&self) -> usize {
            let mut count = 1; // count this node

            let mut current = &self.next;

            while let Some(node) = current {
                count += 1;
                current = &node.next;
            }

            count
        }
        // fn multiply
        // create a mehtod to multiply , add, ... and iether by index (0, 1,2...) or  a custom number.
        // prob create a struct if increment is a None, type Some or a type increment that u create
        // in math objct.
        // it is a bascially eitehr do *2 each index, or +1 + 2 or +0 + 30 + 60..
        // a object hat does that or normal lineraincrment or striaght,
        // So option<...>
        // pretty cool idea ig.
        /*
        let ll_len = ll.len();

        let mut current = &mut ll;

        for i in 0..ll_len {
            current.add_up(i);

            if let Some(next) = current.next.as_mut() {
                current = next;
            }
        }
        // Kinda like this
             */
    }
    let mut ll = LinkedList {
        data: 3,
        next: Some(Box::new(LinkedList {
            data: 20,
            next: Some(Box::new(LinkedList {
                data: 30,
                next: None,
            })),
        })),
    };
    let ll_len = ll.len();
    println!("{:#?}", ll); // ALso this means pretty print u googed smt but smt else came up and it unrelatledly
    // showed that 'this is how u pretty print' good uselessi nfo.
    println!("ll's len = {}", ll_len);
    //     let ll_len = ll.len();
    //     for i in 0..=ll_len {
    //         println!("i ")
    //     }
    //     if let Some(ref mut v) = ll.next.next {
    // doing ll.next.as_mut and, ll.next ... is equa all three mthos are smae
    if let Some(v) = &mut ll.next {
        v.add_up(20);
    }

    {
        let s = " Hello  ".to_string();
        let v = s.trim();

        println!(" s= {s}");
        println!(" v= {v}");
        let mut s: &str = "water";
        println!("s1 = {s}");
        s = "20";
        println!("s1 = {s}");
        println!("s1's len = {},", s.len());
        // better then len for strings and &str is:
        /*
        let len = "foo".len();
        assert_eq!(3, len);

        assert_eq!("ƒoo".len(), 4); // fancy f!
        assert_eq!("ƒoo".chars().count(), 3);
         */

        // mut s means s is mutable not the data it is pointing to
        // mut array and mut array & means same.
        /*
                let mut arr: &[i32] = &[1, 2, 3];
                arr = &[4, 5];
                arr[0] = 10; Is incorrect
                let mut arr = [1, 2, 3];
                arr[0] =  20,
                but u CANT push more elements
                        let mut data = [1, 2, 3];

        let slice: &mut [i32] = &mut data;

        slice[0] = 10;

        println!("{:?}", data); // [10, 2, 3]
                */
        let mut arr = ["water", "fire", "earth"];
        println!("arr = {arr:?}");
        arr[0] = "air";
        println!("arr = {arr:#?}");
        // String array work in the same way but string array have an additional feature.
        // u can push shii into it., so modify the indexis them selves
        let mut arr = [String::from("water"), String::from("fire")];
        arr[0].push('!');
        // arr[1] = String::from("earth"); same feature as &str1
    }
    {
        let mut wal: &str = "wal";
        println!("Wal = {wal}");
        {
            wal = "20";
            println!("Wal = {wal}");
        }
        println!("Wal = {wal}");
    }
    let mut v: Vec<String> = Vec::with_capacity(100);
    println!(
        "v.len = {}, v.capacity = {}, v = {:#?}",
        v.len(),
        v.capacity(),
        v
    );
    v.push("hello".to_string());
    v.push("goodbye".to_string());
    println!(
        "v.len = {}, v.capacity = {}, v = {:#?}",
        v.len(),
        v.capacity(),
        v
    );
    for i in 0..105 {
        println!("added {} to v,", i);
        v.push(i.to_string());
    }
    println!(
        "v.len = {}, v.capacity = {}, v = {:#?}",
        v.len(),
        v.capacity(),
        v
    );
    println!(
        "v.len = {}, v.capacity = {}, v = {:#?}",
        v.len(),
        v.capacity(),
        v
    );
    for i in 0..105 {
        v.push(i.to_string());
        println!("added {} to v,", i);
    }
    println!(
        "v.len = {}, v.capacity = {}, v = {:#?}",
        v.len(),
        v.capacity(),
        v
    );
    // .to_owned convert refecne to onwed,
    // .to_string does wht it says
    id1.print_s();

    id1.generate_pid()
        .validate();
    id1.print_s();
    let pid = id1
        .pid
        .expect("NO PID AHAH");
    println!("id1 is {}", id1.validate.clone());
    println!("PID = {}, {:?}", std::process::id(), pid);
    println!("Calling from udemy::section_1_9()::section_1_9");
    println!("###### End of udemy::section_1_9() ######\n\n\n\n\n");
    println!("{:?}", id1.clone());
    id1.s_status(Some("Free to be used".to_string()), Some(200));
    id1.print_s();

    id1
}
pub fn section_1_10() -> Identifier {
    println!("\n\n\n\n\n\n ###### Start of udemy::section_1_10() ######");

    let mut h = "       hello        ".to_string();

    println!("Hello = {h}");

    // let p = h.trim()
    // let p = h.clone().trim();
    let p = h.trim().to_owned();
    h.push_str("Nice");
    println!("p = {p}, \n h = {h}");
    // pub fn string_shi(a: str) {}
    // pub fn string_shi(a: Box<str>) {}// but taht is a String after all.
    pub fn string_find_f(a: &str) -> &str {
        // let n = 0;
        // for x in s.char().enumerate(){}
        // for (n, x) in a
        //     .chars()
        //     .enumerate()
        // {}// but diff size so we just do the other unc

        for (i, c) in a.char_indices() {
            if c == 'f' {
                return &a[i..];
            }
        }
        "b"
    }
    let fstr = "help me find house";
    println!("ff_str = {fstr}");
    let ffstr = string_find_f(fstr);
    println!("ff_str = {ffstr}");

    //

    pub fn choose_str(n: u32) -> &'static str {
        match n {
            0 => "hello",
            1 => "bye",
            _ => "ligma",
        }
    }

    // we do 'static here cauz the 'hello', 'bye' and 'ligma' are EMBEDED inside hte binary, they NEVER finish, they are ALWAYS there.
    // ill try a different approach
    // pub fn choose_str<'b>(n: u32, b: &'b str) -> &'b str {
    //     match n {
    //         0 => "hello",
    //         _ => b,
    //     }
    // }
    // pub fn choose_str(n: u32, b: &str) -> &str {
    //     match n {
    //         0 => "hello",
    //         _ => b,
    //     }
    // }
    // let walter = choose_str(0, "20");
    let walter = choose_str(0);
    println!("{walter}");
    let mut id1 = Identifier {
        name: "section_1_10".to_string(),
        id: "udemy.1.10".to_string(),
        pid: None,
        location: "udemy.rs".to_string(),
        description: Some("Section 1.10 of udemy DSA course.".to_string()),
        return_type: None,
        return_value: None,
        args_type: None,
        number_of_args: None,
        args: None,
        source: Some("udemy.rs::section_1_10(){}".to_string()),
        source_call: Some("call.udemy.1.10".to_string()),
        cid: None,
        called_by: Some(vec!["main.rs::main()".to_string()]),
        status: Status {
            status_title: crate::utils::func::ident::Status_T::Working(Some(
                "for_c_math".to_string(),
            )),
            status_code: 523,
        },
        validate: false,
        func_pointer: Some(section_1_10 as fn() -> Identifier),
    };
    id1.print_s();
    id1.generate_pid()
        .validate();
    id1.print_s();
    let pid = id1
        .pid
        .expect("NO PID AHAH");
    println!("id1 is {}", id1.validate.clone());
    println!("PID = {}, {:?}", std::process::id(), pid);
    println!("Calling from udemy.rs::section_1_10");
    println!("###### End of udemy::section_1_10() ######\n\n\n\n\n");
    println!("{:?}", id1.clone());
    id1.s_status(Some("Free to be used".to_string()), Some(200));
    id1.print_s();

    id1
}
/*
let r = 100..200;

println!("{}", r.start);
println!("{}", r.end); */

pub mod dsa {

    use std::fmt::Debug;

    pub fn bubble_sort<T: PartialOrd + Debug>(v: &mut [T]) {
        for i in 0..v.len() {
            println!("Arr = {v:#?}");
            println!("Count = {i}");
            let mut sorted = true;

            for l in 0..v.len() - 1 {
                println!("Arr = {v:#?}");
                println!("InnerCount = {l}");

                if v[l] > v[l + 1] {
                    v.swap(l, l + 1);
                    sorted = false
                }
            }
            if sorted {
                break;
            }
        }
    }
    //### TEST
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_bubble_sort() {
            let mut v = vec![5, 2, 2, 4, 1, 3, 3];
            // println!("left = {v:#2?}");
            bubble_sort(&mut v);
            assert_eq!(v, vec![1, 2, 2, 3, 3, 4, 5]);
            // println!("left = {v:#?}");
        }
    }
}
