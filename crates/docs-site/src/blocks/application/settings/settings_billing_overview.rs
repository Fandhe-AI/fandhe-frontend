//! `settings-billing-overview` block（イシュー #2984。Application / Settings
//! カテゴリ）。上段の請求関連統計 3 件、中段の現在プラン・
//! 候補プランのカード 2 枚、下段のサブスクリプション一覧テーブルを合成する。
//! 主参照 R0240（代表構成）。`_/blocks-intake/` の対応ファイルは本イシュー
//! 着手時点で本 worktree に存在しないため、原稿・本コメントには対応表 ID
//! のみを記す（`profile-detail-datalist`〔イシュー #2937〕・
//! `list-title-meta`〔イシュー #2925〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `stat` / `card` / `badge` / `button` / `table` / `toggle-tip` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 集約元は R0240 の単一構成
//!
//! 他の block（`profile-detail-datalist` 等）が持つ「版 A/B の複数構成」を
//! 束ねる作りとは異なり、本 block は主参照 R0240 のみを対象とする単一構成
//! である。
//!
//! # 現在プランの強調は toggle-tip を `Open` 固定で常時可視にする
//!
//! headless 層 `toggle_tip` の `positioner`/`content` は `OpenState::Closed`
//! のとき `hidden` 存在属性を付与するため（`crates/headless-ui/src/
//! toggle_tip.rs`）、docs サイトの無 JS 静的 Demo で `Closed` のまま使うと
//! 補足ヒントを一切閲覧できない（`pricing_tiers_extra_row` の既存判断と
//! 同型）。本 block は現在プランカードの補足ヒント 1 個だけを持つため、
//! `OpenState::Open` 固定で常時可視にする（`Closed` 併記による補足文の
//! 別経路は不要）。`trigger` は `disabled: true`（静的表示のため開閉自体を
//! 提供しない）。headless 層の `positioner` は既定 `position: absolute` の
//! ため、`Open` 固定のままだとプラン価格・説明の上へ重なって表示される。
//! [`LAYOUT_CSS`] で `.blocks-settings-billing-overview-stack
//! [data-scope="toggle-tip"][data-part="positioner"]` を `position: static`
//! へ中和し、通常のドキュメントフローへ乗せて重なりを解消する
//! （`pricing_tiers_extra_row` の既存判断と同型）。このセレクタは block
//! ルート class で限定する（`blocks.css` は全 block ページ共通で読み込ま
//! れるため、限定しないと他 block の toggle-tip も `position: static` に
//! なりオーバーレイ表示が崩れる）。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::card::root`]・
//! [`fandhe_frontend_pre_styled_ui::stat::root`]・
//! [`fandhe_frontend_pre_styled_ui::badge::badge`]・
//! [`fandhe_frontend_pre_styled_ui::button::button`]・
//! [`fandhe_frontend_pre_styled_ui::table::root`] はいずれも `drop_class_attr`
//! で呼び出し側 `class` を除去してから内部 variant クラスと合成するため、
//! これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-settings-billing-overview-*`）。レイアウト用ラッパー
//! （統計・プランのグリッド）・`card::header`/`body`/`footer`/`title`・
//! `stat::label`/`value_text`/`help_text`・`table::row`/`cell` は素の
//! `class="blocks-settings-billing-overview-*"` を使う。
//!
//! # 現在プランの強調・狭幅の 1 列化は block 側 CSS が描く
//!
//! [`LAYOUT_CSS`] の `[data-blocks-settings-billing-overview-plan-card]
//! [data-current]` へ `border-color: var(--fandhe-color-accent)` を宣言し
//! 強調する（`pricing_tiers_extra_row` の featured カード強調と同型の
//! トークン参照）。Demo 枠の幅はビューポート幅と一致しないため、
//! `@container`（コンテナクエリ）で判定し（`profile_detail_datalist` 等と
//! 同型のパターン）、`.blocks-settings-billing-overview-stack` へ
//! `container-type: inline-size` を宣言したうえで、コンテナ幅が `40rem`
//! 未満のとき統計・プランの各グリッドを 1 列へ切り替える。table は
//! `.blocks-demo` 側の `overflow-x: auto` に任せ、本 block 固有の CSS は
//! 持たない。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。操作ボタンは
//! `button::button` の既定 `type="button"` のまま用いる。請求・決済処理、
//! 送信先 URL、カード番号・請求先住所などの決済情報は一切含めない
//! （表示は金額・プラン名・請求日のみ、すべて架空）。
//!
//! # ダミー素材について
//!
//! プラン名・価格は `crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS`
//! （Starter $9 / Growth $29 / Scale $79）を使い、Growth を現在プラン・
//! Scale を候補プランとする。統計値（今月の請求額・利用シート数・次回
//! 請求日）・サブスクリプション一覧の行はすべて本ファイル内の架空値で、
//! 実在の企業・人物・メールアドレス・クレデンシャル・PII は含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::toggle_tip::{self, OpenState};

