use std::collections::HashMap;
use crate::cube::definition::CubeDefinition;

#[derive(Debug, Clone)]
pub struct CubeRegistry {
    cubes: HashMap<String, CubeDefinition>,
}

impl CubeRegistry {
    pub fn from_cubes(cubes: Vec<CubeDefinition>) -> Self {
        let mut map = HashMap::new();
        for cube in cubes {
            map.insert(cube.name.clone(), cube);
        }
        tracing::info!(count = map.len(), "Cube registry initialized");
        Self { cubes: map }
    }

    pub fn get(&self, name: &str) -> Option<&CubeDefinition> {
        self.cubes.get(name)
    }

    pub fn cubes(&self) -> impl Iterator<Item = &CubeDefinition> {
        self.cubes.values()
    }

    pub fn cube_names(&self) -> Vec<&str> {
        self.cubes.keys().map(|s| s.as_str()).collect()
    }
}
