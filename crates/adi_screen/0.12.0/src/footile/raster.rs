// raster.rs    A 2D raster image.
//
// Copyright (c) 2017-2018  Douglas P Lau
//
use std::ptr;
use footile::mask::Mask;
use footile::palette::Rgba;
use footile::palette::Blend;

/// A raster image to composite plot output.
///
/// # Example
/// ```
/// use footile::{PathBuilder, Plotter, Raster};
/// let path = PathBuilder::new().pen_width(5f32)
///                        .move_to(16f32, 48f32)
///                        .line_to(32f32, 0f32)
///                        .line_to(-16f32, -32f32)
///                        .close().build();
/// let mut p = Plotter::new(100, 100);
/// let mut r = Raster::new(p.width(), p.height());
/// p.add_ops(&path);
/// p.stroke();
/// r.composite(p.mask(), [208u8, 255u8, 208u8]);
/// ```
pub struct Raster {
    width  : u32,
    height : u32,
    pixels : Vec<u8>,
}

impl Raster {
    /// Create a new raster image.
    ///
    /// * `width` Width in pixels.
    /// * `height` Height in pixels.
    pub fn new(width: u32, height: u32) -> Raster {
        let n = width as usize * height as usize * 4 as usize;
        let pixels = vec![0u8; n];
        Raster { width: width, height: height, pixels: pixels }
    }
    /// Clear all pixels.
    pub fn clear(&mut self) {
        let len = self.pixels.len();
        unsafe {
            let pix = self.pixels.as_mut_ptr().offset(0 as isize);
            ptr::write_bytes(pix, 0u8, len);
        }
    }
    /// Composite a color with a mask.
    ///
    /// * `mask` Mask for compositing.
    /// * `clr` RGB color.
    pub fn composite(&mut self, mask: &Mask, clr: [u8; 4]) {
        for (p, m) in self.pixels.chunks_mut(4).zip(mask.iter()) {
            let alpha = ((*m as f32) * (clr[3] as f32) / 255.0) as u8;
            let src = Rgba::<f32>::new_u8(clr[0], clr[1], clr[2], alpha);
            let dst = Rgba::<f32>::new_u8(p[0], p[1], p[2], p[3]);
            let c = src.over(dst);
            let d = c.to_pixel::<[u8; 4]>();
            p[0] = d[0];
            p[1] = d[1];
            p[2] = d[2];
            p[3] = d[3];
        }
    }
    /// Composite a color with a mask.
    ///
    /// * `mask` Mask for compositing.
    /// * `clr` RGB color.
    pub fn cut(&mut self, mask: &Mask) {
        for (p, m) in self.pixels.chunks_mut(4).zip(mask.iter()) {
            let src = Rgba::<f32>::new_u8(0, 0, 0, *m);
            let dst = Rgba::<f32>::new_u8(p[0], p[1], p[2], p[3]);
            let c = dst - src;
            let d = c.to_pixel::<[u8; 4]>();
            p[0] = d[0];
            p[1] = d[1];
            p[2] = d[2];
            p[3] = d[3];
        }
    }
    /// Get the RGBA pixels for the mask.
    pub fn get_pixels<'a>(&'a self) -> (u32, u32, &'a [u8]) {
        (self.width, self.height, &self.pixels[..])
    }
}

impl AsRef<[u8]> for Raster {
    fn as_ref(&self) -> &[u8] {
        self.pixels.as_slice()
    }
}
