# ai-chat-playground

モデル・プリセット選択、応答プレビュー、プロンプト入力欄を左カラムへ縦積み
にし、右カラムへ生成パラメータ（温度・最大トークン数・Top P のスライダーと
ストリーミング/システムプロンプト同梱の switch）を並べたモデル実験用
プレイグラウンドです。`field` / `native-select` / `textarea` / `slider` /
`switch` / `button` / `popover` の 7 部品を合成します。Blocks は既存部品の
合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0005（代表構成）です。モデル名・プリセット名・
プロンプト・応答例はすべて架空のデータであり、実在ベンダーのモデル名・
PII は含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。生成中
（スピナー・ストリーミング途中）の状態は表現せず、入力済みプロンプトと
応答例の固定表示に留めます。プリセットの説明は popover を常時開いた
静的カードとして表示します（無 JS のため開閉は実演できません）。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::slider::Slider;
use fandhe_frontend_pre_styled_ui::field::{self, FieldIds, FieldProps, FieldRootProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::popover::{self, OpenState};
use fandhe_frontend_pre_styled_ui::slider::{self, SliderProps};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Orientation, Size};

/// popover の content/title/description の `id`（trigger の `aria-controls`・
/// content 自身の `aria-labelledby`/`aria-describedby` と対で参照する）。
const POPOVER_CONTENT_ID: &str = "blocks-ai-chat-playground-popover-content";
const POPOVER_TITLE_ID: &str = "blocks-ai-chat-playground-popover-title";
const POPOVER_DESC_ID: &str = "blocks-ai-chat-playground-popover-desc";

/// スライダー thumb の可視ラベル `id`（`aria-labelledby` の参照先）。
const TEMPERATURE_LABEL_ID: &str = "blocks-ai-chat-playground-temperature-label";
const MAX_TOKENS_LABEL_ID: &str = "blocks-ai-chat-playground-max-tokens-label";
const TOP_P_LABEL_ID: &str = "blocks-ai-chat-playground-top-p-label";

/// 「モデル」「プリセット」選択欄の 1 件を組む（`field::root` +
/// `field::label` + [`native_select::native_select`]、`form_layout_
/// property_panel::unit_select` と同型の構成で可視ラベルを持たせる）。
fn select_field(
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
        .map(|(value, label)| el("option", vec![("value", value)], vec![text(*label)]))
        .collect();
    field::root(
        &FieldRootProps::default(),
        &field_props,
        vec![],
        vec![
            field::label(&field_props, vec![], vec![text(label_text)]),
            native_select::native_select(
                &NativeSelectProps::default(),
                &field_props,
                vec![],
                option_nodes,
            ),
        ],
    )
}

