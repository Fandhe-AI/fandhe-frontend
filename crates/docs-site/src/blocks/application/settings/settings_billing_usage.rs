//! `settings-billing-usage` block（イシュー #2985。Application / Settings
//! カテゴリ、最初の block）。現在プランの定義リスト → 使用量・残席数の
//! 進捗バー → 支払方法と請求詳細を縦に積んだ版（A）と、残席数 + 請求履歴
//! テーブルを添えた版（B）を並記する。主参照は対応表 ID R0239（代表構成）と
//! R0238（残席数 + 請求履歴テーブル）。`_/blocks-intake/` の対応ファイルは
//! 本イシュー着手時点で本 worktree に存在しないため、原稿・本コメントには
//! 対応表 ID のみを記す（`profile-detail-datalist`〔#2937〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `data-list` / `progress` / `badge` / `button` / `card` / `table` /
//! `toggle-tip` の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 2 版と集約元の対応（原稿「原案差分メモ」節と対になる索引）
//!
//! - **A（代表構成）**: R0239。現在プラン（`data-list` 横並び + バッジ）→
//!   使用量（`progress` 3 本 + toggle tip 補足）→ 支払方法と請求詳細
//!   （`data-list` 縦積み）の 3 `card` を縦に積む
//! - **B（残席数 + 請求履歴テーブル）**: R0238。残席数の `progress` 1 本を
//!   持つ `card` + 請求履歴の `table` の 2 ブロックで構成する
//!
//! # 警告色の進捗バー（しきい値 [`WARN_THRESHOLD`]）
//!
//! 上限に近い使用量を視覚的に示すため、消費率が [`WARN_THRESHOLD`]（80%）
//! 以上の `progress` は [`fandhe_frontend_pre_styled_ui::recipe::ColorPalette::Warning`]
//! を、それ未満は既定の `Accent` を使う（`feature_tabs_panel.rs` の
//! `trigger_with_progress` と同じ `Progress::new` + styled `root`/`range`
//! の組み方を踏襲）。
//!
//! # `aria-label` の明示（Codex P1 是正の踏襲）
//!
//! [`fandhe_frontend_pre_styled_ui::progress::root`] は
//! `aria-labelledby` を自動配線しないため、`marketing/feature/feature_tabs_panel.rs`
//! の是正と同じく呼び出し側で `aria-label` を明示する（無地の
//! `role="progressbar"` だけでは支援技術から進捗の意味が読めないため）。
//!
//! # toggle tip は `OpenState::Open` 固定・`disabled` トリガー
//!
//! 使用量カードの見出し横に補足（集計更新タイミング）を添える toggle tip
//! は、`marketing/pricing/pricing_tiers_extra_row.rs` の `feature_note` と
//! 異なり `OpenState::Open` 固定とする（headless 層の `positioner`/`content`
//! は `Closed` のとき `hidden` を付与するため、無 JS 静的 Demo で常時可視の
//! 補足文にするには `Open` 固定が必要）。開閉操作自体は無 JS では意味を
//! 持たないため `trigger` は `disabled: true` で無効化する
//! （`aria-controls`/`aria-expanded` は disabled でも出力される、
//! headless 層の契約どおり）。[`LAYOUT_CSS`] は positioner を
//! `position: static` へ中和し、通常のドキュメントフローに乗せる
//! （`pricing_tiers_extra_row.rs` と同型の判断）。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::card::root`]・
//! [`fandhe_frontend_pre_styled_ui::badge::badge`]・
//! [`fandhe_frontend_pre_styled_ui::button::button`]・
//! [`fandhe_frontend_pre_styled_ui::table::root`]・
//! [`fandhe_frontend_pre_styled_ui::data_list::root`] はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-settings-billing-usage-*`）。レイアウト用ラッパー
//! （素の `<div>`）は `class="blocks-settings-billing-usage-*"` を使う
//! （`profile_detail_datalist` と同型の判断）。
//!
//! # 版 A は常時 1 列・版 B のみ狭幅で 1 列へ切り替え（`@container`）
//!
//! `.blocks-settings-billing-usage-cards` は両版で共有するが、版 A
//! （3 `card` 縦積み、モジュール doc冒頭「2 版と集約元の対応」節参照）は
//! `[data-variant="a"] > .blocks-settings-billing-usage-cards` の
//! grid-template-columns 上書きで常時 1 列固定とする。版 B（`card` +
//! `table` の 2 ブロック）のみ、`@container`（コンテナクエリ）で広幅
//! 2 列 → 狭幅 1 列へ切り替える。Demo 枠の幅はビューポート幅と一致
//! しないため `@container` で判定し、[`LAYOUT_CSS`] のラッパー
//! `.blocks-settings-billing-usage-stack` へ `container-type:
//! inline-size` を宣言する。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。操作ボタンは
//! `button::button` の既定 `type="button"` のまま用いる。
//!
//! # ダミー素材について
//!
//! プラン名・価格は `crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS`、
//! 請求先名は `dummy_assets::COMPANY_NAMES` を使う。金額・日付・件数は
//! 本ファイル内 `const` の架空値であり、カード番号・請求先住所・メール
//! アドレス・実在ブランド名は含まない（`pricing_tiers_extra_row.rs` と
//! 同じ基準）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
use fandhe_frontend_pre_styled_ui::progress::{self, Orientation, ProgressProps};
use fandhe_frontend_pre_styled_ui::recipe::{ColorPalette, Size};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::toggle_tip::{self, OpenState};

