//! styled ImageCropper（headless ラッパー、イシュー #844、親トラッキング
//! #520/#546）。
//!
//! `fandhe_frontend_headless_ui::image_cropper`（イシュー #844。§3.22
//! （イシュー #735）の意図的非採用の再導入、直接の先例は AngleSlider
//! 再導入イシュー #842）の Root / Viewport / Image / Grid / Handle の
//! anatomy パーツをそのまま再エクスポートし、[`stylesheet`] で既定 CSS を
//! 追加提供する。薄い委譲の根拠は [`crate::slider`] の rustdoc と同じ方針に
//! 従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`ImageCropper` 型・
//! headless `selection` を再エクスポートしない理由）
//!
//! [`crate::slider`] と同型の判断: 動的な位置・寸法を伝える唯一の経路
//! （[`ImageCropper::x_percent`](fandhe_frontend_headless_ui::image_cropper::ImageCropper::x_percent)
//! 等 4 アクセサから導出する `--fandhe-image-cropper-x`/`-y`/`-w`/`-h` の 4 個の
//! CSS custom property、下記「動的な値は 4 個の custom property のみ」参照）
//! は本モジュールの styled [`selection`] が一元的に組み立てる。headless
//! 自由関数 `selection` を呼び出し側が直接使うとこの唯一の経路を経由せず
//! 選択枠が描画されない事故を誘発するため、意図的に非公開のまま
//! [`selection`] 内部からのみ委譲する。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::image_cropper::ImageCropper`] も
//! **あえて**再エクスポートしない（[`crate::slider`] の `Slider` 非再
//! エクスポートと同じ理由）。状態管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::image_cropper::ImageCropper` を直接 import
//! し、実際の描画は本モジュールの styled [`root`]/[`selection`]（および
//! 再エクスポート済みのパーツ関数）を組み合わせて構築すること。
//!
//! # 動的な値は 4 個の custom property のみ（chakra-ui/Zag.js 方式）
//!
//! [`selection`] の位置・寸法は、headless 中立な `x_percent`/`y_percent`/
//! `width_percent`/`height_percent`（いずれも `0.0..=100.0` の正規化済み
//! 有限 `f64`）から [`percent_style`] が組み立てる
//! `style="--fandhe-image-cropper-x: <x>%; --fandhe-image-cropper-y: <y>%; \
//! --fandhe-image-cropper-w: <w>%; --fandhe-image-cropper-h: <h>%"` の 1 属性のみで
//! 伝搬する。[`crate::slider`] と同じく [`drop_style_attr`]（本モジュール内
//! 個別実装、`crates/headless-ui/src/progress.rs` の同名ヘルパと同型の
//! 判断）で呼び出し側 `attrs` に含まれる `style`（大文字小文字を無視）を
//! 除去してからフレームワーク側の `style` を優先する（重複属性による
//! 無効な HTML 出力・後勝ちの非決定的な描画を防ぐ、fail-closed）。
//!
//! # `size` variant のみ（`palette` は持たない）
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-image-cropper-handle-size` の root スコープ custom property
//! （CSS の通常のプロパティ継承により `handle` へ伝わる）経由で寸法を切り
//! 替える（[`crate::slider`] と同型）。`ColorPalette` は持たない
//! （selection 枠・handle の配色は装飾用途で固定色のままとし、切り抜き UI
//! に配色バリアントを持ち込む必然性がないため。`crate::steps` 等の
//! `size`-only コンポーネントと同型の判断）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - headless 層と同じく canvas による実画像切り出し・pointer ドラッグ/
//!   キーボード操作の DOM 配線はスコープ外
//!   （`fandhe_frontend_headless_ui::image_cropper` モジュール doc 参照）。
//! - `examples/headless-pre-styled-ui`（crates.io バージョン依存）への
//!   ImageCropper 追加は、未公開の新バージョンを参照できないため本
//!   イシューのスコープ外とする（[`crate::slider`] 冒頭 rustdoc の先例
//!   どおり crates.io 公開後に追随）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, StateCondition, VariantValue};

// `ImageCropper` 状態機械・headless 自由関数 `selection` はあえて
// 再エクスポートしない（本モジュール冒頭の rustdoc「選択的 re-export」節
// 参照）。状態管理・hydration が必要な呼び出し側は
// `fandhe_frontend_headless_ui::image_cropper::ImageCropper` を直接
// import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::image_cropper::ImageCropper;
pub use fandhe_frontend_headless_ui::image_cropper::{
    grid, handle, image, viewport, HandlePosition, ImageCropperAction,
};

/// headless `image-cropper` anatomy の `data-part` 一覧（
/// `crates/headless-ui/src/image_cropper.rs` の `ANATOMY.part(...)` 呼び出し
/// と同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を出力しない
/// fail-closed 側の不具合として現れるため、変更時は両ファイルを合わせて
/// 確認する）。
const SLOTS: &[&str] = &["root", "viewport", "image", "selection", "handle", "grid"];

/// `attrs` から `style`（ASCII 大文字小文字を無視）を除いた列を返す
/// （[`crate::slider::drop_style_attr`] と同型の判断。重複属性による無効な
/// HTML 出力・後勝ちの非決定的な描画を防ぐ、fail-closed）。
fn drop_style_attr<'a>(attrs: Vec<(&'a str, &'a str)>) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("style"))
        .collect()
}

/// `x`/`y`/`width`/`height` の百分率（[`ImageCropper::x_percent`] 等が返す
/// 正規化済み有限 `f64`）から 4 個の `--fandhe-image-cropper-*` custom property を
/// 設定する `style` 属性値を組み立てる（動的値はこの 1 箇所のみ、モジュール
/// doc「動的な値は 4 個の custom property のみ」参照）。
fn percent_style(x: f64, y: f64, width: f64, height: f64) -> String {
    format!(
        "--fandhe-image-cropper-x: {x}%; --fandhe-image-cropper-y: {y}%; \
         --fandhe-image-cropper-w: {width}%; --fandhe-image-cropper-h: {height}%"
    )
}

/// この styled ImageCropper の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("image-cropper", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-block"),
                decl("position", "relative"),
            ],
        )
        .base(
            "viewport",
            vec![
                decl("position", "relative"),
                decl("overflow", "hidden"),
                decl("display", "block"),
                decl("width", "100%"),
                decl("height", "100%"),
            ],
        )
        .base(
            "image",
            vec![decl("display", "block"), decl("max-width", "100%")],
        )
        .base(
            "selection",
            vec![
                decl("position", "absolute"),
                decl("left", "var(--fandhe-image-cropper-x, 0%)"),
                decl("top", "var(--fandhe-image-cropper-y, 0%)"),
                decl("width", "var(--fandhe-image-cropper-w, 100%)"),
                decl("height", "var(--fandhe-image-cropper-h, 100%)"),
                decl("box-sizing", "border-box"),
                decl("border", "2px solid var(--fandhe-color-bg)"),
                decl("box-shadow", "0 0 0 9999px rgba(0, 0, 0, 0.5)"),
                decl("cursor", "move"),
            ],
        )
        .base(
            "handle",
            vec![
                decl("position", "absolute"),
                decl("width", "var(--fandhe-image-cropper-handle-size, 0.75rem)"),
                decl("height", "var(--fandhe-image-cropper-handle-size, 0.75rem)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("box-sizing", "border-box"),
                decl("transform", "translate(-50%, -50%)"),
            ],
        )
        .state(
            "handle",
            StateCondition::AttrEq("data-handle-position", "n"),
            vec![
                decl("top", "0"),
                decl("left", "50%"),
                decl("cursor", "ns-resize"),
            ],
        )
        .state(
            "handle",
            StateCondition::AttrEq("data-handle-position", "s"),
            vec![
                decl("top", "100%"),
                decl("left", "50%"),
                decl("cursor", "ns-resize"),
            ],
        )
        .state(
            "handle",
            StateCondition::AttrEq("data-handle-position", "e"),
            vec![
                decl("top", "50%"),
                decl("left", "100%"),
                decl("cursor", "ew-resize"),
            ],
        )
        .state(
            "handle",
            StateCondition::AttrEq("data-handle-position", "w"),
            vec![
                decl("top", "50%"),
                decl("left", "0"),
                decl("cursor", "ew-resize"),
            ],
        )
        .state(
            "handle",
            StateCondition::AttrEq("data-handle-position", "ne"),
            vec![
                decl("top", "0"),
                decl("left", "100%"),
                decl("cursor", "nesw-resize"),
            ],
        )
        .state(
            "handle",
            StateCondition::AttrEq("data-handle-position", "nw"),
            vec![
                decl("top", "0"),
                decl("left", "0"),
                decl("cursor", "nwse-resize"),
            ],
        )
        .state(
            "handle",
            StateCondition::AttrEq("data-handle-position", "se"),
            vec![
                decl("top", "100%"),
                decl("left", "100%"),
                decl("cursor", "nwse-resize"),
            ],
        )
        .state(
            "handle",
            StateCondition::AttrEq("data-handle-position", "sw"),
            vec![
                decl("top", "100%"),
                decl("left", "0"),
                decl("cursor", "nesw-resize"),
            ],
        )
        .state(
            "handle",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "1px"),
            ],
        )
        .base(
            "grid",
            vec![
                decl("position", "absolute"),
                decl("inset", "0"),
                decl("pointer-events", "none"),
                decl(
                    "background-image",
                    "linear-gradient(to right, rgba(255, 255, 255, 0.5) 1px, transparent 1px), \
                     linear-gradient(to bottom, rgba(255, 255, 255, 0.5) 1px, transparent 1px)",
                ),
                decl(
                    "background-size",
                    "calc(100% / 3) 100%, 100% calc(100% / 3)",
                ),
            ],
        )
        .variant(
            Size::Xs,
            "root",
            vec![decl("--fandhe-image-cropper-handle-size", "0.35rem")],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl("--fandhe-image-cropper-handle-size", "0.55rem")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("--fandhe-image-cropper-handle-size", "0.75rem")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("--fandhe-image-cropper-handle-size", "0.95rem")],
        )
        .variant(
            Size::Xl,
            "root",
            vec![decl("--fandhe-image-cropper-handle-size", "1.15rem")],
        )
        .default_variant(Size::Md)
}

/// この styled ImageCropper が生成する静的 CSS 全量を返す（決定的。
/// [`crate::slider::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は
/// [`fandhe_frontend_headless_ui::image_cropper::ImageCropper::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_headless_ui::image_cropper::ImageCropper;
/// use fandhe_frontend_pre_styled_ui::image_cropper;
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let c = ImageCropper::default();
/// let node = image_cropper::root(Size::Md, &c, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="image-cropper" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    state: &ImageCropper,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    state.root(merged, children)
}

