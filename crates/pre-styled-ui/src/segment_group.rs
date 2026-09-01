//! styled SegmentGroup（headless ラッパー、イシュー #743、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::segment_group`（イシュー #743）の Indicator /
//! Item / ItemControl / ItemText / ItemHiddenInput 5 anatomy パーツと
//! [`fandhe_frontend_headless_ui::segment_group::SegmentGroup`] 状態機械を
//! そのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い
//! 委譲の根拠・スコープ外事項は [`crate::radio_group`] の rustdoc と同じ
//! 方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! 本モジュールは `size` variant クラス付与のため styled `root`
//! （[`crate::radio_group::root`]・[`crate::number_input::root`] と同型）を
//! 本モジュールで再定義する。headless 自由関数 `root` と名前衝突するため、
//! `pub use ...::*` ではなく必要な識別子（[`item`]/[`item_control`]/
//! [`item_text`]/[`item_hidden_input`]/[`indicator`]/[`SegmentGroup`]/
//! [`DATA_STATE_CHECKED`]/[`DATA_STATE_UNCHECKED`]）のみを選択的に
//! 再エクスポートする。
//!
//! [`SegmentGroup`] 状態機械は inherent `root()` を持たない（item/indicator
//! 系メソッドのみ、`crates/headless-ui/src/segment_group.rs` 参照）ため、
//! [`crate::radio_group`] の `RadioGroup` 再エクスポートと同じ根拠で、
//! そのまま再エクスポートしても未スタイル root の静かな適用漏れは発生しない。
//!
//! # item-hidden-input の視覚的非表示化（[`crate::radio_group`] と同じ責務分担）
//!
//! headless 層はネイティブ `<input type="radio">` に `type`/`value`/`name`/
//! `checked`/`disabled`/`data-state` のみを設定し、視覚的な非表示化は行わない
//! 契約になっている。本モジュールが visually-hidden パターン（`position:
//! absolute` + 1px クリップ、[`crate::radio_group`] の `item-hidden-input`
//! 規則と同一の 9 宣言）で覆い隠し、`item-control` をカスタムセグメント枠と
//! して描画する。フォーム送信・キーボード操作・グループ内排他選択はネイティブ
//! semantics のまま維持される（headless 側モジュール doc 参照）。
//!
//! # Indicator の位置表現とスタイル連動（イシュー #1498 で幾何を是正）
//!
//! [`fandhe_frontend_headless_ui::segment_group::indicator`] が出力する
//! `--fandhe-segment-group-index`/`--fandhe-segment-group-count` CSS 変数を
//! 前提に、等幅セグメントの
//! `width: calc((100% - 2 * space) / var(--fandhe-segment-group-count))` と
//! `transform: translateX(calc(100% * var(--fandhe-segment-group-index)))`
//! （`data-orientation="vertical"` のときは `height`/translateY の対称形）
//! で位置を表現する。`translateX(100% * index)` は自身の幅を単位とする
//! 移動量であり、幅が項目 1 個分の実寸（`(root 内側幅) / count`）と一致して
//! はじめて `index` 番目の項目位置へ正しく到達する。イシュー #1498 以前の
//! 式（`calc(100% / count - space)`）は幅が項目幅より `space` 分小さく、
//! `count = 2` のときのみ偶然両者が一致し `count >= 3` で到達位置が徐々に
//! ドリフトしていた（是正のみで CSS 変数名・セレクタは変更していない）。
//! `indicator[data-state="unchecked"]`（未選択、headless 層が `style` 属性
//! 自体を省略する状態）は `display: none` にして描画しない。移動アニメー
//! ションは [`crate::recipe::transition_declarations`]（イシュー #1425）を
//! 経由し、`Theme::to_css` の `prefers-reduced-motion: reduce` 一括無効化に
//! 従う。
//!
//! # hover・フォーカスリング（イシュー #1425/#1424 の canonical 化）
//!
//! `item` は [`crate::recipe::hover_surface_declarations`] による面色変化を
//! `StateCondition::HoverExcept("data-state", "checked")` で持つ（checked
//! 項目は indicator が下に描画されているため、hover 面を重ねて見た目が
//! 濁るのを避ける。disabled 項目の除外は `HoverExcept` 自体が行う）。
//!
//! `item-hidden-input` を視覚的に隠すため、[`crate::radio_group`] と同じく
//! `item`（`<label>`、input の祖先）へ `:focus-within` を当てる（no-JS
//! フォールバック）。加えて headless 層の `data-focus-visible`（イシュー
//! #709 の契約、`crates/headless-ui/src/data_attrs.rs` 参照）を `item-control`
//! slot の状態規則として追加する（`fandhe-frontend-wasm-full` の focus 配線
//! 接続は別イシューのスコープ、下記「本イシューのスコープ外」参照）。両者
//! とも [`crate::recipe::focus_ring_declarations`]（イシュー #1424、`--fandhe-
//! focus-ring-*`/`--fandhe-color-focus-ring` トークン経由）へ canonical 化
//! した。`item-control` base は `display: contents` でありボックスを生成
//! しないため、この slot への outline は現行実装では描画されない
//! （canonical 化のみに留め、視覚的な主経路は `item` の `:focus-within` が
//! 担う。ボックス化を伴う是正は本イシューのスコープ外とする）。
//!
//! # `size` variant（`color-palette` 軸は非提供）
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! root スコープ custom property（`--fandhe-segment-group-font-size`/
//! `-padding-block`/`-padding-inline`）経由でセグメントの寸法・文字サイズを
//! 切り替える。`color-palette` 軸は提供しない（chakra-ui SegmentedControl の
//! 既定が中立色であること、[`crate::number_input`] の「フォーム操作部品として
//! `size` のみを対象とする」軸提供基準と同じ判断。選択状態は
//! indicator の移動 + テキストの色の階層で表現する）。base/state 規則の `var()` には
//! いずれも Md サイズ相当のフォールバック値を書き、styled `root` を経由しない
//! headless 直接利用マークアップでも既定外観を維持する（fail-safe）。
//!
//! # スタイル調整（イシュー #1499、親 #1497 分割 2/2）
//!
//! 分割 1/2（#1498）のインジケータ幾何是正に続き、残りの size /
//! orientation バリアントと項目ラベル（item-text）の型階層を親イシュー
//! #1497 の 7 軸チェックリストに沿って調整した。
//!
//! - **size バリアントの一括登録化**: 5 連の
//!   `.variant(Size::*, "root", ...)` + 末尾 `.default_variant(Size::Md)`
//!   を [`crate::recipe::SlotRecipe::size_variants`]（イシュー #1424 の
//!   共通生成手段、`crate::radio_group`/`crate::checkbox` と同型）へ
//!   置換した。既定 `md` の設定漏れを構造的に防ぐ（規約は `docs/design/
//!   pre-styled-ui-focus-ring-and-size-conventions.md` §4）。5 段の宣言値
//!   （font-size / padding-block / padding-inline）は既に単調で 2px/4px
//!   格子に載っており是正対象がないため現状維持とした。
//! - **項目ラベル（item-text）の型階層**: 未選択項目を
//!   `var(--fandhe-color-fg-muted)`、選択項目を `var(--fandhe-color-fg)`
//!   の 2 段階へ是正した（参照 3 サイト中 chakra-ui / Radix Themes は
//!   選択項目を中立色で表現しており、accent 色採用は ark-ui のみ。本節
//!   冒頭「選択状態は indicator の移動 + テキストの色の階層で表現する」
//!   という既存の設計宣言とも整合する）。あわせて生 `font-weight: 600`
//!   リテラルを廃し、全項目へ `font-weight: var(--fandhe-font-font-weight-
//!   medium)` を付与した（選択の強調は色差 + indicator が担う）。
//!   `line-height: var(--fandhe-font-line-height-normal)` と
//!   `user-select: none`（`<label>` クリックでの選択トグルに伴う誤選択
//!   防止）は [`crate::radio_group`]/[`crate::checkbox_group`] の
//!   item-text と同型で追加した。
//! - **orientation**: vertical の CSS（root `flex-direction: column`・
//!   indicator の translateY 対称形）は #1498 で成立済みのため変更して
//!   いない。本イシューの担当は docs-site showcase での可視化に留まる。
//!
//! **本イシューで意図的に合わせなかった点**
//! （`.claude/rules/code-comment-style.md`）:
//!
//! - **項目の等幅（`flex: 1`）を chakra の content-width に合わせない**:
//!   indicator の CSS 変数幾何（`width = (root 内側幅) / count`・
//!   `translateX(100% * index)`）が全項目等幅を前提とするため。
//!   content-width 化は indicator 位置決めの再設計（JS 計測）を要し、
//!   無 JS・決定的 SSR の設計前提と衝突する。
//! - **size 5 段（xs〜xl）を chakra 4 段（xs〜lg）/ Radix Themes 3 段
//!   （1〜3）に揃えない**: 本リポジトリの Forms 部品共通の 5 段語彙
//!   （`pre-styled-ui-focus-ring-and-size-conventions.md` §4）を優先する。
//! - **vertical orientation での項目テキスト左寄せ化は行わない**（中央
//!   揃えを維持）: 参照サイトに明確な根拠が確認できなかったため。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは headless 層の再エクスポートと静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラルで
//! あり、動的値（`value`/`name`/属性/children）へ CSS 値として流し込む経路
//! を持たない（動的値は headless 層経由で `fandhe_frontend_core::render` の
//! 既定エスケープを必ず通る、REQ-1）。styled `root` は [`drop_class_attr`]
//! により呼び出し側の `class` を除去してから合成するため、`class` 属性は
//! 常に単一（[`crate::radio_group::root`] と同型）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `fandhe-frontend-wasm-full` の CSR 配線（`(scope, part) =
//!   ("segment-group", "item") -> "select"` の静的マッピング表追加・
//!   focus_visible 配線・dispatch 後の indicator CSS 変数の動的更新）は
//!   headless 層と同じく未着手（別イシューでの追跡を提案する）。
//! - 矢印キーによる roving tabindex・chakra 拡張 sub-parts（`Label`/`Items`）
//!   ・`readOnly`・`xs` サイズは本イシューのスコープ外。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe,
    StateCondition, VariantValue,
};

