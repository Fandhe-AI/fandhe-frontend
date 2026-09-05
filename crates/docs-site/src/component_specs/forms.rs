//! Forms カテゴリ（`site/nav.toml` の `title = "Forms"`）部品ページの原稿
//! データ（イシュー #945、親 #928）。
//!
//! `download-trigger` / `color-picker` / `calendar` / `date-picker` /
//! `date-input` の 5 ページは当初本ファイルへ Examples 空欄のスタブとして
//! 登録していたが、後続のイシュー #948 が同じ 5 path へ Examples 込みの
//! より充実した spec を `crate::component_page_specs_948::SPECS` へ登録した
//! ことで [`crate::component_page::SPEC_TABLES`]（`spec_for` の
//! first-wins 解決）上で二重登録となり、本ファイル側のスタブが常に優先され
//! #948 側の Examples が到達不能なデッドコード化していた（PR #982
//! レビュー指摘で発覚）。#948 側の登録が正のため、本ファイルからは当該 5
//! エントリを削除した。
//!
//! # 責務境界・呼び出し文脈
//!
//! [`crate::component_page::generated_content`] が `page_path` から
//! [`SPECS`] を線形探索し、Features / API Reference の引数表 / Examples /
//! Accessibility の各節を合成する（[`crate::component_page::ComponentPageSpec`]
//! 参照）。Demo 節は原則 [`crate::showcase::COMPONENT_PAGES`]（正）から供給
//! されるが、`showcase.rs` に節を持たない 4 部品（Angle Slider / Image
//! Cropper / Pin Input / Signature Pad）に限り、本ファイル末尾の `demo_*`
//! 関数が [`ComponentPageSpec::demo`] 経由で Demo 節を供給する
//! （`showcase.rs` 自体は変更しない、イシュー #945 の受け入れ条件）。
//! Toggle / Toggle Group はイシュー #980 で `showcase.rs` の
//! `COMPONENT_PAGES` 正経路（`toggle_section`/`toggle_group_section`）へ
//! 移設済みのため、本ファイルの `demo` フィールドは両方とも `None`
//! （[`crate::component_page::generated_content`] が
//! `showcase::generated_content` 側を優先照会するため二重供給しない）。
//!
//! # 一次情報・非捏造の方針
//!
//! - Features / Arguments: 各部品の `fandhe-frontend-pre-styled-ui` 公開
//!   関数のシグネチャ・rustdoc から採る（React 風の props 名を発明しない）。
//! - Accessibility（`aria`）: `fandhe-frontend-headless-ui` が実際に SSR
//!   出力する `aria-*` 属性のみを記載する。未確認の属性は書かない。
//! - Keyboard: 本 docs サイトは `crate::script`（テーマトグル + 目次
//!   スクロールスパイ）以外の JS を出力しない。JS 状態機械前提のキー操作
//!   （矢印キーでの候補移動等）は「できる」と書かない方針のため、ネイティブ
//!   要素（`<input>`/`<select>`/`<button>`）のブラウザ標準操作を除き
//!   `keyboard` は空のままとする（Accessibility 節は自動省略される、
//!   `docs/design/docs-site-component-pages.md` §7）。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! 本ファイルはノード木 API のみで組み立て、`raw_html()` を使わない
//! （`tests/component_pages.rs::component_page_source_does_not_use_raw_html`
//! が `component_specs/` 配下を再帰走査してこれを固定する）。

use fandhe_frontend_core::{el, p, text, Node};
use fandhe_frontend_pre_styled_ui::angle_slider;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::angle_slider::{
    AngleSlider, AngleSliderProps,
};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::image_cropper::ImageCropper;
use fandhe_frontend_pre_styled_ui::image_cropper;
use fandhe_frontend_pre_styled_ui::image_cropper::{HandlePosition, ImageCropperProps};
use fandhe_frontend_pre_styled_ui::pin_input;
use fandhe_frontend_pre_styled_ui::signature_pad;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

use crate::component_page::{ArgRow, AriaRow, ComponentPageSpec};

/// Forms 27 ページ（当初 31 ページから、#948 と二重登録だった 5 ページを
/// 削除・#997 で Checkbox Group を追加済み）の `path -> ComponentPageSpec`
/// テーブル。
/// [`crate::component_page::SPEC_TABLES`] が集約する。
pub const SPECS: &[(&str, ComponentPageSpec)] = &[
    ("/themes/angle-slider/", ANGLE_SLIDER),
    ("/themes/button/", BUTTON),
    ("/themes/checkbox/", CHECKBOX),
    ("/themes/checkbox-card/", CHECKBOX_CARD),
    ("/themes/checkbox-group/", CHECKBOX_GROUP),
    ("/themes/combobox/", COMBOBOX),
    ("/themes/editable/", EDITABLE),
    ("/themes/file-upload/", FILE_UPLOAD),
    ("/themes/image-cropper/", IMAGE_CROPPER),
    ("/themes/input/", INPUT),
    ("/themes/listbox/", LISTBOX),
    ("/themes/native-select/", NATIVE_SELECT),
    ("/themes/number-input/", NUMBER_INPUT),
    ("/themes/password-input/", PASSWORD_INPUT),
    ("/themes/pin-input/", PIN_INPUT),
    ("/themes/radio-card/", RADIO_CARD),
    ("/themes/radio-group/", RADIO_GROUP),
    ("/themes/rating-group/", RATING_GROUP),
    ("/themes/segment-group/", SEGMENT_GROUP),
    ("/themes/select/", SELECT),
    ("/themes/signature-pad/", SIGNATURE_PAD),
    ("/themes/slider/", SLIDER),
    ("/themes/switch/", SWITCH),
    ("/themes/tags-input/", TAGS_INPUT),
    ("/themes/textarea/", TEXTAREA),
    ("/themes/toggle/", TOGGLE),
    ("/themes/toggle-group/", TOGGLE_GROUP),
];

