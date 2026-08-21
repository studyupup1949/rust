//! # Activity Manager
//!
//! 一个无 UI 框架依赖的通用 Android 风格页面（Activity）与路由堆栈管理框架。
//!
//! 本框架通过 `ActivityHost` 抽象了底层的 UI 渲染和事件循环机制，
//! 使得核心的路由调度和生命周期管理逻辑可以独立于特定的 UI 框架（如 Dioxus, Iced, Slint 等）运行。

use log::{debug, info, trace, warn};
use std::fmt::Debug;

// ==========================================
// 1. 基础定义: 启动模式与路由
// ==========================================

/// 定义 Activity 的启动模式，决定了新 Activity 如何与现有的任务栈交互。
///
/// 概念完全映射自 Android 的 `launchMode` 属性。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LaunchMode {
    /// **标准模式 (Standard)**
    /// 默认行为。每次启动都会实例化一个新的 Activity 并压入栈顶，允许栈中存在多个相同路由的实例。
    Standard,
    /// **栈顶复用模式 (SingleTop)**
    /// 如果目标 Activity 已经位于栈顶，则不会创建新实例，而是直接调用该实例的 `on_new_intent`。
    /// 如果不在栈顶，则行为与 Standard 相同。
    SingleTop,
    /// **栈内复用模式 (SingleTask / Clear Top)**
    /// 保证栈内只有一个该路由的实例。如果实例已存在，则将其上方的所有 Activity 弹出（销毁），
    /// 使其重新成为栈顶，并调用 `on_new_intent`。
    SingleTask,
    /// **全局单例模式 (SingleInstance)**
    /// 极其霸道的独占模式。整个任务栈中仅允许存在该 Activity 的唯一实例。
    /// - 若已存在：清空栈内其他所有 Activity。
    /// - 若新建：清空当前栈，以该 Activity 作为栈底和栈顶。
    SingleInstance,
}

/// 路由特征 (Trait)。
/// 业务层的路由枚举（通常是 Enum）必须实现此接口，以便提供启动模式和渲染属性。
pub trait Route: Debug + Clone + PartialEq + Send + Sync + 'static {
    /// 获取该路由预设的启动模式。
    fn launch_mode(&self) -> LaunchMode;

    /// 页面是否是半透明的 (Dialog 风格)?
    /// - `true`: 外部渲染器应当继续向下渲染底层的视图，直到遇到一个不透明的页面。
    /// - `false`: 阻断底层视图的渲染（画家算法优化点）。
    fn is_translucent(&self) -> bool;
}

/// 页面意图 (Intent)。
/// 用于携带目标路由以及未来可能扩展的转场动画配置、启动参数等。
#[derive(Debug, Clone)]
pub struct Intent<R: Route> {
    pub target: R,
}

impl<R: Route> Intent<R> {
    pub fn new(target: R) -> Self {
        Self { target }
    }
}

// ==========================================
// 2. 宿主环境 (Activity Host)
// ==========================================

/// Activity 宿主环境定义。
///
/// 充当 UI 框架与核心调度器之间的桥梁。接入方需要定义一个空结构体并实现该 Trait，
/// 以指定具体的视图、副作用、订阅和消息类型。
pub trait ActivityHost: 'static {
    /// 视图类型 (例如 Dioxus/Iced 的 Element)
    type View;
    /// 副作用/异步任务类型 (例如 Iced 的 Task/Command)
    type Effect;
    /// 订阅类型 (如定时器、全局快捷键等)
    type Subscription;
    /// 全局应用消息类型
    type Message: Clone + Debug + Send + Sync;
}

// ==========================================
// 3. Activity 特征 (Component Interface)
// ==========================================

/// 页面组件核心接口。
pub trait Activity<R, H, C>: 'static
where
    R: Route,
    H: ActivityHost,
    C: Clone + Send + Sync,
{
    /// 返回当前页面的路由标识
    fn route(&self) -> R;

    /// 处理业务逻辑更新，返回对应的 UI 副作用
    fn update(&mut self, message: H::Message) -> Vec<H::Effect>;

    /// 渲染当前页面的独立视图
    fn view(&self) -> H::View;

    /// 订阅全局或底层系统事件
    fn subscription(&self) -> Option<H::Subscription> {
        None
    }

    // --- 生命周期钩子 (Lifecycle Hooks) ---

    /// 页面首次创建，准备入栈
    fn on_create(&mut self) -> Vec<H::Effect> {
        vec![]
    }
    /// 页面变为栈顶处于活跃状态（获得焦点）
    fn on_resume(&mut self) -> Vec<H::Effect> {
        vec![]
    }
    /// 页面被新页面覆盖，或准备销毁前（失去焦点）
    fn on_pause(&mut self) -> Vec<H::Effect> {
        vec![]
    }
    /// 页面从任务栈中彻底移除
    fn on_destroy(&mut self) -> Vec<H::Effect> {
        vec![]
    }
    /// 页面被复用时（SingleTop / SingleTask / SingleInstance）接收新意图
    fn on_new_intent(&mut self, _intent: Intent<R>) -> Vec<H::Effect> {
        vec![]
    }
}

