# footer-cta-columns

`heading` / `text` / `link` / `separator` / `icon` を合成した、CTA 付き
リンクカラム footer です（主参照は対応表 ID R0961。出典の固有名・
ファイル名は記載しません）。

3 段構成です。

1. 上段: 中央寄せの CTA（小見出し・見出し・説明文・ボタン風リンク）。
2. 区切り線の下の中段: ロゴと 4 列のリンク。
3. 区切り線の下の下段: SNS アイコンのリンクと著作権表記。

狭い幅では中段のリンクを 2 列で並べ、`48rem` 以上でロゴ 1 列 + リンク
4 列へ切り替わります。

CTA のボタンは `<a>`（リンク）として描いており、`<form>` 要素・送信処理・
状態管理は一切持ちません。文言はすべて独自に書いたものであり、実企業名・
実クレデンシャル・PII を含みません。リンク先はすべて `Fandhe-AI` の実在
GitHub リポジトリへの外部 URL です。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, footer, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
const REPO_SPEC: &str = "https://github.com/Fandhe-AI/fandhe-frontend-spec";

/// 自作の抽象幾何ロゴ（`cta_centered::logo_mark` と同型）。
fn logo_mark() -> Node {
    span(
        vec![
            ("class", "blocks-footer-cta-columns-logo"),
            ("aria-hidden", "true"),
        ],
        vec![],
    )
}

/// 装飾用の自作幾何アイコン（`contact_split_info::geo_icon` と同型、
/// lucide 等の既存アイコンセットの path を複製しない単純図形）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
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

/// リンク列 1 群（見出し + リンク一覧、`footer_newsletter::link_column` と
/// 同型）。
fn link_column(title: &str, links: &[(&str, &str)]) -> Node {
    let items: Vec<Node> = links
        .iter()
        .map(|(href, label)| {
            div(
                vec![("class", "blocks-footer-cta-columns-link-item")],
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
        vec![("class", "blocks-footer-cta-columns-column")],
        std::iter::once(p(
            vec![("class", "blocks-footer-cta-columns-column-title")],
            vec![text(title)],
        ))
        .chain(items)
        .collect(),
    )
}

/// SNS 風リンク 1 件（自作幾何アイコンのみ + 遷移先と一致する `aria-label`。
/// `contact_split_info::social_link` と異なり可視テキストを持たないため、
/// 3 件とも `aria-label` を付与する）。
fn social_link(href: &str, aria_label: &str, icon_node: Node) -> Node {
    link::root(
        href,
        &LinkProps::default(),
        vec![
            ("aria-label", aria_label),
            ("data-blocks-footer-cta-columns-social-link", ""),
        ],
        vec![icon_node],
    )
}

/// 上段: 中央寄せの CTA（小見出し・見出し・説明文・ボタン風リンク）。
fn cta_section() -> Node {
    div(
        vec![("class", "blocks-footer-cta-columns-cta")],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("class", "blocks-footer-cta-columns-eyebrow")],
                vec![text("コミュニティ")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("今すぐプロジェクトに参加しましょう")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "GitHub で最新のリリースを追いかけ、Issue や Pull Request で\
                     フィードバックを送りましょう。",
                )],
            ),
            link::root(
                REPO,
                &LinkProps::default(),
                vec![("data-blocks-footer-cta-columns-cta", "")],
                vec![text("GitHub で見る")],
            ),
        ],
    )
}

