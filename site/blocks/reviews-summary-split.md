# reviews-summary-split

左列に評価サマリ（平均評価・星別割合バー・導線）、右列にレビュー
一覧を置く 2 カラムのレビュー欄です。`heading` / `rating-group` /
`progress` / `avatar` / `text` / `link` / `button` / `image` / `card`
の 9 部品を合成します。Blocks は既存部品の合成例であり、新しい UI
部品は追加しません。

主参照は対応表 ID R1219（代表構成）、集約元は R0215 です。

評価・件数・割合・レビュー本文はすべて架空のデータであり、実在の
企業・ブランド・PII・実クレデンシャルは含みません。サムネイル画像は
同梱の SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。
「レビューを書く」ボタンは `disabled` の静的ボタンで、送信処理・
状態管理は一切持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::progress::{self, Orientation, ProgressProps};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc 参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// [`REPO`] の accessible name（遷移先と食い違わない固定文字列）。
const REPO_LABEL: &str = "fandhe-frontend の GitHub リポジトリ";

/// 全体の平均評価（表示用の数値文字列。半星表現がないための代替、
/// モジュール doc「平均評価は数値テキスト + 整数丸めの `rating-group`」節
/// 参照）。
const AVERAGE_RATING_DISPLAY: &str = "4.3";
/// [`AVERAGE_RATING_DISPLAY`] の切り捨て整数（`rating-group` の表示用）。
const AVERAGE_RATING_ROUNDED: u8 = 4;
/// 総レビュー件数（架空の固定値）。
const TOTAL_REVIEW_COUNT: u32 = 128;

/// 星別の割合（5〜1 の順、合計 100%。架空の固定値）。
const STAR_BREAKDOWN: [(u8, f64); 5] = [(5, 62.0), (4, 21.0), (3, 9.0), (2, 5.0), (1, 3.0)];

/// レビュー 1 件分のダミーデータ（架空、実在の企業・人物とは無関係）。
struct Review {
    author: &'static str,
    body: &'static str,
    /// 評価（1〜5）。
    rating: u8,
}

/// レビュー 6 件（[`dummy_assets::PERSON_NAMES`] の先頭 6 件を投稿者名に
/// 流用する。版 A はこのうち先頭 3 件のみを使う）。
const REVIEWS: [Review; 6] = [
    Review {
        author: dummy_assets::PERSON_NAMES[0],
        body: "質感が良く、写真で見るより満足度が高かったです。",
        rating: 5,
    },
    Review {
        author: dummy_assets::PERSON_NAMES[1],
        body: "届くまでが早く、梱包も丁寧でした。",
        rating: 4,
    },
    Review {
        author: dummy_assets::PERSON_NAMES[2],
        body: "値段の割にしっかりした作りで満足しています。",
        rating: 5,
    },
    Review {
        author: dummy_assets::PERSON_NAMES[3],
        body: "説明書がもう少し詳しいと助かります。",
        rating: 3,
    },
    Review {
        author: dummy_assets::PERSON_NAMES[4],
        body: "色味が想像どおりで気に入っています。",
        rating: 5,
    },
    Review {
        author: dummy_assets::PERSON_NAMES[5],
        body: "サイズ感がちょうど良く、普段使いしやすいです。",
        rating: 4,
    },
];

/// readonly の星評価（[`reviews_card_grid`](super::reviews_card_grid)の
/// `rating_row` と同型）。`label_text` が可視ラベルを兼ねるため
/// `visually_hidden` は使わない。
fn rating_stars(rating: u8, label_id: &str, label_text: String) -> Node {
    let props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let state = RatingGroup::new(5, Some(u32::from(rating)), true);
    let label = rating_group::label(&props, Some(label_id), vec![], vec![text(label_text)]);
    let items: Vec<Node> = (1..=state.count())
        .map(|i| {
            rating_group::item(
                i,
                RatingItemFlags {
                    checked: state.is_checked(i),
                    highlighted: state.is_highlighted(i),
                    disabled: false,
                    readonly: true,
                },
                &format!("{i} star{}", if i == 1 { "" } else { "s" }),
                vec![],
                vec![],
            )
        })
        .collect();
    let control = rating_group::control(&props, Some(label_id), vec![], items);
    rating_group::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![],
        vec![label, control],
    )
}

