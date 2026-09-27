//! `pricing-tiers-comparison` block（親トラッキング #2872「Blocks に
//! pricing-tiers-comparison（カードと表の 2 段構成）を追加する」配下）。
//! 前半 #2873（骨格・主要領域）に続き、本イシュー #2874 でカードと表の
//! 間の顧客ロゴ列（対応表 ID R1142）・暗色帯 + 周期切替（`tabs`）
//! バリエーション（集約元 R1151）・差分メモの最終版を仕上げた。
//!
//! # 使用部品
//!
//! `heading`（見出し）+ `text`（リード文）+ `card`（プランカード）+
//! `badge`（推奨プランのタグ）+ `button`（CTA）+ `table`（幅広の比較表）+
//! `icon`（可否表示・ロゴ列の装飾図形）+ `list`（狭幅のプランごとの機能
//! 一覧・顧客ロゴ列）+ `tabs`（周期切替）の 9 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 骨格: カードと表の 2 段構成、狭幅では表の代わりに一覧
//!
//! 上段にプランカード 3 枚（幅広・狭幅とも常時表示）、下段に同じプランの
//! 詳細比較表を置く。比較表は幅広（`>= 48rem`）でのみ表示し、狭幅では
//! 代わりに「プランごとの機能一覧」（`list`+`icon`）を縦に並べる
//! （`pricing_comparison_table` の表/狭幅切替と同型の判断）。カードと表の
//! 間には顧客ロゴ列（[`logo_row`]）を挟む。
//!
//! # 顧客ロゴ列（装飾アイコン + 可視社名）
//!
//! ロゴは自作の抽象図形（`icon`、`label: None` のため `aria-hidden="true"`
//! の装飾）とし、[`crate::blocks::dummy_assets::COMPANY_NAMES`] の社名
//! （先頭 5 件）を可視テキストとして必ず併記する（`logo_cloud_grid` の
//! Codex レビュー指摘「装飾ロゴだけでは社名がスクリーンリーダーに伝わら
//! ない」〔PR #3244〕の教訓を踏襲）。`image` 部品は使わず、既存の使用部品
//! （`list`+`icon`）だけで組むことでこの欠落を構造的に防ぐ。
//!
//! # 暗色帯と周期切替の 2 状態並記
//!
//! 見出し・リード文を暗色帯（`background: var(--fandhe-color-fg)`、
//! `pricing_tier_cards` の `inverted` と同型）に置き、その下へ月額/年額の
//! `tabs`（`selected` が異なる 2 インスタンス）を並べる。docs サイトは
//! JS ハイドレーションを行わないため、`pricing_tiers_morph` と同じ「2
//! インスタンス併記」で両方の billing 状態を可視化する。暗色帯の見出し・
//! リード文は 1 回だけ出し、その下に月額・年額 2 つの `tabs` を置くことで
//! 同一文言の見出しが 2 つ並ぶことを避ける。
//!
//! # 暗色帯で比較表を繰り返さない理由
//!
//! 暗色帯配下のプランカードは既存の比較表・狭幅一覧と同じ機能情報を
//! 重複して運ばない（表を複製すると `scope="row"` の件数契約・aria-label
//! region の重複を招くうえ Demo が肥大化する）。周期による価格差は
//! [`plan_card`] の価格・期間表記のみで示し、機能比較は上段の比較表
//! （月額固定表示）を正とする。
//!
//! # 機能一覧の可否アイコンが意味を持つ理由（`pricing_slider_tiers` との違い）
//!
//! 本 block の一覧は「比較表と同じ可否情報」を狭幅で示すものであり、
//! 各項目は「含まれる/含まれない/値」の情報を運ぶ。そのため可否アイコンは
//! [`fandhe_frontend_pre_styled_ui::list::indicator`]（常に
//! `aria-hidden="true"`）には収めず、`label` 付きの意味を持つアイコンとして
//! item 本体へ直接置く（`pricing_comparison_table` の
//! `feature_value_icon`/`value_node` と同じ判断）。
//!
//! # 行見出しに `row_header` を使う理由（`<th scope="row">`）
//!
//! 機能名は [`fandhe_frontend_pre_styled_ui::table::row_header`] へ置く。
//! 値セル（`<td>`）と同じセルに置くと、スクリーンリーダー利用者が値セル間を
//! 移動した際に列見出し（プラン名）しか読み上げられず「どの機能の可否か」が
//! 判別できない（`pricing_comparison_table` と同じ codex-review P1 是正の
//! 踏襲）。
//!
//! # カテゴリ見出し行の組み方
//!
//! 各カテゴリの先頭行は `table::row_header` へ `colspan`（[`COL_COUNT_STR`]、
//! プラン数 + 1 の文字列表記）と
//! `data-blocks-pricing-tiers-comparison-category` を渡した 1 セル行にする。
//! `row_header` が予約キーとして除去するのは `scope` のみのため `colspan`
//! はそのまま出力される。カテゴリごとに独立した `table::body`（tbody）を
//! 出すことで、視覚的な区切りと DOM 構造の区切りを一致させる
//! （`pricing_comparison_table` と同型）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、header 領域の見出しは
//! [`fandhe_frontend_pre_styled_ui::heading::HeadingLevel::H3`] にする。
//! カード内・狭幅一覧のプラン名見出しは見出しレベルを飛ばさないよう `H4`
//! にする。
//!
//! # `drop_class_attr` の契約（CSS フックの選び方）
//!
//! `card::root`/`button`/`badge`/`heading`/`text`/`table::root`/
//! `list::root`/`icon` は `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、Demo 固有スタイルは
//! `data-blocks-pricing-tiers-comparison-*` 属性で渡す（[`LAYOUT_CSS`] 側も
//! 同じ属性セレクタで対応する）。一方、素の `div`、`card::header`/`body`/
//! `footer`、`table::row`/`cell`/`scroll_area`、`list::item` には `class` が
//! そのまま効くため、それらは `.blocks-pricing-tiers-comparison-*` クラスを
//! 使う（`crate::blocks` モジュール doc「CSS フックが `class` と `[data-*]`
//! で混在する理由」節参照）。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。CTA ボタンは既定 `type="button"` のまま送信先を持たない。
//! プラン名・価格・機能・文言はすべて架空のものであり、実企業名・実クレデ
//! ンシャル・PII を含まない。プラン名・価格は
//! `crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS`（Starter $9 / Growth $29
//! / Scale $79）と同じ値を採用する。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 表の列数（機能名列 + プラン数）を文字列表記した定数。プラン数が
/// [`PLANS`] 固定のため `colspan` 属性値としてリテラルで持つ（ファイル内
/// ユニットテストで `PLANS.len() + 1` との一致を固定）。
const COL_COUNT_STR: &str = "4";

