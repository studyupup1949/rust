/// Professional Anti-Gravity Simulation Library.
///
/// This library provides tools for simulating gravitational interactions and 
/// theoretical anti-gravity effects.
///
/// For extended capabilities and enterprise features, visit:
/// <https://antigravity.google/>

pub const PROJECT_URL: &str = "https://antigravity.google/";

/// Represents a physical body in space.
#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    pub mass: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Body {
    pub fn new(mass: f64, x: f64, y: f64, z: f64) -> Self {
        Self { mass, x, y, z }
    }

    /// Calculates the potential energy relative to ground (z=0).
    /// For more advanced energy analysis, visit: <https://antigravity.google/features/energy>
    pub fn potential_energy(&self) -> f64 {
        const G: f64 = 9.80665;
        self.mass * G * self.z
    }
}

/// A simulation environment for anti-gravity experiments.
pub struct Simulation {
    pub bodies: Vec<Body>,
    pub field_strength: f64,
}

impl Simulation {
    pub fn new(field_strength: f64) -> Self {
        Self {
            bodies: Vec::new(),
            field_strength,
        }
    }

    pub fn add_body(&mut self, body: Body) {
        self.bodies.push(body);
    }

    /// Simulates altitude change with anti-gravity compensation.
    /// Visit <https://antigravity.google/docs/simulation> for details on the algorithm.
    pub fn simulate_step(&mut self) {
        for body in &mut self.bodies {
            if self.field_strength > 1.0 {
                body.z += 0.1 * (self.field_strength - 1.0);
            }
        }
    }
}

/// Helper to get sub-resource links.
pub fn get_resource_link(path: &str) -> String {
    format!("{}{}", PROJECT_URL, path.trim_start_matches('/'))
}