//! styled Toolbar（headless ラッパー、イシュー #991、親 #932 Phase 8）。
//!
//! `fandhe_frontend_headless_ui::toolbar`（イシュー #991）の Root / Button /
//! Link / Separator / ToggleGroup / ToggleItem 6 anatomy パーツと
//! [`fandhe_frontend_headless_ui::toolbar::Toolbar`] roving tabindex 状態
//! 機械をそのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する
//! （[`crate::action_bar`] と同型の薄い委譲）。
//!
//! # レイアウト
//!
//! `root` は `display: flex` + `gap` の横並びを既定とし、
//! `data-orientation="vertical"` のとき `flex-direction: column` へ切り替える
//! （headless 層が `data-orientation` を固定出力する契約、
//! `crates/headless-ui/src/toolbar.rs` 参照）。
//!
//! # separator の向き別太さ
//!
//! headless 層の `separator` は toolbar 自身の向きと直交する
//! `aria-orientation` を出力する（横向き toolbar → 縦線）。本モジュールは
//! `aria-orientation` の値そのものをセレクタに使い、縦線（`width: 1px;
//! align-self: stretch`）と横線（`height: 1px; width: 100%`）を出し分ける。
//!
//! # 参考サイト基準へのスタイル調整（イシュー #1547）
//!
//! 参照サイトは **Radix Primitives Toolbar のみ**（chakra-ui / Radix Themes /
//! ark-ui には Toolbar 相当が存在しない。
//! `docs/design/reference-screenshots/radixp-toolbar-1.png` と
//! `themes-toolbar.png` を比較根拠とする）。Radix のデモは「白い角丸パネル +
//! 影、25px 高の ghost 項目（muted 文字色）、hover で淡い accent 背景、on
//! 状態で accent 淡背景 + accent 文字色、1px セパレータ、右端に solid の
//! Share ボタン」という構成であり、これを基準に Phase 0 共通基盤
//! （イシュー #1424 フォーカスリング / #1425 hover・disabled・transition /
//! #1422 色トークン）へ載せ替えて以下を是正した。
//!
//! - **hover の視覚フィードバック**: `button`/`link`/`toggle-item` へ
//!   [`crate::recipe::hover_surface_declarations`] +
//!   `color: var(--fandhe-color-fg)` を追加した。`toggle-item` は on 状態を
//!   hover が洗い流さないよう
//!   [`crate::recipe::StateCondition::HoverExcept`]`("data-state", "on")`
//!   を使う（[`crate::color_picker`]/toggle-group と同型）。`link` は Radix
//!   では hover 背景を持たないが、同一バー内の項目で hover 表現を統一する
//!   意図的差分とする。
//! - **フォーカスリングの canonical 化**: 旧実装の手書き
//!   `outline: 2px solid var(--fandhe-color-accent)` を 3 箇所とも
//!   [`crate::recipe::focus_ring_declarations`]（イシュー #1424）へ移行した
//!   （`palette` 軸を持たないため [`crate::recipe::FocusRingColor::Token`]）。
//! - **disabled の canonical 化**: `button`/`toggle-item` の直書き 2 宣言を
//!   [`crate::recipe::disabled_declarations`] へ置換した（値は同一）。
//! - **トランジション**: `button`/`link`/`toggle-item` の共通宣言へ
//!   [`crate::recipe::transition_declarations`]`("background, color",
//!   MotionDuration::Fast)` を追加した。
//! - **角丸のトークン化**: `root` の `border-radius: 0.5rem` →
//!   `var(--fandhe-radius-lg)`、項目（`button`/`link`/`toggle-item`）の
//!   `0.25rem` → `var(--fandhe-radius-sm)`（いずれも同値のスケールトークン
//!   参照へ置換のみ）。
//! - **項目の色調**: 既定文字色を `fg` → `fg-muted`（Radix の mauve-11
//!   相当）へ、on 状態の文字色に `accent-fg-subtle`（Radix の violet-11
//!   相当、tint 面用に設計されたトークン対）を追加した。
//! - **枠線・高さの統一**: `toggle-item` の `border: 1px solid transparent`
//!   を `button`/`link` と同じ `border: none` に統一し、3 項目へ
//!   `box-sizing: border-box` を付与した（枠線有無の違いによる 2px の
//!   高さずれの解消、[`crate::button`] #1787 と同型の問題）。on 状態は
//!   `border-color: accent` の代わりに `background: accent-subtle` +
//!   `color: accent-fg-subtle` で表現する（Radix の violet-5 / violet-11
//!   相当）。
//!
//! ## 意図的に合わせなかった点
//!
//! - **root の box-shadow は追加しない**: [`crate::menubar`]（#1702）で確定
//!   した「アプリケーションバー位置付け」の判断と整合させる意図的差分。
//! - **solid の Share ボタン相当は提供しない**: 利用者が [`crate::button`]
//!   を持ち込む構成（[`crate::action_bar`] と同じ責務分担）。
//!
//! ## size / variant 軸を追加しない根拠
//!
//! Radix Primitives Toolbar は unstyled で size prop を持たず、
//! `REEXPORT-GLOB-REVIEWED` 規約 B-2（variant 軸非提供）と
//! `tests/reexport_policy.rs` の `toolbar` エントリ（「variant 軸なし」）を
//! 維持する（[`crate::action_bar`] #1516 / [`crate::menubar`] #1702 と同じ
//! 判断）。項目寸法は `padding` + `font-size: sm` で Radix の 25px 高に近い
//! compact な密度へ寄せる。
//!
//! ## `data-pressed` に規則を設けない理由
//!
//! `data-pressed` は headless `toggle-item` が `data-state="on"` と同時に
//! 発行する存在属性であり、視覚的な意味は `data-state="on"` の規則が既に
//! 網羅する（本モジュールの `toggle-item[data-state="on"]` 規則参照）。
//! 重複するセレクタを追加しても表現が増えないため、追加規則を設けない。
//!
//! # 本イシューのスコープ外
//!
//! headless 層（`crates/headless-ui/src/toolbar.rs`）のモジュール doc
//! 「スコープ外」節をそのまま継承する（矢印キー実 DOM 配線・skip-disabled
//! モード・`loopFocus` の視覚表現・オーバーフロー時のスクロール折りたたみ）。
//! size / variant 軸の新設・root の box-shadow も対象外（上記「意図的に
//! 合わせなかった点」参照。要望が出た時点で再評価）。

