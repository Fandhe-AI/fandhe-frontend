//! styled Select（headless ラッパー第 1 弾、イシュー #551、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::select`（イシュー #541）の Root / Label /
//! Control / Trigger / ValueText / ClearTrigger / Indicator / Positioner /
//! Content / ItemGroup / ItemGroupLabel / Item / ItemText / ItemIndicator /
//! HiddenSelect 15 anatomy パーツと
//! [`fandhe_frontend_headless_ui::select::Select`] 状態機械をそのまま
//! 再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の
//! 根拠・スコープ外事項は [`crate::dialog`] の rustdoc と同じ方針に従う。
//!
//! # data-state とスタイルの連動（イシュー #551 受け入れ条件）
//!
//! `trigger`（listbox 開閉）・`item`（選択有無、`data-state` を再利用）の
//! `data-state` に応じた見た目の切り替えを [`recipe`] へ登録する
//! （[`crate::recipe::SlotRecipe::state`]、イシュー #643。`serialize_rule` を
//! 直接呼ぶ手書きセレクタ機構は廃止した）。
//!
//! # キーボード操作系属性の反映（イシュー #643）
//!
//! `item` は [`crate::menu`] と同じ virtual focus パターン（イシュー #581）
//! を使い、実 DOM フォーカスは `trigger` に留まる。ハイライト中の項目には
//! `data-highlighted` が付与されるため、highlight 表示は
//! [`crate::recipe::StateCondition::Attr`]`("data-highlighted")` で反映し
//! （既存の選択済み `item[data-state="open"]` とは背景色を変えて視覚的に
//! 区別する）、`item` へ `:focus-visible` は付けない。実際にフォーカスを
//! 受ける `trigger` には `:focus-visible` を登録する。
//!
//! # `--fandhe-reference-width` の消費（イシュー #643）
//!
//! [`crate::menu`] と同じ理由（モジュール rustdoc 参照）で、`content` の
//! `min-width` が `var(--fandhe-reference-width, auto)` を参照し、listbox
//! 幅がトリガー幅へ追随する sameWidth 相当の見た目を実現する。Menu の
//! フォールバック値（`10rem`）とは異なり `auto` を採用する: Select の
//! `content` は元々固定 `min-width` を持たず（trigger 由来の `control`/
//! `hidden-select` の幅で視覚的に揃う設計だった）、変数未設定時の SSR
//! 静的表示での見た目変化を避けるため。
//!
//! # 位置ジオメトリ（`--fandhe-x`/`--fandhe-y`）の消費（イシュー #663）
//!
//! [`crate::menu`] と同じ理由・同じ仕組み（モジュール rustdoc 参照）で、
//! `positioner` へ `data-positioned` マーカーが付与されたときのみ確定座標
//! （viewport 座標系の `position: fixed`）へ切り替える。arrow は
//! `PositionedKind::has_arrow()` が Select を対象外とする（ADR §4.2）ため、
//! `--fandhe-arrow-*` の消費は Select には追加しない。

//!
//! # hidden-select の視覚的非表示化・positioner のオーバーレイ配置（PR #575 Bugbot 指摘対応）
//!
//! `hidden-select` は form 送信用のネイティブ `<select>` を保持する専用パーツで、
//! headless 層（`crates/headless-ui/src/select.rs`）は `aria-hidden`/`tabindex`
//! のみを設定し視覚的な非表示化は行わない契約になっている。styled 層である
//! 本モジュールが visually-hidden パターン（`position: absolute` + 1px クリップ）
//! で覆い隠す責務を負う（[`recipe`] の `hidden-select` 規則）。また `positioner`
//! は `position: absolute` で配置し、開いた listbox が通常のフローに残らず
//! オーバーレイ表示になるようにする（[`crate::dialog`] の `positioner` と同じ
//! 配置責務）。`control`/`positioner` は headless 側 `root`（同ファイル）の子と
//! して並置される兄弟要素であり、`control` は `positioner` の祖先になれない。
//! そのため containing block を提供する `position: relative` は共通の祖先で
//! ある `root` に付与する（PR #575 Bugbot 指摘 2 対応、`control` への誤付与を
//! 修正）。

