# `.gui` draft format

## Core model

Each GUI model is defined by four main parts:

- `drill`: a forest for semantic drill-down in the information space
- `inherit`: a forest for visual or behavioral inheritance
- `nav`: named target page sets
- `node`: attributed nodes keyed by node id

Every `inherit` leaf is a page.
Every node that appears in the `drill` forest is a page.
Non-leaf nodes in the `inherit` forest may instead represent layouts, shells,
or templates.

Any node may declare attributes such as `path`, `nav`, `title`, or future
capabilities. Those attributes are inherited through the `inherit` forest.

Attribute inheritance follows these rules:

- scalar attributes override inherited values
- vector attributes merge by set union

`nav` is a vector attribute and is unordered at the language level.

Lines whose first non-space character is `#` are comments.
`#import "path.gui"` imports another `.gui` file inline before parsing.
Imported files are merged section-by-section:

- `drill` and `inherit`: root entries are appended by key
- `nav`: page-id sets are union-merged by nav id
- `node`: scalar attrs override and vector attrs union-merge by node id
- `groups`: groups with the same `id` union-merge their members

The model intentionally does not require a top-level `transitions` section.
Many practical transitions are derived from:

- movement along the `drill` tree
- selection of a target in a `nav`

## Minimal shape

```yaml
app: Example

drill:
  Home:

inherit:
  RootLayout:
    Home:

nav:
  GlobalNav:
    - Home

node:
  RootLayout:
    nav: [GlobalNav]

  Home:
    path: /
```

## Rules

- `node` is a map keyed by unique node id.
- `drill` and `inherit` are tree mappings, not ordered sequences.
- leaf nodes in `drill` / `inherit` may be written as `Leaf:` or as a bare line `Leaf`.
- `#import "..."` is resolved relative to the importing file.
- import cycles are invalid.
- every `inherit` leaf is a page.
- every node that appears in `drill` is a page.
- a non-leaf `inherit` node is not necessarily a page.
- a non-leaf `inherit` node may represent a layout, shell, or template.
- a page may appear in at most one place in the `drill` forest.
- each `nav` entry is a page id set.
- any node may define attributes.
- `node.nav` is a nav id set.
- scalar attributes override.
- vector attributes merge by set union.
- attributes implied only by inheritance need not be restated on child nodes.
- `groups` is optional and may overlap.

## Shorthand

`foo: [bar]` is shorthand for:

```yaml
foo:
  - bar
```

## Intended visualization

- inheritance forest
- drill-down forest
- nav overlay
- trait/group overlays

## Notes

- `inherit` answers: what does this page share?
- `drill` answers: what is this page drilling into?
- `nav` answers: where can this shared navigator take the user?
- ordering is intentionally outside the core language model.

## CLI input resolution

- `gui check`、`gui page`、`gui drill`、`gui inherit`、`gui node`、`gui nav` は複数 `.gui` ファイルを受け取れる。
- 複数ファイルは top-level section merge により 1 つの document として解釈する。
- 引数が省略された場合は、実行ディレクトリ配下を再帰走査して見つかったすべての
  `*.gui` を入力集合として用いる。