const ANGLE_SLIDER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "headless-ui の `angle_slider::AngleSlider` 状態機械を薄くラップし、`size`/`colorPalette` variant クラスを付与する。",
        "`root` パーツが唯一 variant クラスを持ち、`thumb_styled` パーツが `--fandhe-angle` custom property を含む `style` を動的値の唯一の出力点として持つ。",
        "`Theme::to_css` から生成される骨格 CSS（`assets/pre-styled-ui.css`）で `--fandhe-angle-slider-track-size`/`--fandhe-angle-slider-thumb-size` を含む既定スタイルを提供する。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant（`root` の variant クラスへ反映）。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "ColorPalette::Accent",
            description: "colorPalette 軸。`accent`/`info`/`success`/`warning`/`danger` から選択する。",
        },
        ArgRow {
            name: "state",
            kind: "&AngleSlider",
            default: "",
            description: "headless-ui の角度状態機械（現在角度・ステップ幅を保持）。",
        },
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "無効化状態。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性（`class` は `drop_class_attr` により除去されてから合成される）。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード（通常 `thumb_styled` を含む）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: Some(demo_angle_slider),
};

const BUTTON: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "見た目 variant（`Solid`/`Outline`/`Ghost`/`Subtle`/`Surface`/`Plain`、イシュー #1448）・サイズ（`Xs`/`Sm`/`Md`/`Lg`/`Xl` の 5 段、イシュー #1449）・colorPalette の 3 軸を持つ単一 recipe styled 部品。",
        "size variant は高さ・水平 padding・font-size をトークン（`--fandhe-size-control-*`）で固定し、`icon_button`/`close_button`（icon-only）は高さ基準の正方形になる（イシュー #1449）。",
        "`loading: true` のとき `disabled` と同様に `disabled` 属性・`data-disabled`・`aria-disabled=\"true\"` を付与し、`aria-busy=\"true\"` を追加する。",
        "`loading: true` のとき装飾用途の Spinner（`role`/`aria-label` を持たない）を子ノード先頭へ自動挿入する。Spinner のサイズはボタンの `size` へ追随する（`Xs`/`Sm`/`Md` → 小、`Lg`/`Xl` → 中、イシュー #1449）。",
        "`:focus-visible` で palette 連動のフォーカスリングを表示する（イシュー #1449、#1424 準拠）。",
        "`button`/`icon_button`/`close_button` の 3 公開関数が共通の組み立てロジックを共有する（イシュー #830）。",
    ],
    arguments: &[
        ArgRow {
            name: "props",
            kind: "&ButtonProps",
            default: "",
            description: "`variant`（既定 `Solid`）・`size`（既定 `Md`）・`palette`（既定 `Accent`）・`disabled`・`loading` を束ねる構造体。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "`<button>` へ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "ボタンラベルとなる子ノード。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: None,
};

const CHECKBOX: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "headless-ui の `checkbox::root` へ委譲し、`size`/`colorPalette` variant クラスを付与する。",
        "3 値チェック状態（`Unchecked`/`Checked`/`Indeterminate`）を型で表現し、`aria-checked` の偽装・不整合な値を型で塞ぐ。",
        "`control` パーツ（`<div aria-hidden=\"true\">`）は視覚的なチェックボックス表現であり、支援技術からは隠して二重読み上げを防ぐ。",
        "`invalid`/`required`/`readonly` の各フラグを `CheckboxProps` で受け取り、`data-*` 属性・ネイティブ属性へ反映する。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "ColorPalette::Accent",
            description: "colorPalette 軸。",
        },
        ArgRow {
            name: "props",
            kind: "&CheckboxProps",
            default: "",
            description: "`checked`（3 値）・`disabled`・`invalid`・`required`・`readonly` を束ねる構造体。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード（通常 control/label/hidden-input を含む）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-checked",
            description: "`hidden-input` パーツへ `CheckedState` から算出した値（`true`/`false`/`mixed`）を付与する（`Indeterminate` のとき `\"mixed\"`）。",
        },
        AriaRow {
            attribute: "aria-invalid",
            description: "`props.invalid` が `true` のとき `hidden-input` パーツへ `\"true\"` を付与する。",
        },
        AriaRow {
            attribute: "aria-hidden",
            description: "`control` パーツ（視覚的表現のみを担う `<div>`）に固定付与し、支援技術からの重複読み上げを防ぐ。",
        },
    ],
    demo: None,
};

