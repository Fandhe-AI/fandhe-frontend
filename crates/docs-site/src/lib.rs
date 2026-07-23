//! `fandhe-frontend-docs-site` のライブラリ入口。
//!
//! 公式 docs サイト（自前 SSG ドッグフーディング、親イシュー #459 / ルート #456）の
//! ページ生成ロジックを束ねるクレート。バイナリ本体（`src/main.rs`）は
//! fail-closed の未実装終了を維持したまま、統合テスト（`tests/`）から
//! `layout` / `markdown` / `nav` の各モジュールを直接検証できるように
//! するために `[lib]` ターゲットを併設する。crate 外部への公開・配布は
//! 行わない（`Cargo.toml` の `publish = false`）。
//!
//! - [`layout`]: docs レイアウトコンポーネント（イシュー #469）
//! - [`markdown`]: Markdown ブロック構文 → Node 木レンダラ（イシュー #466）
//! - [`nav`]: `site/nav.toml` のパース・サイドバー / 前後ナビ生成（イシュー #468）
//! - [`linkcheck`]: `.md` リンクのサイト内パスへの書き換え・内部リンク突合検証
//!   （イシュー #470）
//! - [`build`]: `nav.toml` 読込 → ページ組み立て → linkcheck →
//!   `generate_pages()` 書き出し → アセットコピーの一連のビルドパイプライン
//!   本体（イシュー #470）。`main.rs`（バイナリ本体）は本モジュールの
//!   [`build::build_site`] を呼ぶ薄いラッパーとして統合済み。
//! - [`showcase`]: pre-styled-ui コンポーネントを実レンダリングして掲載する
//!   UI ショーケースページの Rust 生成コンテンツと専用 CSS
//!   （`StyleSheet` 経由）。Markdown パイプラインの後段で [`build`] が
//!   `page.path` 照会により合成する
//! - [`admonition`]: `> [!NOTE]` 等の admonition 構文（[`markdown`] が検出し
//!   pre-styled-ui の alert 部品で描画する）が参照する専用 CSS の組み立てと、
//!   ページが admonition を含むかどうかの判定（イシュー #715）
//!
//! `fandhe-frontend-core` / `fandhe-frontend-app` / `fandhe-frontend-server` /
//! `fandhe-frontend-pre-styled-ui` のみに依存し、外部クレートは追加しない
//! （`Cargo.toml` の REQ-3 非影響コメント参照）。headless 型が必要な場合は
//! pre-styled-ui のルート再エクスポート（イシュー #685）経由で得るため
//! headless-ui への直接依存は持たない（イシュー #693）。
//!
//! サイト骨格（`layout` / `nav` / `markdown` が生成する `site/assets/site.css`
//! のクラス名契約）への pre-styled-ui styled 部品・テーマトークンの適用は
//! 評価の上で見送り、[`showcase`] の分離 CSS 方式（`site.css` のカスケード
//! へ影響させない適用境界）のみを採用している。評価内容・再評価トリガーは
//! `docs/design/docs-site-styled-ui-adoption.md`（イシュー #694）を参照。
//!
//! `#![forbid(unsafe_code)]` は `crates/core` / `crates/interactive` と同様に
//! 本クレートでも維持する（`.claude/rules/coding-rust.md` の一般規約）。

#![forbid(unsafe_code)]

pub mod admonition;
pub mod build;
pub mod layout;
pub mod linkcheck;
pub mod markdown;
pub mod nav;
pub mod showcase;
pub mod skip_nav;
#[cfg(test)]
mod test_scratch;
