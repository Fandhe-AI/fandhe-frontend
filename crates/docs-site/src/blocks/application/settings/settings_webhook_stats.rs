//! `settings-webhook-stats` block（イシュー #3020。Application / Settings
//! カテゴリ）。Webhook エンドポイント詳細画面を「統計付き」で表す合成例。
//! 上段の配信統計カード 4 枚 → 配信推移バー・署名シークレット・直近の
//! 配信一覧テーブルを下段へ積む。主参照は対応表 ID R0373（代表構成）・
//! 集約元 R0371（統計 3 枚 + タイムライン）。`_/blocks-intake/` の対応
//! ファイルは本イシュー着手時点で本 worktree に存在しないため、原稿・
//! 本コメントには対応表 ID のみを記す（`settings_billing_overview.rs` と
//! 同じ扱い）。
//!
//! # 使用部品
//!
//! `stat` / `card` / `table` / `timeline` / `progress` / `button` /
//! `clipboard` / `badge` の 8 部品を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新しい UI 部品は追加しない。
//!
//! # R0373/R0371 の畳み込み方（Demo は単一構成）
//!
//! R0373（統計カード + テーブル）の骨格を主体とし、R0371 が持つ
//! タイムライン表現は下段の「イベントタイムライン」カードとして同じ
//! Demo 内に追加することで両参照を 1 構成へ畳み込む（版 A/B を別々に
//! 並記しない。原稿「原案差分メモ」節に対応を明記する）。
//!
//! # 失敗した配信の強調
//!
//! テーブル行の状態バッジ・タイムラインの状態バッジはいずれも
//! `BadgeVariant::Subtle` + `ColorPalette::Danger`（失敗）/
//! `ColorPalette::Warning`（再試行中）/ `ColorPalette::Success`（成功）で
//! 強調する（`settings_log_table.rs::status_badge` と同型の判断）。行の
//! 背景色は変えない。
//!
//! # 狭幅での統計カード列数切り替え
//!
//! [`LAYOUT_CSS`] は `.blocks-settings-webhook-stats-stack` へ
//! `container-type: inline-size` を宣言し、`@container` で統計グリッドを
//! `48rem` 未満で 2 列・`28rem` 未満で 1 列へ切り替える（Issue 要件
//! 「狭幅では統計カードを 2 列 → 1 列へ減らす」節）。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::card::root`]・
//! [`fandhe_frontend_pre_styled_ui::stat::root`]・
//! [`fandhe_frontend_pre_styled_ui::table::root`]・
//! [`fandhe_frontend_pre_styled_ui::timeline::root`]・
//! [`fandhe_frontend_pre_styled_ui::progress::root`]・
//! [`fandhe_frontend_pre_styled_ui::button::button`]・
//! [`fandhe_frontend_pre_styled_ui::clipboard::root`]（headless 直接委譲）・
//! [`fandhe_frontend_pre_styled_ui::badge::badge`] はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-settings-webhook-stats-*`）。素の `<div>` ラッパー・
//! `card::header`/`body`/`footer`・`stat::label`/`value_text`/
//! `help_text`・`table::row`/`cell` は素の
//! `class="blocks-settings-webhook-stats-*"` を使う。
//!
//! # 無 JS のため `clipboard`/`progress` は静的固定
//!
//! `clipboard::root` は `copied=false` 固定（未コピー状態の静的表示のみ、
//! `hero_install_command.rs` の idle インスタンスと同型）。`progress::root`
//! は `Progress::new` で `value` を固定した determinate 表示のみで、
//! インタラクションは一切持たない。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。「テスト
//! 送信」「無効化」ボタンは [`fandhe_frontend_pre_styled_ui::button::button`]
//! の既定 `type="button"` のまま `disabled: true` で押下不能にする
//! （`hero_install_command.rs` の CTA ボタンと同型の判断）。
//!
//! # ダミー素材・秘密情報の扱いについて
//!
//! エンドポイント URL は `https://example.com/...` ドメイン固定。署名
//! シークレットは実在トークン形式を模さない架空の伏せ字表示
//! （`whsec_••••••••••••3f9a`）であり、全桁の実値は出力しない
//! （`crates/docs-site/tests/blocks_contract.rs` 相当の秘匿方針）。統計値・
//! 配信一覧・タイムラインの日時・件数はすべて本ファイル内の架空値で、
//! 実在の企業・人物・メールアドレス・クレデンシャル・PII は含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
use fandhe_frontend_pre_styled_ui::progress::{self, Orientation, ProgressProps};
use fandhe_frontend_pre_styled_ui::recipe::{ColorPalette, Size};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::timeline::{self, TimelineVariant};

