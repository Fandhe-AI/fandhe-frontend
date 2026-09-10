# dashboard-01

`fandhe-frontend-pre-styled-ui` の `sidebar` / `card` + `stat` + `badge` /
`area_chart` / `toggle_group` / `tabs` / `table` + `checkbox` / `avatar` +
`menu` / `separator` / `link` / `button` を合成した、shadcn/ui Blocks の
`dashboard-01` に相当する管理画面ダッシュボードの合成例です。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集
であることに注意してください。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、値の送信・検証・認証処理・
データ取得を一切行いません。期間切替（toggle-group）・タブ切替・行選択
（checkbox）はいずれも初期状態を固定して掲示します（無 JS 制約、
`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポーネント層
はアプリケーションロジックを内包しません）。ブランド名・ユーザー名・メール
アドレス・統計値はすべて架空のものであり、実企業名・実在人物・実クレデンシャル・
PII を含みません。

shadcn/ui 側の `dashboard-01` は TanStack Table 相当の data-table（列表示切替・
ページネーション・行ドラッグ）を使いますが、本リポジトリの `data-table` 系部品は
本イシュー時点で未実装（#2124〜#2127）のため、既存の `table`（構造）+
`checkbox`（選択列）の合成で静的な行選択の見た目のみを再現しています。

## Rust コード

```rust
use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::area_chart::{self, AreaChartProps, AreaCurve, AreaFill};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::link;
use fandhe_frontend_pre_styled_ui::link::LinkProps;
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps, SeparatorVariant};
use fandhe_frontend_pre_styled_ui::sidebar;
use fandhe_frontend_pre_styled_ui::sidebar::{
    Sidebar, SidebarMenuButtonProps, SidebarProps, SidebarState, SidebarVariant,
};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::toggle_group::{self, ToggleGroupVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 週次カテゴリラベル（Apr 3 〜 Jun 30 の 13 週、shadcn 側の日次 90 点を
/// 週次へ縮約する。x 軸ラベルの重なり回避が理由、モジュール doc参照）。
fn visitors_categories() -> Vec<String> {
    [
        "Apr 3", "Apr 10", "Apr 17", "Apr 24", "May 1", "May 8", "May 15", "May 22", "May 29",
        "Jun 5", "Jun 12", "Jun 19", "Jun 26",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

/// 1 枚の統計カード（Total Revenue 等）を組み立てる。
fn stat_card(
    label: &'static str,
    value: &'static str,
    trend_up: bool,
    trend_label: &'static str,
    help: &'static str,
) -> Node {
    let trend_badge = badge::badge(
        &BadgeProps {
            variant: BadgeVariant::Outline,
            ..BadgeProps::default()
        },
        vec![],
        vec![
            if trend_up {
                stat::up_indicator(vec![])
            } else {
                stat::down_indicator(vec![])
            },
            text(trend_label),
        ],
    );

    card::root(
        CardProps::default(),
        vec![("data-blocks-dashboard-01-card", "")],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    card::title(
                        vec![("class", "blocks-dashboard-01-stat-label")],
                        vec![text(label)],
                    ),
                    card::action(vec![], vec![trend_badge]),
                ],
            ),
            card::body(
                vec![],
                vec![stat::root(
                    Size::Lg,
                    vec![],
                    vec![stat::value_text(vec![], vec![text(value)])],
                )],
            ),
            card::footer(
                vec![],
                vec![stat::help_text(
                    vec![],
                    vec![
                        if trend_up {
                            stat::up_indicator(vec![])
                        } else {
                            stat::down_indicator(vec![])
                        },
                        text(help),
                    ],
                )],
            ),
        ],
    )
}

