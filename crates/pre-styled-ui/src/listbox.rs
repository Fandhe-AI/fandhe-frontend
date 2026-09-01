//! styled Listbox（headless ラッパー、イシュー #750、親 #520/#546/#748）。
//!
//! `fandhe_frontend_headless_ui::listbox`（イシュー #750）の Label / Content /
//! ItemGroup / ItemGroupLabel / Item / ItemText / ItemIndicator / ValueText
//! 8 anatomy パーツをそのまま再エクスポートし、[`stylesheet`] で既定 CSS を
//! 追加提供する。薄い委譲の根拠・スコープ外事項は [`crate::select`] の
//! rustdoc と同じ方針に従う。
//!
//! # [`crate::select`] との責務境界
//!
//! headless 層と同じく、styled Select はポップアップ型（trigger/positioner
//! を持つ）であるのに対し、styled Listbox は常時展開（`content` が常に
//! 表示される、`hidden`/`positioner`/`trigger` を一切持たない）。「常に
//! 見えているリストから選ぶ」用途には本モジュールを、「クリックで開閉する
//! ドロップダウン」用途には [`crate::select`] を使う（詳細は
//! `fandhe_frontend_headless_ui::listbox` module doc 参照）。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Listbox`/
//! `MultiListbox` 型・headless `root` を再エクスポートしない理由）
//!
//! `size` variant クラス付与のため styled [`root`]（[`crate::select::root`]
//! と同型）を本モジュールで新設する。headless 自由関数 `root` と名前が
//! 衝突するため、`pub use ...::*` ではなく必要な識別子のみを選択的に再
//! エクスポートする。状態機械 [`fandhe_frontend_headless_ui::listbox::Listbox`]/
//! [`fandhe_frontend_headless_ui::listbox::MultiListbox`] は**あえて**
//! 再エクスポートしない（[`crate::select`]/[`crate::switch`]/[`crate::menu`]
//! の状態機械非再エクスポートと同じ理由）。状態管理・hydration が必要な
//! 呼び出し側は `fandhe_frontend_headless_ui::listbox::{Listbox, MultiListbox}`
//! を直接 import し、実際の描画は本モジュールの styled [`root`]（および
//! 再エクスポート済みのパーツ関数）を組み合わせて構築すること。
//!
//! # data-state とスタイルの連動
//!
//! `item`（選択有無、`data-state` を再利用）・`root`（disabled）の
//! `data-*` に応じた見た目の切り替えを [`recipe`] へ登録する
//! （[`crate::recipe::SlotRecipe::state`]、[`crate::select`] と同じ機構）。
//!
//! # ハイライト表示（SSR 静的表現）
//!
//! `item` の `data-highlighted`（[`crate::select`] と同じ virtual focus
//! パターン、イシュー #581/#599）は選択済み `item[data-state="open"]` とは
//! 背景色を変えて視覚的に区別する。`content` 自身が DOM フォーカスを受ける
//! （headless module doc 参照）ため `:focus-visible` は `content` slot へ
//! 登録する（[`crate::select`] の `trigger` に相当）。
//!
//! # `color-palette` 軸を提供しない判断
//!
//! [`crate::select`]/[`crate::menu`]/[`crate::tags_input`] の既存判断に
//! 追随し、`size` variant のみを提供する（chakra 固有の `variant`
//! （subtle/solid/plain）展開は out-of-scope として PR 本文で別イシュー化を
//! 提案する）。
//!
//! # スタイル調整（イシュー #1483）
//!
//! 参考サイト（chakra-ui / ark-ui の listbox）基準・Phase 0 共通ビジュアル
//! 言語（`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
//! 〔#1424〕・`docs/design/pre-styled-ui-interaction-visual-language.md`
//! 〔#1425〕）へ揃えるための是正。combobox の先行是正（イシュー #1468、
//! `crate::combobox` rustdoc「スタイル調整（#1468）」節）と同型のパターンを
//! 踏襲する。
//!
//! - **角丸**: `content`/`item` の生リテラル `border-radius` を
//!   `var(--fandhe-radius-md, 0.375rem)`/`var(--fandhe-radius-sm, 0.25rem)`
//!   トークンへ置換（旧リテラル値をフォールバックとして残し、
//!   `Theme::empty()` ベースのカスタムテーマや `listbox::stylesheet()`
//!   単独利用時にトークン未定義でも既存表示を維持する）。
//! - **disabled**: `root`/`item` の手書き宣言（宣言順が不一致だった）を
//!   [`crate::recipe::disabled_declarations`] へ統一。
//! - **hover**: `item` は `cursor: pointer` を持つ操作可能 slot だが hover
//!   状態が未登録だったため、[`crate::recipe::hover_bg_muted`]（base）+
//!   `StateCondition::HoverExceptAttr("data-highlighted")` +
//!   [`crate::recipe::hover_surface_declarations`]（state）を追加する。
//!   素の `Hover` ではなく `HoverExceptAttr` を使う理由は combobox item と
//!   同一（[`crate::combobox`] rustdoc 参照）: headless
//!   （`crates/headless-ui/src/listbox.rs::item`）は `data-highlighted` を
//!   常に空文字値の存在属性として出すため、highlight 中の item にポインタが
//!   重なった際に hover の背景（muted）が highlight の背景（accent）を
//!   specificity で上書きし、`accent-fg` 文字色だけが取り残されてコントラ
//!   ストが崩れる問題を避ける。選択済み `data-state="open"`（bg-muted）と
//!   hover 色（bg-muted）は同値のため衝突しない（combobox と同じ受容判断）。
//! - **focus ring**: `content:focus-visible` の旧形直書きを
//!   [`crate::recipe::focus_ring_declarations`]（`FocusRingColor::Token`
//!   — `color-palette` 軸を持たないため。`FocusRingOffset::Inset` —
//!   `content` は境界を持つ面であり枠内側にリングを描く既存判断を維持）へ
//!   置換。
//! - **transition**: `item` へ `background, color` の
//!   [`crate::recipe::transition_declarations`]（`MotionDuration::Fast`）を
//!   純追加。`prefers-reduced-motion` は `Theme::to_css` 側のトークン一括
//!   上書きで自動的に尊重されるため、本モジュール側で `@media` を追加する
//!   必要はない。
//! - **意図的に合わせない点**: (1) 影 — 本部品は常時展開リスト（ポップアップ
//!   ではない）であり、参考サイトも影を持たないため追加しない。(2)
//!   `item-indicator` の `margin-left: auto` — `item-text` が既に `flex: 1`
//!   を持ち右端整列が成立済みのため、combobox のような追加は不要。
//!   (3) `variant`（色軸）の追加 — 「`color-palette` 軸を提供しない判断」
//!   節に記載のとおり Forms 家族横断の判断であり単独先行しない。
//!
//! # スタイル調整（PR #1762 レビュー対応）
//!
//! 上記「hover」節の初版実装（`StateCondition::HoverExceptAttr`）は item
//! 自身の `[data-disabled]` のみを検査するため、`root(..., disabled =
//! true, ...)` で root 全体を disabled にしても（headless 層は disabled を
//! 子 item へ伝播しない、`crates/headless-ui/src/listbox.rs` 参照）個々の
//! item が disabled でなければ hover 背景が変化し、操作不能な UI が操作
//! 可能に見えるフィードバックを出していた（codex-review P1 指摘）。
//! [`stylesheet`] が item hover 規則を raw CSS として追記する形へ変更し、
//! 祖先 `root` に `[data-disabled]` が無いことをセレクタへ明示することで
//! 是正した（詳細は [`stylesheet`] rustdoc 参照）。

