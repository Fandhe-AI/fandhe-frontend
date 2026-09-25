# error-page-popular-links

`empty-state` / `heading` / `text` / `item` / `list` / `icon` / `link` /
`separator` の合成例（既存部品のみで組んだ、人気ページ一覧付きの 404
ページです）。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R1105。出典の固有名・ファイル名は記載しません）。

上部にロゴ、中央にエラーコード・見出し・説明文、その下に人気ページの一覧
（アイコンタイル・タイトル・説明・右向きシェブロンからなる、行全体が
リンクの一覧）、戻るリンク、最下部に著作権表記と SNS リンクの footer を
並べます。リンク先は実際の利用時は自分のページ URL へ差し替えることを
前提としています。狭い幅でも一覧は 1 列のまま幅いっぱいに広がり、footer
は縦積みへ切り替わります。本 Demo は静的な表示例であり、`<form>` 要素を
一切持たず、遷移処理・送信処理を行いません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, footer, text, Node};
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps, EmptyStateVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::item::{self, ItemMediaVariant, ItemRootProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::{Orientation, Size};

/// 自リポジトリの実在 URL（モジュール doc「href の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
const REPO_ISSUES: &str = "https://github.com/Fandhe-AI/fandhe-frontend/issues";
const REPO_RELEASES: &str = "https://github.com/Fandhe-AI/fandhe-frontend/releases";

const ROOT_CLASS: &str = "blocks-error-page-popular-links-root";
const LOGO_CLASS: &str = "blocks-error-page-popular-links-logo";
const CONTENT_CLASS: &str = "blocks-error-page-popular-links-content";
const POPULAR_CLASS: &str = "blocks-error-page-popular-links-popular";
const ROW_CLASS: &str = "blocks-error-page-popular-links-row";
const BACK_CLASS: &str = "blocks-error-page-popular-links-back";
const FOOTER_CLASS: &str = "blocks-error-page-popular-links-footer";
const FOOTER_ROW_CLASS: &str = "blocks-error-page-popular-links-footer-row";
const SOCIALS_CLASS: &str = "blocks-error-page-popular-links-socials";

const MESSAGE_ATTR: &str = "data-blocks-error-page-popular-links-message";
const CODE_ATTR: &str = "data-blocks-error-page-popular-links-code";
const TITLE_ATTR: &str = "data-blocks-error-page-popular-links-title";
const DESCRIPTION_ATTR: &str = "data-blocks-error-page-popular-links-description";
const POPULAR_HEADING_ATTR: &str = "data-blocks-error-page-popular-links-popular-heading";
const LIST_ATTR: &str = "data-blocks-error-page-popular-links-list";
const ITEM_ATTR: &str = "data-blocks-error-page-popular-links-item";
const TILE_ATTR: &str = "data-blocks-error-page-popular-links-tile";
const CHEVRON_ATTR: &str = "data-blocks-error-page-popular-links-chevron";
const BACK_ATTR: &str = "data-blocks-error-page-popular-links-back";
const FOOTER_RULE_ATTR: &str = "data-blocks-error-page-popular-links-footer-rule";
const COPYRIGHT_ATTR: &str = "data-blocks-error-page-popular-links-copyright";
const FOOTER_DIVIDER_ATTR: &str = "data-blocks-error-page-popular-links-footer-divider";
const SOCIAL_ATTR: &str = "data-blocks-error-page-popular-links-social";

/// 人気ページ 1 行分のデータ（モジュール doc「href の方針」節参照。
/// `icon_path_d` は `stroke` 系の自作幾何アイコンの `d` 属性値）。
struct PopularPage {
    href: &'static str,
    title: &'static str,
    description: &'static str,
    icon_path_d: &'static str,
}

/// 人気ページ一覧（サイト内に実在する索引ページのみを指す）。
const POPULAR_PAGES: [PopularPage; 4] = [
    PopularPage {
        href: "../../guides/",
        title: "Guides",
        description: "Step-by-step guides for building with the framework.",
        icon_path_d: "M4 4h12v16H4zM8 8h4M8 12h4",
    },
    PopularPage {
        href: "../../api/",
        title: "API Reference",
        description: "Detailed reference for every public API surface.",
        icon_path_d: "M4 6h16M4 12h16M4 18h10",
    },
    PopularPage {
        href: "../../examples/",
        title: "Examples",
        description: "Full example projects you can run and adapt.",
        icon_path_d: "M12 3l2.5 5.5L20 9l-4 4 1 6-5-3-5 3 1-6-4-4 5.5-.5z",
    },
    PopularPage {
        href: "../../themes/",
        title: "Themes",
        description: "Pre-styled components ready to drop into your app.",
        icon_path_d: "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
    },
];

/// 線画（stroke）の自作幾何アイコンを組み立てる（
/// [`crate::blocks::marketing::cta::cta_feature_links::geo_icon`] と同型の
/// パターン。装飾用途のため `IconProps::label` は付けない）。
fn stroke_icon(path_d: &'static str) -> Node {
    icon::icon(
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

/// 人気ページ行の右端に置くシェブロンアイコン（全行共通、装飾用途）。
fn chevron_icon() -> Node {
    stroke_icon("m9 6 6 6-6 6")
}

/// ロゴ（識別要素、モジュール doc「ロゴをリンクにしない理由」節参照）。
/// リンクにはせず `role="img"` + ダミー社名のアクセシブルネームのみを
/// 付与する。
fn logo_mark() -> Node {
    icon::icon(
        &IconProps {
            size: Size::Lg,
            label: Some(dummy_assets::COMPANY_NAMES[0]),
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", "M12 3 21 12 12 21 3 12Z")], vec![])],
    )
}

/// SNS リンク用の塗り面（filled）アイコン。`label` はアクセシブルネーム
/// （モジュール doc「アイコンは自作の抽象幾何図形」節参照）。
fn social_icon(label: &'static str, shape: Node) -> Node {
    icon::icon(
        &IconProps {
            label: Some(label),
            ..IconProps::default()
        },
        vec![],
        vec![shape],
    )
}

/// 人気ページ 1 行分を組み立てる（`list::item` の直接の子として
/// `item::root` を置き、行全体をリンクにする。モジュール doc「行全体が
/// リンクになる仕組み」節参照）。
fn popular_row(page: &PopularPage) -> Node {
    list::item(
        vec![("class", ROW_CLASS)],
        vec![item::root(
            ItemRootProps {
                href: Some(page.href),
                ..ItemRootProps::default()
            },
            vec![(ITEM_ATTR, "")],
            vec![
                item::media(
                    ItemMediaVariant::Icon,
                    vec![(TILE_ATTR, "")],
                    vec![stroke_icon(page.icon_path_d)],
                ),
                item::content(
                    vec![],
                    vec![
                        item::title(vec![], vec![text(page.title)]),
                        item::description(vec![], vec![text(page.description)]),
                    ],
                ),
                item::actions(vec![(CHEVRON_ATTR, "")], vec![chevron_icon()]),
            ],
        )],
    )
}

/// `error-page-popular-links` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。
#[must_use]
pub fn demo() -> Node {
    let logo = div(vec![("class", LOGO_CLASS)], vec![logo_mark()]);

    let code = styled_text::text(
        &TextProps {
            weight: TextWeight::Semibold,
            ..TextProps::default()
        },
        vec![(CODE_ATTR, "")],
        vec![text("404")],
    );

    let title = empty_state::title(
        vec![],
        vec![heading::heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl3,
                ..HeadingProps::default()
            },
            vec![(TITLE_ATTR, "")],
            vec![text("Page not found")],
        )],
    );

    let description = empty_state::description(
        vec![],
        vec![styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![(DESCRIPTION_ATTR, "")],
            vec![text(
                "We couldn't find the page you were looking for. \
                 Try one of the popular pages below instead.",
            )],
        )],
    );

    let message = empty_state::root(
        &EmptyStateProps {
            variant: EmptyStateVariant::Plain,
            ..EmptyStateProps::default()
        },
        vec![(MESSAGE_ATTR, "")],
        vec![empty_state::content(
            vec![("class", CONTENT_CLASS)],
            vec![code, title, description],
        )],
    );

    let popular_heading = heading::heading(
        HeadingLevel::H4,
        &HeadingProps {
            size: HeadingSize::Sm,
            ..HeadingProps::default()
        },
        vec![(POPULAR_HEADING_ATTR, "")],
        vec![text("Popular pages")],
    );

    let rows: Vec<Node> = POPULAR_PAGES.iter().map(popular_row).collect();

    let list = list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![(LIST_ATTR, "")],
        rows,
    );

    let popular = div(vec![("class", POPULAR_CLASS)], vec![popular_heading, list]);

    let back = div(
        vec![("class", BACK_CLASS)],
        vec![link::root(
            "../../",
            &LinkProps::default(),
            vec![(BACK_ATTR, "")],
            vec![text("← Back to home")],
        )],
    );

    let footer_rule =
        separator::separator(&SeparatorProps::default(), vec![(FOOTER_RULE_ATTR, "")]);

    let copyright = styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![(COPYRIGHT_ATTR, "")],
        vec![text(format!(
            "© 2026 {}. All rights reserved.",
            dummy_assets::COMPANY_NAMES[0]
        ))],
    );

    let footer_divider = separator::separator(
        &SeparatorProps {
            orientation: Orientation::Vertical,
            ..SeparatorProps::default()
        },
        vec![(FOOTER_DIVIDER_ATTR, "")],
    );

    let socials = div(
        vec![("class", SOCIALS_CLASS)],
        vec![
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![(SOCIAL_ATTR, "")],
                vec![social_icon(
                    "GitHub",
                    el(
                        "circle",
                        vec![("cx", "12"), ("cy", "12"), ("r", "8")],
                        vec![],
                    ),
                )],
            ),
            link::root(
                REPO_ISSUES,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![(SOCIAL_ATTR, "")],
                vec![social_icon(
                    "Issues",
                    el(
                        "rect",
                        vec![
                            ("x", "5"),
                            ("y", "5"),
                            ("width", "14"),
                            ("height", "14"),
                            ("rx", "3"),
                        ],
                        vec![],
                    ),
                )],
            ),
            link::root(
                REPO_RELEASES,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![(SOCIAL_ATTR, "")],
                vec![social_icon(
                    "Releases",
                    el("polygon", vec![("points", "12,4 20,20 4,20")], vec![]),
                )],
            ),
        ],
    );

    let footer_row = div(
        vec![("class", FOOTER_ROW_CLASS)],
        vec![copyright, footer_divider, socials],
    );

    let page_footer = footer(vec![("class", FOOTER_CLASS)], vec![footer_rule, footer_row]);

    div(
        vec![("class", ROOT_CLASS)],
        vec![logo, message, popular, back, page_footer],
    )
}
```

## 原案差分メモ

参照（対応表 ID R1105。出典の固有名・ファイル名は記載しません）からの
意図的な差分は次のとおりです。

- SNS アイコンは実在サービスのロゴ形状を持ち込まず、汎用の幾何アイコン
  （丸・角丸四角・三角）と汎用ラベルにしました（実ブランド・商標を持ち
  込まない方針。`signup-05` のプロバイダ一般化と同じ判断）。
- ロゴはリンクにせず、`icon`（`role="img"`）でアクセシブルネームのみを
  与える装飾要素として表示しました（リンク先の無い `href="#"` を使わない
  方針の帰結）。
- 人気ページの見出しは、画面上に見えない見出しではなく表示される小見出し
  （`Popular pages`）にしました。
- 行リンクと SNS リンクは、リンク先の無い `href="#"` ではなく、サイト内の
  実在ページまたは自リポジトリの実在 URL を指します。
- 参照元は全画面の高さで表示していますが、本 block は Blocks セクションの
  Demo 枠内に収める必要があるため、`min-height` による枠内表示へ変更して
  います。
- 著作権表記の社名は共通ダミー素材 `dummy_assets::COMPANY_NAMES` から
  取りました。
- 行間の区切り線は `ul` の直下に `hr` を置けないため CSS の境界線で描き、
  `separator` は footer 上端の罫線と、著作権表記と SNS の間の縦線で使い
  ました。
- 集約元は R1105 の 1 件のみで、他の対応表 ID との差分統合は行っていませ
  ん。
