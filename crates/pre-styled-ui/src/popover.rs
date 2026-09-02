//! styled Popover（headless ラッパー第 2 弾、イシュー #664、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::popover`（イシュー #532）の Root / Trigger /
//! Anchor / Positioner / Arrow / ArrowTip / Content / Title / Description /
//! CloseTrigger / Indicator 11 anatomy パーツと
//! [`fandhe_frontend_headless_ui::popover::Popover`] 状態機械をそのまま
//! 再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の
//! 根拠・スコープ外事項は [`crate::dialog`]/[`crate::menu`] の rustdoc と
//! 同じ方針に従う。
//!
//! # data-state とスタイルの連動（イシュー #664 受け入れ条件）
//!
//! `trigger`/`content` の開閉 `data-state`（open/closed）に応じた見た目の
//! 切り替えを [`recipe`] へ登録する（[`crate::recipe::SlotRecipe::state`]）。
//!
//! # キーボード操作系属性の反映
//!
//! `trigger`/`close-trigger` はフォーカス可能なボタン要素であり、
//! キーボード操作時のみのフォーカスリング（`:focus-visible`）を
//! [`crate::recipe::StateCondition::FocusVisible`] 経由で登録する
//! （[`crate::dialog`] と同じ判断）。
//!
//! # positioner のオーバーレイ配置
//!
//! headless 側 `root`（`crates/headless-ui/src/popover.rs`）の子として
//! `trigger`/`positioner` が並置される兄弟関係のため、containing block を
//! 提供する `position: relative` は共通祖先の `root` に付与し、
//! `positioner` 自体は `position: absolute; top: 100%; left: 0` の
//! dropdown 型オーバーレイとする（[`crate::menu`] と同じ tier、
//! `z-index: 10`。[`crate::dialog`] のビューポート全体オーバーレイ
//! （z-index: 1000/1001）とは役割が異なる）。`positioner` は base 規則で
//! `display` を宣言しないため、closed 時に headless 層が付与する `hidden`
//! 存在属性は UA 既定 `[hidden] { display: none }` がそのまま機能する
//! （[`crate::dialog`] の `positioner` のように `display: flex` 等の base
//! 宣言で UA 既定を上書きしていないため、`[hidden]` の明示的な上書き規則は
//! 不要。dialog で発生した PR #575 Bugbot 指摘（High）と同種の不具合を
//! 構造的に回避する）。
//!
//! # `--fandhe-reference-width` の消費（イシュー #664 受け入れ条件 2）
//!
//! `crates/wasm-full/src/position.rs::reposition_one` が `positioner` の
//! `style` 属性へ書き込む `--fandhe-reference-width`（`trigger` の実測幅）を
//! `content` の `min-width` が `var(--fandhe-reference-width, auto)` として
//! 消費する（[`crate::select`] と同じフォールバック判断。popover の
//! `content` は menu/select の listbox と異なり任意の自由形式コンテンツを
//! 保持するため、`auto` フォールバックが `10rem` 固定より適切）。
//! `--fandhe-x`/`--fandhe-y`/`--fandhe-arrow-*`（座標ジオメトリ）は
//! [`crate::menu`] と同じ理由で本イシューの対象外とする。
//!
//! # イシュー #1534 の参照サイト比較（7 軸チェック）
//!
//! chakra-ui / Radix Themes / Radix Primitives / ark-ui と視覚比較した結果を
//! 記録する（先行モデル: [`crate::hover_card`] イシュー #1523、
//! [`crate::dialog`] イシュー #1692/#1693）。
//!
//! - **サイズ / バリアント**: 「複合部品の variant 統一方針 方針 3」
//!   （オーバーレイの配置・寸法がコンテンツ起因の popover/tooltip には
//!   提供しない）に従い、意図的に単一スケールのまま据え置く（下記
//!   スコープ外節参照）。
//! - **色**: `content` の `box-shadow` に残っていた唯一の生色リテラル
//!   （`rgba(0, 0, 0, 0.15)`）は shadow トークン化（後述）で解消した。
//! - **状態（`data-*`）**: headless 層（`crates/headless-ui/src/
//!   popover.rs`）の `trigger` は `disabled`/`data-disabled` を出力する
//!   （[`crate::hover_card`] の「disabled 概念なし」判断はここでは
//!   当てはまらない）。[`crate::recipe::disabled_declarations`] を `trigger`
//!   の `data-disabled` 状態へ新設した（[`crate::button`] と同じ判断）。
//!   `trigger[data-state=open]`/`content[data-state=closed]` の既存連動は
//!   維持する。
//! - **ダーク**: `content` の影が生リテラルでダーク非追従だった点を、
//!   ダーク値内蔵の `var(--fandhe-shadow-md)` へ移行して解消した。他の
//!   宣言はすべて色トークン参照のみで既にダーク追従済み。
//! - **フォーカス**: `trigger`/`close-trigger` の `:focus-visible` を直書き
//!   2 宣言から [`crate::recipe::focus_ring_declarations`]
//!   （`FocusRingColor::Token`、palette 軸を持たないため）へ canonical 化
//!   した（[`crate::hover_card`] と同じ判断）。
//! - **余白・角丸・影**: `trigger` の `border-radius`（生値 `0.375rem`）を
//!   `var(--fandhe-radius-md, 0.375rem)` へ、`content` の `border-radius`
//!   （生値）を `docs/design/pre-styled-ui-scale-tokens.md` §3.1「面パネル
//!   = lg」の分類（popover を名指し）に従い `var(--fandhe-radius-lg,
//!   0.5rem)` へ、`content` の `box-shadow`（生値）を同文書 §3.2
//!   「dropdown 型 overlay = md」に従い `var(--fandhe-shadow-md, 0 4px 6px
//!   rgba(0, 0, 0, 0.15))` へ、`positioner` の `z-index`（生値 `10`）を
//!   同文書 §3.4 の `popover` 段トークン（`var(--fandhe-z-index-popover,
//!   10)`）へそれぞれ移行した（fallback は旧来値を維持し `stylesheet()`
//!   単独利用者のテーマ CSS 未注入時にも壊れない、[`crate::hover_card`]/
//!   [`crate::toast`]/[`crate::date_picker`] と同じ後方互換方針）。
//!   `close-trigger` にも `border-radius: var(--fandhe-radius-sm)` と
//!   `padding: var(--fandhe-space-1)` を新設した（下記「`close-trigger` の
//!   スタイル調整」節参照）。
//! - **hover**: `trigger`/`close-trigger` とも `:hover` 装飾がなかった点を
//!   新設した。両者ともボタン実体で面を持つため
//!   [`crate::recipe::hover_bg_muted`]（base の `--fandhe-hover-bg` 定義）+
//!   [`crate::recipe::StateCondition::Hover`] 経由の
//!   [`crate::recipe::hover_surface_declarations`]（`background` 切替）を
//!   登録する（[`crate::dialog`] の `trigger`/`close-trigger` と同型）。
//! - **disabled**: 前述のとおり `trigger` へ
//!   [`crate::recipe::disabled_declarations`] を新設した。`close-trigger`
//!   は headless 層に disabled 概念を持たないため対象外（N/A）。
//! - **トランジション**: `trigger` へ
//!   [`crate::recipe::transition_declarations`]（`"background, border-color"`、
//!   `MotionDuration::Fast`。`data-state=open` の `border-color` 変化も
//!   対象に含む）、`close-trigger` へ同ヘルパ（`"background"`、
//!   `MotionDuration::Fast`）を新設した。`prefers-reduced-motion` は
//!   [`crate::theme::Theme::to_css`] の duration 一括 0ms 化で自動的に
//!   尊重される。**`content` の開閉フェード演出は導入しない**
//!   （[`crate::hover_card`] と同じ理由: headless 層が closed 時に即座に
//!   `hidden` 存在属性を付与し UA 既定 `[hidden] { display: none }` が
//!   同時に適用されるため、transition の開始点・終了待ちのいずれも
//!   成立せず描画されない。PR #1799 の codex-review/Bugbot 指摘と同種の
//!   既知問題であり、headless/実行時層をまたぐ設計変更を要するため本
//!   イシューのスコープ外とする）。
//!
//! ## `close-trigger` のスタイル調整（ゴーストボタン絶対配置は見送り）
//!
//! [`crate::dialog`]（イシュー #1693）は `close-trigger` を `content` 右上へ
//! 絶対配置するゴーストアイコンボタンへ改修し、children 契約をアイコン
//! 専用へ破壊的変更した。本イシューでは popover の `close-trigger` へ同型の
//! 絶対配置化は適用せず、hover surface + transition + canonical フォーカス
//! リング + `border-radius`/`padding` の付与までに留める。理由:
//! 絶対配置化には `content` への `position: relative` 追加と children 契約
//! 変更（アイコン専用化、マイナーバンプ）が連鎖し、参照サイト比較による
//! 意匠是正という本イシューの範囲を超える。必要であれば
//! `.claude/rules/out-of-scope-tracking.md` に従い別イシューとして提案する。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - variant（size 等）ごとのクラス切り替えは headless ラッパー第 1 弾
//!   （#551）と同じくスコープ外とする。
//! - フォーカストラップ・Escape キー閉鎖・外側クリック閉鎖・
//!   `autoFocus`/portal/modal モード・アニメーションは headless 層の
//!   ドキュメント（`crates/headless-ui/src/popover.rs`）で既にスコープ外と
//!   明記済みであり、本モジュールもそれを継承する。
//! - `close-trigger` の絶対配置ゴーストボタン化（上記節参照）・`content`
//!   の開閉フェード演出（上記「トランジション」節参照）はいずれもイシュー
//!   #1534 のスコープ外とする。

