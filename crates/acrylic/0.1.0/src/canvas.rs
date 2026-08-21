use crate::{
    primitives::{Line, Primitive, Rect},
    Color, Image, Rgba,
};

pub struct Canvas {
    image: Image<Rgba>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            image: Image::new(width, height),
        }
    }

    pub fn from_image(image: Image<Rgba>) -> Self {
        Self { image }
    }

    pub fn into_image(self) -> Image<Rgba> {
        self.image
    }

    pub fn image(&self) -> &Image<Rgba> {
        &self.image
    }

    pub fn fill<C: Color>(&mut self, color: C) {
        self.image = Image::with_color(self.image.width(), self.image.height(), color.into());
    }

    pub fn draw<P: Into<Primitive>, C: Color>(&mut self, primitive: P, color: C) {
        match primitive.into() {
            Primitive::Line(line) => self.draw_line(line, color.into()),
            Primitive::Rect(rect) => self.draw_rect(rect, color.into()),
        }
    }

    fn draw_line(&mut self, line: Line, color: Rgba) {}

    fn draw_rect(&mut self, rect: Rect, color: Rgba) {
        let from = rect.top_left();
        let to = rect.bottom_right();

        for x in from.x.max(0)..to.x.min(self.image.width) {
            for y in from.y.max(0)..to.y.min(self.image.height) {
                let index = y * self.image.width + x;
                self.image.data[index] = color;
            }
        }
    }
}
