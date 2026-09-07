//! styled Collapsible（headless ラッパー、イシュー #1682、親 #1670/#520/#546）。
//!
//! `fandhe_frontend_headless_ui::collapsible`（イシュー #529、参考サイト突合は
//! #1637）の Root / Trigger / Indicator / Content 4 anatomy パーツと
//! [`fandhe_frontend_headless_ui::collapsible::Collapsible`] 状態機械をそのまま
//! 再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の
//! 根拠・スコープ外事項は [`crate::toggle_tip`]/[`crate::accordion`] の
//! rustdoc と同じ方針に従う。
//!
//! # 参考サイト比較（7 軸チェック、イシュー #1682）
//!
//! 参照 3 サイト（chakra-ui / Radix Primitives / ark-ui）はいずれも
//! Collapsible に `size`/`variant`/`colorPalette` prop を持たない
//! （chakra はプレーンなテキストトリガー、ark はカード状トリガー +
//! シェブロン、Radix は `IconButton` トグル）。
//!
//! - **サイズ / バリアント**: 提供しない（下記スコープ外節参照）。
//! - **色**: 生の色リテラルなし。すべて `--fandhe-*` トークン参照。
//! - **`data-*` 状態**: headless 層が `trigger`/`indicator`/`content` へ
//!   出力する `data-state`（open/closed）・`data-disabled` を視覚へ反映する
//!   （`trigger[data-state="open"]` の文字色強調、
//!   `indicator[data-state="open"]` の回転、`trigger[data-disabled]` の
//!   `disabled_declarations()`）。
//! - **ダーク**: `--fandhe-color-*` トークン経由で自動追従する。
//! - **フォーカス**: `trigger` のみ [`crate::recipe::focus_ring_declarations`]
//!   （`FocusRingOffset::Outside`。`root` は `overflow: hidden` を持たず
//!   [`crate::toggle_tip`] と同じ判断）。
//! - **余白・角丸・影**: `--fandhe-radius-md`（trigger）/`--fandhe-radius-lg`
//!   （content、chakra のボックス表現に合わせた密度差）・`--fandhe-space-*`
//!   のスケールトークンのみを使う。
//! - **hover / disabled / transition**: `trigger` に
//!   [`crate::recipe::hover_bg_muted`] +
//!   [`crate::recipe::StateCondition::Hover`] →
//!   [`crate::recipe::hover_surface_declarations`]、
//!   [`crate::recipe::transition_declarations`]（`background, color` /
//!   `transform`）、`data-disabled` に
//!   [`crate::recipe::disabled_declarations`] を登録する。
//!
//! # `content` は base で `display` を宣言しない
//!
//! headless 層（`crates/headless-ui/src/collapsible.rs`）は closed のとき
//! `content` へ `hidden` 存在属性を付与する。base 規則で `display` を宣言
//! すると UA 既定 `[hidden] { display: none }` を上書きして閉じなくなる
//! （PR #575 Bugbot 指摘・dialog で発生した不具合と同種）。
//! [`crate::toggle_tip`] の `positioner` と同じ構造的回避を採る。
//!
//! # `indicator` の `display: inline-block`
//!
//! headless 層は `indicator` を `span`（非置換インライン要素、`transform`
//! が効かない）として描画する。[`accordion`]（`item-indicator`）と同じ
//! 理由で `display: inline-block` を base へ設定し、open 時の
//! `rotate(180deg)` が実際に適用されるようにする。
//!
//! [`accordion`]: crate::accordion
//!
//! # shadcn/ui（Base UI）突合（イシュー #2029）
//!
//! shadcn/ui（補完参照、`docs/design/shadcn-reference-adoption-policy.md`）の
//! Collapsible（<https://ui.shadcn.com/docs/components/base/collapsible>、
//! Base UI ベース）と突合した。掲載 Example は Basic（chevron 付きトリガーと
//! カード）・Settings Panel（複数フォームフィールドの開閉）・File Tree
//! （chevron と Folder/File アイコンでネストした複数 Collapsible）の 3 件のみで、
//! variant/size prop は持たない（`data-slot` 命名等 Base UI 固有語彙のみで
//! 差別化しており、当モジュールの `data-state`/`data-disabled` 語彙とは
//! 対応しない）。突合結果は以下のとおり:
//!
//! - **新規の variant/size/state 軸は無い**: 3 Example はいずれも「アイコン
//!   付きトリガー」「ネスト合成」という合成パターンであり、当モジュール・
//!   headless 層のコード変更を要する差分ではない。
//! - **アイコン付きトリガー**（Basic/File Tree の chevron・Folder/File
//!   アイコン）: [`trigger`]/[`indicator`] の `children: Vec<Node>` は
//!   自由合成のため、追加引数なしで表現できる。
//! - **ネスト合成**（File Tree の入れ子構造）: [`root`] の `content` に
//!   さらに [`root`] を子として渡すだけで再現できる、既存 API のみの合成
//!   パターン。
//! - **Settings Panel**（複数フォームフィールドの開閉）: `content` の
//!   `children` に任意のノードを渡せる既存の自由合成で対応済み。
//!
//! 上記はいずれも [`root`]/[`trigger`]/[`indicator`]/[`content`] の既存
//! API で表現可能なため、`recipe()`/公開シグネチャ/CSS 出力は一切変更
//! しない。「アイコン付きトリガー」「ネスト合成」の合成パターンは docs
//! サイトの Examples 節新設（`crates/docs-site/src/component_specs_overlay.rs`
//! の `ex_collapsible_nested_tree`）で可視化した（本モジュールのコード
//! 自体は不変）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - **高さアニメーション**（Radix `--radix-collapsible-content-height`・
//!   `collapsedHeight` 部分表示相当。shadcn/ui にも JS レスの代替実装は
//!   無く、Base UI 自身も `--collapsible-panel-height` を JS 実測で提供する
//!   点はイシュー #2029 の突合で確認済み）: 「content 高さの実測が JS
//!   前提」という理由付けだけでは不完全であり、実際の構造的ブロッカーは
//!   headless 層（`crates/headless-ui/src/collapsible.rs`）が closed 時に
//!   `content` へ `hidden` 存在属性を付与している点にある。pre-styled-ui
//!   側で `[hidden]` の `display` を上書きする実装（`grid-template-rows:
//!   0fr → 1fr` 等の CSS のみのアニメーション手法を含む）は、(a) 下記
//!   `content_base_does_not_declare_display` テストの契約に反し、(b) 閉状態
//!   でも DOM 上へ再露出させてしまう（a11y ツリー上は非表示のはずが視覚上
//!   見えてしまう逆転）。これは headless 層が `hidden` を使わない構造
//!   （常時レンダリング + 高さ 0 の CSS 制御）へ変更されない限り
//!   pre-styled-ui 単独では実施できない設計変更であり、`headless-ui` へ
//!   レイアウト計測の関心を持ち込まない方針
//!   （`docs/policy/intentional-non-adoption.md` §3.25）とも整合するため、
//!   本イシューでは非採用のまま維持する。
//! - **size / variant / colorPalette 軸の追加**: 参照 4 サイト（chakra-ui/
//!   Radix Primitives/ark-ui/shadcn-ui）いずれも持たないため提供しない。
//! - Themes ページ（`site/themes/collapsible.md`）・`site/nav.toml` 登録・
//!   `docs/design/component-coverage-map.md` 更新は兄弟イシュー #1683
//!   （Demo・原稿の初出）の担当範囲を踏襲し、本イシューでは触らない。原稿の
//!   Features 文言（`component_specs_overlay.rs` の `COLLAPSIBLE.features`）
//!   と Examples 節（`ex_collapsible_nested_tree`）は本イシューの突合結果を
//!   反映するため更新する。

