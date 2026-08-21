// 在包的根模块或任意模块文件顶部增加模块级文档；
// 所谓模块级文档，是指为整个模块而不是单独为其下方的语法元素生成文档；

//! This is the documentation for the 'aaa_csv_challenge' lib crate.
//!
//! Usage:
//!```
//! use aaa_csv_challenge::{Opt,{load_csv,write_csv},replace_column};
//!```

mod core;
mod err;
mod opt;

pub use self::core::{
    read::{load_csv, write_csv},
    write::replace_column,
};
pub use self::opt::Opt;
