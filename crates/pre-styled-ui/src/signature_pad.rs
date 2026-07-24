//! styled SignaturePad（headless ラッパー、イシュー #843、親 #520/#735）。
//!
//! `fandhe_frontend_headless_ui::signature_pad`（イシュー #843）の Root /
//! Label / Control / Segment / SegmentPath / Guide / ClearTrigger /
//! HiddenInput 8 anatomy パーツのうち、`class` を付与する必要がない
//! パーツ（Label/SegmentPath/Guide/HiddenInput）はそのまま再エクスポートし、
//! [`stylesheet`] で既定 CSS（寸法・枠線・ボタン外観）を追加提供する。
//! 薄い委譲の根拠・スコープ外事項は `crates/pre-styled-ui/src/qr_code.rs`
//! の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、[`crate::qr_code`]
//! と同型）
//!
//! `root`/`control`/`segment`/`clear_trigger` は本モジュールで再定義する
//! （headless 自由関数と名前衝突するため）。`pub use ...::*` ではなく必要な
//! 識別子（[`label`]/[`segment_path`]/[`guide`]/[`hidden_input`]/
//! [`stroke_path_d`]/[`stroke_to_payload`]/[`parse_stroke_payload`]/
//! [`Point`]/[`Stroke`]/[`StrokeError`]/[`SignaturePad`]/
//! [`SignaturePadAction`]/[`MAX_POINTS_PER_STROKE`]/[`MAX_STROKES`]）のみを
//! 選択的に再エクスポートする。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは headless 層の再エクスポートと静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラルで
//! あり、動的値へ CSS 値として流し込む経路を持たない（動的値は headless 層
//! 経由で `fandhe_frontend_core::render` の既定エスケープを必ず通る、
//! REQ-1）。styled `root`/`control`/`segment`/`clear_trigger` は
//! [`drop_class_attr`] により呼び出し側の `class` を除去してから合成する
//! ため、`class` 属性は常に単一（[`crate::qr_code::root`] と同型）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! headless 層と同じくスコープ外（`crates/headless-ui/src/signature_pad.rs`
//! doc 参照）: 画像エクスポート・筆圧シミュレーション・可変線幅・
//! `examples/headless-pre-styled-ui` への追随（crates.io 公開後に別途）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::signature_pad::{
    clear_trigger as headless_clear_trigger, control as headless_control,
    segment as headless_segment,
};
pub use fandhe_frontend_headless_ui::signature_pad::{
    guide, hidden_input, label, parse_stroke_payload, segment_path, stroke_path_d,
    stroke_to_payload, Point, SignaturePad, SignaturePadAction, Stroke, StrokeError,
    MAX_POINTS_PER_STROKE, MAX_STROKES,
};

/// headless `signature_pad` anatomy の `data-part` 一覧（
/// `crates/headless-ui/src/signature_pad.rs` の `ANATOMY.part(...)` 呼び出し
/// と同期させる契約）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "control",
    "segment",
    "segment-path",
    "guide",
    "clear-trigger",
    "hidden-input",
];

