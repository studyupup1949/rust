// use crate::utils::get_input;

// use bigdecimal::BD;

// #[derive(Default)]
// pub struct Tables {
//     number: BD,
//     start: BD,
//     end: BD,
//     step: BD,
//     table_data: Vec<(String, String, String)>,
// }

// impl Tables {
//     pub fn new() -> Self {
//         Self::default()
//     }

//     /// Makes a table.
//     pub fn generate(&self) {
//         let number: BD = get_input("Enter number ");
//         let start: BD = get_input("Enter start  ");
//         let end: BD = get_input("Enter end    ");
//         let step: BD = get_input("Enter step   ");
//         let mut table = Tables {
//             number: number,
//             start: start,
//             end: end,
//             step: step,
//             table_data: Vec::new(),
//         };
//         let mut current = table.start;
//         // let mut rows: Vec<(String, String, String)> = Vec::new();
//         let rows = &table.table_data;
//         while current <= table.end {
//             let result = &table.number * &current;
//             table
//                 .table_data
//                 .push((number.to_string(), current.to_string(), result.to_string()));
//             current += step.clone();
//         }

//         // widest string in each column
//         let num_w = rows
//             .iter()
//             .map(|(n, _, _)| n.len())
//             .max()
//             .unwrap_or(0);
//         let cur_w = rows
//             .iter()
//             .map(|(_, c, _)| c.len())
//             .max()
//             .unwrap_or(0);
//         let res_w = rows
//             .iter()
//             .map(|(_, _, r)| r.len())
//             .max()
//             .unwrap_or(0);
//         // So each x_w calculates width.
//         // SINCE vec is 3 long and we know what we enter into what,
//         // ie: 1_2_3;
//         // 1 = number
//         // 2 = current (the multiplier)
//         // 3 = result
//         let line_w = 1 + num_w + 3 + cur_w + 3 + res_w + 1;
//         let border = "=".repeat(line_w);

//         println!("{border}");
//         for (n, c, r) in rows {
//             println!("|{:<num_w$} × {:>cur_w$} = {:>res_w$}|", n, c, r);
//             // I guess right aligh is better cauz
//             // 20
//             // 100
//             // is just weird
//             //  20
//             // 100
//             // is better :/
//         }
//         println!("{border}");
//     }
// }
use crate::traits::Reset;
use crate::utils::read_input;
use bigdecimal::BigDecimal as BD;

// pub trait Reset {
// fn reset(&mut self);
// }
// #[derive(Reset)]
// Returns a new instance of Table. The the state of table is default.
#[doc = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/docs/math/new_table.md"
))]
pub fn new() -> Tables {
    Tables::default()
}

