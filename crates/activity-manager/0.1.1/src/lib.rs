use std::fmt::Debug;

/// 定义路由的特征。
///
/// 路由负责提供页面的启动模式以及是否为半透明页面，这决定了页面的入栈行为和视图渲染逻辑。
pub trait Route: Clone + Debug + PartialEq {
    /// 获取该路由的启动模式。
    fn launch_mode(&self) -> LaunchMode;
    /// 指示该路由对应的页面是否为半透明。
    ///
    /// 如果为 `false`（不透明），渲染器将不会渲染其底部的页面。
    fn is_translucent(&self) -> bool;
}

/// Android 风格的页面启动模式。
///
/// 决定了新页面启动时，如何与当前的页面堆栈进行交互。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchMode {
    /// 标准模式。
    ///
    /// 每次启动该路由，都会创建一个全新的 Activity 实例并压入栈顶。
    Standard,
    /// 栈顶复用模式。
    ///
    /// 如果目标路由已经位于栈顶，则不会创建新实例，而是调用栈顶实例的 `on_new_intent`；
    /// 否则，创建新实例并压入栈顶。
    SingleTop,
    /// 栈内复用模式。
    ///
    /// 检查整个堆栈。如果目标路由已存在于栈内，则将其上方所有的 Activity 弹出并销毁，
    /// 使其成为新的栈顶，并调用其 `on_new_intent`。如果不存在，则创建新实例压入栈顶。
    SingleTask,
    /// 单例模式。
    ///
    /// 堆栈中仅允许存在这唯一一个页面。启动此模式的路由时，会清空栈内的所有其他页面。
    SingleInstance,
}

/// 宿主环境抽象。
///
/// 为框架提供类型映射，使其可以兼容不同的 UI 引擎（如 Iced、Dioxus 等）。
pub trait ActivityHost: Sized + 'static {
    /// 框架全局传递的消息类型。
    type Message: Send + Debug;
    /// 全局上下文类型，通常用于存储全局状态或共享服务。
    type Context;
    /// 业务路由枚举类型。
    type Route: Route;

    /// 视图类型。
    ///
    /// 利用 GATs（通用关联类型）将生命周期 `'a` 绑定到视图上，
    /// 允许视图直接安全地借用 `Activity` 内部的数据（例如 Iced 的 `Element<'a, Message>`）。
    type View<'a>
    where
        Self: 'a;

    /// 异步任务类型。
    ///
    /// 用于封装页面生命周期和交互中产生的副作用（例如 Iced 的 `Task<Message>`）。
    type Task;

    /// 创建一个不执行任何操作的空任务。
    ///
    /// 作为任务系统的“单位元”，用于那些不需要产生副作用的生命周期钩子。
    fn empty_task() -> Self::Task;

    /// 将多个异步任务合并为一个宏任务。
    fn batch_tasks(tasks: Vec<Self::Task>) -> Self::Task;
}

/// 页面组件特征。
///
/// 代表一个拥有完整生命周期、独立状态并能响应全局上下文变化的页面模块。
pub trait Activity<H: ActivityHost>: Send {
    /// 渲染页面视图。
    ///
    /// 生命周期 `'a` 确保返回的视图可以安全地借用 `&self` 或 `&context` 中的数据。
    fn view<'a>(&'a self, context: &'a H::Context) -> H::View<'a>;

    /// 处理业务逻辑更新，并返回可能产生的异步任务。
    fn update(&mut self, _context: &mut H::Context, _message: H::Message) -> H::Task {
        H::empty_task()
    }

    /// 生命周期钩子：页面实例被创建时调用。
    fn on_create(&mut self, _context: &mut H::Context) -> H::Task {
        H::empty_task()
    }

    /// 生命周期钩子：页面进入前台、恢复交互时调用。
    fn on_resume(&mut self, _context: &mut H::Context) -> H::Task {
        H::empty_task()
    }

    /// 生命周期钩子：页面被新页面覆盖，或即将被销毁时调用。
    fn on_pause(&mut self, _context: &mut H::Context) -> H::Task {
        H::empty_task()
    }

