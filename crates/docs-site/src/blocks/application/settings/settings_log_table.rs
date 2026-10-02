//! `settings-log-table` block（イシュー #2997。Application / Settings
//! カテゴリ 5 件目）。上部の絞り込み UI → 日時・操作者・イベント・結果の
//! ログテーブル → ページ送りを縦に積んだ合成例。主参照は対応表 ID
//! R0236（代表構成）・R0377（タブによる結果絞り込み）・R0379（選択欄 3 種 +
//! ページ送り）。`_/blocks-intake/` の対応ファイルは本イシュー着手時点で
//! 本 worktree に存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`settings-billing-usage`〔#2985〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `native-select` / `tabs` / `table` / `badge` / `avatar` / `button` /
//! `pagination` の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 2 版と集約元の対応（原稿「原案差分メモ」節と対になる索引）
//!
//! - **A（監査ログ）**: R0236 + R0379。期間・種類・結果の `native-select`
//!   3 種 + 適用/クリアボタンの絞り込みツールバー → ログテーブル →
//!   ページ送り。
//! - **B（配信ログ）**: R0377。`tabs`（すべて/成功/失敗/保留）で結果を
//!   絞り込む版。各タブの content は静的に確定した該当行のみを表示する
//!   （無 JS のため実絞り込みではなく初期状態固定の表示）。
//!
//! # 無 JS のため選択・タブ状態は初期状態固定
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は状態機械を
//! 持たない。版 A の `native-select` は `option` の 1 つに `selected` を
//! 固定し、版 B の `tabs` は `TabsProps::selected` を `"all"` に固定する
//! （`dashboard_01.rs::tabs_and_table_card` と同型の判断）。ページ送りも
//! `ItemMode::Button` の静的表示のみで、クリックしても状態は変化しない。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::field::root`]・
//! [`fandhe_frontend_pre_styled_ui::native_select::native_select`]・
//! [`fandhe_frontend_pre_styled_ui::button::button`]・
//! [`fandhe_frontend_pre_styled_ui::avatar::root`]・
//! [`fandhe_frontend_pre_styled_ui::table::root`]・
//! [`fandhe_frontend_pre_styled_ui::pagination::root`] はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-settings-log-table-*`）。レイアウト用ラッパー（素の
//! `<div>`）は `class="blocks-settings-log-table-*"` を使う
//! （`settings_billing_usage.rs` と同型の判断）。
//!
//! # `id` の重複防止（2 版で衝突させない）
//!
//! 版 A のフィールド id は `blocks-settings-log-table-audit-{period,kind,
//! result}`、版 B の tabs id は `blocks-settings-log-table-delivery-tabs`
//! とし、`crates/docs-site/tests/blocks_contract.rs` の `id` 重複検査に
//! 対して 2 版が衝突しないようにする。
//!
//! # ページ送りは `ItemMode::Button` のみ（死リンクを出さない）
//!
//! `href="#"` は使わず、[`fandhe_frontend_pre_styled_ui::pagination`] の
//! `ItemMode::Button` のみを使う。両版とも先頭ページを表示中のため
//! prev トリガーは `disabled: true`（ネイティブ `disabled` は headless 層が
//! 付与する）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。操作ボタンは
//! [`fandhe_frontend_pre_styled_ui::button::button`] の既定 `type="button"`
//! のまま用いる。
//!
//! # ダミー素材について
//!
//! 操作者名は `crate::blocks::dummy_assets::PERSON_NAMES`、アバター画像は
//! `dummy_assets::AVATAR_SRC` を使う。日時・イベント文言・件数はすべて
//! 本ファイル内の架空値であり、実在の人物・企業・メールアドレス・IP・
//! トークン等の PII/クレデンシャルを含まない（`settings_billing_usage.rs`
//! と同じ基準）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::pagination::{self, ItemMode};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::tabs::{self, ActivationMode, Orientation, TabItem, TabsProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// ログ 1 行の結果種別。[`palette`](LogStatus::palette)/[`label`](LogStatus::label)
/// でバッジへ写像する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogStatus {
    Success,
    Failure,
    Pending,
}

