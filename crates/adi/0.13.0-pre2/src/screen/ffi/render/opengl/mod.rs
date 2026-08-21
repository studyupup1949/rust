// Copyright Jeron Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at https://www.boost.org/LICENSE_1_0.txt)

//! OpenGL implementation for adi_gpu.

use std::cell::Cell;

mod asi;

use crate::screen::ffi::Matrix;
use std::mem;
use std::os::raw::c_void;

pub use self::base::Gradient;
pub use self::base::Model;
pub use self::base::Shape;
pub use self::base::TexCoords;
pub use self::base::Texture;

pub(crate) use self::asi::OPENGL;
use self::asi::{
    Buffer, Feature, OpenGL, OpenGLBuilder, Program, Topology, UniformData, VertexData,
};
use super::base;
use super::base::*;

const SHADER_SOLID_FRAG: &'static [u8] = include_bytes!("shaders/solid-frag.glsl");
const SHADER_SOLID_VERT: &'static [u8] = include_bytes!("shaders/solid-vert.glsl");
const SHADER_GRADIENT_FRAG: &'static [u8] = include_bytes!("shaders/gradient-frag.glsl");
const SHADER_GRADIENT_VERT: &'static [u8] = include_bytes!("shaders/gradient-vert.glsl");
const SHADER_TEX_FRAG: &'static [u8] = include_bytes!("shaders/texture-frag.glsl");
const SHADER_TEX_VERT: &'static [u8] = include_bytes!("shaders/texture-vert.glsl");
const SHADER_FADED_VERT: &'static [u8] = include_bytes!("shaders/faded-vert.glsl");
const SHADER_TINTED_FRAG: &'static [u8] = include_bytes!("shaders/tinted-frag.glsl");
const SHADER_COMPLEX_VERT: &'static [u8] = include_bytes!("shaders/complex-vert.glsl");
const SHADER_COMPLEX_FRAG: &'static [u8] = include_bytes!("shaders/complex-frag.glsl");
const SHADER_GUI_VERT: &'static [u8] = include_bytes!("shaders/gui-vert.glsl");
const SHADER_GUI_FRAG: &'static [u8] = include_bytes!("shaders/gui-frag.glsl");

const STYLE_GRADIENT: usize = 0;
const STYLE_TEXTURE: usize = 1;
const STYLE_FADED: usize = 2;
const STYLE_TINTED: usize = 3;
const STYLE_SOLID: usize = 4;
const STYLE_COMPLEX: usize = 5;
const STYLE_GUI: usize = 6;

struct Style {
    shader: Program,
    matrix_uniform: UniformData,
    camera_uniform: UniformData,
    alpha: UniformData,
    color: UniformData,
    position: VertexData,
    texpos: VertexData,
    acolor: VertexData,
}

impl Style {
    // Create a new style.
    fn new(context: &OpenGL, vert: &[u8], frag: &[u8]) -> Style {
        let shader = Program::new(context, vert, frag);
        let matrix_uniform = shader.uniform(b"models_tfm\0");
        let camera_uniform = shader.uniform(b"matrix\0");
        let alpha = shader.uniform(b"alpha\0");
        let color = shader.uniform(b"color\0");
        let position = shader.vertex_data(b"position\0");
        let texpos = shader.vertex_data(b"texpos\0");
        let acolor = shader.vertex_data(b"acolor\0");

        Style {
            shader,
            matrix_uniform,
            camera_uniform,
            position,
            texpos,
            alpha,
            color,
            acolor,
        }
    }
}

struct ShapeData {
    style: usize,
    buffers: [Option<Buffer>; 2],
    alpha: Option<f32>,
    color: Option<[f32; 4]>,
    transform: Matrix, // Transformation matrix.
    texture: Option<asi::Texture>,
    vertex_buffer: Buffer,
    fans: Vec<(u32, u32)>,
    viewer: usize,
}

/*impl base::Point for ShapeData {
    fn point(&self) -> Vector {
        // Position vector at origin * object transform.
        self.transform * (vector!(0f32, 0f32, 0f32), 1f32)
    }
}*/

