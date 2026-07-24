//! docs サイト骨格（ヘッダー・3 カラム grid・サイドバー・本文タイポグラフィ・
//! toc・前後ナビ）が使う CSS の配線（イシュー #905、3 カラム化はイシュー #907）。
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
//! 左ナビ（`crate::nav::sidebar`）は headless `nav_list` markup をそのまま
//! 出力し、[`stylesheet`] が `fandhe_frontend_pre_styled_ui::nav_list::stylesheet()`
//! を基底 CSS として取り込む（イシュー #910）。markup（`data-scope`/
//! `data-part`・`aria-current`）は変更せず、docs 固有の視覚差分のみ
//! [`STRUCTURAL_CSS`] 側の `.docs-sidebar nav.sidebar ...` セレクタで上乗せ
//! する。前後ページャ（`nav.prev-next`、headless `link_overlay`）は
//! `link_overlay::stylesheet()` を採用しない（`overlay` が唯一の子要素の
//! カードに `position: absolute` を適用すると高さが 0 に潰れるため。
//! `crate::nav::prev_next_nav` rustdoc 参照）。トークンベースのカード風
//! CSS を [`STRUCTURAL_CSS`] 側で手書きのまま維持する。
//!
//! # クラス名契約（`crates/docs-site/src/layout.rs` / `nav.rs` / `markdown.rs` の
//! 実出力が正。ここに書かれたセレクタはすべて実際に生成される class 値であり、
//! `crates/docs-site/tests/site_css_contract.rs` が両者の乖離を検知する）
//!
//! ```text
//! body
//!   header.docs-header            … ヘッダバー（サイトタイトルへのリンク）
//!   div.docs-container            … 3 カラムのグリッドコンテナ（header の下、
//!     イシュー #907・`docs/design/docs-site-three-column-redesign.md` §3.1）。
//!     見出しの無いページでは `docs-container--no-toc` 修飾 class が付く
//!     （`crate::layout::docs_page_with_assets` 参照）
//!       aside.docs-sidebar        … 左ナビカラムのラッパー
//!         input.docs-sidebar-toggle（先頭・sr-only）… `< 768px` の折りたたみ
//!           開閉状態を保持するチェックボックス（`:checked` が唯一の情報源）
//!         label.docs-sidebar-toggle-label … 上記チェックボックスの可視トリガー
//!           （`min-width: 768px` では非表示）
//!         nav.sidebar              … `nav::sidebar()` の実出力（headless nav_list）
//!           a[aria-current="page"] … 現在ページのリンク
//!       main.docs-main            … 中央コンテンツカラムのラッパー
//!         article.docs-content    … Markdown レンダラの出力 + `nav.prev-next`
//!           - `pre > code.language-*` … フェンス付きコードブロック
//!       aside.docs-toc-aside（任意・見出しが存在するページのみ）
//!         … 右目次カラムのラッパー
//!         nav.docs-toc            … ページ内目次
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

use fandhe_frontend_pre_styled_ui::css::{decl, serialize_rule, Declaration};
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
/// （現在ページリンクのアクセント背景・構造寸法 4 種）を追加する。
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

    // 構造寸法（3 カラム grid の左ナビ幅・本文最大幅・ヘッダー高さ・
    // 右目次カラム幅）。
    theme.push_space("docs-sidebar-width", "16rem")?;
    theme.push_space("docs-max-content-width", "46rem")?;
    theme.push_space("docs-header-height", "3.25rem")?;
    // 右目次カラム（`aside.docs-toc-aside`）の列幅。3 カラム表示になる
    // `min-width: 1200px` 以上でのみ参照される（イシュー #907）。sticky 追従・
    // 視覚スタイルの仕上げは #909 スコープのため、ここでは列幅のみ定義する。
    theme.push_space("docs-toc-width", "14rem")?;

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
/*\n\
 * ---- 3 カラムレイアウト（イシュー #907、mobile-first） ----\n\
 *\n\
 * 基底（768px 未満）は 1 カラム縦積み。`min-width: 768px` で左ナビ + 中央\n\
 * コンテンツの 2 カラム grid、`min-width: 1200px` で右目次カラムを加えた\n\
 * 3 カラム grid になる（設計文書 §3.2）。右目次カラムの sticky 追従・\n\
 * 視覚スタイルの仕上げは #909。左ナビの折りたたみトグル UI・pre-styled-ui\n\
 * スタイル適用は #910 で完了した（`nav_list::stylesheet()` の配線、下記\n\
 * 「サイドバー（左ナビ）」節参照）。\n\
 */\n\