use crate::css::{decl, Declaration};
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe,
    StateCondition,
};

// REEXPORT-GLOB-REVIEWED: 本モジュールが定義する pub 項目は stylesheet() の
// みで styled パーツ関数・variant 型を再定義しない（規約 B-1）。variant 軸
// も提供せず（規約 B-2）、CSS 到達は [data-scope]/[data-part] 属性セレクタ
// のみに依存する（規約 B-3）。headless 側 `toolbar` モジュールが持つ
// `pub use`（`ToggleGroup`/`MultiToggleGroup`）は下記の明示再エクスポート名
// と衝突しないことを確認済み（イシュー #1062 規約参照）。
pub use fandhe_frontend_headless_ui::toolbar::*;
// `Orientation` は `root`/`separator` の引数型・`Toolbar::new` の引数型として
// 呼び出し側が組み立てる必要があるが、`toolbar` モジュールの glob 再エクス
// ポートでは到達しない（`data_attrs` モジュール由来のため）。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証する
// ための明示再エクスポート（[`crate::action_bar`] の `OpenState`/
// `DisclosureAction` と同型のパターン）。
pub use fandhe_frontend_headless_ui::data_attrs::Orientation;

/// headless `toolbar` anatomy の `data-part` 一覧（`crates/headless-ui/src/toolbar.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "button",
    "link",
    "separator",
    "toggle-group",
    "toggle-item",
];