const CHECKBOX_CARD: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`checkbox::CheckboxProps` を共有し、カード形状のコンテナ（`control`/`content`/`label`/`description`/`addon`/`indicator` の各パーツ）へ配置する複合部品。",
        "`size`/`colorPalette` variant クラスを `root` へ付与する。",
        "`indicator_check` パーツでチェック済み時のみ描画されるチェックマークを提供する。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "ColorPalette::Accent",
            description: "colorPalette 軸。",
        },
        ArgRow {
            name: "props",
            kind: "&CheckboxProps",
            default: "",
            description: "`checkbox` モジュールと共有の状態構造体。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root（`<label>`）パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード（通常 control/content を含む）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: None,
};

const CHECKBOX_GROUP: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`size`/`colorPalette` variant クラスを `root` へ付与し、headless-ui の `checkbox_group::root` へ委譲する。",
        "`radio_group` と対称の構造だが、複数選択状態機械（`MultiSelect`）を埋め込み、dispatch は `select`/`deselect`/`toggle` の 3 語彙を受理する。",
        "ネイティブ `<input type=\"checkbox\">` は自前パーツを持たず、`checkbox::hidden_input` を `item` 配下へ入れ子で再利用する（`hidden-input` の視覚的非表示化は `checkbox` の recipe が担う）。",
        "`aria-checked`/`role=\"checkbox\"` は `item-control` へ重複付与しない（二重読み上げ防止）。グループ全体の関連付けは `root` の `aria-labelledby` で行う。",
        "`data-orientation=\"horizontal\"` では折り返し（`flex-wrap: wrap`）付きの横並びへ切り替わる。",
        "`root` の `data-invalid` は custom property 経由で `item-control` の border-color へ伝播する（headless 層が `CheckboxGroupProps` 経由で出力する、イシュー #1603。`attrs` 経由の直接付与も引き続き有効）。`root` の `data-disabled` は CSS のみでは各 item のネイティブ入力を実際に無効化できない（タブ順序を変更できない）ため伝播しない。グループ全体を無効化する場合は、各 item・`checkbox::hidden_input` の `disabled` を利用者が一貫して付与する。",
        "`size` は control 寸法・root/item 余白・font-size を custom property 経由で連動させ、`label`（グループ見出し）は medium ウェイト、`item-text`（項目）は通常ウェイトの 2 段階の型階層を持つ。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "ColorPalette::Accent",
            description: "colorPalette 軸。",
        },
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "グループ全体の無効化状態。",
        },
        ArgRow {
            name: "orientation",
            kind: "Option<Orientation>",
            default: "None",
            description: "キーボード操作方向のヒント（`aria-orientation`）。",
        },
        ArgRow {
            name: "labelled_by",
            kind: "Option<&str>",
            default: "None",
            description: "`Some` のとき `aria-labelledby` を付与する。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード。",
        },
    ],
    examples: &[],
    keyboard: &[crate::component_page::KeyRow {
        key: "Space",
        description: "ネイティブ `<input type=\"checkbox\">`（`checkbox::hidden_input` の再利用）のブラウザ既定動作としてチェック状態をトグルする（ブラウザ実装依存、本フレームワークの JS 出力によらない）。",
    }],
    aria: &[
        AriaRow {
            attribute: "aria-labelledby",
            description: "`labelled_by` が `Some` のときのみ `root` へ付与する。",
        },
        AriaRow {
            attribute: "aria-orientation",
            description: "`orientation` が `Some` のとき `root` へ `data-orientation` と対で付与する。",
        },
    ],
    demo: None,
};

const COMBOBOX: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "headless-ui の `combobox::root` へ委譲し、`size` variant クラスのみを付与する。",
        "開閉状態は `OpenState`（`Open`/`Closed`）で受け取る。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "state",
            kind: "OpenState",
            default: "OpenState::Closed",
            description: "開閉状態。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: None,
};

const EDITABLE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`size` variant クラスを `root` へ付与し、headless-ui の `editable::root` へ委譲する。",
        "`mode`（`EditMode::Preview`/`Edit`）・`activation_mode`・`submit_mode` の各軸をそのまま引数で受け取る。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "mode",
            kind: "EditMode",
            default: "EditMode::Preview",
            description: "プレビュー/編集の表示モード。",
        },
        ArgRow {
            name: "flags",
            kind: "EditableInputFlags",
            default: "EditableInputFlags::default()",
            description: "disabled/readonly/required/invalid のフラグ束。root へは disabled/readonly のみ反映する。",
        },
        ArgRow {
            name: "activation_mode",
            kind: "EditableActivationMode",
            default: "EditableActivationMode::default()",
            description: "編集モードへの遷移トリガー（クリック/ダブルクリック等）。",
        },
        ArgRow {
            name: "submit_mode",
            kind: "EditableSubmitMode",
            default: "EditableSubmitMode::default()",
            description: "確定操作（Enter/blur 等）の種別。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: None,
};

