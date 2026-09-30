# settings-preferences

表示テーマ・文字サイズ・配置をラジオカード群で、言語・地域・タイムゾーン・
日付形式・通貨をネイティブセレクトで選ばせる環境設定画面のブロックです。
`radio-card` / `native-select` / `switch` / `field` / `button` / `heading`
の 6 部品を合成します。Blocks は既存部品の合成例であり、新しい UI 部品は
追加しません。

主参照は対応表 ID R0235（代表構成）と R0257（選択欄 5 つの言語・地域設定）
を集約しています（`_/blocks-intake/` の対応ファイルは本 worktree に存在しない
ため、対応表 ID のみを記載）。言語・地域・タイムゾーン・日付形式・通貨の選択肢は
すべて架空の一般語であり、実在の企業・人物・メールアドレス・クレデンシャル・
PII を含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。保存ボタンは
`type="button"` で送信先を持ちません。全ラジオカード・セレクト・スイッチは
ネイティブ `disabled` で操作自体を封じています（クリックでネイティブ state
のみ変わり render 時固定の状態と乖離するのを防ぐため）。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldIds, FieldProps, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::native_select::{native_select, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::radio_card::{self, Orientation};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// `field_id`/ラジオカードグループの一意な `id` を組み立てる（版 A/B で
/// 衝突させない。`blocks_contract::demo_output_has_no_dangling_aria_references_or_duplicate_ids`
/// が同一 Demo 内の id 重複を検査する）。
fn field_id(version: &str, name: &str) -> String {
    format!("blocks-settings-preferences-{version}-{name}")
}

/// 1 件のネイティブセレクト欄（ラベル + セレクト + 補足の `field` 合成）を
/// 組み立てる。
fn select_field(
    id: &str,
    label_text: &str,
    helper: &str,
    options: &[(&str, &str)],
    selected: &str,
) -> Node {
    let props = FieldProps {
        id,
        ids: FieldIds::default(),
        // 無 JS 静的デモの契約（モジュール doc「初期状態は固定（無 JS）」節）:
        // クリックでネイティブ state のみ変わり render 時固定の data-state と
        // 乖離することを防ぐため、ネイティブ disabled で操作自体を封じる
        // （`settings_notification_matrix` の checkbox と同じ判断）。
        disabled: true,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: true,
    };
    let opts: Vec<Node> = options
        .iter()
        .map(|(value, label)| {
            let mut attrs = vec![("value", *value)];
            if *value == selected {
                attrs.push(("selected", ""));
            }
            el("option", attrs, vec![text(*label)])
        })
        .collect();
    field::root(
        &FieldRootProps::default(),
        &props,
        vec![],
        vec![
            field::label(&props, vec![], vec![text(label_text)]),
            native_select(&NativeSelectProps::default(), &props, vec![], opts),
            field::helper_text(&props, vec![], vec![text(helper)]),
        ],
    )
}

/// 1 群のラジオカード（グループ見出し + 選択肢カード列）を組み立てる。
/// `items` は `(value, label, description, checked)` の並び。
fn radio_card_group(
    group_id: &str,
    name: &str,
    title: &str,
    items: &[(&str, &str, &str, bool)],
) -> Node {
    let mut children = vec![radio_card::label(Some(group_id), vec![], vec![text(title)])];
    // 無 JS 静的デモの契約（モジュール doc「初期状態は固定（無 JS）」節）:
    // クリックでネイティブ state のみ変わり render 時固定の data-state と
    // 乖離することを防ぐため、各カードをネイティブ disabled にする
    // （`settings_notification_matrix` の checkbox と同じ判断）。
    children.extend(items.iter().map(|(value, label, description, checked)| {
        radio_card::item(
            *checked,
            true,
            value,
            vec![],
            vec![
                radio_card::item_hidden_input(*checked, true, Some(name), value, vec![]),
                radio_card::item_control(
                    *checked,
                    true,
                    vec![],
                    vec![
                        radio_card::item_content(
                            vec![],
                            vec![
                                radio_card::item_text(vec![], vec![text(*label)]),
                                radio_card::item_description(vec![], vec![text(*description)]),
                            ],
                        ),
                        radio_card::item_indicator(*checked, true, false, vec![]),
                    ],
                ),
            ],
        )
    }));
    radio_card::root(
        Size::Md,
        ColorPalette::Accent,
        false,
        Some(Orientation::Horizontal),
        Some(group_id),
        vec![("data-blocks-settings-preferences-cards", "")],
        children,
    )
}

/// 1 行のスイッチ（見出し・説明文とスイッチ本体を横並びにする）。
fn switch_row(name: &str, label_text: &str, description: &str, checked: bool) -> Node {
    let props = SwitchProps {
        // 無 JS 静的デモの契約（モジュール doc「初期状態は固定（無 JS）」節）:
        // クリックでネイティブ state のみ変わり render 時固定の data-state と
        // 乖離することを防ぐため、ネイティブ disabled で操作自体を封じる
        // （`settings_notification_matrix` の checkbox と同じ判断）。
        disabled: true,
        ..SwitchProps::default()
    };
    div(
        vec![("class", "blocks-settings-preferences-switch-row")],
        vec![
            div(
                vec![("class", "blocks-settings-preferences-group")],
                vec![
                    div(vec![], vec![text(label_text)]),
                    div(
                        vec![("class", "blocks-settings-preferences-description")],
                        vec![text(description)],
                    ),
                ],
            ),
            switch::root(
                Size::Md,
                ColorPalette::Accent,
                checked,
                &props,
                vec![],
                vec![
                    // `switch::root` の兄弟要素にラベルを置いても `<label>`
                    // 内の入力名として認識されないため（codex-review 指摘）、
                    // `hidden_input` へ直接 `aria-label` を付与しアクセシブル
                    // ネームを確定させる（見出しテキストは switch-row の
                    // 兄弟 div に残し、見た目の配置は変えない）。
                    switch::hidden_input(
                        name,
                        "on",
                        checked,
                        &props,
                        vec![("aria-label", label_text)],
                    ),
                    switch::control(
                        checked,
                        &props,
                        vec![],
                        vec![switch::thumb(checked, &props, vec![], vec![])],
                    ),
                ],
            ),
        ],
    )
}

/// 版 A（代表構成、R0235）: テーマ・文字サイズ・配置のラジオカード群 3 つ、
/// 言語・タイムゾーンのセレクト 2 つ、アニメーション低減のスイッチ 1 つ。
fn version_representative() -> Node {
    let theme_group = radio_card_group(
        &field_id("a", "theme-group"),
        "settings-preferences-a-theme",
        "テーマ",
        &[
            ("light", "ライト", "明るい配色で表示します。", false),
            ("dark", "ダーク", "暗い配色で表示します。", false),
            (
                "system",
                "システムに従う",
                "OS の設定に合わせて自動で切り替えます。",
                true,
            ),
        ],
    );
    let font_size_group = radio_card_group(
        &field_id("a", "font-size-group"),
        "settings-preferences-a-font-size",
        "文字サイズ",
        &[
            ("small", "小", "本文の文字を小さく表示します。", false),
            ("standard", "標準", "既定の文字サイズです。", true),
            ("large", "大", "本文の文字を大きく表示します。", false),
        ],
    );
    let density_group = radio_card_group(
        &field_id("a", "density-group"),
        "settings-preferences-a-density",
        "配置",
        &[
            (
                "compact",
                "コンパクト",
                "余白を減らして密に表示します。",
                false,
            ),
            ("standard", "標準", "既定の余白で表示します。", true),
            ("comfortable", "ゆったり", "余白を広めに表示します。", false),
        ],
    );

    let language = select_field(
        &field_id("a", "language"),
        "言語",
        "画面表示に使う言語です。",
        &[("ja", "日本語"), ("en", "English"), ("de", "Deutsch")],
        "ja",
    );
    let timezone = select_field(
        &field_id("a", "timezone"),
        "タイムゾーン",
        "日時の表示に使うタイムゾーンです。",
        &[
            ("asia-tokyo", "Asia/Tokyo"),
            ("utc", "UTC"),
            ("america-los-angeles", "America/Los_Angeles"),
        ],
        "asia-tokyo",
    );

    let reduce_motion = switch_row(
        "settings-preferences-a-reduce-motion",
        "アニメーションを減らす",
        "画面遷移・演出のアニメーションを最小限にします。",
        false,
    );

    div(
        vec![("class", "blocks-settings-preferences-section")],
        vec![
            heading(
                HeadingLevel::H2,
                &HeadingProps::default(),
                vec![],
                vec![text("表示と言語")],
            ),
            div(
                vec![("class", "blocks-settings-preferences-group")],
                vec![theme_group, font_size_group, density_group],
            ),
            div(
                vec![("class", "blocks-settings-preferences-group")],
                vec![language, timezone, reduce_motion],
            ),
            div(
                vec![("class", "blocks-settings-preferences-actions")],
                vec![button(
                    &ButtonProps::default(),
                    vec![],
                    vec![text("設定を保存")],
                )],
            ),
        ],
    )
}

/// 版 B（言語・地域設定、R0257）: 言語・地域・タイムゾーン・日付形式・通貨の
/// セレクト 5 つ。
fn version_locale_five_selects() -> Node {
    let language = select_field(
        &field_id("b", "language"),
        "言語",
        "画面表示に使う言語です。",
        &[("ja", "日本語"), ("en", "English"), ("de", "Deutsch")],
        "ja",
    );
    let region = select_field(
        &field_id("b", "region"),
        "地域",
        "通貨・単位系の既定値に使う地域です。",
        &[("jp", "日本"), ("us", "アメリカ合衆国"), ("de", "ドイツ")],
        "jp",
    );
    let timezone = select_field(
        &field_id("b", "timezone"),
        "タイムゾーン",
        "日時の表示に使うタイムゾーンです。",
        &[
            ("asia-tokyo", "Asia/Tokyo"),
            ("utc", "UTC"),
            ("america-los-angeles", "America/Los_Angeles"),
        ],
        "asia-tokyo",
    );
    let date_format = select_field(
        &field_id("b", "date-format"),
        "日付形式",
        "一覧・詳細画面で日付を表示する形式です。",
        &[
            ("iso", "2026-09-30"),
            ("slash", "2026/09/30"),
            ("dmy", "30 Sep 2026"),
        ],
        "iso",
    );
    let currency = select_field(
        &field_id("b", "currency"),
        "通貨",
        "金額の表示に使う通貨単位です。",
        &[("jpy", "JPY"), ("usd", "USD"), ("eur", "EUR")],
        "jpy",
    );

    div(
        vec![("class", "blocks-settings-preferences-section")],
        vec![
            heading(
                HeadingLevel::H2,
                &HeadingProps::default(),
                vec![],
                vec![text("言語と地域")],
            ),
            div(
                vec![("class", "blocks-settings-preferences-group")],
                vec![language, region, timezone, date_format, currency],
            ),
            div(
                vec![("class", "blocks-settings-preferences-actions")],
                vec![button(
                    &ButtonProps::default(),
                    vec![],
                    vec![text("設定を保存")],
                )],
            ),
        ],
    )
}

/// `settings-preferences` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-preferences-stack")],
        vec![version_representative(), version_locale_five_selects()],
    )
}
```

## 原案差分メモ

- 版 A（R0235: 代表構成）はテーマ・文字サイズ・配置のラジオカード 3 群 +
  言語・タイムゾーンのセレクト 2 つ + アニメーション低減のスイッチ 1 つ +
  保存ボタンです。
- 版 B（R0257: 言語・地域設定）は言語・地域・タイムゾーン・日付形式・通貨の
  セレクト 5 つ + 保存ボタンのみのシンプルな構成です。
- 狭幅（コンテナ幅 36rem 未満）ではラジオカード群が 1 列に積まれ、スイッチ行
  も縦積みに切り替わります。いずれも本 block 側の CSS（`@container`）が担って
  おり、`radio-card`/`switch` 部品自体の機能ではありません。
- 初期状態（選択中のテーマ・文字サイズ・配置・言語等）はすべて固定値です
  （無 JS のため動的な状態遷移は持ちません）。
- 全ラジオカード・セレクト・スイッチはネイティブ `disabled` にしています。
  クリックでネイティブ state のみ変わり render 時固定の状態と乖離するのを
  防ぐための判断です（`settings-notification-matrix` の checkbox と同じ）。

関連情報: [Radio Card](../themes/radio-card.md) /
[Native Select](../themes/native-select.md) / [Switch](../themes/switch.md) /
[Field](../themes/field.md) / [Button](../themes/button.md) /
[Heading](../themes/heading.md)
