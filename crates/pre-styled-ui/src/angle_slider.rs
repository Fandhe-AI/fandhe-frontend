//! styled AngleSlider（headless ラッパー、イシュー #842、親トラッキング #520）。
//!
//! `fandhe_frontend_headless_ui::angle_slider`（イシュー #842、非採用の
//! 再導入）の Label / Control / ValueText / HiddenInput の 4 anatomy パーツ
//! をそのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。
//! 薄い委譲の根拠は [`crate::slider`] の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! （[`crate::slider::root`] と同型）を本モジュールで再定義する。headless
//! 自由関数 `root` と名前衝突するため、`pub use ...::*` ではなく必要な
//! 識別子（[`label`]/[`control`]/[`hidden_input`]/[`value_text`]/
//! [`AngleSliderAction`]）のみを選択的に再エクスポートする。
//!
//! `thumb` も再エクスポートしない。動的な回転角を伝える唯一の経路
//! （[`AngleSlider::angle_deg`](fandhe_frontend_headless_ui::angle_slider::AngleSlider::angle_deg)
//! から導出する `--fandhe-angle` CSS custom property、モジュール doc
//! 「動的な値は 1 点のみ」参照）は本モジュールの styled [`thumb_styled`]
//! が一元的に組み立てる。headless 自由関数 `thumb` を呼び出し側が直接
//! 使うとこの唯一の経路を経由せず回転が描画されない事故を誘発するため、
//! 意図的に非公開のまま [`thumb_styled`] 内部からのみ委譲する（[`crate::slider`]
//! の `range` 非再エクスポートと同型の判断）。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::angle_slider::AngleSlider`] も
//! **あえて**再エクスポートしない（[`crate::slider`] の `Slider` 非
//! 再エクスポートと同じ理由）。状態管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::angle_slider::AngleSlider` を直接 import
//! し、実際の描画は本モジュールの styled [`root`]/[`thumb_styled`]
//! （および再エクスポート済みのパーツ関数）を組み合わせて構築すること。
//!
//! # 動的な値は `--fandhe-angle` の 1 点のみ（canvas 不使用）
//!
//! [`thumb_styled`] の回転は、headless 中立な
//! [`AngleSlider::angle_deg`](fandhe_frontend_headless_ui::angle_slider::AngleSlider::angle_deg)
//! （`0..=359` の整数）から [`angle_style`] が組み立てる
//! `style="--fandhe-angle: <value>deg"` の 1 属性のみで伝搬し、CSS 側は
//! `transform: rotate(var(--fandhe-angle))` で描画する。canvas の描画命令
//! 列・変換行列のような内部状態は一切持たない（非採用理由への回答、
//! `fandhe_frontend_headless_ui::angle_slider` モジュール doc「非採用の
//! 再導入であること」節参照）。[`crate::slider`]/[`crate::progress`]
//! と同型の [`crate::css::drop_style_attr`] 相当のヘルパを本モジュール内に
//! 個別実装し（[`drop_style_attr`]）、呼び出し側 `attrs` に含まれる
//! `style`（大文字小文字を無視）を除去してからフレームワーク側の `style`
//! を優先する（重複属性による無効な HTML 出力・後勝ちの非決定的な描画を
//! 防ぐ、fail-closed）。
//!
//! # `size`/`palette` variant
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-angle-slider-track-size`/`-thumb-size` の root スコープ custom
//! property（CSS の通常のプロパティ継承により `control`/`thumb` へ伝わる）
//! 経由で寸法を切り替える（[`crate::slider`] と同型）。`palette`
//! （[`ColorPalette`]）は既存の [`crate::recipe::palette_scale_declarations`]
//! （chakra-ui virtual token 方式、#606）を `root` へ登録し、`thumb` の色を
//! `var(--fandhe-palette, ...)` 経由で切り替える。
//!
//! # `focus-visible`（キーボードフォーカスリング）
//!
//! [`thumb_styled`] はネイティブにフォーカス可能な要素（`tabindex`）である
//! ため、通常の `:focus-visible` 疑似クラスを [`recipe`] へ直接登録する
//! （[`StateCondition::FocusVisible`]、[`crate::slider`] と同型）。
//!
//! # マーカー（目盛り）は CSS-only 表現（anatomy 非追加、イシュー #1445）
//!
//! ark-ui の Angle Slider は外周の目盛りリングと中心点を持つが、headless
//! `angle-slider` anatomy（`crates/headless-ui/src/angle_slider.rs`）に
//! MarkerGroup/Marker パーツは存在しない（意図的スコープ外、headless
//! モジュール doc に明記済み）。本イシューの対象ファイルは本モジュール
//! （`crates/pre-styled-ui/src/angle_slider.rs`）のみであり headless 側の
//! anatomy を増やす変更は範囲外のため、目盛り・中心点は新しい DOM 要素を
//! 追加せず [`recipe`] の `control` `background` に多層グラデーション
//! （`radial-gradient` の中心点 + 内側を面色で覆う `radial-gradient` +
//! `repeating-conic-gradient` の目盛りリング + 面色）を静的リテラルとして
//! 重ねることで再現する（先例: [`crate::color_picker`]/
//! [`crate::color_swatch`] の複数背景レイヤー手法）。`decl()` の
//! `&'static str` 制約によりすべてソースコード内リテラルで完結し、実行時
//! 文字列連結は行わない（本モジュールの XSS 不変条件を保つ）。
//!
//! # サムの形状はニードルへ寄せない（意図的に合わせない差分）
//!
//! ark-ui はニードル（線 + 先端の点）形状だが、本実装は既存の「回転する
//! 点」形状を維持する。ニードル形状にすると `transform-origin` の基準点・
//! `--fandhe-angle` 1 点で完結する回転機構（モジュール冒頭 doc「動的な値は
//! `--fandhe-angle` の 1 点のみ」参照）の再設計が必要になり、本イシューが
//! 担当するトラック・サム・マーカーの見た目調整の範囲を超えるため、最小
//! 差分（サイズ・面・フォーカスリング・hover・transition の是正のみ）に
//! 留める。
//!
//! # variant 軸を追加しない（バリアント判断）
//!
//! ark-ui Angle Slider に variant 軸はなく、chakra-ui / Radix UI にも同種の
//! 部品は存在しない（`docs/design/component-coverage-map.md` 参照）ため、
//! 新しい variant 軸（`ButtonVariant` 相当）は追加しない。既存の `size`/
//! `palette` 2 軸のみで十分であり、過不足のない状態と判断する。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - headless 層と同じく MarkerGroup/Marker・pointer ドラッグ/キーボード
//!   操作の DOM 配線はスコープ外
//!   （`fandhe_frontend_headless_ui::angle_slider` モジュール doc 参照）。
//! - `examples/headless-pre-styled-ui`（crates.io バージョン依存）への
//!   AngleSlider 追加は、未公開の新バージョンを参照できないため本イシュー
//!   のスコープ外とする（[`crate::slider`] 冒頭 rustdoc の先例どおり
//!   crates.io 公開後に追随）。
//! - ラベル・値テキスト（`label`/`value_text`）の型階層調整は分割 2/2
//!   （イシュー #1446）の担当であり本イシューでは触らない。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_solid, hover_surface_declarations,
    palette_scale_declarations, transition_declarations, ColorPalette, FocusRingColor,
    FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition, VariantValue,
};

