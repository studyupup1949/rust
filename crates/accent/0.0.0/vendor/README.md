# Accent 🎨

`accent` is a abstraction API for color by Rose Waters. 

Instead of treating colors as dumb `[f32; 4]` buffers or generic hex values, `accent` introduces compile-time type safety for pixel pipelines. It allows graphics engineers, engine developers, and shader designers to represent, transform, and connect standard display, broadcast, and perceptual color models without runtime overhead.

## 🚀 Why Accent?

In modern rendering and compositing, **color spaces are math math-intensive coordinate systems**. Doing math in the wrong space results in ugly artifacts like the infamous "dark band" during linear blending, or unpredictable hue shifts in UI gradients.

`accent` uses Rust's type system to make incorrect color math a compile-time error.

* **Zero-Cost Abstractions**: Space transformations compile down to raw, optimized matrix multiplication and vector math operations.
* **Layout vs. Space Separation**: Decouples the structural memory layout (`Rgb`, `Cmyk`, `Hsv`) from the color profile (`sRGB`, `Display P3`, `Adobe RGB`).
* **Graphics-First Engine Ready**: Native support for scalar types (`f32`, `f64`, `u8`) making it a drop-in match for GPU buffers and vertex layouts.

---

## 🗺️ Graph Ecosystem (Color Families)

`accent` provides mathematical mappings across 5 core structural layout families:

### 1. RGB Layouts (Additive & Display Spaces)
Engine-level linear and non-linear spaces driving modern monitors, projectors, and virtual viewports.
* `sRGB` / `Linear RGB` – The standard web rendering, texturing, and shading baseline.
* `Display P3` – Wide-gamut standard adopted by modern mobile displays and digital cinema.
* `Adobe RGB` / `Wide Gamut RGB` – Extended gamuts for photography rendering pipelines.
* `ProPhoto RGB` – Ultra-wide mastering gamut designed to prevent clipping.
* `Rec. 709` / `Rec. 2020` – Standard and Ultra-HD broadcast raster frameworks.
* `scRGB` – Wide-gamut, high-dynamic-range (HDR) float-based rendering pipeline target.
* `ACEScg` – Academy Color Encoding System for VFX rendering, lighting, and CGI pipelines.

### 2. Cylindrical Transformations (UI & Math Generation)
Geometric reformats of RGB cubes into cylinders, perfect for parametric color pickers, procedural generation, and UI math.
* `HSL` (Lightness) & `HSV` / `HSB` (Value/Brightness)
* `HSI` (Intensity) – Ideal for computer vision filtering algorithms.
* `HWB` (Whiteness/Blackness) & `HCG` (Chroma/Grayness) – Optimized structures for artist-friendly UI controls.

### 3. Perceptual Uniformity (Blending & Gradient Engines)
Math models spaced relative to human sight. **Essential for generating flawless, artifact-free linear interpolation (lerp) and lighting gradients.**
* `OKLab` & `OKLCH` – Modern standards optimized for ultra-smooth gradients without ugly midway hue-shifting.
* `CIELAB (Lab)` & `LCHab` – The foundational perceptual coordinate standards.
* `CIELUV` & `LCHuv` – Optimized mathematically for additive lighting and display emit profiles.
* `Hunter Lab` – Industrial measurement geometry.

### 4. CMYK Print Modeling
Subtractive color matrices for physical output rendering, soft-proofing, and print asset export.
* `Device CMYK` | `SWOP` | `U.S. Web Coated` | `FOGRA39` | `FOGRA51` | `Japan Color` | `GRACoL`

### 5. Utility & Composite Vectors
Mathematical coordinate vectors, luminance trackers, and legacy broadcast matrices.
* `XYZ` & `xyY` – The foundational reference spaces acting as the mathematical root for all transforms.
* `YCbCr` / `YUV` / `YPbPr` / `YIQ` – Luma/Chroma split channels for hardware video compression and video streams.
* `ICtCp` / `JzAzBz` – Specialized high-performance mathematical spaces optimized for modern HDR signals.
* `IPT` – Tuned specifically for constant hue mapping algorithms.
* `LMS` – Cone-response simulation matrices (Long, Medium, Short wavelengths).
* `Hexadecimal` – Fast string parsing for CSS/Web asset pipelines.

**Note**: current support [OUTLINE](OUTLINE.md)
---

## 🛠️ API & Architecture Architecture

The core of the library is the `Color` struct, parameterized by its physical component **Model**, its conceptual **Space**, and its scalar data storage **Type**:

```rust
pub struct Color<M, S, T = f32>
where 
    M: Model,
    S: Space<ModelType = M>,
{
    pub channels: M::Channels<T>,
    pub alpha: T,
    _space: PhantomData<S>
}