/// 使用量進捗バーの警告しきい値（%）。これ以上の消費率は
/// [`ColorPalette::Warning`] で示す（モジュール doc「警告色の進捗バー」
/// 節参照）。
const WARN_THRESHOLD: f64 = 80.0;

/// 使用量カードの補足 toggle tip の `content` id（ページ内一意、版 A のみ
/// 使用する）。
const USAGE_TIP_ID: &str = "blocks-settings-billing-usage-usage-tip";

/// ラベル・値の 1 行（`data_list::item` + `item-label` + `item-value`）を
/// 組み立てる。
fn row(label: &'static str, value: String) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 指定 orientation の `data_list::root`（CSS フック付き）を組む。
fn list(orientation: DataListOrientation, rows: Vec<Node>) -> Node {
    data_list::root(
        DataListProps {
            orientation,
            ..DataListProps::default()
        },
        vec![("data-blocks-settings-billing-usage-list", "")],
        rows,
    )
}

/// ラベル・消費率（%）・可視の値テキストから 1 本の進捗バー行を組む
/// （モジュール doc「警告色の進捗バー」「`aria-label` の明示」節参照）。
fn usage_bar(label: &'static str, percent: f64, value_text: String) -> Node {
    let p = Progress::new(0.0, 100.0, Some(percent), Orientation::Horizontal);
    let palette = if percent >= WARN_THRESHOLD {
        ColorPalette::Warning
    } else {
        ColorPalette::Accent
    };
    // ヘッダーはラベル（左）と、数値・バッジをまとめた 1 つの flex グループ
    // （右）の 2 child のみを `justify-content: space-between` で並べる。
    // 数値を独立の flex child にすると、バッジ側との間隔がラベル側との間隔と
    // 同じ扱いになり数値とバッジが離れて見えるため（PR #3437 レビュー指摘）、
    // 数値とバッジは常に同じ右側グループへ入れる。
    let label_node = styled_text::text(&TextProps::default(), vec![], vec![text(label)]);
    let value_node = styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(value_text)],
    );
    let mut value_and_badge_children = vec![value_node];
    if percent >= WARN_THRESHOLD {
        value_and_badge_children.push(badge(
            &BadgeProps {
                variant: BadgeVariant::Outline,
                palette: ColorPalette::Warning,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("上限間近")],
        ));
    }
    let value_and_badge = div(
        vec![(
            "class",
            "blocks-settings-billing-usage-usage-row-value-group",
        )],
        value_and_badge_children,
    );
    let header_children = vec![label_node, value_and_badge];
    div(
        vec![("class", "blocks-settings-billing-usage-usage-row")],
        vec![
            div(
                vec![("class", "blocks-settings-billing-usage-usage-row-header")],
                header_children,
            ),
            progress::root(
                &p,
                &ProgressProps {
                    size: Size::Sm,
                    palette,
                    ..ProgressProps::default()
                },
                None,
                vec![("aria-label", label)],
                vec![p.track(vec![], vec![progress::range(&p, vec![])])],
            ),
        ],
    )
}

