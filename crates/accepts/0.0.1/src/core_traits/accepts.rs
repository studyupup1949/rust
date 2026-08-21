/// Receives a value and processes it immediately.
///
/// The trait abstracts over *acceptors*: objects that can be invoked with a
/// value without returning any meaningful result.  This is particularly handy
/// when modelling sinks, callbacks, or visitor-like APIs.
///
/// ```rust
/// use accepts::core_traits::Accepts;
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
