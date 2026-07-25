//! Forms カテゴリ（`site/nav.toml` の `title = "Forms"`）31 部品ページの
//! 原稿データ（イシュー #945、親 #928）。
//!
//! # 責務境界・呼び出し文脈
//!
//! [`crate::component_page::generated_content`] が `page_path` から
//! [`SPECS`] を線形探索し、Features / API Reference の引数表 / Examples /
//! Accessibility の各節を合成する（[`crate::component_page::ComponentPageSpec`]
//! 参照）。Demo 節は原則 [`crate::showcase::COMPONENT_PAGES`]（正）から供給
//! されるが、`showcase.rs` に節を持たない 6 部品（Angle Slider / Image
//! Cropper / Pin Input / Signature Pad / Toggle / Toggle Group）に限り、
//! 本ファイル末尾の `demo_*` 関数が [`ComponentPageSpec::demo`] 経由で Demo
//! 節を供給する（`showcase.rs` 自体は変更しない、イシュー #945 の受け入れ
//! 条件）。
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
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::angle_slider::AngleSlider;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::image_cropper::ImageCropper;
use fandhe_frontend_pre_styled_ui::image_cropper;
use fandhe_frontend_pre_styled_ui::pin_input;
use fandhe_frontend_pre_styled_ui::signature_pad;
use fandhe_frontend_pre_styled_ui::toggle;
use fandhe_frontend_pre_styled_ui::toggle_group;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

use crate::component_page::{ArgRow, AriaRow, ComponentPageSpec};

/// Forms 31 ページの `path -> ComponentPageSpec` テーブル。
/// [`crate::component_page::SPEC_TABLES`] が集約する。
pub const SPECS: &[(&str, ComponentPageSpec)] = &[
    ("/components/angle-slider/", ANGLE_SLIDER),
    ("/components/button/", BUTTON),
    ("/components/calendar/", CALENDAR),
    ("/components/checkbox/", CHECKBOX),
    ("/components/checkbox-card/", CHECKBOX_CARD),
    ("/components/color-picker/", COLOR_PICKER),
    ("/components/combobox/", COMBOBOX),
    ("/components/date-input/", DATE_INPUT),
    ("/components/date-picker/", DATE_PICKER),
    ("/components/download-trigger/", DOWNLOAD_TRIGGER),
    ("/components/editable/", EDITABLE),
    ("/components/file-upload/", FILE_UPLOAD),
    ("/components/image-cropper/", IMAGE_CROPPER),
    ("/components/input/", INPUT),
    ("/components/listbox/", LISTBOX),
    ("/components/native-select/", NATIVE_SELECT),
    ("/components/number-input/", NUMBER_INPUT),
    ("/components/password-input/", PASSWORD_INPUT),
    ("/components/pin-input/", PIN_INPUT),
    ("/components/radio-card/", RADIO_CARD),
    ("/components/radio-group/", RADIO_GROUP),
    ("/components/rating-group/", RATING_GROUP),
    ("/components/segment-group/", SEGMENT_GROUP),
    ("/components/select/", SELECT),
    ("/components/signature-pad/", SIGNATURE_PAD),
    ("/components/slider/", SLIDER),
    ("/components/switch/", SWITCH),
    ("/components/tags-input/", TAGS_INPUT),
    ("/components/textarea/", TEXTAREA),
    ("/components/toggle/", TOGGLE),
    ("/components/toggle-group/", TOGGLE_GROUP),
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
        "見た目 variant（`Solid`/`Outline`/`Ghost`/`Subtle`）・サイズ・colorPalette の 3 軸を持つ単一 recipe styled 部品。",
        "`loading: true` のとき `disabled` と同様に `disabled` 属性・`data-disabled`・`aria-disabled=\"true\"` を付与し、`aria-busy=\"true\"` を追加する。",
        "`loading: true` のとき装飾用途の Spinner（`role`/`aria-label` を持たない）を子ノード先頭へ自動挿入する。",
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

const CALENDAR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "headless-ui の Root / Heading / PrevTrigger / NextTrigger / Table / TableHeader / TableRow / TableHeadCell / TableBody / TableCell / DayTrigger 11 パーツをそのまま再エクスポートする薄いラッパー。",
        "`root` パーツのみ `size` variant クラスを付与する（他パーツは無変更で再エクスポート）。",
        "`Calendar` 状態機械自体は再エクスポートしない（呼び出し側が `fandhe_frontend_headless_ui::calendar` を直接使う設計）。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ variant。",
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
            description: "root 配下の子ノード（通常 Heading/Table を含む）。",
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

const COLOR_PICKER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`size` variant を持たず、headless-ui の `ColorPicker` 状態機械への単純委譲として `root`/`trigger`/`area`/`area_background`/`area_thumb`/`channel_slider`/`channel_slider_track`/`channel_slider_thumb` の各パーツを提供する。",
        "`trigger` パーツが `--fandhe-color-picker-preview`（現在色の HEX、アルファ込み）を含む `style` を付与する唯一のパーツ。",
    ],
    arguments: &[
        ArgRow {
            name: "state",
            kind: "&ColorPicker",
            default: "",
            description: "headless-ui の色状態機械。",
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
            description: "root 配下の子ノード（通常 trigger/area を含む）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
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

const DATE_INPUT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "headless-ui の `date_input::root` へ委譲し、`size` variant クラスを付与する。",
        "`disabled`/`invalid` の 2 状態フラグを直接引数で受け取る（`Props` 構造体を持たない）。",
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
            description: "root 配下の子ノード（通常 `DateSegment` 列を含む）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: None,
};

const DATE_PICKER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "headless-ui の `date_picker::root` へ委譲し、`size` variant クラスを付与する。",
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

const DOWNLOAD_TRIGGER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`button` と同じ `variant`/`size`/`palette` 3 軸を持つが、`<button>` ではなく `download`/`href` 属性付き `<a>` を組み立てる。",
        "`file_name` を渡すと `download` 属性へダウンロード時のファイル名を付与する。",
    ],
    arguments: &[
        ArgRow {
            name: "props",
            kind: "&DownloadTriggerProps",
            default: "",
            description: "`variant`・`size`・`palette` を束ねる構造体（`ButtonProps` と同型）。",
        },
        ArgRow {
            name: "href",
            kind: "&str",
            default: "",
            description: "ダウンロード対象のリンク先。",
        },
        ArgRow {
            name: "file_name",
            kind: "Option<&str>",
            default: "None",
            description: "`download` 属性へ渡すファイル名（`None` のときブラウザ既定名）。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "`<a>` へ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "リンクラベルとなる子ノード。",
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
        "`selection` パーツが `--fandhe-cropper-*`（4 個）custom property を含む `style` を動的値の唯一の出力点として持つ。",
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
    keyboard: &[],
    aria: &[],
    demo: Some(demo_toggle),
};

const TOGGLE_GROUP: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "`size`/`colorPalette` variant クラスを `root` へ付与し、headless-ui の `toggle_group::root` へ委譲する。",
        "`radio_group`/`radio_card`/`segment_group` と同型の `orientation`/`labelled_by` 軸を持つ。",
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
            description: "root 配下の子ノード（通常 `toggle_group::item` 列）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[],
    demo: Some(demo_toggle_group),
};

