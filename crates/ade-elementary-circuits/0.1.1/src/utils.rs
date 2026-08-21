// Helper function to compare two sets of circuits
pub fn circuits_equal(circuits1: &Vec<Vec<u32>>, circuits2: &Vec<Vec<u32>>) -> bool {
    let norm1 = normalize_circuits(circuits1);
    let norm2 = normalize_circuits(circuits2);
    norm1 == norm2
}

// Function to normalize a set of circuits
fn normalize_circuits(circuits: &Vec<Vec<u32>>) -> Vec<Vec<u32>> {
    let mut normalized_circuits: Vec<Vec<u32>> = circuits
        .iter()
        .map(|circuit| normalize_circuit(circuit))
        .collect();

    // Sort circuits for deterministic comparison
    normalized_circuits.sort();
    normalized_circuits
}

// Function to normalize a single circuit
// Find the node with minimum value and rotate the circuit to start from that node
fn normalize_circuit(circuit: &[u32]) -> Vec<u32> {
    if circuit.is_empty() {
        return Vec::new();
    }

    // Remove the last element if it equals the first (circuit closure)
    let nodes = if circuit.len() > 1 && circuit[0] == circuit[circuit.len() - 1] {
        &circuit[..circuit.len() - 1]
    } else {
        circuit
    };

    if nodes.is_empty() {
        return Vec::new();
    }

    // Find the index of the node with minimum value
    let min_value = *nodes.iter().min().unwrap();
    let min_index = nodes.iter().position(|&x| x == min_value).unwrap();

    // Rotate the circuit to start from the minimum node
    let mut normalized = Vec::new();
    for i in 0..nodes.len() {
        normalized.push(nodes[(min_index + i) % nodes.len()]);
    }

    // Add the initial node at the end to close the circuit
    normalized.push(normalized[0]);

    normalized
}

fn factorial(x: usize) -> usize {
    (1..=x).product()
}

// Return the number of elementary circuits in a complete directed graph with n nodes
pub fn number_circuits(n: usize) -> usize {
    let mut total = 0;

    for k in 2..=n {
        let numerator = factorial(n);
        let denominator = factorial(n - k) * k;
        total += numerator / denominator;
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_circuits() {
        assert_eq!(number_circuits(2), 1);
        assert_eq!(number_circuits(3), 5);
        assert_eq!(number_circuits(4), 20);
        assert_eq!(number_circuits(5), 84);
    }
}