    /// 生命周期钩子：页面实例被彻底销毁、移出堆栈前调用。
    fn on_destroy(&mut self, _context: &mut H::Context) -> H::Task {
        H::empty_task()
    }

    /// 生命周期钩子：当页面命中 `SingleTop` 或 `SingleTask` 复用机制时调用。
    fn on_new_intent(&mut self, _context: &mut H::Context, _route: H::Route) -> H::Task {
        H::empty_task()
    }

    /// 获取当前 Activity 对应的路由配置。
    fn route(&self) -> H::Route;
}

/// 页面堆栈与路由调度器。
///
/// 负责管理 Activity 的入栈、出栈，协调生命周期方法的调用，并聚合页面切换产生的异步任务。
pub struct ActivityManager<H: ActivityHost> {
    stack: Vec<Box<dyn Activity<H>>>,
    factory: Box<dyn Fn(&H::Route) -> Box<dyn Activity<H>>>,
}

impl<H: ActivityHost> ActivityManager<H> {
    /// 创建一个新的 ActivityManager。
    ///
    /// `factory` 是一个闭包，负责根据路由枚举实例化对应的 `Activity`。
    pub fn new<F>(factory: F) -> Self
    where
        F: Fn(&H::Route) -> Box<dyn Activity<H>> + 'static,
    {
        log::info!("ActivityManager initialized.");
        Self {
            stack: Vec::new(),
            factory: Box::new(factory),
        }
    }

    /// 将消息分发给当前处于栈顶的活动页面。
    pub fn update(&mut self, context: &mut H::Context, message: H::Message) -> H::Task {
        if let Some(top) = self.stack.last_mut() {
            log::trace!("Dispatching update to top activity: {:?}", message);
            top.update(context, message)
        } else {
            log::warn!("Attempted to update but activity stack is empty.");
            H::empty_task()
        }
    }

    /// 启动一个新页面，处理路由逻辑及堆栈变化，并返回相关的异步任务。
    pub fn start_activity(&mut self, context: &mut H::Context, route: H::Route) -> H::Task {
        let mode = route.launch_mode();
        log::info!(
            "Starting activity with route: {:?}, launch mode: {:?}",
            route,
            mode
        );

        match mode {
            LaunchMode::Standard => self.push_activity(context, route),

            LaunchMode::SingleTop => {
                if let Some(top) = self.stack.last_mut() {
                    if top.route() == route {
                        log::debug!("SingleTop matched. Reusing top activity.");
                        return top.on_new_intent(context, route);
                    }
                }
                log::debug!("SingleTop not matched. Pushing as Standard.");
                self.push_activity(context, route)
            }

            LaunchMode::SingleTask => {
                let mut found_index = None;
                for (i, act) in self.stack.iter().enumerate() {
                    if act.route() == route {
                        found_index = Some(i);
                        break;
                    }
                }

                if let Some(index) = found_index {
                    log::debug!("SingleTask matched at index {}. Clearing top.", index);
                    let mut tasks = Vec::new();

                    // 1. 销毁目标上方所有页面 (Clear Top)
                    while self.stack.len() > index + 1 {
                        tasks.push(self.pop_internal(context));
                    }

                    // 2. 复用目标页面
                    let target = &mut self.stack[index];
                    tasks.push(target.on_new_intent(context, route));
                    tasks.push(target.on_resume(context));

                    H::batch_tasks(tasks)
                } else {
                    log::debug!("SingleTask target not found in stack. Pushing new.");
                    self.push_activity(context, route)
                }
            }

            LaunchMode::SingleInstance => {
                log::debug!("SingleInstance mode. Clearing entire stack.");
                let mut tasks = Vec::new();
                while !self.stack.is_empty() {
                    tasks.push(self.pop_internal(context));
                }
                tasks.push(self.push_activity(context, route));
                H::batch_tasks(tasks)
            }
        }
    }

