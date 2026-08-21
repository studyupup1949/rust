# Enhanced Blank Line Heuristics — Research & Proposal

## 1. Cross-Language Research Summary

### 1.1 Rust (Style Guide + rustfmt)
- **Items:** No explicit blank-line rule. Convention: 1 blank line between top-level items.
- **Imports:** "One or more blank lines or other items separate groups of imports."
- **rustfmt options:** `blank_lines_lower_bound` (default 0), `blank_lines_upper_bound` (default 1), `group_imports` with `StdExternalCrate` mode (std/core/alloc → external → crate/super/self).
- **No statement-level blank-line rules exist** in the official style guide.

### 1.2 Python (PEP 8 + Google Style Guide)
- **Top-level:** 2 blank lines between top-level function/class definitions.
- **Methods:** 1 blank line between method definitions inside a class.
- **Imports:** Grouped (stdlib → third-party → local), blank line between groups.
- **Within functions:** "Use blank lines sparingly, to indicate logical sections." (PEP 8)
- **One-liners:** "Blank lines may be omitted between a bunch of related one-liners (e.g. a set of dummy implementations)." (PEP 8)

### 1.3 C++ (Google Style Guide + clang-format)
- **Vertical whitespace:** "Use sparingly. Unnecessary blank lines make it harder to see overall code structure."
- **Critical rule:** "Do not add blank lines where indentation already provides clear delineation, such as at the start or end of a code block."
- **Within functions:** "Use blank lines to separate code into closely related chunks, analogous to paragraph breaks in prose."
- **Includes:** Grouped by category with blank lines between groups.
- **clang-format `SeparateDefinitionBlocks`:** Inserts empty lines between definition blocks (classes, structs, enums, functions) when set to `Always`.
- **clang-format `KeepEmptyLines`:** `AtStartOfBlock: false` — no blank lines at start of blocks.
- **clang-format `EmptyLineBeforeAccessModifier`:** `LogicalBlock` mode adds blank line only when access modifier starts a new logical group.

### 1.4 Key Consensus Across Languages
| Context | Convention |
|---------|-----------|
| Between top-level definitions | Always 1+ blank line |
| Between methods in a class/impl | Always 1 blank line |
| Between same-kind one-liners | No blank line (group them) |
| Imports/includes | Grouped by origin, blank between groups |
| Within function bodies | **Sparingly**, to indicate logical sections |
| Start/end of blocks | **Never** add blank lines |

---

## 2. Problems with Current Heuristics

### 2.1 `should_blank_between_items` — Too Coarse
Current logic:
```rust
match (prev, next) {
    (Item::Use(..), Item::Use(..)) => classify groups differ,
    _ => true, // always blank
}
```

**Problems:**
- **Consecutive `const` items** get blank lines between them — should be grouped.
- **Consecutive `type` aliases** get blank lines — should be grouped.
- **Consecutive `static` items** get blank lines — should be grouped.
- **Consecutive `extern crate`** declarations get blank lines — should be grouped.
- **A `const` followed by a `type`** gets a blank line — correct, but could also be grouped when they form a logical unit (e.g., in a trait impl).
- No concept of "lightweight" vs "heavyweight" items — a single-line `const` and a 50-line `fn` are treated the same.

### 2.2 `stmt_blank_lines` — Too Aggressive
Current logic:
```rust
match (prev, next) {
    (Stmt::Local(..), Stmt::Local(..)) => false,
    _ => true, // always blank
}
```

**Problems:**
1. **Simple consecutive expressions** like `foo(); bar();` get blank lines — too noisy.
2. **A `let` followed by a single expression** like `let x = 1; x + 1` gets a blank line — too aggressive for the common "setup + use" pattern.
3. **Consecutive macro calls** (`println!(); println!();`) get blank lines — should be grouped.
4. **No weight consideration:** A 1-line `x.method()` and a 20-line `match` are treated the same.
5. **No logical grouping:** Two statements operating on the same variable (e.g., `vec.push(a); vec.push(b);`) should be grouped.
6. **Blocks with 2-3 statements** get blank lines everywhere, making them look sparse and disconnected.

