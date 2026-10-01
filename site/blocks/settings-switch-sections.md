# settings-switch-sections

見出し付きセクション内に、ラベル・説明・右端スイッチの行を区切り線で
並べ、末尾に保存ボタンを置く設定画面の定番レイアウトです。
`switch` / `field` / `fieldset` / `radio_group` / `card` / `button` /
`separator` / `heading` / `kbd` の 9 部品を合成します。Blocks は既存部品の
合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0260 です。2 セクション構成（R0261）・条件選択の
ラジオ群を差し込む版（R0259）・ショートカット案内を差し込む版
（R0231）・カード内に収める版（R0026）の 4 版を、下記 3 variant として
まとめて掲載します。セクション名・設定項目名・説明文はすべて架空のもので、
実在の人物・企業・PII は含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。送信処理・
送信先は持たず、スイッチ・ラジオはすべてネイティブ `disabled` で固定した
静的表示です（ブラウザのネイティブ切り替えと SSR 表示のずれを防ぐため）。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::kbd::{self, KbdProps};
use fandhe_frontend_pre_styled_ui::radio_group::{self, RadioGroupProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::{
    ColorPalette, FieldOrientation, FieldRootProps, FieldsetRootProps, Size,
};

/// セクション見出し（H3、`Lg`/`Semibold` で本文より一段目立たせる）。
fn section_heading(title: &'static str) -> Node {
    heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Lg,
            weight: HeadingWeight::Semibold,
        },
        vec![],
        vec![text(title)],
    )
}

/// 行間の区切り線。`separator::separator` は `class` を除去するため
/// `data-*` でフックする（モジュール doc「`class` と `data-*` の使い分け」
/// 節参照）。
fn row_separator() -> Node {
    separator::separator(
        &SeparatorProps::default(),
        vec![("data-blocks-settings-switch-sections-separator", "")],
    )
}

/// 「ラベル + 説明 + 右端スイッチ」の 1 行を組み立てる。`instance`
/// （variant 名）と `key`（行を一意にする短い識別子）から id を導出する
/// （モジュール doc「id の一意化」節参照）。
fn switch_row(
    instance: &str,
    key: &str,
    title: &'static str,
    description: &'static str,
    checked: bool,
) -> Node {
    let id = format!("blocks-settings-switch-sections-{instance}-{key}");
    // `field::label`/`field::helper_text` は `FieldProps::id` から
    // `"{id}-control"`/`"{id}-helper-text"` を決定的に導出する
    // （headless `crates/headless-ui/src/field.rs` の既定規則）。switch 側の
    // `hidden_input` id・`aria-describedby` はここで同じ値を組み立てて渡す。
    let control_id = format!("{id}-control");
    let helper_id = format!("{id}-helper-text");
    let field_props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: true,
    };
    let switch_props = SwitchProps {
        disabled: true,
        ..SwitchProps::default()
    };
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Horizontal,
        },
        &field_props,
        vec![("data-blocks-settings-switch-sections-row", "")],
        vec![
            div(
                vec![("class", "blocks-settings-switch-sections-row-text")],
                vec![
                    field::label(&field_props, vec![], vec![text(title)]),
                    field::helper_text(&field_props, vec![], vec![text(description)]),
                ],
            ),
            switch::root(
                Size::Md,
                ColorPalette::Accent,
                checked,
                &switch_props,
                vec![("data-blocks-settings-switch-sections-switch", "")],
                vec![
                    switch::hidden_input(
                        &id,
                        "on",
                        checked,
                        &switch_props,
                        vec![
                            ("id", control_id.as_str()),
                            ("aria-describedby", helper_id.as_str()),
                        ],
                    ),
                    switch::control(
                        checked,
                        &switch_props,
                        vec![],
                        vec![switch::thumb(checked, &switch_props, vec![], vec![])],
                    ),
                ],
            ),
        ],
    )
}

/// セクション（見出し + 行群、行の間に区切り線を挟む。末尾には置かない）。
fn section(title: &'static str, rows: Vec<Node>) -> Node {
    let last_idx = rows.len().saturating_sub(1);
    let mut children = vec![section_heading(title)];
    for (idx, row) in rows.into_iter().enumerate() {
        children.push(row);
        if idx != last_idx {
            children.push(row_separator());
        }
    }
    div(
        vec![("class", "blocks-settings-switch-sections-section")],
        children,
    )
}

