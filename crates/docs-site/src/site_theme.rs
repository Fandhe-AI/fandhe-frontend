//! docs サイト骨格（ヘッダー・2 カラム grid・サイドバー・本文タイポグラフィ・
//! toc・前後ナビ）が使う CSS の配線（イシュー #905）。
//!
//! # 役割・呼び出し文脈
//!
//! 従来 `site/assets/site.css`（`--docs-*` トークンで自己完結する単一静的
//! ファイル）が担っていたサイト骨格スタイルを、`fandhe_frontend_pre_styled_ui`
//! の [`Theme`]/[`StyleSheet`]（`crates/pre-styled-ui/src/{theme,stylesheet}.rs`）
//! によるビルド時生成へ切り替える（`crate::showcase`/`crate::admonition`/
//! `crate::skip_nav` と同型の「[`StyleSheet`] 経由のみで CSS を組み立てる」
//! 方針）。`--docs-*` トークンは本モジュールの導入により全廃し、サイト骨格・
//! 各種コンポーネント CSS はすべて `--fandhe-*` トークンへ一本化する（テーマ
//! トークンの波及、`docs/design/docs-site-three-column-redesign.md` §4）。
//!
//! [`crate::build::build_site`] は [`stylesheet`] が返す CSS 本体を
//! [`STYLESHEET_REL_PATH`]（`assets/site.css`、旧 `site/assets/site.css` の
//! コピー先と同一パス）へ**全ビルドで無条件に**書き出す（[`crate::skip_nav`]
//! と同じ「全ページ適用」区分。showcase/admonition のような「使われている
//! ページだけ」の条件判定は不要）。`crate::layout::docs_page_with_assets` の
//! `<head>` `<link>` は本モジュールの [`STYLESHEET_REL_PATH`] 定数を参照する
//! ため、生成物のパスを変えれば `<link>` 側も追随する（単一実装点）。
//!
//! # クラス名契約（`crates/docs-site/src/layout.rs` / `nav.rs` / `markdown.rs` の
//! 実出力が正。ここに書かれたセレクタはすべて実際に生成される class 値であり、
//! `crates/docs-site/tests/site_css_contract.rs` が両者の乖離を検知する）
//!
//! ```text
//! body
//!   header.docs-header            … ヘッダバー（サイトタイトルへのリンク）
//!   div.docs-container            … 2 カラムのグリッドコンテナ（header の下）
//!     aside.docs-sidebar          … サイドバーのラッパー
//!       nav.sidebar               … `nav::sidebar()` の実出力（headless nav_list）
//!         a[aria-current="page"]  … 現在ページのリンク
//!     main.docs-main              … 本文カラムのラッパー
//!       nav.docs-toc（任意・先頭）… ページ内目次
//!       article.docs-content      … Markdown レンダラの出力 + `nav.prev-next`
//!         - `pre > code.language-*` … フェンス付きコードブロック
//! ```
//!
//! # 不変条件
//!
//! - `@import` / `@font-face` / リモート `url()` を追加しない（外部参照ゼロを
//!   維持する。[`stylesheet_never_references_external_resources`] が機械検証する）
//! - フォントはシステムフォントスタックのみを使う
//! - `--docs-*` トークンは 1 箇所も残さない（[`stylesheet_contains_no_docs_prefixed_tokens`]
//!   が機械検証する）
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! CSS は [`StyleSheet`]（検証済み CSS のみを保持し `<`・不正な制御文字を拒否
//! する型）経由でのみ組み立てる（[`crate::skip_nav::stylesheet`] と同方針）。
//! docs 固有トークン（アクセント背景・構造寸法）は手書きの `--fandhe-*` 宣言
//! ではなく [`Theme::push_color`]/[`Theme::push_space`] の allowlist 検証付き
//! API 経由でのみ追加する（新たなエスケープ迂回経路を作らない）。

use fandhe_frontend_pre_styled_ui::theme::{Theme, ThemeError};
use fandhe_frontend_pre_styled_ui::{StyleSheet, StylesheetError};

/// サイト骨格 CSS の出力先（`out_dir` 起点の相対パス）。旧 `site/assets/site.css`
/// の静的コピー先と同一パスを維持し、`crate::layout` の `<link>` 契約・
/// `.github/workflows/docs-site.yml` の生成物検証を不変に保つ。
pub const STYLESHEET_REL_PATH: &str = "assets/site.css";

