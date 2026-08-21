pub fn complete_graph_data(n: usize) -> (Vec<u32>, Vec<(u32, u32)>) {
    let nodes: Vec<u32> = (0..n as u32).collect();

    let mut edges = Vec::new();
    for &src in &nodes {
        for &dst in &nodes {
            if src != dst {
                edges.push((src, dst));
            }
        }
    }

    (nodes, edges)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_graph_data() {
        let (nodes, edges) = complete_graph_data(3);
        assert_eq!(nodes.len(), 3);
        assert_eq!(edges.len(), 3 * 2);

        let (nodes, edges) = complete_graph_data(4);
        assert_eq!(nodes.len(), 4);
        assert_eq!(edges.len(), 4 * 3);

        let (nodes, edges) = complete_graph_data(7);
        assert_eq!(nodes.len(), 7);
        assert_eq!(edges.len(), 7 * 6);
    }
}
