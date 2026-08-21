use iced::widget::{button, center, column, container, stack, text};
use iced::{Element, Length, Task, color};
use std::fmt::Debug;

// 引入外部 crate (即你的 activity-manager 核心库)
use activity_manager::{Activity, ActivityHost, ActivityManager, LaunchMode, Route};

// ============================================================================
// 1. 业务配置与模型定义
// ============================================================================

/// 全局上下文（将被借用并渲染）
pub struct GlobalContext {
    pub user_name: String,
}

/// 路由定义
#[derive(Debug, Clone, PartialEq)]
pub enum AppRoute {
    Home,
    Detail(String), // 携带参数的路由
    Dialog,         // 弹窗页面
}

impl Route for AppRoute {
    fn launch_mode(&self) -> LaunchMode {
        match self {
            AppRoute::Home => LaunchMode::SingleTask, // 首页回退时清空上方栈
            _ => LaunchMode::Standard,
        }
    }

    fn is_translucent(&self) -> bool {
        matches!(self, AppRoute::Dialog) // 只有 Dialog 是半透明的
    }
}

/// 消息定义
#[derive(Debug, Clone)]
pub enum AppMessage {
    // ---- 导航消息（框架截获） ----
    NavigateTo(AppRoute),
    Back,
    // ---- 页面业务消息（分发给栈顶页面） ----
    IncrementCounter,
}

/// 宿主环境实现
pub struct IcedHost;

impl ActivityHost for IcedHost {
    type Message = AppMessage;
    type Context = GlobalContext;
    type Route = AppRoute;
    type View<'a> = Element<'a, AppMessage>;
    type Task = Task<AppMessage>;

    fn empty_task() -> Self::Task {
        Task::none()
    }

    fn batch_tasks(tasks: Vec<Self::Task>) -> Self::Task {
        Task::batch(tasks)
    }
}

// ============================================================================
// 2. 页面实现 (Activities)
// ============================================================================

/// 页面一：首页
pub struct HomeActivity {
    counter: i32,
    welcome_text: String,
}

impl Activity<IcedHost> for HomeActivity {
    fn route(&self) -> AppRoute {
        AppRoute::Home
    }

    fn on_create(&mut self, _ctx: &mut GlobalContext) -> Task<AppMessage> {
        log::info!("HomeActivity created.");
        Task::none()
    }

    fn on_resume(&mut self, _ctx: &mut GlobalContext) -> Task<AppMessage> {
        log::debug!("HomeActivity resumed.");
        Task::none()
    }

    fn on_pause(&mut self, _ctx: &mut GlobalContext) -> Task<AppMessage> {
        log::debug!("HomeActivity paused.");
        Task::none()
    }

    fn update(&mut self, _ctx: &mut GlobalContext, msg: AppMessage) -> Task<AppMessage> {
        if let AppMessage::IncrementCounter = msg {
            log::debug!("Incrementing counter in HomeActivity.");
            self.counter += 1;
        }
        Task::none()
    }

    fn view<'a>(&'a self, ctx: &'a GlobalContext) -> Element<'a, AppMessage> {
        let content = column![
            text(&self.welcome_text).size(40),
            text(format!("当前用户: {}", ctx.user_name)).size(20),
            text(format!("页面内计数器: {}", self.counter)).size(50),
            button("计数 + 1").on_press(AppMessage::IncrementCounter),
            button("跳转到详情页 (Standard)").on_press(AppMessage::NavigateTo(AppRoute::Detail(
                "订单 #1024".into()
            ))),
            button("打开弹窗 (Translucent)").on_press(AppMessage::NavigateTo(AppRoute::Dialog)),
        ]
        .spacing(20)
        .padding(20);

        center(content).into()
    }
}

/// 页面二：详情页
pub struct DetailActivity {
    param_id: String,
}

impl Activity<IcedHost> for DetailActivity {
    fn route(&self) -> AppRoute {
        AppRoute::Detail(self.param_id.clone())
    }

    fn on_create(&mut self, _ctx: &mut GlobalContext) -> Task<AppMessage> {
        log::info!("DetailActivity created with ID: {}", self.param_id);
        Task::none()
    }

    fn on_destroy(&mut self, _ctx: &mut GlobalContext) -> Task<AppMessage> {
        log::info!("DetailActivity destroyed (ID: {}).", self.param_id);
        Task::none()
    }

