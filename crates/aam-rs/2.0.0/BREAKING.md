# AAM-RS Architecture Refactoring — From Monolithic to Pipeline

## Overview

The AAM-RS codebase has been refactored from a monolithic `AAML` struct into a clean five-stage pipeline architecture. This document explains the new design, component responsibilities, and how to extend or customize the pipeline.

## Architecture Diagram

```
Text Input
    ↓
┌───────────────────────────────────────────────────────────────┐
│                          Pipeline                             │
├───────────────────────────────────────────────────────────────┤
│                                                               │
│  Stage 1: Lexer          →  Token Stream (Vec<Token>)       │
│  ├─ Tokenizes raw text                                       │
│  ├─ Preserves line/column info for error diagnostics        │
│  └─ Produces: Vec<Token>                                    │
│                                                               │
│  Stage 2: Parser + ScopeManager  →  AST (Vec<AstNode>)     │
│  ├─ Consumes tokens                                          │
│  ├─ Builds Abstract Syntax Tree                             │
│  ├─ Tracks nesting depth and block context                  │
│  └─ Produces: Vec<AstNode>                                  │
│                                                               │
│  Stage 3: Validator  →  Validation Result                   │
│  ├─ Applies semantic checks                                  │
│  ├─ Validates type and schema constraints                   │
│  └─ Produces: Result<(), AamlError>                         │
│                                                               │
│  Stage 4: Executer  →  PipelineOutput                       │
│  ├─ Executes directives (@import, @schema, @type, @derive) │
│  ├─ Populates final key-value map                           │
│  └─ Produces: PipelineOutput                                │
│                                                               │
└───────────────────────────────────────────────────────────────┘
    ↓
Final State
├─ Key-Value Map
├─ Schema Definitions
└─ Type Registry


Public API
├─ AAML::parse() — backward-compatible facade
├─ AAML::load() — backward-compatible facade
├─ AAML::find_obj() — query API (unchanged)
├─ AAML::find_deep() — query API (unchanged)
└─ Pipeline::process() — new direct pipeline access
```

## Component Details

### 1. Lexer Stage (`src/pipeline/lexer.rs`)

**Responsibility:** Tokenize raw AAML text into a flat stream of tokens.

**Input:** `&str` (raw file content)  
**Output:** `Result<Vec<Token>, AamlError>`

**Key Types:**
- `Token` — single token with kind, line, column, and text
- `TokenKind` — enum: Identifier, Assign, String, Number, Boolean, LeftBrace, RightBrace, LeftBracket, RightBracket, Comma, At, Newline, Comment
- `Lexer` trait — implement this for custom lexers
- `DefaultLexer` — production implementation

**Features:**
- Character-by-character scanning
- Line/column tracking for precise error diagnostics
- String literal handling (quoted with `"` or `'`)
- Number and boolean literal recognition
- Comment recognition (`#`)
- Directive prefix (`@`) tokenization
- Inline object/list bracket matching

**Example:**
```rust
use aam_rs::pipeline::DefaultLexer;

let lexer = DefaultLexer::new();
let tokens = lexer.tokenize("host = localhost")?;
// tokens: [Identifier("host"), Assign, Identifier("localhost"), Newline]
```

### 2. Parser Stage (`src/pipeline/parser.rs`)

**Responsibility:** Parse tokens into an Abstract Syntax Tree.

**Input:** `Vec<Token>`  
**Output:** `Result<Vec<AstNode>, AamlError>`

**Key Types:**
- `AstNode` — enum representing parsed statements
  - `Assignment { key, value, line }`
  - `Directive { name, args, line }`
  - `InlineObject { pairs, line }`
  - `InlineList { items, line }`
- `Parser` trait — implement for custom parsers
- `DefaultParser` — production implementation

**Features:**
- Token consumption and validation
- Multi-line directive accumulation
- Inline object/list parsing
- Balanced delimiter checking
- Line number preservation

**Example:**
```rust
use aam_rs::pipeline::{DefaultLexer, DefaultParser, Lexer, Parser};

let lexer = DefaultLexer::new();
let tokens = lexer.tokenize("@import base.aam")?;

let parser = DefaultParser::new();
let ast = parser.parse(tokens)?;
// ast: [Directive { name: "import", args: "base.aam", line: 1 }]
```

### 3. ScopeManager (`src/pipeline/scope_manager.rs`)

**Responsibility:** Track syntactic nesting context during parsing.

**Usage:** Used internally by the Parser to manage block nesting state.

**Key Types:**
- `ScopeManager` — manages nesting depth and accumulated content
- `BlockType` — enum: None, Object, List, DirectiveBlock

