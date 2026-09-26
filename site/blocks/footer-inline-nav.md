# footer-inline-nav

`icon` / `link` / `nav_list` / `separator` を合成した、1 行ナビ型 footer の
合成例です。上段にロゴ・横並びのナビリンク・SNS アイコン、区切り線を挟んで
下段に著作権表記と法務リンクを置きます。ナビと法務リンクを省いた最小形、全段を
中央寄せの縦積みにする形の 3 種を並記します。狭幅では各段を中央寄せの縦積み
にし、`48rem`（md）以上では両端揃えの横並びに切り替わります。無 JS の静的
表示で `<form>` は使用しません。リンク先はすべて GitHub への外部 URL です。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, footer, p, section, text, Node};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::nav_list;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};

/// 実在の GitHub リポジトリへの固定外部 URL（`href="#"` 等の非実在リンクを
/// 避けるため、footer 系 block 共通の判断に倣う）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 主ナビゲーションのラベルと遷移先（同一リポジトリ内の実在パス）。
/// 全リンクが `REPO` 直下へ揃うとラベルと遷移先が食い違う（Codex レビュー
/// 指摘）ため、ラベルの意味に対応する実在サブパスへ個別に張る。
const NAV_LINKS: &[(&str, &str)] = &[
    ("概要", REPO),
    (
        "ガイド",
        "https://github.com/Fandhe-AI/fandhe-frontend/tree/main/docs/guides",
    ),
    (
        "API リファレンス",
        "https://github.com/Fandhe-AI/fandhe-frontend/tree/main/docs/api",
    ),
    (
        "更新履歴",
        "https://github.com/Fandhe-AI/fandhe-frontend/releases",
    ),
];

/// 法務リンクのラベルと遷移先。本リポジトリはプライバシーポリシー等を
/// 持たないため、実在するデュアルライセンス表記（`LICENSE-MIT`/
/// `LICENSE-APACHE`）へ遷移先を合わせ、ラベルもそれに合わせて改めた
/// （Codex レビュー指摘: 表示と遷移先の食い違い是正）。
const LEGAL_LINKS: &[(&str, &str)] = &[
    (
        "MIT ライセンス",
        "https://github.com/Fandhe-AI/fandhe-frontend/blob/main/LICENSE-MIT",
    ),
    (
        "Apache ライセンス",
        "https://github.com/Fandhe-AI/fandhe-frontend/blob/main/LICENSE-APACHE",
    ),
];

/// SNS リンクのアクセシブルネーム・線画パス・遷移先（実在ブランドを模さない
/// 抽象図形: 円・三角・四角）。「動画」ラベルに対応する実在ページが本
/// リポジトリに存在しないため、ラベルを実在の GitHub Discussions/リポジトリ
/// トップへ合わせて改めた（Codex レビュー指摘）。
const SOCIAL_LINKS: &[(&str, &str, &str)] = &[
    (
        "更新情報",
        "M12 3a9 9 0 100 18 9 9 0 000-18z",
        "https://github.com/Fandhe-AI/fandhe-frontend/releases",
    ),
    (
        "コミュニティ",
        "M12 4l8 16H4z",
        "https://github.com/Fandhe-AI/fandhe-frontend/discussions",
    ),
    ("リポジトリ", "M4 4h16v16H4z", REPO),
];