/// 中段: ロゴ + 4 列のリンク。
fn columns_section() -> Node {
    div(
        vec![("class", "blocks-footer-cta-columns-columns")],
        vec![
            div(
                vec![("class", "blocks-footer-cta-columns-brand")],
                vec![
                    logo_mark(),
                    p(
                        vec![("class", "blocks-footer-cta-columns-brand-name")],
                        vec![text("Fandhe Frontend")],
                    ),
                ],
            ),
            link_column(
                "Product",
                &[
                    (&format!("{REPO}#readme"), "Guide"),
                    (&format!("{REPO}/tree/main/examples"), "Examples"),
                    (&format!("{REPO}/releases"), "Releases"),
                ],
            ),
            link_column(
                "Docs",
                &[
                    (&format!("{REPO}/tree/main/docs/guides"), "Guides"),
                    (&format!("{REPO}/tree/main/docs/api"), "API Reference"),
                    (&format!("{REPO}/tree/main/docs/design"), "Design Docs"),
                ],
            ),
            link_column(
                "Community",
                &[
                    (&format!("{REPO}/issues"), "Issues"),
                    (&format!("{REPO}/pulls"), "Pull Requests"),
                    (&format!("{REPO}/blob/main/CLAUDE.md"), "Contributing"),
                ],
            ),
            link_column(
                "Specification",
                &[
                    (&format!("{REPO_SPEC}#readme"), "Spec Repository"),
                    (&format!("{REPO_SPEC}/issues"), "Spec Issues"),
                    (&format!("{REPO_SPEC}/pulls"), "Spec Pull Requests"),
                ],
            ),
        ],
    )
}

/// 下段: SNS アイコンのリンクと著作権表記。
fn bottom_row() -> Node {
    div(
        vec![("class", "blocks-footer-cta-columns-bottom")],
        vec![
            div(
                vec![("class", "blocks-footer-cta-columns-social")],
                vec![
                    social_link(
                        "https://github.com/Fandhe-AI",
                        "GitHub（Fandhe-AI）",
                        geo_icon(
                            "M9 3a6 6 0 00-2 11.6c0 .3 0 1 0 1.9 M9 3a6 6 0 012 11.6c0 .3 0 1 0 1.9 \
                             M6 17c-1.2.5-2 0-2.5-1",
                        ),
                    ),
                    social_link(
                        REPO,
                        "GitHub（fandhe-frontend）",
                        geo_icon(
                            "M4 4h8l4 4v12H4V4z M12 4v4h4 M8 12h6 M8 15h6",
                        ),
                    ),
                    social_link(
                        REPO_SPEC,
                        "GitHub（fandhe-frontend-spec）",
                        geo_icon("M12 3l7 4v10l-7 4-7-4V7l7-4z M12 3v18 M5 7l7 4 7-4"),
                    ),
                ],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("class", "blocks-footer-cta-columns-copyright")],
                vec![text("© 2026 Fandhe Frontend.")],
            ),
        ],
    )
}

/// `footer-cta-columns` の Demo 本体（モジュール doc「3 段構成」節参照）。
pub fn demo() -> Node {
    footer(
        vec![("class", "blocks-footer-cta-columns-layout")],
        vec![
            cta_section(),
            separator(&SeparatorProps::default(), vec![]),
            columns_section(),
            separator(&SeparatorProps::default(), vec![]),
            bottom_row(),
        ],
    )
}
```

## 原案差分メモ

対応表 ID R0961（主参照、集約元はこの 1 件のみ）からの意図的な差分は
次のとおりです。

- 原案はボタンとして描かれていますが、本実装では CTA を `<a>`（リンク）
  として描いています。`fandhe-frontend-pre-styled-ui` の `button` は
  `<button type="button">` のみを出力し href を持つ形を公開しておらず、
  `<a>` の中へ `<button>` を入れ子にすることは HTML として不正なため、
  リンクをボタン風の外見に装飾する形を採りました。この判断により
  `button` は使用部品（`## 使用部品`）から除外しています。
- ロゴは自作の抽象幾何図形（角丸枠 + グラデーション）で代用しています。
  実ブランドのロゴ・商標は持ち込んでいません。
- SNS 行のアイコンも自作の幾何図形です。実在 SNS プラットフォームの
  ロゴ・ブランドは描いていません。
- 配色・余白・角丸は既存のテーマトークン（`--fandhe-color-accent` /
  `--fandhe-space-*` / `--fandhe-radius-*` 等）に従っています。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Link](../themes/link.md) / [Separator](../themes/separator.md) /
[Icon](../themes/icon.md)