const FILE_UPLOAD: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "headless-ui の `file_upload::root` へ委譲し、`size` variant クラスのみを付与する。",
        "`FileUploadProps`（disabled/readonly/invalid/required）と `dragging`（`data-dragging` の DOM ローカル状態）を引数で受け取り、headless 層へそのまま委譲する（イシュー #1609 の headless 側破壊的変更への追随）。",
        "`item` の `data-invalid` は headless 層が出力しないため（旧 `checkbox_group` の判断。#1603 で checkbox_group 側は headless 出力へ移行済み）、利用者が `item` の `attrs` へ `(\"data-invalid\", \"\")` を直接付与することで border-color を danger 色化できる。",
        "`item` は border と border-color の transition（`data-invalid` 用）を持つ。`item-delete-trigger` は hover（`@media (hover: hover)`）を持つが、`item` が既に opacity 0.5 で dim 済みのため disabled は `cursor: not-allowed` のみに留め、opacity の三重適用（root × item × item-delete-trigger）を避ける。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "props",
            kind: "&FileUploadProps",
            default: "",
            description: "disabled/readonly/invalid/required の状態束（headless 層へそのまま委譲）。",
        },
        ArgRow {
            name: "dragging",
            kind: "bool",
            default: "false",
            description: "root の `data-dragging`（wasm-full 側が DOM ローカルにトグルする想定）。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード（ドロップゾーン・トリガー等）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: None,
};

const IMAGE_CROPPER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`size` variant クラスを `root` へ付与し、headless-ui の `ImageCropper` 状態機械（切り抜き範囲・原寸を保持）へ委譲する。",
        "`selection` パーツが `--fandhe-image-cropper-*`（4 個）custom property を含む `style` を動的値の唯一の出力点として持つ。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "state",
            kind: "&ImageCropper",
            default: "",
            description: "headless-ui の切り抜き状態機械。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード（通常 `selection` を含む）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: Some(demo_image_cropper),
};

const INPUT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`variant`（既定 `Outline`）/`size` の 2 軸を持ち、headless-ui の `field::input`（`data-scope=\"field\"`）へ委譲する。",
        "`FieldProps` を通じて `disabled`/`invalid`/`required`/`readonly` を制御する（`Field` 自体は headless-ui のみが提供し、pre-styled-ui 側に独立ページを持たない）。",
    ],
    arguments: &[
        ArgRow {
            name: "props",
            kind: "&InputProps",
            default: "",
            description: "`variant`（既定 `Outline`）・`size`（既定 `Md`）を束ねる構造体。",
        },
        ArgRow {
            name: "field",
            kind: "&FieldProps<'_>",
            default: "",
            description: "`id`・`disabled`・`invalid`・`required`・`readonly` 等、headless-ui `field` スコープ共通の状態。",
        },
        ArgRow {
            name: "extra_attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "`<input>` へ合成する追加属性（`type` 等）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: None,
};

const LISTBOX: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "headless-ui の `listbox::root` へ委譲し、`size` variant クラスを付与する。",
        "`selection_state`（`OpenState`）・`disabled` の 2 軸を直接引数で受け取る。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "selection_state",
            kind: "OpenState",
            default: "OpenState::Closed",
            description: "選択状態。",
        },
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "無効化状態。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: None,
};

const NATIVE_SELECT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`variant`/`size` の 2 軸を持ち、headless-ui の `field::select`（`data-scope=\"field\"`）へ委譲する。ネイティブ `<select>` を組み立てるため、キーボード操作はブラウザ標準に委ねる。",
    ],
    arguments: &[
        ArgRow {
            name: "props",
            kind: "&NativeSelectProps",
            default: "",
            description: "`variant`・`size` を束ねる構造体。",
        },
        ArgRow {
            name: "field",
            kind: "&FieldProps<'_>",
            default: "",
            description: "headless-ui `field` スコープ共通の状態。",
        },
        ArgRow {
            name: "extra_attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "`<select>` へ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "`<option>` 等の子ノード。",
        },
    ],
    examples: &[],
    keyboard: &[
        crate::component_page::KeyRow {
            key: "ArrowUp / ArrowDown",
            description: "ネイティブ `<select>` の標準挙動として選択肢を移動する（ブラウザ実装依存、本フレームワークの JS 出力によらない）。",
        },
    ],
    aria: &[],
    demo: None,
};

const NUMBER_INPUT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "headless-ui の `number_input::root` へ委譲し、`size` variant クラスを付与する。",
        "`disabled`/`invalid` の 2 状態フラグを直接引数で受け取る。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "無効化状態。",
        },
        ArgRow {
            name: "invalid",
            kind: "bool",
            default: "false",
            description: "入力検証エラー状態。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード（通常インクリメント/デクリメントトリガーを含む）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: None,
};

const PASSWORD_INPUT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`size`/`colorPalette` variant クラスを付与し、headless-ui の `password_input::root` へ委譲する。",
        "`visible` フラグで表示/非表示を制御し、`PasswordInputProps` で `autocomplete`（`PasswordAutocomplete`）等を渡す。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "ColorPalette::Accent",
            description: "colorPalette 軸。",
        },
        ArgRow {
            name: "visible",
            kind: "bool",
            default: "false",
            description: "パスワード文字列の表示/非表示。",
        },
        ArgRow {
            name: "props",
            kind: "&PasswordInputProps<'_>",
            default: "",
            description: "`id`・`disabled`・`invalid`・`required`・`autocomplete` を束ねる構造体。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード（通常 input/visibility_trigger を含む）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-invalid",
            description: "`props.invalid` が `true` のとき `input` パーツへ `\"true\"` を付与する。",
        },
    ],
    demo: None,
};

