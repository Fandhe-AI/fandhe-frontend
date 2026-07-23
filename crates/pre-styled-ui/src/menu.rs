//! styled Menu（headless ラッパー第 1 弾、イシュー #551、親 #520/#545。
//! `size` variant 展開はイシュー #729、親 #708）。
//!
//! `fandhe_frontend_headless_ui::menu`（イシュー #540）の Root / Trigger /
//! Indicator / Positioner / Content / Arrow / ArrowTip / Item / ItemGroup /
//! ItemGroupLabel / Separator 11 anatomy パーツを再エクスポートし、
//! [`stylesheet`] で既定 CSS を追加提供する。薄い委譲の根拠・スコープ外
//! 事項は [`crate::dialog`] の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Menu` 型・headless
//! `root` を再エクスポートしない理由、イシュー #729）
//!
//! `size` variant クラス付与のため styled [`root`]（[`crate::dialog::root`]
//! と同型）を本モジュールで新設する。headless 自由関数 `root` と名前が
//! 衝突するため、`pub use ...::*` ではなく必要な識別子のみを選択的に再
//! エクスポートする。状態機械 [`fandhe_frontend_headless_ui::menu::Menu`] は
//! **あえて**再エクスポートしない（[`crate::switch`]/[`crate::dialog`] の
//! 状態機械非再エクスポートと同じ理由）。`Menu` による状態管理・hydration が
//! 必要な呼び出し側は `fandhe_frontend_headless_ui::menu::Menu` を直接
//! import し、実際の描画は本モジュールの styled [`root`]（および再エクスポート
//! 済みのパーツ関数）を組み合わせて構築すること。
//!
//! # data-state とスタイルの連動（イシュー #551 受け入れ条件）
//!
//! `trigger`/`content` の開閉 `data-state`（open/closed）に応じた見た目の
//! 切り替えを [`recipe`] へ登録する（[`crate::recipe::SlotRecipe::state`]、
//! イシュー #643。`serialize_rule` を直接呼ぶ手書きセレクタ機構は廃止した）。
//!
//! # キーボード操作系属性の反映（イシュー #643）
//!
//! `item` は headless 層（`crates/headless-ui/src/menu.rs`）の virtual focus
//! パターン（イシュー #581）でハイライトされる。実 DOM フォーカスは
//! `trigger` に留まり続け、選択中の項目には `data-highlighted` 属性が
//! 付与される契約のため、`item` の highlight 表示は
//! [`crate::recipe::StateCondition::Attr`]`("data-highlighted")` で反映し、
//! `:focus-visible` は付けない（フォーカスが実際に来ないパーツへ付けても
//! 発火しないため）。`trigger` は実際にフォーカスを受けるボタン要素のため
//! `:focus-visible` によるフォーカスリングを登録する。
//!
//! # `--fandhe-reference-width` の消費（イシュー #643）
//!
//! `crates/wasm-full/src/position.rs::reposition_one`（イシュー #588）が
//! `positioner` の `style` 属性へ書き込む `--fandhe-reference-width`
//! （`trigger` の実測幅、CSS カスタムプロパティ継承で子孫の `content` から
//! 参照可能）を `content` の `min-width` が `var(--fandhe-reference-width,
//! 10rem)` として消費し、chakra-ui の `sameWidth` 相当（listbox 幅がトリガー
//! 幅へ追随する見た目）を実現する。wasm 未稼働の SSR 静的表示では変数が
//! 未定義のため `10rem`（従来の固定値）へフォールバックする。
//! # 位置ジオメトリ（`--fandhe-x`/`--fandhe-y`/`--fandhe-arrow-*`）の消費（イシュー #663）
//!
//! SSR 静的フォールバック（`positioner` の `position: absolute; top: 100%;
//! left: 0`、`root` の `position: relative` をローカル座標系とする）と、
//! `crates/wasm-full/src/position.rs::wiring::reposition_one` が書き込む
//! 確定座標（`getBoundingClientRect`/`window.innerWidth/innerHeight` 由来の
//! **viewport 原点**座標）は座標系が異なるため、`positioner` へ書き込まれる
//! `data-positioned`（値なしの存在マーカー、wasm 層のみが付与し headless 層
//! の SSR/SSG 出力には決して現れない）の有無で `position` 種別ごと切り替える
//! （`docs/design/anchor-positioning-design.md` §4.4b 参照）。マーカーが
//! 無い場合は本来の静的フォールバックのまま（wasm 未稼働環境でも表示が
//! 壊れない fail-closed 動作）:
//!
//! ```css
//! [data-scope="menu"][data-part="positioner"][data-positioned] {
//!   position: fixed;
//!   top: 0;
//!   left: 0;
//!   margin-top: 0;
//!   transform: translate3d(var(--fandhe-x, 0px), var(--fandhe-y, 0px), 0);
//! }
//! ```
//!
//! `arrow`/`arrow-tip`（Menu のみ、ADR §4.2 で Select は arrow 非対象）は
//! マーカー切り替え不要で変数フォールバックのみで両立する。
//! `reposition_one` は positioner の `style`（CSS カスタムプロパティは
//! 子孫へ継承される）に加えて arrow 要素自身の `style` へも同じ値を複製
//! するため、arrow の base 規則で直接 `var(--fandhe-arrow-x, 50%)`/
//! `var(--fandhe-arrow-y, 0)` を参照できる（フォールバック値は SSR 既定
//! placement（bottom）で anchor 中央上端に相当する）。
//!
//! # positioner のオーバーレイ配置（PR #575 Bugbot 指摘対応）
//!
//! `positioner` に `position: absolute` を設定し、開いた menu が通常のフローに
//! 残らずオーバーレイ表示になるようにする（[`crate::dialog`] の `positioner`・
//! [`crate::select`] の `positioner` と同じ配置責務）。`trigger`/`positioner` は
//! headless 側 `root`（`crates/headless-ui/src/menu.rs`）の子として並置される
//! 兄弟要素であり、`trigger` は `positioner` の祖先になれない。そのため
//! containing block を提供する `position: relative` は共通の祖先である `root`
//! に付与する（PR #575 Bugbot 指摘 1 対応、`trigger` への誤付与を修正）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, StateCondition, VariantValue};

