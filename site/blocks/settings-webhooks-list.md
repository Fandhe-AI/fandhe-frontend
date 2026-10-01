# settings-webhooks-list

Webhook エンドポイントの一覧画面の合成例です。`table` / `card` / `badge` /
`switch` / `menu` / `button` / `clipboard` の 7 部品を合成します。同じ一覧
データを 3 版（テーブル / カード / 区切り線のみの簡素版）で併記し、対応表
ID R0384（代表構成、主参照）/ R0270（テーブル + 署名シークレット欄）/
R0385（行ごとの有効スイッチ）/ R0386（区切り線のみの簡素版）に対応します。
参照ファイル置き場 `_/blocks-intake/` はこの worktree に存在せず、取得
手段・ファイル名・内部コンポーネント識別子は記載しません。

各行は 名前 / 宛先 URL / 状態バッジ / 有効・無効スイッチ / 操作メニュー を
表示します。宛先 URL・署名シークレットはすべて架空の値（`hooks.example.com`
形式・`fd_demo_whsec_` 接頭辞）であり、実在の企業・PII は含みません。

- 無 JS の docs サイトのため、状態（有効スイッチ・操作メニューの開閉）は
  初期値に固定して静的表示します。
- `<form>` を含みません。ボタンはすべて `type="button"` です。送信・テスト
  送信・編集・削除・有効化の切り替えは行わず、実際の実装は利用者自身の
  Rust/JS コードで行います。
- 署名シークレットの表示欄（`clipboard`）は版 A（テーブル）のみに置きます
  （1 root : 1 状態機械契約のため、Demo 全体で `clipboard::root` の呼び
  出しは 1 箇所のみ）。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// Webhook エンドポイント 1 行（名前, 宛先 URL, 状態表示文字列, 有効か）。
/// すべて架空の値（モジュール doc「ダミー素材について」節参照）。
const ENDPOINTS: &[(&str, &str, &str, bool)] = &[
    (
        "注文連携 Webhook",
        "https://hooks.example.com/ingest/order-sync",
        "正常",
        true,
    ),
    (
        "請求同期 Webhook",
        "https://hooks.example.com/ingest/invoice-sync",
        "失敗あり",
        true,
    ),
    (
        "在庫通知 Webhook",
        "https://hooks.example.com/ingest/inventory-alert",
        "正常",
        true,
    ),
    (
        "レガシー連携 Webhook",
        "https://hooks.example.com/ingest/legacy-bridge",
        "停止中",
        false,
    ),
];

/// 署名シークレット（架空。`fd_demo_whsec_` 接頭辞の明白な架空パターンで
/// 実運用プレフィックス `whsec_` 単独とは衝突しない）。
const SECRET: &str = "fd_demo_whsec_3a7c91e4f2b6d80c5a1e7f9b2d4c6803";

/// 3 版の識別（`id`/`name` の一意性確保・レイアウト切り替えに使う。
/// モジュール doc「版と集約元の対応」節参照）。
#[derive(Clone, Copy)]
enum Variant {
    Table,
    Cards,
    Divided,
}

impl Variant {
    /// `id`/`name` に付与する接尾辞（`a`/`b`/`c`）。
    fn suffix(self) -> &'static str {
        match self {
            Variant::Table => "a",
            Variant::Cards => "b",
            Variant::Divided => "c",
        }
    }
}

/// 状態表示文字列 → バッジ色（正常=Success、失敗あり=Danger、
/// それ以外〔停止中〕=Neutral）。
fn status_palette(status: &str) -> ColorPalette {
    match status {
        "正常" => ColorPalette::Success,
        "失敗あり" => ColorPalette::Danger,
        _ => ColorPalette::Neutral,
    }
}

/// 状態バッジ 1 個。
fn status_badge(status: &'static str) -> Node {
    badge::badge(
        &BadgeProps {
            palette: status_palette(status),
            ..BadgeProps::default()
        },
        vec![],
        vec![text(status)],
    )
}

/// 有効スイッチ 1 個（readonly + disabled で静的固定。モジュール doc
/// 「スイッチ・メニューは静的固定のみ」節参照）。
///
/// `endpoint_name` を label テキストに含める（`settings_integrations_list`
/// と同じ判断）。label パーツは visually-hidden のため表示レイアウトへは
/// 影響しないが、`<label>` が `hidden_input` と関連付くアクセシブルネームに
/// なるため、全行が同一の「有効」になると支援技術でどの行のスイッチか
/// 判別できない（PR #3489 Codex 指摘）。
fn enabled_switch(v: Variant, index: usize, enabled: bool, endpoint_name: &str) -> Node {
    let name = format!(
        "blocks-settings-webhooks-list-enabled-{}-{index}",
        v.suffix()
    );
    let props = SwitchProps {
        readonly: true,
        disabled: true,
        ..SwitchProps::default()
    };
    switch::root(
        Size::Sm,
        ColorPalette::Accent,
        enabled,
        &props,
        vec![("data-blocks-settings-webhooks-list-switch", "")],
        vec![
            switch::label(
                enabled,
                &props,
                vec![],
                vec![text(format!("{endpoint_name} を有効化"))],
            ),
            switch::hidden_input(&name, "on", enabled, &props, vec![]),
            switch::control(
                enabled,
                &props,
                vec![],
                vec![switch::thumb(enabled, &props, vec![], vec![])],
            ),
        ],
    )
}

