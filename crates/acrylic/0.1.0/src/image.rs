use crate::{Color, Rgba};

pub struct Image<C: Color> {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) data: Vec<C>,
}

impl<C: Color> Image<C> {
    pub fn new(width: usize, height: usize) -> Self {
        Self::with_color(width, height, C::default())
    }

    pub fn with_color(width: usize, height: usize, color: C) -> Self {
        Self {
            width,
            height,
            data: vec![color; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

impl Image<Rgba> {
    pub fn into_bytes(self) -> Vec<u8> {
        unsafe {
            let mut data = std::mem::transmute::<Vec<Rgba>, Vec<u8>>(self.data);
            data.set_len(data.len() * 4);
            data
        }
    }
}
