//! Quote（イシュー #995）: variant を持たない最小静的部品。短いインライン
//! 引用（`<q>`）を既定スタイルで組み立てる。[`crate::em`] / [`crate::link_overlay`]
//! と同型の「variant 軸を持たない slot recipe」パターンに従う。
//!
//! [`crate::blockquote`]（`<figure>`/`<blockquote>`/`<figcaption>` の
//! ブロック引用）との役割差: `quote` は文中に埋め込む短いインライン引用
//! （Radix Themes の Quote と同じ位置付け）であり、ブロックレベルの構造・
//! 出典表示（`caption` パーツ）を持たない。ブロック引用が必要な場合は
//! [`crate::blockquote`] を使う。
//!
//! ブラウザ既定の引用符（`q::before`/`q::after` の生成コンテンツ）は本
//! recipe が意図的に上書きしない（[`crate::recipe::SlotRecipe`] は宣言のみを
//! 持つため擬似要素規則は元々表現できない）。`font-style: italic` の 1 宣言
//! のみを base として持たせる。
//!
//! ## 参考サイト基準との 7 軸比較（イシュー #1440）
//!
//! chakra-ui / ark-ui / Radix Primitives には Quote 相当のインライン引用
//! 部品が存在せず（chakra は Blockquote のみを持つ）、実質の参照軸は
//! **Radix Themes の Quote 単独**になる。サイズ・バリアント・色・状態・
//! ダーク・フォーカス・余白 / hover / disabled / transition の 7 軸で
//! 比較した結果:
//!
//! - **サイズ・バリアント軸**: Radix Themes の Quote は `size`/`variant`
//!   prop を持たず、周囲のテキストからサイズを継承する。`quote` も軸を
//!   追加しない（Typography 周辺部品は size 軸を持たない、
//!   `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §4 (c)）。
//! - **font-style**: 両者とも italic を適用する。既存の `font-style: italic`
//!   宣言のみで一致しており、変更不要。
//! - **font-family（serif 差し替え）**: Radix Themes は Quote に serif 系
//!   font-family（他の Typography 部品と同じ意匠）を当てるが、**意図的に
//!   非採用**とする。理由は [`crate::em`] モジュール rustdoc の 7 軸比較
//!   （イシュー #1433）で確定した判断をそのまま踏襲する: (i) 参照可能な
//!   もう一方の軸（chakra-ui）が Quote 相当部品を持たず比較不能であり
//!   Radix 固有の意匠に追随する根拠が弱いこと、(ii) `theme.rs` の
//!   typography トークンは `font-body`（sans）のみで、`quote` 1 部品の
//!   ためだけに serif トークンを新設するとテーマ体系の一貫性を崩すこと、
//!   (iii) 和文本文（docs サイトを含む）で Times 系 fallback は表示品質が
//!   安定しないこと。
//! - **引用符グリフ（`quotes` プロパティ）**: Radix Themes はカーリー
//!   クォート（“…”）を明示スタイルとして固定しているように見えるが、
//!   これはブラウザ既定の `q::before`/`q::after` 生成コンテンツが `lang`
//!   属性（英語ページでは `en`）に応じて描画した結果であり、Quote 側が
//!   `quotes` プロパティで上書きしているわけではない。日本語コンテンツ
//!   （`lang="ja"` 相当）では同じブラウザ既定機構が和文の鉤括弧
//!   （「」）を描画する。`quotes` を英語カーリークォートへ固定
//!   すると和文コンテンツで不自然な表記になり国際利用（本フレーム
//!   ワークの前提）を損なうため、**意図的に非採用**とし本 recipe では
//!   `quotes` を宣言しない（ブラウザの `lang` 依存既定へ委ねる）。
//! - **色・ダーク**: 両者とも色宣言を持たず本文色を継承するため、
//!   ライト / ダークどちらでも自動的に本文へ追従する（一致）。
//! - **hover / disabled / transition / フォーカスリング**: `quote` は
//!   非インタラクティブな表示専用インライン要素であり、Radix Themes の
//!   Quote も同様にこれらの状態を持たない。本フレームワークでも
//!   hover / disabled / transition はインタラクティブ slot のみに適用
//!   する方針（`docs/design/pre-styled-ui-interaction-visual-language.md`
//!   §3）であり、フォーカスリングもフォーカス対象部品限定
//!   （`pre-styled-ui-focus-ring-and-size-conventions.md` §3）のため、
//!   `quote` には付与しない。
//! - **余白・角丸・影**: 両者とも持たない。`quote` も持たない。
//!
//! 以上より、CSS 出力（`font-style: italic` の 1 宣言のみ）は現状のまま
//! 変更不要と判定した。差分はすべて意図的非採用として本節に記録する。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="quote"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("quote");

/// Quote の recipe（scope `"quote"`、slot `"root"` のみ、variant 軸なし）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("quote", &["root"]).base("root", vec![decl("font-style", "italic")])
}

/// Quote の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Quote 1 個（`<q>`）を組み立てる。variant 軸を持たないため `class` 属性は
/// 付与しない（呼び出し側 `attrs` の `class` は他 styled 部品との一貫性の
/// ため [`drop_class_attr`] で除去する。[`crate::em::em`] と同型の判断）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::quote::quote;
///
/// let node = quote(vec![], vec![text("to be or not to be")]);
/// let html = render(&node);
/// assert!(html.starts_with("<q"));
/// assert!(html.contains("to be or not to be"));
/// ```
#[must_use]
pub fn quote<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("root", "q", drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_renders_q_tag_with_scope_and_part() {
        let html = render(&quote(vec![], vec![text("important")]));
        assert_eq!(
            html,
            r#"<q data-scope="quote" data-part="root">important</q>"#
        );
    }

    #[test]
    fn caller_class_attr_is_dropped() {
        let html = render(&quote(vec![("class", "attacker-controlled")], vec![]));
        assert!(!html.contains("class="));
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        let html = render(&quote(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn xss_payload_in_caller_attrs_is_escaped() {
        let html = render(&quote(
            vec![("data-testid", "\"><script>alert(1)</script>")],
            vec![],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_italic() {
        let out = css();
        assert!(out.contains("font-style: italic;"));
    }

    #[test]
    fn css_output_is_deterministic() {
        assert_eq!(css(), css());
    }
}
