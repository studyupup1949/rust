// Copyright Jeron A. Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at https://www.boost.org/LICENSE_1_0.txt)

use self::PathOp::*;
use crate::screen::ffi::render::*;
use crate::screen::Matrix;
use afi::PathOp;

/// Macro to load models from files for the window.
///
/// The first model file listed is indexed 0, and each subsequent model
/// increases by 1.  See: [`sprites!()`](macro.sprites.html) for example.
#[macro_export]
macro_rules! models {
	($window:expr, $( $x:expr ),*) => { {
		use $crate::prelude::ModelBuilder as model;

		let a = vec![ $( include!($x).finish($window) ),* ];

		$window.models(a);
	} }
}

/// The builder for `Model`.
pub struct ModelBuilder {
    // Final output
    vertices: Vec<f32>,
    // Build a tristrip
    ts: Vec<[f32; 4]>,
    // A list of the fans to draw (start vertex, vertex count)
    fans: Vec<(u32, u32)>,
}

impl ModelBuilder {
    #[doc(hidden)]
    pub fn new() -> Self {
        ModelBuilder {
            vertices: vec![],
            ts: vec![],
            fans: vec![],
        }
    }

    /// Set the vertices for the following faces.
    pub fn vert<'a, T>(mut self, vertices: T) -> Self
    where
        T: IntoIterator<Item = &'a PathOp>,
    {
        let mut vertices = vertices.into_iter();

        self.ts = vec![];

        if let &Move(x, y, z) = vertices.next().unwrap() {
            self.ts.push([x, y, z, 1.0]);
        } else {
            panic!("Origin coordinates unknown");
        }

        for i in vertices {
            if let &Line(x, y, z) = i {
                self.ts.push([x, y, z, 1.0]);
            } else {
                panic!("Ops other than Line not supported yet.");
            }
        }

        self
    }

    /// Set the vertices for a double-sided face (actually 2 faces)
    pub fn dface(mut self, mat4: ami::Matrix) -> Self {
        self = self.shape(mat4);
        self.ts.reverse();
        let origin = self.ts.pop().unwrap();
        self.ts.insert(0, origin);
        self = self.shape(mat4);
        self
    }

    /// Add a face to the model, this unapplies the transformation matrix.
    pub fn face(mut self, mat4: Matrix) -> Self {
        self = self.shape(mat4);
        self
    }

    /// Add a shape to the model.
    fn shape(mut self, mat4: Matrix) -> Self {
        if self.ts.len() == 0 {
            return self;
        }

        let start = self.vertices.len() / 4;
        let length = self.ts.len();

        self.fans.push((start as u32, length as u32));

        for i in &self.ts {
            let v = mat4 * (vector!(i[0], i[1], i[2]), i[3]);

            self.vertices.push(v.x as f32);
            self.vertices.push(v.y as f32);
            self.vertices.push(v.z as f32);
            self.vertices.push(1.0); // W=1 for Point
        }

        self
    }

    /// Create the model / close the path.
    pub fn close(self) -> Model {
        crate::shared::screen().model(self.vertices.as_slice(), self.fans)
    }
}
