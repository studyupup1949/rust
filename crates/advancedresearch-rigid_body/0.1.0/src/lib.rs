// # Rigid Body
//
// A minimalistic rigid body library

/// The type of attitude for orientation, torque and wrench.
pub type Attitude<T> = (T, [T; 3]);
/// Reexport 3D vector from vecmath.
pub use vecmath::Vector3;

use vecmath::traits::Float;

/// A minimalistic description of a rigid body.
#[derive(Clone, Copy, Debug)]
pub struct RigidBody<T> {
    /// Linear position.
    pub pos: Vector3<T>,
    /// Linear velocity.
    pub vel: Vector3<T>,
    /// Linear acceleration.
    pub acc: Vector3<T>,
    /// Orientation (angular position).
    pub ori: Attitude<T>,
    /// Torque (angular velocity).
    pub tor: Attitude<T>,
    /// Wrench (angular acceleration).
    pub wre: Attitude<T>,
}

impl<T: Float> RigidBody<T> {
    /// Updates linear coordinates by moving through time.
    pub fn update_linear(&mut self, dt: T) {
        use vecmath::vec3_add as add;
        use vecmath::vec3_scale as scale;

        let half_dt = T::from_f64(0.5) * dt;
        self.vel = add(self.vel, scale(self.acc, half_dt));
        self.pos = add(self.pos, scale(self.vel, dt));
        self.vel = add(self.vel, scale(self.acc, half_dt));
    }

    /// Updates angular coordinates by moving through time.
    pub fn update_angular(&mut self, dt: T) {
        let half_dt = T::from_f64(0.5) * dt;
        self.tor = angular(self.tor, self.wre, half_dt);
        self.ori = angular(self.ori, self.tor, dt);
        self.tor = angular(self.tor, self.wre, half_dt);
    }

    /// Update coordinates by moving it through time.
    pub fn update(&mut self, dt: T) {
        self.update_linear(dt);
        self.update_angular(dt);
    }
}

/// Solves the analogue of `s' = s + v * t` for attitude.
pub fn angular<T: Float>(a: Attitude<T>, b: Attitude<T>, t: T) -> Attitude<T> {
    use vecmath::vec3_scale as scale;
    use vecmath::vec3_dot as dot;
    use vecmath::vec3_cross as cross;
    use vecmath::vec3_add as add;

    let angle = a.0 + dot(a.1, b.1) * b.0 * t;
    let cos = b.0.cos();
    let sin = b.0.sin();
    // Uses Rodigrues' rotation formula.
    let axis = add(scale(a.1, cos), add(scale(cross(b.1, a.1), sin),
               scale(b.1, dot(b.1, a.1) * (T::one() - cos))));
    (angle, axis)
}
