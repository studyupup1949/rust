# Scanning HTML

`gui scan` reads already-rendered HTML files and emits abstract `.gui`.

## Current extraction model

### Page structure

- canonical URL metadata overrides file-derived path inference when present
- file names are used as a fallback path source
- breadcrumb signals are preferred over plain path prefix when inferring `drill`

### Navigation

- high-confidence containers such as `nav`, `tablist`, `header`, and `footer`
  are scanned for internal links
- repeated link sets become `nav` clusters
- near-duplicate clusters are merged

### Suppression heuristics

The scanner intentionally suppresses several noisy structures:

- action-like links such as `login`, `cart`, and `checkout`
- absolute external URLs when the page host is not known
- locale switchers
- very large footer directories
- very large documentation index navs

### Dialogs

- `dialog`, `role=dialog`, `role=alertdialog`, and `aria-modal=true` are
  emitted as `kind: dialog`
- triggers are connected through `opens`
- supported trigger hints include `aria-controls`, `href=#id`, `data-dialog*`,
  and `data-modal-target`
- repeated dialog opens across related pages may be promoted to a layout node

### Kinds

Current inferred `node.kind` values are:

- `page`
- `section`
- `layout`
- `action`
- `index`
- `dialog`

Dialogs also carry `dialog-kind`:

- `generic`
- `form`
- `confirm`
- `alert`
- `consent`
- `sheet`
- `picker`
- `promo`

## Practical limits

The scanner is heuristic.

It does not yet fully solve:

- primary vs secondary nav ranking
- page alias consolidation
- dynamic JS-only modal triggers without useful attributes
- rich docs taxonomy inference across many pages

For the design rationale behind the scanner, see [`spec/scan.md`](../spec/scan.md).
