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

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, StateCondition, VariantValue};

// headless 自由関数 `root` はあえて再エクスポートしない（本モジュール冒頭の
// rustdoc「選択的 re-export」節参照）。未スタイル・variant クラス非付与の
// 実体が必要な呼び出し側は `fandhe_frontend_headless_ui::accordion` を
// 直接 import する。
pub use fandhe_frontend_headless_ui::accordion::{
    item, item_content, item_indicator, item_trigger, Accordion, MultiAccordion,
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
                decl("border-radius", "0.5rem"),
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
        .base(
            "item-trigger",
            vec![
                decl("display", "flex"),
                decl("width", "100%"),
                decl(
                    "padding",
                    "var(--fandhe-accordion-trigger-padding, var(--fandhe-space-4))",
                ),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("cursor", "pointer"),
                decl("border", "0"),
                decl("text-align", "left"),
            ],
        )
        .base(
            "item-indicator",
            vec![
                decl("display", "inline-block"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
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
        // イシュー #643: キーボード操作時のみのフォーカスリング。
        .state(
            "item-trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
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
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::accordion;
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = accordion::root(Size::Md, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="accordion" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(size: Size, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::accordion::root(merged, children)
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
        let html = render(&root(Size::Md, vec![], vec![]));
        assert!(html.contains(r#"data-scope="accordion""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    // --- イシュー #729: size variant ---

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Sm, Size::Md, Size::Lg] {
            let html = render(&root(size, vec![("class", "attacker")], vec![]));
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

        let ssr_html = render(&a.item("panel-1", false, vec![], vec![]));
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
        // イシュー #643 受け入れ条件: キーボード操作系属性（:focus-visible）
        // が recipe 経由で反映されることを固定する。
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="accordion"][data-part="item-trigger"]:focus-visible {"#)
        );
        assert!(css.contains("outline: 2px solid var(--fandhe-color-accent);"));
    }
}
