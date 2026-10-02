# reviews-card-grid

見出し・平均評価・投稿ボタンを並べたヘッダ行、区切り線、レビューカード
6 枚のグリッド、末尾中央の「さらに読み込む」ボタンで構成するブロック
です。`heading` / `rating-group` / `button` / `separator` / `card` /
`text` の 6 部品を合成します。Blocks は既存部品の合成例であり、新しい
UI 部品は追加しません。

主参照は対応表 ID R0214 です。

広い幅ではカードグリッドを 2 列、狭い幅では 1 列に畳みます。狭幅では
ヘッダ行（見出し・平均評価・投稿ボタン）も折り返します。

投稿者名・レビュー件名・本文・日付はすべて架空のデータであり、実在の
企業・ブランド・PII・実クレデンシャルは含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。投稿
ボタン・もっと見るボタンはいずれも `disabled` の静的ボタンで、送信
処理・状態管理は一切持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// レビュー 1 件分のダミーデータ（架空、実在の企業・人物とは無関係）。
struct Review {
    author: &'static str,
    title: &'static str,
    body: &'static str,
    /// 評価（1〜5）。
    rating: u8,
    date: &'static str,
}

/// レビュー 6 件（[`dummy_assets::PERSON_NAMES`] の先頭 6 件を投稿者名に
/// 流用する）。
const REVIEWS: [Review; 6] = [
    Review {
        author: dummy_assets::PERSON_NAMES[0],
        title: "期待どおりの使い心地",
        body: "思っていたより軽くて、毎日気軽に持ち歩けています。",
        rating: 5,
        date: "2026-08-02",
    },
    Review {
        author: dummy_assets::PERSON_NAMES[1],
        title: "コスパが良い",
        body: "この価格帯でこの品質なら十分満足です。",
        rating: 4,
        date: "2026-08-10",
    },
    Review {
        author: dummy_assets::PERSON_NAMES[2],
        title: "配送が早かった",
        body: "注文の翌日に届いて驚きました。梱包も丁寧でした。",
        rating: 5,
        date: "2026-08-14",
    },
    Review {
        author: dummy_assets::PERSON_NAMES[3],
        title: "サイズ感に注意",
        body: "写真よりやや小さく感じたので、サイズ表は要確認です。",
        rating: 3,
        date: "2026-08-21",
    },
    Review {
        author: dummy_assets::PERSON_NAMES[4],
        title: "リピート購入しました",
        body: "前回購入分が良かったので、色違いを追加で買いました。",
        rating: 5,
        date: "2026-09-01",
    },
    Review {
        author: dummy_assets::PERSON_NAMES[5],
        title: "手入れがしやすい",
        body: "お手入れ方法が簡単で、長く使えそうな印象です。",
        rating: 4,
        date: "2026-09-05",
    },
];

/// 全体の平均評価（丸めた表示用。実データではなくデモ固定値）。
const AVERAGE_RATING: u8 = 4;
const TOTAL_REVIEW_COUNT: u32 = 128;

/// 評価行（readonly `rating-group`。件数込みラベルで明文化する、
/// `product_list_rich_cards.rs::rating_row` と同型）。
fn rating_row(rating: u8, count: u32, label_id: &str) -> Node {
    let props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let state = RatingGroup::new(5, Some(u32::from(rating)), true);
    let label_text = format!("評価 {rating}.0（{count} 件）");
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

/// ヘッダ行（見出し・平均評価・投稿ボタン）。
fn header() -> Node {
    div(
        vec![("class", "blocks-reviews-card-grid-header")],
        vec![
            div(
                vec![("class", "blocks-reviews-card-grid-header-summary")],
                vec![
                    heading::heading(
                        HeadingLevel::H2,
                        &HeadingProps::default(),
                        vec![],
                        vec![text("お客様のレビュー")],
                    ),
                    rating_row(
                        AVERAGE_RATING,
                        TOTAL_REVIEW_COUNT,
                        "blocks-reviews-card-grid-average-rating-label",
                    ),
                ],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-reviews-card-grid-post", "")],
                vec![text("レビューを書く")],
            ),
        ],
    )
}

/// レビューカード 1 枚（評価 → 件名 → 本文 → 投稿者・日付）。
fn review_card(review: &Review, index: usize) -> Node {
    let rating_label_id = format!("blocks-reviews-card-grid-{index}-rating-label");
    card::root(
        CardVariant::Outline,
        vec![("data-blocks-reviews-card-grid-card", "")],
        vec![card::body(
            vec![],
            vec![
                rating_row(review.rating, 1, &rating_label_id),
                card::title(vec![], vec![text(review.title)]),
                card::description(vec![], vec![text(review.body)]),
                styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text(format!("{} ・ {}", review.author, review.date))],
                ),
            ],
        )],
    )
}

/// レビューカードグリッド（6 枚、1→2 列）。
fn grid() -> Node {
    div(
        vec![("class", "blocks-reviews-card-grid-grid")],
        REVIEWS
            .iter()
            .enumerate()
            .map(|(i, r)| review_card(r, i))
            .collect(),
    )
}

/// 末尾中央の「さらに読み込む」ボタン。
fn load_more() -> Node {
    div(
        vec![("class", "blocks-reviews-card-grid-more")],
        vec![button::button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                disabled: true,
                ..ButtonProps::default()
            },
            vec![("data-blocks-reviews-card-grid-load-more", "")],
            vec![text("さらに読み込む")],
        )],
    )
}

/// `reviews-card-grid` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（`crate::blocks` モジュール doc「静的表示」節）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-reviews-card-grid-layout")],
        vec![
            header(),
            separator::separator(&SeparatorProps::default(), vec![]),
            grid(),
            load_more(),
        ],
    )
}
```

## 原案差分メモ

集約元は主参照 R0214 の 1 件のみです。

- **R0214（主参照・唯一の集約元）**: ヘッダ行 + 6 件グリッド + もっと
  見るの構成をそのまま採用しています。集約元が 1 件のため並記すべき
  差分はありません
- **ボタン**: 投稿ボタン・もっと見るボタンはいずれもリンクではなく
  `disabled` の静的ボタンです。無 JS のため押しても表示が変わらない
  ボタンを操作可能なまま残していません
- **参照素材が閲覧不能**: 本イシュー着手時点で `_/blocks-intake/` の
  参照ファイルが worktree に存在しなかったため、対応表 ID（R0214）
  のみを根拠に実装しています。列幅・余白等の見た目の細部は未調整で、
  参照素材を閲覧できる環境での最終調整はスコープ外です

関連情報: [Heading](../themes/heading.md) / [Rating Group](../themes/rating-group.md) /
[Button](../themes/button.md) / [Separator](../themes/separator.md) /
[Card](../themes/card.md) / [Text](../themes/text.md)