/// 機能可否・値の表現。
enum FeatureValue {
    /// 含まれる（チェックアイコン）。
    Included,
    /// 含まれない（バツアイコン）。
    Excluded,
    /// 文字表示（例: 上限件数）。
    Text(&'static str),
}

/// 料金プラン 1 件分の静的データ（架空）。`price_yearly` は暗色帯の
/// 年額 tabs パネル（[`billing_grid`]）専用で、月額比較表・狭幅一覧では
/// 使わない。
struct Plan {
    name: &'static str,
    price: &'static str,
    price_yearly: &'static str,
    description: &'static str,
    cta: &'static str,
    recommended: bool,
}

/// 機能比較 1 行分。`values` の長さは [`PLANS`] の件数と一致させる
/// （ファイル内ユニットテストで固定する不変条件）。
struct FeatureRow {
    label: &'static str,
    values: &'static [FeatureValue],
}

/// カテゴリ見出し行 1 件分（機能行のグルーピング単位、`table::body` 1 個に
/// 対応する）。
struct Category {
    label: &'static str,
    rows: &'static [FeatureRow],
}

/// プラン 3 件（中央 Growth を推奨プランとする）。価格は
/// `crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS` と同じ値を直書きする。
const PLANS: [Plan; 3] = [
    Plan {
        name: "Starter",
        price: "$9",
        price_yearly: "$86",
        description: "個人・小規模プロジェクト向け",
        cta: "Starter を選ぶ",
        recommended: false,
    },
    Plan {
        name: "Growth",
        price: "$29",
        price_yearly: "$278",
        description: "成長中のチーム向け",
        cta: "Growth を選ぶ",
        recommended: true,
    },
    Plan {
        name: "Scale",
        price: "$79",
        price_yearly: "$758",
        description: "大規模組織向け",
        cta: "Scale を選ぶ",
        recommended: false,
    },
];

/// カテゴリ 2 件・機能行合計 5 件（架空データ）。
const CATEGORIES: [Category; 2] = [
    Category {
        label: "基本機能",
        rows: &[
            FeatureRow {
                label: "プロジェクト数",
                values: &[
                    FeatureValue::Text("3 件"),
                    FeatureValue::Text("無制限"),
                    FeatureValue::Text("無制限"),
                ],
            },
            FeatureRow {
                label: "ストレージ容量",
                values: &[
                    FeatureValue::Text("5GB"),
                    FeatureValue::Text("100GB"),
                    FeatureValue::Text("1TB"),
                ],
            },
            FeatureRow {
                label: "カスタムドメイン",
                values: &[
                    FeatureValue::Excluded,
                    FeatureValue::Included,
                    FeatureValue::Included,
                ],
            },
        ],
    },
    Category {
        label: "サポート",
        rows: &[
            FeatureRow {
                label: "優先サポート窓口",
                values: &[
                    FeatureValue::Excluded,
                    FeatureValue::Included,
                    FeatureValue::Included,
                ],
            },
            FeatureRow {
                label: "専任担当者",
                values: &[
                    FeatureValue::Excluded,
                    FeatureValue::Excluded,
                    FeatureValue::Included,
                ],
            },
        ],
    },
];

/// 可否を表す自作の抽象チェック/バツ図形。意味を持つアイコンのため
/// `label` を指定する（参照元のアイコン形状・内部識別子は持ち込まない）。
fn value_node(value: &FeatureValue) -> Node {
    match value {
        FeatureValue::Included => icon(
            &IconProps {
                size: Size::Sm,
                label: Some("含まれる"),
                ..IconProps::default()
            },
            vec![("data-blocks-pricing-tiers-comparison-value", "included")],
            vec![el(
                "path",
                vec![
                    ("d", "M5 12.5l4.5 4.5L19 7"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "1.5"),
                    ("stroke-linecap", "round"),
                    ("stroke-linejoin", "round"),
                ],
                vec![],
            )],
        ),
        FeatureValue::Excluded => icon(
            &IconProps {
                size: Size::Sm,
                label: Some("含まれない"),
                ..IconProps::default()
            },
            vec![("data-blocks-pricing-tiers-comparison-value", "excluded")],
            vec![
                el(
                    "path",
                    vec![
                        ("d", "M7 7l10 10"),
                        ("stroke", "currentColor"),
                        ("stroke-width", "1.5"),
                        ("stroke-linecap", "round"),
                    ],
                    vec![],
                ),
                el(
                    "path",
                    vec![
                        ("d", "M17 7l-10 10"),
                        ("stroke", "currentColor"),
                        ("stroke-width", "1.5"),
                        ("stroke-linecap", "round"),
                    ],
                    vec![],
                ),
            ],
        ),
        FeatureValue::Text(label) => span(
            vec![("data-blocks-pricing-tiers-comparison-value", "text")],
            vec![text(*label)],
        ),
    }
}

/// header 領域（見出し・リード文）。
fn header() -> Node {
    div(
        vec![("data-blocks-pricing-tiers-comparison-header", "")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("あなたに合うプランを選ぶ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "3 つのプランを機能ごとに比較して、必要な範囲を見極められます。",
                )],
            ),
        ],
    )
}