/// 三点メニュー（常に閉状態の静的表示。モジュール doc
/// 「スイッチ・メニューは静的固定のみ」節参照）。
fn overflow_menu(v: Variant, index: usize, name: &str) -> Node {
    let content_id = format!("blocks-settings-webhooks-list-menu-{}-{index}", v.suffix());
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(&content_id),
        vec![("aria-label", &format!("{name} の操作"))],
        vec![text("\u{2026}")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(&content_id),
        None,
        vec![],
        vec![
            menu::item("test", false, false, vec![], vec![text("テスト送信")]),
            menu::item("edit", false, false, vec![], vec![text("編集")]),
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

/// 共通の「エンドポイントを追加」ボタン行（全版の先頭に置く）。
fn add_endpoint_row() -> Node {
    div(
        vec![("class", "blocks-settings-webhooks-list-intro")],
        vec![
            el(
                "p",
                vec![("class", "blocks-settings-webhooks-list-description")],
                vec![text(
                    "このプロジェクトから送信される Webhook エンドポイントの一覧です。",
                )],
            ),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-settings-webhooks-list-add", "")],
                vec![text("エンドポイントを追加")],
            ),
        ],
    )
}

/// 署名シークレット欄（版 A のみ、Demo 全体で唯一の `clipboard::root`
/// 呼び出し。モジュール doc「`clipboard` root は版 A 1 個に限る」節参照）。
fn secret_area() -> Node {
    let input_id = "blocks-settings-webhooks-list-secret";
    clipboard::root(
        SECRET,
        false,
        vec![("data-blocks-settings-webhooks-list-clipboard", "")],
        vec![
            clipboard::label(
                false,
                Some(input_id),
                vec![],
                vec![text("署名シークレット")],
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
                            clipboard::indicator(false, false, vec![], vec![text("コピー")]),
                            clipboard::indicator(true, false, vec![], vec![text("コピーしました")]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// **版 A（Table）**: `table::scroll_area` + `table::root` の 5 列
/// （名前/URL/状態/有効/操作）+ 署名シークレット欄。
fn table_variant() -> Node {
    let header_row = table::row(
        vec![],
        vec![
            table::column_header(vec![], vec![text("名前")]),
            table::column_header(vec![], vec![text("宛先 URL")]),
            table::column_header(vec![], vec![text("状態")]),
            table::column_header(vec![], vec![text("有効")]),
            table::column_header(vec![], vec![text("操作")]),
        ],
    );
    let body_rows: Vec<Node> = ENDPOINTS
        .iter()
        .enumerate()
        .map(|(i, (name, url, status, enabled))| {
            table::row(
                vec![],
                vec![
                    table::row_header(vec![], vec![text(*name)]),
                    table::cell(
                        vec![("class", "blocks-settings-webhooks-list-url-cell")],
                        vec![text(*url)],
                    ),
                    table::cell(vec![], vec![status_badge(status)]),
                    table::cell(
                        vec![],
                        vec![enabled_switch(Variant::Table, i, *enabled, name)],
                    ),
                    table::cell(vec![], vec![overflow_menu(Variant::Table, i, name)]),
                ],
            )
        })
        .collect();
    let table_node = table::root(
        TableProps {
            size: Size::Sm,
            ..TableProps::default()
        },
        vec![("aria-label", "Webhook エンドポイント")],
        vec![
            table::header(vec![], vec![header_row]),
            table::body(vec![], body_rows),
        ],
    );
    let scroll = table::scroll_area(
        vec![
            ("data-blocks-settings-webhooks-list-scroll", ""),
            ("role", "region"),
            ("aria-label", "Webhook エンドポイント一覧"),
            ("tabindex", "0"),
        ],
        vec![table_node],
    );
    div(
        vec![("class", "blocks-settings-webhooks-list-table-section")],
        vec![scroll, secret_area()],
    )
}

/// **版 B（Cards）**: 1 件 1 `card::root` をグリッドに並べる。
fn cards_variant() -> Node {
    let cards: Vec<Node> = ENDPOINTS
        .iter()
        .enumerate()
        .map(|(i, (name, url, status, enabled))| {
            card::root(
                CardProps::default(),
                vec![("data-blocks-settings-webhooks-list-card", "")],
                vec![card::body(
                    vec![],
                    vec![
                        div(
                            vec![("class", "blocks-settings-webhooks-list-card-head")],
                            vec![
                                el(
                                    "p",
                                    vec![("class", "blocks-settings-webhooks-list-card-name")],
                                    vec![text(*name)],
                                ),
                                overflow_menu(Variant::Cards, i, name),
                            ],
                        ),
                        el(
                            "p",
                            vec![("class", "blocks-settings-webhooks-list-url-cell")],
                            vec![text(*url)],
                        ),
                        div(
                            vec![("class", "blocks-settings-webhooks-list-card-foot")],
                            vec![
                                status_badge(status),
                                enabled_switch(Variant::Cards, i, *enabled, name),
                            ],
                        ),
                    ],
                )],
            )
        })
        .collect();
    div(vec![("class", "blocks-settings-webhooks-list-grid")], cards)
}

/// **版 C（Divided）**: 素の `<ul>`/`<li>` + `border-top` のみの区切り
/// （`separator` 部品は Issue 指定 7 部品に含まれないため使わない）。
fn divided_variant() -> Node {
    let items: Vec<Node> = ENDPOINTS
        .iter()
        .enumerate()
        .map(|(i, (name, url, status, enabled))| {
            el(
                "li",
                vec![("class", "blocks-settings-webhooks-list-row")],
                vec![
                    div(
                        vec![("class", "blocks-settings-webhooks-list-row-main")],
                        vec![
                            el(
                                "p",
                                vec![("class", "blocks-settings-webhooks-list-card-name")],
                                vec![text(*name)],
                            ),
                            el(
                                "p",
                                vec![("class", "blocks-settings-webhooks-list-url-cell")],
                                vec![text(*url)],
                            ),
                        ],
                    ),
                    div(
                        vec![("class", "blocks-settings-webhooks-list-row-actions")],
                        vec![
                            status_badge(status),
                            enabled_switch(Variant::Divided, i, *enabled, name),
                            overflow_menu(Variant::Divided, i, name),
                        ],
                    ),
                ],
            )
        })
        .collect();
    el(
        "ul",
        vec![("class", "blocks-settings-webhooks-list-divided")],
        items,
    )
}

/// 版のキャプション（素の `<p>`。`heading` を使わずページ右目次への混入を
/// 避ける、`settings_webhook_detail` 等と同型の判断）。
fn variant_caption(label: &'static str) -> Node {
    el(
        "p",
        vec![("class", "blocks-settings-webhooks-list-variant-label")],
        vec![text(label)],
    )
}

/// `settings-webhooks-list` の Demo 本体（版 A・B・C を縦に並記。呼び出し
/// ごとに同一の `Node` を返す純関数）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-webhooks-list-stack")],
        vec![
            add_endpoint_row(),
            variant_caption("版 A: テーブル + 署名シークレット欄（R0384 代表構成 + R0270）"),
            table_variant(),
            variant_caption("版 B: カード 1 件 1 行（R0385 行ごとの有効スイッチ）"),
            cards_variant(),
            variant_caption("版 C: 区切り線のみの簡素版（R0386）"),
            divided_variant(),
        ],
    )
}
```

## 原案差分メモ

- 版 A（テーブル、R0384 代表構成 + R0270）: `table` + `scroll_area` で
  名前 / 宛先 URL / 状態 / 有効 / 操作 の 5 列を組み、表の下に署名
  シークレット欄（`clipboard`）を置きました。
- 版 B（カード、R0385 行ごとの有効スイッチ）: 1 件 1 `card` に名前・URL・
  バッジ・スイッチ・メニューをまとめ、2 列グリッドに並べました。
- 版 C（区切り線のみの簡素版、R0386）: 素の `<ul>`/`<li>` + CSS の
  `border-top` のみで行を区切りました。`separator` 部品は Issue 指定 7
  部品に含まれないため使っていません。
- 狭幅（36rem 以下）ではカードを 1 列へ、区切り版の各行を縦積みへ
  切り替えます。
- 参照ファイル置き場 `_/blocks-intake/` はこの worktree に存在せず、R0384
  / R0270 / R0385 / R0386 の実物は未参照のため、対応表 ID のみを記録
  しています。

関連情報: [Table](../themes/table.md) / [Card](../themes/card.md) /
[Badge](../themes/badge.md) / [Switch](../themes/switch.md) /
[Menu](../themes/menu.md) / [Button](../themes/button.md) /
[Clipboard](../themes/clipboard.md)
