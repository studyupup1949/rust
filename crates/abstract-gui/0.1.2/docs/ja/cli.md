# CLI

## コマンド

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

## 入力解決

ファイル引数を省略した場合は、カレントディレクトリ配下を再帰走査して対象
ファイルを集めます。

複数の `.gui` ファイルを渡した場合は、1 つの論理 document として merge して
からコマンドを実行します。

ディレクトリを渡した場合も、配下を再帰走査して対象ファイルを集めます。

- `check`, `page`, `drill`, `inherit`, `node`, `nav`: `*.gui`
- `scan`: `*.html`, `*.htm`

## コマンド概要

- `check`: `.gui` を parse / validate
- `page`: 現在の page 規則に合致する node 一覧
- `drill`: `drill` 木をインデント付き表示
- `inherit`: `inherit` 木をインデント付き表示
- `node`: node id 一覧
- `nav`: nav id 一覧
- `scan`: rendered HTML 群から `.gui` を推定して stdout へ出力

## よくある使い方

```sh
gui check examples/demo.gui
gui page
gui scan saved/home.html saved/pricing.html > site.gui
```

## 注意

- `gui scan` 自体はページ取得や JavaScript 実行を行いません。
- HTML の取得は別ツールの責務です。
