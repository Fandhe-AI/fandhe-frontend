//! Em（イシュー #771）: variant を持たない最小静的部品。強調テキスト
//! （`<em>`）を既定スタイルで組み立てる。[`crate::link_overlay`] と同型の
//! 「variant 軸を持たない slot recipe」パターンに従う。
//!
//! 重要性の強調（`<strong>`）は [`crate::strong`] が担う（イシュー #995）。
//! `em` は文法的な強勢（`font-style: italic`）、`strong` は重要性
//! （`font-weight: bold`）と役割・見た目を区別する。
//!
//! ## 参考サイト基準との 7 軸比較（イシュー #1433）
//!
//! chakra-ui / Radix Themes の `Em` 相当部品とサイズ・バリアント・色・
//! 状態・ダーク・フォーカス・余白 / hover / disabled / transition の
//! 7 軸で比較した結果:
//!
//! - **サイズ・バリアント軸**: 両サイト共に `size`/`variant` prop を
//!   持たない。`em` も軸を追加しない（Typography 周辺部品は size 軸を
//!   持たない、`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
//!   §4 (c)）。
//! - **font-weight**: 両サイト共に本文から weight を継承させており、
//!   `em` 起点での上書きを持たない。旧実装が持っていた
//!   `font-weight: medium` の上書きは参照サイトのいずれにも無い装飾
//!   だったため廃止し、継承へ是正した。
//! - **font-family（serif 差し替え）**: Radix Themes は `em` に serif 系
//!   font-family（`--em-font-family` 相当）を当てるが、**意図的に非採用**
//!   とする。理由: (i) chakra は採用しておらず参照 2 軸で一致しない
//!   Radix 固有の意匠であること、(ii) `theme.rs` の typography トークンは
//!   `font-body`（sans）のみであり、`em` 1 部品のためだけに serif トークン
//!   を新設するとテーマ体系の一貫性を崩すこと、(iii) 和文本文
//!   （docs サイトを含む）で Times 系 fallback は表示品質が安定しない
//!   こと。
//! - **色・ダーク**: 色宣言を持たず本文色を継承するため、ライト / ダーク
//!   どちらでも自動的に本文へ追従する（両サイトと一致）。
//! - **hover / disabled / transition / フォーカスリング**: `em` は非
//!   インタラクティブな表示専用 slot であり、両サイトともこれらの状態を
//!   持たない。本フレームワークでも hover / disabled / transition は
//!   インタラクティブ slot のみに適用する方針
//!   （`docs/design/pre-styled-ui-interaction-visual-language.md` §3）
//!   であり、フォーカスリングもフォーカス対象部品限定
//!   （`pre-styled-ui-focus-ring-and-size-conventions.md` §3）のため、
//!   `em` には付与しない。
//! - **余白・角丸・影**: 両サイト共に持たない。`em` も持たない。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="em"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("em");

/// Em の recipe（scope `"em"`、slot `"root"` のみ、variant 軸なし）。
///
/// `font-style: italic` のみを宣言する。旧実装が持っていた
/// `font-weight: medium` の上書きはイシュー #1433 の 7 軸比較で参照
/// サイト（chakra-ui / Radix Themes）のいずれにも存在しないことが確認
/// できたため廃止し、本文の font-weight を継承させる（モジュール
/// rustdoc の「参考サイト基準との 7 軸比較」節を参照）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("em", &["root"]).base("root", vec![decl("font-style", "italic")])
}

/// Em の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Em 1 個（`<em>`）を組み立てる。variant 軸を持たないため `class` 属性は
/// 付与しない（呼び出し側 `attrs` の `class` は他 styled 部品との一貫性の
/// ため [`drop_class_attr`] で除去する。[`crate::link_overlay::root`] と
/// 同型の判断）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::em::em;
///
/// let node = em(vec![], vec![text("important")]);
/// let html = render(&node);
/// assert!(html.starts_with("<em"));
/// assert!(html.contains("important"));
/// ```
#[must_use]
pub fn em<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("root", "em", drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_renders_em_tag_with_scope_and_part() {
        let html = render(&em(vec![], vec![text("important")]));
        assert_eq!(
            html,
            r#"<em data-scope="em" data-part="root">important</em>"#
        );
    }

    #[test]
    fn caller_class_attr_is_dropped() {
        let html = render(&em(vec![("class", "attacker-controlled")], vec![]));
        assert!(!html.contains("class="));
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        let html = render(&em(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn xss_payload_in_caller_attrs_is_escaped() {
        let html = render(&em(
            vec![("data-testid", "\"><script>alert(1)</script>")],
            vec![],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_italic_only() {
        let out = css();
        assert!(out.contains("font-style: italic;"));
        // イシュー #1433: 参考サイト（chakra-ui / Radix Themes）は em の
        // font-weight を継承させており、fandhe 側の medium 上書きは廃止
        // した。誤って復活していないことをこのテストで固定する。
        assert!(!out.contains("font-weight"));
    }

    #[test]
    fn css_output_is_deterministic() {
        assert_eq!(css(), css());
    }
}
