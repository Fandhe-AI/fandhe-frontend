# form-layout-property-panel

`fandhe-frontend-pre-styled-ui` の `fieldset` / `field` / `number-input` /
`native-select` / `select` / `color-picker` / `color-swatch` /
`segment-group` / `collapsible` / `tooltip` / `card` / `breadcrumb` /
`clipboard` / `separator` の 14 部品を合成した、デザインツール風プロパティ
パネルの実例です。Blocks セクションは新規部品を追加するものではなく、既存
の Themes 部品を組み合わせた実例集であることに注意してください（集約元は
対応表 ID R0225〜R0228 の 4 件。出典の固有名・ファイル名は記載しません）。

4 つの版（A〜D）を並記しています。

- **A（基本）**: 幅の狭い縦長パネルに「位置」「レイアウト」「文字」の 3 節
  を区切り線（`separator`）で分けて縦に積みます。各節は 2 列グリッドの行に
  ラベル付き数値入力・選択・切替を詰めて配置し、狭い画面幅でもパネル幅を
  保ちます。
- **B（テーマ選択 + 開閉式の節）**: 先頭にテーマ選択、続けて開閉式の節
  （`collapsible`）を 2 つ持ちます。「塗り」は開いた状態（`color_picker`
  の閉じた trigger + 不透明度）、「枠線」は閉じた状態（太さ + 色見本）で、
  開閉 2 状態を並べて示します。
- **C（カード枠 + パンくずのヘッダー）**: `card` で包み、ヘッダーに
  `breadcrumb` とタイトルを、本文にフォント設定を、フッターに CSS 宣言を
  コピーする `clipboard` を置きます。行間ラベルの横には常時表示の
  `tooltip` を添えます。
- **D（極小サイズの狭幅パネル）**: 入力に `Size::Xs` を使う、より狭い
  パネルです。位置節の X/Y/幅/高さと単位選択のみを持ちます。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。`number_input` の増減ボタンと入力本体は静的
