//! styled DateInput（headless ラッパー、イシュー #834、親 #520/#832）。
//!
//! `fandhe_frontend_headless_ui::date_input`（イシュー #834）の Label /
//! Control / SegmentGroup / Segment / HiddenInput の 5 anatomy パーツを
//! そのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。
//! 薄い委譲の根拠は [`crate::number_input`]/[`crate::pin_input`] の rustdoc
//! と同じ方針に従う。
//!
//! # 選択的 re-export（`DateInput` 型・headless `root` を再エクスポートしない
//! 理由）
//!
//! 本モジュールは `size` variant クラス付与のため styled `root`
//! （[`crate::number_input::root`] と同型）を本モジュールで再定義する。
//! headless 自由関数 `root` と名前衝突するため、`pub use ...::*` ではなく
//! 必要な識別子（[`label`]/[`control`]/[`segment_group`]/[`segment`]/
//! [`hidden_input`]/[`DateInputAction`]/[`DateSegment`]/[`DateInputProps`]）
//! のみを選択的に再エクスポートする（イシュー #1626 で headless 側が
//! `DateSegmentFlags` を全パーツ共通の `DateInputProps` へ置換したため
//! 追随した）。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::date_input::DateInput`] は
//! **あえて**再エクスポートしない（[`crate::number_input`] の `NumberInput`
//! 非再エクスポートと同じ理由）。`DateInput` は `.root(disabled, readonly,
//! attrs, children)` という inherent メソッドを持つが、これは headless
//! 自由関数 `root` へそのまま委譲するのみで `size` variant クラスを一切
//! 付与しない未スタイルの実体である。状態管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::date_input::DateInput` を直接 import し、
//! 実際の描画は本モジュールの styled [`root`]（および再エクスポート済みの
//! パーツ関数）を組み合わせて構築すること。
//!
//! # styled `root` が露出しない `readonly`/`focused`（イシュー #1626）
//!
//! headless 側の `DateInputProps` は `readonly`/`focused` を追加したが、
//! styled [`root`] のシグネチャは `(size, disabled, invalid, attrs,
//! children)` のまま維持する（#1884 の styled root 維持方針を踏襲）。
//! `readonly`/`focused` を styled `root` へ露出する拡張・対応する
//! `[data-readonly]`/`[data-focus]`（root スコープ）の CSS 追加は本イシューの
//! スコープ外とする（「本イシューのスコープ外」節参照）。
//!
//! # `size` variant（イシュー #708 方針の踏襲）
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-date-input-segment-size`/`-font-size`（root スコープの CSS
//! custom property。通常の CSS 継承により `segment-group`/`segment` へ
//! 伝わる）経由で寸法を切り替える。`color-palette` 軸は本コンポーネントでは
//! 提供しない（`crate` rustdoc「複合部品の variant 統一方針」の軸提供基準に
//! 従い、フォーム入力部品として `size` のみを対象とする）。base 規則の
//! `var()` には Md 相当のフォールバック値を書き、styled `root` を経由しない
//! headless 直接利用マークアップでも現行外観を維持する（fail-safe）。
//!
//! # フォーカスリング（`role="spinbutton"` の `div` が実フォーカスを受ける）
//!
//! [`segment`] はネイティブ `<input>` ではなく `div role="spinbutton"
//! tabindex="0"` であり、要素自身が実フォーカスを受けるため
//! [`crate::recipe::StateCondition::FocusVisible`] で足りる（hidden-input
//! パターン非該当。[`crate::splitter`] の `resize-trigger` と同型の判断）。
//! 実装は `outline`/`outline-offset` の canonical 形
//! （[`crate::recipe::focus_ring_declarations`]）を使う（イシュー #1469 で
//! `box-shadow` から移行、下記「スタイル調整」節参照）。`palette` 軸を
//! 持たない部品のため [`crate::recipe::FocusRingColor::Token`] を使う。
//! オフセットは `Inset`（要素内側）を選ぶ: `segment` は
//! `segment-group` の枠内に隙間なく並ぶため `Outside`（既定）だと隣接
//! セグメントの当たり判定・視覚的な枠と `outline` が重なり境界が不明瞭に
//! なる（[`crate::splitter`] の `resize-trigger` と同じ「祖先が
//! `overflow` を持たなくても密に並ぶ slot は inset を選ぶ」判断）。
//!
//! # スタイル調整（イシュー #1469、親 UI 部品スタイル調整ツリー #1420）
//!
//! chakra-ui / ark-ui の date-input と 7 軸で比較し是正した点・意図的に
//! 合わせなかった点を記録する（差分メモは issue #1469 のコメントにも
//! 転記する）。
//!
//! - **是正**: `segment` の `outline: none`（単独使用、`forced-colors:
//!   active` でリングが消える構成）を除去し、`:focus-visible` の
//!   リング表現を `box-shadow` から `outline` の canonical 形へ移行
//!   （上記「フォーカスリング」節）。`segment` へ `hover_bg_muted()` +
//!   `StateCondition::Hover` による hover 背景、
//!   `transition_declarations` による background/color の遷移、
//!   `padding: 0 var(--fandhe-space-1)` による当たり面・視覚余白を追加
//!   （参照 3 サイトはいずれもセグメント間・セグメントとボーダー間に
//!   小さい水平余白を持つ）。`root` の `[data-disabled]` を
//!   `disabled_declarations()`（`opacity` + `cursor: not-allowed`）へ
//!   統一。headless（`crates/headless-ui/src/date_input.rs`）が
//!   `segment` へ出す `data-readonly` を新たに `cursor: default` へ
//!   消費（[`crate::rating_group`] の同型消費と同じ判断）。disabled かつ
//!   readonly が同一 `segment` に共存する場合（headless 側は独立した
//!   2 属性のため両立しうる）、`cursor` は継承プロパティで `root`/
//!   `segment-group` の `[data-disabled]` は祖先規則にすぎず、`segment`
//!   自身への直接指定である readonly の `cursor: default` が常に勝って
//!   しまい無効コントロール上で通常カーソルに見える不整合があった
//!   （codex-review 指摘、PR #1746）。これを避けるため `segment`
//!   `[data-disabled]` へ `cursor: not-allowed` のみを readonly 規則の
//!   後段に追加し（同じ詳細度 `[name]` 同士のため登録順で後勝ちさせる）、
//!   disabled を readonly より優先させる。`opacity` は含めないため
//!   直後の「意図的に合わせなかった点」の二重減光回避とは矛盾しない。
//! - **意図的に合わせなかった点**:
//!   - **variant 軸（chakra `outline`/`subtle`/`flushed` 相当）は追加
//!     しない**。`root` のシグネチャ変更を伴う破壊的変更であり、
//!     Forms 家族横断の軸語彙判断（combobox #1467・checkbox #1454 と
//!     同一の判断軸）のため本イシュー単独では先行しない。
//!   - **`segment` 単体の `data-invalid` へは色宣言を追加しない**。
//!     `segment-group` の `[data-invalid]`（`border-color: danger`）で
//!     参照サイト相当の invalid 表現が既に成立しており、二重の視覚強調
//!     は不要と判断した。
//!   - **`segment` へ `disabled_declarations()`（`opacity` 込み）は付与
//!     しない**。`root` の disabled 状態が既に `opacity: 0.5` を子孫へ
//!     継承させるため、`segment` へも付けると `0.5 × 0.5` の二重減光に
//!     なる（`segment-group` の `[data-disabled]` 側 `cursor:
//!     not-allowed` は headless 直接利用時の fail-safe として維持）。
//!     `segment` `[data-disabled]` へは上記「是正」節のとおり `cursor:
//!     not-allowed` のみを別途追加しており、`opacity` は含めないため
//!     二重減光は生じない。
//!   - **サイズ / バリアント（5 段）・色（トークン参照のみ）・ダーク
//!     （`--fandhe-color-focus-ring` 等が追従済み）は元々参照サイト水準に
//!     達していたため変更しない**。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - headless 層と同じく granularity（hour/minute/second）・range 選択・
//!   locale 依存整形・キーボード操作の DOM 配線はスコープ外
//!   （`fandhe_frontend_headless_ui::date_input` モジュール doc 参照）。
//! - `examples/headless-pre-styled-ui`（crates.io バージョン依存）への
//!   DateInput 追加は、未公開の新バージョンを参照できないため本イシューの
//!   スコープ外とする（9c0e4f6 の先例どおり crates.io 公開後に追随）。
//! - variant 軸（chakra `outline`/`subtle`/`flushed` 相当）の追加は
//!   上記「スタイル調整」節のとおり本イシューのスコープ外とする。
//! - styled `root` への `readonly`/`focused` 引数露出（イシュー #1626、
//!   上記「styled `root` が露出しない `readonly`/`focused`」節参照）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe,
    StateCondition, VariantValue,
};

