# settings-page-tabs

上部にアプリ用ナビバー、その下にタブ状のセクション切替を持つページ見出し、
本文に設定カードを並べる合成例です。`navigation-menu` / `tab-nav` / `card` /
`table` / `badge` / `menu` / `input-group` / `input` / `button` / `progress` /
`checkbox` / `pagination` の 12 部品を合成します。Blocks は既存部品の合成例
であり、新しい UI 部品は追加しません。

選択中タブが異なる 2 版（版 A: API 設定、版 B: プラン）を静的に併記します。
版 A は主参照 R0660（代表構成）、版 B は集約元 R0658 に対応します。無 JS の
ため各版の選択中タブを初期状態として固定表示します。狭い幅ではタブ列を横
スクロールできます。API キー・請求書番号・金額・プラン名はすべて架空の
ダミー値であり、実在サービスの鍵形式・実在の人物・企業・PII は含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, header, section, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
use fandhe_frontend_pre_styled_ui::field::{FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps};
use fandhe_frontend_pre_styled_ui::pagination::{self, ItemMode};
use fandhe_frontend_pre_styled_ui::progress::{self, Orientation, ProgressProps};
use fandhe_frontend_pre_styled_ui::tab_nav;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// 実在の自組織 URL。
const ORG: &str = "https://github.com/Fandhe-AI";

/// 装飾用の自作幾何アイコン（実在ブランドのロゴを模さない、`label: None`）。
fn geo_icon(d: &'static str) -> Node {
    icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 再生成アイコン（円弧 + 矢印）。
fn refresh_icon() -> Node {
    geo_icon("M20 12a8 8 0 1 1-2.34-5.66 M20 4v5h-5")
}

/// 縦 3 点の行操作アイコン（`menu` トリガーの視覚的ラベル、アクセシブル
/// ネームは `aria-label` が担うため装飾用途）。
fn kebab_icon() -> Node {
    geo_icon("M12 6v.01 M12 12v.01 M12 18v.01")
}

/// ロゴ（幾何図形 + ブランド名テキスト）。`navbar_app_links.rs` の
/// `logo()` と同型。
fn logo() -> Node {
    span(
        vec![("data-blocks-settings-page-tabs-logo", "")],
        vec![
            geo_icon("M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z"),
            span(vec![], vec![text("Fandhe Console")]),
        ],
    )
}

/// 上部のアプリ用ナビバー（ダッシュボード / プロジェクト / 設定〔現在地〕
/// + 右端の Ghost/Sm ボタン 1 個）。
fn navbar() -> Node {
    let props = NavigationMenuProps::default();
    let nav_node = navigation_menu::root(
        &props,
        "メインナビゲーション",
        vec![("data-blocks-settings-page-tabs-nav", "")],
        vec![navigation_menu::list(
            &props,
            vec![],
            vec![
                navigation_menu::item(
                    navigation_menu::OpenState::Closed,
                    false,
                    &props,
                    "dashboard",
                    vec![],
                    vec![navigation_menu::link(
                        REPO,
                        false,
                        vec![],
                        vec![text("ダッシュボード")],
                    )],
                ),
                navigation_menu::item(
                    navigation_menu::OpenState::Closed,
                    false,
                    &props,
                    "projects",
                    vec![],
                    vec![navigation_menu::link(
                        ORG,
                        false,
                        vec![],
                        vec![text("プロジェクト")],
                    )],
                ),
                navigation_menu::item(
                    navigation_menu::OpenState::Closed,
                    false,
                    &props,
                    "settings",
                    vec![],
                    vec![navigation_menu::link(
                        "./",
                        true,
                        vec![],
                        vec![text("設定")],
                    )],
                ),
            ],
        )],
    );
    header(
        vec![("class", "blocks-settings-page-tabs-navbar")],
        vec![
            logo(),
            nav_node,
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-settings-page-tabs-navbar-cta", "")],
                vec![text("新規プロジェクト")],
            ),
        ],
    )
}

