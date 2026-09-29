# profile-detail-skills

`avatar` / `badge` / `status` / `stat` / `heading` / `text` / `list` /
`icon` の 8 部品を合成した、統計値とスキルを持つプロフィール詳細の合成例
です。見出しにアバター・名前・肩書・所在地・オンライン状態を置き、本文に
単価・評価・実績の統計値 → 自己紹介 → スキル（バッジ群 + チェック付き
2 列リスト）を縦に並べます。狭いコンテナ幅ではスキルの 2 列リストと統計
3 列をそれぞれ 1 列に畳みます。

代表構成（見出しに「評価」「上位認定」バッジを添えたフル構成）と中量版
（見出しバッジ・チェックリストを持たない構成）の 2 インスタンスを並記し
ます。人名・肩書・所在地・数値はすべて独自に書いた架空データです。

いずれも `<form>` を持たず、送信処理・状態機械のない静的な表示例です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::status::{self, StatusProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の単純な幾何アイコン（開いた線分パスのアウトライン描画、
/// `page_heading_meta::geo_icon` と同型）。装飾用途のみのため
/// `IconProps::default()`（`label: None`、`aria-hidden`）を使う。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
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

/// 所在地ピンアイコン（塗りつぶし、`meta_item` 用の位置情報アイコン）。
fn pin_icon() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                (
                    "d",
                    "M12 21s7-7.5 7-12a7 7 0 10-14 0c0 4.5 7 12 7 12zm0-9a3 3 0 110-6 3 3 0 010 6z",
                ),
                ("fill", "currentColor"),
            ],
            vec![],
        )],
    )
}

/// チェックマークアイコン（`list::indicator` の子として使う装飾）。
fn check_icon() -> Node {
    geo_icon("M5 13l4 4L19 7")
}

/// 見出し（アバター・名前・肩書・所在地・オンライン状態 + 任意の見出し
/// バッジ群）。`header_badges` が空なら R0220（中量版）相当、非空なら
/// R0222（代表構成）相当になる。
fn profile_header(name: &str, title: &str, location: &str, header_badges: Vec<Node>) -> Node {
    let identity_meta = div(
        vec![("class", "blocks-profile-detail-skills-identity-meta")],
        std::iter::once(styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(title)],
        ))
        .chain(header_badges)
        .collect(),
    );

    let location_row = div(
        vec![("class", "blocks-profile-detail-skills-location")],
        vec![pin_icon(), text(location)],
    );

    let online_status = status::root(
        &StatusProps {
            palette: ColorPalette::Success,
            ..StatusProps::default()
        },
        vec![],
        vec![status::indicator(vec![]), text("オンライン")],
    );

    div(
        vec![("class", "blocks-profile-detail-skills-header")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Xl,
                    ..AvatarProps::default()
                },
                vec![],
                vec![avatar::image(
                    ImageStatus::Loaded,
                    dummy_assets::AVATAR_SRC,
                    name,
                    vec![],
                )],
            ),
            div(
                vec![("class", "blocks-profile-detail-skills-identity")],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Lg,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text(name)],
                    ),
                    identity_meta,
                    location_row,
                    online_status,
                ],
            ),
        ],
    )
}

/// 統計値 1 件（`stat::root`(`<dl>`) + `label`(`<dt>`)/`value_text`(`<dd>`)）。
///
/// `stat::help_text`（`<span>`）は `stat::root`（`<dl>`）の直下へ置くと
/// `<dt>`/`<dd>` のみを許容する定義リストとして不正になるため、
/// `value_text`（`<dd>`）の内側へ入れ子にする（`chart_metric_area.rs`
/// 「`<dl>` 直下に `<span>` を置かない」節と同型の是正、PR #3390 Cursor
/// Bugbot Medium 指摘）。数値＋単位は baseline 揃えの内側 `span` へ包み、
/// `value_text` 自身は `LAYOUT_CSS` の override で縦積み（値＋単位の行、
/// help-text の行）へ切り替える。
fn stat_card(label: &str, value: &str, unit: Option<&str>, help: &str) -> Node {
    let mut value_children = vec![text(value)];
    if let Some(unit) = unit {
        value_children.push(stat::value_unit(vec![], vec![text(unit)]));
    }

    let children = vec![
        stat::label(vec![], vec![text(label)]),
        stat::value_text(
            vec![],
            vec![
                el(
                    "span",
                    vec![("class", "blocks-profile-detail-skills-stat-value")],
                    value_children,
                ),
                stat::help_text(vec![], vec![text(help)]),
            ],
        ),
    ];

    stat::root(
        Size::Md,
        vec![("data-blocks-profile-detail-skills-stat", "")],
        children,
    )
}

/// 自己紹介（見出し + 本文段落）。
fn intro(paragraph: &str) -> Node {
    div(
        vec![("class", "blocks-profile-detail-skills-section")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text("自己紹介")],
            ),
            styled_text::text(&TextProps::default(), vec![], vec![text(paragraph)]),
        ],
    )
}