/// 保存ボタン。`<form>` を持たないため `button::button` の既定
/// `type="button"` のまま用いる（送信処理は持たない）。
fn save_actions() -> Node {
    div(
        vec![("class", "blocks-settings-switch-sections-actions")],
        vec![button::button(
            &ButtonProps::default(),
            vec![],
            vec![text("保存")],
        )],
    )
}

/// `basic` variant（R0260 + R0261）: 「通知」「セキュリティ」の 2 セクション。
fn basic_instance() -> Node {
    div(
        vec![("data-blocks-settings-switch-sections-variant", "basic")],
        vec![
            section(
                "通知",
                vec![
                    switch_row(
                        "basic",
                        "email",
                        "メール通知",
                        "重要な更新をメールでお知らせします",
                        true,
                    ),
                    switch_row(
                        "basic",
                        "push",
                        "プッシュ通知",
                        "デバイスへプッシュ通知を送信します",
                        false,
                    ),
                    switch_row(
                        "basic",
                        "digest",
                        "週次ダイジェスト",
                        "1 週間の活動をまとめて通知します",
                        false,
                    ),
                ],
            ),
            section(
                "セキュリティ",
                vec![
                    switch_row(
                        "basic",
                        "two-factor",
                        "2 段階認証",
                        "ログイン時に確認コードを要求します",
                        true,
                    ),
                    switch_row(
                        "basic",
                        "new-device",
                        "新しい端末からのサインイン通知",
                        "未知の端末からのログインを通知します",
                        true,
                    ),
                ],
            ),
            save_actions(),
        ],
    )
}

/// メンション通知条件のラジオ 1 件。
fn mention_condition_item(
    checked: bool,
    props: &RadioGroupProps,
    value: &'static str,
    label: &'static str,
) -> Node {
    radio_group::item(
        checked,
        props,
        value,
        vec![("data-blocks-settings-switch-sections-radio-item", "")],
        vec![
            radio_group::item_hidden_input(
                checked,
                props,
                Some("blocks-settings-switch-sections-mention-condition"),
                value,
                vec![],
            ),
            radio_group::item_control(checked, props, vec![]),
            radio_group::item_text(checked, props, vec![], vec![text(label)]),
        ],
    )
}

/// メンション通知の条件を選ぶラジオ群（`fieldset` + `radio_group`）。
fn mention_conditions_fieldset() -> Node {
    let fieldset_id = "blocks-settings-switch-sections-detailed-conditions";
    let legend_id = format!("{fieldset_id}-legend");
    let fieldset_props = FieldsetProps {
        id: fieldset_id,
        disabled: false,
        invalid: false,
        has_helper_text: false,
    };
    let radio_props = RadioGroupProps {
        disabled: true,
        ..RadioGroupProps::default()
    };
    fieldset::root(
        &FieldsetRootProps::default(),
        &fieldset_props,
        vec![("data-blocks-settings-switch-sections-fieldset", "")],
        vec![
            fieldset::legend(&fieldset_props, vec![], vec![text("通知する条件")]),
            radio_group::root(
                Size::Md,
                ColorPalette::Accent,
                true,
                None,
                Some(legend_id.as_str()),
                vec![("data-blocks-settings-switch-sections-radio-group", "")],
                vec![
                    mention_condition_item(true, &radio_props, "all", "すべて"),
                    mention_condition_item(false, &radio_props, "mentions", "自分宛のみ"),
                    mention_condition_item(false, &radio_props, "none", "なし"),
                ],
            ),
        ],
    )
}