/// footer のユーザー行（avatar + 閉じた menu）。
fn user_menu() -> Node {
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some("blocks-dashboard-01-user-menu-content"),
        vec![
            ("aria-label", "Open user menu"),
            ("data-blocks-dashboard-01-user-trigger", ""),
        ],
        vec![text("\u{2026}")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some("blocks-dashboard-01-user-menu-content"),
        None,
        vec![],
        vec![
            menu::item("account", false, false, vec![], vec![text("Account")]),
            menu::item("billing", false, false, vec![], vec![text("Billing")]),
            menu::separator(vec![], vec![]),
            menu::item("logout", false, false, vec![], vec![text("Log out")]),
        ],
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    let root = menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    );

    div(
        vec![("class", "blocks-dashboard-01-user")],
        vec![
            avatar::root(
                &AvatarProps::default(),
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text("AL")],
                )],
            ),
            div(
                vec![("class", "blocks-dashboard-01-user-text")],
                vec![
                    span(vec![], vec![text("Ada Lovelace")]),
                    span(vec![], vec![text("ada@example.com")]),
                ],
            ),
            root,
        ],
    )
}

/// 1 件のナビメニュー項目（アイコンなし、テキストのみ）。
fn nav_item(label: &'static str, active: bool) -> Node {
    sidebar::menu_item(
        vec![],
        vec![sidebar::menu_button(
            &SidebarMenuButtonProps {
                href: None,
                active,
                ..Default::default()
            },
            None,
            vec![],
            vec![text(label)],
        )],
    )
}

/// 左サイドバーの `root`（`provider > root` の子結合子 CSS に合わせて、
/// 呼び出し元 [`demo`] が本関数の戻り値を `provider` の直接の子として置く。
/// `rail` は `root` の直接の子として置く）。
fn sidebar_root(state: &Sidebar, props: &SidebarProps) -> Node {
    let nav_group = sidebar::group(
        None,
        vec![],
        vec![sidebar::group_content(
            vec![],
            vec![sidebar::menu(
                vec![],
                vec![
                    nav_item("Dashboard", true),
                    nav_item("Lifecycle", false),
                    nav_item("Analytics", false),
                    nav_item("Projects", false),
                    nav_item("Team", false),
                ],
            )],
        )],
    );

    let documents_group = sidebar::group(
        Some("blocks-dashboard-01-documents-label"),
        vec![],
        vec![
            sidebar::group_label(
                Some("blocks-dashboard-01-documents-label"),
                vec![],
                vec![text("Documents")],
            ),
            sidebar::group_content(
                vec![],
                vec![sidebar::menu(
                    vec![],
                    vec![
                        nav_item("Data Library", false),
                        nav_item("Reports", false),
                        nav_item("Word Assistant", false),
                        nav_item("More", false),
                    ],
                )],
            ),
        ],
    );

    let secondary_group = sidebar::group(
        None,
        vec![],
        vec![sidebar::group_content(
            vec![],
            vec![sidebar::menu(
                vec![],
                vec![
                    nav_item("Settings", false),
                    nav_item("Get Help", false),
                    nav_item("Search", false),
                ],
            )],
        )],
    );

    let header = sidebar::header(
        vec![],
        vec![
            div(
                vec![("class", "blocks-dashboard-01-brand")],
                vec![text("Acme Inc.")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("Quick Create")],
            ),
        ],
    );

    sidebar::root(
        state,
        props,
        "Main navigation",
        Some("blocks-dashboard-01-sidebar"),
        vec![],
        vec![
            header,
            sidebar::content(vec![], vec![nav_group, documents_group]),
            sidebar::footer(vec![], vec![secondary_group, user_menu()]),
            sidebar::rail(state, "Toggle sidebar rail", vec![], vec![]),
        ],
    )
}

/// 4 枚の統計カードを 2x2 grid で並べる。
fn stat_cards() -> Node {
    div(
        vec![("class", "blocks-dashboard-01-stats")],
        vec![
            stat_card(
                "Total Revenue",
                "$1,250.00",
                true,
                "+12.5%",
                "Trending up this month",
            ),
            stat_card(
                "New Customers",
                "1,234",
                false,
                "-20%",
                "Down 20% this period",
            ),
            stat_card(
                "Active Accounts",
                "45,678",
                true,
                "+12.5%",
                "Strong user retention",
            ),
            stat_card(
                "Growth Rate",
                "4.5%",
                true,
                "+4.5%",
                "Steady performance increase",
            ),
        ],
    )
}

