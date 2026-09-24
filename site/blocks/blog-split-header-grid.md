# blog-split-header-grid

`heading` / `text` / `badge` / `card` / `image` / `link` を
合成した、見出し左 + 記事グリッド右のブログセクションです。

幅 lg（64rem）以上では左に見出し列（tagline・見出し・説明・「すべての
記事を見る」リンク）、右に記事カードの 2 列グリッドが並びます。それより
狭い幅では見出し列の下にカードが 1 列で積まれます。文言・数値はすべて
架空のもので、データ取得・送信は行わない静的な表示例です。リンク先は
すべてリポジトリへの固定外部 URL です（記事タイトル・「すべての記事を
見る」ともに実際に遷移する `<a href>`、`<form>` は使用しません）。

集約元は 1 件のみ（対応表 ID R0419）です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// リンク先の固定外部 URL（モジュール doc「`href="#"` を使わない」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 記事 1 件分のダミーデータ（架空、実在の人物・企業とは無関係）。
struct Post {
    date_iso: &'static str,
    date_label: &'static str,
    category: &'static str,
    title: &'static str,
    image_src: &'static str,
    author_index: usize,
}

/// 記事一覧（架空、4 件。`author_index` は
/// [`dummy_assets::PERSON_NAMES`] への添字）。
const POSTS: [Post; 4] = [
    Post {
        date_iso: "2026-09-16",
        date_label: "2026年9月16日",
        category: "設計",
        title: "見出しと一覧を分ける 2 カラム構成の考え方",
        image_src: dummy_assets::SCREENSHOT_SRC,
        author_index: 0,
    },
    Post {
        date_iso: "2026-09-09",
        date_label: "2026年9月9日",
        category: "運用",
        title: "block の索引を長文追記なしで回す",
        image_src: dummy_assets::PRODUCT_SRC,
        author_index: 1,
    },
    Post {
        date_iso: "2026-09-01",
        date_label: "2026年9月1日",
        category: "アクセシビリティ",
        title: "装飾画像の alt を空にしてよい条件",
        image_src: dummy_assets::BACKGROUND_SRC,
        author_index: 2,
    },
    Post {
        date_iso: "2026-08-24",
        date_label: "2026年8月24日",
        category: "テスト",
        title: "合成例のグリッドをブレークポイント別に固定する",
        image_src: dummy_assets::LOGO_SRC,
        author_index: 3,
    },
];

/// `<time datetime>` を組み立てる（モジュール doc「`<time datetime>` と
/// 表示日付の一致」節）。
fn post_date(iso: &str, label: &str) -> Node {
    el(
        "time",
        vec![
            ("class", "blocks-blog-split-header-grid-date"),
            ("datetime", iso),
        ],
        vec![text(label)],
    )
}

/// メタ行（日付 + カテゴリ badge）。
fn meta_row(post: &Post) -> Node {
    div(
        vec![("class", "blocks-blog-split-header-grid-meta")],
        vec![
            post_date(post.date_iso, post.date_label),
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-blog-split-header-grid-category", "")],
                vec![text(post.category)],
            ),
        ],
    )
}

/// 記事カード 1 件（画像 + メタ行 + タイトルリンク + 著者）。
fn post_card(post: &Post) -> Node {
    let author = dummy_assets::PERSON_NAMES[post.author_index];
    card::root(
        CardProps {
            variant: CardVariant::Outline,
            ..CardProps::default()
        },
        vec![("data-blocks-blog-split-header-grid-card", "")],
        vec![
            card::cover(
                vec![("class", "blocks-blog-split-header-grid-cover")],
                vec![image::image(
                    &ImageProps {
                        aspect_ratio: AspectRatio::Landscape,
                        shape: ImageShape::Square,
                        ..ImageProps::new(post.image_src, "")
                    },
                    vec![("data-blocks-blog-split-header-grid-image", "")],
                )],
            ),
            card::body(
                vec![("class", "blocks-blog-split-header-grid-body")],
                vec![
                    meta_row(post),
                    heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![link::root(
                            REPO,
                            &LinkProps::default(),
                            vec![("data-blocks-blog-split-header-grid-title-link", "")],
                            vec![text(post.title)],
                        )],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![("data-blocks-blog-split-header-grid-author", "")],
                        vec![text(author)],
                    ),
                ],
            ),
        ],
    )
}

/// 左列（tagline + 見出し + 説明 + 「すべての記事を見る」リンク）。
fn lead_column() -> Node {
    div(
        vec![("class", "blocks-blog-split-header-grid-lead")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-blog-split-header-grid-tagline", "")],
                vec![text("開発ブログ")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![("data-blocks-blog-split-header-grid-heading", "")],
                vec![text("最新の開発ノート")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "設計・運用・アクセシビリティに関する記事を、書きためた順に紹介しています。",
                )],
            ),
            link::root(
                REPO,
                &LinkProps {
                    variant: LinkVariant::Underline,
                    ..LinkProps::default()
                },
                vec![("data-blocks-blog-split-header-grid-view-all", "")],
                vec![
                    text("すべての記事を見る"),
                    el("span", vec![("aria-hidden", "true")], vec![text(" →")]),
                ],
            ),
        ],
    )
}

/// `blog-split-header-grid` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「レイアウト」節）。
pub fn demo() -> Node {
    let grid = div(
        vec![("class", "blocks-blog-split-header-grid-grid")],
        POSTS.iter().map(post_card).collect(),
    );

    div(
        vec![("class", "blocks-blog-split-header-grid-layout")],
        vec![lead_column(), grid],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0419。出典の固有名・ファイル名は記載しません）からの
意図的な差分は次のとおりです。

- 見出しレベルを `h2`/`h3` から `h3`/`h4` へ 1 段下げました（ページ側が
  `## Demo` として `h2` を出すため）。
- 参照側のカードには著者が無いところ、イシューの仕様に従い著者名を
  カードへ追加しました。
- カテゴリはテキストではなく badge で表示しました。
- カードのグリッドは参照側の 48rem からではなく 64rem から 2 列に
  切り替えます（狭い幅では常にカードを 1 列で積む、というイシューの
  仕様を優先しました）。
- 左右の列比は 2:3 にしています。
- `href="#"` の死リンクをリポジトリへの固定外部 URL へ置き換えました。
- 「すべての記事を見る」は当初、実際には遷移しない静的な
  `type="button"` のボタンでしたが、codex レビュー指摘（イシュー #2814
  PR #3165）を受けて実際に遷移する `link` へ置き換えました。
- 画像は `data:` URI を使わず、ビルド時生成のプレースホルダー画像を
  相対パスで参照します。`alt` は装飾扱いとして空にしています。
- `id` 属性・`aria-describedby` は出力しません（宙に浮いた ARIA 参照・
  id 重複を構造的に避けるため）。
- 文言（tagline・見出し・説明・記事タイトル・カテゴリ・著者名）は
  すべて独自に書き直しました。
- 配色・余白・角丸は独自実装せず、既存のテーマトークンにそのまま従います。
- 日付の機械可読値（`datetime`）と表示値が食い違わないようにしています。