    /// 模拟物理返回键逻辑，弹出栈顶页面。
    ///
    /// 返回一个元组：
    /// - `bool`: 如果成功返回上一页则为 `true`；如果栈已空或仅剩一页无法返回则为 `false`。
    /// - `H::Task`: 退出动作产生的异步任务。
    pub fn back(&mut self, context: &mut H::Context) -> (bool, H::Task) {
        if self.stack.len() > 1 {
            log::info!("Back navigation triggered. Popping top activity.");
            let mut tasks = Vec::new();
            tasks.push(self.pop_internal(context)); // 销毁当前页

            if let Some(top) = self.stack.last_mut() {
                log::debug!("Resuming the new top activity.");
                tasks.push(top.on_resume(context)); // 恢复前一个页
            }
            (true, H::batch_tasks(tasks))
        } else {
            log::info!("Back navigation ignored. Stack has 1 or fewer activities.");
            (false, H::empty_task())
        }
    }

    /// 执行视图构建，利用“画家算法”收集需要渲染的页面。
    ///
    /// 从栈顶向下查找，直到遇到第一个不透明（`is_translucent() == false`）的页面为止，
    /// 仅收集并返回可见层级的视图。
    pub fn views<'a>(&'a self, context: &'a H::Context) -> Vec<H::View<'a>> {
        log::trace!("Collecting views for rendering.");
        let mut views = Vec::new();
        let mut start_index = 0;

        // 从后往前扫描，寻找渲染截断点
        for (i, act) in self.stack.iter().enumerate().rev() {
            if !act.route().is_translucent() {
                start_index = i;
                log::trace!(
                    "Found opaque activity at index {}. Stopping downward scan.",
                    i
                );
                break;
            }
        }

        for i in start_index..self.stack.len() {
            views.push(self.stack[i].view(context));
        }
        views
    }

    /// 内部方法：执行入栈逻辑并触发响应生命周期
    fn push_activity(&mut self, context: &mut H::Context, route: H::Route) -> H::Task {
        log::debug!("Pushing new activity: {:?}", route);
        let mut tasks = Vec::new();

        if let Some(top) = self.stack.last_mut() {
            tasks.push(top.on_pause(context));
        }

        let mut new_act = (self.factory)(&route);
        tasks.push(new_act.on_create(context));
        tasks.push(new_act.on_resume(context));

        self.stack.push(new_act);
        H::batch_tasks(tasks)
    }

    /// 内部方法：执行出栈逻辑并触发响应生命周期
    fn pop_internal(&mut self, context: &mut H::Context) -> H::Task {
        if let Some(mut act) = self.stack.pop() {
            log::debug!("Popping activity: {:?}", act.route());
            let t1 = act.on_pause(context);
            let t2 = act.on_destroy(context);
            H::batch_tasks(vec![t1, t2])
        } else {
            H::empty_task()
        }
    }

    /// 获取当前栈中的页面数量（仅用于测试或状态监控）
    pub fn stack_len(&self) -> usize {
        self.stack.len()
    }
}