/// この styled SignaturePad の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。寸法・枠線は固定トークンのみで variant は
/// 提供しない（署名欄はフォーム内で寸法を呼び出し側が明示指定する場面が
/// 大半であり、`crates/headless-ui/src/signature_pad.rs::segment` の
/// `width`/`height` 引数が既に呼び出し側制御の主経路であるため、
/// `class` variant による寸法切替を重ねて設けない設計判断）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("signature-pad", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("flex-direction", "column"),
                decl("gap", "0.5rem"),
            ],
        )
        .base(
            "control",
            vec![
                decl("position", "relative"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.375rem"),
                decl("background", "var(--fandhe-color-bg)"),
            ],
        )
        .base(
            "segment",
            vec![decl("display", "block"), decl("width", "100%")],
        )
        .base(
            "segment-path",
            vec![
                decl("fill", "none"),
                decl("stroke", "var(--fandhe-color-fg)"),
                decl("stroke-width", "2"),
                decl("stroke-linecap", "round"),
                decl("stroke-linejoin", "round"),
            ],
        )
        .base(
            "guide",
            vec![
                decl("position", "absolute"),
                decl("left", "0.75rem"),
                decl("right", "0.75rem"),
                decl("bottom", "1.5rem"),
                decl("border-bottom", "1px dashed var(--fandhe-color-border)"),
            ],
        )
        .base(
            "clear-trigger",
            vec![
                decl("align-self", "flex-start"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.25rem"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("padding", "0.25rem 0.75rem"),
                decl("cursor", "pointer"),
            ],
        )
}

/// この styled SignaturePad が生成する静的 CSS 全量を返す（決定的。
/// [`crate::qr_code::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。[`drop_class_attr`] により呼び出し側の
/// `class` は除去してから合成する。実体は
/// [`fandhe_frontend_headless_ui::signature_pad::root`] へ委譲する。
#[must_use]
pub fn root<'a>(
    disabled: bool,
    empty: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::signature_pad::root(
        disabled,
        empty,
        drop_class_attr(attrs),
        children,
    )
}

/// styled control パーツ。呼び出し側 `attrs` の `class` は
/// [`drop_class_attr`] で除去する。実体は
/// [`fandhe_frontend_headless_ui::signature_pad::control`] へ委譲する。
#[must_use]
pub fn control<'a>(disabled: bool, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    headless_control(disabled, drop_class_attr(attrs), children)
}

/// styled segment パーツ。呼び出し側 `attrs` の `class` は
/// [`drop_class_attr`] で除去する。実体は
/// [`fandhe_frontend_headless_ui::signature_pad::segment`] へ委譲する。
#[must_use]
pub fn segment<'a>(
    width: u32,
    height: u32,
    aria_label_text: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    headless_segment(
        width,
        height,
        aria_label_text,
        drop_class_attr(attrs),
        children,
    )
}

/// styled clear-trigger パーツ。呼び出し側 `attrs` の `class` は
/// [`drop_class_attr`] で除去する。実体は
/// [`fandhe_frontend_headless_ui::signature_pad::clear_trigger`] へ委譲する。
#[must_use]
pub fn clear_trigger<'a>(
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    headless_clear_trigger(disabled, drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="signature-pad"][data-part="control"]"#));
        assert!(a.contains(r#"[data-scope="signature-pad"][data-part="segment-path"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn root_drops_caller_class() {
        let node = root(false, true, vec![("class", "attacker")], vec![]);
        let html = render(&node);
        assert!(!html.contains("attacker"));
        assert!(!html.contains("class="));
        assert!(html.contains("data-empty"));
    }

    #[test]
    fn control_delegates_to_headless() {
        let node = control(false, vec![], vec![]);
        let html = render(&node);
        assert!(html.contains(r#"data-scope="signature-pad" data-part="control""#));
    }

    #[test]
    fn segment_delegates_to_headless() {
        let node = segment(300, 150, None, vec![], vec![]);
        let html = render(&node);
        assert!(html.contains(r#"viewBox="0 0 300 150""#));
        assert!(html.contains(r#"data-scope="signature-pad" data-part="segment""#));
    }

    #[test]
    fn clear_trigger_reflects_disabled() {
        let node = clear_trigger(true, vec![], vec![]);
        let html = render(&node);
        assert!(html.contains("disabled"));
    }

    #[test]
    fn label_guide_hidden_input_are_reexported_from_headless() {
        let label_html = render(&label(
            vec![],
            vec![fandhe_frontend_core::text("Sign here")],
        ));
        assert!(label_html.contains(r#"data-part="label""#));

        let guide_html = render(&guide(vec![], vec![]));
        assert!(guide_html.contains(r#"data-part="guide""#));

        let hidden_html = render(&hidden_input("signature", "M0.00,0.00", false, vec![]));
        assert!(hidden_html.contains(r#"data-part="hidden-input""#));
    }
}