/// プリセットの説明を添える popover（常時開いた静的カード、モジュール doc
/// 「popover はプリセット説明を常時開いた静的カードとして表示する」節）。
fn preset_help_popover() -> Node {
    popover::root(
        OpenState::Open,
        vec![],
        vec![
            popover::trigger(
                OpenState::Open,
                true,
                Some(POPOVER_CONTENT_ID),
                vec![],
                vec![text("プリセットとは")],
            ),
            popover::positioner(
                OpenState::Open,
                vec![],
                vec![popover::content(
                    OpenState::Open,
                    Some(POPOVER_CONTENT_ID),
                    Some(POPOVER_TITLE_ID),
                    Some(POPOVER_DESC_ID),
                    vec![],
                    vec![
                        popover::title(
                            Some(POPOVER_TITLE_ID),
                            vec![],
                            vec![text("プリセットとは")],
                        ),
                        popover::description(
                            Some(POPOVER_DESC_ID),
                            vec![],
                            vec![text(
                                "プリセットを選ぶと、右側の生成設定（温度・最大トークン数・Top P 等）がまとめて切り替わります。",
                            )],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// 応答プレビュー欄の発話 1 件（話者ラベル + 本文）。
fn message(speaker: &'static str, body: &'static str) -> Node {
    div(
        vec![("class", "blocks-ai-chat-playground-message")],
        vec![
            div(
                vec![("class", "blocks-ai-chat-playground-message-speaker")],
                vec![text(speaker)],
            ),
            div(
                vec![("class", "blocks-ai-chat-playground-message-body")],
                vec![text(body)],
            ),
        ],
    )
}

/// [`param_slider`] の 1 件分の仕様（クリッピーの引数過多〔≤7〕対策で
/// 構造体へ集約する。呼び出し側 [`demo`] の可読性はフィールド名で担保する）。
struct SliderSpec {
    label_id: &'static str,
    hidden_name: &'static str,
    label_text: &'static str,
    min: f64,
    max: f64,
    step: f64,
    value: f64,
    value_text: &'static str,
}

/// 指定パラメータのスライダー 1 件（ラベル + control/track/range/thumb +
/// hidden_input、`pricing_usage_slider` と同型の構成）。
fn param_slider(spec: SliderSpec) -> Node {
    let props = SliderProps::default();
    let state = Slider::new(
        spec.min,
        spec.max,
        spec.step,
        spec.value,
        Orientation::Horizontal,
    );
    slider::root(
        Size::Md,
        ColorPalette::Accent,
        &state,
        &props,
        vec![],
        vec![
            slider::label(
                &props,
                vec![("id", spec.label_id)],
                vec![text(spec.label_text)],
            ),
            slider::control(
                Orientation::Horizontal,
                &props,
                vec![],
                vec![
                    slider::track(
                        Orientation::Horizontal,
                        &props,
                        vec![],
                        vec![slider::range(&state, &props, vec![])],
                    ),
                    slider::thumb_styled(
                        &state,
                        Some(spec.value_text),
                        &props,
                        vec![("aria-labelledby", spec.label_id)],
                    ),
                ],
            ),
            slider::hidden_input(spec.hidden_name, spec.value_text, false, vec![]),
        ],
    )
}

/// 生成設定の switch 1 件（`pricing_seats_split` と同型の構成）。
fn param_switch(hidden_name: &'static str, label_text: &'static str, checked: bool) -> Node {
    let props = SwitchProps::default();
    switch::root(
        Size::Md,
        ColorPalette::Accent,
        checked,
        &props,
        vec![],
        vec![
            switch::label(checked, &props, vec![], vec![text(label_text)]),
            switch::hidden_input(hidden_name, "on", checked, &props, vec![]),
            switch::control(
                checked,
                &props,
                vec![],
                vec![switch::thumb(checked, &props, vec![], vec![])],
            ),
        ],
    )
}

/// `ai-chat-playground` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    let field_props = FieldProps {
        id: "blocks-ai-chat-playground-prompt",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let prompt_field = field::root(
        &FieldRootProps::default(),
        &field_props,
        vec![],
        vec![
            field::label(&field_props, vec![], vec![text("プロンプト")]),
            textarea::textarea(
                &TextareaProps::default(),
                &field_props,
                false,
                vec![("rows", "4")],
                vec![text(
                    "四半期レポートの要点を 3 行で要約してください。読み手は非エンジニアの経営層です。",
                )],
            ),
        ],
    );

    let main_column = div(
        vec![("class", "blocks-ai-chat-playground-main")],
        vec![
            div(
                vec![("class", "blocks-ai-chat-playground-selectors")],
                vec![
                    select_field(
                        "blocks-ai-chat-playground-model",
                        "モデル",
                        &[
                            ("nova-mini", "Nova Mini"),
                            ("nova-standard", "Nova Standard"),
                            ("nova-pro", "Nova Pro"),
                        ],
                    ),
                    select_field(
                        "blocks-ai-chat-playground-preset",
                        "プリセット",
                        &[
                            ("summarize", "要約"),
                            ("translate", "翻訳"),
                            ("explain-code", "コード説明"),
                        ],
                    ),
                    preset_help_popover(),
                ],
            ),
            div(
                vec![("class", "blocks-ai-chat-playground-preview")],
                vec![
                    div(
                        vec![("class", "blocks-ai-chat-playground-preview-label")],
                        vec![text("応答プレビュー")],
                    ),
                    message(
                        "あなた",
                        "四半期レポートの要点を 3 行で要約してください。読み手は非エンジニアの経営層です。",
                    ),
                    message(
                        "アシスタント",
                        "1) 売上は前四半期比 8% 増。2) 主因は新規契約の増加。3) 来期はコスト最適化が焦点です。",
                    ),
                ],
            ),
            prompt_field,
            div(
                vec![("class", "blocks-ai-chat-playground-actions")],
                vec![
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Solid,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("送信")],
                    ),
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("クリア")],
                    ),
                ],
            ),
        ],
    );

    let settings_column = div(
        vec![
            ("class", "blocks-ai-chat-playground-settings"),
            ("aria-label", "生成設定"),
        ],
        vec![
            param_slider(SliderSpec {
                label_id: TEMPERATURE_LABEL_ID,
                hidden_name: "temperature",
                label_text: "温度",
                min: 0.0,
                max: 2.0,
                step: 0.1,
                value: 0.7,
                value_text: "0.7",
            }),
            param_slider(SliderSpec {
                label_id: MAX_TOKENS_LABEL_ID,
                hidden_name: "max-tokens",
                label_text: "最大トークン数",
                min: 0.0,
                max: 4096.0,
                step: 64.0,
                value: 1024.0,
                value_text: "1024",
            }),
            param_slider(SliderSpec {
                label_id: TOP_P_LABEL_ID,
                hidden_name: "top-p",
                label_text: "Top P",
                min: 0.0,
                max: 1.0,
                step: 0.05,
                value: 0.9,
                value_text: "0.9",
            }),
            param_switch("streaming", "ストリーミング応答", true),
            param_switch("include-system-prompt", "システムプロンプトを含める", false),
        ],
    );

    div(
        vec![("class", "blocks-ai-chat-playground-layout")],
        vec![main_column, settings_column],
    )
}
```

## 原案差分メモ

- 主参照 R0005（代表構成）のみを実装し、並記する版はありません。
- 左カラム: モデル選択・プリセット選択（+ プリセット説明の popover）→
  応答プレビュー → プロンプト入力欄の縦並びです。右カラム: 温度・最大
  トークン数・Top P のスライダーと、ストリーミング応答・システム
  プロンプト同梱の switch です。
- 狭幅（コンテナ幅 48rem 未満）では設定カラムが本文カラムの下段へ回ります
  （`@container` によるコンテナクエリ判定、開閉パネルではなく 1 列化で
  対応します）。
- popover は無 JS のため常時開いた説明カード（trigger は `disabled`）として
  表示し、位置決め用の `positioner` は本 block のレイアウト CSS で
  `position: static` に上書きしてインライン表示しています。
- 生成中の状態（スピナー・ストリーミング途中）は表現していません。

関連情報: [Field](../themes/field.md) / [Native Select](../themes/native-select.md) /
[Textarea](../themes/textarea.md) / [Slider](../themes/slider.md) /
[Switch](../themes/switch.md) / [Button](../themes/button.md) /
[Popover](../themes/popover.md)