impl LogStatus {
    /// バッジ表示文言。
    fn label(self) -> &'static str {
        match self {
            LogStatus::Success => "成功",
            LogStatus::Failure => "失敗",
            LogStatus::Pending => "保留",
        }
    }

    /// バッジの `colorPalette`（モジュール doc冒頭の警告色方針と同じ判断:
    /// 成功=Success・失敗=Danger・保留=Warning）。
    fn palette(self) -> ColorPalette {
        match self {
            LogStatus::Success => ColorPalette::Success,
            LogStatus::Failure => ColorPalette::Danger,
            LogStatus::Pending => ColorPalette::Warning,
        }
    }
}

/// ログテーブル 1 行（日時・操作者・イベント・結果）。
struct LogRow {
    at: &'static str,
    actor: &'static str,
    event: &'static str,
    status: LogStatus,
}

/// 版 A（監査ログ）の全 5 行。
const AUDIT_ROWS: &[LogRow] = &[
    LogRow {
        at: "2026-09-28 09:12",
        actor: dummy_assets::PERSON_NAMES[0],
        event: "ログイン",
        status: LogStatus::Success,
    },
    LogRow {
        at: "2026-09-27 18:40",
        actor: dummy_assets::PERSON_NAMES[1],
        event: "権限を変更",
        status: LogStatus::Success,
    },
    LogRow {
        at: "2026-09-27 14:05",
        actor: dummy_assets::PERSON_NAMES[2],
        event: "API キーを再発行",
        status: LogStatus::Pending,
    },
    LogRow {
        at: "2026-09-26 11:22",
        actor: dummy_assets::PERSON_NAMES[0],
        event: "ログイン試行",
        status: LogStatus::Failure,
    },
    LogRow {
        at: "2026-09-25 08:50",
        actor: dummy_assets::PERSON_NAMES[3],
        event: "メンバーを招待",
        status: LogStatus::Success,
    },
];

/// 版 B（配信ログ）の全 6 行（成功 3・失敗 2・保留 1、タブ絞り込みの母集団）。
const DELIVERY_ROWS: &[LogRow] = &[
    LogRow {
        at: "2026-09-29 07:00",
        actor: dummy_assets::PERSON_NAMES[1],
        event: "Webhook 配信",
        status: LogStatus::Success,
    },
    LogRow {
        at: "2026-09-29 06:00",
        actor: dummy_assets::PERSON_NAMES[2],
        event: "メール送信",
        status: LogStatus::Success,
    },
    LogRow {
        at: "2026-09-28 22:15",
        actor: dummy_assets::PERSON_NAMES[3],
        event: "Webhook 配信",
        status: LogStatus::Success,
    },
    LogRow {
        at: "2026-09-28 21:40",
        actor: dummy_assets::PERSON_NAMES[0],
        event: "Webhook 配信",
        status: LogStatus::Failure,
    },
    LogRow {
        at: "2026-09-28 20:05",
        actor: dummy_assets::PERSON_NAMES[1],
        event: "メール送信",
        status: LogStatus::Failure,
    },
    LogRow {
        at: "2026-09-28 19:30",
        actor: dummy_assets::PERSON_NAMES[2],
        event: "Webhook 配信",
        status: LogStatus::Pending,
    },
];

/// 指定した結果でだけ [`DELIVERY_ROWS`] を絞り込んだ行を返す（`None` は
/// 全行、モジュール doc「無 JS のため選択・タブ状態は初期状態固定」節の
/// とおり実行時の絞り込みではなく初期状態固定の表示）。
fn delivery_rows(status: Option<LogStatus>) -> Vec<&'static LogRow> {
    DELIVERY_ROWS
        .iter()
        .filter(|row| status.is_none_or(|s| row.status == s))
        .collect()
}