// headless 自由関数 `root` はあえて再エクスポートしない（本モジュール冒頭
// の rustdoc「選択的 re-export」節参照、`root` は本モジュールで styled 版
// として再定義する）。
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::segment_group::{
    indicator, item, item_control, item_hidden_input, item_text, SegmentGroup, DATA_STATE_CHECKED,
    DATA_STATE_UNCHECKED,
};

/// headless `segment_group` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/segment_group.rs` の `ANATOMY.part(...)`
/// 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &[
    "root",
    "indicator",
    "item",
    "item-control",
    "item-text",
    "item-hidden-input",
];

/// この styled SegmentGroup の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("segment-group", SLOTS)
        .base(
            "root",
            vec![
                decl("position", "relative"),
                decl("display", "inline-flex"),
                decl("background", "var(--fandhe-color-bg-muted)"),
                decl("border-radius", "var(--fandhe-radius-md, 0.375rem)"),
                decl("padding", "var(--fandhe-space-1, 0.25rem)"),
            ],
        )
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            vec![decl("opacity", "0.5")],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![decl("flex-direction", "column")],
        )
        .base(
            "indicator",
            vec![
                decl("position", "absolute"),
                decl("z-index", "0"),
                decl("top", "var(--fandhe-space-1, 0.25rem)"),
                decl("left", "var(--fandhe-space-1, 0.25rem)"),
                decl(
                    "width",
                    "calc((100% - 2 * var(--fandhe-space-1, 0.25rem)) / var(--fandhe-segment-group-count, 1))",
                ),
                decl(
                    "height",
                    "calc(100% - 2 * var(--fandhe-space-1, 0.25rem))",
                ),
                decl(
                    "transform",
                    "translateX(calc(100% * var(--fandhe-segment-group-index, 0)))",
                ),
                decl("background", "var(--fandhe-color-bg)"),
                decl("border-radius", "var(--fandhe-radius-sm, 0.25rem)"),
                decl("box-shadow", "var(--fandhe-shadow-sm)"),
            ],
        )
        // トランジションの canonical 化（イシュー #1425）。`--fandhe-motion-
        // duration-fast` トークン経由になり、`Theme::to_css` の
        // `prefers-reduced-motion: reduce` 一括無効化（duration を 0ms へ）
        // が効くようになる（生の `0.15s ease` リテラルのままでは対象外
        // だった）。
        .base(
            "indicator",
            transition_declarations("transform", MotionDuration::Fast),
        )
        .state(
            "indicator",
            StateCondition::AttrEq("data-state", "unchecked"),
            vec![decl("display", "none")],
        )
        .state(
            "indicator",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl(
                    "width",
                    "calc(100% - 2 * var(--fandhe-space-1, 0.25rem))",
                ),
                decl(
                    "height",
                    "calc((100% - 2 * var(--fandhe-space-1, 0.25rem)) / var(--fandhe-segment-group-count, 1))",
                ),
                decl("transform", "translateY(calc(100% * var(--fandhe-segment-group-index, 0)))"),
            ],
        )
        .base(
            "item",
            vec![
                decl("position", "relative"),
                decl("z-index", "1"),
                decl("flex", "1"),
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("cursor", "pointer"),
                decl("border-radius", "var(--fandhe-radius-sm, 0.25rem)"),
                decl(
                    "padding-block",
                    "var(--fandhe-segment-group-padding-block, 0.375rem)",
                ),
                decl(
                    "padding-inline",
                    "var(--fandhe-segment-group-padding-inline, 0.75rem)",
                ),
                // hover 面の色（`crate::recipe` 冒頭 doc「disabled / hover /
                // transition の共通ビジュアル言語」節と同型の間接参照設計）。
                // root 面が `bg-muted` のため、`hover_bg_muted()` が返す
                // `bg-muted` では視覚差が出ない。1 段強い `bg-emphasized` を
                // 直接指定する（他部品の `hover_bg_muted`/`hover_bg_solid`
                // のいずれにも該当しない segment-group 固有の面色関係）。
                decl("--fandhe-hover-bg", "var(--fandhe-color-bg-emphasized)"),
            ],
        )
        .base(
            "item",
            transition_declarations("background", MotionDuration::Fast),
        )
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // checked（選択中）の item は indicator が下に描画されるため hover
        // 面を出さない（`HoverExcept("data-state", "checked")` で除外。
        // `disabled` の除外は `HoverExcept`/`Hover` いずれも自動で行う、
        // `crate::recipe::StateCondition::Hover` rustdoc 参照）。
        .state(
            "item",
            StateCondition::HoverExcept("data-state", "checked"),
            hover_surface_declarations(),
        )
        .state(
            "item",
            StateCondition::FocusWithin,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .base(
            "item-control",
            vec![decl("display", "contents")],
        )
        .state(
            "item-control",
            StateCondition::Attr("data-focus-visible"),
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // イシュー #1499: 未選択 muted / 選択 fg の 2 段階の型階層へ是正
        // （参照 3 サイト中 chakra-ui / Radix Themes は選択項目を中立色で
        // 表現しており、本モジュール冒頭 rustdoc の「選択状態は indicator の
        // 移動 + テキストの色の階層で表現する」宣言とも整合する）。
        // `font-weight: medium` は全項目へ付与し（chakra 相当）、選択の
        // 強調は色差 + indicator が担う。`line-height`/`user-select` は
        // `crate::radio_group`/`crate::checkbox_group` の item-text と同型
        // （`<label>` クリックでの誤選択防止）。
        .base(
            "item-text",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl(
                    "font-size",
                    "var(--fandhe-segment-group-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("user-select", "none"),
            ],
        )
        .state(
            "item-text",
            StateCondition::AttrEq("data-state", "checked"),
            vec![decl("color", "var(--fandhe-color-fg)")],
        )
        .base(
            "item-hidden-input",
            vec![
                decl("position", "absolute"),
                decl("width", "1px"),
                decl("height", "1px"),
                decl("padding", "0"),
                decl("margin", "-1px"),
                decl("overflow", "hidden"),
                decl("clip", "rect(0, 0, 0, 0)"),
                decl("white-space", "nowrap"),
                decl("border", "0"),
            ],
        )
        // イシュー #1499: 5 連の `.variant(Size::*, "root", ...)` +
        // 末尾 `.default_variant(Size::Md)` の手書きから、共通生成手段
        // `size_variants`（イシュー #1424、`crate::radio_group`/
        // `crate::checkbox` と同型）へ置換した。既定 `md` の設定漏れを
        // 構造的に防ぐ（`docs/design/
        // pre-styled-ui-focus-ring-and-size-conventions.md` §4）。宣言値
        // （font-size / padding-block / padding-inline）は現状維持（既に
        // 単調で 2px/4px 格子に載っており是正対象がない）。
        .size_variants(
            "root",
            &[
                (
                    Size::Xs,
                    vec![
                        decl(
                            "--fandhe-segment-group-font-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                        decl("--fandhe-segment-group-padding-block", "0.125rem"),
                        decl("--fandhe-segment-group-padding-inline", "0.25rem"),
                    ],
                ),
                (
                    Size::Sm,
                    vec![
                        decl(
                            "--fandhe-segment-group-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl("--fandhe-segment-group-padding-block", "0.25rem"),
                        decl("--fandhe-segment-group-padding-inline", "0.5rem"),
                    ],
                ),
                (
                    Size::Md,
                    vec![
                        decl(
                            "--fandhe-segment-group-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl("--fandhe-segment-group-padding-block", "0.375rem"),
                        decl("--fandhe-segment-group-padding-inline", "0.75rem"),
                    ],
                ),
                (
                    Size::Lg,
                    vec![
                        decl(
                            "--fandhe-segment-group-font-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                        decl("--fandhe-segment-group-padding-block", "0.5rem"),
                        decl("--fandhe-segment-group-padding-inline", "1rem"),
                    ],
                ),
                (
                    Size::Xl,
                    vec![
                        decl(
                            "--fandhe-segment-group-font-size",
                            "var(--fandhe-font-font-size-lg)",
                        ),
                        decl("--fandhe-segment-group-padding-block", "0.625rem"),
                        decl("--fandhe-segment-group-padding-inline", "1.25rem"),
                    ],
                ),
            ],
        )
}

/// この styled SegmentGroup が生成する静的 CSS 全量を返す（決定的。
/// [`crate::radio_group::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は
/// [`fandhe_frontend_headless_ui::segment_group::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::segment_group;
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = segment_group::root(Size::Md, false, None, None, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="segment-group" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    disabled: bool,
    orientation: Option<Orientation>,
    labelled_by: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::segment_group::root(
        disabled,
        orientation,
        labelled_by,
        merged,
        children,
    )
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
        assert!(a.contains(r#"[data-scope="segment-group"][data-part="indicator"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn item_hidden_input_is_visually_hidden() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="segment-group"][data-part="item-hidden-input"]"#));
        assert!(css.contains("clip: rect(0, 0, 0, 0);"));
        assert!(css.contains("position: absolute;"));
    }

    #[test]
    fn indicator_uses_css_vars_for_width_and_transform() {
        let css = stylesheet();
        assert!(css.contains("var(--fandhe-segment-group-count, 1)"));
        assert!(css.contains("var(--fandhe-segment-group-index, 0)"));
    }

    #[test]
    fn indicator_width_formula_normalizes_by_padded_inner_size_before_dividing() {
        // イシュー #1498: `translateX(100% * index)` は自身の幅を単位とする
        // 移動量のため、幅が「(root 内側幅) / count」の項目幅と一致しない
        // 限り count >= 3 で到達位置がドリフトする（本モジュール冒頭 rustdoc
        // 「Indicator の位置表現とスタイル連動」節参照）。是正後の式
        // （先に space を引いてから count で割る）が出力されることを固定
        // する。
        let css = stylesheet();
        assert!(css.contains(
            "width: calc((100% - 2 * var(--fandhe-space-1, 0.25rem)) / var(--fandhe-segment-group-count, 1));"
        ));
        assert!(css.contains(
            "height: calc((100% - 2 * var(--fandhe-space-1, 0.25rem)) / var(--fandhe-segment-group-count, 1));"
        ));
        // 是正前の式（count で割ってから space を引く）が残っていないこと。
        assert!(!css.contains("calc(100% / var(--fandhe-segment-group-count, 1) - "));
    }

    #[test]
    fn indicator_transition_uses_canonical_motion_tokens() {
        // イシュー #1425 の `transition_declarations` へ canonical 化した
        // ことで `prefers-reduced-motion: reduce` 一括無効化が効くように
        // なる（生の `transition: transform 0.15s ease` リテラルのままでは
        // duration が別トークン経由にならず対象外だった）。
        let css = stylesheet();
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
        assert!(!css.contains("0.15s ease"));
    }

    #[test]
    fn indicator_shadow_has_no_raw_color_fallback() {
        // 他部品（card / angle-slider / image-cropper）が収斂済みの
        // フォールバックなし `var(--fandhe-shadow-sm)` へ揃える（イシュー
        // #1498）。
        let css = stylesheet();
        assert!(css.contains("box-shadow: var(--fandhe-shadow-sm);"));
        assert!(!css.contains("rgba(0, 0, 0, 0.1)"));
    }

    #[test]
    fn indicator_unchecked_state_is_hidden() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="segment-group"][data-part="indicator"][data-state="unchecked"]"#
        ));
        assert!(css.contains("display: none;"));
    }

    #[test]
    fn indicator_vertical_orientation_switches_to_translate_y() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="segment-group"][data-part="indicator"][data-orientation="vertical"]"#
        ));
        assert!(css.contains("translateY(calc(100% * var(--fandhe-segment-group-index, 0)))"));
    }

    #[test]
    fn disabled_item_gets_not_allowed_cursor() {
        // イシュー #1498: `crate::recipe::disabled_declarations()`
        // （共通ビジュアル言語、宣言順は opacity → cursor）へ canonical
        // 化した。宣言内容は既存の ad-hoc 実装と同値。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="segment-group"][data-part="item"][data-disabled]"#));
        assert!(css.contains("opacity: 0.5;"));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn checked_item_text_gets_type_hierarchy_emphasis() {
        // イシュー #1499: 選択の強調は色（中立 fg）で表現し、生の
        // `font-weight: 600` リテラルは使わない（未選択は `fg-muted`、
        // 選択は `fg` の 2 段階。ark-ui のみが accent 色を採用するが
        // chakra-ui / Radix Themes は中立色であり、本モジュール冒頭
        // rustdoc の設計宣言とも整合する）。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="segment-group"][data-part="item-text"][data-state="checked"]"#
        ));
        let checked_body = extract_rule_body(
            &css,
            r#"[data-scope="segment-group"][data-part="item-text"][data-state="checked"] {"#,
        );
        assert!(checked_body.contains("color: var(--fandhe-color-fg);"));
        assert!(!checked_body.contains("font-weight:"));
        // フォーカスリングのフォールバック（`focus_ring_declarations`）は
        // `--fandhe-color-accent` を経由し続けるため、checked 規則本体
        // （上で抜き出した `checked_body`）に限定して accent 色不使用を
        // 確認する。
        assert!(!checked_body.contains("var(--fandhe-color-accent)"));
    }

    #[test]
    fn item_text_base_gets_muted_color_and_type_hierarchy_declarations() {
        // イシュー #1499: 未選択項目は `fg-muted`（型階層の下段）。
        // `font-weight: medium`・`line-height: normal`・`user-select: none`
        // は `crate::radio_group`/`crate::checkbox_group` の item-text と
        // 同型（`<label>` クリックでの誤選択防止）。
        let css = stylesheet();
        let base_body = extract_rule_body(
            &css,
            r#"[data-scope="segment-group"][data-part="item-text"] {"#,
        );
        assert!(base_body.contains("color: var(--fandhe-color-fg-muted);"));
        assert!(base_body.contains("font-weight: var(--fandhe-font-font-weight-medium);"));
        assert!(base_body.contains("line-height: var(--fandhe-font-line-height-normal);"));
        assert!(base_body.contains("user-select: none;"));
    }

    /// `selector {` からその直後の最初の `}` までの規則本体を抜き出す
    /// （テスト専用ヘルパ。複数セレクタが同一先頭文字列を部分一致するのを
    /// 避けるため、`checked_item_text_gets_type_hierarchy_emphasis` と
    /// `item_text_base_gets_muted_color_and_type_hierarchy_declarations`
    /// が個別ルールの宣言集合を厳密に検証する目的で使う）。
    fn extract_rule_body<'a>(css: &'a str, selector_with_brace: &str) -> &'a str {
        let start = css
            .find(selector_with_brace)
            .unwrap_or_else(|| panic!("selector not found: {selector_with_brace}"))
            + selector_with_brace.len();
        let end = css[start..]
            .find('}')
            .unwrap_or_else(|| panic!("closing brace not found after: {selector_with_brace}"));
        &css[start..start + end]
    }

    #[test]
    fn item_focus_within_gets_canonical_focus_ring_tokens() {
        // イシュー #1424 の `focus_ring_declarations` へ canonical 化した
        // ことを固定する（生の `2px solid var(--fandhe-color-accent)` の
        // 手書きから、太さ・色・オフセットをテーマ 1 箇所で変更できる
        // `--fandhe-focus-ring-*`/`--fandhe-color-focus-ring` トークン経由へ
        // 移行）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="segment-group"][data-part="item"]:focus-within {"#));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
    }

    #[test]
    fn item_control_focus_visible_gets_canonical_focus_ring_tokens() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="segment-group"][data-part="item-control"][data-focus-visible]"#
        ));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(css.contains("outline-offset: var(--fandhe-focus-ring-offset, 2px);"));
    }

    #[test]
    fn item_hover_shows_emphasized_surface_except_when_checked() {
        // hover フィードバックの追加（親イシュー #1497 が実測で指摘した
        // 代表的欠落）。checked 項目は indicator が下にあるため hover 面を
        // 出さない（`HoverExcept("data-state", "checked")`）。タッチ端末の
        // hover 貼り付き対策として `@media (hover: hover)` 配下へ集約
        // 出力される（`crate::recipe::StateCondition::HoverExcept` 参照）。
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(
            r#"[data-scope="segment-group"][data-part="item"]:hover:not([data-disabled]):not([data-state="checked"]) {"#
        ));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
        assert!(css.contains("--fandhe-hover-bg: var(--fandhe-color-bg-emphasized);"));
    }

    // --- variant クラス ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(Size::Md, false, None, None, vec![], vec![]));
        assert!(html.contains(r#"data-scope="segment-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="radiogroup""#));
    }

    #[test]
    fn default_variant_is_md() {
        let html = render(&root(Size::Md, false, None, None, vec![], vec![]));
        assert!(html.contains("fd-segment-group--size-md"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-segment-group--size-xs"),
            (Size::Sm, "fd-segment-group--size-sm"),
            (Size::Md, "fd-segment-group--size-md"),
            (Size::Lg, "fd-segment-group--size-lg"),
            (Size::Xl, "fd-segment-group--size-xl"),
        ] {
            let html = render(&root(size, false, None, None, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn reexported_root_with_vertical_orientation_emits_data_orientation() {
        let html = render(&root(
            Size::Md,
            false,
            Some(Orientation::Vertical),
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-orientation="vertical""#));
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            false,
            None,
            None,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_contains_size_variant_selectors() {
        let css = stylesheet();
        assert!(css.contains("--size-"));
        assert!(css.contains("--fandhe-segment-group-font-size"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            Size::Md,
            false,
            None,
            None,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="segment-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn xss_payload_in_item_value_is_escaped_by_render() {
        let payload = "\"><script>alert(1)</script>";
        let html = render(&item(false, false, payload, vec![], vec![text(payload)]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn xss_payload_in_item_text_children_is_escaped_by_render() {
        let payload = "\"><img src=x onerror=alert(1)>";
        let html = render(&item_text(false, false, vec![], vec![text(payload)]));
        assert!(!html.contains("<img"));
        assert!(html.contains("&lt;img"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_segment_group_state_machine() {
        // 再エクスポートされた `SegmentGroup`（headless の Component/Hydrate
        // 実装を radio_group 経由で継承）経由で SSR/hydration 往復を固定する
        // （[`crate::radio_group`] の同型テストに準拠）。
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut g = SegmentGroup::default();
        assert_eq!(g.value(), None);

        assert!(dispatch(&mut g, "select", "list"));
        assert_eq!(g.value(), Some("list"));

        let ssr_html = render(&g.item_control("list", false, vec![]));
        assert!(ssr_html.contains(r#"data-state="checked""#));

        let hydrate_html = render(&render_for_hydration(&g));
        assert!(hydrate_html.contains("data-hydrate-"));

        let restored = SegmentGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored.value(), Some("list"));
    }
}