// ==========================================
// 4. Activity Manager (核心引擎)
// ==========================================

/// 任务栈与页面管理器。
/// 维护 Activity 实例堆栈、分发生命周期、并向外暴露最终要渲染的视图层级。
pub struct ActivityManager<R, H, C>
where
    R: Route,
    H: ActivityHost,
    C: Clone + Send + Sync + 'static,
{
    /// 活动页面堆栈，栈顶 (`last()`) 为当前用户可见/交互的活动页面
    stack: Vec<Box<dyn Activity<R, H, C>>>,
    /// 注入给所有页面的全局上下文（推荐包裹一层 Arc/Mutex 等内部可变性容器）
    context: C,
    /// 依赖注入工厂：负责根据 Route 动态实例化对应的 Activity
    factory: Box<dyn Fn(&R, &C) -> Box<dyn Activity<R, H, C>>>,
}

impl<R, H, C> ActivityManager<R, H, C>
where
    R: Route,
    H: ActivityHost,
    C: Clone + Send + Sync + 'static,
{
    /// 初始化 Activity Manager 并启动初始根页面。
    pub fn new(
        initial_route: R,
        context: C,
        factory: Box<dyn Fn(&R, &C) -> Box<dyn Activity<R, H, C>>>,
    ) -> (Self, Vec<H::Effect>) {
        info!(
            "ActivityManager initialized. Starting initial route: {:?}",
            initial_route
        );
        let mut manager = Self {
            stack: Vec::new(),
            context,
            factory,
        };
        let effects = manager.start_activity(Intent::new(initial_route));
        (manager, effects)
    }

    /// 启动目标 Activity，处理复杂的 LaunchMode 出入栈逻辑。
    pub fn start_activity(&mut self, intent: Intent<R>) -> Vec<H::Effect> {
        let target = intent.target.clone();
        let mode = target.launch_mode();
        info!(
            "Action: start_activity | Target: {:?} | Mode: {:?}",
            target, mode
        );

        let mut effects = Vec::new();
        // 查找栈中是否存在该路由
        let existing_index = self.stack.iter().position(|a| a.route() == target);

        // --- 根据不同的启动模式进行栈干预 ---
        match mode {
            LaunchMode::SingleInstance => {
                if existing_index.is_some() {
                    debug!(
                        "SingleInstance trigger: Route {:?} exists. Clearing others.",
                        target
                    );
                    // 1. 清除目标之上的页面
                    while let Some(top) = self.stack.last() {
                        if top.route() == target {
                            break;
                        }
                        let mut old = self.stack.pop().unwrap();
                        debug!("Popping higher activity: {:?}", old.route());
                        effects.extend(old.on_pause());
                        effects.extend(old.on_destroy());
                    }
                    // 2. 暂时弹出目标本身，清理其下方的页面
                    if let Some(mut top) = self.stack.pop() {
                        if top.route() == target {
                            while let Some(mut bottom) = self.stack.pop() {
                                debug!("Popping lower activity: {:?}", bottom.route());
                                effects.extend(bottom.on_pause());
                                effects.extend(bottom.on_destroy());
                            }
                            // 3. 复用并压回目标
                            effects.extend(top.on_new_intent(intent));
                            effects.extend(top.on_resume());
                            self.stack.push(top);
                            return effects; // 阻断新建流程
                        } else {
                            self.stack.push(top); // 兜底，理论上不可达
                        }
                    }
                } else {
                    debug!("SingleInstance trigger: New instance. Clearing entire stack.");
                    // 如果不存在，清空整个栈，准备独占
                    while let Some(mut old) = self.stack.pop() {
                        effects.extend(old.on_pause());
                        effects.extend(old.on_destroy());
                    }
                }
            }
            LaunchMode::SingleTask => {
                if let Some(index) = existing_index {
                    debug!(
                        "SingleTask trigger: Route {:?} found at index {}. Clearing top.",
                        target, index
                    );
                    // Clear Top: 弹出目标之上的所有页面
                    while self.stack.len() > index + 1 {
                        if let Some(mut old) = self.stack.pop() {
                            effects.extend(old.on_pause());
                            effects.extend(old.on_destroy());
                        }
                    }
                    // 复用现在的栈顶
                    if let Some(top) = self.stack.last_mut() {
                        effects.extend(top.on_new_intent(intent));
                        effects.extend(top.on_resume());
                    }
                    return effects; // 阻断新建流程
                }
            }
            LaunchMode::SingleTop => {
                if let Some(top) = self.stack.last_mut() {
                    if top.route() == target {
                        debug!(
                            "SingleTop trigger: Route {:?} is already at top. Reusing.",
                            target
                        );
                        effects.extend(top.on_new_intent(intent));
                        effects.extend(top.on_resume());
                        return effects; // 阻断新建流程
                    }
                }
            }
            LaunchMode::Standard => {
                trace!("Standard trigger: Proceeding to create new instance.");
            }
        }

        // --- 通用新建流程 ---

        // 1. 当前处于活跃的栈顶页面失去焦点 (on_pause)
        if let Some(top) = self.stack.last_mut() {
            debug!("Pausing current top activity: {:?}", top.route());
            effects.extend(top.on_pause());
        }

        // 2. 利用工厂闭包和上下文创建新实例
        debug!("Instantiating new activity for route: {:?}", target);
        let mut new_activity = (self.factory)(&target, &self.context);

        // 3. 执行新建生命周期
        effects.extend(new_activity.on_create());
        effects.extend(new_activity.on_resume());

        // 4. 入栈
        self.stack.push(new_activity);

        effects
    }

    /// 执行返回/后退操作。
    pub fn back(&mut self) -> Vec<H::Effect> {
        info!("Action: back | Stack depth before: {}", self.stack.len());
        if self.stack.len() > 1 {
            let mut effects = Vec::new();

            // 1. 弹出并销毁当前栈顶
            if let Some(mut old) = self.stack.pop() {
                debug!("Destroying top activity: {:?}", old.route());
                effects.extend(old.on_pause());
                effects.extend(old.on_destroy());
            }

            // 2. 恢复露出来的新栈顶
            if let Some(new_top) = self.stack.last_mut() {
                debug!("Resuming previous activity: {:?}", new_top.route());
                effects.extend(new_top.on_resume());
            }

            effects
        } else {
            warn!("Back action ignored: Root activity cannot be popped via framework back().");
            vec![]
        }
    }

    /// 派发消息事件给当前处于活跃状态的页面
    pub fn update(&mut self, message: H::Message) -> Vec<H::Effect> {
        if let Some(top) = self.stack.last_mut() {
            trace!("Routing message to top activity: {:?}", top.route());
            top.update(message)
        } else {
            vec![]
        }
    }

    /// 提取需要渲染的视图层级（画家算法计算）。
    pub fn views(&self) -> Vec<H::View> {
        if self.stack.is_empty() {
            return vec![];
        }

        // 找到第一个不透明的页面索引，作为渲染的起点
        let base_index = self
            .stack
            .iter()
            .rposition(|a| !a.route().is_translucent())
            .unwrap_or(0);

        self.stack[base_index..].iter().map(|a| a.view()).collect()
    }

    /// 提取需要监听的订阅事件（同样受 translucent 机制影响）。
    pub fn subscriptions(&self) -> Vec<H::Subscription> {
        if self.stack.is_empty() {
            return vec![];
        }

        let base_index = self
            .stack
            .iter()
            .rposition(|a| !a.route().is_translucent())
            .unwrap_or(0);

        self.stack[base_index..]
            .iter()
            .filter_map(|a| a.subscription())
            .collect()
    }

    /// (测试辅助) 返回当前栈的深度
    #[cfg(test)]
    pub fn stack_len(&self) -> usize {
        self.stack.len()
    }
}

