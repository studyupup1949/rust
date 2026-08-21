// use crate::utils::r#use::get_input;
use crate::utils::{
    self,
    func::ident::{
        //Argument,
        Identifier,
        /*PID_TABLE,*/ Status, /*Status_T*/
    },
    // r#use::get_input,
};
pub fn test() -> Identifier {
    let mut id1 = Identifier {
        name: "m1".to_string(),
        id: "testm1".to_string(),
        pid: None,
        location: "test::m::test()".to_string(),
        description: Some("Constructs a struct htat ahs aadd and print methods, to print u32 orincremntt it by 1 .".to_string()),
        return_type: Some(vec!["i32".to_string()]),
        return_value: None,
        args_type: None,
        number_of_args: None,
        args: None,
        source: Some("test::m::test(){}".to_string()),
        source_call: Some("mtest.m".to_string()),
        cid: Some("testing1".to_string()),
        called_by: Some(vec!["main.rs::test()".to_string()]),
        status: Status {
            status_title: utils::func::ident::Status_T::Working(Some(
                "printing and modifying struct 'simga' : test::m::Walter. for: test::m.rs"
                    .to_string(),
            )),
            status_code: 1000,
        },
        validate: false,
        func_pointer: Some(test as fn() -> Identifier),
    };
    id1.print_s();
    id1.generate_pid()
        .validate();
    id1.print_s();
    // println!("")
    println!("Pid  = {:?}", id1.pid);
    id1.s_status(
        Some("cant be called anymore. Nothing more todo. only one call per runtime".to_string()),
        Some(1201),
    );
    // id1.lock();
    // Create a lock and unlock state
    id1
}