/// タブ項目（id, ラベル）。実在の遷移先は [`page_heading`] が `current`
/// 引数と突き合わせて決める（モジュール doc「ナビは実在 URL、タブは現在地
/// のみ `./`・他タブは実在フラグメント」節参照）。
const TAB_ITEMS: [(&str, &str); 5] = [
    ("general", "一般"),
    ("members", "メンバー"),
    ("api", "API"),
    ("plan", "プラン"),
    ("billing", "請求"),
];

/// タブ見出し（heading「設定」+ タブ列）。`current` は現在地タブの `id`
/// （`href="./"` + `aria-current="page"` を付与）、`tabs_label` は
/// `tab_nav::root` の `aria-label`（版ごとに一意にする、モジュール doc
/// 「版の並記」節参照）。
fn page_heading(current: &str, tabs_label: &str) -> Node {
    let tab_nodes: Vec<Node> = TAB_ITEMS
        .iter()
        .map(|(id, label)| {
            let is_current = *id == current;
            let href = if is_current {
                "./".to_string()
            } else {
                format!("#{id}")
            };
            tab_nav::link(&href, is_current, vec![], vec![text(*label)])
        })
        .collect();
    let tabs = tab_nav::root(
        Size::Md,
        tabs_label,
        vec![("data-blocks-settings-page-tabs-tabs", "")],
        tab_nodes,
    );
    div(
        vec![("class", "blocks-settings-page-tabs-heading")],
        vec![
            heading(
                HeadingLevel::H2,
                &HeadingProps::default(),
                vec![],
                vec![text("設定")],
            ),
            tabs,
        ],
    )
}

