//! `stats-cards` block（イシュー #2802。親トラッキング #2738 / #2730 系
//! 「Blocks 目的別パーツ拡充」配下、Marketing / Stats カテゴリ最初の
//! block。対応表 ID R0705（主参照）・R0338/R0341/R1300/R1304/R1309/R1310
//! （集約元、計 7 件）を構造の参照元とする合成例。取得手段・ファイル名・
//! 内部コンポーネント識別子は記載しない（`contact_split_info` と同じ
//! ライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `card` / `stat` / `icon` の 6 部品のみを
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 3 変種構成（集約元との差分は Demo の並記で扱う）
//!
//! 1. **基準形**（R0705 主参照・R1300「中央見出し + カード 4 件」・R1309
//!    「見出し + カード化した指標 3 列」・R1310「カード + アイコンバッジ +
//!    増減表示 + 下部リンク帯」を統合）: 中央寄せの見出し（eyebrow badge +
//!    heading + 説明文）+ カード 4 枚。各カードは装飾アイコン + 増減
//!    badge（up/down indicator）+ `stat`（label/value/help）+ footer の
//!    補足テキスト帯。R1310 の「下部リンク帯」は使用部品に `link` が無い
//!    ため本物のリンクにはせず、`card::footer` のテキスト帯へ置き換えた
//!    （差分メモは `site/blocks/stats-cards.md` 参照）。
//! 2. **淡色パネル + 大アイコン**（R0338「アイコン付きカード 3 枚」・
//!    R0341「淡色パネル + 大アイコン 3 枚」を統合）: 左寄せ見出し + カード
//!    3 枚。`CardVariant::Subtle` の中に大きい装飾アイコン（`Size::Xl`）を
//!    面色パネルへ収め、`stat` を並べる。増減 badge は持たない。
//! 3. **段状**（R1304「高さ違いの段状カード 3 枚。色違いは持ち込まない」）:
//!    `CardVariant::Elevated` のカード 3 枚に `data-blocks-stats-cards-step`
//!    （`1`〜`3`）を付与し、`md` 以上でのみ `min-height` を段階的に変えて
//!    段差を表現する（`md` 未満は 1 列で段差をなくす）。色違いの装飾は
//!    使わない（テーマの面色のみで見せる要件どおり）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、各変種見出しは
//! `HeadingLevel::H3` にする。カード内の項目名は `card::title`
//! （`<h3>` 固定）を使うと変種見出しと同じレベルになって階層が崩れるため、
//! `stat::label`（`<dt>`）で表し `heading` を増やさない
//! （`contact_split_info` の窓口名/拠点名と同じ判断）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # 自作幾何アイコン
//!
//! lucide 等の既存アイコンセットの path を複製しないため、
//! `contact_split_info::geo_icon` と同型の自作ヘルパ [`geo_icon`] で描く。
//! `path` へ `fill="none"` + `stroke="currentColor"` を明示し、
//! `icon::icon` の `<svg>` 側が固定で持つ `fill="currentColor"`（塗り面）を
//! 上書きして線画として描画する。いずれも隣に可視テキストがあるため装飾
//! 扱い（`IconProps::label` は `None` のまま、`aria-hidden="true"`）とする。
//! `stat::up_indicator`/`down_indicator` も装飾用途で `aria-hidden` 固定
//! （変化率は隣接テキストで伝わるため色だけに頼らない、WCAG 1.4.1）。
//!
//! # ブレークポイント（48rem/64rem をリテラル直書きする理由）
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint` の `Md`（768px =
//! 48rem）・`Lg`（1024px = 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ
//! 直書きする（`contact_split_info` と同じ判断）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 状態機械を持たない静的な合成例である。文言・数値はすべて架空のもの
//! （実企業名・実データ・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 装飾用の自作幾何アイコン（lucide 等の既存アイコンセットの path を
/// 複製しないための単純図形、`contact_split_info::geo_icon` と同型）。
fn geo_icon(size: Size, path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
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

/// 人物 2 体の幾何アイコン（利用者）。
fn users_icon() -> Node {
    geo_icon(
        Size::Sm,
        "M8 11a3 3 0 100-6 3 3 0 000 6z M3 20c0-3 2.5-5 5-5s5 2 5 5 \
         M16 7a2.5 2.5 0 110 5 M15 20c0-2.5 2-4.5 5-4.5",
    )
}

/// 時計の幾何アイコン（応答時間）。
fn clock_icon() -> Node {
    geo_icon(Size::Sm, "M12 3a9 9 0 100 18 9 9 0 000-18z M12 7v5l4 2")
}

/// チェックマーク盾の幾何アイコン（稼働率）。
fn shield_icon() -> Node {
    geo_icon(
        Size::Sm,
        "M12 3l7 3v6c0 4.5-3 7.5-7 9-4-1.5-7-4.5-7-9V6l7-3z M9 12l2 2 4-4",
    )
}

/// 建物の幾何アイコン（導入チーム数）。
fn building_icon() -> Node {
    geo_icon(
        Size::Sm,
        "M6 21V5h8v16 M14 21v-9h4v9 M9 8h2 M9 12h2 M9 16h2",
    )
}

/// 稲妻の幾何アイコン（処理速度）。
fn bolt_icon() -> Node {
    geo_icon(Size::Xl, "M13 3L4 14h6l-1 7 9-11h-6l1-7z")
}

/// 盾の幾何アイコン（ダウンタイム）。淡色パネル用に `Xl` サイズで描く。
fn panel_shield_icon() -> Node {
    geo_icon(
        Size::Xl,
        "M12 3l7 3v6c0 4.5-3 7.5-7 9-4-1.5-7-4.5-7-9V6l7-3z",
    )
}

/// ハートの幾何アイコン（満足度）。
fn heart_icon() -> Node {
    geo_icon(
        Size::Xl,
        "M12 20s-7-4.5-9.5-9A5 5 0 0112 6a5 5 0 019.5 5c-2.5 4.5-9.5 9-9.5 9z",
    )
}

/// 数値指標 1 件分の架空データ（実在の企業・データとは無関係）。
struct StatItem {
    label: &'static str,
    value: &'static str,
    unit: Option<&'static str>,
    help: &'static str,
    /// `Some((is_up, change_label))`: 増減表示。方向は実測値の増減を表し
    /// 「良し悪し」を表さない（応答時間の短縮は `is_up = false`）。
    trend: Option<(bool, &'static str)>,
    icon_fn: fn() -> Node,
}

/// 基準形（4 件）。R0705/R1300/R1309/R1310 の統合。
const BASELINE_ITEMS: [StatItem; 4] = [
    StatItem {
        label: "月間アクティブ利用者",
        value: "128,400",
        unit: None,
        help: "前月比の伸びを示す架空の指標です。",
        trend: Some((true, "+12%")),
        icon_fn: users_icon,
    },
    StatItem {
        label: "平均応答時間",
        value: "182",
        unit: Some("ms"),
        help: "直近 30 日間の平均値（架空）。",
        trend: Some((false, "-8%")),
        icon_fn: clock_icon,
    },
    StatItem {
        label: "稼働率",
        value: "99.98",
        unit: Some("%"),
        help: "直近 90 日間の実測値（架空）。",
        trend: Some((true, "+0.02pt")),
        icon_fn: shield_icon,
    },
    StatItem {
        label: "導入チーム数",
        value: "1,240",
        unit: None,
        help: "累計導入チーム数（架空）。",
        trend: Some((true, "+5%")),
        icon_fn: building_icon,
    },
];

/// 淡色パネル + 大アイコン（3 件）。R0338/R0341 の統合。
const PANEL_ITEMS: [StatItem; 3] = [
    StatItem {
        label: "平均処理速度",
        value: "4.2",
        unit: Some("倍"),
        help: "旧バージョン比（架空）。",
        trend: None,
        icon_fn: bolt_icon,
    },
    StatItem {
        label: "ダウンタイム",
        value: "1.5",
        unit: Some("分/月"),
        help: "月間平均（架空）。",
        trend: None,
        icon_fn: panel_shield_icon,
    },
    StatItem {
        label: "サポート満足度",
        value: "96",
        unit: Some("%"),
        help: "アンケート回答者の割合（架空）。",
        trend: None,
        icon_fn: heart_icon,
    },
];

/// 段状（3 件）。R1304。色違いは使わずテーマの面色のみで段差を表現する。
const STEPPED_ITEMS: [StatItem; 3] = [
    StatItem {
        label: "初年度",
        value: "120",
        unit: Some("社"),
        help: "導入社数（架空）。",
        trend: None,
        icon_fn: users_icon,
    },
    StatItem {
        label: "2 年目",
        value: "480",
        unit: Some("社"),
        help: "導入社数（架空）。",
        trend: None,
        icon_fn: users_icon,
    },
    StatItem {
        label: "3 年目",
        value: "1,240",
        unit: Some("社"),
        help: "導入社数（架空）。",
        trend: None,
        icon_fn: users_icon,
    },
];

/// `stat::root` 本体（label/value(+unit)/help_text）を組み立てる。
fn stat_body(item: &StatItem) -> Node {
    let mut value_children = vec![text(item.value)];
    if let Some(unit) = item.unit {
        value_children.push(stat::value_unit(vec![], vec![text(unit)]));
    }
    let mut children = vec![
        stat::label(vec![], vec![text(item.label)]),
        stat::value_text(vec![], value_children),
    ];
    children.push(stat::help_text(vec![], vec![text(item.help)]));
    stat::root(Size::Lg, vec![], children)
}

/// 基準形のカード 1 件（装飾アイコン + 増減 badge + stat + footer 補足）。
fn baseline_card(item: &StatItem) -> Node {
    let (is_up, change_label) = item.trend.expect("baseline items always carry a trend");
    let trend_badge = badge::badge(
        &BadgeProps {
            variant: BadgeVariant::Outline,
            palette: ColorPalette::Neutral,
            ..BadgeProps::default()
        },
        vec![("data-blocks-stats-cards-trend", "")],
        vec![
            if is_up {
                stat::up_indicator(vec![])
            } else {
                stat::down_indicator(vec![])
            },
            text(change_label),
        ],
    );

    card::root(
        CardVariant::Outline,
        vec![("data-blocks-stats-cards-card", "")],
        vec![
            card::header(
                vec![("data-blocks-stats-cards-card-header", "")],
                vec![
                    div(
                        vec![("data-blocks-stats-cards-icon", "")],
                        vec![(item.icon_fn)()],
                    ),
                    trend_badge,
                ],
            ),
            card::body(vec![], vec![stat_body(item)]),
            card::footer(
                vec![],
                vec![styled_text::text(
                    &TextProps {
                        size: TextSize::Sm,
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text("詳細は月次レポートでご確認いただけます（架空）。")],
                )],
            ),
        ],
    )
}

/// 淡色パネル + 大アイコンのカード 1 件。
fn panel_card(item: &StatItem) -> Node {
    card::root(
        CardVariant::Subtle,
        vec![("data-blocks-stats-cards-card", "")],
        vec![card::body(
            vec![("data-blocks-stats-cards-panel-body", "")],
            vec![
                div(
                    vec![("data-blocks-stats-cards-icon-panel", "")],
                    vec![(item.icon_fn)()],
                ),
                stat_body(item),
            ],
        )],
    )
}

/// 段状のカード 1 件（`data-blocks-stats-cards-step` で段差 CSS を選ぶ）。
fn stepped_card(item: &StatItem, step: u8) -> Node {
    let step_label = step.to_string();
    card::root(
        CardVariant::Elevated,
        vec![
            ("data-blocks-stats-cards-card", ""),
            ("data-blocks-stats-cards-step", step_label.as_str()),
        ],
        vec![card::body(vec![], vec![stat_body(item)])],
    )
}

/// 変種 1 件分（見出し + 説明 + カードグリッド）を組み立てる。
fn variant_section(
    eyebrow: Option<&'static str>,
    heading_text: &'static str,
    description: &'static str,
    grid_data_attr: &'static str,
    cards: Vec<Node>,
) -> Node {
    let mut head_children = vec![];
    if let Some(eyebrow) = eyebrow {
        head_children.push(badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Subtle,
                ..BadgeProps::default()
            },
            vec![],
            vec![text(eyebrow)],
        ));
    }
    head_children.push(heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text(heading_text)],
    ));
    head_children.push(styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(description)],
    ));

    div(
        vec![("data-blocks-stats-cards-section", "")],
        vec![
            div(
                vec![("data-blocks-stats-cards-section-head", "")],
                head_children,
            ),
            div(vec![(grid_data_attr, "")], cards),
        ],
    )
}

/// `stats-cards` の Demo 本体（基準形・淡色パネル・段状の 3 変種を縦に
/// 並べる）。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let baseline = variant_section(
        Some("PLATFORM METRICS"),
        "数字で見るプラットフォームの成果",
        "架空の指標です。ダミー数値は毎回同一の内容を返します。",
        "data-blocks-stats-cards-grid-baseline",
        BASELINE_ITEMS.iter().map(baseline_card).collect(),
    );

    let panel = variant_section(
        None,
        "運用品質の指標",
        "淡色パネルに大きなアイコンを添えた表示形です。",
        "data-blocks-stats-cards-grid-panel",
        PANEL_ITEMS.iter().map(panel_card).collect(),
    );

    let stepped_cards: Vec<Node> = STEPPED_ITEMS
        .iter()
        .enumerate()
        .map(|(index, item)| stepped_card(item, (index + 1) as u8))
        .collect();
    let stepped = variant_section(
        None,
        "導入社数の推移",
        "高さを段状に変えたカードで年次の伸びを示します。",
        "data-blocks-stats-cards-grid-stepped",
        stepped_cards,
    );

    div(
        vec![("class", "blocks-stats-cards-layout")],
        vec![baseline, panel, stepped],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/stats-cards/",
    title: "stats-cards",
    category: BlockCategory::Stats,
    rust_source: "crates/docs-site/src/blocks/marketing/stats/stats_cards.rs",
    demo_class: "blocks-stats-cards",
    parts: &[
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Stat",
            path: "/themes/stat/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `stats_cards` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節と同型で `pub(super)` ではなく本ファイル内
/// private 定数として `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-stats-cards-*` と `[data-blocks-stats-cards-*]` の
/// みを用い、他 block や部品の素のセレクタへ影響させない
/// （`contact_split_info` と同じ名前空間分離）。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-stats-cards` だが、`demo()` が返す
/// ルート `div` の class は `blocks-stats-cards-layout` という別名にする
/// （`contact_split_info` と同じ Bugbot 教訓の回避）。
const LAYOUT_CSS: &str = "\
.blocks-stats-cards-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
[data-blocks-stats-cards-section-head] {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-2);\n  margin-bottom: var(--fandhe-space-6);\n}\n\
[data-blocks-stats-cards-grid-baseline], [data-blocks-stats-cards-grid-panel], [data-blocks-stats-cards-grid-stepped] {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-stats-cards-card-header] {\n  display: flex;\n  align-items: flex-start;\n  justify-content: space-between;\n}\n\
[data-blocks-stats-cards-icon] {\n  color: var(--fandhe-color-accent);\n}\n\
[data-blocks-stats-cards-panel-body] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-stats-cards-icon-panel] {\n  display: inline-flex;\n  align-items: center;\n  justify-content: center;\n  width: 3.5rem;\n  height: 3.5rem;\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-subtle);\n  color: var(--fandhe-color-accent);\n}\n\
@media (min-width: 48rem) {\n  [data-blocks-stats-cards-section-head] {\n    align-items: center;\n    text-align: center;\n    max-width: 36rem;\n    margin-inline: auto;\n    margin-bottom: var(--fandhe-space-8);\n  }\n\
  [data-blocks-stats-cards-grid-baseline] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
  [data-blocks-stats-cards-grid-panel] {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n\
  [data-blocks-stats-cards-grid-stepped] {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n    align-items: end;\n  }\n\
  [data-blocks-stats-cards-step=\"1\"] {\n    min-height: 9rem;\n  }\n\
  [data-blocks-stats-cards-step=\"2\"] {\n    min-height: 12rem;\n  }\n\
  [data-blocks-stats-cards-step=\"3\"] {\n    min-height: 15rem;\n  }\n\
}\n\
@media (min-width: 64rem) {\n  [data-blocks-stats-cards-grid-baseline] {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（badge/heading/text/card/stat/icon）の anatomy を
    /// すべて実際に出力していることと、カード総数を固定する
    /// （`contact_split_info_composes_expected_parts` と同型）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"card\"",
            "data-scope=\"stat\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("data-blocks-stats-cards-card=\"\"").count(),
            10,
            "demo should render exactly 10 cards (4 baseline + 3 panel + 3 stepped)"
        );
        assert_eq!(
            html.matches("data-blocks-stats-cards-step=\"1\"").count(),
            1
        );
        assert_eq!(
            html.matches("data-blocks-stats-cards-step=\"2\"").count(),
            1
        );
        assert_eq!(
            html.matches("data-blocks-stats-cards-step=\"3\"").count(),
            1
        );
    }

    /// 増減 indicator（up/down）がそれぞれ 1 件以上出力されること
    /// （基準形の 4 件中、is_up=true が 3 件・false が 1 件）。
    #[test]
    fn demo_renders_up_and_down_indicators() {
        let html = render(&demo());
        assert!(html.contains("data-scope=\"stat\" data-part=\"up-indicator\""));
        assert!(html.contains("data-scope=\"stat\" data-part=\"down-indicator\""));
    }

    /// 非対話・XSS の不変条件（`crate::blocks` モジュール doc）を固定する。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in [
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "src=\"data:",
            "id=\"",
            "<script",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が `<` を含まず、想定するブレークポイント・列数を
    /// 持つこと（REQ-1: `</style>` によるスタイル脱出を防ぐ）。
    #[test]
    fn layout_css_declares_breakpoints_and_column_counts() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("repeat(4"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「ルート class を `demo_class` と別名に
    /// する理由」節の固定、`contact_split_info` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-stats-cards-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-stats-cards-layout");
    }
}
