//! ColorSwatch（イシュー #838、親 #837）: 色見本の静的表示 slot recipe styled
//! 部品。chakra-ui `forms/color-swatch.md` 相当の単一 root パーツで構成する。
//!
//! [`crate::tag`]/[`crate::kbd`]（#768）と同型の「pre-styled 層で anatomy を
//! 直接宣言する単純 styled 部品」として実装する。headless 層には対応する
//! anatomy を新設しない（`docs/design/component-coverage-map.md` の
//! ColorSwatch 行は headless 列が「—」のまま実装済みへ更新する）。canvas は
//! 使わず、CSS `background-image`（色レイヤー + 透過色の視認用チェッカー
//! ボード模様の 2 レイヤー、詳細は [`recipe`] 参照）のみで見た目を組み立てる。
//!
//! # 色値は検証済み型経由のみ（セキュリティ不変条件）
//!
//! [`ColorSwatchProps::value`] は
//! [`fandhe_frontend_headless_ui::color::Color`] 型のみを受け取り、任意
//! 文字列（`&str`）を受け取る API は持たない。`style` 属性へ到達する動的値は
//! [`fandhe_frontend_headless_ui::color::Color::to_hex_string`] の出力
//! （常に `#` + 小文字 16 進数字に閉じる、`crates/headless-ui/src/color.rs`
//! 冒頭 rustdoc の不変条件）のみであり、CSS インジェクション・属性破りの
//! 経路を構造的に持たない。呼び出し側 `attrs` の `class`/`style`
//! （大文字小文字非依存）はフレームワーク生成値を上書きされないよう破棄
//! してから合成する（[`crate::tag`]・[`crate::slider`] と同型の契約）。
//!
//! # 最小サブセット方針（chakra-ui との差分）
//!
//! chakra-ui の ColorSwatch は `size` に `2xs`〜`2xl`/`full` の 8 段階を
//! 持つが、本実装は既存共通軸 [`crate::recipe::Size`]（Sm/Md/Lg）の 3 段階
//! に限定する（[`crate::tag::TagVariant`] が chakra `surface` を見送ったのと
//! 同型の最小サブセット判断、スコープ外）。`shape` は chakra 同様
//! `square`/`circle`/`rounded`（既定）の 3 値を提供する。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

// `Color`/`Rgb` を再エクスポートし、`fandhe-frontend-pre-styled-ui` のみに
// 依存する利用者・showcase が headless-ui への直接依存なしに
// `ColorSwatchProps::value` を構築できるようにする（#685 の再エクスポート
// 方針に整合）。
pub use fandhe_frontend_headless_ui::color::{Color, Rgb};

/// `data-scope="color-swatch"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("color-swatch");

/// ColorSwatch の外形（chakra-ui の `shape` prop、既定 `Rounded`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SwatchShape {
    /// 角丸なしの正方形。
    Square,
    /// 完全な円形。
    Circle,
    /// 小さめの角丸（既定）。
    #[default]
    Rounded,
}

impl VariantValue for SwatchShape {
    fn axis(self) -> &'static str {
        "shape"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Square => "square",
            Self::Circle => "circle",
            Self::Rounded => "rounded",
        }
    }
}

/// [`color_swatch`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct ColorSwatchProps {
    /// 表示する色（検証済み [`Color`] 型のみを受け取る、モジュール冒頭
    /// 「色値は検証済み型経由のみ」参照）。
    pub value: Color,
    /// サイズ variant（既定 `Md`）。chakra-ui の 8 段階に対する最小
    /// サブセット（モジュール冒頭「最小サブセット方針」参照）。
    pub size: Size,
    /// 外形 variant（既定 `Rounded`）。
    pub shape: SwatchShape,
}

impl Default for ColorSwatchProps {
    /// `value` は不透明の黒を既定値とする（意味のある既定色は存在しないため、
    /// `..Default::default()` での部分上書き利用を想定した placeholder）。
    fn default() -> Self {
        Self {
            value: Color::from_rgb(Rgb::new(0, 0, 0)),
            size: Size::Md,
            shape: SwatchShape::Rounded,
        }
    }
}

