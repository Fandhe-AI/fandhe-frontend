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
use fandhe_frontend_pre_styled_ui::recipe::{
    disabled_declarations, hover_bg_muted, hover_bg_solid, hover_surface_declarations,
    palette_declarations, palette_scale_declarations, transition_declarations, when,
    ColorPalette as StdColorPalette, MotionDuration, Size, SlotRecipe, StateCondition,
    VariantValue,
};
use fandhe_frontend_pre_styled_ui::theme::Theme;

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
        // イシュー #604: 「sm サイズ かつ red カラー」の組み合わせ条件のみに
        // 適用する compound variant（chakra-ui compoundVariants 相当）。
        .compound_variant(
            vec![when(Size::Sm), when(ColorPalette::Red)],
            "trigger",
            vec![decl("font-weight", "bold")],
        )
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
        "\n",
        "[data-scope=\"tabs\"][data-part=\"trigger\"].fd-tabs--size-sm.fd-tabs--colorpalette-red {\n",
        "  font-weight: bold;\n",
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
        indicator: false,
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

/// イシュー #606: 標準 `ColorPalette` 軸の `axis()`/`value()` を固定する
/// （`fd-<scope>--color-palette-<value>` 形式のクラス名生成の前提）。
#[test]
fn color_palette_axis_and_value_are_stable() {
    for (variant, value) in [
        (StdColorPalette::Accent, "accent"),
        (StdColorPalette::Info, "info"),
        (StdColorPalette::Success, "success"),
        (StdColorPalette::Warning, "warning"),
        (StdColorPalette::Danger, "danger"),
        (StdColorPalette::Neutral, "neutral"),
    ] {
        assert_eq!(variant.axis(), "color-palette");
        assert_eq!(variant.value(), value);
    }
    assert_eq!(StdColorPalette::default(), StdColorPalette::Accent);
}

/// イシュー #1678: `Size` が `Xs`〜`Xl` の 5 段・順序どおりに `value()` を
/// 返すことを固定する（既存 3 段の `Sm`/`Md`/`Lg` は #606 導入時の値から
/// 変更していない）。
#[test]
fn size_axis_has_five_ordered_values() {
    for (variant, value) in [
        (Size::Xs, "xs"),
        (Size::Sm, "sm"),
        (Size::Md, "md"),
        (Size::Lg, "lg"),
        (Size::Xl, "xl"),
    ] {
        assert_eq!(variant.axis(), "size");
        assert_eq!(variant.value(), value);
    }
}

/// イシュー #606: `palette_declarations` が各 palette 値に対して
/// `--fandhe-palette`/`--fandhe-palette-emphasized`/`--fandhe-palette-fg` の
/// 3 宣言を、対応するテーマ色トークン（`--fandhe-color-<name>*`）への
/// `var()` 参照として返すことを固定する。
#[test]
fn palette_declarations_reference_matching_theme_color_tokens() {
    for (variant, theme_name) in [
        (StdColorPalette::Accent, "accent"),
        (StdColorPalette::Info, "info"),
        (StdColorPalette::Success, "success"),
        (StdColorPalette::Warning, "warning"),
        (StdColorPalette::Danger, "danger"),
        (StdColorPalette::Neutral, "neutral"),
    ] {
        let decls = palette_declarations(variant);
        assert_eq!(decls.len(), 3);
        assert_eq!(decls[0].property(), "--fandhe-palette");
        assert_eq!(
            decls[0].value(),
            format!("var(--fandhe-color-{theme_name})")
        );
        assert_eq!(decls[1].property(), "--fandhe-palette-emphasized");
        assert_eq!(
            decls[1].value(),
            format!("var(--fandhe-color-{theme_name}-emphasized)")
        );
        assert_eq!(decls[2].property(), "--fandhe-palette-fg");
        assert_eq!(
            decls[2].value(),
            format!("var(--fandhe-color-{theme_name}-fg)")
        );
    }
}