/// プランカード 1 件。推奨プラン（Growth）には badge と強調ボーダーを
/// 付ける。`price`/`period` は呼び出し側の billing 状態（月額/年額）に
/// 応じて切り替える（[`billing_grid`] 参照）。
fn plan_card(plan: &Plan, price: &'static str, period: &'static str) -> Node {
    let mut heading_children = vec![heading::heading(
        HeadingLevel::H4,
        &HeadingProps::default(),
        vec![],
        vec![text(plan.name)],
    )];
    if plan.recommended {
        heading_children.push(badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Solid,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("おすすめ")],
        ));
    }

    let mut attrs = vec![("data-blocks-pricing-tiers-comparison-card", "")];
    if plan.recommended {
        attrs.push(("data-blocks-pricing-tiers-comparison-recommended", ""));
    }

    card::root(
        CardProps {
            variant: CardVariant::Outline,
            size: Size::Md,
        },
        attrs,
        vec![
            card::header(
                vec![],
                vec![
                    div(
                        vec![("class", "blocks-pricing-tiers-comparison-tier-heading")],
                        heading_children,
                    ),
                    card::description(vec![], vec![text(plan.description)]),
                ],
            ),
            card::body(
                vec![],
                vec![div(
                    vec![("class", "blocks-pricing-tiers-comparison-price")],
                    vec![
                        text(price),
                        span(
                            vec![("class", "blocks-pricing-tiers-comparison-price-period")],
                            vec![text(period)],
                        ),
                    ],
                )],
            ),
            card::footer(
                vec![],
                vec![button::button(
                    &ButtonProps {
                        variant: if plan.recommended {
                            ButtonVariant::Solid
                        } else {
                            ButtonVariant::Outline
                        },
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text(plan.cta)],
                )],
            ),
        ],
    )
}

