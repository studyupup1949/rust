/// Receives a value and processes it immediately.
///
/// The trait abstracts over *acceptors*: objects that can be invoked with a
/// value without returning any meaningful result. This is useful for sinks,
/// callbacks, or visitor-like APIs where forwarding or side effects are all
/// that is needed.
///
/// ```rust
/// use accepts::Accepts;
///
/// struct Printer;
///
/// impl Accepts<&'static str> for Printer {
///     fn accept(&self, value: &'static str) {
///         println!("{}", value);
///     }
/// }
///
/// let printer = Printer;
/// printer.accept("Hello, world!");
/// ```
pub trait Accepts<Value> {
    fn accept(&self, value: Value);
}
