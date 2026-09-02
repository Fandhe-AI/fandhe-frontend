//! styled HoverCard（headless ラッパー、イシュー #759、親トラッキング #520/#726）。
//!
//! `fandhe_frontend_headless_ui::hover_card`（イシュー #759）の Root /
//! Trigger / Positioner / Content / Arrow / ArrowTip 6 anatomy パーツと
//! [`fandhe_frontend_headless_ui::hover_card::HoverCard`] 状態機械をそのまま
//! 再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の
//! 根拠・スコープ外事項は [`crate::tooltip`]/[`crate::popover`] の rustdoc と
//! 同じ方針に従う（構造上最も近い先行例は [`crate::tooltip`]）。
//!
//! # data-state とスタイルの連動
//!
//! `content` の開閉 `data-state`（open/closed）に応じた見た目の切り替えを
//! [`recipe`] へ登録する（[`crate::recipe::SlotRecipe::state`]、
//! [`crate::tooltip`] と同じ判断）。
//!
//! # キーボード操作系属性の反映
//!
//! `trigger` はフォーカス可能なリンク要素であり、キーボード操作時のみの
//! フォーカスリング（`:focus-visible`）を
//! [`crate::recipe::StateCondition::FocusVisible`] 経由で登録する
//! （[`crate::tooltip`]/[`crate::popover`] と同じ判断）。
//!
//! # positioner のオーバーレイ配置
//!
//! headless 側 `root`（`crates/headless-ui/src/hover_card.rs`）の子として
//! `trigger`/`positioner` が並置される兄弟関係のため、containing block を
//! 提供する `position: relative` は共通祖先の `root` に付与する
//! （[`crate::popover`]/[`crate::tooltip`] と同じ判断）。`positioner` は
//! `position: absolute; top: 100%; left: 0;
//! z-index: var(--fandhe-z-index-popover, 10)` の dropdown 型オーバーレイ
//! とする（イシュー #1523 で `docs/design/pre-styled-ui-scale-tokens.md`
//! §3.4 の `popover` 段トークンへ移行。[`crate::popover`] と同じ tier。
//! tooltip の `z-index: 1100` より前面性が低い補助オーバーレイとして
//! 扱う）。
//! `positioner` は base 規則で `display` を宣言しないため、closed 時に
//! headless 層が付与する `hidden` 存在属性は UA 既定
//! `[hidden] { display: none }` がそのまま機能する（[`crate::tooltip`]/
//! [`crate::popover`] と同じ構造的な回避、dialog で発生した PR #575 Bugbot
//! 指摘（High）と同種の不具合を避ける）。
//!
//! # `content` は `--fandhe-reference-width` を消費しない
//!
//! [`crate::tooltip`] と同じ判断: hover card の `content` はプレビュー内容へ
//! 幅が追随すべきであり、`sameWidth` 相当は用途として不適切なため意図的に
//! `--fandhe-reference-width` を消費しない（既存の CSS 変数規約自体には
//! 反しない選択であることをここに明記する）。
//!
//! # イシュー #1523 の参照サイト比較（7 軸チェック）
//!
//! chakra-ui / Radix Themes / Radix Primitives / ark-ui と視覚比較した結果を
//! 記録する。
//!
//! - **サイズ / バリアント**: [`crate::popover`]/[`crate::tooltip`] と同じ
//!   判断で単一スケールのまま据え置く（意図的に合わせない。下記スコープ外
//!   節参照）。
//! - **色**: `content` の `box-shadow` に残っていた唯一の生色リテラル
//!   （`rgba(0, 0, 0, 0.15)`）は shadow トークン化（後述）で解消した。
//! - **状態（`data-*`）**: headless 層（`crates/headless-ui/src/
//!   hover_card.rs`）は `disabled` 概念を持たないため disabled 軸は
//!   非該当（N/A）。`content` の `data-state` 連動は既存のまま維持する。
//! - **ダーク**: `content` の影が生リテラルでダーク非追従だった点を、
//!   ダーク値内蔵の `var(--fandhe-shadow-md)` へ移行して解消した。他の
//!   宣言はすべて色トークン参照のみで既にダーク追従済み。
//! - **フォーカス**: `trigger` の `:focus-visible` を直書き 2 宣言から
//!   [`crate::recipe::focus_ring_declarations`]（`FocusRingColor::Token`、
//!   palette 軸を持たないため）へ canonical 化した
//!   （[`crate::accordion`]/[`crate::link`] と同じ判断）。
//! - **余白・角丸・影**: `content` の `border-radius`（生値 `0.375rem`）を
//!   `docs/design/pre-styled-ui-scale-tokens.md` §3.1「面パネル = lg」の
//!   分類に従い `var(--fandhe-radius-lg, 0.5rem)` へ、`box-shadow`
//!   （生値）を同文書 §3.2「dropdown 型 overlay = md」に従い
//!   `var(--fandhe-shadow-md, ...)` へ、`positioner` の `z-index`（生値
//!   `10`）を同文書 §3.4 の `popover` 段トークンへそれぞれ移行した
//!   （fallback は旧来値を維持し `stylesheet()` 単独利用者のテーマ CSS
//!   未注入時にも壊れない、[`crate::toast`]/[`crate::date_picker`] と同じ
//!   後方互換方針）。§5.3 の機械対応表は生値 `0.375rem` を `radius-md` へ
//!   割り当てているが、§3.1 の部品カテゴリ別方針が hover-card を名指しで
//!   「面パネル = lg」に分類しているため本モジュールでは §3.1 を優先した
//!   （[`crate::accordion`] も同じ選択をした先例）。
//! - **hover**: `trigger` に `:hover` 装飾がなかった点を新設した。
//!   [`crate::link`] と同じ理由（面を持たないインラインテキストの slot）で
//!   [`crate::recipe::hover_surface_declarations`] は使わず、`color` のみを
//!   `var(--fandhe-color-accent-emphasized)` へ強調する（palette 軸を持た
//!   ないため直接トークンを参照し、`crate::link` の palette 経由参照とは
//!   異なる）。
//! - **disabled**: 前述のとおり headless 層に disabled 概念がないため
//!   適用しない（N/A）。
//! - **トランジション**: `trigger` の `color` に
//!   [`crate::recipe::transition_declarations`]（`MotionDuration::Fast`）
//!   を、`content` の開閉フェードとして `opacity, visibility` に同
//!   （`MotionDuration::Normal`）を新設した。`content` の closed state へ
//!   `opacity: 0` を追加し、`visibility` は transition 期間中旧値を維持
//!   するため `opacity` が 0 へ落ちきってから非表示化される（フェード
//!   アウトの視認性を確保する意図的な多重指定）。
//!   `prefers-reduced-motion` は [`crate::theme::Theme::to_css`] の
//!   duration 一括 0ms 化で自動的に尊重される。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - variant（size 等）ごとのクラス切り替えは他の styled 部品と同じく
//!   スコープ外とする（イシュー #1523 でも見送りを継続）。
//! - `openDelay`/`closeDelay`/`interactive` は headless 層のドキュメント
//!   （`crates/headless-ui/src/hover_card.rs`）で既にスコープ外と明記済みの
//!   クライアントサイド実行時挙動であり、本モジュールもそれを継承する。
//! - `--fandhe-x`/`--fandhe-y`/`--fandhe-arrow-*`（座標ジオメトリ）は
//!   [`crate::tooltip`]/[`crate::popover`] と同じ理由で本イシューの対象外。