**Features:**
- Tracks brace/bracket nesting depth
- Accumulates multi-line directive content
- Detects balanced block completeness
- Provides block context for semantic analysis

**Example:**
```rust
use aam_rs::pipeline::scope_manager::{ScopeManager, BlockType};

let mut scope = ScopeManager::new();
scope.enter_block(BlockType::Object);
assert_eq!(scope.nesting_depth(), 1);
scope.exit_block().unwrap();
assert_eq!(scope.nesting_depth(), 0);
```

### 4. Validator Stage (`src/pipeline/validator.rs`)

**Responsibility:** Perform semantic validation on the AST.

**Input:** `&[AstNode]`  
**Output:** `Result<(), AamlError>`

**Key Types:**
- `Validator` trait — implement for custom validators
- `DefaultValidator` — production implementation

**Checks Performed:**
- Non-empty keys in assignments
- Non-empty directive names
- Balanced braces and brackets in values
- Basic syntactic correctness

**Note:** Schema/type validation happens in the Executer after directives are processed, since schemas and types are defined via `@schema` and `@type` directives.

**Example:**
```rust
use aam_rs::pipeline::{DefaultLexer, DefaultParser, DefaultValidator, Lexer, Parser, Validator};

let lexer = DefaultLexer::new();
let tokens = lexer.tokenize("key = value")?;
let parser = DefaultParser::new();
let ast = parser.parse(tokens)?;

let validator = DefaultValidator::new();
validator.validate(&ast)?;  // Returns Ok(()) if valid
```

### 5. Executer Stage (`src/pipeline/executer.rs`)

**Responsibility:** Execute directives and populate the final runtime state.

**Input:** `Vec<AstNode>`  
**Output:** `Result<PipelineOutput, AamlError>`

**Key Types:**
- `Executer` trait — implement for custom executers
- `DefaultExecuter` — production implementation
- `ExecutionDescriptor` — metadata about what was executed
- `PipelineOutput` — final state (map, schemas, types)

**Workflow:**
1. Create an `AAML` instance with default command handlers
2. Iterate through AST nodes in order
3. For assignments: store in the key-value map
4. For directives: find and execute registered command handler
5. Return `PipelineOutput` with final state

**Example:**
```rust
use aam_rs::pipeline::{DefaultExecuter, Executer, AstNode};

let ast = vec![
    AstNode::Assignment { key: "a".to_string(), value: "b".to_string(), line: 1 },
];

let executer = DefaultExecuter::new();
let output = executer.execute(ast)?;
assert!(output.map.contains_key("a"));
```

### 6. Pipeline Orchestrator (`src/pipeline/mod.rs`)

**Responsibility:** Coordinate all five stages and provide the unified entry point.

**Key Types:**
- `Pipeline` — orchestrator
- `PipelineOutput` — final result

**Main Method:**
```rust
pub fn process(&self, content: &str) -> Result<PipelineOutput, AamlError>
```

**Example:**
```rust
use aam_rs::pipeline::Pipeline;

let pipeline = Pipeline::new();
let output = pipeline.process("host = localhost\nport = 8080")?;

println!("Keys: {:?}", output.map.keys().collect::<Vec<_>>());
// Keys: ["host", "port"]
```

## Backward Compatibility

The existing `AAML` public API remains unchanged and is now a **thin facade** over the pipeline:

```rust
use aam_rs::aaml::AAML;

// Old API still works exactly the same
let cfg = AAML::parse("host = localhost")?;
let value = cfg.find_obj("host")?;
```

**Implementation:**
- `AAML::parse()` and `AAML::load()` internally use `Pipeline::process()`
- `AAML::find_obj()` and `AAML::find_deep()` query the finalized map (unchanged)
- All existing directives and commands work identically
- All v1 syntax remains supported

## Error Handling

Each pipeline stage can produce specialized errors:

**Lexer Errors:**
- `AamlError::LexError` — invalid character or unclosed delimiter

**Parser Errors:**
- `AamlError::ParseError` — malformed syntax, missing operators, unbalanced braces

**Validator Errors:**
- `AamlError::ParseError` — semantic violations (empty keys, unbalanced delimiters)

**Executer Errors:**
- `AamlError::DirectiveError` — unknown or malformed directive
- `AamlError::SchemaValidationError` — schema constraint violated
- `AamlError::InvalidType` — type validation failed

**Line Number Tracking:**
Each error carries the source line number for precise diagnostics.

## Extending the Pipeline

### Custom Lexer

Implement the `Lexer` trait:

