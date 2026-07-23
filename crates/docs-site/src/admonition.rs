//! admonition 構文（`> [!NOTE]` 等）が使う専用 CSS の配線（イシュー #715）。
//!
//! # 役割・呼び出し文脈
//!
//! [`crate::markdown::render_markdown`] は `> [!NOTE]` 等のマーカーを検出すると
//! `pre-styled-ui` の [`fandhe_frontend_pre_styled_ui::alert`] 部品で描画する
//! （`crate::markdown` のモジュール doc 参照）。本モジュールはその alert 部品が
//! 必要とする CSS（テーマトークン + alert recipe + admonition 専用の配置
//! スタイル）を [`crate::showcase`] と同型の「分離 CSS 方式」で組み立てる。
//!
//! `site/assets/site.css`（サイト骨格スタイル）は一切変更しない。生成 CSS は
//! [`crate::build::build_site`] が [`STYLESHEET_REL_PATH`] へ書き出し、
//! admonition を実際に含むページだけが `crate::layout::docs_page_with_assets`
//! の追加 `<link>` でこれを参照する（[`contains_admonition`] が「含むか」を
//! `build_site` に伝える判定関数）。admonition を含まないページ・フィクスチャ
//! サイトのビルド結果は本モジュールの追加により一切変わらない。
//!
//! # ダークモード配色（イシュー #732）
//!
//! `alert` recipe（[`alert::css`]）のダーク背景（`--fandhe-color-bg-subtle`
//! のニュートラルグレー）は、docs サイト骨格（`site/assets/site.css`）の
//! 青みがかったダークパレット（`--docs-color-bg-subtle` 等）と調和しない。
//! [`DARK_CSS`] は `.docs-content` スコープに限定した上書きを、
//! [`fandhe_frontend_pre_styled_ui::theme::Theme::to_css`] と同じ 2 ブロック
//! 構造（`@media (prefers-color-scheme: dark) { :root:not([data-theme="light"])
//! ... }` ＋ 末尾の `:root[data-theme="dark"] ...`、明示指定が常に勝つ）で
//! 追加する。参照する `--docs-color-*` custom property は `var()` の第 2
//! 引数フォールバックで theme トークンへ退避するため、`site.css` 側の名前が
//! 変わっても真っ白/無指定にはならない（fail-safe。ドリフト検知は
//! `crates/docs-site/tests/site_css_contract.rs` 参照）。`.docs-content`
//! スコープ限定のため light モード出力・admonition を含まないページの
//! ビルド結果は不変（#715 の分離 CSS 不変条件を維持）。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! CSS は [`StyleSheet`]（検証済み CSS のみを保持し `<`・不正な制御文字を
//! 拒否する型、`crates/pre-styled-ui/src/stylesheet.rs`）経由でのみ組み立てる。
//! 配置用の追加 CSS（[`LAYOUT_CSS`]・[`DARK_CSS`]）も同じ検証を通す。

use fandhe_frontend_core::Node;
use fandhe_frontend_pre_styled_ui::theme::Theme;
use fandhe_frontend_pre_styled_ui::{alert, StyleSheet, StylesheetError};

/// admonition 専用 CSS の出力先（`out_dir` 起点の相対パス）。
/// `crate::build::build_site` が [`stylesheet`] の内容をこのパスへ書き出し、
/// ページ `<head>` の追加 `<link>`（`docs_page_with_assets`）が参照する。
pub const STYLESHEET_REL_PATH: &str = "assets/admonition.css";

/// admonition として描画された alert の `.docs-content` 内での配置調整。
/// alert 部品自体の見た目は recipe（[`alert::css`]）が担い、ここでは
/// Markdown 本文フロー内での上下マージン・indicator（イシュー #732 で
/// 追加した種別アイコン）の光学的整列のみを補う（`site.css` のクラス名
/// 契約・カスケードには影響させない、`crate::showcase` と同じ分離方針）。
const LAYOUT_CSS: &str = "\
.docs-content [data-scope=\"alert\"][data-part=\"root\"] {\n  margin: 1rem 0;\n}\n\
.docs-content [data-scope=\"alert\"][data-part=\"indicator\"] {\n  display: inline-flex;\n  margin-top: 0.125rem;\n}\n";

