//! # Art
//! 描述：这是一个测试库
//! 作者：张维
//! 时间：2021-05-09
//! 版本：1.0




// 文档注释

/// 函数 add 
/// 描述：两个数相加
/// 参数：
/// a: i32
/// b: i32
/// 返回值：i32
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// 使用 pub use 将内部层级较深的模块 导出到外层 方便外部使用
pub use self::kinds::PrimaryColor;
pub use self::kinds::SecondaryColor;
pub use self::utils::*;

pub mod kinds {
    pub enum PrimaryColor {
        Red,
        Yellow,
        Blue,
    }

    pub enum SecondaryColor {
        Orange,
        Green,
        Purple,
    }
}

pub mod utils { 
    use crate::kinds::*;
    pub fn mix(c1: PrimaryColor, c2: PrimaryColor) -> SecondaryColor { 
        // --snip--
        SecondaryColor::Orange
    }
}
