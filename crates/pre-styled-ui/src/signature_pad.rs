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
//! # スタイル調整（イシュー #1503）
//!
//! 参考サイト（ark-ui signature-pad。chakra-ui / Radix に同等部品なし）と
//! 比較し、以下を是正した。
//!
//! - `control` を寸法・枠線のみの空欄から、淡い面（
//!   `var(--fandhe-color-bg-muted)`）・大きめ角丸（`--fandhe-radius-lg`）・
//!   空でも潰れない最小寸法（`min-width`/`min-height`）を持つ「署名欄」の
//!   見た目へ実体化した。`cursor: crosshair` と `touch-action: none` は
//!   描画面の操作性のための追加（装飾ではなく操作契約）
//! - `label`/`root`/`guide`/`clear-trigger` のリテラル値
//!   （`0.375rem`/`0.25rem`/`0.5rem`/`0.75rem`/`1.5rem` 等）をトークン
//!   スケール（`--fandhe-radius-*`/`--fandhe-space-*`）参照へ置き換えた
//! - `label` に型階層（`font-size`/`font-weight`/`line-height`）を新設した
//!   （rating-group/radio-group と同型）
//! - `clear-trigger` に hover 背景・`:focus-visible` リング・
//!   `data-disabled` の視覚反映・transition を追加した（Phase 0 共通
//!   ビジュアル言語、`crate::recipe` のヘルパ経由）
//! - `root`/`control` にも `data-disabled` の視覚反映（`cursor`。`opacity`
//!   は `root` にのみ適用し、子孫（`control`/`clear-trigger`）は `cursor`
//!   のみへ縮小する。両方に `opacity: 0.5` を重ねると子孫が祖先の減光と
//!   乗算され `0.25` へ二重減光してしまうため（`password_input`/
//!   `date_input` #1469 と同型の判断、イシュー #1503 PR #1776 Bugbot
//!   レビュー Medium severity 指摘対応）を追加した
//! - `control` に `data-readonly` の視覚反映（`touch-action: auto`・
//!   `cursor: default`）を追加した。read-only な署名欄は描画操作を
//!   開始できないため、`base` の `touch-action: none`／`cursor:
//!   crosshair`（描画面としての操作契約）をそのまま適用するとモバイルで
//!   ブラウザ標準のパン・スクロールができない領域になり、かつ
//!   `crosshair` が「描画可能」という誤った状態表示になる（イシュー
//!   #1503 PR #1776 codex-review P1 指摘対応）
//! - `control` の `data-disabled` 規則にも `touch-action: auto` を追加
//!   した。disabled と read-only は独立した属性であり、disabled 単独
//!   （read-only ではない）の control でも `base` の `touch-action:
//!   none` が残ったままだとモバイルでこの領域からページをパン・
//!   スクロールできなくなるため（イシュー #1503 PR #1776 codex-review
//!   P1 再指摘対応）
//!
//! 意図的に参考サイトへ合わせない点（理由付き）:
//!
//! - **size / variant 軸は追加しない**: ark-ui にも size/variant 軸はなく、
//!   寸法は headless `segment` の `width`/`height` 引数が既に呼び出し側
//!   制御の主経路であるため（既存設計判断、変更なし）
//! - **`control` へ `:hover` は付けない**: 描画面は「押すと描ける」面で
//!   あり、hover 背景変化の対象（ボタン・リンク類）ではない。ark も pad
//!   面に hover 変化を持たない
//! - **`control` へ `:focus-visible`/`:focus-within` リングは付けない**:
//!   headless `control` は `<div>`（tabindex なし）でフォーカスを持たず、
//!   フォーカス可能要素はリング付与済みの `clear-trigger` と
//!   `hidden-input`（視覚外）のみ
//! - **`data-empty` の視覚差は付けない**: guide（破線）は ark 同様に常時
//!   表示で足り、空状態の表示切替は利用者判断（headless の `data-empty`
//!   は既に出力されており利用者 CSS で拡張可能）
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
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe,
    StateCondition,
};
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
/// `class` variant による寸法切替を重ねて設けない設計判断）。state 規則
/// （hover/focus-visible/disabled）は [`crate::recipe`] の Phase 0 共通
/// ビジュアル言語ヘルパを使い、モジュール冒頭「スタイル調整」節の
/// 是正点・意図的差分の理由に従う（イシュー #1503）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("signature-pad", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .base(
            "label",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
            ],
        )
        .base(
            "control",
            vec![
                decl("position", "relative"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-lg)"),
                decl("background", "var(--fandhe-color-bg-muted)"),
                decl("min-width", "16rem"),
                decl("min-height", "8rem"),
                decl("cursor", "crosshair"),
                decl("touch-action", "none"),
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
                decl("left", "var(--fandhe-space-3)"),
                decl("right", "var(--fandhe-space-3)"),
                decl("bottom", "var(--fandhe-space-6)"),
                decl("border-bottom", "1px dashed var(--fandhe-color-border)"),
            ],
        )
        .base("clear-trigger", {
            let mut declarations = vec![
                decl("align-self", "flex-start"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-3)"),
                decl("cursor", "pointer"),
                // 面を持たない（淡い bg のみの）ボタン系 slot のため
                // `hover_bg_muted()` で `--fandhe-hover-bg` を定義する
                // （`crate::button::ButtonVariant::Outline`/`Ghost` と同型、
                // 定義と適用〔下記 `.state(..., StateCondition::Hover, ...)`〕
                // を分離する既存パターン）。
                hover_bg_muted(),
            ];
            declarations.extend(transition_declarations(
                "background, border-color",
                MotionDuration::Fast,
            ));
            declarations
        })
        .state(
            "clear-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "clear-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // `clear-trigger`/`control` はいずれも `root` の子孫であり、`root`
        // の `[data-disabled]` へ `disabled_declarations()`（`opacity: 0.5`）
        // が付くと、同じ opacity を子孫へも重ねて `0.25` へ二重減光する
        // （`password_input`/`date_input` #1469 と同型の既存不整合、
        // イシュー #1503 PR #1776 Bugbot レビュー Medium severity 指摘
        // 対応）。他のフォームコントロールの慣例（`opacity` は `root`
        // のみに適用し、子孫は `cursor: not-allowed` のみを持つ）に合わせ、
        // `clear-trigger`/`control` は `cursor` のみへ縮小する。
        .state(
            "clear-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        // read-only な署名欄は描画操作を開始できないため、`base` の
        // `touch-action: none`（描画中のブラウザ標準パン・スクロール抑止、
        // 本モジュール冒頭 rustdoc「スタイル調整」節参照）を維持すると
        // モバイルでスクロール操作自体ができない領域になってしまう
        // （イシュー #1503 PR #1776 codex-review P1 指摘対応）。`data-
        // readonly` では `touch-action: auto`（ブラウザ標準操作を復元）・
        // `cursor: default`（`crosshair` は「描画可能」という誤った状態
        // 表示になるため非描画用カーソルへ上書き）へ差し替える。
        .state(
            "control",
            StateCondition::Attr("data-readonly"),
            vec![decl("touch-action", "auto"), decl("cursor", "default")],
        )
        // `data-readonly` 規則より後に登録する（同じ詳細度 `[data-*]`
        // 同士のため、`state()` の「登録順」契約〔`crate::recipe` の
        // `SlotRecipe::css` rustdoc「LastChild」節と同型〕により後勝ちで
        // 上書きさせる）。disabled かつ readonly の両方が真な control
        // （headless 側は独立した 2 属性として出しうる、
        // `crates/headless-ui/src/signature_pad.rs::control` 参照）で
        // `cursor: default` に上書きされ、disabled の視覚契約
        // （`not-allowed`）が失われる不具合を防ぐ（date_input #1469 と
        // 同型の判断、イシュー #1503 PR #1776 codex-review P1 / Bugbot
        // 指摘対応）。disabled と read-only は独立した属性であり
        // （`crates/headless-ui/src/signature_pad.rs::control`）、
        // disabled 単独（read-only ではない）の control でも `base` の
        // `touch-action: none` が残ったままだとモバイルでこの領域から
        // ページをパン・スクロールできなくなるため、`touch-action:
        // auto` も readonly 規則と同様にここへ含める（イシュー #1503
        // PR #1776 codex-review P1 再指摘対応）。
        .state(
            "control",
            StateCondition::Attr("data-disabled"),
            vec![decl("touch-action", "auto"), decl("cursor", "not-allowed")],
        )
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
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

    /// イシュー #1503 で追加した hover / focus-visible / disabled の
    /// state 規則を検証する（Phase 0 共通ビジュアル言語ヘルパの適用結果）。
    #[test]
    fn stylesheet_contains_interaction_visual_language_rules() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="signature-pad"][data-part="clear-trigger"]:hover:not([data-disabled])"#
        ));
        assert!(css
            .contains(r#"[data-scope="signature-pad"][data-part="clear-trigger"]:focus-visible"#));
        assert!(css
            .contains(r#"[data-scope="signature-pad"][data-part="clear-trigger"][data-disabled]"#));
        assert!(css.contains(r#"[data-scope="signature-pad"][data-part="control"][data-disabled]"#));
        assert!(css.contains(r#"[data-scope="signature-pad"][data-part="root"][data-disabled]"#));
        assert!(css.contains("--fandhe-hover-bg"));
    }

    /// イシュー #1503 PR #1776 codex-review P1 指摘対応: `data-readonly` の
    /// `control` は `touch-action: auto`／`cursor: default` へ上書きされ、
    /// `base` の `touch-action: none`／`cursor: crosshair`（描画面の操作
    /// 契約）が read-only 時に残らないことを検証する。
    #[test]
    fn stylesheet_readonly_control_restores_scroll_and_non_drawing_cursor() {
        let css = stylesheet();
        let readonly_rule_start = css
            .find(r#"[data-scope="signature-pad"][data-part="control"][data-readonly]"#)
            .expect("data-readonly control 規則が存在する");
        let readonly_rule_end = css[readonly_rule_start..]
            .find('}')
            .map(|offset| readonly_rule_start + offset)
            .expect("規則の終端 `}` が存在する");
        let readonly_rule = &css[readonly_rule_start..readonly_rule_end];
        assert!(readonly_rule.contains("touch-action: auto;"));
        assert!(readonly_rule.contains("cursor: default;"));
    }

    /// イシュー #1503 PR #1776 codex-review P1 再指摘対応: disabled と
    /// read-only は独立した属性であり、disabled 単独（read-only では
    /// ない）の `control` でも `base` の `touch-action: none`
    /// （モバイルのパン・スクロール抑止、描画中のみ有効な操作契約）が
    /// 残ったままにならないことを検証する（`control[data-disabled]`
    /// 規則が `touch-action: auto` を持つこと）。
    #[test]
    fn stylesheet_disabled_only_control_restores_scroll() {
        let css = stylesheet();
        let disabled_rule_start = css
            .find(r#"[data-scope="signature-pad"][data-part="control"][data-disabled]"#)
            .expect("data-disabled control 規則が存在する");
        let disabled_rule_end = css[disabled_rule_start..]
            .find('}')
            .map(|offset| disabled_rule_start + offset)
            .expect("規則の終端 `}` が存在する");
        let disabled_rule = &css[disabled_rule_start..disabled_rule_end];
        assert!(disabled_rule.contains("touch-action: auto;"));
        assert!(disabled_rule.contains("cursor: not-allowed;"));
    }

    /// イシュー #1503 PR #1776 Cursor Bugbot レビュー Medium severity 指摘
    /// 対応: `disabled_declarations()`（`opacity: 0.5` 込み）は `root` の
    /// `[data-disabled]` にのみ適用され、子孫 `control`/`clear-trigger` の
    /// `[data-disabled]` 規則は `opacity` を持たない（二重減光の回避）。
    #[test]
    fn stylesheet_disabled_opacity_applies_only_to_root() {
        let css = stylesheet();
        let control_rule_start = css
            .find(r#"[data-scope="signature-pad"][data-part="control"][data-disabled]"#)
            .expect("control disabled 規則が存在する");
        let control_rule_end = css[control_rule_start..]
            .find('}')
            .map(|offset| control_rule_start + offset)
            .expect("規則の終端 `}` が存在する");
        assert!(!css[control_rule_start..control_rule_end].contains("opacity"));

        let clear_trigger_rule_start = css
            .find(r#"[data-scope="signature-pad"][data-part="clear-trigger"][data-disabled]"#)
            .expect("clear-trigger disabled 規則が存在する");
        let clear_trigger_rule_end = css[clear_trigger_rule_start..]
            .find('}')
            .map(|offset| clear_trigger_rule_start + offset)
            .expect("規則の終端 `}` が存在する");
        assert!(!css[clear_trigger_rule_start..clear_trigger_rule_end].contains("opacity"));

        let root_rule_start = css
            .find(r#"[data-scope="signature-pad"][data-part="root"][data-disabled]"#)
            .expect("root disabled 規則が存在する");
        let root_rule_end = css[root_rule_start..]
            .find('}')
            .map(|offset| root_rule_start + offset)
            .expect("規則の終端 `}` が存在する");
        assert!(css[root_rule_start..root_rule_end].contains("opacity: 0.5;"));
    }

    /// イシュー #1503 PR #1776 codex-review P1 / Cursor Bugbot 重複指摘
    /// 対応: `control` が disabled かつ read-only の両方を持つとき、
    /// `data-readonly` 規則（`cursor: default`）ではなく `data-disabled`
    /// 規則（`cursor: not-allowed`）が最終的な表示カーソルとして勝つこと
    /// を検証する（date_input #1469 と同型の「登録順」契約、
    /// `crate::recipe::SlotRecipe::css` rustdoc「LastChild」節参照）。
    #[test]
    fn stylesheet_disabled_cursor_wins_over_readonly_cursor_when_both_present() {
        let css = stylesheet();
        let readonly_idx = css
            .find(r#"[data-scope="signature-pad"][data-part="control"][data-readonly] {"#)
            .expect("control readonly 規則が存在する");
        let disabled_idx = css
            .find(r#"[data-scope="signature-pad"][data-part="control"][data-disabled] {"#)
            .expect("control disabled 規則が存在する");
        assert!(
            disabled_idx > readonly_idx,
            "control[data-disabled] must be registered after control[data-readonly] so it wins by source order"
        );
        let disabled_block = &css[disabled_idx..];
        let block_end = disabled_block.find('}').unwrap_or(disabled_block.len());
        assert!(disabled_block[..block_end].contains("cursor: not-allowed;"));
    }

    /// イシュー #1503 で `control` を空でも潰れない署名欄の見た目へ
    /// 実体化したことを検証する（トークン化・最小寸法・操作性宣言）。
    #[test]
    fn stylesheet_control_has_visible_pad_declarations() {
        let css = stylesheet();
        assert!(css.contains("background: var(--fandhe-color-bg-muted);"));
        assert!(css.contains("border-radius: var(--fandhe-radius-lg);"));
        assert!(css.contains("min-width: 16rem;"));
        assert!(css.contains("min-height: 8rem;"));
        assert!(css.contains("cursor: crosshair;"));
        assert!(css.contains("touch-action: none;"));
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
            false,
            vec![],
            vec![fandhe_frontend_core::text("Sign here")],
        ));
        assert!(label_html.contains(r#"data-part="label""#));

        let guide_html = render(&guide(false, vec![], vec![]));
        assert!(guide_html.contains(r#"data-part="guide""#));

        let hidden_html = render(&hidden_input("signature", "M0.00,0.00", false, vec![]));
        assert!(hidden_html.contains(r#"data-part="hidden-input""#));
    }
}
