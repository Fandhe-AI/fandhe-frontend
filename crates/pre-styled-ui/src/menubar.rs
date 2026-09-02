//! styled Menubar（headless ラッパー、イシュー #992、親 #932 Phase 8）。
//!
//! `fandhe_frontend_headless_ui::menubar`（イシュー #992）の Root / Menu /
//! Trigger / Positioner / Content / Item / ItemGroup / ItemGroupLabel /
//! Separator / SubTrigger / SubContent 11 anatomy パーツと
//! [`fandhe_frontend_headless_ui::menubar::Menubar`] roving tabindex + 単一
//! 開閉状態機械をそのまま再エクスポートし、[`stylesheet`] で既定 CSS を
//! 追加提供する（[`crate::toolbar`] と同型の薄い委譲）。
//!
//! # `size`/`color-palette` variant 軸は提供しない
//!
//! [`crate::menu`]（イシュー #729）とは異なり、本モジュールは `size`/
//! `color-palette` variant を提供しない（既定 1 種の見た目のみ）。Menubar
//! は Radix Primitives 上でもトップレベルのナビゲーション構造という位置
//! 付けであり、サイズバリエーションの需要が薄いという判断（受け入れ条件・
//! 計画で確定済み）。将来 variant 需要が生じた場合は [`crate::menu`] の
//! `Size` variant パターンをそのまま踏襲できる。
//!
//! # レイアウト
//!
//! `root` は `display: flex` + `align-items: center` + `gap` の横並びを
//! 既定とし、`data-orientation="vertical"` のとき `flex-direction: column`
//! へ切り替える（headless 層が `data-orientation` を固定出力する契約、
//! `crates/headless-ui/src/menubar.rs` 参照。[`crate::toolbar`] と同判断）。
//!
//! # `menu` パーツの `position: relative`
//!
//! [`crate::menu`] の styled `root` が `position: relative`（`positioner`
//! の containing block）を担うのに対し、Menubar では 1 Menubar に複数
//! Menu が並ぶため、per-menu ラッパーである `menu` パーツがこの責務を担う
//! （headless 層の `menu` anatomy パーツ、`crates/headless-ui/src/menubar.rs`
//! 「`role="none"` の根拠と制約」参照）。
//!
//! # `content` パーツの `position: relative`（サブメニューの containing block）
//!
//! `sub-trigger`/`sub-content` は `content` の子として並ぶ兄弟パーツであり
//! （headless 層は Portal による実 DOM 移送を行わない、本 rustdoc「本イシュー
//! のスコープ外」節参照）、`sub-content` は `position: absolute; top: 0;
//! left: 100%` で自身の containing block の右上角を基準に配置される。
//! `content` 自身に `position` を明示していないと、containing block 検索は
//! さらに外側の祖先（既定では `positioner`）まで遡る。この既定状態は
//! `positioner` の padding box が実質的に `content` の外接矩形とほぼ一致する
//! ため見た目上の破綻は起きにくいが、`crates/docs-site/src/showcase.rs` の
//! `SHOWCASE_LAYOUT_CSS`（PR #1000 Bugbot 指摘 1 対応）が掲示用に `menubar`
//! の `positioner` を `position: static` へ中和すると、containing block
//! 検索は `positioner` を素通りしてさらに外側の `menu`（`position: relative`,
//! 本モジュール「`menu` パーツの `position: relative`」節参照）まで遡って
//! しまい、`sub-content` が `content` の右上角ではなく Menubar 上の
//! per-menu ラッパー（File トリガー行を含む）の右上角を基準に配置される
//! 回帰を招く（PR #1000 Bugbot 指摘 2）。[`crate::menu`] の `root` が
//! `trigger`/`positioner` 共通祖先として `position: relative` を担うのと
//! 同型の判断として、`sub-trigger`/`sub-content` の共通祖先である `content`
//! 自身に `position: relative` を宣言し、外側の祖先（`positioner`/`menu`）の
//! 中和有無に依存しない安定した containing block を確定させる。トリガー行
//! そのものを基準にした厳密な配置計算（`placement` 相当）は本 rustdoc
//! 「本イシューのスコープ外」節が示すとおり対象外のまま。
//!
//! # focus-visible リング
//!
//! `trigger` はネイティブなフォーカス可能要素（`<button>`）であり、
//! キーボード操作時のみのフォーカスリングを
//! [`crate::recipe::StateCondition::FocusVisible`] 経由で登録する
//! （[`crate::toolbar`]/[`crate::menu`] と同じ判断）。`item`/`sub-trigger`
//! は virtual focus パターン（実 DOM フォーカスは `trigger` に留まる）の
//! ため `:focus-visible` は付けず、`data-highlighted` で表現する
//! （[`crate::menu`] の `item` と同判断）。
//!
//! # イシュー #1702（root / trigger の外枠パート是正、親 #1528）
//!
//! 親イシュー #1528（menubar のスタイルを参考サイト基準へ調整）の 2h 分割
//! 1 本目。トップレベルの外枠パート（`root`/`trigger`）のみを Phase 0
//! 共通基盤（[`crate::recipe`] の canonical ヘルパ・`--fandhe-*` トークン）
//! へ揃えた。是正内容:
//!
//! - `trigger` の `border-radius` を生リテラル `0.25rem` から
//!   `var(--fandhe-radius-sm)` へトークン化（値同一・外観不変）。
//! - `trigger` に [`crate::recipe::hover_bg_muted`]・
//!   [`crate::recipe::StateCondition::HoverExceptAttrEq`]`("data-highlighted",
//!   "data-state", "open")`・[`crate::recipe::hover_surface_declarations`]
//!   で hover 背景を追加。`Hover`（無条件）ではなく `HoverExceptAttrEq` を
//!   使う理由は、highlight 中・open 中のいずれでも hover の淡い背景が
//!   accent / accent-subtle 背景を洗い流す回帰（highlight 分は
//!   PR #1745 P1 指摘、open 分は PR #1803 Bugbot Medium severity 指摘
//!   「Hover washes out open trigger」）を避けるため。`data-highlighted`
//!   のみを除外する `HoverExceptAttr`（menu 3/3・PR #1802 `trigger-item`
//!   と同型の判断）では、open だが highlighted ではない trigger への
//!   hover が open の `accent-subtle` 背景を上書きしてしまうため、両方を
//!   除外する複合 variant が必要だった。
//! - `trigger` に `data-highlighted`（headless 層が roving tabindex の
//!   ポインタ移動時に trigger へも出力する属性、`crates/headless-ui/
//!   src/menubar.rs::trigger` 参照）の視覚反映を追加（`item`/`sub-trigger`
//!   と同じ accent 配色）。登録順は open → highlighted → disabled → hover
//!   （highlighted が open を後勝ちで上書きする、`trigger`/`trigger-item`
//!   の既存規約と同順序）。
//! - `trigger` の disabled（直書き 2 宣言）・focus-visible（直書き
//!   `outline` 2 宣言）を [`crate::recipe::disabled_declarations`]・
//!   [`crate::recipe::focus_ring_declarations`]`(FocusRingColor::Token,
//!   FocusRingOffset::Outside)` へ置換（値同一・外観不変。`palette` 軸を
//!   持たないため `Token` を選ぶ、menu 1/3 と同じ選択）。
//! - `trigger` に [`crate::recipe::transition_declarations`]`("background,
//!   color", MotionDuration::Fast)` を追加。
//! - `root` の `border-bottom` 単独宣言を `border`（全辺）+
//!   `border-radius: var(--fandhe-radius-md)` へ拡張し、Radix Primitives
//!   Menubar の角丸パネル外観へ整合させた（[`crate::toolbar`] の
//!   root〔full border + radius パネル〕とも同型）。
//!
//! ## 意図的に合わせなかった点
//!
//! - **root の box-shadow は追加しない**: 参照サイトのデモは root に影を
//!   持つが、本部品はアプリケーションバーという位置付け（[`crate::toolbar`]
//!   と同型）を優先し、影は付けない意図的差分とする。
//! - **`size`/`color-palette` variant 軸は導入しない**: 本モジュール冒頭
//!   「`size`/`color-palette` variant 軸は提供しない」節の判断を維持する。
//!
//! ## スコープ境界
//!
//! 本イシューは `root`/`trigger` のみを担当する。`positioner`/`content`/
//! `item`/`item-group`/`separator`/`sub-trigger`/`sub-content` と開閉
//! トランジションは兄弟イシュー #1703 の担当範囲であり、本イシューでは
//! 一切変更しない（`menu` パーツの `position: relative` も維持）。
//!
//! # 本イシューのスコープ外
//!
//! headless 層（`crates/headless-ui/src/menubar.rs`）のモジュール doc
//! 「スコープ外」節をそのまま継承する（矢印キー実 DOM 配線・
//! CheckboxItem/RadioGroup/RadioItem/ItemIndicator/Arrow/ArrowTip・Portal の
//! 実 DOM 移送・placement 計算・skip-disabled モード）。