/// 配信結果の種別。バッジ・タイムライン両方で共有する（`settings_log_table.rs
/// ::LogStatus` と同型のマッピング方針）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeliveryStatus {
    Success,
    Failure,
    Pending,
}

impl DeliveryStatus {
    fn label(self) -> &'static str {
        match self {
            DeliveryStatus::Success => "成功",
            DeliveryStatus::Failure => "失敗",
            DeliveryStatus::Pending => "再試行中",
        }
    }

    fn palette(self) -> ColorPalette {
        match self {
            DeliveryStatus::Success => ColorPalette::Success,
            DeliveryStatus::Failure => ColorPalette::Danger,
            DeliveryStatus::Pending => ColorPalette::Warning,
        }
    }
}

/// 状態バッジ（[`DeliveryStatus::palette`]/[`DeliveryStatus::label`] を委譲、
/// `settings_log_table.rs::status_badge` と同型）。
fn status_badge(status: DeliveryStatus) -> Node {
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

/// 上段の統計カード 1 枚分のデータ。
struct StatDatum {
    label: &'static str,
    value: &'static str,
    unit: Option<&'static str>,
    trend_up: bool,
    help: &'static str,
}

/// 上段 4 統計（配信数・成功率・平均応答時間・失敗件数、いずれも架空値）。
const STATS: &[StatDatum] = &[
    StatDatum {
        label: "配信数",
        value: "1,284",
        unit: None,
        trend_up: true,
        help: "過去 30 日間",
    },
    StatDatum {
        label: "成功率",
        value: "98.4",
        unit: Some("%"),
        trend_up: true,
        help: "先月比で改善",
    },
    StatDatum {
        label: "平均応答時間",
        value: "212",
        unit: Some("ms"),
        trend_up: false,
        help: "先月比で短縮",
    },
    StatDatum {
        label: "失敗",
        value: "21",
        unit: None,
        trend_up: false,
        help: "直近 30 日間の累計",
    },
];

/// 配信推移 1 日分（日付ラベル・成功率・説明文・強調有無）。
struct TrendDay {
    date: &'static str,
    percent: f64,
    detail: &'static str,
    danger: bool,
}

/// 直近 7 日分の配信成功率推移（架空値、末尾 2 日を失敗率上昇で強調）。
const TRENDS: &[TrendDay] = &[
    TrendDay {
        date: "9/23",
        percent: 99.0,
        detail: "9/23: 成功率 99%",
        danger: false,
    },
    TrendDay {
        date: "9/24",
        percent: 98.0,
        detail: "9/24: 成功率 98%",
        danger: false,
    },
    TrendDay {
        date: "9/25",
        percent: 99.0,
        detail: "9/25: 成功率 99%",
        danger: false,
    },
    TrendDay {
        date: "9/26",
        percent: 97.0,
        detail: "9/26: 成功率 97%",
        danger: false,
    },
    TrendDay {
        date: "9/27",
        percent: 94.0,
        detail: "9/27: 成功率 94%（低下傾向）",
        danger: true,
    },
    TrendDay {
        date: "9/28",
        percent: 91.0,
        detail: "9/28: 成功率 91%（低下傾向）",
        danger: true,
    },
    TrendDay {
        date: "9/29",
        percent: 98.0,
        detail: "9/29: 成功率 98%（回復）",
        danger: false,
    },
];

/// 直近の配信一覧テーブル 1 行。
struct DeliveryRow {
    at: &'static str,
    event: &'static str,
    http_status: &'static str,
    latency: &'static str,
    status: DeliveryStatus,
}

/// 直近の配信 5 件（架空値、2 件を失敗・再試行中にする）。
const DELIVERIES: &[DeliveryRow] = &[
    DeliveryRow {
        at: "2026-09-29 07:02",
        event: "invoice.paid",
        http_status: "200",
        latency: "188 ms",
        status: DeliveryStatus::Success,
    },
    DeliveryRow {
        at: "2026-09-29 06:40",
        event: "invoice.paid",
        http_status: "200",
        latency: "204 ms",
        status: DeliveryStatus::Success,
    },
    DeliveryRow {
        at: "2026-09-29 05:58",
        event: "customer.updated",
        http_status: "503",
        latency: "4,021 ms",
        status: DeliveryStatus::Failure,
    },
    DeliveryRow {
        at: "2026-09-28 22:15",
        event: "subscription.canceled",
        http_status: "429",
        latency: "512 ms",
        status: DeliveryStatus::Pending,
    },
    DeliveryRow {
        at: "2026-09-28 20:03",
        event: "invoice.paid",
        http_status: "200",
        latency: "176 ms",
        status: DeliveryStatus::Success,
    },
];

/// タイムライン 1 件分（イベント名・時刻・結果バッジの有無）。
struct TimelineEntry {
    at: &'static str,
    title: &'static str,
    status: Option<DeliveryStatus>,
}

/// イベントタイムライン 4 件（R0371 の集約元、架空値）。
const TIMELINE_ENTRIES: &[TimelineEntry] = &[
    TimelineEntry {
        at: "2026-09-29 07:02",
        title: "配信成功",
        status: Some(DeliveryStatus::Success),
    },
    TimelineEntry {
        at: "2026-09-29 05:58",
        title: "配信失敗（再試行 1/3）",
        status: Some(DeliveryStatus::Failure),
    },
    TimelineEntry {
        at: "2026-09-20 10:00",
        title: "シークレットを再生成",
        status: None,
    },
    TimelineEntry {
        at: "2026-08-02 09:30",
        title: "エンドポイントを作成",
        status: None,
    },
];

/// 1 枚の統計カード（`card` + `stat` の合成、`settings_billing_overview.rs
/// ::stat_card` と同型）。
fn stat_card(datum: &StatDatum) -> Node {
    let mut value_children = vec![stat::value_text(vec![], vec![text(datum.value)])];
    if let Some(unit) = datum.unit {
        value_children.push(stat::value_unit(vec![], vec![text(unit)]));
    }

    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-webhook-stats-stat-card", "")],
        vec![
            card::header(
                vec![],
                vec![card::title(
                    vec![("class", "blocks-settings-webhook-stats-stat-label")],
                    vec![text(datum.label)],
                )],
            ),
            card::body(vec![], vec![stat::root(Size::Lg, vec![], value_children)]),
            card::footer(
                vec![],
                vec![stat::help_text(
                    vec![],
                    vec![
                        if datum.trend_up {
                            stat::up_indicator(vec![])
                        } else {
                            stat::down_indicator(vec![])
                        },
                        text(datum.help),
                    ],
                )],
            ),
        ],
    )
}

