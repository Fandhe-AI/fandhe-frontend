# blog-grid-image

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `badge` / `card` /
`image` / `avatar` / `link` / `link-overlay` の 8 部品を合成した、画像付き
記事カードグリッドの合成例です。Blocks セクションは新規部品を追加する
ものではなく、既存の Themes/Primitives 部品を組み合わせた実例集である
ことに注意してください。

記事データ・人名・画像はすべて架空のもので、実在の企業・製品・人物とは
関係ありません。画像は `crate::blocks::dummy_assets` が供給するプレース
ホルダー SVG（docs サイト内部の素材ヘルパ）です。実際に利用する際は、
自分の記事データ・画像 URL・リンク先へ置き換えてください。記事カードは
`link-overlay` でカード全面をクリック領域にしていますが、`<form>` や
送信処理・データ整形は一切持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text as core_text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay;
use fandhe_frontend_pre_styled_ui::text::{
    self as text_part, TextProps, TextSize, TextVariant, TextWeight,
};

/// 1 件の架空記事データ（実企業名・実人物は使わない）。
struct Article {
    category: &'static str,
    date: &'static str,
    read_time: &'static str,
    title: &'static str,
    excerpt: &'static str,
    /// [`dummy_assets::PERSON_NAMES`]/[`dummy_assets::JOB_TITLES`] への添字。
    author_index: usize,
    /// サイト内に実在する索引ページへの相対パス（モジュール doc「href の
    /// 方針」節参照）。
    href: &'static str,
}

/// インスタンス A（カード枠あり・3 件）の記事データ。
const ARTICLES_FRAMED: [Article; 3] = [
    Article {
        category: "Engineering",
        date: "2026-03-04",
        read_time: "5 min read",
        title: "型で守る既定エスケープ設計",
        excerpt: "テキスト補間を型システムの外へ逃がさない設計判断と、\
                   レビューで見落としを防ぐ観点をまとめました。",
        author_index: 0,
        href: "../../guides/",
    },
    Article {
        category: "Release",
        date: "2026-02-18",
        read_time: "3 min read",
        title: "単一実行ファイル配布の運用知見",
        excerpt: "Docker イメージのサイズ削減と起動時間の実測結果を、\
                   移行前後で比較しながら振り返ります。",
        author_index: 1,
        href: "../../examples/",
    },
    Article {
        category: "Guides",
        date: "2026-01-30",
        read_time: "6 min read",
        title: "SSR から始めるルーティング設計",
        excerpt: "パスマッチングの基本方針と、よくある詰まりどころへの\
                   対処法を手順に沿って解説します。",
        author_index: 2,
        href: "../../api/",
    },
];

/// インスタンス B（枠なし・正方形画像・2 件）の記事データ。
const ARTICLES_PLAIN: [Article; 2] = [
    Article {
        category: "Design",
        date: "2026-03-12",
        read_time: "4 min read",
        title: "色トークン運用の小さな改善",
        excerpt: "配色の一貫性を保つための命名規則と、レビューで使える\
                   簡易チェックリストを共有します。",
        author_index: 3,
        href: "../../themes/",
    },
    Article {
        category: "Performance",
        date: "2026-02-27",
        read_time: "3 min read",
        title: "計測から始める描画速度の見直し",
        excerpt: "ベンチマークの取り方と、数値を読み違えないための\
                   前提条件の揃え方を紹介します。",
        author_index: 4,
        href: "../../primitives/",
    },
];

/// 記事カード共通のメタ行（カテゴリ badge + 日付・読了時間）。日付は
/// `<time datetime>` でマークアップする（レビュー指摘対応: 読了時間と
/// 結合した通常テキストのままでは機械可読な公開日として認識できない）。
/// `article.date` は ISO 8601 表記のためそのまま `datetime` 属性と表示
/// 文字列の両方に使い、表示・機械可読値の食い違いを持ち込まない。
fn meta_row(category: &str, date: &str, read_time: &str) -> Node {
    div(
        vec![("class", "blocks-blog-grid-image-meta")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![core_text(category)]),
            text_part::text(
                &TextProps {
                    size: TextSize::Xs,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![
                    el("time", vec![("datetime", date)], vec![core_text(date)]),
                    core_text(format!(" ・ {read_time}")),
                ],
            ),
        ],
    )
}