    fn view<'a>(&'a self, _ctx: &'a GlobalContext) -> Element<'a, AppMessage> {
        let content = column![
            text("详情页面").size(40),
            text(format!("接收到的参数: {}", self.param_id)).size(20),
            button("返回 (Back)").on_press(AppMessage::Back),
            button("回到首页 (SingleTask)").on_press(AppMessage::NavigateTo(AppRoute::Home)),
        ]
        .spacing(20);

        container(content)
            // 修改点：传入明确的 Length::Fill，修复 center_x missing argument
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            // 修改点：使用 Iced 0.14 的 container::Style 建造者模式
            .style(|_theme| container::Style::default().background(color!(0x111111)))
            .into()
    }
}

/// 页面三：弹窗页面 (测试半透明画家算法)
pub struct DialogActivity;

impl Activity<IcedHost> for DialogActivity {
    fn route(&self) -> AppRoute {
        AppRoute::Dialog
    }

    fn on_create(&mut self, _ctx: &mut GlobalContext) -> Task<AppMessage> {
        log::info!("DialogActivity (Translucent) opened.");
        Task::none()
    }

    fn view<'a>(&'a self, _ctx: &'a GlobalContext) -> Element<'a, AppMessage> {
        let dialog_box = container(
            column![
                text("这是一个半透明叠加弹窗！").size(24),
                button("关闭弹窗").on_press(AppMessage::Back)
            ]
            .spacing(20),
        )
        .padding(30)
        // 修改点：使用 Iced 0.14 的 Style::default() 配置背景和边框
        .style(|_theme| {
            container::Style::default()
                .background(color!(0x333333))
                .border(iced::Border {
                    radius: 10.0.into(),
                    ..Default::default()
                })
        });

        // 半透明遮罩
        container(center(dialog_box))
            .width(Length::Fill)
            .height(Length::Fill)
            // 修改点：更新为新的容器样式结构
            .style(|_theme| {
                container::Style::default().background(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.7))
            })
            .into()
    }
}

// ============================================================================
// 3. 应用入口逻辑
// ============================================================================

/// 工厂函数：根据 Route 实例化相应的 Activity
fn activity_factory(route: &AppRoute) -> Box<dyn Activity<IcedHost>> {
    match route {
        AppRoute::Home => Box::new(HomeActivity {
            counter: 0,
            welcome_text: "欢迎来到 Iced 0.14 路由引擎".into(),
        }),
        AppRoute::Detail(id) => Box::new(DetailActivity {
            param_id: id.clone(),
        }),
        AppRoute::Dialog => Box::new(DialogActivity),
    }
}

/// 顶层应用结构
struct IcedApp {
    manager: ActivityManager<IcedHost>,
    context: GlobalContext,
}

impl IcedApp {
    /// 初始化应用
    fn new() -> (Self, Task<AppMessage>) {
        log::info!("Initializing IcedApp and routing to Home.");
        let mut manager = ActivityManager::new(activity_factory);
        let mut context = GlobalContext {
            user_name: "Rust开发者".into(),
        };

        let task = manager.start_activity(&mut context, AppRoute::Home);
        (Self { manager, context }, task)
    }

    /// Iced 主更新循环
    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::NavigateTo(route) => {
                log::debug!("Global navigation requested: {:?}", route);
                self.manager.start_activity(&mut self.context, route)
            }
            AppMessage::Back => {
                log::debug!("Global back navigation requested.");
                self.manager.back(&mut self.context).1
            }
            _ => {
                // 将其他业务消息透传给栈顶页面
                self.manager.update(&mut self.context, message)
            }
        }
    }

    /// Iced 主渲染循环
    fn view(&self) -> Element<'_, AppMessage> {
        // 从 Manager 获取渲染视图（内部触发画家算法过滤）
        let views = self.manager.views(&self.context);
        stack(views).width(Length::Fill).height(Length::Fill).into()
    }
}

pub fn main() -> iced::Result {
    // 初始化日志系统，允许在终端查看 ActivityManager 的生命周期调用
    env_logger::init();
    log::info!("Starting the Iced application...");

    // 修改点：Iced 0.14 的启动链式调用（入参变为 BootFn, UpdateFn, ViewFn）
    iced::application(IcedApp::new, IcedApp::update, IcedApp::view)
        .title("Activity Manager Iced Demo")
        .run()
}
