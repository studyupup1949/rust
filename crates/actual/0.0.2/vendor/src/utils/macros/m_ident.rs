#![allow(unused)]
// use crate::utils::func::ident::Identifier;
macro_rules! c_wrapper {
    (
        fn_name: $fn_name:ident,
        id: $id:expr,
        location: $location:expr,
        target: $target:path,
        desc: $desc:expr,
        args: [ $( ($arg_name:ident : $arg_ty:ty) ),* ],
        ret_ty: $ret_ty:ty
    ) => {
        pub fn $fn_name() -> crate::utils::func::ident::Identifier {
            println!("\n\n\n\n\n\n ###### Start of {}() ######", stringify!($target));

            // $( let $arg_name: $arg_ty = crate::utils::get_input(concat!("Enter ", stringify!($arg_name), " ")); )*

            // let result = crate::$target( $($arg_name),* );

            let mut id1 = crate::utils::func::ident::Identifier {
                name: stringify!($fn_name).to_string(),
                id: $id.to_string(),
                pid: None,
                location: $location.to_string(),
                description: Some($desc.to_string()),
                return_type: None,
                return_value: None,
                args_type: Some(vec![ $(stringify!($arg_ty).to_string()),* ]),
                number_of_args: Some(0 $(+ { let _ = stringify!($arg_name); 1})*),
                args:None,
                source: Some(format!("{}::{}(){{}}", $location, stringify!($fn_name))),
                source_call: Some(format!("call.{}", $id)),
                cid: Some("calle.1".to_string()), // still needs a real generator eventually
                called_by: Some(vec!["main.rs::main()".to_string()]),
                status: crate::utils::func::ident::Status {
                    status_title: crate::utils::func::ident::Status_T::Working(Some("for_c_math".to_string())),
                    status_code: 523,
                },
                validate: false,
                func_pointer: Some($fn_name as fn() -> crate::utils::func::ident::Identifier),
            };

            id1.print_s();
            id1.generate_pid().validate();
            id1.print_s();
            let pid = id1.pid.expect("NO PID AHAH");
            println!("id1 is {}", id1.validate.clone());
            println!("Result:" );
            println!("PID = {}, {:?}", std::process::id(), pid);
            println!("Calling from {}::{} to {}", $location, stringify!($fn_name), stringify!($target));
            println!("###### End of {}() ######\n\n\n\n\n", stringify!($target));
            println!("{:?}", id1.clone());
            id1.s_status(Some("Free to be used".to_string()), Some(200));
            id1.print_s();

            id1
        }
    };
}
// pub(crate) use c_wrapper;
// pub use c_wrapper;
#[allow(unused)]
pub(crate) use c_wrapper;