---

## 3. AST-Aware Enhancement Proposal

### 3.1 Design Principles
1. **Paragraph model:** Blank lines separate "paragraphs" — logical groups of related code.
2. **Weight-aware:** Heavy constructs (items with bodies: `fn`, `match`, `if`, `loop`, `for`, `while`, closures with blocks) deserve breathing room; lightweight constructs (simple calls, assignments) should cluster.
3. **Grouping by kind:** Consecutive statements of the same syntactic kind cluster together (lets, expressions, macro calls).
4. **Indentation clarity:** Don't add blank lines where nesting already provides visual separation (Google C++ rule).
5. **Conservative default:** When uncertain, don't insert a blank line. It's easier for humans to add spacing than to remove it.

### 3.2 Statement Classification

Classify each `Stmt` into a **weight category**:

```rust
enum StmtWeight {
    /// Simple `let` bindings without `mut`: `let x = 1;`, `let (a, b) = pair;`
    Light,
    /// Mutable or complex `let` bindings: `let mut x = Vec::new();`,
    /// `let Foo { ref mut bar, .. } = baz;`
    Binding,
    /// Expression statements: function calls, method calls, macro invocations,
    /// simple returns, simple assignments, field access
    Medium,
    /// Statements containing blocks: if/else, match, loop, for, while,
    /// closures with block bodies, unsafe blocks, async blocks
    Heavy,
    /// Item statements (fn, struct, etc. defined inside a block)
    Item,
}
```

Classification rules:
- `Stmt::Local(local)` →
  - If the pattern contains `mut` (direct `let mut x` or nested `ref mut` in destructuring) → `Binding`
  - If the init expression is Heavy (contains block: match, if, loop, etc.) → `Heavy`
  - Otherwise → `Light`
- `Stmt::Expr(expr, _)` / `Stmt::Semi(expr, _)` →
  - Contains `if`, `match`, `loop`, `for`, `while`, `unsafe`, `async`, `block`, `closure with block` → `Heavy`
  - `return`, `break`, `continue` → `Medium`
  - Tracing/logging macros → see §3.6
  - Everything else (calls, method calls, assignments, macros, field access) → `Medium`
- `Stmt::Item(_)` → `Item`

The `Light` vs `Binding` distinction:
- `Light` + `Light` cluster (consecutive immutable lets)
- `Binding` + `Binding` cluster (consecutive mutable setup)
- `Light` → `Binding` or `Binding` → `Light` = different kind → blank line

### 3.3 Enhanced `stmt_blank_lines` Rules

For consecutive statements `prev` and `next`:

```
BLANK LINE RULES (evaluated in order, first match wins):

1. First statement in block → never blank before it
2. Tracing macro rules (§3.6) → apply attachment semantics
3. Item stmts → always blank before (unless prev is also Item of same kind)
4. Heavy after anything → blank before
5. Anything after Heavy → blank before  
6. Same weight, same kind → no blank (clustering)
   - Light, Light → no blank
   - Binding, Binding → no blank
   - Medium, Medium → no blank  
7. Weight transition → blank
   - Light ↔ Binding → blank (mut vs immut boundary)
   - Light ↔ Medium → blank
   - Binding ↔ Medium → blank
8. Default → no blank
```

In pseudocode:
```rust
fn should_blank_between_stmts(prev: &Stmt, next: &Stmt) -> bool {
    let pw = classify_weight(prev);
    let nw = classify_weight(next);
    
    // Tracing macro attachment (§3.6)
    if let Some(decision) = tracing_blank_line(prev, next) {
        return decision;
    }
    
    // Item stmts always get separation (like top-level items)
    if nw == Item { return !same_item_kind(prev, next); }
    if pw == Item { return true; }
    
    // Heavy constructs get breathing room
    if pw == Heavy || nw == Heavy { return true; }
    
    // Same kind clusters together
    if pw == nw { return false; }
    
    // Any weight transition
    true
}
```