// `DateInput` 状態機械・headless 自由関数 `root` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。状態管理・
// hydration が必要な呼び出し側は
// `fandhe_frontend_headless_ui::date_input::DateInput` を直接 import する。
pub use fandhe_frontend_headless_ui::date_input::{
    control, hidden_input, label, segment, segment_group, DateInputAction, DateInputProps,
    DateSegment,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// headless `date_input` anatomy の `data-part` 一覧（`crates/headless-ui/src/date_input.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`]
/// が一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "control",
    "segment-group",
    "segment",
    "hidden-input",
];

/// この styled DateInput の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("date-input", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("flex-direction", "column"),
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
            vec![decl(
                "font-size",
                "var(--fandhe-date-input-font-size, var(--fandhe-font-font-size-sm))",
            )],
        )
        .base(
            "control",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
            ],
        )
        .base(
            "segment-group",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md, 0.375rem)"),
                decl("padding", "0 var(--fandhe-space-2)"),
                decl("background", "var(--fandhe-color-bg)"),
            ],
        )
        .state(
            "segment-group",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        .state(
            "segment-group",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        .base(
            "segment",
            vec![
                decl("box-sizing", "border-box"),
                decl("height", "var(--fandhe-date-input-segment-size, 2.5rem)"),
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl(
                    "font-size",
                    "var(--fandhe-date-input-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("border-radius", "var(--fandhe-radius-sm, 0.25rem)"),
                decl("padding", "0 var(--fandhe-space-1)"),
                hover_bg_muted(),
            ],
        )
        // 別 `.base` 呼び出しでの純追加（イシュー #1469。combobox #1744 の
        // 「既存 base ブロックを書き換えない」パターンを踏襲する）。
        .base(
            "segment",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .state(
            "segment",
            // イシュー #1626: headless 側が ark-ui Data Attributes 表の語彙
            // （`data-placeholder-shown`）へ改名したため追随（見た目は不変）。
            StateCondition::Attr("data-placeholder-shown"),
            vec![decl("color", "var(--fandhe-color-fg-muted)")],
        )
        .state(
            "segment",
            StateCondition::Attr("data-readonly"),
            vec![decl("cursor", "default")],
        )
        // `data-readonly` 規則より後に登録する（同じ詳細度 `[name]`
        // (0,1,0) 同士のため、`state()` の「登録順」契約〔`crate::recipe`
        // の `SlotRecipe::css` rustdoc「LastChild」節と同型〕により
        // 後勝ちで上書きさせる）。disabled かつ readonly の両方が真な
        // segment（headless 側は独立した 2 属性として出しうる、
        // `crates/headless-ui/src/date_input.rs::segment` 参照）で
        // `cursor: default` に上書きされ通常カーソルへ戻ってしまう
        // 不具合を防ぐ（codex-review 指摘、イシュー #1469 PR #1746）。
        // `root`/`segment-group` の `[data-disabled]` は継承値のため、
        // 同一要素に直接付く本規則がなければ readonly の直接指定が
        // 常に勝ってしまう。
        .state(
            "segment",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        .state(
            "segment",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "segment",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Inset),
        )
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-date-input-segment-size", "1.5rem"),
                decl(
                    "--fandhe-date-input-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-date-input-segment-size", "2rem"),
                decl(
                    "--fandhe-date-input-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-date-input-segment-size", "2.5rem"),
                decl(
                    "--fandhe-date-input-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-date-input-segment-size", "3rem"),
                decl(
                    "--fandhe-date-input-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-date-input-segment-size", "3.5rem"),
                decl(
                    "--fandhe-date-input-font-size",
                    "var(--fandhe-font-font-size-lg)",
                ),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled DateInput が生成する静的 CSS 全量を返す（決定的。
/// [`crate::number_input::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::date_input::root`]
/// へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::date_input;
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = date_input::root(Size::Md, false, false, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="date-input" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    disabled: bool,
    invalid: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    let props = fandhe_frontend_headless_ui::date_input::DateInputProps {
        disabled,
        invalid,
        ..fandhe_frontend_headless_ui::date_input::DateInputProps::default()
    };
    fandhe_frontend_headless_ui::date_input::root(props, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="date-input"][data-part="segment"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_links_segment_to_invalid_and_placeholder_states() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="date-input"][data-part="segment-group"][data-invalid] {"#)
        );
        assert!(css.contains(
            r#"[data-scope="date-input"][data-part="segment"][data-placeholder-shown] {"#
        ));
    }

    #[test]
    fn stylesheet_links_segment_to_focus_visible() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="date-input"][data-part="segment"]:focus-visible {"#));
        // イシュー #1469: `box-shadow` ではなく canonical `outline` 形
        // （`FocusRingColor::Token`・`FocusRingOffset::Inset`）を検証する。
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(css.contains("outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));"));
        assert!(!css.contains("box-shadow"));
        // `outline: none` 単独使用（forced-colors でリングが消える構成）は
        // 除去済みであることも合わせて確認する。
        assert!(!css.contains("outline: none"));
    }

    #[test]
    fn segment_hover_rule_is_wrapped_in_hover_media_query() {
        let css = stylesheet();
        let media_idx = css
            .find("@media (hover: hover) {")
            .expect("hover media query block must exist");
        let media_block = &css[media_idx..];
        assert!(media_block.contains(
            r#"[data-scope="date-input"][data-part="segment"]:hover:not([data-disabled]) {"#
        ));
        assert!(media_block.contains("background: var(--fandhe-hover-bg);"));
    }

    #[test]
    fn root_disabled_uses_canonical_disabled_declarations() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="date-input"][data-part="root"][data-disabled] {"#));
        assert!(css.contains("opacity: 0.5;"));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn segment_consumes_data_readonly_attribute() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="date-input"][data-part="segment"][data-readonly] {"#));
        assert!(css.contains("cursor: default;"));
    }

    #[test]
    fn segment_disabled_cursor_overrides_readonly_by_source_order() {
        // イシュー #1469 PR #1746 codex-review 指摘: disabled かつ readonly が
        // 同一 segment に共存する場合、`cursor` は継承プロパティのため
        // `root`/`segment-group` 側の `[data-disabled]`（祖先規則）では
        // readonly の直接指定に勝てない。`segment` 自身への
        // `[data-disabled]` 規則を readonly 規則より後段に登録することで
        // 同一詳細度・登録順の後勝ちにより disabled を優先させる。
        let css = stylesheet();
        let readonly_idx = css
            .find(r#"[data-scope="date-input"][data-part="segment"][data-readonly] {"#)
            .expect("segment readonly rule must exist");
        let disabled_idx = css
            .find(r#"[data-scope="date-input"][data-part="segment"][data-disabled] {"#)
            .expect("segment disabled rule must exist");
        assert!(
            disabled_idx > readonly_idx,
            "segment[data-disabled] must be registered after segment[data-readonly] so it wins by source order"
        );
        let disabled_block = &css[disabled_idx..];
        let block_end = disabled_block.find('}').unwrap_or(disabled_block.len());
        assert!(disabled_block[..block_end].contains("cursor: not-allowed;"));
    }

    #[test]
    fn segment_base_declares_padding_and_transition() {
        let css = stylesheet();
        assert!(css.contains("padding: 0 var(--fandhe-space-1);"));
        assert!(css.contains("transition-property: background, color;"));
    }

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(Size::Md, false, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="date-input""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md() {
        let html = render(&root(Size::Md, false, false, vec![], vec![]));
        assert!(html.contains("fd-date-input--size-md"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-date-input--size-xs"),
            (Size::Sm, "fd-date-input--size-sm"),
            (Size::Md, "fd-date-input--size-md"),
            (Size::Lg, "fd-date-input--size-lg"),
            (Size::Xl, "fd-date-input--size-xl"),
        ] {
            let html = render(&root(size, false, false, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            false,
            false,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            Size::Md,
            false,
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="date-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            Size::Md,
            false,
            false,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_label_children_are_escaped_on_render() {
        let html = render(&label(
            DateInputProps::default(),
            None,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn reexported_hidden_input_name_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&hidden_input(PAYLOAD, "2026-07-22", false, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_date_input_state_machine() {
        // `DateInput` は本モジュールから再エクスポートしない（本モジュール
        // 冒頭の rustdoc「選択的 re-export」節参照）ため、headless-ui から
        // 直接 import して state machine 契約のみ検証する。
        use fandhe_frontend_headless_ui::date_input::DateInput;

        let mut d = DateInput::new(Some(2026), Some(7), Some(22), None, None);
        assert_eq!(d.year(), Some(2026));

        let ssr_html = render(&d.control(false, false, vec![], vec![]));
        assert!(ssr_html.contains(r#"data-part="control""#));

        assert!(dispatch(&mut d, "clear", ""));
        let hydrate_html = render(&render_for_hydration(&d));
        assert!(hydrate_html.contains(r#"data-hydrate-year="none""#));

        let restored = DateInput::from_hydration_attrs(&d.hydration_attrs()).unwrap();
        assert_eq!(restored, d);
    }
}
