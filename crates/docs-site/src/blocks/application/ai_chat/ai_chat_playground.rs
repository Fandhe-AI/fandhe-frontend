//! `ai-chat-playground` block（イシュー #2960。Application / AI Chat
//! カテゴリ、最初の block）。モデル・プリセット選択 → 応答プレビュー →
//! プロンプト入力欄を左カラムへ縦積みし、右カラムへ生成パラメータ
//! （温度・最大トークン数・Top P のスライダーとストリーミング/システム
//! プロンプト同梱の switch）を並べたモデル実験用プレイグラウンドを合成する。
//! 主参照は対応表 ID R0005（代表構成）のみ。`_/blocks-intake/` の対応
//! ファイルは本イシュー着手時点で本 worktree に存在しないため、原稿・本
//! コメントには対応表 ID のみを記す（`profile_detail_datalist`〔イシュー
//! #2937〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `field` / `native-select` / `textarea` / `slider` / `switch` / `button` /
//! `popover` の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 狭幅では設定カラムを下段へ回す（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため `@container`（コンテナ
//! クエリ）で判定する（`profile_detail_datalist` 等と同型のパターン）。
//! コンテナクエリは `container-type` を持つ要素自身を再スタイル対象に
//! できない（Bugbot/codex レビュー指摘、コンテナ自身は不変のまま子孫の
//! みが再評価される）ため、[`LAYOUT_CSS`] のルート
//! `.blocks-ai-chat-playground-layout` へ `container-type: inline-size` を
//! 宣言し、実際にグリッド化する子要素 `.blocks-ai-chat-playground-grid`
//! を別要素として分離する。コンテナ幅が `48rem` 未満のとき `-grid` 側の
//! 2 列グリッドを 1 列へ切り替える。設定カラムは DOM 順で本文カラムの
//! 後ろに置くため、`order` の上書きは不要（開閉パネル案は無 JS のため
//! 開閉できず不採用、1 列化のみで対応する）。
//!
//! # popover はプリセット説明を常時開いた静的カードとして表示する
//!
//! docs サイトは JS ハイドレーションを一切行わないため、popover の開閉は
//! 実演できない。[`fandhe_frontend_pre_styled_ui::popover::trigger`] を
//! `disabled: true` + 常時 `OpenState::Open` で出し、無 JS では押しても
//! 何も起きないボタンを操作可能に見せない（`form_layout_property_panel`
//! の tooltip trigger と同じ判断）。[`fandhe_frontend_pre_styled_ui::popover`]
//! の `positioner` は素の再エクスポートで `position: absolute; top: 100%`
//! の overlay として描画されるため、[`LAYOUT_CSS`] で本 block のルート配下
//! に限定したセレクタへ `position: static` を上書きし、説明カードとして
//! 本文の流れへインライン表示する（他 block・他ページの popover 見た目へ
//! 波及しない）。`.blocks-ai-chat-playground-selectors`（`align-items:
//! flex-end` の flex row）内では popover の `root`（trigger + positioner
//! を子に持つ既定 `display: block`）が単一の flex item となり、
//! `position: static` 化した説明カードの高さが item の cross-size へ
//! 合算されるため、広い viewport で select 欄・trigger の下端が説明
//! カード分だけずれる（Bugbot レビュー指摘）。[`LAYOUT_CSS`] は
//! `.selectors` 配下の popover `root` を `display: contents` で透過させ
//! `trigger`/`positioner` を `.selectors` 自身の直接の flex item へ昇格
//! させたうえで、`positioner` に `flex-basis: 100%` を与えて常に独立した
//! 折り返し行へ送る（flex-wrap の定番手法）。これにより 1 行目
//! （モデル・プリセット select + trigger）は `align-items: flex-end` の
//! まま高さを揃え、説明カードは 2 行目として独立した cross-size で描画
//! される。[`fandhe_frontend_pre_styled_ui::field::root`] の base 宣言は
//! `width: 100%` を持つため、`.selectors` 配下では model/preset の
//! `select_field` が生成する `field::root` 自身も flex item として
//! 「コンテナ幅 100%」を basis に取り、2 件目が同じ行に収まらず折り返す
//! （Bugbot レビュー指摘）。[`LAYOUT_CSS`] は `.selectors` 配下の
//! `field::root`（`data-scope="field"][data-part="root"]`）へ `width: auto`
//! と `flex: 1 1 10rem` を上書きし、model/preset select が 1 行目で並んで
//! 伸縮する通常の flex item として振る舞うようにする。
//!
//! # `<form>` を使わない・生成中状態を表現しない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先・API キー入力欄を一切持たな
//! い。ボタンは [`fandhe_frontend_pre_styled_ui::button::button`] の既定
//! `type="button"` のまま用いるが、`disabled: true` を指定し操作不能な
//! ボタンとして表示する（codex レビュー指摘。popover trigger と同じく
//! 「押しても何も起きない有効ボタン」を作らず、静的操作例であることを
//! 明示する）。生成中（スピナー・ストリーミング途中）の状態も表現しない
//! （入力済みプロンプトと応答例の固定表示のみ）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `field::root` / `native_select::native_select` / `textarea::textarea` /
//! `slider::root` / `switch::root` / `button::button` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、これらへ `class` 属性の CSS フックは付与しない（本
//! block はレイアウト用ラッパーのみへスタイルを与え、部品自体の見た目は
//! 変更しない）。`field::root` の `.selectors` 内幅上書き（上記「popover
//! はプリセット説明を常時開いた静的カードとして表示する」節末尾）は
//! `class` ではなく popover 側と同じ `[data-scope="field"][data-part=
//! "root"]` 属性セレクタで行うため、この `drop_class_attr` 契約と矛盾
//! しない。`popover` は headless anatomy を素の再エクスポートで使うため
//! `class` 属性がそのまま出力される。レイアウト用ラッパー（素の `<div>`）は
//! `class="blocks-ai-chat-playground-*"` を使う。ルート class
//! （`blocks-ai-chat-playground-layout`）は [`Block::demo_class`]
//! （`blocks-ai-chat-playground`）とは意図的に別名にする（`profile_detail_
//! datalist` 等と同じ Bugbot 教訓の回避）。
//!
//! # ダミー素材について
//!
//! モデル名・プリセット名・プロンプト・応答例はすべて本モジュール内で作成
//! した架空の文言であり、実在ベンダーのモデル名・実クレデンシャル・PII を
//! 含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
        // 無 JS の静的デモではプリセット説明文どおり選択欄の値が変わらない
        // ため `disabled: true` にする（codex レビュー指摘、PR #3412）。
        disabled: true,
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
                                "プリセットは、右側の生成設定（温度・最大トークン数・Top P 等）の組み合わせ例です。本ページは静的な構成例のため、選択欄・生成設定は固定表示で切り替わりません。",
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
    // 無 JS の静的デモではキー操作をしても値が変わらないため、`disabled: true`
    // で thumb を `tabindex="-1"` にしフォーカス不能にする（codex レビュー
    // 指摘。popover trigger・button と同じく「操作可能に見える静的要素」を
    // 作らない判断）。
    let props = SliderProps {
        disabled: true,
        ..SliderProps::default()
    };
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
///
/// `disabled: true` を指定する（Bugbot レビュー指摘）。docs サイトは JS
/// ハイドレーションを行わないため、`disabled` なしでは hidden checkbox が
/// native トグル可能なまま残り、クリックで checked state だけが変化して
/// track/thumb の SSR 時点 `data-state`（本 Demo は変化しない固定表示）と
/// 乖離し、支援技術が視覚と異なる on/off を読み上げる。ボタン・popover
/// trigger を `disabled: true` にした判断（モジュール doc「`<form>` を
/// 使わない・生成中状態を表現しない」節）と同じ理由で switch も操作不能に
/// する。
fn param_switch(hidden_name: &'static str, label_text: &'static str, checked: bool) -> Node {
    let props = SwitchProps {
        disabled: true,
        ..SwitchProps::default()
    };
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
        // 応答プレビューは固定表示のため、入力文を編集不能にして両者の
        // 見た目と実際の編集可否を一致させる（codex レビュー指摘、PR #3412）。
        readonly: true,
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
                            disabled: true,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("送信")],
                    ),
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            disabled: true,
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
            ("role", "group"),
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
        vec![div(
            vec![("class", "blocks-ai-chat-playground-grid")],
            vec![main_column, settings_column],
        )],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/ai-chat-playground/",
    title: "ai-chat-playground",
    category: BlockCategory::AiChat,
    rust_source: "crates/docs-site/src/blocks/application/ai_chat/ai_chat_playground.rs",
    demo_class: "blocks-ai-chat-playground",
    parts: &[
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Native Select",
            path: "/themes/native-select/",
        },
        Part {
            label: "Textarea",
            path: "/themes/textarea/",
        },
        Part {
            label: "Slider",
            path: "/themes/slider/",
        },
        Part {
            label: "Switch",
            path: "/themes/switch/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Popover",
            path: "/themes/popover/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `ai_chat_playground` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-ai-chat-playground-layout {\n  container-type: inline-size;\n  container-name: blocks-ai-chat-playground;\n}\n\
.blocks-ai-chat-playground-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr) 18rem;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-ai-chat-playground-main {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-5);\n  min-width: 0;\n}\n\
.blocks-ai-chat-playground-settings {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-5);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: 0.5rem;\n  padding: var(--fandhe-space-5);\n}\n\
.blocks-ai-chat-playground-selectors {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-4);\n  align-items: flex-end;\n}\n\
.blocks-ai-chat-playground-selectors [data-scope=\"field\"][data-part=\"root\"] {\n  width: auto;\n  flex: 1 1 10rem;\n  min-width: 0;\n}\n\
.blocks-ai-chat-playground-preview {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: 0.5rem;\n  padding: var(--fandhe-space-4);\n  background: var(--fandhe-color-bg);\n}\n\
.blocks-ai-chat-playground-preview-label {\n  font-weight: 600;\n  color: var(--fandhe-color-fg-muted, var(--fandhe-color-fg));\n}\n\
.blocks-ai-chat-playground-message {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-ai-chat-playground-message-speaker {\n  font-weight: 600;\n}\n\
.blocks-ai-chat-playground-actions {\n  display: flex;\n  gap: var(--fandhe-space-2);\n  justify-content: flex-end;\n}\n\
.blocks-ai-chat-playground-selectors [data-scope=\"popover\"][data-part=\"root\"] {\n  display: contents;\n}\n\
.blocks-ai-chat-playground-layout [data-scope=\"popover\"][data-part=\"positioner\"] {\n  position: static;\n  flex-basis: 100%;\n  margin-top: var(--fandhe-space-2);\n}\n\
@container blocks-ai-chat-playground (max-width: 48rem) {\n  \
.blocks-ai-chat-playground-grid {\n    grid-template-columns: minmax(0, 1fr);\n  }\n\
}\n";

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
            "data-scope=\"field\"",
            "data-scope=\"slider\"",
            "data-scope=\"switch\"",
            "data-scope=\"button\"",
            "data-scope=\"popover\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<select").count(), 2);
        assert_eq!(html.matches("<textarea").count(), 1);
        assert_eq!(
            html.matches("data-scope=\"slider\" data-part=\"root\"")
                .count(),
            3
        );
        assert_eq!(
            html.matches("data-scope=\"switch\" data-part=\"root\"")
                .count(),
            2
        );
        assert_eq!(html.matches("<button").count(), 3);
        assert_eq!(html.matches(r#"type="button""#).count(), 3);
    }

    #[test]
    fn no_form_and_static_only() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("http://"));
        assert!(!html.contains("https://"));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains("disabled"));
    }

    #[test]
    fn layout_css_uses_tokens_and_scopes_popover_override() {
        assert!(LAYOUT_CSS.contains("--fandhe-space-"));
        assert!(LAYOUT_CSS.contains(
            ".blocks-ai-chat-playground-layout [data-scope=\"popover\"][data-part=\"positioner\"]"
        ));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// select_field が生成する `field::root` は base 宣言に `width: 100%`
    /// を持つため（`crates/pre-styled-ui/src/field.rs`）、`.selectors` 内で
    /// 上書きしないと model/preset select が同じ行に並ばない
    /// （Bugbot レビュー指摘、モジュール doc「popover はプリセット説明を
    /// 常時開いた静的カードとして表示する」節末尾）。
    #[test]
    fn layout_css_overrides_field_root_width_inside_selectors() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-ai-chat-playground-selectors [data-scope=\"field\"][data-part=\"root\"]"
        ));
        assert!(LAYOUT_CSS.contains("flex: 1 1 10rem"));
    }

    /// switch は無 JS では native トグル後も track/thumb の SSR `data-state`
    /// が変化せず視覚と乖離するため `disabled` にする（Bugbot レビュー
    /// 指摘）。hidden checkbox の native `disabled` 属性の有無で検証する。
    #[test]
    fn param_switches_are_disabled() {
        let html = demo_html();
        assert!(html.contains(
            "data-part=\"hidden-input\" data-state=\"checked\" data-disabled=\"\" \
             type=\"checkbox\" role=\"switch\" name=\"streaming\" value=\"on\" \
             checked=\"\" disabled=\"\""
        ));
        assert!(html.contains(
            "data-part=\"hidden-input\" data-state=\"unchecked\" data-disabled=\"\" \
             type=\"checkbox\" role=\"switch\" name=\"include-system-prompt\" \
             value=\"on\" disabled=\"\""
        ));
    }

    #[test]
    fn slider_thumbs_are_labelled_by_visible_labels() {
        let html = demo_html();
        for id in [
            super::TEMPERATURE_LABEL_ID,
            super::MAX_TOKENS_LABEL_ID,
            super::TOP_P_LABEL_ID,
        ] {
            assert!(
                html.contains(&format!("aria-labelledby=\"{id}\"")),
                "missing aria-labelledby for {id}"
            );
            assert!(html.contains(&format!("id=\"{id}\"")), "missing id {id}");
        }
    }
}