use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, transition_declarations, FocusRingColor, FocusRingOffset,
    MotionDuration, SlotRecipe, StateCondition,
};

// REEXPORT-GLOB-REVIEWED: 本モジュールが定義する pub 項目は stylesheet() の
// みで styled パーツ関数を再定義しない（規約 B-1）。オーバーレイ配置系の
// [`crate::popover`]/[`crate::tooltip`] と同じ判断で variant 軸を提供せず
// （規約 B-2）、CSS 到達は [data-scope]/[data-part] 属性セレクタのみに依存
// する（規約 B-3、イシュー #1062 規約参照）。
pub use fandhe_frontend_headless_ui::hover_card::*;
// `root`/`trigger` 等の `state` 引数・`HoverCard::new`・`HoverCard` の
// `Component::Action`（dispatch 対象）はいずれも `state` モジュール由来で
// 上記 glob 再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証するための
// 明示再エクスポート（[`crate::tooltip`]/[`crate::popover`] と同じ判断、
// イシュー #685）。
pub use fandhe_frontend_headless_ui::state::{DisclosureAction, OpenState};

/// headless `hover-card` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/hover_card.rs` の `ANATOMY.part(...)` 呼び出しと
/// 同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を出力しない
/// fail-closed 側の不具合として現れるため、変更時は両ファイルを合わせて
/// 確認する）。
const SLOTS: &[&str] = &[
    "root",
    "trigger",
    "positioner",
    "content",
    "arrow",
    "arrow-tip",
];

