# settings-webhook-detail

Webhook 1 件の詳細画面向けブロックです。戻る導線・宛先 URL の見出し・
概要の定義リスト・署名シークレット表示・有効スイッチ・直近の配信
テーブルを縦に並べます。`link` / `heading` / `button` / `menu` / `card` /
`data-list` / `clipboard` / `switch` / `code` / `table` の 10 部品を
合成します。Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

代表構成（版 A、対応表 ID R0370 主参照）と戻り導線 + ペイロード例のコード
表示（版 B、対応表 ID R0372）の 2 版を並記します（`_/blocks-intake/` の
対応ファイルは本 worktree に存在しないため、対応表 ID のみを記載）。
宛先 URL・署名シークレット・購読イベント名・配信ログ・ペイロード JSON は
すべて架空のデータであり、実在のサービス・企業・PII・実クレデンシャル
形式を含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。三点メニュー・
有効スイッチはいずれも開閉・切替を行えない静的な初期状態のみを描きます。

## Rust コード

```rust
use crate::blocks::dummy_assets::PERSON_NAMES;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 宛先 URL（架空、実在サービスを模さない）。
const ENDPOINT_URL: &str = "https://hooks.example.com/ingest/8f2c91ab";

/// 署名シークレット（架空。`fd_demo_whsec_` 接頭辞の明白な架空パターンで
/// Stripe/Svix 実運用プレフィックス `whsec_` 単独とは衝突しない）。
const SECRET: &str = "fd_demo_whsec_7f2a9c1e4b6d8035a1c7e9f2b4d6803f";

/// 直近の配信 1 行（イベント名・結果・応答時間・配信日時の表示文字列）。
const DELIVERIES: &[(&str, &str, &str, &str)] = &[
    ("order.created", "200", "182ms", "2026-09-29 09:12"),
    ("order.updated", "200", "211ms", "2026-09-29 08:47"),
    ("invoice.paid", "500", "—", "2026-09-28 23:03"),
    ("order.cancelled", "200", "176ms", "2026-09-28 14:20"),
];

/// ペイロード例（架空の JSON、実クレデンシャル・PII を含まない）。
const PAYLOAD_JSON: &str = "{\n  \"event\": \"order.created\",\n  \"id\": \"evt_9f3c2a1b\",\n  \"data\": {\n    \"order_id\": \"ord_7731\",\n    \"amount\": 4800\n  }\n}";

/// 版 A・B の識別（`id`/`name` の一意性確保・領域差の切り替えに使う。
/// モジュール doc「版 A・B と集約元の対応」節参照）。
#[derive(Clone, Copy)]
enum Variant {
    A,
    B,
}

impl Variant {
    /// `id`/`name` に付与する接尾辞（`a`/`b`）。
    fn suffix(self) -> &'static str {
        match self {
            Variant::A => "a",
            Variant::B => "b",
        }
    }
}

/// 戻る導線（実在するページ `/blocks/` への相対パス、`href="#"` は使わない）。
fn back_link() -> Node {
    link::root(
        "../",
        &LinkProps {
            variant: LinkVariant::Underline,
            ..LinkProps::default()
        },
        vec![],
        vec![text("一覧に戻る")],
    )
}

/// 三点メニュー（`card_heading_toolbar.rs::overflow_menu` と同型。
/// 常に閉状態の静的表示、モジュール doc「三点メニューは閉じた静的状態の
/// み」節参照）。
fn overflow_menu(v: Variant) -> Node {
    let content_id = format!("blocks-settings-webhook-detail-menu-{}", v.suffix());
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(&content_id),
        vec![("aria-label", "その他の操作")],
        vec![text("\u{2026}")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(&content_id),
        None,
        vec![],
        vec![
            menu::item("test", false, false, vec![], vec![text("テスト送信")]),
            menu::item("pause", false, false, vec![], vec![text("一時停止")]),
            menu::separator(vec![], vec![]),
            menu::item("delete", false, false, vec![], vec![text("削除")]),
        ],
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// 見出し行（宛先 URL の `h3` 見出し + 「編集」ボタン + 三点メニュー）。
fn header(v: Variant) -> Node {
    div(
        vec![("class", "blocks-settings-webhook-detail-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![code::code(
                    &CodeProps::default(),
                    vec![],
                    vec![text(ENDPOINT_URL)],
                )],
            ),
            div(
                vec![("class", "blocks-settings-webhook-detail-actions")],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            size: Size::Sm,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("編集")],
                    ),
                    overflow_menu(v),
                ],
            ),
        ],
    )
}

/// 概要の定義リスト 1 行。
fn row(label: &'static str, value: &'static str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 概要カード（`card` + 横並び `data-list` 5 行）。
fn overview_card() -> Node {
    let author = PERSON_NAMES[0];
    card::root(
        CardProps::default(),
        vec![],
        vec![card::body(
            vec![],
            vec![data_list::root(
                DataListProps {
                    orientation: DataListOrientation::Horizontal,
                    ..DataListProps::default()
                },
                vec![("data-blocks-settings-webhook-detail-list", "")],
                vec![
                    row("説明", "注文イベントを外部システムへ連携する"),
                    row("購読イベント", "order.*, invoice.paid"),
                    row("配信形式", "application/json"),
                    row("作成日", "2026-05-12"),
                    row("作成者", author),
                ],
            )],
        )],
    )
}

/// 署名シークレット領域。版 A は `clipboard`（Demo 全体で唯一の
/// `clipboard::root` 呼び出し）、版 B は伏せ字の `code::code` + 無効化済み
/// 「再生成」ボタンに差し替える（モジュール doc「版 A・B と集約元の対応」
/// 節参照）。
///
/// 版 A の `clipboard::label` は見出し（h4 「署名シークレット」）と同じ
/// 文言だと画面上に重複表示されるため、`visually_hidden::root` で視覚的に
/// 隠しつつ `<label for>` の関連付けは保持する（支援技術には読み上げられ
/// 続ける。版 B が見出しのみの表示と一貫する）。
fn secret_card(v: Variant) -> Node {
    let body = match v {
        Variant::A => {
            let input_id = "blocks-settings-webhook-detail-secret-a";
            clipboard::root(
                SECRET,
                false,
                vec![("data-blocks-settings-webhook-detail-clipboard", "")],
                vec![
                    clipboard::label(
                        false,
                        Some(input_id),
                        vec![],
                        vec![visually_hidden::root(vec![], vec![text("署名シークレット")])],
                    ),
                    clipboard::control(
                        false,
                        vec![],
                        vec![
                            clipboard::input(SECRET, false, vec![("id", input_id)]),
                            clipboard::trigger(
                                false,
                                vec![("aria-label", "署名シークレットをコピー")],
                                vec![
                                    clipboard::indicator(
                                        false,
                                        false,
                                        vec![],
                                        vec![text("コピー")],
                                    ),
                                    clipboard::indicator(
                                        true,
                                        false,
                                        vec![],
                                        vec![text("コピーしました")],
                                    ),
                                ],
                            ),
                        ],
                    ),
                ],
            )
        }
        Variant::B => div(
            vec![("class", "blocks-settings-webhook-detail-secret-masked")],
            vec![
                code::code(
                    &CodeProps::default(),
                    vec![],
                    vec![text(
                        "fd_demo_whsec_\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}",
                    )],
                ),
                button::button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        size: Size::Sm,
                        disabled: true,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("再生成")],
                ),
            ],
        ),
    };
    div(
        vec![("class", "blocks-settings-webhook-detail-section")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text("署名シークレット")],
            ),
            body,
        ],
    )
}

/// 有効スイッチ（readonly + disabled で静的固定。モジュール doc
/// 「スイッチは readonly + disabled で静的固定」節参照）。
fn enabled_switch(v: Variant) -> Node {
    let name = format!("blocks-settings-webhook-detail-enabled-{}", v.suffix());
    let props = SwitchProps {
        readonly: true,
        disabled: true,
        ..SwitchProps::default()
    };
    div(
        vec![("class", "blocks-settings-webhook-detail-switch-row")],
        vec![switch::root(
            Size::Md,
            ColorPalette::Accent,
            true,
            &props,
            vec![],
            vec![
                switch::label(
                    true,
                    &props,
                    vec![],
                    vec![text("このエンドポイントを有効にする")],
                ),
                switch::hidden_input(&name, "on", true, &props, vec![]),
                switch::control(
                    true,
                    &props,
                    vec![],
                    vec![switch::thumb(true, &props, vec![], vec![])],
                ),
            ],
        )],
    )
}

/// 直近の配信テーブル（副次列は `応答時間`/`配信日時`、`settings_api_keys_table`
/// と同型）。
fn deliveries_table() -> Node {
    let header_row = table::row(
        vec![],
        vec![
            table::column_header(vec![], vec![text("イベント")]),
            table::column_header(vec![], vec![text("結果")]),
            table::column_header(
                vec![("data-blocks-settings-webhook-detail-secondary", "")],
                vec![text("応答時間")],
            ),
            table::column_header(
                vec![("data-blocks-settings-webhook-detail-secondary", "")],
                vec![text("配信日時")],
            ),
            table::column_header(vec![], vec![text("操作")]),
        ],
    );
    let body_rows: Vec<Node> = DELIVERIES
        .iter()
        .map(|(event, status, latency, at)| {
            table::row(
                vec![],
                vec![
                    table::row_header(vec![], vec![text(*event)]),
                    table::cell(
                        vec![],
                        vec![code::code(
                            &CodeProps::default(),
                            vec![],
                            vec![text(*status)],
                        )],
                    ),
                    table::cell(
                        vec![("data-blocks-settings-webhook-detail-secondary", "")],
                        vec![text(*latency)],
                    ),
                    table::cell(
                        vec![("data-blocks-settings-webhook-detail-secondary", "")],
                        vec![text(*at)],
                    ),
                    table::cell(
                        vec![],
                        vec![button::button(
                            &ButtonProps {
                                variant: ButtonVariant::Ghost,
                                size: Size::Sm,
                                ..ButtonProps::default()
                            },
                            vec![("aria-label", &format!("{event} を再送"))],
                            vec![text("再送")],
                        )],
                    ),
                ],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-settings-webhook-detail-section")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text("直近の配信")],
            ),
            table::root(
                TableProps {
                    size: Size::Sm,
                    ..TableProps::default()
                },
                vec![("aria-label", "直近の配信")],
                vec![
                    table::header(vec![], vec![header_row]),
                    table::body(vec![], body_rows),
                ],
            ),
        ],
    )
}

/// ペイロード例カード（版 B のみ。複数行 JSON を `<pre>` + `code::code` で
/// 表示する、`ai_chat_code_preview.rs` と同型）。
fn payload_card() -> Node {
    div(
        vec![("class", "blocks-settings-webhook-detail-section")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text("ペイロード例")],
            ),
            card::root(
                CardProps::default(),
                vec![],
                vec![card::body(
                    vec![],
                    vec![el(
                        "pre",
                        vec![("class", "blocks-settings-webhook-detail-code")],
                        vec![code::code(
                            &CodeProps::default(),
                            vec![],
                            vec![text(PAYLOAD_JSON)],
                        )],
                    )],
                )],
            ),
        ],
    )
}

/// 1 版分のページ全体（版 A/B 共通の骨格、[`Variant`] で領域差を切り替える、
/// モジュール doc「版 A・B と集約元の対応」節参照）。
fn page(v: Variant) -> Node {
    let mut children = vec![
        back_link(),
        header(v),
        overview_card(),
        secret_card(v),
        enabled_switch(v),
        deliveries_table(),
    ];
    if matches!(v, Variant::B) {
        children.push(payload_card());
    }
    div(
        vec![("class", "blocks-settings-webhook-detail-page")],
        children,
    )
}

/// 版のキャプション（素の `<p>`。`heading` を使わずページ右目次への混入を
/// 避ける、`settings_billing_overview` 等と同型の判断）。
fn variant_caption(label: &'static str) -> Node {
    el(
        "p",
        vec![("class", "blocks-settings-webhook-detail-variant-label")],
        vec![text(label)],
    )
}

/// `settings-webhook-detail` の Demo 本体（版 A・B を縦に並記。呼び出し
/// ごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-webhook-detail-stack")],
        vec![
            variant_caption("版 A: 代表構成（署名シークレットはコピー表示、R0370）"),
            page(Variant::A),
            variant_caption(
                "版 B: 戻り導線 + ペイロード例のコード表示（署名シークレットは伏せ字表示、R0372）",
            ),
            page(Variant::B),
        ],
    )
}
```