固定のため `readonly`（`aria-valuenow` は初期値のまま更新されないため、
表示値とのずれを防ぐ目的です）、`segment_group`/`collapsible` の trigger は
開閉/切替が JS ハイドレーション前提のためネイティブ `disabled` で操作不能
にし、現在値は固定表示にしています（単位・配置・太さ・テーマ・フォントの
ネイティブ `<select>` は JS なしでも実際に動作するため `disabled` を付け
ず、可視ラベル付きにしています）。`tooltip`/`clipboard` の trigger は
開閉・コピー操作が JS ハイドレーション前提のため（無 JS の docs サイト
では押しても機能しません）ネイティブ `disabled` で操作不能にし、`tooltip`
は説明を常時表示、`clipboard` は idle 表示のまま固定しています。パンくず
の途中項目は遷移先を持ちません（末尾のみ現在ページ
を示します）。文言はすべて独自に書いた架空のものであり、実企業名・実
クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::collapsible;
use fandhe_frontend_pre_styled_ui::color_picker::{self, ColorPickerProps};
use fandhe_frontend_pre_styled_ui::color_swatch::{self, Color, ColorSwatchProps, Rgb};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::color_picker::ColorPicker;
use fandhe_frontend_pre_styled_ui::field::{self, FieldIds, FieldProps, FieldRootProps};
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps, FieldsetRootProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::number_input::{self, NumberInputFlags};
use fandhe_frontend_pre_styled_ui::segment_group::{self, SegmentGroupProps};
use fandhe_frontend_pre_styled_ui::select::{self, OpenState, SelectProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::tooltip;
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

const FONT_LABEL_ID: &str = "blocks-form-layout-property-panel-font-label";
const FONT_CONTENT_ID: &str = "blocks-form-layout-property-panel-font-content";
const COLOR_HEX_ID: &str = "blocks-form-layout-property-panel-color-hex";
const DIRECTION_LABEL_ID: &str = "blocks-form-layout-property-panel-direction-label";
const TEXT_ALIGN_LABEL_ID: &str = "blocks-form-layout-property-panel-text-align-label";

// --- B（R0226: テーマ選択 + 開閉式の節 2 つ）で使う id 群。他版と衝突
// しない `-b-` 接頭辞で分離する（`blocks_contract` の id 重複検査対応）。
const B_FILL_CONTENT_ID: &str = "blocks-form-layout-property-panel-b-fill-content";
const B_BORDER_CONTENT_ID: &str = "blocks-form-layout-property-panel-b-border-content";
const B_COLOR_LABEL_ID: &str = "blocks-form-layout-property-panel-b-color-label";

// --- C（R0227: カード枠 + パンくずのヘッダー）で使う id 群。
const C_TOOLTIP_CONTENT_ID: &str = "blocks-form-layout-property-panel-c-tooltip-content";
const C_CLIP_LABEL_ID: &str = "blocks-form-layout-property-panel-c-clip-label";
const C_CLIP_INPUT_ID: &str = "blocks-form-layout-property-panel-c-clip-input";

/// コピー対象の CSS 宣言（C 版の `card::footer` clipboard）。無害な
/// フォント指定 1 行のみで、実在のトークン・秘密情報を含まない。
const C_FONT_CSS_DECLARATION: &str = "font: 700 20px/1.4 \"Noto Sans JP\", sans-serif;";

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
/// 支援技術へ伝える範囲を区別する）。`size` は D 版（狭幅パネル）が
/// `Size::Xs` を渡せるよう #2914 で引数化した（A〜C は従来どおり
/// `Size::Sm`、イシュー #2914）。
fn number_field(
    id: &'static str,
    label_text: &'static str,
    value: &'static str,
    min: &'static str,
    max: &'static str,
    size: Size,
) -> Node {
    let flags = NumberInputFlags {
        readonly: true,
        ..NumberInputFlags::default()
    };
    number_input::root(
        size,
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
///
/// `note`（`Some` のとき `field::helper_text` を添える）は、選択操作自体が
/// 実際に効く（`native_select` は無 JS でも開閉・選択できる）が選択結果を
/// 反映する対象がこの静的デモには無い欄向け。B 版のテーマ選択がこれに
/// 該当し、選んでも配下のパネル配色は変わらない（無 JS デモのため）ことを
/// 明示して誤解を防ぐ（イシュー #2914 コードレビュー是正、P2）。他の呼び
/// 出し（単位・配置・フォント）は選択自体が示す意味が完結するため `None`。
fn unit_select(
    id: &'static str,
    label_text: &'static str,
    options: &[(&'static str, &'static str)],
    size: Size,
    note: Option<&'static str>,
) -> Node {
    let field_props = FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: note.is_some(),
    };
    let option_nodes: Vec<Node> = options
        .iter()
        .map(|(value, label)| {
            fandhe_frontend_core::el("option", vec![("value", value)], vec![text(*label)])
        })
        .collect();
    let mut children = vec![
        field::label(&field_props, vec![], vec![text(label_text)]),
        native_select::native_select(
            &NativeSelectProps {
                size,
                ..NativeSelectProps::default()
            },
            &field_props,
            vec![],
            option_nodes,
        ),
    ];
    if let Some(note_text) = note {
        children.push(field::helper_text(
            &field_props,
            vec![],
            vec![text(note_text)],
        ));
    }
    field::root(&FieldRootProps::default(), &field_props, vec![], children)
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
                        Size::Sm,
                    ),
                    number_field(
                        "blocks-form-layout-property-panel-pos-y",
                        "Y",
                        "48",
                        "-9999",
                        "9999",
                        Size::Sm,
                    ),
                    number_field(
                        "blocks-form-layout-property-panel-pos-w",
                        "幅",
                        "320",
                        "0",
                        "9999",
                        Size::Sm,
                    ),
                    number_field(
                        "blocks-form-layout-property-panel-pos-h",
                        "高さ",
                        "180",
                        "0",
                        "9999",
                        Size::Sm,
                    ),
                    div(
                        vec![("class", "blocks-form-layout-property-panel-grid-span-2")],
                        vec![unit_select(
                            "blocks-form-layout-property-panel-pos-unit",
                            "位置・サイズの単位",
                            &[("px", "px"), ("percent", "%"), ("rem", "rem")],
                            Size::Sm,
                            None,
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
                                    Size::Sm,
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
                        Size::Sm,
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
                            Size::Sm,
                            None,
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
    // 隣接する color_swatch は固定値 #1a1a1a のまま更新されない静的 Demo
    // のため、input を編集可能に見せると値と色見本がずれる（イシュー #3355
    // コードレビューで是正、モジュール doc「静的表示」節と同じ判断）。
    let color_id_props = FieldProps {
        id: COLOR_HEX_ID,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: true,
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
                        Size::Sm,
                    ),
                    unit_select(
                        "blocks-form-layout-property-panel-font-weight",
                        "太さ",
                        &[("400", "標準"), ("700", "太字"), ("300", "細字")],
                        Size::Sm,
                        None,
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

/// 版見出し（A〜D の先頭に置く短いキャプション）。
fn version_caption(label_text: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label_text)],
    )
}

/// A（R0225 基準形）: 骨格（狭い縦長パネル・節の区切り・行の 2 列配置）と
/// 位置／レイアウト／文字の主要 3 節。#2913 で実装済みの中身は変えず、
/// `.blocks-form-layout-property-panel-panel` へ外枠クラスを移した
/// （#2914、モジュール doc「集約元と版の対応」節参照）。
fn panel_basic() -> Node {
    div(
        vec![("class", "blocks-form-layout-property-panel-panel")],
        vec![
            version_caption("基本（位置・レイアウト・文字）"),
            position_section(),
            separator::separator(&SeparatorProps::default(), vec![]),
            layout_section(),
            separator::separator(&SeparatorProps::default(), vec![]),
            text_section(),
        ],
    )
}

/// B（R0226）: テーマ選択（先頭）+ 開閉式の節 2 つ（「塗り」= open・
/// 「枠線」= closed）。`collapsible::trigger`/`content` は
/// `app_shell_stacked_overlap::hamburger`/`mobile_panel` と同じ判断で
/// `disabled: true` 固定（無 JS の docs サイトでは押しても何も起きない
/// ため、モジュール doc「静的表示（無 JS）の扱い」節参照）。
fn panel_theme_collapsible() -> Node {
    let fill_color = ColorPicker::from_color(Color::from_rgb(Rgb::new(0x22, 0x63, 0xeb)));
    let readonly_color_props = ColorPickerProps {
        readonly: true,
        ..ColorPickerProps::default()
    };
    // `trigger` のみ `disabled: true` も重ねる（`readonly` は
    // ネイティブ `disabled` 属性を出さず操作可能に見えてしまうため、
    // `number_field` の増減ボタンと同じ判断。イシュー #2914 コード
    // レビュー是正、P1）。
    let disabled_trigger_color_props = ColorPickerProps {
        readonly: true,
        disabled: true,
        ..ColorPickerProps::default()
    };
    let fill_section = collapsible::content(
        collapsible::OpenState::Open,
        true,
        Some(B_FILL_CONTENT_ID),
        vec![("data-blocks-form-layout-property-panel-b-section", "")],
        vec![
            color_picker::root(
                &fill_color,
                &readonly_color_props,
                vec![],
                vec![
                    color_picker::label(
                        &readonly_color_props,
                        vec![("id", B_COLOR_LABEL_ID)],
                        vec![text("塗り色")],
                    ),
                    color_picker::control(
                        OpenState::Closed,
                        &readonly_color_props,
                        vec![],
                        vec![
                            color_picker::channel_input(
                                fill_color.hex().as_str(),
                                &readonly_color_props,
                                vec![("aria-labelledby", B_COLOR_LABEL_ID)],
                            ),
                            color_picker::trigger(
                                &fill_color,
                                &disabled_trigger_color_props,
                                None,
                                vec![("aria-label", "塗り色を選ぶ")],
                                vec![],
                            ),
                        ],
                    ),
                ],
            ),
            number_field(
                "blocks-form-layout-property-panel-b-opacity",
                "不透明度",
                "100",
                "0",
                "100",
                Size::Sm,
            ),
        ],
    );
    let border_section = collapsible::content(
        collapsible::OpenState::Closed,
        true,
        Some(B_BORDER_CONTENT_ID),
        vec![("data-blocks-form-layout-property-panel-b-section", "")],
        vec![
            number_field(
                "blocks-form-layout-property-panel-b-border-width",
                "太さ",
                "1",
                "0",
                "9999",
                Size::Sm,
            ),
            color_swatch::color_swatch(
                &ColorSwatchProps {
                    value: Color::from_rgb(Rgb::new(0xd1, 0xd5, 0xdb)),
                    size: Size::Sm,
                    ..ColorSwatchProps::default()
                },
                vec![("aria-label", "枠線の色")],
                vec![],
            ),
        ],
    );
    div(
        vec![("class", "blocks-form-layout-property-panel-panel")],
        vec![
            version_caption("テーマ選択 + 開閉式の節（R0226）"),
            unit_select(
                "blocks-form-layout-property-panel-b-theme",
                "テーマ",
                &[
                    ("standard", "標準"),
                    ("dark", "ダーク"),
                    ("high-contrast", "ハイコントラスト"),
                ],
                Size::Sm,
                Some("選択操作自体は無 JS でも行えますが、この静的デモでは配下パネルの配色は変わりません。"),
            ),
            separator::separator(&SeparatorProps::default(), vec![]),
            collapsible::root(
                collapsible::OpenState::Open,
                true,
                vec![],
                vec![
                    collapsible::trigger(
                        collapsible::OpenState::Open,
                        true,
                        Some(B_FILL_CONTENT_ID),
                        vec![],
                        vec![
                            text("塗り"),
                            collapsible::indicator(
                                collapsible::OpenState::Open,
                                true,
                                vec![],
                                vec![],
                            ),
                        ],
                    ),
                    fill_section,
                ],
            ),
            collapsible::root(
                collapsible::OpenState::Closed,
                true,
                vec![],
                vec![
                    collapsible::trigger(
                        collapsible::OpenState::Closed,
                        true,
                        Some(B_BORDER_CONTENT_ID),
                        vec![],
                        vec![
                            text("枠線"),
                            collapsible::indicator(
                                collapsible::OpenState::Closed,
                                true,
                                vec![],
                                vec![],
                            ),
                        ],
                    ),
                    border_section,
                ],
            ),
        ],
    )
}

/// C（R0227）: カード枠 + パンくずのヘッダーで包んだフォント設定。
fn panel_card_font() -> Node {
    let breadcrumb_nav = breadcrumb::root(
        Size::Sm,
        BreadcrumbVariant::Plain,
        Some("プロパティの階層"),
        vec![],
        vec![breadcrumb::list(
            vec![],
            vec![
                breadcrumb::item(vec![], vec![text("レイヤー")]),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(vec![], vec![text("テキスト")]),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(vec![], vec![text("フォント")])],
                ),
            ],
        )],
    );
    // `number_field` ヘルパーは呼び出し側で属性を追加できないため、行間欄
    // だけは `number_input` パーツを直接組み立てる。ヘルパーへ describedby
    // 引数を足して全 15 呼び出し箇所を書き換えるより、この 1 箇所だけ
    // 展開するほうが差分が小さい。tooltip の説明文（`C_TOOLTIP_CONTENT_ID`）
    // を input へ `aria-describedby` で関連付ける（disabled な
    // `tooltip::trigger` にしか説明が結び付いておらず、行間欄へフォーカス
    // 移動した支援技術利用者に説明が伝わらなかったため、イシュー #2914
    // コードレビュー是正、P2）。
    let line_height_id = "blocks-form-layout-property-panel-c-line-height";
    let line_height_flags = NumberInputFlags {
        readonly: true,
        ..NumberInputFlags::default()
    };
    let line_height_field = number_input::root(
        Size::Sm,
        false,
        false,
        true,
        vec![],
        vec![
            number_input::label(
                line_height_flags,
                Some(line_height_id),
                vec![],
                vec![text("行間")],
            ),
            number_input::control(
                line_height_flags,
                vec![],
                vec![
                    number_input::decrement_trigger(
                        Some(line_height_id),
                        true,
                        vec![],
                        vec![text("-")],
                    ),
                    number_input::input(
                        line_height_id,
                        Some(line_height_id),
                        Some("1.4"),
                        "0",
                        "9999",
                        line_height_flags,
                        vec![("aria-describedby", C_TOOLTIP_CONTENT_ID)],
                    ),
                    number_input::increment_trigger(
                        Some(line_height_id),
                        true,
                        vec![],
                        vec![text("+")],
                    ),
                ],
            ),
        ],
    );
    let line_height_row = div(
        vec![("class", "blocks-form-layout-property-panel-c-tooltip-row")],
        vec![
            line_height_field,
            tooltip::root(
                OpenState::Open,
                vec![],
                vec![
                    // trigger は `disabled: true` 固定 + 常時 open で出す。無 JS の
                    // docs サイトでは押しても開閉できず、closed のまま出すと
                    // 操作可能なボタンに見えて実際には機能しない見た目になるため
                    // （イシュー #2914 コードレビュー是正、P2）。説明を常時表示
                    // することで代替する（レビュー指摘の代替案）。
                    tooltip::trigger(
                        OpenState::Open,
                        true,
                        Some(C_TOOLTIP_CONTENT_ID),
                        vec![("aria-label", "行間についてのヘルプ")],
                        vec![text("?")],
                    ),
                    tooltip::positioner(
                        OpenState::Open,
                        vec![],
                        vec![tooltip::content(
                            OpenState::Open,
                            Some(C_TOOLTIP_CONTENT_ID),
                            vec![],
                            vec![text("フォントサイズに対する行の高さの倍率です。")],
                        )],
                    ),
                ],
            ),
        ],
    );
    div(
        vec![("class", "blocks-form-layout-property-panel-panel")],
        vec![
            version_caption("カード枠 + パンくず（R0227）"),
            card::root(
                CardProps {
                    size: Size::Sm,
                    ..CardProps::default()
                },
                vec![],
                vec![
                    card::header(
                        vec![],
                        vec![
                            breadcrumb_nav,
                            card::title(vec![], vec![text("フォント設定")]),
                        ],
                    ),
                    card::body(
                        vec![],
                        vec![
                            // 選択肢を [`C_FONT_CSS_DECLARATION`]（Noto Sans JP 固定）と
                            // 一致する 1 件のみにする。無 JS の docs サイトでは選択を
                            // 変えても footer の clipboard コピー内容は更新されないため、
                            // 選べる値を 1 つに固定して表示/コピー内容の不一致を構造的に
                            // 防ぐ（イシュー #2914 コードレビュー是正、P1）。
                            unit_select(
                                "blocks-form-layout-property-panel-c-font",
                                "フォント",
                                &[("noto-sans-jp", "Noto Sans JP")],
                                Size::Sm,
                                None,
                            ),
                            number_field(
                                "blocks-form-layout-property-panel-c-size",
                                "サイズ",
                                "20",
                                "0",
                                "9999",
                                Size::Sm,
                            ),
                            line_height_row,
                        ],
                    ),
                    card::footer(
                        vec![],
                        vec![clipboard::root(
                            C_FONT_CSS_DECLARATION,
                            false,
                            vec![],
                            vec![
                                visually_hidden::root(
                                    vec![],
                                    vec![clipboard::label(
                                        false,
                                        Some(C_CLIP_INPUT_ID),
                                        vec![("id", C_CLIP_LABEL_ID)],
                                        vec![text("CSS 宣言")],
                                    )],
                                ),
                                clipboard::control(
                                    false,
                                    vec![],
                                    vec![
                                        // `value_text`（span）ではなく `input` を使う。前者は
                                        // `id` を持てても `label::for` の関連付け先になれない
                                        // （`for` はフォームコントロールのみを指す）ため、
                                        // 隣接する `label` の `for` が宙に浮いていた
                                        // （イシュー #2914 コードレビュー是正、Bugbot 指摘）。
                                        // A 版（`hero_install_command`）と同じ
                                        // `clipboard::input` パターンへ揃える。
                                        clipboard::input(
                                            C_FONT_CSS_DECLARATION,
                                            false,
                                            vec![("id", C_CLIP_INPUT_ID)],
                                        ),
                                        // headless `clipboard::trigger` は disabled 引数を
                                        // 持たないため、ネイティブ `disabled` 属性を
                                        // `attrs` 経由で直接付与する（`gallery_carousel`
                                        // の `indicator` と同じ手段。減光 CSS は
                                        // [`LAYOUT_CSS`] のブロック固有セレクタが担う、
                                        // イシュー #2914 コードレビュー是正、P1）。
                                        clipboard::trigger(
                                            false,
                                            vec![
                                                ("disabled", ""),
                                                ("data-blocks-form-layout-property-panel-c-clipboard-trigger", ""),
                                            ],
                                            vec![
                                                clipboard::indicator(
                                                    false,
                                                    false,
                                                    vec![],
                                                    vec![text("Copy")],
                                                ),
                                                clipboard::indicator(
                                                    true,
                                                    false,
                                                    vec![],
                                                    vec![text("Copied!")],
                                                ),
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

/// D（R0228）: 極小サイズの入力を並べた狭幅パネル。「位置」の X/Y/幅/高さ +
/// 単位選択のみを持つ（`fieldset` は Xs 段を登録していないため
/// `Size::Sm` のまま、モジュール doc「使用部品」節参照）。
fn panel_compact() -> Node {
    let props = fieldset_props("blocks-form-layout-property-panel-d-position");
    div(
        vec![(
            "class",
            "blocks-form-layout-property-panel-panel blocks-form-layout-property-panel-panel--compact",
        )],
        vec![
            version_caption("極小サイズの狭幅パネル（R0228）"),
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
                                "blocks-form-layout-property-panel-d-pos-x",
                                "X",
                                "120",
                                "-9999",
                                "9999",
                                Size::Xs,
                            ),
                            number_field(
                                "blocks-form-layout-property-panel-d-pos-y",
                                "Y",
                                "48",
                                "-9999",
                                "9999",
                                Size::Xs,
                            ),
                            number_field(
                                "blocks-form-layout-property-panel-d-pos-w",
                                "幅",
                                "320",
                                "0",
                                "9999",
                                Size::Xs,
                            ),
                            number_field(
                                "blocks-form-layout-property-panel-d-pos-h",
                                "高さ",
                                "180",
                                "0",
                                "9999",
                                Size::Xs,
                            ),
                            div(
                                vec![("class", "blocks-form-layout-property-panel-grid-span-2")],
                                vec![unit_select(
                                    "blocks-form-layout-property-panel-d-unit",
                                    "単位",
                                    &[("px", "px"), ("percent", "%"), ("rem", "rem")],
                                    Size::Xs,
                                    None,
                                )],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// `form-layout-property-panel` の Demo 本体。集約元 4 件（R0225〜R0228、
/// A〜D）を並記する（#2914。モジュール doc「集約元と版の対応」節参照）。
/// 呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-form-layout-property-panel-layout")],
        vec![
            panel_basic(),
            panel_theme_collapsible(),
            panel_card_font(),
            panel_compact(),
        ],
    )
}
```

## 原案差分メモ

- **R0225（A・主参照）**: 位置／レイアウト／文字の 3 節を持つプロパティ
  パネルです（前半 #2913 で実装済み）。単位選択（px/%/rem）は X/Y/幅/高さ
  の 4 欄で 1 個の `native_select` を共有する簡略化をしています（デザイン
  ツールの慣行に合わせた判断で、欄ごとに 4 個の selector を並べていませ
  ん）。回転は単位が `deg` 固定のため selector を持たず、平文で添えていま
  す。`segment_group`（方向・揃え）はネイティブ `disabled` の radio で選択
  状態を固定表示にしています。色欄は `color_swatch`（装飾、`aria-hidden`）
  + 16 進値のテキスト入力（`field`/`input`）の組み合わせで表現していま
  す。
- **R0226（B）**: テーマ選択を先頭に、開閉式の節を 2 つ持つ版です。「塗り」
  は開いた状態、「枠線」は閉じた状態で並記し、状態の違いを示します。
  `collapsible` の trigger/content は無 JS の docs サイトでは開閉できない
  ため `disabled: true` で固定し、開いた節・閉じた節の両方を SSR へ含める
  ことで表現しています。`color_picker` は `ColorPickerProps { readonly:
  true, .. }` を全パーツへ一律に渡し、表示値と見本がずれないようにしてい
  ます。
- **R0227（C）**: カード枠 + パンくずのヘッダーで包んだフォント設定の版で
  す。パンくずの途中項目は遷移先を持たない平文表示（末尾のみ現在ページを
  示す `current-link`）にしています。行間ラベルの横に添えた `tooltip` は
  開いた状態（`OpenState::Open`）で常時表示です。フッターの `clipboard` は
  コピー対象を無害な CSS 宣言 1 行にしています。
- **R0228（D）**: 極小サイズ（`Size::Xs`）の入力を並べた狭幅パネルです。
  `fieldset` は Xs 段のサイズ variant を持たないため `Size::Sm` のままにし
  ています。
- `tooltip`/`clipboard` の trigger は開閉・コピー操作が JS ハイドレーション
  前提のため（無 JS の docs サイトでは押しても機能しません）ネイティブ
  `disabled` で操作不能にしています（`segment_group`/`collapsible`/フォント
  欄の `select` も同様です）。
- 文言・配色・数値はすべて架空のものです。

関連情報: [Field](../themes/field.md) / [Fieldset](../themes/fieldset.md) /
[Number Input](../themes/number-input.md) /
[Native Select](../themes/native-select.md) / [Select](../themes/select.md) /
[Color Picker](../themes/color-picker.md) /
[Color Swatch](../themes/color-swatch.md) /
[Segment Group](../themes/segment-group.md) /
[Collapsible](../themes/collapsible.md) / [Tooltip](../themes/tooltip.md) /
[Card](../themes/card.md) / [Breadcrumb](../themes/breadcrumb.md) /
[Clipboard](../themes/clipboard.md) / [Separator](../themes/separator.md)