/// イシュー #1678: `palette_scale_declarations` の先頭 3 件が
/// `palette_declarations` と完全一致する（同一順序・同一 property・同一
/// value）ことを固定する。Phase 1 の各部品 issue が golden 更新時に
/// `palette_declarations` から本関数へ移行する際、既存の 3 役割部分の
/// 挙動が変わらないことの保証になる。
#[test]
fn palette_scale_declarations_prefix_equals_palette_declarations() {
    for variant in [
        StdColorPalette::Accent,
        StdColorPalette::Info,
        StdColorPalette::Success,
        StdColorPalette::Warning,
        StdColorPalette::Danger,
        StdColorPalette::Neutral,
    ] {
        let base = palette_declarations(variant);
        let scale = palette_scale_declarations(variant);
        assert_eq!(scale.len(), 6);
        assert_eq!(&scale[..3], base.as_slice());
    }
}

/// イシュー #1678: `palette_scale_declarations` が返す 6 宣言すべてが、
/// `Theme::default().to_css()` に実在する `--fandhe-color-<name>-<role>`
/// トークンへの `var()` 参照であることを fail-closed に確認する
/// （未定義変数参照による「透明描画」バグ、#1704 が閉じた種類の再発防止）。
#[test]
fn palette_scale_declarations_reference_existing_theme_tokens() {
    let default_css = Theme::default().to_css();

    for variant in [
        StdColorPalette::Accent,
        StdColorPalette::Info,
        StdColorPalette::Success,
        StdColorPalette::Warning,
        StdColorPalette::Danger,
        StdColorPalette::Neutral,
    ] {
        let theme_name = variant.value();
        let decls = palette_scale_declarations(variant);
        assert_eq!(decls.len(), 6);

        let expected_properties = [
            "--fandhe-palette",
            "--fandhe-palette-emphasized",
            "--fandhe-palette-fg",
            "--fandhe-palette-subtle",
            "--fandhe-palette-muted",
            "--fandhe-palette-fg-subtle",
        ];
        let expected_roles = ["", "-emphasized", "-fg", "-subtle", "-muted", "-fg-subtle"];

        for (i, decl) in decls.iter().enumerate() {
            assert_eq!(decl.property(), expected_properties[i]);
            let expected_value = format!("var(--fandhe-color-{theme_name}{})", expected_roles[i]);
            assert_eq!(decl.value(), expected_value);

            // 参照先トークンが既定テーマに実在することを確認する。
            let token_decl = format!("--fandhe-color-{theme_name}{}:", expected_roles[i]);
            assert!(
                default_css.contains(&token_decl),
                "既定テーマに {token_decl:?} が存在しない（{variant:?} の {} 役割）",
                expected_properties[i]
            );
        }
    }
}

/// イシュー #604: compound variant の適用条件（2 個以上の軸が同時に
/// 一致した場合のみ）を golden fixture 以外の recipe でも固定する。
/// 単一 variant（`fd-tabs--size-sm` のみ）にはヒットせず、両条件を満たす
/// クラスの組み合わせにのみヒットすることを確認する。
#[test]
fn compound_variant_selector_requires_all_conditions() {
    let recipe = tabs_recipe();
    let css = recipe.css();
    assert!(css.contains(
        "[data-scope=\"tabs\"][data-part=\"trigger\"].fd-tabs--size-sm.fd-tabs--colorpalette-red {\n  font-weight: bold;\n}\n"
    ));
}