/// A: 現在プラン・使用量・支払方法の 3 `card`（R0239）。
fn version_representative() -> Node {
    let (tier_name, tier_price) = dummy_assets::SAMPLE_PRICE_TIERS[1];

    let plan_card = card::root(
        CardProps::default(),
        vec![],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    card::title(vec![], vec![text("現在のプラン")]),
                    card::action(
                        vec![],
                        vec![badge(
                            &BadgeProps {
                                variant: BadgeVariant::Solid,
                                ..BadgeProps::default()
                            },
                            vec![],
                            vec![text(tier_name)],
                        )],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![list(
                    DataListOrientation::Horizontal,
                    vec![
                        row("プラン名", format!("{tier_name}（{tier_price} / 月）")),
                        row("請求周期", "月次".to_string()),
                        row("次回請求日", "2026-10-15".to_string()),
                        row("利用シート数", "22 / 25 席".to_string()),
                    ],
                )],
            ),
            card::footer(
                vec![],
                vec![button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("プランを変更")],
                )],
            ),
        ],
    );

    let usage_card = card::root(
        CardProps::default(),
        vec![],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    card::title(vec![], vec![text("使用量")]),
                    card::action(
                        vec![],
                        vec![toggle_tip::root(
                            OpenState::Open,
                            vec![],
                            vec![
                                toggle_tip::trigger(
                                    OpenState::Open,
                                    true,
                                    Some(USAGE_TIP_ID),
                                    vec![("aria-label", "使用量の補足")],
                                    vec![text("?")],
                                ),
                                toggle_tip::positioner(
                                    OpenState::Open,
                                    vec![],
                                    vec![toggle_tip::content(
                                        OpenState::Open,
                                        Some(USAGE_TIP_ID),
                                        vec![],
                                        vec![text("集計は毎日 0:00 UTC に更新されます。")],
                                    )],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
            card::body(
                vec![("class", "blocks-settings-billing-usage-usage-body")],
                vec![
                    usage_bar("API リクエスト", 82.0, "8,200 / 10,000 回".to_string()),
                    usage_bar("ストレージ", 46.0, "46 / 100 GB".to_string()),
                    usage_bar("メンバー", 50.0, "5 / 10 名".to_string()),
                ],
            ),
        ],
    );

    let billing_card = card::root(
        CardProps::default(),
        vec![],
        vec![
            card::header(
                vec![],
                vec![card::title(vec![], vec![text("支払方法と請求詳細")])],
            ),
            card::body(
                vec![],
                vec![list(
                    DataListOrientation::Vertical,
                    vec![
                        row("支払方法", "クレジットカード（末尾 4321）".to_string()),
                        row("請求先名", dummy_assets::COMPANY_NAMES[2].to_string()),
                        row("請求書送付", "メール".to_string()),
                    ],
                )],
            ),
            card::footer(
                vec![],
                vec![
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("支払方法を更新")],
                    ),
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("請求先を編集")],
                    ),
                ],
            ),
        ],
    );

    div(
        vec![
            ("class", "blocks-settings-billing-usage-variant"),
            ("data-variant", "a"),
        ],
        vec![
            div(
                vec![("class", "blocks-settings-billing-usage-variant-label")],
                vec![styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        size: TextSize::Sm,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text("A: 代表構成（プラン・使用量・支払方法）")],
                )],
            ),
            div(
                vec![("class", "blocks-settings-billing-usage-cards")],
                vec![plan_card, usage_card, billing_card],
            ),
        ],
    )
}

