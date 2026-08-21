# `gui scan` design draft

## Goal

`gui scan foo.html bar.html` reads one or more rendered HTML files and emits a
`.gui` document to stdout.

The command does not execute JavaScript by itself. HTML acquisition is the
responsibility of another tool. `gui scan` only analyzes already-rendered HTML.

## Pipeline

The scanner is split into four phases:

1. Parse HTML into a concrete GUI tree.
2. Normalize and prune nodes that are structurally redundant.
3. Extract cross-page common structure and navigation sets.
4. Render the final abstract `.gui` document.

The key design choice is that the intermediate representation may itself be a
redundant `.gui` document. That keeps the parser, diff, formatter, and debug
workflow unified.

## Input model

- Each input HTML file is treated as one page candidate.
- File paths are meaningful and may be used to infer page paths.
- If a page provides canonical URL metadata, that should override file-derived
  path inference.

## Concrete GUI IR

The first stage produces a redundant GUI tree that is close to the HTML DOM.
This is not yet the final abstract GUI.

### Concrete node attributes

Concrete nodes may carry extra attributes that are not expected to survive the
reduction phase:

- `kind`: `page-root`, `wrapper`, `nav-candidate`, `section`, `heading`,
  `action`, `form`, `dialog`, `content-group`, `list`, `item`
- `source-tag`
- `source-id`
- `source-class`
- `role`
- `text`
- `heading`
- `href`
- `path`
- `link-targets`
- `generated-from`

### Concrete tree generation

The HTML DOM is traversed from `body` downward.

- `script`, `style`, `template`, and `noscript` are ignored.
- Every remaining meaningful element becomes a concrete node.
- Child order is preserved during the concrete stage, even though final `nav`
  in abstract `.gui` is modeled as an unordered set.

### Initial kind inference

The scanner assigns a provisional `kind` based on tag and local structure:

- `nav` element -> `nav-candidate`
- `header`, `footer`, `aside`, `main` -> container kinds
- `form` -> `form`
- `button` and submit-like links -> `action`
- heading-bearing sections -> `section`
- generic `div` and `span` without semantic signal -> `wrapper`

## Normalization

Normalization makes trees comparable across pages before common extraction.

### Wrapper compression

Drop or compress nodes that add little GUI meaning:

- empty nodes
- pure wrapper `div` / `span`
- nodes with one meaningful child and no semantic attributes
- decorative icon-only nodes

### Text normalization

- collapse whitespace
- keep only summary text when text is too long
- normalize heading strings for comparison

### Link normalization

- normalize relative links
- prefer canonical path form
- strip fragment identifiers by default
- optionally strip query parameters unless explicitly preserved

### Fingerprinting

Each normalized node gets a structural fingerprint used for comparison.

Suggested fingerprint inputs:

- `kind`
- normalized heading/text label
- `role`
- link target set
- child kind sequence
- optional source tag

## Cross-page common extraction

This phase compares normalized page trees and lifts repeated structure into the
abstract GUI model.

### Layout extraction

Goal: detect shared structure that should become `inherit` nodes such as
`RootLayout`, `AdminShell`, or product-specific layouts.

Algorithm sketch:

1. Compute fingerprints for upper tree regions of every page.
2. Compare root-adjacent and large-subtree candidates across pages.
3. Cluster highly similar subtrees.
4. Lift clusters into shared `inherit` ancestors.
5. Attach pages to the nearest matching shared ancestor.

Expected output examples:

- All pages share header + footer -> `RootLayout`
- Admin pages share extra side navigation -> `AdminShell`

### Navigation extraction

Goal: detect repeated link sets that should become `nav` entries.

Algorithm sketch:

1. Collect all `nav-candidate` nodes per page.
2. Extract the normalized target set from each candidate.
3. Cluster candidates by target-set similarity.
4. Promote repeated clusters to named `nav` entries.
5. Attach the resulting nav ids to the relevant `node` entries.

Naming heuristics:

- page-global repeated nav -> `GlobalNav`
- footer cluster -> `FooterNav`
- local product-detail cluster -> `ProductTabs`
- fallback names -> `Nav1`, `Nav2`, ...

### Drill extraction

Goal: infer `drill` from page relationships.

Initial implementation should be conservative and path-based.

Algorithm sketch:

1. Determine canonical page path for each scanned page.
2. Build a prefix tree from the path segments.
3. Project real pages onto that tree.
4. Use breadcrumb or heading hints only as secondary evidence.

Examples:

- `/products` -> `Products`
- `/products/detail` -> child of `Products`
- `/products/detail/reviews` -> child of `ProductDetail`

## Reduction into abstract `.gui`

The final abstract output removes concrete-only noise.

### Keep

- `inherit`
- `drill`
- `nav`
- `node.path`
- `node.title`
- `node.nav`
- future stable semantic attributes

### Drop

- `source-tag`
- `source-id`
- `source-class`
- comparison fingerprints
- temporary wrapper nodes
- page-local redundant containers that are absorbed by shared layout inference

## Command behavior

### Primary command

```sh
gui scan foo.html bar.html
```

Reads the input HTML files and writes abstract `.gui` to stdout.

### Possible future debug stages

These are optional later additions, not required for the first version:

```sh
gui scan --stage concrete foo.html
gui scan --stage normalized foo.html
gui scan --stage abstract foo.html
```

or split into separate commands:

```sh
gui scan foo.html > concrete.gui
gui reduce concrete.gui > abstract.gui
```

## Naming rules

Page ids may be inferred from:

1. canonical path
2. title
3. h1
4. file name

Identifiers should then be normalized into stable GUI ids such as
`ProductDetail` or `AdminUsers`.

## First implementation scope

The first implementation should stop at:

- static HTML only
- one scanned page per input HTML file
- wrapper compression
- path/title extraction
- repeated nav detection
- shared root layout detection
- path-based drill inference

This is enough to produce useful `.gui` while keeping the heuristics explainable.

## Non-goals for v0

- JavaScript execution
- CSS visual comparison
- pixel-level layout analysis
- full SPA route discovery
- perfect naming heuristics

## Why this design

The scanner deliberately separates facts from inference.

- Concrete tree: close to source HTML
- Normalized tree: comparable structure
- Abstract GUI: reduced semantic output

This makes the pipeline debuggable and lets future commands such as
`gui reduce`, `gui normalize`, or `gui diff` reuse the same `.gui`-based IR.