/// イシュー #604 fail-closed 検証（§3.3 の各条件）:
/// compound variant は不正入力を panic せず出力から除外する。
#[test]
fn compound_variant_fail_closed_cases_are_skipped_not_panicking() {
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
        .variant(Size::Sm, "root", vec![decl("padding", "2px")])
        .variant(Size::Md, "root", vec![decl("padding", "4px")])
        .default_variant(Size::Md)
        // 1. slot が slots 未宣言。
        .compound_variant(
            vec![when(Size::Sm)],
            "ghost-slot",
            vec![decl("color", "red")],
        )
        // 2. 条件の axis が識別子として不正。
        .compound_variant(vec![when(BadAxis)], "root", vec![decl("color", "green")])
        // 3. conditions が空（base と同義になる無意味な規則）。
        .compound_variant(vec![], "root", vec![decl("color", "purple")])
        // 4. conditions 内に同一 axis が重複する矛盾条件。
        .compound_variant(
            vec![when(Size::Sm), when(Size::Md)],
            "root",
            vec![decl("color", "orange")],
        )
        // 5. (axis, value) の組が variant()/default_variant() のいずれにも未登録。
        .compound_variant(vec![when(Size::Lg)], "root", vec![decl("color", "yellow")])
        // 構造破壊文字を含む宣言値は既存の serialize_rule 検証で個別にスキップされる。
        .compound_variant(
            vec![when(Size::Sm)],
            "root",
            vec![decl("color", "blue; } .evil {")],
        );

    let css = recipe.css();
    assert!(!css.contains("ghost-slot"));
    assert!(!css.contains("green"));
    assert!(!css.contains("purple"));
    assert!(!css.contains("orange"));
    assert!(!css.contains("yellow"));
    assert!(!css.contains("evil"));
    assert!(!css.contains("blue"));
}

/// イシュー #643 golden テスト: [`SlotRecipe::state`] が生成する 3 形式
/// （`Attr`/`AttrEq`/`FocusVisible`）のセレクタが byte 単位で固定されることと、
/// states が base/variants/compound variants より後に出力されることを固定する。
#[test]
fn state_css_output_matches_golden_fixture_and_is_ordered_last() {
    let recipe = SlotRecipe::new("widget", &["root", "item"])
        .base("root", vec![decl("display", "flex")])
        .variant(Size::Sm, "root", vec![decl("padding", "2px")])
        .state(
            "item",
            StateCondition::Attr("data-highlighted"),
            vec![decl("background", "blue")],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("opacity", "1")],
        )
        .state(
            "root",
            StateCondition::FocusVisible,
            vec![decl("outline", "2px solid red")],
        );

    let expected = concat!(
        "[data-scope=\"widget\"][data-part=\"root\"] {\n",
        "  display: flex;\n",
        "}\n",
        "\n",
        "[data-scope=\"widget\"][data-part=\"root\"].fd-widget--size-sm {\n",
        "  padding: 2px;\n",
        "}\n",
        "\n",
        "[data-scope=\"widget\"][data-part=\"item\"][data-highlighted] {\n",
        "  background: blue;\n",
        "}\n",
        "\n",
        "[data-scope=\"widget\"][data-part=\"root\"][data-state=\"open\"] {\n",
        "  opacity: 1;\n",
        "}\n",
        "\n",
        "[data-scope=\"widget\"][data-part=\"root\"]:focus-visible {\n",
        "  outline: 2px solid red;\n",
        "}\n",
    );
    assert_eq!(recipe.css(), expected);
}

/// イシュー #683 golden テスト: [`StateCondition::FocusWithin`] が
/// `:focus-within` セレクタを生成することを固定する（visually-hidden 化した
/// ネイティブ子孫要素へのフォーカスを、祖先要素の見た目へ反映する唯一の経路、
/// `crate::radio_group` の `item` が最初の消費者）。
#[test]
fn state_focus_within_generates_pseudo_class_selector() {
    let recipe = SlotRecipe::new("widget", &["item"]).state(
        "item",
        StateCondition::FocusWithin,
        vec![decl("outline", "2px solid blue")],
    );

    let expected = concat!(
        "[data-scope=\"widget\"][data-part=\"item\"]:focus-within {\n",
        "  outline: 2px solid blue;\n",
        "}\n",
    );
    assert_eq!(recipe.css(), expected);
}