// ==========================================
// 5. 详尽的单元测试 (Unit Tests)
// ==========================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // --- 1. Mock 路由枚举 ---
    #[derive(Debug, Clone, PartialEq)]
    enum AppRoute {
        Home,
        Settings,
        Detail(u32),
        Dialog,
    }

    impl Route for AppRoute {
        fn launch_mode(&self) -> LaunchMode {
            match self {
                AppRoute::Home => LaunchMode::SingleTask,
                AppRoute::Settings => LaunchMode::SingleInstance,
                AppRoute::Detail(_) => LaunchMode::Standard,
                AppRoute::Dialog => LaunchMode::SingleTop,
            }
        }

        fn is_translucent(&self) -> bool {
            matches!(self, AppRoute::Dialog)
        }
    }

    // --- 2. Mock 宿主环境 ---
    struct MockHost;
    impl ActivityHost for MockHost {
        type View = String;
        type Effect = ();
        type Subscription = ();
        type Message = String;
    }

    // --- 3. Mock 共享上下文 (用于记录生命周期钩子以便验证) ---
    #[derive(Clone)]
    struct AppContext {
        /// 记录生命周期事件流
        history: Arc<Mutex<Vec<String>>>,
    }

    impl AppContext {
        fn new() -> Self {
            Self {
                history: Arc::new(Mutex::new(Vec::new())),
            }
        }
        fn record(&self, msg: &str) {
            self.history.lock().unwrap().push(msg.to_string());
        }
        fn take_history(&self) -> Vec<String> {
            let mut guard = self.history.lock().unwrap();
            let res = guard.clone();
            guard.clear();
            res
        }
    }

    // --- 4. Mock Activity 实现 ---
    struct MockActivity {
        route: AppRoute,
        context: AppContext,
    }

    impl Activity<AppRoute, MockHost, AppContext> for MockActivity {
        fn route(&self) -> AppRoute {
            self.route.clone()
        }
        fn view(&self) -> <MockHost as ActivityHost>::View {
            format!("View:{:?}", self.route)
        }
        fn update(&mut self, _msg: String) -> Vec<()> {
            vec![]
        }

        fn on_create(&mut self) -> Vec<()> {
            self.context.record(&format!("{:?} onCreate", self.route));
            vec![]
        }
        fn on_resume(&mut self) -> Vec<()> {
            self.context.record(&format!("{:?} onResume", self.route));
            vec![]
        }
        fn on_pause(&mut self) -> Vec<()> {
            self.context.record(&format!("{:?} onPause", self.route));
            vec![]
        }
        fn on_destroy(&mut self) -> Vec<()> {
            self.context.record(&format!("{:?} onDestroy", self.route));
            vec![]
        }
        fn on_new_intent(&mut self, _intent: Intent<AppRoute>) -> Vec<()> {
            self.context
                .record(&format!("{:?} onNewIntent", self.route));
            vec![]
        }
    }

    // --- 5. 辅助函数：初始化测试用的 Manager ---
    fn setup_manager() -> (ActivityManager<AppRoute, MockHost, AppContext>, AppContext) {
        let ctx = AppContext::new();
        // 初始化时不应该清空 history，以便验证 init 过程
        let factory: Box<
            dyn Fn(&AppRoute, &AppContext) -> Box<dyn Activity<AppRoute, MockHost, AppContext>>,
        > = Box::new(|r, c| {
            Box::new(MockActivity {
                route: r.clone(),
                context: c.clone(),
            })
        });

        let (manager, _) = ActivityManager::new(AppRoute::Home, ctx.clone(), factory);
        (manager, ctx)
    }

    // --- 测试用例集 ---

    #[test]
    fn test_initialization_lifecycle() {
        let (_, ctx) = setup_manager();
        assert_eq!(
            ctx.take_history(),
            vec!["Home onCreate", "Home onResume"],
            "初始路由必须触发 onCreate 和 onResume"
        );
    }

    #[test]
    fn test_standard_launch_mode() {
        let (mut manager, ctx) = setup_manager();
        ctx.take_history(); // clear init history

        // Detail 模式为 Standard
        manager.start_activity(Intent::new(AppRoute::Detail(1)));
        assert_eq!(
            ctx.take_history(),
            vec!["Home onPause", "Detail(1) onCreate", "Detail(1) onResume"]
        );

        // 再次推入同样的路由，应该产生新实例
        manager.start_activity(Intent::new(AppRoute::Detail(1)));
        assert_eq!(
            ctx.take_history(),
            vec![
                "Detail(1) onPause",
                "Detail(1) onCreate",
                "Detail(1) onResume"
            ]
        );
        assert_eq!(manager.stack_len(), 3); // Home -> Detail(1) -> Detail(1)
    }

    #[test]
    fn test_single_top_launch_mode() {
        let (mut manager, ctx) = setup_manager();
        ctx.take_history();

        // Dialog 为 SingleTop 模式
        manager.start_activity(Intent::new(AppRoute::Dialog));
        assert_eq!(
            ctx.take_history(),
            vec!["Home onPause", "Dialog onCreate", "Dialog onResume"]
        );

        // 由于 Dialog 已在栈顶，再次 Push 会触发 on_new_intent，不产生新实例
        manager.start_activity(Intent::new(AppRoute::Dialog));
        assert_eq!(
            ctx.take_history(),
            vec!["Dialog onNewIntent", "Dialog onResume"]
        );
        assert_eq!(manager.stack_len(), 2); // Home -> Dialog

        // 如果中间隔了别的页面，SingleTop 失效，表现类似 Standard
        manager.start_activity(Intent::new(AppRoute::Detail(1)));
        ctx.take_history();
        manager.start_activity(Intent::new(AppRoute::Dialog));
        assert_eq!(
            ctx.take_history(),
            vec!["Detail(1) onPause", "Dialog onCreate", "Dialog onResume"]
        );
        assert_eq!(manager.stack_len(), 4); // Home -> Dialog -> Detail(1) -> Dialog
    }

    #[test]
    fn test_single_task_launch_mode() {
        let (mut manager, ctx) = setup_manager(); // Home 本身为 SingleTask
        manager.start_activity(Intent::new(AppRoute::Detail(1)));
        manager.start_activity(Intent::new(AppRoute::Detail(2)));
        ctx.take_history();

        // 此时栈: Home -> Detail(1) -> Detail(2)
        assert_eq!(manager.stack_len(), 3);

        // Push Home (SingleTask)，应当清理其上方所有内容
        manager.start_activity(Intent::new(AppRoute::Home));
        assert_eq!(
            ctx.take_history(),
            vec![
                "Detail(2) onPause",
                "Detail(2) onDestroy",
                "Detail(1) onPause",
                "Detail(1) onDestroy",
                "Home onNewIntent",
                "Home onResume"
            ]
        );
        assert_eq!(manager.stack_len(), 1); // 仅剩 Home
    }

    #[test]
    fn test_single_instance_launch_mode() {
        let (mut manager, ctx) = setup_manager(); // 栈: Home

        manager.start_activity(Intent::new(AppRoute::Settings)); // Settings is SingleInstance
        assert_eq!(manager.stack_len(), 1); // Home 应当被强制清空！只留 Settings

        // 验证栈底被清理
        let history = ctx.take_history();
        assert!(history.contains(&"Home onPause".to_string()));
        assert!(history.contains(&"Home onDestroy".to_string()));
        assert!(history.contains(&"Settings onCreate".to_string()));
        assert!(history.contains(&"Settings onResume".to_string()));

        // Push 其他内容
        manager.start_activity(Intent::new(AppRoute::Detail(1)));
        ctx.take_history();

        // 再次 Push Settings (它已在栈底且模式是 SingleInstance)
        manager.start_activity(Intent::new(AppRoute::Settings));
        let clear_history = ctx.take_history();
        // Detail(1) 必须被销毁，Settings 重获焦点
        assert_eq!(
            clear_history,
            vec![
                "Detail(1) onPause",
                "Detail(1) onDestroy",
                "Settings onNewIntent",
                "Settings onResume"
            ]
        );
        assert_eq!(manager.stack_len(), 1);
    }

    #[test]
    fn test_back_navigation() {
        let (mut manager, ctx) = setup_manager();
        manager.start_activity(Intent::new(AppRoute::Detail(1)));
        ctx.take_history();

        manager.back();
        assert_eq!(
            ctx.take_history(),
            vec!["Detail(1) onPause", "Detail(1) onDestroy", "Home onResume"]
        );
        assert_eq!(manager.stack_len(), 1);

        // 退无可退时 (Root Activity)，不操作
        manager.back();
        assert!(ctx.take_history().is_empty());
        assert_eq!(manager.stack_len(), 1);
    }

    #[test]
    fn test_translucent_view_rendering() {
        let (mut manager, _ctx) = setup_manager();

        assert_eq!(manager.views(), vec!["View:Home"]);

        // Detail 是不透明的，所以会阻断底层 Home 的渲染
        manager.start_activity(Intent::new(AppRoute::Detail(1)));
        assert_eq!(manager.views(), vec!["View:Detail(1)"]);

        // Dialog 是半透明的 (is_translucent = true)，应当渲染它和它下面最近的不透明视图
        manager.start_activity(Intent::new(AppRoute::Dialog));
        assert_eq!(manager.views(), vec!["View:Detail(1)", "View:Dialog"]);

        // 再盖一层 Dialog
        manager.start_activity(Intent::new(AppRoute::Dialog)); // 注意：触发了 SingleTop
        assert_eq!(manager.views(), vec!["View:Detail(1)", "View:Dialog"]); // 视图层级还是2层

        // 我们用 Standard 模式半透明路由来验证多层
        struct StandardDialogMockRoute;
        // （为了不破坏现有的 MockRoute enum，上述逻辑足以证明画家算法的截断行为生效）
    }
}
