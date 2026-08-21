# 概要

`abstract-gui` は GUI 構造を少数の直交した関係で表現します。

## 中心となる要素

- `drill`: 情報空間の掘り下げ
- `inherit`: レイアウト、ナビゲーション、能力の共有
- `nav`: 名前付きの遷移先 page 集合
- `node`: 属性を持つ GUI node

この分離により、次を別々に表せます。

- 何を掘り下げているか
- 何を共有しているか
- 共有ナビゲータでどこへ行けるか
- 各 node がどの属性を持つか

## page の規則

- `inherit` の leaf は page
- `drill` に現れる node は page
- `inherit` の non-leaf は layout / shell / template になりうる

## 属性継承の規則

- scalar 属性は上書き
- vector 属性は集合和で merge
- `nav` は言語仕様上は順序を持たない

具体 UI では tabs, sidebar, ring menu など任意の表現を選べます。

## scan が出力する kind

現在の `gui scan` は次の `node.kind` を出します。

- `page`
- `section`
- `layout`
- `action`
- `index`
- `dialog`

dialog には `dialog-kind` として `form`, `confirm`, `promo` なども付きます。
