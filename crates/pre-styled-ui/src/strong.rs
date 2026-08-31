//! Strong（イシュー #995）: variant を持たない最小静的部品。重要性の強調
//! テキスト（`<strong>`）を既定スタイルで組み立てる。[`crate::em`] /
//! [`crate::link_overlay`] と同型の「variant 軸を持たない slot recipe」
//! パターンに従う。
//!
//! [`crate::em`]（`<em>`、文法的な強勢の強調）との役割差: `strong` は
//! 重要性・緊急性を表す強調（HTML 意味論上 `<strong>` と `<em>` は別概念）
//! であり、見た目上も `strong` は `font-weight: bold`、`em` は
//! `font-style: italic`（font-weight は本文から継承し `bold` への上書きは
//! 持たない）で区別する。イシュー #1433 の参考サイト（chakra-ui /
//! Radix Themes）7 軸比較を受け、`em` が旧実装で持っていた
//! `font-weight: medium` への上書きは参照サイトのいずれにも無い装飾
//! だったため廃止し、継承へ是正済みである。両部品の役割差は「太字（weight
//! 上書き）か斜体（style 上書き・weight は継承）か」という様式の違いで
//! 成立する（[`crate::em`] モジュール rustdoc 参照）。
//!
//! ## 参考サイト基準との 7 軸比較（イシュー #1441）
//!
//! Radix Themes の `Strong` とサイズ・バリアント・色・状態 `data-*`・
//! ダーク・フォーカス・余白 / hover / disabled / transition の 7 軸で
//! 比較した結果（chakra-ui には `Strong` 相当部品が存在しないため比較
//! 対象は Radix Themes 単独とする。`docs/design/component-coverage-map.md`
//! の Strong 行でも chakra-ui 対応欄は `—` である）:
//!
//! - **サイズ・バリアント軸**: Radix Themes は `size`/`variant` prop を
//!   持たない（size は親 `Text` から継承）。`strong` も軸を追加しない
//!   （Typography 周辺部品は size 軸を持たない、
//!   `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
//!   §4 (c)）。
//! - **font-weight**: Radix Themes は `--strong-font-weight:
//!   var(--font-weight-bold)` というトークン間接参照で太字化しており、
//!   本実装の `font-weight: var(--fandhe-font-font-weight-bold)`
//!   （`--fandhe-*` トークンの直接参照）と結果は一致する。`strong`
//!   1 部品のためだけの専用トークン層（`--strong-font-weight` 相当）は
//!   新設しない（**意図的に非採用**）。理由: 本リポジトリのテーマ体系は
//!   部品専用の間接トークン層を持たず `--fandhe-font-font-weight-bold`
//!   の直接参照のみで完結しており、1 部品専用の中間トークンを追加すると
//!   テーマ体系の一貫性を崩す（`em` の serif トークン非採用、イシュー
//!   #1433、と同じ判断軸）。
//! - **font-family / font-style / letter-spacing の部品専用オーバーライド**:
//!   Radix Themes は `--strong-font-family` 等の専用変数層を持つが、
//!   いずれも既定値は継承（無指定）であり実質的な差分を生まない。上記と
//!   同じ理由で専用トークン層は新設しない（**意図的に非採用**）。
//! - **色・ダーク**: 色宣言を持たず本文色を継承するため、ライト / ダーク
//!   どちらでも自動的に本文へ追従する（Radix Themes と一致）。
//! - **状態 `data-*`**: headless-ui 側に `strong` の状態属性は存在せず、
//!   Radix Themes も状態を持たない表示専用部品のため変更不要。
//! - **hover / disabled / transition / フォーカスリング**: `strong` は
//!   非インタラクティブな表示専用 slot であり、Radix Themes もこれらの
//!   状態を持たない。本フレームワークでも hover / disabled / transition
//!   はインタラクティブ slot のみに適用する方針
//!   （`docs/design/pre-styled-ui-interaction-visual-language.md` §3）
//!   であり、フォーカスリングもフォーカス対象部品限定
//!   （`pre-styled-ui-focus-ring-and-size-conventions.md` §3）のため、
//!   `strong` には付与しない。
//! - **余白・角丸・影**: Radix Themes は持たない。`strong` も持たない。
//!
//! 上記比較の結果、CSS 出力の変更は不要と判断した（既存の
//! `font-weight: var(--fandhe-font-font-weight-bold)` 1 宣言のみで
//! 参照サイト基準を満たす）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="strong"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("strong");

/// Strong の recipe（scope `"strong"`、slot `"root"` のみ、variant 軸なし）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("strong", &["root"]).base(
        "root",
        vec![decl("font-weight", "var(--fandhe-font-font-weight-bold)")],
    )
}

/// Strong の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Strong 1 個（`<strong>`）を組み立てる。variant 軸を持たないため `class`
/// 属性は付与しない（呼び出し側 `attrs` の `class` は他 styled 部品との
/// 一貫性のため [`drop_class_attr`] で除去する。[`crate::em::em`] と同型の
/// 判断）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::strong::strong;
///
/// let node = strong(vec![], vec![text("important")]);
/// let html = render(&node);
/// assert!(html.starts_with("<strong"));
/// assert!(html.contains("important"));
/// ```
#[must_use]
pub fn strong<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("root", "strong", drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_renders_strong_tag_with_scope_and_part() {
        let html = render(&strong(vec![], vec![text("important")]));
        assert_eq!(
            html,
            r#"<strong data-scope="strong" data-part="root">important</strong>"#
        );
    }

    #[test]
    fn caller_class_attr_is_dropped() {
        let html = render(&strong(vec![("class", "attacker-controlled")], vec![]));
        assert!(!html.contains("class="));
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        let html = render(&strong(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn xss_payload_in_caller_attrs_is_escaped() {
        let html = render(&strong(
            vec![("data-testid", "\"><script>alert(1)</script>")],
            vec![],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_font_weight_bold() {
        let out = css();
        assert!(out.contains("font-weight: var(--fandhe-font-font-weight-bold);"));
    }

    #[test]
    fn css_output_is_deterministic() {
        assert_eq!(css(), css());
    }
}