// ============================================================================
// 单元测试模块
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    // 1. 定义 Mock 的路由
    #[derive(Debug, Clone, PartialEq)]
    enum TestRoute {
        StandardPage,
        SingleTopPage,
        SingleTaskPage,
        SingleInstancePage,
        TranslucentPage,
        OpaquePage,
    }

    impl Route for TestRoute {
        fn launch_mode(&self) -> LaunchMode {
            match self {
                TestRoute::StandardPage => LaunchMode::Standard,
                TestRoute::SingleTopPage => LaunchMode::SingleTop,
                TestRoute::SingleTaskPage => LaunchMode::SingleTask,
                TestRoute::SingleInstancePage => LaunchMode::SingleInstance,
                _ => LaunchMode::Standard,
            }
        }

        fn is_translucent(&self) -> bool {
            matches!(self, TestRoute::TranslucentPage)
        }
    }

    // 2. 定义 Mock 的宿主环境
    struct TestHost;

    impl ActivityHost for TestHost {
        type Message = ();
        type Context = Vec<String>; // 使用全局 Vec 记录生命周期调用日志
        type Route = TestRoute;
        type View<'a> = &'a str;
        type Task = Vec<String>; // 任务简化为收集命令字符串

        fn empty_task() -> Self::Task {
            vec![]
        }

        fn batch_tasks(tasks: Vec<Self::Task>) -> Self::Task {
            tasks.into_iter().flatten().collect()
        }
    }

    // 3. 定义 Mock 的页面 Activity
    struct TestActivity {
        route: TestRoute,
        name: String,
    }

    impl TestActivity {
        fn new(route: TestRoute) -> Self {
            let name = format!("{:?}", route);
            Self { route, name }
        }
    }

    impl Activity<TestHost> for TestActivity {
        fn view<'a>(
            &'a self,
            _context: &'a <TestHost as ActivityHost>::Context,
        ) -> <TestHost as ActivityHost>::View<'a> {
            &self.name
        }

        fn on_create(
            &mut self,
            context: &mut <TestHost as ActivityHost>::Context,
        ) -> <TestHost as ActivityHost>::Task {
            context.push(format!("on_create: {}", self.name));
            vec![format!("task_create_{}", self.name)]
        }

        fn on_resume(
            &mut self,
            context: &mut <TestHost as ActivityHost>::Context,
        ) -> <TestHost as ActivityHost>::Task {
            context.push(format!("on_resume: {}", self.name));
            vec![]
        }

        fn on_pause(
            &mut self,
            context: &mut <TestHost as ActivityHost>::Context,
        ) -> <TestHost as ActivityHost>::Task {
            context.push(format!("on_pause: {}", self.name));
            vec![]
        }

        fn on_destroy(
            &mut self,
            context: &mut <TestHost as ActivityHost>::Context,
        ) -> <TestHost as ActivityHost>::Task {
            context.push(format!("on_destroy: {}", self.name));
            vec![format!("task_destroy_{}", self.name)]
        }

        fn on_new_intent(
            &mut self,
            context: &mut <TestHost as ActivityHost>::Context,
            _route: <TestHost as ActivityHost>::Route,
        ) -> <TestHost as ActivityHost>::Task {
            context.push(format!("on_new_intent: {}", self.name));
            vec![format!("task_new_intent_{}", self.name)]
        }

        fn route(&self) -> <TestHost as ActivityHost>::Route {
            self.route.clone()
        }
    }

    // 工厂函数
    fn create_manager() -> ActivityManager<TestHost> {
        ActivityManager::new(|route: &TestRoute| {
            Box::new(TestActivity::new(route.clone())) as Box<dyn Activity<TestHost>>
        })
    }

    #[test]
    fn test_launch_mode_standard() {
        let mut manager = create_manager();
        let mut context = Vec::new();

        // 连续启动两个相同的 Standard 页面
        manager.start_activity(&mut context, TestRoute::StandardPage);
        manager.start_activity(&mut context, TestRoute::StandardPage);

        assert_eq!(manager.stack_len(), 2);
        // 检查生命周期日志：A 创建 -> A 恢复 -> A 暂停 -> A2 创建 -> A2 恢复
        assert_eq!(
            context,
            vec![
                "on_create: StandardPage",
                "on_resume: StandardPage",
                "on_pause: StandardPage",
                "on_create: StandardPage",
                "on_resume: StandardPage"
            ]
        );
    }

    #[test]
    fn test_launch_mode_single_top() {
        let mut manager = create_manager();
        let mut context = Vec::new();

        manager.start_activity(&mut context, TestRoute::StandardPage);
        manager.start_activity(&mut context, TestRoute::SingleTopPage);
        context.clear(); // 清空前面的日志以聚焦复用逻辑

        // 再次启动栈顶相同的 SingleTop 页面
        let task = manager.start_activity(&mut context, TestRoute::SingleTopPage);

        // 栈深度仍然是 2
        assert_eq!(manager.stack_len(), 2);
        // 只触发了 on_new_intent
        assert_eq!(context, vec!["on_new_intent: SingleTopPage"]);
        // 确保相关的 task 被正确聚合返回
        assert_eq!(task, vec!["task_new_intent_SingleTopPage"]);
    }

    #[test]
    fn test_launch_mode_single_task() {
        let mut manager = create_manager();
        let mut context = Vec::new();

        manager.start_activity(&mut context, TestRoute::StandardPage);
        manager.start_activity(&mut context, TestRoute::SingleTaskPage); // target
        manager.start_activity(&mut context, TestRoute::StandardPage); // target top 1
        manager.start_activity(&mut context, TestRoute::StandardPage); // target top 2

        assert_eq!(manager.stack_len(), 4);
        context.clear();

        // 再次启动 SingleTask 页面
        let task = manager.start_activity(&mut context, TestRoute::SingleTaskPage);

        // 栈顶的两个 StandardPage 被销毁，栈深恢复为 2
        assert_eq!(manager.stack_len(), 2);
        assert_eq!(
            context,
            vec![
                "on_pause: StandardPage",        // top 2 pause
                "on_destroy: StandardPage",      // top 2 destroy
                "on_pause: StandardPage",        // top 1 pause
                "on_destroy: StandardPage",      // top 1 destroy
                "on_new_intent: SingleTaskPage", // 目标复用
                "on_resume: SingleTaskPage"
            ]
        );
        // 验证任务批量合并（包含销毁任务和复用任务）
        assert_eq!(
            task,
            vec![
                "task_destroy_StandardPage",
                "task_destroy_StandardPage",
                "task_new_intent_SingleTaskPage"
            ]
        );
    }

    #[test]
    fn test_launch_mode_single_instance() {
        let mut manager = create_manager();
        let mut context = Vec::new();

        manager.start_activity(&mut context, TestRoute::StandardPage);
        manager.start_activity(&mut context, TestRoute::StandardPage);

        // 启动 SingleInstance
        manager.start_activity(&mut context, TestRoute::SingleInstancePage);

        // 栈被清空，仅剩下 SingleInstance
        assert_eq!(manager.stack_len(), 1);
        assert_eq!(manager.stack[0].route(), TestRoute::SingleInstancePage);
    }

    #[test]
    fn test_back_navigation() {
        let mut manager = create_manager();
        let mut context = Vec::new();

        manager.start_activity(&mut context, TestRoute::StandardPage); // root
        manager.start_activity(&mut context, TestRoute::SingleTopPage); // top
        context.clear();

        let (success, task) = manager.back(&mut context);
        assert!(success);
        assert_eq!(manager.stack_len(), 1);
        assert_eq!(
            context,
            vec![
                "on_pause: SingleTopPage",
                "on_destroy: SingleTopPage",
                "on_resume: StandardPage" // 前一个页面恢复
            ]
        );
        assert_eq!(task, vec!["task_destroy_SingleTopPage"]);

        // 栈中仅剩一页，无法继续 back
        context.clear();
        let (success, _) = manager.back(&mut context);
        assert!(!success);
        assert!(context.is_empty());
    }

    #[test]
    fn test_views_painters_algorithm() {
        let mut manager = create_manager();
        let mut context = Vec::new();

        manager.start_activity(&mut context, TestRoute::OpaquePage);
        manager.start_activity(&mut context, TestRoute::TranslucentPage);
        manager.start_activity(&mut context, TestRoute::TranslucentPage);

        // Opaque -> Translucent -> Translucent
        // 应该渲染全部三个
        let views = manager.views(&context);
        assert_eq!(views.len(), 3);
        assert_eq!(
            views,
            vec!["OpaquePage", "TranslucentPage", "TranslucentPage"]
        );

        // 在顶部推入一个不透明页面： Opaque -> Trans -> Trans -> Opaque
        manager.start_activity(&mut context, TestRoute::OpaquePage);
        let views_top = manager.views(&context);

        // 发生裁剪，仅渲染顶部那个 Opaque
        assert_eq!(views_top.len(), 1);
        assert_eq!(views_top, vec!["OpaquePage"]);
    }
}