/// 「API アクセス」カード（マスク済みダミーキーの読み取り専用
/// `input_group` + 「再生成」ボタン）。
fn api_access_card() -> Node {
    let field_id = "blocks-settings-page-tabs-access-key";
    let field_props = FieldProps {
        id: field_id,
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
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-page-tabs-access-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("API アクセス")]),
                    card::description(vec![], vec![text("サーバー間連携に使うアクセスキーです。")]),
                ],
            ),
            card::body(
                vec![],
                vec![input_group::root(
                    &group_props,
                    vec![("data-blocks-settings-page-tabs-access-group", "")],
                    vec![
                        input::input(
                            &InputProps::default(),
                            &field_props,
                            vec![
                                ("aria-label", "アクセスキー"),
                                ("value", "fd_demo_••••••••4f2a"),
                            ],
                        ),
                        input_group::addon(
                            InputGroupAlign::InlineEnd,
                            &group_props,
                            vec![],
                            vec![button::button(
                                &ButtonProps {
                                    variant: ButtonVariant::Outline,
                                    size: Size::Sm,
                                    ..ButtonProps::default()
                                },
                                vec![],
                                vec![refresh_icon(), text("再生成")],
                            )],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// 発行済み API キー 1 行分。`content_id` は行ごとに一意な `menu` の
/// `content_id`（`aria-controls` の宙吊り防止、モジュール doc参照）。
struct ApiKeyRow {
    name: &'static str,
    prefix: &'static str,
    scope: &'static str,
    last_used: &'static str,
}

/// 発行済み API キーの操作メニュー（無 JS のため `disabled: true` 固定）。
/// `aria-label` は行名（`row_name`）を含めて行ごとに一意にする（codex/
/// Bugbot レビュー指摘: 全行「行の操作」では対象行をスクリーンリーダーが
/// 区別できない）。
fn key_actions_menu(content_id: &str, row_name: &str) -> Node {
    let trigger_label = format!("{row_name}の操作");
    let trigger = menu::trigger(
        OpenState::Closed,
        true,
        Some(content_id),
        vec![
            ("aria-label", trigger_label.as_str()),
            ("data-blocks-settings-page-tabs-key-trigger", ""),
        ],
        vec![kebab_icon()],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id),
        None,
        vec![],
        vec![
            menu::item("rename", false, false, vec![], vec![text("名前を変更")]),
            menu::item("revoke", false, false, vec![], vec![text("失効させる")]),
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

/// 「発行済み API キー」カード（`table::scroll_area` で包んだ表）。
fn api_keys_table_card() -> Node {
    let rows: [ApiKeyRow; 3] = [
        ApiKeyRow {
            name: "本番サーバー",
            prefix: "fd_demo_prod_",
            scope: "読み取り/書き込み",
            last_used: "2026-09-28",
        },
        ApiKeyRow {
            name: "CI パイプライン",
            prefix: "fd_demo_ci_",
            scope: "読み取りのみ",
            last_used: "2026-09-25",
        },
        ApiKeyRow {
            name: "検証環境",
            prefix: "fd_demo_test_",
            scope: "読み取り/書き込み",
            last_used: "未使用",
        },
    ];
    let body_rows: Vec<Node> = rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let content_id = format!("blocks-settings-page-tabs-key-menu-{i}");
            table::row(
                vec![],
                vec![
                    table::row_header(vec![], vec![text(row.name)]),
                    table::cell(vec![], vec![text(row.prefix)]),
                    table::cell(
                        vec![],
                        vec![badge::badge(
                            &BadgeProps {
                                variant: BadgeVariant::Subtle,
                                ..BadgeProps::default()
                            },
                            vec![],
                            vec![text(row.scope)],
                        )],
                    ),
                    table::cell(vec![], vec![text(row.last_used)]),
                    table::cell(vec![], vec![key_actions_menu(&content_id, row.name)]),
                ],
            )
        })
        .collect();

    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-page-tabs-keys-card", "")],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    card::title(vec![], vec![text("発行済み API キー")]),
                    card::action(
                        vec![],
                        vec![button::button(
                            &ButtonProps {
                                size: Size::Sm,
                                ..ButtonProps::default()
                            },
                            vec![],
                            vec![text("キーを発行")],
                        )],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![table::scroll_area(
                    vec![
                        ("role", "region"),
                        ("aria-label", "発行済み API キー一覧"),
                        ("tabindex", "0"),
                    ],
                    vec![table::root(
                        TableProps::default(),
                        vec![],
                        vec![
                            table::header(
                                vec![],
                                vec![table::row(
                                    vec![],
                                    vec![
                                        table::column_header(vec![], vec![text("名前")]),
                                        table::column_header(vec![], vec![text("先頭文字列")]),
                                        table::column_header(vec![], vec![text("権限")]),
                                        table::column_header(vec![], vec![text("最終使用")]),
                                        table::column_header(vec![], vec![text("操作")]),
                                    ],
                                )],
                            ),
                            table::body(vec![], body_rows),
                        ],
                    )],
                )],
            ),
        ],
    )
}

/// 非選択タブ（一般/メンバー）の実在するリンク先。両タブとも着手予定が
/// ないため、「準備中」であることを示す最小限の見出しスタブのみを置き、
/// タブの `href="#<id>"` を実在させる（モジュール doc「ナビは実在 URL、
/// タブは現在地のみ `./`・他タブは実在フラグメント」節参照、codex レビュー
/// P1 是正 v2）。API/プラン/請求の 3 タブは [`version_api`]/[`plan_card`]/
/// [`invoices_table_card`] が出力する実在の `id` へリンクするため、本スタブ
/// の対象ではない。
fn other_section_stub(id: &str, label: &str) -> Node {
    card::root(
        CardProps::default(),
        vec![("id", id), ("data-blocks-settings-page-tabs-stub", "")],
        vec![card::header(
            vec![],
            vec![
                card::title(vec![], vec![text(label)]),
                card::description(vec![], vec![text("このセクションは準備中です。")]),
            ],
        )],
    )
}

/// 版 A（API 設定、選択中タブ「API」）。既存の [`api_access_card`]/
/// [`api_keys_table_card`] を `id="api"` の本文ラッパーへ包み、
/// [`page_heading`] へ現在地を渡す（モジュール doc「版の並記」節参照）。
fn version_api() -> Node {
    section(
        vec![
            ("aria-label", "版 A: API 設定"),
            ("data-blocks-settings-page-tabs-version", "api"),
        ],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![("data-blocks-settings-page-tabs-version-title", "")],
                vec![text("版 A: API 設定")],
            ),
            page_heading("api", "設定セクション（API 選択）"),
            div(
                vec![
                    ("id", "api"),
                    ("class", "blocks-settings-page-tabs-content"),
                ],
                vec![api_access_card(), api_keys_table_card()],
            ),
        ],
    )
}

/// ラベル・消費率（%）・可視の値テキストから 1 本の進捗バー行を組む
/// （`settings_billing_usage::usage_bar` と同型。`aria-label` は
/// [`fandhe_frontend_pre_styled_ui::progress::root`] が自動配線しないため
/// 明示が必須、codex P1 是正の踏襲）。
fn usage_bar(label: &'static str, percent: f64, value_text: String) -> Node {
    let p = Progress::new(0.0, 100.0, Some(percent), Orientation::Horizontal);
    div(
        vec![("class", "blocks-settings-page-tabs-usage-row")],
        vec![
            div(
                vec![("class", "blocks-settings-page-tabs-usage-row-header")],
                vec![
                    span(vec![], vec![text(label)]),
                    span(vec![], vec![text(value_text)]),
                ],
            ),
            progress::root(
                &p,
                &ProgressProps {
                    size: Size::Sm,
                    ..ProgressProps::default()
                },
                None,
                vec![("aria-label", label)],
                vec![p.track(vec![], vec![progress::range(&p, vec![])])],
            ),
        ],
    )
}

/// 「現在のプラン」カード（`id="plan"`。プランバッジ + 変更ボタン + 進捗
/// バー 2 本）。
fn plan_card() -> Node {
    let (tier_name, _tier_price) = dummy_assets::SAMPLE_PRICE_TIERS[1];
    card::root(
        CardProps::default(),
        vec![
            ("id", "plan"),
            ("data-blocks-settings-page-tabs-plan-card", ""),
        ],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    card::title(vec![], vec![text("現在のプラン")]),
                    card::action(
                        vec![],
                        vec![
                            badge::badge(
                                &BadgeProps {
                                    variant: BadgeVariant::Solid,
                                    ..BadgeProps::default()
                                },
                                vec![],
                                vec![text(tier_name)],
                            ),
                            button::button(
                                &ButtonProps {
                                    variant: ButtonVariant::Outline,
                                    size: Size::Sm,
                                    ..ButtonProps::default()
                                },
                                vec![],
                                vec![text("プランを変更")],
                            ),
                        ],
                    ),
                ],
            ),
            card::body(
                vec![("class", "blocks-settings-page-tabs-usage-body")],
                vec![
                    usage_bar("シート使用状況", 66.7, "8 / 12 シート".to_string()),
                    usage_bar("API 呼び出し", 64.0, "6,400 / 10,000 回".to_string()),
                ],
            ),
        ],
    )
}

