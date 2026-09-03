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
//! 持つが、本実装は既存共通軸 [`crate::recipe::Size`]（Xs〜Xl の 5 段、
//! #1681）に限定する（[`crate::tag::TagVariant`] が chakra `surface` を
//! 見送ったのと同型の最小サブセット判断、スコープ外）。`shape` は chakra
//! 同様 `square`/`circle`/`rounded`（既定）の 3 値を提供する。chakra は
//! `variant`/`colorPalette` 軸を持たない（色は `value` そのもの）ため、
//! 本実装もこれらの軸を持たない。
//!
//! # イシュー #1558 の参照サイト比較（7 軸チェック）
//!
//! chakra-ui（ColorSwatch、`size`（`2xs`〜`2xl`/`inherit`/`full`、既定
//! `md`）+ `shape`（`square`/`circle`/`rounded`(既定)）、`variant`/
//! `colorPalette` 軸なし）とスクリーンショット
//! （`docs/design/reference-screenshots/chakra-color-swatch-{1,2,3}.png`・
//! `themes-color-swatch.png`）を比較した結果を記録する。ark-ui / Radix
//! Themes には対応部品がないため比較対象は chakra-ui のみ。
//!
//! - **サイズ**: [`crate::recipe::Size`] の Xs〜Xl の実寸を、#1681 時点の
//!   等差外挿（Xs=8px/Sm=16px/Md=24px/Lg=32px/Xl=40px）から chakra の
//!   同名段の実寸（xs=16px/sm=18px/md=20px/lg=24px/xl=28px）へ是正した
//!   （avatar #1554 と同型の判断）。chakra の `2xs`/`2xl`/`inherit`/`full`
//!   は共通語彙の 5 段に存在しないため採らない。
//! - **バリアント**: chakra 同様 `variant`/`colorPalette` 軸を持たない
//!   （色は [`ColorSwatchProps::value`] そのもので決まる）。増減なし。
//! - **色**: チェッカーボードの 2 色目は chakra の `bg.emphasized` では
//!   なく `--fandhe-color-border`（light `#d9d9d9`、chakra `#e2e2e2` と
//!   ほぼ同値）を維持する（#838 PR #858 の Bugbot 指摘で確定した「
//!   `--fandhe-color-bg` と常にコントラストが取れる固定トークン」の根拠を
//!   優先し変更しない）。生の色リテラルは持ち込まない。
//! - **状態（`data-*`）**: 増減なし。`data-scope`/`data-part` のみ
//!   （[`ANATOMY`]）。
//! - **ダーク**: 追加宣言（`--fandhe-color-border-muted` 参照の
//!   `box-shadow`）はトークン参照のため `write_dark_declarations` へ
//!   自動追従する。
//! - **フォーカス**: 非適用（意図的）。表示専用でフォーカス不能な
//!   `<span>` であり #1424 の適用対象外。
//! - **余白・角丸・影**: base へ内側 1px の輪郭リング
//!   `box-shadow: inset 0 0 0 1px var(--fandhe-color-border-muted)` を
//!   追加した（chakra の `inset 0 0 0 1px rgba(0, 0, 0, 0.1)` 相当。白・
//!   淡色・低アルファ色でも輪郭が判別できるようにするため。`border` では
//!   なく `box-shadow: inset` にするのは、要素サイズを変えずに
//!   `background-image` の色レイヤーの上に重なる描画順を得るため
//!   （`docs/design/pre-styled-ui-scale-tokens.md` §5.2 の「リング・
//!   ドット描画用途は影トークン化対象外」に該当する意図的な生値だが色は
//!   トークン経由）。角丸 `--fandhe-radius-sm`（Rounded 既定）は chakra
//!   `l1`（xs=2px 相当）より 1 段大きいが、参照スクショの見た目と一致し
//!   badge/kbd と同じ密なインライン部品の段のため維持する。`box-sizing`
//!   は `border` を使わないため不要（avatar の `border-box` 追加とは前提
//!   が異なる）。
//! - **レイアウト（余白・角丸・影に準ずる是正）**: base を
//!   `display: inline-block` から `display: inline-flex` +
//!   `align-items: center` + `justify-content: center` +
//!   `flex-shrink: 0` へ変更した（chakra の `inline-flex` 中央寄せ +
//!   `flex-shrink: 0` に整合）。`vertical-align: middle` は維持するため
//!   インライン文中配置は変わらず、`children` を受け取る API（アイコン等
//!   の重ね表示）が中央寄せされ flex 行で潰れなくなる。
//! - **hover / disabled / transition**: 非適用（意図的）。
//!   `docs/design/pre-styled-ui-interaction-visual-language.md` §3 が
//!   「表示専用には hover を付けない」と明記しており、disabled 概念・
//!   遷移対象もない。
//!
//! ## スコープ外（変更しない点）
//!
//! チェッカーボードタイル `8px 8px` の生値のトークン化は
//! [`crate::color_picker`] との横断事項であり本イシューでは対応しない
//! （PR 本文の対象外に記載）。chakra `ColorSwatchMix`（複数色の分割表示）・
//! `Group attached` 相当の連結レイアウトも見送る（最小サブセット方針）。

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
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("flex-shrink", "0"),
                decl("vertical-align", "middle"),
                decl(
                    "background-image",
                    "linear-gradient(var(--fd-swatch-color), var(--fd-swatch-color)), repeating-conic-gradient(var(--fandhe-color-border) 0% 25%, var(--fandhe-color-bg) 0% 50%)",
                ),
                decl("background-size", "100% 100%, 8px 8px"),
                // イシュー #1558: 参照サイト（chakra-ui）は
                // `inset 0 0 0 1px rgba(0, 0, 0, 0.1)` の内側 1px リングで
                // 白・淡色・低アルファ色でも輪郭を判別できるようにしている。
                // 生の `rgba()` は持ち込まず、light `#e6e6e6` / dark `#2a2a2a`
                // のトークン `--fandhe-color-border-muted` を参照する（light
                // 値は chakra の実効色とほぼ一致、ダークはトークン再定義経由
                // で自動追従）。`box-shadow: inset` は要素サイズを変えずに
                // `background-image` の色レイヤーの上に重なる描画順のため、
                // `border` よりチェッカーボードとの整合が良い
                // （`docs/design/pre-styled-ui-scale-tokens.md` §5.2 の
                // 「リング・ドット描画用途は影トークン化対象外」に該当する
                // 意図的な生値だが、色自体はトークン経由にする）。
                decl(
                    "box-shadow",
                    "inset 0 0 0 1px var(--fandhe-color-border-muted)",
                ),
            ],
        )
        // イシュー #1558: chakra-ui の同名 size 段（xs=16px/sm=18px/
        // md=20px/lg=24px/xl=28px）の実寸へ是正した（#1681 時点は
        // Sm→Md→Lg の 0.5rem 刻み等差外挿だった）。`SlotRecipe::size_variants`
        // 経由にすることで既定 `Md` を構造的に保証する
        // （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
        // §4）。chakra の `2xs`/`2xl`/`inherit`/`full` は共通語彙
        // `recipe::Size`（5 段）に存在しないため採らない
        // （モジュール冒頭「最小サブセット方針」参照）。
        .size_variants(
            "root",
            &[
                (Size::Xs, vec![decl("width", "1rem"), decl("height", "1rem")]),
                (
                    Size::Sm,
                    vec![decl("width", "1.125rem"), decl("height", "1.125rem")],
                ),
                (
                    Size::Md,
                    vec![decl("width", "1.25rem"), decl("height", "1.25rem")],
                ),
                (
                    Size::Lg,
                    vec![decl("width", "1.5rem"), decl("height", "1.5rem")],
                ),
                (
                    Size::Xl,
                    vec![decl("width", "1.75rem"), decl("height", "1.75rem")],
                ),
            ],
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
            (Size::Xs, "fd-color-swatch--size-xs"),
            (Size::Sm, "fd-color-swatch--size-sm"),
            (Size::Md, "fd-color-swatch--size-md"),
            (Size::Lg, "fd-color-swatch--size-lg"),
            (Size::Xl, "fd-color-swatch--size-xl"),
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
        assert!(out.contains("width: 1rem;"));
        assert!(out.contains("width: 1.125rem;"));
        assert!(out.contains("width: 1.25rem;"));
        assert!(out.contains("width: 1.5rem;"));
        assert!(out.contains("width: 1.75rem;"));
        assert!(out.contains("border-radius: 9999px;"));
        assert!(out.contains("border-radius: var(--fandhe-radius-sm);"));
    }

    #[test]
    fn root_base_declares_inset_ring_via_border_muted_token() {
        let out = css();
        assert!(out.contains("box-shadow: inset 0 0 0 1px var(--fandhe-color-border-muted);"));
    }

    #[test]
    fn root_base_is_inline_flex_centered_and_non_shrinking() {
        let out = css();
        assert!(out.contains("display: inline-flex;"));
        assert!(out.contains("align-items: center;"));
        assert!(out.contains("justify-content: center;"));
        assert!(out.contains("flex-shrink: 0;"));
        assert!(out.contains("vertical-align: middle;"));
    }

    #[test]
    fn css_has_no_raw_color_literals() {
        let out = css();
        assert!(!out.contains('#'));
        assert!(!out.contains("rgb("));
        assert!(!out.contains("rgba("));
    }

    #[test]
    fn size_md_is_default_variant() {
        let props = ColorSwatchProps {
            value: opaque_blue(),
            ..ColorSwatchProps::default()
        };
        let html = render(&color_swatch(&props, vec![], vec![]));
        assert!(html.contains("fd-color-swatch--size-md"));
    }
}
