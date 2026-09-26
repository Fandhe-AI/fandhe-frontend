# stats-timeline

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `separator` の 3 部品
のみを合成した、日付付きの出来事（沿革）を横一列に並べる合成例です。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/
Primitives 部品を組み合わせた実例集であることに注意してください。

出来事は本リポジトリの架空の沿革として 4 件並べます。日付・題・説明は
すべて説明用の架空の内容であり、実企業名・PII・実データは含みません。
`<form>` は使いません（本 block は非対話・静的表示です）。

`timeline` 部品は縦向きの沿革表示にしか対応していないため、本 block では
`timeline` を使わず、配置は block 固有の CSS と `separator` の組み合わせで
横向きに構成しています。詳細は「原案差分メモ」を参照してください。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 出来事 1 件分のダミーデータ（架空、実企業名・PII とは無関係）。
struct EventItem {
    /// `<time datetime>` へそのまま使う ISO 8601 表記（年月）。
    date_attr: &'static str,
    /// 表示用の日付文言。
    date_label: &'static str,
    title: &'static str,
    body: &'static str,
}

/// 出来事一覧（架空、4 件、日付・題とも相異なる）。
const EVENTS: [EventItem; 4] = [
    EventItem {
        date_attr: "2023-04",
        date_label: "2023 年 4 月",
        title: "プロジェクト発足",
        body: "説明用の架空のキックオフです。最小構成の設計をまとめました。",
    },
    EventItem {
        date_attr: "2023-11",
        date_label: "2023 年 11 月",
        title: "最初の公開版",
        body: "説明用の架空の初回リリースです。中核機能を公開しました。",
    },
    EventItem {
        date_attr: "2024-07",
        date_label: "2024 年 7 月",
        title: "利用チームの拡大",
        body: "説明用の架空の拡大期です。導入チーム数が大きく伸びました。",
    },
    EventItem {
        date_attr: "2025-05",
        date_label: "2025 年 5 月",
        title: "大型刷新",
        body: "説明用の架空の刷新です。既定エスケープの製品化を完了しました。",
    },
];

/// 出来事 1 件（`<time>` + ドット/罫線 + 題 + 説明）。
fn event_item(item: &EventItem) -> Node {
    el(
        "li",
        vec![("class", "blocks-stats-timeline-item")],
        vec![
            el(
                "time",
                vec![
                    ("datetime", item.date_attr),
                    ("class", "blocks-stats-timeline-date"),
                ],
                vec![text(item.date_label)],
            ),
            div(
                vec![("class", "blocks-stats-timeline-marker")],
                vec![
                    el(
                        "span",
                        vec![
                            ("aria-hidden", "true"),
                            ("class", "blocks-stats-timeline-dot"),
                        ],
                        vec![],
                    ),
                    separator(
                        &SeparatorProps::default(),
                        vec![("data-blocks-stats-timeline-rule", "")],
                    ),
                ],
            ),
            heading(
                HeadingLevel::H4,
                &HeadingProps {
                    size: HeadingSize::Lg,
                    weight: HeadingWeight::Semibold,
                },
                vec![],
                vec![text(item.title)],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(item.body)],
            ),
        ],
    )
}

/// `stats-timeline` の Demo 本体（大見出し + リード文 + 出来事 4 件）。
/// 呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let intro = div(
        vec![("class", "blocks-stats-timeline-intro")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("プロジェクトの歩み")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("日付と出来事はすべて説明用の架空の内容です。")],
            ),
        ],
    );

    let list = el(
        "ol",
        vec![("class", "blocks-stats-timeline-list")],
        EVENTS.iter().map(event_item).collect(),
    );

    div(
        vec![("class", "blocks-stats-timeline-layout")],
        vec![intro, list],
    )
}
```

## 原案差分メモ

- **R1303（日付付きの沿革を横一列に並べる）**: 参照元は縦向きのタイムライン
  でしたが、`fandhe-frontend-pre-styled-ui::timeline` は縦向きにしか対応
  していません。本 block では `timeline` を使わず、`ol`（時系列のため
  リスト意味論に `ol` を使用）と block 固有の配置 CSS + `separator` の
  組み合わせで、要求どおりの横向き配置（`lg` 以上で 4 列）を独自に構成
  しました。
- **ドット・罫線の実現手段**: ドットは装飾のみのため `aria-hidden="true"`
  を付けた `<span>` を CSS の円（`border-radius: var(--fandhe-radius-full)`）
  として描いています。罫線は `separator` の既定（`Horizontal`/`Solid`）を
  そのまま使い、`flex: 1 1 auto` で残り幅いっぱいへ伸ばしています。
- **狭幅時の畳み方**: `lg`（64rem）以上は横 1 列（4 列）、`sm`（40rem）以上
  `lg` 未満は 2 列、`sm` 未満は縦 1 列（1 列）に畳みます。
- 参照元の文言・配色・装飾・アイコンは持ち込んでいません。日付・出来事は
  すべて架空のものです。
