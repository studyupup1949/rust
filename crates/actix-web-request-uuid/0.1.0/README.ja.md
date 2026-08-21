# actix-web-request-uuid

actix-webフレームワークでリクエストID機能を追加するためのRustライブラリです。

[![CI](https://github.com/YusukeYoshida8849/actix-web-request-uuid/workflows/CI/badge.svg)](https://github.com/YusukeYoshida8849/actix-web-request-uuid/actions?query=workflow%3ACI)
[![crates.io](https://img.shields.io/crates/v/actix-web-request-uuid)](https://crates.io/crates/actix-web-request-uuid)
[![Documentation](https://docs.rs/actix-web-request-uuid/badge.svg)](https://docs.rs/actix-web-request-uuid)
[![License](https://img.shields.io/crates/l/actix-web-request-uuid)](https://github.com/YusukeYoshida8849/actix-web-request-uuid#license)

## インストール

`Cargo.toml`に以下を追加してください：

```toml
[dependencies]
actix-web-request-uuid = "0.1.0"
```

## 使用方法

クレートのルートに以下を追加してください：

```rust
use actix_web::{web, App, HttpServer, HttpResponse, Error};
use actix_web_request_uuid::RequestIDMiddlware;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(
        || App::new()
            .wrap(RequestIDMiddleware::new(36))
            .service(web::resource("/").to(|| HttpResponse::Ok())))
        .bind("127.0.0.1:59880")?
        .run()
        .await
}
```

## 機能

- 各HTTPリクエストに対する自動的なリクエストID生成
- actix-webミドルウェアとの簡単な統合
- 軽量で効率的な実装
- actix-webエコシステムとの互換性

### 元のプロジェクトとの変更点

このプロジェクトは[pastjean/actix-web-requestid](https://github.com/pastjean/actix-web-requestid)をベースに、以下の大幅な機能拡張を行っています：

#### 1. **Thread-Localによるグローバルアクセス機能**
元のプロジェクトにはないThread-Local変数を利用したリクエストID管理機能を追加：

- `get_current_request_id()`: リクエスト処理中のどこからでもリクエストIDにアクセス可能
- `set_current_request_id()`: リクエストIDを手動で設定
- `clear_current_request_id()`: リクエストID情報をクリア

```rust
// 任意の関数からリクエストIDを取得
async fn my_service() -> Result<(), Error> {
    if let Some(request_id) = get_current_request_id() {
        log::info!("Processing request: {}", request_id);
    }
    Ok(())
}
```

#### 2. **豊富なカスタマイズオプション**
元のプロジェクトはUUID v4のみでしたが、多様な形式をサポート：

- **フルUUID形式**: `with_full_uuid()` - 36文字、ハイフン付き
- **シンプルUUID形式**: `with_simple_uuid()` - 32文字、ハイフンなし
- **カスタム形式**: `with_custom_uuid_format()` - 独自のフォーマッター
- **カスタムジェネレーター**: `generator()` - 完全に独自のID生成ロジック
- **ヘッダー名変更**: `header_name()` - デフォルトの`request-id`を変更可能

```rust
// 設定例
let middleware = RequestIDMiddleware::new(32)
    .with_simple_uuid()
    .header_name("X-Request-ID")
    .generator(|| format!("req-{}", Uuid::new_v4().simple()));
```

#### 3. **Actix Web エクストラクター対応**
元のプロジェクトにはないFromRequest実装により、より直感的なAPI：

```rust
// ハンドラー関数で直接リクエストIDを取得
async fn show_id(request_id: RequestID) -> impl Responder {
    format!("Your request ID: {}", request_id)
}
```

#### 4. **拡張トレイト**
HttpMessageからリクエストIDを直接取得できるトレイト：

```rust
pub trait RequestIDMessage {
    fn request_id(&self) -> RequestID;
}

// 使用例
fn my_middleware(req: &HttpRequest) {
    let id = req.request_id(); // 直接取得
}
```

#### 5. **エラーハンドリングと堅牢性**
- **ID長検証**: 0以下の長さでpanicする安全機能
- **既存ID再利用**: エクステンションに保存されたIDの再利用
- **自動クリーンアップ**: リクエスト処理完了後の自動的なThread-Local変数クリア

#### 6. **包括的なテストスイート**
元のプロジェクトより詳細なテストカバレッジ：

- カスタムID長テスト
- UUID形式別テスト
- カスタムヘッダー名テスト
- Thread-Local機能テスト
- エラーケーステスト

#### 7. **充実したドキュメント**
- 各公開関数に詳細なdocstring
- 実用的なコードサンプル
- ユースケース別の説明
- ベストプラクティスガイド

これらの拡張により、元のシンプルなリクエストID生成ミドルウェアから、**ログ収集・分散トレーシング・デバッグ支援に対応した企業レベルのリクエストトラッキングシステム**へと大幅に更新しました。

## ドキュメント

詳細なドキュメントと例については、[docs.rsページ](https://docs.rs/actix-web-request-uuid)を参照してください。

## コントリビューション

コントリビューションを歓迎します！お気軽にPull Requestを提出してください。
明示的に別段の定めがない限り、Apache-2.0ライセンスで定義されているように、あなたによって作業に含めるために意図的に提出された貢献は、追加の条件なしに上記のようにデュアルライセンスされるものとします。

## ライセンス

このプロジェクトは以下のいずれかのライセンスの下でライセンスされています：

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) または http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) または http://opensource.org/licenses/MIT)

お好みの方をお選びください。

## 元のプロジェクト

これは[pastjean/actix-web-requestid](https://github.com/pastjean/actix-web-requestid)のフォークです。
Rustコミュニティへの貢献に対して元の作者に感謝いたします。
