use crate::implementations::Graph;
use crate::{EdgeTrait, GraphViewTrait, NodeTrait};
use std::collections::HashMap;

struct KeyIndexMap {
    key_to_index: HashMap<u32, u32>,
    index_to_key: Vec<u32>,
}

fn create_key_index_map(keys: impl Iterator<Item = u32>) -> KeyIndexMap {
    let mut key_to_index: HashMap<u32, u32> = HashMap::new();
    let mut index_to_key: Vec<u32> = Vec::new();

    for (index, key) in keys.enumerate() {
        key_to_index.insert(key, index as u32);
        index_to_key.push(key);
    }

    KeyIndexMap {
        key_to_index,
        index_to_key,
    }
}

fn adjacency_structure<N: NodeTrait, E: EdgeTrait>(
    graph: &Graph<N, E>,
    key_to_index: &HashMap<u32, u32>,
) -> Graph<N, E> {
    let nodes = graph
        .get_nodes()
        .map(|node| {
            let index = key_to_index[&node.key()];
            N::new(index)
        })
        .collect();

    let edges = graph
        .get_edges()
        .map(|edge| {
            let source = key_to_index[&edge.source()];
            let target = key_to_index[&edge.target()];
            E::new(source, target)
        })
        .collect();

    Graph::new(nodes, edges)
}

pub enum GraphRefOrOwned<'a, N: NodeTrait, E: EdgeTrait> {
    Borrowed(&'a Graph<N, E>),
    Owned(Graph<N, E>),
}

impl<'a, N: NodeTrait, E: EdgeTrait> GraphRefOrOwned<'a, N, E> {
    pub fn as_ref(&self) -> &Graph<N, E> {
        match self {
            GraphRefOrOwned::Borrowed(g) => g,
            GraphRefOrOwned::Owned(g) => g,
        }
    }
}

pub fn normalize_graph_keys<'a, N: NodeTrait, E: EdgeTrait>(
    graph: &'a Graph<N, E>,
) -> (GraphRefOrOwned<'a, N, E>, Option<Vec<u32>>) {
    let n = match graph.get_nodes().count() {
        0 => return (GraphRefOrOwned::Borrowed(graph), None),
        len => (len - 1) as u32,
    };
    let min_key = graph.get_node_keys().min().unwrap();
    let max_key = graph.get_node_keys().max().unwrap();
    let keys_are_sequential = min_key == 0 && max_key == n;

    if keys_are_sequential {
        (GraphRefOrOwned::Borrowed(graph), None)
    } else {
        let KeyIndexMap {
            key_to_index,
            index_to_key,
        } = create_key_index_map(graph.get_node_keys());

        let new_graph = adjacency_structure(graph, &key_to_index);
        (GraphRefOrOwned::Owned(new_graph), Some(index_to_key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_key_index_map() {
        let keys = vec![1, 2, 3, 4];
        let map = create_key_index_map(keys.into_iter());

        assert_eq!(map.key_to_index.get(&1), Some(&0));
        assert_eq!(map.key_to_index.get(&2), Some(&1));
        assert_eq!(map.key_to_index.get(&3), Some(&2));
        assert_eq!(map.key_to_index.get(&4), Some(&3));

        assert_eq!(map.index_to_key[0], 1);
        assert_eq!(map.index_to_key[1], 2);
        assert_eq!(map.index_to_key[2], 3);
        assert_eq!(map.index_to_key[3], 4);
    }
}