impl Reset for Tables {}
#[derive(Debug, Clone)]
/// Table struct is used to generate, print and modify tables, their contents and their rows.
///
/// Table struct that contains 7 fields one being private
/// struct is private Becauz we dont want user to manually accidently do v.initialized = false
///
///```rust
/// pub struct Tables {
/// pub number : BD // BD here is short for BigDecimal
/// pub start :
/// pub struct Tables {
///    pub number: BD,
///    pub start: BD,
///    pub end: BD,
///    pub step: BD,
///    pub table_data: Vec<(String, String, String)>,
///    pub curr: BD,
///    initialized: bool,
///}
/// ```
/// example usage:
/// ```rust
/// let math = actual::math::new();
/// let tables = math.table();
/// let t1 = tables.clone();
/// t1.auto_generate();
/// t1.print();
/// t1.reset();
/// t1.print();
///```
///$$ 10+ 10$$
pub struct Tables {
    // struct is private Becauz we dont want user to manually accidently do v.initialized = false
    pub number: BD,
    pub start: BD,
    pub end: BD,
    pub step: BD,
    pub table_data: Vec<(String, String, String)>,
    pub curr: BD,
    initialized: bool,
}
/*
start 0.1
step 0.2
end 1.

-1

end / step
= s

0.1
0.3
0.5
0.7
0.9
1.1 <-

*/
impl Default for Tables {
    fn default() -> Self {
        Self {
            number: BD::from(1),
            start: BD::from(1),
            end: BD::from(12),
            step: BD::from(1),
            table_data: Vec::new(),
            curr: BD::from(1),
            initialized: false,
        }
    }
} // ivauglue rmember doign some string to BD 
impl Tables {
    /// Creates a new instance of type 'Table'
    /// it roughly returns:
    /// ```rust
    ///Tables {
    ///    number: BD::from(1),
    ///    start: BD::from(1),
    ///    curr: BD::from(1),
    ///    end: BD::from(12),
    ///    step: BD::from(1),
    ///    table_data: vec![(1,1,1), ..., (1,12,12)],
    ///    initialized: true,
    ///  }
    /// ```
    pub fn new() -> Tables {
        let mut table = Self::default();
        table.generate();
        table
    }
    pub fn reset(&mut self) -> &mut Self {
        *self = Self::new();
        self.initialized = false;
        self
    }
    pub fn drop(self) {}
    pub fn init(&mut self, number: BD, start: BD, end: BD, step: BD) -> &mut Self {
        // macro_rules! nonzero_bd {
        //     (0) => {
        //         compile_error!("Step cannot be zero");
        //     };
        //     ($n:literal) => {
        //         bigdecimal::BigDecimal::from($n)
        //     };
        // }
        // nonzero_bd!(step);
        self.start = start.clone();
        self.curr = start;
        self.number = number;
        self.end = end;
        self.step = step;
        self.initialized = true;
        if self.step == 0 {
            loop {
                // self.stepper_0();
                self.step = read_input("Please enter a valid stepper :? PRETYPED ");
                if self.step == 0 {
                    continue;
                } else {
                    break;
                }
                //this should make status = false or wtvr
            }
        }
        self
    }
    pub fn auto_init(&mut self) -> &mut Self {
        self.number = read_input("Enter the number you want the table for ");
        self.start = read_input("Enter what number you want table to start from ");
        self.end = read_input("Where do you want the table to end ");
        self.curr = self.start.clone();
        loop {
            self.step = read_input("By how much do you want to increment the multiplier? ");
            #[allow(clippy::cmp_owned)]
            if self.step == BD::from(0) {
                // self.stepper_0();
                println!("STEP CANT BE 0");
            } else {
                self.initialized = true;
                break;
            }
        }
        self
    }
    pub fn auto_generate(&mut self) -> &Self {
        if !self.initialized {
            self.auto_init();
        }
        self.generate();
        self
        // self
    } // make it so that generate checks if initiizlaised or if Some returned,
    // it is not init and None, then it just makes table of 0 * 0 = 0,
    // if init and none then use init
    // if init and some then use init adn disguard some? idk bruh i cant make architectureim otooo stupid
    pub fn generate(&mut self) -> &Self {
        if !self.initialized {
            let zero_string = String::from("0");
            self.table_data = vec![(zero_string.clone(), zero_string.clone(), zero_string)];
        } else {
            let mut end_reached = false;
            while self.curr <= self.end {
                if self.curr == self.end {
                    end_reached = true;
                }
                self.table_data
                    .push((
                        self.number
                            .to_string(),
                        self.curr
                            .to_string(),
                        (&self.curr * &self.number).to_string(),
                    ));
                self.curr += &self.step;
            }
            if !end_reached {
                self.table_data
                    .push((
                        self.number
                            .to_string(),
                        self.end.to_string(),
                        (&self.end * &self.number).to_string(),
                    ))
            }
        }
        self
    }
    fn column_width(&self) -> (usize, usize, usize) {
        let num_w = self
            .table_data
            .iter()
            .map(|(n, _, _)| n.len())
            .max()
            .unwrap_or(0);
        let cur_w = self
            .table_data
            .iter()
            .map(|(_, c, _)| c.len())
            .max()
            .unwrap_or(0);
        let res_w = self
            .table_data
            .iter()
            .map(|(_, _, r)| r.len())
            .max()
            .unwrap_or(0);
        (num_w, cur_w, res_w)
    }
    pub fn print(&self) {
        if self.table_data == Vec::new() {
            println!("table_data not generated. needs fixing generating")
        }
        let len = (&self.end - &self.start) / &self.step;
        let mut zero = BD::from(0);
        if zero <= len {
            zero += BD::from(1);
        }
        let (num_w, cur_w, res_w) = self.column_width();
        let line_w = 1 + num_w + 3 + cur_w + 3 + res_w + 1;
        let border = "=".repeat(line_w);
        if self.end == 0 {
            println!("END = 0 LOOP END RN")
        }
        if self.number == 0 {
            println!("all number finna be 0 fr fr")
        }

        println!("{border}");
        for (n, c, r) in &self.table_data {
            println!("|{:<num_w$} × {:>cur_w$} = {:>res_w$}|", n, c, r);
            // make a trait for this aswell
        }
        println!("{border}");
        // fn return_row(&mut self) {}
    }
    pub fn stepper_0(&self) {
        println!("{self:?}");
    }
    pub fn into_init() {}
    // pub fn
}
