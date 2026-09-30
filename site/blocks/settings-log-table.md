# settings-log-table

絞り込み UI・ログテーブル・ページ送りを縦に積んだ設定ログブロックです。
`native-select` / `tabs` / `table` / `badge` / `avatar` / `button` /
`pagination` の 7 部品を合成します。Blocks は既存部品の合成例であり、新しい
UI 部品は追加しません。

主参照は対応表 ID R0236（代表構成）・R0377（タブによる結果絞り込み）・
R0379（選択欄 3 種 + ページ送り）です。操作者名・日時・イベント文言・件数は
すべて架空のデータであり、実在の人物・企業・メールアドレス・IP・トークン等
の PII/クレデンシャルは含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。ページ送りは
`ItemMode::Button` のみを使い、`href="#"` の死リンクは出しません。

## Rust コード

```rust
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
```

## 原案差分メモ

- **版 A（監査ログ、R0236 + R0379）**: 期間・種類・結果の `native-select`
  3 種と「絞り込みを適用」「条件をクリア」ボタンを並べたツールバーの下に
  ログテーブルとページ送りを置きます。無 JS のため各選択欄は `option` の
  1 つに `selected` を固定しています。
- **版 B（配信ログ、R0377）**: `tabs`（すべて/成功/失敗/保留）で結果を
  絞り込む版です。各タブの trigger には全 18 件に対する該当総件数
  （`delivery_total`: すべて 18・成功 9・失敗 6・保留 3）のバッジを添え、
  content はページ送り 1 ページ目（「全 18 件中 1〜6 件を表示」）に
  含まれる該当行のみを静的に表示します（実行時の絞り込みではなく初期状態
  固定の表示）。バッジはページ内件数ではなく総件数のため、表示行数とは
  一致しません。
- 結果バッジは成功 = Success・失敗 = Danger・保留 = Warning の
  `colorPalette` で示します。
- 操作者セルはアバター（Xs）+ 氏名の横並びです。
- ページ送りは両版とも常に先頭ページを表示中の静的表示で、
  `ItemMode::Button` のみを使い `href="#"` の死リンクは出しません。
- Demo 枠の幅はビューポート幅と一致しないため、絞り込みツールバーの折り
  返しは `@container`（コンテナ幅 36rem 未満）で判定し、選択欄を全幅に
  切り替えます。

関連情報: [Native Select](../themes/native-select.md) / [Tabs](../themes/tabs.md) /
[Table](../themes/table.md) / [Badge](../themes/badge.md) /
[Avatar](../themes/avatar.md) / [Button](../themes/button.md) /
[Pagination](../themes/pagination.md)