use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe,
    StateCondition,
};

// REEXPORT-GLOB-REVIEWED: 本モジュールが定義する pub 項目は stylesheet() の
// みで styled パーツ関数を再定義しない（規約 B-1）。参照 3 サイトいずれも
// size/variant/colorPalette prop を持たないため variant 軸を提供せず
// （規約 B-2）、CSS 到達は [data-scope]/[data-part] 属性セレクタのみに依存する
// （規約 B-3、イシュー #1062 規約参照）。
pub use fandhe_frontend_headless_ui::collapsible::*;
// `root`/`trigger`/`indicator`/`content` の `state` 引数・`Collapsible` の
// `Component::Action`（dispatch 対象）はいずれも `state` モジュール由来で
// 上記 glob 再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証するための
// 明示再エクスポート（イシュー #685 の方針、[`crate::toggle_tip`] と同型）。
pub use fandhe_frontend_headless_ui::state::{DisclosureAction, OpenState};

/// headless `collapsible` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/collapsible.rs` の `ANATOMY.part(...)` 呼び出しと
/// 同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を出力しない
/// fail-closed 側の不具合として現れるため、変更時は両ファイルを合わせて
/// 確認する）。
const SLOTS: &[&str] = &["root", "trigger", "indicator", "content"];

/// この styled Collapsible の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("collapsible", SLOTS)
        .base("root", vec![decl("display", "block")])
        .base(
            "trigger",
            [
                vec![
                    decl("display", "inline-flex"),
                    decl("align-items", "center"),
                    decl("justify-content", "space-between"),
                    decl("gap", "var(--fandhe-space-2)"),
                    decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                    decl("background", "transparent"),
                    decl("color", "var(--fandhe-color-fg)"),
                    decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                    decl("border", "0"),
                    decl("border-radius", "var(--fandhe-radius-md)"),
                    decl("cursor", "pointer"),
                    decl("text-align", "left"),
                    hover_bg_muted(),
                ],
                transition_declarations("background, color", MotionDuration::Fast),
            ]
            .concat(),
        )
        .base(
            "indicator",
            [
                vec![
                    decl("display", "inline-block"),
                    decl("color", "var(--fandhe-color-fg-muted)"),
                ],
                transition_declarations("transform", MotionDuration::Normal),
            ]
            .concat(),
        )
        // イシュー #1682: closed 時に headless 層が付与する `hidden` 存在属性
        // （UA 既定 `[hidden] { display: none }`）を base 規則で上書きしない
        // よう、`display` を意図的に宣言しない（モジュール doc 参照）。
        .base(
            "content",
            vec![
                decl("margin-top", "var(--fandhe-space-2)"),
                decl("padding", "var(--fandhe-space-4)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-lg)"),
            ],
        )
        // 開いている trigger/indicator を強調する（[`crate::accordion`] と同型）。
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("color", "var(--fandhe-color-accent)")],
        )
        .state(
            "indicator",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("transform", "rotate(180deg)")],
        )
        // headless 層が出力する data-disabled を CSS 側で消費する。
        .state(
            "trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "content",
            StateCondition::Attr("data-disabled"),
            vec![decl("color", "var(--fandhe-color-fg-muted)")],
        )
        // キーボード操作時のみのフォーカスリング。root は overflow: hidden
        // を持たないため Outside（[`crate::toggle_tip`] と同じ判断）。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // hover 時の視覚フィードバック。
        .state(
            "trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
}