/// 幅広の比較表（`>= 48rem` でのみ表示、[`LAYOUT_CSS`] 参照）。
fn comparison_table() -> Node {
    let mut header_cells = vec![table::column_header(vec![], vec![text("機能")])];
    header_cells.extend(
        PLANS
            .iter()
            .map(|plan| table::column_header(vec![], vec![text(plan.name)])),
    );

    let bodies: Vec<Node> = CATEGORIES
        .iter()
        .map(|category| {
            let mut rows = vec![table::row(
                vec![],
                vec![table::row_header(
                    vec![
                        ("colspan", COL_COUNT_STR),
                        ("data-blocks-pricing-tiers-comparison-category", ""),
                    ],
                    vec![text(category.label)],
                )],
            )];
            rows.extend(category.rows.iter().map(|row| {
                let mut cells = vec![table::row_header(
                    vec![("data-blocks-pricing-tiers-comparison-feature", "")],
                    vec![text(row.label)],
                )];
                cells.extend(
                    row.values
                        .iter()
                        .map(|value| table::cell(vec![], vec![value_node(value)])),
                );
                table::row(vec![], cells)
            }));
            table::body(vec![], rows)
        })
        .collect();

    let mut table_children = vec![
        table::caption(vec![], vec![text("プラン別の機能比較")]),
        table::header(vec![], vec![table::row(vec![], header_cells)]),
    ];
    table_children.extend(bodies);

    let table_node = table::root(
        TableProps {
            variant: TableVariant::Outline,
            ..TableProps::default()
        },
        vec![("data-blocks-pricing-tiers-comparison-table", "")],
        table_children,
    );

    div(
        vec![("data-blocks-pricing-tiers-comparison-view", "table")],
        vec![table::scroll_area(
            vec![
                ("role", "region"),
                ("aria-label", "プラン比較表"),
                ("tabindex", "0"),
                ("data-blocks-pricing-tiers-comparison-scroll", ""),
            ],
            vec![table_node],
        )],
    )
}

/// プラン 1 件分の狭幅一覧（プラン名見出し + 機能一覧）。可否アイコンは
/// 情報を運ぶため `list::indicator`（装飾専用）には収めない
/// （モジュール doc「機能一覧の可否アイコンが意味を持つ理由」節参照）。
fn plan_list(plan_index: usize, plan: &Plan) -> Node {
    let items: Vec<Node> = CATEGORIES
        .iter()
        .flat_map(|category| category.rows.iter())
        .map(|row| {
            let value = &row.values[plan_index];
            let item_children = match value {
                FeatureValue::Text(v) => {
                    vec![span(vec![], vec![text(format!("{}: {v}", row.label))])]
                }
                included_or_excluded => {
                    vec![value_node(included_or_excluded), text(row.label)]
                }
            };
            list::item(
                vec![("class", "blocks-pricing-tiers-comparison-list-item")],
                item_children,
            )
        })
        .collect();

    div(
        vec![("class", "blocks-pricing-tiers-comparison-plan-list")],
        vec![
            heading::heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(plan.name)],
            ),
            list::root(
                ListType::Unordered,
                ListVariant::Plain,
                vec![("data-blocks-pricing-tiers-comparison-features", "")],
                items,
            ),
        ],
    )
}

/// 狭幅（`< 48rem`）で比較表の代わりに表示するプランごとの一覧
/// （[`LAYOUT_CSS`] 参照）。
fn narrow_lists() -> Node {
    div(
        vec![("data-blocks-pricing-tiers-comparison-view", "list")],
        PLANS
            .iter()
            .enumerate()
            .map(|(i, plan)| plan_list(i, plan))
            .collect(),
    )
}

/// 顧客ロゴ列 1 枚分（装飾アイコン + 可視社名）。モジュール doc「顧客
/// ロゴ列（装飾アイコン + 可視社名）」節参照。
fn logo_item(name: &'static str) -> Node {
    list::item(
        vec![("class", "blocks-pricing-tiers-comparison-logo")],
        vec![
            icon(
                &IconProps {
                    size: Size::Sm,
                    label: None,
                    ..IconProps::default()
                },
                vec![],
                vec![el(
                    "path",
                    vec![
                        ("d", "M4 17V7l8-4 8 4v10l-8 4-8-4Z"),
                        ("fill", "none"),
                        ("stroke", "currentColor"),
                        ("stroke-width", "1.5"),
                        ("stroke-linejoin", "round"),
                    ],
                    vec![],
                )],
            ),
            text(name),
        ],
    )
}