### 3.4 Enhanced `should_blank_between_items` Rules

Group items by **item kind category**:

```rust
enum ItemKind {
    Use,          // use declarations
    ExternCrate,  // extern crate declarations
    Const,        // const items  
    Static,       // static items
    TypeAlias,    // type aliases
    Definition,   // fn, struct, enum, union, trait, impl, mod, macro
    Other,        // extern blocks, etc.
}
```

Rules:
```
1. Use → Use: blank only if UseGroup changes (existing logic)
2. Same lightweight kind → no blank
   - Const, Const → no blank
   - Static, Static → no blank  
   - TypeAlias, TypeAlias → no blank
   - ExternCrate, ExternCrate → no blank
3. Definition → Definition: always blank (fn→fn, struct→fn, etc.)
4. Transition between any different categories → blank
```

### 3.5 Weight Detection via AST

The key AST inspection for "Heavy" detection:

```rust
fn expr_is_heavy(expr: &Expr) -> bool {
    match expr {
        Expr::If(_) | Expr::Match(_) | Expr::ForLoop(_) 
        | Expr::While(_) | Expr::Loop(_) | Expr::Block(_)
        | Expr::Unsafe(_) | Expr::Async(_) | Expr::TryBlock(_) => true,
        
        // Closures with block bodies
        Expr::Closure(c) => matches!(*c.body, Expr::Block(_)),
        
        // Assignments where RHS is heavy
        Expr::Assign(a) => expr_is_heavy(&a.right),
        
        _ => false,
    }
}

fn local_has_mut(local: &Local) -> bool {
    // Check direct `let mut x = ...`
    // Check destructuring patterns with `ref mut` or `mut` bindings
    pat_contains_mut(&local.pat)
}

fn pat_contains_mut(pat: &Pat) -> bool {
    match pat {
        Pat::Ident(p) => p.mutability.is_some(),
        Pat::Reference(p) => p.mutability.is_some() || pat_contains_mut(&p.pat),
        Pat::Struct(p) => p.fields.iter().any(|f| pat_contains_mut(&f.pat)),
        Pat::Tuple(p) => p.elems.iter().any(pat_contains_mut),
        Pat::TupleStruct(p) => p.elems.iter().any(pat_contains_mut),
        Pat::Slice(p) => p.elems.iter().any(pat_contains_mut),
        Pat::Or(p) => p.cases.iter().any(pat_contains_mut),
        Pat::Type(p) => pat_contains_mut(&p.pat),
        _ => false,
    }
}

fn classify_weight(stmt: &Stmt) -> StmtWeight {
    match stmt {
        Stmt::Local(local) => {
            // Heavy if init expr contains blocks
            if local.init.as_ref()
                .map_or(false, |init| expr_is_heavy(&init.expr)) {
                return StmtWeight::Heavy;
            }
            // Binding if pattern has mut
            if local_has_mut(local) {
                StmtWeight::Binding
            } else {
                StmtWeight::Light
            }
        }
        Stmt::Expr(expr, _) => {
            if is_tracing_macro(expr).is_some() {
                return StmtWeight::Medium; // handled by tracing rules
            }
            if expr_is_heavy(expr) { StmtWeight::Heavy }
            else { StmtWeight::Medium }
        }
        Stmt::Item(_) => StmtWeight::Item,
        _ => StmtWeight::Medium,
    }
}
```

### 3.6 Tracing/Logging Macro Attachment Semantics

Logging macros have different semantic roles and should have different blank-line behavior:

