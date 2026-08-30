//! styled Tabs（headless ラッパー第 1 弾、イシュー #551、親 #520/#545。
//! `size`/`color-palette` variant 展開はイシュー #729、親 #708）。
//!
//! `fandhe_frontend_headless_ui::tabs`（イシュー #528）は Root / List /
//! Trigger / Content / Indicator（#601、opt-in）の 5 anatomy パーツを [`tabs`]
//! 単一の合成関数として組み立てる（パーツごとの自由関数を持たない、他 4
//! コンポーネントとの非対称点）。イシュー #729 以前は headless 側に root への
//! attrs 注入点自体が存在せず本モジュールは headless `tabs` をそのまま
//! 再エクスポートしていたが、`size`/`color-palette` variant クラスを root へ
//! 付与するために headless 側へ [`fandhe_frontend_headless_ui::tabs::tabs_with_root_attrs`]
//! （非破壊的な追加関数）が新設された（`crates/headless-ui/src/tabs.rs`
//! rustdoc 参照）。本モジュールはそれを呼ぶ styled [`tabs`] を新たに定義する。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、イシュー #729）
//!
//! headless 自由関数 `tabs` と名前が衝突するため、`pub use ...::*` ではなく
//! [`TabsProps`]/[`TabItem`]/[`ActivationMode`] のみを選択的に再エクスポート
//! する（[`crate::switch`]・[`crate::avatar`] と同型の判断）。headless 自由
//! 関数 `tabs`/`tabs_with_root_attrs`（未スタイル・variant クラス非付与）が
//! 必要な呼び出し側は
//! `fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::tabs` を
//! 直接 import すること。
//!
//! # data-state とスタイルの連動（イシュー #551 受け入れ条件）
//!
//! Tabs は `data-state` に `"open"`/`"closed"` ではなく `"active"`/`"inactive"`
//! 語彙を使う（`crates/headless-ui/src/tabs.rs` の `DATA_STATE_ACTIVE`/
//! `DATA_STATE_INACTIVE`）。選択中の `trigger` を強調する CSS を
//! [`crate::recipe::SlotRecipe::state`]（イシュー #643）経由で [`recipe`] へ
//! 登録する（`serialize_rule` を直接呼ぶ手書きセレクタ機構は廃止した）。
//!
//! # キーボード操作系スタイル（イシュー #643）
//!
//! `trigger` は roving tabindex（`.claude/rules` 外部だが headless 層 tabs の
//! キーボードナビゲーション実装）でフォーカス移動するボタン要素であり、
//! キーボード操作時のみのフォーカスリング（`:focus-visible`）を [`recipe`]
//! へ登録する。
//!
//! # `size`/`color-palette` variant（イシュー #729）
//!
//! `size`（[`Size`]）は root へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-tabs-trigger-padding`/`-content-padding` の root スコープ CSS
//! custom property（通常の CSS 継承により `trigger`/`content` へ伝わる。
//! `root` は両パーツを内包する祖先要素であるため、
//! [`crate::recipe::SlotRecipe`] へ子孫セレクタ機構を追加せずに実現できる）
//! 経由で寸法を切り替える。`color-palette`（[`ColorPalette`]、tabs のみが
//! 対応する第 2 軸）は既存の [`crate::recipe::palette_declarations`]
//! （chakra-ui virtual token 方式、#606）を root へ登録し、選択中 trigger の
//! 強調色（`border-bottom-color`）を `var(--fandhe-palette, ...)` 経由で
//! 切り替える。`base`/`state` 規則の `var()` にはいずれも Md サイズ・Accent
//! パレット相当のフォールバック値を書き、styled `root`/`tabs` を経由しない
//! headless 直接利用マークアップでも現行外観を維持する（fail-safe、
//! `crate::lib` rustdoc「複合部品の variant 統一方針」節参照）。

use crate::css::{decl, Declaration};
use crate::recipe::{
    palette_scale_declarations, ColorPalette, Size, SlotRecipe, StateCondition, VariantValue,
};

