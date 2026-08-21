use crate::utils::func::ident::{Identifier, Status};
use rug::{Float, Rational};

// Source - https://stackoverflow.com/a/79989234
// Posted by Matthieu M.
// Retrieved 2026-07-30, License - CC BY-SA 4.0

use std::num::NonZeroUsize;

pub fn c_ool_patterns() -> Identifier {
    println!("\n\n\n\n\n\n ###### Start of math::c_ool_patterns() ######");

    let mut id1 = Identifier {
        name: "c_ool_patterns".to_string(),
        id: "cp".to_string(),
        pid: None,
        location: "math::c_ool_patterns::c_ool_patterns()".to_string(),
        description: Some("Bascially a function to show cool math shi".to_string()),
        return_type: None,
        return_value: None,
        args_type: None,
        number_of_args: None,
        args: None,
        source: Some("math::c_ool_patterns::c_ool_patterns()::c_ool_patterns(){}".to_string()),
        source_call: Some("call.cp".to_string()),
        cid: Some("Nice".to_string()),
        called_by: Some(vec!["main.rs::main()".to_string()]),
        status: Status {
            status_title: crate::utils::func::ident::Status_T::Working(Some(
                "for_self".to_string(),
            )),
            status_code: 200,
        },
        validate: false,
        func_pointer: Some(c_ool_patterns as fn() -> Identifier),
    };
    let prec = 700; // ~700 bits ≈ 210 decimal digits

    let values = [
        ("1/99980001 ", Rational::from((1, 99980001))),
        ("1/998001", Rational::from((1, 998001))),
        ("1/9801", Rational::from((1, 9801))),
        ("1/81", Rational::from((1, 99))),
        ("1/9", Rational::from((1, 9))),
    ];

    for (name, r) in values {
        let f = Float::with_val(prec, &r);
        println!("{name} = {:.200}", f);
    }
    id1.print_s();
    id1.generate_pid()
        .validate();
    id1.print_s();
    let pid = id1
        .pid
        .expect("NO PID AHAH");
    println!("id1 is {}", id1.validate.clone());
    println!("PID = {}, {:?}", std::process::id(), pid);
    println!("Calling from math::c_ool_patterns::c_ool_patterns()::c_ool_patterns");
    println!("###### End of math::c_ool_patterns() ######\n\n\n\n\n");
    println!("{:?}", id1.clone());
    id1.s_status(Some("Free to be used".to_string()), Some(200));
    id1.print_s();

    id1
}
