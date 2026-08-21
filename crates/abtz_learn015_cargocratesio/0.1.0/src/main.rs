// 通过 release profile 自定义构建
// 如何在 https://crates.io 上发布
// 通过 workspaces 组织大工程
// 从 https://crates.io 上安装库
// 使用自定义命令扩展 cargo

// release profile 叫做 发布配置
// 预定义
// 可自定义
// 每个 profile 都独立于其它的 profile
// 默认有 dev、release 两个 profile
// cargo build --release 执行 release 配置
// cargo build 用的是 dev profile

// 自定义 profile
// 针对每一个profile 都提供了默认的配置
// 想自定义 xxx profile 的配置
// 在 Cargo.toml 中添加 [profile.xxx]
// [profile.dev]
// opt-level = 0 // 优化等级 0 1 2 3
// debug = true

// 如何发布 crate 到 https://crates.io/
// cargo publish
// 文档注释 用于生成文档
/// This is a doc comment!
/// 支持 Markdown
/// 方置在被说明条目之前
//
// cargo doc 生成文档
// cargo doc --open 生成文档 并 打开文档
// cargo doc --no-deps 不生成依赖项的文档
// 文档注释常用章节
/// # Examples
/// # Panics
/// # Safety
/// # Errors
/// # Implementation
/// # Assumptions
/// # Examples
/// # References
/// # See Also
/// # Notes
// 文档注释作为测试
// 为包含注释的项添加文档注释
// 符号: //!
// 这类注释通常用描述crate的文档

// pub use
// 使用 pub use 导出方便使用的公共API
// 将内部层级较多的模块的API，统一导出到一个公共模块中 方便 使用者使用 不用 导入多层级内部模块
use abtz_learn015_cargocratesio::add;
use abtz_learn015_cargocratesio::{PrimaryColor, SecondaryColor, mix};


// 发布 crate 到 crates.io
// 创建并设置 Crates.io 账号
// 创建 API Token
// cioHzP6K3WLYNTSyADBB9EKfbllE6F5mRtK
// cargo login cioHzP6K3WLYNTSyADBB9EKfbllE6F5mRtK
// 设置当前项目 Cargo.toml 文件
// name 需要是独一无二的
// description 添加描述 crate的描述
// license 添加许可协议 可到 https://spdx.org/licenses/ 中选择
// 可以指定多个 license 使用 OR
// version 添加版本号
// authors 添加作者
// cargo publish 发布

fn main() {
    println!("Hello, world!");
}