// headless 自由関数 `tabs`/`tabs_with_root_attrs` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。未スタイル・
// variant クラス非付与の実体が必要な呼び出し側は
// `fandhe_frontend_headless_ui::tabs` を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::tabs::{ActivationMode, TabItem, TabsProps};
// `TabsProps.orientation` フィールドの型（`data_attrs` モジュール由来のため
// 上記選択的再エクスポートでは到達しない）。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して `tabs()` を呼び出せることを
// 保証するための明示再エクスポート（イシュー #685）。
pub use fandhe_frontend_headless_ui::data_attrs::Orientation;

/// headless `tabs` anatomy の `data-part` 一覧（`crates/headless-ui/src/tabs.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &["root", "list", "trigger", "content", "indicator"];

/// `tabs`（`list` パーツ）/`tab_nav`（`root` パーツ）が共有する
/// 「タブ列コンテナ」の基底宣言（イシュー #996）。
///
/// 両モジュールとも `data-scope` が異なる（`tabs`/`tab-nav`）ため
/// [`crate::recipe::SlotRecipe`] のセレクタ文字列そのものは共有できないが、
/// 見た目を担う宣言列は本関数として一元化し、片方の変更がもう片方へ
/// サイレントに乖離しないようにする。`crates/pre-styled-ui/tests/tabs_css.rs`
/// の `TABS_GOLDEN_CSS` はこのリファクタ後もバイト単位で不変であることが
/// 絶対条件（`crates/pre-styled-ui/tests/tab_nav_css.rs` も同型の golden を
/// 別 scope で持つ）。
pub(crate) fn shared_tab_list_declarations() -> Vec<Declaration> {
    vec![
        decl("display", "flex"),
        decl("gap", "var(--fandhe-space-2)"),
        decl("border-bottom", "1px solid var(--fandhe-color-border)"),
    ]
}

/// `tabs`（`trigger` パーツ）/`tab_nav`（`link` パーツ）が共有する基底宣言
/// （イシュー #996）。`padding` は各部品固有の CSS custom property 名を含む
/// 値を呼び出し側がリテラルで渡す（`tabs` は `--fandhe-tabs-trigger-padding`、
/// `tab_nav` は `--fandhe-tab-nav-link-padding` を参照する別の値）。
pub(crate) fn shared_tab_item_declarations(padding: &'static str) -> Vec<Declaration> {
    vec![
        decl("padding", padding),
        decl("background", "transparent"),
        decl("color", "var(--fandhe-color-fg-muted)"),
        decl("border", "0"),
        decl("border-bottom", "2px solid transparent"),
        decl("cursor", "pointer"),
    ]
}

/// 選択中（`tabs`: `data-state="active"` / `tab_nav`: `aria-current="page"`）
/// の強調宣言（イシュー #996）。強調色は `color-palette` variant（`tabs` の
/// みが対応、`tab_nav` は Accent 固定フォールバックを直接参照）が登録する
/// `--fandhe-palette` 経由で切り替わる。
pub(crate) fn shared_tab_item_active_declarations() -> Vec<Declaration> {
    vec![
        decl("color", "var(--fandhe-color-fg)"),
        decl(
            "border-bottom-color",
            "var(--fandhe-palette, var(--fandhe-color-accent))",
        ),
    ]
}