/// イシュー #767 golden テスト: [`StateCondition::NthChildEven`] が
/// `:nth-child(even)` セレクタを生成することを固定する（[`crate::table`] の
/// striped 表現の唯一の消費者）。
#[test]
fn state_nth_child_even_generates_pseudo_class_selector() {
    let recipe = SlotRecipe::new("widget", &["row"]).state(
        "row",
        StateCondition::NthChildEven,
        vec![decl("background", "gray")],
    );

    let expected = concat!(
        "[data-scope=\"widget\"][data-part=\"row\"]:nth-child(even) {\n",
        "  background: gray;\n",
        "}\n",
    );
    assert_eq!(recipe.css(), expected);
}

/// イシュー #847 golden テスト: [`StateCondition::Hover`] が `:hover`
/// セレクタを生成することを固定する（[`crate::charts::tooltip`] のデータ点
/// 強調表示の唯一の消費者、SVG `<title>` によるネイティブ hover 表示と
/// 組み合わせて JS なしで「ホバーで詳細が分かる」体験を表現する）。
#[test]
fn state_hover_generates_pseudo_class_selector() {
    let recipe = SlotRecipe::new("chart", &["datum"]).state(
        "datum",
        StateCondition::Hover,
        vec![decl("opacity", "0.8")],
    );

    // イシュー #1425: `StateCondition::Hover` の出力先が `@media (hover:
    // hover)` ブロックへ変更され、セレクタにも `:not([data-disabled])` が
    // 付与された（`recipe.rs` の `SlotRecipe::css` rustdoc 参照）。
    let expected = concat!(
        "@media (hover: hover) {\n",
        "  [data-scope=\"chart\"][data-part=\"datum\"]:hover:not([data-disabled]) {\n",
        "    opacity: 0.8;\n",
        "  }\n",
        "}\n",
    );
    assert_eq!(recipe.css(), expected);
}

