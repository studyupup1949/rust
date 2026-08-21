# abstract-gui

Installable package name: `abstract-gui`.
Executable command name: `gui`.

Declarative `.gui` format for modeling GUI structure with two forests:

- `drill`: information-space drill-down
- `inherit`: layout/navigation/capability inheritance

The format treats `nav` as a first-class shared component.

- a `nav` is a named target page set
- any `node` may declare attributes such as `path` or `nav`
- those attributes are inherited through `inherit`
- every `inherit` leaf is a page
- every node that appears in `drill` is a page
- non-leaf `inherit` nodes may be layouts or shells
- scalar attributes override inherited values
- vector attributes merge by set union
- many apparent transitions are derived from `drill` and `nav`

In this abstract language, `nav` is unordered. Concrete UI layers may choose an
ordering or spatial arrangement such as tabs, side menus, or ring menus.

Lines whose first non-space character is `#` are treated as comments.
`#import "foo.gui"` inlines another `.gui` file before parsing. Imported files
are merged by top-level section:

- `drill` and `inherit`: root entries are merged
- `nav`: target sets are union-merged by nav id
- `node`: scalar attrs override, vector attrs union-merge by node id
- `groups`: groups with the same `id` union-merge their members

## Example

```gui
app: Demo

drill:
  Home:
    Products:
      ProductDetail:
        ProductReviews:
    AdminRoot:
      AdminUsers:

inherit:
  RootLayout:
    Home:
    Products:
    AdminShell:
      AdminRoot:
      AdminUsers:

nav:
  GlobalNav:
    - Home
    - Products
    - AdminRoot

  ProductTabs:
    - ProductDetail
    - ProductReviews

node:
  RootLayout:
    nav: [GlobalNav]

  Home:
    path: /

  Products:
    path: /products

  ProductDetail:
    path: /products/:id
    nav: [ProductTabs]
```

`foo: [bar]` is shorthand for:

```gui
foo:
  - bar
```

`drill` と `inherit` は sequence ではなく tree mapping として書くのが基本です。
leaf は `Page:` のように null child で表現でき、loader/preparser は bare leaf 行
`Page` も `Page:` と同義に扱います。

## Repository layout

- `examples/`: sample `.gui` files
- `spec/`: draft format specification

## CLI

```sh
gui check examples/demo.gui
gui check examples/demo.gui other.gui
gui check
gui page examples/demo.gui
gui page
gui drill
gui inherit
gui node
gui nav
```

引数を省略した場合は、実行ディレクトリ配下を再帰走査して見つかったすべての
`*.gui` を 1 つの入力集合として merge してから各コマンドを実行します。
明示的に複数ファイルを渡した場合も同じく merge 後の和集合に対して実行します。

- `page`: page 条件に合致する node 一覧
- `drill`: `drill` 木を indent 付きで表示
- `inherit`: `inherit` 木を indent 付きで表示
- `node`: `node` セクションのキー一覧
- `nav`: `nav` セクションのキー一覧

## Status

This repository currently contains an initial draft only.
