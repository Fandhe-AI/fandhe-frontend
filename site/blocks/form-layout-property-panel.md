# form-layout-property-panel

`fandhe-frontend-pre-styled-ui` の `fieldset` / `field` / `number-input` /
`native-select` / `select` / `segment-group` / `color-swatch` / `separator`
部品を合成した、デザインツール風プロパティパネルの実例です。Blocks
セクションは新規部品を追加するものではなく、既存の Themes 部品を組み合わ
せた実例集であることに注意してください（主参照は対応表 ID R0225。出典の
固有名・ファイル名は記載しません）。

幅の狭い縦長パネルに「位置」「レイアウト」「文字」の 3 節を区切り線
（`separator`）で分けて縦に積んでいます。各節は 2 列グリッドの行にラベル
付き数値入力・選択・切替を詰めて配置し、狭い画面幅でもパネル幅を保ちます。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。`number_input` の増減ボタンと入力本体は静的
固定のため `readonly`（`aria-valuenow` は初期値のまま更新されないため、
表示値とのずれを防ぐ目的です）、`segment_group` の選択・フォント欄の
`select` は開閉/切替が JS ハイドレーション前提のためネイティブ `disabled`
で操作不能にし、現在値は固定表示にしています（単位・配置・太さのネイティブ
`<select>` は JS なしでも実際に動作するため `disabled` を付けず、可視ラベル
付きにしています）。文言はすべて独自に書いた架空のものであり、実企業名・
実クレデンシャル・PII を含みません。

後続イシュー #2914 で、テーマ選択＋開閉式の節・カード枠＋パンくずの
ヘッダー・極小サイズの狭幅パネルを追加する予定です。

## Rust コード

