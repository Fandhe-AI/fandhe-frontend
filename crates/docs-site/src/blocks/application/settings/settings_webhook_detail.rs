//! `settings-webhook-detail` block（イシュー #3018、親 #2951。Application /
//! Settings カテゴリ）。Webhook 1 件の詳細画面（戻る導線 → 宛先 URL の
//! 見出し → 概要の定義リスト → 署名シークレット表示 → 有効スイッチ →
//! 直近の配信テーブルを縦に並べる）。集約元は R0370（代表構成、主参照）/
//! R0372（戻り導線 + ペイロード例のコード表示）の 2 件。`_/blocks-intake/`
//! の対応ファイルは本イシュー着手時点で本 worktree に存在しないため、
//! 原稿・本コメントには対応表 ID のみを記す
//! （`settings_integrations_list` と同じ扱い）。
//!
//! # 使用部品
//!
//! `link` / `heading` / `button` / `menu` / `card` / `data-list` /
//! `clipboard` / `switch` / `code` / `table` の 10 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。`link`（戻る導線）・`heading`
//! （見出し）は Issue 列挙の 8 部品に加えた構造上必須の部品であり、
//! 新しい UI 部品は追加しない。
//!
//! # 版 A・B と集約元の対応
//!
//! - **版 A（R0370 代表構成）**: 署名シークレットを `clipboard`
//!   （コピー可能な表示、Demo 全体で唯一の `clipboard::root` 呼び出し）
//!   で示す
//! - **版 B（R0372 戻り導線 + ペイロード例のコード表示）**: 署名シークレットを
//!   伏せ字の `code::code` 表示 + 無効化済み「再生成」ボタンに差し替え、
//!   末尾にペイロード例（JSON）の複数行コード表示を追加する
//!
//! どちらの版も「一覧に戻る」導線（`link::root`、既存ページ `/blocks/`
//! への相対パス `../`）を持つ。
//!
//! # `clipboard` root は版 A 1 個に限る
//!
//! [`fandhe_frontend_pre_styled_ui::clipboard`] モジュール doc の
//! 「1 root : 1 状態機械契約」（`settings_integrations_list` と同じ制約）に
//! 従い、本 Demo 全体で `clipboard::root` の呼び出しは版 A の署名シークレット
//! 領域 1 箇所のみとする。
//!
//! # スイッチは readonly + disabled で静的固定
//!
//! `switch::root` は `settings_integrations_list` と同じ判断で
//! `SwitchProps { readonly: true, disabled: true, .. }` を使う。無 JS の
//! 静的デモで native トグル操作自体を止める。
//!
//! # `<form>` を使わない・送信先を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。全ボタンは `button::button` の既定 `type="button"` のまま
//! 用いる。テスト送信・一時停止・削除・再送・編集・再生成の実処理
//! （送信先・検証・永続化）は一切持たない静的な合成例である
//! （`docs/policy/intentional-non-adoption.md` §3.25）。
//!
//! # 三点メニューは閉じた静的状態のみ
//!
//! `card_heading_toolbar.rs::overflow_menu` と同型に、`menu::trigger`
//! （`aria-label="その他の操作"`）+ `menu::content`（`OpenState::Closed`
//! 固定）+ `menu::positioner` + `menu::root` で組む。無 JS のため開閉を
//! 切り替えられず、常に閉状態を描く。
//!
//! # 狭幅では副次列を隠す（操作列は隠さない）
//!
//! `応答時間`/`配信日時` の 2 列は `data-blocks-settings-webhook-detail-secondary`
//! を持つ `th`/`td` にのみ適用される `@container
//! blocks-settings-webhook-detail (max-width: 36rem)` の `display: none` で
//! 狭幅時に隠す（`settings_api_keys_table` と同型）。`イベント`/`結果`/`操作`
//! の 3 列は常に到達可能なまま残す。同じブレークポイントで、概要の定義
//! リスト（`data-list`）の横並びも縦積みへ切り替える
//! （`profile_detail_datalist` と同型）。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui`] の各パーツ関数（`card`/`button`/
//! `menu`/`switch`/`clipboard`/`data_list`/`table`）はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-settings-webhook-detail-*`）。レイアウト用ラッパー
//! （素の `<div>`/`<pre>`/`<p>`）は `class="blocks-settings-webhook-detail-*"`
//! を使う。
//!
//! # ダミー素材について
//!
//! 宛先 URL は `hooks.example.com`、作成者名は
//! `crate::blocks::dummy_assets::PERSON_NAMES`、イベント名・ステータス・
//! 署名シークレット・ペイロード JSON はすべて架空の値であり、実在の
//! サービス・企業・PII・実クレデンシャル形式を含まない
//! （`fd_demo_whsec_` 接頭辞は明白な架空パターンで、Stripe/Svix 実運用
//! プレフィックス `whsec_` 単独とは衝突しない。他の settings blocks の
//! `fd_demo_` 接頭辞規約に揃える）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
                    visually_hidden::root(
                        vec![],
                        vec![clipboard::label(
                            false,
                            Some(input_id),
                            vec![],
                            vec![text("署名シークレット")],
                        )],
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-webhook-detail/",
    title: "settings-webhook-detail",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_webhook_detail.rs",
    demo_class: "blocks-settings-webhook-detail",
    parts: &[
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Data List",
            path: "/themes/data-list/",
        },
        Part {
            label: "Clipboard",
            path: "/themes/clipboard/",
        },
        Part {
            label: "Switch",
            path: "/themes/switch/",
        },
        Part {
            label: "Code",
            path: "/themes/code/",
        },
        Part {
            label: "Table",
            path: "/themes/table/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_webhook_detail` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-settings-webhook-detail-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  container-type: inline-size;\n  container-name: blocks-settings-webhook-detail;\n}\n\
.blocks-settings-webhook-detail-page {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-settings-webhook-detail-header {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-webhook-detail-actions {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  margin-inline-start: auto;\n}\n\
.blocks-settings-webhook-detail-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-webhook-detail-switch-row {\n  display: flex;\n}\n\
.blocks-settings-webhook-detail-secret-masked {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"data-list\"][data-part=\"root\"][data-blocks-settings-webhook-detail-list] {\n  --fandhe-data-list-gap: 0;\n}\n\
[data-blocks-settings-webhook-detail-list] > [data-scope=\"data-list\"][data-part=\"item\"] {\n  border-top: 1px solid var(--fandhe-color-border);\n  padding-block: var(--fandhe-space-3);\n}\n\
.blocks-settings-webhook-detail-code {\n  margin: 0;\n  padding: var(--fandhe-space-3);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  overflow-x: auto;\n}\n\
.blocks-settings-webhook-detail-variant-label {\n  font-weight: var(--fandhe-font-font-weight-medium);\n  color: var(--fandhe-color-fg-muted);\n  margin: 0;\n}\n\
@container blocks-settings-webhook-detail (max-width: 36rem) {\n  \
.blocks-settings-webhook-detail-actions {\n    margin-inline-start: 0;\n    width: 100%;\n  }\n  \
.blocks-settings-webhook-detail-stack [data-scope=\"data-list\"][data-part=\"item\"] {\n    flex-direction: column;\n    gap: var(--fandhe-space-1);\n  }\n  \
.blocks-settings-webhook-detail-stack [data-scope=\"data-list\"][data-part=\"item-label\"] {\n    min-width: auto;\n  }\n  \
[data-blocks-settings-webhook-detail-secondary] {\n    display: none;\n  }\n\
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
            "data-scope=\"link\"",
            "data-scope=\"heading\"",
            "data-scope=\"button\"",
            "data-scope=\"menu\"",
            "data-scope=\"card\"",
            "data-scope=\"data-list\"",
            "data-scope=\"clipboard\"",
            "data-scope=\"switch\"",
            "data-scope=\"code\"",
            "data-scope=\"table\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<h3").count(), 2);
        assert_eq!(
            html.matches("data-scope=\"clipboard\" data-part=\"root\"")
                .count(),
            1,
            "clipboard root は 1 root : 1 状態機械契約のため Demo 全体で 1 個のみ"
        );
        assert_eq!(html.matches("<table").count(), 2);
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
        assert!(html.contains("href=\"../\""));
    }

    #[test]
    fn switch_is_static_and_disabled() {
        let html = demo_html();
        assert!(html.contains("data-readonly"));
        assert!(html.matches("data-state=\"checked\"").count() >= 2);
        for line in html.split("<input") {
            if line.contains("role=\"switch\"") {
                assert!(
                    line.contains("disabled"),
                    "switch hidden_input should carry native disabled: {line}"
                );
            }
        }
    }

    #[test]
    fn payload_card_only_in_version_b() {
        let html = demo_html();
        assert_eq!(
            html.matches("<h4").count(),
            5,
            "版 A は 2 見出し、版 B は 3 見出し（ペイロード例を含む）"
        );
        assert_eq!(html.matches("再生成").count(), 1);
    }

    #[test]
    fn layout_css_is_safe_and_hides_secondary_columns_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-settings-webhook-detail (max-width: 36rem)"));
        assert!(LAYOUT_CSS
            .contains("[data-blocks-settings-webhook-detail-secondary] {\n    display: none;"));
        assert!(!LAYOUT_CSS.contains("-actions] {\n    display: none"));
    }

    #[test]
    fn ids_are_unique_across_versions() {
        let html = demo_html();
        assert_eq!(
            html.matches("id=\"blocks-settings-webhook-detail-menu-a\"")
                .count(),
            1
        );
        assert_eq!(
            html.matches("id=\"blocks-settings-webhook-detail-menu-b\"")
                .count(),
            1
        );
        assert!(html.contains("name=\"blocks-settings-webhook-detail-enabled-a\""));
        assert!(html.contains("name=\"blocks-settings-webhook-detail-enabled-b\""));
    }
}
