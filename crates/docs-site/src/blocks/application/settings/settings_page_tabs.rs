//! `settings-page-tabs` block（親トラッキング #3006「Blocks に
//! settings-page-tabs を追加する」配下。#3006 は規模 L のため 2 分割し、
//! 骨格・主要領域・登録一式（版 A: API 設定）を #3007（PR #3451 で main
//! マージ済み）、残り領域（版 B: プラン・進捗・請求書テーブル + ページ
//! 送り）・状態並記（選択中タブの並記）・原案差分メモの仕上げを #3008
//! （本ファイル）で完了させた。対応表 ID は主参照 R0660（版 A の代表構成）、
//! 集約元 R0658（版 B へ集約）。`_/blocks-intake/` の対応ファイルはメイン
//! worktree に存在しないため、`profile-detail-datalist`（#2937）と同じ
//! 扱いとし、issue のレイアウト仕様のみから実装する（参照元の再現は
//! 行わない）。
//!
//! # 構成（ナビバー → タブ見出し → 本文カード列）
//!
//! 上部に 1 段のアプリ用ナビバー、その下にタブ状のセクション切替を持つ
//! ページ見出し、本文に設定カードを縦に並べる。無 JS のため選択中タブを
//! 初期状態として固定表示する。狭幅ではタブ列を横スクロールさせる（下記
//! 「狭幅ではタブを横スクロールする」節）。
//!
//! # 版の並記 = タブ選択状態の並記（#3008）
//!
//! 実運用ではタブ操作に応じて選択中セクションが切り替わるが、本 Demo は
//! 無 JS の静的表示しかできない。そのため「共有ナビバー 1 段 + 版 A + 版
//! B」の縦積みとし、各版が [`page_heading`]（タブ見出し）+ 本文カード列を
//! 個別に持ち、選択中タブだけが異なる状態を静的に併記する（`table_
//! sortable_bulk`/`settings_billing_usage` 等と同型の判断）。ナビバー
//! ([`navbar`]) は 1 個のみで、版ごとに複製しない（ランドマーク名・`id`
//! 重複を避ける）。各版は [`fandhe_frontend_core::section`] で包み、
//! `aria-label`（「版 A: API 設定」/「版 B: プラン」）+
//! `data-blocks-settings-page-tabs-version` 属性で区別し、先頭に
//! [`fandhe_frontend_pre_styled_ui::heading::heading`]（H3）のキャプションを
//! 置く（`heading` は `data-scope="heading"` を持ち TOC 収集から除外される
//! ため、目次を汚染しない。素の `h3` は使わない）。`tab_nav::root` の
//! `aria-label` も版ごとに一意化する（「設定セクション（API 選択）」/
//! 「設定セクション（プラン選択）」）。
//!
//! | 版 | 選択タブ | 本文 | 集約元 |
//! |---|---|---|---|
//! | A | API | API アクセスカード + 発行済み API キー表 | R0660（代表構成） |
//! | B | プラン | 現在のプランカード（`progress` 2 本）+ 請求書テーブル
//!   カード（`checkbox` 行選択 + `badge` 状態 + `pagination`） | R0658 |
//!
//! # 使用部品
//!
//! `navigation-menu` / `tab-nav` / `card` / `table` / `badge` / `menu` /
//! `input-group` / `input` / `button` / `progress` / `checkbox` /
//! `pagination` の 12 部品を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。新しい UI 部品は追加しない。
//!
//! # チェックボックス・ページ送りは無 JS の静的固定
//!
//! 請求書テーブルの行選択チェックボックスは `id` を持たず（`aria-label`
//! のみで名前付け、`table_sortable_bulk::row_select_checkbox` と同型）、
//! 1 行を `CheckedState::Checked`、ヘッダーの全選択チェックボックスを
//! `CheckedState::Indeterminate`（`aria-checked="mixed"`）に固定し、
//! チェック状態の違いを静的に並記する。ページ送り（`pagination`）は
//! `ItemMode::Button` 固定で、`INVOICES` 4 件が 1 ページに収まるため
//! `prev_trigger`/`next_trigger` を両方 disabled・1 ページ目を
//! `aria-current="page"` として固定表示する（`table_with_toolbar::footer`
//! と同型）。
//!
//! # ナビバー（`navigation_menu` + `button`）
//!
//! [`super::super::navbar::navbar_app_links`] と同じ語彙で、ブランド文言 +
//! メインナビ（ダッシュボード / プロジェクト / 設定〔現在地〕）+ 右端に
//! 1 個の Ghost/Sm ボタンを置く最小構成にする（本 block の主眼はタブ以下の
//! 設定領域のため、ナビバー自体はハンバーガー折り畳み等を持たない単純な
//! 1 段バーとする）。
//!
//! # タブ見出し（`tab_nav`）
//!
//! `tab_nav::root` の `aria-label` は本 block 内で一意にする。「API」タブへ
//! `aria-current="page"` を固定する。無 JS のため他タブは実在フラグメント
//! （`href="#<id>"`）で [`other_section_stub`] のスタブ見出しへ遷移し、本
//! Demo〔設定画面〕からは離脱しない（`href` の方針は「ナビは実在 URL、タブ
//! は現在地のみ `./`・他タブは実在フラグメント」節参照）。
//!
//! # 狭幅ではタブを横スクロールする
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container` を使わず
//! ネイティブ CSS のみで常時 `overflow-x: auto` を宣言する（親 issue の
//! 「狭幅ではタブを横スクロール」要件はいつでも横スクロール可能であれば
//! 満たされるため、コンテナクエリでの分岐は不要）。[`LAYOUT_CSS`] が
//! `data-blocks-settings-page-tabs-tabs` 付きの `tab-nav` root へ
//! `overflow-x: auto; flex-wrap: nowrap; white-space: nowrap;` を宣言する。
//! `.blocks-settings-page-tabs-heading` は親（`.blocks-settings-page-tabs-stack`
//! の flex column）の item であるため、`min-width: 0` を明示しないと縦積み
//! flex item の既定最小幅（内容の自然幅）に縛られ、子の `overflow-x: auto`
//! が実際には発火せず設定ページ全体が横スクロールしてしまう
//! （Bugbot レビュー指摘の是正）。
//!
//! # 版 A: API 設定（`card` + `input_group` + `table` + `menu` + `badge`）
//!
//! 1 枚目のカード「API アクセス」は読み取り専用のマスク済みダミーキーを
//! 表示する `input_group`（末尾に「再生成」ボタンの addon）を持つ。2 枚目の
//! カード「発行済み API キー」は `table::scroll_area` で包んだ表（列: 名前
//! 〔`row_header`〕/ 先頭文字列 / 権限〔`badge`〕/ 最終使用 / 操作
//! 〔`menu`〕）を持つ。`menu` の `content_id` は行ごとに一意にする
//! （`blocks-settings-page-tabs-key-menu-{n}`、
//! `blocks_contract.rs::demo_output_has_no_dangling_aria_references_or_duplicate_ids`
//! の対象）。トリガーの `aria-label` も行名を含めて行ごとに一意にする
//! （`"<行名>の操作"`。全行同一の「行の操作」では対象行をスクリーン
//! リーダーが区別できないという codex/Bugbot レビュー指摘の是正）。無 JS
//! のため `disabled: true` 固定だが、既定の `[data-disabled]` スタイル
//! （`opacity: 0.5`）は「操作しても変わらない」ことを示す意図に反し他の
//! 非活性ボタンと表示強度が不揃いになるため、[`LAYOUT_CSS`] で
//! `opacity: 1` へ中和する（`page_heading_tabs.rs::view_switch` と同型の
//! 是正）。
//!
//! # `class` と `data-*` の使い分け
//!
//! `navigation_menu`/`tab_nav`/`card`/`table`/`menu`/`button`/`badge`/
//! `input_group`/`input` はいずれも `drop_class_attr` で呼び出し側
//! `class` を除去するため、これらへの CSS フックは
//! `data-blocks-settings-page-tabs-*` 属性で渡す。素の `<div>`/`<header>`
//! ラッパーのみ `class="blocks-settings-page-tabs-*"` を使う
//! （`navbar_app_links`/`profile_detail_datalist` と同型の判断）。
//!
//! # ナビは実在 URL、タブは現在地のみ `./`・他タブは実在フラグメント
//! （codex レビュー P1 是正 v2）
//!
//! docs サイトの link check（`crate::linkcheck`）は絶対パス（`/settings` 等）
//! を実在ページとして検証するため、サイトに存在しない架空パスは使えない
//! （`navbar_app_links.rs`・`comparison_table.rs` 等の既存 block も同じ制約
//! に従う）。ナビバーの別アプリ領域（ダッシュボード/プロジェクト）は
//! `navbar_app_links.rs` と同じ自リポジトリ・自組織の実在 URL
//! （`REPO`/`ORG`）へ遷移してよいが、タブは同一の設定画面内の他セクション
//! （一般/メンバー/API/プラン/請求）を表すため、`REPO`/`ORG` へリンクすると
//! 利用者が設定画面から離脱してしまう（当初実装への codex レビュー指摘）。
//!
//! 当初は全タブの `href` を `"./"` に統一していたが、これは非選択タブを
//! クリックしても同一ページを再読み込みするだけで操作結果と表示が一致せず、
//! 別 block（`page_heading_tabs.rs`、イシュー #2934）が確立した「無 JS の
//! 静的 SSR ページではページ内フラグメントリンクを実在させる」規約に反する
//! との指摘を受けた（codex レビュー P1 是正 v2）。本 block も同じ規約を
//! 採用する: 各版の現在地タブのみ自ページを指す `href="./"` を維持し
//! （すでに表示中のセクションへの自己参照であり誤解を生まない）、他タブは
//! `href="#<id>"` で実在の `id` へリンクする。#3008 で版 B（プラン・請求の
//! フル本文）を実装したため、API/プラン/請求の 3 タブは実在する `id`
//! （[`version_api`] の `id="api"`、[`plan_card`] の `id="plan"`、
//! [`invoices_table_card`] の `id="billing"`）へリンクし、一般/メンバーの
//! 2 タブのみ [`other_section_stub`] が出力する「準備中」の最小限スタブ
//! 見出しへリンクする（着手予定なし）。タブとして提示する以上は実在する
//! 遷移先を必ず持たせる（`href="#"`・`data:` URI は持ち込まない）。
//!
//! # アイコンは自作の線画
//!
//! `icon::icon` + `el("path", …)` の自作線画（`fill="none"` /
//! `stroke="currentColor"`）のみで構成する。装飾用途は
//! `IconProps { label: None, .. }`（`aria-hidden`）とし、可視テキストが
//! アクセシブルネームを担う。実在ブランドのロゴ・商標は持ち込まない。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。全ボタンは
//! `button::button`/`button::icon_button` の既定 `type="button"` のまま
//! 用いる。API キーは明らかなダミー値（マスク表記）で、実在サービスの鍵
//! 形式・実在の人物・企業・PII は含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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

/// 請求書テーブルのフッター（件数表示 + ページ送り。`INVOICES` 4 件が
/// 1 ページに収まるため前へ/次へとも無効固定、`table_with_toolbar::footer`
/// と同型）。
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
                    // 請求書は INVOICES 4 件のみで全件 1 ページに収まるため、
                    // ページ送りは前へ/次へとも disabled 固定（件数表示
                    // 「4 件中 1–4 件を表示」との整合、イシュー #3460 review）。
                    pagination::prev_trigger(ItemMode::Button, true, vec![], vec![text("前へ")]),
                    pagination::item(ItemMode::Button, 1, true, false, vec![], vec![text("1")]),
                    pagination::next_trigger(ItemMode::Button, true, vec![], vec![text("次へ")]),
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-page-tabs/",
    title: "settings-page-tabs",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_page_tabs.rs",
    demo_class: "blocks-settings-page-tabs",
    parts: &[
        Part {
            label: "Navigation Menu",
            path: "/themes/navigation-menu/",
        },
        Part {
            label: "Tab Nav",
            path: "/themes/tab-nav/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Progress",
            path: "/themes/progress/",
        },
        Part {
            label: "Checkbox",
            path: "/themes/checkbox/",
        },
        Part {
            label: "Pagination",
            path: "/themes/pagination/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_page_tabs` 固有のレイアウト規則（`--fandhe-*` トークンのみ