/// Total Visitors カード（期間切替 toggle-group + グラデーション面グラフ）。
fn visitors_chart_card() -> Node {
    let toggle = toggle_group::root(
        Size::Sm,
        ToggleGroupVariant::Outline,
        ColorPalette::Accent,
        false,
        None,
        None,
        vec![("aria-label", "Select date range")],
        vec![
            toggle_group::item(
                &Default::default(),
                true,
                false,
                false,
                "90d",
                vec![],
                vec![text("Last 3 months")],
            ),
            toggle_group::item(
                &Default::default(),
                false,
                false,
                false,
                "30d",
                vec![],
                vec![text("Last 30 days")],
            ),
            toggle_group::item(
                &Default::default(),
                false,
                false,
                false,
                "7d",
                vec![],
                vec![text("Last 7 days")],
            ),
        ],
    );

    let data = ChartData::new(
        visitors_categories(),
        vec![
            Series::new(
                "desktop",
                vec![
                    120.0, 180.0, 150.0, 220.0, 190.0, 250.0, 210.0, 280.0, 260.0, 310.0, 290.0,
                    340.0, 320.0,
                ],
            ),
            Series::new(
                "mobile",
                vec![
                    60.0, 90.0, 80.0, 110.0, 100.0, 130.0, 120.0, 150.0, 140.0, 170.0, 160.0,
                    190.0, 180.0,
                ],
            ),
        ],
    )
    .expect("dashboard-01 固定データは常に有効な ChartData を構築できる");

    let chart = area_chart::area_chart(
        &AreaChartProps {
            curve: AreaCurve::Natural,
            fill: AreaFill::Gradient,
            gradient_id: "blocks-dashboard-01-visitors",
            show_x_axis: true,
            show_y_axis: true,
            show_grid: true,
            width: 900.0,
            height: 250.0,
            range: Some("90d"),
            ..AreaChartProps::new(&data, "Total visitors for the last 3 months")
        },
        vec![("data-blocks-dashboard-01-chart", "")],
    )
    .expect("dashboard-01 固定データは常に有効な area_chart を構築できる");

    card::root(
        CardProps::default(),
        vec![],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    card::title(vec![], vec![text("Total Visitors")]),
                    card::description(vec![], vec![text("Total for the last 3 months")]),
                    card::action(vec![], vec![toggle]),
                ],
            ),
            card::body(vec![], vec![chart]),
        ],
    )
}

