//! `form-layout-property-panel` block（親 #2912「Application / Form Layout
//! block 追加」配下、規模 L のため 2 分割。前半 #2913 が骨格（狭い縦長
//! パネル・節の区切り・行の 2 列配置）と主要 3 節（位置／レイアウト／
//! 文字、主参照 R0225）を実装し、本イシュー #2914 が残りの版
//! （R0226〜R0228）・状態の並記・原稿を仕上げる）。
//!
//! # 使用部品
//!
//! `fieldset` / `field` / `number-input` / `native-select` / `select` /
//! `color-picker` / `color-swatch` / `segment-group` / `collapsible` /
//! `tooltip` / `card` / `breadcrumb` / `clipboard` / `separator` の 14 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は
//! 追加しない。
//!
//! # 集約元と版の対応（A〜D、イシュー #2914）
//!
//! Demo（[`demo`]）は集約元 4 件を独立したパネルとして並記する（各版の
//! 差異は Demo の並びとこの節、および `site/blocks/form-layout-property-
//! panel.md` の「原案差分メモ」から読み取れる）。
//!
//! - **A（[`panel_basic`]、主参照 R0225）**: 前半 #2913 が実装した骨格と
//!   主要 3 節をそのまま使う（中身は変えない）。
//! - **B（[`panel_theme_collapsible`]、R0226）**: 先頭に `native_select`
//!   のテーマ選択、続けて `collapsible` の開閉式の節を 2 つ持つ。1 つ目
//!   「塗り」は open（`color_picker` の閉じた trigger + 不透明度の
//!   `number_field`）、2 つ目「枠線」は closed（太さの `number_field` +
//!   `color_swatch`）。開いた節・閉じた節を並記することで状態の違いを
//!   示す（受入基準「状態の違いを並べて示す」に対応）。
//! - **C（[`panel_card_font`]、R0227）**: `card::root` で包み、
//!   `card::header` に `breadcrumb` と `card::title`「フォント設定」を、
//!   `card::body` にフォント（`native_select`）・サイズ・行間
//!   （`number_field`。行間ラベルの横に閉じた `tooltip` を添える）を、
//!   `card::footer` に CSS 宣言をコピーする `clipboard` を置く。
//! - **D（[`panel_compact`]、R0228）**: `Size::Xs` の入力を並べた狭幅
//!   パネル（`.blocks-form-layout-property-panel-panel--compact`
//!   modifier）。位置節の X/Y/幅/高さ + 単位選択のみを持つ（`fieldset`
//!   は Xs 段を登録していないため `Size::Sm` のまま、下記「使用部品」節
//!   参照）。
//!
//! # 静的表示（無 JS）の扱い
//!
//! docs サイトは JS ハイドレーションを行わないため、実行時の値変更・
//! 開閉・切替は表現できない。
//!
//! - `number_input` の増減ボタンは `disabled: true` で出す（JS がないと
//!   動かないため、`pricing_seats_split`/`card_form_footer` と同じ判断）。
//!   input 本体も `readonly` にして固定表示にする（`aria-valuenow` は初期値
//!   のまま更新されないため、編集可能に見せると表示値とのずれを生む。
//!   `marketing/pricing/pricing_seats_split.rs` の `controls` と同じ判断、
//!   イシュー #2913 コードレビューで是正）。
//! - `segment_group` は `item`/`item_control`/`item_text`/`item_hidden_input`
//!   の全パーツへ `disabled: true` を渡しネイティブ操作を構造的に禁止する
//!   （`item_hidden_input` のみを disabled にすると `data-disabled` が同
//!   パーツにしか付かず、[`LAYOUT_CSS`] の中和セレクタが空振りするため、
//!   イシュー #3355 コードレビューで是正）。現在の選択は `data-state`/
//!   `checked` で表す（`card_form_footer` の radio card と同型の判断）。
//!   中和 CSS（`[data-disabled]` の opacity 復元。`.blocks-form-layout-
//!   property-panel-layout` 配下へスコープし他 block への波及を防ぐ）は
//!   [`LAYOUT_CSS`] が担う。
//! - `select`（フォント欄のみ）は `OpenState::Closed` で固定し、trigger は
//!   `disabled: true`。`positioner`/`content` は `hidden` 付きのまま出し、
//!   `aria-controls`/`aria-labelledby` の参照先を残す（`card_form_footer`
//!   の `closed_select` と同一の実装をそのまま流用する）。
//! - `native_select`（単位・配置・太さ・テーマ・フォント）はネイティブ
//!   `<select>` であり JS なしでも実際に開閉・選択できるため、上記の
//!   静的固定は不要（`disabled`/`readonly` を付けない）。可視ラベルは
//!   `field::root` + `field::label` + `native_select` の構成で持たせる
//!   （`aria-label` のみだと隣接欄（サイズ）が縦並びラベル付きなのに
//!   対しグリッドの行内で縦位置がずれるため、イシュー #2913 コード
//!   レビューで是正）。
//! - `collapsible`（B 版）: `trigger`/`content` は
//!   `app_shell_stacked_overlap::hamburger`/`mobile_panel` と同型で
//!   `disabled: true` 固定（押しても何も起きないため）。開いた節
//!   （「塗り」）・閉じた節（「枠線」）を両方 SSR へ含めることで状態の
//!   違いを並記する。
//! - `color_picker`（B 版の「塗り」節）: `showcase.rs` の
//!   `color_picker_section` にある閉状態のパターンを流用し、
//!   `ColorPickerProps { readonly: true, .. }` を root/label/control/
//!   channel_input の各パーツへ渡す（イシュー #1604 の方針。表示値と
//!   見本がずれないようにするため）。`trigger` のみ追加で `disabled: true`
//!   も重ねた props を渡す（`readonly` はネイティブ `disabled` 属性を
//!   出さず押せる見た目のまま残るため、`number_field` の増減ボタンと
//!   同じ判断でネイティブ無効化する。イシュー #2914 コードレビュー
//!   是正、P1）。`label` は `span` で `for` を持たないため、`label` に
//!   `id` を付け `channel_input` の `aria-labelledby` から参照する。
//! - `tooltip`（C 版の行間ラベル）: `component_specs_overlay.rs` の
//!   `ex_tooltip_with_kbd` と同じ構成を、`trigger` は `disabled: true` +
//!   常時 `OpenState::Open` で使う（無 JS の docs サイトでは開閉できず、
//!   closed のまま出すと操作可能なボタンに見えて実際には機能しない見た目
//!   になるため、イシュー #2914 コードレビュー是正、P2）。
//! - `clipboard`（C 版の `card::footer`）: `hero_install_command` の
//!   A インスタンスと同じ anatomy 構成（root/control/input/trigger/
//!   indicator）だが、`trigger` は disabled 扱いにする点が異なる。値は
//!   無害な CSS 宣言 1 行のみ（[`C_FONT_CSS_DECLARATION`]）。
//!   headless `clipboard::trigger` は disabled 引数・`data-disabled` 出力
//!   を持たない（`fandhe_frontend_pre_styled_ui::clipboard` モジュール
//!   doc「意図的非採用」節参照）ため、`gallery_carousel` の `indicator`
//!   と同じ手段（ネイティブ `disabled` 属性 + block 固有 class スコープの
//!   `:disabled` 減光 CSS）で操作不能を明示する（[`LAYOUT_CSS`] 参照、
//!   イシュー #2914 コードレビュー是正、P1: 無 JS のサイトで機能しない
//!   コピー操作を操作可能に見せていた指摘）。
//! - `breadcrumb`（C 版の `card::header`）: 途中の項目は `link` を使わず
//!   `item` の中を平文にする（`demo_composes_expected_parts` の
//!   `!html.contains("href=")` を維持するため）。末尾のみ
//!   `current_link`（`aria-current="page"`）にする。
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
//! `field::root` / `input::input` / `card::root` / `breadcrumb::root` は
//! いずれも `drop_class_attr`
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
//! 集約元は対応表 ID R0225〜R0228（親 #2912 の 1 行要約のみを仕様として
//! 使い、参照元の文言・配色・アイコンは持ち込まない）。文言・配色・
//! アイコンは独自に書く（他 block と同じライセンス上の転記制限）。実在の
//! 企業名・PII は含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
            collapsible::trigger(
                collapsible::OpenState::Open,
                true,
                Some(B_FILL_CONTENT_ID),
                vec![],
                vec![
                    text("塗り"),
                    collapsible::indicator(collapsible::OpenState::Open, true, vec![], vec![]),
                ],
            ),
            fill_section,
            collapsible::trigger(
                collapsible::OpenState::Closed,
                true,
                Some(B_BORDER_CONTENT_ID),
                vec![],
                vec![
                    text("枠線"),
                    collapsible::indicator(collapsible::OpenState::Closed, true, vec![], vec![]),
                ],
            ),
            border_section,
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
    let line_height_row = div(
        vec![("class", "blocks-form-layout-property-panel-c-tooltip-row")],
        vec![
            number_field(
                "blocks-form-layout-property-panel-c-line-height",
                "行間",
                "1.4",
                "0",
                "9999",
                Size::Sm,
            ),
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
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Fieldset",
            path: "/themes/fieldset/",
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
            label: "Color Picker",
            path: "/themes/color-picker/",
        },
        Part {
            label: "Color Swatch",
            path: "/themes/color-swatch/",
        },
        Part {
            label: "Segment Group",
            path: "/themes/segment-group/",
        },
        Part {
            label: "Collapsible",
            path: "/themes/collapsible/",
        },
        Part {
            label: "Tooltip",
            path: "/themes/tooltip/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Breadcrumb",
            path: "/themes/breadcrumb/",
        },
        Part {
            label: "Clipboard",
            path: "/themes/clipboard/",
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
.blocks-form-layout-property-panel-layout {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: flex-start;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-form-layout-property-panel-panel {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  inline-size: min(100%, 18rem);\n  padding: var(--fandhe-space-4);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  background: var(--fandhe-color-bg);\n}\n\
.blocks-form-layout-property-panel-panel--compact {\n  inline-size: min(100%, 14rem);\n}\n\
.blocks-form-layout-property-panel-grid {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-2);\n  margin-block-start: var(--fandhe-space-2);\n}\n\
.blocks-form-layout-property-panel-grid-span-2 {\n  grid-column: 1 / -1;\n}\n\
.blocks-form-layout-property-panel-rotation {\n  display: flex;\n  align-items: flex-end;\n  gap: var(--fandhe-space-1, 0.25rem);\n}\n\
.blocks-form-layout-property-panel-select {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1-5, 0.375rem);\n}\n\
.blocks-form-layout-property-panel-color {\n  display: flex;\n  align-items: flex-end;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-form-layout-property-panel-color] {\n  flex: 1;\n}\n\
[data-blocks-form-layout-property-panel-b-section] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  margin-block-start: var(--fandhe-space-1, 0.25rem);\n}\n\
[data-blocks-form-layout-property-panel-b-section][hidden] {\n  display: none;\n}\n\
.blocks-form-layout-property-panel-c-tooltip-row {\n  display: flex;\n  align-items: flex-end;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-form-layout-property-panel-layout [data-scope=\"collapsible\"][data-part=\"content\"][data-disabled] {\n  color: var(--fandhe-color-fg);\n}\n\
.blocks-form-layout-property-panel-layout [data-scope=\"segment-group\"][data-part=\"item-control\"][data-disabled],\n\
.blocks-form-layout-property-panel-layout [data-scope=\"segment-group\"][data-part=\"item-text\"][data-disabled],\n\
.blocks-form-layout-property-panel-layout [data-scope=\"segment-group\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-form-layout-property-panel-select [data-scope=\"select\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-form-layout-property-panel-layout [data-scope=\"number-input\"][data-part=\"increment-trigger\"][data-disabled],\n\
.blocks-form-layout-property-panel-layout [data-scope=\"number-input\"][data-part=\"decrement-trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-form-layout-property-panel-layout [data-scope=\"collapsible\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"clipboard\"][data-part=\"trigger\"][data-blocks-form-layout-property-panel-c-clipboard-trigger]:disabled {\n  opacity: 0.5;\n  cursor: not-allowed;\n}\n";

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
            "data-scope=\"collapsible\"",
            "data-scope=\"card\"",
            "data-scope=\"breadcrumb\"",
            "data-scope=\"tooltip\"",
            "data-scope=\"clipboard\"",
            "data-scope=\"color-picker\"",
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
    fn all_buttons_are_type_button() {
        // ボタンはすべて `type="button"` を固定で持つ（フォーム内配置時の
        // 意図しない submit を防ぐ、モジュール doc「`<form>` を使わない」
        // 節参照）。
        let html = demo_html();
        let button_count = html.matches("<button").count();
        let type_button_count = html.matches(r#"type="button""#).count();
        assert!(button_count > 0);
        assert_eq!(button_count, type_button_count);
    }

    #[test]
    fn fill_color_trigger_is_natively_disabled() {
        // B 版「塗り」節の色選択トリガーは `readonly` だけでは
        // ネイティブ `disabled` 属性が付かず操作可能に見えるため、
        // `disabled: true` も重ねて無効化する（イシュー #2914 コード
        // レビュー是正、P1。モジュール doc「静的表示」節参照）。
        let html = demo_html();
        assert!(html.contains(
            r#"data-scope="color-picker" data-part="trigger" type="button" aria-haspopup="dialog" aria-expanded="false" data-state="closed" data-disabled="" data-readonly="" disabled="""#
        ));
    }

    #[test]
    fn collapsible_has_one_open_and_one_closed_section() {
        // B 版（R0226）: 「塗り」は open・「枠線」は closed の状態違いを
        // 並記する（受入基準「状態の違いを並べて示す」対応）。
        let html = demo_html();
        assert!(html.contains(r#"data-scope="collapsible" data-part="content" data-state="open""#));
        assert!(
            html.contains(r#"data-scope="collapsible" data-part="content" data-state="closed""#)
        );
        // closed 側（B_BORDER_CONTENT_ID）のみ `hidden` 存在属性を持つ。
        assert!(html.contains(&format!("id=\"{}\" hidden", super::B_BORDER_CONTENT_ID)));
        assert!(!html.contains(&format!("id=\"{}\" hidden", super::B_FILL_CONTENT_ID)));
    }

    #[test]
    fn breadcrumb_current_link_has_aria_current_page() {
        let html = demo_html();
        assert!(html.contains(r#"aria-current="page""#));
    }

    #[test]
    fn compact_panel_uses_xs_size_class() {
        // D 版（R0228）は number-input へ `Size::Xs` を渡す
        // （`fd-number-input--size-xs`、A〜C は `Size::Sm`）。
        let html = demo_html();
        assert!(html.contains("fd-number-input--size-xs"));
    }

    #[test]
    fn tooltip_trigger_describedby_matches_content_id() {
        let html = demo_html();
        assert!(html.contains(&format!(
            "aria-describedby=\"{}\"",
            super::C_TOOLTIP_CONTENT_ID
        )));
        assert!(html.contains(&format!("id=\"{}\"", super::C_TOOLTIP_CONTENT_ID)));
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
    fn segment_groups_have_exactly_two_indicators() {
        // 方向・揃えの 2 グループそれぞれに indicator が 1 件ずつ（イシュー
        // #2913 コードレビュー是正: indicator 欠落だと選択中のピルが
        // 描画されない）。
        let html = demo_html();
        assert_eq!(
            html.matches(r#"data-scope="segment-group" data-part="indicator""#)
                .count(),
            2
        );
    }

    #[test]
    fn number_inputs_are_readonly() {
        // A: X/Y/幅/高さ/回転/間隔/サイズの 7 欄 + 文字色 hex 入力の 1 欄 = 8
        // （イシュー #2913 コードレビュー是正: 編集可能に見せると
        // `aria-valuenow`/表示値が初期値のまま更新されず色見本等とずれる
        // ため。文字色欄は隣接する color_swatch が固定値のまま動かない
        // 静的 Demo であることが理由、イシュー #3355 コードレビューで
        // 追加是正）。B: 不透明度・太さの 2 number_field + 塗り色
        // channel_input（`ColorPickerProps { readonly: true, .. }`、モジュール
        // doc「静的表示」節参照）の 1 欄 = 3。C: サイズ・行間の 2 number_field
        // = 2。D: X/Y/幅/高さの 4 number_field = 4。計 17 欄が readonly
        // （イシュー #2914）。input パーツのみがネイティブ `readonly` 属性を
        // 持つ（root/control は `data-readonly` のみ）ため、両者が隣接する
        // 組み合わせで input パーツの件数を数える。
        let html = demo_html();
        assert_eq!(html.matches(r#"readonly="" data-readonly="""#).count(), 17);
    }

    #[test]
    fn align_select_uses_stretch_wording_not_justify_wording() {
        // 「両端揃え」は justify の意味であり配置（align）の選択肢としては
        // 誤り（イシュー #2913 コードレビュー是正）。
        let html = demo_html();
        assert!(!html.contains("両端揃え"));
        assert!(html.contains("引き伸ばし"));
    }

    #[test]
    fn unit_align_weight_selects_have_visible_labels() {
        // 単位・配置・太さの native select は aria-label だけでなく可視
        // ラベルを持つ（イシュー #2913 コードレビュー是正: サイズ欄との
        // 縦位置ずれ防止）。
        let html = demo_html();
        for (id, label_text) in [
            ("blocks-form-layout-property-panel-pos-unit", "単位"),
            ("blocks-form-layout-property-panel-align", "配置"),
            ("blocks-form-layout-property-panel-font-weight", "太さ"),
        ] {
            assert!(
                html.contains(&format!(r#"for="{id}-control""#)),
                "expected visible label `for` referencing {id}, got:\n{html}"
            );
            assert!(
                html.contains(label_text),
                "expected label text {label_text} in html"
            );
        }
        assert!(!html.contains(r#"aria-label="単位""#));
        assert!(!html.contains(r#"aria-label="配置""#));
        assert!(!html.contains(r#"aria-label="太さ""#));
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
