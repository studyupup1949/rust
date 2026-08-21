# parser-infrastructure Specification

## Purpose
TBD - created by archiving change migrate-parser-to-chumsky. Update Purpose after archive.
## Requirements
### Requirement: Parser uses chumsky combinators
The system SHALL construct the AbySS AST by applying `chumsky` parser combinators defined in Rust code, compiled against the latest supported `chumsky 0.11` API and its lifetime-aware parser traits, replacing the legacy `pest` grammar and `build_ast` transformer.

#### Scenario: Parse valid incantation
- **GIVEN** a `.aby` script whose syntax is valid under the current language rules
- **WHEN** the CLI or library consumer invokes `parser::parse` using the `chumsky::input::Stream` adapter
- **THEN** the parser SHALL return a `Vec<AST>` identical to the legacy implementation's output
- **AND** no `pest`-specific code SHALL be executed during parsing
- **AND** the implementation SHALL compile without relying on deprecated `chumsky` symbols such as `SimpleReason`, the root-level `Stream`, or `map_with_span`

### Requirement: Parser emits themed diagnostics through ariadne
The system SHALL format parse errors via an abstraction that renders `ariadne`-based, AbySS-themed diagnostics to standard error and remains compatible with the `ariadne` 0.6 span and report builder APIs.

#### Scenario: Report unexpected token
- **GIVEN** a `.aby` script with a missing semicolon after a statement
- **WHEN** the parser encounters the syntax error
- **THEN** it SHALL emit a diagnostic using the reporting abstraction that highlights the offending span, includes a themed title, and suggests adding the semicolon while passing a span type accepted by `ariadne::Report::build`
- **AND** the abstraction SHALL avoid removed helpers such as `Simple::expected()` or the three-argument form of `Report::build`, ensuring diagnostics render without compilation warnings

### Requirement: AST nodes capture precise source spans
The system SHALL attach `LineInfo` derived from the new parser's spans to every AST node that previously carried line information.

#### Scenario: Preserve line info for errors
- **GIVEN** a script that triggers an evaluation-time type error referencing a specific line
- **WHEN** the evaluator formats the error using `LineInfo`
- **THEN** the reported line and column SHALL match the positions produced by the new parser

### Requirement: Collection Type Keywords
The parser SHALL treat `scroll`, `lexicon`, and `materia` as reserved type keywords, lex them distinctly from identifiers, and emit the matching `Type` variants when they appear in `forge`, `engrave`, or `orbit` declarations.

#### Scenario: Parse scroll declaration
- **GIVEN** source `forge items: scroll = [];`
- **WHEN** the parser processes the declaration
- **THEN** it SHALL emit a `Type::Scroll` entry for the variable and reject using `scroll` as an identifier in the same position.

#### Scenario: Parse materia parameter
- **GIVEN** source `engrave print_any(val: materia) { unveil(val); }`
- **WHEN** the parser builds the function signature
- **THEN** it SHALL emit an `AST::EngraveParam` whose `param_type` is `Type::Materia`.

### Requirement: Collection Literals
The parser SHALL support list (`scroll`) and map (`lexicon`) literals using `[...]` and `{...}` syntax and emit `AST::ListLiteral` / `AST::MapLiteral` nodes that preserve element order and literal spans.

#### Scenario: Parse scroll literal
- **GIVEN** source `forge mixed: scroll = [1, "hi", boon];`
- **WHEN** the parser encounters the bracketed expression
- **THEN** it SHALL produce an `AST::ListLiteral` containing three child expressions in order.

#### Scenario: Parse lexicon literal with rune keys
- **GIVEN** source `forge data: lexicon = {"id": 1, "name": "abyss"};`
- **WHEN** the parser processes the braces
- **THEN** it SHALL produce an `AST::MapLiteral` and enforce that each key token is a `rune` literal.

### Requirement: Collection Indexing Syntax
The parser SHALL emit dedicated AST nodes for `expr[index]` access and `expr[index] = value` assignment so the evaluator can differentiate between reads and writes.

#### Scenario: Parse scroll index access
- **GIVEN** source `forge val: arcana = items[2];`
- **WHEN** the parser handles the square-bracket expression
- **THEN** it SHALL emit an `AST::IndexAccess` whose left child is the `items` reference and whose right child is the literal `2`.

#### Scenario: Parse indexed assignment
- **GIVEN** source `items[i] = rune_value;`
- **WHEN** the parser consumes the assignment tokens
- **THEN** it SHALL emit an `AST::IndexAssignment` capturing the collection expression, index expression, and assigned value while preserving the assignment span for diagnostics.

