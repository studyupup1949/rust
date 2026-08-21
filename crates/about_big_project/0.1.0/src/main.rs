// use about_big_project::kinds::PrimaryColor;
// use about_big_project::utils::mix;
use about_big_project::PrimaryColor;
use about_big_project::mix;

fn main() {
    let red = PrimaryColor::Red;
    let yellow = PrimaryColor::Yellow;
    mix(red, yellow);
}
/*
release profile是预定义的
可以自定义 使用不同的配置对代码编译有更多控制
每个profile的配置都独立于其他的profile

dev profile 适用于开发  cargo build
release profile 适用于发布 cargo build-release
*/

/*发布crate之前需要在cargo.toml的[package]为crate添加一些元数据
cargo 需要唯一的名称name
description 会出现在crate搜索的结果
license 需要提供许可证标识值，可以到http://spdx.org/licenses/ 里面查找
可以指定多个license 用OR隔开
version
author
发布用cargo publish
*/