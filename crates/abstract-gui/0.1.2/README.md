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

- `docs/`: reader-friendly guides and workflow documentation
- `examples/`: sample `.gui` files
- `spec/`: draft format specification
- `spec/scan.md`: design draft for future `gui scan` HTML-to-GUI pipeline

Start with [`docs/README.md`](./docs/README.md) if you want the guided version.
Japanese guides are available under [`docs/ja/`](./docs/ja/README.md).

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
gui scan page1.html page2.html
```

引数を省略した場合は、実行ディレクトリ配下を再帰走査して見つかったすべての
`*.gui` を 1 つの入力集合として merge してから各コマンドを実行します。
明示的に複数ファイルを渡した場合も同じく merge 後の和集合に対して実行します。
入力引数にディレクトリを渡した場合は、その配下を再帰走査して適合するファイルを
入力集合に追加します。

- `check`, `page`, `drill`, `inherit`, `node`, `nav`: `*.gui`
- `scan`: `*.html`, `*.htm`

- `page`: page 条件に合致する node 一覧
- `drill`: `drill` 木を indent 付きで表示
- `inherit`: `inherit` 木を indent 付きで表示
- `node`: `node` セクションのキー一覧
- `nav`: `nav` セクションのキー一覧
- `scan`: rendered HTML files を解析して `.gui` を stdout に出力
  - 高信頼の `nav` / `tablist` / `header` / `footer` から内部リンクを拾い、
    入力HTMLに無い page も stub として補完する
  - `login` / `cart` / `checkout` など action 寄りの導線は page stub 化を抑制する
  - page host を解決できない場合は absolute URL を内部 page 候補に含めない
  - locale switcher、巨大 footer directory、巨大 docs index は nav/page 候補から抑制する
  - `dialog` / `role=dialog` / `aria-modal=true` を dialog node として抽出し、trigger 側 page に `opens` 関係を付ける
  - dialog には `dialog-kind: generic|form|confirm|alert|consent|sheet|picker|promo` を推定付与する
  - 複数 page で共通に開かれる dialog は `RootLayout` や部分 layout の `opens` へ昇格させる
  - link cluster の path 傾向から `CategoryNav` / `AccountNav` / `FooterNav` を推定する
  - 近似した nav cluster は統合し、重複気味の nav を減らす
  - 出力 `node` には `kind: page|section|layout|action|index|dialog` を付け、抽象化後の役割を明示する
  - breadcrumb がある場合は `drill` の親推定で path prefix より優先して使う
  - 複数ページで共有される non-root nav target 群は `AccountLayout` などの部分 layout 推定に使う

`scan` は HTML の取得や JavaScript 実行を行いません。入力は別ツールで保存した
rendered HTML を想定します。設計メモは `spec/scan.md` にあります。

## Status

This repository currently contains an initial draft only.