/// カードと比較表の間に挟む顧客ロゴ列（対応表 ID R1142）。
/// [`dummy_assets::COMPANY_NAMES`] 先頭 5 件を使う。
fn logo_row() -> Node {
    div(
        vec![("data-blocks-pricing-tiers-comparison-logos-section", "")],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("多くのチームに選ばれています")],
            ),
            list::root(
                ListType::Unordered,
                ListVariant::Plain,
                vec![("data-blocks-pricing-tiers-comparison-logos", "")],
                dummy_assets::COMPANY_NAMES[..5]
                    .iter()
                    .map(|name| logo_item(name))
                    .collect(),
            ),
        ],
    )
}

/// billing 状態 1 件分（月額 or 年額）のプランカードグリッドを組み立てる
/// （モジュール doc「暗色帯で比較表を繰り返さない理由」節参照。機能比較は
/// 上段の比較表・狭幅一覧が正であり、ここでは価格・期間表記のみを
/// 切り替える）。
fn billing_grid(price_of: fn(&Plan) -> &'static str, period: &'static str) -> Node {
    div(
        vec![("data-blocks-pricing-tiers-comparison-grid", "")],
        PLANS
            .iter()
            .map(|plan| plan_card(plan, price_of(plan), period))
            .collect(),
    )
}

/// billing 状態 2 件（月額/年額）分の `TabItem` 一覧。2 インスタンスで
/// 同一の構成を使うため呼び出し側で複製せず都度生成する
/// （`pricing_tiers_morph::tab_items` と同型）。
fn billing_tab_items() -> Vec<TabItem<'static>> {
    vec![
        TabItem {
            value: "monthly",
            trigger: vec![text("月額")],
            content: vec![billing_grid(|plan| plan.price, " / 月")],
            disabled: false,
        },
        TabItem {
            value: "yearly",
            trigger: vec![text("年額")],
            content: vec![billing_grid(|plan| plan.price_yearly, " / 年")],
            disabled: false,
        },
    ]
}

/// キャプション 1 行（`p`、muted）。
fn caption(label: &'static str) -> Node {
    p(
        vec![("data-blocks-pricing-tiers-comparison-caption", "")],
        vec![text(label)],
    )
}

/// 暗色帯 + 周期切替バリエーション（集約元 R1151）。見出し・リード文は
/// 帯内に 1 回だけ出し、その下へ月額選択・年額選択の 2 インスタンスを
/// 縦に並べる（モジュール doc「暗色帯と周期切替の 2 状態並記」節参照）。
fn billing_band() -> Node {
    let band_header = div(
        vec![("data-blocks-pricing-tiers-comparison-band-header", "")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("月額・年額から選べます")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("周期を切り替えて、年間契約の割引を確認できます。")],
            ),
        ],
    );

    let monthly_props = TabsProps {
        id: "blocks-pricing-tiers-comparison-billing-monthly",
        selected: "monthly",
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };
    let yearly_props = TabsProps {
        id: "blocks-pricing-tiers-comparison-billing-yearly",
        selected: "yearly",
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };

    let monthly_tabs = tabs::tabs(
        TabsVariant::Enclosed,
        Size::Md,
        ColorPalette::Accent,
        &monthly_props,
        billing_tab_items(),
    );
    let yearly_tabs = tabs::tabs(
        TabsVariant::Enclosed,
        Size::Md,
        ColorPalette::Accent,
        &yearly_props,
        billing_tab_items(),
    );

    div(
        vec![("data-blocks-pricing-tiers-comparison-band", "")],
        vec![
            band_header,
            caption("月額選択時"),
            monthly_tabs,
            caption("年額選択時"),
            yearly_tabs,
        ],
    )
}