/// 選択列付きの data table 行（`table` + `checkbox` の合成、
/// `data-table` 系部品〔#2124〜#2127〕未実装のための代替）。
fn row_select_checkbox(name: &'static str, checked: bool, label: &'static str) -> Node {
    let props = CheckboxProps {
        checked: if checked {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        },
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

/// [`outline_table`] のデータ行 1 件分（`clippy::too_many_arguments` 回避の
/// ための引数集約構造体、docs-site の他ショーケースコードには前例が無い
/// ためこの block 内限定のローカル定義とする）。
struct DataRow {
    select_name: &'static str,
    header_label: &'static str,
    section_type: &'static str,
    status: &'static str,
    status_palette: ColorPalette,
    target: &'static str,
    limit: &'static str,
    reviewer: &'static str,
}

fn outline_table() -> Node {
    fn data_row(row: DataRow) -> Node {
        table::row(
            vec![],
            vec![
                table::cell(
                    vec![],
                    vec![row_select_checkbox(row.select_name, false, "Select row")],
                ),
                table::cell(vec![], vec![text(row.header_label)]),
                table::cell(vec![], vec![text(row.section_type)]),
                table::cell(
                    vec![],
                    vec![badge::badge(
                        &BadgeProps {
                            variant: BadgeVariant::Subtle,
                            palette: row.status_palette,
                            ..BadgeProps::default()
                        },
                        vec![],
                        vec![text(row.status)],
                    )],
                ),
                table::cell(vec![("data-align", "end")], vec![text(row.target)]),
                table::cell(vec![("data-align", "end")], vec![text(row.limit)]),
                table::cell(vec![], vec![text(row.reviewer)]),
            ],
        )
    }

    table::root(
        TableProps {
            interactive: true,
            ..TableProps::default()
        },
        vec![("data-blocks-dashboard-01-table", "")],
        vec![
            table::header(
                vec![],
                vec![table::row(
                    vec![],
                    vec![
                        table::column_header(
                            vec![],
                            vec![row_select_checkbox("select-all", false, "Select all rows")],
                        ),
                        table::column_header(
                            vec![("aria-sort", "ascending")],
                            vec![
                                text("Header "),
                                button::button(
                                    &ButtonProps {
                                        variant: ButtonVariant::Plain,
                                        size: Size::Sm,
                                        ..ButtonProps::default()
                                    },
                                    vec![("aria-label", "Sort by header")],
                                    vec![text("\u{25b2}")],
                                ),
                            ],
                        ),
                        table::column_header(vec![], vec![text("Section Type")]),
                        table::column_header(vec![], vec![text("Status")]),
                        table::column_header(vec![("data-align", "end")], vec![text("Target")]),
                        table::column_header(vec![("data-align", "end")], vec![text("Limit")]),
                        table::column_header(vec![], vec![text("Reviewer")]),
                    ],
                )],
            ),
            table::body(
                vec![],
                vec![
                    table::row(
                        vec![("data-selected", "")],
                        vec![
                            table::cell(
                                vec![],
                                vec![row_select_checkbox("select-cover-page", true, "Select row")],
                            ),
                            table::cell(vec![], vec![text("Cover page")]),
                            table::cell(vec![], vec![text("Cover page")]),
                            table::cell(
                                vec![],
                                vec![badge::badge(
                                    &BadgeProps {
                                        variant: BadgeVariant::Subtle,
                                        palette: ColorPalette::Success,
                                        ..BadgeProps::default()
                                    },
                                    vec![],
                                    vec![text("Done")],
                                )],
                            ),
                            table::cell(vec![("data-align", "end")], vec![text("18")]),
                            table::cell(vec![("data-align", "end")], vec![text("5")]),
                            table::cell(vec![], vec![text("Eddie Lake")]),
                        ],
                    ),
                    data_row(DataRow {
                        select_name: "select-table-of-contents",
                        header_label: "Table of contents",
                        section_type: "Table of contents",
                        status: "Done",
                        status_palette: ColorPalette::Success,
                        target: "29",
                        limit: "24",
                        reviewer: "Eddie Lake",
                    }),
                    data_row(DataRow {
                        select_name: "select-executive-summary",
                        header_label: "Executive summary",
                        section_type: "Narrative",
                        status: "In Process",
                        status_palette: ColorPalette::Warning,
                        target: "10",
                        limit: "13",
                        reviewer: "Jamik Tashpulatov",
                    }),
                    data_row(DataRow {
                        select_name: "select-technical-approach",
                        header_label: "Technical approach",
                        section_type: "Narrative",
                        status: "Done",
                        status_palette: ColorPalette::Success,
                        target: "27",
                        limit: "23",
                        reviewer: "Jamik Tashpulatov",
                    }),
                    data_row(DataRow {
                        select_name: "select-design",
                        header_label: "Design",
                        section_type: "Narrative",
                        status: "In Process",
                        status_palette: ColorPalette::Warning,
                        target: "2",
                        limit: "16",
                        reviewer: "Jamik Tashpulatov",
                    }),
                ],
            ),
        ],
    )
}

fn short_note(text_content: &'static str) -> Node {
    div(vec![], vec![text(text_content)])
}

/// tabs + data table カード。
fn tabs_and_table_card() -> Node {
    let props = TabsProps {
        id: "blocks-dashboard-01-tabs",
        selected: "outline",
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };

    let items = vec![
        TabItem {
            value: "outline",
            trigger: vec![text("Outline")],
            content: vec![outline_table()],
            disabled: false,
        },
        TabItem {
            value: "past-performance",
            trigger: vec![
                text("Past Performance "),
                badge::badge(&BadgeProps::default(), vec![], vec![text("3")]),
            ],
            content: vec![short_note(
                "四半期ごとの実績サマリ（静的な合成例のためデータ取得は行いません）。",
            )],
            disabled: false,
        },
        TabItem {
            value: "key-personnel",
            trigger: vec![
                text("Key Personnel "),
                badge::badge(&BadgeProps::default(), vec![], vec![text("2")]),
            ],
            content: vec![short_note(
                "担当者一覧（静的な合成例のためデータ取得は行いません）。",
            )],
            disabled: false,
        },
        TabItem {
            value: "focus-documents",
            trigger: vec![text("Focus Documents")],
            content: vec![short_note(
                "重点ドキュメント一覧（静的な合成例のためデータ取得は行いません）。",
            )],
            disabled: false,
        },
    ];

    div(
        vec![],
        vec![tabs::tabs(
            TabsVariant::Enclosed,
            Size::Md,
            ColorPalette::Accent,
            &props,
            items,
        )],
    )
}

/// `dashboard-01` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let state = Sidebar::new(SidebarState::Expanded);
    let props = SidebarProps {
        variant: SidebarVariant::Inset,
        ..SidebarProps::default()
    };

    let inset_header = div(
        vec![("data-blocks-dashboard-01-header", "")],
        vec![
            sidebar::trigger(
                &state,
                "Toggle sidebar",
                Some("blocks-dashboard-01-sidebar"),
                vec![],
                vec![],
            ),
            separator::separator(
                &SeparatorProps {
                    orientation: Orientation::Vertical,
                    variant: SeparatorVariant::Solid,
                },
                vec![],
            ),
            span(vec![], vec![text("Documents")]),
            link::root(
                "https://github.com/Fandhe-AI/fandhe-frontend",
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![("data-blocks-dashboard-01-header-link", "")],
                vec![text("GitHub")],
            ),
        ],
    );

    let inset = sidebar::inset(
        vec![],
        vec![
            inset_header,
            div(
                vec![("class", "blocks-dashboard-01-main")],
                vec![stat_cards(), visitors_chart_card(), tabs_and_table_card()],
            ),
        ],
    );

    // `variant: Inset` の面パネル化は `provider[data-variant="inset"] >
    // inset` の子結合子 CSS で効くため（`crates/pre-styled-ui/src/
    // sidebar.rs` の recipe 参照）、`root`/`inset` は同じ `provider` の
    // 直接の子として並べる（モジュール doc「CSS フックの選び方」節参照）。
    sidebar::provider(
        &state,
        &props,
        vec![],
        vec![sidebar_root(&state, &props), inset],
    )
}
```

## shadcn 側との差分メモ

- 期間切替（toggle-group）・タブ切替・行選択（checkbox）は静的表示のみで、
  実際の並べ替え・フィルタ・送信は行いません。
- data table のドラッグハンドル（行並べ替え）・列表示切替・ページネーション・
  行ごとのアクションメニューは未合成です（`data-table` 系部品〔#2124〜#2127〕が
  未実装のため）。実装され次第、本 block を `table` + `checkbox` の代替合成から
  data-table ベースへ差し替える対象として記録します。
- チャートは shadcn 側の日次 90 点ではなく週次 13 点（Apr 3 〜 Jun 30）で構成して
  います（x 軸ラベルの重なり回避のため）。モバイル用の 7 日・30 日表示は静的な
  toggle-group ラベルのみで、実際の期間切替に応じたデータ再取得は行いません。
- sidebar の Quick Create はドロップダウンメニューではなく単一の
  `button::button`（Outline variant）で代替しています。キーボードショート
  カット（Cmd/Ctrl+B によるサイドバー開閉）は wasm-full 側の実行時責務であり、
  本 SSR 合成例の対象外です。
- 統計カードのトレンドバッジは shadcn 側のアイコン付きバッジを
  `badge::badge`（Outline variant）+ `stat::up_indicator`/`down_indicator`
  で代替しています。

関連情報: [Sidebar](../themes/sidebar.md) / [Card](../themes/card.md) /
[Stat](../themes/stat.md) / [Badge](../themes/badge.md) /
[Area Chart](../themes/area-chart.md) / [Toggle Group](../themes/toggle-group.md) /
[Tabs](../themes/tabs.md) / [Table](../themes/table.md) /
[Checkbox](../themes/checkbox.md) / [Avatar](../themes/avatar.md) /
[Menu](../themes/menu.md) / [Separator](../themes/separator.md) /
[Link](../themes/link.md) / [Button](../themes/button.md)
