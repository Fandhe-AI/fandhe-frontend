# section-heading-stats

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `link` / `stat` の
4 部品のみを合成した、統計とリンク列付きの見出しの合成例です。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（構造の参照元は
対応表 ID R0993 の 1 件です。出典の固有名・ファイル名は記載しません）。

上から順に、大見出し・リード文・矢印付きリンク 4 点の列・数値指標 4 件を
縦に並べています。背景画像は使わず、テーマの面色トークンで層を見せる
静的な合成例です（本 block はそもそも開閉状態を持ちません）。`md`
（768px）未満の画面幅では、リンクと指標を 1〜2 列に畳みます。

リンクの遷移先はすべて本リポジトリの固定 URL（リポジトリトップ・Issue
一覧・Pull Requests・リリース履歴）で、可視テキストへ遷移先が分かる文言を
付けています。数値指標（稼働率・平均対応時間・導入チーム数・満足度）は
すべて説明用の架空値であり、実企業名・PII・実データは含みません。`<form>`
要素は出力せず、送信処理・入力値検証は一切持ちません（
`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポー
ネント層はアプリケーションロジックを内包しません）。

## Rust コード

```rust
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク列 1 件分のダミーデータ（実在する本リポジトリ配下の URL のみを
/// 使う。架空の遷移先は用意しない）。
struct LinkItem {
    label: &'static str,
    href: &'static str,
}

/// リンク列（4 件、可視テキストが相異なるため `aria-label` は付与しない）。
const LINKS: [LinkItem; 4] = [
    LinkItem {
        label: "リポジトリを見る",
        href: REPO,
    },
    LinkItem {
        label: "Issue 一覧",
        href: "https://github.com/Fandhe-AI/fandhe-frontend/issues",
    },
    LinkItem {
        label: "Pull Requests",
        href: "https://github.com/Fandhe-AI/fandhe-frontend/pulls",
    },
    LinkItem {
        label: "リリース履歴",
        href: "https://github.com/Fandhe-AI/fandhe-frontend/releases",
    },
];

/// 数値指標 1 件分のダミーデータ（架空、実データ・実企業とは無関係）。
struct StatItem {
    label: &'static str,
    value: &'static str,
}

/// 数値指標一覧（架空、4 件）。
const STATS: [StatItem; 4] = [
    StatItem {
        label: "稼働率",
        value: "99.9%",
    },
    StatItem {
        label: "平均対応時間",
        value: "12 分",
    },
    StatItem {
        label: "導入チーム数",
        value: "480",
    },
    StatItem {
        label: "満足度",
        value: "4.8 / 5",
    },
];

/// 矢印付きリンク 1 件（可視テキスト + `aria-hidden` の矢印文字）。
fn arrow_link(item: &LinkItem) -> Node {
    el(
        "li",
        vec![],
        vec![link::root(
            item.href,
            &LinkProps {
                external: true,
                palette: ColorPalette::Accent,
                ..LinkProps::default()
            },
            vec![("data-blocks-section-heading-stats-link", "")],
            vec![
                text(item.label),
                el("span", vec![("aria-hidden", "true")], vec![text(" →")]),
            ],
        )],
    )
}

/// 数値指標 1 件（`stat::root` + `label`/`value_text`）。
fn stat_item(item: &StatItem) -> Node {
    stat::root(
        Size::Lg,
        vec![("data-blocks-section-heading-stats-stat", "")],
        vec![
            stat::label(vec![], vec![text(item.label)]),
            stat::value_text(vec![], vec![text(item.value)]),
        ],
    )
}

/// `section-heading-stats` の Demo 本体（大見出し + リード文 + 矢印付き
/// リンク 4 点 + 数値指標 4 件）。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let intro = div(
        vec![("class", "blocks-section-heading-stats-intro")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("開発の歩みを数字で見る")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "リポジトリの活動状況と主要な導線をまとめました。数値はすべて説明用の架空値です。",
                )],
            ),
        ],
    );

    let links = el(
        "ul",
        vec![("class", "blocks-section-heading-stats-links")],
        LINKS.iter().map(arrow_link).collect(),
    );

    let stats = div(
        vec![("class", "blocks-section-heading-stats-stats")],
        STATS.iter().map(stat_item).collect(),
    );

    div(
        vec![("class", "blocks-section-heading-stats-layout")],
        vec![intro, links, stats],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0993。出典の固有名・ファイル名は記載しません）から取り
込んだのは構造（大見出し・リード文・リンク列・数値指標という領域配置と
部品構成）のみであり、次の点を独自に設計・変更しています。

- 背景画像は持ち込まず、テーマの面色トークン（`--fandhe-color-bg-muted`）
  で層を見せる構成にしました。
- 矢印アイコンは `icon` 部品を使わず、`aria-hidden="true"` を付けた文字
  「→」で表しました（使用部品を `heading`/`text`/`link`/`stat` の 4 つに
  限定するため）。
- リンクの可視テキスト・遷移先は独自に決め、本リポジトリの固定 URL
  （リポジトリトップ・Issue 一覧・Pull Requests・リリース履歴）のみを
  使いました。架空の遷移先は作らず、`href="#"` も使いません。
- 数値指標の項目・数値はすべて独自に書いた架空のものにしました（稼働率・
  平均対応時間・導入チーム数・満足度）。
- レイアウトの畳み方を「1 列 → `sm`（640px）以上でリンク列 2 列 →
  `md`（768px）以上でリンク・指標とも 4 列」にしました。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Link](../themes/link.md) / [Stat](../themes/stat.md)