```rust
use aam_rs::pipeline::{Lexer, Token};
use crate::error::AamlError;

struct MyCustomLexer;

impl Lexer for MyCustomLexer {
    fn tokenize(&self, content: &str) -> Result<Vec<Token>, AamlError> {
        // Your tokenization logic
        todo!()
    }
}
```

### Custom Parser

Implement the `Parser` trait:

```rust
use aam_rs::pipeline::{Parser, Token, AstNode};
use crate::error::AamlError;

struct MyCustomParser;

impl Parser for MyCustomParser {
    fn parse(&self, tokens: Vec<Token>) -> Result<Vec<AstNode>, AamlError> {
        // Your parsing logic
        todo!()
    }
}
```

### Custom Validator

Implement the `Validator` trait:

```rust
use aam_rs::pipeline::{Validator, AstNode};
use crate::error::AamlError;

struct MyCustomValidator;

impl Validator for MyCustomValidator {
    fn validate(&self, ast: &[AstNode]) -> Result<(), AamlError> {
        // Your validation logic
        todo!()
    }
}
```

### Custom Executer

Implement the `Executer` trait:

```rust
use aam_rs::pipeline::{Executer, AstNode, PipelineOutput};
use crate::error::AamlError;

struct MyCustomExecuter;

impl Executer for MyCustomExecuter {
    fn execute(&self, ast: Vec<AstNode>) -> Result<PipelineOutput, AamlError> {
        // Your execution logic
        todo!()
    }
}
```

### Using Custom Stages

The `Pipeline` struct currently uses `Box<dyn Trait>` for dependency injection. To use custom implementations, you would need to make `Pipeline::new()` flexible or create new orchestrator methods. For now, you can instantiate stages independently:

```rust
let lexer = MyCustomLexer;
let tokens = lexer.tokenize(content)?;

let parser = MyCustomParser;
let ast = parser.parse(tokens)?;

let validator = MyCustomValidator;
validator.validate(&ast)?;

let executer = MyCustomExecuter;
let output = executer.execute(ast)?;
```

## Testing

Each stage has independent unit tests:

```bash
# Run all pipeline tests
cargo test pipeline::

# Run specific stage tests
cargo test pipeline::lexer::
cargo test pipeline::parser::
cargo test pipeline::scope_manager::
cargo test pipeline::validator::
cargo test pipeline::executer::
```

Integration tests verify end-to-end behavior:

```bash
# Run all integration tests (existing v1 tests still pass)
cargo test --test '*'
```

## Performance Considerations

The pipeline adds a small overhead by introducing an intermediate AST representation:

- **Lexer:** O(n) single pass
- **Parser:** O(n) token consumption
- **Validator:** O(n) AST traversal
- **Executer:** O(n) AST traversal + directive execution

**Total:** O(n) overall, comparable to the original monolithic approach.

For large files, the AST is memory-efficient (enum variants are small). Stress tests with the `standard_stress.rs` example verify acceptable performance.

## Migration Guide (For Contributors)

If you're updating existing code:

1. **Parsing logic:** Move to `lexer.rs` (tokenization) or `parser.rs` (AST building)
2. **Validation logic:** Move to `validator.rs` (semantic checks)
3. **Directive execution:** Move to `executer.rs` (command handlers in `src/commands/`)
4. **Query API:** Keep in `src/aaml/lookup.rs` (unchanged)
5. **Type registration:** Remains in `src/aaml/types_registry.rs` (unchanged)

## FAQ

**Q: Is the old monolithic approach still available?**
A: No, internally everything uses the pipeline. But the public `AAML` API remains unchanged, so user code is unaffected.

**Q: Can I mix custom and default stages?**
A: Yes, you can instantiate individual stages and chain them manually. See "Using Custom Stages" above.

**Q: Where are the command handlers (@import, @schema, etc.) implemented?**
A: In `src/commands/` (unchanged from v1). They're registered in `AAML::register_default_commands()` and invoked by the Executer.

**Q: How do I add a new builtin type?**
A: Implement the `Type` trait in `src/types/` and register it in `resolve_builtin()` (unchanged from v1).

**Q: Why separate Parser and ScopeManager?**
A: `ScopeManager` is a helper for tracking syntactic nesting. It could be internal to the Parser, but separating it allows for clearer testing and potential reuse.

## Related Files

- **Pipeline module:** `src/pipeline/`
- **AAML facade:** `src/aaml/mod.rs`
- **Error types:** `src/error.rs`
- **Commands:** `src/commands/`
- **Types:** `src/types/`
- **Integration tests:** `tests/`

