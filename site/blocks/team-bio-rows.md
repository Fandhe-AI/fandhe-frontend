# team-bio-rows

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `image` / `separator` /
`link` / `icon` 部品を合成した、写真と紹介文を横並びにするチームメンバー
一覧です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R1354。出典の固有名・ファイル名は記載しません）。

各メンバーは縦長の写真と、氏名・役職・紹介文・SNS リンクを横に並べた項目
です。Demo は配置違いの 2 variant を並記します: 見出しを左列に置き、右列を
区切り線付きの縦リストにする形（R1354、主参照）と、見出しを上に置き、
項目を 2 列グリッドで並べる形（R1355）です。どちらの variant も、狭い幅
では各項目の写真を本文の上に縦積みにします。

本 Demo は静的な表示例であり、docs サイトは JS ハイドレーションを行わない
ため、`<form>` 要素は一切持たず、データの取得・送信・状態管理を行いません。
文言・人名・役職はすべて独自に書いた架空のものであり、実在人物・実企業名・
実クレデンシャル・PII を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc 参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// 実在の自組織 URL。
const ORG: &str = "https://github.com/Fandhe-AI";

/// メンバーの紹介文（架空の日本語、検索インデックス容量対策で短くする）。
const BIOS: &[&str] = &[
    "小さく試して確かめてから広げる進め方を、チーム全体に広めています。",
    "利用者からのフィードバックを設計へ素早く反映する仕組みを作ります。",
    "運用で見えた課題を、次の開発計画へ落とし込む役割を担っています。",
    "数字の裏側にある背景を掘り下げ、次の一手を提案しています。",
];

/// SNS アイコン用の装飾幾何アイコン（`label` は呼び出し側が指定する）。
fn geo_icon(size: Size, label: &str, path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size,
            label: Some(label),
            ..IconProps::default()
        },
        vec![],
        vec![fandhe_frontend_core::el(
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

/// メンバー 1 名分の SNS リンク行（実在の自リポジトリ・自組織 URL のみ）。
fn social_links(name: &str) -> Node {
    ul(
        vec![("class", "blocks-team-bio-rows-social")],
        vec![
            li(
                vec![],
                vec![link::root(
                    REPO,
                    &LinkProps {
                        external: true,
                        ..LinkProps::default()
                    },
                    vec![],
                    vec![geo_icon(
                        Size::Sm,
                        &format!("{name} のリポジトリ"),
                        "M4 4h16v6H4zM4 14h16v6H4z",
                    )],
                )],
            ),
            li(
                vec![],
                vec![link::root(
                    ORG,
                    &LinkProps {
                        external: true,
                        ..LinkProps::default()
                    },
                    vec![],
                    vec![geo_icon(
                        Size::Sm,
                        &format!("{name} の所属組織"),
                        "M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5",
                    )],
                )],
            ),
        ],
    )
}

/// メンバー 1 名分（縦長の写真 + 氏名・役職・紹介文・SNS リンク）。
fn member(index: usize) -> Node {
    let name = dummy_assets::PERSON_NAMES[index];
    let role = dummy_assets::JOB_TITLES[index % dummy_assets::JOB_TITLES.len()];
    let bio = BIOS[index % BIOS.len()];
    div(
        vec![("data-blocks-team-bio-rows-member", "")],
        vec![
            image::image(
                &ImageProps {
                    aspect_ratio: AspectRatio::Portrait,
                    ..ImageProps::new(
                        dummy_assets::AVATAR_SRC,
                        &format!("{name} のプロフィール写真"),
                    )
                },
                vec![("data-blocks-team-bio-rows-photo", "")],
            ),
            div(
                vec![("class", "blocks-team-bio-rows-body")],
                vec![
                    heading::heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(name)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(role)],
                    ),
                    styled_text::text(&TextProps::default(), vec![], vec![text(bio)]),
                    social_links(name),
                ],
            ),
        ],
    )
}

/// 見出し左・区切り線付き縦リスト（主参照 R1354、`variant="side"`）。
fn instance_side() -> Node {
    let members: Vec<Node> = (0..3)
        .map(|i| {
            let item = member(i);
            if i == 0 {
                li(vec![], vec![item])
            } else {
                li(
                    vec![],
                    vec![
                        separator::separator(
                            &SeparatorProps::default(),
                            vec![("data-blocks-team-bio-rows-separator", "")],
                        ),
                        item,
                    ],
                )
            }
        })
        .collect();
    div(
        vec![
            ("class", "blocks-team-bio-rows-layout"),
            ("data-blocks-team-bio-rows-variant", "side"),
        ],
        vec![
            div(
                vec![("class", "blocks-team-bio-rows-intro")],
                vec![
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl2,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("チーム紹介")],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("日々の運用を支えるメンバーです。")],
                    ),
                ],
            ),
            ul(vec![("class", "blocks-team-bio-rows-list")], members),
        ],
    )
}

/// 見出し上・2 列グリッド（集約元 R1355、`variant="top"`）。
fn instance_top() -> Node {
    let members: Vec<Node> = (0..4).map(|i| li(vec![], vec![member(i)])).collect();
    div(
        vec![
            ("class", "blocks-team-bio-rows-layout"),
            ("data-blocks-team-bio-rows-variant", "top"),
        ],
        vec![
            div(
                vec![("class", "blocks-team-bio-rows-intro")],
                vec![
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl2,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("メンバー")],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("それぞれの持ち場で運用を進めています。")],
                    ),
                ],
            ),
            ul(vec![("class", "blocks-team-bio-rows-grid")], members),
        ],
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &str) -> Node {
    fandhe_frontend_core::p(
        vec![("class", "blocks-team-bio-rows-caption")],
        vec![text(label)],
    )
}

/// `team-bio-rows` の Demo 本体（2 variant 併記）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-team-bio-rows-stack")],
        vec![
            caption("見出し左・区切り線付き縦リスト"),
            instance_side(),
            caption("見出し上・2 列グリッド"),
            instance_top(),
        ],
    )
}
```

## 集約元との差分メモ

- R1354（主参照）は見出しを左列に置き、右列を区切り線付きの縦リストで並べる
  形です。本 Demo の `variant="side"` に対応します。
- R1355 は見出しを上に置き、項目を 2 列グリッドで並べる形です。本 Demo の
  `variant="top"` に対応します。
- 狭い幅ではどちらの variant も、各項目の写真を本文の上に縦積みにします。
- 文言・配色・アイコンは参照元から持ち込まず、すべて独自に書いた架空の
  ものです。SNS リンクは実在の自リポジトリ・自組織 URL に差し替えています。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Image](../themes/image.md) / [Separator](../themes/separator.md) /
[Link](../themes/link.md) / [Icon](../themes/icon.md)