/// 記事カード共通の著者行（アバター + 氏名 + 役職）。
fn author_row(author_index: usize) -> Node {
    let name = dummy_assets::PERSON_NAMES[author_index % dummy_assets::PERSON_NAMES.len()];
    let role = dummy_assets::JOB_TITLES[author_index % dummy_assets::JOB_TITLES.len()];
    div(
        vec![("class", "blocks-blog-grid-image-author")],
        vec![
            avatar::root(
                &AvatarProps::default(),
                vec![("data-blocks-blog-grid-image-avatar", "")],
                vec![avatar::image(
                    ImageStatus::Loaded,
                    dummy_assets::AVATAR_SRC,
                    name,
                    vec![],
                )],
            ),
            div(
                vec![("class", "blocks-blog-grid-image-author-name")],
                vec![
                    text_part::text(
                        &TextProps {
                            size: TextSize::Sm,
                            weight: TextWeight::Semibold,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![core_text(name)],
                    ),
                    text_part::text(
                        &TextProps {
                            size: TextSize::Xs,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![core_text(role)],
                    ),
                ],
            ),
        ],
    )
}

/// 記事 1 件分のカードを組み立てる。`framed` が `true` のときカード枠あり
/// かつ 16:9 画像（インスタンス A）、`false` のとき枠なしかつ正方形画像
/// （インスタンス B）になる。
fn article_card(article: &Article, framed: bool) -> Node {
    // レビュー指摘対応: 全記事で同一の汎用 alt テキストを使うと記事ごとの
    // 識別ができないため、記事タイトルを差し込んで 1 件ずつ異なる alt に
    // する（`article.title` は架空の日本語文でありユーザー入力ではない）。
    let alt = format!("{}の記事サムネイル画像", article.title);
    let image_node = if framed {
        let mut props = ImageProps::new(dummy_assets::SCREENSHOT_SRC, &alt);
        props.aspect_ratio = AspectRatio::Video;
        image::image(&props, vec![])
    } else {
        let mut props = ImageProps::new(dummy_assets::PRODUCT_SRC, &alt);
        props.aspect_ratio = AspectRatio::Square;
        props.shape = ImageShape::Rounded;
        image::image(&props, vec![])
    };

    // レビュー指摘対応（PR #3156）: 各インスタンスの導入見出し（`intro_framed`/
    // `intro_plain`）が H3 のため、記事タイトルは H4 にして見出し階層を
    // 導入見出しの子として保つ（`blog_list_image`/`blog_split_header_grid`
    // と同型の判断）。
    let title = heading::heading(
        HeadingLevel::H4,
        &HeadingProps {
            size: HeadingSize::Md,
            weight: HeadingWeight::Semibold,
        },
        vec![("data-blocks-blog-grid-image-title", "")],
        vec![core_text(article.title)],
    );
    let excerpt = text_part::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-blog-grid-image-excerpt", "")],
        vec![core_text(article.excerpt)],
    );
    let meta = meta_row(article.category, article.date, article.read_time);
    let author = author_row(article.author_index);

    let content = if framed {
        card::root(
            CardProps::default(),
            vec![("data-blocks-blog-grid-image-card", "")],
            vec![
                card::cover(vec![], vec![image_node]),
                card::body(
                    vec![("class", "blocks-blog-grid-image-body")],
                    vec![meta, title, excerpt, author],
                ),
            ],
        )
    } else {
        div(
            vec![("class", "blocks-blog-grid-image-plain")],
            vec![image_node, meta, title, excerpt, author],
        )
    };

    link_overlay::root(
        vec![("data-blocks-blog-grid-image-article", "")],
        vec![
            content,
            // モジュール doc「link_overlay の使い方」節: 可視タイトルは
            // 上の heading が担うため overlay の子は空にし、aria-label で
            // アクセシブルネームを与える。
            link_overlay::overlay(article.href, vec![("aria-label", article.title)], vec![]),
        ],
    )
}

/// インスタンス A の導入部（中央寄せ）。
fn intro_framed() -> Node {
    div(
        vec![("class", "blocks-blog-grid-image-intro")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![core_text("Blog")]),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![("data-blocks-blog-grid-image-heading", "")],
                vec![core_text("最新の開発日誌")],
            ),
            text_part::text(
                &TextProps {
                    size: TextSize::Md,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![core_text(
                    "チームの設計判断や運用の知見を、記事としてまとめています。",
                )],
            ),
        ],
    )
}

