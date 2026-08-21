use crate::{
    // dsa::collections::c_bubble_sort,
    utils::{
        self,
        func::ident::{Argument, Identifier, /*PID_TABLE,*/ Status /*Status_T*/},
        // r#use::get_input,
    },
};

pub fn bs_1() -> Identifier {
    // cutility::
    println!("\n\n\n\n\n\n ###### Start of dsa::c_bubble_sort::bs_1() ######");

    let mut vectors = vec![30, 40, 10, 0, 20, 100, 20, 1000];
    //     let asnwer: &str = &format!("{:?}", vectors);
    let un_sorted = vectors
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(",");
    crate::dsa::collections::bubble_sort::bubble_sort(&mut vectors);
    let sorted = vectors
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(",");
    // ##################################
    let mut id2 = Identifier {
        name: "bs_1".to_string(),
        id: "sortB_1".to_string(),
        pid: None,
        location: "dsa::c_bubble".to_string(),
        description: Some("Sorts the array, adn returns sorted (uses &mut self).".to_string()),
        return_type: Some(vec!["Vec<i32>".to_string()]),
        return_value: Some(vec![sorted.clone()]), // we do some because it might return noin, but vector nothing menaswati,
        // why we do none if some is vec?
        args_type: Some(vec!["Vec<i32>".to_string()]),
        number_of_args: Some(1),
        args: Some(vec![Argument {
            name: "vectors".to_string(),
            ty: "Vec<i32>".to_string(),
            val: un_sorted.clone(),
        }]), // Add a system to make ure numbres of arg is euqla to vector indeix of each thingy, lets say we accdily, didi
        // args = two args then that would be incorrect.
        // and also add a system for validate to be a struct, bool + string, string would contian the reason for falure
        // rather a shitting thing.
        // vasdd on validate's string error (or enum contianngi n astring) we change status_code,
        // if a func syas it has 2 args but only shows 1 , we change status_code to 'works but might acuze issues'
        // So avlidate looks like:
        // struct { validation : bool, message: Enum(String)}
        // enum = 'arg_failure', 'return_value_faluire', 'issue iwth cid being euqal to somethign else', 'two identifcal'
        // 'two idientiical functs in smae file mtnoned anyhwer'
        // but that would require everyhtingto be in a database, and i wont use mutex,or static.
        // we gotta wait untill i leanr rust enought o jump to databases
        source: Some("dsa::bubble_sort::bubble_sort()".to_string()),
        source_call: Some("dsa::c_bubble_sort".to_string()),
        cid: Some("calle.2".to_string()),
        called_by: Some(vec!["main.rs::bubble_sort()".to_string()]),
        status: Status {
            status_title: utils::func::ident::Status_T::Working(Some(
                "for_c_bubble_sort".to_string(),
            )),
            status_code: 523,
        },
        validate: false,
        func_pointer: Some(bs_1 as fn() -> Identifier),
    };
    id2.print_s();
    id2.generate_pid()
        .validate();
    id2.print_s();
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
    let pid = id2
        .pid
        .expect("NO PID AHAH");
    //Money Talks
    // just realise  i can just do;
    // let w = id1.pid.expect("NO PID AHAH");
    //let w = id1.pid.unwrap_or_else(|| panic!("NO PID AHAH"));
    // or, for a default:
    // let w = id1.pid.unwrap_or(0);
    println!("id1 is {}", id2.validate.clone());
    //     println!("Result: {}", result);
    println!("PID = {}, {:?}", std::process::id(), pid);
    //     println!("Unsorted = ?}",);
    println!("Unsored = {}", un_sorted);
    println!("Unsored = {}", sorted);
    println!("Calling from c_bubble_sort::bs_1 to dsa::bubble_sort");
    println!("###### End of dsa::c_bubble_sort::c_bubble_sort::bs_1() ######\n\n\n\n\n");
    println!("{:?}", id2.clone());
    // id1.status
    //     .status_title = utility::func::ident::Status_T::Good(Some("Free to be used".to_owned()));
    // id1.status
    //     .status_code = 200;
    // id1.s_free(Some("Free to be used".to_string()));
    id2.s_status(Some("Free to be used".to_string()), Some(200));
    id2.print_s();

    id2
}

// fn bs_2()