/// この styled Tabs の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("tabs", SLOTS)
        .base("list", shared_tab_list_declarations())
        .base(
            "trigger",
            shared_tab_item_declarations(
                "var(--fandhe-tabs-trigger-padding, var(--fandhe-space-2) var(--fandhe-space-4))",
            ),
        )
        .base(
            "content",
            vec![
                decl(
                    "padding",
                    "var(--fandhe-tabs-content-padding, var(--fandhe-space-4) 0)",
                ),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        // イシュー #551 受け入れ条件: 選択中の `trigger` を強調する。
        // イシュー #729: 強調色は `color-palette` variant（root へ登録される
        // `--fandhe-palette`）経由で切り替わる。フォールバックは Accent 相当。
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "active"),
            shared_tab_item_active_declarations(),
        )
        .state(
            "content",
            StateCondition::AttrEq("data-state", "inactive"),
            vec![decl("display", "none")],
        )
        // イシュー #643: キーボード操作時のみのフォーカスリング。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        // イシュー #729: `size` variant（root スコープの CSS custom property。
        // Md はフォールバック値と同一の現行外観を維持する）。
        // イシュー #1681: Xs は Sm(1,3)→Md(2,4)→Lg(3,5) の等差進行を 1 段
        // 外挿した (0-5, 2)（`space-0`は未定義のため最小刻み `space-0-5`）。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl(
                    "--fandhe-tabs-trigger-padding",
                    "var(--fandhe-space-0-5) var(--fandhe-space-2)",
                ),
                decl("--fandhe-tabs-content-padding", "var(--fandhe-space-2) 0"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl(
                    "--fandhe-tabs-trigger-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-3)",
                ),
                decl("--fandhe-tabs-content-padding", "var(--fandhe-space-3) 0"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl(
                    "--fandhe-tabs-trigger-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-4)",
                ),
                decl("--fandhe-tabs-content-padding", "var(--fandhe-space-4) 0"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl(
                    "--fandhe-tabs-trigger-padding",
                    "var(--fandhe-space-3) var(--fandhe-space-5)",
                ),
                decl("--fandhe-tabs-content-padding", "var(--fandhe-space-5) 0"),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl(
                    "--fandhe-tabs-trigger-padding",
                    "var(--fandhe-space-4) var(--fandhe-space-6)",
                ),
                decl("--fandhe-tabs-content-padding", "var(--fandhe-space-6) 0"),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(ColorPalette::Accent);

    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
        ColorPalette::Neutral,
    ] {
        recipe = recipe.variant(palette, "root", palette_scale_declarations(palette));
    }
    recipe
}

/// この styled Tabs が生成する静的 CSS 全量を返す（決定的。[`crate::dialog::stylesheet`]
/// と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled Tabs を組み立てる。`size`/`color-palette` に応じたクラスを root へ
/// 付与する唯一のパーツ。`tabs` は headless 層に呼び出し側 attrs を受け取る
/// 引数を持たない（[`TabsProps`]/`items` のみ、モジュール冒頭 rustdoc「root
/// への attrs 注入点」節参照）ため、他の styled 部品の `root`
/// （[`crate::class_attr::drop_class_attr`] で呼び出し側 `class` を除去して
/// から合成）とは異なり、生成した variant クラスをそのまま root の `class`
/// として渡す（`drop_class_attr` は不要）。実体は
/// [`fandhe_frontend_headless_ui::tabs::tabs_with_root_attrs`] へ委譲する
/// （選択状態の決定則・roving tabindex・XSS 不変条件は headless 層と完全に
/// 同一）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::tabs::{self, ActivationMode, Orientation, TabItem, TabsProps};
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = tabs::tabs(
///     Size::Md,
///     ColorPalette::Accent,
///     &TabsProps {
///         id: "t",
///         selected: "a",
///         orientation: Orientation::Horizontal,
///         activation_mode: ActivationMode::Automatic,
///         loop_focus: true,
///         indicator: false,
///     },
///     vec![TabItem {
///         value: "a",
///         trigger: vec![],
///         content: vec![],
///         disabled: false,
///     }],
/// );
/// assert!(render(&node).contains(r#"data-scope="tabs" data-part="root""#));
/// ```
#[must_use]
pub fn tabs(
    size: Size,
    palette: ColorPalette,
    props: &TabsProps<'_>,
    items: Vec<TabItem<'_>>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    // `tabs` は headless 層に呼び出し側 attrs を受け取る引数を持たない
    // （headless `tabs_with_root_attrs` の rustdoc 参照）ため、ここで
    // `drop_class_attr` を通す対象（呼び出し側 attrs）は存在しない。生成した
    // variant クラスをそのまま root_attrs として渡す。
    let root_attrs: Vec<(&str, &str)> = vec![("class", class.as_str())];
    fandhe_frontend_headless_ui::tabs::tabs_with_root_attrs(props, root_attrs, items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn item<'a>(value: &'a str) -> TabItem<'a> {
        TabItem {
            value,
            trigger: vec![],
            content: vec![],
            disabled: false,
        }
    }

    fn default_props<'a>(id: &'a str, selected: &'a str) -> TabsProps<'a> {
        TabsProps {
            id,
            selected,
            orientation: Orientation::Horizontal,
            activation_mode: ActivationMode::Automatic,
            loop_focus: true,
            indicator: false,
        }
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="tabs"][data-part="trigger"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn styled_tabs_renders_with_headless_anatomy_attrs() {
        let props = default_props("t1", "one");
        let items = vec![item("one")];
        let html = render(&tabs(Size::Md, ColorPalette::Accent, &props, items));
        assert!(html.contains(r#"data-scope="tabs""#));
        assert!(html.contains(r#"data-part="list""#));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_active_and_inactive() {
        // イシュー #551 受け入れ条件: 「headless 層の data-state とスタイルの
        // 連動テスト（[data-state='open'] セレクタ等）」を固定する（Tabs は
        // open/closed ではなく active/inactive 語彙を使う）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tabs"][data-part="trigger"][data-state="active"]"#));
        assert!(css.contains(r#"[data-scope="tabs"][data-part="content"][data-state="inactive"]"#));
    }

    #[test]
    fn ssr_selected_tab_reflects_active_data_state() {
        // イシュー #551 受け入れ条件: 「SSR / hydration 両経路の動作確認」。
        // Tabs は状態機械を持たないため（headless 側スコープ外）、SSR 側の
        // 静的選択状態が data-state="active"/"inactive" として決定的に
        // 描画されることを固定する。
        let props = default_props("t1", "one");
        let items = vec![item("one"), item("two")];
        let html = render(&tabs(Size::Md, ColorPalette::Accent, &props, items));
        assert!(html.contains(r#"data-state="active""#));
        assert!(html.contains(r#"data-state="inactive""#));
    }

    #[test]
    fn trigger_declares_focus_visible_ring() {
        // イシュー #643 受け入れ条件: キーボード操作系属性（:focus-visible）
        // が recipe 経由で反映されることを固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tabs"][data-part="trigger"]:focus-visible {"#));
        assert!(css.contains("outline: 2px solid var(--fandhe-color-accent);"));
    }

    // --- イシュー #729: size/color-palette variant ---

    #[test]
    fn root_outputs_scope_and_part() {
        let props = default_props("t1", "one");
        let html = render(&tabs(
            Size::Md,
            ColorPalette::Accent,
            &props,
            vec![item("one")],
        ));
        assert!(html.contains(r#"data-scope="tabs""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Sm, Size::Md, Size::Lg] {
            let props = default_props("t1", "one");
            let html = render(&tabs(size, ColorPalette::Accent, &props, vec![item("one")]));
            let expected_class = format!("fd-tabs--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn color_palette_variant_appends_class_to_root() {
        for palette in [
            ColorPalette::Accent,
            ColorPalette::Info,
            ColorPalette::Success,
            ColorPalette::Warning,
            ColorPalette::Danger,
            ColorPalette::Neutral,
        ] {
            let props = default_props("t1", "one");
            let html = render(&tabs(Size::Md, palette, &props, vec![item("one")]));
            let expected_class = format!("fd-tabs--color-palette-{}", palette.value());
            assert!(html.contains(&expected_class), "html={html}");
        }
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let css = stylesheet();
        assert!(css.contains("--fandhe-tabs-trigger-padding"));
        // Md はフォールバック値と同一の現行外観を維持する（不変条件）。
        assert!(
            css.contains("padding: var(--fandhe-tabs-trigger-padding, var(--fandhe-space-2) var(--fandhe-space-4));")
        );
        assert!(
            css.contains("padding: var(--fandhe-tabs-content-padding, var(--fandhe-space-4) 0);")
        );
    }

    #[test]
    fn active_trigger_border_color_consumes_fandhe_palette_with_accent_fallback() {
        let css = stylesheet();
        assert!(
            css.contains("border-bottom-color: var(--fandhe-palette, var(--fandhe-color-accent));")
        );
    }
}
