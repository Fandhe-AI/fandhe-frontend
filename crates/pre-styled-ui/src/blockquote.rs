//! Blockquote（イシュー #771）: slot recipe styled 部品。root（`<figure>`）/
//! content（`<blockquote>`）/ caption（`<figcaption>`）の 3 パーツで構成する
//! 引用ブロック（[`crate::card`] と同型のパーツ関数群パターン）。
//!
//! anatomy 上 content パーツが素の `<blockquote>` 要素になることで、
//! 引用テキストの HTML 意味論（スクリーンリーダー・検索エンジンが引用として
//! 認識する）をそのまま保つ（本イシュー冒頭「素の HTML 意味論をそのまま
//! styled 化する」方針）。
//!
//! 文中に埋め込む短いインライン引用（`<q>`）は [`crate::quote`] が担う
//! （イシュー #995）。`blockquote` はブロックレベルの構造・出典表示
//! （`caption` パーツ）を持つ点で `quote` と役割が異なる。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{palette_declarations, ColorPalette, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="blockquote"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("blockquote");

/// [`SlotRecipe::new`] に渡す slot 一覧（recipe とレンダリング関数の両方が
/// この配列を共有し、slot 名の乖離を防ぐ、[`crate::card`] 前例と同型）。
const SLOTS: &[&str] = &["root", "content", "caption"];

/// Blockquote の見た目 variant。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlockquoteVariant {
    /// 淡色背景 + アクセント色の左罫線（既定）。
    #[default]
    Subtle,
    /// 塗りつぶし。
    Solid,
    /// 罫線のみ・背景なし。
    Plain,
}

impl VariantValue for BlockquoteVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Subtle => "subtle",
            Self::Solid => "solid",
            Self::Plain => "plain",
        }
    }
}

/// Blockquote の recipe（scope `"blockquote"`、[`SLOTS`] の 3 パーツ）。
///
/// variant/colorPalette クラスは `root` のみへ付与する（複合部品の variant
/// 統一方針、`crate` 冒頭 rustdoc「クラスは root slot のみに付与する」）。
/// 装飾（背景・罫線色）も `root` 自身が直接宣言する（子孫セレクタ機構は
/// 追加しない）。
///
/// `caption` の文字色は `--fandhe-blockquote-caption-fg`（既定
/// `var(--fandhe-color-fg-muted)`）というローカル custom property 経由で
/// 参照する（[`palette_declarations`] と同型のパターン）。custom property は
/// クラスの有無に関わらず DOM の継承で子要素（`caption` は `root` の子）へ
/// 伝わるため、`caption` 自身にクラスを付けなくても `root` 側の variant
/// 宣言で上書きできる。`Solid` variant は `root` の背景を `--fandhe-palette`
/// で塗る（`caption` はその子孫）ため、`--fandhe-blockquote-caption-fg` を
/// `var(--fandhe-palette-fg)` へ上書きし、muted な前景色が solid 背景の上で
/// コントラスト不足になることを防ぐ（Bugbot 指摘）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("blockquote", SLOTS)
        .base(
            "root",
            vec![
                decl("margin", "0"),
                decl("padding-inline-start", "1rem"),
                decl("padding-block", "0.5rem"),
                decl(
                    "--fandhe-blockquote-caption-fg",
                    "var(--fandhe-color-fg-muted)",
                ),
            ],
        )
        .base("content", vec![decl("margin", "0")])
        .base(
            "caption",
            vec![
                decl("display", "block"),
                decl("margin-block-start", "0.5rem"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("color", "var(--fandhe-blockquote-caption-fg)"),
            ],
        )
        .variant(
            BlockquoteVariant::Subtle,
            "root",
            vec![
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("border-inline-start", "4px solid var(--fandhe-palette)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
            ],
        )
        .variant(
            BlockquoteVariant::Solid,
            "root",
            vec![
                decl("background", "var(--fandhe-palette)"),
                decl("color", "var(--fandhe-palette-fg)"),
                decl(
                    "border-inline-start",
                    "4px solid var(--fandhe-palette-emphasized)",
                ),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("--fandhe-blockquote-caption-fg", "var(--fandhe-palette-fg)"),
            ],
        )
        .variant(
            BlockquoteVariant::Plain,
            "root",
            vec![
                decl("background", "transparent"),
                decl("border-inline-start", "4px solid var(--fandhe-palette)"),
            ],
        )
        .default_variant(BlockquoteVariant::Subtle)
        .default_variant(ColorPalette::Accent);

    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
    ] {
        recipe = recipe.variant(palette, "root", palette_declarations(palette));
    }
    recipe
}

