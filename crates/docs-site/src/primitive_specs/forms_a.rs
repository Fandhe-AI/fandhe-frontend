//! Primitives（`fandhe-frontend-headless-ui`）Forms A（入力系 11 部品）の
//! 原稿レジストリ（イシュー #1024、親 #1030、ルート #1035 Phase 5）。
//!
//! # 役割・呼び出し文脈
//!
//! [`crate::component_page::spec_for`] が `Layer::Primitives` のときに探索
//! する [`crate::primitive_specs::SPEC_TABLES`] の 1 要素として本モジュールの
//! [`SPECS`] を返す。対象 11 部品（angle-slider / checkbox / checkbox-group /
//! color-picker / combobox / editable / field / fieldset / file-upload /
//! image-cropper / listbox）の Demo（Anatomy・`data-*` 属性表の機械導出元）
//! はすでに [`crate::primitive_showcase::forms_a`]（イシュー #1022）が供給
//! 済みであり、本モジュールは Features / API Reference 引数表 / Examples /
//! Accessibility の 4 節のみを埋める（[`crate::component_page::ComponentPageSpec`]
//! 参照）。CSS 変数表は Primitives 層で恒常的に省略される
//! （headless-ui に CSS の概念が無い、`docs/design/docs-site-primitives-themes-split.md`
//! §5）。
//!
//! # 一次情報・非捏造の方針
//!
//! `features`/`arguments`/`keyboard`/`aria` の各行は `crates/headless-ui/src/`
//! の実ソースを一次情報とし、各定数の doc コメントに `file:line` 形式で
//! 根拠を付す。根拠を示せない行は掲載しない
//! （`.claude/rules/out-of-scope-tracking.md` の「推測で補完しない」方針、
//! `docs/design/docs-site-component-pages.md` §10 と同じ判断軸）。
//!
//! # Phase 5 の house style（D1〜D5、#1025〜#1029 が踏襲する規約）
//!
//! - **D1（`ArgRow` にパーツ列を足さず `name` セルへ所有者を埋め込む）**:
//!   [`crate::component_page::ArgRow`] は `{ name, kind, default, description }`
//!   の 4 列固定（共有ファイル `component_page.rs` へフィールドを足さない）。
//!   Primitives の 1 部品は複数の自由関数 + Props 構造体で構成されるため、
//!   関数引数は `関数名(引数名)`、Props フィールドは `props.<field>`、
//!   列挙型の別コンストラクタは `型::variant` と表記する。`attrs`/`children`
//!   のような全パーツ共通の定型引数は代表 1 行に集約し、その旨を
//!   `description` に明記する。全公開関数の全引数を網羅する必要はない
//!   （目安 6〜12 行）。
//! - **D2（`demo: None` を固定する）**: [`crate::component_page::generated_content`]
//!   は `Layer::Primitives` のとき [`crate::primitive_showcase`] を先に
//!   照会し、`None` のときだけ [`crate::component_page::ComponentPageSpec::demo`]
//!   へフォールバックする（`component_page.rs` 参照）。Forms A は 11 件
//!   すべて `primitive_showcase::forms_a` に登録済みのため、ここで `Some`
//!   を書くと到達不能なデッドコードになる（PR #982 の二重登録事故と同じ形。
//!   `crate::component_specs::forms` モジュール doc 参照）。
//! - **D3（Accessibility は `aria` が主・`keyboard` は限定）**: 本 docs
//!   サイトは `crate::script`（テーマトグル + 目次スクロールスパイ）以外の
//!   JS を出力せず、headless-ui 自体も「キーボードナビゲーション・
//!   typeahead の実 DOM 配線は wasm 層の将来イシュー」と out-of-scope を
//!   明言している（例: `crates/headless-ui/src/listbox.rs` の
//!   スコープ外節）。したがってネイティブ要素（`<input>`/`<button>`/
//!   `<fieldset>` 等）のブラウザ標準操作に限って `keyboard` を記載し、
//!   JS 状態機械前提のキー操作（矢印キーでの候補移動等）は「できる」と
//!   書かない。`aria` は実装が実際に SSR 出力する `aria-*`/`role` のみを
//!   書く。
//! - **D4（Examples は headless-ui API のみ・`hui` 再エクスポート経由）**:
//!   import は `crate::primitive_showcase::forms_a` と同型にし、
//!   `use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui as hui;`
//!   経由でのみ headless-ui のパート関数を呼ぶ（`fandhe_frontend_pre_styled_ui::`
//!   を直接 import しない。イシュー #693 方針、`crates/docs-site` は
//!   headless-ui へ直接依存を追加しない）。Examples レンダラは `h2`/`h3` を
//!   出さず（[`crate::component_page::examples_section`] が `h3` を供給
//!   済み、右カラム目次汚染の過去事故 #980 と同型の回避）、`docs-` 接頭辞の
//!   class も持ち込まない。可読性のための枠は
//!   `primitives-demo-frame`/`primitives-demo-note`（[`crate::primitive_showcase`]
//!   の `stylesheet()` に実セレクタがあり、Primitives ページに
//!   `assets/primitives-showcase.css` が配線されているため契約テストを
//!   通る）に限定する。
//! - **D5（`KNOWN_UNCOVERED` パーツをページ上で確認できる前提にしない）**:
//!   `combobox`/`item-group-label`、`field`/`select`・`field`/`textarea`、
//!   `color_picker` の 9 スライダーパートは `tests/primitive_showcase.rs`
//!   の `KNOWN_UNCOVERED` により Anatomy 表へ意図的に出ない
//!   （`primitive_showcase::forms_a` を編集して「直す」ことはしない）。
//!   実装に存在するため API Reference 引数表・Features での言及自体は可。
//!
//! # 責務境界（`docs/policy/intentional-non-adoption.md` §3.25）
//!
//! UI コンポーネント層が担うのは anatomy・アクセシビリティ・表示状態
//! （`data-*`）まで。バリデーション・送信処理・データ整形・永続化を
//! 部品が担うかのような Features/Examples の記述はしない（`combobox` の
//! 候補データ取得、`file_upload`/`image_cropper` の実ファイル保存・画像
//! 処理はいずれも利用者側の責務）。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! すべてのデータは `&'static str` リテラルであり、[`crate::component_page`]
//! 側で `fandhe_frontend_core::text()` 経由（既定エスケープ）にのみ出力
//! される。本モジュールは `raw_html()` および HTML 文字列の直接組み立て
//! （`format!("<td>{}</td>", …)`）を一切使わない。Examples レンダラも
//! ノード木 API（[`fandhe_frontend_core::el`]/[`fandhe_frontend_core::text`]
//! と headless-ui のパート関数）のみで組み立てる。

use fandhe_frontend_core::{code, div, p, pre, text, Node};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui as hui;
use hui::angle_slider::AngleSliderProps;
use hui::checkbox::{CheckboxProps, CheckedState};
use hui::checkbox_group;
use hui::color_picker;
use hui::combobox::{self, ComboboxProps};
use hui::editable::{
    self, EditMode, EditableActivationMode, EditableInputFlags, EditableInputProps,
    EditableSubmitMode,
};
use hui::field::{self, FieldProps};
use hui::fieldset::{self, FieldsetProps};
use hui::file_upload;
use hui::image_cropper::{self, HandlePosition};
use hui::listbox;
use hui::{angle_slider, checkbox, OpenState};

use crate::component_page::{ArgRow, AriaRow, ComponentPageSpec, ExampleEntry, KeyRow};

/// Forms A 11 ページの `path -> ComponentPageSpec` テーブル（path 昇順）。
/// [`crate::primitive_specs::SPEC_TABLES`] が集約する。
pub const SPECS: &[(&str, ComponentPageSpec)] = &[
    ("/primitives/angle-slider/", ANGLE_SLIDER),
    ("/primitives/checkbox/", CHECKBOX),
    ("/primitives/checkbox-group/", CHECKBOX_GROUP),
    ("/primitives/color-picker/", COLOR_PICKER),
    ("/primitives/combobox/", COMBOBOX),
    ("/primitives/editable/", EDITABLE),
    ("/primitives/field/", FIELD),
    ("/primitives/fieldset/", FIELDSET),
    ("/primitives/file-upload/", FILE_UPLOAD),
    ("/primitives/image-cropper/", IMAGE_CROPPER),
    ("/primitives/listbox/", LISTBOX),
];

/// Examples 用の枠組み。[`crate::primitive_showcase::forms_a`] のデモ本体と
/// 同じ `primitives-demo-frame`/`primitives-demo-note` class のみを使い、
/// `h2`/`h3` は出さない（D4 参照。`examples_section` が `h3` を供給済み）。
fn wrap_example(note: &'static str, body: Vec<Node>) -> Node {
    div(
        vec![],
        vec![
            p(vec![("class", "primitives-demo-note")], vec![text(note)]),
            div(vec![("class", "primitives-demo-frame")], body),
        ],
    )
}

