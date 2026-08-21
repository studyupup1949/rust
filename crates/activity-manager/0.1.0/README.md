# Activity Manager

[![Crates.io](https://img.shields.io/crates/v/activity-manager.svg)](https://crates.io/crates/activity-manager)
[![Documentation](https://docs.rs/activity-manager/badge.svg)](https://docs.rs/activity-manager)
[![License](https://img.shields.io/crates/l/activity-manager.svg)](https://github.com/paulzhang5511/activity-manager)

**Activity Manager** is a UI-agnostic, Android-style activity and routing stack manager for Rust. 
它是一个无 UI 框架依赖的通用 Android 风格页面（Activity）与路由堆栈管理框架。

本框架通过 `ActivityHost` 抽象了底层的 UI 渲染和事件循环机制，使得核心的路由调度和生命周期管理逻辑可以独立于特定的 UI 框架（如 Iced）运行。

## 🌟 核心特性 (Core Features)

* **高度解耦 (UI-Agnostic)**: 完全剥离具体 UI 框架依赖，通过 `ActivityHost` Trait 适配任意 UI 库。
* **生命周期管理 (Lifecycle)**: 提供 `on_create`, `on_resume`, `on_pause`, `on_destroy`, `on_new_intent` 等完整的生命周期流转。
* **经典启动模式 (Launch Modes)**: 完美还原 Android 的四种经典启动模式：
  * `Standard`: 标准新建模式
  * `SingleTop`: 栈顶复用模式
  * `SingleTask`: 栈内复用模式 (Clear Top)
  * `SingleInstance`: 全局单例独占模式
* **半透明叠加渲染 (Painter's Algorithm)**: 原生支持 Dialog 风格的半透明页面叠加，智能阻断底层被遮挡页面的无效渲染与无效事件订阅。
* **依赖注入 (DI)**: 在 Activity 创建时通过工厂闭包自动注入全局 Context。

## 📦 安装 (Installation)

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
activity-manager = "0.1.0"
````

或者使用命令快速添加：

```bash
cargo add activity-manager
```

## 📖 完整使用指南 (Usage Guide)

Activity Manager 的核心思想是\*\*“托管”\*\*。你需要将它嵌入到你所使用的具体 UI 框架（如 Iced 等）的主状态机中。以下演示如何在一个典型的应用生命周期中集成并使用本框架。

### 1\. 定义你的环境：Host 与 Route

首先，告诉框架你所使用的 UI 底层类型（Host）以及你的页面路由表（Route）。

```rust
use activity_manager::{ActivityHost, LaunchMode, Route};

// 1. 定义 Host (以假设的某个 UI 框架为例)
pub struct AppHost;
impl ActivityHost for AppHost {
    type View = String;            // 你的 UI 框架的视图类型 (如 iced::Element)
    type Effect = ();              // 你的 UI 框架的异步任务类型 (如 iced::Task)
    type Subscription = ();        // 订阅类型 (如 iced::Subscription)
    type Message = AppMessage;     // 全局消息枚举
}

// 2. 定义全局消息
#[derive(Debug, Clone)]
pub enum AppMessage {
    GoToSettings,
    GoBack,
    UserClickedButton,
}

// 3. 定义路由与启动模式
#[derive(Debug, Clone, PartialEq)]
pub enum AppRoute {
    Home,
    Settings,
}
impl Route for AppRoute {
    fn launch_mode(&self) -> LaunchMode {
        match self {
            AppRoute::Home => LaunchMode::SingleTask,
            AppRoute::Settings => LaunchMode::Standard,
        }
    }
    fn is_translucent(&self) -> bool { false }
}
```

### 2\. 编写你的 Activity (页面)

每个页面都是一个独立的状态机，只需实现 `Activity` 接口：

```rust
use activity_manager::{Activity, Intent};

// 共享的全局状态 (Context)
#[derive(Clone)]
pub struct AppContext {
    pub user_token: String,
}

// 首页 Activity
pub struct HomeActivity;
impl Activity<AppRoute, AppHost, AppContext> for HomeActivity {
    fn route(&self) -> AppRoute { AppRoute::Home }

    // 处理当前页面的逻辑
    fn update(&mut self, message: AppMessage) -> Vec<()> {
        match message {
            AppMessage::UserClickedButton => {
                println!("Button clicked on Home!");
                vec![] // 返回 UI Effect / Task
            }
            _ => vec![]
        }
    }

    // 渲染当前页面的 UI
    fn view(&self) -> String {
        "<h1>Welcome Home</h1>".to_string()
    }
    
    // 可选：实现 on_create, on_resume, on_pause 等生命周期钩子
}
```

### 3\. 在应用主循环中集成 Manager

这是最关键的一步。你需要将 `ActivityManager` 存放在你应用的最顶层状态中，并将应用的 `update` 和 `view` 委托给 Manager。

```rust
use activity_manager::{ActivityManager, Intent};

// 你的应用根状态
pub struct MyApplication {
    manager: ActivityManager<AppRoute, AppHost, AppContext>,
}

impl MyApplication {
    /// 应用初始化
    pub fn new() -> Self {
        let context = AppContext { user_token: "123".to_string() };
        
        // 定义依赖注入工厂：根据 Route 创建对应的 Activity 实例
        let factory = Box::new(|route: &AppRoute, _ctx: &AppContext| -> Box<dyn Activity<AppRoute, AppHost, AppContext>> {
            match route {
                AppRoute::Home => Box::new(HomeActivity),
                AppRoute::Settings => /* Box::new(SettingsActivity) */ unimplemented!(),
            }
        });

        // 启动初始页面 (Home)
        let (manager, _initial_effects) = ActivityManager::new(AppRoute::Home, context, factory);

        Self { manager }
    }

    /// 接管来自 UI 框架的消息流转
    pub fn update(&mut self, message: AppMessage) -> Vec<()> {
        let mut effects = Vec::new();

        // 1. 处理全局级别的路由跳转
        match message {
            AppMessage::GoToSettings => {
                let intent = Intent::new(AppRoute::Settings);
                effects.extend(self.manager.start_activity(intent));
                return effects; // 发生路由跳转时，拦截消息不再向下传递
            }
            AppMessage::GoBack => {
                effects.extend(self.manager.back());
                return effects;
            }
            _ => {}
        }

        // 2. 将非路由消息派发给当前处于栈顶的活跃 Activity 去处理
        effects.extend(self.manager.update(message));
        
        effects
    }

    /// 接管视图渲染
    pub fn view(&self) -> String {
        // manager.views() 会返回当前需要渲染的页面层级（已处理半透明与遮挡逻辑）。
        // 你需要使用你的 UI 框架的容器（如 Stack / ZStack）将它们从底到顶叠放起来。
        let active_views: Vec<String> = self.manager.views();
        
        // 伪代码：将多个视图层叠渲染
        format!("<Stack>\n{}\n</Stack>", active_views.join("\n"))
    }
}
```

### 💡 核心工作流总结

1.  **渲染流**：UI 框架请求视图 -\> `MyApplication::view` -\> `manager.views()` -\> 获取活动页面的 `Activity::view` 组合并返回。
2.  **事件流**：用户交互 -\> UI 框架触发 `AppMessage` -\> `MyApplication::update` -\> 拦截路由消息或透传给 `manager.update()` -\> 栈顶的 `Activity::update` 接收消息并更新自身状态。

## 📄 协议 (License)

[MIT](https://www.google.com/search?q=LICENSE-MIT) 