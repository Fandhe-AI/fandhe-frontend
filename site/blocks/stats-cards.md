# stats-cards

`fandhe-frontend-pre-styled-ui` の `badge` / `heading` / `text` / `card` /
`stat` / `icon` の 6 部品のみを合成した、カード型の数値指標です。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0705、集約元は対応表 ID R0338/R0341/R1300/R1304/R1309/R1310 の 6 件です。
出典の固有名・ファイル名は記載しません）。

見出しの下にカードを 3〜4 枚グリッドで並べる形を、集約元との差分に
合わせて 3 変種で並記しています。

1. **基準形**: 中央寄せの見出し（eyebrow badge + heading + 説明文）の下に
   カード 4 枚。各カードは装飾アイコン + 増減 badge（上昇/下降の
   indicator）+ 数値指標（label/value/help）+ 下部の補足テキスト帯を
   持ちます。`md`（768px）未満は 1 列、`md` 以上で 2 列、`lg`（1024px）
   以上で 4 列になります。
2. **淡色パネル + 大アイコン**: 左寄せ見出しの下にカード 3 枚。淡色の
   カードの中に大きな装飾アイコンを面色パネルへ収め、数値指標を並べます。
   `md` 以上で 3 列になります。
3. **段状**: カード 3 枚の高さを `md` 以上でのみ段階的に変え、年次の伸びを
   段差で示します。`md` 未満は 1 列で段差をなくします。

色違いの装飾は使わず、テーマの面色（Outline/Subtle/Elevated の Card
variant・`--fandhe-*` トークン）のみで見せています。増減表示は矢印だけに
頼らず、隣接するテキスト（`+12%` 等）でも変化率が伝わるようにしています。
数値・ラベル・説明文はすべて独自に書いた架空のものです。

## Rust コード

```rust
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
```

## 原案差分メモ

参照（主参照は対応表 ID R0705、集約元は対応表 ID R0338/R0341/R1300/R1304/
R1309/R1310。出典の固有名・ファイル名は記載しません）から取り込んだのは
構造（領域の配置と部品構成）のみであり、次の点を独自に設計・変更して
います。

- 6 件の集約元を 1 つの Demo へ統合するため、3 変種の並記に整理しました。
  基準形（R0705/R1300/R1309/R1310）・淡色パネル + 大アイコン
  （R0338/R0341）・段状（R1304）です。
- R1310 の「下部リンク帯」は使用部品に `link` が含まれないため、本物の
  リンクにはせずカード footer のテキスト帯へ置き換えました。
- 増減表示（up/down indicator）は `aria-hidden` の装飾として扱い、変化率
  は隣接する可視テキスト（`+12%` 等）で伝えています（色や矢印だけに
  頼らない）。
- 段状（R1304）は「色違いは持ち込まない」というイシュー要件に合わせ、
  `CardVariant::Elevated` 1 種類に統一し、`md`（768px）未満では段差を
  なくして 1 列にしています。
- カード内の項目名は `stat::label`（`<dt>`）で表し、`card::title`
  （`<h3>` 固定）は使っていません（変種見出し `H3` と階層が重なるため）。
- 数値・ラベル・説明文はすべて独自に書いた架空のものです（実企業名・実
  データは含みません）。
