# a9-prettyplease vs rustfmt: Formatting Divergences

This document catalogs all known formatting divergences between a9-prettyplease (forked from dtolnay/prettyplease) and rustfmt, with root-cause analysis grounded in both codebases. The goal is to make a9-prettyplease's output identical to rustfmt's default configuration.

> **Note:** Upstream prettyplease explicitly does _not_ aim for rustfmt parity (see [#25](https://github.com/dtolnay/prettyplease/issues/25): "It is not a goal to match rustfmt exactly"). a9-prettyplease diverges from upstream by targeting exact rustfmt compatibility.

---

## Divergence 1: Method Chain Breaking

### Symptom
```rust
// rustfmt
self.violations.push((line, msg, true));

// a9-prettyplease
self.violations
    .push((line, msg, true));
```

### Root Cause

**rustfmt** (`src/chains.rs`): Collects the entire chain (`receiver.method1().method2()`) into a flat `Vec<ChainItem>`, then attempts to fit everything on one line. Only breaks to multi-line when the single-line version exceeds `max_width`. When breaking, it uses a "root absorption" strategy: short chain parents (e.g., `self.violations`) absorb the first child (`.push(...)`) onto the same line if the parent + child fit within `tab_width` characters. The key heuristic is in `ChainFormatterBlock::format_root`:
```
while root_rewrite.len() <= tab_width && !root_rewrite.contains('\n') {
    // absorb next child into root
}
```

**a9-prettyplease** (`src/expr.rs:755-800`): Uses the Wadler-Lindig algorithm with `cbox(INDENT)` wrapping. Method chains are printed via `prefix_subexpr_method_call` which inserts a `scan_break` before each `.method()` call. The break decision is: unless the receiver is a short ident (≤4 chars) at beginning-of-line, a breakpoint is inserted. Additionally, `end_with_max_width(60)` forces breaking if the chain body exceeds 60 chars — regardless of whether the whole expression fits within the 100-char margin.

### Fix Direction
The `end_with_max_width(60)` threshold is too aggressive. rustfmt only breaks chains when they exceed `max_width` (100). The receiver-length heuristic (≤4 chars) is also too narrow — rustfmt absorbs based on the full root width vs `tab_width` (4). The fix should:
1. Remove or raise `end_with_max_width(60)` for method chains
2. Match rustfmt's root-absorption: keep `receiver.method(...)` on one line when the receiver is not multi-line and the total fits within margin

---

## Divergence 2: Let-Chain Formatting in `if` Expressions

### Symptom
```rust
// rustfmt
if let Item::Use(u) = item
    && !has_cfg(&u.attrs)
{

// a9-prettyplease
if let Item::Use(u) = item && !has_cfg(&u.attrs) {
```

### Root Cause

**rustfmt** (`src/expr.rs`, `ControlFlow::rewrite_pat_expr`): Formats the condition via `rewrite_assign_rhs_with_comments`, which uses the standard RHS formatting logic. When the condition contains `&&` (let-chains), the formatting respects `max_width` and breaks each `&&` clause to its own line with indentation. The `ControlFlow` struct handles `if let ... && ...` as a single condition expression, but the rewriting treats `&&` as a binary operator that gets pair-formatted with line breaks.

**a9-prettyplease** (`src/expr.rs:143-175`): `expr_condition` checks `contains_let_chain(expr)`. When a let-chain is detected, it wraps in `cbox(INDENT)` and calls `let_chain_clauses` which inserts `self.space()` (breakable) between clauses and `self.nbsp()` (non-breaking) after `&&`. The outer `cbox` means Consistent breaking — all clauses break or none do. When the total fits within margin, **none break**, collapsing the chain to one line.

### Fix Direction
The consistent-break strategy is correct in principle, but the single-line threshold is wrong. rustfmt always breaks let-chains across lines when there are multiple `&&` clauses, regardless of line width. The fix should force breaking when there are ≥2 let-chain clauses (or when any clause contains `let`).

---

## Divergence 3: Unary `!` Operator Spacing

### Symptom
```rust
// rustfmt
assert!(!vs.is_empty());

// a9-prettyplease
assert!(! vs.is_empty());
```

### Root Cause

**rustfmt** (`src/expr.rs`, `rewrite_unary_op`): Calls `rewrite_unary_prefix(context, op.as_str(), expr, shape)` where `op.as_str()` returns `"!"`. `rewrite_unary_prefix` does `format!("{}{}", prefix, r)` — no space between operator and operand.

**a9-prettyplease** (`src/expr.rs:957-963`): `expr_unary` calls `unary_operator(&expr.op)` then `subexpr(...)`. `unary_operator` at line 1296 prints the bare `"!"` string. **No explicit space is inserted.**

However, the issue is likely in the **macro argument parsing**. When a9-prettyplease encounters `assert!(!vs.is_empty())`, the macro body `!vs.is_empty()` may be tokenized and reprinted with a space after `!` due to the token-stream printer inserting spaces between tokens. The `macro_rules_tokens` printer likely adds a space between the `!` token and the identifier token `vs`.

### Fix Direction
Investigate `macro_rules_tokens` in `src/mac.rs` — the token-level printer likely inserts a space between `!` (punctuation) and the next token. A special case is needed: when `!` is followed by an identifier or `(`, no space should be inserted (matching rustfmt's behavior for unary `!`).

---

## Divergence 4: Struct Literal Field Separator Spacing

### Symptom
```rust
// rustfmt
Violation { line: 0, message: "...".into(), fixable: false }

// a9-prettyplease
Violation { line : 0, message : "...".into(), fixable : false }
```

### Root Cause

**rustfmt** (`src/utils.rs`, `colon_spaces`): Uses `config.space_before_colon()` and `config.space_after_colon()` settings. Default is `(false, true)` → `": "` (no space before, space after).

**a9-prettyplease** (`src/expr.rs:1114-1123`): `field_value` prints `self.word(": ")` — hardcoded `": "` with space after, no space before. **This is correct.**

The issue must be in the same-line struct literal case. When struct fields are kept on one line (due to `end_with_max_width(18)`), the `trailing_comma_or_space` function produces `{blank_space: 1, pre_break: Some(',')}`. If the token printer is inserting spaces differently in the one-line case, or if the colon token is being emitted from the token stream rather than the structured printer... This needs investigation — the space before `:` may come from the macro token printer handling struct literals within macros like `vec![Violation { line: 0 }]`.

### Fix Direction
Check whether the struct literal appears inside a macro context where the token-level printer handles `:`. If so, the macro token printer needs to match `field: value` patterns and suppress the pre-colon space.

---

## Divergence 5: Array Literal Element Breaking

### Symptom
```rust
// rustfmt (dense, multi-per-line)
const NAMES: &[&str] = &[
    "i", "j", "k", "n", "x", "y", "z", "e", "f", "s", "r", "w",
    "tx", "rx", "ch", "db", "id", "fd", "fs", "io", "re", "wg",
];

// a9-prettyplease (one-per-line)
const NAMES: &[&str] = &[
    "i",
    "j",
    "k",
    ...
];
```

### Root Cause

**rustfmt** (`src/expr.rs`, `rewrite_array` → `overflow::rewrite_with_square_brackets`): Uses `definitive_tactic` which chooses between `HorizontalVertical` (pack as many per line as possible) and `Vertical` (one per line). For simple string literals, it typically uses `HorizontalVertical`, packing multiple items per line.

**a9-prettyplease** (`src/expr.rs:191-222`): Has two modes based on `simple_array()` which returns `true` only for `Lit::Byte | Char | Int | Bool`. **String literals (`Lit::Str`) are NOT considered simple.** Non-simple arrays use `cbox(INDENT)` with Consistent breaking — ALL elements on separate lines or ALL on one line. Since the array exceeds the margin, Consistent breaking puts each element on its own line.

### Fix Direction
1. Expand `simple_array()` to include `Lit::Str` (and possibly all literal types)
2. For simple arrays, the inner `ibox(0)` (Inconsistent) allows individual breaks, which will pack multiple items per line — matching rustfmt's `HorizontalVertical` tactic

---

## Divergence 6: Use Tree Grouping

### Symptom
```rust
// rustfmt
use std::{
    fs,
    path::{Path, PathBuf},
};

// a9-prettyplease
use std::{fs, path::{Path, PathBuf}};
```

### Root Cause

**rustfmt** (`src/imports.rs`): Uses `definitive_tactic` with `ListTactic::HorizontalVertical`. When nested `UseTree::Group` items are present, it factors in item complexity. For use groups with nested paths (`path::{Path, PathBuf}`), rustfmt tends to put each top-level item on its own line.

**a9-prettyplease** (`src/item.rs:776-808`): `use_group` uses `cbox(INDENT)` (Consistent). Items separated by `","` + `space()`, with `hardbreak()` only when an item ends with a nested UseGroup. The key decision: `hardbreak()` is inserted after items containing nested groups, but the outer `cbox` still uses Consistent breaking. If the total fits within margin, no breaks occur — everything stays on one line.

### Fix Direction
When a use-group contains items with nested sub-groups (e.g., `path::{Path, PathBuf}`), force multi-line layout to match rustfmt. The heuristic should be: if any item in the group is itself a `UseTree::Group`, use vertical layout.

---

## Divergence 7: Trailing Comma in Function-Like Macro Arguments

### Symptom
```rust
// rustfmt
format!("[BUG] fixable violation remains after enforce: {}", v.message)

// a9-prettyplease
format!("[BUG] fixable violation remains after enforce: {}", v.message,)
//                                                                    ^ trailing comma
```

### Root Cause

**rustfmt** (`src/expr.rs`, `choose_separator_tactic`): Inside macros, checks `span_ends_with_comma()` to preserve the original trailing comma status. If the original source had no trailing comma, rustfmt doesn't add one.

**a9-prettyplease** (`src/convenience.rs:67-90`): `trailing_comma(is_last)` emits `scan_break(BreakToken { pre_break: Some(','), ..default() })` for the last element. When the group **breaks** (multi-line), the comma is printed. When it **doesn't break** (single-line), `pre_break` is skipped and `blank_space: 0` means no space either. **But** — if the group is borderline and the algorithm decides to break, the trailing comma appears. This is especially problematic when clippy's `unnecessary_trailing_comma` lint (nursery) is enabled.

### Fix Direction
For macro arguments (especially format-like macros), do not emit trailing commas. The `trailing_comma` pattern should be replaced with a simple `word(",")` + `space()` for non-last items, and nothing for the last item in macro contexts. Alternatively, use `no_break: None` instead of `pre_break: Some(',')` for the last element in macro argument lists.

---

## Divergence 8: Function Call Argument Breaking

### Symptom
```rust
// rustfmt
errs.push(LintError::ParseError(format!(
    "failed to parse source: {e}"
)));

// a9-prettyplease
errs.push(
    LintError::ParseError(format!("failed to parse source: {e}")),
);
```

### Root Cause

**rustfmt** (`src/overflow.rs`): Has "overflow" logic that allows the last argument of a function call to extend past the call's closing paren. When the last argument is a nested call (like `format!(...)`), rustfmt tries to keep `push(Format(` on one line and lets the format macro's body wrap inside. This creates the characteristic `push(LintError::ParseError(format!(\n    "..."\n)));` pattern.

**a9-prettyplease**: Uses `cbox(INDENT)` + `zerobreak` for function arguments. When arguments don't fit on one line, **all** arguments break (Consistent). There is no special "overflow the last argument" heuristic. This means `push(\n    LintError::...,\n)` rather than `push(LintError::...(format!(\n    "..."\n)))`.

### Fix Direction
Implement a last-argument overflow heuristic: when the last (or only) argument of a function call is itself a call/macro/closure/block, try formatting it "overflowed" (on the same line as the opening paren) before falling back to the broken layout. This is a significant algorithmic change and is the most complex divergence to fix.

---

## Summary

| # | Divergence | Severity | Estimated Complexity | Root File(s) |
|---|-----------|----------|---------------------|---------------|
| 1 | Method chain breaking | High | Medium | `src/expr.rs` (prefix_subexpr_method_call, end_with_max_width) |
| 2 | Let-chain formatting | Medium | Low | `src/expr.rs` (expr_condition, let_chain_clauses) |
| 3 | Unary `!` spacing in macros | Medium | Low | `src/mac.rs` (macro_rules_tokens) |
| 4 | Struct field `:` spacing in macros | Low | Low | `src/mac.rs` or `src/expr.rs` (field_value in macro context) |
| 5 | Array literal element breaking | Medium | Low | `src/expr.rs` (simple_array) |
| 6 | Use tree grouping | Medium | Medium | `src/item.rs` (use_group) |
| 7 | Trailing comma in macros | High | Low | `src/convenience.rs` (trailing_comma) + `src/mac.rs` |
| 8 | Function call argument overflow | High | High | `src/expr.rs` (new heuristic needed) |

### Priority Order (by impact × fix feasibility)
1. **#5** Array literals — one-line fix in `simple_array()`
2. **#7** Trailing commas — causes clippy failures, small change in macro contexts
3. **#3** Unary `!` spacing — visible in all `assert!(!...)` calls
4. **#2** Let-chains — simple heuristic change
5. **#6** Use tree grouping — medium complexity
6. **#1** Method chains — most visible, medium complexity
7. **#4** Struct field spacing — rare, may be same root as #3
8. **#8** Argument overflow — complex but high impact

---

## Critical Review: Grounding vs Monkey-Patching

### #3 and #4 share the same root cause: `macro_rules_tokens` state machine

Both issues stem from the token-level printer in `src/mac.rs:128-240`. The state machine uses a catch-all `(_, _) => (true, Other)` that emits a space before any unrecognized punctuation. Specifically:

**#3 (Unary `!`)**: Inside `assert!(!vs.is_empty())`, the group contents `!vs.is_empty()` are recursively processed. The `!` token transitions state to `Other`. Then `vs` (an `Ident`) checks `state != Dot && state != Colon2` → `true` → `needs_space = true`. A space is emitted between `!` and `vs`. The principled fix is: add state tracking for unary prefix operators. After `!` at the start of an expression (i.e., when state is `Start` or after `(`, `,`, `=`, `&&`, `||`, etc.), suppress the space before the next token. This matches rustfmt's `rewrite_unary_prefix` which formats `!expr` with no space.

**#4 (Struct `:`)**: In `Violation { line: 0 }`, the `:` is `Punct(':', Alone)`. The state machine only handles `Punct(':', Joint)` (for `::`). `Alone` colon hits the catch-all → `needs_space = true` → space before `:`. The principled fix: add `(Ident, Token::Punct(':', Spacing::Alone)) => (false, Other)` to suppress the space after an identifier before a standalone colon. This matches rustfmt's `colon_spaces` default of `": "` (space after, not before).

**Assessment: Both fixes are grounded.** They address a missing state transition in a well-defined state machine, not a case-specific hack.

### #5 (Array literals) — Grounded

The `simple_array()` function explicitly excludes `Lit::Str`. This is a deliberate upstream design choice (prettyplease aimed for proc-macro output, where string arrays are rare). For a9-prettyplease targeting rustfmt compatibility, including `Lit::Str` in `simple_array()` switches to the `ibox(0)` (Inconsistent) layout, which naturally packs multiple items per line — exactly matching rustfmt's `HorizontalVertical` tactic. **No monkey-patching**: the correct formatting path already exists, we just need to route string literal arrays to it.

### #7 (Trailing commas) — Needs deeper analysis

The `trailing_comma` function's `pre_break: Some(',')` is a principled design: comma only on break. The issue occurs in macro contexts where the break decision is borderline. The question is: **does rustfmt ever add trailing commas in macro arguments?** Answer: rustfmt preserves the original trailing comma status via `span_ends_with_comma()`. It never adds one. The principled fix: for macro argument lists (detected in `mac.rs`), use a variant of `trailing_comma` that never emits `pre_break: Some(',')` for the last argument. This needs a new `trailing_comma_no_trailing` or passing a flag.

**However**, looking more carefully at the actual divergence — the trailing comma `format!("...", v.message,)` — this likely comes from the **structured** printer (not the macro token printer), because `format!` is recognized as a known macro and gets structured formatting. Need to verify which code path handles `format!` args.

### #1 (Method chains) — Partially grounded, risk of over-correction

The `end_with_max_width(60)` is a principled heuristic, but the threshold is wrong for rustfmt compatibility. Raising it to 100 (matching `MARGIN`) would be a blunt fix that could cause other regressions. The deeper issue is that rustfmt's chain formatter has a "root absorption" strategy that a9-prettyplease lacks entirely. A principled fix would implement root absorption: when `receiver.len() + ".method(...)" <= MARGIN`, keep on one line. The `end_with_max_width(60)` should be removed for method chains specifically, letting the natural break logic handle it.

**Assessment: Medium risk.** Changing the threshold is a monkey-patch; implementing root absorption is grounded but complex.

### #2 (Let-chains) — Needs verification

My original claim was "rustfmt always breaks let-chains across lines." Let me verify: rustfmt formats `if let X = y && z {` as a single condition expression. The `ControlFlow::rewrite_cond` path treats the entire condition as one `ast::Expr`. Whether it breaks depends on width. For short conditions like `if let Some(x) = opt && x > 0 {`, rustfmt keeps it on one line. For longer ones, it breaks. So a9-prettyplease's behavior (keep on one line when fits) may actually be **correct** for short chains but **wrong** for longer ones due to different break-point placement.

**Assessment: Need more test cases** to determine if this is actually a divergence in all cases or only when the chain is long enough that the consistent-break threshold differs.

### #6 (Use tree grouping) — Grounded but complex

The current `use_group` uses `space()` between items unless the item ends in a nested group (then `hardbreak()`). The issue is that rustfmt uses `definitive_tactic` which considers item complexity. The principled fix: when any item in a use-group is itself a `UseTree::Path` leading to a `UseTree::Group`, force vertical layout (all items on separate lines). This matches rustfmt's behavior and is a clean heuristic based on tree structure.

### #8 (Argument overflow) — Genuinely missing feature

This is not a bug or misconfiguration — it's a formatting strategy that prettyplease never implemented. rustfmt's `overflow.rs` tries formatting the last argument "overflowed" (on the same line as the opening paren) and compares it against the vertical layout. Adding this to a9-prettyplease would require significant new logic in the argument formatting path. **Grounded but high effort.**
