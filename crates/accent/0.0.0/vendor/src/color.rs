#![allow(unused)]
// color.rs
use std::marker::PhantomData;

use crate::{model::Model, space::Space};

/// # Color
/// A strongly-typed, memory-efficient color representation designed for graphics pipelines.
///
/// `Color` decouples the mathematical memory layout ([`Model`]) from its colorimetric
/// interpretation ([`Space`]). This guarantees compile-time safety when performing 
/// blending, shading, or texture manipulation, eliminating runtime overhead.
///
/// ### Type Parameters
/// * `M`: The structural layout or data shape (e.g., `Rgb`, `Cmyk`, `Hsl`, `Luma`).
/// * `S`: The specific color space standard providing context to the channels (e.g., `Srgb`, `DisplayP3`, `Oklab`).
/// * `T`: The underlying scalar component type (defaults to `f32` for GPU/shader compatibility).
///
/// ### Examples
/// ```rust
/// // A standard 8-bit web/UI color texturing element
/// let ui_red: Color<Rgb, Srgb, u8> = Color::new([255, 0, 0], 255);
///
/// // A high-dynamic-range linear float color for a physics-based shader pipeline
/// let render_pixel: Color<Rgb, LinearRgb, f32> = ui_red.into_linear();
/// ```
pub struct Color<M, S, T = f32>
where M: Model,
      S: Space<ModelType = M>,
{
    /// The raw color components (e.g., `[T; 3]` or a packed SIMD array) mapped by the layout `M`.
    pub channels: M::Channels<T>,
    /// The scalar transparency value, matching the precision of the color channels.
    pub alpha: T,
    /// Compile-time marker binding this layout instance to its mathematical coordinate space.
    _space: PhantomData<S>
}