/// 呼び出し側 `attrs` から `class`/`style`（いずれも ASCII 大文字小文字を
/// 無視）を除いた列を返す（[`color_swatch`] がフレームワーク側で両属性を
/// 組み立てた後、呼び出し側 `attrs` を連結する前に使う dedup ヘルパ。
/// `crates/pre-styled-ui/src/slider.rs::drop_style_attr` と同型の判断）。
fn drop_class_and_style_attr<'a>(attrs: Vec<(&'a str, &'a str)>) -> Vec<(&'a str, &'a str)> {
    drop_class_attr(attrs)
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("style"))
        .collect()
}

/// この ColorSwatch の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみが呼ぶ）。
///
/// 透過色の視認用に固定のチェッカーボード模様（`repeating-conic-gradient`）
/// を敷き、その上に呼び出し側の色（CSS カスタムプロパティ
/// `--fd-swatch-color` 経由）を重ねる。CSS の `background-color` は常に
/// `background-image` の下に描画される（レイヤ順は逆転できない）ため、色を
/// `background-color` にすると不透明色でもチェッカーが常に透けて見えてしまう
/// （Bugbot 指摘、イシュー #838 PR #858）。そのため色自体も
/// `background-image` の最前面レイヤー（`linear-gradient(color, color)`）
/// として表現し、チェッカーボードをその背後の第 2 レイヤーに置く。不透明色
/// は前面レイヤーが完全に覆い隠し、半透明色は前面レイヤーを透かしてチェッカー
/// が見える（宣言はすべて静的定数、決定的）。
///
/// チェッカーボード自体の 2 色目は `transparent` ではなく固定トークン
/// `--fandhe-color-bg` を使う。`transparent` にすると親要素の実際の背景色が
/// そのまま透けて見えるため、ダーク/カラー背景上ではチェッカーの
/// コントラストが失われコンポーネント単体でのプレビューが破綻する
/// （Bugbot 指摘, イシュー #838 PR #858）。`--fandhe-color-bg` は
/// `--fandhe-color-border` と常にコントラストが取れる固定トークンであり、
/// 周囲のレイアウトに依存せずチェッカーの視認性を保証する。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("color-swatch", &["root"])
        .base(
            "root",
            vec![
                decl("display", "inline-block"),
                decl("vertical-align", "middle"),
                decl(
                    "background-image",
                    "linear-gradient(var(--fd-swatch-color), var(--fd-swatch-color)), repeating-conic-gradient(var(--fandhe-color-border) 0% 25%, var(--fandhe-color-bg) 0% 50%)",
                ),
                decl("background-size", "100% 100%, 8px 8px"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl("width", "1rem"), decl("height", "1rem")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("width", "1.5rem"), decl("height", "1.5rem")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("width", "2rem"), decl("height", "2rem")],
        )
        .variant(
            SwatchShape::Square,
            "root",
            vec![decl("border-radius", "0")],
        )
        .variant(
            SwatchShape::Circle,
            "root",
            vec![decl("border-radius", "9999px")],
        )
        .variant(
            SwatchShape::Rounded,
            "root",
            vec![decl("border-radius", "var(--fandhe-radius-sm)")],
        )
        .default_variant(Size::Md)
        .default_variant(SwatchShape::Rounded)
}

