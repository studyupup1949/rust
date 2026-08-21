pub trait Color: Clone + Default + Into<Rgba> {}

#[derive(Copy, Clone, Debug)]
pub struct Rgba {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Rgba {
    pub const TRANSPARENT: Rgba = Rgba::new(0, 0, 0, 0);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl Color for Rgba {}

impl Default for Rgba {
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

impl From<(u8, u8, u8, u8)> for Rgba {
    fn from(comps: (u8, u8, u8, u8)) -> Self {
        Self::new(comps.0, comps.1, comps.2, comps.3)
    }
}