/// B: 残席数 + 請求履歴テーブル（R0238）。
fn version_seats_and_history() -> Node {
    let seats_percent = 88.0;
    let seats_card = card::root(
        CardProps::default(),
        vec![],
        vec![
            card::header(vec![], vec![card::title(vec![], vec![text("残席数")])]),
            card::body(
                vec![],
                vec![
                    usage_bar("シート使用状況", seats_percent, "22 / 25 席".to_string()),
                    div(
                        vec![("class", "blocks-settings-billing-usage-seats-footer")],
                        vec![
                            badge(
                                &BadgeProps {
                                    variant: BadgeVariant::Outline,
                                    palette: ColorPalette::Warning,
                                    ..BadgeProps::default()
                                },
                                vec![],
                                vec![text("残り 3 席")],
                            ),
                            button(&ButtonProps::default(), vec![], vec![text("席を追加")]),
                        ],
                    ),
                ],
            ),
        ],
    );

    const HISTORY_ROWS: &[(&str, &str, &str, &str, BadgeVariant)] = &[
        (
            "2026-09-15",
            "月額プラン（Growth）",
            "$29.00",
            "支払済",
            BadgeVariant::Subtle,
        ),
        (
            "2026-08-15",
            "月額プラン（Growth）",
            "$29.00",
            "支払済",
            BadgeVariant::Subtle,
        ),
        (
            "2026-07-15",
            "追加シート x3",
            "$15.00",
            "処理中",
            BadgeVariant::Outline,
        ),
        (
            "2026-06-15",
            "月額プラン（Growth）",
            "$29.00",
            "失敗",
            BadgeVariant::Surface,
        ),
    ];

    let history_table = table::root(
        TableProps::default(),
        vec![],
        vec![
            table::caption(vec![], vec![text("請求履歴")]),
            table::header(
                vec![],
                vec![table::row(
                    vec![],
                    vec![
                        table::column_header(vec![], vec![text("請求日")]),
                        table::column_header(vec![], vec![text("内容")]),
                        table::column_header(vec![("data-align", "end")], vec![text("金額")]),
                        table::column_header(vec![], vec![text("状態")]),
                    ],
                )],
            ),
            table::body(
                vec![],
                HISTORY_ROWS
                    .iter()
                    .map(|(date, desc, amount, status, variant)| {
                        table::row(
                            vec![],
                            vec![
                                table::row_header(vec![], vec![text(*date)]),
                                table::cell(vec![], vec![text(*desc)]),
                                table::cell(vec![("data-align", "end")], vec![text(*amount)]),
                                table::cell(
                                    vec![],
                                    vec![badge(
                                        &BadgeProps {
                                            variant: *variant,
                                            ..BadgeProps::default()
                                        },
                                        vec![],
                                        vec![text(*status)],
                                    )],
                                ),
                            ],
                        )
                    })
                    .collect(),
            ),
        ],
    );

    div(
        vec![
            ("class", "blocks-settings-billing-usage-variant"),
            ("data-variant", "b"),
        ],
        vec![
            div(
                vec![("class", "blocks-settings-billing-usage-variant-label")],
                vec![styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        size: TextSize::Sm,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text("B: 残席数 + 請求履歴テーブル")],
                )],
            ),
            div(
                vec![("class", "blocks-settings-billing-usage-cards")],
                vec![seats_card, history_table],
            ),
        ],
    )
}