use crate::class_attr::drop_class_attr;
use crate::css::{decl, serialize_rule};
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe,
    StateCondition, VariantValue,
};

// headless 自由関数 `root`・状態機械 `Listbox`/`MultiListbox` はあえて再
// エクスポートしない（本モジュール冒頭の rustdoc「選択的 re-export」節参照）。
// 未スタイル・variant クラス非付与の実体・状態管理が必要な呼び出し側は
// `fandhe_frontend_headless_ui::listbox` を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::listbox::{
    content, item, item_group, item_group_label, item_indicator, item_text, label, value_text,
};
// `root`/`item`/`item_indicator` 等の状態引数はいずれも headless
// `state`/`OpenState` 由来で上記選択的再エクスポートでは到達しない。呼び出し
// 側が `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証
// するための明示再エクスポート（[`crate::select`] の `OpenState` 再
// エクスポートと同じ理由）。
pub use fandhe_frontend_headless_ui::state::OpenState;

/// headless `listbox` anatomy の `data-part` 一覧（`crates/headless-ui/src/listbox.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`]
/// が一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "content",
    "item-group",
    "item-group-label",
    "item",
    "item-text",
    "item-indicator",
    "value-text",
];

/// この styled Listbox の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("listbox", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .base(
            "label",
            vec![
                decl("display", "block"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("margin-bottom", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "content",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md, 0.375rem)"),
                decl("overflow-y", "auto"),
                decl(
                    "max-height",
                    "var(--fandhe-listbox-content-max-height, 16rem)",
                ),
                decl(
                    "padding",
                    "var(--fandhe-listbox-content-padding, var(--fandhe-space-2))",
                ),
            ],
        )
        .base(
            "item-group-label",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl(
                    "padding",
                    "var(--fandhe-listbox-item-padding, var(--fandhe-space-2) var(--fandhe-space-3))",
                ),
                decl("cursor", "pointer"),
                decl("border-radius", "var(--fandhe-radius-sm, 0.25rem)"),
                hover_bg_muted(),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結されるため、
        // 上記 base ブロックを書き換えずに純追加する（combobox item と
        // 同型のパターン、モジュール rustdoc「スタイル調整（#1483）」節参照）。
        .base(
            "item",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .base(
            "item-text",
            vec![decl("flex", "1"), decl("min-width", "0")],
        )
        .base(
            "value-text",
            vec![decl("color", "var(--fandhe-color-fg-muted)")],
        )
        // 選択済み item の見た目の切り替え（headless `data-state` 語彙の再利用）。
        .state(
            "item",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("background", "var(--fandhe-color-bg-muted)")],
        )
        // virtual focus の highlight 表示（`item` は実 DOM フォーカスを受けない
        // ため `:focus-visible` ではなく `data-highlighted` で表現する。既存の
        // 選択済み表示（背景 `bg-muted`）とは異なる強度にして視覚的に区別する、
        // モジュール rustdoc 参照）。
        .state(
            "item",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        // disabled item は減光 + cursor: not-allowed。
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // item の hover 規則は本関数（`SlotRecipe::state`）へ登録せず
        // [`stylesheet`] 側で raw CSS として追加する（root disabled 時の
        // 抑止に祖先セレクタが要るため、モジュール rustdoc「スタイル調整
        // （PR #1762 レビュー対応）」節参照）。
        // `content` 自身が DOM フォーカスを受けるため（headless module doc
        // 参照）、キーボード操作時のみのフォーカスリングを `content` へ登録する
        // （[`crate::select`] の `trigger` に相当）。イシュー #1483:
        // リング宣言を canonical ヘルパへ置換（`color-palette` 軸を持たない
        // ため `FocusRingColor::Token`、`content` は境界を持つ面のため
        // `FocusRingOffset::Inset`、モジュール rustdoc「スタイル調整
        // （#1483）」節参照）。
        .state(
            "content",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Inset),
        )
        // `size` variant（root スコープの CSS custom property。Md はフォールバック
        // 値と同一の現行外観を維持する。[`crate::select`] の `size` variant と
        // 同型の判断）。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-listbox-item-padding", "var(--fandhe-space-0-5) var(--fandhe-space-1)"),
                decl("--fandhe-listbox-content-padding", "var(--fandhe-space-0-5)"),
                decl("--fandhe-listbox-content-max-height", "8rem"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl(
                    "--fandhe-listbox-item-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl("--fandhe-listbox-content-padding", "var(--fandhe-space-1)"),
                decl("--fandhe-listbox-content-max-height", "12rem"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl(
                    "--fandhe-listbox-item-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-3)",
                ),
                decl("--fandhe-listbox-content-padding", "var(--fandhe-space-2)"),
                decl("--fandhe-listbox-content-max-height", "16rem"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl(
                    "--fandhe-listbox-item-padding",
                    "var(--fandhe-space-3) var(--fandhe-space-4)",
                ),
                decl("--fandhe-listbox-content-padding", "var(--fandhe-space-3)"),
                decl("--fandhe-listbox-content-max-height", "20rem"),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-listbox-item-padding", "var(--fandhe-space-4) var(--fandhe-space-5)"),
                decl("--fandhe-listbox-content-padding", "var(--fandhe-space-4)"),
                decl("--fandhe-listbox-content-max-height", "24rem"),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled Listbox が生成する静的 CSS 全量を返す（決定的。[`crate::select::stylesheet`]
/// と同じ契約）。
///
/// # item hover を raw CSS で追記する理由（PR #1762 レビュー対応）
///
/// [`fandhe_frontend_headless_ui::listbox::root`] の `disabled` は子
/// `item` へ伝播しない（`crates/headless-ui/src/listbox.rs` の `root`/
/// `item` 各関数 doc 参照。`root(..., disabled = true, ...)` で root にのみ
/// `data-disabled` が付与され、個々の `item` は呼び出し側が渡す `disabled`
/// にのみ従う）。[`SlotRecipe::state`] が生成するセレクタは常に
/// `[data-scope="listbox"][data-part="<slot>"]` を先頭に固定した自パーツ
/// 属性条件のみで、祖先パーツの属性を検査するセレクタを組めない。その
/// ため旧実装の `StateCondition::HoverExceptAttr("data-highlighted")`
/// （`:hover:not([data-disabled]):not([data-highlighted])`）は item 自身の
/// `data-disabled` しか見ておらず、root 全体が disabled でも個々の item が
/// disabled でなければ hover 背景が変化し、操作不能な UI が操作可能に見える
/// フィードバックを出していた（`checkbox_group` が同種の CSS のみでの
/// disabled 偽装（`root[data-disabled]` からの伝播）を意図的に見送った
/// 判断、`crate::checkbox_group` module doc「`root` の `data-disabled` から
/// `item`/`item-control` への CSS 伝播は行わない」節と対称の問題）。
///
/// 本関数は [`SlotRecipe::css`] の出力へ、祖先 `root` の `[data-disabled]`
/// 不在を前提に含む item hover 規則を [`marquee::css`] と同型の raw CSS
/// 追記パターンで追加する（[`crate::marquee`] の `content` pause 規則
/// 参照）。`checkbox_group` のケース（`pointer-events`/`cursor` 等で
/// キーボード操作の実効性まで偽装しようとして撤回）と異なり、本追記は
/// 装飾専用の hover 背景色 1 プロパティのみを対象とし、tabbability・
/// クリック到達性・ツールチップ表示のいずれにも影響しないため、
/// CSS のみでの是正が成立する。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();
    let selector = "[data-scope=\"listbox\"][data-part=\"root\"]:not([data-disabled]) \
        [data-scope=\"listbox\"][data-part=\"item\"]:hover:not([data-disabled]):not([data-highlighted])";
    if let Some(rule) = serialize_rule(selector, &hover_surface_declarations()) {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str("@media (hover: hover) {\n");
        for line in rule.lines() {
            out.push_str("  ");
            out.push_str(line);
            out.push('\n');
        }
        out.push_str("}\n");
    }
    out
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::listbox::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::listbox::{self, OpenState};
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = listbox::root(Size::Md, OpenState::Closed, false, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="listbox" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    selection_state: OpenState,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::listbox::root(selection_state, disabled, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use fandhe_frontend_headless_ui::state::OpenState;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="listbox"][data-part="item"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(Size::Md, OpenState::Closed, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="listbox""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl] {
            let html = render(&root(
                size,
                OpenState::Closed,
                false,
                vec![("class", "attacker")],
                vec![],
            ));
            let expected_class = format!("fd-listbox--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn default_variant_is_md_and_matches_fallback_values() {
        let css = stylesheet();
        assert!(css.contains(
            "padding: var(--fandhe-listbox-item-padding, var(--fandhe-space-2) var(--fandhe-space-3));"
        ));
        assert!(css.contains("max-height: var(--fandhe-listbox-content-max-height, 16rem);"));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="listbox"][data-part="item"][data-state="open"]"#));
    }

    #[test]
    fn item_highlighted_and_disabled_states_are_styled_distinctly() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="listbox"][data-part="item"][data-highlighted] {"#));
        assert!(css.contains(r#"[data-scope="listbox"][data-part="item"][data-disabled] {"#));
    }

    #[test]
    fn content_has_focus_visible_ring_since_it_receives_dom_focus() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="listbox"][data-part="content"]:focus-visible {"#));
        // イシュー #1483: canonical フォーカスリングヘルパ（inset）へ
        // 置換したことの確認。
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(css.contains("outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));"));
    }

    #[test]
    fn item_has_hover_state_except_when_highlighted() {
        // イシュー #1483: item の hover 実適用。highlight 中の item は
        // hover 側の背景で上書きされない（モジュール rustdoc「スタイル
        // 調整（#1483）」節参照）。PR #1762 レビュー対応により、item 自身の
        // 属性条件に加え祖先 `root` が `[data-disabled]` を持たないことも
        // セレクタへ含める（[`stylesheet`] rustdoc「item hover を raw CSS
        // で追記する理由」節参照）。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="listbox"][data-part="root"]:not([data-disabled]) [data-scope="listbox"][data-part="item"]:hover:not([data-disabled]):not([data-highlighted]) {"#
        ));
        assert!(css.contains("@media (hover: hover)"));
    }

    #[test]
    fn item_and_content_use_radius_tokens_not_raw_literals() {
        // イシュー #1483: 生リテラル border-radius をトークンへ置換した
        // ことの確認（`0.375rem`/`0.25rem` の再発防止）。
        let css = stylesheet();
        assert!(css.contains("border-radius: var(--fandhe-radius-md, 0.375rem);"));
        assert!(css.contains("border-radius: var(--fandhe-radius-sm, 0.25rem);"));
    }

    #[test]
    fn item_has_transition_declarations() {
        // イシュー #1483: item へ transition を純追加したことの確認。
        let css = stylesheet();
        assert!(css.contains("transition-property: background, color;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
    }

    #[test]
    fn root_disabled_state_is_styled() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="listbox"][data-part="root"][data-disabled] {"#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_listbox_state_machine() {
        // SSR / hydration 両経路の動作確認: 本モジュールから状態機械を再
        // エクスポートしないため、エスケープハッチ経由で直接 import する
        // （モジュール冒頭の rustdoc「選択的 re-export」節参照）。
        use fandhe_frontend_headless_ui::listbox::Listbox;
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut l = Listbox::default();
        let ssr_html = render(&l.root(false, vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut l, "select", "apple"));
        let hydrate_html = render(&render_for_hydration(&l));
        assert!(hydrate_html.contains("data-hydrate-"));

        let restored = Listbox::from_hydration_attrs(&l.hydration_attrs()).unwrap();
        assert_eq!(restored.selected(), Some("apple"));
    }
}
