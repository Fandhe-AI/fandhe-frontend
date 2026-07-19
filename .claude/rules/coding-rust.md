# Rust コーディング規約

## 本リポジトリ特有の厳守事項

- **既定エスケープを弱めない（REQ-1）**: テキスト補間は必ず既定エスケープを経由する。エスケープ迂回は `raw_html()` 等の明示的オプトイン API のみとし、新たな迂回経路を作らない
- **`#![forbid(unsafe_code)]`（REQ-2）**: `core` / `interactive` では `unsafe` を一切使用しない。`unsafe` は WASM バインディング層・FFI 境界に限定し、使用箇所は `docs/policy/unsafe-boundary.md` に列挙する
- **依存グラフ上限（REQ-3）**: 標準サーバー構成で依存パッケージ 60 件以内・深さ 6 以内。依存クレートの追加は事前に `cargo metadata` で影響を確認し、**ユーザー承認を得る**
- **`core` は外部依存ゼロ**: `core/Cargo.toml` に外部クレートを追加しない
- **HTML 文字列の直接組み立て禁止**: `format!("<div>{}</div>", user_input)` のようなコードは書かない。必ずノード木 API を使う

## 一般規約

- Rust 2021 以降のエディションに従い、`cargo fmt`（rustfmt 既定設定）で整形する
- `cargo clippy -- -D warnings` を通す
- エラーハンドリングは `Result` + `?` を基本とし、ライブラリコードでの `unwrap()` / `expect()` / `panic!` を避ける（テストコードは可）
- 公開 API には rustdoc（`///`）を付ける（`.claude/rules/code-comment-style.md` 参照）
- 命名は Rust API Guidelines に従う（型: UpperCamelCase / 関数・変数: snake_case / 定数: SCREAMING_SNAKE_CASE）
- 不要な `clone()` を避け、借用を優先する
- モジュール分割はクレートの責務境界に従い、循環依存を作らない

## テスト

- 新機能・バグ修正には対応するテストを追加する
- XSS 回帰テスト（SSR / SSG / CSR / WASM の各経路）は削除・弱体化しない
- テストの `#[ignore]` 追加でごまかさない