/// 請求書 1 行分（すべて架空のダミー値、実在の企業・PII を含まない）。
struct InvoiceRow {
    number: &'static str,
    issued_on: &'static str,
    amount: &'static str,
    status: &'static str,
    status_variant: BadgeVariant,
    status_palette: ColorPalette,
    selected: bool,
}

/// 請求書 4 件（1 件を選択中として固定し、ヘッダーの全選択チェックボックス
/// は `Indeterminate` にする。状態違いの静的併記、モジュール doc参照）。
const INVOICES: [InvoiceRow; 4] = [
    InvoiceRow {
        number: "INV-2026-0091",
        issued_on: "2026-09-01",
        amount: "$29.00",
        status: "支払済み",
        status_variant: BadgeVariant::Subtle,
        status_palette: ColorPalette::Accent,
        selected: true,
    },
    InvoiceRow {
        number: "INV-2026-0078",
        issued_on: "2026-08-01",
        amount: "$29.00",
        status: "支払済み",
        status_variant: BadgeVariant::Subtle,
        status_palette: ColorPalette::Accent,
        selected: false,
    },
    InvoiceRow {
        number: "INV-2026-0065",
        issued_on: "2026-07-01",
        amount: "$34.00",
        status: "未払い",
        status_variant: BadgeVariant::Outline,
        status_palette: ColorPalette::Warning,
        selected: false,
    },
    InvoiceRow {
        number: "INV-2026-0052",
        issued_on: "2026-06-01",
        amount: "$29.00",
        status: "処理中",
        status_variant: BadgeVariant::Surface,
        status_palette: ColorPalette::Accent,
        selected: false,
    },
];

