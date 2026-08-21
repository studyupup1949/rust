use super::AdicEl;


/// Trait for shapes that can be displayed
pub trait DisplayShape {

    /// Display-independent elements needed to draw this shape.
    fn adic_els(&self) -> impl Iterator<Item=AdicEl>;

    /// Default css class
    fn default_class(&self) -> String;

    /// Height of svg viewbox
    fn viewbox_height(&self) -> u32;
    /// Width of svg viewbox
    fn viewbox_width(&self) -> u32;
    /// Viewbox string
    fn viewbox_str(&self) -> String {
        [0, 0, self.viewbox_width(), self.viewbox_height()].map(|x| x.to_string()).join(" ")
    }

}
