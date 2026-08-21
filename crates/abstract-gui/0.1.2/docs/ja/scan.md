# HTML解析

`gui scan` は保存済みの rendered HTML を読み、抽象 `.gui` を出力します。

## 現在の抽出方針

### page 構造

- canonical URL があれば file path 推定より優先
- canonical がなければ file 名から path を推定
- `drill` の親推定では path prefix より breadcrumb を優先

### nav

- `nav`, `tablist`, `header`, `footer` など高信頼 container を対象にする
- 繰り返し出る link 集合を `nav` cluster としてまとめる
- 近似 duplicate な cluster は統合する

### 抑止 heuristic

ノイズになりやすい次の構造は意図的に抑止します。

- `login`, `cart`, `checkout` など action 寄り導線
- page host が不明なときの absolute 外部 URL
- locale switcher
- 巨大 footer directory
- 巨大 docs index nav

### dialog

- `dialog`, `role=dialog`, `role=alertdialog`, `aria-modal=true` を `kind: dialog` として抽出
- trigger は `opens` 関係で表現
- `aria-controls`, `href=#id`, `data-dialog*`, `data-modal-target` を使って結び付ける
- 複数 page で共通に開かれる dialog は layout 側へ昇格する場合がある

### kind

現在の `node.kind`:

- `page`
- `section`
- `layout`
- `action`
- `index`
- `dialog`

dialog には追加で `dialog-kind` が付きます。

- `generic`
- `form`
- `confirm`
- `alert`
- `consent`
- `sheet`
- `picker`
- `promo`

## まだ弱い点

- 主 nav / 補助 nav の順位付け
- 同一実体 page の alias 統合
- semantic attribute がない JS 専用 modal trigger
- 大規模 docs 群での taxonomy 推定

設計の背景は [`spec/scan.md`](../../spec/scan.md) を参照してください。
