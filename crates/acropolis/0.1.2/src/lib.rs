//! Acropolis OS is a new OS built by Knott Dynamics.

/// Returns the crate name.
pub fn name() -> &'static str {
    "acropolis"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_name() {
        assert_eq!(name(), "acropolis");
    }
}