struct ModelData {
    vertex_buffer: Buffer,
    // TODO alot could be in base as duplicate
    vertex_count: u32,
    fans: Vec<(u32, u32)>,
}

struct TexcoordsData {
    vertex_buffer: Buffer,
    vertex_count: u32,
}

struct GradientData {
    vertex_buffer: Buffer,
    vertex_count: u32,
}

struct TextureData {
    t: asi::Texture,
}

/// To render anything with adi_gpu, you have to make a `Display`
pub struct Display {
    window: crate::screen::ffi::Window,
    context: OpenGL,
    color: (f32, f32, f32),
    opaque_ind: Vec<u32>,
    alpha_ind: Vec<u32>,
    opaque_vec: Cell<Vec<ShapeData>>,
    alpha_vec: Cell<Vec<ShapeData>>,
    gui: Cell<ShapeData>,
    gui_tex: TextureData,
    gui_pix: Vec<u8>,
    models: Vec<ModelData>,
    texcoords: Vec<TexcoordsData>,
    gradients: Vec<GradientData>,
    textures: Vec<TextureData>,
    styles: [Style; 7],
    ar: f32,
}

pub fn new() -> Result<Box<Display>, &'static str> {
    if let Some(tuple) = OpenGLBuilder::new() {
        let (builder, v) = tuple;
        let window = crate::screen::ffi::Window::new(Some(v));

        let context = builder.to_opengl({
            #[cfg(any(
                target_os = "linux",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "bitrig",
                target_os = "openbsd",
                target_os = "netbsd",
            ))]
            unsafe {
                mem::transmute(window.window as usize)
            }

            #[cfg(target_os = "windows")]
            window.window
        });

        // Set the settings.
        context.disable(Feature::Dither);
        context.enable(Feature::CullFace);
        context.enable(Feature::Blend);
        context.blend();

        // Load shaders
        let style_solid = Style::new(&context, SHADER_SOLID_VERT, SHADER_SOLID_FRAG);
        let style_gradient = Style::new(&context, SHADER_GRADIENT_VERT, SHADER_GRADIENT_FRAG);
        let style_texture = Style::new(&context, SHADER_TEX_VERT, SHADER_TEX_FRAG);
        let style_faded = Style::new(&context, SHADER_FADED_VERT, SHADER_TEX_FRAG);
        let style_tinted = Style::new(&context, SHADER_TEX_VERT, SHADER_TINTED_FRAG);
        let style_complex = Style::new(&context, SHADER_COMPLEX_VERT, SHADER_COMPLEX_FRAG);
        let style_gui = Style::new(&context, SHADER_GUI_VERT, SHADER_GUI_FRAG);

        let wh = window.wh();
        let ar = wh.0 as f32 / wh.1 as f32;

        context.projection_set(crate::screen::shared::projection(ar, 0.5 * PI));

        // GUI Texture Coordinate
        let vertex_buffer = Buffer::new(&context);
        vertex_buffer.set(&crate::screen::shared::GUI_TC);
        let tc = TexcoordsData {
            vertex_buffer,
            vertex_count: crate::screen::shared::GUI_TC.len() as u32,
        };

        // GUI Texture
        let t = context.texture();
        let gui_pix = vec![0; wh.0 as usize * wh.1 as usize * 4];
        t.set(wh.0, wh.1, gui_pix.as_slice());
        let gui_tex = TextureData { t };

        // GUI Model
        let vertex_buffer = Buffer::new(&context);
        vertex_buffer.set(&crate::screen::shared::GUI_MC);
        let gui_model = ModelData {
            vertex_buffer,
            vertex_count: crate::screen::shared::GUI_MC.len() as u32 / 4,
            fans: crate::screen::shared::gui_mc_fans(),
        };

        let gui = ShapeData {
            style: STYLE_TEXTURE,
            buffers: [Some(tc.vertex_buffer.clone()), None],
            alpha: None,
            color: None,
            texture: Some(gui_tex.t.clone()),
            vertex_buffer: gui_model.vertex_buffer.clone(),
            transform: matrix!(), // Transformation matrix.
            fans: gui_model.fans.clone(),
            viewer: 0,
        };

        // Adjust the viewport
        context.viewport(wh.0, wh.1);

        let display = self::Display {
            window,
            context,
            color: (0.0, 0.0, 0.0),
            alpha_ind: vec![],
            opaque_ind: vec![],
            alpha_vec: Cell::new(vec![]),
            opaque_vec: Cell::new(vec![]),
            //            gui_vec: Cell::new(),
            gui: Cell::new(gui),
            gui_pix,
            gui_tex,
            models: vec![],
            texcoords: vec![],
            gradients: vec![],
            textures: vec![],
            styles: [
                style_gradient,
                style_texture,
                style_faded,
                style_tinted,
                style_solid,
                style_complex,
                style_gui,
            ],
            ar,
        };

        Ok(Box::new(display))
    } else {
        Err("Couldn't find OpenGL!")
    }
}

