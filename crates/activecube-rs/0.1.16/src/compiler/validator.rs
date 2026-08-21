use crate::compiler::ir::{FilterNode, QueryIR};

const MAX_QUERY_DEPTH: usize = 6;
const ABSOLUTE_MAX_LIMIT: u32 = 10000;

pub fn validate(ir: QueryIR) -> Result<QueryIR, async_graphql::Error> {
    if ir.limit > ABSOLUTE_MAX_LIMIT {
        return Err(async_graphql::Error::new(format!(
            "Limit {} exceeds maximum allowed ({})", ir.limit, ABSOLUTE_MAX_LIMIT
        )));
    }

    let depth = filter_depth(&ir.filters);
    if depth > MAX_QUERY_DEPTH {
        return Err(async_graphql::Error::new(format!(
            "Filter nesting depth {} exceeds maximum ({})", depth, MAX_QUERY_DEPTH
        )));
    }

    let having_depth = filter_depth(&ir.having);
    if having_depth > MAX_QUERY_DEPTH {
        return Err(async_graphql::Error::new(format!(
            "HAVING filter depth {} exceeds maximum ({})", having_depth, MAX_QUERY_DEPTH
        )));
    }

    if ir.selects.is_empty() {
        return Err(async_graphql::Error::new("Query must select at least one field"));
    }

    Ok(ir)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::ir::*;

    fn minimal_ir() -> QueryIR {
        QueryIR {
            cube: "Test".into(), schema: "test".into(), table: "test_table".into(),
            selects: vec![SelectExpr::Column { column: "id".into(), alias: None }],
            filters: FilterNode::Empty, having: FilterNode::Empty,
            group_by: vec![], order_by: vec![], limit: 25, offset: 0,
            limit_by: None,
            use_final: false,
            joins: vec![],
            custom_query_builder: None,
            from_subquery: None,
        }
    }

    #[test]
    fn test_valid_ir_passes() { assert!(validate(minimal_ir()).is_ok()); }

    #[test]
    fn test_excessive_limit_rejected() {
        let mut ir = minimal_ir(); ir.limit = 99999;
        assert!(validate(ir).is_err());
    }

    #[test]
    fn test_empty_selects_rejected() {
        let mut ir = minimal_ir(); ir.selects.clear();
        assert!(validate(ir).is_err());
    }

    #[test]
    fn test_deep_filter_rejected() {
        fn deep_filter(depth: usize) -> FilterNode {
            if depth == 0 {
                FilterNode::Condition { column: "x".into(), op: CompareOp::Eq, value: SqlValue::Int(1) }
            } else {
                FilterNode::And(vec![deep_filter(depth - 1)])
            }
        }
        let mut ir = minimal_ir(); ir.filters = deep_filter(10);
        assert!(validate(ir).is_err());
    }
}
