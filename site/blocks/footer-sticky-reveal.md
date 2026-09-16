# footer-sticky-reveal

`card` / `heading` / `link` / `nav_list` を合成した、Motion+
`sections/footers` の sticky reveal 相当の合成例です。

Demo 枠を固定高スクロールコンテナにし、`position: sticky` のみで footer が
本文の下から現れます（JS 不使用）。`prefers-reduced-motion: reduce` では
フェードイン強調を無効化します。リンク先は GitHub への外部 URL です。
`<form>` は使用しません。

## Rust コード

```rust
use fandhe_frontend_core::{div, footer, p, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::nav_list;

/// ダミー段落（架空）。
const PARAGRAPHS: &[&str] = &["スクロールすると footer が下から現れます。"];

/// ダミーカードの (見出し, 説明) 一覧（架空、sticky reveal を見せる高さ用）。
const CARDS: &[(&str, &str)] = &[
    ("Realtime Sync", "同期します。"),
    ("Access Control", "保護します。"),
    ("Usage Insights", "可視化します。"),
];

/// ダミーカード 1 件（架空）。
fn content_card(title: &str, description: &str) -> Node {
    card::root(
        CardProps::default(),
        vec![],
        vec![
            card::header(vec![], vec![card::title(vec![], vec![text(title)])]),
            card::body(
                vec![],
                vec![card::description(vec![], vec![text(description)])],
            ),
        ],
    )
}

/// 本文ダミー（段落 + カード群）。
fn dummy_content() -> Node {
    let mut children: Vec<Node> = PARAGRAPHS
        .iter()
        .map(|t| p(vec![], vec![text(*t)]))
        .collect();
    children.extend(CARDS.iter().map(|(t, d)| content_card(t, d)));

    div(
        vec![("data-blocks-footer-sticky-reveal-content", "")],
        children,
    )
}

/// footer リンク列（見出し + リンク群）。`heading_text` は `nav_list::heading`
/// （固定 `h2`）ではなく `p` で表現する（モジュール doc「`nav_list::heading`
/// を使わない理由」節参照）。
fn footer_nav_group(heading_text: &str, links: &[(&str, &str)]) -> Node {
    let items: Vec<Node> = links
        .iter()
        .map(|(href, label)| {
            nav_list::item(
                vec![],
                vec![nav_list::link(href, false, vec![], vec![text(*label)])],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-footer-sticky-reveal-group")],
        vec![
            p(
                vec![("class", "blocks-footer-sticky-reveal-group-title")],
                vec![text(heading_text)],
            ),
            nav_list::list(vec![], items),
        ],
    )
}

/// footer 本体（ブランド見出し + リンク群 2 種）。
fn footer_element() -> Node {
    let brand = heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Lg,
            weight: HeadingWeight::Bold,
        },
        vec![("data-blocks-footer-sticky-reveal-brand", "")],
        vec![text("Fandhe Frontend")],
    );

    const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
    let nav = nav_list::root(
        "Footer",
        vec![("data-blocks-footer-sticky-reveal-nav", "")],
        vec![
            footer_nav_group("Product", &[(REPO, "Guide"), (REPO, "API Reference")]),
            footer_nav_group("Community", &[(REPO, "Spec"), (REPO, "GitHub")]),
        ],
    );

    let bottom_link = link::root(
        REPO,
        &LinkProps::default(),
        vec![],
        vec![text("© Fandhe Frontend")],
    );

    footer(
        vec![("data-blocks-footer-sticky-reveal-footer", "")],
        vec![div(
            vec![("data-blocks-footer-sticky-reveal-footer-inner", "")],
            vec![brand, nav, bottom_link],
        )],
    )
}

/// `footer-sticky-reveal` の Demo 本体（モジュール doc「sticky reveal の
/// 実体」節参照）。
pub fn demo() -> Node {
    div(vec![], vec![dummy_content(), footer_element()])
}
```