// `AngleSlider` 状態機械・headless 自由関数 `root`/`thumb` はあえて再
// エクスポートしない（本モジュール冒頭の rustdoc「選択的 re-export」節
// 参照）。状態管理・hydration が必要な呼び出し側は
// `fandhe_frontend_headless_ui::angle_slider::AngleSlider` を直接 import する。
use fandhe_frontend_headless_ui::angle_slider::AngleSlider;
pub use fandhe_frontend_headless_ui::angle_slider::{
    control, hidden_input, label, value_text, AngleSliderAction,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// headless `angle-slider` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/angle_slider.rs` の `ANATOMY.part(...)` 呼び出し
/// と同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を出力
/// しない fail-closed 側の不具合として現れるため、変更時は両ファイルを
/// 合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "control",
    "thumb",
    "hidden-input",
    "value-text",
];

/// `attrs` から `style`（ASCII 大文字小文字を無視）を除いた列を返す。
///
/// [`thumb_styled`] がフレームワーク側で `--fandhe-angle` を含む `style` を
/// 組み立てた後、呼び出し側 `attrs` を連結する前に使う dedup ヘルパ
/// （`crates/headless-ui/src/progress.rs::drop_style_attr`/
/// `crates/pre-styled-ui/src/slider.rs::drop_style_attr` と同型の判断。
/// 重複属性による無効な HTML 出力・後勝ちの非決定的な描画を防ぐ、
/// fail-closed）。
fn drop_style_attr<'a>(attrs: Vec<(&'a str, &'a str)>) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("style"))
        .collect()
}

