//! Entry Point to actual code.
//! Offically the BEST library inthe world

// Mkae a clippy for hte step != 0 thingyz
// #[macro_use]
extern crate lazy_static;
// use std::vec;
#[allow(unused)]
pub(crate) use utils::macros::m_ident::c_wrapper;

// pub(crate) use utils::r#use::get_input;

// use vector_double::DoubleAll;

// use std::intrinsics::powf128;
/*
make a prelude for each:


// prelude.rs (crate root)
pub use crate::utils::prelude::*;
pub use crate::dsa::prelude::*;
pub use crate::math::prelude::*;

The idea taht u do get_input, make input and diff funcs meaning diff shit idr what it was on about
with_x means return clone, x means mutate or wtvr let me check

*/
//I guess do something like:
// input.new ..
// input.reset (cauz input would have a couple fieds)
// and if we want to lets say read inputfrom a file we do:
// .read_from(..)
// and return to return a value
// drop to drop input
// reset to default state of input
// smt liek:
// enum :wrte
// onto
// from
// struc tinput
// write : write::Onto
// file locaiton: <Option<String>
// content = Option
// into means requres a mutable refce of a varbles u want to rwite input.contents onto.
// if uwant to mkae input do .read().
// if u want to mutabl ear u do .into(&mut x)
// if u want to modiyf file we just do .to(Option string)
// if to optin is given we overrwite locaiton
// if not we write to curr
// if it is alr none then we make a temp file
// adn if we want to use that file later , we can access, a pub mutex vec contianting all fiels.
// i guess make a file have somethign like x.l.m.  x being type, l being wtvr i geuss m
// so smt futureistic. i geuss
// what did we start and waht are we ending at lol.
// .return() to reutnr val.
// .from() to modify conents to new.
// .add_to_regex() if we want to add contnes into regis (lets ay we wnat ot .rseet )
// but in furute we want to access taht contnte again yk.
//
//
pub mod acc_soft;
mod database;
pub mod dsa;
pub mod init_call_to;
pub mod math;
pub mod menu;
pub mod prelude;
pub mod reexports;
pub mod rust_book;
pub mod test;
pub mod traits;
pub mod tui;
pub mod udemy;
pub mod ui;
pub mod utils;
// pub fn print(&self) { // make a trait for this aswell
// pretty pirnt for tables andhwantot
// also make a 'range' method, cauz i did alot of .len custom shit like starts from 1 stesp up by 1 and goes till 12 what is len.
// 1
// 2
// ..
// 12.
// len is 12.
// how come? 1 * 12 ?
// wait let me think
// 0.1 start
// 1 end.
// step 0.2
// 0.3
// 0.5
// 0.7
// 0.9
// 1.1
// wait also make it so taht it compares
// curr+step <= end.
// so uhh
// what was i doing
// yes so
// so 6 count.
// hmmm let me think
// 5 count (Technically)
// len  = end/step
// now idr why i have the 'location thigny, but my guess is that for c_x,
// we know locaiton of c_x + location of what its calling to.
// so location wuld be the actual func (add.rs)
// and hten cID would be equal to the c_x which is c_add,
// and if none htat meann o c_x?
// WHY DID I ADD ALL TEHS SHI T ODIENFIER BUT NEVER TOLD MY SELF WHAT IT IS
/*/
logged_fn! {
    name: "section_1_8",
    id: "ud.1.8",
    location: "udemy::section_1_8",
    description: "So basically no c_func for htis",
    body: {
        let mut p = Person::new("Walter".to_owned(), 20);
        println!("P's values = {}", p.greet());
        p.age_up(20);
        // ... whatever you want, freely
    }
}P */
/* fn main() {
    println!("Hello, world!");
    // powf128(10, 10)
    math::add::add_two(5, 3);
    math::add::add_two(5, 3);
}
    */
// use crossterm::event;
// use utils::func::init_call_to;

// mod utils;

