//! `settings-webhooks-list` block（イシュー #3023、親 #2951「Blocks 目的別
//! パーツ拡充」配下）。Webhook 宛先の一覧画面を 3 版（テーブル / カード /
//! 区切り線のみの簡素版）で併記する合成例。集約元は R0384（代表構成、主
//! 参照）/ R0270（テーブル + 署名シークレット欄）/ R0385（行ごとの有効
//! スイッチ）/ R0386（区切り線のみの簡素版）の 4 件。`_/blocks-intake/`
//! の対応ファイルは本イシュー着手時点で本 worktree に存在しないため、
//! 原稿・本コメントには対応表 ID のみを記す
//! （`settings_webhook_detail` と同じ扱い）。
//!
//! # 使用部品
//!
//! `table` / `card` / `badge` / `switch` / `menu` / `button` / `clipboard`
//! の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `blocks_nav.rs`/`blocks_contract.rs` が検証する）。`heading`/
//! `visually-hidden` は使わない。版の見出しは素の `<p>`（[`variant_caption`]）
//! で書き、ページ右の目次への混入を避ける（`settings_webhook_detail` 等と
//! 同型の判断）。
//!
//! # 版と集約元の対応
//!
//! - **版 A（Table、R0384 代表構成 + R0270）**: `table::scroll_area` +
//!   `table::root` で一覧を組み、表の下に署名シークレット欄
//!   （[`secret_area`]）を置く
//! - **版 B（Cards、R0385 行ごとの有効スイッチ）**: 1 件 1 `card::root` +
//!   `card::body` をグリッドに並べる
//! - **版 C（Divided、R0386 区切り線のみの簡素版）**: 素の `<ul>`/`<li>` を
//!   `border-top` のみの CSS で区切る（`separator` 部品は Issue 指定 7
//!   部品に含まれないため使わない）
//!
//! # `clipboard` root は版 A 1 個に限る
//!
//! [`fandhe_frontend_pre_styled_ui::clipboard`] モジュール doc の
//! 「1 root : 1 状態機械契約」（`settings_webhook_detail`/
//! `settings_integrations_list` と同じ制約）に従い、本 Demo 全体で
//! `clipboard::root` の呼び出しは版 A の署名シークレット領域 1 箇所のみ
//! とする。
//!
//! # スイッチ・メニューは静的固定のみ
//!
//! `switch::root` は `SwitchProps { readonly: true, disabled: true, .. }`
//! で固定し、ネイティブトグル操作自体を止める（`settings_webhook_detail`
//! と同じ判断）。アクセシブルネームは [`enabled_switch`] が
//! `switch::label` へ見える文言「有効」を置くことで付ける（行名との
//! 関連付けは `<label>` 要素〔`switch::root`〕のネスト構造が担うため、
//! `aria-label` 方式は採らない）。三点メニューは
//! `menu::trigger`/`content`/`positioner`/`root` を `OpenState::Closed`
//! 固定で組み、無 JS のため常に閉状態のみを描く（`settings_webhook_detail`
//! の `overflow_menu` と同型）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。全ボタンは `button::button` の既定 `type="button"` のまま
//! 用いる。テスト送信・編集・削除・有効化切り替えの実処理（送信先・検証・
//! 永続化）は一切持たない静的な合成例である
//! （`docs/policy/intentional-non-adoption.md` §3.25）。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui`] の各パーツ関数（`card`/`badge`/
//! `switch`/`menu`/`clipboard`/`button`）はいずれも `drop_class_attr` で
//! 呼び出し側 `class` を除去してから内部 variant クラスと合成するため、
//! これらへの CSS フックは `data-blocks-settings-webhooks-list-*` 属性で
//! 渡す。`table::header`/`body`/`row`/`column_header`/`cell`/`row_header`/
//! `scroll_area` は `drop_class_attr` を経由しないため `class` がそのまま
//! 効くが、他 block と同じ名前空間分離のため素のラッパー要素（`<div>`/
//! `<ul>`/`<li>`/`<p>`）とあわせて `class="blocks-settings-webhooks-list-*"`
//! を使う。
//!
//! # ダミー素材について
//!
//! 宛先 URL はすべて `hooks.example.com` 形式、署名シークレットは
//! `fd_demo_whsec_` 接頭辞の明白な架空パターン（他の settings blocks の
//! 規約と同一）であり、実在のサービス・企業・PII・実クレデンシャル形式を
//! 含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-webhooks-list/",
    title: "settings-webhooks-list",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_webhooks_list.rs",
    demo_class: "blocks-settings-webhooks-list",
    parts: &[
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Switch",
            path: "/themes/switch/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Clipboard",
            path: "/themes/clipboard/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_webhooks_list` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
///
/// `divided`（`ul`）/`description`・`url-cell`・`card-name`・
/// `variant-label`（いずれも `p`）は、素の
/// `.blocks-settings-webhooks-list-*` 単一クラス（詳細度 (0,1,0)）のみで
/// 宣言すると、サイト共通 typography（`site_theme.rs` の
/// `.docs-content ul,ol`/`.docs-content li`/`.docs-content p`、いずれも
/// 詳細度 (0,1,1)）に負けてビュレット・インデント・`margin`（docs 既定の
/// `0 0 1.05rem`）が復活する（PR #3489 Bugbot 指摘）。
/// `settings_integrations_list` の先例（PR #3441/#3447）と同じ判断で、
/// 祖先 `.blocks-settings-webhooks-list-stack`/`-divided` を持つ子孫
/// セレクタへ書き換えてクラス数を 2 に増やし（詳細度 (0,2,0)）、クラス数
/// 比較で `.docs-content` 側を確実に上回る。`row` はさらに
/// `margin-block: 0` を明示し、`.docs-content li` の `margin-block` を
/// 打ち消して行間を `border-top` のみに委ねる。
const LAYOUT_CSS: &str = "\
.blocks-settings-webhooks-list-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  container-type: inline-size;\n  container-name: blocks-settings-webhooks-list;\n}\n\
.blocks-settings-webhooks-list-intro {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-webhooks-list-stack .blocks-settings-webhooks-list-description {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-settings-webhooks-list-table-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-webhooks-list-stack .blocks-settings-webhooks-list-url-cell {\n  margin: 0;\n  overflow-wrap: anywhere;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-settings-webhooks-list-grid {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-webhooks-list-card-head {\n  display: flex;\n  align-items: flex-start;\n  justify-content: space-between;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-settings-webhooks-list-stack .blocks-settings-webhooks-list-card-name {\n  margin: 0;\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
.blocks-settings-webhooks-list-card-foot {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n  margin-top: var(--fandhe-space-3);\n}\n\
.blocks-settings-webhooks-list-stack .blocks-settings-webhooks-list-divided {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n}\n\
.blocks-settings-webhooks-list-divided .blocks-settings-webhooks-list-row {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  padding-block: var(--fandhe-space-3);\n  border-top: 1px solid var(--fandhe-color-border);\n  margin-block: 0;\n}\n\
.blocks-settings-webhooks-list-divided .blocks-settings-webhooks-list-row:first-child {\n  border-top: none;\n}\n\
.blocks-settings-webhooks-list-row-main {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-settings-webhooks-list-row-actions {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  flex-shrink: 0;\n}\n\
.blocks-settings-webhooks-list-stack .blocks-settings-webhooks-list-variant-label {\n  font-weight: var(--fandhe-font-font-weight-medium);\n  color: var(--fandhe-color-fg-muted);\n  margin: 0;\n}\n\
@container blocks-settings-webhooks-list (max-width: 36rem) {\n  \
.blocks-settings-webhooks-list-grid {\n    grid-template-columns: 1fr;\n  }\n  \
.blocks-settings-webhooks-list-divided .blocks-settings-webhooks-list-row {\n    flex-direction: column;\n    align-items: flex-start;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, ENDPOINTS, LAYOUT_CSS};
    use fandhe_frontend_core::render;
    use std::collections::HashSet;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"table\"",
            "data-scope=\"card\"",
            "data-scope=\"badge\"",
            "data-scope=\"switch\"",
            "data-scope=\"menu\"",
            "data-scope=\"button\"",
            "data-scope=\"clipboard\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<table").count(), 1);
        assert_eq!(
            html.matches("data-scope=\"clipboard\" data-part=\"root\"")
                .count(),
            1,
            "clipboard root は 1 root : 1 状態機械契約のため Demo 全体で 1 個のみ"
        );
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn switches_are_static_and_disabled() {
        let html = demo_html();
        for line in html.split("<input") {
            if line.contains("role=\"switch\"") {
                assert!(
                    line.contains("disabled"),
                    "switch hidden_input should carry native disabled: {line}"
                );
            }
        }
        // 3 版 x 4 行、enabled=true が 3 行・false が 1 行なので checked は
        // 合計 9 個、unchecked は合計 3 個出力される。
        assert!(html.matches("data-state=\"checked\"").count() >= 9);
        assert!(html.contains("data-state=\"unchecked\""));
    }

    #[test]
    fn menu_and_switch_ids_are_unique_across_variants() {
        let html = demo_html();
        let mut ids = HashSet::new();
        let mut names = HashSet::new();
        for suffix in ["a", "b", "c"] {
            for i in 0..4 {
                let menu_id = format!("blocks-settings-webhooks-list-menu-{suffix}-{i}");
                assert_eq!(
                    html.matches(&format!("id=\"{menu_id}\"")).count(),
                    1,
                    "menu id should appear exactly once: {menu_id}"
                );
                assert!(ids.insert(menu_id));
                let name = format!("blocks-settings-webhooks-list-enabled-{suffix}-{i}");
                assert!(html.contains(&format!("name=\"{name}\"")));
                assert!(names.insert(name));
            }
        }
    }

    #[test]
    fn layout_css_is_safe() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-settings-webhooks-list (max-width: 36rem)"));
    }

    /// PR #3489 Codex 指摘（各行スイッチが「有効」のみの同一アクセシブル
    /// ネーム）の回帰テスト。`switch::label` の文言に endpoint 名が含まれ、
    /// 3 版 x 4 行すべてで一意になることを固定する。
    #[test]
    fn switch_label_includes_endpoint_name() {
        let html = demo_html();
        for (name, ..) in ENDPOINTS {
            assert!(
                html.contains(&format!(">{name} を有効化<")),
                "switch label should include endpoint name: {name}"
            );
        }
        // 「有効」のみの旧文言（endpoint 名なし）の switch label が復活して
        // いないことを確認する（テーブル版の列見出し「有効」は
        // `<th>有効</th>` 相当でこの `data-part="label"` span とは別要素
        // のため対象外）。
        assert!(!html.contains(r#"data-part="label">有効<"#));
    }

    /// PR #3489 Codex/Bugbot 指摘（コンテナクエリ内 `.blocks-settings-webhooks-list-row`
    /// が通常時の `.blocks-settings-webhooks-list-divided .blocks-settings-webhooks-list-row`
    /// より詳細度で負け、36rem 以下でも区切り線版が縦積みにならない）の
    /// 回帰テスト。コンテナクエリ内セレクタが祖先クラスを含み通常時と
    /// 同等以上の詳細度（クラス数 2）を持つことを固定する。
    #[test]
    fn container_query_row_selector_matches_normal_specificity() {
        let container_block = LAYOUT_CSS
            .split("@container blocks-settings-webhooks-list (max-width: 36rem) {")
            .nth(1)
            .expect("container query block should exist");
        assert!(
            container_block.contains(
                ".blocks-settings-webhooks-list-divided .blocks-settings-webhooks-list-row {"
            ),
            "narrow-width row selector must keep the divided-list ancestor class for specificity parity"
        );
    }
}
