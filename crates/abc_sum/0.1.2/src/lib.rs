pub mod calc {
    pub mod section {
        pub fn sum(input1: u32, input2: u32) -> u32 {
            input1 + input2
        }
        pub fn div(input1: u32, input2: u32) -> u32 {
            input1 + input2
        }
    }
}

pub use calc::section::sum;
