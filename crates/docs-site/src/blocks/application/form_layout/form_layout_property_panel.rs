//! `form-layout-property-panel` block（イシュー #2913。親 #2912「Application
//! / Form Layout block 追加」配下、規模 L のため 2 分割された前半。主参照
//! R0225（位置／レイアウト／文字の 3 節を持つデザインツール風プロパティ
//! パネル）を対象とする骨格と主要 3 節を実装する。
//!
//! # 使用部品
//!
//! `fieldset` / `field` / `number-input` / `native-select` / `select` /
//! `segment-group` / `color-swatch` / `separator` の 8 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は追加しない。
//!
//! # 後続（#2914、対象外）
//!
//! テーマ選択＋開閉式の節（`collapsible`）・カード枠＋パンくずのヘッダー
//! （`card`/`breadcrumb`）・極小サイズの狭幅パネル・`tooltip`/`clipboard`/
//! `color-picker` の組み込みは #2914 で追加する。本 block は骨格（狭い
//! 縦長パネル・節の区切り・行の 2 列配置）と主要 3 節（位置／レイアウト／
//! 文字）のみを実装する。
//!
//! # 静的表示（無 JS）の扱い
//!
//! docs サイトは JS ハイドレーションを行わないため、実行時の値変更・
//! 開閉・切替は表現できない。
//!
//! - `number_input` の増減ボタンは `disabled: true` で出す（JS がないと
//!   動かないため、`pricing_seats_split`/`card_form_footer` と同じ判断）。
//!   input 本体はネイティブ text input のままで良い（`readonly` にはせず、
//!   値を編集できるように見せる。増減ボタンのみ操作不能にする）。
//! - `segment_group` は `item_hidden_input` へ `disabled: true` を渡し
//!   ネイティブ操作を構造的に禁止する。現在の選択は `data-state`/`checked`
//!   で表す（`card_form_footer` の radio card と同型の判断）。中和 CSS
//!   （`[data-disabled]` の opacity 復元）は [`LAYOUT_CSS`] が担う。
//! - `select`（フォント欄のみ）は `OpenState::Closed` で固定し、trigger は
//!   `disabled: true`。`positioner`/`content` は `hidden` 付きのまま出し、
//!   `aria-controls`/`aria-labelledby` の参照先を残す（`card_form_footer`
//!   の `closed_select` と同一の実装をそのまま流用する）。
//! - `native_select`（単位・配置・太さ）はネイティブ `<select>` であり
//!   JS なしでも実際に開閉・選択できるため、上記の静的固定は不要
//!   （`disabled`/`readonly` を付けない）。
//!
//! # 単位選択の粒度（意図的な簡略化）
//!
//! X/Y/幅/高さの 4 欄はデザインツールの慣行に合わせ単位（px/%/rem）を
//! 共有する 1 個の `native_select` に統一した（欄ごとに 4 個の単位
//! selector を並べるより実際の道具に近く、Demo の行数も抑えられる）。
//! 回転は単位が deg 固定のため selector を持たず、単位表示は
//! [`styled_text`] の平文で添える。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `fieldset::root` / `number_input::root` / `native_select::native_select` /
//! `select::root` / `segment_group::root` / `color_swatch::color_swatch` /
//! `field::root` / `input::input` はいずれも `drop_class_attr`
//! （`color_swatch` は `drop_class_and_style_attr`）により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、パネル固有の
//! フックは `data-blocks-form-layout-property-panel-*` 属性で渡す。素の
//! `div`（`drop_class_attr` を経由しない）には名前空間分離のため
//! `.blocks-form-layout-property-panel-*` クラスを使う。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。送信処理・送信先を一切持たない。
//!
//! # 参照について
//!
//! 主参照は対応表 ID R0225。文言・配色・アイコンは独自に書く（他 block と
//! 同じライセンス上の転記制限）。実在の企業名・PII は含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::color_swatch::{self, Color, ColorSwatchProps, Rgb};
use fandhe_frontend_pre_styled_ui::field::{self, FieldIds, FieldProps, FieldRootProps};
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps, FieldsetRootProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::number_input::{self, NumberInputFlags};
use fandhe_frontend_pre_styled_ui::segment_group::{self, SegmentGroupProps};
use fandhe_frontend_pre_styled_ui::select::{self, OpenState, SelectProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

const FONT_LABEL_ID: &str = "blocks-form-layout-property-panel-font-label";
const FONT_CONTENT_ID: &str = "blocks-form-layout-property-panel-font-content";
const COLOR_HEX_ID: &str = "blocks-form-layout-property-panel-color-hex";

/// 節見出し 1 節分の `FieldsetProps`（`legend`/`helper-text` 等の id 生成に
/// 使う base id のみ渡す。本 Demo は helper-text/error-text を出さない）。
fn fieldset_props(id: &'static str) -> FieldsetProps<'static> {
    FieldsetProps {
        id,
        disabled: false,
        invalid: false,
        has_helper_text: false,
    }
}

/// 数値入力 1 行分（`number_input`。増減ボタンは静的固定のため
/// `disabled: true`、モジュール doc「静的表示」節）。`id` はラベルの
/// `for`/コントロールの紐付けに使う。
fn number_field(id: &'static str, label_text: &'static str, value: &'static str) -> Node {
    let flags = NumberInputFlags::default();
    number_input::root(
        Size::Sm,
        false,
        false,
        false,
        vec![],
        vec![
            number_input::label(flags, Some(id), vec![], vec![text(label_text)]),
            number_input::control(
                flags,
                vec![],
                vec![
                    number_input::decrement_trigger(Some(id), true, vec![], vec![text("-")]),
                    number_input::input(id, Some(id), Some(value), "0", "9999", flags, vec![]),
                    number_input::increment_trigger(Some(id), true, vec![], vec![text("+")]),
                ],
            ),
        ],
    )
}

/// 単位・その他の選択欄（ネイティブ `<select>`。JS なしでも実際に動作
/// するため静的固定は行わない、モジュール doc「静的表示」節）。
/// `visible_label` を与えると可視ラベル付き（`field::root` 相当を持たず
/// [`native_select::native_select`] 単体で `aria-label` によりラベル付けする）。
fn unit_select(
    id: &'static str,
    aria_label: &'static str,
    options: &[(&'static str, &'static str)],
) -> Node {
    let field_props = FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let option_nodes: Vec<Node> = options
        .iter()
        .map(|(value, label)| {
            fandhe_frontend_core::el("option", vec![("value", value)], vec![text(*label)])
        })
        .collect();
    native_select::native_select(
        &NativeSelectProps::default(),
        &field_props,
        vec![("aria-label", aria_label)],
        option_nodes,
    )
}

/// 「位置」節。X/Y/幅/高さ（数値 4 件・単位共有）+ 回転（数値 + `deg` 平文）。
fn position_section() -> Node {
    let props = fieldset_props("blocks-form-layout-property-panel-position");
    fieldset::root(
        &FieldsetRootProps { size: Size::Sm },
        &props,
        vec![],
        vec![
            fieldset::legend(&props, vec![], vec![text("位置")]),
            div(
                vec![("class", "blocks-form-layout-property-panel-grid")],
                vec![
                    number_field("blocks-form-layout-property-panel-pos-x", "X", "120"),
                    number_field("blocks-form-layout-property-panel-pos-y", "Y", "48"),
                    number_field("blocks-form-layout-property-panel-pos-w", "幅", "320"),
                    number_field("blocks-form-layout-property-panel-pos-h", "高さ", "180"),
                    div(
                        vec![("class", "blocks-form-layout-property-panel-grid-span-2")],
                        vec![unit_select(
                            "blocks-form-layout-property-panel-pos-unit",
                            "位置・サイズの単位",
                            &[("px", "px"), ("percent", "%"), ("rem", "rem")],
                        )],
                    ),
                    div(
                        vec![("class", "blocks-form-layout-property-panel-grid-span-2")],
                        vec![div(
                            vec![("class", "blocks-form-layout-property-panel-rotation")],
                            vec![
                                number_field(
                                    "blocks-form-layout-property-panel-rotation",
                                    "回転",
                                    "0",
                                ),
                                styled_text::text(
                                    &TextProps {
                                        size: TextSize::Xs,
                                        variant: TextVariant::Muted,
                                        ..TextProps::default()
                                    },
                                    vec![],
                                    vec![text("deg")],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// 「レイアウト」節。方向（segment group、横を選択して固定）+ 間隔（数値）
/// + 配置（ネイティブ select）。
fn layout_section() -> Node {
    let props = fieldset_props("blocks-form-layout-property-panel-layout-fieldset");
    let direction_props = SegmentGroupProps::default();
    fieldset::root(
        &FieldsetRootProps { size: Size::Sm },
        &props,
        vec![],
        vec![
            fieldset::legend(&props, vec![], vec![text("レイアウト")]),
            div(
                vec![("class", "blocks-form-layout-property-panel-grid")],
                vec![
                    div(
                        vec![("class", "blocks-form-layout-property-panel-grid-span-2")],
                        vec![
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text("方向")],
                            ),
                            segment_group::root_with_props(
                                Size::Sm,
                                &direction_props,
                                None,
                                None,
                                vec![],
                                vec![
                                    segment_group::item(
                                        true,
                                        &direction_props,
                                        "row",
                                        vec![],
                                        vec![
                                            segment_group::item_hidden_input(
                                                true,
                                                &SegmentGroupProps {
                                                    disabled: true,
                                                    ..direction_props
                                                },
                                                Some("blocks-form-layout-property-panel-direction"),
                                                "row",
                                                vec![],
                                            ),
                                            segment_group::item_control(
                                                true,
                                                &direction_props,
                                                vec![],
                                            ),
                                            segment_group::item_text(
                                                true,
                                                &direction_props,
                                                vec![],
                                                vec![text("横")],
                                            ),
                                        ],
                                    ),
                                    segment_group::item(
                                        false,
                                        &direction_props,
                                        "column",
                                        vec![],
                                        vec![
                                            segment_group::item_hidden_input(
                                                false,
                                                &SegmentGroupProps {
                                                    disabled: true,
                                                    ..direction_props
                                                },
                                                Some("blocks-form-layout-property-panel-direction"),
                                                "column",
                                                vec![],
                                            ),
                                            segment_group::item_control(
                                                false,
                                                &direction_props,
                                                vec![],
                                            ),
                                            segment_group::item_text(
                                                false,
                                                &direction_props,
                                                vec![],
                                                vec![text("縦")],
                                            ),
                                        ],
                                    ),
                                ],
                            ),
                        ],
                    ),
                    number_field("blocks-form-layout-property-panel-gap", "間隔", "16"),
                    div(
                        vec![("class", "blocks-form-layout-property-panel-grid-span-2")],
                        vec![unit_select(
                            "blocks-form-layout-property-panel-align",
                            "配置",
                            &[
                                ("start", "先頭揃え"),
                                ("center", "中央揃え"),
                                ("end", "末尾揃え"),
                                ("stretch", "両端揃え"),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// 「文字」節。フォント（styled select・閉じた固定表示）+ サイズ（数値）+
/// 太さ（ネイティブ select）+ 揃え（segment group）+ 色（swatch + hex 表示）。
fn text_section() -> Node {
    let props = fieldset_props("blocks-form-layout-property-panel-text-fieldset");
    let align_props = SegmentGroupProps::default();
    let color_id_props = FieldProps {
        id: COLOR_HEX_ID,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    fieldset::root(
        &FieldsetRootProps { size: Size::Sm },
        &props,
        vec![],
        vec![
            fieldset::legend(&props, vec![], vec![text("文字")]),
            div(
                vec![("class", "blocks-form-layout-property-panel-grid")],
                vec![
                    div(
                        vec![("class", "blocks-form-layout-property-panel-grid-span-2")],
                        vec![closed_font_select()],
                    ),
                    number_field(
                        "blocks-form-layout-property-panel-font-size",
                        "サイズ",
                        "16",
                    ),
                    unit_select(
                        "blocks-form-layout-property-panel-font-weight",
                        "太さ",
                        &[("400", "標準"), ("700", "太字"), ("300", "細字")],
                    ),
                    div(
                        vec![("class", "blocks-form-layout-property-panel-grid-span-2")],
                        vec![
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text("揃え")],
                            ),
                            segment_group::root_with_props(
                                Size::Sm,
                                &align_props,
                                None,
                                None,
                                vec![],
                                vec![
                                    align_item(true, "left", "左"),
                                    align_item(false, "center", "中央"),
                                    align_item(false, "right", "右"),
                                ],
                            ),
                        ],
                    ),
                    div(
                        vec![("class", "blocks-form-layout-property-panel-grid-span-2")],
                        vec![div(
                            vec![("class", "blocks-form-layout-property-panel-color")],
                            vec![
                                color_swatch::color_swatch(
                                    &ColorSwatchProps {
                                        value: Color::from_rgb(Rgb::new(0x1a, 0x1a, 0x1a)),
                                        size: Size::Sm,
                                        ..ColorSwatchProps::default()
                                    },
                                    vec![("aria-hidden", "true")],
                                    vec![],
                                ),
                                field::root(
                                    &FieldRootProps::default(),
                                    &color_id_props,
                                    vec![("data-blocks-form-layout-property-panel-color", "")],
                                    vec![
                                        field::label(&color_id_props, vec![], vec![text("文字色")]),
                                        input::input(
                                            &InputProps::default(),
                                            &color_id_props,
                                            vec![
                                                ("type", "text"),
                                                ("value", "#1a1a1a"),
                                                ("inputmode", "text"),
                                            ],
                                        ),
                                    ],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// 「揃え」segment group の項目 1 件。
fn align_item(checked: bool, value: &'static str, label_text: &'static str) -> Node {
    let props = SegmentGroupProps::default();
    let disabled_props = SegmentGroupProps {
        disabled: true,
        ..props
    };
    segment_group::item(
        checked,
        &props,
        value,
        vec![],
        vec![
            segment_group::item_hidden_input(
                checked,
                &disabled_props,
                Some("blocks-form-layout-property-panel-text-align"),
                value,
                vec![],
            ),
            segment_group::item_control(checked, &props, vec![]),
            segment_group::item_text(checked, &props, vec![], vec![text(label_text)]),
        ],
    )
}

/// 閉じた状態で固定した styled select 1 件（フォント欄。`card_form_footer`
/// の `closed_select` と同一方針、モジュール doc「静的表示」節参照）。
fn closed_font_select() -> Node {
    let select_props = SelectProps {
        disabled: true,
        ..SelectProps::default()
    };
    let option_label = "Noto Sans JP";
    div(
        vec![("class", "blocks-form-layout-property-panel-select")],
        vec![
            select::label(
                &select_props,
                Some(FONT_LABEL_ID),
                vec![],
                vec![text("フォント")],
            ),
            select::root(
                Size::Sm,
                OpenState::Closed,
                &select_props,
                vec![],
                vec![
                    select::control(
                        OpenState::Closed,
                        &select_props,
                        vec![],
                        vec![select::trigger(
                            OpenState::Closed,
                            &select_props,
                            false,
                            Some(FONT_CONTENT_ID),
                            Some(FONT_LABEL_ID),
                            vec![],
                            vec![
                                select::value_text(
                                    false,
                                    &select_props,
                                    vec![],
                                    vec![text(option_label)],
                                ),
                                select::indicator(OpenState::Closed, &select_props, vec![], vec![]),
                            ],
                        )],
                    ),
                    select::positioner(
                        OpenState::Closed,
                        vec![],
                        vec![select::content(
                            OpenState::Closed,
                            Some(FONT_CONTENT_ID),
                            Some(FONT_LABEL_ID),
                            None,
                            vec![],
                            vec![select::item(
                                OpenState::Open,
                                &select_props,
                                false,
                                false,
                                "noto-sans-jp",
                                None,
                                vec![],
                                vec![select::item_text(
                                    OpenState::Open,
                                    &select_props,
                                    false,
                                    false,
                                    None,
                                    vec![],
                                    vec![text(option_label)],
                                )],
                            )],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// `form-layout-property-panel` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-form-layout-property-panel-layout")],
        vec![
            position_section(),
            separator::separator(&SeparatorProps::default(), vec![]),
            layout_section(),
            separator::separator(&SeparatorProps::default(), vec![]),
            text_section(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/form-layout-property-panel/",
    title: "form-layout-property-panel",
    category: BlockCategory::FormLayout,
    rust_source:
        "crates/docs-site/src/blocks/application/form_layout/form_layout_property_panel.rs",
    demo_class: "blocks-form-layout-property-panel",
    parts: &[
        Part {
            label: "Fieldset",
            path: "/themes/fieldset/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Number Input",
            path: "/themes/number-input/",
        },
        Part {
            label: "Native Select",
            path: "/themes/native-select/",
        },
        Part {
            label: "Select",
            path: "/themes/select/",
        },
        Part {
            label: "Segment Group",
            path: "/themes/segment-group/",
        },
        Part {
            label: "Color Swatch",
            path: "/themes/color-swatch/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `form_layout_property_panel` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。狭い縦長パネルを固定し、節内は
/// 2 列グリッドで小さい入力を詰めて配置する。
const LAYOUT_CSS: &str = "\
.blocks-form-layout-property-panel-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  inline-size: min(100%, 18rem);\n  padding: var(--fandhe-space-4);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  background: var(--fandhe-color-bg);\n}\n\
.blocks-form-layout-property-panel-grid {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-2);\n  margin-block-start: var(--fandhe-space-2);\n}\n\
.blocks-form-layout-property-panel-grid-span-2 {\n  grid-column: 1 / -1;\n}\n\
.blocks-form-layout-property-panel-rotation {\n  display: flex;\n  align-items: flex-end;\n  gap: var(--fandhe-space-1, 0.25rem);\n}\n\
.blocks-form-layout-property-panel-select {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1-5, 0.375rem);\n}\n\
.blocks-form-layout-property-panel-color {\n  display: flex;\n  align-items: flex-end;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-form-layout-property-panel-color] {\n  flex: 1;\n}\n\
[data-scope=\"segment-group\"][data-part=\"item-control\"][data-disabled],\n\
[data-scope=\"segment-group\"][data-part=\"item-text\"][data-disabled],\n\
[data-scope=\"segment-group\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-form-layout-property-panel-select [data-scope=\"select\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"number-input\"][data-part=\"increment-trigger\"][data-disabled],\n\
[data-scope=\"number-input\"][data-part=\"decrement-trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"fieldset\"",
            "data-scope=\"field\"",
            "data-scope=\"number-input\"",
            "data-scope=\"segment-group\"",
            "data-scope=\"select\"",
            "data-scope=\"color-swatch\"",
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(html.contains("data-part=\"select\""));
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("href="));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn section_legends_are_present() {
        let html = demo_html();
        assert!(html.contains("位置"));
        assert!(html.contains("レイアウト"));
        assert!(html.contains("文字"));
    }

    #[test]
    fn segment_group_items_have_exactly_one_checked_each_group() {
        let html = demo_html();
        // 方向（横/縦）+ 揃え（左/中央/右）の 2 グループ、計 2 件の checked。
        assert_eq!(html.matches(" checked").count(), 2);
        assert_eq!(html.matches(r#"type="radio""#).count(), 5);
    }

    #[test]
    fn layout_css_is_safe_and_uses_fandhe_tokens() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("--fandhe-"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: repeat(2, minmax(0, 1fr));"));
        assert!(LAYOUT_CSS.contains("inline-size: min(100%, 18rem);"));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-form-layout-property-panel-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-form-layout-property-panel-layout"
        );
    }

    #[test]
    fn no_dangling_aria_or_duplicate_ids() {
        let html = demo_html();
        assert!(html.contains(&format!("id=\"{}\"", super::FONT_LABEL_ID)));
        assert!(html.contains(&format!("id=\"{}\"", super::FONT_CONTENT_ID)));
        assert!(html.contains(&format!("aria-labelledby=\"{}\"", super::FONT_LABEL_ID)));
        assert!(html.contains(&format!("aria-controls=\"{}\"", super::FONT_CONTENT_ID)));
    }
}