/// 請求書の行選択チェックボックス（`table_sortable_bulk::row_select_
/// checkbox` と同型）。`name` は行ごとに一意にし、`id` 属性は持たない
/// （`aria-label` のみで名前付け、id 重複検知を回避する）。
fn invoice_select_checkbox(name: &str, checked: CheckedState, label: &str) -> Node {
    let props = CheckboxProps {
        checked,
        ..CheckboxProps::default()
    };
    checkbox::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![],
        vec![
            checkbox::hidden_input(&props, name, "on", vec![("aria-label", label)]),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
        ],
    )
}

/// 請求書テーブルのフッター（件数表示 + ページ送り。1 ページ目固定・前
/// ページ無効、`table_with_toolbar::footer` と同型）。
fn invoices_footer() -> Node {
    div(
        vec![("class", "blocks-settings-page-tabs-invoices-footer")],
        vec![
            span(
                vec![("data-blocks-settings-page-tabs-invoices-count", "")],
                vec![text("4 件中 1–4 件を表示")],
            ),
            pagination::root(
                Size::Sm,
                ColorPalette::Accent,
                "請求書ページ",
                vec![("data-blocks-settings-page-tabs-pagination", "")],
                vec![
                    pagination::prev_trigger(ItemMode::Button, true, vec![], vec![text("前へ")]),
                    pagination::item(ItemMode::Button, 1, true, false, vec![], vec![text("1")]),
                    pagination::item(ItemMode::Button, 2, false, false, vec![], vec![text("2")]),
                    pagination::item(ItemMode::Button, 3, false, false, vec![], vec![text("3")]),
                    pagination::next_trigger(ItemMode::Button, false, vec![], vec![text("次へ")]),
                ],
            ),
        ],
    )
}

/// 「請求書」カード（`id="billing"`。`table::scroll_area` で包んだ表 +
/// [`invoices_footer`]）。
fn invoices_table_card() -> Node {
    let body_rows: Vec<Node> = INVOICES
        .iter()
        .enumerate()
        .map(|(i, invoice)| {
            let checked = if invoice.selected {
                CheckedState::Checked
            } else {
                CheckedState::Unchecked
            };
            let name = format!("invoice-{i}");
            let label = format!("{}を選択", invoice.number);
            table::row(
                vec![],
                vec![
                    table::cell(
                        vec![("data-blocks-settings-page-tabs-select-cell", "")],
                        vec![invoice_select_checkbox(&name, checked, &label)],
                    ),
                    table::row_header(vec![], vec![text(invoice.number)]),
                    table::cell(vec![], vec![text(invoice.issued_on)]),
                    table::cell(vec![], vec![text(invoice.amount)]),
                    table::cell(
                        vec![],
                        vec![badge::badge(
                            &BadgeProps {
                                variant: invoice.status_variant,
                                palette: invoice.status_palette,
                                ..BadgeProps::default()
                            },
                            vec![],
                            vec![text(invoice.status)],
                        )],
                    ),
                    table::cell(
                        vec![],
                        vec![button::button(
                            &ButtonProps {
                                variant: ButtonVariant::Ghost,
                                size: Size::Sm,
                                ..ButtonProps::default()
                            },
                            vec![],
                            vec![text("PDF")],
                        )],
                    ),
                ],
            )
        })
        .collect();

    card::root(
        CardProps::default(),
        vec![
            ("id", "billing"),
            ("data-blocks-settings-page-tabs-invoices-card", ""),
        ],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    card::title(vec![], vec![text("請求書")]),
                    card::action(
                        vec![],
                        vec![button::button(
                            &ButtonProps {
                                size: Size::Sm,
                                ..ButtonProps::default()
                            },
                            vec![],
                            vec![text("まとめてダウンロード")],
                        )],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![
                    table::scroll_area(
                        vec![
                            ("role", "region"),
                            ("aria-label", "請求書一覧"),
                            ("tabindex", "0"),
                        ],
                        vec![table::root(
                            TableProps::default(),
                            vec![],
                            vec![
                                table::header(
                                    vec![],
                                    vec![table::row(
                                        vec![],
                                        vec![
                                            table::column_header(
                                                vec![(
                                                    "data-blocks-settings-page-tabs-select-cell",
                                                    "",
                                                )],
                                                vec![invoice_select_checkbox(
                                                    "invoice-all",
                                                    CheckedState::Indeterminate,
                                                    "すべての請求書を選択",
                                                )],
                                            ),
                                            table::column_header(vec![], vec![text("請求書番号")]),
                                            table::column_header(vec![], vec![text("発行日")]),
                                            table::column_header(vec![], vec![text("金額")]),
                                            table::column_header(vec![], vec![text("状態")]),
                                            table::column_header(vec![], vec![text("操作")]),
                                        ],
                                    )],
                                ),
                                table::body(vec![], body_rows),
                            ],
                        )],
                    ),
                    invoices_footer(),
                ],
            ),
        ],
    )
}

