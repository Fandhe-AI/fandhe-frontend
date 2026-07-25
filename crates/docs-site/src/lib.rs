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
//! - [`component_page`][]: 部品ページ（`/components/<kebab>/`）の雛形
//!   レンダラ（イシュー #942）。[`showcase`] の部品単位デモを「Demo」節
//!   として受け取り、Features/Anatomy/API Reference/Examples/Accessibility
//!   と合わせた 6 節（H2 固定）へ組み立て直す。Anatomy・`data-*` 属性表・
//!   CSS 変数表は headless-ui/pre-styled-ui の出力から機械導出し、[`build`]
//!   は `page.path` 照会をこのモジュール経由に切り替える（索引ページ
//!   `showcase::PAGE_PATH` はイシュー #943 で改組済みのため Rust 生成
//!   コンテンツを持たず `None` を返す）
//! - [`admonition`]: `> [!NOTE]` 等の admonition 構文（[`markdown`] が検出し
//!   pre-styled-ui の alert 部品で描画する）が参照する専用 CSS の組み立てと、
//!   ページが admonition を含むかどうかの判定（イシュー #715）
//! - [`site_theme`]: サイト骨格（ヘッダー・3 カラム grid・サイドバー・本文
//!   タイポグラフィ・toc・前後ナビ）が使う CSS（`assets/site.css`）の
//!   ビルド時生成。旧 `site/assets/site.css`（`--docs-*` トークンで自己完結
//!   する単一静的ファイル）を置き換え、`--fandhe-*` テーマトークンへ一本化
//!   する（イシュー #905）。右カラム目次の独立カラム化と sticky 追従は
//!   イシュー #909 で追加
//! - [`script`]: docs サイトが出力する唯一の JS（`assets/site.js`）と、
//!   `<head>` の FOUC 抑止インラインスニペットの組み立て。テーマトグル
//!   （ダーク/ライト切替）・GitHub リンクの追加に伴い初めて docs サイトへ
//!   クライアント側 JS を持ち込む（イシュー #951）
//!
//! `fandhe-frontend-core` / `fandhe-frontend-app` / `fandhe-frontend-server` /
//! `fandhe-frontend-pre-styled-ui` のみに依存し、外部クレートは追加しない
//! （`Cargo.toml` の REQ-3 非影響コメント参照）。headless 型が必要な場合は
//! pre-styled-ui のルート再エクスポート（イシュー #685）経由で得るため
//! headless-ui への直接依存は持たない（イシュー #693）。
//!
//! サイト骨格（`layout` / `nav` / `markdown` が生成する `assets/site.css` の
//! クラス名契約）は [`site_theme`] が pre-styled-ui のテーマトークンで
//! ビルド時生成する（イシュー #905。旧評価は
//! `docs/design/docs-site-styled-ui-adoption.md`（イシュー #694）§3.4 参照、
//! 再評価により見送りから導入へ転換した）。[`showcase`] の分離 CSS 方式
//! （サイト骨格のカスケードへ影響させない適用境界）は showcase/admonition
//! 専用 CSS について引き続き維持する。
//!
//! `#![forbid(unsafe_code)]` は `crates/core` / `crates/interactive` と同様に
//! 本クレートでも維持する（`.claude/rules/coding-rust.md` の一般規約）。

#![forbid(unsafe_code)]

pub mod admonition;
pub mod build;
pub mod component_page;
pub(crate) mod component_specs_nav_data;
pub mod component_specs_overlay;
pub mod layout;
pub mod linkcheck;
pub mod markdown;
pub mod nav;
pub mod script;
pub mod showcase;
pub mod site_theme;
pub mod skip_nav;
#[cfg(test)]
mod test_scratch;
