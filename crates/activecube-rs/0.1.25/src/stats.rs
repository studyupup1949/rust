use std::sync::Arc;

use crate::compiler::ir::{QueryIR, SelectExpr, FilterNode};

/// Query-level metadata collected after resolver execution.
/// Application layer uses this for observability, billing, or logging.
/// This struct intentionally contains no cost/credit/price fields —
/// billing logic belongs in the application, not the generic library.
#[derive(Debug, Clone)]
pub struct QueryStats {
    pub cube: String,
    pub table: String,
    pub limit: u32,
    pub offset: u32,
    pub select_count: usize,
    pub aggregate_count: usize,
    pub has_group_by: bool,
    pub has_having: bool,
    pub filter_depth: usize,
    pub row_count: usize,
    pub sql_generated: String,
}

impl QueryStats {
    /// Build stats from a validated QueryIR and execution results.
    pub fn from_ir(ir: &QueryIR, row_count: usize, sql: &str) -> Self {
        let aggregate_count = ir.selects.iter()
            .filter(|s| matches!(s, SelectExpr::Aggregate { .. }))
            .count();

        Self {
            cube: ir.cube.clone(),
            table: ir.table.clone(),
            limit: ir.limit,
            offset: ir.offset,
            select_count: ir.selects.len(),
            aggregate_count,
            has_group_by: !ir.group_by.is_empty(),
            has_having: !ir.having.is_empty(),
            filter_depth: filter_depth(&ir.filters),
            row_count,
            sql_generated: sql.to_string(),
        }
    }
}

fn filter_depth(node: &FilterNode) -> usize {
    match node {
        FilterNode::Empty | FilterNode::Condition { .. } => 1,
        FilterNode::And(children) | FilterNode::Or(children) => {
            1 + children.iter().map(filter_depth).max().unwrap_or(0)
        }
        FilterNode::ArrayIncludes { element_conditions, .. } => {
            1 + element_conditions.iter()
                .flat_map(|conds| conds.iter().map(filter_depth))
                .max()
                .unwrap_or(0)
        }
    }
}

/// Callback invoked after each cube query execution with query metadata.
/// The application layer registers this to collect stats for billing, logging, etc.
pub type StatsCallback = Arc<dyn Fn(QueryStats) + Send + Sync>;