use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe,
    StateCondition,
};

// REEXPORT-GLOB-REVIEWED: 本モジュールが定義する pub 項目は stylesheet() の
// みで styled パーツ関数を再定義しない（規約 B-1）。variant 軸も提供せず
// （規約 B-2）、CSS 到達は [data-scope]/[data-part] 属性セレクタのみに依存
// する（規約 B-3、イシュー #1062 規約参照）。
pub use fandhe_frontend_headless_ui::menubar::*;
// `Orientation`/`OpenState` は本モジュールの再エクスポート対象パーツ関数
// （`root`/`menu`/`positioner`/`content`/`sub_trigger`/`sub_content` 等）の
// 引数型として呼び出し側が組み立てる必要があるが、`menubar` モジュールの
// glob 再エクスポートでは到達しない（`data_attrs`/`state` モジュール由来の
// ため）。呼び出し側が `fandhe-frontend-pre-styled-ui` のみに依存して
// 呼び出せることを保証するための明示再エクスポート（[`crate::toolbar`] の
// `Orientation` と同型のパターン）。
pub use fandhe_frontend_headless_ui::data_attrs::Orientation;
pub use fandhe_frontend_headless_ui::state::OpenState;

/// headless `menubar` anatomy の `data-part` 一覧（`crates/headless-ui/src/menubar.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "menu",
    "trigger",
    "positioner",
    "content",
    "item",
    "item-group",
    "item-group-label",
    "separator",
    "sub-trigger",
    "sub-content",
];