/// タブバッジに出す各結果の総件数。[`DELIVERY_ROWS`] はページ送り
/// フッター「全 18 件中 1〜6 件を表示」の 1 ページ目分（6 件）のみを
/// 保持する架空データのため、`delivery_rows` の長さ（表示中ページ内件数）
/// をそのままバッジへ出すとフッターの総件数（18 件）と食い違う
/// （codex レビュー指摘、イシュー #2997）。バッジは「該当件数」を示す
/// 表示のため、フッターと整合する総件数側の架空値（合計 18 件）を返す。
fn delivery_total(status: Option<LogStatus>) -> u32 {
    match status {
        None => 18,
        Some(LogStatus::Success) => 9,
        Some(LogStatus::Failure) => 6,
        Some(LogStatus::Pending) => 3,
    }
}

/// 結果バッジ（[`LogStatus::palette`]/[`LogStatus::label`] を委譲）。
fn status_badge(status: LogStatus) -> Node {
    badge::badge(
        &BadgeProps {
            variant: BadgeVariant::Subtle,
            palette: status.palette(),
            ..BadgeProps::default()
        },
        vec![],
        vec![text(status.label())],
    )
}

/// 操作者セル（アバター Xs + 氏名）。
fn operator_cell(name: &'static str) -> Node {
    div(
        vec![("class", "blocks-settings-log-table-operator")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Xs,
                    ..AvatarProps::default()
                },
                vec![],
                vec![
                    avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, name, vec![]),
                    avatar::fallback(
                        ImageStatus::Loaded,
                        vec![],
                        vec![text(name.chars().take(1).collect::<String>())],
                    ),
                ],
            ),
            text(name),
        ],
    )
}

/// 日時・操作者・イベント・結果の 4 列テーブル（版 A/B 共通の骨格）。
/// `label` はテーブル・スクロールコンテナ双方の accessible name に使う
/// （版 A の監査ログ 1 件・版 B のタブ 4 件で計 5 テーブル、支援技術が
/// 互いを区別できるよう呼び出し側で一意な文言を渡す契約）。
fn log_table(rows: &[&LogRow], label: &str) -> Node {
    table::scroll_area(
        vec![("role", "region"), ("aria-label", label), ("tabindex", "0")],
        vec![table::root(
            TableProps {
                size: Size::Sm,
                ..TableProps::default()
            },
            vec![("aria-label", label)],
            vec![
                table::header(
                    vec![],
                    vec![table::row(
                        vec![],
                        vec![
                            table::column_header(vec![], vec![text("日時")]),
                            table::column_header(vec![], vec![text("操作者")]),
                            table::column_header(vec![], vec![text("イベント")]),
                            table::column_header(vec![], vec![text("結果")]),
                        ],
                    )],
                ),
                table::body(
                    vec![],
                    rows.iter()
                        .map(|row| {
                            table::row(
                                vec![],
                                vec![
                                    table::row_header(vec![], vec![text(row.at)]),
                                    table::cell(vec![], vec![operator_cell(row.actor)]),
                                    table::cell(vec![], vec![text(row.event)]),
                                    table::cell(vec![], vec![status_badge(row.status)]),
                                ],
                            )
                        })
                        .collect(),
                ),
            ],
        )],
    )
}

/// 絞り込みツールバーの `native-select` 1 個（`option` の 1 つを
/// `selected` 固定、モジュール doc「無 JS のため選択・タブ状態は初期状態
/// 固定」節参照）。
fn filter_field(id_suffix: &str, label_text: &'static str, options: &[(&str, &str, bool)]) -> Node {
    let id = format!("blocks-settings-log-table-audit-{id_suffix}");
    let props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &props,
        vec![("data-blocks-settings-log-table-field", "")],
        vec![
            field::label(&props, vec![], vec![text(label_text)]),
            native_select::native_select(
                &NativeSelectProps::default(),
                &props,
                vec![],
                options
                    .iter()
                    .map(|(value, label, selected)| {
                        let mut attrs = vec![("value", *value)];
                        if *selected {
                            attrs.push(("selected", ""));
                        }
                        el("option", attrs, vec![text(*label)])
                    })
                    .collect(),
            ),
        ],
    )
}