use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe,
    StateCondition,
};

// REEXPORT-GLOB-REVIEWED: 本モジュールが定義する pub 項目は stylesheet() の
// みで styled パーツ関数を再定義しない（規約 B-1）。variant 軸（size/
// color-palette）は上記「複合部品の variant 統一方針」方針 3 でオーバー
// レイの配置・寸法がコンテンツ起因の popover/tooltip には提供しないと確定
// 済み（規約 B-2）、CSS 到達は [data-scope]/[data-part] 属性セレクタのみ
// に依存する（規約 B-3、イシュー #1062 規約参照）。
pub use fandhe_frontend_headless_ui::popover::*;
// `root`/`trigger` 等の `state` 引数・`Popover::new`・`Popover` の
// `Component::Action`（dispatch 対象）はいずれも `state` モジュール由来で
// 上記 glob 再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証するための
// 明示再エクスポート（イシュー #685）。
pub use fandhe_frontend_headless_ui::state::{DisclosureAction, OpenState};

/// headless `popover` anatomy の `data-part` 一覧（`crates/headless-ui/src/popover.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "trigger",
    "anchor",
    "positioner",
    "arrow",
    "arrow-tip",
    "content",
    "title",
    "description",
    "close-trigger",
    "indicator",
];

/// この styled Popover の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("popover", SLOTS)
        .base("root", vec![decl("position", "relative")])
        .base(
            "trigger",
            [
                vec![
                    decl("cursor", "pointer"),
                    decl("background", "var(--fandhe-color-bg)"),
                    decl("color", "var(--fandhe-color-fg)"),
                    decl("border", "1px solid var(--fandhe-color-border)"),
                    decl("border-radius", "var(--fandhe-radius-md, 0.375rem)"),
                    decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                    hover_bg_muted(),
                ],
                transition_declarations("background, border-color", MotionDuration::Fast),
            ]
            .concat(),
        )
        .base(
            "positioner",
            vec![
                decl("position", "absolute"),
                decl("top", "100%"),
                decl("left", "0"),
                // イシュー #1534: docs/design/pre-styled-ui-scale-tokens.md
                // §3.4 の z-index 割り当て表で popover は `popover` 段
                // （`--fandhe-z-index-popover`）に分類されている。
                // `stylesheet()` 単独利用者（テーマ CSS 未注入）でも
                // z-index 宣言全体が無効化されないよう旧来値 `10` を
                // fallback に据える（[`crate::hover_card`] と同じ後方互換
                // 方針）。
                decl("z-index", "var(--fandhe-z-index-popover, 10)"),
                decl("margin-top", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "content",
            vec![
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                // イシュー #1534: docs/design/pre-styled-ui-scale-tokens.md
                // §3.1「面パネル = lg」で popover が名指しされているため
                // §5.3 の機械対応表（`radius-md`）ではなく §3.1 を優先する
                // （[`crate::hover_card`] と同じ選択）。
                decl("border-radius", "var(--fandhe-radius-lg, 0.5rem)"),
                // イシュー #1534: 同文書 §3.2「dropdown 型 overlay = md」に
                // 従いトークン化（[`crate::hover_card`] と同じ判断）。
                decl(
                    "box-shadow",
                    "var(--fandhe-shadow-md, 0 4px 6px rgba(0, 0, 0, 0.15))",
                ),
                decl("padding", "var(--fandhe-space-4)"),
                decl("min-width", "var(--fandhe-reference-width, auto)"),
            ],
        )
        .base(
            "title",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("margin", "0 0 var(--fandhe-space-2) 0"),
            ],
        )
        .base(
            "description",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("margin", "0"),
            ],
        )
        .base(
            "close-trigger",
            [
                vec![
                    decl("cursor", "pointer"),
                    decl("color", "var(--fandhe-color-fg-muted)"),
                    decl("background", "transparent"),
                    decl("border", "none"),
                    decl("border-radius", "var(--fandhe-radius-sm)"),
                    decl("padding", "var(--fandhe-space-1)"),
                    hover_bg_muted(),
                ],
                transition_declarations("background", MotionDuration::Fast),
            ]
            .concat(),
        )
        // イシュー #664 受け入れ条件: `trigger`/`content` の開閉状態に応じた見た目の切り替え。
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("border-color", "var(--fandhe-color-accent)")],
        )
        .state(
            "content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("visibility", "hidden")],
        )
        // イシュー #1534: trigger/close-trigger の hover 強調（両者ともボタン
        // 実体で面を持つため hover_bg_muted（base の --fandhe-hover-bg 定義）
        // + hover_surface_declarations の組み合わせを使う。[`crate::dialog`]
        // の trigger/close-trigger と同型）。
        .state(
            "trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "close-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // イシュー #1534: headless 層の trigger は disabled/data-disabled を
        // 出力する（crates/headless-ui/src/popover.rs）。close-trigger は
        // disabled 概念を持たないため対象外（N/A、モジュール doc 参照）。
        .state(
            "trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // キーボード操作時のみのフォーカスリング。イシュー #1534:
        // canonical ヘルパへ移行した（[`crate::hover_card`]/[`crate::dialog`]
        // と同じ判断）。palette 軸を持たない部品のため `FocusRingColor::Token`
        // を使う。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "close-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
}

/// この styled Popover が生成する静的 CSS 全量を返す（決定的。
/// [`crate::dialog::stylesheet`] と同じ契約）。
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
        assert!(a.contains(r#"[data-scope="popover"][data-part="content"]"#));
        assert!(a.contains(r#"[data-scope="popover"][data-part="trigger"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn positioner_is_absolutely_positioned_for_overlay() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="popover"][data-part="positioner"]"#));
        assert!(css.contains("position: absolute;"));
    }

    #[test]
    fn positioner_z_index_uses_popover_token_with_legacy_fallback() {
        // イシュー #1534: docs/design/pre-styled-ui-scale-tokens.md §3.4 の
        // popover 段トークンへ移行しつつ、旧来値 10 を fallback に維持する
        // ことを固定する（stylesheet() 単独利用者のテーマ CSS 未注入時にも
        // z-index 宣言が無効化されない契約）。
        let css = stylesheet();
        assert!(css.contains("z-index: var(--fandhe-z-index-popover, 10);"));
    }

    #[test]
    fn root_provides_containing_block_for_positioner() {
        // trigger/positioner は headless root の下の兄弟要素であり、trigger は
        // positioner の祖先になれない。position: relative は共通祖先の root に
        // 付与する（menu と同じ判断）。
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"popover\"][data-part=\"root\"] {\n  position: relative;\n}\n"
        ));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open_and_closed() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="popover"][data-part="trigger"][data-state="open"]"#));
        assert!(css.contains(r#"[data-scope="popover"][data-part="content"][data-state="closed"]"#));
    }

    #[test]
    fn trigger_and_close_trigger_declare_focus_visible_ring() {
        // イシュー #1534: focus_ring_declarations(FocusRingColor::Token, ...)
        // への canonical 化後の出力を固定する（`crate::recipe` rustdoc 参照）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="popover"][data-part="trigger"]:focus-visible {"#));
        assert!(
            css.contains(r#"[data-scope="popover"][data-part="close-trigger"]:focus-visible {"#)
        );
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(css.contains("outline-offset: var(--fandhe-focus-ring-offset, 2px);"));
    }

    #[test]
    fn trigger_and_close_trigger_declare_hover_surface_under_hover_media_query() {
        // イシュー #1534: trigger/close-trigger とも面を持つボタン実体の
        // ため hover_bg_muted + hover_surface_declarations を新設したことを
        // 固定する。SlotRecipe::css は Hover 規則を
        // `@media (hover: hover)` 配下へ集約出力する契約（recipe.rs 参照）。
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(css
            .contains(r#"[data-scope="popover"][data-part="trigger"]:hover:not([data-disabled])"#));
        assert!(css.contains(
            r#"[data-scope="popover"][data-part="close-trigger"]:hover:not([data-disabled])"#
        ));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
        assert!(css.contains("--fandhe-hover-bg: var(--fandhe-color-bg-muted);"));
    }

    #[test]
    fn trigger_declares_disabled_declarations_for_data_disabled() {
        // イシュー #1534: headless trigger は disabled/data-disabled を
        // 出力する（close-trigger は disabled 概念を持たないため対象外、
        // モジュール doc 参照）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="popover"][data-part="trigger"][data-disabled] {"#));
        assert!(
            !css.contains(r#"[data-scope="popover"][data-part="close-trigger"][data-disabled] {"#)
        );
    }

    #[test]
    fn trigger_and_close_trigger_declare_background_transition() {
        // イシュー #1534: trigger の background/border-color transition、
        // close-trigger の background transition を新設したことを固定する。
        let css = stylesheet();
        assert!(css.contains("transition-property: background, border-color;"));
        assert!(css.contains("transition-property: background;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
    }

    #[test]
    fn trigger_and_content_use_scale_tokens_for_radius_with_legacy_fallback() {
        // イシュー #1534: docs/design/pre-styled-ui-scale-tokens.md
        // §3.1（面パネル = lg、content）と popover 名指しの md（trigger）
        // に従いトークン化しつつ、旧来値を fallback に維持することを固定する。
        let css = stylesheet();
        assert!(css.contains("border-radius: var(--fandhe-radius-md, 0.375rem);"));
        assert!(css.contains("border-radius: var(--fandhe-radius-lg, 0.5rem);"));
        assert!(css.contains("border-radius: var(--fandhe-radius-sm);"));
    }

    #[test]
    fn content_uses_shadow_token_with_legacy_fallback() {
        // イシュー #1534: §3.2「dropdown 型 overlay = md」に従いトークン化
        // しつつ、旧来値を fallback に維持することを固定する。
        let css = stylesheet();
        assert!(css.contains("box-shadow: var(--fandhe-shadow-md, 0 4px 6px rgba(0, 0, 0, 0.15));"));
    }

    #[test]
    fn content_consumes_reference_width_css_var() {
        // イシュー #664 受け入れ条件 2: --fandhe-reference-width を CSS
        // 継承で消費するスタイルが反映されることを固定する（SSR 静的表示では
        // auto へフォールバック。select と同じ判断）。
        let css = stylesheet();
        assert!(css.contains("min-width: var(--fandhe-reference-width, auto);"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="popover""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_popover_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut p = Popover::default();
        assert_eq!(p.state(), OpenState::Closed);

        let ssr_html = render(&p.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut p, "open", ""));
        let hydrate_html = render(&render_for_hydration(&p));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));

        let restored = Popover::from_hydration_attrs(&p.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
    }
}
