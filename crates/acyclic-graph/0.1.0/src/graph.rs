use fnv::{FnvHashMap, FnvHasher};
use std::hash::{BuildHasherDefault, Hash};

pub struct Graph<V, E> {
    vertices: FnvHashMap<V, Vec<E>>,
}

impl<V, E> Graph<V, E>
where
    V: Eq + Hash,
    E: Eq,
{
    pub fn new() -> Graph<V, E> {
        Graph {
            vertices: FnvHashMap::default(),
        }
    }

    pub fn with_capacity_and_hasher(
        capacity: usize,
        hash_builder: BuildHasherDefault<FnvHasher>,
    ) -> Graph<V, E> {
        Graph {
            vertices: FnvHashMap::with_capacity_and_hasher(capacity, hash_builder),
        }
    }

    pub fn insert_vertice(&mut self, vertice: V) {
        self.vertices.insert(vertice, Vec::new());
    }

    pub fn remove_edge(&mut self, vertice: &V, edge: &E) {
        if let Some(vert) = self.vertices.get_mut(vertice) {
            vert.retain(|e| e != edge);
        };
    }

    pub fn add_edge(&mut self, vertice: &V, edge: E) {
        if let Some(vert) = self.vertices.get_mut(vertice) {
            vert.push(edge);
        }
    }
}