/// 補足ヒント `content` の一意な `id`（`toggle_tip::trigger` の `controls`
/// と対で使う。ダングリング aria 参照・id 重複防止は `blocks_contract.rs`
/// が検証する）。
const PLAN_TIP_CONTENT_ID: &str = "blocks-settings-billing-overview-plan-tip";

/// 上段の統計カード 1 枚分のデータ（ラベル・値・増減方向・増減バッジ文言・
/// 補足文）。
struct StatDatum {
    label: &'static str,
    value: &'static str,
    trend_up: bool,
    trend_label: &'static str,
    help: &'static str,
}

/// 上段 3 統計（今月の請求額・利用シート数・次回請求日、いずれも架空値）。
const STATS: &[StatDatum] = &[
    StatDatum {
        label: "今月の請求額",
        value: "$29.00",
        trend_up: false,
        trend_label: "-2.1%",
        help: "先月比で減少",
    },
    StatDatum {
        label: "利用シート数",
        value: "12",
        trend_up: true,
        trend_label: "+3",
        help: "先月比で増加",
    },
    StatDatum {
        label: "次回請求日",
        value: "2026-10-15",
        trend_up: true,
        trend_label: "自動更新",
        help: "解約は前日まで可能",
    },
];

/// サブスクリプション一覧 1 行分（サービス名・プラン・請求日・金額・状態）。
struct SubscriptionRow {
    service: &'static str,
    plan: &'static str,
    billed_on: &'static str,
    amount: &'static str,
    status: &'static str,
    status_variant: BadgeVariant,
}

/// 下段テーブルの架空サブスクリプション 4 件。
const SUBSCRIPTIONS: &[SubscriptionRow] = &[
    SubscriptionRow {
        service: "Lumenbridge Systems",
        plan: "Growth",
        billed_on: "2026-09-15",
        amount: "$29.00",
        status: "有効",
        status_variant: BadgeVariant::Subtle,
    },
    SubscriptionRow {
        service: "Verdant Foundry",
        plan: "Starter",
        billed_on: "2026-09-01",
        amount: "$9.00",
        status: "有効",
        status_variant: BadgeVariant::Subtle,
    },
    SubscriptionRow {
        service: "Trellisworks Co.",
        plan: "Scale",
        billed_on: "2026-08-20",
        amount: "$79.00",
        status: "更新待ち",
        status_variant: BadgeVariant::Outline,
    },
    SubscriptionRow {
        service: "Aurelia Dynamics",
        plan: "Starter",
        billed_on: "2026-07-30",
        amount: "$9.00",
        status: "解約済み",
        status_variant: BadgeVariant::Surface,
    },
];

/// 1 枚の統計カード（`card` + `stat` + `badge` の合成、`dashboard_01` の
/// `stat_card` ヘルパと同型）。
fn stat_card(datum: &StatDatum) -> Node {
    let trend_badge = badge(
        &BadgeProps {
            variant: BadgeVariant::Outline,
            ..BadgeProps::default()
        },
        vec![],
        vec![text(datum.trend_label)],
    );

    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-billing-overview-stat-card", "")],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    card::title(
                        vec![("class", "blocks-settings-billing-overview-stat-label")],
                        vec![text(datum.label)],
                    ),
                    card::action(vec![], vec![trend_badge]),
                ],
            ),
            card::body(
                vec![],
                vec![stat::root(
                    Size::Lg,
                    vec![],
                    vec![stat::value_text(vec![], vec![text(datum.value)])],
                )],
            ),
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

