# activecube-rs

A generic **GraphQL-to-SQL OLAP query engine** library for Rust.

## Overview

activecube-rs provides a declarative way to expose analytical databases (ClickHouse, etc.) through GraphQL. Define your data model as **Cubes** with dimensions, metrics, and selectors — the library generates a full GraphQL schema and compiles queries into parameterized SQL.

```
GraphQL Query → Parser → QueryIR → SqlDialect → Parameterized SQL
```

## Core Concepts

| Concept | Description |
|---------|-------------|
| **Cube** | Maps a GraphQL type to a database table with dimensions, metrics, and filters |
| **Dimension** | A queryable field (column) that can be nested into groups |
| **Metric** | An aggregate function (count, sum, avg, min, max, uniq) |
| **Selector** | A named top-level filter argument on a cube |
| **SqlDialect** | Pluggable SQL generation backend (ClickHouse included) |

## Quick Start

```rust
use activecube_rs::*;
use std::sync::Arc;

// 1. Define a Cube
let cube = CubeBuilder::new("DEXTrades")
    .schema("dexes_dwd")
    .table("{chain}_trades")
    .dimension(dim_group("Block", vec![
        dim("Height", "block_height", DimType::Int),
        dim("Timestamp", "block_timestamp", DimType::DateTime),
    ]))
    .dimension(dim_group("Trade", vec![
        dim("Amount", "token_amount", DimType::Float),
        dim("AmountInUSD", "token_amount_in_usd", DimType::Float),
    ]))
    .metrics(&["count", "sum", "avg"])
    .selector(selector("date", "block_timestamp", DimType::DateTime))
    .build();

// 2. Create registry + dialect + executor
let registry = CubeRegistry::from_cubes(vec![cube]);
let dialect: Arc<dyn SqlDialect> = Arc::new(ClickHouseDialect::new());
let executor: QueryExecutor = Arc::new(|sql, bindings| {
    Box::pin(async move {
        // Execute SQL against your database and return rows
        Ok(vec![])
    })
});

// 3. Build the GraphQL schema
let schema = build_schema(registry, dialect, executor, SchemaConfig::default()).unwrap();

// Now serve `schema` via any async-graphql HTTP integration (Axum, Actix, etc.)
```

## GraphQL Query Example

```graphql
{
  ChainStream(network: sol) {
    DEXTrades(
      where: {
        Trade: { AmountInUSD: { gt: 1000 } }
        Block: { Timestamp: { since: "2024-01-01" } }
      }
      limit: { count: 10 }
      orderBy: Trade_AmountInUSD_DESC
    ) {
      Block { Height, Timestamp }
      Trade { Amount, AmountInUSD }
      count(of: "Trade_Amount")
    }
  }
}
```

Generated SQL:
```sql
SELECT `block_height`, `block_timestamp`, `token_amount`, `token_amount_in_usd`,
       count(`token_amount`) AS `__count`
FROM `dexes_dwd`.`sol_trades`
WHERE (`token_amount_in_usd` > ? AND `block_timestamp` >= ?)
GROUP BY `block_height`, `block_timestamp`, `token_amount`, `token_amount_in_usd`
ORDER BY `token_amount_in_usd` DESC
LIMIT 10
-- bindings: [1000, "2024-01-01"]
```

## Features

- **Dynamic GraphQL Schema** — generated at startup from Cube definitions using `async-graphql` dynamic schema
- **Nested Filters** — `where` with arbitrary nesting, `any` for OR conditions, `isNull` support
- **Metrics** — count, sum, avg, min, max, uniq (COUNT DISTINCT) with `selectWhere` (HAVING)
- **Selectors** — named top-level filter arguments beyond dimension-based `where`
- **SqlDialect trait** — pluggable SQL generation (ClickHouse included)
- **Parameterized Queries** — all values use `?` placeholders, never interpolated
- **Chain-aware Tables** — `{chain}` placeholder in table names, or `chain_column` for shared tables
- **Validation** — query depth limits, max result size, field whitelist
- **Array Dimensions** — parallel Array columns with `includes` filter (arrayExists)
- **Union Types** — GraphQL Union for polymorphic values (e.g. ABI argument types)

## Architecture

```
activecube-rs/
  src/
    cube/           # CubeDefinition, CubeBuilder, CubeRegistry
    schema/         # GraphQL schema generation + filter types
    compiler/       # QueryIR, parser, filter engine, validator
    sql/            # SqlDialect trait + ClickHouse implementation
    response/       # RowMap, QueryResult
```

## License

MIT