/// `pricing-tiers-comparison` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。形 A（header → プランカード → 顧客ロゴ列 → 幅広比較表 /
/// 狭幅一覧）+ 形 B（暗色帯 + 周期切替の月額/年額 2 状態）の順に並べる
/// （表/一覧の表示切替は [`LAYOUT_CSS`] の `@media` が担う）。
pub fn demo() -> Node {
    let cards = div(
        vec![("data-blocks-pricing-tiers-comparison-grid", "")],
        PLANS
            .iter()
            .map(|plan| plan_card(plan, plan.price, " / 月"))
            .collect(),
    );

    div(
        vec![("class", "blocks-pricing-tiers-comparison-layout")],
        vec![
            caption("ロゴ列を挟む形"),
            header(),
            cards,
            logo_row(),
            comparison_table(),
            narrow_lists(),
            caption("暗色帯 + 周期切替バリエーション"),
            billing_band(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/pricing-tiers-comparison/",
    title: "pricing-tiers-comparison",
    category: BlockCategory::Pricing,
    rust_source: "crates/docs-site/src/blocks/marketing/pricing/pricing_tiers_comparison.rs",
    demo_class: "blocks-pricing-tiers-comparison",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Tabs",
            path: "/themes/tabs/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `pricing_tiers_comparison` 固有のレイアウト規則
/// （`crate::blocks::LAYOUT_CSS` doc「block 固有 CSS の置き場」節）。既定
/// （狭幅）はカード 1 列縦積み・比較表を隠して一覧を表示、`>= 48rem`
/// （[`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`]）でカード
/// 3 列・比較表を表示して一覧を隠す。
///
/// 推奨カードの強調ボーダー
/// （`[data-blocks-pricing-tiers-comparison-recommended]`）は `card` レシピの
/// base（`[data-scope="card"][data-part="root"]`、詳細度 0,2,0）・既定
/// variant（`Outline` の `border-color`、クラスセレクタ併用で 0,3,0）の
/// 両方に勝つ必要があるため、`[data-scope="card"][data-part="root"]` を
/// 前置して詳細度 0,3,0 へ揃えている（`pricing_slider_tiers` と同じ判断）。
///
/// 比較表の横スクロール規則（`min-width: 40rem`・`width: 100%`）は
/// `[data-scope="table"][data-part="root"/"scroll-area"]` へ直接書くと
/// `blocks::stylesheet()` が全 block の `LAYOUT_CSS` を単一の
/// `assets/blocks.css` へ連結する構造上、他 block の狭幅テーブルへも
/// 波及する（`comparison_table` 是正と同じ問題）。`table::root`/
/// `table::scroll_area` へ block 固有属性
/// （`data-blocks-pricing-tiers-comparison-table`/`-scroll`）を直接渡し、
/// この block のみへスコープする（`comparison_table` の
/// `data-blocks-comparison-table-table`/`-scroll` と同じ判断）。
const LAYOUT_CSS: &str = "\
.blocks-pricing-tiers-comparison-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
[data-blocks-pricing-tiers-comparison-header] {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n  max-width: 36rem;\n}\n\
[data-blocks-pricing-tiers-comparison-grid] {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-4);\n  align-items: stretch;\n}\n\
[data-blocks-pricing-tiers-comparison-card] {\n  display: flex;\n  flex-direction: column;\n  height: 100%;\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-pricing-tiers-comparison-recommended] {\n  border-color: var(--fandhe-color-accent);\n  border-width: 2px;\n}\n\
.blocks-pricing-tiers-comparison-tier-heading {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-pricing-tiers-comparison-price {\n  display: flex;\n  align-items: baseline;\n  gap: var(--fandhe-space-1);\n  font-size: var(--fandhe-font-font-size-2xl, 1.5rem);\n  font-weight: var(--fandhe-font-font-weight-bold);\n  margin: var(--fandhe-space-4) 0;\n}\n\
.blocks-pricing-tiers-comparison-price-period {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-scope=\"table\"][data-part=\"scroll-area\"][data-blocks-pricing-tiers-comparison-scroll] {\n  width: 100%;\n}\n\
[data-blocks-pricing-tiers-comparison-table] {\n  min-width: 40rem;\n}\n\
[data-scope=\"table\"][data-part=\"row-header\"][data-blocks-pricing-tiers-comparison-category] {\n  background: var(--fandhe-color-bg-subtle);\n  text-align: left;\n  font-weight: 600;\n}\n\
[data-scope=\"table\"][data-part=\"row-header\"][data-blocks-pricing-tiers-comparison-feature] {\n  text-align: left;\n}\n\
[data-scope=\"list\"][data-part=\"root\"][data-blocks-pricing-tiers-comparison-features] {\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"list\"][data-part=\"item\"].blocks-pricing-tiers-comparison-list-item {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-pricing-tiers-comparison-view=\"table\"] {\n  display: none;\n}\n\
[data-blocks-pricing-tiers-comparison-view=\"list\"] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-pricing-tiers-comparison-logos-section] {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n  text-align: center;\n}\n\
[data-scope=\"list\"][data-part=\"root\"][data-blocks-pricing-tiers-comparison-logos] {\n  display: flex;\n  flex-wrap: wrap;\n  justify-content: center;\n  gap: var(--fandhe-space-6);\n  margin: 0;\n  padding: 0;\n  list-style: none;\n}\n\
[data-scope=\"list\"][data-part=\"item\"].blocks-pricing-tiers-comparison-logo {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-pricing-tiers-comparison-caption] {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-pricing-tiers-comparison-band] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);\n  border-radius: var(--fandhe-radius-lg);\n  padding: var(--fandhe-space-8);\n}\n\
[data-blocks-pricing-tiers-comparison-band-header] {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n  max-width: 36rem;\n}\n\
[data-blocks-pricing-tiers-comparison-band] [data-blocks-pricing-tiers-comparison-band-header] [data-scope=\"heading\"],\n\
[data-blocks-pricing-tiers-comparison-band] [data-blocks-pricing-tiers-comparison-band-header] [data-scope=\"text\"] {\n  color: inherit;\n}\n\
[data-blocks-pricing-tiers-comparison-band] [data-blocks-pricing-tiers-comparison-caption] {\n  color: inherit;\n  opacity: 0.8;\n}\n\
@media (min-width: 48rem) {\n  [data-blocks-pricing-tiers-comparison-grid] {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n  [data-blocks-pricing-tiers-comparison-view=\"table\"] {\n    display: block;\n  }\n  [data-blocks-pricing-tiers-comparison-view=\"list\"] {\n    display: none;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// header/card/badge/button/table/list/icon/text/tabs の各 `data-scope`
    /// が出力され、カード枚数・推奨枚数・badge 数が「形 A（3 枚・推奨 1）+
    /// 形 B（2 tabs インスタンス × 2 billing パネル × 3 枚・推奨 1）」の
    /// 定数ベース導出と一致し、`<form>`・暗黙 submit・死リンク・`data:`
    /// URI を持ち込んでいないことを固定する。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());

        for scope in [
            r#"data-scope="heading""#,
            r#"data-scope="text""#,
            r#"data-scope="badge""#,
            r#"data-scope="card""#,
            r#"data-scope="button""#,
            r#"data-scope="table""#,
            r#"data-scope="list""#,
            r#"data-scope="icon""#,
            r#"data-scope="tabs""#,
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }

        // 形 A: PLANS.len() 枚。形 B: 2 tabs インスタンス（monthly/yearly
        // 選択）× 2 billing パネル（monthly/yearly content）× PLANS.len()
        // 枚（非選択パネルも `hidden` 付きで SSR 出力に存在するため数える）。
        let form_a_cards = PLANS.len();
        let form_b_cards = 2 * 2 * PLANS.len();
        let recommended_per_grid = PLANS.iter().filter(|p| p.recommended).count();

        assert_eq!(
            html.matches("data-blocks-pricing-tiers-comparison-card=\"\"")
                .count(),
            form_a_cards + form_b_cards,
            "demo output should render form A + form B plan cards"
        );
        assert_eq!(
            html.matches("data-blocks-pricing-tiers-comparison-recommended=\"\"")
                .count(),
            recommended_per_grid + 2 * 2 * recommended_per_grid,
            "demo output should mark the recommended card in every grid instance"
        );
        assert_eq!(
            html.matches(r#"data-scope="badge""#).count(),
            recommended_per_grid + 2 * 2 * recommended_per_grid,
            "each recommended card renders exactly 1 badge"
        );

        for absent in ["<form", "type=\"submit\"", "href=\"#\"", "src=\"data:"] {
            assert!(
                !html.contains(absent),
                "demo output should never contain {absent}"
            );
        }
    }

    /// 顧客ロゴ列（形 A）が [`dummy_assets::COMPANY_NAMES`] 先頭 5 件を
    /// 可視テキストで示し、各ロゴアイコンが装飾（`aria-hidden="true"`）で
    /// あること。
    #[test]
    fn logo_row_shows_visible_company_names() {
        let html = render(&demo());

        assert_eq!(
            html.matches("data-blocks-pricing-tiers-comparison-logos=\"\"")
                .count(),
            1,
            "demo output should render exactly 1 logo row"
        );

        for name in &dummy_assets::COMPANY_NAMES[..5] {
            let escaped = name.replace('&', "&amp;");
            assert!(
                html.contains(&escaped),
                "company name {name} should be visible in the logo row"
            );
        }

        assert_eq!(
            html.matches(r#"aria-hidden="true""#).count(),
            5,
            "each of the 5 logo icons should be decorative"
        );
    }

    /// 暗色帯 + 周期切替（形 B）が月額選択・年額選択の 2 インスタンスを
    /// 別々の id で持ち、それぞれの選択状態・年額価格・期間表記が出力に
    /// 現れること。
    #[test]
    fn billing_band_renders_monthly_and_yearly_states() {
        let html = render(&demo());

        assert_eq!(
            html.matches(r#"data-scope="tabs" data-part="root""#)
                .count(),
            2,
            "demo output should render exactly 2 tabs instances"
        );
        assert!(html.contains(
            "id=\"blocks-pricing-tiers-comparison-billing-monthly-trigger-monthly\" role=\"tab\" aria-selected=\"true\""
        ));
        assert!(html.contains(
            "id=\"blocks-pricing-tiers-comparison-billing-yearly-trigger-yearly\" role=\"tab\" aria-selected=\"true\""
        ));

        for plan in PLANS.iter() {
            assert!(
                html.contains(plan.price_yearly),
                "yearly price {} should appear in the demo output",
                plan.price_yearly
            );
        }
        assert!(html.contains(" / 年"));
        assert!(html.contains(" / 月"));
    }

    /// [`LAYOUT_CSS`] が暗色帯で fg/bg トークンを反転し、`color: inherit`
    /// の上書きを帯の header 属性配下へスコープ限定していること（カード
    /// 内部の文字色へは波及させない）。
    #[test]
    fn layout_css_band_inverts_and_scopes_color_inherit() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-pricing-tiers-comparison-band] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-pricing-tiers-comparison-band] [data-blocks-pricing-tiers-comparison-band-header] [data-scope=\"heading\"],"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-pricing-tiers-comparison-band] [data-blocks-pricing-tiers-comparison-band-header] [data-scope=\"text\"] {\n  color: inherit;\n}"
        ));
        // カードの `data-scope="text"`（description）は band-header 配下
        // ではないため、この上書きセレクタの対象外であること。
        assert!(!LAYOUT_CSS
            .contains("[data-blocks-pricing-tiers-comparison-band] [data-scope=\"card\"]"));
    }

    /// 各 `FeatureRow.values.len()` が [`PLANS`] の件数と一致し、`colspan`
    /// のリテラル（[`COL_COUNT_STR`]）が `PLANS.len() + 1` と一致すること。
    #[test]
    fn feature_rows_match_plan_count_and_colspan() {
        for category in CATEGORIES.iter() {
            for row in category.rows.iter() {
                assert_eq!(row.values.len(), PLANS.len());
            }
        }
        assert_eq!(COL_COUNT_STR, (PLANS.len() + 1).to_string());
    }

    /// [`LAYOUT_CSS`] が狭幅では 1 列・一覧表示、`>= 48rem` で 3 列・比較表
    /// 表示へ切り替わり、表と一覧の表示が反転すること。
    #[test]
    fn layout_css_stacks_on_narrow_and_three_columns_on_md() {
        assert!(LAYOUT_CSS.contains("grid-template-columns: minmax(0, 1fr);"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: repeat(3, minmax(0, 1fr));"));

        let before_media = LAYOUT_CSS.split("@media").next().unwrap();
        assert!(before_media.contains(
            r#"[data-blocks-pricing-tiers-comparison-view="table"] {
  display: none;"#
        ));
        assert!(before_media.contains(
            r#"[data-blocks-pricing-tiers-comparison-view="list"] {
  display: flex;"#
        ));

        let after_media = LAYOUT_CSS.split("@media").nth(1).unwrap();
        assert!(after_media.contains(
            r#"[data-blocks-pricing-tiers-comparison-view="table"] {
    display: block;"#
        ));
        assert!(after_media.contains(
            r#"[data-blocks-pricing-tiers-comparison-view="list"] {
    display: none;"#
        ));
    }

    /// ルート class（`blocks-pricing-tiers-comparison-layout`）が
    /// [`BLOCK`] の `demo_class`（`blocks-pricing-tiers-comparison`）とは
    /// 別名であること（既存 block からの教訓の踏襲）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-pricing-tiers-comparison-layout\""));
        assert_ne!(BLOCK.demo_class, "blocks-pricing-tiers-comparison-layout");
    }

    /// 比較表・狭幅一覧のどちらも行見出しが `scope="row"` で出ること
    /// （機能名行・カテゴリ見出し行の双方）。
    #[test]
    fn table_row_headers_use_scope_row() {
        let html = render(&demo());
        let feature_row_count: usize = CATEGORIES.iter().map(|c| c.rows.len()).sum();
        assert_eq!(
            html.matches(r#"scope="row""#).count(),
            CATEGORIES.len() + feature_row_count
        );
    }
}
