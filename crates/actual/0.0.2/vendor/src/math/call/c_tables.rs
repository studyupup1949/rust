use crate::{
    // math::tables::Tables::generate,
    utils::{
        func::ident::{Identifier, Status},
        // get_input,
    },
};
use bigdecimal::BigDecimal as BD;

use std::str::FromStr;
pub fn t1() -> Identifier {
    println!("\n\n\n\n\n\n ###### Start of math::c_tables:tables.rs's methods() ######");

    // let m = crate::math::tables::Tables::new();
    // m.auto_generate();
    // m.auto_generate();
    let math = crate::math::new();
    let table = math.table();
    let mut m = table.clone();
    // let x = 1 / 0;
    // println!("{x}");
    m.init(
        BD::from(20),
        BD::from_str("1.12").unwrap(),
        BD::from(2),
        BD::from(0),
    )
    .generate();
    m.print();
    m.reset();
    m.generate();
    m.auto_generate();
    m.print();
    m.reset();
    m.print();

    // println!("table 1");
    // m.auto_generate();
    // println!("table 1");
    // m.reset();
    // println!("table 1");
    // m.auto_generate()
    //     .print();
    // println!("table 1");
    // m.reset().print();
    // println!("table 1");
    // m.initialize(BD::from(20), BD::from(1), BD::from(10), BD::from(0))
    //     .print();
    // println!("table 1");
    // m.auto_generate()
    //     .print();
    // println!("table 1");
    // m.generate();

    // println!("table 1");
    // m.auto_initialize();
    // m.generate();
    println!("table 1");
    let mut id1 = Identifier {
        name: "t1".to_string(),
        id: "t.1".to_string(),
        pid: None,
        location: "math::tables::c_tables".to_string(),
        description: Some("Basically tables reutrn a struct which we call to to test".to_string()),
        return_type: None,
        return_value: None,
        args_type: None,
        number_of_args: None,
        args: None,
        source: Some("math::tables::c_tables::t1(){}".to_string()),
        source_call: Some("call.t.1".to_string()),
        cid: Some("c_t.1".to_string()),
        called_by: Some(vec!["main.rs::main()".to_string()]),
        status: Status {
            status_title: crate::utils::func::ident::Status_T::Working(Some(
                "for_c_tables".to_string(),
            )),
            status_code: 523,
        },
        validate: false,
        func_pointer: Some(t1 as fn() -> Identifier),
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
    println!("Calling from math::tables::c_tables::t1");
    println!("###### End of math::c_tables:tables.rs's methods() ######\n\n\n\n\n");
    println!("{:?}", id1.clone());
    id1.s_status(Some("Free to be used".to_string()), Some(200));
    id1.print_s();

    id1
}