/// インスタンス B の導入部（左寄せ + 「すべての記事を見る」リンク）。
fn intro_plain() -> Node {
    div(
        vec![
            ("class", "blocks-blog-grid-image-intro"),
            ("data-align", "start"),
        ],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![core_text("Notes")]),
            div(
                vec![("class", "blocks-blog-grid-image-intro-row")],
                vec![
                    div(
                        vec![],
                        vec![
                            heading::heading(
                                HeadingLevel::H3,
                                &HeadingProps::default(),
                                vec![("data-blocks-blog-grid-image-heading", "")],
                                vec![core_text("運用ノート")],
                            ),
                            text_part::text(
                                &TextProps {
                                    size: TextSize::Md,
                                    variant: TextVariant::Muted,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![core_text("短い運用メモを不定期に公開しています。")],
                            ),
                        ],
                    ),
                    link::root(
                        "../../wireframes/",
                        &LinkProps::default(),
                        vec![],
                        vec![core_text("すべての記事を見る")],
                    ),
                ],
            ),
        ],
    )
}

/// 1 インスタンス分（導入部 + グリッド）を組み立てる。`two_col` が `true`
/// のときグリッドは常に 2 列（インスタンス B）、`false` のとき 3 列 → 2 列
/// → 1 列のレスポンシブ（インスタンス A）になる。
fn instance(intro: Node, articles: &[Article], framed: bool, two_col: bool) -> Node {
    let mut grid_attrs = vec![("class", "blocks-blog-grid-image-grid")];
    if two_col {
        grid_attrs.push(("data-columns", "2"));
    }
    let cards: Vec<Node> = articles
        .iter()
        .map(|article| article_card(article, framed))
        .collect();

    div(
        vec![("class", "blocks-blog-grid-image-instance")],
        vec![intro, div(grid_attrs, cards)],
    )
}

/// `blog-grid-image` の Demo 本体。呼び出しごとに同一の `Node` を返す純
/// 関数。2 インスタンス（モジュール doc「2 インスタンスで示す差分」節）を
/// 縦に並べた静的な表示のみを描く。
#[must_use]
pub fn demo() -> Node {
    let note_a = text_part::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(
            "中央寄せの導入部・カード枠あり・16:9 画像の構成例。",
        )],
    );
    let note_b = text_part::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(
            "左寄せの導入部・カード枠なし・正方形画像の構成例。",
        )],
    );

    div(
        vec![("class", "blocks-blog-grid-image")],
        vec![
            note_a,
            instance(intro_framed(), &ARTICLES_FRAMED, true, false),
            note_b,
            instance(intro_plain(), &ARTICLES_PLAIN, false, true),
        ],
    )
}
```

**原案差分メモ**

集約元（対応表 ID R0017 が主参照、R0018/R0019/R0020/R0416/R0418/R0420/
R0773 を集約）の差分は、Demo の 2 インスタンスまたは本節のいずれかで
表現しています。

- **見出しの中央寄せ（R0017）**: インスタンス A の導入部で表現しました。
- **見出しの左寄せ + 「すべての記事を見る」リンク（R0018/R0773）**:
  インスタンス B の導入部で表現しました。
- **カード枠あり（R0017/R0018）**: インスタンス A で `card::root` により
  表現しました。
- **カード枠なし（R0019/R0020）**: インスタンス B で `card::root` を使わ
  ない素の `div` 構造により表現しました。
- **正方形画像 + 著者アバター（R0420）**: インスタンス B の
  `AspectRatio::Square` + `avatar::root` で表現しました。
- **4 列グリッド（R0416）**: Demo には含めていません。本文カラム幅
  （最大 46rem）では 1 列あたり 10rem を切り読みにくくなるため、Demo の
  グリッドは 2〜3 列に留めています。実装するだけであれば、レイアウト CSS
  の `grid-template-columns: repeat(3, ...)` を `repeat(4, ...)` へ変更
  するだけで表現でき、コードを増やす必要はありません。
- **画像を本文の下に置く配置（R0418）**: Demo には含めていません。カード
  内のノードを並べる順序（DOM 順、または CSS の `order`）を入れ替える
  だけで表現でき、コードを増やす必要はありません。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Badge](../themes/badge.md) / [Card](../themes/card.md) /
[Image](../themes/image.md) / [Avatar](../themes/avatar.md) /
[Link](../themes/link.md) / [Link Overlay](../themes/link-overlay.md)