```rust
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
const DIRECTION_LABEL_ID: &str = "blocks-form-layout-property-panel-direction-label";
const TEXT_ALIGN_LABEL_ID: &str = "blocks-form-layout-property-panel-text-align-label";

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
/// `for`/コントロールの紐付けに使う。`min`/`max` は欄ごとの実際の許容範囲
/// を `aria-valuemin`/`aria-valuemax` へそのまま反映する（X/Y 座標・回転角
/// のように負値を取り得る欄と、幅・高さ・間隔のように非負のみの欄とで
/// 支援技術へ伝える範囲を区別する）。
fn number_field(
    id: &'static str,
    label_text: &'static str,
    value: &'static str,
    min: &'static str,
    max: &'static str,
) -> Node {
    let flags = NumberInputFlags {
        readonly: true,
        ..NumberInputFlags::default()
    };
    number_input::root(
        Size::Sm,
        false,
        false,
        true,
        vec![],
        vec![
            number_input::label(flags, Some(id), vec![], vec![text(label_text)]),
            number_input::control(
                flags,
                vec![],
                vec![
                    number_input::decrement_trigger(Some(id), true, vec![], vec![text("-")]),
                    number_input::input(id, Some(id), Some(value), min, max, flags, vec![]),
                    number_input::increment_trigger(Some(id), true, vec![], vec![text("+")]),
                ],
            ),
        ],
    )
}

/// 単位・その他の選択欄（ネイティブ `<select>`。JS なしでも実際に動作
/// するため静的固定は行わない、モジュール doc「静的表示」節）。
/// `field::root` + `field::label` + [`native_select::native_select`] の
/// 構成で可視ラベルを持たせる（`label_text` がそのままラベル文言になる。
/// `aria-label` だけだと隣のサイズ欄〔ラベルが上にある縦並び〕とグリッドの
/// 行内で縦位置がずれるため、`form_layout_two_column` の「国・地域」欄と
/// 同型の構成へ揃えた、イシュー #2913 コードレビューで是正）。
fn unit_select(
    id: &'static str,
    label_text: &'static str,
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
    field::root(
        &FieldRootProps::default(),
        &field_props,
        vec![],
        vec![
            field::label(&field_props, vec![], vec![text(label_text)]),
            native_select::native_select(
                &NativeSelectProps {
                    size: Size::Sm,
                    ..NativeSelectProps::default()
                },
                &field_props,
                vec![],
                option_nodes,
            ),
        ],
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
                    number_field(
                        "blocks-form-layout-property-panel-pos-x",
                        "X",
                        "120",
                        "-9999",
                        "9999",
                    ),
                    number_field(
                        "blocks-form-layout-property-panel-pos-y",
                        "Y",
                        "48",
                        "-9999",
                        "9999",
                    ),
                    number_field(
                        "blocks-form-layout-property-panel-pos-w",
                        "幅",
                        "320",
                        "0",
                        "9999",
                    ),
                    number_field(
                        "blocks-form-layout-property-panel-pos-h",
                        "高さ",
                        "180",
                        "0",
                        "9999",
                    ),
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
                                    "-360",
                                    "360",
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
    // ネイティブ操作を構造的に禁止するため item 系パーツ全体を disabled
    // 扱いにする（モジュール doc「静的表示」節）。`item`/`item_control`/
    // `item_text` も `item_hidden_input` と揃えて disabled: true を渡し、
    // 3 パーツすべてに `data-disabled` を反映させる（disabled_direction_props
    // を経由しないと data-disabled が `item_hidden_input` にしか付かず、
    // LAYOUT_CSS の中和セレクタが空振りするため）。
    let disabled_direction_props = SegmentGroupProps {
        disabled: true,
        ..direction_props
    };
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
                                vec![("id", DIRECTION_LABEL_ID)],
                                vec![text("方向")],
                            ),
                            segment_group::root_with_props(
                                Size::Sm,
                                &direction_props,
                                None,
                                Some(DIRECTION_LABEL_ID),
                                vec![],
                                vec![
                                    segment_group::indicator(
                                        Some((0, 2)),
                                        &disabled_direction_props,
                                        None,
                                        vec![],
                                    ),
                                    segment_group::item(
                                        true,
                                        &disabled_direction_props,
                                        "row",
                                        vec![],
                                        vec![
                                            segment_group::item_hidden_input(
                                                true,
                                                &disabled_direction_props,
                                                Some("blocks-form-layout-property-panel-direction"),
                                                "row",
                                                vec![],
                                            ),
                                            segment_group::item_control(
                                                true,
                                                &disabled_direction_props,
                                                vec![],
                                            ),
                                            segment_group::item_text(
                                                true,
                                                &disabled_direction_props,
                                                vec![],
                                                vec![text("横")],
                                            ),
                                        ],
                                    ),
                                    segment_group::item(
                                        false,
                                        &disabled_direction_props,
                                        "column",
                                        vec![],
                                        vec![
                                            segment_group::item_hidden_input(
                                                false,
                                                &disabled_direction_props,
                                                Some("blocks-form-layout-property-panel-direction"),
                                                "column",
                                                vec![],
                                            ),
                                            segment_group::item_control(
                                                false,
                                                &disabled_direction_props,
                                                vec![],
                                            ),
                                            segment_group::item_text(
                                                false,
                                                &disabled_direction_props,
                                                vec![],
                                                vec![text("縦")],
                                            ),
                                        ],
                                    ),
                                ],
                            ),
                        ],
                    ),
                    number_field(
                        "blocks-form-layout-property-panel-gap",
                        "間隔",
                        "16",
                        "0",
                        "9999",
                    ),
                    div(
                        vec![("class", "blocks-form-layout-property-panel-grid-span-2")],
                        vec![unit_select(
                            "blocks-form-layout-property-panel-align",
                            "配置",
                            &[
                                ("start", "先頭揃え"),
                                ("center", "中央揃え"),
                                ("end", "末尾揃え"),
                                ("stretch", "引き伸ばし"),
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
                        "0",
                        "9999",
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
                                vec![("id", TEXT_ALIGN_LABEL_ID)],
                                vec![text("揃え")],
                            ),
                            segment_group::root_with_props(
                                Size::Sm,
                                &align_props,
                                None,
                                Some(TEXT_ALIGN_LABEL_ID),
                                vec![],
                                vec![
                                    segment_group::indicator(
                                        Some((0, 3)),
                                        &SegmentGroupProps {
                                            disabled: true,
                                            ..SegmentGroupProps::default()
                                        },
                                        None,
                                        vec![],
                                    ),
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
                                            &InputProps {
                                                size: Size::Sm,
                                                ..InputProps::default()
                                            },
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
    // レイアウト節の direction 同様、item/item_control/item_text も
    // disabled_props で揃え、3 パーツすべてに `data-disabled` を反映させる
    // （item_hidden_input のみ disabled だと LAYOUT_CSS の中和セレクタが
    // 空振りする、モジュール doc「静的表示」節参照）。
    let disabled_props = SegmentGroupProps {
        disabled: true,
        ..SegmentGroupProps::default()
    };
    segment_group::item(
        checked,
        &disabled_props,
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
            segment_group::item_control(checked, &disabled_props, vec![]),
            segment_group::item_text(checked, &disabled_props, vec![], vec![text(label_text)]),
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
```

## 原案差分メモ

- 主参照（対応表 ID R0225）は位置／レイアウト／文字の 3 節を持つプロパティ
  パネルです。本 block は骨格（狭い縦長パネル・節の区切り・行の 2 列配置）
  と主要 3 節を実装し、テーマ選択・カード枠・極小サイズ構成は後続イシュー
  #2914 に回しています。
- 単位選択（px/%/rem）は X/Y/幅/高さの 4 欄で 1 個の `native_select` を
  共有する簡略化をしています（デザインツールの慣行に合わせた判断で、
  欄ごとに 4 個の selector を並べていません）。回転は単位が `deg` 固定
  のため selector を持たず、平文で添えています。
- `segment_group`（方向・揃え）はネイティブ `disabled` の radio で選択
  状態を固定表示にしています。フォント欄の `select` も閉じた状態で固定
  です（開閉には `fandhe-frontend-wasm-full` の JS 配線が必要で、docs
  サイトは JS ハイドレーションを行いません）。
- 色欄は `color_swatch`（装飾、`aria-hidden`）+ 16 進値のテキスト入力
  （`field`/`input`）の組み合わせで表現しています。
- 文言・配色・数値はすべて架空のものです。

関連情報: [Fieldset](../themes/fieldset.md) / [Field](../themes/field.md) /
[Number Input](../themes/number-input.md) /
[Native Select](../themes/native-select.md) / [Select](../themes/select.md) /
[Segment Group](../themes/segment-group.md) /
[Color Swatch](../themes/color-swatch.md) / [Separator](../themes/separator.md)