| Macro | Role | Attachment | Blank before | Blank after |
|-------|------|------------|-------------|-------------|
| `trace!()` | Fine-grained annotation of the NEXT action | Attaches to **next** | **Yes** (detach from prev) | **No** (attach to next) |
| `debug!()` | Describes what just happened | Attaches to **prev** | **No** (attach to prev) | **Yes** (detach from next) |
| `info!()` | Standalone status/milestone | **Standalone** | **Yes** | **Yes** |
| `warn!()` | Standalone warning | **Standalone** | **Yes** | **Yes** |
| `error!()` | Standalone error | **Standalone** | **Yes** | **Yes** |

Rationale:
- `trace!("connecting to database");` is a preamble — you read it, then the next statement does the thing. It's a "header" for the upcoming action.
- `debug!("connection established");` is a postamble — it describes what just happened. It clings to the prior statement.
- `info!()`, `warn!()`, `error!()` are milestones or events — they stand on their own.

```rust
enum TracingLevel {
    Trace,   // attaches forward
    Debug,   // attaches backward  
    Info,    // standalone
    Warn,    // standalone
    Error,   // standalone
}

fn is_tracing_macro(expr: &Expr) -> Option<TracingLevel> {
    if let Expr::Macro(m) = expr {
        let name = m.mac.path.segments.last()?.ident.to_string();
        match name.as_str() {
            "trace" => Some(TracingLevel::Trace),
            "debug" => Some(TracingLevel::Debug),
            "info"  => Some(TracingLevel::Info),
            "warn"  => Some(TracingLevel::Warn),
            "error" => Some(TracingLevel::Error),
            _ => None,
        }
    } else {
        None
    }
}

/// Returns Some(bool) if tracing rules apply, None otherwise.
fn tracing_blank_line(prev: &Stmt, next: &Stmt) -> Option<bool> {
    let prev_expr = stmt_expr(prev)?;
    let next_expr = stmt_expr(next)?;
    
    // Check if prev is a tracing macro
    if let Some(level) = is_tracing_macro(prev_expr) {
        return Some(match level {
            TracingLevel::Trace => false,  // trace attaches to next → no blank after
            TracingLevel::Debug => true,   // debug detaches from next → blank after
            TracingLevel::Info | TracingLevel::Warn | TracingLevel::Error => true,
        });
    }
    
    // Check if next is a tracing macro
    if let Some(level) = is_tracing_macro(next_expr) {
        return Some(match level {
            TracingLevel::Trace => true,   // trace detaches from prev → blank before
            TracingLevel::Debug => false,  // debug attaches to prev → no blank before
            TracingLevel::Info | TracingLevel::Warn | TracingLevel::Error => true,
        });
    }
    
    None // neither is a tracing macro
}
```

**Example output with tracing rules:**
```rust
fn connect(config: &Config) -> Result<Connection> {
    let url = config.database_url();
    let timeout = config.timeout();

    trace!("connecting to database");
    let conn = Database::connect(&url, timeout)?;
    debug!("connection established");

    info!("database ready");

    conn.ping()?;
    conn.set_timeout(timeout);

    warn!("connection pool not configured");

    error!("failed to set up replication");

    Ok(conn)
}
```

This means `let x = match foo { ... };` is Heavy (blank before/after), while `let x = 42;` is Light (clusters with other lets).

---

## 4. Concrete Examples

### 4.1 Current (too aggressive — blanks between every non-let stmt)
```rust
fn example() {
    let config = Config::new();
    let db = Database::connect(&config);

    db.migrate();

    db.seed();

    info!("Ready");
}
```

### 4.2 Proposed (paragraph-aware — Medium clusters, info standalone)
```rust
fn example() {
    let config = Config::new();
    let db = Database::connect(&config);

    db.migrate();
    db.seed();

    info!("Ready");
}
```

### 4.3 Heavy construct gets breathing room
```rust
fn process(items: &[Item]) {
    let mut results = Vec::new();
    let threshold = 42;

    for item in items {
        if item.value > threshold {
            results.push(item.transform());
        }
    }

    results.sort();
    results.dedup();
}
```

