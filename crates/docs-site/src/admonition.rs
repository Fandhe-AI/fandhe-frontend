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
//! サイト骨格スタイル（`crate::site_theme` がビルド時生成する `assets/site.css`、
//! イシュー #905）は一切変更しない。生成 CSS は [`crate::build::build_site`]
//! が [`STYLESHEET_REL_PATH`] へ書き出し、admonition を実際に含むページ
//! だけが `crate::layout::docs_page_with_assets` の追加 `<link>` でこれを
//! 参照する（[`contains_admonition`] が「含むか」を `build_site` に伝える
//! 判定関数）。admonition を含まないページ・フィクスチャサイトのビルド結果は
//! 本モジュールの追加により一切変わらない。
//!
//! # ダークモード配色（イシュー #732 → #905 で撤去）
//!
//! `alert` recipe（[`alert::css`]）のダーク背景は元々
//! `--fandhe-color-bg-subtle` のニュートラルグレーで、docs サイト骨格の
//! 旧 `--docs-*` 青みがかったダークパレットと調和しなかったため、
//! `.docs-content` スコープに限定した専用の上書き CSS（`DARK_CSS`）を追加
//! していた。イシュー #905 でサイト骨格 CSS 自体を `--fandhe-*` テーマ
//! トークンへ一本化した結果、alert recipe とサイト骨格が同一パレットを
//! 共有するようになり、この上書きは構造的に不要になった（撤去済み）。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! CSS は [`StyleSheet`]（検証済み CSS のみを保持し `<`・不正な制御文字を
//! 拒否する型、`crates/pre-styled-ui/src/stylesheet.rs`）経由でのみ組み立てる。
//! 配置用の追加 CSS（[`LAYOUT_CSS`]）も同じ検証を通す。

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

/// admonition が参照する CSS 全量を組み立てる。
///
/// 内訳: テーマトークン（`Theme::default`）→ alert recipe CSS（[`alert::css`]）
/// → [`LAYOUT_CSS`]、の順で決定的に連結する。
///
/// # Errors
///
/// いずれかの CSS 断片が [`StyleSheet::push_css`] の検証（`<`・制御文字の
/// 拒否）に落ちた場合 [`StylesheetError`] を返す。pre-styled-ui 側の生成 CSS・
/// [`LAYOUT_CSS`] は構造上 `<` を含み得ないため通常は到達しないが、
/// 黙って欠けた CSS を公開しない fail-closed 方針で伝播させる（`crate::showcase`
/// と同じ扱い）。
pub fn stylesheet() -> Result<StyleSheet, StylesheetError> {
    let mut sheet = StyleSheet::new();
    sheet.push_theme(&Theme::default());
    sheet.push_css(&alert::css())?;
    sheet.push_css(LAYOUT_CSS)?;
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
    fn stylesheet_covers_indicator_layout_css() {
        // イシュー #732: indicator 配置調整が stylesheet に含まれること。
        // 旧 DARK_CSS（`.docs-content` スコープの `--docs-color-*` 上書き）は
        // イシュー #905 のトークン一本化により撤去済み（モジュール doc
        // 「ダークモード配色」節参照）。ダーク配色自体は `Theme::default`
        // 由来の `--fandhe-color-bg-subtle`/`--fandhe-color-border` の
        // ダーク値へ alert recipe が追従する。
        let sheet = stylesheet().expect("admonition stylesheet should assemble");
        let css = sheet.as_css();
        assert!(css.contains(r#".docs-content [data-scope="alert"][data-part="indicator"]"#));
        assert!(!css.contains("--docs-"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_is_deterministic() {
        let a = stylesheet().unwrap().as_css().to_string();
        let b = stylesheet().unwrap().as_css().to_string();
        assert_eq!(a, b);
    }
}