/// [`stylesheet`] の失敗理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SiteThemeError {
    /// docs 固有トークン（[`docs_theme`]）の追加が [`Theme`] の allowlist
    /// 検証に落ちた（既定値は allowlist を満たすよう手動検証済みの定数の
    /// ため通常は到達しない。fail-closed で伝播させる）。
    Theme(ThemeError),
    /// 構造 CSS（[`STRUCTURAL_CSS`]）の取り込みが [`StyleSheet::push_css`] の
    /// 検証に落ちた（`<`・制御文字を含み得ない定数のため通常は到達しない）。
    Stylesheet(StylesheetError),
}

impl std::fmt::Display for SiteThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SiteThemeError::Theme(e) => write!(f, "{e}"),
            SiteThemeError::Stylesheet(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for SiteThemeError {}

impl From<ThemeError> for SiteThemeError {
    fn from(e: ThemeError) -> Self {
        SiteThemeError::Theme(e)
    }
}

impl From<StylesheetError> for SiteThemeError {
    fn from(e: StylesheetError) -> Self {
        SiteThemeError::Stylesheet(e)
    }
}

/// [`Theme::default`] を基礎に、サイト骨格が使う docs 固有トークン
/// （現在ページリンクのアクセント背景・構造寸法 3 種）を追加する。
///
/// 色は [`Theme::push_color`]、寸法は [`Theme::push_space`] の allowlist
/// 検証付き API 経由で追加する（手書き `--fandhe-*` 宣言を [`StyleSheet::push_css`]
/// に持ち込まない）。
///
/// # Errors
///
/// 追加するトークン名・値が allowlist 検証を通過しない場合 [`ThemeError`] を
/// 返す。本関数内の値はすべて allowlist を満たす定数のため通常は到達しない。
fn docs_theme() -> Result<Theme, ThemeError> {
    let mut theme = Theme::default();

    // 現在ページのサイドバーリンク背景（`--fandhe-color-accent` 系と調和する
    // アクセント淡色。theme 既定パレットに accent-subtle 相当が無いため
    // docs 固有トークンとして追加する）。
    theme.push_color("docs-accent-bg", "#ebf4ff", "#1c2740")?;

    // 構造寸法（2 カラム grid のサイドバー幅・本文最大幅・ヘッダー高さ）。
    theme.push_space("docs-sidebar-width", "16rem")?;
    theme.push_space("docs-max-content-width", "46rem")?;
    theme.push_space("docs-header-height", "3.25rem")?;

    Ok(theme)
}

/// 旧 `site/assets/site.css` からトークン定義ブロック（`:root` の
/// `--docs-*` 宣言・`prefers-color-scheme: dark` メディアクエリ）を除いた
/// 構造 CSS。`var(--docs-...)` はすべて対応する `var(--fandhe-...)`
/// （[`docs_theme`] が追加したトークンを含む）へ置換済み。
///
/// セレクタ集合（class 契約）は旧 `site.css` から一切変更していない
/// （`crates/docs-site/tests/site_css_contract.rs` が乖離を検知する）。
/// フォントスタックは [`crate::theme::CssValue`] が引用符（`"`）を拒否する
/// ため（CJK フォールバックフォント名を持つ）トークン化せず、本定数へ
/// 直接記述する（[`StyleSheet::push_css`] は引用符を許容する）。
const STRUCTURAL_CSS: &str = "\
*,\n\
*::before,\n\
*::after {\n\
  box-sizing: border-box;\n\
}\n\
\n\
html {\n\
  color-scheme: light dark;\n\
}\n\
\n\
body {\n\
  margin: 0;\n\
  background: var(--fandhe-color-bg);\n\
  color: var(--fandhe-color-fg);\n\
  font-family: system-ui, -apple-system, \"Segoe UI\", \"Hiragino Sans\",\n\
    \"Noto Sans CJK JP\", \"Yu Gothic\", sans-serif;\n\
  font-size: 15.5px;\n\
  line-height: 1.65;\n\
  -webkit-font-smoothing: antialiased;\n\
}\n\
\n\
/* ---- ヘッダー ---- */\n\
\n\
.docs-header {\n\
  position: sticky;\n\
  top: 0;\n\
  z-index: 10;\n\
  display: flex;\n\
  align-items: center;\n\
  height: var(--fandhe-space-docs-header-height);\n\
  padding: 0 1.5rem;\n\
  border-bottom: 1px solid var(--fandhe-color-border);\n\
  background: var(--fandhe-color-bg);\n\
}\n\
\n\
.docs-header a {\n\
  font-weight: 600;\n\
  font-size: 0.95rem;\n\
  letter-spacing: -0.01em;\n\
  color: var(--fandhe-color-fg);\n\
  text-decoration: none;\n\
}\n\
\n\
/* ---- 2 カラムレイアウト ---- */\n\
\n\
.docs-container {\n\
  display: grid;\n\
  grid-template-columns: var(--fandhe-space-docs-sidebar-width) minmax(0, 1fr);\n\
  align-items: start;\n\
  max-width: 72rem;\n\
  margin: 0 auto;\n\
}\n\
\n\
/* ---- サイドバー ---- */\n\
\n\
.docs-sidebar {\n\
  position: sticky;\n\
  top: var(--fandhe-space-docs-header-height);\n\
  align-self: start;\n\
  max-height: calc(100vh - var(--fandhe-space-docs-header-height));\n\
  overflow-y: auto;\n\
  padding: 1.75rem 1rem 2rem;\n\
  border-right: 1px solid var(--fandhe-color-border);\n\
  font-size: 0.875rem;\n\
}\n\
\n\
.docs-sidebar nav.sidebar {\n\
  display: flex;\n\
  flex-direction: column;\n\
  gap: 0.2rem;\n\
}\n\
\n\
.docs-sidebar nav.sidebar h2 {\n\
  font-weight: 600;\n\
  font-size: 0.72rem;\n\
  letter-spacing: 0.06em;\n\
  text-transform: uppercase;\n\
  color: var(--fandhe-color-fg-muted);\n\
  margin: 1.1rem 0.5rem 0.3rem;\n\
}\n\
\n\
.docs-sidebar nav.sidebar h2:first-child {\n\
  margin-top: 0;\n\
}\n\
\n\
.docs-sidebar nav.sidebar ul {\n\
  list-style: none;\n\
  margin: 0 0 0.4rem;\n\
  padding: 0;\n\
  display: flex;\n\
  flex-direction: column;\n\
  gap: 0.05rem;\n\
}\n\
\n\
.docs-sidebar nav.sidebar li {\n\
  margin: 0;\n\
}\n\
\n\
.docs-sidebar nav.sidebar a {\n\
  display: block;\n\
  padding: 0.32rem 0.5rem;\n\
  border-radius: 0.4rem;\n\
  color: var(--fandhe-color-fg-muted);\n\
  text-decoration: none;\n\
  font-size: 0.875rem;\n\
  border-left: 2px solid transparent;\n\
}\n\
\n\
.docs-sidebar nav.sidebar a:hover {\n\
  color: var(--fandhe-color-fg);\n\
  background: var(--fandhe-color-bg-subtle);\n\
}\n\
\n\
.docs-sidebar nav.sidebar a[aria-current=\"page\"] {\n\
  background: var(--fandhe-color-docs-accent-bg);\n\
  color: var(--fandhe-color-accent);\n\
  border-left-color: var(--fandhe-color-accent);\n\
  font-weight: 600;\n\
}\n\
\n\
/* ---- 本文カラム ---- */\n\
\n\
.docs-main {\n\
  min-width: 0;\n\
  padding: 2.25rem 2rem 5rem;\n\
}\n\
\n\
.docs-content {\n\
  display: block;\n\
  max-width: var(--fandhe-space-docs-max-content-width);\n\
  margin: 0 auto;\n\
}\n\
\n\
.docs-content h1,\n\
.docs-content h2,\n\
.docs-content h3 {\n\
  line-height: 1.35;\n\
  font-weight: 650;\n\
  letter-spacing: -0.01em;\n\
}\n\
\n\
.docs-content h1 {\n\
  font-size: 2rem;\n\
  margin: 0 0 1.1rem;\n\
}\n\
\n\
.docs-content h2 {\n\
  font-size: 1.375rem;\n\
  margin: 2.25rem 0 0.85rem;\n\
  padding-top: 0.35rem;\n\
  border-top: 1px solid var(--fandhe-color-border);\n\
}\n\
\n\
.docs-content h3 {\n\
  font-size: 1.1rem;\n\
  margin: 1.6rem 0 0.5rem;\n\
}\n\
\n\
.docs-content p {\n\
  margin: 0 0 1.05rem;\n\
}\n\
\n\
.docs-content ul,\n\
.docs-content ol {\n\
  margin: 0 0 1.05rem;\n\
  padding-left: 1.4rem;\n\
}\n\
\n\
.docs-content li {\n\
  margin: 0.3rem 0;\n\
}\n\
\n\
.docs-content a {\n\
  color: var(--fandhe-color-accent);\n\
  text-decoration: none;\n\
}\n\
\n\
.docs-content a:hover {\n\
  text-decoration: underline;\n\
}\n\
\n\
.docs-content blockquote {\n\
  margin: 0 0 1.05rem;\n\
  padding: 0.5rem 1rem;\n\
  border-left: 3px solid var(--fandhe-color-border);\n\
  color: var(--fandhe-color-fg-muted);\n\
}\n\
\n\
.docs-content code {\n\
  font-family: ui-monospace, SFMono-Regular, \"SF Mono\", Consolas,\n\
    \"Liberation Mono\", monospace;\n\
  font-size: 0.875em;\n\
  color: var(--fandhe-color-fg);\n\
  background: var(--fandhe-color-bg-muted);\n\
  padding: 0.15em 0.4em;\n\
  border-radius: 0.35em;\n\
}\n\
\n\
.docs-content pre {\n\
  margin: 0 0 1.05rem;\n\
  padding: 1rem 1.1rem;\n\
  background: var(--fandhe-color-bg-muted);\n\
  border: 1px solid var(--fandhe-color-border);\n\
  border-radius: 0.6rem;\n\
  overflow-x: auto;\n\
}\n\
\n\
.docs-content pre code {\n\
  background: none;\n\
  border: none;\n\
  padding: 0;\n\
  font-size: 0.85em;\n\
}\n\
\n\
.docs-content table {\n\
  display: block;\n\
  overflow-x: auto;\n\
  border-collapse: collapse;\n\
  margin: 0 0 1.05rem;\n\
  max-width: 100%;\n\
  font-size: 0.925em;\n\
}\n\
\n\
.docs-content th,\n\
.docs-content td {\n\
  border: 1px solid var(--fandhe-color-border);\n\
  padding: 0.45rem 0.75rem;\n\
  text-align: left;\n\
}\n\
\n\
.docs-content th {\n\
  background: var(--fandhe-color-bg-subtle);\n\
  font-weight: 600;\n\
}\n\
\n\
/* ---- ページ内目次（任意、本文の前に配置） ---- */\n\
\n\
.docs-toc {\n\
  font-size: 0.85rem;\n\
  padding: 0.9rem 1.1rem;\n\
  margin: 0 auto 1.75rem;\n\
  max-width: var(--fandhe-space-docs-max-content-width);\n\
  border: 1px solid var(--fandhe-color-border);\n\
  border-radius: 0.65rem;\n\
  background: var(--fandhe-color-bg-subtle);\n\
}\n\
\n\
.docs-toc ul {\n\
  list-style: none;\n\
  margin: 0.5rem 0 0;\n\
  padding: 0;\n\
  display: flex;\n\
  flex-direction: column;\n\
  gap: 0.2rem;\n\
}\n\
\n\
.docs-toc a {\n\
  color: var(--fandhe-color-fg-muted);\n\
  text-decoration: none;\n\
}\n\
\n\
.docs-toc a:hover {\n\
  color: var(--fandhe-color-accent);\n\
}\n\
\n\
.docs-toc-level-2 {\n\
  padding-left: 0;\n\
}\n\
\n\
.docs-toc-level-3 {\n\
  padding-left: 1rem;\n\
}\n\
\n\
/* ---- 前後ページナビ（本文末尾、カード風） ---- */\n\
\n\
nav.prev-next {\n\
  display: flex;\n\
  gap: 0.75rem;\n\
  margin: 2.5rem 0 0;\n\
  max-width: var(--fandhe-space-docs-max-content-width);\n\
}\n\
\n\
nav.prev-next .prev,\n\
nav.prev-next .next {\n\
  flex: 1 1 0;\n\
  display: flex;\n\
}\n\
\n\
nav.prev-next [data-part=\"overlay\"] {\n\
  flex: 1 1 0;\n\
  display: block;\n\
  padding: 0.85rem 1rem;\n\
  border: 1px solid var(--fandhe-color-border);\n\
  border-radius: 0.65rem;\n\
  color: var(--fandhe-color-fg);\n\
  text-decoration: none;\n\
  font-size: 0.9rem;\n\
  font-weight: 500;\n\
  background: var(--fandhe-color-bg-subtle);\n\
}\n\
\n\
nav.prev-next [data-part=\"overlay\"]:hover {\n\
  border-color: var(--fandhe-color-accent);\n\
  color: var(--fandhe-color-accent);\n\
}\n\
\n\
nav.prev-next .prev [data-part=\"overlay\"] {\n\
  text-align: left;\n\
}\n\
\n\
nav.prev-next .next [data-part=\"overlay\"] {\n\
  text-align: right;\n\
}\n\
\n\
/* ---- モバイル対応: 768px 以下で 1 カラムに畳む ---- */\n\
\n\
@media (max-width: 768px) {\n\
  .docs-container {\n\
    display: block;\n\
  }\n\
\n\
  .docs-sidebar {\n\
    position: static;\n\
    max-height: none;\n\
    width: 100%;\n\
    border-right: none;\n\
    border-bottom: 1px solid var(--fandhe-color-border);\n\
    padding: 1rem;\n\
  }\n\
\n\
  .docs-main {\n\
    padding: 1.5rem 1.1rem 3rem;\n\
  }\n\
\n\
  nav.prev-next {\n\
    flex-direction: column;\n\
  }\n\
\n\
  nav.prev-next .next [data-part=\"overlay\"] {\n\
    text-align: left;\n\
  }\n\
}\n\
";

/// サイト骨格が参照する CSS 全量を組み立てる。
///
/// 内訳: テーマトークン（[`docs_theme`]、`Theme::default` + docs 固有拡張）
/// → [`STRUCTURAL_CSS`]（構造 CSS）の順で決定的に連結する（[`crate::skip_nav::stylesheet`]
/// と同型の組み立て順）。
///
/// # Errors
///
/// [`docs_theme`] のトークン追加、または [`StyleSheet::push_css`] の検証
/// （`<`・制御文字の拒否）に落ちた場合 [`SiteThemeError`] を返す。本関数内の
/// 値はすべて allowlist を満たす定数のため通常は到達しないが、黙って欠けた
/// CSS を公開しない fail-closed 方針で伝播させる。
pub fn stylesheet() -> Result<StyleSheet, SiteThemeError> {
    let theme = docs_theme()?;
    let mut sheet = StyleSheet::new();
    sheet.push_theme(&theme);
    sheet.push_css(STRUCTURAL_CSS)?;
    Ok(sheet)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stylesheet_contains_theme_tokens_and_dark_mode_blocks() {
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();
        assert!(css.contains("--fandhe-color-bg:"));
        assert!(css.contains("@media (prefers-color-scheme: dark)"));
        assert!(css.contains(":root[data-theme=\"dark\"]"));
        assert!(css.contains("--fandhe-color-docs-accent-bg:"));
        assert!(css.contains("--fandhe-space-docs-sidebar-width:"));
    }

    #[test]
    fn stylesheet_contains_structural_selectors() {
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();
        for selector in [
            ".docs-header",
            ".docs-container",
            ".docs-sidebar",
            ".docs-main",
            ".docs-content",
            ".docs-toc",
            "nav.prev-next",
        ] {
            assert!(css.contains(selector), "missing selector: {selector}");
        }
    }

    #[test]
    fn stylesheet_never_references_external_resources() {
        // 受け入れ条件 2（外部参照ゼロ）を機械的に固定する。従来は
        // `site/assets/site.css` のコメントによる「運用担保」に留まっていた。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();
        assert!(!css.contains("@import"));
        assert!(!css.contains("@font-face"));
        assert!(!css.contains("url("));
    }

    #[test]
    fn stylesheet_contains_no_docs_prefixed_tokens() {
        // トークン一本化（`--docs-*` 廃止）を fail-closed で固定する。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        assert!(!sheet.as_css().contains("--docs-"));
    }

    #[test]
    fn stylesheet_is_deterministic() {
        let a = stylesheet().unwrap().as_css().to_string();
        let b = stylesheet().unwrap().as_css().to_string();
        assert_eq!(a, b);
    }

    #[test]
    fn stylesheet_never_contains_angle_brackets() {
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        assert!(!sheet.as_css().contains('<'));
    }
}
