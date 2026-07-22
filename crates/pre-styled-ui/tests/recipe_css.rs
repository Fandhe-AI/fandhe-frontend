//! `SlotRecipe`（イシュー #548）の統合テスト。
//!
//! - golden テスト: `tabs` scope 相当の recipe（headless-ui の Tabs anatomy と
//!   同一 scope/slots）から生成した静的 CSS 全文を固定する
//! - headless 接続テスト（受け入れ条件 2）: recipe が emit する
//!   `[data-scope][data-part]` セレクタが、`fandhe_frontend_headless_ui::tabs`
//!   が実際にレンダリングする属性と一致することを機械照合する
//! - fail-closed テスト: 不正識別子・構造破壊文字を含む値・未宣言 slot が
//!   panic せず出力から除外されることを固定する
//! - `variant_classes()`: 明示指定・defaultVariants 補完・axis 登録順連結を固定する

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::tabs::{tabs, TabItem, TabsProps};
use fandhe_frontend_headless_ui::Orientation;
use fandhe_frontend_pre_styled_ui::decl;
use fandhe_frontend_pre_styled_ui::recipe::{Size, SlotRecipe, VariantValue};

/// `colorPalette` 相当を独立の仕組みとしてではなく通常の variant 軸として
/// 表現できることを示すための variant enum（イシューの設計方針 §3.2 参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColorPalette {
    Blue,
    Red,
}

impl VariantValue for ColorPalette {
    fn axis(self) -> &'static str {
        "colorpalette"
    }

    fn value(self) -> &'static str {
        match self {
            ColorPalette::Blue => "blue",
            ColorPalette::Red => "red",
        }
    }
}

/// Tabs 相当の recipe を組み立てる（headless-ui の Tabs anatomy: scope
/// `"tabs"`、slots `root`/`list`/`trigger`/`content`）。golden テスト・
/// headless 接続テストの双方から共有する。
fn tabs_recipe() -> SlotRecipe {
    SlotRecipe::new("tabs", &["root", "list", "trigger", "content"])
        .base("root", vec![decl("display", "flex")])
        .base("list", vec![decl("display", "flex"), decl("gap", "4px")])
        .base(
            "trigger",
            vec![decl("border", "none"), decl("cursor", "pointer")],
        )
        .base("content", vec![decl("padding", "8px")])
        .variant(Size::Sm, "trigger", vec![decl("padding", "2px 6px")])
        .variant(Size::Md, "trigger", vec![decl("padding", "4px 10px")])
        .variant(Size::Lg, "trigger", vec![decl("padding", "6px 14px")])
        .variant(
            ColorPalette::Blue,
            "trigger",
            vec![decl("color", "var(--fd-color-blue-solid)")],
        )
        .variant(
            ColorPalette::Red,
            "trigger",
            vec![decl("color", "var(--fd-color-red-solid)")],
        )
        .default_variant(Size::Md)
        .default_variant(ColorPalette::Blue)
}

#[test]
fn css_output_matches_golden_fixture() {
    let recipe = tabs_recipe();
    let expected = concat!(
        "[data-scope=\"tabs\"][data-part=\"root\"] {\n",
        "  display: flex;\n",
        "}\n",
        "\n",
        "[data-scope=\"tabs\"][data-part=\"list\"] {\n",
        "  display: flex;\n",
        "  gap: 4px;\n",
        "}\n",
        "\n",
        "[data-scope=\"tabs\"][data-part=\"trigger\"] {\n",
        "  border: none;\n",
        "  cursor: pointer;\n",
        "}\n",
        "\n",
        "[data-scope=\"tabs\"][data-part=\"content\"] {\n",
        "  padding: 8px;\n",
        "}\n",
        "\n",
        "[data-scope=\"tabs\"][data-part=\"trigger\"].fd-tabs--size-sm {\n",
        "  padding: 2px 6px;\n",
        "}\n",
        "\n",
        "[data-scope=\"tabs\"][data-part=\"trigger\"].fd-tabs--size-md {\n",
        "  padding: 4px 10px;\n",
        "}\n",
        "\n",
        "[data-scope=\"tabs\"][data-part=\"trigger\"].fd-tabs--size-lg {\n",
        "  padding: 6px 14px;\n",
        "}\n",
        "\n",
        "[data-scope=\"tabs\"][data-part=\"trigger\"].fd-tabs--colorpalette-blue {\n",
        "  color: var(--fd-color-blue-solid);\n",
        "}\n",
        "\n",
        "[data-scope=\"tabs\"][data-part=\"trigger\"].fd-tabs--colorpalette-red {\n",
        "  color: var(--fd-color-red-solid);\n",
        "}\n",
    );
    assert_eq!(recipe.css(), expected);
}

#[test]
fn variant_class_and_variant_classes_are_stable() {
    let recipe = tabs_recipe();

    assert_eq!(recipe.variant_class(Size::Sm), "fd-tabs--size-sm");
    assert_eq!(
        recipe.variant_class(ColorPalette::Red),
        "fd-tabs--colorpalette-red"
    );

    // 明示指定のみ: axis 登録順（size → colorpalette、`variant()` の初回登録順）で連結。
    assert_eq!(
        recipe.variant_classes(&[("size", "lg"), ("colorpalette", "red")]),
        "fd-tabs--size-lg fd-tabs--colorpalette-red"
    );

    // 未指定 axis は defaultVariants で補完される。
    assert_eq!(
        recipe.variant_classes(&[]),
        "fd-tabs--size-md fd-tabs--colorpalette-blue"
    );

    // 一部のみ指定した場合、残りは defaultVariants で補完される。
    assert_eq!(
        recipe.variant_classes(&[("size", "sm")]),
        "fd-tabs--size-sm fd-tabs--colorpalette-blue"
    );
}