/// Blockquote の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// root パーツ（`<figure>`）を組み立てる。`variant`/`palette` に応じたクラスを
/// 付与する唯一のパーツ。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
/// use fandhe_frontend_pre_styled_ui::ColorPalette;
///
/// let node = blockquote::root(BlockquoteVariant::default(), ColorPalette::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="blockquote" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    variant: BlockquoteVariant,
    palette: ColorPalette,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("variant", variant.value()),
        ("color-palette", palette.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "figure", merged, children)
}

/// content パーツ（`<blockquote>`）を組み立てる。呼び出し側 `attrs` の
/// `cite`（引用元 URL）はそのまま透過する（既定の属性エスケープ・URL
/// 属性検証は `fandhe_frontend_core::render` 側の既存責務）。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("content", "blockquote", attrs, children)
}

/// caption パーツ（`<figcaption>`）を組み立てる。
#[must_use]
pub fn caption<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("caption", "figcaption", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_variant_is_subtle_accent() {
        let html = render(&root(
            BlockquoteVariant::default(),
            ColorPalette::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-blockquote--variant-subtle"));
        assert!(html.contains("fd-blockquote--color-palette-accent"));
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (BlockquoteVariant::Subtle, "fd-blockquote--variant-subtle"),
            (BlockquoteVariant::Solid, "fd-blockquote--variant-solid"),
            (BlockquoteVariant::Plain, "fd-blockquote--variant-plain"),
        ] {
            let html = render(&root(variant, ColorPalette::default(), vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"{class} fd-blockquote--color-palette-accent\""
                )),
                "variant={variant:?} -> {html}"
            );
        }
    }

    #[test]
    fn parts_use_expected_tags_and_data_part() {
        assert!(render(&content(vec![], vec![]))
            .starts_with(r#"<blockquote data-scope="blockquote" data-part="content""#));
        assert!(render(&caption(vec![], vec![]))
            .starts_with(r#"<figcaption data-scope="blockquote" data-part="caption""#));
    }

    #[test]
    fn composed_blockquote_snapshot() {
        let node = root(
            BlockquoteVariant::Subtle,
            ColorPalette::Accent,
            vec![],
            vec![
                content(vec![("cite", "https://example.com")], vec![text("Quote")]),
                caption(vec![], vec![text("— Author")]),
            ],
        );
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<figure data-scope="blockquote" data-part="root" class="fd-blockquote--variant-subtle fd-blockquote--color-palette-accent">"#,
                r#"<blockquote data-scope="blockquote" data-part="content" cite="https://example.com">Quote</blockquote>"#,
                r#"<figcaption data-scope="blockquote" data-part="caption">— Author</figcaption>"#,
                r#"</figure>"#,
            )
        );
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let html = render(&root(
            BlockquoteVariant::default(),
            ColorPalette::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_content_children_is_escaped() {
        let html = render(&content(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_radius_and_border_inline_start() {
        let out = css();
        assert!(out.contains("border-radius: var(--fandhe-radius-sm);"));
        assert!(out.contains("border-inline-start: 4px solid var(--fandhe-palette);"));
    }

    /// Solid variant では `caption` の文字色が muted 固定ではなく
    /// `--fandhe-palette-fg`（solid 背景の上で読める前景色）へ切り替わる
    /// ことを確認する（Bugbot 指摘の contrast 不足の回帰防止）。
    #[test]
    fn solid_variant_overrides_caption_color_for_contrast() {
        let out = css();
        assert!(out.contains(
            "[data-part=\"caption\"] {\n  display: block;\n  margin-block-start: 0.5rem;\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-blockquote-caption-fg);\n}"
        ));
        assert!(out.contains(
            "[data-part=\"root\"].fd-blockquote--variant-solid {\n  background: var(--fandhe-palette);\n  color: var(--fandhe-palette-fg);\n  border-inline-start: 4px solid var(--fandhe-palette-emphasized);\n  border-radius: var(--fandhe-radius-sm);\n  --fandhe-blockquote-caption-fg: var(--fandhe-palette-fg);\n}"
        ));
    }
}