use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};

pub use fandhe_frontend_headless_ui::select::*;

/// headless `select` anatomy の `data-part` 一覧（`crates/headless-ui/src/select.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "control",
    "trigger",
    "value-text",
    "clear-trigger",
    "indicator",
    "positioner",
    "content",
    "item-group",
    "item-group-label",
    "item",
    "item-text",
    "item-indicator",
    "hidden-select",
];

/// この styled Select の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("select", SLOTS)
        .base("root", vec![decl("position", "relative")])
        .base(
            "label",
            vec![
                decl("display", "block"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("margin-bottom", "var(--fandhe-space-1)"),
            ],
        )
        .base("control", vec![decl("display", "inline-flex")])
        .base(
            "trigger",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "space-between"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.375rem"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                decl("cursor", "pointer"),
            ],
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
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.375rem"),
                decl("box-shadow", "0 4px 6px rgba(0, 0, 0, 0.15)"),
                decl("padding", "var(--fandhe-space-2)"),
                decl("min-width", "var(--fandhe-reference-width, auto)"),
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
            "item-group-label",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
            ],
        )
        .base(
            "clear-trigger",
            vec![
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "hidden-select",
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
        // イシュー #551 受け入れ条件: `trigger`（開閉）・`item`（選択済み）の見た目の切り替え。
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("border-color", "var(--fandhe-color-accent)")],
        )
        .state(
            "item",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("background", "var(--fandhe-color-bg-muted)")],
        )
        // イシュー #643 受け入れ条件: virtual focus の highlight 表示
        // （`item` は実 DOM フォーカスを受けないため `:focus-visible` ではなく
        // `data-highlighted` で表現する。既存の選択済み表示（背景
        // `bg-muted`）とは異なる強度にして視覚的に区別する、モジュール
        // rustdoc 参照）。
        .state(
            "item",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        // イシュー #643: `trigger` はキーボード操作時のみのフォーカスリング。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        // イシュー #663: wasm 層が `data-positioned` マーカーを付与したら
        // 確定座標（viewport 座標系の `position: fixed`）へ切り替える
        // （[`crate::menu`] と同じ契約、モジュール rustdoc 参照）。
        .state(
            "positioner",
            StateCondition::Attr("data-positioned"),
            vec![
                decl("position", "fixed"),
                decl("top", "0"),
                decl("left", "0"),
                decl("margin-top", "0"),
                decl(
                    "transform",
                    "translate3d(var(--fandhe-x, 0px), var(--fandhe-y, 0px), 0)",
                ),
            ],
        )
}

