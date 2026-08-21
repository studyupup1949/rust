use diesel::pg::PgConnection;
use diesel::connection::Connection;
use ackorelic::nr_connection::NRConnection;
use ackorelic::skill::Skill;
use ackorelic::tables::users_skill::dsl::users_skill;
use ackorelic::tables::users_skill::dsl;

use diesel::prelude::*;



use ackorelic::newrelic_fn::{nr_start_web_transaction};






//thread_local! {
//    static transaction: Transaction;
//}


//mod nr_connection;

use std::cell::RefCell;
use std::thread;
use core::borrow::BorrowMut;

thread_local! {
    static FOO: RefCell<f32> = RefCell::new(1.0);
}

pub fn main(){
    nr_start_web_transaction("main tr");
    let database_url = "postgres://root@127.0.0.1/acko";
    //println!("in");
    let conn = PgConnection::establish(database_url).expect(&format!("Error connecting to {}", database_url));
    //println!("out");
    let query = "select * from users_skill";
    let _result = conn.execute(query).unwrap();
    //println!("pg result : {}", result);

    let nr_conn = NRConnection::establish(database_url).expect(&format!("Error connecting to {}", database_url));
    let _nr_result = nr_conn.execute(query).unwrap();
    //println!("nr result : {}", nr_result);

    let results = users_skill
        .filter(dsl::id.gt(20))
        .load::<Skill>(&nr_conn)
        .expect("Error loading skills");

    //println!("Displaying {} skills", results.len());
    for _skill in results {
        //println!("id: {} name: {}", skill.id, skill.name);
    }



//    let result1 = sql_query(query)
//        .load::<Skill>(&nr_conn)
//        .expect("Error loading skills from sql query");
//
//    //println!("Displaying {} skills from sql query", results.len());
//    for skill in result1 {
//        //println!("id: {} name: {}", skill.id, skill.name);
//    }

    FOO.with(|foo| {
        // `foo` is of type `&RefCell<f64>`
        *foo.borrow_mut() = 3.0;
    });

    thread::spawn(move|| {
        // Note that static objects do not move (`FOO` is the same everywhere),
        // but the `foo` you get inside the closure will of course be different.
        FOO.with(|_foo| {
            //println!("inner: {}", *foo.borrow());
        });
    }).join().unwrap();

    FOO.with(|_foo| {
        //println!("main: {}", *foo.borrow());
    });

//    let x : bool  = *ENABLE_NEW_RELIC;
//    if x {
//        println!("HI");
//    }
//        let enable_nr = env::var("ENABLE_NEW_RELIC").unwrap_or_else(|_| "false".to_string());
//    println!("{}", &enable_nr);
//        let x : bool = FromStr::from_str(&enable_nr).unwrap();
//        println!("{}", &x);
//        //println!("ENABLE_NEW_RELIC :{}", x.clone().unwrap());

}