fn as_mut(slf: &Cell<Vec<ShapeData>>) -> &mut Vec<ShapeData> {
    unsafe { ::std::mem::transmute(slf.as_ptr()) }
}

impl base::Display for Display {
    fn color(&mut self, color: (u8, u8, u8)) {
        self.color = (
            color.0 as f32 / 255.0,
            color.1 as f32 / 255.0,
            color.2 as f32 / 255.0,
        );
        self.context.color(self.color.0, self.color.1, self.color.2);
    }

    fn update(&mut self) -> f32 {
        if self.window.update() {
            let wh = self.window.wh();

            // Resize.
            self.ar = wh.0 as f32 / wh.1 as f32;
            self.context.viewport(wh.0, wh.1);

            self.context
                .projection_set(crate::screen::shared::projection(self.ar, 0.5 * PI));
        }

        // Enable for 3D depth testing
        self.context.enable(Feature::DepthTest);

        for shape in as_mut(&self.opaque_vec).iter() {
            self.styles[shape.style]
                .camera_uniform
                .set_mat4(self.context.get().fakebuffer[shape.viewer]);
            draw_shape(&self.styles[shape.style], shape);
        }
        for shape in as_mut(&self.alpha_vec).iter() {
            self.styles[shape.style]
                .camera_uniform
                .set_mat4(self.context.get().fakebuffer[shape.viewer]);
            draw_shape(&self.styles[shape.style], shape);
        }

        // Disable Depth Testing for GUI
        self.context.disable(Feature::DepthTest);

        // Draw GUI elements.
        draw_shape(&self.styles[STYLE_GUI], self.gui.get_mut());

        self.context.update()
    }

    fn model(&mut self, vertices: &[f32], fans: Vec<(u32, u32)>) -> Model {
        // TODO most is duplicate from other implementation.
        let index = self.models.len();

        let buffer = Buffer::new(&self.context);

        let vertex_buffer = buffer;
        vertex_buffer.set(vertices);

        self.models.push(ModelData {
            vertex_buffer,
            vertex_count: vertices.len() as u32 / 4,
            fans,
        });

        Model(index)
    }

    fn texture(&mut self, wh: (u16, u16), graphic: &VFrame) -> Texture {
        let (w, h) = wh;
        let pixels = graphic.0.as_slice();

        let t = self.context.texture();

        t.set(w, h, pixels);

        let a = self.textures.len();

        self.textures.push(TextureData { t });

        Texture(a, w, h)
    }

    fn gradient(&mut self, colors: &[f32]) -> Gradient {
        // TODO: A lot of duplication here from adi_gpu_vulkan.  Put in
        // base.
        let vertex_buffer = Buffer::new(&self.context);
        vertex_buffer.set(colors);

        let a = self.gradients.len();

        self.gradients.push(GradientData {
            vertex_buffer,
            vertex_count: colors.len() as u32 / 4,
        });

        Gradient(a)
    }