/// イシュー #643 fail-closed テスト: [`SlotRecipe::state`] は不正な `slot`・
/// 属性名・属性値を panic せず出力から除外する（既存 `base`/`variant`/
/// `compound_variant` と同じ方針）。
#[test]
fn state_fail_closed_cases_are_skipped_not_panicking() {
    let recipe = SlotRecipe::new("widget", &["root"])
        // 1. slot が slots 未宣言。
        .state(
            "ghost-slot",
            StateCondition::Attr("hidden"),
            vec![decl("color", "red")],
        )
        // 2. Attr の属性名が識別子として不正。
        .state(
            "root",
            StateCondition::Attr("Data-Highlighted"),
            vec![decl("color", "green")],
        )
        // 3. AttrEq の属性名が識別子として不正。
        .state(
            "root",
            StateCondition::AttrEq("Data-State", "open"),
            vec![decl("color", "purple")],
        )
        // 4. AttrEq の属性値が識別子として不正。
        .state(
            "root",
            StateCondition::AttrEq("data-state", "Open"),
            vec![decl("color", "orange")],
        )
        // 有効な規則も混在させ、無効規則の除外が他の規則へ波及しないことを確認する。
        .state(
            "root",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        );

    let css = recipe.css();
    assert!(!css.contains("ghost-slot"));
    assert!(!css.contains("green"));
    assert!(!css.contains("purple"));
    assert!(!css.contains("orange"));
    assert!(css.contains(r#"[data-scope="widget"][data-part="root"][hidden] {"#));
    assert!(css.contains("display: none;"));
}

/// disabled / hover / transition の共通ヘルパ（イシュー #1425）の golden
/// テスト。`recipe.rs` の doc コメント「disabled / hover / transition の
/// 共通ビジュアル言語」節が示す契約（`decl()` 経由・`&'static str` のみ）を
/// 固定する。
#[test]
fn disabled_declarations_matches_existing_de_facto_standard() {
    // 既存 40 箇所超（switch/checkbox_group 等）が個別に手書きしていた
    // `opacity: 0.5` + `cursor: not-allowed` と同一の値であることを固定する。
    let declarations = disabled_declarations();
    assert_eq!(declarations.len(), 2);
    assert_eq!(declarations[0].property(), "opacity");
    assert_eq!(declarations[0].value(), "0.5");
    assert_eq!(declarations[1].property(), "cursor");
    assert_eq!(declarations[1].value(), "not-allowed");
}

#[test]
fn hover_surface_declarations_references_indirection_custom_property() {
    // 実際の色を埋め込まず `var(--fandhe-hover-bg)` の間接参照のみを持つ
    // ことを固定する（variant 側が `--fandhe-hover-bg` を定義する契約）。
    let declarations = hover_surface_declarations();
    assert_eq!(declarations.len(), 1);
    assert_eq!(declarations[0].property(), "background");
    assert_eq!(declarations[0].value(), "var(--fandhe-hover-bg)");
}

#[test]
fn hover_bg_solid_and_muted_define_distinct_custom_property_values() {
    let solid = hover_bg_solid();
    assert_eq!(solid.property(), "--fandhe-hover-bg");
    assert_eq!(solid.value(), "var(--fandhe-palette-emphasized)");

    let muted = hover_bg_muted();
    assert_eq!(muted.property(), "--fandhe-hover-bg");
    assert_eq!(muted.value(), "var(--fandhe-color-bg-muted)");
}

#[test]
fn transition_declarations_produces_longhand_triplet_per_duration() {
    // shorthand `transition:` ではなく longhand 3 プロパティへ分解する契約
    // （`recipe.rs::transition_declarations` rustdoc 参照）。duration ごとに
    // `transition-duration` の値だけが切り替わることを固定する。
    let fast = transition_declarations("background, color", MotionDuration::Fast);
    assert_eq!(fast.len(), 3);
    assert_eq!(fast[0].property(), "transition-property");
    assert_eq!(fast[0].value(), "background, color");
    assert_eq!(fast[1].property(), "transition-duration");
    assert_eq!(fast[1].value(), "var(--fandhe-motion-duration-fast)");
    assert_eq!(fast[2].property(), "transition-timing-function");
    assert_eq!(fast[2].value(), "var(--fandhe-motion-easing-standard)");

    let normal = transition_declarations("background, color", MotionDuration::Normal);
    assert_eq!(normal[1].value(), "var(--fandhe-motion-duration-normal)");

    let slow = transition_declarations("background, color", MotionDuration::Slow);
    assert_eq!(slow[1].value(), "var(--fandhe-motion-duration-slow)");
}

#[test]
fn transition_declarations_serializes_into_recipe_css() {
    let recipe = SlotRecipe::new("widget", &["root"]).base(
        "root",
        transition_declarations("background, box-shadow", MotionDuration::Normal),
    );

    let expected = concat!(
        "[data-scope=\"widget\"][data-part=\"root\"] {\n",
        "  transition-property: background, box-shadow;\n",
        "  transition-duration: var(--fandhe-motion-duration-normal);\n",
        "  transition-timing-function: var(--fandhe-motion-easing-standard);\n",
        "}\n",
    );
    assert_eq!(recipe.css(), expected);
}

#[test]
fn hover_state_and_other_states_coexist_with_hover_block_emitted_once_at_end() {
    // イシュー #1425: `Hover` 以外の state（`Attr`）は通常どおり登録順で
    // 出力され、`Hover` は種類・登録順に関わらず `@media (hover: hover)`
    // ブロックへ集約されて css() 出力の最後尾に 1 つだけ現れることを固定
    // する（`SlotRecipe::css` rustdoc 出力順序節参照）。
    let recipe = SlotRecipe::new("widget", &["root"])
        .state("root", StateCondition::Hover, hover_surface_declarations())
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        );

    let css = recipe.css();
    assert_eq!(css.matches("@media (hover: hover)").count(), 1);

    let disabled_pos = css
        .find("[data-disabled]")
        .expect("disabled rule must exist");
    let media_pos = css
        .find("@media (hover: hover)")
        .expect("hover media block must exist");
    assert!(
        disabled_pos < media_pos,
        "Hover 以外の state は @media ブロックより前に出力される"
    );
    assert!(
        css.ends_with("}\n"),
        "@media ブロックが css() 出力の末尾であること"
    );
}
