//! Separator（イシュー #772）: 単一 recipe styled 静的部品。区切り線を
//! `<hr>` として組み立てる。badge/spinner/skeleton（イシュー #550/#764）と
//! 同型の、headless 状態機械を要しない静的部品
//! （`docs/design/component-coverage-map.md` separator 行）。
//!
//! # マークアップ・aria 出力方針（受け入れ条件 1）
//!
//! root は子ノードを持たない `<hr>` 1 個のみで構成する。
//! `crates/headless-ui/src/menu.rs::separator`（メニュー内区切り線）が
//! `<hr>` に `role="separator"` + `aria-orientation` を固定付与する前例に
//! 倣い、本モジュールの [`separator`] も同じ 2 属性を常時出力する。
//! 加えて headless 層の data-* 語彙（[`data_orientation`]）を共用し、
//! `data-orientation="horizontal"|"vertical"` も常時出力する（tabs/accordion
//! 等の複合部品が `data-orientation` を CSS セレクタとして使う既存の不変
//! 条件を、状態機械を持たない静的部品にも一貫させる）。
//!
//! `role`/`aria-orientation`/`data-orientation`/`class` はいずれも
//! コンポーネント側が決定する契約属性であり、呼び出し側 `attrs` に同名の
//! キー（大文字小文字を無視）が含まれていても除去してから合成する
//! （[`crate::skeleton`] が `aria-hidden` を除去する判断と同型の fail-closed
//! 方針。契約属性の偽装を許すと、支援技術・CSS セレクタの双方が誤った
//! 状態を読み取ってしまう）。
//!
//! # variant 軸（`variant`/`orientation` の 2 軸のみ）
//!
//! chakra-ui Separator の `size`（罫線太さ）・`colorPalette` 軸は提供しない。
//! 区切り線は中立的な罫線でありステータス色を持たない
//! （[`crate::card`]/[`crate::skeleton`] が「中立コンテナ／装飾的占位要素の
//! ため colorPalette 軸を付与しない」とした判断と同じ整理）。太さは
//! `variant`（`solid`/`dashed`）と `orientation`（`horizontal`/`vertical`）の
//! 直交する 2 軸のみを受け入れ条件どおりに提供する。
//!
//! `orientation` の型は headless 層の
//! [`fandhe_frontend_headless_ui::data_attrs::Orientation`] をそのまま
//! 再利用する（[`crate::tabs`] が
//! `pub use fandhe_frontend_headless_ui::data_attrs::Orientation;` する
//! のと同じ判断）。ただし `Orientation` へ直接 [`VariantValue`] を実装すると
//! 他の複合部品の recipe（tabs/accordion 等、いずれ `orientation` 軸を
//! 持ちうる）に本モジュール固有の axis 名解釈が意図せず波及する懸念がある
//! ため、本モジュール内限定のニュータイプ [`OrientationAxis`] を介して
//! `VariantValue` を実装する。
//!
//! # 縦方向の高さについて
//!
//! chakra-ui と同じく、縦方向の区切り線は自身では高さを決定できない
//! （親コンテナのレイアウトに依存する）。`--fandhe-separator-height` を
//! フォールバック付き custom property として公開し、呼び出し側が必要に
//! 応じて上書きする前提とする（[`crate::skeleton`] の
//! `--fandhe-skeleton-size` と同型のパターン）。
//!
//! # Examples
//!
//! ```
//! use fandhe_frontend_core::render;
//! use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
//!
//! let html = render(&separator(&SeparatorProps::default(), vec![]));
//! assert!(html.contains(r#"role="separator""#));
//! assert!(html.contains(r#"aria-orientation="horizontal""#));
//! ```

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::data_attrs::{data_orientation, Orientation};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, aria_orientation, role, Anatomy};

/// `data-scope="separator"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("separator");

/// Separator の罫線 variant。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SeparatorVariant {
    /// 実線（既定）。
    #[default]
    Solid,
    /// 破線。
    Dashed,
}