// headless 自由関数 `root`・状態機械 `Menu` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。未スタイル・
// variant クラス非付与の実体・状態管理が必要な呼び出し側は
// `fandhe_frontend_headless_ui::menu` を直接 import する。
// `MenuCheckboxItem`/`MenuRadioItemGroup` は `Menu` と異なり root への
// inherent メソッドを持たず、未スタイル root の静かな適用漏れが起きないため
// 従来通り再エクスポートを維持する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::menu::{
    arrow, arrow_tip, checkbox_item, content, context_trigger, indicator, item, item_group,
    item_group_label, positioner, radio_item, radio_item_group, separator, trigger, trigger_item,
    MenuCheckboxItem, MenuRadioItemGroup,
};
// `trigger`/`trigger_item`/`context_trigger` 等の `state` 引数・
// `MenuCheckboxItem`/`MenuRadioItemGroup` の `Component::Action`
// （dispatch 対象）はいずれも `state` モジュール由来で上記選択的再エクスポート
// では到達しない。呼び出し側が `fandhe-frontend-pre-styled-ui` のみに依存して
// 呼び出せることを保証するための明示再エクスポート（イシュー #685）。
pub use fandhe_frontend_headless_ui::state::{
    CheckableAction, DisclosureAction, OpenState, SingleSelectAction,
};

/// headless `menu` anatomy の `data-part` 一覧（`crates/headless-ui/src/menu.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &[
    "root",
    "trigger",
    "indicator",
    "positioner",
    "content",
    "arrow",
    "arrow-tip",
    "item",
    "item-group",
    "item-group-label",
    "separator",
];