/// 現在プランカードに添える補足ヒント（`OpenState::Open` 固定で常時可視。
/// モジュール doc「現在プランの強調は toggle-tip を `Open` 固定で常時可視
/// にする」節参照）。
fn plan_tip() -> Node {
    let state = OpenState::Open;
    toggle_tip::root(
        state,
        vec![],
        vec![
            toggle_tip::trigger(
                state,
                true,
                Some(PLAN_TIP_CONTENT_ID),
                vec![("aria-label", "現在プランの補足")],
                vec![text("\u{24d8}")],
            ),
            toggle_tip::positioner(
                state,
                vec![],
                vec![toggle_tip::content(
                    state,
                    Some(PLAN_TIP_CONTENT_ID),
                    vec![],
                    vec![text(
                        "次回請求日の前日までにダウングレード・解約手続きを完了すると、\
                         当月分の差額は請求されません。",
                    )],
                )],
            ),
        ],
    )
}

/// 現在プランカード（Growth）。ヒント常時表示・強調枠・「プランを管理」
/// ボタンを持つ。
fn current_plan_card() -> Node {
    let (name, price) = SAMPLE_PRICE_TIERS[1];
    card::root(
        CardProps::default(),
        vec![
            ("data-blocks-settings-billing-overview-plan-card", ""),
            ("data-current", ""),
        ],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    card::title(vec![], vec![text(name)]),
                    card::action(
                        vec![],
                        vec![badge(
                            &BadgeProps {
                                variant: BadgeVariant::Solid,
                                ..BadgeProps::default()
                            },
                            vec![],
                            vec![text("現在のプラン")],
                        )],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![
                    div(
                        vec![("class", "blocks-settings-billing-overview-plan-price")],
                        vec![text(format!("{price} / 月"))],
                    ),
                    card::description(
                        vec![],
                        vec![text("成長中のチーム向けに機能を拡張した構成です。")],
                    ),
                    plan_tip(),
                ],
            ),
            card::footer(
                vec![],
                vec![button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("プランを管理")],
                )],
            ),
        ],
    )
}

/// 候補プランカード（Scale）。「アップグレード」導線を持つ。
fn candidate_plan_card() -> Node {
    let (name, price) = SAMPLE_PRICE_TIERS[2];
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-billing-overview-plan-card", "")],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    card::title(vec![], vec![text(name)]),
                    card::action(
                        vec![],
                        vec![badge(
                            &BadgeProps {
                                variant: BadgeVariant::Outline,
                                ..BadgeProps::default()
                            },
                            vec![],
                            vec![text("おすすめ")],
                        )],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![
                    div(
                        vec![("class", "blocks-settings-billing-overview-plan-price")],
                        vec![text(format!("{price} / 月"))],
                    ),
                    card::description(vec![], vec![text("大規模なチーム向けの上位構成です。")]),
                ],
            ),
            card::footer(
                vec![],
                vec![button(
                    &ButtonProps {
                        variant: ButtonVariant::Solid,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("アップグレード")],
                )],
            ),
        ],
    )
}

/// サブスクリプション一覧テーブルの 1 行。
fn subscription_row(row: &SubscriptionRow) -> Node {
    table::row(
        vec![],
        vec![
            table::row_header(vec![], vec![text(row.service)]),
            table::cell(vec![], vec![text(row.plan)]),
            table::cell(vec![], vec![text(row.billed_on)]),
            table::cell(vec![("data-align", "end")], vec![text(row.amount)]),
            table::cell(
                vec![],
                vec![badge(
                    &BadgeProps {
                        variant: row.status_variant,
                        ..BadgeProps::default()
                    },
                    vec![],
                    vec![text(row.status)],
                )],
            ),
        ],
    )
}

