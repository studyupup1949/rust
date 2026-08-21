use super::*;

#[cfg(feature = "wolfram")]
use graph_types::{ToWolfram, WolframValue};

impl Display for StaticUndirected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size = self.max_degree().to_string().len();
        let max = self.nodes();
        for i in 0..max {
            for j in 0..max {
                if j > i {
                    write!(f, "{:width$} ", ".", width = size)?;
                }
                else {
                    let index = i * (i + 1) / 2 + j;
                    let edge = unsafe { self.edges.get_unchecked(index) };
                    write!(f, "{:width$} ", edge, width = size)?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(feature = "wolfram")]
impl ToWolfram for StaticUndirected {
    fn to_wolfram(&self) -> WolframValue {
        let nodes = self.nodes();
        let mut rows = Vec::with_capacity(nodes);
        for i in 0..nodes {
            let mut row = Vec::with_capacity(nodes);
            for j in 0..nodes {
                if j > i {
                    let index = j * (j + 1) / 2 + i;
                    let edge = unsafe { self.edges.get_unchecked(index) };
                    row.push(WolframValue::Integer64(*edge as i64));
                }
                else {
                    let index = i * (i + 1) / 2 + j;
                    let edge = unsafe { self.edges.get_unchecked(index) };
                    row.push(WolframValue::Integer64(*edge as i64));
                }
            }
            rows.push(WolframValue::list(row));
        }
        WolframValue::function("AdjacencyGraph", vec![WolframValue::list(rows)])
    }
}