/// この styled Menu の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("menu", SLOTS)
        .base("root", vec![decl("position", "relative")])
        .base(
            "trigger",
            vec![
                decl("cursor", "pointer"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.375rem"),
                decl(
                    "padding",
                    "var(--fandhe-menu-trigger-padding, var(--fandhe-space-2) var(--fandhe-space-3))",
                ),
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
                decl(
                    "padding",
                    "var(--fandhe-menu-content-padding, var(--fandhe-space-2))",
                ),
                decl("min-width", "var(--fandhe-reference-width, 10rem)"),
            ],
        )
        // イシュー #663: arrow はマーカー切り替え不要（モジュール rustdoc
        // 参照）。フォールバック値は SSR 既定 placement（bottom）で anchor
        // 中央上端に相当する。
        .base(
            "arrow",
            vec![
                decl("position", "absolute"),
                decl("left", "var(--fandhe-arrow-x, 50%)"),
                decl("top", "var(--fandhe-arrow-y, 0)"),
                decl("transform", "translate(-50%, -50%)"),
            ],
        )
        .base(
            "arrow-tip",
            vec![
                decl("width", "0.5rem"),
                decl("height", "0.5rem"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("border-left", "1px solid var(--fandhe-color-border)"),
                decl("border-top", "1px solid var(--fandhe-color-border)"),
                decl("transform", "rotate(45deg)"),
            ],
        )
        .base(
            "item",
            vec![
                decl(
                    "padding",
                    "var(--fandhe-menu-item-padding, var(--fandhe-space-2) var(--fandhe-space-3))",
                ),
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
            "separator",
            vec![
                decl("border", "0"),
                decl("border-top", "1px solid var(--fandhe-color-border-muted)"),
                decl("margin", "var(--fandhe-space-2) 0"),
            ],
        )
        // イシュー #551 受け入れ条件: `trigger`/`content` の開閉状態に応じた見た目の切り替え。
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
        // イシュー #643 受け入れ条件: virtual focus の highlight 表示
        // （`item` は実 DOM フォーカスを受けないため `:focus-visible` ではなく
        // `data-highlighted` で表現する、モジュール rustdoc 参照）。
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
        // （モジュール rustdoc 参照）。base の `positioner` 規則（absolute）
        // より詳細度が高く、CSS 記述順（states は最後尾）でも上書きする。
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
        // イシュー #729: `size` variant（root スコープの CSS custom property。
        // Md はフォールバック値と同一の現行外観を維持する）。`--fandhe-reference-width`/
        // `--fandhe-arrow-*`/`--fandhe-x`/`--fandhe-y`（wasm positioning 契約、
        // #663/#588）には手を触れない（モジュール rustdoc 参照）。
        .variant(
            Size::Sm,
            "root",
            vec![
                decl(
                    "--fandhe-menu-trigger-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl(
                    "--fandhe-menu-item-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl("--fandhe-menu-content-padding", "var(--fandhe-space-1)"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl(
                    "--fandhe-menu-trigger-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-3)",
                ),
                decl(
                    "--fandhe-menu-item-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-3)",
                ),
                decl("--fandhe-menu-content-padding", "var(--fandhe-space-2)"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl(
                    "--fandhe-menu-trigger-padding",
                    "var(--fandhe-space-3) var(--fandhe-space-4)",
                ),
                decl(
                    "--fandhe-menu-item-padding",
                    "var(--fandhe-space-3) var(--fandhe-space-4)",
                ),
                decl("--fandhe-menu-content-padding", "var(--fandhe-space-3)"),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled Menu が生成する静的 CSS 全量を返す（決定的。[`crate::dialog::stylesheet`]
/// と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::menu::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = menu::root(Size::Md, OpenState::Open, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="menu" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::menu::root(state, merged, children)
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
        assert!(a.contains(r#"[data-scope="menu"][data-part="content"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn positioner_is_absolutely_positioned_for_overlay() {
        // PR #575 Bugbot 指摘対応: positioner がオーバーレイ配置になっている
        // ことを固定する（通常のフローに残ったままにならない）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menu"][data-part="positioner"]"#));
        assert!(css.contains("position: absolute;"));
    }

    #[test]
    fn root_provides_containing_block_for_positioner() {
        // PR #575 Bugbot 指摘 1 対応: `trigger` と `positioner` は headless
        // `root` の下の兄弟要素であり、`trigger` は `positioner` の祖先には
        // なれない。そのため `position: relative` は共通祖先である `root`
        // に付与されていることを固定する（`trigger` への誤付与への回帰防止）。
        let css = stylesheet();
        assert!(
            css.contains("[data-scope=\"menu\"][data-part=\"root\"] {\n  position: relative;\n}\n")
        );
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(Size::Md, OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="menu""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    // --- イシュー #729: size variant ---

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Sm, Size::Md, Size::Lg] {
            let html = render(&root(
                size,
                OpenState::Closed,
                vec![("class", "attacker")],
                vec![],
            ));
            let expected_class = format!("fd-menu--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn default_variant_is_md_and_matches_pre_729_fallback() {
        // Md はフォールバック値と同一の現行外観を維持する（不変条件）。
        let css = stylesheet();
        assert!(css.contains(
            "padding: var(--fandhe-menu-trigger-padding, var(--fandhe-space-2) var(--fandhe-space-3));"
        ));
        assert!(css.contains(
            "padding: var(--fandhe-menu-item-padding, var(--fandhe-space-2) var(--fandhe-space-3));"
        ));
        assert!(css.contains("padding: var(--fandhe-menu-content-padding, var(--fandhe-space-2));"));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open_and_closed() {
        // イシュー #551 受け入れ条件: 「headless 層の data-state とスタイルの
        // 連動テスト（[data-state='open'] セレクタ等）」を固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menu"][data-part="trigger"][data-state="open"]"#));
        assert!(css.contains(r#"[data-scope="menu"][data-part="content"][data-state="closed"]"#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_menu_state_machine() {
        // イシュー #551 受け入れ条件: 「SSR / hydration 両経路の動作確認」を
        // headless `Menu`（イシュー #729 により本モジュールから再エクスポート
        // しないため、エスケープハッチ経由で直接 import する。モジュール
        // 冒頭の rustdoc「選択的 re-export」節参照）経由で固定する。
        use fandhe_frontend_headless_ui::menu::Menu;
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut m = Menu::default();
        assert_eq!(m.state(), OpenState::Closed);

        let ssr_html = render(&m.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut m, "open", ""));
        let hydrate_html = render(&render_for_hydration(&m));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));

        let restored = Menu::from_hydration_attrs(&m.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
    }

    #[test]
    fn item_highlighted_attr_is_styled_and_trigger_has_focus_visible_ring() {
        // イシュー #643 受け入れ条件: virtual focus の highlight 表示
        // （`data-highlighted`）とキーボード操作系属性（`:focus-visible`）が
        // recipe 経由で反映されることを固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menu"][data-part="item"][data-highlighted] {"#));
        assert!(css.contains(r#"[data-scope="menu"][data-part="trigger"]:focus-visible {"#));
        assert!(css.contains("outline: 2px solid var(--fandhe-color-accent);"));
    }

    #[test]
    fn content_min_width_consumes_fandhe_reference_width_css_var() {
        // イシュー #643 受け入れ条件: `--fandhe-reference-width`（wasm 層
        // `crates/wasm-full/src/position.rs::reposition_one` が positioner へ
        // 書き込む変数）を CSS 継承で消費する sameWidth 相当のスタイルが
        // 反映されることを固定する（SSR 静的表示では 10rem へフォールバック）。
        let css = stylesheet();
        assert!(css.contains("min-width: var(--fandhe-reference-width, 10rem);"));
    }

    #[test]
    fn positioner_switches_to_fixed_geometry_when_data_positioned_marker_is_present() {
        // イシュー #663 受け入れ条件: wasm 層が付与する `data-positioned`
        // マーカーが立っているときのみ、positioner が確定座標（viewport
        // 座標系の `position: fixed`）へ切り替わることをゴールデンで固定する。
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"menu\"][data-part=\"positioner\"][data-positioned] {\n  \
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
        // 回帰固定（`positioner_is_absolutely_positioned_for_overlay` と
        // 重複しない観点として `top: 100%;` も確認する）。
        let css = stylesheet();
        assert!(css.contains("position: absolute;"));
        assert!(css.contains("top: 100%;"));
    }

    #[test]
    fn arrow_consumes_fandhe_arrow_geometry_css_vars_and_arrow_tip_is_declared() {
        // イシュー #663 受け入れ条件: arrow はマーカー切り替え不要で
        // `--fandhe-arrow-x`/`--fandhe-arrow-y` を変数フォールバックのみで
        // 消費することを固定する（モジュール rustdoc 参照）。arrow-tip は
        // 座標変数を持たない（arrow の子として相対配置される装飾要素）ため、
        // base 規則が登録されていることのみ確認する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menu"][data-part="arrow"]"#));
        assert!(css.contains("left: var(--fandhe-arrow-x, 50%);"));
        assert!(css.contains("top: var(--fandhe-arrow-y, 0);"));
        assert!(css.contains(r#"[data-scope="menu"][data-part="arrow-tip"]"#));
    }

    #[test]
    fn position_geometry_var_references_never_lack_an_explicit_fallback() {
        // fail-closed 回帰（イシュー #663 §5 手順 6）: 本イシューが導入する
        // 位置ジオメトリ変数（`--fandhe-x`/`--fandhe-y`/`--fandhe-arrow-*`）
        // への参照はすべて明示フォールバック値を持つ（裸の `var(--x)` 禁止）。
        // 変数未定義（SSR・wasm 失敗時）でも表示が壊れないことを保証する
        // （テーマトークン系の `--fandhe-color-*` 等はフォールバック不要の
        // 常時定義済み変数のため対象外とする）。
        let css = stylesheet();
        for marker in ["var(--fandhe-x", "var(--fandhe-y", "var(--fandhe-arrow-"] {
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