impl VariantValue for SeparatorVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Dashed => "dashed",
        }
    }
}

/// [`Orientation`] を recipe の `orientation` 軸として登録するための
/// モジュール内限定ニュータイプ（モジュール冒頭 rustdoc「`orientation` 軸」
/// 節参照。他モジュールの recipe への意図しない axis 波及を避けるため
/// `Orientation` 自体へは実装しない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OrientationAxis(Orientation);

impl VariantValue for OrientationAxis {
    fn axis(self) -> &'static str {
        "orientation"
    }

    fn value(self) -> &'static str {
        self.0.as_str()
    }
}

/// [`separator`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct SeparatorProps {
    /// 向き（既定 `Horizontal`）。`aria-orientation`/`data-orientation`/
    /// variant クラスの 3 箇所へ連動する。
    pub orientation: Orientation,
    /// 罫線 variant（既定 `Solid`）。
    pub variant: SeparatorVariant,
}

impl Default for SeparatorProps {
    fn default() -> Self {
        Self {
            orientation: Orientation::Horizontal,
            variant: SeparatorVariant::Solid,
        }
    }
}

/// Separator の recipe（scope `"separator"`、slot `"root"` のみ）。
///
/// `orientation` 軸を `variant` 軸より先に登録し、[`SlotRecipe::variant_classes`]
/// が返すクラス文字列の並び（axis 登録順）を
/// `fd-separator--orientation-* fd-separator--variant-*` に固定する
/// （[`separator`] のユニットテストが全文一致で検証する）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("separator", &["root"])
        .base(
            "root",
            vec![
                decl("border-width", "0"),
                decl("border-color", "var(--fandhe-color-border)"),
                decl("margin", "0"),
                decl("flex-shrink", "0"),
            ],
        )
        .variant(
            OrientationAxis(Orientation::Horizontal),
            "root",
            vec![decl("border-top-width", "1px"), decl("width", "100%")],
        )
        .variant(
            OrientationAxis(Orientation::Vertical),
            "root",
            vec![
                decl("border-inline-start-width", "1px"),
                decl("align-self", "stretch"),
                decl("height", "var(--fandhe-separator-height, auto)"),
            ],
        )
        .default_variant(OrientationAxis(Orientation::Horizontal))
        .variant(
            SeparatorVariant::Solid,
            "root",
            vec![decl("border-style", "solid")],
        )
        .variant(
            SeparatorVariant::Dashed,
            "root",
            vec![decl("border-style", "dashed")],
        )
        .default_variant(SeparatorVariant::Solid)
}

