//! Text（イシュー #771）: 単一 recipe styled 部品。段落テキスト（`<p>`）を
//! `size` variant 付きで組み立てる。[`crate::heading`] と対になる本文向け
//! 静的部品（headless 状態機械を要しない、badge/skeleton と同型）。
//!
//! # 呼び出し側の名前衝突に関する注意
//!
//! この関数名 [`text`] は [`fandhe_frontend_core::text`]（テキストノード
//! 生成関数）と同名である。両方を同一スコープへ `use` する場合は呼び出し側
//! でモジュールパスを使う（`text::text(...)`）か `use ... as` でどちらかを
//! 別名にする必要がある。本クレートの他モジュール（例:
//! `fandhe_frontend_pre_styled_ui::input`）も `fandhe_frontend_core::text`
//! （子ノードのテキスト）と styled 部品を同時に使うため、この衝突は本部品
//! 固有の問題ではない。
//!
//! # colorPalette 軸を持たない理由
//!
//! [`crate::heading`] と同じ（中立部品、前景色トークンを継承する）。
//!
//! # prose（記事全体カスケード）とこの部品群の役割分担（イシュー #771）
//!
//! chakra-ui の `Prose`（記事全体へ一括カスケード適用するコンポーネント）
//! に相当する機構は、本クレート（`fandhe-frontend-pre-styled-ui`）へは
//! 導入しない。[`mod@heading`]/[`mod@text`]/[`mod@em`]/[`mod@mark`]/
//! [`mod@blockquote`]/[`mod@list`] はいずれも「要素単位のオプトイン適用」
//! （呼び出し側が明示的に呼んだ要素にのみ既定スタイルが付く）であり、
//! Markdown 由来の記事本文へ無選別にカスケード適用する仕組みは持たない。
//!
//! 記事全体へのカスケードスタイルは `fandhe-frontend-docs-site` の
//! `site/assets/site.css`（`.docs-content` 配下の `h1`-`h3`/`p`/`ul`/`ol`/
//! `blockquote` 規則）が既に担っており、本イシューはこの既存機構を置き換え
//! ない（`docs/design/component-coverage-map.md` prose.md 行の「対象外」
//! 区分の根拠）。両者の役割は次のように分かれる:
//!
//! - `site.css` の `.docs-content` 規則: Markdown から生成した記事本文全体へ
//!   無条件にカスケード適用する、docs サイト固有の骨格スタイル
//! - 本モジュール群: 呼び出し側が Rust コードで明示的に組み立てる UI 部品
//!   として、要素単位で個別に呼び出して使う styled 部品
//!
//! 将来 `fandhe-frontend-pre-styled-ui` 側にも prose 相当の一括適用機構を
//! 追加する場合は、`site.css` との重複適用・詳細度の衝突を新たな設計課題
//! として扱う必要があり、本イシューのスコープには含めない。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="text"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("text");

/// Text の視覚サイズ variant（`font-size`/`line-height`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextSize {
    /// 極小サイズ。
    Xs,
    /// 小サイズ。
    Sm,
    /// 中サイズ（既定）。
    #[default]
    Md,
    /// 大サイズ。
    Lg,
    /// 特大サイズ。
    Xl,
}

impl VariantValue for TextSize {
    fn axis(self) -> &'static str {
        "size"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
        }
    }
}

/// [`text`] の設定。
#[derive(Debug, Clone, Copy, Default)]
pub struct TextProps {
    /// 視覚サイズ variant（既定 `Md`）。
    pub size: TextSize,
}

/// Text の recipe（scope `"text"`、slot `"root"` のみ、`<p>` 固定）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("text", &["root"])
        .base("root", vec![decl("margin", "0")])
        .variant(
            TextSize::Xs,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("line-height", "1.4"),
            ],
        )
        .variant(
            TextSize::Sm,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("line-height", "1.45"),
            ],
        )
        .variant(
            TextSize::Md,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-md)"),
                decl("line-height", "1.5"),
            ],
        )
        .variant(
            TextSize::Lg,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
                decl("line-height", "1.5"),
            ],
        )
        .variant(
            TextSize::Xl,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-xl)"),
                decl("line-height", "1.55"),
            ],
        )
        .default_variant(TextSize::Md)
}

/// Text の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Text 1 個（`<p>`）を組み立てる。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text as core_text};
/// use fandhe_frontend_pre_styled_ui::text::{text, TextProps};
///
/// let node = text(&TextProps::default(), vec![], vec![core_text("Body copy")]);
/// let html = render(&node);
/// assert!(html.starts_with("<p"));
/// assert!(html.contains("Body copy"));
/// ```
#[must_use]
pub fn text<'a>(props: &TextProps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", props.size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "p", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text as core_text};

    #[test]
    fn default_props_render_p_md_size() {
        let html = render(&text(
            &TextProps::default(),
            vec![],
            vec![core_text("Body")],
        ));
        assert_eq!(
            html,
            r#"<p data-scope="text" data-part="root" class="fd-text--size-md">Body</p>"#
        );
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (TextSize::Xs, "fd-text--size-xs"),
            (TextSize::Sm, "fd-text--size-sm"),
            (TextSize::Md, "fd-text--size-md"),
            (TextSize::Lg, "fd-text--size-lg"),
            (TextSize::Xl, "fd-text--size-xl"),
        ] {
            let props = TextProps { size };
            let html = render(&text(&props, vec![], vec![]));
            assert!(
                html.contains(&format!("class=\"{class}\"")),
                "size={size:?} -> {html}"
            );
        }
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&text(
            &TextProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        let html = render(&text(
            &TextProps::default(),
            vec![],
            vec![core_text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_all_size_font_tokens() {
        let out = css();
        for token in [
            "font-size-xs",
            "font-size-sm",
            "font-size-md",
            "font-size-lg",
            "font-size-xl",
        ] {
            assert!(
                out.contains(&format!("var(--fandhe-font-{token})")),
                "missing {token} in {out}"
            );
        }
    }
}
