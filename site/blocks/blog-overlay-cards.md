# blog-overlay-cards

`heading` / `text` / `card` / `image` / `avatar` / `link-overlay` を合成した、
背景画像へ下から上へのグラデーションを重ねる記事カードのグリッドです。

中央寄せの見出し + 導入文の下へ記事カードを格子状に並べます。幅 lg
（64rem）未満では 1 列、それ以上では 3 列になり、同じ行のカードは高さが
揃います。各カードは背景画像の上にグラデーションを重ねて文字を読みやすく
し、下端へ日付・著者（アバター + 氏名）のメタ行と記事タイトルを載せ、
カード全体が 1 つのリンクになります。文言・数値はすべて架空のもので、
データ取得・送信は行わない静的な表示例です。リンク先はすべてリポジトリ
への固定リンクです。`<form>` は使用しません。

集約元は 1 件のみ（対応表 ID R0774）です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 記事カード 1 件分のダミーデータ（架空、実在の人物・企業とは無関係）。
struct Post {
    date_iso: &'static str,
    date_label: &'static str,
    title: &'static str,
    author_name: &'static str,
}

/// 記事カード 3 件（架空）。タイトルの長さを意図的にばらつかせ、
/// `grid-auto-rows: 1fr` による行の高さ統一が実際に効いていることを
/// 目視確認しやすくする。
const POSTS: [Post; 3] = [
    Post {
        date_iso: "2026-09-18",
        date_label: "2026年9月18日",
        title: "既定エスケープを崩さないレビュー観点",
        author_name: "遠藤 佑奈",
    },
    Post {
        date_iso: "2026-09-11",
        date_label: "2026年9月11日",
        title: "単一バイナリ配布で削った依存",
        author_name: "冨田 千夏",
    },
    Post {
        date_iso: "2026-09-04",
        date_label: "2026年9月4日",
        title: "ノード木 API のまま重ね表示を組み立てる",
        author_name: "宮下 大和",
    },
];

/// 日付・区切り・著者（アバター + 氏名）のメタ行を組み立てる。
fn post_meta(post: &Post) -> Node {
    let initials: String = post
        .author_name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .collect();
    div(
        vec![("class", "blocks-blog-overlay-cards-meta")],
        vec![
            el(
                "time",
                vec![("datetime", post.date_iso)],
                vec![text(post.date_label)],
            ),
            span(vec![("aria-hidden", "true")], vec![text("\u{00B7}")]),
            div(
                vec![("class", "blocks-blog-overlay-cards-author")],
                vec![
                    avatar::root(
                        &AvatarProps {
                            size: Size::Xs,
                            ..AvatarProps::default()
                        },
                        vec![("data-blocks-blog-overlay-cards-avatar", "")],
                        vec![
                            avatar::image(
                                ImageStatus::Loaded,
                                dummy_assets::AVATAR_SRC,
                                "",
                                vec![],
                            ),
                            avatar::fallback(ImageStatus::Loaded, vec![], vec![text(initials)]),
                        ],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(post.author_name)],
                    ),
                ],
            ),
        ],
    )
}

/// 記事カード 1 件（背景画像 + スクリム + メタ行/タイトル + 全面
/// クリック用 `overlay`。モジュール doc「重なり順」節参照）。
fn overlay_card(post: &Post) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-blog-overlay-cards-card", "")],
        vec![link_overlay::root(
            vec![("data-blocks-blog-overlay-cards-link", "")],
            vec![
                image::image(
                    &ImageProps::new(dummy_assets::BACKGROUND_SRC, ""),
                    vec![("data-blocks-blog-overlay-cards-bg", "")],
                ),
                div(
                    vec![
                        ("class", "blocks-blog-overlay-cards-scrim"),
                        ("aria-hidden", "true"),
                    ],
                    vec![],
                ),
                div(
                    vec![("class", "blocks-blog-overlay-cards-content")],
                    vec![
                        post_meta(post),
                        heading::heading(
                            HeadingLevel::H4,
                            &HeadingProps::default(),
                            vec![("data-blocks-blog-overlay-cards-title", "")],
                            vec![text(post.title)],
                        ),
                    ],
                ),
                overlay(
                    REPO,
                    vec![
                        ("aria-label", post.title),
                        ("data-blocks-blog-overlay-cards-overlay", ""),
                    ],
                    vec![],
                ),
            ],
        )],
    )
}

/// `blog-overlay-cards` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「レイアウト」節）。
pub fn demo() -> Node {
    let cards: Vec<Node> = POSTS.iter().map(overlay_card).collect();
    div(
        vec![("class", "blocks-blog-overlay-cards")],
        vec![
            div(
                vec![("class", "blocks-blog-overlay-cards-header")],
                vec![
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl2,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("ブログ")],
                    ),
                    styled_text::text(
                        &TextProps::default(),
                        vec![],
                        vec![text(
                            "チームの運用ノウハウをまとめた記事から、注目の 3 本を選びました。",
                        )],
                    ),
                ],
            ),
            div(vec![("class", "blocks-blog-overlay-cards-grid")], cards),
        ],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0774。出典の固有名・ファイル名は記載しません）からの
意図的な差分は次のとおりです。

- 見出しレベルを `h2` から `h3` へ変更しました（ページ側が `## Demo` として
  `h2` を出すため）。
- 暗色固定の配色を、テーマトークンの fg/bg 反転ペアへ置き換えました
  （dark テーマでは明るいスクリムと暗色文字になります）。
- 装飾リング要素は持ち込まず、card 部品の既定の外観にそのまま従います。
- 実写画像を共通のダミー背景画像へ、実在風の人物写真を共通のダミー
  アバター画像 + イニシャル fallback へ置き換えました。
- `href="#"` の死リンクをリポジトリへの固定外部 URL へ置き換えました。
- 全面クリックを疑似要素ではなく `link-overlay` 部品で表現しました。
- 文言（見出し・導入文・記事タイトル・著者名）はすべて独自に書き直しました。
- `datetime` の機械可読値と表示値が食い違う組み合わせは持ち込みません
  （本実装は常に同じ日を指す値の組にしています）。
