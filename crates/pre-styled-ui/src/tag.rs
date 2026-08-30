//! Tag（イシュー #768）: slot recipe styled 部品。ラベル・分類・除去可能な
//! チップ表示のための root/label/close-trigger の 3 パーツで構成する。
//!
//! [`crate::badge`]（#550/#606）と同型の「pre-styled 層で anatomy を直接
//! 宣言する単純 styled 部品」として実装する。chakra-ui v3 の Tag anatomy
//! （Root/Label/StartElement/EndElement/CloseTrigger）のうち StartElement/
//! EndElement は専用パーツを設けない。children に任意 [`Node`] を並べれば
//! 同等の表現ができるため（スコープ外、イシュー #768 計画 §9 参照）。
//!
//! # close-trigger と dispatch 契約
//!
//! [`close_trigger`] は状態機械を持たない。`action` 引数を渡すと
//! `data-action` 属性を出力するのみで、実際のクリック処理・状態変更は
//! `fandhe-frontend-wasm-full` の `wire_events`
//! （`crates/wasm-full/src/events.rs::wire_events`）が
//! `closest("[data-action]")` で祖先探索して拾う既存契約に委ねる
//! （`docs/api/interactive-api.md` 決定 2/6 参照）。未知の `data-action`
//! 値は `Component::decode_action` が `None` を返し状態変更なしの no-op と
//! して吸収されるため（決定 6）、本パーツが新たな攻撃面を作ることはない。
//!
//! # `data-action` 語彙（イシュー #1063）
//!
//! `data-action` は `fandhe-frontend-headless-ui` の
//! `timer::action_trigger`（`crates/headless-ui/src/timer.rs`）も出力する、
//! 両層で共有される語彙である。意味論（「この要素をクリックしたときに
//! 発火する action の識別子」）は同一であり改名しない一方、値域は部品ごとに
//! 個別定義する（`docs/design/pre-styled-ui-data-attr-vocabulary.md` 規約
//! B-2）。本モジュールでは値域を固定せず呼び出し側が渡す任意文字列を
//! そのまま出力する（`timer` 側は `TimerControl::as_str` の 4 値固定と
//! 対照的）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{palette_scale_declarations, ColorPalette, Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="tag"` を固定した本コンポーネントの anatomy。scope `"tag"` は
/// 既存の `"tags-input"`（[`crate::tags_input`]）と衝突しない。
const ANATOMY: Anatomy = anatomy("tag");

/// Tag の見た目 variant（[`crate::badge::BadgeVariant`] と同型の 3 値。
/// chakra-ui の `surface` は最小サブセット方針により見送り）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TagVariant {
    /// 塗りつぶし。
    Solid,
    /// 淡色背景（既定）。
    #[default]
    Subtle,
    /// 輪郭のみ。
    Outline,
}

impl VariantValue for TagVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Subtle => "subtle",
            Self::Outline => "outline",
        }
    }
}

/// [`root`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct TagProps {
    /// 見た目 variant（既定 `Subtle`）。
    pub variant: TagVariant,
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
    /// colorPalette 軸（既定 `Accent`）。[`crate::theme`] のセマンティック色
    /// から選択する（[`crate::badge::BadgeProps::palette`] と同型）。
    pub palette: ColorPalette,
}

impl Default for TagProps {
    fn default() -> Self {
        TagProps {
            variant: TagVariant::Subtle,
            size: Size::Md,
            palette: ColorPalette::Accent,
        }
    }
}

/// Tag の recipe（scope `"tag"`、slot `"root"`/`"label"`/`"close-trigger"`）。
///
/// `root` のみが variant クラスを持つ（#708 統一方針: 子 slot へは
/// `[data-scope][data-part]` セレクタで base 宣言のみを当て、`class` は
/// 付与しない。[`crate::alert`] の非 root パーツと同型）。非 root パーツ
/// ([`label`]・[`close_trigger`]) も呼び出し側 `attrs` の `class` は
/// [`crate::class_attr::drop_class_attr`] で破棄する（[`crate::kbd::kbd`]・
/// [`crate::code::code`] と同じ一貫性契約）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("tag", &["root", "label", "close-trigger"])
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
            ],
        )
        .base(
            "label",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
            ],
        )
        .base(
            "close-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("border", "none"),
                decl("background", "transparent"),
                decl("cursor", "pointer"),
                decl("padding", "0"),
                decl("color", "inherit"),
            ],
        )
        // イシュー #1681: badge の recipe（`crate::badge::recipe`）と同一
        // 進行則（Xs は垂直 2 倍刻み・水平 0.125rem 刻みの外挿、Xl は水平
        // 0.125rem 刻みの外挿・font-size は Lg の 1 段上）。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("padding", "0.03125rem 0.25rem"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("padding", "0.0625rem 0.375rem"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("padding", "0.125rem 0.5rem"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("padding", "0.25rem 0.625rem"),
                decl("font-size", "var(--fandhe-font-font-size-md)"),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("padding", "0.5rem 0.75rem"),
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
            ],
        )
        .variant(
            TagVariant::Solid,
            "root",
            vec![
                decl("background", "var(--fandhe-palette)"),
                decl("color", "var(--fandhe-palette-fg)"),
            ],
        )
        .variant(
            TagVariant::Subtle,
            "root",
            vec![
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("color", "var(--fandhe-palette)"),
            ],
        )
        .variant(
            TagVariant::Outline,
            "root",
            vec![
                decl("background", "transparent"),
                decl("color", "var(--fandhe-palette)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(TagVariant::Subtle)
        .default_variant(ColorPalette::Accent);

    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
        ColorPalette::Neutral,
    ] {
        recipe = recipe.variant(palette, "root", palette_scale_declarations(palette));
    }
    recipe
}

/// Tag の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// root パーツを組み立てる。`variant`/`size`/`palette` に応じたクラスを
/// 付与する唯一のパーツ（`class_attr::drop_class_attr` により呼び出し側の
/// `class` は除去してから合成する。[`crate::badge::badge`] と同型の契約）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::tag::{self, TagProps};
///
/// let node = tag::root(&TagProps::default(), vec![], vec![text("beta")]);
/// assert!(render(&node).contains("beta"));
/// ```
#[must_use]
pub fn root<'a>(props: &TagProps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("variant", props.variant.value()),
        ("size", props.size.value()),
        ("color-palette", props.palette.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "span", merged, children)
}

