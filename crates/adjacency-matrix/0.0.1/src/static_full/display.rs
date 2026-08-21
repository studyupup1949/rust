use super::*;
#[cfg(feature = "wolfram")]
use graph_types::{ToWolfram, WolframValue};

impl Display for StaticDirected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size = self.max_degree().to_string().len();
        let max = self.nodes();
        for i in 0..max {
            for j in 0..max {
                let index = i * max + j;
                let edge = unsafe { self.edges.get_unchecked(index) };
                write!(f, "{:width$} ", edge, width = size)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(feature = "wolfram")]
impl ToWolfram for StaticDirected {
    fn to_wolfram(&self) -> WolframValue {
        let rows = self
            .edges
            .chunks_exact(self.nodes())
            .map(|row| WolframValue::list(row.iter().map(|edge| WolframValue::Integer64(*edge as i64)).collect()))
            .collect();
        WolframValue::function("AdjacencyGraph", vec![WolframValue::list(rows)])
    }
}