/// 星別割合バー 1 行（`feature_tabs_panel.rs::trigger_with_progress` と
/// 同型）。`aria-label` を明示する（モジュール doc「星別割合バー」節参照）。
fn breakdown_row(stars: u8, percent: f64) -> Node {
    let p = Progress::new(0.0, 100.0, Some(percent), Orientation::Horizontal);
    let aria_label = format!("{stars} つ星の割合");
    div(
        vec![("class", "blocks-reviews-summary-split-breakdown-row")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![],
                vec![text(format!("{stars} 星"))],
            ),
            progress::root(
                &p,
                &ProgressProps {
                    size: Size::Sm,
                    ..ProgressProps::default()
                },
                None,
                vec![("aria-label", aria_label.as_str())],
                vec![p.track(vec![], vec![progress::range(&p, vec![])])],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(format!("{percent:.0}%"))],
            ),
        ],
    )
}

/// サムネイル画像グリッド（8 枚、版 B 限定。モジュール doc「ダミー素材
/// について」節参照）。
fn thumbnail_grid() -> Node {
    // 8 枚すべて同一の `dummy_assets::PRODUCT_SRC`（抽象図形のプレースホルダー
    // 画像で実際の購入者写真ではない）を参照している。「購入者が投稿した
    // 写真」のような実態と異なる alt を付けるとスクリーンリーダー利用者に
    // 誤情報を伝えるため（PR #3538 レビュー P2 指摘）、代表画像についても
    // 固有の説明文を持たせず、8 枚全てを重複描画（装飾目的）として空 alt
    // にする。
    let tiles: Vec<Node> = (0..8)
        .map(|_| {
            image::image(
                &ImageProps {
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                },
                vec![("data-blocks-reviews-summary-split-thumbnail", "")],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-reviews-summary-split-thumbnails")],
        tiles,
    )
}

/// 左列（評価サマリ）。`show_thumbnails` が true の版（B）のみ末尾へ
/// サムネイルグリッドを足す。
fn summary_panel(suffix: &str, show_thumbnails: bool) -> Node {
    let rating_label_id = format!("blocks-reviews-summary-split-{suffix}-summary-rating-label");
    let rating_label_text = format!("平均評価 {AVERAGE_RATING_DISPLAY}（{TOTAL_REVIEW_COUNT} 件）");
    let mut children = vec![
        heading::heading(
            HeadingLevel::H4,
            &HeadingProps::default(),
            vec![],
            vec![text("カスタマーレビュー")],
        ),
        div(
            vec![("class", "blocks-reviews-summary-split-rating-number")],
            vec![
                styled_text::text(
                    &TextProps {
                        size: TextSize::Xl,
                        weight: TextWeight::Bold,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text(AVERAGE_RATING_DISPLAY)],
                ),
                rating_stars(AVERAGE_RATING_ROUNDED, &rating_label_id, rating_label_text),
            ],
        ),
        styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(format!("{TOTAL_REVIEW_COUNT} 件のレビュー"))],
        ),
        div(
            vec![("class", "blocks-reviews-summary-split-breakdown")],
            STAR_BREAKDOWN
                .iter()
                .map(|(stars, percent)| breakdown_row(*stars, *percent))
                .collect(),
        ),
        button::button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                disabled: true,
                ..ButtonProps::default()
            },
            vec![("data-blocks-reviews-summary-split-post", "")],
            vec![text("レビューを書く")],
        ),
        link::root(
            REPO,
            &LinkProps {
                external: true,
                ..LinkProps::default()
            },
            vec![],
            vec![text(REPO_LABEL)],
        ),
    ];
    if show_thumbnails {
        children.push(thumbnail_grid());
    }
    div(
        vec![("class", "blocks-reviews-summary-split-summary")],
        children,
    )
}

/// アバター（author の頭文字を fallback に使う、`profile_detail_datalist`
/// と同型）。
fn review_avatar(author: &str) -> Node {
    avatar::root(
        &AvatarProps {
            size: Size::Md,
            ..AvatarProps::default()
        },
        vec![],
        vec![
            avatar::image(
                ImageStatus::Loaded,
                dummy_assets::AVATAR_SRC,
                author,
                vec![],
            ),
            avatar::fallback(
                ImageStatus::Loaded,
                vec![],
                vec![text(author.chars().take(1).collect::<String>())],
            ),
        ],
    )
}