/// 受け入れ条件 2: headless 層の `data-scope`/`data-part` セレクタと接続する
/// CSS 生成テスト。recipe が base として emit する各セレクタの
/// `data-scope`/`data-part` 対が、`tabs()` が実際にレンダリングした HTML に
/// 実在することを機械照合する（セレクタと実マークアップの乖離を検知）。
#[test]
fn base_selectors_match_actual_headless_markup() {
    let recipe = tabs_recipe();

    let props = TabsProps {
        id: "demo",
        selected: "one",
        orientation: Orientation::Horizontal,
        activation_mode: fandhe_frontend_headless_ui::ActivationMode::Automatic,
        loop_focus: true,
    };
    let items = vec![
        TabItem {
            value: "one",
            trigger: vec![],
            content: vec![],
            disabled: false,
        },
        TabItem {
            value: "two",
            trigger: vec![],
            content: vec![],
            disabled: false,
        },
    ];
    let html = render(&tabs(&props, items));

    // `SlotRecipe::new` に渡した slots がそのまま headless 層が実際に
    // 描画する data-part 値であることを固定する（recipe 側が slot 名を
    // 誤記した場合にこのアサーションが破綻する）。
    for slot in ["root", "list", "trigger", "content"] {
        let needle = format!("data-scope=\"tabs\" data-part=\"{slot}\"");
        assert!(
            html.contains(&needle),
            "headless markup に {needle:?} が見つからない: {html}"
        );
    }

    // recipe が emit する base セレクタの scope/part 文字列が上記と同一形式
    // であることも確認する（セレクタ生成ロジック自体の固定）。
    let css = recipe.css();
    for slot in ["root", "list", "trigger", "content"] {
        let selector = format!("[data-scope=\"tabs\"][data-part=\"{slot}\"]");
        assert!(
            css.contains(&selector),
            "css に {selector:?} が見つからない"
        );
    }
}

#[test]
fn invalid_identifiers_and_structural_chars_are_skipped_not_panicking() {
    #[derive(Clone, Copy)]
    struct BadAxis;
    impl VariantValue for BadAxis {
        fn axis(self) -> &'static str {
            "Bad Axis"
        }
        fn value(self) -> &'static str {
            "1nvalid"
        }
    }

    let recipe = SlotRecipe::new("widget", &["root"])
        // 未宣言 slot への base 登録は出力から除外される。
        .base("ghost-slot", vec![decl("color", "red")])
        // 構造破壊文字を含む値は宣言単位でスキップされる。
        .base(
            "root",
            vec![decl("color", "red; } .evil {"), decl("background", "blue")],
        )
        // 不正な識別子を持つ variant 軸は出力から除外される。
        .variant(BadAxis, "root", vec![decl("color", "green")]);

    let css = recipe.css();
    assert!(!css.contains("ghost-slot"));
    assert!(!css.contains("evil"));
    assert!(!css.contains("green"));
    assert!(css.contains("background: blue;"));
    assert_eq!(recipe.variant_class(BadAxis), "");
}

/// #572 レビュー指摘（Medium）: `scope` は `slot`/`axis`/`value` と同様に
/// セレクタ・クラス名へそのまま埋め込まれるため、不正な `scope`（構造破壊
/// 文字・大文字・空文字等）を渡した recipe は `css()`/`variant_class()`/
/// `variant_classes()` のいずれも fail-closed で空文字列を返す（`value` 側の
/// denylist だけでは防げない `scope` 経由のセレクタ脱出・`</style>` 混入を防ぐ）。
#[test]
fn invalid_scope_is_rejected_fail_closed_across_all_outputs() {
    let recipe = SlotRecipe::new("widget\" ] {}</style><script>", &["root"])
        .base("root", vec![decl("color", "red")])
        .variant(Size::Sm, "root", vec![decl("padding", "2px")])
        .default_variant(Size::Sm);

    assert_eq!(recipe.css(), "");
    assert_eq!(recipe.variant_class(Size::Sm), "");
    assert_eq!(recipe.variant_classes(&[("size", "sm")]), "");
    assert_eq!(recipe.variant_classes(&[]), "");
}

/// #572 レビュー指摘（Low）: `base()` を `slots` の宣言順と異なる順序で
/// 呼び出しても、`css()` の base 出力は `slots` 宣言順に固定される
/// （`docs/api/pre-styled-recipe-api.md` §4 が凍結する契約）。
#[test]
fn base_output_order_follows_slots_declaration_not_registration_order() {
    let recipe = SlotRecipe::new("widget", &["root", "trigger"])
        // 登録順は trigger → root（slots 宣言順 root → trigger とは逆）。
        .base("trigger", vec![decl("cursor", "pointer")])
        .base("root", vec![decl("display", "flex")]);

    let css = recipe.css();
    let root_pos = css
        .find("[data-part=\"root\"]")
        .expect("root selector present");
    let trigger_pos = css
        .find("[data-part=\"trigger\"]")
        .expect("trigger selector present");
    assert!(
        root_pos < trigger_pos,
        "base 出力は slots 宣言順（root → trigger）であるべき: {css}"
    );
}