/// この styled Collapsible が生成する静的 CSS 全量を返す（決定的。
/// [`crate::toggle_tip::stylesheet`] と同じ契約: 同一プロセス内の複数回
/// 呼び出しは常にバイト単位で同一の文字列を返す）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="collapsible"][data-part="trigger"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn indicator_has_transformable_display() {
        // headless indicator は span（非置換インライン要素）のため、
        // transform を効かせるには inline-block 等の非デフォルト display
        // が必要（accordion item-indicator と同じ理由、PR #575 系）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="collapsible"][data-part="indicator"] {"#));
        assert!(css.contains("display: inline-block;"));
    }

    #[test]
    fn content_base_does_not_declare_display() {
        // closed 時の `hidden` 存在属性（UA 既定 [hidden]{display:none}）を
        // base 規則が上書きしないことを固定する（モジュール doc 参照）。
        let css = stylesheet();
        let start = css
            .find(r#"[data-scope="collapsible"][data-part="content"] {"#)
            .expect("content base ブロックが見つからない");
        let block_end = css[start..]
            .find('}')
            .map(|i| start + i)
            .expect("content base ブロックの終端が見つからない");
        let block = &css[start..block_end];
        assert!(
            !block.contains("display:"),
            "content の base ブロックが display を宣言している: {block}"
        );
    }

    #[test]
    fn stylesheet_links_trigger_and_indicator_data_state_to_open_style() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="collapsible"][data-part="trigger"][data-state="open"] {"#)
        );
        assert!(css
            .contains(r#"[data-scope="collapsible"][data-part="indicator"][data-state="open"] {"#));
        assert!(css.contains("transform: rotate(180deg);"));
    }

    #[test]
    fn trigger_declares_focus_visible_ring() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="collapsible"][data-part="trigger"]:focus-visible {"#));
    }

    #[test]
    fn trigger_and_content_declare_disabled_visual() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="collapsible"][data-part="trigger"][data-disabled] {"#));
        assert!(css.contains("opacity: 0.5;"));
        assert!(css.contains(r#"[data-scope="collapsible"][data-part="content"][data-disabled] {"#));
    }

    #[test]
    fn trigger_hover_surface_is_gated_by_hover_media_and_not_disabled() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover) {"));
        assert!(css.contains(
            r#"[data-scope="collapsible"][data-part="trigger"]:hover:not([data-disabled]) {"#
        ));
    }

    #[test]
    fn stylesheet_has_no_raw_color_literals() {
        let css = stylesheet();
        assert!(!css.contains('#'));
        assert!(!css.contains("rgb("));
        assert!(!css.contains("rgba("));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(OpenState::Closed, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="collapsible""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_collapsible_state_machine() {
        let mut c = Collapsible::default();
        assert_eq!(c.state(), OpenState::Closed);

        let ssr_html = render(&c.root(false, vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut c, "toggle", ""));
        let hydrate_html = render(&render_for_hydration(&c));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));

        let restored = Collapsible::from_hydration_attrs(&c.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
    }
}
