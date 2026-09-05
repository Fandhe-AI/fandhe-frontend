//! styled Accordion（headless ラッパー第 1 弾、イシュー #551、親 #520/#545。
//! `size` variant 展開はイシュー #729、親 #708）。
//!
//! `fandhe_frontend_headless_ui::accordion`（イシュー #527）の Root / Item /
//! ItemTrigger / ItemIndicator / ItemContent 5 anatomy パーツと
//! [`fandhe_frontend_headless_ui::accordion::Accordion`] 状態機械（single
//! モード）を再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。
//! 薄い委譲の根拠・スコープ外事項は [`crate::dialog`] の rustdoc と同じ方針に従う
//! （`data-scope`/`data-part` セレクタへの CSS 適用のみで、パーツ関数へ手を
//! 加えない）。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、イシュー #729）
//!
//! `size` variant クラス付与のため styled [`root`]（[`crate::switch::root`]
//! と同型）を本モジュールで新設する。headless 自由関数 `root` と名前が
//! 衝突するため、`pub use ...::*` ではなく必要な識別子（[`item`]/
//! [`item_trigger`]/[`item_indicator`]/[`item_content`]/[`Accordion`]/
//! [`MultiAccordion`] 等）のみを選択的に再エクスポートする。headless 自由
//! 関数 `root`（未スタイル・variant クラス非付与）が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::accordion` を直接 import すること。
//! `Accordion`/`MultiAccordion` は状態機械であり root への inherent メソッド
//! を持たない（`item`/`item_trigger` 等のみを介する設計）ため、[`crate::switch`]
//! の `Switch` 非再エクスポートとは異なり従来通り再エクスポートを維持する。
//!
//! # data-state とスタイルの連動（イシュー #551 受け入れ条件）
//!
//! 項目の開閉 `data-state`（open/closed）に応じて `item-trigger`/
//! `item-indicator` の見た目を切り替える CSS を [`recipe`] へ登録する
//! ([`crate::recipe::SlotRecipe::state`]、イシュー #643。`serialize_rule` を
//! 直接呼ぶ手書きセレクタ機構は廃止した)。`item-indicator` は headless 層
//! （`crates/headless-ui/src/accordion.rs`）でデフォルト `span`（非置換インライン
//! 要素）としてレンダリングされ `transform` が効かないため、[`recipe`] の
//! base 規則で `display: inline-block` を設定し `rotate(180deg)` が実際に
//! 適用されるようにする（PR #575 Bugbot 指摘対応）。
//!
//! # キーボード操作系スタイル（イシュー #643）
//!
//! `item-trigger` は roving tabindex でフォーカス移動するボタン要素であり、
//! キーボード操作時のみのフォーカスリング（`:focus-visible`）を [`recipe`]
//! へ登録する。
//!
//! # `size` variant（イシュー #729）
//!
//! `size`（[`Size`]）は [`root`] へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-accordion-trigger-padding`/`-content-padding` の root スコープ
//! CSS custom property（通常の CSS 継承により `item-trigger`/`item-content`
//! へ伝わる。`root` は両パーツを内包する祖先要素であるため、
//! [`crate::recipe::SlotRecipe`] へ子孫セレクタ機構を追加せずに実現できる）
//! 経由で寸法を切り替える。`base` 規則の `var()` には Md サイズ相当の
//! フォールバック値を書き、styled `root` を経由しない headless 直接利用
//! マークアップでも現行外観を維持する（fail-safe、`crate::lib` rustdoc
//! 「複合部品の variant 統一方針」節参照）。accordion は `color-palette`
//! 軸を持たない（variant 表の方針、`docs/api/pre-styled-ui-api.md` §4d 参照）。
//!
//! # 参考サイト基準への調整（イシュー #1515）
//!
//! 参照 3 サイト（chakra-ui / Radix Primitives / ark-ui）と比較し、
//! 以下を [`recipe`] へ追加した: `root` の角丸トークン化
//! （`--fandhe-radius-lg`）・最終 item の二重罫線解消
//! （[`crate::recipe::StateCondition::LastChild`]）・`item-trigger` の
//! ラベル左/シェブロン右レイアウトと見出し級タイポ・hover
//! （[`crate::recipe::hover_bg_muted`] + [`crate::recipe::StateCondition::Hover`]）・
//! disabled（headless 層が出力する `data-disabled` の CSS 消費）・
//! transition（`item-trigger`/`item-indicator`）・フォーカスリングの
//! canonical 化（[`crate::recipe::focus_ring_declarations`]）。
//!
//! 以下は意図的に非採用とした（`docs/policy/intentional-non-adoption.md`
//! の評価軸を再確認せず単独判断で持ち込まない）:
//!
//! - **variant 軸（chakra `outline`/`subtle`/`enclosed`/`plain` 相当）の
//!   新設**: 現行既定は chakra `enclosed` 相当（外枠 + 角丸）であり用途を
//!   満たす。軸の新設は [`root`] の公開シグネチャ変更（0.x 破壊的変更・
//!   minor バンプ）と Demo/原稿の波及を伴うため、toggle（#1512 → PR #1785
//!   で見送り済み）と同じ判断を踏襲する。
//! - **開閉時の高さアニメーション**（Radix
//!   `--radix-accordion-content-height` 相当）: content 高さの実測（JS）が
//!   前提であり、レイアウト計測の関心を `headless-ui` へ持ち込まない方針
//!   （`docs/policy/intentional-non-adoption.md` §3.25）と、docs サイトの
//!   無 JS 制約に反するため非採用。
//! - **size 軸・palette 軸の追加**: size は既に 5 段（xs〜xl）で chakra と
//!   同数のため過不足なし。palette 軸非保有は本モジュール冒頭の既定方針の
//!   まま維持する。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe,
    StateCondition, VariantValue,
};