/// キーボードショートカットの案内行（`kbd`）。
fn shortcut_hint_row() -> Node {
    div(
        vec![("class", "blocks-settings-switch-sections-row-text")],
        vec![
            div(
                vec![("class", "blocks-settings-switch-sections-shortcut-label")],
                vec![text("キーボードショートカット")],
            ),
            div(
                vec![("class", "blocks-settings-switch-sections-description")],
                vec![
                    text("切り替えのショートカット例（この静的レイアウト例では操作できません）: "),
                    kbd::group(
                        vec![],
                        vec![
                            kbd::kbd(&KbdProps::default(), vec![], vec![text("⌘")]),
                            text("+"),
                            kbd::kbd(&KbdProps::default(), vec![], vec![text("K")]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// `detailed` variant（R0259 + R0231）: 「メンション」セクション 1 つに
/// ラジオ群とショートカット案内を差し込む。
fn detailed_instance() -> Node {
    div(
        vec![("data-blocks-settings-switch-sections-variant", "detailed")],
        vec![
            div(
                vec![("class", "blocks-settings-switch-sections-section")],
                vec![
                    section_heading("メンション"),
                    switch_row(
                        "detailed",
                        "mention",
                        "メンション通知",
                        "自分へのメンションを通知します",
                        true,
                    ),
                    row_separator(),
                    mention_conditions_fieldset(),
                    row_separator(),
                    shortcut_hint_row(),
                ],
            ),
            save_actions(),
        ],
    )
}

/// `card` variant（R0026）: 同じ行構成を `card` の中に収める。
fn card_instance() -> Node {
    div(
        vec![("data-blocks-settings-switch-sections-variant", "card")],
        vec![card::root(
            CardProps::default(),
            vec![("data-blocks-settings-switch-sections-card", "")],
            vec![
                card::header(
                    vec![],
                    vec![
                        section_heading("表示設定"),
                        div(
                            vec![("class", "blocks-settings-switch-sections-description")],
                            vec![text(
                                "このカードは静的レイアウト例です（スイッチは操作できません）。",
                            )],
                        ),
                    ],
                ),
                card::body(
                    vec![("class", "blocks-settings-switch-sections-section")],
                    vec![
                        switch_row(
                            "card",
                            "compact",
                            "コンパクト表示",
                            "余白を詰めて一覧性を高めます",
                            false,
                        ),
                        row_separator(),
                        switch_row(
                            "card",
                            "auto-save",
                            "自動保存",
                            "変更を自動的に保存します",
                            true,
                        ),
                    ],
                ),
                card::footer(
                    vec![("class", "blocks-settings-switch-sections-actions")],
                    vec![button::button(
                        &ButtonProps::default(),
                        vec![],
                        vec![text("保存")],
                    )],
                ),
            ],
        )],
    )
}

/// `settings-switch-sections` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。ルートの `class`（`-layout`）は [`BLOCK::demo_class`]
/// （`blocks-settings-switch-sections`）とは異なる名前にする（他 block と
/// 同じ規約、CSS セレクタの衝突防止）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-switch-sections-layout")],
        vec![basic_instance(), detailed_instance(), card_instance()],
    )
}
```

## 原案差分メモ

- `basic`（R0260 + R0261）: 「通知」「セキュリティ」の 2 セクション構成。
  主参照 R0260 の単一セクション構成に、R0261 の複数セクション構成を
  重ねています。
- `detailed`（R0259 + R0231）: 「メンション」1 セクションに、R0259 の
  条件選択ラジオ群（`fieldset` + `radio_group`）と、R0231 の
  キーボードショートカット案内（`kbd`）を差し込んでいます。
- `card`（R0026）: 同じ行構成を `card::root`/`header`/`body`/`footer` の
  中に収めています。
- 参照ファイル置き場（`_/blocks-intake/`）が本イシュー着手時点で存在
  しなかったため、レイアウトはイシューのレイアウト仕様文のみから設計
  しました。実機ブラウザでの見た目確認（ライト/ダーク・狭幅）は未実施
  です。
- スイッチ・ラジオはすべて `disabled: true` のネイティブ固定です。無 JS
  の SSR ではこれらの見た目（`data-state`）は初期描画のまま変化しない
  ため、操作可能に見えて実は無反応という不整合を避けるための判断です
  （`settings-notification-matrix`/`form-layout-stacked` と同型）。

関連情報: [Switch](../themes/switch.md) / [Field](../themes/field.md) /
[Fieldset](../themes/fieldset.md) /
[Radio Group](../themes/radio-group.md) / [Card](../themes/card.md) /
[Button](../themes/button.md) / [Separator](../themes/separator.md) /
[Heading](../themes/heading.md) / [Kbd](../themes/kbd.md)