/// ColorSwatch の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// root パーツ（`<span>`）を組み立てる。`size`/`shape` に応じたクラスを
/// 付与し、`value` から導出した `style`（`--fd-swatch-color` custom
/// property）を出力する唯一のパーツ。呼び出し側 `attrs` の `class`/`style`
/// は [`drop_class_and_style_attr`] により除去してから合成する
/// （モジュール冒頭「色値は検証済み型経由のみ」参照）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::color_swatch::{self, Color, ColorSwatchProps, Rgb};
///
/// let props = ColorSwatchProps {
///     value: Color::from_rgb(Rgb::new(0x3b, 0x82, 0xf6)),
///     ..ColorSwatchProps::default()
/// };
/// let node = color_swatch::color_swatch(&props, vec![], vec![]);
/// assert!(render(&node).contains("--fd-swatch-color: #3b82f6"));
/// ```
#[must_use]
pub fn color_swatch<'a>(
    props: &ColorSwatchProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", props.size.value()), ("shape", props.shape.value())]);
    let style = format!("--fd-swatch-color: {}", props.value.to_hex_string());
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str()), ("style", style.as_str())];
    merged.extend(drop_class_and_style_attr(attrs));
    ANATOMY.part("root", "span", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn opaque_blue() -> Color {
        Color::from_rgb(Rgb::new(0x3b, 0x82, 0xf6))
    }

    #[test]
    fn default_props_render_md_rounded() {
        let props = ColorSwatchProps {
            value: opaque_blue(),
            ..ColorSwatchProps::default()
        };
        let html = render(&color_swatch(&props, vec![], vec![]));
        assert_eq!(
            html,
            r#"<span data-scope="color-swatch" data-part="root" class="fd-color-swatch--size-md fd-color-swatch--shape-rounded" style="--fd-swatch-color: #3b82f6"></span>"#
        );
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Sm, "fd-color-swatch--size-sm"),
            (Size::Md, "fd-color-swatch--size-md"),
            (Size::Lg, "fd-color-swatch--size-lg"),
        ] {
            let props = ColorSwatchProps {
                value: opaque_blue(),
                size,
                ..ColorSwatchProps::default()
            };
            let html = render(&color_swatch(&props, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn shape_enumeration_maps_to_expected_classes() {
        for (shape, class) in [
            (SwatchShape::Square, "fd-color-swatch--shape-square"),
            (SwatchShape::Circle, "fd-color-swatch--shape-circle"),
            (SwatchShape::Rounded, "fd-color-swatch--shape-rounded"),
        ] {
            let props = ColorSwatchProps {
                value: opaque_blue(),
                shape,
                ..ColorSwatchProps::default()
            };
            let html = render(&color_swatch(&props, vec![], vec![]));
            assert!(html.contains(class), "shape={shape:?} -> {html}");
        }
    }

    #[test]
    fn transparent_value_emits_alpha_suffixed_hex_in_style() {
        let props = ColorSwatchProps {
            value: Color::from_rgba(Rgb::new(0x3b, 0x82, 0xf6), 0x80),
            ..ColorSwatchProps::default()
        };
        let html = render(&color_swatch(&props, vec![], vec![]));
        assert!(html.contains("--fd-swatch-color: #3b82f680"));
    }

    #[test]
    fn caller_class_and_style_attrs_are_dropped_not_duplicated() {
        let props = ColorSwatchProps {
            value: opaque_blue(),
            ..ColorSwatchProps::default()
        };
        let html = render(&color_swatch(
            &props,
            vec![("class", "attacker-controlled"), ("style", "attacker: 1")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert_eq!(html.matches("style=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
        assert!(!html.contains("attacker: 1"));
    }

    #[test]
    fn caller_data_scope_and_data_part_forgery_is_dropped() {
        let props = ColorSwatchProps {
            value: opaque_blue(),
            ..ColorSwatchProps::default()
        };
        let html = render(&color_swatch(
            &props,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="color-swatch""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        use fandhe_frontend_core::text;
        let props = ColorSwatchProps {
            value: opaque_blue(),
            ..ColorSwatchProps::default()
        };
        let html = render(&color_swatch(
            &props,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn to_hex_string_output_in_style_is_closed_over_hash_and_lowercase_hex_digits() {
        // Color::to_hex_string() の出力字母が `#[0-9a-f]` に閉じることの
        // 回帰（本モジュール冒頭「色値は検証済み型経由のみ」不変条件）。
        for value in [
            Color::from_rgb(Rgb::new(255, 255, 255)),
            Color::from_rgba(Rgb::new(0, 0, 0), 0),
            Color::from_rgba(Rgb::new(18, 52, 86), 171),
        ] {
            let hex = value.to_hex_string();
            assert!(hex.starts_with('#'));
            assert!(hex[1..]
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));
        }
    }

    #[test]
    fn css_output_declares_expected_size_and_shape_rules() {
        let out = css();
        assert!(out.contains("width: 1.5rem;"));
        assert!(out.contains("border-radius: 9999px;"));
        assert!(out.contains("border-radius: var(--fandhe-radius-sm);"));
    }
}