// ---------------------------------------------------------------------
// Angle Slider
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/angle_slider.rs:274-451`（root/label/
/// control/thumb/hidden_input/value_text/marker_group/marker の各パーツ
/// 関数、`AngleSliderProps`）。イシュー #1601 で参照突合し、readonly/
/// invalid・marker_group/marker・role="presentation" を追加した。
fn ex_angle_slider() -> Node {
    let props = AngleSliderProps {
        readonly: true,
        invalid: true,
        ..Default::default()
    };
    let body = vec![angle_slider::root(
        &props,
        vec![],
        vec![
            angle_slider::label(&props, vec![], vec![text("Wind direction")]),
            angle_slider::control(
                &props,
                vec![],
                vec![
                    angle_slider::thumb("270", "270deg", &props, vec![], vec![]),
                    angle_slider::marker_group(
                        vec![],
                        vec![
                            angle_slider::marker(180, 270, false, vec![], vec![]),
                            angle_slider::marker(270, 270, false, vec![], vec![]),
                            angle_slider::marker(315, 270, false, vec![], vec![]),
                        ],
                    ),
                ],
            ),
            angle_slider::hidden_input("wind-direction", "270", false, vec![]),
            angle_slider::value_text(vec![], vec![text("270°")]),
        ],
    )];
    wrap_example(
        "readonly かつ invalid な 270 度の風向を、marker_group/marker（180/270/315 度の目盛り）付きで組み立てた例です。",
        body,
    )
}

const ANGLE_SLIDER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "root/label/control/thumb/marker_group/marker/hidden_input/value_text の 8 anatomy パーツで構成し、角度値は常に 0..=359 の整数へ正規化される（angle_slider.rs:170-177）。",
        "AngleSliderProps（disabled/readonly/invalid）を root/label/control/thumb で共有し、data-disabled/data-readonly/data-invalid を一律付与する（angle_slider.rs:274-291）。",
        "control は role=\"presentation\" を固定出力する（angle_slider.rs:331-346）。",
        "thumb は WAI-ARIA slider パターンに従い role=\"slider\"/aria-valuemin=\"0\"/aria-valuemax=\"360\"/aria-valuenow/aria-valuetext を常時出力する。props.disabled が true のとき tabindex=\"-1\" + aria-disabled、それ以外（readonly を含む）は tabindex=\"0\"（angle_slider.rs:349-382）。",
        "marker は value（目盛り角度）と現在角度の大小から data-state を under-value/over-value/at-value の 3 値へ固定する（angle_slider.rs:423-451）。",
        "hidden_input（<input type=\"hidden\">）はフォーム送信専用であり、意味論（role=\"slider\"）は thumb 側が担う（angle_slider.rs:386-401）。",
        "\"home\"/\"end\" dispatch（AngleSliderAction::SetToMin/SetToMax）で最小値（0 度）/step グリッド上の最大値へ設定する状態機械契約を持つ（angle_slider.rs:613-617, 650-651）。fandhe-frontend-wasm-full の DOM keydown 配線は REQ-11（WASM バンドルサイズ）予算逼迫のため本イシューでは未対応（Arrow キーのみ配線済み、下記 Keyboard 節参照）。",
    ],
    arguments: &[
        ArgRow {
            name: "root/label/control/thumb(props)",
            kind: "&AngleSliderProps",
            default: "&AngleSliderProps::default()",
            description: "disabled/readonly/invalid の状態束。4 パーツ共通のため代表 1 行に集約（angle_slider.rs:274-291）。",
        },
        ArgRow {
            name: "thumb(now)",
            kind: "&str",
            default: "",
            description: "現在角度の文字列表現。aria-valuenow へそのまま出力される。",
        },
        ArgRow {
            name: "thumb(value_text)",
            kind: "&str",
            default: "",
            description: "\"{value}deg\" 形式の文字列。aria-valuetext へ出力される。",
        },
        ArgRow {
            name: "marker(value, current, disabled)",
            kind: "u16, u16, bool",
            default: "",
            description: "目盛り角度・現在角度・無効化。data-value/data-state（under-value/over-value/at-value）へ反映される（angle_slider.rs:423-451）。",
        },
        ArgRow {
            name: "hidden_input(name, value)",
            kind: "&str, &str",
            default: "",
            description: "フォーム送信名・送信値。<input type=\"hidden\"> の name/value 属性へそのまま出力される。",
        },
        ArgRow {
            name: "attrs / children",
            kind: "Vec<(&str, &str)> / Vec<Node>",
            default: "",
            description: "各パーツ共通の追加属性・子ノード（全パーツで型が同じため代表 1 行に集約）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Readonly + Invalid",
        description: "readonly かつ invalid な 270 度の風向を marker_group/marker 付きで組み立てた例です。",
        render: ex_angle_slider,
    }],
    keyboard: &[
        KeyRow {
            key: "ArrowUp / ArrowRight",
            description: "step 分だけ時計回りに増加する（fandhe-frontend-wasm-full の action_for_key で配線済み）。",
        },
        KeyRow {
            key: "ArrowDown / ArrowLeft",
            description: "step 分だけ反時計回りに減少する（同上）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "role=\"presentation\"",
            description: "control パーツに固定付与する。意味論は thumb の role=\"slider\" が単独で担う（angle_slider.rs:331-346）。",
        },
        AriaRow {
            attribute: "role=\"slider\"",
            description: "thumb パーツに固定付与する。",
        },
        AriaRow {
            attribute: "aria-valuemin / aria-valuemax",
            description: "thumb パーツへ常に \"0\"/\"360\" を固定出力する。",
        },
        AriaRow {
            attribute: "aria-valuenow / aria-valuetext",
            description: "呼び出し側が渡す現在値・整形済みテキストをそのまま出力する。",
        },
        AriaRow {
            attribute: "aria-disabled",
            description: "props.disabled が true のとき tabindex=\"-1\" と対で thumb へ付与する（readonly のみでは付与しない）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Checkbox
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/checkbox.rs:219-330`（root/control/
/// indicator/hidden_input の各パーツ関数、`CheckboxProps`）。
fn ex_checkbox() -> Node {
    let props = CheckboxProps {
        checked: CheckedState::Unchecked,
        invalid: true,
        required: true,
        ..Default::default()
    };
    let body = vec![checkbox::root(
        &props,
        vec![],
        vec![
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![text("✓")])],
            ),
            checkbox::label(&props, vec![], vec![text("Agree to terms")]),
            checkbox::hidden_input(&props, "terms", "on", vec![]),
        ],
    )];
    wrap_example(
        "invalid かつ required な未チェック状態の Checkbox の組み立て例です。",
        body,
    )
}

/// 自前 CSS の最小例。headless-ui 自体はスタイルを持たないため、利用者が
/// `data-scope`/`data-part`/`data-state`/`data-focus-visible`/`data-disabled`
/// 属性セレクタで見た目を組み立てる例を示す（イシュー #1602）。CSS は
/// テキストノード（[`code`]/[`pre`]）として既定エスケープを経由し、
/// `crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは
/// 追加しない。
const CHECKBOX_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"checkbox\"][data-part=\"control\"] {\n  \
  width: 1.25rem;\n  height: 1.25rem;\n  border: 1px solid #888;\n  border-radius: 4px;\n\
}\n\
[data-scope=\"checkbox\"][data-part=\"control\"][data-state=\"checked\"] {\n  \
  background: #2563eb;\n  border-color: #2563eb;\n\
}\n\
[data-scope=\"checkbox\"][data-part=\"control\"][data-state=\"indeterminate\"] {\n  \
  background: #6b7280;\n  border-color: #6b7280;\n\
}\n\
[data-scope=\"checkbox\"][data-part=\"control\"][data-focus-visible] {\n  \
  outline: 2px solid #2563eb;\n  outline-offset: 2px;\n\
}\n\
[data-scope=\"checkbox\"][data-part=\"root\"][data-disabled] {\n  \
  opacity: 0.5;\n\
}\n";

fn ex_checkbox_custom_css() -> Node {
    let props = CheckboxProps {
        checked: CheckedState::Checked,
        ..Default::default()
    };
    let markup = checkbox::root(
        &props,
        vec![],
        vec![
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![text("✓")])],
            ),
            checkbox::label(&props, vec![], vec![text("Accept newsletter")]),
            checkbox::hidden_input(&props, "newsletter", "on", vec![]),
        ],
    );
    wrap_example(
        "利用者が data-scope / data-part / data-state / data-focus-visible / data-disabled 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
        vec![
            markup,
            pre(vec![], vec![code(vec![], vec![text(CHECKBOX_CUSTOM_CSS_SNIPPET)])]),
        ],
    )
}

