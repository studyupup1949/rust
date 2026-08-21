//! #About_big_project Crate
//! 
//! `about_big_project` is a collection of utilities to make performing certain
//! calculations more convenient.
///Adds one to the number given
/// 
/// # Examples
/// 
/// ```
/// let arg = 5;
/// let answer = about_big_project::add_one(arg);
/// 
/// assert_eq!(6,answer);
/// ```
/// 
pub fn add_one(x:i32)->i32{
    x+1
}//U can use cargo doc to generate a HTML doc in target/doc 
//by using cargo doc -open to crate and open the HTML
//Some common chapters
//#Example 
//#Panic  where func could happen panic  
//#Errors if func returns Result,describe possible ErrorSort and the conditions lead to the Error 
//#Safety if the func used unsafely,explain the reason and the premise‌ of using the user need to ensure

/* //!可以对外层条目添加注释，常用于描述crate 模块 */

//crate的程序结构会被开发者分成很多层，这使得使用者想找到这种深层结构中的某个类型很费劲
//eg. my_crate::some_module::a_new_module::UsefulType;
//我们不需要重新组织内部代码结构，使用pub use就可以创建一个与内部私有结构不同的对外公共结构
pub use self::kinds::PrimaryColor;
pub use self::kinds::SecondaryColor;
pub use self::utils::mix;

pub mod kinds{
    #[derive(PartialEq)]
    pub enum PrimaryColor{
        Red,
        Yellow,
        Blue,
    }

    pub enum SecondaryColor{
        Orange,
        Green,
        Purple,
    }
}

pub mod utils{
    use crate::kinds::*;
    pub fn mix(c1:PrimaryColor,c2:PrimaryColor)->SecondaryColor{
        if(c1==PrimaryColor::Red&&c2==PrimaryColor::Yellow)||(c2==PrimaryColor::Red&&c1==PrimaryColor::Yellow) 
        {SecondaryColor::Orange}
        else if(c1==PrimaryColor::Red&&c2==PrimaryColor::Blue)||(c2==PrimaryColor::Red&&c1==PrimaryColor::Blue)
        {SecondaryColor::Purple}
        else if(c1==PrimaryColor::Yellow&&c2==PrimaryColor::Blue)||(c2==PrimaryColor::Yellow&&c1==PrimaryColor::Blue)
        {SecondaryColor::Green}
        else {panic!("Wrong match")}
    }
}