## 原案差分メモ

- **版 A（代表構成、R0370 主参照）**: 署名シークレットを `clipboard`
  （コピー可能な表示、Demo 全体で唯一の `clipboard::root` 呼び出し）
  で示す。
- **版 B（戻り導線 + ペイロード例のコード表示、R0372）**: 署名シークレットを
  伏せ字の `code::code` 表示 + 無効化済み「再生成」ボタンに差し替え、
  末尾にペイロード例（JSON）の複数行コード表示（`<pre>` + `code::code`）を
  追加する。どちらの版も「一覧に戻る」導線（Blocks 索引ページへの相対
  パス `../`）を持つ。
- 三点メニュー（テスト送信・一時停止・削除）・有効スイッチは無 JS のため
  開閉・切替を行えない静的な初期状態のみを描く。有効スイッチは
  `readonly` + `disabled` の両方を付与し、ネイティブ操作自体を止める。
- 直近の配信テーブルは `応答時間`/`配信日時` の 2 列を副次列とし、コンテナ
  幅 36rem 未満で非表示にする（操作列「再送」は隠さない）。同じ
  ブレークポイントで概要の定義リスト（`data-list`）も横並びから縦積みへ
  切り替える。
- `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
  存在しないため、対応表 ID（R0370・R0372）のみを記載しています。
- ブラウザでの実機確認（36rem 前後の幅切替・ライト/ダーク表示）は
  サンドボックス環境の制約により未実施です。

関連情報: [Link](../themes/link.md) / [Heading](../themes/heading.md) /
[Button](../themes/button.md) / [Menu](../themes/menu.md) /
[Card](../themes/card.md) / [Data List](../themes/data-list.md) /
[Clipboard](../themes/clipboard.md) / [Switch](../themes/switch.md) /
[Code](../themes/code.md) / [Table](../themes/table.md)
