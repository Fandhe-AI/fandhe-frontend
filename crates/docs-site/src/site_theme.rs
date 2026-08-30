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
//!     div.docs-header-actions     … 右側のアクション群（イシュー #951、
//!       header_nav の有無に関わらず常に出力）
//!       div.docs-search            … 検索ブロック（既定 `hidden`、イシュー #958）
//!         label.docs-search-label（`for="docs-search-input"`、視覚上のみ
//!           clip で隠す。fandhe-backend とのデザイン統一のため追加）
//!         input.docs-search-input（`id="docs-search-input"`） … `data-search-index`
//!           でインデックス JSON を参照
//!         ul.docs-search-results（`#docs-search-results`、既定 `hidden`）
//!           li.docs-search-result（JS 実行時生成。`role="option"`）
//!             a（クリック可能なヒットターゲット本体。行のクロム
//!               〔`padding`・`color`・`text-decoration`〕は
//!               `.docs-search-result > a` に付与し、`li` 自身は
//!               `[aria-selected="true"]` 時の背景色のみを持つ）
//!               span.docs-search-result-title
//!               span.docs-search-result-section（見出し一致時のみ）
//!           li.docs-search-empty（JS 実行時生成。0 件 or fetch 失敗時）
//!       a.docs-github-link        … GitHub リポジトリへの外部リンク
//!       button.docs-theme-toggle  … テーマトグル（既定 `hidden`。可視化・
//!         イベント配線は `assets/site.js`（`crate::script`）のみが行う）
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
//!           details.docs-nav-group（`[[section.group]]` ごと、イシュー #940）
//!             summary.docs-nav-group-summary … カテゴリ見出し（プレーンテキスト）
//!             ul.docs-nav-group-list         … グループ配下ページ（nav_list list 再利用）
//!       main.docs-main            … 中央コンテンツカラムのラッパー
//!         nav.docs-toc-inline（任意・見出しが存在するページのみ、イシュー #1080）
//!           … `< 1200px`（右目次カラムが非表示になる帯域）の代替目次。
//!             第 1 子、SkipNav ターゲットより前
//!           details（既定は閉）
//!             summary.docs-toc-inline-summary … "On this page"（`nav.docs-toc` と文言共有）
//!             ul
//!               li.docs-toc-level-2 > a  … href は右目次と同一の `#<id>`
//!               li.docs-toc-level-3 > a
//!         article.docs-content    … Markdown レンダラの出力 + `nav.prev-next`
//!           - `pre > code.language-*` … フェンス付きコードブロック
//!       aside.docs-toc-aside（任意・見出しが存在するページのみ）
//!         … 右目次カラムのラッパー
//!         nav.docs-toc            … ページ内目次（`aria-labelledby` で下記見出しに紐付く）
//!           h2.docs-toc-title     … "On this page" 見出し（イシュー #950）
//!           ul
//!             li.docs-toc-level-2 > a[aria-current="location"]?  … 現在地は JS のみが付与
//!             li.docs-toc-level-3 > a
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
/// （現在ページリンクのアクセント背景・構造寸法 6 種）を追加する。
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
    // アクセント淡色）。イシュー #1422 で theme 既定パレットへ
    // `--fandhe-color-accent-subtle` が正式追加されたが、本トークンは
    // それ以前から docs サイト固有の値として運用されており、値・トークン名
    // ともに変更せず docs 固有トークンのまま維持する（無用な見た目変更を
    // 避けるため）。
    theme.push_color("docs-accent-bg", "#ebf4ff", "#1c2740")?;

    // テーブル横スクロールの端フェード影（イシュー #1079）。`.docs-content
    // table` の背景に「cover 用グラデーション（`--fandhe-color-bg` で影を
    // 覆う）」と「影本体グラデーション（本トークン）」を重ねる 4 レイヤ構成
    // を組む。`th` の不透明背景（下記 `.docs-content th,td` 規則）がテーブル
    // 自身の背景レイヤより手前に描画されるため、ヘッダー行帯には構造的に
    // フェードが乗らない（ラッパー要素なしでは回避不能。詳細は下記テーブル
    // 規則直前のコメント §1〜3）。影自体の色はライト/ダークで別値とし、
    // 両テーマで可視性を確保する。
    theme.push_color(
        "docs-scroll-shadow",
        "rgba(15,23,42,0.20)",
        "rgba(0,0,0,0.55)",
    )?;

    // 3 カラム grid の外枠幅（イシュー #949）。値は fandhe-backend の docs
    // サイトとデザインを統一するため、fandhe-backend `site/assets/site.css`
    // （`.docs-container { max-width: 88rem; }`）の値へ合わせた（旧 84rem）。
    // ヘッダー詳細統一（イシュー #1110 の続き）で `.docs-header-inner` は
    // backend と同じ全幅バー（max-width なし・`padding: 0 1.5rem`）へ移行
    // したため、本トークンを参照するのは `.docs-container` のみとなった
    // （#949 の「ヘッダー左端とサイドバー左端を同一計測枠で揃える」設計は
    // backend 統一を優先して廃止）。
    theme.push_space("docs-container-width", "88rem")?;
    // カラム内側の左右余白（イシュー #949）。`.docs-sidebar` /
    // `.docs-toc-aside` の左右 padding が共有する単一のドリフト源。
    // `.docs-header-inner` はヘッダー詳細統一（イシュー #1110 の続き）で
    // fandhe-backend の `.docs-header { padding: 0 1.5rem; }`（全幅バーの
    // viewport 基準 padding）と同値へ移行したため本トークンを参照しない。
    theme.push_space("docs-gutter", "1rem")?;

    // 構造寸法（3 カラム grid の左ナビ幅・本文最大幅・ヘッダー高さ・
    // 右目次カラム幅）。
    //
    // 値は fandhe-backend の docs サイトとデザインを統一するため、
    // fandhe-backend `site/assets/site.css` の `--docs-sidebar-width` /
    // `--docs-toc-width` / `--docs-max-content-width` へ合わせた（旧
    // 17rem/15rem/48rem）。旧値はイシュー #949 の幅予算計算式
    // （84 − 17 − 15 − 2×2 = 48）から導出した内部整合値だったが、
    // backend の値（88/16/14/46）はこの式に従わない独立した値であり、
    // frontend 側もこの式を廃して backend の値をそのまま採用する
    // （88 − 16 − 14 − 2×2 = 54 ≠ 46 だが、`docs-max-content-width` は
    // grid トラック幅の自動導出ではなく `.docs-content` の `max-width` を
    // 独立に定める値であるため不整合ではない。中央カラムのトラック幅
    // （88 − 16 − 14 = 58rem、`.docs-main` の左右 padding 2rem×2 を引くと
    // 実効 54rem）から `.docs-content` の `max-width: 46rem` が
    // `margin: 0 auto` で中央寄せされる — backend も同一の計算構造
    // （中央トラック 58rem − padding 4rem = 実効 54rem に対し
    // `.docs-content` を 46rem で中央寄せ）であり、レイアウト上は等価な
    // 挙動になる。
    theme.push_space("docs-sidebar-width", "16rem")?;
    theme.push_space("docs-max-content-width", "46rem")?;
    theme.push_space("docs-header-height", "3.25rem")?;
    // 右目次カラム（`aside.docs-toc-aside`）の列幅。3 カラム表示になる
    // `min-width: 1200px` 以上でのみ参照される（イシュー #907）。sticky 追従・
    // 視覚スタイルの仕上げは #909 スコープのため、ここでは列幅のみ定義する。
    theme.push_space("docs-toc-width", "14rem")?;

    // コードブロックのシンタックスハイライトトークン（イシュー #1078）。
    // `crate::highlight::TokenKind::class` が返す `token-*` クラスへ
    // `crate::highlight_css` の CSS セレクタが対応させる。コードブロック背景
    // `var(--fandhe-color-bg-muted)`（STRUCTURAL_CSS の `.docs-content pre`
    // 参照）に対して WCAG AA（4.5:1 目安）を満たすようライト/ダーク双方で
    // 選定する。コメントのみ彩度を落として視覚的に後景化する。
    theme.push_color("docs-code-keyword", "#8250df", "#d2a8ff")?;
    theme.push_color("docs-code-string", "#0a7d3f", "#7ee787")?;
    theme.push_color("docs-code-comment", "#6e7781", "#8b949e")?;
    theme.push_color("docs-code-number", "#0550ae", "#79c0ff")?;
    theme.push_color("docs-code-tag", "#b35900", "#ffa657")?;
    theme.push_color("docs-code-attr", "#953800", "#ffab70")?;

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
  padding: 0;\n\
  border-bottom: 1px solid var(--fandhe-color-border);\n\
  background: var(--fandhe-color-bg);\n\
}\n\
\n\
/*\n\
 * ヘッダー内側の枠。fandhe-backend の docs サイトとヘッダーの見た目を\n\
 * 統一するため（イシュー #1110 の続き）、backend `site/assets/site.css` の\n\
 * `.docs-header { padding: 0 1.5rem; }` と同じ全幅バー（max-width なし・\n\
 * viewport 基準 padding）にする。イシュー #949 の「ヘッダー左端と\n\
 * サイドバー左端を同一計測枠（`--fandhe-space-docs-container-width` +\n\
 * gutter）で揃える」設計は backend 統一を優先して廃止した。\n\
 */\n\
.docs-header-inner {\n\
  display: flex;\n\
  align-items: center;\n\
  width: 100%;\n\
  height: 100%;\n\
  min-width: 0;\n\
  padding: 0 1.5rem;\n\
}\n\
\n\
/*\n\
 * ブランド。fandhe-backend `.docs-header a.docs-brand` と同値（イシュー\n\
 * #1110 の続き。backend は brand に padding を持たない）。旧 #949 の\n\
 * サイドバー左端揃え用 padding（0.32rem 0.5rem 0.32rem calc(0.5rem + 2px)）\n\
 * は全幅バー化に伴い廃止した。\n\
 */\n\