/// `settings-billing-usage` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-billing-usage-stack")],
        vec![version_representative(), version_seats_and_history()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-billing-usage/",
    title: "settings-billing-usage",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_billing_usage.rs",
    demo_class: "blocks-settings-billing-usage",
    parts: &[
        Part {
            label: "Data List",
            path: "/themes/data-list/",
        },
        Part {
            label: "Progress",
            path: "/themes/progress/",
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
            label: "Card",
            path: "/themes/card/",
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

/// `settings_billing_usage` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
///
/// toggle-tip positioner の `position: static` 中和は `pricing_tiers_extra_row`
/// と同型の判断（モジュール doc「toggle tip は `OpenState::Open` 固定」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-settings-billing-usage-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  container-type: inline-size;\n  container-name: blocks-settings-billing-usage;\n}\n\
.blocks-settings-billing-usage-variant {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-billing-usage-variant-label {\n  font-weight: 600;\n}\n\
.blocks-settings-billing-usage-cards {\n  display: grid;\n  grid-template-columns: repeat(2, 1fr);\n  gap: var(--fandhe-space-4);\n}\n\
[data-variant=\"a\"] > .blocks-settings-billing-usage-cards {\n  grid-template-columns: 1fr;\n}\n\
.blocks-settings-billing-usage-usage-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-billing-usage-usage-row {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-settings-billing-usage-usage-row-header {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  justify-content: space-between;\n}\n\
.blocks-settings-billing-usage-usage-row-value-group {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: baseline;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-settings-billing-usage-seats-footer {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-2);\n  margin-block-start: var(--fandhe-space-4);\n}\n\
.blocks-settings-billing-usage-stack [data-scope=\"toggle-tip\"][data-part=\"positioner\"] {\n  position: static;\n}\n\
@container blocks-settings-billing-usage (max-width: 40rem) {\n  \
.blocks-settings-billing-usage-cards {\n    grid-template-columns: 1fr;\n  }\n\
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
            "data-scope=\"data-list\"",
            "data-scope=\"progress\"",
            "data-scope=\"badge\"",
            "data-scope=\"button\"",
            "data-scope=\"card\"",
            "data-scope=\"table\"",
            "data-scope=\"toggle-tip\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn warning_progress_count_matches_threshold_crossings() {
        let html = demo_html();
        // 版 A の API リクエスト（82%）+ 版 B のシート使用状況（88%）の 2 本が
        // WARN_THRESHOLD（80%）以上（モジュール doc「警告色の進捗バー」節参照）。
        assert_eq!(
            html.matches("fd-progress--color-palette-warning").count(),
            2
        );
    }

    #[test]
    fn all_progressbars_have_aria_label() {
        let html = demo_html();
        let progressbar_count = html.matches(r#"role="progressbar""#).count();
        let aria_label_count = html.matches(r#"aria-label=""#).count()
            - html.matches(r#"aria-label="使用量の補足""#).count();
        assert_eq!(progressbar_count, 4, "4 本の progress を想定");
        assert_eq!(aria_label_count, progressbar_count);
    }

    #[test]
    fn demo_has_two_variants() {
        let html = demo_html();
        assert!(html.contains(r#"data-variant="a""#));
        assert!(html.contains(r#"data-variant="b""#));
        assert_eq!(html.matches("data-variant=").count(), 2);
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"submit\""));
        let button_count = html.matches("<button").count();
        let type_button_count = html.matches(r#"type="button""#).count();
        assert_eq!(button_count, type_button_count);
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-settings-billing-usage (max-width: 40rem)"));
    }

    #[test]
    fn toggle_tip_is_fixed_open_and_trigger_disabled() {
        let html = demo_html();
        assert!(html.contains(r#"data-part="trigger""#));
        assert!(html.contains("disabled"));
        assert!(html.contains(r#"data-state="open""#));
        assert!(!html.contains("hidden"));
    }

    #[test]
    fn billing_history_table_has_four_rows_and_no_dummy_pii() {
        let html = demo_html();
        assert_eq!(html.matches("<tr").count(), 5, "見出し行 1 + データ 4 行");
        assert!(!html.contains("@example.com"));
    }
}
