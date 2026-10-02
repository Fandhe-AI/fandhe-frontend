//! `profile-detail-skills` block（イシュー #2938。Application/Profile
//! カテゴリ、親トラッキング #2892「Blocks 目的別パーツ拡充」
//! 配下）。見出し（アバター・名前・肩書・所在地・オンライン状態）+ 統計値
//! （単価・評価・実績）+ 自己紹介 + スキル（バッジ群・チェック付き 2 列
//! リスト）から成るプロフィール詳細の合成例。対応表 ID R0222（見出しバッジ
//! 〔評価・上位認定〕付きの代表構成、主参照）と R0220（統計 + スキル
//! バッジのみの中量版、集約元）を 2 版として `demo()` へ並記する。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`profile_detail_datalist`〔#2937〕・`list_title_meta`〔#2925〕と同じ
//! 扱い）。
//!
//! # 使用部品
//!
//! `avatar` / `badge` / `status` / `stat` / `heading` / `text` / `list` /
//! `icon` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # `class` と `data-*` の使い分け
//!
//! `avatar::root` / `status::root` / `list::root` / `badge` / `text` /
//! `heading` は `drop_class_attr` により呼び出し側 `class` を除去するため、
//! これらの CSS フックは `data-blocks-profile-detail-skills-*` 属性で渡す。
//! 素の `div` ラッパーには `class="blocks-profile-detail-skills-*"` を使う
//! （`section_heading_stats`/`page_heading_meta` と同じ判断軸）。
//!
//! # `@container` で 2 列 → 1 列にする理由
//!
//! Demo 枠の幅はビューポート幅と一致しないため `@media` ではなく
//! `@container`（`collapsible_height_animation`/`form_layout_two_column` と
//! 同型）を使う。`.blocks-profile-detail-skills-stack` へ
//! `container-type: inline-size; container-name: blocks-profile-detail-skills;`
//! を宣言し、`(max-width: 36rem)` でスキルの 2 列チェックリストと統計 3 列を
//! それぞれ 1 列へ畳む。
//!
//! # `list::indicator` を装飾として使う根拠
//!
//! `list::indicator` は常に `aria-hidden="true"`（`crate::list` の fail-closed
//! 仕様）。チェックリストの全項目が「保有しているスキル」で一律のチェック
//! マークであり、意味（何のスキルか）は項目テキストがすでに担っているため、
//! 印は装飾として扱ってよい（`comparison_cards` の「可否」列がチェック
//! マークの有無で情報を運ぶのとは異なる。あちらは意味を持つため使わない
//! 判断だった一方、本 block は全項目同一装飾のため使用可）。
//!
//! # `<form>` を持たない・静的表示
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である（開閉状態も持たない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/profile-detail-skills/",
    title: "profile-detail-skills",
    category: BlockCategory::Profile,
    rust_source: "crates/docs-site/src/blocks/application/profile/profile_detail_skills.rs",
    demo_class: "blocks-profile-detail-skills",
    parts: &[
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Status",
            path: "/themes/status/",
        },
        Part {
            label: "Stat",
            path: "/themes/stat/",
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
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `profile_detail_skills` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節と同型で `pub(super)` ではなく本ファイル
/// 内 private 定数として `super::stylesheet` 経由の `push_css` で連結
/// される）。
///
/// セレクタは `.blocks-profile-detail-skills-*` と
/// `[data-blocks-profile-detail-skills-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない。`list` の size/variant 詳細度に勝つため
/// チェックリストのグリッド宣言は `data-scope`/`data-part` を前置する。
const LAYOUT_CSS: &str = "\
.blocks-profile-detail-skills-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  container-type: inline-size;\n  container-name: blocks-profile-detail-skills;\n}\n\
.blocks-profile-detail-skills-version {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-profile-detail-skills-header {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-profile-detail-skills-identity {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-profile-detail-skills-identity-meta {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-profile-detail-skills-location {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-profile-detail-skills-stats {\n  display: grid;\n  grid-template-columns: repeat(3, minmax(0, 1fr));\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-profile-detail-skills-stat] [data-part=\"value-text\"] {\n  flex-direction: column;\n  align-items: flex-start;\n}\n\
.blocks-profile-detail-skills-stat-value {\n  display: flex;\n  align-items: baseline;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-profile-detail-skills-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-profile-detail-skills-badges {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"list\"][data-part=\"root\"][data-blocks-profile-detail-skills-list] {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-2) var(--fandhe-space-6);\n}\n\
[data-scope=\"list\"][data-part=\"root\"].fd-list--variant-plain[data-blocks-profile-detail-skills-list] > [data-scope=\"list\"][data-part=\"item\"] {\n  display: flex;\n  align-items: center;\n}\n\
@container blocks-profile-detail-skills (max-width: 36rem) {\n  \
.blocks-profile-detail-skills-stats {\n    grid-template-columns: 1fr;\n  }\n  \
[data-scope=\"list\"][data-part=\"root\"][data-blocks-profile-detail-skills-list] {\n    grid-template-columns: 1fr;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, dummy_assets, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 8 部品を出力すること、非対話制約
    /// （`<form>` 不在・`data:` URI 不在・スクリプト不在）を満たすことの
    /// 単体回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と
    /// 重複し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"avatar\"",
            "data-scope=\"badge\"",
            "data-scope=\"status\"",
            "data-scope=\"stat\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"list\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        // 統計は版ごとに 3 件、2 版で 6 件。
        assert_eq!(
            html.matches("data-scope=\"stat\" data-part=\"root\"")
                .count(),
            6
        );
        // チェック付き 2 列リストは代表構成（版 A）のみ持つ。
        assert_eq!(
            html.matches("data-blocks-profile-detail-skills-list=\"\"")
                .count(),
            1
        );
        assert!(html.contains(dummy_assets::AVATAR_SRC));
    }

    /// `<form>`・`data:` URI・`<script`・送信ボタンを含まない静的表示で
    /// あること。
    #[test]
    fn no_form_submit_or_dead_links() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"submit\""));
        assert!(html.contains(dummy_assets::AVATAR_SRC));
    }

    /// [`LAYOUT_CSS`] が `<` を含まず（XSS 回帰対象外の安全な CSS）、
    /// `@container` によるコンテナクエリで統計・チェックリストを 1 列へ
    /// 畳むこと。
    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-profile-detail-skills (max-width: 36rem)"));
    }

    /// アバター画像の `alt` に人物のフルネームが渡ること
    /// （装飾目的の空 `alt` ではなく意味のある代替テキスト）。
    #[test]
    fn avatar_image_alt_carries_full_name() {
        let html = render(&demo());
        assert!(html.contains(&format!("alt=\"{}\"", dummy_assets::PERSON_NAMES[0])));
        assert!(html.contains(&format!("alt=\"{}\"", dummy_assets::PERSON_NAMES[1])));
    }

    /// オンライン状態が色だけでなく可視テキスト「オンライン」でも
    /// 伝わること（色のみに依存しない a11y 判断）。
    #[test]
    fn status_text_is_visible() {
        let html = render(&demo());
        assert_eq!(html.matches("オンライン").count(), 2);
    }
}