const PIN_INPUT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`size` variant クラスを `root` へ付与し、headless-ui の `pin_input::root` へ委譲する。",
        "`complete`（全桁入力済み）・`disabled` の 2 状態フラグを直接引数で受け取る。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "complete",
            kind: "bool",
            default: "false",
            description: "全桁入力済み状態。",
        },
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "無効化状態。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード（通常 1 桁ずつの input パーツ列）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: Some(demo_pin_input),
};

const RADIO_CARD: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`size`/`colorPalette` variant クラスを `root` へ付与し、`role=\"radiogroup\"` を固定付与する。",
        "`item`/`item_control`/`item_content`/`item_text`/`item_description`/`item_addon`/`item_indicator`/`item_hidden_input` の各パーツでカード形状の選択肢を構成する。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "ColorPalette::Accent",
            description: "colorPalette 軸。",
        },
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "グループ全体の無効化状態。",
        },
        ArgRow {
            name: "orientation",
            kind: "Option<Orientation>",
            default: "None",
            description: "キーボード操作方向のヒント（`aria-orientation`）。",
        },
        ArgRow {
            name: "labelled_by",
            kind: "Option<&str>",
            default: "None",
            description: "`Some` のとき `aria-labelledby` を付与する。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード（通常 `item` 列）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: None,
};

const RADIO_GROUP: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`size`/`colorPalette` variant クラスを `root` へ付与し、headless-ui の `radio_group::root` へ委譲する。",
        "`aria-checked` は各アイテムへ重複付与しない（二重読み上げ防止）。グループ全体の関連付けは `root` の `aria-labelledby` で行う。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "ColorPalette::Accent",
            description: "colorPalette 軸。",
        },
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "グループ全体の無効化状態。",
        },
        ArgRow {
            name: "orientation",
            kind: "Option<Orientation>",
            default: "None",
            description: "キーボード操作方向のヒント（`aria-orientation`）。",
        },
        ArgRow {
            name: "labelled_by",
            kind: "Option<&str>",
            default: "None",
            description: "`Some` のとき `aria-labelledby` を付与する。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-labelledby",
            description: "`labelled_by` が `Some` のときのみ `root` へ付与する。",
        },
        AriaRow {
            attribute: "aria-orientation",
            description: "`orientation` が `Some` のとき `root` へ `data-orientation` と対で付与する。",
        },
    ],
    demo: None,
};

const RATING_GROUP: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`size`/`colorPalette` variant クラスを `root` へ付与し、headless-ui の `rating_group::root` へ委譲する。",
        "`readonly` フラグを持つ（`radio_group` 等と異なり読み取り専用表示を単独でサポート）。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "ColorPalette::Accent",
            description: "colorPalette 軸。",
        },
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "無効化状態。",
        },
        ArgRow {
            name: "readonly",
            kind: "bool",
            default: "false",
            description: "読み取り専用状態。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード（通常アイテム列）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: None,
};

const SEGMENT_GROUP: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`size` variant クラスのみを `root` へ付与し、headless-ui の `segment_group::root` へ委譲する（`colorPalette` 軸を持たない）。",
        "`orientation`/`labelled_by` を `radio_group`/`radio_card`/`toggle_group` と同型で受け取る。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "グループ全体の無効化状態。",
        },
        ArgRow {
            name: "orientation",
            kind: "Option<Orientation>",
            default: "None",
            description: "キーボード操作方向のヒント（`aria-orientation`）。",
        },
        ArgRow {
            name: "labelled_by",
            kind: "Option<&str>",
            default: "None",
            description: "`Some` のとき `aria-labelledby` を付与する。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: None,
};

const SELECT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "headless-ui の `select::root` へ委譲し、`size` variant クラスを付与する。",
        "`trigger` パーツが `aria-haspopup=\"listbox\"` を固定付与し、`content`/`label` との関連付けを `aria-controls`/`aria-labelledby` で行う。",
        "`content` パーツの `aria-activedescendant` は選択中アイテムの `id` を参照する（select-only combobox パターン）。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "state",
            kind: "OpenState",
            default: "OpenState::Closed",
            description: "開閉状態。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード（通常 trigger/content を含む）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-haspopup",
            description: "`trigger` パーツへ `\"listbox\"` を固定付与する。",
        },
        AriaRow {
            attribute: "aria-expanded",
            description: "開閉状態に応じて `trigger` パーツへ `\"true\"`/`\"false\"` を付与する。",
        },
        AriaRow {
            attribute: "aria-controls",
            description: "`controls` が `Some` のとき `trigger` パーツから `content` パーツへ関連付ける。",
        },
        AriaRow {
            attribute: "aria-activedescendant",
            description: "`content` パーツへ、フォーカス対象アイテムの `id` を参照値として付与する（`Some` のときのみ）。",
        },
        AriaRow {
            attribute: "aria-selected",
            description: "各アイテムパーツへ選択状態に応じて `\"true\"`/`\"false\"` を付与する。",
        },
    ],
    demo: None,
};

