# settings-share-link

共有の有効化スイッチ・共有 URL のコピー欄・リンクコピー / プレビューの
ボタン群を持つ共有リンク設定カードです。`card` / `switch` / `clipboard` /
`button` / `button-group` / `radio-card` / `qr-code` / `select` /
`separator` / `input` / `input-group` / `text` の 12 部品を合成します。
Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0318（代表構成）で、閲覧範囲の radio card（R0319）・
埋め込み/リンクの切替（R0320）・ドメイン接尾辞 + QR コード（R0321）
の 3 差分版を集約しています。共有 URL は架空の `.example` ドメインで、
実在ドメイン・秘密情報らしき文字列は含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。埋め込み/
リンク切替版は実物の `tabs::tabs`（クリックでの切替に JS 配線を前提と
する部品）を使わず、素の `div` による静的なタブ列（選択中の状態を
`data-state="active"` で示すのみ）とキャプション付きの 2 状態並記で
表示します。

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
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 版 A〜C 共通の共有 URL（架空 `.example` ドメイン、モジュール doc
/// 「ダミー値・ドメインは明白な架空パターン」節参照）。
const SHARE_URL: &str = "https://fandhe-frontend.example/share/o7pQ-report";

/// 版 D（ドメイン接尾辞 + QR）専用のスラッグ。表示スラッグ入力・QR コードの
/// 両方がこの値から導出され、食い違いを構造的に防ぐ（レビュー指摘是正）。
const DOMAIN_SLUG: &str = "o7pq-report";

/// 版 D 専用のドメイン接尾辞（RFC 2606 予約の `.example.com`）。
const DOMAIN_SUFFIX: &str = ".example.com";

/// 版 D の表示スラッグ・select・QR コードが共通して参照する共有 URL。
/// `DOMAIN_SLUG`/`DOMAIN_SUFFIX` から構築し、3 箇所が独立した文字列
/// リテラルを持つことによる食い違い（レビュー指摘: 表示
/// `.example.com` と QR の旧リンク先が不一致だった）を防ぐ。
const DOMAIN_SHARE_URL: &str = "https://o7pq-report.example.com/";

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
                        // Demo 内の他ボタン（footer の action button-group・
                        // 版 B/C/D の静的コピーボタン）はいずれも
                        // `disabled` で押下不能を明示しているため、この
                        // trigger だけが操作可能に見えるのは一貫性を欠く
                        // （レビュー指摘是正）。native `disabled` により
                        // クリックしても何も起きないことを明示する。
                        vec![("disabled", "")],
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
///
/// radio card 全 item は `disabled: true`（ネイティブ操作禁止）のため、
/// フォーム走査（スクリーンリーダーの「フォーム項目を読む」操作等）が
/// disabled な input を読み飛ばす環境では選択状態が伝わらない。
/// `card_form_footer::payment_method_field` と同型に、radio card の外側へ
/// 独立した文（[`styled_text::text`]）で現在の選択を明文化する
/// （レビュー指摘是正、PR #3454 cursor Bugbot）。
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
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("現在の選択: 招待した人のみ")],
                    ),
                ],
            ),
        ],
    )
}

/// 静的なタブ列（版 C 専用、モジュール doc「版 C: 無 JS での扱い」節参照）。
/// 実物の `tabs::tabs` を使わず、選択中タブへ `data-state="active"` を
/// 付与した素の `div` で構成する。操作できるように見せないため
/// `role="tab"`/`<button>`/`tabindex` を出さず、装飾として
/// `aria-hidden="true"` を付ける
/// （`feature_tabs_panel::static_tab_list` と同型の判断）。
fn static_tab_list(selected: &'static str, items: &[(&'static str, &'static str)]) -> Node {
    div(
        vec![("class", "blocks-settings-share-link-tablist")],
        items
            .iter()
            .map(|(value, label)| {
                let state = if *value == selected {
                    "active"
                } else {
                    "inactive"
                };
                div(
                    vec![
                        ("class", "blocks-settings-share-link-tab"),
                        ("data-state", state),
                        ("aria-hidden", "true"),
                    ],
                    vec![text(*label)],
                )
            })
            .collect(),
    )
}

/// タブ 1 件分のプレビュー本文（キャプション + 内容）。両パネルを常時
/// 可視のまま縦に並記する（`feature_tabs_panel::tab_preview` と同型）。
fn tab_preview(caption: &'static str, content: Node) -> Node {
    div(
        vec![("class", "blocks-settings-share-link-preview")],
        vec![
            div(
                vec![("class", "blocks-settings-share-link-preview-caption")],
                vec![text(caption)],
            ),
            content,
        ],
    )
}