// headless 自由関数 `root` はあえて再エクスポートしない（本モジュール冒頭の
// rustdoc「選択的 re-export」節参照）。未スタイル・variant クラス非付与の
// 実体が必要な呼び出し側は `fandhe_frontend_headless_ui::accordion` を
// 直接 import する。
// `AccordionProps`（イシュー #1636、orientation/disabled）は `item`/
// `item_trigger`/`item_indicator`/`item_content` の全パーツ関数と本モジュール
// の styled `root` が引数に取るため、`item`/`Accordion` 等と同じく明示
// 再エクスポートする（呼び出し側が `fandhe-frontend-pre-styled-ui` のみへの
// 依存で完結できることを保証する、イシュー #685 と同じ判断）。
pub use fandhe_frontend_headless_ui::accordion::{
    item, item_content, item_indicator, item_trigger, Accordion, AccordionProps, MultiAccordion,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
// `item`/`item_trigger`/`item_indicator`/`item_content` の `state` 引数・
// `Accordion::item_state` 戻り値・`Accordion`/`MultiAccordion` の
// `Component::Action`（dispatch 対象）はいずれも `state` モジュール由来で
// 上記選択的再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証するための
// 明示再エクスポート（イシュー #685）。
pub use fandhe_frontend_headless_ui::state::{MultiSelectAction, OpenState, SingleSelectAction};

/// headless `accordion` anatomy の `data-part` 一覧（`crates/headless-ui/src/accordion.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &[
    "root",
    "item",
    "item-trigger",
    "item-indicator",
    "item-content",
];