/// Separator の静的 CSS 全文。recipe が生成する規則群のみで完結する
/// （skeleton のような `@keyframes` 追記はない、badge/spinner と同型）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Separator 1 個を組み立てる。
///
/// 子ノードを取らない（区切り線は実コンテンツを持たない）。呼び出し側
/// `attrs` に含まれる `class`/`role`/`aria-orientation`/`data-orientation`
/// は大文字小文字を無視して除去してから合成する（モジュール冒頭 rustdoc
/// 「マークアップ・aria 出力方針」節参照。契約属性の偽装を許さない
/// fail-closed 判断）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_headless_ui::data_attrs::Orientation;
/// use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
///
/// let vertical = SeparatorProps {
///     orientation: Orientation::Vertical,
///     ..SeparatorProps::default()
/// };
/// let html = render(&separator(&vertical, vec![]));
/// assert!(html.contains(r#"aria-orientation="vertical""#));
/// assert!(html.contains(r#"data-orientation="vertical""#));
/// ```
#[must_use]
pub fn separator<'a>(props: &SeparatorProps, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("orientation", OrientationAxis(props.orientation).value()),
        ("variant", props.variant.value()),
    ]);
    let contract_keys = ["role", "aria-orientation", "data-orientation"];
    let attrs: Vec<(&str, &str)> = drop_class_attr(attrs)
        .into_iter()
        .filter(|(k, _)| !contract_keys.iter().any(|c| k.eq_ignore_ascii_case(c)))
        .collect();
    let mut merged: Vec<(&str, &str)> = vec![
        ("class", class.as_str()),
        role("separator"),
        aria_orientation(props.orientation),
        data_orientation(props.orientation),
    ];
    merged.extend(attrs);
    ANATOMY.part("root", "hr", merged, vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn default_props_render_expected_markup() {
        let html = render(&separator(&SeparatorProps::default(), vec![]));
        assert_eq!(
            html,
            r#"<hr data-scope="separator" data-part="root" class="fd-separator--orientation-horizontal fd-separator--variant-solid" role="separator" aria-orientation="horizontal" data-orientation="horizontal">"#
        );
    }

    #[test]
    fn orientation_enumeration_maps_to_expected_aria_and_data_attrs() {
        for (orientation, class_fragment, value) in [
            (
                Orientation::Horizontal,
                "fd-separator--orientation-horizontal",
                "horizontal",
            ),
            (
                Orientation::Vertical,
                "fd-separator--orientation-vertical",
                "vertical",
            ),
        ] {
            let props = SeparatorProps {
                orientation,
                ..SeparatorProps::default()
            };
            let html = render(&separator(&props, vec![]));
            assert!(
                html.contains(class_fragment),
                "orientation={orientation:?} -> {html}"
            );
            assert!(
                html.contains(&format!(r#"aria-orientation="{value}""#)),
                "orientation={orientation:?} -> {html}"
            );
            assert!(
                html.contains(&format!(r#"data-orientation="{value}""#)),
                "orientation={orientation:?} -> {html}"
            );
        }
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class_fragment) in [
            (SeparatorVariant::Solid, "fd-separator--variant-solid"),
            (SeparatorVariant::Dashed, "fd-separator--variant-dashed"),
        ] {
            let props = SeparatorProps {
                variant,
                ..SeparatorProps::default()
            };
            let html = render(&separator(&props, vec![]));
            assert!(
                html.contains(class_fragment),
                "variant={variant:?} -> {html}"
            );
        }
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&separator(
            &SeparatorProps::default(),
            vec![("class", "attacker-controlled")],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    /// 契約属性（`role`/`aria-orientation`/`data-orientation`）の偽装を
    /// 大文字小文字を無視して除去し、常にコンポーネント側の正値のみを
    /// 出力することの回帰テスト（[`crate::skeleton`] の `aria-hidden` 除去
    /// テストと同型）。
    #[test]
    fn caller_supplied_contract_attrs_are_dropped_case_insensitively() {
        for (key, spoofed) in [
            ("role", "not-a-separator"),
            ("Role", "not-a-separator"),
            ("aria-orientation", "diagonal"),
            ("ARIA-ORIENTATION", "diagonal"),
            ("data-orientation", "diagonal"),
            ("Data-Orientation", "diagonal"),
        ] {
            let html = render(&separator(&SeparatorProps::default(), vec![(key, spoofed)]));
            assert!(!html.contains(spoofed), "key={key} html={html}");
            assert_eq!(html.matches("role=").count(), 1, "key={key} html={html}");
            assert_eq!(
                html.matches("aria-orientation=").count(),
                1,
                "key={key} html={html}"
            );
            assert_eq!(
                html.matches("data-orientation=").count(),
                1,
                "key={key} html={html}"
            );
        }
    }

    #[test]
    fn xss_payload_in_caller_attrs_is_escaped() {
        let html = render(&separator(
            &SeparatorProps::default(),
            vec![("data-testid", "\"><script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn css_output_is_deterministic_and_non_empty() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="separator"][data-part="root"]"#));
    }

    #[test]
    fn css_output_declares_orientation_and_variant_rules() {
        let out = css();
        assert!(out.contains("border-top-width: 1px;"));
        assert!(out.contains("border-inline-start-width: 1px;"));
        assert!(out.contains("border-style: solid;"));
        assert!(out.contains("border-style: dashed;"));
    }
}