    fn texcoords(&mut self, texcoords: &[(f32, f32)]) -> TexCoords {
        // TODO: A lot of duplication here from adi_gpu_vulkan.  Put in
        // base.
        let vertex_buffer = Buffer::new(&self.context);
        let mut buffer = vec![];
        for i in texcoords {
            buffer.push(i.0);
            buffer.push(i.1);
            buffer.push(1.0);
            buffer.push(1.0);
        }
        vertex_buffer.set(buffer.as_slice());

        let a = self.texcoords.len();

        self.texcoords.push(TexcoordsData {
            vertex_buffer,
            vertex_count: texcoords.len() as u32,
        });

        TexCoords(a)
    }

    fn set_texture(&mut self, texture: &mut Texture, wh: (u16, u16), graphic: &VFrame) {
        self.textures[texture.0]
            .t
            .set(wh.0, wh.1, graphic.0.as_slice());
    }

    #[inline(always)]
    fn shape_solid(
        &mut self,
        model: &Model,
        transform: Matrix,
        color: [f32; 4],
        blending: bool,
        camera: *const c_void,
    ) -> Shape {
        let shape = ShapeData {
            style: STYLE_SOLID,
            buffers: [None, None],
            alpha: None,
            color: Some(color),
            texture: None,
            vertex_buffer: self.models[model.0].vertex_buffer.clone(),
            transform, // Transformation matrix.
            fans: self.models[model.0].fans.clone(),
            viewer: unsafe { *(std::mem::transmute::<*const c_void, *const usize>(camera)) },
        };

        base::new_shape(if blending {
            let alpha_vec = self.alpha_vec.get_mut();
            let index = alpha_vec.len() as u32;
            alpha_vec.push(shape);
            self.alpha_ind.push(index);
            base::ShapeHandle::Alpha(index)
        } else {
            let opaque_vec = self.opaque_vec.get_mut();
            let index = opaque_vec.len() as u32;
            opaque_vec.push(shape);
            self.opaque_ind.push(index);
            base::ShapeHandle::Opaque(index)
        })
    }

    #[inline(always)]
    fn shape_gradient(
        &mut self,
        model: &Model,
        transform: Matrix,
        colors: Gradient,
        blending: bool,
        camera: *const c_void,
    ) -> Shape {
        // TODO: is copied from adi_gpu_vulkan, move to base
        if self.models[model.0].vertex_count != self.gradients[colors.0].vertex_count {
            panic!("TexCoord length doesn't match gradient length");
        }

        let shape = ShapeData {
            style: STYLE_GRADIENT,
            buffers: [Some(self.gradients[colors.0].vertex_buffer.clone()), None],
            alpha: None,
            color: None,
            texture: None,
            vertex_buffer: self.models[model.0].vertex_buffer.clone(),
            transform, // Transformation matrix.
            fans: self.models[model.0].fans.clone(),
            viewer: unsafe { *(std::mem::transmute::<*const c_void, *const usize>(camera)) },
        };

        base::new_shape(if blending {
            let alpha_vec = self.alpha_vec.get_mut();
            let index = alpha_vec.len() as u32;
            alpha_vec.push(shape);
            self.alpha_ind.push(index);
            base::ShapeHandle::Alpha(index)
        } else {
            let opaque_vec = self.opaque_vec.get_mut();
            let index = opaque_vec.len() as u32;
            opaque_vec.push(shape);
            self.opaque_ind.push(index);
            base::ShapeHandle::Opaque(index)
        })
    }

