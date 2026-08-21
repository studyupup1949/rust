//! # 我的 a 加 b 函数
//!
//! `a_add_b` 它实现了传入两个数值，返回这两个数字相加之和。
//! 感兴趣的话就玩一玩吧！！！
//! 
//! # Art
//!
//! A library for modeling artistic concepts.

pub use self::kinds::PrimaryColor;
pub use self::kinds::SecondaryColor;
pub use self::utils::mix;

/// 这是一个加法的公开函数
///
/// # 异常情况
/// 
/// 这个程序这样会报错
/// ```
/// println!(add(arg));
/// ```
/// 
/// # 必须满足以下条件
///
/// ```
/// // 必须要有传入两个参数
/// // add(arg1,arg2);
/// ```
///  
/// # 用法
///
/// ```
/// let arg = 5;
/// let arg1 = 6;
/// let answer = a_add_b::add(arg,arg1);
///
/// assert_eq!(11, answer);
/// ```
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}



pub mod kinds {
    /// The primary colors according to the RYB color model.
    pub enum PrimaryColor {
        Red,
        Yellow,
        Blue,
    }

    /// The secondary colors according to the RYB color model.
    pub enum SecondaryColor {
        Orange,
        Green,
        Purple,
    }
}

pub mod utils {
    use std::iter::Enumerate;

    use crate::kinds::*;

    /// Combines two primary colors in equal amounts to create
    /// a secondary color.
    pub fn mix(c1: PrimaryColor, c2: PrimaryColor) -> SecondaryColor {
            SecondaryColor::Green
    }
}