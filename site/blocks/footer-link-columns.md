# footer-link-columns

`link` / `separator` / `icon` / `heading` / `text` を合成した、定番の
リンクカラム型 footer です。上段はブランド列（ロゴマーク・名称・説明）と
カテゴリ見出し付きのリンク列 2〜5 群、下段は区切り線を挟んだ著作権表示と
SNS・法務リンクで構成します。

狭い幅ではブランド列の下にリンク列が 2 列 grid で並び、lg（64rem）以上で
全列が横 1 行に並びます。`<form>` を持たず状態を持たない静的な表示です。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, footer, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
/// 表示名と遷移先の対応が付く実在ページのみを使い、対応が付かない項目
/// （SNS アカウント・Privacy Policy・Terms of Service 等）は非表示にする
/// （Bugbot 指摘 PR #3271）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// docs サイト「Guide」の実在ページ（`site/nav.toml` `index_path = "/guides/"`）。
const GUIDE_URL: &str = "https://fandhe-ai.github.io/fandhe-frontend/guides/";
/// docs サイト「API Reference」の実在ページ（同 `index_path = "/api/"`）。
const API_REFERENCE_URL: &str = "https://fandhe-ai.github.io/fandhe-frontend/api/";
/// docs サイト「Examples」の実在ページ（同 `index_path = "/examples/"`）。
const EXAMPLES_URL: &str = "https://fandhe-ai.github.io/fandhe-frontend/examples/";
/// リポジトリのリリース一覧（「Changelog」の実在先）。
const RELEASES_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend/releases";
/// リポジトリの Discussions（「Discussions」の実在先）。
const DISCUSSIONS_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend/discussions";

/// 自作の幾何パスによる装飾アイコン（`feature_three_column_icons::
/// geo_icon` と同型の線画。`label` は呼び出し側が指定する）。
fn geo_icon(path_d: &'static str, label: Option<&'static str>) -> Node {
    icon(
        &IconProps {
            size: Size::Md,
            label,
            ..IconProps::default()
        },
        vec![("data-blocks-footer-link-columns-icon", "")],
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

/// ロゴマーク（装飾扱い、`label: None`）。
fn logo_mark() -> Node {
    geo_icon(
        "M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5",
        None,
    )
}

/// SNS アイコンのみのリンク（accessible name はリンクごと異なる固定
/// 文字列、WCAG 2.4.4）。`href` は呼び出し側が実在の遷移先を指定する
/// （モジュール doc「リンク先の方針」節参照）。
fn social_icon_link(href: &'static str, path_d: &'static str, label: &'static str) -> Node {
    link::root(
        href,
        &LinkProps {
            external: true,
            ..LinkProps::default()
        },
        vec![("data-blocks-footer-link-columns-social-link", "")],
        vec![geo_icon(path_d, Some(label))],
    )
}

/// リンク群 1 個（見出し + `ul`/`li` のリンク一覧）。各項目は
/// `(href, label)` の組で、実在する遷移先のみを渡す契約（モジュール doc
/// 「リンク先の方針」節参照）。
fn link_group(heading_text: &'static str, links: &[(&'static str, &'static str)]) -> Node {
    let items: Vec<Node> = links
        .iter()
        .map(|(href, label)| {
            li(
                vec![],
                vec![link::root(
                    href,
                    &LinkProps::default(),
                    vec![],
                    vec![text(*label)],
                )],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-footer-link-columns-group")],
        vec![
            heading::heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![("data-blocks-footer-link-columns-group-title", "")],
                vec![text(heading_text)],
            ),
            ul(vec![("class", "blocks-footer-link-columns-list")], items),
        ],
    )
}

/// ブランド列（ロゴマーク + 名称 + 説明。`wide` で B 用の 2 列幅指定を
/// 付ける）。
fn brand_column(description: &'static str, wide: bool) -> Node {
    let mut attrs = vec![("class", "blocks-footer-link-columns-brand")];
    if wide {
        attrs.push(("data-wide", ""));
    }
    div(
        attrs,
        vec![
            div(
                vec![("class", "blocks-footer-link-columns-brand-mark")],
                vec![
                    logo_mark(),
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text("Fandhe Frontend")],
                    ),
                ],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-footer-link-columns-brand-desc", "")],
                vec![text(description)],
            ),
        ],
    )
}

/// 著作権表示。
fn copyright() -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-footer-link-columns-copyright", "")],
        vec![text("\u{00a9} 2026 Fandhe Frontend")],
    )
}