/// styled selection パーツを組み立てる。4 個の `--fandhe-image-cropper-*` custom
/// property を含む `style` を付与する唯一のパーツ（[`drop_style_attr`] に
/// より呼び出し側の `style` は除去してから合成する。動的値はこの 1 箇所
/// のみ、モジュール doc「動的な値は 4 個の custom property のみ」参照）。
/// 実体は
/// [`fandhe_frontend_headless_ui::image_cropper::ImageCropper::selection`]
/// へ委譲する。
#[must_use]
pub fn selection<'a>(
    state: &ImageCropper,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let style = percent_style(
        state.x_percent(),
        state.y_percent(),
        state.width_percent(),
        state.height_percent(),
    );
    let mut merged: Vec<(&str, &str)> = vec![("style", style.as_str())];
    merged.extend(drop_style_attr(attrs));
    state.selection(merged, children)
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
        assert!(a.contains(r#"[data-scope="image-cropper"][data-part="selection"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_references_cropper_custom_properties() {
        let css = stylesheet();
        for prop in [
            "--fandhe-image-cropper-x",
            "--fandhe-image-cropper-y",
            "--fandhe-image-cropper-w",
            "--fandhe-image-cropper-h",
            "--fandhe-image-cropper-handle-size",
        ] {
            assert!(css.contains(prop), "missing {prop} in css");
        }
    }

    #[test]
    fn stylesheet_links_handle_to_all_eight_positions() {
        let css = stylesheet();
        for pos in ["n", "s", "e", "w", "ne", "nw", "se", "sw"] {
            assert!(css.contains(&format!(
                r#"[data-scope="image-cropper"][data-part="handle"][data-handle-position="{pos}"] {{"#
            )));
        }
    }

    #[test]
    fn stylesheet_links_handle_to_focus_visible() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="image-cropper"][data-part="handle"]:focus-visible {"#));
    }

    #[test]
    fn stylesheet_contains_size_variant_selectors() {
        let css = stylesheet();
        assert!(css.contains("--size-"));
        assert!(css.contains("--fandhe-image-cropper-handle-size"));
    }

    // --- root ---

    #[test]
    fn root_outputs_scope_and_part() {
        let c = ImageCropper::default();
        let html = render(&root(Size::Md, &c, vec![], vec![]));
        assert!(html.contains(r#"data-scope="image-cropper""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md() {
        let c = ImageCropper::default();
        let html = render(&root(Size::Md, &c, vec![], vec![]));
        assert!(html.contains("fd-image-cropper--size-md"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        let c = ImageCropper::default();
        for (size, class) in [
            (Size::Xs, "fd-image-cropper--size-xs"),
            (Size::Sm, "fd-image-cropper--size-sm"),
            (Size::Md, "fd-image-cropper--size-md"),
            (Size::Lg, "fd-image-cropper--size-lg"),
            (Size::Xl, "fd-image-cropper--size-xl"),
        ] {
            let html = render(&root(size, &c, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let c = ImageCropper::default();
        let html = render(&root(
            Size::Md,
            &c,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let c = ImageCropper::default();
        let html = render(&root(
            Size::Md,
            &c,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="image-cropper""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- selection: --fandhe-image-cropper-* の唯一の動的値経路 ---

    #[test]
    fn selection_outputs_percent_style() {
        let c = ImageCropper::new(200, 100, 50, 25, 100, 50, None, 1);
        let html = render(&selection(&c, vec![], vec![]));
        assert!(html.contains("--fandhe-image-cropper-x: 25%"));
        assert!(html.contains("--fandhe-image-cropper-y: 25%"));
        assert!(html.contains("--fandhe-image-cropper-w: 50%"));
        assert!(html.contains("--fandhe-image-cropper-h: 50%"));
    }

    #[test]
    fn selection_caller_style_attr_is_dropped_not_duplicated() {
        let c = ImageCropper::default();
        let html = render(&selection(&c, vec![("style", "attacker: 1")], vec![]));
        assert_eq!(html.matches("style=\"").count(), 1);
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let c = ImageCropper::default();
        let html = render(&root(
            Size::Md,
            &c,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_image_src_alt_are_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&image(PAYLOAD, PAYLOAD, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_grid_children_text_are_escaped_on_render() {
        let html = render(&selection(
            &ImageCropper::default(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_image_cropper_state_machine() {
        // `ImageCropper` は本モジュールから再エクスポートしない（本モジュール
        // 冒頭の rustdoc「`ImageCropper` 型を再エクスポートしない理由」参照）
        // ため、headless-ui から直接 import して state machine 契約のみ
        // 検証する。
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut c = ImageCropper::new(200, 100, 0, 0, 50, 50, None, 1);
        let ssr_html = render(&root(Size::Md, &c, vec![], vec![]));
        assert!(!ssr_html.contains("data-hydrate-"));

        assert!(dispatch(&mut c, "move", "10,10"));
        assert_eq!(c.x(), 10);

        let hydrate_html = render(&render_for_hydration(&c));
        assert!(hydrate_html.contains(r#"data-hydrate-x="10""#));

        let restored = ImageCropper::from_hydration_attrs(&c.hydration_attrs()).unwrap();
        assert_eq!(restored, c);
    }
}