/// 件数サマリ + ページ送り（版 A/B 共通の骨格、常に先頭ページ表示中の
/// 静的表示）。`total_pages` は実件数から算出した実ページ数（版 A: 42 件 /
/// 5 件毎 = 9 ページ、版 B: 18 件 / 6 件毎 = 3 ページ）を渡す契約とし、
/// 存在しないページ番号を表示しない。4 ページ以下は省略記号を使わず
/// 全ページを列挙し、5 ページ以上は先頭 3 ページ + 省略記号 + 最終ページの
/// 形に畳む。
fn pager(aria_label: &'static str, summary: &'static str, total_pages: u64) -> Node {
    let mut items: Vec<Node> = vec![pagination::prev_trigger(
        ItemMode::Button,
        true,
        vec![],
        vec![text("前へ")],
    )];
    if total_pages <= 4 {
        for page in 1..=total_pages {
            items.push(pagination::item(
                ItemMode::Button,
                page,
                page == 1,
                false,
                vec![],
                vec![text(page.to_string())],
            ));
        }
    } else {
        for page in 1..=3 {
            items.push(pagination::item(
                ItemMode::Button,
                page,
                page == 1,
                false,
                vec![],
                vec![text(page.to_string())],
            ));
        }
        items.push(pagination::ellipsis(vec![], vec![text("…")]));
        items.push(pagination::item(
            ItemMode::Button,
            total_pages,
            false,
            false,
            vec![],
            vec![text(total_pages.to_string())],
        ));
    }
    items.push(pagination::next_trigger(
        ItemMode::Button,
        false,
        vec![],
        vec![text("次へ")],
    ));

    div(
        vec![("class", "blocks-settings-log-table-footer")],
        vec![
            text(summary),
            pagination::root(Size::Sm, ColorPalette::Accent, aria_label, vec![], items),
        ],
    )
}

/// A: 選択欄 3 種 + ページ送り（監査ログ、R0236 + R0379）。
fn version_audit() -> Node {
    let toolbar = div(
        vec![("class", "blocks-settings-log-table-toolbar")],
        vec![
            filter_field(
                "period",
                "期間",
                &[
                    ("7d", "過去 7 日間", false),
                    ("30d", "過去 30 日間", true),
                    ("90d", "過去 90 日間", false),
                ],
            ),
            filter_field(
                "kind",
                "種類",
                &[
                    ("all", "すべて", true),
                    ("auth", "認証", false),
                    ("admin", "管理操作", false),
                ],
            ),
            filter_field(
                "result",
                "結果",
                &[
                    ("all", "すべて", true),
                    ("success", "成功のみ", false),
                    ("failure", "失敗のみ", false),
                ],
            ),
            div(
                vec![("class", "blocks-settings-log-table-toolbar-actions")],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            size: Size::Sm,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("絞り込みを適用")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            size: Size::Sm,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("条件をクリア")],
                    ),
                ],
            ),
        ],
    );

    div(
        vec![
            ("class", "blocks-settings-log-table-variant"),
            ("data-variant", "a"),
        ],
        vec![
            div(
                vec![("class", "blocks-settings-log-table-variant-label")],
                vec![text("A: 選択欄で絞り込み（監査ログ）")],
            ),
            toolbar,
            log_table(&AUDIT_ROWS.iter().collect::<Vec<_>>(), "監査ログ"),
            pager("監査ログのページ送り", "全 42 件中 1〜5 件を表示", 9),
        ],
    )
}