/// admonition（`.docs-content` 内の alert）のダークモード配色調整
/// （モジュール doc「ダークモード配色」節参照、イシュー #732）。
///
/// `Theme::to_css` と同じ 2 ブロック構造（OS 追従 + 明示 `data-theme` が
/// 末尾で勝つ）を採り、docs サイトのダークパレット（`--docs-color-*`、
/// `site/assets/site.css`）へ面色・枠線色を合わせる。`var()` の第 2 引数
/// フォールバック（`--fandhe-color-*`）により、`site.css` 側の custom
/// property 名が変わっても theme トークンへ fail-safe に退避する。
const DARK_CSS: &str = "\
@media (prefers-color-scheme: dark) {\n\
  :root:not([data-theme=\"light\"]) .docs-content [data-scope=\"alert\"][data-part=\"root\"] {\n\
    background: var(--docs-color-bg-subtle, var(--fandhe-color-bg-subtle));\n\
    border: 1px solid var(--docs-color-border, var(--fandhe-color-border));\n\
  }\n\
}\n\
:root[data-theme=\"dark\"] .docs-content [data-scope=\"alert\"][data-part=\"root\"] {\n\
  background: var(--docs-color-bg-subtle, var(--fandhe-color-bg-subtle));\n\
  border: 1px solid var(--docs-color-border, var(--fandhe-color-border));\n\
}\n";

/// admonition が参照する CSS 全量を組み立てる。
///
/// 内訳: テーマトークン（`Theme::default`）→ alert recipe CSS（[`alert::css`]）
/// → [`LAYOUT_CSS`] → [`DARK_CSS`]、の順で決定的に連結する。
///
/// # Errors
///
/// いずれかの CSS 断片が [`StyleSheet::push_css`] の検証（`<`・制御文字の
/// 拒否）に落ちた場合 [`StylesheetError`] を返す。pre-styled-ui 側の生成 CSS・
/// [`LAYOUT_CSS`]・[`DARK_CSS`] は構造上 `<` を含み得ないため通常は到達しないが、
/// 黙って欠けた CSS を公開しない fail-closed 方針で伝播させる（`crate::showcase`
/// と同じ扱い）。
pub fn stylesheet() -> Result<StyleSheet, StylesheetError> {
    let mut sheet = StyleSheet::new();
    sheet.push_theme(&Theme::default());
    sheet.push_css(&alert::css())?;
    sheet.push_css(LAYOUT_CSS)?;
    sheet.push_css(DARK_CSS)?;
    Ok(sheet)
}

/// `node` の木の中に admonition（alert 部品、`data-scope="alert"`）が
/// 1 つでも含まれるかどうかを判定する。
///
/// `crate::build::build_site` がページごとにこの結果を見て、admonition を
/// 含むページにのみ [`STYLESHEET_REL_PATH`] への `<link>` を追加し・
/// linkcheck の既知 href へ登録する（含まないページ・フィクスチャサイトの
/// ビルド結果を変えないため）。ノード数に対して線形の走査であり、
/// アルゴリズム的計算量 DoS を導入しない。
#[must_use]
pub fn contains_admonition(node: &Node) -> bool {
    match node {
        Node::Element {
            attrs, children, ..
        } => {
            attrs.iter().any(|(k, v)| k == "data-scope" && v == "alert")
                || children.iter().any(contains_admonition)
        }
        Node::Text(_) | Node::RawHtml(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{div, p, text};
    use fandhe_frontend_pre_styled_ui::AlertStatus;

    #[test]
    fn contains_admonition_detects_alert_scope_anywhere_in_tree() {
        let with_alert = div(
            vec![],
            vec![
                p(vec![], vec![text("plain")]),
                alert::root(AlertStatus::Info, vec![], vec![]),
            ],
        );
        assert!(contains_admonition(&with_alert));

        let without_alert = div(vec![], vec![p(vec![], vec![text("plain only")])]);
        assert!(!contains_admonition(&without_alert));
    }

    #[test]
    fn stylesheet_covers_theme_and_alert_recipe_and_layout_css() {
        let sheet = stylesheet().expect("admonition stylesheet should assemble");
        let css = sheet.as_css();
        assert!(css.contains("--fandhe-color-"));
        assert!(css.contains(".fd-alert--status-info"));
        assert!(css.contains(r#".docs-content [data-scope="alert"][data-part="root"]"#));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_covers_indicator_layout_and_dark_mode_conditions() {
        // イシュー #732: indicator 配置調整・ダーク配色（OS 追従 +
        // 明示 data-theme の 2 ブロック）が stylesheet に含まれること。
        let sheet = stylesheet().expect("admonition stylesheet should assemble");
        let css = sheet.as_css();
        assert!(css.contains(r#".docs-content [data-scope="alert"][data-part="indicator"]"#));
        assert!(css.contains("@media (prefers-color-scheme: dark)"));
        assert!(css.contains(
            r#":root[data-theme="dark"] .docs-content [data-scope="alert"][data-part="root"]"#
        ));
        assert!(css.contains("--docs-color-bg-subtle"));
        assert!(css.contains("--docs-color-border"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_is_deterministic() {
        let a = stylesheet().unwrap().as_css().to_string();
        let b = stylesheet().unwrap().as_css().to_string();
        assert_eq!(a, b);
    }
}