/// レビュー 1 件（アバター + 投稿者名 + 星評価 + 本文）。`as_card` が true
/// の版（B）は `card::root` で包む。
fn review_item(review: &Review, suffix: &str, index: usize, as_card: bool) -> Node {
    let label_id = format!("blocks-reviews-summary-split-{suffix}-review-{index}-rating-label");
    let rating_label_text = format!("評価 {}.0", review.rating);
    let body = div(
        vec![("class", "blocks-reviews-summary-split-review-body")],
        vec![
            styled_text::text(
                &TextProps {
                    weight: TextWeight::Bold,
                    ..TextProps::default()
                },
                vec![],
                vec![text(review.author)],
            ),
            rating_stars(review.rating, &label_id, rating_label_text),
            styled_text::text(&TextProps::default(), vec![], vec![text(review.body)]),
        ],
    );
    let avatar_node = review_avatar(review.author);
    if as_card {
        card::root(
            CardVariant::Outline,
            vec![("data-blocks-reviews-summary-split-card", "")],
            vec![card::body(vec![], vec![avatar_node, body])],
        )
    } else {
        div(
            vec![("class", "blocks-reviews-summary-split-review")],
            vec![avatar_node, body],
        )
    }
}

/// 右列（レビュー一覧）。`count` 件を [`REVIEWS`] の先頭から使う。
fn review_list(suffix: &str, count: usize, as_card: bool) -> Node {
    let items: Vec<Node> = REVIEWS[0..count]
        .iter()
        .enumerate()
        .map(|(i, r)| review_item(r, suffix, i, as_card))
        .collect();
    let class = if as_card {
        "blocks-reviews-summary-split-cards"
    } else {
        "blocks-reviews-summary-split-list"
    };
    div(vec![("class", class)], items)
}

/// 1 版分の 2 カラム本体（サマリ → レビュー一覧、DOM 順固定）。
fn split_panel(suffix: &str, show_thumbnails: bool, review_count: usize, as_card: bool) -> Node {
    div(
        vec![("class", "blocks-reviews-summary-split-split")],
        vec![
            summary_panel(suffix, show_thumbnails),
            review_list(suffix, review_count, as_card),
        ],
    )
}

/// caption（並記された各版の見出し）。`docs-content` 配下のページ見出し
/// タイポグラフィを継承させないため、`h3` ではなく `p` + 専用クラスで
/// 控えめなスタイルを明示する（`product_overview_gallery_split` 等と
/// 同型の判断。PR #3538 レビュー指摘対応）。
fn caption(label: &str) -> Node {
    p(
        vec![("class", "blocks-reviews-summary-split-caption")],
        vec![text(label)],
    )
}

/// `reviews-summary-split` の Demo 本体（2 版併記）。呼び出しごとに
/// 同一の `Node` を返す純関数。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-reviews-summary-split-stack")],
        vec![
            caption("代表構成（評価サマリ + 割合バー + レビュー 3 件、対応表 ID R1219）"),
            split_panel("a", false, 3, false),
            caption("サムネイル付きサマリ + カード化されたレビュー 6 件（対応表 ID R0215）"),
            split_panel("b", true, 6, true),
        ],
    )
}
```

## 原案差分メモ

- **版 A（代表構成・対応表 ID R1219）**: 平均評価・星別割合バー 5 本・
  「レビューを書く」導線を持つサマリと、区切り線のみのレビュー 3 件を
  並べています。半星の表示は `rating-group` の API に無いため、平均
  評価は切り捨て整数の星と数値テキストの併記で表現しています
- **版 B（集約元・対応表 ID R0215）**: 版 A のサマリへサムネイル画像
  8 枚のグリッドを追加し、レビューを `card` で囲んだ 6 件に増やして
  います
- **参照素材が閲覧不能**: 本イシュー着手時点で `_/blocks-intake/` の
  参照ファイルが worktree に存在しなかったため、対応表 ID（R1219・
  R0215）のみを根拠に実装しています。列幅・余白等の見た目の細部は
  未調整で、参照素材を閲覧できる環境での最終調整はスコープ外です

関連情報: [Heading](../themes/heading.md) / [Rating Group](../themes/rating-group.md) /
[Progress](../themes/progress.md) / [Avatar](../themes/avatar.md) /
[Text](../themes/text.md) / [Link](../themes/link.md) /
[Button](../themes/button.md) / [Image](../themes/image.md) /
[Card](../themes/card.md)