/// スキルバッジ群（見出し + `badge` 列挙）。
fn skill_badges(names: &[&str]) -> Node {
    div(
        vec![("class", "blocks-profile-detail-skills-section")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text("スキル")],
            ),
            el(
                "div",
                vec![("class", "blocks-profile-detail-skills-badges")],
                names
                    .iter()
                    .map(|name| {
                        badge::badge(
                            &BadgeProps {
                                variant: BadgeVariant::Subtle,
                                ..BadgeProps::default()
                            },
                            vec![],
                            vec![text(*name)],
                        )
                    })
                    .collect(),
            ),
        ],
    )
}

/// スキルのチェック付き 2 列リスト（狭幅で 1 列に畳む、[`LAYOUT_CSS`]
/// 参照）。
fn skill_checklist(names: &[&str]) -> Node {
    list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![("data-blocks-profile-detail-skills-list", "")],
        names
            .iter()
            .map(|name| {
                list::item(
                    vec![],
                    vec![list::indicator(vec![], vec![check_icon()]), text(*name)],
                )
            })
            .collect(),
    )
}

/// 版 A（R0222 主参照）: 見出しバッジ（評価・上位認定）付き代表構成。
/// 統計 3 件（単価・評価・実績）→ 自己紹介 → スキルバッジ → チェック付き
/// 2 列リストの順。
fn version_representative() -> Node {
    let header_badges = vec![
        badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Solid,
                palette: ColorPalette::Warning,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("評価 4.9")],
        ),
        badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Outline,
                palette: ColorPalette::Accent,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("上位認定")],
        ),
    ];

    div(
        vec![("class", "blocks-profile-detail-skills-version")],
        vec![
            profile_header(
                dummy_assets::PERSON_NAMES[0],
                dummy_assets::JOB_TITLES[0],
                "東京都渋谷区",
                header_badges,
            ),
            div(
                vec![("class", "blocks-profile-detail-skills-stats")],
                vec![
                    stat_card("単価", "8,500", Some("円 / 時"), "全国平均比 +12%"),
                    stat_card("評価", "4.9", Some("/ 5.0"), "レビュー 128 件"),
                    stat_card("実績", "312", Some("件"), "直近 12 か月"),
                ],
            ),
            intro(
                "10 年以上にわたり Web フロントエンドの設計・実装に携わってきました。\
                 チーム開発でのコードレビューやドキュメント整備も得意としています。",
            ),
            skill_badges(&["TypeScript", "React", "Rust", "GraphQL", "CI/CD"]),
            skill_checklist(&[
                "コンポーネント設計",
                "アクセシビリティ対応",
                "パフォーマンス最適化",
                "チームリード経験",
            ]),
        ],
    )
}

/// 版 B（R0220 集約元）: 中量版。見出しバッジ・チェックリストを持たず、
/// 統計 3 件 → 自己紹介 1 段落 → スキルバッジのみ。
fn version_compact() -> Node {
    div(
        vec![("class", "blocks-profile-detail-skills-version")],
        vec![
            profile_header(
                dummy_assets::PERSON_NAMES[1],
                dummy_assets::JOB_TITLES[1],
                "大阪府大阪市",
                vec![],
            ),
            div(
                vec![("class", "blocks-profile-detail-skills-stats")],
                vec![
                    stat_card("単価", "6,200", Some("円 / 時"), "全国平均相当"),
                    stat_card("評価", "4.6", Some("/ 5.0"), "レビュー 54 件"),
                    stat_card("実績", "97", Some("件"), "直近 12 か月"),
                ],
            ),
            intro("プロダクト開発全般の設計・実装を担当しています。要件整理から実装まで一貫して対応可能です。"),
            skill_badges(&["Vue", "Node.js", "テスト設計"]),
        ],
    )
}

/// `profile-detail-skills` の Demo 本体（代表構成 + 中量版を並記）。
/// 呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-profile-detail-skills-stack")],
        vec![version_representative(), version_compact()],
    )
}
```

## 原案差分メモ

- 版 A（R0222 主参照）は見出しに「評価 4.9」「上位認定」バッジを添えた
  代表構成で、統計 3 件 → 自己紹介 → スキルバッジ → チェック付き 2 列
  リストの順に並べます。
- 版 B（R0220 集約元）は見出しバッジ・チェックリストを省いた中量版で、
  統計 3 件 → 自己紹介 1 段落 → スキルバッジのみです。
- 2 列 → 1 列の切り替えは `@container`（36rem）で行い、統計 3 列も同時に
  1 列へ畳みます。
- チェックマーク・所在地ピンのアイコンは自作の線画です。チェックマークは
  `list::indicator`（`aria-hidden`）による装飾で、意味は項目テキストが
  担います。