    #[inline(always)]
    fn shape_texture(
        &mut self,
        model: &Model,
        transform: Matrix,
        texture: &Texture,
        tc: TexCoords,
        blending: bool,
        camera: *const c_void,
    ) -> Shape {
        // TODO: from adi_gpu_vulkan, move to the base
        if self.models[model.0].vertex_count != self.texcoords[tc.0].vertex_count {
            panic!("TexCoord length doesn't match vertex length");
        }

        let shape = ShapeData {
            style: STYLE_TEXTURE,
            buffers: [Some(self.texcoords[tc.0].vertex_buffer.clone()), None],
            alpha: None,
            color: None,
            texture: Some(self.textures[texture.0].t.clone()),
            vertex_buffer: self.models[model.0].vertex_buffer.clone(),
            transform, // Transformation matrix.
            fans: self.models[model.0].fans.clone(),
            viewer: unsafe { *(std::mem::transmute::<*const c_void, *const usize>(camera)) },
        };

        base::new_shape(if blending {
            let alpha_vec = self.alpha_vec.get_mut();
            let index = alpha_vec.len() as u32;
            alpha_vec.push(shape);
            self.alpha_ind.push(index);
            base::ShapeHandle::Alpha(index)
        } else {
            let opaque_vec = self.opaque_vec.get_mut();
            let index = opaque_vec.len() as u32;
            opaque_vec.push(shape);
            self.opaque_ind.push(index);
            base::ShapeHandle::Opaque(index)
        })
    }

    #[inline(always)]
    fn shape_faded(
        &mut self,
        model: &Model,
        transform: Matrix,
        texture: &Texture,
        tc: TexCoords,
        alpha: f32,
        camera: *const c_void,
    ) -> Shape {
        // TODO: from adi_gpu_vulkan, move to the base
        if self.models[model.0].vertex_count != self.texcoords[tc.0].vertex_count {
            panic!("TexCoord length doesn't match vertex length");
        }

        let shape = ShapeData {
            style: STYLE_FADED,
            buffers: [Some(self.texcoords[tc.0].vertex_buffer.clone()), None],
            alpha: Some(alpha),
            color: None,
            texture: Some(self.textures[texture.0].t.clone()),
            vertex_buffer: self.models[model.0].vertex_buffer.clone(),
            transform, // Transformation matrix.
            fans: self.models[model.0].fans.clone(),
            viewer: unsafe { *(std::mem::transmute::<*const c_void, *const usize>(camera)) },
        };

        base::new_shape({
            let alpha_vec = self.alpha_vec.get_mut();
            let index = alpha_vec.len() as u32;
            alpha_vec.push(shape);
            self.alpha_ind.push(index);
            base::ShapeHandle::Alpha(index)
        })
    }

    #[inline(always)]
    fn shape_tinted(
        &mut self,
        model: &Model,
        transform: Matrix,
        texture: &Texture,
        tc: TexCoords,
        tint: [f32; 4],
        blending: bool,
        camera: *const c_void,
    ) -> Shape {
        // TODO: from adi_gpu_vulkan, move to the base
        if self.models[model.0].vertex_count != self.texcoords[tc.0].vertex_count {
            panic!("TexCoord length doesn't match vertex length");
        }

        let shape = ShapeData {
            style: STYLE_TINTED,
            buffers: [Some(self.texcoords[tc.0].vertex_buffer.clone()), None],
            alpha: None,
            color: Some(tint),
            texture: Some(self.textures[texture.0].t.clone()),
            vertex_buffer: self.models[model.0].vertex_buffer.clone(),
            transform, // Transformation matrix.
            fans: self.models[model.0].fans.clone(),
            viewer: unsafe { *(std::mem::transmute::<*const c_void, *const usize>(camera)) },
        };

        base::new_shape(if blending {
            let alpha_vec = self.alpha_vec.get_mut();
            let index = alpha_vec.len() as u32;
            alpha_vec.push(shape);
            self.alpha_ind.push(index);
            base::ShapeHandle::Alpha(index)
        } else {
            let opaque_vec = self.opaque_vec.get_mut();
            let index = opaque_vec.len() as u32;
            opaque_vec.push(shape);
            self.opaque_ind.push(index);
            base::ShapeHandle::Opaque(index)
        })
    }