/// この styled Select が生成する静的 CSS 全量を返す（決定的。[`crate::dialog::stylesheet`]
/// と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
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
        assert!(a.contains(r#"[data-scope="select"][data-part="trigger"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn hidden_select_is_visually_hidden_and_positioner_is_absolute() {
        // PR #575 Bugbot 指摘対応: hidden-select が視覚的に隠され、positioner が
        // オーバーレイ配置になっていることを固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="select"][data-part="hidden-select"]"#));
        assert!(css.contains("clip: rect(0, 0, 0, 0);"));
        assert!(css.contains(r#"[data-scope="select"][data-part="positioner"]"#));
        assert!(css.contains("position: absolute;"));
    }

    #[test]
    fn root_provides_containing_block_for_positioner() {
        // PR #575 Bugbot 指摘 2 対応: `control` と `positioner` は headless
        // `root` の下の兄弟要素であり、`control` は `positioner` の祖先には
        // なれない。そのため `position: relative` は共通祖先である `root`
        // に付与されていることを固定する（`control` への誤付与への回帰防止）。
        let css = stylesheet();
        assert!(css
            .contains("[data-scope=\"select\"][data-part=\"root\"] {\n  position: relative;\n}\n"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="select""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open() {
        // イシュー #551 受け入れ条件: 「headless 層の data-state とスタイルの
        // 連動テスト（[data-state='open'] セレクタ等）」を固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="select"][data-part="trigger"][data-state="open"]"#));
        assert!(css.contains(r#"[data-scope="select"][data-part="item"][data-state="open"]"#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_select_state_machine() {
        // イシュー #551 受け入れ条件: 「SSR / hydration 両経路の動作確認」を
        // 再エクスポートされた `Select`（headless の Component/Hydrate 実装を
        // そのまま継承）経由で固定する。
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut s = Select::default();
        assert_eq!(s.open_state(), OpenState::Closed);

        let ssr_html = render(&s.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut s, "open", ""));
        let hydrate_html = render(&render_for_hydration(&s));
        assert!(hydrate_html.contains("data-hydrate-"));

        let restored = Select::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored.open_state(), OpenState::Open);
    }

    #[test]
    fn item_highlighted_attr_is_styled_and_trigger_has_focus_visible_ring() {
        // イシュー #643 受け入れ条件: virtual focus の highlight 表示
        // （`data-highlighted`）とキーボード操作系属性（`:focus-visible`）が
        // recipe 経由で反映されることを固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="select"][data-part="item"][data-highlighted] {"#));
        assert!(css.contains(r#"[data-scope="select"][data-part="trigger"]:focus-visible {"#));
        assert!(css.contains("outline: 2px solid var(--fandhe-color-accent);"));
    }

    #[test]
    fn content_min_width_consumes_fandhe_reference_width_css_var() {
        // イシュー #643 受け入れ条件: `--fandhe-reference-width` を CSS
        // 継承で消費する sameWidth 相当のスタイルが反映されることを固定する
        // （SSR 静的表示では auto へフォールバックし従来の見た目を維持する）。
        let css = stylesheet();
        assert!(css.contains("min-width: var(--fandhe-reference-width, auto);"));
    }

    #[test]
    fn positioner_switches_to_fixed_geometry_when_data_positioned_marker_is_present() {
        // イシュー #663 受け入れ条件: wasm 層が付与する `data-positioned`
        // マーカーが立っているときのみ、positioner が確定座標（viewport
        // 座標系の `position: fixed`）へ切り替わることをゴールデンで固定する。
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"select\"][data-part=\"positioner\"][data-positioned] {\n  \
             position: fixed;\n  \
             top: 0;\n  \
             left: 0;\n  \
             margin-top: 0;\n  \
             transform: translate3d(var(--fandhe-x, 0px), var(--fandhe-y, 0px), 0);\n\
             }\n"
        ));
    }

    #[test]
    fn positioner_base_rule_keeps_static_ssr_fallback_geometry() {
        // イシュー #663: `data-positioned` マーカー不在（SSR 静的表示・wasm
        // 未稼働）では従来どおり absolute + ローカル座標系のままであることの
        // 回帰固定。
        let css = stylesheet();
        assert!(css.contains("position: absolute;"));
        assert!(css.contains("top: 100%;"));
    }

    #[test]
    fn select_stylesheet_never_consumes_fandhe_arrow_geometry() {
        // イシュー #663: Select は `PositionedKind::has_arrow() == false`
        // （ADR §4.2）のため arrow ジオメトリ変数を一切消費しないことを固定する。
        let css = stylesheet();
        assert!(!css.contains("--fandhe-arrow-"));
    }

    #[test]
    fn position_geometry_var_references_never_lack_an_explicit_fallback() {
        // fail-closed 回帰（イシュー #663 §5 手順 6）: 本イシューが導入する
        // 位置ジオメトリ変数（`--fandhe-x`/`--fandhe-y`）への参照はすべて
        // 明示フォールバック値を持つ（裸の `var(--x)` 禁止）。変数未定義
        // （SSR・wasm 失敗時）でも表示が壊れないことを保証する（テーマ
        // トークン系の `--fandhe-color-*` 等はフォールバック不要の常時
        // 定義済み変数のため対象外とする）。
        let css = stylesheet();
        for marker in ["var(--fandhe-x", "var(--fandhe-y"] {
            for (idx, _) in css.match_indices(marker) {
                let close = css[idx..]
                    .find(')')
                    .expect("every var( occurrence must be closed within the stylesheet");
                let inside = &css[idx + "var(".len()..idx + close];
                assert!(
                    inside.contains(','),
                    "var() reference without an explicit fallback found: var({inside})"
                );
            }
        }
    }
}