/// この styled HoverCard の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("hover-card", SLOTS)
        .base("root", vec![decl("position", "relative")])
        .base(
            "trigger",
            vec![
                decl("color", "var(--fandhe-color-accent)"),
                decl("cursor", "pointer"),
                decl("text-decoration", "underline"),
            ],
        )
        // trigger の色変化（hover/focus 出入り）を滑らかにする（イシュー
        // #1523、[`crate::link`] と同じ判断）。
        .base(
            "trigger",
            transition_declarations("color", MotionDuration::Fast),
        )
        .base(
            "positioner",
            vec![
                decl("position", "absolute"),
                decl("top", "100%"),
                decl("left", "0"),
                // イシュー #1523: `docs/design/pre-styled-ui-scale-tokens.md`
                // §3.4 の z-index 割り当て表で hover-card は `popover` 段
                // （`--fandhe-z-index-popover`）に分類されている。
                // `Theme::to_css` を経由しない `stylesheet()` 単独利用者
                // （テーマ CSS 未注入）でもトークン未定義のまま `z-index`
                // 宣言全体が無効化されないよう、旧来値 `10` を fallback に
                // 据える（`date_picker.rs`/`toast.rs` と同じ後方互換方針）。
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
                // イシュー #1523: 同文書 §3.1「面パネル = lg」で hover-card
                // が名指しされているため §5.3 の機械対応表（`radius-md`）
                // ではなく §3.1 を優先する（[`crate::accordion`] も root に
                // `radius-lg` を採用した先例と同じ判断）。フォールバックは
                // [`crate::dialog`] と同じく旧来値を維持する。
                decl("border-radius", "var(--fandhe-radius-lg, 0.5rem)"),
                // イシュー #1523: 同文書 §3.2「dropdown 型 overlay = md」
                // に従いトークン化（[`crate::action_bar`]/[`crate::toast`]
                // と同じ判断）。フォールバックは旧来値を維持する。
                decl(
                    "box-shadow",
                    "var(--fandhe-shadow-md, 0 4px 6px rgba(0, 0, 0, 0.15))",
                ),
                decl("padding", "var(--fandhe-space-4)"),
                decl("max-width", "20rem"),
            ],
        )
        // content の開閉フェード（イシュー #1523）。`opacity` の
        // transition のみでは `visibility: hidden` が即座に切り替わり
        // フェードアウトが視認できないため `visibility` も transition
        // 対象に含める（transition 期間中は旧値が維持されるため、
        // opacity が 0 へ落ちきってから非表示化される。
        // `prefers-reduced-motion` は [`crate::theme::Theme::to_css`] の
        // duration 一括 0ms 化で自動的に尊重される）。
        .base(
            "content",
            transition_declarations("opacity, visibility", MotionDuration::Normal),
        )
        // `content` の開閉状態に応じた見た目の切り替え（[`crate::tooltip`] と
        // 同じ判断）。
        .state(
            "content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("visibility", "hidden"), decl("opacity", "0")],
        )
        // trigger の hover 強調（イシュー #1523）。hover-card の trigger は
        // 面を持たないインラインテキストの slot であり、
        // [`crate::link`] と同じ理由で `hover_surface_declarations` は
        // 使わず `color` のみを強調する。palette 軸を持たない部品のため
        // `crate::link` の palette 経由参照ではなく直接トークンを参照する。
        .state(
            "trigger",
            StateCondition::Hover,
            vec![decl("color", "var(--fandhe-color-accent-emphasized)")],
        )
        // キーボード操作時のみのフォーカスリング。イシュー #1523:
        // `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
        // に従い canonical ヘルパへ移行した（[`crate::accordion`]/
        // [`crate::link`] と同じ判断）。palette 軸を持たない部品のため
        // `FocusRingColor::Token` を使う。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
}