const SIGNATURE_PAD: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`size` variant を持たず、headless-ui の `signature_pad::root` へそのまま委譲する。",
        "`disabled`/`empty`（未署名）の 2 状態フラグを `root` が直接引数で受け取る。",
        "`control`/`segment`/`clear_trigger` の各パーツで署名領域・消去操作を構成する。",
    ],
    arguments: &[
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "無効化状態。",
        },
        ArgRow {
            name: "empty",
            kind: "bool",
            default: "true",
            description: "未署名状態。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード（通常 control/clear_trigger を含む）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: Some(demo_signature_pad),
};

const SLIDER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`size`/`colorPalette` variant クラスを `root` へ付与し、headless-ui の `Slider` 状態機械へ委譲する。",
        "`range` パーツが `--fandhe-slider-percent` を含む `style` を動的値の唯一の出力点として持つ。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "ColorPalette::Accent",
            description: "colorPalette 軸。",
        },
        ArgRow {
            name: "state",
            kind: "&Slider",
            default: "",
            description: "headless-ui の値状態機械。",
        },
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "無効化状態。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード（通常 range/thumb_styled を含む）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: None,
};

const SWITCH: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`size`/`colorPalette` variant クラスを `root` へ付与し、headless-ui の `switch::root` へ委譲する。",
        "ネイティブ `checked` 状態がブラウザにより `aria-checked` へ自動マップされるため、本モジュールは `aria-checked` を明示付与しない（二重読み上げ防止）。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "ColorPalette::Accent",
            description: "colorPalette 軸。",
        },
        ArgRow {
            name: "checked",
            kind: "bool",
            default: "false",
            description: "オン/オフ状態。",
        },
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "無効化状態。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード（通常 hidden-input/control/label を含む）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-hidden",
            description: "視覚的な表現のみを担う `control`/`thumb` パーツへ固定付与し、支援技術からの重複読み上げを防ぐ。",
        },
    ],
    demo: None,
};

const TAGS_INPUT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "headless-ui の `tags_input::root` へ委譲し、`size` variant クラスのみを付与する。",
        "`disabled` の単一状態フラグを直接引数で受け取る。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "無効化状態。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード（通常タグ列 + 入力欄を含む）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: None,
};

const TEXTAREA: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`variant`/`size` の 2 軸を持ち、headless-ui の `field::textarea`（`data-scope=\"field\"`）へ委譲する。",
        "`autoresize` フラグでコンテンツに応じた自動リサイズの意図を headless 側へ伝える。",
    ],
    arguments: &[
        ArgRow {
            name: "props",
            kind: "&TextareaProps",
            default: "",
            description: "`variant`・`size` を束ねる構造体。",
        },
        ArgRow {
            name: "field",
            kind: "&FieldProps<'_>",
            default: "",
            description: "headless-ui `field` スコープ共通の状態。",
        },
        ArgRow {
            name: "autoresize",
            kind: "bool",
            default: "false",
            description: "自動リサイズを有効にするかどうかのヒント。",
        },
        ArgRow {
            name: "extra_attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "`<textarea>` へ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "テキストコンテンツとなる子ノード。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: None,
};

const TOGGLE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`size`/`colorPalette` variant クラスを `root` へ付与し、headless-ui の `toggle::root` へ委譲する。",
        "`pressed`/`disabled` の 2 状態フラグを直接引数で受け取る。",
        "状態機械は Switch と同じ `Checkable` を内部再利用するが、公開語彙は `\"on\"`/`\"off\"`（`aria-pressed` と `data-pressed` を併記）で Switch とは異なる。",
        "`root` 自身がネイティブ `<button type=\"button\">` であり、Switch/RadioGroup のような hidden input を持たない。",
        "`indicator` は off 時に styled 層 CSS が `display: none` で隠す（headless 層は `data-state` のみ出力する）。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "ColorPalette::Accent",
            description: "colorPalette 軸。",
        },
        ArgRow {
            name: "pressed",
            kind: "bool",
            default: "false",
            description: "押下状態。",
        },
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "無効化状態。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "ラベルとなる子ノード。",
        },
    ],
    examples: &[],
    keyboard: &[crate::component_page::KeyRow {
        key: "Space / Enter",
        description: "ネイティブ `<button>` のブラウザ既定動作により押下状態を切り替える（ブラウザ実装依存、本フレームワークの JS 出力によらない）。",
    }],
    aria: &[AriaRow {
        attribute: "aria-pressed",
        description: "`root` に付与。押下状態（true/false）を表す。",
    }],
    demo: None,
};