/// この styled Menubar の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("menubar", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("padding", "var(--fandhe-space-1)"),
            ],
        )
        .base("menu", vec![decl("position", "relative")])
        .base(
            "trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-3)"),
                hover_bg_muted(),
            ],
        )
        .base(
            "trigger",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .base(
            "positioner",
            vec![
                decl("position", "absolute"),
                decl("top", "100%"),
                decl("left", "0"),
                decl("z-index", "10"),
                decl("margin-top", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "content",
            vec![
                decl("position", "relative"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.375rem"),
                decl("box-shadow", "0 4px 6px rgba(0, 0, 0, 0.15)"),
                decl("padding", "var(--fandhe-space-2)"),
                decl("min-width", "10rem"),
            ],
        )
        .base(
            "item",
            vec![
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                decl("cursor", "pointer"),
                decl("border-radius", "0.25rem"),
            ],
        )
        .base(
            "item-group",
            vec![decl("display", "flex"), decl("flex-direction", "column")],
        )
        .base(
            "item-group-label",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
            ],
        )
        .base(
            "separator",
            vec![
                decl("border", "0"),
                decl("border-top", "1px solid var(--fandhe-color-border)"),
                decl("margin", "var(--fandhe-space-2) 0"),
            ],
        )
        .base(
            "sub-trigger",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "space-between"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                decl("cursor", "pointer"),
                decl("border-radius", "0.25rem"),
            ],
        )
        .base(
            "sub-content",
            vec![
                decl("position", "absolute"),
                decl("top", "0"),
                decl("left", "100%"),
                decl("z-index", "10"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.375rem"),
                decl("box-shadow", "0 4px 6px rgba(0, 0, 0, 0.15)"),
                decl("padding", "var(--fandhe-space-2)"),
                decl("min-width", "10rem"),
            ],
        )
        // root が縦向きのとき列方向へ切り替える（本モジュール冒頭 rustdoc
        // 「レイアウト」節参照）。
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![decl("flex-direction", "column")],
        )
        // trigger は実フォーカスを受ける通常ボタン（headless 層の
        // roving tabindex）であり、開閉・ポインタ highlight・disabled・
        // hover・focus-visible の 5 状態を持つ。登録順は open →
        // highlighted → disabled → hover（highlighted が open を後勝ちで
        // 上書きし、hover は highlighted 中は洗い流さない）。menu 3/3
        // （PR #1802）の `trigger-item` と同じ判断（本モジュール冒頭
        // rustdoc「イシュー #1702」節参照）。
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("background", "var(--fandhe-color-accent-subtle)")],
        )
        .state(
            "trigger",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        .state(
            "trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // highlight 中・open 中の双方で hover の淡い背景が accent /
        // accent-subtle 背景を洗い流さないよう `HoverExceptAttrEq` で
        // 両方を除外する（`data-highlighted`・`[data-state="open"]` は
        // specificity が等しく、`HoverExceptAttr("data-highlighted")`
        // 単体では open のみの trigger への hover が open の
        // `accent-subtle` 背景を上書きしてしまう。PR #1803 Bugbot Medium
        // severity 指摘「Hover washes out open trigger」対応、本モジュール
        // 冒頭 rustdoc「イシュー #1702」節参照）。
        .state(
            "trigger",
            StateCondition::HoverExceptAttrEq("data-highlighted", "data-state", "open"),
            hover_surface_declarations(),
        )
        // 開いている sub-trigger を視覚的に強調する（trigger と同じ配色）。
        .state(
            "sub-trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("background", "var(--fandhe-color-accent-subtle)")],
        )
        // virtual focus の highlight 表示（item/sub-trigger は実フォーカス
        // を受けない、本モジュール冒頭 rustdoc「focus-visible リング」節
        // 参照）。
        .state(
            "item",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        .state(
            "sub-trigger",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        // disabled でもフォーカス順序には残るため（headless 層の意図的な
        // 設計判断、`crates/headless-ui/src/menubar.rs` モジュール doc
        // 「スコープ外」節参照）、視覚的にのみ操作不能を示す。
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            vec![decl("opacity", "0.5"), decl("cursor", "not-allowed")],
        )
        .state(
            "sub-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("opacity", "0.5"), decl("cursor", "not-allowed")],
        )
        // trigger はキーボード操作時のみのフォーカスリング（イシュー
        // #1424 の canonical ヘルパへ置換。値は旧実装〔`2px solid
        // var(--fandhe-color-accent)`〕と同一で外観不変、本モジュール
        // 冒頭 rustdoc「イシュー #1702」節参照）。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
}

/// この styled Menubar が生成する静的 CSS 全量を返す（決定的。
/// [`crate::toolbar::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
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
        for part in SLOTS {
            let needle = format!(r#"[data-scope="menubar"][data-part="{part}"]"#);
            assert!(a.contains(&needle), "missing selector for part={part}");
        }
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn root_switches_to_column_when_vertical() {
        let css = stylesheet();
        assert!(css
            .contains(r#"[data-scope="menubar"][data-part="root"][data-orientation="vertical"]"#));
        assert!(css.contains("flex-direction: column;"));
    }

    #[test]
    fn trigger_open_state_is_visually_distinct() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menubar"][data-part="trigger"][data-state="open"]"#));
    }

    #[test]
    fn content_provides_containing_block_for_sub_content() {
        // PR #1000 Bugbot 指摘 2 対応: `sub-trigger`/`sub-content` は `content`
        // の子として並ぶ兄弟パーツであり、`sub-content` の `position: absolute`
        // な配置はいずれかの祖先が containing block を提供しないと不定になる
        // （既定では `positioner` が担うが、showcase の `SHOWCASE_LAYOUT_CSS`
        // が `positioner` を `position: static` へ中和すると検索が `menu` まで
        // 遡ってしまい per-menu ラッパーの角を基準に配置される回帰が起きる、
        // 本モジュール冒頭 rustdoc「`content` パーツの `position: relative`」
        // 節参照）。`content` 自身が `position: relative;` を宣言し、外側の
        // 祖先の中和有無に依存しない containing block になっていることを
        // 固定する。
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"menubar\"][data-part=\"content\"] {\n  position: relative;\n  "
        ));
    }

    #[test]
    fn trigger_declares_focus_visible_ring_but_item_does_not() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menubar"][data-part="trigger"]:focus-visible {"#));
        assert!(!css.contains(r#"[data-scope="menubar"][data-part="item"]:focus-visible {"#));
    }

    #[test]
    fn trigger_focus_ring_uses_canonical_token_form() {
        // イシュー #1702: 直書き `outline: 2px solid var(--fandhe-color-accent)`
        // から `focus_ring_declarations(FocusRingColor::Token,
        // FocusRingOffset::Outside)` へ置換した canonical トークン参照形
        // （フォールバック連鎖込み）であることを固定する。
        let css = stylesheet();
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(css.contains("outline-offset: var(--fandhe-focus-ring-offset, 2px);"));
    }

    #[test]
    fn trigger_declares_highlighted_state() {
        // イシュー #1702: headless 層が roving tabindex のポインタ移動時に
        // trigger へも `data-highlighted` を出力する契約
        // （`crates/headless-ui/src/menubar.rs::trigger` 参照）ため、
        // item/sub-trigger と同じ accent 配色を trigger にも反映する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menubar"][data-part="trigger"][data-highlighted] {"#));
    }

    #[test]
    fn trigger_declares_hover_rule_excluding_highlighted_and_open() {
        // イシュー #1702: highlight 中・open 中のいずれでも hover の淡い
        // 背景が accent / accent-subtle 背景を洗い流さないよう
        // `HoverExceptAttrEq("data-highlighted", "data-state", "open")` を
        // 使う（highlighted 分は PR #1745 P1 指摘、open 分は PR #1803
        // Bugbot Medium severity 指摘「Hover washes out open trigger」の
        // 回帰防止）。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="menubar"][data-part="trigger"]:hover:not([data-disabled]):not([data-highlighted]):not([data-state="open"]) {"#
        ));
    }

    #[test]
    fn trigger_has_transition_declarations() {
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"menubar\"][data-part=\"trigger\"] {\n  display: inline-flex;\n"
        ));
        assert!(css.contains("transition-property: background, color;"));
    }

    #[test]
    fn root_has_border_and_radius() {
        // イシュー #1702: `border-bottom` 単独から全辺 `border` +
        // `border-radius: var(--fandhe-radius-md)` へ拡張（root shadow は
        // 意図的に追加しない、本モジュール冒頭 rustdoc「意図的に合わせ
        // なかった点」節参照）。
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"menubar\"][data-part=\"root\"] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n"
        ));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(Orientation::Horizontal, "Menubar", vec![], vec![]));
        assert!(html.contains(r#"data-scope="menubar""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="menubar""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_menubar_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut m = Menubar::new(0, 3, None, false, Orientation::Horizontal);
        assert_eq!(m.focused(), 0);

        let ssr_html = render(&m.trigger(0, false, false, None, vec![], vec![]));
        assert!(ssr_html.contains(r#"tabindex="0""#));

        assert!(dispatch(&mut m, "open", "1"));
        assert_eq!(m.open(), Some(1));

        let hydrate_html = render(&render_for_hydration(&m));
        assert!(hydrate_html.contains(r#"data-hydrate-focused="1""#));
        assert!(hydrate_html.contains(r#"data-hydrate-open="1""#));

        let restored = Menubar::from_hydration_attrs(&m.hydration_attrs()).unwrap();
        assert_eq!(restored, m);
    }
}