/// 注記行（各インスタンスの原案差分の要約）。
fn note(body: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-footer-link-columns-note", "")],
        vec![text(body)],
    )
}

/// インスタンス A: 基準形（ブランド列 + リンク 4 群、下段に SNS アイコン
/// のみのリンク 1 個。X・Mastodon は実アカウントが無いため非表示）。
fn instance_a() -> Node {
    footer(
        vec![("class", "blocks-footer-link-columns-instance")],
        vec![
            note("基準形。ブランド列 + リンク 4 群。下段に SNS アイコンのみのリンク 1 個。"),
            div(
                vec![("class", "blocks-footer-link-columns-top")],
                vec![
                    brand_column(
                        "決定的なビルドを目指す Rust 製フロントエンドフレームワークです。",
                        false,
                    ),
                    div(
                        vec![("class", "blocks-footer-link-columns-columns")],
                        vec![
                            link_group(
                                "Product",
                                &[(GUIDE_URL, "Guide"), (API_REFERENCE_URL, "API Reference")],
                            ),
                            link_group(
                                "Resources",
                                &[(EXAMPLES_URL, "Examples"), (RELEASES_URL, "Changelog")],
                            ),
                            link_group(
                                "Community",
                                &[(REPO, "GitHub"), (DISCUSSIONS_URL, "Discussions")],
                            ),
                            link_group("Company", &[(REPO, "About")]),
                        ],
                    ),
                ],
            ),
            separator::separator(
                &SeparatorProps::default(),
                vec![("data-blocks-footer-link-columns-separator", "")],
            ),
            div(
                vec![("class", "blocks-footer-link-columns-bottom")],
                vec![
                    copyright(),
                    div(
                        vec![("class", "blocks-footer-link-columns-social")],
                        vec![social_icon_link(
                            REPO,
                            "M12 2L2 7l10 5 10-5-10-5z",
                            "GitHub",
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// インスタンス B: ブランド列を強調（2 列幅、mission 文 + 架空の連絡先 +
/// SNS アイコン）。リンクは 2 群。下段は著作権のみ（Privacy Policy・
/// Terms of Service は実在ページが無いため非表示、モジュール doc
/// 「リンク先の方針」節参照）。
fn instance_b() -> Node {
    footer(
        vec![("class", "blocks-footer-link-columns-instance")],
        vec![
            note("ブランド列を 2 列幅に広げ、mission 文 + 連絡先 + SNS アイコンを収める。リンクは 2 群。下段は著作権のみ。"),
            div(
                vec![("class", "blocks-footer-link-columns-top"), ("data-brand", "wide")],
                vec![
                    div(
                        vec![("class", "blocks-footer-link-columns-brand"), ("data-wide", "")],
                        vec![
                            div(
                                vec![("class", "blocks-footer-link-columns-brand-mark")],
                                vec![
                                    logo_mark(),
                                    heading::heading(
                                        HeadingLevel::H3,
                                        &HeadingProps::default(),
                                        vec![],
                                        vec![text("Fandhe Frontend")],
                                    ),
                                ],
                            ),
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    variant: TextVariant::Muted,
                                    ..TextProps::default()
                                },
                                vec![("data-blocks-footer-link-columns-brand-desc", "")],
                                vec![text(
                                    "AI 時代のセキュリティリスク低減を目指し、プレーンな HTML/JS/CSS を尊重するフレームワークを開発しています。",
                                )],
                            ),
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    variant: TextVariant::Muted,
                                    ..TextProps::default()
                                },
                                vec![("data-blocks-footer-link-columns-brand-contact", "")],
                                vec![text("123 Example Street, Sample City, ST 00000")],
                            ),
                            div(
                                vec![("class", "blocks-footer-link-columns-social")],
                                vec![social_icon_link(REPO, "M12 2L2 7l10 5 10-5-10-5z", "GitHub")],
                            ),
                        ],
                    ),
                    div(
                        vec![("class", "blocks-footer-link-columns-columns")],
                        vec![
                            link_group(
                                "Product",
                                &[(GUIDE_URL, "Guide"), (API_REFERENCE_URL, "API Reference")],
                            ),
                            link_group("Company", &[(REPO, "About")]),
                        ],
                    ),
                ],
            ),
            separator::separator(
                &SeparatorProps::default(),
                vec![("data-blocks-footer-link-columns-separator", "")],
            ),
            div(
                vec![("class", "blocks-footer-link-columns-bottom")],
                vec![copyright()],
            ),
        ],
    )
}

/// インスタンス C: 下段なし。リンク 5 群（最後の Social 群はアイコン +
/// テキストのリンク 1 個。X は実アカウントが無いため非表示）。
fn instance_c() -> Node {
    footer(
        vec![("class", "blocks-footer-link-columns-instance")],
        vec![
            note("下段なし。リンク 5 群。最後の Social 群はアイコン + テキストのリンク 1 個。"),
            div(
                vec![("class", "blocks-footer-link-columns-top")],
                vec![
                    brand_column("外部依存ゼロの描画コアを持つフレームワークです。", false),
                    div(
                        vec![("class", "blocks-footer-link-columns-columns")],
                        vec![
                            link_group(
                                "Product",
                                &[(GUIDE_URL, "Guide"), (API_REFERENCE_URL, "API Reference")],
                            ),
                            link_group(
                                "Resources",
                                &[(EXAMPLES_URL, "Examples"), (RELEASES_URL, "Changelog")],
                            ),
                            link_group("Community", &[(DISCUSSIONS_URL, "Discussions")]),
                            link_group("Company", &[(REPO, "About")]),
                            social_link_group(),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// C 用の Social 群（アイコン + テキストの `link`。`link_group` とは異なり
/// 各項目がアイコンを伴うため専用実装にする）。実在の遷移先を持つ
/// GitHub のみを掲載する（モジュール doc「リンク先の方針」節参照）。
fn social_link_group() -> Node {
    let socials: [(&'static str, &'static str, &'static str); 1] =
        [(REPO, "M12 2L2 7l10 5 10-5-10-5z", "GitHub")];
    let items: Vec<Node> = socials
        .iter()
        .map(|(href, path_d, label)| {
            li(
                vec![],
                vec![link::root(
                    href,
                    &LinkProps {
                        external: true,
                        ..LinkProps::default()
                    },
                    vec![("data-blocks-footer-link-columns-social-text-link", "")],
                    vec![geo_icon(path_d, None), text(*label)],
                )],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-footer-link-columns-group")],
        vec![
            heading::heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![("data-blocks-footer-link-columns-group-title", "")],
                vec![text("Social")],
            ),
            ul(vec![("class", "blocks-footer-link-columns-list")], items),
        ],
    )
}

/// `footer-link-columns` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（他 block と同じ状態を持たない設計）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-footer-link-columns-layout")],
        vec![instance_a(), instance_b(), instance_c()],
    )
}
```

## 原案差分メモ

集約元 7 件の差分を、次のとおり 3 インスタンスへ統合しています。

- **A（基準形）**: ブランド列（ロゴマーク + 名称 + 一文説明）+ リンク 4 群。
  下段に区切り線・著作権・SNS アイコンのみのリンク 3 個
- **B（ブランド列を強調）**: ブランド列を 2 列幅に広げ、mission 文・架空の
  連絡先・SNS アイコンを収める。リンクは 2 群。下段は著作権 + 法務リンク
  （プライバシー・利用規約）
- **C（下段なし）**: リンク 5 群。最後の「Social」群はアイコン + テキストの
  リンク。下段（区切り線・著作権）は持たない
