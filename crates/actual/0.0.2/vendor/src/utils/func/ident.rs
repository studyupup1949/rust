#![allow(unused, dead_code)]
use std::collections::HashMap;

// use std::sync::Mutex;
use std::sync::{
    LazyLock, Mutex,
    atomic::{AtomicU64, Ordering},
};
#[derive(Clone, Debug)]
// i should write doscs ofor htis cauz evertime i sue this i change the featuring,
// 'cid means that , we put that in sourcec_all'
// i've only used this 2 times  adn geuss waht both of thsose times  changed.

pub struct Identifier {
    pub name: String,
    pub id: String,
    pub pid: Option<u64>,
    pub location: String,
    pub description: Option<String>,
    pub return_type: Option<Vec<String>>,
    pub return_value: Option<Vec<String>>,
    pub args_type: Option<Vec<String>>,
    pub number_of_args: Option<usize>,
    pub args: Option<Vec<Argument>>,
    pub source: Option<String>,
    pub source_call: Option<String>,
    pub cid: Option<String>,
    pub called_by: Option<Vec<String>>,
    pub status: Status,
    pub validate: bool, // make this a sperpate mutex struct that is contiant is_valid, valdating, and status, which is either 'curentl inserted'
    // or multiple tries of trying to insert it cuaz the func was ran mmultiply itmes nad add a countter for it enum::x(u32)
    // or atomic u32, whtvr,
    // also curnl if is_valid is false we wount nto knwow the reason lets learn the reason for it aswel.
    // a vector contianting all the reasons of alure, like more argumetns in x in x
    // or smt else faliure.
    // mkaeg
    pub func_pointer: Option<fn() -> Identifier>,
}
//the myserios bandid sosoerty

#[derive(Clone, Debug)]
pub struct Status {
    pub status_title: Status_T,
    pub status_code: u64,
}

#[derive(Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum Status_T {
    Working(Option<String>),
    Good(Option<String>),
    Failed(Option<String>),
    Paused(Option<String>),
}

#[derive(Clone, Debug)]
pub struct Argument {
    pub name: String,
    pub ty: String,
    pub val: String,
}

impl Identifier {
    pub fn print_s(&self) {
        println!("{:?}", self.status); // Sikol
    } // This is source code.
    pub fn s_status(&mut self, a: Option<String>, code: Option<u64>) {
        self.status
            .status_code = code.unwrap_or(41414141);
        self.status
            .status_title = match self
            .status
            .status_code
        {
            100..=199 => Status_T::Working(a),
            200..=299 => Status_T::Good(a),
            400..=599 => Status_T::Failed(a),
            _ => Status_T::Failed(None),
        }
        /*
         ***REMOVED***
         ***REMOVED***
         ***REMOVED***
         ***REMOVED***
         ***REMOVED***
         ***REMOVED***
         ***REMOVED***
         ***REMOVED***
         */
        // anyhow . so lwk fix this. and create a .json file contianing all the statuscode frm ths ewbstie1:
        /*
        https://www.rfc-editor.org/info/rfc6455/#section-7.4.1
        https://en.wikipedia.org/wiki/Deprecation
        WEBSOCKET RULES / CODES NOT HTTP
        https://www.rfc-editor.org/info/rfc9110/
        https://ryam.medium.com/http-status-codes-arent-optional-stop-using-200-for-everything-e1698a09f8fa
        https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Status
        READ the rest for full info.
        NO AI allowed asusual
         */
    }
    // AHAHA
    //
    // Pu pur rui rui rui atau rui rui rui rui rui rui uri
    // AI ADDED THE CHECUP PART THE REST WAS ALR MADE
    pub fn generate_pid(&mut self) -> &mut Identifier {
        if self.pid.is_none() {
            let mut name_map = PID_TABLE
                .name_to_pid
                .lock()
                .unwrap();
            if let Some(&existing_pid) = name_map.get(&self.name) {
                self.pid = Some(existing_pid); // reuse — same function as before
            } else {
                let new_pid = PID_TABLE.next_pid();
                name_map.insert(self.name.clone(), new_pid);
                self.pid = Some(new_pid);
            }
        }
        self
    }
    pub fn s_lock(&mut self, a: String) {
        self.status
            .status_title = Status_T::Failed(Some("".to_string())) // why are we doing a "".to_string , why not none 
    }
    // Learn to use phantom data or wtvr to lock and unclokc
    // pub fn generate_pid(&mut self) -> &mut Identifier {
    //     // dont do self her cauz if we do sel id would be consued, adn if we do u32 then method vlalidate wont get called
    //     self.pid = PID_TABLE.next_pid();
    //     self
    // }
    // pub fn