// use ui::tui;
// use utils::func::init_call_to;
/*
A couple rpoblesm other then the fact that we want custom ye,
well we mark funcs with 'input takes' as smt else, and lockthingy,
adn thnebfoer calling to func we chkec its id1 adn seei fi t CAN be caleld.


arg types nad value type as a Int, string or wtvr



anoteh thing registry, TUI.rs , and uhh anyhgn related to registry is
NOT coded by u, MAKE srue to leanr them after learnjng basics of rust

check if  itis  posible that i add a mehtod .working_for()
and it bascially modifes the status of idenfiter to working for the fucntion it is in rn, example tui.rs::tui
Or mian.rs or c_v.rs?


and iguess TUI refuses to call to identifers that hav estatus 'working' but since im planing on saving logs
OH fuhh logs adn hwantot

so waht was i thinkig ye, so if it sees that idenfei struct one is locked ,what we do is we need to have another
 thingy or hashmap that has stuats and pid of func, and it is 100% syncehd cauz calling to stautus updates that
 other hasmpa, and if u modiy status hasmp but idenfier is locked it does not allow to modify
 the reaso nfor htis is lets say TUI wants to check if pid-1 is callable, but how will it know if
 accessing identifer is not allowed since it is locked? will simple the hasmpa solution.

add a 'clear screen' / databases program to this.
cler cscrene or chnge color.  or remove option temp.
Or view logs, or all the TUI for calc or a new TUI instance. or refersh. (relaod all databases., notice
how i said all, that means whnever we open file we do so through a util helper . :brian)

I mean i guess thats a littel TOO ambisions for someone who used ai for the TUI, im againsed AI
so ig LEANR rust fiist tsi only been like 2-3moths, leanr the shi wan tt othen TUI then finish.

a text editor would be cool

a  copy result or show desciptin using t could be cool and using arrow key toselect would bekooler;


a vsde extenio ntaht allwo u to copy
        ("1/998001", Rational::from((1, 998001))),
like u wanted to change  the 998001 in 1/... and an extnein did htat ooudl be crazy

*/
// also in the init folder in all functiosn that take input create a systme so that the inputs are auto fed.
//and for funcs like this where they input has to equal to smt or lets say it lawasy has to be pstile
// createa  input value = Enum::Type(val), and each func identifer has type, and if type type not found panic, if found.
// use that type in example of inputs (ill adda 'two input takaen' and' input type'thing),
// so that this loading process is insatn, and make a loading animaton
//
// Make a systemfor rust_book where it open another menu u can exit and u can all to rust_books's funcs (seperare ident system)
//
//
// println!("\n\n\n\n\n\n ###### Start of rust_book::guessing_game::guessing_game() ######");
//lwk i think this gotta go.
// lets add anotehr mutex or global thingy that holds the location
// as a  string.
// then we can use it to print his cauz this shit is ugly af to write again and agian.
// ye ig anotehr todo list
fn main() {
    let _ = init_call_to::call();
    // init_call_to::init_all_functions();
    print!("\x1B[2J\x1B[1;1H");
    ui::run_tui(); // turn this into a _header that calls to UI or wtvr needed and UI allows to 
    // chose between this TUI (which si for learing purposes) or anotehr TUI whcih would contain
    //stuff u need on a daily.
}

/*
So make tui.rs return a TUI struct which u can call methods on.

Cauz each fuckin uhh PID system (one for learning, one for actual project, oen for udemy inside learning, one for fukin, rust_book
, and uhh one for sections inside Udemy.)

*/

/*/
official/actual/src/math/call/c_tables.rs

So we see in taht and in get input + in the tables.rs we use &str , and String is useless aera.
get input cANT return &str cauz lifetime issue it is easy solvable using 'static or wtvr.
SO now use that to make sure getinput can return &str (cauz current FromStrr is not for str only for String.)
and tables .rs does not acept String for somereason?
*/

/*

So now the auto register func without intit + the auto einput is neede make enum that has tpe and hten indadata base hwtvr input matches it
uses that and auto entr input yk.*/
/*

SO make it so that  u have a func 'writeLog' and each function (math::add()) bascially funcsitosn adn writes logs, called by, performed , adn whatnto
and hten each c_x or hlper func clals writes logs
SO alo


/*

Make docs for all funcstiosn*/
current method for making sure PID does not get reasing or wtvr whnw func is redeclared or wtvr cauz one func run mutlply time
usign the hashmap thingy AI said is bad ifx it ur self assoon as u can:
                pid_validate = false; // fix this by using the validat struct idea (is validate, vec<enum> nd other answrs)
adn a couple other probes in ident.rs.


                */

// So it takes type T and goes till U which is whatever ending.
// and it makes a new object taht u can call struct for either power table,
// multiplicaiton.
// or sqeuesce *2 or / 2 or  + 1  + 2  or smt esle.
// pub struct table {}
// pub struct Tables {
// number : bignum/decimal
// end table : ....
// co
// }

/*
   make ts into alib so anyone can refence any func they have based on PID

   Me
*/
