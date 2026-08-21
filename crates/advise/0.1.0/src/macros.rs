/// Prints advise to [stderr][`std::io::stderr`].
///
/// Advise at the specified [tag][`super::Render`] with message body created
/// using a [`format!`]-based argument list.
///
/// # Examples
///
/// ```
/// use advise::advise;
/// use log::Level;
///
/// # fn main() {
/// let data = (42, "Forty-two");
///
/// advise!(Level::Error, "Received errors: {}, {}", data.0, data.1);
/// # }
/// ```
///
/// Produces:
///
/// <pre>
/// <b><span style=color:red>error</span>:</b> Received errors: 42, Forty-two
/// </pre>
#[macro_export]
macro_rules! advise {
    ($tag:expr, $($arg:tt)*) => {
        $crate::anstream::eprintln!("{}", $crate::Advise::new($tag, format!($($arg)*)))
    }
}

/// Advises about an [error][`log::Level::Error`] message.
///
/// # Examples
///
/// ```
/// use advise::error;
///
/// # fn main() {
/// let (err_info, port) = ("No connection", 22);
///
/// error!("{} on port {}", err_info, port);
/// # }
/// ```
///
/// Produces:
///
/// <pre>
/// <b><span style=color:red>error</span>:</b> No connection on port 22
/// </pre>
#[macro_export]
macro_rules! error {
    // error!("a {} event", "log")
    ($($arg:tt)*) => ($crate::advise!($crate::log::Level::Error, $($arg)*))
}

/// Advises about a [warning][`log::Level::Warn`] message.
///
/// # Examples
///
/// ```
/// use advise::warn;
///
/// # fn main() {
/// let warn_description = "Invalid Input";
///
/// warn!("{}!", warn_description);
/// # }
/// ```
///
/// Produces:
///
/// <pre>
/// <span style=color:orange>warn</span><b>:</b> Invalid Input!
/// </pre>
#[macro_export]
macro_rules! warn {
    // warn!("a {} event", "log")
    ($($arg:tt)*) => ($crate::advise!($crate::log::Level::Warn, $($arg)*))
}

/// Advises about an [info][`log::Level::Info`] message.
///
/// # Examples
///
/// ```
/// use advise::info;
///
/// # fn main() {
/// # struct Connection { port: u32, speed: f32 }
/// let conn_info = Connection { port: 40, speed: 3.20 };
///
/// info!("Connected to port {} at {} Mb/s", conn_info.port, conn_info.speed);
/// # }
/// ```
///
/// Produces:
///
/// <pre>
/// <span style=color:green>info</span><b>:</b> Connected to port 40 at 3.2 MB/s
/// </pre>
#[macro_export]
macro_rules! info {
    // info!("a {} event", "log")
    ($($arg:tt)*) => ($crate::advise!($crate::log::Level::Info, $($arg)*))
}

/// Advises about a [debug][`log::Level::Debug`] message.
///
/// # Examples
///
/// ```
/// use advise::debug;
///
/// # fn main() {
/// # struct Position { x: f32, y: f32 }
/// let pos = Position { x: 3.234, y: -1.223 };
///
/// debug!("New position: x: {}, y: {}", pos.x, pos.y);
/// # }
/// ```
///
/// Produces:
///
/// <pre>
/// <span style=color:blue>debug</span><b>:</b> New position: x: 3.234, y: -1.223
/// </pre>
#[macro_export]
macro_rules! debug {
    // debug!("a {} event", "log")
    ($($arg:tt)*) => ($crate::advise!($crate::log::Level::Debug, $($arg)*))
}

/// Advises about a [trace][`log::Level::Trace`] message.
///
/// # Examples
///
/// ```
/// use advise::trace;
///
/// # fn main() {
/// # struct Position { x: f32, y: f32 }
/// let pos = Position { x: 3.234, y: -1.223 };
///
/// trace!("Position is: x: {}, y: {}", pos.x, pos.y);
/// # }
/// ```
///
/// Produces:
///
/// <pre>
/// <span style=color:magenta>trace</span><b>:</b> Position is: x: 3.234, y: -1.223
/// </pre>
#[macro_export]
macro_rules! trace {
    // trace!("a {} event", "log")
    ($($arg:tt)*) => ($crate::advise!($crate::log::Level::Trace, $($arg)*))
}
