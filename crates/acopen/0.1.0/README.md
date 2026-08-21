# acopen

## For non-Japanese speakers

The English README has not yet been created. If you are able to help me, I would appreciate it.

## acopenについて

* AtCoderの問題ページをコマンドラインから開けるようにするツールです。
* [cargo-atcoder](https://github.com/tanakh/cargo-atcoder)との併用を前提としており、コンテストIDと同名のフォルダ内でしか使えない仕様になっています。
  * プルリクエストを送ることも考えましたが、同リポジトリにはかなりプルリクエストが溜まっており、確認してもらえるか分からなかったので別で作成しました。
* [cargo-compete](https://github.com/qryxip/cargo-compete)には同様の機能が備わっているため、cargo-competeを使用する場合はacopenと併用する必要はありません。
  * つまり本ライブラリは何らかの理由でcargo-competeではなくcargo-atcoderを使用したい人向けのものです。

## インストール

```sh
cargo install acopen
```

## 使い方

コンテストIDと同名のフォルダ内で、以下のコマンドを実行します。

```sh
acopen <problem-id>
```

problem-idは、cargo-atcoderを使用している場合、ファイル名から拡張子を取り除いた部分です。
