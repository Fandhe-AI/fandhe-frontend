# blog-list-image

`heading` / `text` / `badge` / `image` / `avatar` / `separator` / `link` /
`link-overlay` を合成した、見出し + 画像横並びの記事リストです。

幅 lg（64rem）以上では画像が左、本文が右の横並びになり、それより狭い幅
では画像の下に本文を縦に積む表示に切り替わります。文言・数値はすべて
架空のもので、データ取得・送信は行わない静的な表示例です。リンク先は
すべてリポジトリへの固定リンクです。`<form>` は使用しません。

集約元は 1 件のみ（対応表 ID R0776）です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{article, div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「`href="#"` を使わない」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 記事 1 件分のダミーデータ（架空、実在の人物・企業とは無関係）。
struct Post {
    date_iso: &'static str,
    date_label: &'static str,
    category: &'static str,
    title: &'static str,
    excerpt: &'static str,
    image_src: &'static str,
    author_index: usize,
}

/// 記事一覧（架空、3 件）。`author_index` は
/// [`dummy_assets::PERSON_NAMES`]/[`dummy_assets::JOB_TITLES`] への添字。
const POSTS: [Post; 3] = [
    Post {
        date_iso: "2026-09-18",
        date_label: "2026年9月18日",
        category: "設計",
        title: "ノード木 API で HTML 文字列組み立てを避ける",
        excerpt: "既定エスケープを弱めずに合成例を増やすための、部品合成の考え方をまとめました。",
        image_src: dummy_assets::SCREENSHOT_SRC,
        author_index: 0,
    },
    Post {
        date_iso: "2026-09-10",
        date_label: "2026年9月10日",
        category: "運用",
        title: "block 追加を長文追記なしで回せるようにした話",
        excerpt: "ドキュメントへの逐次追記をやめ、レジストリと原稿を正にした運用の振り返りです。",
        image_src: dummy_assets::PRODUCT_SRC,
        author_index: 1,
    },
    Post {
        date_iso: "2026-09-02",
        date_label: "2026年9月2日",
        category: "アクセシビリティ",
        title: "画像スロットの alt を空にしてよい条件",
        excerpt: "装飾目的の画像とそうでない画像を切り分ける、実務上の判断基準を紹介します。",
        image_src: dummy_assets::BACKGROUND_SRC,
        author_index: 2,
    },
];

/// `<time datetime>` を組み立てる（モジュール doc「`<time datetime>` と
/// 表示日付の一致」節）。
fn post_date(iso: &str, label: &str) -> Node {
    el(
        "time",
        vec![("class", "blocks-blog-list-image-date"), ("datetime", iso)],
        vec![text(label)],
    )
}

/// メタ行（カテゴリ badge + 日付）。
fn meta_row(post: &Post) -> Node {
    div(
        vec![("class", "blocks-blog-list-image-meta")],
        vec![
            post_date(post.date_iso, post.date_label),
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-blog-list-image-category", "")],
                vec![text(post.category)],
            ),
        ],
    )
}

/// 著者行（アバターのイニシャル fallback + 氏名 + 肩書）。`overlay` の外へ
/// 兄弟として置くことでクリック可能なまま保つ（モジュール doc「記事
/// リンクの入れ子を避ける 2 段構成」節参照）。
fn author_row(name: &str, role: &str) -> Node {
    let initials: String = name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .collect();
    div(
        vec![("data-blocks-blog-list-image-author", "")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Sm,
                    ..AvatarProps::default()
                },
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text(initials)],
                )],
            ),
            div(
                vec![],
                vec![
                    link::root(
                        REPO,
                        &LinkProps::default(),
                        vec![("data-blocks-blog-list-image-author-link", "")],
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
                ],
            ),
        ],
    )
}

/// 記事 1 件（正方形画像 + 本文。本文上部はメタ行・見出し・抜粋 +
/// 全面クリック用 `overlay`、下端に区切り線 + 著者行）。
fn post_item(post: &Post) -> Node {
    let author = dummy_assets::PERSON_NAMES[post.author_index];
    let role = dummy_assets::JOB_TITLES[post.author_index % dummy_assets::JOB_TITLES.len()];
    article(
        vec![("class", "blocks-blog-list-image-article")],
        vec![
            div(
                vec![("class", "blocks-blog-list-image-figure")],
                vec![image::image(
                    &ImageProps {
                        aspect_ratio: AspectRatio::Square,
                        shape: ImageShape::Rounded,
                        ..ImageProps::new(post.image_src, "")
                    },
                    vec![("data-blocks-blog-list-image-image", "")],
                )],
            ),
            div(
                vec![("class", "blocks-blog-list-image-body")],
                vec![
                    link_overlay::root(
                        vec![("data-blocks-blog-list-image-main", "")],
                        vec![
                            meta_row(post),
                            heading(
                                HeadingLevel::H4,
                                &HeadingProps::default(),
                                vec![],
                                vec![text(post.title)],
                            ),
                            styled_text::text(
                                &TextProps {
                                    variant: TextVariant::Muted,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(post.excerpt)],
                            ),
                            overlay(REPO, vec![("aria-label", post.title)], vec![]),
                        ],
                    ),
                    div(
                        vec![("class", "blocks-blog-list-image-footer")],
                        vec![
                            separator(
                                &SeparatorProps::default(),
                                vec![("data-blocks-blog-list-image-separator", "")],
                            ),
                            author_row(author, role),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// `blog-list-image` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「レイアウト」節）。
pub fn demo() -> Node {
    let header = div(
        vec![("class", "blocks-blog-list-image-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("開発ブログ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "設計・運用・アクセシビリティに関する記事を掲載しています。",
                )],
            ),
        ],
    );

    let list = div(
        vec![("class", "blocks-blog-list-image-list")],
        POSTS.iter().map(post_item).collect(),
    );

    div(
        vec![("class", "blocks-blog-list-image-layout")],
        vec![header, list],
    )
}
```

**原案差分メモ**

参照（対応表 ID R0776。出典の固有名・ファイル名は記載しません）からの
意図的な差分は次のとおりです。

- 見出しレベルを `h2`/`h3` から `h3`/`h4` へ 1 段下げました（ページ側が
  `## Demo` として `h2` を出すため）。
- 画像は狭い幅・広い幅のいずれでも正方形（アスペクト比 1:1）にしました
  （参照側は狭い幅 16:9・広い幅 2:1 でした）。狭い幅で画像が巨大化しない
  よう幅の上限も付けています。
- カテゴリはリンクではなく badge にしました（overlay の内側に 3 つ目の
  クリック可能要素を重ねないため）。
- `href="#"` の死リンクをリポジトリへの固定外部 URL へ置き換えました。
- 画像はクリック範囲から外し、著者リンクを overlay の外の兄弟として配置
  することで、クリック可能な要素が重ならないようにしました。
- アバターは画像ではなく、既存部品のイニシャル fallback 表示にしました。
- 文言（見出し・抜粋・著者名・肩書）はすべて独自に書き直しました。
- 配色・余白・角丸は独自実装せず、既存のテーマトークンにそのまま従います。
- 日付の機械可読値（`datetime`）と表示値が食い違わないようにしています。
- `id` 属性・`aria-describedby` は出力しません（宙に浮いた ARIA 参照・
  id 重複を構造的に避けるため）。