.docs-brand {\n\
  font-weight: 600;\n\
  font-size: 0.95rem;\n\
  letter-spacing: -0.01em;\n\
  color: var(--fandhe-color-fg);\n\
  text-decoration: none;\n\
}\n\
\n\
/*\n\
 * ---- ヘッダーナビ（セクション別ドロップダウン、イシュー #908） ----\n\
 *\n\
 * 基底（768px 未満）では非表示（`min-width: 768px` の @media ブロックで表示に\n\
 * 切り替える）。モバイルは既存のサイドバー折りたたみトグルがナビ手段を\n\
 * 提供する（`.docs-sidebar-toggle-label` 参照）。ドロップダウンの開閉は JS を\n\
 * 使わず `:hover`/`:focus-within` のみで行う（`crate::nav::header_nav` の\n\
 * rustdoc「イシュータイトルとの差分」参照）。\n\
 */\n\
.docs-header-nav {\n\
  display: none;\n\
}\n\
\n\
.docs-header-menu {\n\
  display: flex;\n\
  list-style: none;\n\
  margin: 0;\n\
  padding: 0;\n\
  gap: 0.25rem;\n\
}\n\
\n\
.docs-header-group {\n\
  position: relative;\n\
}\n\
\n\
/*\n\
 * トリガーは a 要素（href 属性つき、セクショントップページへの遷移\n\
 * リンク、イシュー #1012）。`crate::nav::header_nav` が\n\
 * `data-scope=\"nav-list\"` を持たない要素として直接組み立てるため、\n\
 * `nav_list::stylesheet()` の `[data-scope=\"nav-list\"][data-part=\"link\"]`\n\
 * （詳細度 0,2,0）は適用されない。よってこの class セレクタ（詳細度 0,1,0）\n\
 * だけで確定させてよく、ドロップダウン内リンク（`.docs-header-dropdown a`）\n\
 * が必要とする `.docs-header nav.docs-header-nav` プレフィックスは不要\n\
 * （#908 Bugbot 指摘の再発防止コメント）。旧 button 要素用の\n\
 * `background: none; border: none; cursor: pointer;` リセットは a 要素では\n\
 * 不要（`cursor: pointer` は `a[href]` の既定）なので削除し、a 要素の既定\n\
 * 下線を抑止する `text-decoration: none` を追加する。\n\
 */\n\
.docs-header-trigger {\n\
  display: block;\n\
  /* padding は fandhe-backend `.docs-header-trigger` と同値（イシュー\n\
   * #1110 の続き、旧 0.4rem 0.6rem）。 */\n\
  padding: 0.3rem 0.65rem;\n\
  border-radius: 0.4rem;\n\
  font: inherit;\n\
  font-size: 0.85rem;\n\
  font-weight: 500;\n\
  color: var(--fandhe-color-fg);\n\
  text-decoration: none;\n\
}\n\
\n\
.docs-header-trigger:hover {\n\
  background: var(--fandhe-color-bg-subtle);\n\
}\n\
\n\
.docs-header-trigger:focus-visible {\n\
  outline: 2px solid var(--fandhe-color-accent);\n\
  outline-offset: 2px;\n\
}\n\
\n\
/*\n\
 * 現在セクションの表現（イシュー #1012）。ドロップダウン内リンクの\n\
 * `a[aria-current=\"page\"]`（ページ完全一致）とは軸が異なる\n\
 * `[aria-current=\"true\"]` を使うため、同一トークン\n\
 * `--fandhe-color-docs-accent-bg` を再利用しても意味は衝突しない。\n\
 */\n\
