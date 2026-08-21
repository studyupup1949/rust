#[cfg(all(
    any(target_os = "linux", target_os = "android"),
    any(feature = "speaker", feature = "mic")
))]
pub(crate) mod alsa;

static mut GLOBAL: Global = Global {
    #[cfg(feature = "screen")]
    screen: None,

    #[cfg(feature = "hid")]
    inputm: None,
};

// Global library state.
struct Global {
    #[cfg(feature = "screen")]
    screen: Option<crate::screen::screen::App>,

    #[cfg(feature = "hid")]
    inputm: Option<crate::hid::NativeManager>,
}

#[cfg(feature = "screen")]
pub(crate) fn screen() -> &'static mut crate::screen::screen::App {
    unsafe { GLOBAL.screen.as_mut().unwrap() }
}

/// An Application's Context.
pub trait App {
    /// Initialize the App data.
    fn new() -> Self;

    /// Run the next step of the application.
    fn run(&mut self);
}

/// Enter the app.
pub fn new<Ctx: App>() -> Result<(), String> {
    // Set Global
    unsafe {
        GLOBAL = Global {
            #[cfg(feature = "screen")]
            screen: Some(crate::screen::screen::App::new()),

            #[cfg(feature = "hid")]
            inputm: Some(crate::hid::new()),
        };
    }

    // Initialize the App Data
    let mut app = Ctx::new();

    loop {
        // Run App.
        app.run();

        #[cfg(feature = "screen")]
        {
            screen().dt = screen().display.update();
        }

        #[cfg(feature = "hid")]
        unsafe {
            crate::hid::update(GLOBAL.inputm.as_mut().unwrap());
        }
    }
}

/// Exit the app.
pub fn old() {
    ::std::process::exit(0);
}

/// Create the main function.
/// 
/// ```
/// #[macro_use]
/// extern crate adi;
/// 
/// use adi::App;
///
/// main!(
///     Ctx,
///     struct Ctx {
///         mode: fn(app: &mut Ctx),
///     }
/// );
/// 
/// impl App for Ctx {
///     fn new() -> Ctx {
///         Ctx {
///             mode: mode,
///         }
///     }
/// 
///     fn run(&mut self) {
///         (self.mode)(self)
///     }
/// }
/// 
/// fn mode(app: &mut App, _ctx: &mut Ctx, _runner: &mut Runner<Ctx>, event: Event, _dt: f32) {
///     // Your code.
/// }
/// ```
#[macro_export]
macro_rules! main {
	($ctx_type:ty, $ctx_def:item) => {
        $ctx_def

        fn main() -> Result<(), String> {
            adi::new::<$ctx_type>()
        }
    }
}