/// `button`/`link`/`toggle-item` 3 slot が共有する基底宣言（本モジュール
/// 冒頭 rustdoc「枠線・高さの統一」節参照）。出力順は [`stylesheet`] の
/// golden 固定対象のため、変更時は `tests/toolbar_css.rs` の再生成が必要。
fn item_base_declarations() -> Vec<Declaration> {
    let mut declarations = vec![
        decl("display", "inline-flex"),
        decl("align-items", "center"),
        decl("justify-content", "center"),
        decl("gap", "var(--fandhe-space-1)"),
        decl("box-sizing", "border-box"),
        decl("border", "none"),
        decl("border-radius", "var(--fandhe-radius-sm)"),
        decl("padding", "var(--fandhe-space-1) var(--fandhe-space-2)"),
        decl("font-size", "var(--fandhe-font-font-size-sm)"),
        decl("line-height", "var(--fandhe-font-line-height-normal)"),
        decl("color", "var(--fandhe-color-fg-muted)"),
        decl("background", "transparent"),
        decl("cursor", "pointer"),
    ];
    declarations.push(hover_bg_muted());
    declarations.extend(transition_declarations(
        "background, color",
        MotionDuration::Fast,
    ));
    declarations
}

/// この styled Toolbar の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("toolbar", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-lg)"),
                decl("padding", "var(--fandhe-space-2)"),
                decl("background", "var(--fandhe-color-bg)"),
            ],
        )
        .base("button", item_base_declarations())
        .base("link", {
            let mut declarations = item_base_declarations();
            declarations.push(decl("text-decoration", "none"));
            declarations
        })
        .base(
            "separator",
            vec![
                decl("background", "var(--fandhe-color-border)"),
                decl("width", "1px"),
                decl("align-self", "stretch"),
            ],
        )
        .base(
            "toggle-group",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
            ],
        )
        .base("toggle-item", item_base_declarations())
        // root が縦向きのとき列方向へ切り替える（本モジュール冒頭 rustdoc
        // 「レイアウト」節参照）。
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![decl("flex-direction", "column")],
        )
        // separator は toolbar 自身の向きと直交する aria-orientation を持つ
        // ため、その値そのものをセレクタに使って向き別の太さを出し分ける
        // （本モジュール冒頭 rustdoc「separator の向き別太さ」節参照）。
        .state(
            "separator",
            StateCondition::AttrEq("aria-orientation", "horizontal"),
            vec![
                decl("height", "1px"),
                decl("width", "100%"),
                decl("align-self", "auto"),
            ],
        )
        // 押下中の toggle-item を視覚的に強調する（Radix の violet-5 /
        // violet-11 相当、本モジュール冒頭 rustdoc「枠線・高さの統一」節
        // 参照）。
        .state(
            "toggle-item",
            StateCondition::AttrEq("data-state", "on"),
            vec![
                decl("background", "var(--fandhe-color-accent-subtle)"),
                decl("color", "var(--fandhe-color-accent-fg-subtle)"),
            ],
        )
        // disabled でもフォーカス順序には残るため（headless 層の意図的な
        // 設計判断、`crates/headless-ui/src/toolbar.rs` モジュール doc
        // 「スコープ外」節参照）、視覚的にのみ操作不能を示す
        // （イシュー #1425 canonical ヘルパへ移行）。
        .state(
            "button",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "toggle-item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // キーボード操作時のみのフォーカスリング（イシュー #1424 canonical
        // ヘルパへ移行。`palette` 軸を持たないため `FocusRingColor::Token`）。
        .state(
            "button",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "link",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "toggle-item",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // hover の視覚フィードバック（イシュー #1425 共通ビジュアル言語。
        // `--fandhe-hover-bg` は上記 `item_base_declarations` の
        // `hover_bg_muted()` が定義する。`toggle-item` は on 状態を hover が
        // 洗い流さないよう除外する、本モジュール冒頭 rustdoc「hover の
        // 視覚フィードバック」節参照）。
        .state("button", StateCondition::Hover, {
            let mut declarations = hover_surface_declarations();
            declarations.push(decl("color", "var(--fandhe-color-fg)"));
            declarations
        })
        .state("link", StateCondition::Hover, {
            let mut declarations = hover_surface_declarations();
            declarations.push(decl("color", "var(--fandhe-color-fg)"));
            declarations
        })
        .state(
            "toggle-item",
            StateCondition::HoverExcept("data-state", "on"),
            {
                let mut declarations = hover_surface_declarations();
                declarations.push(decl("color", "var(--fandhe-color-fg)"));
                declarations
            },
        )
}

/// この styled Toolbar が生成する静的 CSS 全量を返す（決定的。
/// [`crate::action_bar::stylesheet`] と同じ契約）。
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
        assert!(a.contains(r#"[data-scope="toolbar"][data-part="root"]"#));
        assert!(a.contains(r#"[data-scope="toolbar"][data-part="button"]"#));
        assert!(a.contains(r#"[data-scope="toolbar"][data-part="link"]"#));
        assert!(a.contains(r#"[data-scope="toolbar"][data-part="separator"]"#));
        assert!(a.contains(r#"[data-scope="toolbar"][data-part="toggle-group"]"#));
        assert!(a.contains(r#"[data-scope="toolbar"][data-part="toggle-item"]"#));
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
            .contains(r#"[data-scope="toolbar"][data-part="root"][data-orientation="vertical"]"#));
        assert!(css.contains("flex-direction: column;"));
    }

    #[test]
    fn separator_horizontal_aria_orientation_overrides_vertical_default() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="toolbar"][data-part="separator"][aria-orientation="horizontal"]"#
        ));
        assert!(css.contains("height: 1px;"));
    }

    #[test]
    fn toggle_item_pressed_state_is_visually_distinct() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="toolbar"][data-part="toggle-item"][data-state="on"]"#));
    }

    #[test]
    fn button_and_toggle_item_declare_focus_visible_ring() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="toolbar"][data-part="button"]:focus-visible {"#));
        assert!(css.contains(r#"[data-scope="toolbar"][data-part="link"]:focus-visible {"#));
        assert!(css.contains(r#"[data-scope="toolbar"][data-part="toggle-item"]:focus-visible {"#));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(css.contains("outline-offset: var(--fandhe-focus-ring-offset, 2px);"));
    }

    #[test]
    fn items_declare_hover_background_inside_hover_media_query() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(css
            .contains(r#"[data-scope="toolbar"][data-part="button"]:hover:not([data-disabled])"#));
        assert!(
            css.contains(r#"[data-scope="toolbar"][data-part="link"]:hover:not([data-disabled])"#)
        );
        assert!(css.contains(
            r#"[data-scope="toolbar"][data-part="toggle-item"]:hover:not([data-disabled]):not([data-state="on"])"#
        ));
        assert!(css.contains("--fandhe-hover-bg: var(--fandhe-color-bg-muted);"));
    }

    #[test]
    fn items_share_border_and_radius_tokens() {
        let css = stylesheet();
        assert_eq!(
            css.matches("border-radius: var(--fandhe-radius-sm);")
                .count(),
            3
        );
        assert!(!css.contains("border: 1px solid transparent;"));
        assert!(css.contains("box-sizing: border-box;"));
    }

    #[test]
    fn toggle_item_on_state_uses_subtle_tint_pair() {
        let css = stylesheet();
        assert!(css.contains("background: var(--fandhe-color-accent-subtle);"));
        assert!(css.contains("color: var(--fandhe-color-accent-fg-subtle);"));
        assert!(!css.contains("border-color: var(--fandhe-color-accent)"));
    }

    #[test]
    fn disabled_uses_canonical_declarations() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="toolbar"][data-part="button"][data-disabled] {"#));
        assert!(css.contains(r#"[data-scope="toolbar"][data-part="toggle-item"][data-disabled] {"#));
        assert!(css.contains("opacity: 0.5;"));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn items_declare_fast_transition() {
        let css = stylesheet();
        assert!(css.contains("transition-property: background, color;"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(Orientation::Horizontal, "Toolbar", vec![], vec![]));
        assert!(html.contains(r#"data-scope="toolbar""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="toolbar""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_toolbar_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut t = Toolbar::new(0, 3, false, Orientation::Horizontal);
        assert_eq!(t.focused(), 0);

        let ssr_html = render(&t.button(0, false, vec![], vec![]));
        assert!(ssr_html.contains(r#"tabindex="0""#));

        assert!(dispatch(&mut t, "next", ""));
        let hydrate_html = render(&render_for_hydration(&t));
        assert!(hydrate_html.contains(r#"data-hydrate-focused="1""#));

        let restored = Toolbar::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }
}