/// この styled HoverCard が生成する静的 CSS 全量を返す（決定的。
/// [`crate::tooltip::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use fandhe_frontend_headless_ui::hover_card::HoverCardDelays;
    use fandhe_frontend_headless_ui::state::OpenState;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="hover-card"][data-part="content"]"#));
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
        assert!(css.contains(r#"[data-scope="hover-card"][data-part="positioner"]"#));
        assert!(css.contains("position: absolute;"));
        assert!(css.contains("top: 100%;"));
    }

    #[test]
    fn positioner_z_index_uses_popover_token_with_legacy_fallback() {
        // イシュー #1523: docs/design/pre-styled-ui-scale-tokens.md §3.4 の
        // popover 段トークンへ移行しつつ、旧来値 10 を fallback に維持する
        // ことを固定する（stylesheet() 単独利用者のテーマ CSS 未注入時にも
        // z-index 宣言が無効化されない契約）。
        let css = stylesheet();
        assert!(css.contains("z-index: var(--fandhe-z-index-popover, 10);"));
    }

    #[test]
    fn root_provides_containing_block_for_positioner() {
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"hover-card\"][data-part=\"root\"] {\n  position: relative;\n}\n"
        ));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open_and_closed() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="hover-card"][data-part="content"][data-state="closed"]"#)
        );
    }

    #[test]
    fn trigger_declares_focus_visible_ring() {
        // イシュー #1523: focus_ring_declarations(FocusRingColor::Token, ...)
        // への canonical 化後の出力を固定する（`crate::recipe` rustdoc 参照）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="hover-card"][data-part="trigger"]:focus-visible {"#));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(css.contains("outline-offset: var(--fandhe-focus-ring-offset, 2px);"));
    }

    #[test]
    fn trigger_declares_hover_color_emphasis_under_hover_media_query() {
        // イシュー #1523: trigger は面を持たないインラインテキストの slot
        // のため hover_surface_declarations ではなく color のみを強調する
        // （[`crate::link`] と同じ判断）。SlotRecipe::css は Hover 規則を
        // `@media (hover: hover)` 配下へ集約出力する契約（recipe.rs 参照）。
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(
            r#"[data-scope="hover-card"][data-part="trigger"]:hover:not([data-disabled])"#
        ));
        assert!(css.contains("color: var(--fandhe-color-accent-emphasized);"));
    }

    #[test]
    fn trigger_and_content_declare_transitions() {
        // イシュー #1523: trigger の色変化・content の開閉フェードへ
        // transition_declarations を新設したことを固定する。
        let css = stylesheet();
        assert!(css.contains("transition-property: color;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
        assert!(css.contains("transition-property: opacity, visibility;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-normal);"));
    }

    #[test]
    fn content_uses_scale_tokens_for_radius_and_shadow_with_legacy_fallback() {
        // イシュー #1523: docs/design/pre-styled-ui-scale-tokens.md
        // §3.1（面パネル = lg）・§3.2（dropdown 型 overlay = md）に従い
        // トークン化しつつ、旧来値を fallback に維持することを固定する。
        let css = stylesheet();
        assert!(css.contains("border-radius: var(--fandhe-radius-lg, 0.5rem);"));
        assert!(css.contains("box-shadow: var(--fandhe-shadow-md, 0 4px 6px rgba(0, 0, 0, 0.15));"));
    }

    #[test]
    fn closed_content_fades_out_via_opacity_and_visibility() {
        // イシュー #1523: 開閉フェードのため closed state へ opacity: 0 を
        // 追加したことを固定する（visibility: hidden との併用理由はモジュール
        // doc の「トランジション」節参照）。
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"hover-card\"][data-part=\"content\"][data-state=\"closed\"] {\n  visibility: hidden;\n  opacity: 0;\n}\n"
        ));
    }

    #[test]
    fn content_does_not_consume_reference_width_css_var() {
        // hover card の content はプレビュー内容へ幅が追随すべきであり、
        // sameWidth 相当は不適切なため意図的に --fandhe-reference-width を
        // 消費しないことを固定する（モジュール doc 参照）。
        let css = stylesheet();
        assert!(!css.contains("--fandhe-reference-width"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(
            OpenState::Closed,
            HoverCardDelays::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="hover-card""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_hover_card_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut hc = HoverCard::default();
        assert_eq!(hc.state(), OpenState::Closed);

        let ssr_html = render(&hc.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut hc, "open", ""));
        let hydrate_html = render(&render_for_hydration(&hc));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));

        let restored = HoverCard::from_hydration_attrs(&hc.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
    }
}