/// `settings-billing-overview` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。
pub fn demo() -> Node {
    let stats = div(
        vec![("class", "blocks-settings-billing-overview-stats")],
        STATS.iter().map(stat_card).collect(),
    );

    let plans = div(
        vec![("class", "blocks-settings-billing-overview-plans")],
        vec![current_plan_card(), candidate_plan_card()],
    );

    let header_row = table::row(
        vec![],
        vec![
            table::column_header(vec![], vec![text("サービス")]),
            table::column_header(vec![], vec![text("プラン")]),
            table::column_header(vec![], vec![text("請求日")]),
            table::column_header(vec![("data-align", "end")], vec![text("金額")]),
            table::column_header(vec![], vec![text("状態")]),
        ],
    );

    let table_node = table::root(
        TableProps {
            size: Size::Md,
            ..TableProps::default()
        },
        vec![],
        vec![
            table::caption(vec![], vec![text("サブスクリプション一覧")]),
            table::header(vec![], vec![header_row]),
            table::body(vec![], SUBSCRIPTIONS.iter().map(subscription_row).collect()),
        ],
    );

    div(
        vec![("class", "blocks-settings-billing-overview-stack")],
        vec![stats, plans, table_node],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-billing-overview/",
    title: "settings-billing-overview",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_billing_overview.rs",
    demo_class: "blocks-settings-billing-overview",
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
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Toggle Tip",
            path: "/themes/toggle-tip/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_billing_overview` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-settings-billing-overview-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  container-type: inline-size;\n  container-name: blocks-settings-billing-overview;\n}\n\
.blocks-settings-billing-overview-stats {\n  display: grid;\n  grid-template-columns: repeat(3, 1fr);\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-billing-overview-plans {\n  display: grid;\n  grid-template-columns: repeat(2, 1fr);\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-billing-overview-plan-price {\n  font-size: var(--fandhe-font-font-size-2xl);\n  font-weight: var(--fandhe-font-font-weight-bold);\n}\n\
.blocks-settings-billing-overview-stat-label {\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-normal);\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-settings-billing-overview-plan-card][data-current] {\n  border-color: var(--fandhe-color-accent);\n}\n\
.blocks-settings-billing-overview-stack [data-scope=\"toggle-tip\"][data-part=\"positioner\"] {\n  position: static;\n}\n\
@container blocks-settings-billing-overview (max-width: 40rem) {\n  \
.blocks-settings-billing-overview-stats {\n    grid-template-columns: 1fr;\n  }\n  \
.blocks-settings-billing-overview-plans {\n    grid-template-columns: 1fr;\n  }\n\
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
            "data-scope=\"stat\"",
            "data-scope=\"card\"",
            "data-scope=\"badge\"",
            "data-scope=\"button\"",
            "data-scope=\"table\"",
            "data-scope=\"toggle-tip\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn demo_has_five_cards_with_one_current() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-settings-billing-overview-stat-card=\"\"")
                .count(),
            3
        );
        assert_eq!(
            html.matches("data-blocks-settings-billing-overview-plan-card=\"\"")
                .count(),
            2
        );
        assert_eq!(html.matches("data-current=\"\"").count(), 1);
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
    fn buttons_use_type_button_only() {
        let html = demo_html();
        assert_eq!(html.matches("<button").count(), 3);
        assert_eq!(html.matches("type=\"button\"").count(), 3);
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(
            LAYOUT_CSS.contains("@container blocks-settings-billing-overview (max-width: 40rem)")
        );
    }

    /// [`LAYOUT_CSS`] が toggle-tip positioner を `position: static` へ
    /// 中和するセレクタを block ルート class で限定していることを固定する
    /// （`blocks.css` は全 block ページ共通で読み込まれるため、限定しない
    /// と他 block の toggle-tip 表示が崩れる。`pricing_tiers_extra_row` と
    /// 同型のセレクタ範囲限定）。
    #[test]
    fn layout_css_scopes_toggle_tip_positioner_neutralization_to_block_root() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-settings-billing-overview-stack [data-scope=\"toggle-tip\"][data-part=\"positioner\"] {\n  position: static;"
        ));
    }
}