/// B: タブで結果を絞り込む配信ログ（R0377）。
fn version_delivery() -> Node {
    let tab = |value: &'static str, trigger_label: &'static str, status: Option<LogStatus>| {
        let rows = delivery_rows(status);
        let mut trigger = vec![text(trigger_label)];
        trigger.push(text(" "));
        trigger.push(badge::badge(
            &BadgeProps::default(),
            vec![],
            vec![text(delivery_total(status).to_string())],
        ));
        let label = format!("配信ログ（{trigger_label}）");
        TabItem {
            value,
            trigger,
            content: vec![log_table(&rows, &label)],
            disabled: false,
        }
    };

    let items = vec![
        tab("all", "すべて", None),
        tab("success", "成功", Some(LogStatus::Success)),
        tab("failure", "失敗", Some(LogStatus::Failure)),
        tab("pending", "保留", Some(LogStatus::Pending)),
    ];

    let props = TabsProps {
        id: "blocks-settings-log-table-delivery-tabs",
        selected: "all",
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };

    div(
        vec![
            ("class", "blocks-settings-log-table-variant"),
            ("data-variant", "b"),
        ],
        vec![
            div(
                vec![("class", "blocks-settings-log-table-variant-label")],
                vec![text("B: タブで結果を絞り込み（配信ログ）")],
            ),
            tabs::tabs(
                tabs::TabsVariant::Line,
                Size::Md,
                ColorPalette::Accent,
                &props,
                items,
            ),
            pager("配信ログのページ送り", "全 18 件中 1〜6 件を表示", 3),
        ],
    )
}

/// `settings-log-table` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-log-table-stack")],
        vec![version_audit(), version_delivery()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-log-table/",
    title: "settings-log-table",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_log_table.rs",
    demo_class: "blocks-settings-log-table",
    parts: &[
        Part {
            label: "Native Select",
            path: "/themes/native-select/",
        },
        Part {
            label: "Tabs",
            path: "/themes/tabs/",
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
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Pagination",
            path: "/themes/pagination/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_log_table` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-settings-log-table-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  container-type: inline-size;\n  container-name: blocks-settings-log-table;\n}\n\
.blocks-settings-log-table-variant {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-log-table-variant-label {\n  font-weight: 600;\n}\n\
.blocks-settings-log-table-toolbar {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: flex-end;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-settings-log-table-field] {\n  min-width: 10rem;\n}\n\
.blocks-settings-log-table-toolbar-actions {\n  display: flex;\n  gap: var(--fandhe-space-2);\n  margin-inline-start: auto;\n}\n\
.blocks-settings-log-table-operator {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  white-space: nowrap;\n}\n\
.blocks-settings-log-table-footer {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n  margin-block-start: var(--fandhe-space-3);\n}\n\
@container blocks-settings-log-table (max-width: 36rem) {\n  \
[data-blocks-settings-log-table-field] {\n    width: 100%;\n  }\n  \
.blocks-settings-log-table-toolbar-actions {\n    margin-inline-start: 0;\n  }\n\
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
            "data-scope=\"field\"",
            "data-scope=\"tabs\"",
            "data-scope=\"table\"",
            "data-scope=\"badge\"",
            "data-scope=\"avatar\"",
            "data-scope=\"button\"",
            "data-scope=\"pagination\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<select").count(), 3);
        // `data-selected=""`（pagination の current item）と区別するため、
        // 直前が空白の ` selected=""`（`<option>` のネイティブ属性）のみ数える。
        assert_eq!(html.matches(r#" selected="""#).count(), 3);
        assert_eq!(html.matches(r#"data-part="prev-trigger""#).count(), 2);
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"submit\""));
        assert!(html.contains("../../assets/blocks-demo-avatar.svg"));
    }

    #[test]
    fn ids_are_unique() {
        let html = demo_html();
        let mut ids: Vec<&str> = Vec::new();
        for chunk in html.split("id=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                ids.push(&chunk[..end]);
            }
        }
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(ids.len(), sorted.len(), "id が重複している: {ids:?}");
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-settings-log-table (max-width: 36rem)"));
    }

    #[test]
    fn status_badges_use_semantic_palettes() {
        let html = demo_html();
        assert!(html.contains("fd-badge--color-palette-success"));
        assert!(html.contains("fd-badge--color-palette-danger"));
        assert!(html.contains("fd-badge--color-palette-warning"));
    }

    #[test]
    fn demo_has_two_variants() {
        let html = demo_html();
        assert!(html.contains(r#"data-variant="a""#));
        assert!(html.contains(r#"data-variant="b""#));
        assert_eq!(html.matches("data-variant=").count(), 2);
    }
}