/// label パーツ（`<span>`。テキストは `children` 経由で既定エスケープを
/// 貫通する）を組み立てる。呼び出し側 `attrs` の `class` は
/// [`crate::class_attr::drop_class_attr`] で破棄する（[`crate::kbd::kbd`]・
/// [`crate::code::code`] と同様、variant クラスを自ら付与しない非 root
/// パーツでも他部品との `class` 破棄契約を一貫させ、呼び出し側が誤って
/// 動的クラスを合成する余地を残さないため）。
#[must_use]
pub fn label<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("label", "span", drop_class_attr(attrs), children)
}

/// close-trigger パーツ（`<button type="button">`）を組み立てる。
///
/// `action` が `Some` のとき `data-action` 属性を出力する（本モジュール
/// 冒頭の dispatch 契約参照）。`None` のときは属性を出力せず、静的表示専用
/// （クリック不要）の Tag として使える。`aria-label`・視覚内容（`×` 等）は
/// [`crate::dialog::close_trigger`] 前例に従い呼び出し側が `attrs`/
/// `children` で付与すること（本関数は固定しない）。`data-payload` が
/// 必要な呼び出し側は `attrs` 経由で渡せる。呼び出し側 `attrs` の `class`
/// は [`crate::class_attr::drop_class_attr`] で破棄する（[`label`] と同様の
/// 一貫性理由）。
#[must_use]
pub fn close_trigger<'a>(
    action: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![("type", "button")];
    if let Some(action) = action {
        merged.push(("data-action", action));
    }
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("close-trigger", "button", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_props_render_subtle_md() {
        let node = root(&TagProps::default(), vec![], vec![text("beta")]);
        let html = render(&node);
        assert_eq!(
            html,
            r#"<span data-scope="tag" data-part="root" class="fd-tag--size-md fd-tag--variant-subtle fd-tag--color-palette-accent">beta</span>"#
        );
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (TagVariant::Solid, "fd-tag--variant-solid"),
            (TagVariant::Subtle, "fd-tag--variant-subtle"),
            (TagVariant::Outline, "fd-tag--variant-outline"),
        ] {
            let props = TagProps {
                variant,
                ..TagProps::default()
            };
            let html = render(&root(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-tag--size-md {class} fd-tag--color-palette-accent\""
                )),
                "variant={variant:?} -> {html}"
            );
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-tag--color-palette-accent"),
            (ColorPalette::Info, "fd-tag--color-palette-info"),
            (ColorPalette::Success, "fd-tag--color-palette-success"),
            (ColorPalette::Warning, "fd-tag--color-palette-warning"),
            (ColorPalette::Danger, "fd-tag--color-palette-danger"),
            (ColorPalette::Neutral, "fd-tag--color-palette-neutral"),
        ] {
            let props = TagProps {
                palette,
                ..TagProps::default()
            };
            let html = render(&root(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-tag--size-md fd-tag--variant-subtle {class}\""
                )),
                "palette={palette:?} -> {html}"
            );
        }
    }

    #[test]
    fn label_and_close_trigger_use_expected_tags_and_data_part() {
        assert!(render(&label(vec![], vec![text("x")]))
            .starts_with(r#"<span data-scope="tag" data-part="label">x"#));
        assert!(render(&close_trigger(None, vec![], vec![]))
            .starts_with(r#"<button data-scope="tag" data-part="close-trigger" type="button">"#));
    }

    #[test]
    fn close_trigger_emits_data_action_only_when_some() {
        let html_with_action = render(&close_trigger(Some("remove_tag"), vec![], vec![]));
        assert!(html_with_action.contains(r#"data-action="remove_tag""#));

        let html_without_action = render(&close_trigger(None, vec![], vec![]));
        assert!(!html_without_action.contains("data-action"));
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&root(
            &TagProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn label_and_close_trigger_caller_class_attr_is_dropped() {
        let label_html = render(&label(vec![("class", "attacker-controlled")], vec![]));
        assert!(!label_html.contains("class="));
        assert!(!label_html.contains("attacker-controlled"));

        let close_trigger_html = render(&close_trigger(
            None,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert!(!close_trigger_html.contains("class="));
        assert!(!close_trigger_html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        let html = render(&root(
            &TagProps::default(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_radius_token() {
        let out = css();
        assert!(out.contains("border-radius: var(--fandhe-radius-sm);"));
    }
}