/// この styled Accordion の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("accordion", SLOTS)
        .base(
            "root",
            vec![
                decl("border", "1px solid var(--fandhe-color-border)"),
                // イシュー #1515: 生値 `0.5rem` からトークン参照へ（値は不変、
                // `docs/design/pre-styled-ui-scale-tokens.md` §5.3 の写像表
                // どおり `radius-lg`）。
                decl("border-radius", "var(--fandhe-radius-lg)"),
                decl("overflow", "hidden"),
            ],
        )
        .base(
            "item",
            vec![decl(
                "border-bottom",
                "1px solid var(--fandhe-color-border-muted)",
            )],
        )
        // イシュー #1515: 最終 item は `root` の外枠 border と `item` の
        // `border-bottom` が重なり二重罫線に見えるため打ち消す
        // （`steps.rs` の `LastChild` 先例と同型。同一 slot への状態規則は
        // 登録順の後勝ちで上書きされる契約のため base より後に置く）。
        .state(
            "item",
            StateCondition::LastChild,
            vec![decl("border-bottom", "0")],
        )
        .base(
            "item-trigger",
            vec![
                decl("display", "flex"),
                // イシュー #1515: 参照 3 サイト（chakra-ui / Radix Primitives
                // / ark-ui）共通のラベル左・シェブロン右配置。
                decl("align-items", "center"),
                decl("justify-content", "space-between"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("width", "100%"),
                decl(
                    "padding",
                    "var(--fandhe-accordion-trigger-padding, var(--fandhe-space-4))",
                ),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                // イシュー #1515: 見出し級のタイポ（参照 3 サイト共通）。
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("cursor", "pointer"),
                decl("border", "0"),
                decl("text-align", "left"),
                // イシュー #1515: hover 面色（off 面 1 色のみのため
                // `hover_bg_muted()` を base 直置きにする。variant 軸を
                // 持たない部品のため `crate::toggle` のような on/off 面差し
                // 替えは不要、`crate::recipe` 冒頭 doc「hover」節参照）。
                hover_bg_muted(),
            ],
        )
        .base(
            "item-trigger",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .base(
            "item-indicator",
            vec![
                decl("display", "inline-block"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "item-indicator",
            transition_declarations("transform", MotionDuration::Normal),
        )
        .base(
            "item-content",
            vec![
                decl(
                    "padding",
                    "var(--fandhe-accordion-content-padding, var(--fandhe-space-4))",
                ),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        // イシュー #551 受け入れ条件: 開いている項目の trigger/indicator を強調する。
        .state(
            "item-trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("color", "var(--fandhe-color-accent)")],
        )
        .state(
            "item-indicator",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("transform", "rotate(180deg)")],
        )
        // イシュー #1515: hover の実適用は 1 本のみ（`--fandhe-hover-bg` の
        // 間接参照経由。`Hover` は `:not([data-disabled])` 込みで
        // `@media (hover: hover)` へ集約出力される既存機構、`crate::toggle`
        // と同型のパターン）。
        .state(
            "item-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // イシュー #1515: headless 層（`crates/headless-ui/src/accordion.rs`）
        // が `item`/`item-trigger` へ出力する `data-disabled` を CSS 側で
        // 消費する（従来は規則がなく視覚差が付かなかった）。
        .state(
            "item-trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // イシュー #643 → #1515: キーボード操作時のみのフォーカスリング。
        // 直書き 2 宣言を Phase 0（#1424）の canonical ヘルパへ置換する。
        // `Token`: accordion は palette 軸を持たない部品
        // （`docs/api/pre-styled-ui-api.md` §4d）。`Inset`: `root` が
        // `overflow: hidden` を持ち、外側リング（`Outside`）は root 境界で
        // 切れるため（`FocusRingOffset::Inset` の rustdoc が明記する想定
        // ユースケース。docs-site ローカルビルドで視覚確認済み）。
        .state(
            "item-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Inset),
        )
        // イシュー #729: `size` variant（root スコープの CSS custom property。
        // Md はフォールバック値と同一の現行外観を維持する）。
        // イシュー #1681: Xs/Xl は Sm→Md→Lg の `space-3`/`4`/`5` の等差進行を
        // 両端へ 1 段ずつ外挿した `space-2`/`space-6`。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl(
                    "--fandhe-accordion-trigger-padding",
                    "var(--fandhe-space-2)",
                ),
                decl(
                    "--fandhe-accordion-content-padding",
                    "var(--fandhe-space-2)",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl(
                    "--fandhe-accordion-trigger-padding",
                    "var(--fandhe-space-3)",
                ),
                decl(
                    "--fandhe-accordion-content-padding",
                    "var(--fandhe-space-3)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl(
                    "--fandhe-accordion-trigger-padding",
                    "var(--fandhe-space-4)",
                ),
                decl(
                    "--fandhe-accordion-content-padding",
                    "var(--fandhe-space-4)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl(
                    "--fandhe-accordion-trigger-padding",
                    "var(--fandhe-space-5)",
                ),
                decl(
                    "--fandhe-accordion-content-padding",
                    "var(--fandhe-space-5)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl(
                    "--fandhe-accordion-trigger-padding",
                    "var(--fandhe-space-6)",
                ),
                decl(
                    "--fandhe-accordion-content-padding",
                    "var(--fandhe-space-6)",
                ),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled Accordion が生成する静的 CSS 全量を返す（決定的。[`crate::dialog::stylesheet`]
/// と同じ契約: 同一プロセス内の複数回呼び出しは常にバイト単位で同一の文字列を返す）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::accordion::root`] へ
/// 委譲する。`props`（[`AccordionProps`]、イシュー #1636）は
/// `data-orientation`/実効 disabled として全パーツへ伝わる（本 styled
/// `root` は CSS レシピを変更せず、headless 側へそのまま透過する）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::accordion::{self, AccordionProps};
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = accordion::root(Size::Md, &AccordionProps::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="accordion" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    props: &AccordionProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::accordion::root(props, merged, children)
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
        assert!(a.contains(r#"[data-scope="accordion"][data-part="item-trigger"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn item_indicator_has_transformable_display() {
        // PR #575 Bugbot 指摘対応: item-indicator が transform 可能な display
        // （非デフォルトのインラインボックス）を持つことを固定する。base の
        // `display: inline-block` がないと `transform: rotate(180deg)`
        // （state_css）が実際には効かない。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="accordion"][data-part="item-indicator"] {"#));
        assert!(css.contains("display: inline-block;"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(Size::Md, &AccordionProps::default(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="accordion""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    // --- イシュー #729: size variant ---

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Sm, Size::Md, Size::Lg] {
            let html = render(&root(
                size,
                &AccordionProps::default(),
                vec![("class", "attacker")],
                vec![],
            ));
            let expected_class = format!("fd-accordion--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn default_variant_is_md_and_matches_pre_729_fallback() {
        // Md はフォールバック値と同一の現行外観を維持する（不変条件）。
        let css = stylesheet();
        assert!(css
            .contains("padding: var(--fandhe-accordion-trigger-padding, var(--fandhe-space-4));"));
        assert!(css
            .contains("padding: var(--fandhe-accordion-content-padding, var(--fandhe-space-4));"));
    }

    #[test]
    fn styled_root_propagates_props_to_headless_data_orientation() {
        // イシュー #1636: styled root は `AccordionProps` をそのまま headless
        // `root` へ透過する（CSS レシピは変更しない）。
        use fandhe_frontend_headless_ui::data_attrs::Orientation;
        let props = AccordionProps {
            orientation: Orientation::Horizontal,
            disabled: false,
        };
        let html = render(&root(Size::Md, &props, vec![], vec![]));
        assert!(html.contains(r#"data-orientation="horizontal""#));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open() {
        // イシュー #551 受け入れ条件: 「headless 層の data-state とスタイルの
        // 連動テスト（[data-state='open'] セレクタ等）」を固定する。
        let css = stylesheet();
        assert!(css
            .contains(r#"[data-scope="accordion"][data-part="item-trigger"][data-state="open"]"#));
        assert!(css.contains(
            r#"[data-scope="accordion"][data-part="item-indicator"][data-state="open"]"#
        ));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_accordion_state_machine() {
        // イシュー #551 受け入れ条件: 「SSR / hydration 両経路の動作確認」を
        // 再エクスポートされた `Accordion`（headless の Component/Hydrate 実装を
        // そのまま継承）経由で固定する。
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut a = Accordion::default();
        assert_eq!(a.expanded(), None);

        let ssr_html =
            render(&a.item("panel-1", false, &AccordionProps::default(), vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut a, "select", "panel-1"));
        assert_eq!(a.expanded(), Some("panel-1"));

        let hydrate_html = render(&render_for_hydration(&a));
        assert!(hydrate_html.contains("data-hydrate-"));

        let restored = Accordion::from_hydration_attrs(&a.hydration_attrs()).unwrap();
        assert_eq!(restored.expanded(), Some("panel-1"));
    }

    #[test]
    fn item_trigger_declares_focus_visible_ring() {
        // イシュー #643 → #1515 受け入れ条件: キーボード操作系属性
        // （:focus-visible）が recipe 経由で反映されることを固定する。
        // #1515 で直書き 2 宣言を canonical ヘルパ（`focus_ring_declarations`、
        // `FocusRingColor::Token` + `FocusRingOffset::Inset`）へ置換した。
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="accordion"][data-part="item-trigger"]:focus-visible {"#)
        );
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(css.contains("outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));"));
    }

    // --- イシュー #1515: 参考サイト基準への調整 ---

    #[test]
    fn root_border_radius_uses_radius_lg_token() {
        // 角丸の生値 `0.5rem` をトークン参照へ（値は不変）。
        let css = stylesheet();
        assert!(css.contains("border-radius: var(--fandhe-radius-lg);"));
        assert!(!css.contains("border-radius: 0.5rem;"));
    }

    #[test]
    fn last_item_has_no_double_border() {
        // root の外枠 border と item の border-bottom が重なる二重罫線を
        // last-child で打ち消す。
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="accordion"][data-part="item"]:last-child {"#),
            "css={css}"
        );
        assert!(css.contains("border-bottom: 0;"));
    }

    #[test]
    fn item_trigger_declares_hover_and_disabled_and_transition() {
        let css = stylesheet();
        // hover: タッチ端末の hover 貼り付き対策のため `@media (hover: hover)`
        // 配下へ集約出力される（`crate::recipe::StateCondition::Hover`）。
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(
            r#"[data-scope="accordion"][data-part="item-trigger"]:hover:not([data-disabled]) {"#
        ));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
        assert!(css.contains("--fandhe-hover-bg: var(--fandhe-color-bg-muted);"));
        // disabled: headless 層が出力する `data-disabled` を CSS 側で消費する。
        assert!(
            css.contains(r#"[data-scope="accordion"][data-part="item-trigger"][data-disabled] {"#)
        );
        assert!(css.contains("opacity: 0.5;"));
        assert!(css.contains("cursor: not-allowed;"));
        // transition: 開閉・hover に伴う視覚変化へ transition を当てる。
        assert!(css.contains("transition-property: background, color;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
        assert!(css.contains("transition-property: transform;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-normal);"));
    }

    #[test]
    fn item_trigger_has_space_between_layout_and_medium_weight() {
        // 参照 3 サイト（chakra-ui / Radix Primitives / ark-ui）共通の
        // ラベル左・シェブロン右配置と見出し級タイポ。
        let css = stylesheet();
        let block_start = css
            .find(r#"[data-scope="accordion"][data-part="item-trigger"] {"#)
            .expect("item-trigger base block must exist");
        let block_end = css[block_start..]
            .find('}')
            .map(|i| block_start + i)
            .expect("item-trigger base block must be closed");
        let block = &css[block_start..block_end];
        assert!(block.contains("align-items: center;"), "block={block}");
        assert!(
            block.contains("justify-content: space-between;"),
            "block={block}"
        );
        assert!(
            block.contains("font-weight: var(--fandhe-font-font-weight-medium);"),
            "block={block}"
        );
    }
}