const TOGGLE_GROUP: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`size`/`colorPalette` variant クラスを `root` へ付与し、headless-ui の `toggle_group::root` へ委譲する。",
        "`radio_group`/`radio_card`/`segment_group` と同型の `orientation`/`labelled_by` 軸を持つ。",
        "各 item は単体 Toggle と同じ押下状態付きネイティブ button で `aria-pressed`/`data-state` 語彙を揃える。",
        "`root` のみが `role=\"group\"` を持つ（`role=\"radiogroup\"` の RadioGroup とは異なる）。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "ColorPalette::Accent",
            description: "colorPalette 軸。",
        },
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "グループ全体の無効化状態。",
        },
        ArgRow {
            name: "orientation",
            kind: "Option<Orientation>",
            default: "None",
            description: "`Some` のとき `root` へ `data-orientation` を出力する（`role=\"group\"` に `aria-orientation` は WAI-ARIA 上許可されないため付与しない）。",
        },
        ArgRow {
            name: "labelled_by",
            kind: "Option<&str>",
            default: "None",
            description: "`Some` のとき `aria-labelledby` を付与する。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード（通常 `toggle_group::item` 列）。",
        },
    ],
    examples: &[],
    keyboard: &[crate::component_page::KeyRow {
        key: "Space / Enter",
        description: "ネイティブ `<button>` のブラウザ既定動作により各 item の押下状態を切り替える（ブラウザ実装依存、本フレームワークの JS 出力によらない）。",
    }],
    aria: &[
        AriaRow {
            attribute: "role=\"group\"",
            description: "`root` に固定付与する。",
        },
        AriaRow {
            attribute: "aria-labelledby",
            description: "`labelled_by` が `Some` のときのみ `root` へ付与する。",
        },
        AriaRow {
            attribute: "aria-pressed",
            description: "各 item に付与する。押下状態（true/false）を表す。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Demo フォールバック（showcase.rs 未登録の 4 部品。Toggle / Toggle Group は
// イシュー #980 で showcase.rs の COMPONENT_PAGES 正経路へ移設済み）
// ---------------------------------------------------------------------
//
// showcase.rs の各 `*_section()` と同じ `div > section > [h2, p, …]` 構造
// （`component_page::strip_demo_heading` が先頭 h2 を 1 個だけ剥がす前提）
// に合わせる。

/// 部品 1 件分の Demo 節を組み立てる（`showcase.rs::section` と同型）。
fn demo_section(heading: &str, description: &str, demo: Node) -> Node {
    el(
        "div",
        vec![],
        vec![el(
            "section",
            vec![],
            vec![
                el("h2", vec![], vec![text(heading)]),
                p(vec![], vec![text(description)]),
                demo,
            ],
        )],
    )
}

fn demo_angle_slider() -> Node {
    let state = AngleSlider::default();
    demo_section(
        "Angle Slider",
        "角度入力用の Slider 部品。headless-ui の `AngleSlider` 状態機械（既定角度 0 度）をラップする。",
        angle_slider::root(
            Size::Md,
            ColorPalette::Accent,
            &state,
            false,
            vec![],
            // 型階層（イシュー #1446）を Demo で視覚確認できるよう
            // label・value_text を追加する（`primitive_specs/forms_a.rs::ex_angle_slider`
            // の構成と整合、`crates/pre-styled-ui/src/angle_slider.rs`
            // モジュール doc 参照）。`thumb_styled` は `position: absolute`
            // を前提に `control`（`position: relative` の円盤）を基準座標
            // として配置されるため、`root` 直下ではなく `control` の子として
            // 配置する（イシュー #1445。従来は `control` を挟んでおらず
            // 円盤・サムが描画されていなかった）。
            // `headless_ui::angle_slider::label` の契約上、呼び出し側が
            // `label` へ一意な `id` を付与し `thumb`（`role="slider"`）側の
            // `aria-labelledby` へ同じ値を渡さない限りアクセシブルネームが
            // 関連付けられない（イシュー #1446 codex-review 指摘）。
            vec![
                angle_slider::label(
                    &AngleSliderProps::default(),
                    vec![("id", "angle-slider-demo-label")],
                    vec![text("Rotation")],
                ),
                angle_slider::control(
                    &AngleSliderProps::default(),
                    vec![],
                    vec![angle_slider::thumb_styled(
                        &state,
                        false,
                        vec![("aria-labelledby", "angle-slider-demo-label")],
                    )],
                ),
                angle_slider::value_text(vec![], vec![text("0deg")]),
            ],
        ),
    )
}

/// パーセントエンコード済みインライン SVG data URI（生の `<`・引用符を含まず、
/// GitHub Pages 上で外部リクエスト・404 を発生させない。`crate::showcase`
/// の `AVATAR_INLINE_SVG_SRC` と同型のプレースホルダー方針、イシュー
/// #1480）。グラデーション矩形のダミー画像。
const IMAGE_CROPPER_DEMO_IMAGE_SRC: &str =
    "data:image/svg+xml,%3Csvg%20xmlns%3D%27http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%27%20viewBox%3D%270%200%20200%20120%27%3E%3Cdefs%3E%3ClinearGradient%20id%3D%27g%27%20x1%3D%270%27%20y1%3D%270%27%20x2%3D%271%27%20y2%3D%271%27%3E%3Cstop%20offset%3D%270%27%20stop-color%3D%27%234a90d9%27%2F%3E%3Cstop%20offset%3D%271%27%20stop-color%3D%27%23274b73%27%2F%3E%3C%2FlinearGradient%3E%3C%2Fdefs%3E%3Crect%20width%3D%27200%27%20height%3D%27120%27%20fill%3D%27url(%23g)%27%2F%3E%3C%2Fsvg%3E";

fn demo_image_cropper() -> Node {
    // 200x120 の画像に対し中央 60%（x=20%/y=20%/w=60%/h=60%）を選択した状態。
    // 既定（全域選択）のままだと selection・handle・grid が画像縁に重なり
    // 視覚確認できないため、本イシュー（#1480）担当パートの視覚確認用に
    // 非全域の選択状態を作る。
    let state = ImageCropper::new(200, 120, 40, 24, 120, 72, None, 1);
    let props = ImageCropperProps::default();
    let handles = [
        HandlePosition::N,
        HandlePosition::S,
        HandlePosition::E,
        HandlePosition::W,
        HandlePosition::Ne,
        HandlePosition::Nw,
        HandlePosition::Se,
        HandlePosition::Sw,
    ]
    .into_iter()
    .map(|position| image_cropper::handle(position, &props, vec![]))
    .collect::<Vec<_>>();

    demo_section(
        "Image Cropper",
        "画像切り抜き範囲の選択 UI。headless-ui の `ImageCropper` 状態機械（クロップ枠・8 方位リサイズハンドル・三分割グリッド線を含む完全な anatomy）をラップする。",
        image_cropper::root(
            Size::Md,
            &state,
            &props,
            vec![],
            vec![image_cropper::viewport(
                &props,
                vec![],
                vec![
                    image_cropper::image(IMAGE_CROPPER_DEMO_IMAGE_SRC, "", vec![]),
                    image_cropper::selection(
                        &state,
                        &props,
                        vec![],
                        handles
                            .into_iter()
                            .chain(std::iter::once(image_cropper::grid(None, &props, vec![])))
                            .collect(),
                    ),
                ],
            )],
        ),
    )
}

/// `count` 桁分の PinInput `input` セルを組み立てる（内部ヘルパ、
/// [`demo_pin_input`] のみが呼ぶ）。`complete: true` の場合は各セルへ
/// ダミー値 `"1"` を入れて `data-complete` の枠色を視覚確認できるように
/// する。
fn pin_input_cells(count: usize, complete: bool, disabled: bool) -> Vec<Node> {
    let props = pin_input::PinInputProps {
        disabled,
        ..Default::default()
    };
    (0..count)
        .map(|i| {
            let value = if complete { "1" } else { "" };
            pin_input::input(
                i,
                count,
                value,
                pin_input::PinInputKind::Numeric,
                false,
                false,
                &props,
                complete,
                vec![],
            )
        })
        .collect()
}

fn demo_pin_input() -> Node {
    // 完全な anatomy（label + control + input×4）を size 3 段
    // （sm/md/lg）・complete・disabled の状態行で並べ、寸法・枠色・
    // hover・フォーカスリング・減光を視覚確認できるようにする
    // （イシュー #1489。従来は children 空の root のみで視覚確認できな
    // かった）。
    let build = |size: Size, complete: bool, disabled: bool| {
        let props = pin_input::PinInputProps {
            disabled,
            ..Default::default()
        };
        pin_input::root(
            size,
            complete,
            disabled,
            vec![],
            vec![
                pin_input::label(complete, &props, vec![], vec![text("PIN code")]),
                pin_input::control(vec![], pin_input_cells(4, complete, disabled)),
            ],
        )
    };
    demo_section(
        "Pin Input",
        "PIN コード等、固定桁数の入力に使う部品。`complete`/`disabled` の 2 状態を持つ。",
        el(
            "div",
            vec![],
            vec![
                build(Size::Sm, false, false),
                build(Size::Md, false, false),
                build(Size::Lg, false, false),
                build(Size::Md, true, false),
                build(Size::Md, false, true),
            ],
        ),
    )
}

fn demo_signature_pad() -> Node {
    // イシュー #1503: 参考サイト（ark-ui signature-pad）のスクショと並べて
    // 比較できるよう、label + control（segment/guide 込み）+ clear-trigger
    // の通常例と disabled 例の 2 系統を並べる（従来は control が空の
    // まま線 1 本しか見えず実態が伝わらなかった）。
    demo_section(
        "Signature Pad",
        "署名入力領域。`empty`（未署名）/`disabled` の 2 状態を持ち、`clear_trigger` で消去操作を提供する。",
        el(
            "div",
            vec![("style", "display: flex; flex-direction: column; gap: 1.5rem;")],
            vec![
                signature_pad::root(
                    false,
                    true,
                    vec![],
                    vec![
                        signature_pad::label(false, vec![], vec![text("Sign here")]),
                        signature_pad::control(
                            false,
                            vec![("aria-label", "Sign here")],
                            vec![
                                signature_pad::segment(600, 200, Some("signature"), vec![], vec![]),
                                signature_pad::guide(false, vec![], vec![]),
                            ],
                        ),
                        signature_pad::clear_trigger(false, vec![], vec![text("Clear")]),
                    ],
                ),
                signature_pad::root(
                    true,
                    true,
                    vec![],
                    vec![
                        signature_pad::label(true, vec![], vec![text("Sign here (disabled)")]),
                        signature_pad::control(
                            true,
                            vec![("aria-label", "Sign here (disabled)")],
                            vec![
                                signature_pad::segment(600, 200, Some("signature"), vec![], vec![]),
                                signature_pad::guide(true, vec![], vec![]),
                            ],
                        ),
                        signature_pad::clear_trigger(true, vec![], vec![text("Clear")]),
                    ],
                ),
            ],
        ),
    )
}