\n\
.docs-container {\n\
  display: block;\n\
  max-width: 72rem;\n\
  margin: 0 auto;\n\
}\n\
\n\
/*\n\
 * ---- サイドバー（左ナビ、イシュー #910） ----\n\
 *\n\
 * headless `nav_list`（`nav::sidebar()` の実出力、`data-scope=\"nav-list\"\n\
 * data-part=\"heading|list|item|link\"`）へ `nav_list::stylesheet()`（下記\n\
 * `stylesheet()` 関数、`push_theme` の直後に `push_css`）が list-style 除去・\n\
 * `ul`/`h2` の margin リセット・link の `display: block`/`text-decoration:\n\
 * none`・`[aria-current=\"page\"]` の accent 色を基底として提供する。\n\
 * ここでは docs 固有の差分（余白・角丸・hover 背景・現在ページの\n\
 * アクセントバー）のみを `.docs-sidebar nav.sidebar` の詳細度\n\
 * （0,2,1・`[data-scope][data-part]` の 0,2,0 より高い）で上書きする\n\
 * （両者の連結順は [`stylesheet`] 参照。docs 側セレクタが常にカスケード\n\
 * 後方かつ高詳細度のため衝突しない）。\n\
 */\n\
\n\
.docs-sidebar {\n\
  width: 100%;\n\
  padding: 1rem;\n\
  border-right: none;\n\
  border-bottom: 1px solid var(--fandhe-color-border);\n\
  font-size: var(--fandhe-font-font-size-sm);\n\
}\n\
\n\
/*\n\
 * 折りたたみ対象は `nav.sidebar`（`aside.docs-sidebar` ではなく）。基底\n\
 * （768px 未満）は `max-height` で 1 行程度に折りたたむ（JS 不要、`layout.rs`\n\
 * モジュール doc 参照）。開閉状態の唯一の情報源はチェックボックスの\n\
 * `:checked` とする（focus-within 系疑似クラスを OR で加えない）。フォーカスが\n\
 * ナビ内に残ったままチェックを外しても閉じられなくなる回帰があったため\n\
 * （Bugbot 指摘 #916 是正）。チェックボックス自体は sr-only パターンで\n\
 * 視覚的に隠すのみで DOM から除去しないため（`.docs-sidebar-toggle`\n\
 * 参照）、クリップされたリンクへも Tab で到達でき、チェックボックス自体を\n\
 * Space キーで開閉できる（a11y 上の後退なし）。\n\
 */\n\
.docs-sidebar nav.sidebar {\n\
  max-height: 2.75rem;\n\
  overflow: hidden;\n\
}\n\
\n\
.docs-sidebar-toggle:checked ~ nav.sidebar {\n\
  max-height: none;\n\
  overflow: visible;\n\
}\n\
\n\
/*\n\
 * `.docs-sidebar-toggle`（input 要素（type=\"checkbox\"））は sr-only パターンで\n\
 * 視覚的に隠すのみで DOM からは除去しない（Tab フォーカス・Enter/Space\n\
 * 操作の対象から外さない。`display: none`/`visibility: hidden` にしない\n\
 * 理由）。`min-width: 768px`（折りたたみ自体を行わない帯域）では\n\
 * `display: none` に切り替える（下記 `@media` ブロック。この帯域では\n\
 * サイドバーが常時全展開のためチェックボックスの状態自体が意味を持たなく\n\
 * なり、キーボードユーザーが無意味なコントロールへ無駄にフォーカスを\n\
 * 奪われる回帰を避ける、Bugbot 指摘 #916 是正）。\n\
 */\n\
.docs-sidebar-toggle {\n\
  position: absolute;\n\
  width: 1px;\n\
  height: 1px;\n\
  padding: 0;\n\
  margin: -1px;\n\
  overflow: hidden;\n\
  clip: rect(0, 0, 0, 0);\n\
  white-space: nowrap;\n\
  border: 0;\n\
}\n\
\n\
/*\n\
 * `.docs-sidebar-toggle-label` はチェックボックスの可視トリガー。基底\n\
 * （768px 未満、折りたたみが有効な帯域）でのみ表示し、`min-width: 768px`\n\
 * （折りたたみ自体を行わない帯域、下記 `@media` ブロック）では非表示にする。\n\
 */\n\