/// 使用）。セレクタは `.blocks-settings-page-tabs-*` /
/// `[data-blocks-settings-page-tabs-*]`、および styled 部品の
/// `[data-scope][data-part]` セレクタとの複合セレクタのみを用いる
/// （`navbar_app_links` と同型の判断）。
const LAYOUT_CSS: &str = "\
.blocks-settings-page-tabs-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-settings-page-tabs;\n}\n\
.blocks-settings-page-tabs-navbar {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n  padding-block: var(--fandhe-space-3);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-settings-page-tabs-logo] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  white-space: nowrap;\n}\n\
[data-blocks-settings-page-tabs-navbar-cta] {\n  margin-inline-start: auto;\n}\n\
.blocks-settings-page-tabs-heading {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
.blocks-settings-page-tabs-heading h2 {\n  border-top: none;\n  padding-top: 0;\n}\n\
[data-scope=\"tab-nav\"][data-part=\"root\"][data-blocks-settings-page-tabs-tabs] {\n  overflow-x: auto;\n  flex-wrap: nowrap;\n  white-space: nowrap;\n}\n\
.blocks-settings-page-tabs-content {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-settings-page-tabs-access-group] {\n  max-inline-size: 28rem;\n}\n\
.blocks-settings-page-tabs-stack [data-scope=\"menu\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 1;\n}\n\
[data-blocks-settings-page-tabs-version] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-settings-page-tabs-version-title] {\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-settings-page-tabs-usage-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-page-tabs-usage-row {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-settings-page-tabs-usage-row-header {\n  display: flex;\n  justify-content: space-between;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-settings-page-tabs-select-cell] {\n  width: 1%;\n  white-space: nowrap;\n}\n\
.blocks-settings-page-tabs-invoices-footer {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// 12 部品の `data-scope` が全て出現し、`type="button"` があり、
    /// `<form>`・`href="#"`・`<script`・`src="data:` を含まないこと。
    #[test]
    fn demo_composes_expected_parts_and_has_no_form() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"navigation-menu\"",
            "data-scope=\"tab-nav\"",
            "data-scope=\"card\"",
            "data-scope=\"table\"",
            "data-scope=\"badge\"",
            "data-scope=\"menu\"",
            "data-scope=\"input-group\"",
            "data-scope=\"field\"",
            "data-scope=\"button\"",
            "data-scope=\"progress\"",
            "data-scope=\"checkbox\"",
            "data-scope=\"pagination\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
    }

    /// 各版がそれぞれの現在地タブを固定表示し、他タブは実在する `id` へ
    /// リンクすること。`aria-current="page"` はナビの「設定」項目 1 +
    /// 版 A・版 B の現在地タブ計 2 + pagination の 1 ページ目 1 の計 4、
    /// `href="./"` はナビの「設定」+ 版 A/B の現在地タブ計 3 のみ出現する
    /// （codex レビュー P1 是正 v2、モジュール doc「版の並記」節参照）。
    #[test]
    fn each_version_shows_its_own_current_tab_with_real_targets() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"aria-current="page""#).count(), 4);
        assert_eq!(html.matches(r#"href="./""#).count(), 3);
        for id in ["general", "members", "api", "plan", "billing"] {
            assert!(
                html.contains(&format!("id=\"{id}\"")),
                "real target id={id} should exist"
            );
            assert!(
                html.contains(&format!("href=\"#{id}\"")),
                "some tab should link to #{id}"
            );
        }
    }

    /// menu の `content_id` が行数分すべて相異なり、`aria-controls` の
    /// 値と同数存在すること（宙吊り ARIA 参照・id 重複防止）。
    #[test]
    fn menu_content_ids_are_unique_and_referenced() {
        let html = render(&demo());
        for i in 0..3 {
            let content_id = format!("blocks-settings-page-tabs-key-menu-{i}");
            assert_eq!(
                html.matches(&format!(r#"id="{content_id}""#)).count(),
                1,
                "content_id {content_id} should appear exactly once"
            );
            assert!(html.contains(&format!(r#"aria-controls="{content_id}""#)));
        }
    }

    /// 行操作メニューの `aria-label` が行名を含み、行ごとに相異なること
    /// （codex/Bugbot レビュー指摘の是正、モジュール doc参照）。
    #[test]
    fn key_actions_menu_aria_labels_are_unique_per_row() {
        let html = render(&demo());
        for name in ["本番サーバー", "CI パイプライン", "検証環境"] {
            assert!(
                html.contains(&format!(r#"aria-label="{name}の操作""#)),
                "expected unique aria-label for row {name}"
            );
        }
    }

    /// `LAYOUT_CSS` に `<` が無く、横スクロール・コンテナ宣言を含むこと。
    /// 狭幅横スクロールを実際に発火させる `min-width: 0` と、無 JS の
    /// disabled 行メニューを他の非活性ボタンと揃える `opacity: 1` 中和も
    /// 含むこと（Bugbot レビュー指摘の是正）。
    #[test]
    fn layout_css_has_no_breakout_and_has_scroll_rules() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("overflow-x: auto;"));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("min-width: 0;"));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"menu\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 1;\n}"
        ));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-settings-page-tabs-stack\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-settings-page-tabs-stack");
    }

    /// ページ見出し「設定」の `h2` が `.docs-content h2`
    /// （`site_theme.rs`）の `border-top`/`padding-top` を継承しないこと
    /// （Bugbot 指摘。兄弟 block `settings_page_aside_nav` と同型の打ち消し）。
    #[test]
    fn page_heading_resets_docs_content_h2_rule() {
        assert!(LAYOUT_CSS.contains(".blocks-settings-page-tabs-heading h2"));
        assert!(LAYOUT_CSS.contains("border-top: none;"));
    }

    /// 版 B の進捗バー 2 本すべてに `aria-label` があること（`progress::root`
    /// は自動配線しないため明示が必須、モジュール doc「使用部品」節参照）。
    #[test]
    fn progressbars_have_aria_label() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"role="progressbar""#).count(), 2);
        assert!(html.contains(r#"aria-label="シート使用状況""#));
        assert!(html.contains(r#"aria-label="API 呼び出し""#));
    }

    /// 請求書の行選択チェックボックスが `name` 属性で一意に識別され、
    /// 1 行が選択済み・ヘッダーの全選択が `Indeterminate`
    /// （`aria-checked="mixed"`）であること（状態違いの静的併記、モジュール
    /// doc「チェックボックス・ページ送りは無 JS の静的固定」節参照）。
    #[test]
    fn invoice_checkboxes_have_unique_names_and_mixed_states() {
        let html = render(&demo());
        assert_eq!(html.matches("name=\"invoice-").count(), 5);
        assert_eq!(html.matches(r#"checked="""#).count(), 1);
        assert_eq!(html.matches(r#"aria-checked="mixed""#).count(), 1);
        for label in [
            "すべての請求書を選択",
            "INV-2026-0091を選択",
            "INV-2026-0078を選択",
            "INV-2026-0065を選択",
            "INV-2026-0052を選択",
        ] {
            assert!(
                html.contains(&format!(r#"aria-label="{label}""#)),
                "expected unique checkbox aria-label {label}"
            );
        }
    }

    /// ページ送りが 1 ページ目固定（前へ/次へとも無効・1 ページ目が
    /// `data-selected`）であること。請求書 4 件が 1 ページに収まるため
    /// ページ 2/3 は存在せず、件数表示「4 件中 1–4 件を表示」との矛盾
    /// （次へ有効 + 複数ページボタン）を再発させない固定テスト
    /// （イシュー #3460 review 是正）。
    #[test]
    fn pagination_is_fixed_on_first_page() {
        let html = render(&demo());
        assert!(html.contains(r#"data-part="prev-trigger""#));
        assert!(html.contains(r#"data-index="1""#));
        assert!(html.contains("disabled"));
        assert_eq!(
            html.matches(r#"data-part="item" type="button" data-index="#)
                .count(),
            1,
            "expected exactly one page item (page 1) since invoices fit on a single page"
        );
        let next_trigger_start = html
            .find(r#"data-part="next-trigger""#)
            .expect("next-trigger should be rendered");
        let next_trigger_end = html[next_trigger_start..]
            .find('>')
            .map(|i| next_trigger_start + i)
            .expect("next-trigger opening tag should be closed");
        assert!(
            html[next_trigger_start..next_trigger_end].contains("disabled"),
            "next-trigger should be disabled: only one page of invoices exists"
        );
    }

    /// `tab_nav::root` の `aria-label` が版ごとに一意であること（版の並記で
    /// ランドマーク名が衝突しないことの確認）。
    #[test]
    fn tab_nav_aria_labels_are_unique_per_version() {
        let html = render(&demo());
        assert!(html.contains(r#"aria-label="設定セクション（API 選択）""#));
        assert!(html.contains(r#"aria-label="設定セクション（プラン選択）""#));
    }
}
