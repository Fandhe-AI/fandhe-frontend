# settings-share-link

共有の有効化スイッチ・共有 URL のコピー欄・リンクコピー / プレビューの
ボタン群を持つ共有リンク設定カードです。`card` / `switch` / `clipboard` /
`button` / `button-group` / `radio-card` / `tabs` / `qr-code` / `select` /
`separator` / `input` / `input-group` の 12 部品を合成します。Blocks は
既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0318（代表構成）で、閲覧範囲の radio card（R0319）・
埋め込み/リンクの tabs 切替（R0320）・ドメイン接尾辞 + QR コード（R0321）
の 3 差分版を集約しています。共有 URL は架空の `.example` ドメインで、
実在ドメイン・秘密情報らしき文字列は含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::button_group::{self, Orientation as ButtonGroupOrientation};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::qr_code::{
    self, encode, ErrorCorrectionLevel, DEFAULT_QUIET_ZONE,
};
use fandhe_frontend_pre_styled_ui::radio_card::{self, Orientation as RadioCardOrientation};
use fandhe_frontend_pre_styled_ui::select::{self, OpenState, SelectProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation as TabsOrientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 全版共通の共有 URL（架空 `.example` ドメイン、モジュール doc「ダミー値・
/// ドメインは明白な架空パターン」節参照）。
const SHARE_URL: &str = "https://fandhe-frontend.example/share/o7pQ-report";

/// 共有の有効化 switch（版 A/B/C 共通、常時 checked 固定の静的表示）。
/// `id_prefix` はカードごとの id 一意性のため。
fn share_toggle(id_prefix: &'static str) -> Node {
    let switch_props = SwitchProps {
        disabled: true,
        ..SwitchProps::default()
    };
    let name = format!("{id_prefix}-toggle");
    switch::root(
        Size::Md,
        ColorPalette::Accent,
        true,
        &switch_props,
        vec![("data-blocks-settings-share-link-toggle", "")],
        vec![
            switch::label(true, &switch_props, vec![], vec![text("共有を有効にする")]),
            switch::hidden_input(&name, "on", true, &switch_props, vec![]),
            switch::control(
                true,
                &switch_props,
                vec![],
                vec![switch::thumb(true, &switch_props, vec![], vec![])],
            ),
        ],
    )
}

/// 共有 URL のコピー欄（版 A/B/C 共通。`clipboard` root は本関数の 1 個に
/// 限る、モジュール doc「Demo 内の `clipboard` root は 1 個に限る」節）。
fn share_url_clipboard(id_prefix: &'static str) -> Node {
    let input_id = format!("{id_prefix}-clipboard-input");
    clipboard::root(
        SHARE_URL,
        false,
        vec![("data-blocks-settings-share-link-clipboard", "")],
        vec![
            visually_hidden::root(
                vec![],
                vec![clipboard::label(
                    false,
                    Some(&input_id),
                    vec![],
                    vec![text("共有 URL")],
                )],
            ),
            clipboard::control(
                false,
                vec![],
                vec![
                    clipboard::input(SHARE_URL, false, vec![("id", &input_id)]),
                    clipboard::trigger(
                        false,
                        vec![],
                        vec![
                            clipboard::indicator(false, false, vec![], vec![text("コピー")]),
                            clipboard::indicator(true, false, vec![], vec![text("コピーしました")]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 共有 URL の非 clipboard 静的表示（版 B・版 C。モジュール doc「Demo 内の
/// `clipboard` root は 1 個に限る」節参照）。readonly の `input` +
/// `input-group` のコピーボタン（`disabled: true`）で構成する
/// （`settings_api_key_created::key_row` と同型）。
fn static_url_display(id_prefix: &'static str) -> Node {
    let input_id = format!("{id_prefix}-url-input");
    let field_props = FieldProps {
        id: &input_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: true,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &field_props,
        vec![],
        vec![
            field::label(&field_props, vec![], vec![text("共有 URL")]),
            input_group::root(
                &group_props,
                vec![("data-blocks-settings-share-link-url", "")],
                vec![
                    input::input(
                        &InputProps::default(),
                        &field_props,
                        vec![("value", SHARE_URL)],
                    ),
                    input_group::addon(
                        InputGroupAlign::InlineEnd,
                        &group_props,
                        vec![],
                        vec![input_group::button(
                            // `clipboard` scope の外側にあり `headless_clipboard`
                            // 配線が届かないため、押しても何も起きないことを
                            // `disabled: true` で明示する。
                            &InputGroupProps {
                                disabled: true,
                                ..group_props
                            },
                            vec![],
                            vec![text("コピー")],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// footer の操作 button-group（リンクをコピー / プレビュー、モジュール doc
/// 「静的固定」節: 実アプリで機能する `clipboard` scope の外側のため
/// `disabled: true` で押下不能を明示する）。
fn action_button_group() -> Node {
    button_group::root(
        ButtonGroupOrientation::Horizontal,
        "共有リンクの操作",
        vec![("data-blocks-settings-share-link-actions", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("リンクをコピー")],
            ),
            button::button(
                &ButtonProps {
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("プレビュー")],
            ),
        ],
    )
}

/// カード骨格（header + body + footer）を束ねる共通ヘルパ。
fn share_card(id_prefix: &'static str, body: Vec<Node>) -> Node {
    card::root(
        CardProps::from(CardVariant::Outline),
        vec![("data-blocks-settings-share-link-card", id_prefix)],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("共有リンク")]),
                    card::description(
                        vec![],
                        vec![text("このページへのアクセスをリンクで共有します。")],
                    ),
                ],
            ),
            card::body(vec![("class", "blocks-settings-share-link-body")], body),
            card::footer(vec![], vec![action_button_group()]),
        ],
    )
}

/// A: 代表構成（R0318）。switch → separator → clipboard の縦積み。
fn version_basic() -> Node {
    let id_prefix = "blocks-settings-share-link-basic";
    share_card(
        id_prefix,
        vec![
            share_toggle(id_prefix),
            separator::separator(&SeparatorProps::default(), vec![]),
            share_url_clipboard(id_prefix),
        ],
    )
}

/// 閲覧範囲 radio card 1 件（モジュール doc「静的固定」節: 全件
/// `disabled: true` のネイティブ操作禁止）。
fn audience_item(checked: bool, value: &'static str, label: &'static str) -> Node {
    radio_card::item(
        checked,
        true,
        value,
        vec![],
        vec![
            radio_card::item_hidden_input(
                checked,
                true,
                Some("blocks-settings-share-link-audience"),
                value,
                vec![],
            ),
            radio_card::item_control(
                checked,
                true,
                vec![],
                vec![
                    radio_card::item_indicator(checked, true, false, vec![]),
                    radio_card::item_content(
                        vec![],
                        vec![radio_card::item_text(vec![], vec![text(label)])],
                    ),
                ],
            ),
        ],
    )
}

/// B: 閲覧範囲版（R0319）。A の body へ radio card 3 択を追加する。
fn version_audience() -> Node {
    let id_prefix = "blocks-settings-share-link-audience";
    let label_id = format!("{id_prefix}-label");
    share_card(
        id_prefix,
        vec![
            share_toggle(id_prefix),
            separator::separator(&SeparatorProps::default(), vec![]),
            static_url_display(id_prefix),
            div(
                vec![("class", "blocks-settings-share-link-audience-field")],
                vec![
                    radio_card::label(Some(&label_id), vec![], vec![text("閲覧できる範囲")]),
                    radio_card::root(
                        Size::Sm,
                        ColorPalette::Accent,
                        true,
                        None::<RadioCardOrientation>,
                        Some(&label_id),
                        vec![("aria-disabled", "true")],
                        vec![
                            audience_item(true, "invited", "招待した人のみ"),
                            audience_item(false, "anyone", "リンクを知っている全員"),
                            audience_item(false, "org", "組織内のメンバー"),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// C: 埋め込み/リンク切替版（R0320）。body 先頭に tabs を置く（`selected:
/// "link"` 固定の 1 インスタンス、モジュール doc「静的固定」節）。
fn version_tabs() -> Node {
    let id_prefix = "blocks-settings-share-link-tabs";
    let link_display = static_url_display(id_prefix);
    let embed_snippet = div(
        vec![("class", "blocks-settings-share-link-embed-snippet")],
        // 山括弧を含まないプレーン文字列（モジュール doc「埋め込み
        // スニペットに山括弧を含めない」節参照）。
        vec![text(format!("[embed] {SHARE_URL}"))],
    );
    let props = TabsProps {
        id: id_prefix,
        selected: "link",
        orientation: TabsOrientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };
    let tabs_node = tabs::tabs(
        TabsVariant::Enclosed,
        Size::Md,
        ColorPalette::Accent,
        &props,
        vec![
            TabItem {
                value: "link",
                trigger: vec![text("リンク")],
                content: vec![link_display],
                disabled: false,
            },
            TabItem {
                value: "embed",
                trigger: vec![text("埋め込み")],
                content: vec![embed_snippet],
                disabled: false,
            },
        ],
    );
    share_card(id_prefix, vec![share_toggle(id_prefix), tabs_node])
}

/// D: ドメイン接尾辞 + QR 版（R0321）。スラッグ入力 + ドメイン接尾辞
/// select（閉じた状態固定）+ QR コード。
fn version_domain_qr() -> Node {
    let id_prefix = "blocks-settings-share-link-domain";
    let slug_id = format!("{id_prefix}-slug");
    let slug_field = FieldProps {
        id: &slug_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        // QR コードは固定の `SHARE_URL` から生成する静的表示のため、スラッグを
        // 編集可能にすると表示スラッグと QR コードのリンク先が食い違う
        // （レビュー指摘、PR #3454）。読み取り専用にして不整合を防ぐ。
        readonly: true,
        has_helper_text: false,
    };
    let select_label_id = format!("{id_prefix}-select-label");
    let select_content_id = format!("{id_prefix}-select-content");
    let select_props = SelectProps {
        disabled: true,
        ..SelectProps::default()
    };
    let domain_select = div(
        vec![("class", "blocks-settings-share-link-domain-select")],
        vec![
            select::label(
                &select_props,
                Some(&select_label_id),
                vec![],
                vec![text("ドメイン接尾辞")],
            ),
            select::root(
                Size::Md,
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
                            Some(&select_content_id),
                            Some(&select_label_id),
                            vec![],
                            vec![
                                select::value_text(
                                    false,
                                    &select_props,
                                    vec![],
                                    vec![text(".example.com")],
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
                            Some(&select_content_id),
                            Some(&select_label_id),
                            None,
                            vec![],
                            vec![select::item(
                                OpenState::Open,
                                &select_props,
                                false,
                                false,
                                "example-com",
                                None,
                                vec![],
                                vec![select::item_text(
                                    OpenState::Open,
                                    &select_props,
                                    false,
                                    false,
                                    None,
                                    vec![],
                                    vec![text(".example.com")],
                                )],
                            )],
                        )],
                    ),
                ],
            ),
        ],
    );
    let matrix = encode(SHARE_URL, ErrorCorrectionLevel::M)
        // 固定短文字列のみを符号化するため `TooLong` になり得ない
        // （`settings_api_key_created` 等と同じくダミー値は本 block 内の
        // 定数として決定的に管理される）。失敗時は空 QR へフォールバック
        // し `expect`/`unwrap` を避ける。
        .unwrap_or_else(|_| {
            encode("share", ErrorCorrectionLevel::L).expect("固定短文字列の符号化は失敗しない")
        });
    let qr = qr_code::root(
        Size::Md,
        vec![("data-blocks-settings-share-link-qr", "")],
        vec![qr_code::frame(
            &matrix,
            DEFAULT_QUIET_ZONE,
            Some("共有 URL の QR コード"),
            vec![],
            vec![qr_code::pattern(&matrix, DEFAULT_QUIET_ZONE, vec![])],
        )],
    );
    share_card(
        id_prefix,
        vec![
            share_toggle(id_prefix),
            div(
                vec![("class", "blocks-settings-share-link-domain-row")],
                vec![
                    field::root(
                        &FieldRootProps::default(),
                        &slug_field,
                        vec![],
                        vec![
                            field::label(&slug_field, vec![], vec![text("公開スラッグ")]),
                            input::input(
                                &InputProps::default(),
                                &slug_field,
                                vec![("type", "text"), ("value", "o7pQ-report")],
                            ),
                        ],
                    ),
                    domain_select,
                ],
            ),
            div(
                vec![("class", "blocks-settings-share-link-qr-row")],
                vec![qr],
            ),
        ],
    )
}

/// `settings-share-link` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。4 版を縦に並記する。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-share-link-layout")],
        vec![
            version_basic(),
            version_audience(),
            version_tabs(),
            version_domain_qr(),
        ],
    )
}
```

## 原案差分メモ

- **版 A（代表構成、basic、R0318）**: 共有有効化 `switch` → `separator` →
  共有 URL の `clipboard` → footer にリンクコピー/プレビューの
  `button-group`。
- **版 B（閲覧範囲、audience、R0319）**: A の body へ「閲覧できる範囲」の
  `radio-card` 3 択を追加します。
- **版 C（埋め込み/リンク切替、tabs、R0320）**: body 先頭に `tabs`
  （リンク/埋め込みの 2 タブ、`selected: "link"` 固定）を置きます。
  「埋め込み」タブは `<iframe>` 風のスニペットを模した表示を検討しました
  が、既定エスケープで `<`/`>` が変換され可読性を損なうため、山括弧を
  含まない `[embed] https://…` 形式の文言にしています。
- **版 D（ドメイン接尾辞 + QR、domain-qr、R0321）**: スラッグ入力 +
  ドメイン接尾辞 `select`（閉じた状態固定）+ 共有 URL の `qr-code` を
  置きます。
- `clipboard` root は版 A の 1 個に限っています。`headless_clipboard`
  配線は「1 root : 1 状態機械契約」（マウントルート配下の全 `clipboard`
  パーツの表示が連動する簡略化）を持つため、Demo 内に `clipboard` root を
  複数置くと表示が連動してしまいます。版 B・版 C の共有 URL は `field` +
  `input`（readonly）+ `input-group` のコピーボタン（`disabled`）で代替
  しています（`settings-api-key-created` 版 B と同型）。
- switch・radio-card・select・footer の button-group はいずれも
  `disabled: true` のネイティブ操作禁止で、無 JS の docs サイトで操作可能
  に見せません。

関連情報: [Card](../themes/card.md) / [Switch](../themes/switch.md) /
[Clipboard](../themes/clipboard.md) / [Button](../themes/button.md) /
[Button Group](../themes/button-group.md) /
[Radio Card](../themes/radio-card.md) / [Tabs](../themes/tabs.md) /
[QR Code](../themes/qr-code.md) / [Select](../themes/select.md) /
[Separator](../themes/separator.md) / [Input](../themes/input.md) /
[Input Group](../themes/input-group.md)
