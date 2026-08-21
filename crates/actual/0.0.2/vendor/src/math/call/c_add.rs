use crate::utils::{
    self,
    func::ident::{Argument, Identifier, /*PID_TABLE,*/ Status /*Status_T*/},
    r#use::read_input,
};
pub fn a1() -> Identifier {
    // cutility::
    println!("\n\n\n\n\n\n ###### Start of math::c_add::add_two() ######");
    let a: i32 = read_input("Enter First Number ");
    let b: i32 = read_input("Enter Second Number ");
    let result = crate::math::add::add_two(a, b);
    // ##################################
    let mut id1 = Identifier {
        name: "a1".to_string(),
        id: "madd.1".to_string(),
        pid: None,
        location: "math::add::c_add".to_string(),
        description: Some("Adds two integers and returns the result.".to_string()),
        return_type: Some(vec!["i32".to_string()]),
        return_value: Some(vec![result.to_string()]),
        args_type: Some(vec!["i32".to_string(), "i32".to_string()]),
        number_of_args: Some(2),
        args: Some(vec![
            Argument {
                name: "a".to_string(),
                ty: "i32".to_string(),
                val: a.to_string(),
            },
            Argument {
                name: "b".to_string(),
                ty: "i32".to_string(),
                val: b.to_string(),
            },
        ]),
        source: Some("math::c_add::a(){}".to_string()),
        source_call: Some("call.madd.1".to_string()),
        cid: Some("calle.1".to_string()),
        called_by: Some(vec!["main.rs::main()".to_string()]),
        status: Status {
            status_title: utils::func::ident::Status_T::Working(Some("for_c_math".to_string())),
            status_code: 523,
        },
        validate: false,
        func_pointer: Some(a1 as fn() -> Identifier),
    };
    id1.print_s();
    id1.generate_pid()
        .validate();
    id1.print_s();
    // let pid;
    // let w;
    // let pid;
    // match id1.pid {
    //     Some(x) => {
    //         pid = x;
    //         // w = pid;
    //     }
    //     _ => panic!("NO PID AHAH "),
    // };
    let pid = id1
        .pid
        .expect("NO PID AHAH");
    //Money Talks
    // just realise  i can just do;
    // let w = id1.pid.expect("NO PID AHAH");
    //let w = id1.pid.unwrap_or_else(|| panic!("NO PID AHAH"));
    // or, for a default:
    // let w = id1.pid.unwrap_or(0);
    println!("id1 is {}", id1.validate.clone());
    println!("Result: {}", result);
    println!("PID = {}, {:?}", std::process::id(), pid);
    println!("Calling from c_add::a1 to add::add_two");
    println!("###### End of math::c_add::add_two() ######\n\n\n\n\n");
    println!("{:?}", id1.clone());
    // id1.status
    //     .status_title = utility::func::ident::Status_T::Good(Some("Free to be used".to_owned()));
    // id1.status
    //     .status_code = 200;
    // id1.s_free(Some("Free to be used".to_string()));
    id1.s_status(Some("Free to be used".to_string()), Some(200));
    id1.print_s();

    id1
}

// println!("")
// let w = PID_TABLE.get(0); // works
// println!("Walter {:?}", w);
// id1.clone()
// .add()
// .unwrap_or_else(|e| eprintln!("Could not add; \n{e}"));
// .expect("FAILED_TO_ADD PID {}");
// pub fn a2() -> (i64)
/* pub fn a2() -> (i32, Identifier) {
    println!("\n\n\n\n\n\n ###### Start of math::c_add::add_two() ######");
}
 */