/// 配信推移 1 日分の横棒（`progress::root` + 日付・説明ラベル）。
fn trend_bar(day: &TrendDay) -> Node {
    let progress = Progress::new(0.0, 100.0, Some(day.percent), Orientation::Horizontal);
    let props = ProgressProps {
        palette: if day.danger {
            ColorPalette::Danger
        } else {
            ColorPalette::Success
        },
        ..ProgressProps::default()
    };

    div(
        vec![("class", "blocks-settings-webhook-stats-trend-row")],
        vec![
            div(
                vec![("class", "blocks-settings-webhook-stats-trend-date")],
                vec![text(day.date)],
            ),
            progress::root(
                &progress,
                &props,
                Some(day.detail),
                vec![],
                vec![progress.track(vec![], vec![progress::range(&progress, vec![])])],
            ),
            div(
                vec![("class", "blocks-settings-webhook-stats-trend-percent")],
                vec![text(format!("{:.0}%", day.percent))],
            ),
        ],
    )
}

/// 署名シークレットのコピー可能表示（`copied=false` 固定、伏せ字混じりの
/// 架空値。モジュール doc「無 JS のため `clipboard`/`progress` は静的固定」
/// 節参照）。
fn secret_clipboard() -> Node {
    const SECRET_INPUT_ID: &str = "blocks-settings-webhook-stats-secret";
    const MASKED_SECRET: &str = "whsec_••••••••••••3f9a";

    clipboard::root(
        MASKED_SECRET,
        false,
        vec![("data-blocks-settings-webhook-stats-secret", "")],
        vec![
            clipboard::label(
                false,
                Some(SECRET_INPUT_ID),
                vec![],
                vec![text("署名シークレット")],
            ),
            clipboard::control(
                false,
                vec![],
                vec![
                    clipboard::value_text(vec![("id", SECRET_INPUT_ID)], vec![text(MASKED_SECRET)]),
                    clipboard::trigger(
                        false,
                        vec![],
                        vec![
                            clipboard::indicator(false, false, vec![], vec![text("Copy")]),
                            clipboard::indicator(true, false, vec![], vec![text("Copied!")]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 直近の配信一覧テーブル（日時・イベント・HTTP ステータス・応答時間・
/// 結果の 5 列）。
fn deliveries_table() -> Node {
    const TABLE_LABEL: &str = "直近の配信一覧";

    table::scroll_area(
        vec![
            ("role", "region"),
            ("aria-label", TABLE_LABEL),
            ("tabindex", "0"),
        ],
        vec![table::root(
            TableProps {
                size: Size::Sm,
                ..TableProps::default()
            },
            vec![("aria-label", TABLE_LABEL)],
            vec![
                table::header(
                    vec![],
                    vec![table::row(
                        vec![],
                        vec![
                            table::column_header(vec![], vec![text("日時")]),
                            table::column_header(vec![], vec![text("イベント")]),
                            table::column_header(vec![], vec![text("HTTP ステータス")]),
                            table::column_header(vec![], vec![text("応答時間")]),
                            table::column_header(vec![], vec![text("結果")]),
                        ],
                    )],
                ),
                table::body(
                    vec![],
                    DELIVERIES
                        .iter()
                        .map(|row| {
                            table::row(
                                vec![],
                                vec![
                                    table::row_header(vec![], vec![text(row.at)]),
                                    table::cell(vec![], vec![text(row.event)]),
                                    table::cell(vec![], vec![text(row.http_status)]),
                                    table::cell(vec![], vec![text(row.latency)]),
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

/// イベントタイムライン（R0371 の集約元。最終 item は `separator` を
/// 持たない、`timeline` モジュール doc の不変条件どおり）。
fn event_timeline() -> Node {
    let last_index = TIMELINE_ENTRIES.len().saturating_sub(1);
    let items = TIMELINE_ENTRIES
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let mut connector_children = vec![timeline::indicator(vec![], vec![])];
            if index != last_index {
                connector_children.push(timeline::separator(vec![], vec![]));
            }
            let connector = timeline::connector(vec![], connector_children);

            let mut content_children = vec![timeline::title(vec![], vec![text(entry.title)])];
            if let Some(status) = entry.status {
                content_children.push(status_badge(status));
            }
            content_children.push(timeline::description(vec![], vec![text(entry.at)]));
            let content = timeline::content(vec![], content_children);

            timeline::item(vec![], vec![connector, content])
        })
        .collect();

    timeline::root(
        TimelineVariant::default(),
        Size::Sm,
        ColorPalette::default(),
        vec![],
        items,
    )
}

/// `settings-webhook-stats` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    let header = div(
        vec![("class", "blocks-settings-webhook-stats-header")],
        vec![
            card::title(vec![], vec![text("注文イベント Webhook")]),
            badge::badge(
                &BadgeProps {
                    variant: BadgeVariant::Subtle,
                    palette: ColorPalette::Success,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text("有効")],
            ),
            div(
                vec![("class", "blocks-settings-webhook-stats-header-actions")],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            disabled: true,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("テスト送信")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            disabled: true,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("無効化")],
                    ),
                ],
            ),
        ],
    );

    let stats = div(
        vec![("class", "blocks-settings-webhook-stats-grid")],
        STATS.iter().map(stat_card).collect(),
    );

    let trend_card = card::root(
        CardProps::default(),
        vec![],
        vec![
            card::header(vec![], vec![card::title(vec![], vec![text("配信の推移")])]),
            card::body(
                vec![("class", "blocks-settings-webhook-stats-trend")],
                TRENDS.iter().map(trend_bar).collect(),
            ),
        ],
    );

    let secret_card = card::root(
        CardProps::default(),
        vec![],
        vec![
            card::header(
                vec![],
                vec![card::title(vec![], vec![text("署名シークレット")])],
            ),
            card::body(vec![], vec![secret_clipboard()]),
        ],
    );

    let deliveries_card = card::root(
        CardProps::default(),
        vec![],
        vec![
            card::header(
                vec![],
                vec![card::title(vec![], vec![text("直近の配信一覧")])],
            ),
            card::body(vec![], vec![deliveries_table()]),
        ],
    );

    let timeline_card = card::root(
        CardProps::default(),
        vec![],
        vec![
            card::header(
                vec![],
                vec![card::title(vec![], vec![text("イベントタイムライン")])],
            ),
            card::body(vec![], vec![event_timeline()]),
        ],
    );

    div(
        vec![("class", "blocks-settings-webhook-stats-stack")],
        vec![
            header,
            stats,
            trend_card,
            secret_card,
            deliveries_card,
            timeline_card,
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-webhook-stats/",
    title: "settings-webhook-stats",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_webhook_stats.rs",
    demo_class: "blocks-settings-webhook-stats",
    parts: &[
        Part {
            label: "Stat",
            path: "/themes/stat/",
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
            label: "Timeline",
            path: "/themes/timeline/",
        },
        Part {
            label: "Progress",
            path: "/themes/progress/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Clipboard",
            path: "/themes/clipboard/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_webhook_stats` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-settings-webhook-stats-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  container-type: inline-size;\n  container-name: blocks-settings-webhook-stats;\n}\n\
.blocks-settings-webhook-stats-header {\n  display: flex;\n  align-items: center;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-webhook-stats-header-actions {\n  display: flex;\n  gap: var(--fandhe-space-2);\n  margin-left: auto;\n}\n\
.blocks-settings-webhook-stats-grid {\n  display: grid;\n  grid-template-columns: repeat(4, 1fr);\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-webhook-stats-stat-label {\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-normal);\n}\n\
.blocks-settings-webhook-stats-trend {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-settings-webhook-stats-trend-row {\n  display: grid;\n  grid-template-columns: 4rem 1fr 3rem;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-webhook-stats-trend-date {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-settings-webhook-stats-trend-percent {\n  font-size: var(--fandhe-font-font-size-sm);\n  text-align: right;\n}\n\
@container blocks-settings-webhook-stats (max-width: 48rem) {\n  \
.blocks-settings-webhook-stats-grid {\n    grid-template-columns: repeat(2, 1fr);\n  }\n\
}\n\
@container blocks-settings-webhook-stats (max-width: 28rem) {\n  \
.blocks-settings-webhook-stats-grid {\n    grid-template-columns: 1fr;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, TRENDS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"stat\"",
            "data-scope=\"card\"",
            "data-scope=\"table\"",
            "data-scope=\"timeline\"",
            "data-scope=\"progress\"",
            "data-scope=\"button\"",
            "data-scope=\"clipboard\"",
            "data-scope=\"badge\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn demo_has_four_stat_cards_and_one_table() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-settings-webhook-stats-stat-card=\"\"")
                .count(),
            4
        );
        assert_eq!(html.matches("<table").count(), 1);
        assert_eq!(
            html.matches("data-scope=\"timeline\" data-part=\"root\"")
                .count(),
            1
        );
        assert_eq!(
            html.matches("data-scope=\"progress\" data-part=\"range\"")
                .count(),
            TRENDS.len()
        );
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"submit\""));
    }

    #[test]
    fn layout_css_is_safe_and_responsive() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-settings-webhook-stats (max-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("@container blocks-settings-webhook-stats (max-width: 28rem)"));
    }

    #[test]
    fn failed_deliveries_are_highlighted() {
        let html = demo_html();
        assert!(html.contains("fd-badge--color-palette-danger"));
        assert!(html.contains("fd-badge--color-palette-warning"));
    }

    #[test]
    fn secret_is_masked() {
        let html = demo_html();
        assert!(html.contains("whsec_"));
        assert!(html.contains("\u{2022}\u{2022}\u{2022}\u{2022}"));
        assert!(!html.contains("whsec_full_secret"));
    }
}