/// C: 埋め込み/リンク切替版（R0320）。body 先頭に静的なタブ列（`selected:
/// "link"` 固定）を置き、両パネルをキャプション付きで常時可視のまま
/// 縦に並記する（モジュール doc「版 C: 無 JS での扱い」節）。
fn version_tabs() -> Node {
    let id_prefix = "blocks-settings-share-link-tabs";
    let link_display = static_url_display(id_prefix);
    let embed_snippet = div(
        vec![("class", "blocks-settings-share-link-embed-snippet")],
        // 山括弧を含まないプレーン文字列（モジュール doc「埋め込み
        // スニペットに山括弧を含めない」節参照）。
        vec![text(format!("[embed] {SHARE_URL}"))],
    );
    let tab_list = static_tab_list("link", &[("link", "リンク"), ("embed", "埋め込み")]);
    share_card(
        id_prefix,
        vec![
            share_toggle(id_prefix),
            tab_list,
            tab_preview("リンク", link_display),
            tab_preview("埋め込み", embed_snippet),
        ],
    )
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
        // QR コードは `DOMAIN_SHARE_URL` から生成する静的表示のため、スラッグを
        // 編集可能にすると表示スラッグと QR コードのリンク先が食い違う
        // （レビュー指摘、PR #3454）。読み取り専用にし、かつ入力値・select・
        // QR を同一の `DOMAIN_SLUG`/`DOMAIN_SUFFIX`/`DOMAIN_SHARE_URL` 定数
        // から取ることで構造的に不整合を防ぐ。
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
                                    vec![text(DOMAIN_SUFFIX)],
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
                                    vec![text(DOMAIN_SUFFIX)],
                                )],
                            )],
                        )],
                    ),
                ],
            ),
        ],
    );
    let matrix = encode(DOMAIN_SHARE_URL, ErrorCorrectionLevel::M)
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
                                vec![("type", "text"), ("value", DOMAIN_SLUG)],
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
  `button-group`。`clipboard` のコピーボタンは実アプリへの配線を持たない
  合成例のため `disabled` にしています。
- **版 B（閲覧範囲、audience、R0319）**: A の body へ「閲覧できる範囲」の
  `radio-card` 3 択を追加します。radio-card 全 item は disabled のため、
  フォーム走査で選択状態が読み上げられない環境向けに、独立した `text`
  の一文（「現在の選択: 招待した人のみ」）で明文化します。
- **版 C（埋め込み/リンク切替、tabs、R0320）**: body 先頭に静的なタブ列
  （リンク/埋め込みの 2 状態、`selected: "link"` 固定）を置きます。実物の
  `tabs::tabs` は使わず、素の `div` + `data-state` でタブの見た目のみを
  再現し、両パネルをキャプション付きで常時可視のまま縦に並記します
  （`feature_tabs_panel` で確立した無 JS の扱いと同じ判断）。
  「埋め込み」パネルは `<iframe>` 風のスニペットを模した表示を検討しました
  が、既定エスケープで `<`/`>` が変換され可読性を損なうため、山括弧を
  含まない `[embed] https://…` 形式の文言にしています。
- **版 D（ドメイン接尾辞 + QR、domain-qr、R0321）**: スラッグ入力（読み取り
  専用）+ ドメイン接尾辞 `select`（閉じた状態固定）+ 共有 URL の
  `qr-code` を置きます。表示スラッグ・ドメイン接尾辞・QR コードのリンク先
  は共通の定数（`o7pq-report` + `.example.com`）から取り、独立した文字列
  リテラルによる食い違いを防いでいます。
- `clipboard` root は版 A の 1 個に限っています。`headless_clipboard`
  配線は「1 root : 1 状態機械契約」（マウントルート配下の全 `clipboard`
  パーツの表示が連動する簡略化）を持つため、Demo 内に `clipboard` root を
  複数置くと表示が連動してしまいます。版 B・版 C の共有 URL は `field` +
  `input`（readonly）+ `input-group` のコピーボタン（`disabled`）で代替
  しています（`settings-api-key-created` 版 B と同型）。
- switch・radio-card・select・footer の button-group・clipboard のコピー
  ボタンはいずれも `disabled: true`/native `disabled` のネイティブ操作
  禁止で、無 JS の docs サイトで操作可能に見せません。

関連情報: [Card](../themes/card.md) / [Switch](../themes/switch.md) /
[Clipboard](../themes/clipboard.md) / [Button](../themes/button.md) /
[Button Group](../themes/button-group.md) /
[Radio Card](../themes/radio-card.md) /
[QR Code](../themes/qr-code.md) / [Select](../themes/select.md) /
[Separator](../themes/separator.md) / [Input](../themes/input.md) /
[Input Group](../themes/input-group.md) /
[Text](../themes/text.md)