    #[inline(always)]
    fn shape_complex(
        &mut self,
        model: &Model,
        transform: Matrix,
        texture: &Texture,
        tc: TexCoords,
        tints: Gradient,
        blending: bool,
        camera: *const c_void,
    ) -> Shape {
        // TODO: from adi_gpu_vulkan, move to the base
        if self.models[model.0].vertex_count != self.texcoords[tc.0].vertex_count {
            panic!("TexCoord length doesn't match vertex length");
        }

        // TODO: is copied from adi_gpu_vulkan, move to base
        if self.models[model.0].vertex_count != self.gradients[tints.0].vertex_count {
            panic!("TexCoord length doesn't match gradient length");
        }

        let shape = ShapeData {
            style: STYLE_COMPLEX,
            buffers: [
                Some(self.texcoords[tc.0].vertex_buffer.clone()),
                Some(self.gradients[tints.0].vertex_buffer.clone()),
            ],
            alpha: None,
            color: None,
            texture: Some(self.textures[texture.0].t.clone()),
            vertex_buffer: self.models[model.0].vertex_buffer.clone(),
            transform, // Transformation matrix.
            fans: self.models[model.0].fans.clone(),
            viewer: unsafe { *(std::mem::transmute::<*const c_void, *const usize>(camera)) },
        };

        base::new_shape(if blending {
            let alpha_vec = self.alpha_vec.get_mut();
            let index = alpha_vec.len() as u32;
            alpha_vec.push(shape);
            self.alpha_ind.push(index);
            base::ShapeHandle::Alpha(index)
        } else {
            let opaque_vec = self.opaque_vec.get_mut();
            let index = opaque_vec.len() as u32;
            opaque_vec.push(shape);
            self.opaque_ind.push(index);
            base::ShapeHandle::Opaque(index)
        })
    }

    #[inline(always)]
    fn drop_shape(&mut self, shape: &Shape) {
        match get_shape(&shape) {
            ShapeHandle::Opaque(x) => {
                let index = self.opaque_ind.iter().position(|y| *y == x).unwrap();
                self.opaque_ind.remove(index);
            }
            ShapeHandle::Alpha(x) => {
                let index = self.alpha_ind.iter().position(|y| *y == x).unwrap();
                self.alpha_ind.remove(index);
            }
        }
    }

    fn transform(&self, shape: &Shape, transform: Matrix) {
        // TODO: put in base, some is copy from vulkan implementation.
        match base::get_shape(shape) {
            ShapeHandle::Opaque(x) => {
                let x = x as usize; // for indexing
                as_mut(&self.opaque_vec)[x].transform = transform;
            }
            ShapeHandle::Alpha(x) => {
                let x = x as usize; // for indexing
                as_mut(&self.alpha_vec)[x].transform = transform;
            }
        }
    }

    fn wh(&self) -> (u16, u16) {
        self.window.wh()
    }

    fn pitch(&self) -> usize {
        self.wh().0 as usize * 4
    }

    fn draw(&mut self, writer: &mut FnMut(*mut u8) -> ()) {
        let (w, h) = self.wh();

        writer(self.gui_pix.as_mut_ptr());

        self.gui_tex.t.set(w, h, self.gui_pix.as_slice());
    }
}

fn draw_shape(style: &Style, shape: &ShapeData) {
    style.matrix_uniform.set_mat4(shape.transform.into());

    if !style.texpos.is_none() {
        // Set texpos for the program from the texpos buffer.
        style.texpos.set(shape.buffers[0].as_ref().unwrap());
        // Bind the texture
        shape.texture.as_ref().unwrap().bind();
    }

    if !style.acolor.is_none() {
        // Set colors for the program from the color buffer.
        // TODO: probably shouldn't be same buffer as texpos.
        style.acolor.set(shape.buffers[0].as_ref().unwrap());
    }

    if !style.alpha.is_none() {
        style.alpha.set_vec1(shape.alpha.unwrap());
    }

    if !style.color.is_none() {
        style.color.set_vec4(&shape.color.unwrap());
    }

    // Set vertices for the program from the vertex buffer.
    style.position.set(&shape.vertex_buffer);
    for i in shape.fans.iter() {
        style.shader.draw_arrays(Topology::TriangleFan, i.0..i.1);
    }
}