// ---------------------------------------------------------------------
// Demo フォールバック（showcase.rs 未登録の 6 部品）
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
            vec![angle_slider::thumb_styled(&state, false, vec![])],
        ),
    )
}

fn demo_image_cropper() -> Node {
    let state = ImageCropper::default();
    demo_section(
        "Image Cropper",
        "画像切り抜き範囲の選択 UI。headless-ui の `ImageCropper` 状態機械（既定: 100x100 の全域選択）をラップする。",
        image_cropper::root(
            Size::Md,
            &state,
            vec![],
            vec![image_cropper::selection(&state, vec![], vec![])],
        ),
    )
}

fn demo_pin_input() -> Node {
    demo_section(
        "Pin Input",
        "PIN コード等、固定桁数の入力に使う部品。`complete`/`disabled` の 2 状態を持つ。",
        pin_input::root(Size::Md, false, false, vec![], vec![]),
    )
}

fn demo_signature_pad() -> Node {
    demo_section(
        "Signature Pad",
        "署名入力領域。`empty`（未署名）/`disabled` の 2 状態を持ち、`clear_trigger` で消去操作を提供する。",
        signature_pad::root(
            false,
            true,
            vec![],
            vec![
                signature_pad::control(false, vec![], vec![]),
                signature_pad::clear_trigger(false, vec![], vec![text("Clear")]),
            ],
        ),
    )
}

fn demo_toggle() -> Node {
    demo_section(
        "Toggle",
        "押下状態を持つトグルボタン。`pressed`/`disabled` の 2 状態を持つ。",
        toggle::root(
            Size::Md,
            ColorPalette::Accent,
            false,
            false,
            vec![],
            vec![text("Bold")],
        ),
    )
}

fn demo_toggle_group() -> Node {
    demo_section(
        "Toggle Group",
        "複数の Toggle をまとめて排他/複数選択させるグループ部品。",
        toggle_group::root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            None,
            vec![],
            vec![
                toggle_group::item(false, false, "left", vec![], vec![text("Left")]),
                toggle_group::item(false, false, "center", vec![], vec![text("Center")]),
                toggle_group::item(false, false, "right", vec![], vec![text("Right")]),
            ],
        ),
    )
}