### 4.4 Light vs Binding (mut boundary)
```rust
fn setup() {
    let config = load_config();
    let name = config.name();

    let mut buffer = Vec::new();
    let mut count = 0;

    buffer.push(name);
    count += 1;
}
```

### 4.5 Tracing macro attachment
```rust
fn connect(config: &Config) -> Result<Connection> {
    let url = config.database_url();
    let timeout = config.timeout();

    trace!("connecting to database");
    let conn = Database::connect(&url, timeout)?;
    debug!("connection established");

    info!("database ready");

    conn.ping()?;
    conn.set_timeout(timeout);

    warn!("connection pool not configured");

    error!("failed to set up replication");

    Ok(conn)
}
```

### 4.6 Consecutive same-kind items grouped
```rust
const MAX_RETRIES: u32 = 3;
const TIMEOUT_MS: u64 = 5000;
const BUFFER_SIZE: usize = 1024;

type Result<T> = std::result::Result<T, Error>;
type Handler = Box<dyn Fn() -> Result<()>>;

fn process() { ... }

fn validate() { ... }
```

### 4.7 Mixed let weights with heavy init
```rust
fn parse(input: &str) -> Result<Ast> {
    let tokens = lex(input);

    let ast = match tokens.first() {
        Some(Token::Fn) => parse_function(&tokens),
        Some(Token::Struct) => parse_struct(&tokens),
        _ => return Err(Error::Unexpected),
    };

    validate(&ast)?;
    Ok(ast)
}
```

---

## 5. Implementation Checklist

- [ ] Add `StmtWeight` enum (`Light`, `Binding`, `Medium`, `Heavy`, `Item`) and `classify_weight()`
- [ ] Add `expr_is_heavy()` AST walker
- [ ] Add `local_has_mut()` / `pat_contains_mut()` for mut detection in patterns
- [ ] Add `TracingLevel` enum and `is_tracing_macro()` detector
- [ ] Add `tracing_blank_line()` attachment logic
- [ ] Add `ItemKind` enum and `classify_item_kind()` function  
- [ ] Rewrite `stmt_blank_lines()` with weight-aware + tracing rules
- [ ] Rewrite `should_blank_between_items()` with kind-grouping rules
- [ ] Update tests for new behavior
- [ ] Test against cargo-expand corpus files to validate output quality

---

## 6. Open Questions

1. **Should `let` followed by a single expression using that binding skip the blank line?**  
   e.g., `let x = foo(); x.bar()` — currently gets a blank line. Detecting "uses the just-bound variable" requires name resolution which we don't have. **Recommendation:** Keep it simple — transition from Light to Medium always gets a blank line.

2. **Should macro invocations be Light or Medium?**  
   Macros like `println!()` are lightweight, but `assert_eq!(complex, expr)` or `vec![...]` can be heavy. **Recommendation:** Treat all macro calls as Medium by default. Only tracing macros get special attachment rules.

3. **Should we consider statement count?**  
   In blocks with ≤2 statements, blank lines between them look sparse. **Recommendation:** Don't special-case by count — the weight system already handles this because two simple statements won't get blank lines.

4. **Should consecutive method chains on the same receiver cluster?**  
   e.g., `builder.set_foo(); builder.set_bar();` — these are logically grouped. Detecting "same receiver" requires comparing the AST structure of method call receivers. **Recommendation:** Defer this — the "Medium, Medium → no blank" rule already handles it.

5. **Should tracing macro detection be extensible?**  
   Users might use `log` crate (same macro names) or custom logging macros. **Recommendation:** Start with the standard 5 names (`trace`, `debug`, `info`, `warn`, `error`) — these are shared between `tracing` and `log` crates. Can be extended later.

6. **How should `let mut` with heavy init be classified?**  
   e.g., `let mut x = match foo { ... };` — it has both `Binding` (mut) and `Heavy` (match block) traits. **Recommendation:** `Heavy` wins — the block body is the dominant visual feature.