.docs-header-trigger[aria-current=\"true\"] {\n\
  color: var(--fandhe-color-accent);\n\
  background: var(--fandhe-color-docs-accent-bg);\n\
}\n\
\n\
/*\n\
 * ---- ヘッダーアクション（GitHub リンク・テーマトグル、イシュー #951） ----\n\
 *\n\
 * `docs-header-nav` の有無に関わらず常に出力される（`crate::layout` 参照）。\n\
 * fandhe-backend `.docs-header-actions` と同値（イシュー #1110 の続き、\n\
 * 旧 gap 0.5rem）: ヘッダーナビは brand 直後に左寄せ（`min-width: 768px`\n\
 * ブロックの `.docs-header-nav { margin-left: 1.25rem; }` 参照）とし、\n\
 * 右寄せは常に本要素の `margin-left: auto` のみが担う（旧設計の\n\
 * `.docs-header-nav + .docs-header-actions` による auto 打ち消し override\n\
 * は不要になり削除した。`header_nav: None` 構成でも同一規則で右端配置\n\
 * される）。\n\
 */\n\
.docs-header-actions {\n\
  display: flex;\n\
  align-items: center;\n\
  gap: 0.9rem;\n\
  margin-left: auto;\n\
}\n\
\n\
/*\n\
 * GitHub リンクは枠なしのプレーンテキストリンク（fandhe-backend\n\
 * `.docs-github-link` と同値、イシュー #1110 の続き。旧 padding/\n\
 * border-radius/hover 背景は廃止）。色は既存 fandhe トークンのまま\n\
 * （配色非統一は既決事項、コミット de83bc2）。\n\
 */\n\
.docs-github-link {\n\
  font-size: 0.85rem;\n\
  font-weight: 500;\n\
  color: var(--fandhe-color-fg-muted);\n\
  text-decoration: none;\n\
}\n\
\n\
.docs-github-link:hover {\n\
  color: var(--fandhe-color-fg);\n\
}\n\
\n\
.docs-github-link:focus-visible {\n\
  outline: 2px solid var(--fandhe-color-accent);\n\
  outline-offset: 2px;\n\
}\n\
\n\
/*\n\
 * テーマトグルはボタン風の枠付き（fandhe-backend `.docs-theme-toggle` と\n\
 * 同値、イシュー #1110 の続き。旧: 枠なし・hover 背景のみ）。色は既存\n\
 * fandhe トークンのまま（配色非統一は既決事項、コミット de83bc2）。\n\
 */\n\
.docs-theme-toggle {\n\
  display: block;\n\
  font: inherit;\n\
  font-size: 0.85rem;\n\
  font-weight: 500;\n\
  color: var(--fandhe-color-fg-muted);\n\
  background: var(--fandhe-color-bg-subtle);\n\
  border: 1px solid var(--fandhe-color-border);\n\
  border-radius: 0.4rem;\n\
  padding: 0.3rem 0.65rem;\n\
  cursor: pointer;\n\
}\n\
\n\
.docs-theme-toggle:hover {\n\
  color: var(--fandhe-color-fg);\n\
  border-color: var(--fandhe-color-accent);\n\
}\n\
\n\
.docs-theme-toggle:focus-visible {\n\
  outline: 2px solid var(--fandhe-color-accent);\n\
  outline-offset: 2px;\n\
}\n\
\n\
/*\n\
 * JS 無効時・`assets/site.js` の読み込み失敗時の退避経路（イシュー #951\n\
 * 受入条件）。`crate::layout` は既定で `hidden` 属性を付与し、`site.js` は\n\
 * イベント配線が完了した後にのみこの属性を除去する（`crate::script`\n\
 * モジュール doc 手順 5 参照）。詳細度 0,2,0 で `.docs-theme-toggle`\n\
 * （0,1,0）に順序非依存で勝つ。\n\
 */\n\
.docs-theme-toggle[hidden] {\n\
  display: none;\n\
}\n\
\n\
/*\n\
 * ---- 検索 UI（素の JS、イシュー #958） ----\n\
 *\n\
 * `.docs-header-actions` の第 1 子（`crate::layout` 参照）。既定 `hidden` の\n\
 * 退避経路は `.docs-theme-toggle[hidden]` と同型（配線完了後にのみ\n\
 * `crate::script::SITE_JS` が `hidden` を除去する）。`display: none` にせず\n\
 * `min-width: 0` + `max-width` で縮小に任せるため、狭幅帯域でも要素自体は\n\
 * 消さない（`hidden` 属性のみが非表示の唯一の情報源）。\n\
 */\n\
.docs-search {\n\
  position: relative;\n\
  display: flex;\n\
  align-items: center;\n\
  min-width: 0;\n\
}\n\
\n\
.docs-search[hidden] {\n\
  display: none;\n\
}\n\
\n\
/*\n\
 * 検索入力のラベル（イシュー #958 の検索 UI に付随する視覚実装。\n\
 * fandhe-backend の docs サイトとデザインを統一するため追加した。正は\n\
 * fandhe-backend `site/assets/site.css` の `.docs-search-label` clip\n\
 * 規則）。`for`/`id`\n\
 * 対応（`crate::layout` の `input#docs-search-input`）は保つが視覚上のみ\n\
 * clip 手法で隠す（`.skip-nav` と同型、a11y のためタブ順序・スクリーン\n\
 * リーダー読み上げ対象からは除外しない）。\n\
 */\n\
.docs-search-label {\n\
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
 * 検索入力は fandhe-backend `.docs-search-input` の寸法（width 12rem・\n\
 * padding 0.3rem 0.65rem・radius 0.4rem・subtle 背景）へ統一（イシュー\n\
 * #1110 の続き。旧: width 100% + max-width 9rem・padding 0.35rem 0.6rem・\n\
 * radius var(--fandhe-radius-sm)・bg 背景）。`min-width: 0` は狭幅帯域で\n\
 * flex 縮小を許すために残す。font-size は既存トークン（0.85rem 相当）の\n\
 * まま。\n\
 */\n\
.docs-search-input {\n\
  width: 12rem;\n\
  min-width: 0;\n\
  font: inherit;\n\
  font-size: var(--fandhe-font-font-size-sm);\n\
  padding: 0.3rem 0.65rem;\n\
  border: 1px solid var(--fandhe-color-border);\n\
  border-radius: 0.4rem;\n\
  background: var(--fandhe-color-bg-subtle);\n\
  color: var(--fandhe-color-fg);\n\
}\n\
\n\
.docs-search-input:focus-visible {\n\
  outline: 2px solid var(--fandhe-color-accent);\n\
  outline-offset: 2px;\n\
}\n\
\n\
/*\n\
 * 検索結果パネル。`.docs-header`（z-index 10）・`.docs-header-dropdown`\n\
 * （z-index 20）より上に出すため z-index 30 を使う（`.docs-header`/\n\
 * `.docs-header-inner` はいずれも `overflow` を宣言していないため絶対配置\n\
 * パネルはクリップされない）。\n\
 */\n\
.docs-search-results {\n\
  position: absolute;\n\
  top: 100%;\n\
  right: 0;\n\
  z-index: 30;\n\
  margin: 0.25rem 0 0;\n\
  padding: 0.25rem;\n\
  list-style: none;\n\
  min-width: 18rem;\n\
  max-height: 60vh;\n\
  overflow-y: auto;\n\
  background: var(--fandhe-color-bg);\n\
  border: 1px solid var(--fandhe-color-border);\n\
  border-radius: var(--fandhe-radius-sm);\n\
}\n\
\n\
.docs-search-results[hidden] {\n\
  display: none;\n\
}\n\
\n\
/* 以下 4 class（`.docs-search-result*`・`.docs-search-empty`）は\n\
 * `crate::script::SITE_JS` が実行時に `document.createElement` で生成する\n\
 * ため SSG 出力（`crates/docs-site/tests/site_css_contract.rs` の層 1\n\
 * `STRUCTURE_CLASS_CONTRACT`）には出現しない。`SEARCH_JS_ONLY_CLASSES`\n\
 * （同ファイル）が (b) セレクタ存在・(a′) SITE_JS へのリテラル出現・\n\
 * (c′) HTML 非出現の 3 方向で別枠検証する。 */\n\
.docs-search-result {\n\
  display: block;\n\
  border-radius: var(--fandhe-radius-sm);\n\
}\n\
\n\
.docs-search-result > a {\n\
  display: block;\n\
  padding: 0.4rem 0.6rem;\n\
  text-decoration: none;\n\
  color: var(--fandhe-color-fg);\n\
}\n\
\n\
.docs-search-result[aria-selected=\"true\"] {\n\
  background: var(--fandhe-color-bg-subtle);\n\
}\n\
\n\
.docs-search-result-title {\n\
  display: block;\n\
  font-size: var(--fandhe-font-font-size-sm);\n\
  font-weight: 500;\n\
}\n\
\n\
.docs-search-result-section {\n\
  display: block;\n\
  color: var(--fandhe-color-fg-muted);\n\
  font-size: var(--fandhe-font-font-size-sm);\n\
}\n\
\n\
.docs-search-empty {\n\
  display: block;\n\
  padding: 0.5rem 0.6rem;\n\
  color: var(--fandhe-color-fg-muted);\n\
}\n\
\n\
/*\n\
 * `.docs-header-dropdown` の開閉状態の唯一の情報源は `:hover`/`:focus-within`\n\
 * （マウス操作・キーボード操作の双方をカバーする。JS を使わないため\n\
 * `aria-expanded` 等の動的属性で開閉状態を表現しない、`header_nav` rustdoc\n\
 * 参照）。\n\
 *\n\
 * `header_nav` は headless `nav_list` anatomy（`nav::sidebar` と同じ\n\
 * `data-scope=\"nav-list\" data-part=\"list|link\"`）を再利用しているため、\n\
 * `nav_list::stylesheet()` の `[data-scope=\"nav-list\"][data-part=\"...\"]`\n\
 * ルール（詳細度 0,2,0・`[data-part][aria-current=\"page\"]` は 0,3,0）が\n\
 * 素の class セレクタ（0,1,0/0,1,1）に競り勝ち、dropdown の padding・\n\
 * リンク色・カレントページの font-weight が適用されない不具合があった\n\
 * （Bugbot 指摘、イシュー #908 PR #919 レビュー）。サイドバーが\n\
 * `.docs-sidebar nav.sidebar ...`（詳細度 0,2,1 以上）で対策済みなのと\n\
 * 同型で、以下 padding/color/font-weight を上書きするセレクタは\n\
 * `.docs-header nav.docs-header-nav .docs-header-dropdown ...`\n\
 * （詳細度 0,3,1 以上、`nav_list` 側の最大 0,3,0 を常に上回る）で\n\
 * 固定する。開閉トグル（`display`）・右端アンカー（`left`/`right`）は\n\
 * `nav_list::stylesheet()` 側に競合するプロパティが存在しないため\n\
 * プレフィックス不要のまま据え置く。\n\
 */\n\
.docs-header nav.docs-header-nav .docs-header-dropdown {\n\
  position: absolute;\n\
  top: 100%;\n\
  left: 0;\n\
  display: none;\n\
  z-index: 20;\n\
  /* 寸法は fandhe-backend `.docs-header-dropdown` と同値（イシュー #1110\n\
   * の続き、旧 min-width 12rem / padding 0.35rem）。 */\n\
  min-width: 14rem;\n\
  margin: 0;\n\
  padding: 0.4rem;\n\
  list-style: none;\n\
  border: 1px solid var(--fandhe-color-border);\n\
  border-radius: 0.5rem;\n\
  background: var(--fandhe-color-bg);\n\
  /* 影はライト/ダークで光量が異なるため `--fandhe-shadow-*`（イシュー #606\n\
   * の DualModeToken、`Theme::default` の `DEFAULT_SHADOWS`）を参照する。\n\
   * 生 rgba を書くとダーク背景で輪郭が視認できない（イシュー #912 是正）。 */\n\
  box-shadow: var(--fandhe-shadow-md);\n\
}\n\
\n\
.docs-header nav.docs-header-nav .docs-header-group:hover > .docs-header-dropdown,\n\
.docs-header nav.docs-header-nav .docs-header-group:focus-within > .docs-header-dropdown {\n\
  display: block;\n\
}\n\
\n\
/*\n\
 * 最後（最右）のグループのみ右端アンカー（`right: 0`）に切り替える\n\
 * （Bugbot 指摘、イシュー #908 PR #919 レビュー由来）。ヘッダー統一\n\
 * （イシュー #1110 の続き）でナビは brand 直後の左寄せになり右端に接する\n\
 * 可能性は下がったが、狭幅（768px 直上）でセクション数が多い場合は依然\n\
 * 右はみ出しが起こりうるため、防御規則として維持する。\n\
 */\n\
.docs-header nav.docs-header-nav .docs-header-group:last-child > .docs-header-dropdown {\n\
  left: auto;\n\
  right: 0;\n\
}\n\
\n\
.docs-header nav.docs-header-nav .docs-header-dropdown a {\n\
  display: block;\n\
  padding: 0.32rem 0.5rem;\n\
  border-radius: 0.4rem;\n\
  color: var(--fandhe-color-fg-muted);\n\
  text-decoration: none;\n\
  font-size: 0.85rem;\n\
}\n\
\n\
.docs-header nav.docs-header-nav .docs-header-dropdown a:hover {\n\
  color: var(--fandhe-color-fg);\n\
  background: var(--fandhe-color-bg-subtle);\n\
}\n\
\n\
.docs-header nav.docs-header-nav .docs-header-dropdown a[aria-current=\"page\"] {\n\
  background: var(--fandhe-color-docs-accent-bg);\n\
  color: var(--fandhe-color-accent);\n\
  font-weight: 600;\n\
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
  max-width: var(--fandhe-space-docs-container-width);\n\
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
  padding: var(--fandhe-space-docs-gutter);\n\
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
/*\n\
 * ---- サイドバー内カテゴリ階層（details/summary、イシュー #940） ----\n\
 *\n\
 * `crate::nav::sidebar()` のグループ描画（`[[section.group]]`）が出力する\n\
 * `details.docs-nav-group` / `.docs-nav-group-summary` / `.docs-nav-group-list`\n\
 * の骨格 CSS。`.docs-nav-group-list` は nav_list `list()` を再利用しており\n\
 * `[data-scope=\"nav-list\"][data-part=\"list\"]`（詳細度 0,2,0）と競合するため、\n\
 * 直下ページ `ul`（上記）と同型に `.docs-sidebar nav.sidebar ...`\n\
 * （詳細度 0,2,1 以上）で確実に上書きする（#908/PR #919 で実際に踏んだ\n\
 * 詳細度の罠の再発防止）。\n\
 */\n\
.docs-sidebar nav.sidebar details.docs-nav-group {\n\
  margin: 0.2rem 0;\n\
  border-top: 1px solid var(--fandhe-color-border);\n\
  padding-top: 0.2rem;\n\
}\n\
\n\
.docs-sidebar nav.sidebar .docs-nav-group-summary {\n\
  cursor: pointer;\n\
  list-style: none;\n\
  padding: 0.32rem 0.5rem;\n\
  border-radius: 0.4rem;\n\
  font-weight: var(--fandhe-font-font-weight-semibold);\n\
  font-size: 0.72rem;\n\
  letter-spacing: 0.06em;\n\
  text-transform: uppercase;\n\
  color: var(--fandhe-color-fg-muted);\n\
}\n\
\n\
/* Chrome/Safari のデフォルト三角マーカーを消し、上記 uppercase 見出し体裁と\n\
 * 揃える（`::marker`（Firefox 系）と `::-webkit-details-marker`\n\
 * （Chromium/Safari 系）の両方を明示的に消す必要がある）。 */\n\
.docs-sidebar nav.sidebar .docs-nav-group-summary::marker,\n\
.docs-sidebar nav.sidebar .docs-nav-group-summary::-webkit-details-marker {\n\
  display: none;\n\
}\n\
\n\
.docs-sidebar nav.sidebar .docs-nav-group-summary:hover {\n\
  background: var(--fandhe-color-bg-subtle);\n\
}\n\
\n\
.docs-sidebar nav.sidebar .docs-nav-group-summary:focus-visible {\n\
  outline: 2px solid var(--fandhe-color-accent);\n\
  outline-offset: 2px;\n\
}\n\
\n\
.docs-sidebar nav.sidebar .docs-nav-group-list {\n\
  margin: 0.1rem 0 0.2rem;\n\
  padding: 0;\n\
  padding-left: 0.6rem;\n\
  display: flex;\n\
  flex-direction: column;\n\
  gap: 0.05rem;\n\
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
/* ---- 右目次カラム内のページ内目次（イシュー #909） ---- */\n\
\n\
/*\n\
 * #907 まではカード風（本文内配置）だったが、#909 で `aside.docs-toc-aside`\n\
 * （右カラム）専用の視覚へ書き換えた。カード装飾（border/background/\n\
 * max-width）を除去し、細身のインデックス表現にする。h2/h3 の階層表現\n\
 * （`.docs-toc-level-2` / `-3` のインデント）は既存契約のまま不変。\n\
 */\n\
.docs-toc {\n\
  font-size: 0.8rem;\n\
  line-height: 1.5;\n\
}\n\
\n\
/*\n\
 * 右目次の見出し（イシュー #950）。サイドバーのカテゴリ見出し\n\
 * （`.docs-sidebar nav.sidebar h2`、上記）と同じ uppercase の小見出し\n\
 * 体裁に揃える。`nav.docs-toc` の `aria-labelledby` が本要素の `id`\n\
 * （`crate::layout::TOC_HEADING_ID`）を参照する。\n\
 */\n\
.docs-toc-title {\n\
  margin: 0 0 0.5rem;\n\
  font-size: 0.72rem;\n\
  font-weight: var(--fandhe-font-font-weight-semibold);\n\
  letter-spacing: 0.06em;\n\
  text-transform: uppercase;\n\
  color: var(--fandhe-color-fg-muted);\n\
}\n\
\n\
.docs-toc ul {\n\
  list-style: none;\n\
  margin: 0;\n\
  padding: 0;\n\
  display: flex;\n\
  flex-direction: column;\n\
  gap: 0.2rem;\n\
}\n\
\n\
.docs-toc a {\n\
  display: block;\n\
  color: var(--fandhe-color-fg-muted);\n\
  text-decoration: none;\n\
  /* 長い `h3` タイトル（例: API ページの `placement API` 節見出し）が\n\
   * 14rem の右カラム内で単語境界を越えて折り返せるようにする。テキスト\n\
   * の切り詰め（ellipsis/line-clamp）は行わない（情報欠落を招くため）。\n\
   */\n\
  overflow-wrap: anywhere;\n\
  padding: 0.12rem 0 0.12rem 0.5rem;\n\
  border-left: 2px solid transparent;\n\
}\n\
\n\
.docs-toc a:hover {\n\
  color: var(--fandhe-color-accent);\n\
}\n\
\n\
/*\n\
 * 現在地ハイライト（イシュー #950）。`aria-current=\"location\"` は\n\
 * `crate::script::SITE_JS` の IntersectionObserver がスクロール位置に\n\
 * 応じて実行時にのみ付与する（SSG が出力する静的 markup には含まれない）。\n\
 * サイドバーの `aria-current=\"page\"`（現在ページ）と値を分けることで\n\
 * 意味の衝突を避ける。JS 無効・`site.js` 読み込み失敗時はこの規則が一切\n\
 * 一致せず、通常のリンク表示のまま機能する（progressive enhancement）。\n\
 */\n\
.docs-toc a[aria-current=\"location\"] {\n\
  color: var(--fandhe-color-accent);\n\
  font-weight: var(--fandhe-font-font-weight-medium);\n\
  border-left-color: var(--fandhe-color-accent);\n\
}\n\
\n\
.docs-toc-level-2 {\n\
  padding-left: 0;\n\
}\n\
\n\
.docs-toc-level-3 {\n\
  /* 1rem から縮小（イシュー #950）。長い `h3` タイトルの折り返し時に\n\
   * 実効幅を確保し、過大インデントで読みにくくなるのを防ぐ。 */\n\
  padding-left: 0.5rem;\n\
}\n\
\n\
/* ---- 本文冒頭の折りたたみ目次（`nav.docs-toc-inline`。イシュー #1080） ---- */\n\
\n\
/*\n\
 * `aside.docs-toc-aside`（右目次カラム）が `min-width: 1200px` 未満で\n\
 * `display: none` になる代替の JS 非依存な到達手段。基底帯域では表示し、\n\
 * `min-width: 1200px` 側（下記）で非表示へ切り替えて右目次カラムとの\n\
 * 重複表示を避ける（`crate::layout::toc_inline` rustdoc 参照）。幅・左端は\n\
 * `.docs-content` と揃える。\n\
 */\n\
.docs-toc-inline {\n\
  display: block;\n\
  max-width: var(--fandhe-space-docs-max-content-width);\n\
  margin: 0 auto 1.5rem;\n\
}\n\
\n\
.docs-toc-inline-summary {\n\
  cursor: pointer;\n\
  font-size: 0.8rem;\n\
  font-weight: var(--fandhe-font-font-weight-semibold);\n\
  padding: 0.5rem 0.7rem;\n\
  border: 1px solid var(--fandhe-color-border);\n\
  border-radius: 0.4rem;\n\
  color: var(--fandhe-color-fg-muted);\n\
}\n\
\n\
.docs-toc-inline-summary:hover {\n\
  color: var(--fandhe-color-fg);\n\
  background: var(--fandhe-color-bg-subtle);\n\
}\n\
\n\
.docs-toc-inline-summary:focus-visible {\n\
  outline: 2px solid var(--fandhe-color-accent);\n\
  outline-offset: 2px;\n\
}\n\
\n\
.docs-toc-inline ul {\n\
  list-style: none;\n\
  margin: 0.5rem 0 0;\n\
  padding: 0.3rem 0.7rem 0.3rem 1rem;\n\
  border-left: 1px solid var(--fandhe-color-border);\n\
  display: flex;\n\
  flex-direction: column;\n\
  gap: 0.25rem;\n\
}\n\
\n\
.docs-toc-inline a {\n\
  display: block;\n\
  color: var(--fandhe-color-fg-muted);\n\
  text-decoration: none;\n\
  overflow-wrap: anywhere;\n\
  padding: 0.1rem 0;\n\
}\n\
\n\
.docs-toc-inline a:hover {\n\
  color: var(--fandhe-color-accent);\n\
}\n\
\n\
/* ---- 右目次カラム（`aside.docs-toc-aside`。イシュー #907） ---- */\n\
\n\
/*\n\
 * 基底・768px 帯域（2 カラム）では非表示。狭幅では「右目次 → 左ナビ」の\n\
 * 順に畳む要件どおり、目次カラムのみを先に隠す。1200px 以上で表示に切り替え、\n\
 * sticky 追従を有効化する（`min-width: 1200px` 側、下記。イシュー #909）。\n\
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
    padding: 1.75rem var(--fandhe-space-docs-gutter) 2rem;\n\
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
  /* `.docs-header-nav`（セクション別ドロップダウン、イシュー #908）は\n\
   * この帯域から表示に切り替える。モバイルはサイドバー折りたたみトグルが\n\
   * ナビ手段を提供するため基底では非表示のまま。配置は fandhe-backend\n\
   * `.docs-header-nav { margin-left: 1.25rem; min-width: 0; }` と同値\n\
   * （イシュー #1110 の続き）: brand 直後の左寄せとし、旧設計の\n\
   * `margin-left: auto` 右寄せ + `.docs-header-nav + .docs-header-actions`\n\
   * の auto 打ち消し override（イシュー #951）は廃止した（右寄せは\n\
   * `.docs-header-actions` の `margin-left: auto` のみが担う）。 */\n\
  .docs-header-nav {\n\
    display: flex;\n\
    margin-left: 1.25rem;\n\
    min-width: 0;\n\
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
 *\n\
 * `.docs-toc-aside` は `.docs-sidebar`（`min-width: 768px` ブロック、上記）と\n\
 * 同型の sticky 追従を持つ: `.docs-header` が `position: sticky; top: 0` で\n\
 * 常時可視のため、ヘッダー高さ分オフセットしてヘッダー直下へ張り付け、\n\
 * `max-height` + `overflow-y: auto` で目次が長いページでもカラム内スクロールに\n\
 * 閉じ、ページ全体のレイアウトを崩さない（イシュー #909、受け入れ条件 1・2）。\n\
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
    padding: 1.75rem var(--fandhe-space-docs-gutter) 2rem;\n\
    position: sticky;\n\
    top: var(--fandhe-space-docs-header-height);\n\
    align-self: start;\n\
    max-height: calc(100vh - var(--fandhe-space-docs-header-height));\n\
    overflow-y: auto;\n\
  }\n\
\n\
  /* 右目次カラムが表示に切り替わるのと同じ帯域で、本文冒頭の折りたたみ\n\
   * 目次（狭幅代替、イシュー #1080）を非表示にし重複表示を避ける。 */\n\
  .docs-toc-inline {\n\
    display: none;\n\
  }\n\
}\n\
\n\
/*\n\
 * `.docs-header` が `position: sticky; top: 0` で常時ヘッダー高さ分を占有する\n\
 * ため、右目次カラムのリンク（`.docs-toc a`）や外部からのフラグメント直接リンクで\n\
 * 見出しへジャンプすると素のブラウザ挙動ではヘッダーの下に隠れてしまう。\n\
 * `with_heading_anchors`（id 注入対象は h2/h3）が生成する全アンカーの到達位置を\n\
 * ヘッダー高さ分オフセットし、ページ内リンクが全ページで機能する状態にする\n\
 * （イシュー #909、受け入れ条件 3）。\n\
 */\n\
.docs-content h2,\n\
.docs-content h3 {\n\
  scroll-margin-top: calc(var(--fandhe-space-docs-header-height) + 1rem);\n\
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
    //
    // 横スクロール可能な `overflow-x: auto` 要素そのものに横スクロール
    // アフォーダンスを与える（イシュー #949）。`crate::markdown` の出力に
    // ラッパー要素を挿入できない制約（本関数冒頭の rustdoc §3.6 不変条件）
    // のため、`table` 自身へ「常時可視スクロールバー」+「枠線フレーム」+
    // 「端のフェード影」の 3 種を重畳する（イシュー #1079。#949 が入れた
    // 前 2 種は削らず、フェードは 3 系統目として重ねる）。
    //
    // #949 は「端のフェードは `th` の不透明背景に覆われて読めなくなる」
    // として一度不採用にしたが、以下 3 点で整理する:
    //   1. 置換ではなく重畳: #949 の 2 種は残したまま、フェードを追加する
    //      だけなので #949 の判断（フェード単独では不十分）とは矛盾しない。
    //   2. ヘッダー行帯が覆われないのは構造的に回避不能: 子要素
    //      （`th`/`td`）の背景は親（`table`）の背景より必ず手前に描画される
    //      ため、`th` の不透明 `background`（下記 `.docs-content th,td`
    //      規則）はテーブル自身の背景レイヤを覆う。`thead`/`tbody` は
    //      コンテンツ幅に広がる table-row-group ボックスであり、
    //      `background-attachment: local` を付けても viewport 端ではなく
    //      コンテンツ端に貼り付くため機能しない。`table::before`/`::after`
    //      も通常フロー内の要素で内容と一緒にスクロールし、全高オーバー
    //      レイの代替にならない。よって `th` の背景色（`bg-subtle`）は
    //      弱めない（`color-mix` 等での半透明化は #949 が確定した
    //      ヘッダー識別性を壊すため不採用）。フェードは主に tbody 領域
    //      （テーブル高さの大半）を覆う。
    //   3. ラッパー `div` 案（完全被覆が可能）は不採用: `crate::markdown`
    //      の出力契約変更（§3.6 不変条件の改訂）が必須であり、本イシュー
    //      は CSS のみでの解決を求めているため対象外（PR 本文のスコープ外
    //      節に記録）。
    //
    // 技法（Roman Komarov 方式の水平版、JS 不使用）: 背景に 4 レイヤを
    // 積む。1〜2 層目は cover（`--fandhe-color-bg` と同色、スクロール
    // 位置と一緒に動く `local`）、3〜4 層目は影本体（本トークン、
    // viewport 端に固定される `scroll`）。オーバーフローしていない
    // テーブルでは cover と影が常に重なるため見た目は現状から不変（#949
    // 由来の枠線フレーム・スクロールバーのみが見える）。
    push_typography_rule(
        &mut out,
        ".docs-content table",
        &[
            decl("display", "block"),
            decl("overflow-x", "auto"),
            decl("margin", "0 0 1.05rem"),
            decl("max-width", "100%"),
            decl("font-size", "0.925em"),
            decl("border", "1px solid var(--fandhe-color-border)"),
            decl("border-radius", "0.5rem"),
            // Firefox 向け: 細身の常時可視スクロールバー。
            decl("scrollbar-width", "thin"),
            decl(
                "scrollbar-color",
                "var(--fandhe-color-border) var(--fandhe-color-bg-subtle)",
            ),
            // 端フェード影（イシュー #1079）。longhand で明示し、
            // `background` ショートハンドは使わない（`background-color`
            // をリセットして既存規則と競合するのを避けるため）。
            decl(
                "background-image",
                "linear-gradient(to right, var(--fandhe-color-bg), transparent), linear-gradient(to left, var(--fandhe-color-bg), transparent), linear-gradient(to right, var(--fandhe-color-docs-scroll-shadow), transparent), linear-gradient(to left, var(--fandhe-color-docs-scroll-shadow), transparent)",
            ),
            decl(
                "background-position",
                "left center, right center, left center, right center",
            ),
            decl("background-size", "2rem 100%, 2rem 100%, 0.9rem 100%, 0.9rem 100%"),
            decl("background-repeat", "no-repeat"),
            decl("background-attachment", "local, local, scroll, scroll"),
        ],
    )?;
    // Chromium/Safari（`scrollbar-width`/`scrollbar-color` 非対応）向け:
    // `::-webkit-scrollbar*` 擬似要素でオーバーレイスクロールバーを常時
    // 表示のトラック付きバーへ切り替える。
    push_typography_rule(
        &mut out,
        ".docs-content table::-webkit-scrollbar",
        &[decl("height", "0.5rem")],
    )?;
    push_typography_rule(
        &mut out,
        ".docs-content table::-webkit-scrollbar-track",
        &[
            decl("background", "var(--fandhe-color-bg-subtle)"),
            decl("border-radius", "0.5rem"),
        ],
    )?;
    push_typography_rule(
        &mut out,
        ".docs-content table::-webkit-scrollbar-thumb",
        &[
            decl("background", "var(--fandhe-color-border)"),
            decl("border-radius", "0.5rem"),
        ],
    )?;
    // `table` 自身の `border`（外枠フレーム）と二重線にならないよう、
    // セル側は内部グリッド線（右辺・下辺のみ）だけを持たせる。`table` は
    // `display: block` のままで `border-collapse: collapse` が効かない
    // ため、外周に接する辺（上辺・左辺は常に、右端列の右辺・最終行の
    // 下辺）はセル側で明示的に打ち消し、外枠は `table` の `border` のみが
    // 担う（Bugbot 指摘、イシュー #949 追補）。
    push_typography_rule(
        &mut out,
        ".docs-content th,\n.docs-content td",
        &[
            decl("border-width", "0 1px 1px 0"),
            decl("border-style", "solid"),
            decl("border-color", "var(--fandhe-color-border)"),
            decl("padding", "0.45rem 0.75rem"),
            decl("text-align", "left"),
        ],
    )?;
    push_typography_rule(
        &mut out,
        ".docs-content tr > :last-child",
        &[decl("border-right", "0")],
    )?;
    // `tbody` へスコープを限定する（Bugbot 指摘、イシュー #949 追補）。
    // docs のテーブルは常に `thead` + `tbody` を出力するため、
    // 無スコープの `tr:last-child` は `thead` 内の唯一の `tr`（＝ヘッダー
    // 行）にも一致してしまい、`thead`/`tbody` 境界線（ヘッダー行下部の
    // ボーダー）まで消してしまう。「最終 body 行のみ下辺を打ち消す」
    // という意図を保つため `tbody` 配下に限定する。
    push_typography_rule(
        &mut out,
        ".docs-content tbody tr:last-child > *",
        &[decl("border-bottom", "0")],
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

/// フェンスコードブロックのシンタックスハイライト CSS を組み立てる
/// （イシュー #1078）。[`crate::highlight::TokenKind::ALL`] が定義する
/// `token-*` クラス全種をここでカバーする（`tests/highlight.rs` が
/// enum 駆動でセレクタ網羅性を機械検証する）。
///
/// セレクタは `.docs-content pre code .token-*` とし、`.docs-content pre code`
/// のリセット規則（[`typography_css`]）より後方かつ高詳細度で確実に
/// 上書きする。`token-*` クラスは意図的に `docs-` 接頭辞を付けない
/// （[`STRUCTURE_CLASS_CONTRACT`] は `docs-*` class がフルページフィクスチャに
/// 出現することを要求するが、当該フィクスチャはコードブロックを含まないため
/// `docs-token-*` にすると必ず失敗する。`classes_outside_contract` の (c)
/// 方向判定も `docs-` 接頭辞のみを対象にするため、`token-*` は `language-*`
/// と同じ「別契約管轄」として自然に収まる。詳細はイシュー #1078 の実装計画
/// §2.6 参照）。
///
/// # Errors
///
/// [`push_typography_rule`] が fail-closed でエラーを返した場合に伝播する
/// （本関数内の宣言はすべて allowlist を満たす定数のため通常は到達しない）。
fn highlight_css() -> Result<String, SiteThemeError> {
    let mut out = String::new();

    push_typography_rule(
        &mut out,
        ".docs-content pre code .token-keyword",
        &[
            decl("color", "var(--fandhe-color-docs-code-keyword)"),
            decl("font-weight", "600"),
        ],
    )?;
    push_typography_rule(
        &mut out,
        ".docs-content pre code .token-string",
        &[decl("color", "var(--fandhe-color-docs-code-string)")],
    )?;
    push_typography_rule(
        &mut out,
        ".docs-content pre code .token-comment",
        &[
            decl("color", "var(--fandhe-color-docs-code-comment)"),
            decl("font-style", "italic"),
        ],
    )?;
    push_typography_rule(
        &mut out,
        ".docs-content pre code .token-number",
        &[decl("color", "var(--fandhe-color-docs-code-number)")],
    )?;
    push_typography_rule(
        &mut out,
        ".docs-content pre code .token-tag",
        &[decl("color", "var(--fandhe-color-docs-code-tag)")],
    )?;
    push_typography_rule(
        &mut out,
        ".docs-content pre code .token-attr",
        &[decl("color", "var(--fandhe-color-docs-code-attr)")],
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
    sheet.push_css(&highlight_css()?)?;
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
            ".docs-brand",
            ".docs-header-nav",
            ".docs-header-menu",
            ".docs-header-group",
            ".docs-header-trigger",
            ".docs-header-dropdown",
            ".docs-header-actions",
            ".docs-search",
            ".docs-search-label",
            ".docs-search-input",
            ".docs-search-results",
            ".docs-search-result",
            ".docs-search-result-title",
            ".docs-search-result-section",
            ".docs-search-empty",
            ".docs-github-link",
            ".docs-theme-toggle",
            ".docs-container",
            ".docs-sidebar",
            ".docs-sidebar-toggle",
            ".docs-sidebar-toggle-label",
            ".docs-main",
            ".docs-content",
            ".docs-toc",
            ".docs-toc-title",
            ".docs-toc-aside",
            ".docs-toc-inline",
            ".docs-toc-inline-summary",
            "nav.prev-next",
            ".docs-nav-group",
            ".docs-nav-group-summary",
            ".docs-nav-group-list",
        ] {
            assert!(css.contains(selector), "missing selector: {selector}");
        }
    }

    /// `.docs-search-result` の行クロム（`padding`/`text-decoration`/
    /// `color`）はクリック可能なヒットターゲット本体である子要素
    /// `a`（`crate::script::SITE_JS` が生成）に付与し、`li` 自身には
    /// 付与しないことを固定する（Bugbot 指摘: `li` に付けると UA の
    /// リンク既定スタイルがタイトルに残り、`li` の非パディング領域が
    /// ヒットターゲットから漏れる）。
    #[test]
    fn stylesheet_search_result_row_chrome_targets_the_anchor_child() {
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();

        let anchor_selector = ".docs-search-result > a {";
        let anchor_start = css
            .find(anchor_selector)
            .expect("missing selector: .docs-search-result > a");
        let anchor_end = css[anchor_start..]
            .find('}')
            .map(|offset| anchor_start + offset)
            .expect(".docs-search-result > a block should be closed");
        let anchor_block = &css[anchor_start..anchor_end];
        for declaration in ["padding", "text-decoration: none", "color:"] {
            assert!(
                anchor_block.contains(declaration),
                ".docs-search-result > a should declare {declaration}"
            );
        }

        let li_selector = ".docs-search-result {";
        let li_start = css
            .find(li_selector)
            .expect("missing selector: .docs-search-result");
        let li_end = css[li_start..]
            .find('}')
            .map(|offset| li_start + offset)
            .expect(".docs-search-result block should be closed");
        let li_block = &css[li_start..li_end];
        assert!(
            !li_block.contains("padding") && !li_block.contains("text-decoration"),
            ".docs-search-result (li) should not carry row chrome that belongs on the anchor child"
        );
    }

    #[test]
    fn stylesheet_header_dropdown_opens_on_hover_and_focus_within() {
        // ヘッダードロップダウン（イシュー #908）は JS を使わず `:hover`/
        // `:focus-within` の両方で開く（マウス・キーボード両対応の固定、
        // `crate::nav::header_nav` rustdoc 参照）。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();
        assert!(css.contains(".docs-header-group:hover > .docs-header-dropdown"));
        assert!(css.contains(".docs-header-group:focus-within > .docs-header-dropdown"));
    }

    #[test]
    fn stylesheet_header_trigger_has_focus_visible_ring() {
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();
        assert!(css.contains(".docs-header-trigger:focus-visible"));
    }

    #[test]
    fn stylesheet_header_dropdown_selectors_outrank_nav_list_data_scope_rules() {
        // Bugbot 指摘（イシュー #908 PR #919 レビュー）是正の回帰テスト:
        // `header_nav` は headless `nav_list` anatomy を再利用しているため、
        // `nav_list::stylesheet()` の `[data-scope="nav-list"][data-part="..."]`
        // 系ルール（詳細度最大 0,3,0、`link`+`aria-current` の組み合わせ）に
        // 素の class セレクタ（0,1,0/0,1,1）が競り負け、padding・リンク色・
        // カレントページの font-weight が適用されない不具合があった。
        // サイドバー（`.docs-sidebar nav.sidebar ...`）と同型に
        // `.docs-header nav.docs-header-nav .docs-header-dropdown ...`
        // （詳細度 0,3,1 以上）へ底上げしたことを固定する。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();
        for selector in [
            ".docs-header nav.docs-header-nav .docs-header-dropdown {",
            ".docs-header nav.docs-header-nav .docs-header-dropdown a {",
            ".docs-header nav.docs-header-nav .docs-header-dropdown a:hover {",
            ".docs-header nav.docs-header-nav .docs-header-dropdown a[aria-current=\"page\"] {",
        ] {
            assert!(
                css.contains(selector),
                "missing high-specificity selector: {selector}"
            );
        }
        // 弱い（負ける）セレクタが CSS に残っていないことも固定する
        // （うっかり両方残して二重管理にする回帰を防ぐ）。
        assert!(!css.contains("\n.docs-header-dropdown {"));
        assert!(!css.contains("\n.docs-header-dropdown a {"));
        assert!(!css.contains("\n.docs-header-dropdown a[aria-current=\"page\"] {"));
    }

    #[test]
    fn stylesheet_nav_group_list_selectors_outrank_nav_list_data_scope_rules() {
        // ヘッダードロップダウン（#908 PR #919）と同型の詳細度の罠回帰テスト
        // （イシュー #940）: `.docs-nav-group-list` は nav_list `list()`
        // （`[data-scope="nav-list"][data-part="list"]`、詳細度 0,2,0）を
        // 再利用しているため、素の class セレクタ単体（0,1,0）では競り負ける。
        // サイドバー直下ページ `ul`（`.docs-sidebar nav.sidebar ul`）と同型に
        // `.docs-sidebar nav.sidebar .docs-nav-group-list`
        // （詳細度 0,3,0 以上）へ底上げしたことを固定する。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();
        assert!(
            css.contains(".docs-sidebar nav.sidebar .docs-nav-group-list {"),
            "missing high-specificity selector: .docs-sidebar nav.sidebar .docs-nav-group-list {{"
        );
        // 弱い（負ける）セレクタが単独で残っていないことも固定する
        // （うっかり両方残して二重管理にする回帰を防ぐ）。
        assert!(!css.contains("\n.docs-nav-group-list {"));
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
        //
        // イシュー #908 でヘッダードロップダウンが `:focus-within` を導入した
        // ため（`.docs-header-group:focus-within`、上記
        // `stylesheet_header_dropdown_opens_on_hover_and_focus_within` 参照）、
        // 本テストの意図（サイドバー開閉の情報源は `:checked` のみ）を保ったまま
        // 検証対象を「`:focus-within` を含む行はすべて `docs-header` 系セレクタ
        // に限られる」へ精密化する（弱体化ではなく対象の精密化。
        // `nav.sidebar`/`.docs-sidebar` と `:focus-within` の組み合わせが
        // 存在しないことは変わらず固定する）。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();
        assert!(css.contains(".docs-sidebar-toggle:checked ~ nav.sidebar"));
        assert!(!css.contains("sidebar:focus-within"));
        assert!(!css.contains("nav.sidebar:focus-within"));
        // セレクタ行（`{` で終わる行）のみを対象にする。コメント中の
        // `:focus-within` への言及（本テストのこのコメント自体を含む）を
        // 誤検知しないため。
        for line in css
            .lines()
            .filter(|line| line.contains(":focus-within") && line.trim_end().ends_with('{'))
        {
            assert!(
                line.contains("docs-header"),
                ":focus-within が docs-header 系以外のセレクタに出現している: {line}"
            );
        }
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
    fn stylesheet_toc_aside_is_sticky_at_three_column_breakpoint() {
        // イシュー #909 受け入れ条件 1・2: `min-width: 1200px`（3 カラム帯域）で
        // `.docs-toc-aside` が `.docs-sidebar` と同型の sticky 追従を持つこと、
        // かつ狭幅帯域（`min-width: 1200px` 未満）では `display: none` のまま
        // 非表示を維持しレイアウトを崩さないことを固定する。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();

        let before_1200 = css
            .split("@media (min-width: 1200px)")
            .next()
            .expect("content before min-width: 1200px block should exist");
        // 行継続（`\`）が次行の先頭空白を除去するため、実際の出力 CSS には
        // インデントが残らない（本ファイル既存の他アサーションと同様の前提）。
        assert!(before_1200.contains(".docs-toc-aside {\ndisplay: none;\n}"));

        let block_1200 = css
            .split("@media (min-width: 1200px)")
            .nth(1)
            .expect("min-width: 1200px block should exist");
        assert!(block_1200.contains(".docs-toc-aside {"));
        assert!(block_1200.contains("position: sticky;"));
        assert!(block_1200.contains("top: var(--fandhe-space-docs-header-height);"));
        assert!(block_1200
            .contains("max-height: calc(100vh - var(--fandhe-space-docs-header-height));"));
        assert!(block_1200.contains("overflow-y: auto;"));
    }

    #[test]
    fn stylesheet_inline_toc_is_visible_below_and_hidden_at_the_three_column_breakpoint() {
        // イシュー #1080 受け入れ条件 2: `nav.docs-toc-inline`（本文冒頭の
        // 折りたたみ目次、狭幅帯域の右目次代替）は `.docs-toc-aside`
        // （直上のテスト）と鏡像の帯域制御を持つ: 基底帯域（`< 1200px`）では
        // 表示し、`min-width: 1200px`（右目次カラムが表示に切り替わる帯域）
        // では非表示にして重複表示を避ける。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();

        let before_1200 = css
            .split("@media (min-width: 1200px)")
            .next()
            .expect("content before min-width: 1200px block should exist");
        assert!(before_1200.contains(".docs-toc-inline {\ndisplay: block;"));

        let block_1200 = css
            .split("@media (min-width: 1200px)")
            .nth(1)
            .expect("min-width: 1200px block should exist");
        assert!(block_1200.contains(".docs-toc-inline {\ndisplay: none;\n}"));
    }

    #[test]
    fn stylesheet_inline_toc_does_not_reuse_the_scrollspy_selector() {
        // `crate::script::SITE_JS` のスクロールスパイは
        // `document.querySelector('.docs-toc')`（DOM 先頭の 1 件のみ）で右目次を
        // 掴む契約（`crate::layout::toc_inline` rustdoc 参照）。折りたたみ目次の
        // 規則が `.docs-toc` 単体セレクタへ相乗りしていない（`.docs-toc-inline`
        // という別クラスのみで組まれている）ことを固定し、将来の「セレクタ
        // 統合による簡素化」でこの分離が崩れる回帰を検知する。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();
        assert!(!css.contains(".docs-toc, .docs-toc-inline"));
        assert!(!css.contains(".docs-toc-inline, .docs-toc {"));
        assert!(css.contains(".docs-toc-inline {"));
    }

    #[test]
    fn stylesheet_headings_have_scroll_margin_for_sticky_header() {
        // イシュー #909 受け入れ条件 3: `.docs-header` の sticky（top: 0）に
        // 隠れず見出しアンカーへページ内リンクが機能するよう、id 注入対象
        // （h2/h3、`with_heading_anchors`）に `scroll-margin-top` があることを固定する。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();
        assert!(css.contains(".docs-content h2,\n.docs-content h3 {"));
        assert!(
            css.contains("scroll-margin-top: calc(var(--fandhe-space-docs-header-height) + 1rem);")
        );
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

    /// [`every_referenced_fandhe_token_is_defined`] が使う判別ルール:
    /// `var(--fandhe-x)`（フォールバック無し）は必ず定義されていなければ
    /// ならない。`var(--fandhe-x, fallback)`（フォールバック付き）は部品側
    /// （例: `fandhe_frontend_pre_styled_ui::link` の
    /// `--fandhe-link-text-decoration`）が任意に上書きするためのフックで
    /// あり、docs 側 CSS が定義しなくても正当に許容される。ここへ列挙する
    /// のはその許容リストであり、フォールバック無し参照が万一ここに現れても
    /// スキップされないよう名前の完全一致でのみ照合する（allowlist の
    /// なし崩し的拡大を防ぐ）。
    const FALLBACK_ONLY_TOKENS: &[&str] = &["--fandhe-link-text-decoration"];

    /// 生成 CSS 全量から `var(--fandhe-<...>)` 参照を手書き走査で収集する
    /// （`site_css_contract.rs::extract_css_class_selectors` と同じ「実出力の
    /// 生文字列から直接抽出する」流儀。外部 CSS パーサ crate は追加しない）。
    /// 戻り値は `(トークン名, フォールバック有無)` の組。
    fn extract_fandhe_token_refs(css: &str) -> Vec<(String, bool)> {
        let mut refs = Vec::new();
        let mut cursor = 0usize;
        while let Some(rel) = css[cursor..].find("var(--fandhe-") {
            let start = cursor + rel + "var(".len();
            let rest = &css[start..];
            let Some(end) = rest.find(')') else {
                break;
            };
            let inner = &rest[..end];
            let has_fallback = inner.contains(',');
            let name = inner.split(',').next().unwrap_or(inner).trim().to_string();
            refs.push((name, has_fallback));
            cursor = start + end;
        }
        refs
    }

    #[test]
    fn every_referenced_fandhe_token_is_defined() {
        // ダークモード回帰（設計文書 §6・イシュー #912）の中核テスト:
        // 生成 CSS が `var(--fandhe-...)` で参照するトークンが 1 件でも
        // 未定義だと、その箇所は「変数未定義 → ブラウザは無効値として
        // 無視」となり配色・寸法が silently 抜け落ちる（ダークモードでの
        // 色抜けは特に気付きにくい）。フォールバック無し参照は fail-closed で
        // 定義存在を要求し、フォールバック付き参照（部品側の任意上書き
        // フック）のみ [`FALLBACK_ONLY_TOKENS`] allowlist と完全一致する
        // 場合に限り許容する。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();

        let mut defined = std::collections::BTreeSet::new();
        for line in css.lines() {
            let line = line.trim().trim_end_matches(';');
            if let Some(rest) = line.strip_prefix("--fandhe-") {
                if let Some(idx) = rest.find(':') {
                    defined.insert(format!("--fandhe-{}", &rest[..idx]));
                }
            }
        }

        for (name, has_fallback) in extract_fandhe_token_refs(css) {
            if defined.contains(&name) {
                continue;
            }
            if has_fallback && FALLBACK_ONLY_TOKENS.contains(&name.as_str()) {
                continue;
            }
            panic!(
                "undefined --fandhe token referenced without a safe fallback: {name} \
                 (has_fallback={has_fallback}). 新規に導入する場合は Theme 側で定義するか、\
                 意図的なフォールバック専用フックなら FALLBACK_ONLY_TOKENS へ追記すること。"
            );
        }
    }

    #[test]
    fn color_and_shadow_tokens_are_defined_in_all_three_mode_blocks() {
        // 色・影トークンは `Theme::to_css` が既定（light）/
        // `@media (prefers-color-scheme: dark)` / `:root[data-theme="dark"]`
        // の 3 ブロックへ書き出す（モード依存トークンのため）。一方
        // 寸法・タイポグラフィトークン（space/radius/font）はモード非依存
        // で既定ブロックのみに 1 回だけ書き出される。両グループを一律
        // 「3 回」で判定すると寸法系が誤って失敗するため、グループ別に
        // 期待回数を分ける（イシュー #912、設計文書 §5 の承継項目）。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();

        for name in [
            "--fandhe-color-bg",
            "--fandhe-color-fg",
            "--fandhe-color-fg-muted",
            "--fandhe-color-border",
            "--fandhe-color-bg-subtle",
            "--fandhe-color-bg-muted",
            "--fandhe-color-accent",
            "--fandhe-color-docs-accent-bg",
            // イシュー #1422 で theme 既定パレットへ追加した新規色トークン。
            // docs サイトも `Theme::default()` を基底に組むため、生成 CSS の
            // 3 ブロック規約（light/`@media` dark/`data-theme` dark）を
            // そのまま満たすことを確認する。
            "--fandhe-color-accent-subtle",
            "--fandhe-color-neutral",
            "--fandhe-shadow-md",
        ] {
            let count = css.matches(&format!("{name}:")).count();
            assert_eq!(
                count, 3,
                "color/shadow token {name} should be defined in exactly 3 blocks (light/dark media/data-theme), found {count}"
            );
        }

        for name in [
            "--fandhe-space-docs-container-width",
            "--fandhe-space-docs-gutter",
            "--fandhe-space-docs-sidebar-width",
            "--fandhe-space-docs-max-content-width",
            "--fandhe-space-docs-header-height",
            "--fandhe-space-docs-toc-width",
            "--fandhe-font-font-size-sm",
            "--fandhe-radius-sm",
        ] {
            let count = css.matches(&format!("{name}:")).count();
            assert_eq!(
                count, 1,
                "mode-independent token {name} should be defined exactly once, found {count}"
            );
        }
    }

    #[test]
    fn stylesheet_declares_color_scheme_light_dark() {
        // `html { color-scheme: light dark; }`（STRUCTURAL_CSS 側）が
        // フォームコントロール・スクロールバー等 OS ネイティブ UI の
        // ダーク追従の前提であることを固定する（`Theme::to_css` 側が
        // `:root` に書く `color-scheme` とは別に、docs サイトの `html`
        // 要素へも明示する）。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();
        assert!(css.contains("html {\ncolor-scheme: light dark;\n}"));
    }

    #[test]
    fn structural_and_typography_css_contain_no_hardcoded_colors() {
        // ハードコード色の再導入を防ぐガード（イシュー #912、§2.4 是正の
        // 回帰防止）。検証対象は STRUCTURAL_CSS と typography_css() のみで
        // あり、組み立て済みシート全体（`stylesheet()`）を対象にしては
        // ならない: `Theme::to_css` はトークン値そのものとして正当に
        // `rgba(...)` を出力するため（shadow トークンの light/dark 値）、
        // シート全体を検査すると常に失敗する。
        //
        // CSS コメント（`/* ... */`）を除去してから走査する: STRUCTURAL_CSS
        // には `#907`/`#908`/`#916` 等のイシュー番号参照コメントが多数あり、
        // 3 桁 16 進数 hex カラーと文字面が一致してしまうため
        // （偽陽性防止）。
        fn strip_css_comments(css: &str) -> String {
            let mut out = String::with_capacity(css.len());
            let mut rest = css;
            while let Some(start) = rest.find("/*") {
                out.push_str(&rest[..start]);
                rest = &rest[start + 2..];
                if let Some(end) = rest.find("*/") {
                    rest = &rest[end + 2..];
                } else {
                    rest = "";
                }
            }
            out.push_str(rest);
            out
        }

        fn assert_no_hardcoded_colors(css: &str, label: &str) {
            let stripped = strip_css_comments(css);
            for needle in ["rgb(", "rgba(", "hsl(", "hsla(", "#"] {
                assert!(
                    !stripped.contains(needle),
                    "{label} should reference colors via --fandhe-* tokens only, found forbidden pattern {needle:?}"
                );
            }
        }

        assert_no_hardcoded_colors(STRUCTURAL_CSS, "STRUCTURAL_CSS");
        let typography = typography_css().expect("typography css should assemble");
        assert_no_hardcoded_colors(&typography, "typography_css()");
    }

    #[test]
    fn stylesheet_base_breakpoint_matches_responsive_contract() {
        // レスポンシブ 3 帯域（設計文書 §3.2）の基底帯域（`< 768px`）契約を
        // 行単位で固定する（イシュー #912）。STRUCTURAL_CSS は mobile-first
        // で組み立てられているため、最初の `@media (min-width: 768px)`
        // より前の区間が基底帯域の宣言となる。
        let before_768 = STRUCTURAL_CSS
            .split("@media (min-width: 768px)")
            .next()
            .expect("STRUCTURAL_CSS should contain the 768px breakpoint");

        // 注意: STRUCTURAL_CSS はソース上インデントされているが、Rust の
        // 行末 `\` 継続は改行と後続行の**全ての先頭空白**を読み飛ばすため
        // （`tmp_debug_print` による実測確認済み）、生成される CSS 文字列
        // 自体には字下げが一切残らない。以下のアサーションはインデント
        // 無しの実出力に合わせる。
        assert!(before_768.contains(".docs-container {\ndisplay: block;\n"));
        assert!(before_768.contains(".docs-toc-aside {\ndisplay: none;\n}"));
        assert!(before_768.contains(".docs-sidebar-toggle-label {\ndisplay: block;\n"));
        assert!(before_768
            .contains(".docs-sidebar nav.sidebar {\nmax-height: 2.75rem;\noverflow: hidden;\n}"));
        assert!(before_768.contains(".docs-header-nav {\ndisplay: none;\n}"));
        assert!(before_768.contains("nav.prev-next {\ndisplay: flex;\n/* 基底（768px 未満）は縦積み。`min-width: 768px` で横並びに切り替える。 */\nflex-direction: column;"));
    }

    #[test]
    fn stylesheet_media_queries_are_ordered_mobile_first() {
        // mobile-first のカスケード順序契約: `@media (min-width: 768px)` の
        // 出現位置が `@media (min-width: 1200px)` より前であることを固定
        // する。逆転すると 1200px ブロックの 3 カラム指定を 768px ブロックの
        // 2 カラム指定が後方から上書きしてしまう（イシュー #912、カスケード
        // 事故の fail-closed 検知）。
        let pos_768 = STRUCTURAL_CSS
            .find("@media (min-width: 768px)")
            .expect("768px breakpoint should exist");
        let pos_1200 = STRUCTURAL_CSS
            .find("@media (min-width: 1200px)")
            .expect("1200px breakpoint should exist");
        assert!(
            pos_768 < pos_1200,
            "768px media query must appear before 1200px to preserve mobile-first cascade"
        );
    }

    #[test]
    fn stylesheet_header_is_a_full_width_bar_matching_backend() {
        // ヘッダー詳細統一（イシュー #1110 の続き）の宣言固定テスト:
        // `.docs-header-inner` は fandhe-backend の
        // `.docs-header { padding: 0 1.5rem; }` と同じ全幅バーであり、
        // イシュー #949 の中央寄せ計測枠（container-width の max-width +
        // `margin: 0 auto` + gutter padding）を持たないこと、一方で
        // `.docs-container`・`.docs-sidebar` は引き続きトークンを参照する
        // ことを固定する。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();

        let inner_start = css
            .find(".docs-header-inner {")
            .expect(".docs-header-inner rule should exist");
        let inner_end = css[inner_start..]
            .find('}')
            .expect(".docs-header-inner rule should close");
        let inner_block = &css[inner_start..inner_start + inner_end];
        assert!(
            inner_block.contains("padding: 0 1.5rem;"),
            ".docs-header-inner は backend 同値の全幅 padding を持つ必要がある: {inner_block}"
        );
        for forbidden in ["max-width", "margin: 0 auto", "docs-gutter"] {
            assert!(
                !inner_block.contains(forbidden),
                ".docs-header-inner は #949 の中央寄せ計測枠（{forbidden}）を復活させてはならない \
                 （イシュー #1110 の続きで全幅バーへ統一済み）: {inner_block}"
            );
        }

        assert!(css.contains(".docs-container {\ndisplay: block;\nmax-width: var(--fandhe-space-docs-container-width);\nmargin: 0 auto;\n}"));

        // ブランドは backend 同様 padding を持たない（旧 #949 の左端揃え
        // padding の復活を検知する）。
        let brand_start = css
            .find(".docs-brand {")
            .expect(".docs-brand rule should exist");
        let brand_end = css[brand_start..]
            .find('}')
            .expect(".docs-brand rule should close");
        let brand_block = &css[brand_start..brand_start + brand_end];
        assert!(
            !brand_block.contains("padding"),
            ".docs-brand は padding を持たない（backend 同値、イシュー #1110 の続き）: {brand_block}"
        );

        assert!(css
            .contains(".docs-sidebar {\nwidth: 100%;\npadding: var(--fandhe-space-docs-gutter);"));
    }

    #[test]
    fn stylesheet_layout_width_budget_is_consistent() {
        // レイアウト幅トークン（container/sidebar/toc/max-content）は
        // fandhe-backend の docs サイトとデザインを統一するため、
        // fandhe-backend `site/assets/site.css` の値（88rem/16rem/14rem/
        // 46rem）へ合わせている（正は同ファイル）。旧イシュー #949 の幅予算
        // 計算式（84 − 17 − 15 − 2×2 = 48）による内部整合の導出は廃止した
        // （`docs_theme` コメント参照。88 − 16 − 14 − 2×2 = 54 ≠ 46 だが、
        // これは意図した不一致であり計算エラーではない）。
        // `.docs-main` の 768px ブロック padding は backend の
        // `.docs-main { padding: 2.25rem 2rem 5rem; }` と元々一致しているため
        // 変更しておらず、引き続き同時に固定する。
        let sheet = stylesheet().expect("site theme stylesheet should assemble");
        let css = sheet.as_css();

        assert!(css.contains("--fandhe-space-docs-container-width: 88rem;"));
        assert!(css.contains("--fandhe-space-docs-sidebar-width: 16rem;"));
        assert!(css.contains("--fandhe-space-docs-toc-width: 14rem;"));
        assert!(css.contains("--fandhe-space-docs-max-content-width: 46rem;"));

        let block_768 = STRUCTURAL_CSS
            .split("@media (min-width: 768px)")
            .nth(1)
            .expect("min-width: 768px block should exist");
        assert!(block_768.contains(".docs-main {\npadding: 2.25rem 2rem 5rem;\n}"));
    }

    #[test]
    fn stylesheet_table_has_horizontal_scroll_affordance() {
        // テーブルの横スクロールアフォーダンス（イシュー #949、受け入れ条件
        // 2）。`push_typography_rule`/`serialize_rule` は不正な宣言を黙って
        // スキップするため、出力文字列を直接検査しないと「入ったつもり」を
        // 検知できない（`push_typography_rule` doc コメント参照）。
        let typography = typography_css().expect("typography css should assemble");

        assert!(typography.contains("scrollbar-width: thin;"));
        assert!(typography.contains(
            "scrollbar-color: var(--fandhe-color-border) var(--fandhe-color-bg-subtle);"
        ));
        assert!(typography.contains(".docs-content table::-webkit-scrollbar {"));
        assert!(typography.contains(".docs-content table::-webkit-scrollbar-track {"));
        assert!(typography.contains(".docs-content table::-webkit-scrollbar-thumb {"));
        assert!(typography.contains(".docs-content table {\n  display: block;"));
        assert!(typography.contains(
            "  border: 1px solid var(--fandhe-color-border);\n  border-radius: 0.5rem;\n  scrollbar-width: thin;"
        ));

        // 端フェード影（イシュー #1079）。longhand 5 宣言が揃っていること、
        // かつレイヤ数（`linear-gradient(` の出現数）と `background-attachment`
        // の値要素数が一致することを検査し、無言のレイヤ欠落を機械検知する。
        assert!(typography.contains("background-attachment: local, local, scroll, scroll;"));
        assert!(typography.contains(
            "background-image: linear-gradient(to right, var(--fandhe-color-bg), transparent), linear-gradient(to left, var(--fandhe-color-bg), transparent), linear-gradient(to right, var(--fandhe-color-docs-scroll-shadow), transparent), linear-gradient(to left, var(--fandhe-color-docs-scroll-shadow), transparent);"
        ));
        assert!(typography.contains(
            "background-position: left center, right center, left center, right center;"
        ));
        assert!(
            typography.contains("background-size: 2rem 100%, 2rem 100%, 0.9rem 100%, 0.9rem 100%;")
        );
        assert!(typography.contains("background-repeat: no-repeat;"));

        let table_rule_start = typography
            .find(".docs-content table {\n")
            .expect("table rule should exist");
        let table_rule_end = typography[table_rule_start..]
            .find("}\n")
            .map(|offset| table_rule_start + offset)
            .expect("table rule should be closed");
        let table_rule_block = &typography[table_rule_start..table_rule_end];
        let gradient_layer_count = table_rule_block.matches("linear-gradient(").count();
        let attachment_value_count = "local, local, scroll, scroll".split(", ").count();
        assert_eq!(
            gradient_layer_count, attachment_value_count,
            "background-image のレイヤ数と background-attachment の値要素数は一致すること"
        );
    }

    #[test]
    fn stylesheet_table_edge_fade_token_covers_light_and_dark() {
        // 端フェード影トークン（イシュー #1079）がライト/ダーク双方の
        // メディアクエリ・data-theme 経路へ展開されていることを検査する
        // （受入条件「ライト / ダーク両テーマで機能する」の機械化）。
        let sheet = stylesheet().expect("stylesheet should assemble");
        let css = sheet.as_css();

        assert!(css.contains("--fandhe-color-docs-scroll-shadow: rgba(15,23,42,0.20);"));
        let dark_occurrences = css
            .matches("--fandhe-color-docs-scroll-shadow: rgba(0,0,0,0.55);")
            .count();
        assert!(
            dark_occurrences >= 2,
            "ダーク値は prefers-color-scheme: dark と data-theme=\"dark\" の双方に出ること（実際: {dark_occurrences} 件）"
        );
    }
}