    fn v() -> i32 {
        1
    }
    pub fn validate(&mut self) -> bool {
        let pid = self.pid.unwrap(); // safe: guaranteed Some by the line above
        let lenght = self.name.len();
        let mut pid_validate = false;
        let mut table = PID_TABLE
            .table
            .lock()
            .unwrap();
        match table.get(&pid) {
            Some(_) => {
                println!("validation => false (duplicate PID: {pid})");
                pid_validate = false; // fix this by using the validat struct idea (is validate, vec<enum> nd other answrs)
            }
            None => {
                self.validate = true;
                table.insert(pid, self.clone());
                pid_validate = true;

                // Auto-register function pointer if it exists
                if let Some(func_ptr) = self.func_pointer {
                    drop(table); // Release lock before acquiring REGISTRY lock
                    let mut registry = REGISTRY
                        .lock()
                        .unwrap();
                    registry.register(pid, func_ptr);
                    println!("Registered: {} (PID: {})", self.name, pid);
                }
            }
        }

        // fn test(A: Identifier) {
        // sum(unsafe { Identifier }, 1)

        // }

        // if pid_validate
        //     && !(1000..=9999).contains(
        //         &self
        //             .status
        //             .status_code,
        //     )
        // {
        //     true
        // } else {
        //     false
        // },this was coded by rust anlayuzer u comented this auz rust analyzer siad u can write hte one bleo w better
        if 41414141
            == self
                .status
                .status_code
        {
            println!("Enter Valid STATUS_CODE.\n STATUSCODE IS MISSING OR INVALID CODE")
        }
        {
            {
                {
                    {
                        {
                            {
                                {
                                    {
                                        {
                                            {
                                                {
                                                    println!("Walter");
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        /*

                ██████╗ ██╗   ██╗███████╗████████╗
                ██╔══██╗██║   ██║██╔════╝╚══██╔══╝
                ██████╔╝██║   ██║███████╗   ██║
                ██╔══██╗██║   ██║╚════██║   ██║
                ██║  ██║╚██████╔╝███████║   ██║
                ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝
        ████████████████████████████████████████████████████████████████████████████████████████
                */
        let status = self
            .status
            .status_code;

        let valid_pid = pid_validate;
        let valid_range = !(1000..=9999).contains(&status);
        let not_magic = status != 41_414_141;
        let valid_length: bool = (3..200).contains(&lenght);
        // if walter >= 10 {...}
        // let walt: bool;
        // println!("{}", walt);
        self.validate = valid_pid && valid_range && not_magic && valid_length;
        // dont do ||, do &&,
        /*
        0----||-|
        1----||-|--------- 1
        0----||-| 0
        0----||-| 0
        So return 1.
        but we want to fail even if one.

        1-----&&--|
        1---------1----||----0    // Now this is 0 cauz one if 0
        0----------0----||--|      // Since one is '0',it reutn o
        1-----&&--|

        */
        self.validate
        // self.validate = pid_validate
        // && !(1000..=9999).contains(
        // &self
        // .status
        // .status_code,
        // )
        // && (41414141
        // != self // iDK why but u dod &self and hten * so *& lol.
        // .status
        // .status_code)
        // && (lenght < 200)
        // && (lenght > 2);
        // self.validate // Create a system msg ysstem for code= 4141... on  line 55.
        // smt like: 'Failed to parse status_code, enter valid status_code'.
    }

    // pub fn validate(&mut self) -> bool {
    //     // if self.validate =
    //     let mut table = PID_TABLE
    //         .table
    //         .lock()
    //         .unwrap();
    //     // PID_TABLE
    //     match table.get(&self.pid) {
    //         Some(_) => {
    //             println!("validation => false (duplicate PID: {})", self.pid);
    //             return false;
    //         }
    //         None => {
    //             self.validate = true;
    //             table.insert(self.pid, self.clone());
    //             return true;
    //         }
    //     }
    //     table
    //         .get(&self.pid)
    //         .is_some()

    //     // .lock()
    // }
    // fn validate(&self) {}
}
pub static PID_TABLE: LazyLock<PidTable> = LazyLock::new(|| PidTable {
    table: Mutex::new(HashMap::new()),
    name_to_pid: Mutex::new(HashMap::new()), // <- new
    next: AtomicU64::new(0),                 // first call returns 0, next call returns 1, etc.
}); // AI told me to addd the name_to_pid
pub struct PidTable {
    pub table: Mutex<HashMap<u64, Identifier>>,
    name_to_pid: Mutex<HashMap<String, u64>>, // <- new
    next: AtomicU64,
}

// Recreate most of this for uhh ig ANOTHER TUI syteminsdie this PId system
impl PidTable {
    pub fn print(&self, pid: Option<u64>) {
        let table = &self
            .table
            .lock()
            .unwrap();
        match pid {
            Some(target) => match table.get(&target) {
                Some(identifier) => println!("{target} -> {identifier:?}"),
                None => println!("no identifier found for pid {target}"),
            },

            None => {
                for (pid, identifier) in table.iter() {
                    println!("{pid} -> {identifier:?}")
                }
            }
        }
    }
    pub fn next_pid(&self) -> u64 {
        self.next
            .fetch_add(1, Ordering::SeqCst)
    }
}

pub struct Registry {
    pub funcs: HashMap<u64, fn() -> Identifier>,
}
impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}
impl Registry {
    // pub fn default() -> Self {
    //     Self.new()
    // }
    pub fn new() -> Self {
        Registry {
            funcs: HashMap::new(),
        }
    }

    pub fn register(&mut self, pid: u64, func: fn() -> Identifier) {
        self.funcs
            .insert(pid, func);
    }

    pub fn call(&self, pid: u64) -> Result<fn() -> Identifier, String> {
        self.funcs
            .get(&pid)
            .copied()
            .ok_or_else(|| format!("Function with PID {} not found", pid))
    }

    pub fn list_all(&self) -> Vec<u64> {
        self.funcs
            .keys()
            .copied()
            .collect()
    }
}

// AI GEN:

pub static REGISTRY: LazyLock<Mutex<Registry>> = LazyLock::new(|| Mutex::new(Registry::new()));
// struct functionName {
// name: Has,
// }

// impl functionName {}

// impl Identifier {
//     pub fn new_pid(self) {
//         self.pid  =
//     }
//     pub fn add_pid(self) -> Result</*String*/ (), String> {
//         let mut table = PID_TABLE1.TABLE
//             .lock()
//             .unwrap(); // is used if the thingy is poisen, best prevetned if the thread is being pannic before that unuse it

//         if table.contains_key(&self.pid) {
//             //     return Err(format!("PID {} already exists!", self.pid));
//             panic!("PID {} already exists!", self.pid);
//         }

//         table.insert(self.pid, self.clone());
//         println!("ADDED PID {}", self.pid.clone());
//         // let nice = "Added".to_string();
//         // Result<nice,
//         Ok(())
//     }
// }

// static mut PID: u32 = 1;
// unsafe {
// static mut PID_TABLE: HashMap<u32, Identifier> = HashMap::new();
// }
// fn longest(a: &str, b: &str) -> &str {
//     if a.len() > b.len() { a } else { b }
// }

// fn pid() {
//     let v: Vec<i32> = vec![20, 20, 20];
//     println!("{:?}", v.clone());
//     unsafe {
//         PID += 1;
//     }
//     //-> u32 {
//     let pid_table: HashMap<u32, Identifier> = HashMap::new();
//     //     let walter = PID_TABLE.get(0);

//     //     std::process::id()
// }
// pub fn test() {
//     let w = PID_TABLE.get(0);
//     println!("{w:?}");
// }
// pub fn print_by_pid(n: Option<u32>) {
//     match n {
//         Some(n) => {
//             let m = n; //n as u32;
//             let v = get(m);
//             print!("{}", m);
//             print!("{:?}", v);
//         }
//         _ => {
//             println!("None")
//         }
//     }
// } // wht the hell where u trin a do
// pub fn get(pid: u32) -> Option<Identifier> {
//     let table = PID_TABLE.TABLE
//         .lock()
//         .unwrap();

//     table
//         .get(&pid)
//         .cloned()
// }
// impl Identifier {
//     fn next_id() -> String {}
//     fn new() {}
//     fn verify(&self) {}
//     fn pid(&self) -> u32 {
//         std::process::id()
//     }
//     fn get_pid_table(&self) -> HashMap<u32, Identifier> {
//         for (pid, id) in &table {
//                 println!("PID: {}, Identifier: {:?}", pid, id);
//         }
//         table
//     }
// }
// use std::collections::HashMap;

// lazy_static! {
//         #[derive(Clone, Copy,Debug)]
//     pub static ref PID_TABLE_HASH: Mutex<HashMap<u32, Identifier>> = Mutex::new(HashMap::new());
//     #[derive(Clone,Debug,Copy)]
//     pub static ref PID_TABLE_MAX: u32 = 0;
//     #[derive(Clone,Debug)]
//     pub static ref PID_TABLE : PID_TABLE1 = PID_TABLE1 {
//         TABLE: PID_TABLE_HASH,
//         MAX : PID_TABLE_MAX
//     };
// }
//     #[derive(Clone,Debug)]
// pub struct PID_TABLE1 {
//     TABLE : PID_TABLE_HASH,
//     MAX : PID_TABLE_MAX,
// }

// impl PID_TABLE1 {
//     pub fn get(&self, pid: u32) -> Option<Identifier> {
//         let table = &self.TABLE.lock().unwrap();

//         let nice = table
//             .get(&pid)
//             .cloned();
//         println!("{nice:?}");
//         nice
//     }
// }
