use anyhow::Result;
use axum::Router;

pub(crate) type HeapedController<S> = Box<dyn Controller<S>>;
pub(crate) type ControllerList<S> = Vec<HeapedController<S>>;

/// ``Controller`` - trait required for the ``use_controller`` method on ``axum::Router``.
///
/// The ``register`` method is called at the moment of ``axum`` web service initialization, and is applied at the moment of ``axum::Router`` creation.
pub trait Controller<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn new() -> Result<Box<Self>> where Self: Sized;
  fn register(&self, router: Router<S>) -> Router<S>;
}

/// ``ApplyControllerOnRouter`` is an internal trait for ``axum::Router``.
///
/// Adds two methods - ``use_controller`` and ``use_controllers``.
pub(crate) trait ApplyControllerOnRouter<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn use_controller(self, controller: &dyn Controller<S>) -> Router<S>;
  fn use_controllers(self, controllers: ControllerList<S>) -> Router<S>;
}

impl<S> ApplyControllerOnRouter<S> for Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn use_controller(self, controller: &dyn Controller<S>) -> Router<S>
  {
    controller.register(self)
  }

  fn use_controllers(self, controllers: ControllerList<S>) -> Router<S> {
    // аллоцированные в хипе
    // бедняшки((
    controllers.into_iter().fold(self, |router, controller| {
      router.use_controller(&*controller)
    })
  }
}

// soooo here we wrapping controller into box,
// it means library eats more resources
/// Creates a vector of controllers allocated to Heap (via Box)
///
/// # Example
/// ```rs
/// pub struct AppState {
///   postgres: Pool<Postgres>
/// };
///
/// Service {
///   name: "Example",
///   state: AppState::default(),
///   controllers: controllers![ExampleController, YetAnotherControler],
///   ..Default::default() // port: None
/// }
/// ```
#[macro_export]
macro_rules! controllers {
  ($($controller:ty),* $(,)?) => {{
    use adjust::controller::Controller;
    let mut vec: Vec<Box<dyn adjust::controller::Controller<AppState>>> = Vec::new();
    $(
      log::debug!("{} connected", stringify!($controller));
      vec.push(<$controller>::new().expect(&format!("{} constructor throws an error", stringify!($controller))));
    )*

    vec
  }};
}