/// 自作の幾何アイコン（線画）。`fill="none"` + `stroke="currentColor"` で
/// `icon` 側の既定塗り面をストロークへ上書きする（`contact_split_form_info`
/// の `geo_icon` と同型）。`label` は `None` なら装飾扱い（`aria-hidden`）、
/// `Some` なら `role="img"` + `aria-label` を付与する（[`icon`] の
/// `IconProps.label` 契約）。隣接するテキストが既に同じ名称を提供する
/// 場合（[`logo`] のロゴアイコン）はブランド名の二重読み上げを避けるため
/// `None` を渡す。
fn geo_icon(path_d: &str, label: Option<&str>) -> Node {
    icon(
        &IconProps {
            label,
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

/// ブランドロゴ（非リンクの抽象図形アイコン + ブランド名）。ロゴアイコンは
/// 直後の `p` テキストと同じブランド名を表すため、`geo_icon` へ `label:
/// None` を渡して装飾扱いにする（アクセシブルネームの二重読み上げ回避、
/// Codex レビュー指摘）。
fn logo() -> Node {
    div(
        vec![("data-blocks-footer-inline-nav-brand", "")],
        vec![
            geo_icon("M4 4h16v16H4z M9 9h6v6H9z", None),
            p(vec![], vec![text("Fandhe Frontend")]),
        ],
    )
}

/// SNS アイコンリンク群（外部リンク、アクセシブルネームは各アイコンの
/// `IconProps.label` で与える）。
fn social_links() -> Node {
    let items: Vec<Node> = SOCIAL_LINKS
        .iter()
        .map(|(label, path_d, href)| {
            link::root(
                href,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![("data-blocks-footer-inline-nav-social", "")],
                vec![geo_icon(path_d, Some(label))],
            )
        })
        .collect();
    div(vec![("data-blocks-footer-inline-nav-socials", "")], items)
}

/// 主ナビゲーション（`nav_list`、横並び）。`aria_label` は形ごとに変えて
/// ランドマークの重複を避ける。
fn primary_nav(aria_label: &'static str) -> Node {
    let items: Vec<Node> = NAV_LINKS
        .iter()
        .map(|(label, href)| {
            nav_list::item(
                vec![],
                vec![nav_list::link(href, false, vec![], vec![text(*label)])],
            )
        })
        .collect();
    nav_list::root(
        aria_label,
        vec![("data-blocks-footer-inline-nav-nav", "")],
        vec![nav_list::list(
            vec![("class", "blocks-footer-inline-nav-links")],
            items,
        )],
    )
}

/// 下段（著作権 + 任意の法務リンク 2 件）。法務リンクはランドマーク乱立を
/// 避けるため `nav_list` を使わず素の `div` に並べる。
fn bottom_row(with_legal: bool) -> Node {
    let mut children = vec![p(vec![], vec![text("© 2026 Fandhe Frontend")])];
    if with_legal {
        let legal_links: Vec<Node> = LEGAL_LINKS
            .iter()
            .map(|(label, href)| {
                link::root(
                    href,
                    &LinkProps {
                        external: true,
                        ..LinkProps::default()
                    },
                    vec![],
                    vec![text(*label)],
                )
            })
            .collect();
        children.push(div(
            vec![("data-blocks-footer-inline-nav-legal", "")],
            legal_links,
        ));
    }
    div(
        vec![(
            "class",
            "blocks-footer-inline-nav-row blocks-footer-inline-nav-bottom",
        )],
        children,
    )
}

/// 1 つの footer バリエーションを組み立てる。
fn footer_variant(
    variant: &'static str,
    align_center: bool,
    with_nav: bool,
    with_socials: bool,
    with_legal: bool,
    aria_label: &'static str,
) -> Node {
    let mut top_children = vec![logo()];
    if with_nav {
        top_children.push(primary_nav(aria_label));
    }
    if with_socials {
        top_children.push(social_links());
    }

    let mut attrs = vec![("data-blocks-footer-inline-nav-variant", variant)];
    if align_center {
        attrs.push(("data-blocks-footer-inline-nav-align", "center"));
    }

    footer(
        attrs,
        vec![
            div(
                vec![("class", "blocks-footer-inline-nav-row")],
                top_children,
            ),
            separator(&SeparatorProps::default(), vec![]),
            bottom_row(with_legal),
        ],
    )
}

/// `footer-inline-nav` の Demo 本体（モジュール doc「3 形の並記」節参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-footer-inline-nav-layout")],
        vec![
            p(
                vec![("class", "blocks-footer-inline-nav-caption")],
                vec![text("標準形")],
            ),
            section(
                vec![],
                vec![footer_variant(
                    "standard",
                    false,
                    true,
                    true,
                    true,
                    "フッターナビゲーション",
                )],
            ),
            p(
                vec![("class", "blocks-footer-inline-nav-caption")],
                vec![text("最小形")],
            ),
            section(
                vec![],
                vec![footer_variant(
                    "minimal",
                    false,
                    false,
                    true,
                    false,
                    "フッターナビゲーション",
                )],
            ),
            p(
                vec![("class", "blocks-footer-inline-nav-caption")],
                vec![text("中央寄せ")],
            ),
            section(
                vec![],
                vec![footer_variant(
                    "centered",
                    true,
                    true,
                    true,
                    true,
                    "フッターナビゲーション（中央寄せ）",
                )],
            ),
        ],
    )
}
```

## 原案差分メモ

主参照 R0493（上段に SNS アイコン列を追加した形）を標準形に採用し、
R0491（ロゴ + ナビ / 区切り線 / 著作権 + 法務リンク）を標準形の下段構成へ
統合しました。R0113（ロゴ/ナビ/SNS を 1 行・下段中央寄せ）は `48rem` 未満
での中央寄せ縦積みと、全段中央寄せの `centered` 形の両方で表現しています。
R0115（ナビなし・ロゴ + SNS の最小形）は `minimal` 形、R0966（SNS + 著作権
の 1 行のみ）は `minimal` 形の下段（法務リンクを持たない著作権のみの構成）
に吸収しました。R0965（全段中央寄せ縦積み版）は `centered` 形が対応します。
参照元の文言・配色・アイコン意匠は持ち込まず、独自の架空文言・
`--fandhe-*` トークンのみで表現しています。

## 関連リンク

- [Icon](../themes/icon.md)
- [Link](../themes/link.md)
- [NavList](../themes/nav-list.md)
- [Separator](../themes/separator.md)