/// 版 B（プラン、選択中タブ「プラン」）。[`plan_card`]/
/// [`invoices_table_card`] を持つ（モジュール doc「版の並記」節参照）。
fn version_plan() -> Node {
    section(
        vec![
            ("aria-label", "版 B: プラン"),
            ("data-blocks-settings-page-tabs-version", "plan"),
        ],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![("data-blocks-settings-page-tabs-version-title", "")],
                vec![text("版 B: プラン")],
            ),
            page_heading("plan", "設定セクション（プラン選択）"),
            div(
                vec![("class", "blocks-settings-page-tabs-content")],
                vec![plan_card(), invoices_table_card()],
            ),
        ],
    )
}

/// `settings-page-tabs` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（共有ナビバー + 版 A + 版 B + 一般/メンバーのスタブ）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-page-tabs-stack")],
        vec![
            navbar(),
            version_api(),
            version_plan(),
            other_section_stub("general", "一般"),
            other_section_stub("members", "メンバー"),
        ],
    )
}
```

## 原案差分メモ

- 骨格はナビバー（ダッシュボード / プロジェクト / 設定〔現在地〕+ 主操作
  ボタン 1 個）→ タブ見出し（一般 / メンバー / API / プラン / 請求）
  → 本文カード列、の順で構成します（親 issue のレイアウト要件どおり）。
- 版 A（API 設定、主参照 R0660）は「API アクセス」カード（マスク済み
  ダミーキーの読み取り専用 `input_group` + 「再生成」ボタン）と「発行済み
  API キー」カード（`table::scroll_area` で包んだ表、行操作は `menu`、
  権限は `badge`）の 2 枚で構成します。
- 版 B（プラン、集約元 R0658）は「現在のプラン」カード（プランバッジ +
  `progress` 2 本）と「請求書」カード（`table::scroll_area` で包んだ表、
  行選択は `checkbox`、状態は `badge`、フッターに件数表示 + `pagination`）
  の 2 枚で構成します。
- 選択中タブが異なる 2 版（版 A: API、版 B: プラン）を共有ナビバーの下へ
  縦に並記し、無 JS のため各版の選択中タブを静的に固定表示します。タブの
  `href` は API/プラン/請求が実在の本文 `id`（`api`/`plan`/`billing`）を、
  一般/メンバーは準備中スタブを指します。
- 請求書テーブルの行選択チェックボックスは 1 行を選択済み、ヘッダーの
  全選択チェックボックスを `Indeterminate`（`aria-checked="mixed"`）に
  固定し、選択状態の違いを静的に併記します。ページ送りは `ItemMode::Button`
  固定で 1 ページ目・前ページ無効の状態のみを示します。
- 狭い幅でのタブ横スクロールは、`tab-nav` root へ常時 `overflow-x: auto;
  flex-wrap: nowrap; white-space: nowrap;` を宣言することで実現します
  （`@container` によるコンテナクエリ分岐は不要です）。

関連情報: [Navigation Menu](../themes/navigation-menu.md) /
[Tab Nav](../themes/tab-nav.md) / [Card](../themes/card.md) /
[Table](../themes/table.md) / [Badge](../themes/badge.md) /
[Menu](../themes/menu.md) / [Input Group](../themes/input-group.md) /
[Input](../themes/input.md) / [Button](../themes/button.md) /
[Progress](../themes/progress.md) / [Checkbox](../themes/checkbox.md) /
[Pagination](../themes/pagination.md)