.docs-sidebar-toggle-label {\n\
  display: block;\n\
  cursor: pointer;\n\
  padding: 0.4rem 0.6rem;\n\
  margin: -0.4rem -0.6rem 0.4rem;\n\
  border-radius: 0.4rem;\n\
  font-size: 0.8rem;\n\
  font-weight: var(--fandhe-font-font-weight-semibold);\n\
  color: var(--fandhe-color-accent);\n\
}\n\
\n\
.docs-sidebar-toggle-label:hover {\n\
  background: var(--fandhe-color-bg-subtle);\n\
}\n\
\n\
.docs-sidebar nav.sidebar {\n\
  display: flex;\n\
  flex-direction: column;\n\
  gap: 0.2rem;\n\
}\n\
\n\
.docs-sidebar nav.sidebar h2 {\n\
  font-weight: var(--fandhe-font-font-weight-semibold);\n\
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
  padding: 0.32rem 0.5rem;\n\
  border-radius: 0.4rem;\n\
  color: var(--fandhe-color-fg-muted);\n\
  font-size: var(--fandhe-font-font-size-sm);\n\
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
  font-weight: var(--fandhe-font-font-weight-semibold);\n\
}\n\
\n\
/* ---- 本文カラム ---- */\n\
\n\
.docs-main {\n\
  min-width: 0;\n\
  padding: 1.5rem 1.1rem 3rem;\n\
}\n\
\n\
.docs-content {\n\
  display: block;\n\
  max-width: var(--fandhe-space-docs-max-content-width);\n\
  margin: 0 auto;\n\
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
/* ---- 右目次カラム（`aside.docs-toc-aside`。イシュー #907） ---- */\n\
\n\
/*\n\
 * 基底・768px 帯域（2 カラム）では非表示。狭幅では「右目次 → 左ナビ」の\n\
 * 順に畳む要件どおり、目次カラムのみを先に隠す。1200px 以上で表示に切り替える\n\
 * （`@media (min-width: 1200px)` 側、下記）。sticky 追従・視覚仕上げは #909。\n\
 */\n\
.docs-toc-aside {\n\
  display: none;\n\
}\n\
\n\
/* ---- 前後ページナビ（本文末尾、カード風） ---- */\n\
\n\
nav.prev-next {\n\
  display: flex;\n\
  /* 基底（768px 未満）は縦積み。`min-width: 768px` で横並びに切り替える。 */\n\
  flex-direction: column;\n\
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
  font-weight: var(--fandhe-font-font-weight-medium);\n\
  background: var(--fandhe-color-bg-subtle);\n\
}\n\
\n\
/*\n\
 * hover 時はサイドバー現在ページと同じアクセント配色（枠線・文字色・\n\
 * `--fandhe-color-docs-accent-bg` 背景）で統一し、左ナビ・前後ページャの\n\
 * アクセント表現をトークンベースで揃える（イシュー #910）。`link_overlay::\n\
 * stylesheet()` は取り込まない（`overlay` が唯一の子要素のカードで\n\
 * `position: absolute` を適用すると高さが 0 に潰れるため、`site_theme.rs`\n\
 * 側の手書きカード CSS を維持する。`crate::nav::prev_next_nav` rustdoc 参照）。\n\
 */\n\
nav.prev-next [data-part=\"overlay\"]:hover {\n\
  border-color: var(--fandhe-color-accent);\n\
  color: var(--fandhe-color-accent);\n\
  background: var(--fandhe-color-docs-accent-bg);\n\
}\n\
\n\
nav.prev-next .prev [data-part=\"overlay\"] {\n\
  text-align: left;\n\
}\n\
\n\
/*\n\
 * 基底（768px 未満、縦積み）では `.prev` と同じ左揃え。`min-width: 768px` で\n\
 * 横並びに切り替わった際に右揃えへ変える（下記 `@media` ブロック参照）。\n\
 */\n\
nav.prev-next .next [data-part=\"overlay\"] {\n\
  text-align: left;\n\
}\n\
\n\
/*\n\
 * ---- `min-width: 768px`: 左ナビ + 中央コンテンツの 2 カラム grid ----\n\
 *\n\
 * 右目次カラム（`.docs-toc-aside`）はこの帯域ではまだ非表示のまま\n\
 * （狭幅では「右目次 → 左ナビ」の順に畳む要件、設計文書 §3.2）。\n\
 */\n\
@media (min-width: 768px) {\n\
  .docs-container {\n\
    display: grid;\n\
    grid-template-columns: var(--fandhe-space-docs-sidebar-width) minmax(0, 1fr);\n\
    align-items: start;\n\
  }\n\
\n\
  .docs-sidebar {\n\
    /*\n\
     * `.docs-header` が sticky top: 0 で常時可視のため、`.docs-sidebar` は\n\
     * ヘッダー分オフセットして常時可視のヘッダー直下に張り付く。\n\
     */\n\
    position: sticky;\n\
    top: var(--fandhe-space-docs-header-height);\n\
    align-self: start;\n\
    max-height: calc(100vh - var(--fandhe-space-docs-header-height));\n\
    overflow-y: auto;\n\
    width: auto;\n\
    padding: 1.75rem 1rem 2rem;\n\
    border-right: 1px solid var(--fandhe-color-border);\n\
    border-bottom: none;\n\
  }\n\
\n\
  /* `min-width: 768px` では折りたたみ自体を行わないため、`nav.sidebar` の\n\
   * 折りたたみ制約とタッチ用トグルの可視トリガーの双方を解除する。 */\n\
  .docs-sidebar nav.sidebar {\n\
    max-height: none;\n\
    overflow: visible;\n\
  }\n\
\n\
  /* この帯域ではサイドバーが常時全展開のためチェックボックス自体を DOM\n\
   * 上から視覚的に取り除くだけでなく操作対象からも外す（Bugbot 指摘 #916\n\
   * 是正、上記 `.docs-sidebar-toggle` doc コメント参照）。 */\n\
  .docs-sidebar-toggle {\n\
    display: none;\n\
  }\n\
\n\
  .docs-sidebar-toggle-label {\n\
    display: none;\n\
  }\n\
\n\
  .docs-main {\n\
    padding: 2.25rem 2rem 5rem;\n\
  }\n\
\n\
  nav.prev-next {\n\
    flex-direction: row;\n\
  }\n\
\n\
  nav.prev-next .next [data-part=\"overlay\"] {\n\
    text-align: right;\n\
  }\n\
}\n\
\n\
/*\n\
 * ---- `min-width: 1200px`: 左ナビ + 中央コンテンツ + 右目次の 3 カラム grid ----\n\
 *\n\
 * `.docs-toc-aside` をこの帯域から表示に切り替える。見出しの無いページ\n\
 * （`div.docs-container.docs-container--no-toc`、`crate::layout` が\n\
 * `aside.docs-toc-aside` 自体を出力しない場合に付与する修飾 class）では\n\
 * 右目次列のグリッドトラック自体を収縮させ、空の右カラムが残ったまま\n\
 * 中央カラムが狭くなる回帰を避ける（Bugbot 指摘 #916 是正）。\n\
 */\n\
@media (min-width: 1200px) {\n\
  .docs-container {\n\
    grid-template-columns:\n\
      var(--fandhe-space-docs-sidebar-width) minmax(0, 1fr)\n\
      var(--fandhe-space-docs-toc-width);\n\
  }\n\
\n\
  .docs-container.docs-container--no-toc {\n\
    grid-template-columns: var(--fandhe-space-docs-sidebar-width) minmax(0, 1fr);\n\
  }\n\
\n\
  .docs-toc-aside {\n\
    display: block;\n\
    min-width: 0;\n\
    padding: 1.75rem 1rem 2rem;\n\
  }\n\
}\n\
";

/// [`typography_css`] が組み立てる 1 セレクタ分の規則を `out` へ追記する
/// 内部ヘルパ。[`serialize_rule`] は宣言がすべて invalid の場合のみ `None`
/// を返す（`crates/pre-styled-ui/src/css.rs` 参照）が、本関数が渡す宣言は
/// すべてソースコード中のリテラル定数のため通常は到達しない。到達した
/// 場合も黙って欠けた CSS を出さず fail-closed で [`SiteThemeError`] へ
/// 伝播させる（本モジュール冒頭のセキュリティ不変条件と同方針）。
fn push_typography_rule(
    out: &mut String,
    selector: &str,
    declarations: &[Declaration],
) -> Result<(), SiteThemeError> {
    match serialize_rule(selector, declarations) {
        Some(rule) => {
            out.push_str(&rule);
            Ok(())
        }
        None => Err(SiteThemeError::Stylesheet(StylesheetError::CssRejected {
            reason: "typography rule produced no valid declarations",
        })),
    }
}

/// 本文タイポグラフィ（`.docs-content` 配下）の CSS を組み立てる（イシュー
/// #911）。[`crate::markdown`] が出力するタグ集合（h1〜h6/p/ul/ol/li/pre/code/
/// blockquote/table 系/em/strong/a、実出力が正）に対し、`.docs-content <tag>`
/// の**タグセレクタ**として反映する。[`crate::markdown`] 自体の出力（素の
/// HTML タグ、既定エスケープ経由）・class 付与ロジックは一切変更しない
/// （`docs/design/docs-site-three-column-redesign.md` §3.6 の不変条件）。
///
/// 各規則は 2 群の宣言からなる:
/// - **ミラー宣言**: `fandhe_frontend_pre_styled_ui` の対応部品 recipe
///   （下表）が生成する宣言と値を一致させる（[`crate::pre-styled-ui`] 本体は
///   変更しない。`crates/pre-styled-ui/src/css.rs` の [`decl`]/[`serialize_rule`]
///   を本モジュールから直接呼び、規則文字列を独自に組み立てる）
/// - **docs 固有宣言**: 部品側に対応が無い、Markdown 文書のブロックフロー
///   装飾（見出し間の `margin`・`h2` の `border-top`・コードブロックの背景/
///   罫線・テーブルの横スクロール等）。部品は `margin: 0` 基調のため、
///   文書の縦方向の余白は docs 側の責務として維持する
///
/// # タグ → 部品/variant 対応表
///
/// | タグ | 部品 / variant | 備考 |
/// |---|---|---|
/// | `h1` | [`fandhe_frontend_pre_styled_ui::heading`] base + `HeadingSize::Xl3` | |
/// | `h2` | 同 base + `Xl2` | `border-top`/`padding-top` は docs 固有 |
/// | `h3` | 同 base + `Xl` | |
/// | `h4` | 同 base + `Lg` | 現行 STRUCTURAL_CSS 未対応だった新規スタイル |
/// | `h5` | 同 base + `Md` | 同上 |
/// | `h6` | 同 base + `Sm` | 同上 |
/// | `p` | [`fandhe_frontend_pre_styled_ui::text`] `TextSize::Md` | |
/// | `ul`/`ol` | [`fandhe_frontend_pre_styled_ui::list`] `ListVariant::Marker`（root） | |
/// | `li` | 同 item base | |
/// | `a` | [`fandhe_frontend_pre_styled_ui::link`] base | hover 下線は docs 固有 |
/// | `blockquote` | [`fandhe_frontend_pre_styled_ui::blockquote`] root base + `Subtle` variant | `--fandhe-palette` 系カスタムプロパティは本文脈で常に accent 固定のため `var(--fandhe-color-accent)` 系トークンへ直接解決する（docs 側の意図的な単純化。palette 切り替え UI を持たない） |
/// | `code`（インライン） | [`fandhe_frontend_pre_styled_ui::code`] base | `pre code` のリセットと `pre` 自体のブロック装飾は docs 固有 |
/// | `em` | [`fandhe_frontend_pre_styled_ui::em`] | |
/// | `table`/`th`/`td`/`strong` | 対応部品なし | 現行トークンベーススタイル・ブラウザ既定を維持（対象外、PR 本文参照） |
///
/// `kbd` は [`crate::markdown`] が出力しないため対象外（死に CSS を追加
/// しない。将来 `kbd` 出力構文を導入する際に本方式で追加する）。
///
/// # Errors
///
/// [`push_typography_rule`] が fail-closed でエラーを返した場合に伝播する
/// （本関数内の宣言はすべて allowlist を満たす定数のため通常は到達しない）。
fn typography_css() -> Result<String, SiteThemeError> {
    let mut out = String::new();

    // ---- 見出し（heading base + サイズ variant のミラー、docs 固有の margin）----
    push_typography_rule(
        &mut out,
        ".docs-content h1",
        &[
            decl("margin", "0 0 1.1rem"),
            decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
            decl("letter-spacing", "-0.01em"),
            decl("font-size", "var(--fandhe-font-font-size-3xl)"),
            decl("line-height", "1.2"),
        ],
    )?;
    push_typography_rule(
        &mut out,
        ".docs-content h2",
        &[
            decl("margin", "2.25rem 0 0.85rem"),
            decl("padding-top", "0.35rem"),
            decl("border-top", "1px solid var(--fandhe-color-border)"),
            decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
            decl("letter-spacing", "-0.01em"),
            decl("font-size", "var(--fandhe-font-font-size-2xl)"),
            decl("line-height", "1.25"),
        ],
    )?;
    push_typography_rule(
        &mut out,
        ".docs-content h3",
        &[
            decl("margin", "1.6rem 0 0.5rem"),
            decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
            decl("letter-spacing", "-0.01em"),
            decl("font-size", "var(--fandhe-font-font-size-xl)"),
            decl("line-height", "1.3"),
        ],
    )?;
    push_typography_rule(
        &mut out,
        ".docs-content h4",
        &[
            decl("margin", "1.4rem 0 0.5rem"),
            decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
            decl("letter-spacing", "-0.01em"),
            decl("font-size", "var(--fandhe-font-font-size-lg)"),
            decl("line-height", "1.3"),
        ],
    )?;
    push_typography_rule(
        &mut out,
        ".docs-content h5",
        &[
            decl("margin", "1.2rem 0 0.4rem"),
            decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
            decl("letter-spacing", "-0.01em"),
            decl("font-size", "var(--fandhe-font-font-size-md)"),
            decl("line-height", "1.3"),
        ],
    )?;
    push_typography_rule(
        &mut out,
        ".docs-content h6",
        &[
            decl("margin", "1.1rem 0 0.4rem"),
            decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
            decl("letter-spacing", "-0.01em"),
            decl("font-size", "var(--fandhe-font-font-size-sm)"),
            decl("line-height", "1.25"),
        ],
    )?;

    // ---- 段落（text base + TextSize::Md のミラー、docs 固有の margin）----
    push_typography_rule(
        &mut out,
        ".docs-content p",
        &[
            decl("margin", "0 0 1.05rem"),
            decl("font-size", "var(--fandhe-font-font-size-md)"),
            decl("line-height", "1.5"),
        ],
    )?;

    // ---- リスト（list base/Marker variant/item base のミラー、docs 固有の margin）----
    push_typography_rule(
        &mut out,
        ".docs-content ul,\n.docs-content ol",
        &[
            decl("margin", "0 0 1.05rem"),
            decl("list-style", "revert"),
            decl("padding-inline-start", "1.5rem"),
        ],
    )?;
    push_typography_rule(
        &mut out,
        ".docs-content li",
        &[decl("margin-block", "0.25rem"), decl("line-height", "1.5")],
    )?;

    // ---- リンク（link base のミラー、hover 下線は docs 固有）----
    push_typography_rule(
        &mut out,
        ".docs-content a",
        &[
            decl(
                "color",
                "var(--fandhe-color-accent, var(--fandhe-color-fg))",
            ),
            decl(
                "text-decoration",
                "var(--fandhe-link-text-decoration, none)",
            ),
            decl("cursor", "pointer"),
        ],
    )?;
    push_typography_rule(
        &mut out,
        ".docs-content a:hover",
        &[decl("text-decoration", "underline")],
    )?;

    // ---- 引用（blockquote root base + Subtle variant のミラー。
    // `--fandhe-palette` は本文脈で常に accent 固定のため
    // `var(--fandhe-color-accent...)` へ直接解決する。docs 固有の margin・
    // caption 文字色相当の本文色を併せ持つ）----
    push_typography_rule(
        &mut out,
        ".docs-content blockquote",
        &[
            decl("margin", "0 0 1.05rem"),
            decl("padding-inline-start", "1rem"),
            decl("padding-block", "0.5rem"),
            decl("background", "var(--fandhe-color-bg-subtle)"),
            decl(
                "border-inline-start",
                "4px solid var(--fandhe-color-accent)",
            ),
            decl("border-radius", "var(--fandhe-radius-sm)"),
            decl("color", "var(--fandhe-color-fg-muted)"),
        ],
    )?;

    // ---- インラインコード（code base のミラー）----
    push_typography_rule(
        &mut out,
        ".docs-content code",
        &[
            decl(
                "font-family",
                "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
            ),
            decl("background", "var(--fandhe-color-bg-subtle)"),
            decl("border-radius", "var(--fandhe-radius-sm)"),
            decl("padding", "0.0625rem 0.375rem"),
            decl("font-size", "var(--fandhe-font-font-size-sm)"),
            decl("color", "var(--fandhe-color-fg)"),
        ],
    )?;

    // ---- フェンス付きコードブロック（pre 自体・pre 内 code のリセットは
    // 対応する pre-styled-ui 部品を持たない docs 固有装飾）----
    push_typography_rule(
        &mut out,
        ".docs-content pre",
        &[
            decl("margin", "0 0 1.05rem"),
            decl("padding", "1rem 1.1rem"),
            decl("background", "var(--fandhe-color-bg-muted)"),
            decl("border", "1px solid var(--fandhe-color-border)"),
            decl("border-radius", "0.6rem"),
            decl("overflow-x", "auto"),
        ],
    )?;
    push_typography_rule(
        &mut out,
        ".docs-content pre code",
        &[
            decl("background", "none"),
            decl("border", "none"),
            decl("padding", "0"),
            decl("font-size", "0.85em"),
        ],
    )?;

    // ---- 強調（em のミラー）----
    push_typography_rule(
        &mut out,
        ".docs-content em",
        &[
            decl("font-style", "italic"),
            decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
        ],
    )?;

    // ---- テーブル（対応する pre-styled-ui 部品なし、対象外事項として
    // 現行トークンベーススタイルを維持）----
    push_typography_rule(
        &mut out,
        ".docs-content table",
        &[
            decl("display", "block"),
            decl("overflow-x", "auto"),
            decl("border-collapse", "collapse"),
            decl("margin", "0 0 1.05rem"),
            decl("max-width", "100%"),
            decl("font-size", "0.925em"),
        ],
    )?;
    push_typography_rule(
        &mut out,
        ".docs-content th,\n.docs-content td",
        &[
            decl("border", "1px solid var(--fandhe-color-border)"),
            decl("padding", "0.45rem 0.75rem"),
            decl("text-align", "left"),
        ],
    )?;
    push_typography_rule(
        &mut out,
        ".docs-content th",
        &[
            decl("background", "var(--fandhe-color-bg-subtle)"),
            decl("font-weight", "600"),
        ],
    )?;

    Ok(out)
}

/// サイト骨格が参照する CSS 全量を組み立てる。
///
/// 内訳: テーマトークン（[`docs_theme`]、`Theme::default` + docs 固有拡張）
/// → [`fandhe_frontend_pre_styled_ui::nav_list::stylesheet`]（styled NavList
/// のコンポーネント CSS。`nav::sidebar()` の実出力である headless `nav_list`
/// markup — `data-scope="nav-list" data-part="heading|list|item|link"` —
/// へそのまま適用される。イシュー #910）→ [`STRUCTURAL_CSS`]（構造 CSS）
/// → [`typography_css`]（本文タイポグラフィ、イシュー #911）の順で決定的に
/// 連結する（[`crate::skip_nav::stylesheet`] と同型の組み立て順）。この順序
/// により、`nav_list` コンポーネント基底（セレクタ詳細度 0,2,0）が先に出力
/// され、docs 固有の `.docs-sidebar nav.sidebar ...` セレクタ（詳細度 0,2,1
/// 以上）が後方かつ高詳細度で常に上書きする（CSS カスケード衝突なし。詳細
/// は [`STRUCTURAL_CSS`] のサイドバー節コメント参照）。
///
/// # Errors
///
/// [`docs_theme`] のトークン追加、[`StyleSheet::push_css`] の検証
/// （`<`・制御文字の拒否）、または [`typography_css`] の組み立てに落ちた場合
/// [`SiteThemeError`] を返す。本関数内の値はすべて allowlist を満たす定数の
/// ため通常は到達しないが、黙って欠けた CSS を公開しない fail-closed 方針で
/// 伝播させる。
pub fn stylesheet() -> Result<StyleSheet, SiteThemeError> {
    let theme = docs_theme()?;
    let mut sheet = StyleSheet::new();
    sheet.push_theme(&theme);
    sheet.push_css(&fandhe_frontend_pre_styled_ui::nav_list::stylesheet())?;
    sheet.push_css(STRUCTURAL_CSS)?;
    sheet.push_css(&typography_css()?)?;
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
            ".docs-sidebar-toggle",
            ".docs-sidebar-toggle-label",
            ".docs-main",
            ".docs-content",
            ".docs-toc",
            ".docs-toc-aside",
            "nav.prev-next",
        ] {
            assert!(css.contains(selector), "missing selector: {selector}");
        }
    }

    #[test]
    fn stylesheet_contains_typography_selectors_for_all_markdown_tags() {
        // 受け入れ条件 1（本文の見出し・段落・リスト・引用・コードが
        // pre-styled-ui のタイポグラフィで表示される）のセレクタ存在検証。
        // h4〜h6 は旧 STRUCTURAL_CSS では未対応だった（イシュー #911）。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();
        for selector in [
            ".docs-content h1",
            ".docs-content h2",
            ".docs-content h3",
            ".docs-content h4",
            ".docs-content h5",
            ".docs-content h6",
            ".docs-content p",
            ".docs-content ul",
            ".docs-content ol",
            ".docs-content li",
            ".docs-content a",
            ".docs-content blockquote",
            ".docs-content code",
            ".docs-content pre",
            ".docs-content em",
            ".docs-content table",
            ".docs-content th",
            ".docs-content td",
        ] {
            assert!(css.contains(selector), "missing selector: {selector}");
        }
    }

    #[test]
    fn stylesheet_typography_references_font_scale_tokens() {
        // 見出しサイズが pre-styled-ui のタイポグラフィスケール
        // （`--fandhe-font-font-size-*`）へ接続されていることを固定する
        // （旧実装は `2rem` 等のハードコード値だった）。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();
        for token in [
            "font-size-3xl", // h1
            "font-size-2xl", // h2
            "font-size-xl",  // h3
            "font-size-lg",  // h4
            "font-size-md",  // h5, p
            "font-size-sm",  // h6, code
        ] {
            assert!(
                css.contains(&format!("var(--fandhe-font-{token})")),
                "missing font scale token: {token}"
            );
        }
    }

    #[test]
    fn stylesheet_sidebar_open_state_has_no_focus_within_fallback() {
        // Bugbot 指摘（PR #916）是正の回帰テスト: `.docs-sidebar-toggle:checked`
        // のみが折りたたみナビの開状態の情報源であり、`:focus-within` を OR で
        // 加えていないことを固定する。チェックを外してもフォーカスがナビ内に
        // 残っている限り閉じられなくなる回帰を防ぐ。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();
        assert!(css.contains(".docs-sidebar-toggle:checked ~ nav.sidebar"));
        assert!(!css.contains(":focus-within"));
    }

    #[test]
    fn stylesheet_hides_sidebar_toggle_checkbox_at_desktop_breakpoint() {
        // Bugbot 指摘（PR #916）是正の回帰テスト: `min-width: 768px`（折りたたみ
        // を行わない帯域）では可視ラベルだけでなくチェックボックス本体も
        // `display: none` にし、キーボードユーザーが無意味なコントロールへ
        // 到達しないことを固定する。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();
        let after_768 = css
            .split("@media (min-width: 768px)")
            .nth(1)
            .expect("min-width: 768px block should exist");
        // `min-width: 1200px` ブロックの手前までが `min-width: 768px` ブロックの
        // 本体（ネストした `{ }` を厳密にパースせず、次の `@media` 開始位置を
        // 終端とみなす。CSS の性質上メディアクエリはトップレベルにしか
        // 出現しないため十分な近似）。
        let block_768 = match after_768.find("@media (min-width: 1200px)") {
            Some(idx) => &after_768[..idx],
            None => after_768,
        };
        assert!(block_768.contains(".docs-sidebar-toggle {"));
        assert!(block_768.contains(".docs-sidebar-toggle-label {"));
    }

    #[test]
    fn stylesheet_collapses_toc_grid_track_when_no_toc_modifier_present() {
        // Bugbot 指摘（PR #916）是正の回帰テスト: 見出しの無いページ
        // （`layout.rs` が付与する `docs-container--no-toc` 修飾 class）では
        // `min-width: 1200px` の 3 カラム grid が右目次列のトラックを
        // 収縮させ、空カラムが残ったまま中央カラムが狭くならないことを固定する。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();
        assert!(css.contains(".docs-container.docs-container--no-toc"));
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

    #[test]
    fn stylesheet_contains_nav_list_component_selectors() {
        // イシュー #910 の乖離検知テスト: `crate::nav::sidebar()` の実出力
        // （headless `nav_list` markup、`data-scope="nav-list"
        // data-part="heading|list|item|link"`）に対応する
        // `fandhe_frontend_pre_styled_ui::nav_list::stylesheet()` のセレクタが
        // 生成 CSS に実在することを固定する（`admonition_markdown_output_classes_are_covered_by_generated_admonition_css`
        // / `docs_page_skip_nav_parts_are_covered_by_generated_skip_nav_css`
        // と同型の「実出力 ⇔ 生成 CSS」検証）。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();
        for selector in [
            r#"[data-scope="nav-list"][data-part="heading"]"#,
            r#"[data-scope="nav-list"][data-part="list"]"#,
            r#"[data-scope="nav-list"][data-part="link"]"#,
            r#"[data-scope="nav-list"][data-part="link"][aria-current="page"]"#,
        ] {
            assert!(css.contains(selector), "missing selector: {selector}");
        }
    }

    #[test]
    fn stylesheet_never_takes_up_link_overlay_stylesheet() {
        // イシュー #910 の設計判断（§2.2）の回帰ガード: `link_overlay::
        // stylesheet()` は `overlay` に `position: absolute; inset: 0;` を
        // 登録する。前後ページャの `overlay` はカードの唯一の子要素のため
        // absolute 化するとカードの高さが 0 に潰れる（`crate::nav::prev_next_nav`
        // rustdoc 参照）。将来 `link_overlay::stylesheet()` を誤って取り込む
        // 回帰を fail-closed で検知する。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();
        assert!(!css.contains(r#"[data-scope="link-overlay"]"#));
    }

    #[test]
    fn stylesheet_defines_docs_accent_bg_token_in_light_dark_and_theme_attr_blocks() {
        // 受け入れ条件 3（現在ページ・ホバーのアクセント配色が light/dark
        // 双方で機能する）の機械検証: `--fandhe-color-docs-accent-bg` が
        // 既定（light）・`@media (prefers-color-scheme: dark)`・
        // `:root[data-theme="dark"]` の 3 箇所すべてに定義されることを固定する。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();
        assert_eq!(
            css.matches("--fandhe-color-docs-accent-bg:").count(),
            3,
            "docs-accent-bg should be defined in default + dark media query + data-theme=dark blocks"
        );
    }
}