const CHECKBOX: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "root（<label>）/control（<div aria-hidden=\"true\">）/indicator/label/hidden_input（<input type=\"checkbox\">）の 5 パーツで構成し、視覚的表現（control）とアクセシビリティ実体（hidden_input）を分離する（checkbox.rs:9-14, 230-232）。",
        "CheckedState（Unchecked/Checked/Indeterminate）の 3 値が aria-checked/data-state 双方の唯一の情報源であり、偽装・不整合な値を型で塞ぐ（checkbox.rs:82-85）。",
        "indicator は CheckedState::Unchecked のとき hidden 存在属性を出力する（checkbox.rs:251-260）。",
        "hidden_input は CheckboxProps の disabled/invalid/required を受け取り、aria-invalid・ネイティブ存在属性へ反映する（checkbox.rs:139-153, 305-330）。",
        "root/control は data-focus-visible を出力できる（実フォーカスを受けるのは visually-hidden な hidden_input のため、switch と同型の hidden-input パターン）。wasm-full の focus 配線が hidden-input の focus を境界 root へ写像する（checkbox.rs:45-53、crates/wasm-full/src/focus_visible.rs:51）。",
        "ark-ui が付与する data-hover/data-active/data-focus は出力しない（DOM ローカルな pointer/focus 操作状態のため、SSR 静的出力の関心外。CSS 擬似クラスまたは wasm-full 配線側で表現する設計判断、data_attrs.rs の data_focus_visible/data_highlighted と同型の契約）。",
    ],
    arguments: &[
        ArgRow {
            name: "props.checked",
            kind: "CheckedState",
            default: "CheckedState::Unchecked",
            description: "3 値のチェック状態。aria-checked と data-state 双方の唯一の情報源（checkbox.rs:82-85）。",
        },
        ArgRow {
            name: "props.disabled / props.invalid / props.required / props.readonly",
            kind: "bool",
            default: "false",
            description: "CheckboxProps の各フラグ（checkbox.rs:139-153）。各パーツの data-* / ネイティブ属性へ反映される（代表 1 行に集約）。",
        },
        ArgRow {
            name: "hidden_input(name, value)",
            kind: "&str, &str",
            default: "",
            description: "フォーム送信名・送信値。暗黙の既定値を持たず呼び出し側が明示する（checkbox.rs:284-287）。",
        },
        ArgRow {
            name: "attrs / children",
            kind: "Vec<(&str, &str)> / Vec<Node>",
            default: "",
            description: "各パーツ共通の追加属性・子ノード（代表 1 行に集約）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Invalid + Required",
            description: "props.invalid/props.required を立てた未チェック状態の例です。",
            render: ex_checkbox,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "data-scope / data-part / data-state / data-focus-visible / data-disabled 属性セレクタで見た目を組み立てる最小例です。",
            render: ex_checkbox_custom_css,
        },
    ],
    keyboard: &[KeyRow {
        key: "Space",
        description: "hidden_input が実際に <input type=\"checkbox\"> を出力するため、ブラウザ標準のチェックボックストグル操作が働く（checkbox.rs:14-15, 305-330）。Enter ではトグルしない（WAI-ARIA Checkbox パターン・ネイティブ input の標準挙動に準拠、Radix Primitives も同じ理由で Enter を無効化する）。",
    }],
    aria: &[
        AriaRow {
            attribute: "aria-checked",
            description: "hidden_input へ CheckedState から算出した値（\"true\"/\"false\"/\"mixed\"）を付与する（Indeterminate のとき \"mixed\"、checkbox.rs:312-320）。",
        },
        AriaRow {
            attribute: "aria-invalid",
            description: "props.invalid が true のとき hidden_input へ \"true\" を付与する（checkbox.rs:322-324）。",
        },
        AriaRow {
            attribute: "aria-hidden",
            description: "control パーツ（視覚的表現のみを担う div）に固定付与し、支援技術からの重複読み上げを防ぐ（checkbox.rs:230-232, 240-243）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Checkbox Group
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/checkbox_group.rs`（root/item/
/// item_control/item_indicator/item_text の各パーツ関数、`CheckboxGroupProps`）。
/// イシュー #1603 で `disabled: bool` 引数が `&CheckboxGroupProps` へ署名
/// 変更されたため追随した。
fn ex_checkbox_group() -> Node {
    let props = hui::checkbox_group::CheckboxGroupProps::default();
    let body = vec![checkbox_group::root(
        &props,
        Some(hui::Orientation::Vertical),
        Some("cg-label"),
        vec![],
        vec![
            checkbox_group::label(Some("cg-label"), vec![], vec![text("Toppings")]),
            checkbox_group::item(
                true,
                &props,
                "cheese",
                vec![],
                vec![
                    checkbox_group::item_control(
                        true,
                        &props,
                        vec![],
                        vec![checkbox_group::item_indicator(
                            true,
                            &props,
                            vec![],
                            vec![text("✓")],
                        )],
                    ),
                    checkbox_group::item_text(true, &props, vec![], vec![text("Cheese")]),
                ],
            ),
        ],
    )];
    wrap_example(
        "縦方向 orientation・aria-labelledby 付きの Checkbox Group の組み立て例です。",
        body,
    )
}

/// 自前 CSS の最小例。headless-ui 自体はスタイルを持たないため、利用者が
/// `data-scope`/`data-part`/`data-state`/`data-invalid`/`data-disabled`/
/// `data-orientation` 属性セレクタで見た目を組み立てる例を示す（イシュー
/// #1603、`ex_checkbox_custom_css`〔#1602〕と同型のパターン）。CSS は
/// テキストノード（[`code`]/[`pre`]）として既定エスケープを経由し、
/// `crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは追加しない。
const CHECKBOX_GROUP_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"checkbox-group\"][data-part=\"item-control\"] {\n  \
  width: 1rem;\n  height: 1rem;\n  border: 1px solid #888;\n  border-radius: 4px;\n\
}\n\
[data-scope=\"checkbox-group\"][data-part=\"item-control\"][data-state=\"checked\"] {\n  \
  background: #2563eb;\n  border-color: #2563eb;\n\
}\n\
[data-scope=\"checkbox-group\"][data-part=\"item-control\"][data-invalid] {\n  \
  border-color: #dc2626;\n\
}\n\
[data-scope=\"checkbox-group\"][data-part=\"item-control\"][data-focus-visible] {\n  \
  outline: 2px solid #2563eb;\n  outline-offset: 2px;\n\
}\n\
[data-scope=\"checkbox-group\"][data-part=\"item\"][data-disabled] {\n  \
  opacity: 0.5;\n\
}\n\
[data-scope=\"checkbox-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] {\n  \
  flex-direction: row;\n  gap: 1rem;\n\
}\n";

fn ex_checkbox_group_custom_css() -> Node {
    let props = hui::checkbox_group::CheckboxGroupProps {
        disabled: false,
        readonly: false,
        invalid: true,
    };
    let markup = checkbox_group::root(
        &props,
        Some(hui::Orientation::Horizontal),
        Some("cg-css-label"),
        vec![],
        vec![
            checkbox_group::label(Some("cg-css-label"), vec![], vec![text("Toppings")]),
            checkbox_group::item(
                false,
                &props,
                "olives",
                vec![],
                vec![
                    checkbox::hidden_input(
                        &CheckboxProps {
                            checked: CheckedState::Unchecked,
                            invalid: true,
                            ..Default::default()
                        },
                        "toppings",
                        "olives",
                        vec![],
                    ),
                    checkbox_group::item_control(
                        false,
                        &props,
                        vec![],
                        vec![checkbox_group::item_indicator(
                            false,
                            &props,
                            vec![],
                            vec![text("✓")],
                        )],
                    ),
                    checkbox_group::item_text(false, &props, vec![], vec![text("Olives")]),
                ],
            ),
        ],
    );
    wrap_example(
        "利用者が data-scope / data-part / data-state / data-invalid / data-disabled / data-orientation 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
        vec![
            markup,
            pre(
                vec![],
                vec![code(
                    vec![],
                    vec![text(CHECKBOX_GROUP_CUSTOM_CSS_SNIPPET)],
                )],
            ),
        ],
    )
}

const CHECKBOX_GROUP: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "root（div、role=\"group\"）/label/item（label）/item_control/item_indicator/item_text の 6 パーツで「0 個以上の項目が同時選択される」複数選択を表現する（checkbox_group.rs:1-7, 36-56）。",
        "CheckboxGroupProps（disabled/readonly/invalid）を root と item 系 4 パーツ全てへ一律注入し、data-disabled/data-readonly/data-invalid を出力する（ark-ui Checkbox.Group props 相当、イシュー #1603）。",
        "labelled_by が Some のときのみ aria-labelledby を付与し、名前なし関連付けを作らない（checkbox_group.rs:236-238）。",
        "orientation が Some のときのみ data-orientation を付与する。role=\"group\" は WAI-ARIA 1.2 の aria-orientation Used in Roles に含まれないため、aria-orientation は付与しない（イシュー #1603 是正、radio_group〔role=\"radiogroup\"〕とは異なる）。",
        "ネイティブ入力は crate::checkbox::hidden_input を item（<label>）配下へ入れ子にして再利用し、item_control 自体には role=\"checkbox\"/aria-checked を付与しない（二重読み上げ防止、checkbox_group.rs:11-16, 297-311）。CheckboxGroup::item_hidden_input は root の invalid/readonly もネイティブ <input> の aria-invalid/data-readonly へ OR 伝播する（イシュー #1603）。",
        "ark-ui が付与する data-hover/data-active/data-focus は出力しない（DOM ローカルな pointer/focus 操作状態のため、SSR 静的出力の関心外）。Radix Themes の RovingFocus（矢印キーによる項目間移動）は採用しない（ネイティブ input は各項目が独立した Tab ストップであり、checkbox-group 用の roving tabindex パターンは WAI-ARIA APG に無い）。",
    ],
    arguments: &[
        ArgRow {
            name: "root(props)",
            kind: "&CheckboxGroupProps",
            default: "",
            description: "disabled/readonly/invalid の 3 フラグ。true のフィールドのみ data-disabled/data-readonly/data-invalid として出力される（checkbox_group.rs、イシュー #1603）。",
        },
        ArgRow {
            name: "root(orientation)",
            kind: "Option<Orientation>",
            default: "None",
            description: "Some のときのみ data-orientation を付与する（aria-orientation は付与しない、checkbox_group.rs:233-235）。",
        },
        ArgRow {
            name: "root(labelled_by)",
            kind: "Option<&str>",
            default: "None",
            description: "Some のときのみ aria-labelledby を付与する（label パーツの id と対で使う、checkbox_group.rs:236-238）。",
        },
        ArgRow {
            name: "item(checked, props, value)",
            kind: "bool, &CheckboxGroupProps, &str",
            default: "",
            description: "選択肢 1 個の状態と送信値。value は data-value として動的値のまま出力され既定エスケープを経由する（checkbox_group.rs:266-284）。",
        },
        ArgRow {
            name: "item_control(checked, props)",
            kind: "bool, &CheckboxGroupProps",
            default: "",
            description: "視覚的なチェックボックス外枠。item_indicator を子として渡す契約（styled recipe の中央揃えに効くため、checkbox_group.rs:297-311）。",
        },
        ArgRow {
            name: "attrs / children",
            kind: "Vec<(&str, &str)> / Vec<Node>",
            default: "",
            description: "各パーツ共通の追加属性・子ノード（代表 1 行に集約）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Vertical orientation",
            description: "aria-labelledby・data-orientation=\"vertical\" を付与した例です。",
            render: ex_checkbox_group,
        },
        ExampleEntry {
            title: "Custom CSS",
            description: "自前 CSS の最小例です（data-invalid・水平 orientation 付き）。",
            render: ex_checkbox_group_custom_css,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "Tab / Shift+Tab",
            description: "各項目のネイティブ <input type=\"checkbox\">（item 配下の checkbox::hidden_input）がブラウザ標準の Tab ストップとして機能する。項目間移動に矢印キーは使わない（Radix Themes の RovingFocus は不採用、ark-ui も Space のみ）。",
        },
        KeyRow {
            key: "Space",
            description: "item（<label>）配下の checkbox::hidden_input（<input type=\"checkbox\">）が実体を担うため、ブラウザ標準のクリック委譲・トグル操作が働く（checkbox_group.rs:11-16, 266-284）。",
        },
        KeyRow {
            key: "Enter",
            description: "ネイティブ <input type=\"checkbox\"> は Enter でトグルしない（ブラウザ標準の挙動、フォーム送信のみに関与する）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "role=\"group\"",
            description: "root パーツへ固定付与する（checkbox_group.rs:232）。",
        },
        AriaRow {
            attribute: "aria-labelledby",
            description: "root の labelled_by が Some のときのみ付与する（checkbox_group.rs:236-238）。",
        },
        AriaRow {
            attribute: "（aria-orientation は付与しない）",
            description: "WAI-ARIA 1.2 の aria-orientation Used in Roles に role=\"group\" は含まれない（scrollbar/select/separator/slider/tablist/toolbar とその継承先のみ対象）。data-orientation は CSS セレクタが依存するため引き続き出力する（イシュー #1603 是正）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Color Picker
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/color_picker.rs:200-420`（root/trigger/
/// content/area_thumb/channel_slider_thumb の各パーツ関数）。
fn ex_color_picker() -> Node {
    let state = OpenState::Closed;
    let props = color_picker::ColorPickerProps::default();
    let body = vec![color_picker::root(
        state,
        &props,
        vec![],
        vec![
            color_picker::label(&props, vec![], vec![text("Accent color")]),
            color_picker::control(
                state,
                &props,
                vec![],
                vec![color_picker::trigger(
                    state,
                    &props,
                    Some("cp-content-2"),
                    vec![],
                    vec![text("#22c55e")],
                )],
            ),
            color_picker::positioner(
                state,
                vec![],
                vec![color_picker::content(
                    state,
                    Some("cp-content-2"),
                    vec![],
                    vec![color_picker::value_text(
                        &props,
                        vec![],
                        vec![text("#22c55e")],
                    )],
                )],
            ),
        ],
    )];
    wrap_example(
        "閉じた状態（aria-expanded=\"false\"）の Color Picker trigger/content の組み立て例です。",
        body,
    )
}

/// 自前 CSS の最小例（イシュー #1604、[`CHECKBOX_CUSTOM_CSS_SNIPPET`]/
/// `ex_checkbox_custom_css`〔#1602〕と同型のパターン）。CSS は
/// テキストノード（[`code`]/[`pre`]）として既定エスケープを経由し、
/// `crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは
/// 追加しない。
const COLOR_PICKER_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"color-picker\"][data-part=\"area\"] {\n  \
  position: relative;\n  width: 100%;\n  aspect-ratio: 4 / 3;\n\
}\n\
[data-scope=\"color-picker\"][data-part=\"area-thumb\"] {\n  \
  position: absolute;\n  left: var(--fandhe-color-picker-x, 0%);\n  \
  top: var(--fandhe-color-picker-y, 0%);\n\
}\n\
[data-scope=\"color-picker\"][data-part=\"hue-slider-track\"] {\n  \
  background: linear-gradient(to right, red, yellow, lime, cyan, blue, magenta, red);\n\
}\n\
[data-scope=\"color-picker\"][data-part=\"root\"][data-disabled] {\n  \
  opacity: 0.5;\n\
}\n\
[data-scope=\"color-picker\"][data-part=\"channel-input\"][data-invalid] {\n  \
  outline: 2px solid crimson;\n\
}\n\
[data-scope=\"color-picker\"][data-part=\"trigger\"][data-state=\"open\"] {\n  \
  outline: 2px solid #333;\n\
}\n";

fn ex_color_picker_custom_css() -> Node {
    let state = OpenState::Closed;
    let props = color_picker::ColorPickerProps::default();
    let markup = color_picker::root(
        state,
        &props,
        vec![],
        vec![
            color_picker::label(&props, vec![], vec![text("Accent color")]),
            color_picker::control(
                state,
                &props,
                vec![],
                vec![color_picker::trigger(
                    state,
                    &props,
                    None,
                    vec![],
                    vec![text("#22c55e")],
                )],
            ),
        ],
    );
    wrap_example(
        "利用者が data-scope / data-part / data-state / data-disabled / data-invalid 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
        vec![
            markup,
            pre(
                vec![],
                vec![code(vec![], vec![text(COLOR_PICKER_CUSTOM_CSS_SNIPPET)])],
            ),
        ],
    )
}

const COLOR_PICKER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "root/label/control/trigger/positioner/content/area/area_background/area_thumb/channel_slider(+track/+thumb)/channel_input/value_text/hidden_input の anatomy を持ち、Disclosure（開閉）+ HSV + アルファの値状態機械と組み合わせる（color_picker.rs:1-9）。",
        "ColorPickerProps（disabled/readonly/invalid/required）を root/label/control/trigger/area/area_background/area_thumb/channel_input へ一律付与し、label のみ data-required を追加する（イシュー #1604 参照突合、color_picker.rs:277-291）。",
        "trigger は type=\"button\" + aria-haspopup=\"dialog\" を固定付与し、controls が Some のとき aria-controls で content と関連付ける（color_picker.rs:426-452）。",
        "content（role=\"dialog\"）・area_thumb/channel_slider_thumb（role=\"slider\"）が WAI-ARIA の該当パターンへ従う（color_picker.rs:470-489, 524-547, 592-628）。",
        "channel_slider(+track/+thumb) は Channel::as_str() 固定語彙による data-channel と、Orientation 引数による data-orientation（thumb には aria-orientation も）を出力する（color_picker.rs:549-628）。",
        "channel_input は data-channel=\"hex\" 固定リテラルを出力し、readonly ネイティブ属性・aria-invalid（invalid のときのみ）を追加する（color_picker.rs:632-657）。",
        "increment/decrement dispatch（Channel 固定語彙 payload）は現在値を ±1 し 0..=Channel::max() へ clamp する（ラップしない、color_picker.rs:1049-1103）。",
        "色領域・スライダーの見た目は CSS グラデーションと決定的な導出 getter のみで表現し、canvas/web-sys へ一切依存しない（color_picker.rs:11-18）。",
    ],
    arguments: &[
        ArgRow {
            name: "root(state) / control(state) / trigger(state) / content(state)",
            kind: "OpenState",
            default: "OpenState::Closed",
            description: "開閉状態。data-state・aria-expanded・positioner の hidden 出力の判定に使われる（同じ型のため代表 1 行に集約）。",
        },
        ArgRow {
            name: "root/label/control/trigger/area/area_background/area_thumb/channel_input(props)",
            kind: "&ColorPickerProps",
            default: "&ColorPickerProps::default()",
            description: "disabled/readonly/invalid を一律付与する（label のみ追加で required→data-required）。",
        },
        ArgRow {
            name: "trigger(controls)",
            kind: "Option<&str>",
            default: "None",
            description: "Some のとき aria-controls で content と関連付ける（color_picker.rs:439-441）。",
        },
        ArgRow {
            name: "area_thumb(hex)",
            kind: "&str",
            default: "",
            description: "2 次元 slider（role=\"slider\"）の aria-valuetext。現在色の HEX 正規形（color_picker.rs:524-537）。",
        },
        ArgRow {
            name: "channel_slider/channel_slider_track/channel_slider_thumb(channel, orientation)",
            kind: "Channel, Orientation",
            default: "",
            description: "data-channel（固定語彙）と data-orientation（thumb には aria-orientation も）を出力する（color_picker.rs:549-628）。",
        },
        ArgRow {
            name: "channel_slider_thumb(min, max, now)",
            kind: "&str, &str, &str",
            default: "",
            description: "WAI-ARIA slider パターンの aria-valuemin/aria-valuemax/aria-valuenow（color_picker.rs:592-628）。",
        },
        ArgRow {
            name: "hidden_input(name, value, props)",
            kind: "&str, &str, &ColorPickerProps",
            default: "",
            description: "フォーム送信用の実体（type=\"hidden\"、props.disabled のときのみ disabled 属性、required は付けない、color_picker.rs:676-691）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Closed trigger",
            description: "aria-expanded=\"false\" の閉じた Color Picker trigger/content の例です。",
            render: ex_color_picker,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "headless-ui 自体はスタイルを持たないため、data-* 属性セレクタで見た目を組み立てる最小例です。",
            render: ex_color_picker_custom_css,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "Enter",
            description: "trigger にフォーカス時: popover を開く（dispatch \"open\"）。channel_input にフォーカス時: HEX 文字列を確定する（dispatch \"set_hex\"）。",
        },
        KeyRow {
            key: "Esc",
            description: "content が開いているとき popover を閉じる（dispatch \"close\"）。trigger へのフォーカス復帰は呼び出し側の DOM 配線が担う（スコープ外、モジュール doc「参照突合」節参照）。",
        },
        KeyRow {
            key: "ArrowLeft / ArrowRight",
            description: "channel_slider_thumb にフォーカス時: 該当チャンネルを 1 減らす/増やす（dispatch \"decrement\"/\"increment\"、0..=Channel::max() へ clamp、ラップしない）。",
        },
        KeyRow {
            key: "ArrowUp / ArrowDown",
            description: "area_thumb にフォーカス時: 彩度・明度をそれぞれ 1 単位変化させる（dispatch \"set_channel\" で表現）。",
        },
        KeyRow {
            key: "Home / End",
            description: "channel_slider_thumb にフォーカス時: 最小値・最大値へ設定する（dispatch \"set_channel\" の 0/最大値ペイロードで表現、専用 dispatch は追加しない）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "aria-haspopup=\"dialog\"",
            description: "trigger パーツへ固定付与する（color_picker.rs:435）。",
        },
        AriaRow {
            attribute: "aria-expanded",
            description: "trigger の開閉状態（state.is_open()）を反映する（color_picker.rs:436）。",
        },
        AriaRow {
            attribute: "role=\"dialog\"",
            description: "content パーツへ固定付与する（color_picker.rs:478）。",
        },
        AriaRow {
            attribute: "role=\"slider\" / aria-valuetext",
            description: "area_thumb・channel_slider_thumb の両方が role=\"slider\" を固定付与する（area_thumb は aria-label=\"Color\" + aria-valuetext のみ、channel_slider_thumb は aria-valuemin/max/now + aria-orientation も付与、color_picker.rs:524-547, 592-628）。",
        },
        AriaRow {
            attribute: "aria-invalid=\"true\"",
            description: "channel_input が props.invalid のときのみ付与する（valid のときは属性自体を省略、color_picker.rs:632-657）。",
        },
        AriaRow {
            attribute: "aria-disabled=\"true\"",
            description: "area_thumb・channel_slider_thumb が props.disabled のとき tabindex=\"-1\" と対で付与する（color_picker.rs:524-547, 592-628）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Combobox
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/combobox.rs:112-398`（root/input/
/// trigger/content/item の各パーツ関数、`filter_options`）。
fn ex_combobox() -> Node {
    let state = OpenState::Closed;
    let props = ComboboxProps::default();
    let body = vec![combobox::root(
        state,
        &props,
        vec![],
        vec![
            combobox::label(
                &props,
                Some("cb2-label"),
                Some("cb2-input"),
                vec![],
                vec![text("Language")],
            ),
            combobox::control(
                state,
                &props,
                vec![],
                vec![
                    combobox::input(
                        state,
                        "",
                        &props,
                        None,
                        None,
                        None,
                        vec![("id", "cb2-input")],
                    ),
                    combobox::trigger(state, &props, None, vec![], vec![text("▾")]),
                ],
            ),
        ],
    )];
    wrap_example(
        "閉じた状態（aria-expanded=\"false\"）で aria-autocomplete=\"list\" を持つ Combobox input/trigger の組み立て例です。",
        body,
    )
}

/// `ComboboxProps` の `data-*` 状態を利用者スタイルで消費する自前 CSS 例
/// （イシュー #1605 参照突合。[`ex_checkbox_custom_css`] と同型。CSS は
/// `pre`/`code` のテキストノードとして既定エスケープ経由で出力するのみで、
/// `primitives-showcase.css` へは一切追加しない）。closed 状態（`content` を
/// 描画しない）で組み、`combobox_aria_association` 契約の対象外にする。
const COMBOBOX_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"combobox\"][data-part=\"root\"][data-disabled] {\n  \
  opacity: 0.5;\n\
}\n\
[data-scope=\"combobox\"][data-part=\"input\"][data-invalid] {\n  \
  border-color: red;\n\
}\n\
[data-scope=\"combobox\"][data-part=\"item\"][data-highlighted] {\n  \
  background: #eef;\n\
}\n\
[data-scope=\"combobox\"][data-part=\"item\"][data-state=\"open\"] {\n  \
  font-weight: bold;\n\
}\n\
[data-scope=\"combobox\"][data-part=\"positioner\"][hidden] {\n  \
  display: none;\n\
}\n";

fn ex_combobox_custom_css() -> Node {
    let state = OpenState::Closed;
    let props = ComboboxProps::default();
    let markup = combobox::root(
        state,
        &props,
        vec![],
        vec![combobox::control(
            state,
            &props,
            vec![],
            vec![
                combobox::input(state, "", &props, None, None, None, vec![]),
                combobox::trigger(state, &props, None, vec![], vec![text("▾")]),
            ],
        )],
    );
    wrap_example(
        "利用者が data-scope / data-part / data-state / data-invalid / data-highlighted 属性セレクタで自前 CSS を当てる最小例です（本 Demo には適用されません）。",
        vec![
            markup,
            pre(vec![], vec![code(vec![], vec![text(COMBOBOX_CUSTOM_CSS_SNIPPET)])]),
        ],
    )
}

const COMBOBOX: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "root/label/control/input/trigger/clear_trigger/positioner/content/item_group/item_group_label/item/item_text/item_indicator/live_region の 14 anatomy パーツを持ち、フォーカスを保持する input（role=\"combobox\"）側に aria-activedescendant を配線する（crate::select と異なる配線先、combobox.rs:59-65）。",
        "ComboboxProps（disabled/readonly/invalid/required）を root/label/control/input/trigger/clear_trigger へ一律付与し、data-required は label にのみ、input は対応するネイティブ disabled/readonly/required 存在属性 + invalid 時のみ aria-invalid=\"true\" を追加する（イシュー #1605 参照突合。ark-ui の data-focus/data-placement/data-empty、item/item-indicator の data-state=\"checked\"/\"unchecked\" 語彙は crate::select/crate::listbox とのクレート横断整合を優先し意図的に追随しない）。",
        "input は role=\"combobox\" + aria-autocomplete=\"list\" を固定付与し、controls/activedescendant が Some のときのみ aria-controls/aria-activedescendant を付与する（combobox.rs:152-205）。",
        "trigger は type=\"button\" + aria-haspopup=\"listbox\" を固定付与する（combobox.rs:205-238）。",
        "候補データの絞り込みは filter_options（純粋関数）が提供するが、候補データ自体の取得・供給は利用者側の責務である（combobox.rs:398、`docs/policy/intentional-non-adoption.md` §3.25）。",
        "live_region は候補件数の変化を通知する live region（role=\"status\" + aria-live=\"polite\" 固定、root の直接の子で control の兄弟として配置する。テキスト更新の実配線は fandhe-frontend-wasm-full の後続責務、イシュー #1069）。",
    ],
    arguments: &[
        ArgRow {
            name: "root(state) / control(state) / input(state) / trigger(state) / content(state)",
            kind: "OpenState",
            default: "OpenState::Closed",
            description: "開閉状態。data-state・aria-expanded の判定に使われる（同じ型のため代表 1 行に集約）。",
        },
        ArgRow {
            name: "root(props) / label(props) / control(props) / input(props) / trigger(props) / clear_trigger(props)",
            kind: "&ComboboxProps",
            default: "&ComboboxProps::default()",
            description: "disabled/readonly/invalid/required の状態束（代表 1 行に集約、イシュー #1605）。",
        },
        ArgRow {
            name: "input(value, controls, activedescendant, name)",
            kind: "&str, Option<&str>, Option<&str>, Option<&str>",
            default: "",
            description: "現在の入力文字列・content の id・ハイライト中 item の id・関連フォームフィールド名（combobox.rs、イシュー #1605 で disabled: bool を props へ置換）。",
        },
        ArgRow {
            name: "item(state, disabled, highlighted, value, id)",
            kind: "OpenState, bool, bool, &str, Option<&str>",
            default: "",
            description: "role=\"option\" を持つ選択肢 1 個の選択状態・無効化・ハイライト中かどうか。value は data-value として既定エスケープ経由で出力される（combobox.rs:333-361）。",
        },
        ArgRow {
            name: "live_region(children)",
            kind: "Vec<Node>",
            default: "",
            description: "role=\"status\"/aria-live=\"polite\"/aria-atomic=\"true\" を固定付与する live region。通知文言は children として呼び出し側が渡す（combobox.rs、イシュー #1069）。",
        },
        ArgRow {
            name: "attrs / children",
            kind: "Vec<(&str, &str)> / Vec<Node>",
            default: "",
            description: "各パーツ共通の追加属性・子ノード（代表 1 行に集約）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Closed input + trigger",
        description: "aria-expanded=\"false\"/aria-autocomplete=\"list\" を持つ閉じた状態の例です。",
        render: ex_combobox,
    }, ExampleEntry {
        title: "利用者スタイルの当て方",
        description: "data-scope / data-part / data-state / data-invalid / data-highlighted 属性セレクタで自前 CSS を当てる最小例です。",
        render: ex_combobox_custom_css,
    }],
    // `fandhe-frontend-wasm-full` の `keynav::combobox_key_action`
    // （イシュー #1071、`crates/wasm-full/src/keynav.rs` 判定表が一次情報）
    // が実配線する keydown 判定を掲載する。ArrowLeft/ArrowRight/Tab・
    // printable 文字は入力欄のネイティブ既定動作（キャレット移動・
    // フィルタ入力・タブ移動）を奪わないため claim しない。
    keyboard: &[
        KeyRow {
            key: "ArrowDown",
            description: "closed: 先頭の非 disabled 候補を highlight して開く。open: 次の候補へ highlight を移動する（keynav.rs combobox_key_action）。",
        },
        KeyRow {
            key: "ArrowUp",
            description: "closed: 末尾の非 disabled 候補を highlight して開く。open: 前の候補へ highlight を移動する（keynav.rs combobox_key_action）。",
        },
        KeyRow {
            key: "Home / End",
            description: "closed: no-op（キャレット移動という input のネイティブ既定動作を奪わない）。open: 先頭/末尾の非 disabled 候補へ highlight を移動する。",
        },
        KeyRow {
            key: "Enter",
            description: "closed: no-op（フォーム submit 等のネイティブ既定動作を奪わない）。open: highlight 中候補を確定選択する。",
        },
        KeyRow {
            key: "Escape",
            description: "closed: no-op（fail-closed。claim すると誤って open してしまうため）。open: highlight をクリアして閉じる。",
        },
        KeyRow {
            key: "Tab / ArrowLeft / ArrowRight / printable 文字",
            description: "claim しない（Tab はフォーカス移動、ArrowLeft/ArrowRight はキャレット移動、printable 文字はフィルタ入力というネイティブ input の既定動作をそのまま許容する）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "role=\"combobox\" / aria-autocomplete=\"list\"",
            description: "input パーツへ固定付与する（combobox.rs:177以降）。",
        },
        AriaRow {
            attribute: "aria-expanded",
            description: "input・trigger の両方が開閉状態を反映する。",
        },
        AriaRow {
            attribute: "aria-activedescendant",
            description: "input パーツ側へ配線する（crate::select と異なり本モジュールは input 側、combobox.rs:59-65, 156）。",
        },
        AriaRow {
            attribute: "aria-invalid=\"true\"",
            description: "input パーツが props.invalid のときのみ付与する（valid のときは属性自体を省略、イシュー #1605）。",
        },
        AriaRow {
            attribute: "role=\"listbox\" / role=\"option\"",
            description: "content パーツは role=\"listbox\"、item パーツは role=\"option\" を固定付与する（combobox.rs:277, 343）。",
        },
        AriaRow {
            attribute: "role=\"status\" / aria-live=\"polite\" / aria-atomic=\"true\" (live_region)",
            description: "候補件数の変化を通知する live region に固定付与する（combobox.rs、イシュー #1069）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Editable
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/editable.rs:184-513`（root/input/
/// edit_trigger/submit_trigger/cancel_trigger の各パーツ関数）。
fn ex_editable() -> Node {
    let mode = EditMode::Edit;
    let body = vec![editable::root(
        mode,
        EditableInputFlags::default(),
        EditableActivationMode::Focus,
        EditableSubmitMode::Both,
        vec![],
        vec![
            editable::label(
                mode,
                EditableInputFlags::default(),
                Some("ed2-input"),
                vec![],
                vec![text("Nickname")],
            ),
            editable::area(
                mode,
                EditableInputFlags::default(),
                false,
                vec![],
                vec![
                    editable::preview(
                        mode,
                        EditableInputFlags::default(),
                        false,
                        vec![],
                        vec![text("grace")],
                    ),
                    editable::input(
                        mode,
                        "nickname",
                        "grace",
                        EditableInputProps {
                            id: Some("ed2-input"),
                            placeholder: None,
                            max_length: Some("20"),
                        },
                        EditableInputFlags::default(),
                        vec![],
                    ),
                ],
            ),
            editable::control(
                mode,
                vec![],
                vec![
                    editable::submit_trigger(mode, false, vec![], vec![text("Save")]),
                    editable::cancel_trigger(mode, false, vec![], vec![text("Cancel")]),
                ],
            ),
        ],
    )];
    wrap_example(
        "edit モード（maxlength=\"20\" 付き）の Editable の組み立て例です。",
        body,
    )
}

/// 自前 CSS の最小例（イシュー #1606、[`CHECKBOX_CUSTOM_CSS_SNIPPET`] と
/// 同型）。`preview`/`input`/`root` の `data-*` セレクタで見た目を組み立てる
/// 例を示す。CSS はテキストノード（[`code`]/[`pre`]）として既定エスケープを
/// 経由し、`primitives-showcase.css` へは追加しない。
const EDITABLE_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"editable\"][data-part=\"preview\"] {\n  \
  cursor: text;\n  border-bottom: 1px dashed transparent;\n\
}\n\
[data-scope=\"editable\"][data-part=\"preview\"]:focus-visible {\n  \
  outline: 2px solid #2563eb;\n  outline-offset: 2px;\n\
}\n\
[data-scope=\"editable\"][data-part=\"preview\"][data-placeholder-shown] {\n  \
  color: #9ca3af;\n  font-style: italic;\n\
}\n\
[data-scope=\"editable\"][data-part=\"input\"][data-invalid] {\n  \
  border-color: #dc2626;\n\
}\n\
[data-scope=\"editable\"][data-part=\"root\"][data-disabled] {\n  \
  opacity: 0.5;\n\
}\n";

fn ex_editable_custom_css() -> Node {
    let mode = EditMode::Preview;
    let markup = editable::root(
        mode,
        EditableInputFlags::default(),
        EditableActivationMode::default(),
        EditableSubmitMode::default(),
        vec![],
        vec![
            editable::area(
                mode,
                EditableInputFlags::default(),
                false,
                vec![],
                vec![editable::preview(
                    mode,
                    EditableInputFlags::default(),
                    false,
                    vec![],
                    vec![text("Ada Lovelace")],
                )],
            ),
            editable::control(
                mode,
                vec![],
                vec![editable::edit_trigger(
                    mode,
                    false,
                    vec![],
                    vec![text("Edit")],
                )],
            ),
        ],
    );
    wrap_example(
        "利用者が data-scope / data-part / data-state / data-invalid / data-disabled / data-placeholder-shown 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
        vec![
            markup,
            pre(vec![], vec![code(vec![], vec![text(EDITABLE_CUSTOM_CSS_SNIPPET)])]),
        ],
    )
}

const EDITABLE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "root/label/area/input/preview/control/edit_trigger/submit_trigger/cancel_trigger の anatomy を持ち、EditMode（Preview/Edit）で表示専用と編集可能を切り替える（editable.rs:163-180）。",
        "input（<input type=\"text\">）は preview モード時に hidden、preview（<span>）は edit モード時に hidden を出力する（全パーツを DOM に掲載し hidden で切り替える方針、editable.rs:349-463）。",
        "edit_trigger/submit_trigger/cancel_trigger はいずれも <button type=\"button\"> で、表示は現在モードに応じて hidden 切り替えされる（editable.rs:445-513）。",
        "activationMode/submitMode の実際の DOM 配線（focus/dblclick 起動・Enter/Escape/blur）は wasm-full 側の後続責務であり、本モジュールは data-activation-mode/data-submit-mode という SSR 静的ヒントのみを提供する（editable.rs module doc「スコープ外」節）。",
        "イシュー #1606 で ark-ui との参照突合を行い、EditableInputFlags（disabled/readonly/required/invalid）を root/label/area/preview/input へ共通で渡す設計へ揃え、label へ data-invalid/data-required、area へ data-disabled、preview へ data-disabled/data-readonly/data-invalid・aria-disabled/aria-invalid（該当時のみ）・tabindex=\"0\"（!disabled && !readonly のときのみ）を追加した。activationMode に Click/None、submitMode に None を追加した。",
        "data-focus（area/label）・data-autoresize（input/preview）・aria-readonly（preview）は意図的に出力しない（data-focus は SSR 静的マークアップで表現できない実行時フォーカス状態、data-autoresize はレイアウト計測関心のため headless 層の対象外、aria-readonly は role なし span への付与が ARIA in HTML 上不正なため）。",
    ],
    arguments: &[
        ArgRow {
            name: "root(mode, flags, activation_mode, submit_mode)",
            kind: "EditMode, EditableInputFlags, EditableActivationMode, EditableSubmitMode",
            default: "",
            description: "現在モードと、起動・確定方式の SSR 静的ヒント。flags のうち disabled/readonly のみ root へ反映する（editable.rs:248-269）。",
        },
        ArgRow {
            name: "label(mode, flags, input_id)",
            kind: "EditMode, EditableInputFlags, Option<&str>",
            default: "",
            description: "input との for/id 関連付け。flags の disabled/invalid/required を data-disabled/data-invalid/data-required へ反映する（editable.rs:273-296）。",
        },
        ArgRow {
            name: "area(mode, flags, placeholder_shown)",
            kind: "EditMode, EditableInputFlags, bool",
            default: "",
            description: "input/preview のラッパー。flags の disabled を data-disabled へ反映する（editable.rs:298-315）。",
        },
        ArgRow {
            name: "preview(mode, flags, placeholder_shown)",
            kind: "EditMode, EditableInputFlags, bool",
            default: "",
            description: "flags の disabled/readonly/invalid を data-* へ、disabled/invalid を aria-disabled/aria-invalid（該当時のみ）へ反映し、!disabled && !readonly のとき tabindex=\"0\" を付与する（editable.rs:405-441）。",
        },
        ArgRow {
            name: "input(mode, name, value, props, flags)",
            kind: "EditMode, &str, &str, EditableInputProps, EditableInputFlags",
            default: "",
            description: "<input type=\"text\"> の name/value と id/placeholder/maxlength・disabled/readonly/required・invalid（invalid 時のみ aria-invalid=\"true\" を追加、editable.rs:349-403）。",
        },
        ArgRow {
            name: "EditableInputFlags { disabled, readonly, required, invalid }",
            kind: "bool ×4",
            default: "false ×4",
            description: "root/label/area/preview/input へ共通で渡すフラグ束。反映先パーツはパートごとに異なる（上記各行参照、editable.rs:317-346）。",
        },
        ArgRow {
            name: "edit_trigger(mode, disabled) / submit_trigger(mode, disabled) / cancel_trigger(mode, disabled)",
            kind: "EditMode, bool",
            default: "",
            description: "各トリガー（<button type=\"button\">）の表示モード切り替えと無効化（editable.rs:445-513、同型の 3 関数のため 1 行に集約）。",
        },
        ArgRow {
            name: "attrs / children",
            kind: "Vec<(&str, &str)> / Vec<Node>",
            default: "",
            description: "各パーツ共通の追加属性・子ノード（代表 1 行に集約）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Edit mode",
            description: "maxlength 付きの edit モード（submit/cancel トリガー表示）の例です。",
            render: ex_editable,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "data-* 属性セレクタで見た目を組み立てる最小例です。",
            render: ex_editable_custom_css,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "Enter（edit 中の input）",
            description: "dispatch(\"submit\") に対応する。submit_mode が Enter/Both のときのみ確定して preview へ戻る想定（DOM 配線は wasm-full 側の後続責務、editable.rs module doc「キーボード操作」節参照）。",
        },
        KeyRow {
            key: "Escape（edit 中の input）",
            description: "dispatch(\"cancel\") に対応する。activation_mode/submit_mode に関わらず常に取消可能（editable.rs module doc「キーボード操作」節参照）。",
        },
        KeyRow {
            key: "Tab（preview、activation_mode=\"focus\"）",
            description: "preview が tabindex=\"0\"（!disabled && !readonly のときのみ付与）を持つため、キーボードで到達できる（editable.rs:405-441）。",
        },
        KeyRow {
            key: "Space / Enter（各 trigger）",
            description: "edit_trigger/submit_trigger/cancel_trigger は <button type=\"button\"> のためネイティブの Space/Enter による活性化が働く（editable.rs:445-513）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "aria-invalid=\"true\"（input・preview）",
            description: "flags.invalid が true のときのみ付与する（false 時は属性自体を省略、editable.rs:349-403, 405-441）。",
        },
        AriaRow {
            attribute: "aria-disabled=\"true\"（preview）",
            description: "flags.disabled が true のときのみ付与する（input は代わりにネイティブ disabled 属性を使う、editable.rs:405-441）。",
        },
        AriaRow {
            attribute: "for / id（label ↔ input）",
            description: "label(mode, flags, input_id) の input_id と input(..., props) の props.id を同じ文字列にして関連付ける。",
        },
        AriaRow {
            attribute: "tabindex=\"0\"（preview）",
            description: "!disabled && !readonly のときのみ付与する（Zag isInteractive と同義、editable.rs:405-441）。",
        },
        AriaRow {
            attribute: "hidden（input・preview・edit_trigger・submit_trigger・cancel_trigger）",
            description: "現在モードに応じて排他表示する存在属性（preview モードで input が hidden、edit モードで preview が hidden、等）。",
        },
        AriaRow {
            attribute: "aria-label 等の翻訳文言",
            description: "利用者が attrs 経由で渡す方針（headless-ui は文言を持たない）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Field
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/field.rs:249-410`（root/label/input/
/// helper_text/error_text の各パーツ関数、`aria-describedby` 合成則）。
fn ex_field() -> Node {
    let props = FieldProps {
        id: "f2-username",
        ids: Default::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: true,
    };
    let body = vec![field::root(
        &props,
        vec![],
        vec![
            field::label(
                &props,
                vec![],
                vec![
                    text("Username"),
                    field::required_indicator(&props, vec![], vec![text("*")]),
                ],
            ),
            field::input(
                &props,
                vec![("type", "text"), ("name", "username"), ("value", "")],
            ),
            field::helper_text(&props, vec![], vec![text("Letters and digits only.")]),
            field::error_text(&props, vec![], vec![text("Username is required.")]),
        ],
    )];
    wrap_example(
        "required かつ helper_text 併用の Field（バリデーション結果は利用者が渡す）の組み立て例です。",
        body,
    )
}

/// 自前 CSS の最小例。headless-ui 自体はスタイルを持たないため、利用者が
/// `data-scope`/`data-part`/`data-invalid`/`data-disabled`/`data-required`
/// 属性セレクタで見た目を組み立てる例を示す（イシュー #1607）。CSS は
/// テキストノード（[`code`]/[`pre`]）として既定エスケープを経由し、
/// `crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは
/// 追加しない。
const FIELD_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"field\"][data-part=\"input\"] {\n  \
  border: 1px solid #888;\n  border-radius: 4px;\n  padding: 0.375rem 0.5rem;\n\
}\n\
[data-scope=\"field\"][data-part=\"input\"][data-invalid] {\n  \
  border-color: #dc2626;\n\
}\n\
[data-scope=\"field\"][data-part=\"root\"][data-disabled] {\n  \
  opacity: 0.5;\n\
}\n\
[data-scope=\"field\"][data-part=\"error-text\"] {\n  \
  color: #dc2626;\n  font-size: 0.875rem;\n\
}\n\
[data-scope=\"field\"][data-part=\"label\"][data-required]::after {\n  \
  content: \" *\";\n  color: #dc2626;\n\
}\n";

fn ex_field_custom_css() -> Node {
    let props = FieldProps {
        id: "f3-username-custom-css",
        ids: Default::default(),
        disabled: false,
        invalid: true,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let markup = field::root(
        &props,
        vec![],
        vec![
            field::label(&props, vec![], vec![text("Username")]),
            field::input(
                &props,
                vec![("type", "text"), ("name", "username"), ("value", "")],
            ),
            field::error_text(&props, vec![], vec![text("Username is required.")]),
        ],
    );
    wrap_example(
        "利用者が data-scope / data-part / data-invalid / data-disabled / data-required 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
        vec![
            markup,
            pre(vec![], vec![code(vec![], vec![text(FIELD_CUSTOM_CSS_SNIPPET)])]),
        ],
    )
}

const FIELD: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "root/label/input/textarea/select/helper_text/error_text/required_indicator の 8 パーツ anatomy を持ち、1 個のコントロールへ label・helper_text・error_text を一貫して結び付ける（ark-ui Field と一致。field.rs:1-10, 54）。",
        "disabled/invalid/required/readonly の 4 種の data-* 存在属性を 8 パーツすべてへ一貫して付与する（zag.js Field の dataAttrs 規約と一致、field.rs::state_data_attrs）。data-state/data-orientation/data-motion 等の局所操作状態・レイアウト計測の関心は出力しない（chakra-ui の orientation バリアントは styled 層の関心として Themes 側の検討事項、`docs/policy/intentional-non-adoption.md` §3.25 規則 2）。Radix Form.Field の data-valid も Form 自体の不採用（同 §3.25 規則 1）に伴い導入しない。",
        "aria-describedby は invalid のとき error id を先頭に、has_helper_text のとき helper id を続けて空白区切りで決定的に合成する（field.rs:249-258, 305-311）。",
        "invalid のとき input/textarea/select へ aria-invalid=\"true\" を付与し、error_text を表示状態にする（field.rs:118-119）。",
        "select はネイティブ readonly を出力しない（HTML 仕様上 <select readonly> が無効なため、data-readonly は他コントロール同様に出力する、field.rs:60-63）。バリデーションの実行自体は利用者側の通常の Rust コードが担い、本モジュールはその結果（invalid/エラーメッセージ）を構造・ARIA へ反映するのみである（`docs/policy/intentional-non-adoption.md` §3.25）。",
        "error_text/required_indicator は非該当状態で hidden 存在属性を付与する fail-closed 描画とする（ark-ui の条件描画を SSR で静的に写像したもの、field.rs::error_text/required_indicator）。has_helper_text は zag.js の MutationObserver による動的検知を静的フラグへ写像したものであり、呼び出し側は実際に helper_text パーツを描画するかどうかと整合させる契約を持つ。",
    ],
    arguments: &[
        ArgRow {
            name: "props.id",
            kind: "&str",
            default: "",
            description: "ベース id。control/label/helper_text/error_text の派生 id 生成に使う（field.rs:103-108）。",
        },
        ArgRow {
            name: "props.disabled / props.invalid / props.required / props.readonly",
            kind: "bool",
            default: "false",
            description: "FieldProps の各フラグ（field.rs:112-135）。data-*・ネイティブ属性・aria-invalid へ反映される（代表 1 行に集約）。",
        },
        ArgRow {
            name: "props.has_helper_text",
            kind: "bool",
            default: "false",
            description: "aria-describedby の合成に helper id を含めるかどうか（field.rs:249-258）。",
        },
        ArgRow {
            name: "input(props, extra_attrs) / textarea(props, autoresize, extra_attrs, children) / select(props, extra_attrs, children)",
            kind: "&FieldProps, ..",
            default: "",
            description: "同一の aria-describedby/aria-invalid 合成則に従う 3 種のコントロールパーツ（field.rs:311-374）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Required + helper text",
            description: "required_indicator・helper_text・error_text を併用した例です。",
            render: ex_field,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "data-scope / data-part / data-invalid / data-disabled / data-required 属性セレクタで見た目を組み立てる最小例です。",
            render: ex_field_custom_css,
        },
    ],
    keyboard: &[KeyRow {
        key: "Tab / 文字入力 / (select) ↑↓ Space Enter",
        description: "input/textarea/select がいずれもネイティブ要素であるため、ブラウザ標準のフォーカス移動・入力操作が働く（field.rs:311-374）。select は ↑/↓/Space/Enter によるネイティブの開閉・選択が働く。参照サイト（ark-ui / chakra-ui）の Field も独自のキー操作を持たない。",
    }],
    aria: &[
        AriaRow {
            attribute: "aria-describedby",
            description: "invalid のとき error id、has_helper_text のとき helper id を空白区切りで合成する（field.rs:249-258）。",
        },
        AriaRow {
            attribute: "aria-invalid",
            description: "props.invalid が true のとき input/textarea/select へ \"true\" を付与する（field.rs:118-119）。",
        },
        AriaRow {
            attribute: "aria-live=\"polite\"",
            description: "error_text パーツへ固定付与する（field.rs:385-387）。",
        },
        AriaRow {
            attribute: "aria-hidden=\"true\"",
            description: "required_indicator パーツへ固定付与する（装飾目的の印のため、field.rs:399-402）。",
        },
        AriaRow {
            attribute: "label[for] / control id",
            description: "label の for と input/textarea/select の id は同一 props から決定的に対応する（ids 上書き時も一貫して伝播する、field.rs::control_id/label_id）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Fieldset
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/fieldset.rs:148-215`（root/legend/
/// helper_text/error_text の各パーツ関数、`aria-describedby` 合成則）。
fn ex_fieldset() -> Node {
    let props = FieldsetProps {
        id: "fs2-payment",
        disabled: false,
        invalid: true,
        has_helper_text: false,
    };
    let body = vec![fieldset::root(
        &props,
        vec![],
        vec![
            fieldset::legend(&props, vec![], vec![text("Payment method")]),
            fieldset::error_text(&props, vec![], vec![text("Select a payment method.")]),
        ],
    )];
    wrap_example(
        "invalid な Fieldset（aria-describedby が error_text のみを指す例）の組み立て例です。",
        body,
    )
}

const FIELDSET: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "root（<fieldset>）/legend（<legend>）/helper_text/error_text の 4 パーツで複数の crate::field をグループ化する（fieldset.rs:1-9）。",
        "root の disabled はネイティブ <fieldset disabled> として出力し、HTML 仕様により内側の全コントロールが自動的に無効化される（fieldset.rs:8-9, 148-167）。",
        "aria-describedby は invalid のとき error id、has_helper_text のとき helper id を空白区切りで合成する（field.rs と同型の合成則、fieldset.rs:123-146）。",
        "invalid はグループ全体のみに反映され、個別 Field の aria-invalid へは伝播しない（誤ったコントロール単位のエラー通知を避けるための意図的な判断、fieldset.rs:69-72）。",
    ],
    arguments: &[
        ArgRow {
            name: "props.id",
            kind: "&str",
            default: "",
            description: "ベース id。legend/helper_text/error_text の派生 id 生成に使う（fieldset.rs:60-62）。",
        },
        ArgRow {
            name: "props.disabled",
            kind: "bool",
            default: "false",
            description: "true のとき root（<fieldset>）へネイティブ disabled + data-disabled を付与する（fieldset.rs:63-66）。",
        },
        ArgRow {
            name: "props.invalid",
            kind: "bool",
            default: "false",
            description: "true のとき root へ data-invalid を付与し、error_text を表示状態にする（fieldset.rs:67-72）。",
        },
        ArgRow {
            name: "props.has_helper_text",
            kind: "bool",
            default: "false",
            description: "aria-describedby の合成に helper id を含めるかどうか（fieldset.rs:73-74, 123-146）。",
        },
        ArgRow {
            name: "merge_field_props(field)",
            kind: "FieldProps -> FieldProps",
            default: "",
            description: "Fieldset の disabled を内包する FieldProps へ OR 伝播する（invalid は伝播しない、fieldset.rs:106-121）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Invalid group",
        description: "invalid な Fieldset と error_text の組み立て例です。",
        render: ex_fieldset,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-describedby",
            description: "root パーツへ、invalid のとき error id・has_helper_text のとき helper id を合成して付与する（fieldset.rs:123-146）。",
        },
        AriaRow {
            attribute: "aria-live=\"polite\"",
            description: "error_text パーツへ固定付与する（fieldset.rs:195-198）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// File Upload
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/file_upload.rs:226-388`（root/dropzone/
/// item_delete_trigger/hidden_input の各パーツ関数）。
fn ex_file_upload() -> Node {
    let body = vec![file_upload::root(
        false,
        vec![],
        vec![
            file_upload::label(vec![], vec![text("Resume")]),
            file_upload::dropzone(
                false,
                false,
                vec![("aria-label", "Drop your resume here")],
                vec![
                    text("Drag & drop or"),
                    file_upload::trigger(false, vec![], vec![text("Choose file")]),
                    file_upload::hidden_input("application/pdf", false, false, vec![]),
                ],
            ),
        ],
    )];
    wrap_example(
        "role=\"button\" のドロップゾーンと単一ファイル選択 hidden_input の組み立て例です。",
        body,
    )
}

const FILE_UPLOAD: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "root/label/dropzone/trigger/item_group/item/item_name/item_size_text_node/item_delete_trigger/clear_trigger/hidden_input の 11 anatomy パーツで構成する（file_upload.rs:1-9）。",
        "dropzone は role=\"button\" + tabindex=\"0\" でフォーカス可能にし、呼び出し側が attrs 経由で aria-label を与える（file_upload.rs:240-246）。",
        "item_delete_trigger の aria-label は「Delete {name}」を動的に組み立てるが、既定エスケープを経由するため注入経路にはならない（file_upload.rs:304-308）。",
        "本モジュールはファイルメタデータ（name/size_bytes/mime_type）のみを保持し、File オブジェクト自体・実アップロード処理は持たない（file_upload.rs:20-27）。",
    ],
    arguments: &[
        ArgRow {
            name: "root(disabled)",
            kind: "bool",
            default: "false",
            description: "root へ data-disabled を反映するかどうか。",
        },
        ArgRow {
            name: "dropzone(disabled, dragging)",
            kind: "bool, bool",
            default: "false",
            description: "role=\"button\"/tabindex=\"0\" 固定。dragging は data-dragging（wasm-full 側が DOM ローカルにトグルする想定、file_upload.rs:240-256）。",
        },
        ArgRow {
            name: "item_delete_trigger(name, disabled)",
            kind: "&str, bool",
            default: "",
            description: "aria-label=\"Delete {name}\" を動的に組み立てる（file_upload.rs:304-318）。",
        },
        ArgRow {
            name: "hidden_input(accept, multiple, disabled)",
            kind: "&str, bool, bool",
            default: "",
            description: "<input type=\"file\">。accept/multiple はネイティブ属性として反映される（file_upload.rs:340-363）。",
        },
        ArgRow {
            name: "attrs / children",
            kind: "Vec<(&str, &str)> / Vec<Node>",
            default: "",
            description: "各パーツ共通の追加属性・子ノード（代表 1 行に集約）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Single file dropzone",
        description: "PDF のみを受け付ける単一ファイル選択の例です。",
        render: ex_file_upload,
    }],
    keyboard: &[KeyRow {
        key: "Space / Enter",
        description: "dropzone は role=\"button\" + tabindex=\"0\" のためフォーカス可能で、trigger/item_delete_trigger/clear_trigger はいずれも <button> であるため、ブラウザ標準の活性化操作が働く（file_upload.rs:240-262, 308-326）。",
    }],
    aria: &[
        AriaRow {
            attribute: "role=\"button\"",
            description: "dropzone パーツへ固定付与する（file_upload.rs:252）。",
        },
        AriaRow {
            attribute: "aria-label",
            description: "item_delete_trigger パーツへ「Delete {name}」を動的に組み立てて付与する（file_upload.rs:304-318）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Image Cropper
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/image_cropper.rs:438-490`（root/handle
/// の各パーツ関数）。
fn ex_image_cropper() -> Node {
    let body = vec![image_cropper::root(
        vec![],
        vec![
            image_cropper::viewport(
                vec![],
                vec![image_cropper::image(
                    "https://example.com/portrait.jpg",
                    "Portrait photo to crop",
                    vec![],
                )],
            ),
            image_cropper::selection(
                vec![],
                vec![
                    image_cropper::handle(HandlePosition::Ne, vec![]),
                    image_cropper::handle(HandlePosition::Sw, vec![]),
                ],
            ),
        ],
    )];
    wrap_example(
        "role=\"group\" の root と方位別 aria-label を持つ 2 個の handle の組み立て例です。",
        body,
    )
}

const IMAGE_CROPPER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "root/viewport/image/selection/handle/grid の anatomy を持ち、矩形の crop 範囲（x/y/width/height）を値として保持する（image_cropper.rs:1-9）。",
        "root は role=\"group\" + aria-roledescription=\"image cropper\" を固定付与する（image_cropper.rs:116-117, 438-449）。",
        "handle は focusable（tabindex=\"0\"）+ 方位別の静的 aria-label（例: \"Resize from bottom right\"）を出力する（image_cropper.rs:117-119, 214, 475-488）。",
        "canvas による実際のピクセル切り出し（画像処理）は本モジュールのスコープ外であり、crop 矩形の値を返すのみである（image_cropper.rs:30-31, 140-144）。",
    ],
    arguments: &[
        ArgRow {
            name: "image(src, alt)",
            kind: "&str, &str",
            default: "",
            description: "対象画像の src/alt。既定エスケープを経由して出力される。",
        },
        ArgRow {
            name: "handle(position)",
            kind: "HandlePosition",
            default: "",
            description: "方位（N/S/E/W/NE/NW/SE/SW）。方位別の静的 aria-label を出力する（image_cropper.rs:214, 475-488）。",
        },
        ArgRow {
            name: "attrs / children",
            kind: "Vec<(&str, &str)> / Vec<Node>",
            default: "",
            description: "各パーツ共通の追加属性・子ノード（代表 1 行に集約）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Two-handle selection",
        description: "NE/SW の 2 ハンドルのみを持つ選択枠の組み立て例です。",
        render: ex_image_cropper,
    }],
    keyboard: &[KeyRow {
        key: "Tab",
        description: "handle パーツは tabindex=\"0\" で focusable である（実際のキーボード nudge の DOM 配線は wasm-full 側の後続責務、image_cropper.rs:475-479, 140-148）。",
    }],
    aria: &[
        AriaRow {
            attribute: "role=\"group\" / aria-roledescription",
            description: "root パーツへ \"image cropper\" を固定付与する（image_cropper.rs:444-449）。",
        },
        AriaRow {
            attribute: "aria-label",
            description: "handle パーツへ方位別の静的文字列（例: \"Resize from bottom right\"）を固定付与する（image_cropper.rs:214, 481-483）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Listbox
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/listbox.rs:88-260`（root/content/item
/// の各パーツ関数）。
fn ex_listbox() -> Node {
    let body = vec![listbox::root(
        OpenState::Open,
        false,
        vec![],
        vec![
            listbox::label(Some("lb2-label"), vec![], vec![text("Country")]),
            listbox::content(
                true,
                Some("lb2-content"),
                Some("lb2-label"),
                None,
                vec![],
                vec![
                    listbox::item(
                        OpenState::Open,
                        false,
                        false,
                        "jp",
                        None,
                        vec![],
                        vec![listbox::item_text(None, vec![], vec![text("Japan")])],
                    ),
                    listbox::item(
                        OpenState::Closed,
                        false,
                        false,
                        "us",
                        None,
                        vec![],
                        vec![listbox::item_text(
                            None,
                            vec![],
                            vec![text("United States")],
                        )],
                    ),
                ],
            ),
        ],
    )];
    wrap_example(
        "multiple=true（aria-multiselectable=\"true\"）の Listbox の組み立て例です。",
        body,
    )
}

const LISTBOX: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "root/label/content/item_group/item_group_label/item/item_text/item_indicator/value_text の anatomy を持ち、content 自身がフォーカスを受ける常時展開のリストである（crate::combobox/select のようなポップオーバー型とは異なる、listbox.rs:1-9, 64）。",
        "content は role=\"listbox\" + tabindex=\"0\" を固定付与し、multiple のとき aria-multiselectable=\"true\" を付与する（single モードでは省略、listbox.rs:117-146）。",
        "item は role=\"option\" + aria-selected を固定付与し、disabled のとき aria-disabled=\"true\" と data-disabled を対で付与する（div[role=\"option\"] はネイティブ disabled を持たないため、listbox.rs:191-233）。",
        "activedescendant が Some のとき content へ aria-activedescendant を付与し、現在ハイライト中の item の id と対応させる（listbox.rs:121-127）。",
    ],
    arguments: &[
        ArgRow {
            name: "root(selection_state, disabled)",
            kind: "OpenState, bool",
            default: "",
            description: "選択有無・無効化を data-* へ反映する（listbox.rs:88-99）。",
        },
        ArgRow {
            name: "content(multiple, id, labelledby, activedescendant)",
            kind: "bool, Option<&str>, Option<&str>, Option<&str>",
            default: "",
            description: "aria-multiselectable/aria-labelledby/aria-activedescendant の各付与条件（listbox.rs:127-146）。",
        },
        ArgRow {
            name: "item(selected_state, disabled, highlighted, value, id)",
            kind: "OpenState, bool, bool, &str, Option<&str>",
            default: "",
            description: "role=\"option\" を持つ選択肢 1 個の状態。value は data-value として既定エスケープ経由で出力される（listbox.rs:207-233）。",
        },
        ArgRow {
            name: "attrs / children",
            kind: "Vec<(&str, &str)> / Vec<Node>",
            default: "",
            description: "各パーツ共通の追加属性・子ノード（代表 1 行に集約）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Multiple selection",
        description: "aria-multiselectable=\"true\" の Listbox の組み立て例です。",
        render: ex_listbox,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "role=\"listbox\"",
            description: "content パーツへ固定付与する（listbox.rs:135）。",
        },
        AriaRow {
            attribute: "aria-multiselectable",
            description: "multiple が true のときのみ \"true\" を付与する（listbox.rs:137-139）。",
        },
        AriaRow {
            attribute: "aria-labelledby",
            description: "content の labelledby が Some のときのみ付与する（listbox.rs:143-145）。",
        },
        AriaRow {
            attribute: "role=\"option\" / aria-selected / aria-disabled",
            description: "item パーツへ固定付与する（aria-disabled は disabled が true のときのみ、listbox.rs:217-227）。",
        },
    ],
    demo: None,
};
