# Activity Manager

[![Crates.io](https://img.shields.io/crates/v/activity-manager.svg)](https://crates.io/crates/activity-manager)
[![Documentation](https://docs.rs/activity-manager/badge.svg)](https://docs.rs/activity-manager)

**Activity Manager** 是一个为 Rust 设计的、无 UI 框架依赖的通用 Android 风格页面（Activity）与路由堆栈管理框架。

它通过抽象底层 UI 渲染和事件循环，使得核心的路由调度逻辑可以独立运行。特别地，它利用 **GAT (Generic Associated Types)** 和 **任务聚合 (Task Batching)** 深度优化了对现代纯函数式 UI 框架（如 **Iced**、Dioxus 等）的支持，完美解决了视图生命周期绑定和异步副作用管理的痛点。

## ✨ 核心特性

* 🤖 **Android 风格的启动模式**：内置完整、严格的四种路由堆栈调度模式：
  * `Standard` (标准模式)
  * `SingleTop` (栈顶复用)
  * `SingleTask` (栈内复用，Clear Top)
  * `SingleInstance` (全局单例)
* 🎨 **画家算法 (Painter's Algorithm) 渲染层**：原生支持半透明弹窗页面 (Dialog)，在渲染时自动向下探查，阻断底层被完全遮挡页面的无效渲染构建。
* 🔄 **GAT 生命周期零拷贝**：彻底告别为了满足 UI 框架生命周期而四处 `.clone()` 的噩梦。允许页面的视图构建函数 (`view`) 直接以引用的方式读取 Activity 状态。
* ⚡ **异步任务聚合器**：通过统一的 `Task` 单元元，将页面跳转过程中触发的多个生命周期钩子（如旧页面的销毁、新页面的创建等）产生的异步副作用完美打包合并。

## 📦 安装

在你的 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
activity-manager = "0.1.1"
```

> **注意**：`activity-manager` 核心库本身极其轻量，仅依赖 `log` 库，不对你的项目强加任何具体的 UI 引擎。

## 🚀 快速体验 (Examples)

框架内置了基于 **Iced 0.14** 的完整应用示例，展示了路由跳转、生命周期任务聚合以及半透明弹窗的实现。

在克隆本仓库后，直接在根目录下运行：

```bash
RUST_LOG=info cargo run --example iced_demo
```
*(设置 `RUST_LOG=info` 可以在终端中实时观察页面生命周期的变化规律)*

## 📖 核心架构与使用指南

要在你的 UI 项目中接入 `Activity Manager`，只需遵循以下三个步骤：

### 1. 定义宿主环境 (`ActivityHost`)
你需要告诉框架，你的应用使用的是什么消息 (`Message`)、什么上下文 (`Context`)、以及你的 UI 引擎如何描述视图 (`View`) 和异步任务 (`Task`)。

```rust
use activity_manager::{ActivityHost, Route, LaunchMode};
use iced::{Element, Task};

// 定义全局状态
pub struct GlobalContext {
    pub user_name: String,
}

// 定义路由
#[derive(Debug, Clone, PartialEq)]
pub enum AppRoute {
    Home,
    Detail(String),
}
impl Route for AppRoute {
    fn launch_mode(&self) -> LaunchMode { LaunchMode::Standard }
    fn is_translucent(&self) -> bool { false }
}

// 实现宿主桥接
pub struct IcedHost;
impl ActivityHost for IcedHost {
    type Message = ();
    type Context = GlobalContext;
    type Route = AppRoute;
    
    // GAT 魔法：绑定生命周期，实现零拷贝渲染
    type View<'a> = Element<'a, ()>; 
    type Task = Task<()>;

    fn empty_task() -> Self::Task { Task::none() }
    fn batch_tasks(tasks: Vec<Self::Task>) -> Self::Task { Task::batch(tasks) }
}
```

### 2. 编写页面组件 (`Activity`)
实现 `Activity` trait，你可以获得类似 Android 的完整生命周期钩子（默认均返回空任务，按需重写即可）。

```rust
use activity_manager::Activity;
use iced::widget::text;

pub struct HomeActivity {
    welcome_text: String,
}

impl Activity<IcedHost> for HomeActivity {
    fn route(&self) -> AppRoute { AppRoute::Home }

    // 生命周期钩子：创建时可派发网络请求等 Task
    fn on_create(&mut self, _ctx: &mut GlobalContext) -> iced::Task<()> {
        log::info!("首页创建！");
        iced::Task::none()
    }

    // 视图渲染：直接借用 self 和 ctx 的数据，无需 clone
    fn view<'a>(&'a self, ctx: &'a GlobalContext) -> Element<'a, ()> {
        text(format!("{}，当前用户：{}", self.welcome_text, ctx.user_name)).into()
    }
}
```

### 3. 驱动管理器 (`ActivityManager`)
在你的主应用循环中，实例化 `ActivityManager` 并将 UI 的 `update` 和 `view` 逻辑委托给它。

```rust
use activity_manager::ActivityManager;
use iced::widget::stack;

struct MyApp {
    manager: ActivityManager<IcedHost>,
    context: GlobalContext,
}

impl MyApp {
    // Iced 的 update 循环
    fn update(&mut self, message: ()) -> iced::Task<()> {
        // 委托分发消息或执行页面跳转
        self.manager.update(&mut self.context, message)
    }

    // Iced 的 view 循环
    fn view(&self) -> Element<()> {
        // Manager 会自动利用画家算法计算出可见的页面列表
        let views = self.manager.views(&self.context);
        
        // 利用 Iced 的 stack 控件将页面层叠
        stack(views).into()
    }
}
```

## 🧠 生命周期行为参考 (Lifecycle)

框架在页面跳转时会自动调度以下钩子函数，并收集其副作用（Task）：

* `on_create`: 页面实例被创建入栈。
* `on_resume`: 页面进入前台，获取焦点。
* `on_pause`: 页面被新页面覆盖，或即将出栈销毁。
* `on_destroy`: 页面被彻底移出堆栈。
* `on_new_intent`: 命中 `SingleTop` 或 `SingleTask` 时触发复用逻辑。

## 📄 许可证 (License)

本项目采用 [MIT License](LICENSE) 开源许可协议。