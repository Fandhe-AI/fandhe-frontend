# Rust コーディング規約

## 本リポジトリ特有の厳守事項

- **既定エスケープを弱めない（REQ-1）**: テキスト補間は必ず既定エスケープを経由する。エスケープ迂回は `raw_html()` 等の明示的オプトイン API のみとし、新たな迂回経路を作らない
- **`#![forbid(unsafe_code)]`（REQ-2）**: `core` / `interactive` では `unsafe` を一切使用しない。`unsafe` は WASM バインディング層・FFI 境界に限定し、使用箇所は `docs/policy/unsafe-boundary.md` に列挙する
- **依存グラフ上限（REQ-3）**: 標準サーバー構成で依存パッケージ 60 件以内・深さ 6 以内。依存クレートの追加は事前に `cargo metadata` で影響を確認し、**ユーザー承認を得る**
- **`core` は外部依存ゼロ**: `crates/core/Cargo.toml` に外部クレートを追加しない
- **HTML 文字列の直接組み立て禁止**: `format!("<div>{}</div>", user_input)` のようなコードは書かない。必ずノード木 API を使う
- **意図的非採用機能の再導入提案には評価軸の充足確認が必須**: 仮想 DOM・ファイルベースルーティング・HMR・signal/store は AI 開発・保守前提（明示性・決定性・機械検証可能性・コンテキスト消費）に基づき意図的に非採用としている。再導入を提案する場合は `docs/policy/intentional-non-adoption.md` の評価軸・再評価トリガーの充足を確認し、Issue・PR に明記する
- **UI 部品の責務境界（ユーザー判断 2026-07-25、`docs/policy/intentional-non-adoption.md` §3.25）**: UI コンポーネント層（`crates/headless-ui/` / `crates/pre-styled-ui/`）が担うのは **anatomy（構造）・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）まで**とする。(1) バリデーション・送信処理・データ整形・永続化といった**アプリケーションロジックを内包する部品は、参照軸（ark-ui / chakra-ui / Radix）に存在しても実装しない**（Radix `Form` が確定対象。構造部分は既存の `field` / `fieldset` が担い、検証は利用者が通常の Rust コードで書いてその結果を渡す）。§3.23 の「数値・日時整形は UI コンポーネント層の責務外」と同じ判断軸の一般化である。(2) 参照元が primitives 層へ持ち込んでいる**装飾・アニメーション・レイアウト計測の関心**（Radix の `data-motion`、viewport 測定等）は `headless-ui` へ持ち込まず、必要なら上層の `pre-styled-ui` の責務として設計する（部品自体を実装対象から外すのではなく、層の割り当てを変える規則）。この 2 規則に反する実装・再導入を提案する場合は §3.25 の再評価トリガーの充足確認を Issue・PR に明記する
- **公開済みクレート（crates.io）の実体変更時は semver バンプ必須（イシュー #638）**: `crates/*` のうち crates.io へ公開済みのクレート（`Cargo.toml` に `publish = false` を持たないもの）は、`src/` ・ `Cargo.toml` ・ `build.rs` を変更する PR で必ず `version` をバンプする。0.x の破壊的変更はマイナーバンプとする（依存元クレートの `version = "..."` 指定・release 手順への波及に注意）。公開 API に影響しない変更（ドキュメントのみ・内部実装のみ等）に限り、PR 本文に `version-bump-exempt: <crate-name>`（理由を続けて記載）を宣言することでバンプを免除できる。機械検知は `xtask check-version-bump`（CI: `version-bump-guard` ジョブ）が行う（`.claude/rules/ci.md` 参照）

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