/// `angle`（[`AngleSlider::angle_deg`] が返す `0..=359` の整数）から
/// `--fandhe-angle` custom property を設定する `style` 属性値を組み立てる
/// （動的値はこの 1 箇所のみ、モジュール doc「動的な値は 1 点のみ」参照）。
fn angle_style(angle: u16) -> String {
    format!("--fandhe-angle: {angle}deg")
}

/// この styled AngleSlider の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("angle-slider", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("flex-direction", "column"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
            ],
        )
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .base(
            "label",
            vec![decl("font-size", "var(--fandhe-font-font-size-sm)")],
        )
        .base(
            "control",
            vec![
                decl("position", "relative"),
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl(
                    "width",
                    "var(--fandhe-angle-slider-track-size, 4.5rem)",
                ),
                decl(
                    "height",
                    "var(--fandhe-angle-slider-track-size, 4.5rem)",
                ),
                decl("border-radius", "var(--fandhe-radius-full)"),
                // ark-ui の「白い面 + 影 + 外周目盛りリング + 中心点」を
                // 新しい DOM 要素を追加せず多層グラデーションで再現する
                // （モジュール冒頭 doc「マーカーは CSS-only 表現」参照）。
                // 手前から: (a) 中心のグレー点、(b) (c) の内側を面色で覆い
                // 目盛りが外周のみに見えるようにするマスク層、(c) 目盛り
                // リング（`repeating-conic-gradient`）、(d) 面色（最終層は
                // `background` shorthand の仕様上プレーンな色のみ許容）。
                // (b) は `circle closest-side` を明示する（サイズキーワード
                // 省略時の既定 `farthest-corner` だと `%` が対角線基準に
                // なり、正方形ボックスでは外接円（`farthest-corner`）と
                // 内接円（`closest-side` = 表示上の円の半径）が一致しない。
                // これにより `calc(100% - 6px)` が円の半径から 6px 引いた
                // 値になり、外側 6px リングだけを露出できる。Bugbot 指摘
                // イシュー #1445 PR #1728 で是正）。
                decl(
                    "background",
                    "radial-gradient(circle, var(--fandhe-color-fg-muted) 0 2px, transparent 2px), radial-gradient(circle closest-side, var(--fandhe-color-bg) 0 calc(100% - 6px), transparent calc(100% - 6px)), repeating-conic-gradient(var(--fandhe-color-border) 0deg 1deg, transparent 1deg 30deg), var(--fandhe-color-bg)",
                ),
                decl("box-shadow", "var(--fandhe-shadow-sm)"),
            ],
        )
        .base(
            "thumb",
            vec![
                // `control` の中心から上端方向へ配置し、`transform: rotate`
                // の回転軸が円環の中心と一致するようにする（canvas 不使用、
                // モジュール冒頭 doc「動的な値は 1 点のみ」参照）。
                decl("position", "absolute"),
                decl("top", "50%"),
                decl("left", "50%"),
                decl(
                    "width",
                    "var(--fandhe-angle-slider-thumb-size, 0.9rem)",
                ),
                decl(
                    "height",
                    "var(--fandhe-angle-slider-thumb-size, 0.9rem)",
                ),
                decl("margin-top", "calc(var(--fandhe-angle-slider-track-size, 4.5rem) / -2)"),
                decl("margin-left", "calc(var(--fandhe-angle-slider-thumb-size, 0.9rem) / -2)"),
                decl(
                    "transform-origin",
                    "calc(var(--fandhe-angle-slider-thumb-size, 0.9rem) / 2) calc(var(--fandhe-angle-slider-track-size, 4.5rem) / 2)",
                ),
                decl("transform", "rotate(var(--fandhe-angle, 0deg))"),
                decl("border-radius", "var(--fandhe-radius-full)"),
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                // hover 時の背景色差し替え用 custom property の定義（実際の
                // `background: var(--fandhe-hover-bg)` 適用は下記
                // `.state("thumb", StateCondition::Hover, ...)` 1 本に集約
                // する、`crate::button` の solid variant と同型）。
                hover_bg_solid(),
                decl("box-sizing", "border-box"),
                decl("cursor", "pointer"),
            ],
        )
        .base(
            "thumb",
            // `transform`（ドラッグ中の角度追従）は含めない —
            // `--fandhe-angle` の変化に transition を掛けるとポインタ操作の
            // 追従が遅延して見えるため、面・影のみを滑らかにする
            // （イシュー #1425 共通ビジュアル言語の適用、`prefers-reduced-motion`
            // は `Theme::to_css` の duration 一括 0ms 化で自動対応）。
            transition_declarations("background, box-shadow", MotionDuration::Fast),
        )
        .state(
            "thumb",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        .state("thumb", StateCondition::Hover, hover_surface_declarations())
        .state(
            "thumb",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-angle-slider-track-size", "2.5rem"),
                decl("--fandhe-angle-slider-thumb-size", "0.5rem"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-angle-slider-track-size", "3.5rem"),
                decl("--fandhe-angle-slider-thumb-size", "0.7rem"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-angle-slider-track-size", "4.5rem"),
                decl("--fandhe-angle-slider-thumb-size", "0.9rem"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-angle-slider-track-size", "5.5rem"),
                decl("--fandhe-angle-slider-thumb-size", "1.1rem"),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-angle-slider-track-size", "6.5rem"),
                decl("--fandhe-angle-slider-thumb-size", "1.3rem"),
            ],
        )
        .default_variant(Size::Md)
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

/// この styled AngleSlider が生成する静的 CSS 全量を返す（決定的。
/// [`crate::slider::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は
/// 除去してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::angle_slider::AngleSlider::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_headless_ui::angle_slider::AngleSlider;
/// use fandhe_frontend_pre_styled_ui::angle_slider;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let s = AngleSlider::default();
/// let node = angle_slider::root(Size::Md, ColorPalette::Accent, &s, false, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="angle-slider" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    state: &AngleSlider,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    state.root(disabled, merged, children)
}

/// styled thumb パーツを組み立てる。`--fandhe-angle` を含む `style` を
/// 付与する唯一のパーツ（[`drop_style_attr`] により呼び出し側の `style`
/// は除去してから合成する。動的値はこの 1 箇所のみ、モジュール doc
/// 「動的な値は 1 点のみ」参照）。実体は
/// [`fandhe_frontend_headless_ui::angle_slider::AngleSlider::thumb`] へ委譲する。
#[must_use]
pub fn thumb_styled<'a>(
    state: &AngleSlider,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let style = angle_style(state.angle_deg());
    let mut merged: Vec<(&str, &str)> = vec![("style", style.as_str())];
    merged.extend(drop_style_attr(attrs));
    state.thumb(disabled, merged, Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="angle-slider"][data-part="thumb"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_references_angle_custom_property_and_rotate() {
        let css = stylesheet();
        assert!(css.contains("--fandhe-angle"));
        assert!(css.contains("rotate(var(--fandhe-angle, 0deg))"));
    }

    #[test]
    fn stylesheet_links_root_to_disabled_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="angle-slider"][data-part="root"][data-disabled] {"#));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn stylesheet_links_thumb_to_focus_visible() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="angle-slider"][data-part="thumb"]:focus-visible {"#));
        assert!(css.contains("var(--fandhe-focus-ring-width, 2px)"));
        assert!(css.contains("var(--fandhe-focus-ring-offset, 2px)"));
    }

    #[test]
    fn stylesheet_control_uses_full_radius_and_shadow_token() {
        // トラック・サムはスケールトークン外のローカル値（`999px`）ではなく
        // 共通 `--fandhe-radius-full` トークンを参照する（イシュー #1445）。
        let css = stylesheet();
        assert!(!css.contains("999px"));
        assert!(css.contains("border-radius: var(--fandhe-radius-full);"));
        assert!(css.contains("box-shadow: var(--fandhe-shadow-sm);"));
    }

    #[test]
    fn stylesheet_control_layers_marker_gradients_without_new_dom() {
        // マーカー（目盛り・中心点）は anatomy を増やさず `control` の
        // `background` 多層グラデーションで表現する（モジュール冒頭 doc
        // 「マーカーは CSS-only 表現」参照）。
        let css = stylesheet();
        assert!(css.contains("repeating-conic-gradient(var(--fandhe-color-border)"));
        assert!(css.contains("radial-gradient(circle, var(--fandhe-color-fg-muted)"));
    }

    #[test]
    fn stylesheet_links_thumb_to_hover_and_transition() {
        let css = stylesheet();
        assert!(css.contains("var(--fandhe-hover-bg)"));
        assert!(css.contains("--fandhe-hover-bg: var(--fandhe-palette-emphasized);"));
        assert!(css.contains("transition-property: background, box-shadow;"));
        // ドラッグ中の角度追従を遅延させないため `transform` は
        // transition-property に含めない（モジュール冒頭 doc「サムの形状」
        // に隣接するコメント、`recipe` の `thumb` transition 登録箇所参照）。
        assert!(!css.contains("transition-property: transform"));
    }

    #[test]
    fn stylesheet_contains_size_and_palette_variant_selectors() {
        let css = stylesheet();
        assert!(css.contains("--size-"));
        assert!(css.contains("--color-palette-"));
        assert!(css.contains("--fandhe-angle-slider-track-size"));
    }

    // --- root ---

    #[test]
    fn root_outputs_scope_and_part() {
        let s = AngleSlider::default();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="angle-slider""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let s = AngleSlider::default();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-angle-slider--size-md"));
        assert!(html.contains("fd-angle-slider--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        let s = AngleSlider::default();
        for (size, class) in [
            (Size::Xs, "fd-angle-slider--size-xs"),
            (Size::Sm, "fd-angle-slider--size-sm"),
            (Size::Md, "fd-angle-slider--size-md"),
            (Size::Lg, "fd-angle-slider--size-lg"),
            (Size::Xl, "fd-angle-slider--size-xl"),
        ] {
            let html = render(&root(size, ColorPalette::Accent, &s, false, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        let s = AngleSlider::default();
        for (palette, class) in [
            (
                ColorPalette::Accent,
                "fd-angle-slider--color-palette-accent",
            ),
            (ColorPalette::Info, "fd-angle-slider--color-palette-info"),
            (
                ColorPalette::Success,
                "fd-angle-slider--color-palette-success",
            ),
            (
                ColorPalette::Warning,
                "fd-angle-slider--color-palette-warning",
            ),
            (
                ColorPalette::Danger,
                "fd-angle-slider--color-palette-danger",
            ),
            (
                ColorPalette::Neutral,
                "fd-angle-slider--color-palette-neutral",
            ),
        ] {
            let html = render(&root(Size::Md, palette, &s, false, vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let s = AngleSlider::default();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let s = AngleSlider::default();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="angle-slider""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- thumb: --fandhe-angle の唯一の動的値経路 ---

    #[test]
    fn thumb_styled_outputs_angle_style() {
        let s = AngleSlider::new(90, 1);
        let html = render(&thumb_styled(&s, false, vec![]));
        assert!(html.contains(r#"style="--fandhe-angle: 90deg""#));
        assert!(html.contains(r#"role="slider""#));
    }

    #[test]
    fn thumb_styled_caller_style_attr_is_dropped_not_duplicated() {
        let s = AngleSlider::new(90, 1);
        let html = render(&thumb_styled(&s, false, vec![("style", "attacker: 1")]));
        assert_eq!(html.matches("style=\"").count(), 1);
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let s = AngleSlider::default();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_label_children_are_escaped_on_render() {
        let html = render(&label(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn reexported_hidden_input_name_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&hidden_input(PAYLOAD, "40", false, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_angle_slider_state_machine() {
        // `AngleSlider` は本モジュールから再エクスポートしない（本モジュール
        // 冒頭の rustdoc「選択的 re-export」節参照）ため、headless-ui から
        // 直接 import して state machine 契約のみ検証する。
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut s = AngleSlider::new(20, 10);
        let ssr_html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![],
            vec![],
        ));
        assert!(!ssr_html.contains("data-hydrate-"));

        assert!(dispatch(&mut s, "increment", ""));
        assert_eq!(s.angle_deg(), 30);

        let hydrate_html = render(&render_for_hydration(&s));
        assert!(hydrate_html.contains(r#"data-hydrate-value="30""#));

        let restored = AngleSlider::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }
}
