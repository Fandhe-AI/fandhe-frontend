# product-overview-tabs-below

`fandhe-frontend-pre-styled-ui` の `image` / `heading` / `text` /
`rating-group` / `button` / `list` / `link` / `avatar` 部品を合成した、
商品詳細ページの構成例です。Blocks セクションは新規部品を追加するもの
ではなく、既存の Themes/Primitives 部品を組み合わせた実例集であることに
注意してください（主参照は対応表 ID R1179。出典の固有名・ファイル名は
記載しません）。

全体の規模が大きいため、骨格と主要領域（本ページ）と残り領域（イシュー
#3075）に分割しています。本ページは上段 2 カラム（商品画像 + 商品名・
評価・説明・購入ボタン・特徴リスト・共有リンク）と、下段の全幅タブ
（レビュー一覧 / よくある質問 / 利用条件、静的表示では「レビュー」を
選択状態にする）までを扱います。狭い幅では上段が縦に積み重なり、タブは
全幅のまま下に表示されます。

無 JS の docs サイトでは実物の `tabs` でカテゴリを切り替える経路が
作れないため、下段のタブ列は見た目のみを示す装飾（クリック・キーボード
操作はできません、ラベルは `aria-hidden` で装飾扱いです）とし、選択中の
「レビュー」パネルのみを見出し付きで下に描画します。「よくある質問」
「利用条件」タブの内容併記は後半のイシュー #3075 で扱います。購入ボタン
は無 JS のため押しても何も起きないボタンを操作可能なまま残さない方針で
`disabled` の静的表示にしています。`<form>` 要素は一切持たず、データの
取得・送信・状態管理を行いません。文言・商品名・レビュー本文・人名は
すべて独自に書いた架空のものであり、実企業名・実クレデンシャル・PII を
含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el_owned, h3, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 商品の評価ラベル（`rating_group::label` の `id`）。
const RATING_LABEL_ID: &str = "blocks-product-overview-tabs-below-rating-label";

/// 本リポジトリの固定 URL（共有リンクの href、他 block と同型の判断）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// レビュー 1 件分のダミーデータ（人名は [`dummy_assets::PERSON_NAMES`] の
/// 索引、評価は 5 段階、本文は架空のもの）。
const REVIEWS: [(usize, u32, &str); 3] = [
    (
        0,
        5,
        "手首の負担が減り、長時間の作業がかなり楽になりました。",
    ),
    (1, 4, "質感は良いですが、もう少し大きいサイズも欲しいです。"),
    (
        2,
        5,
        "梱包も丁寧で、届いてすぐに使い始められました。おすすめです。",
    ),
];

/// レビュー `index` 件目の評価ラベル id（全件を通じて一意にする、
/// モジュール doc「id を block 固有定数にする理由」節）。
fn review_rating_label_id(index: usize) -> String {
    format!("blocks-product-overview-tabs-below-review-{index}-rating-label")
}

/// カテゴリラベルのみを装飾として示す非対話タブ列（モジュール doc「実物の
/// `tabs::tabs` を使わない」節）。`role`/`tabindex`/`<button>` を一切持たず、
/// `selected` に一致するラベルだけ `data-state="active"` にする。各ラベルは
/// `aria-hidden` で支援技術のツリーから除外する。
fn static_tab_list(selected: &str) -> Node {
    let labels = [
        ("reviews", "レビュー"),
        ("faq", "よくある質問"),
        ("terms", "利用条件"),
    ];
    el_owned(
        "div",
        vec![(
            "class".to_string(),
            "blocks-product-overview-tabs-below-tablist".to_string(),
        )],
        labels
            .iter()
            .map(|(value, label)| {
                let state = if *value == selected {
                    "active"
                } else {
                    "inactive"
                };
                el_owned(
                    "div",
                    vec![
                        (
                            "class".to_string(),
                            "blocks-product-overview-tabs-below-tab".to_string(),
                        ),
                        ("data-state".to_string(), state.to_string()),
                        ("aria-hidden".to_string(), "true".to_string()),
                    ],
                    vec![text(*label)],
                )
            })
            .collect(),
    )
}

/// 商品画像（左列）。
fn product_image() -> Node {
    image(
        &ImageProps {
            aspect_ratio: AspectRatio::Square,
            ..ImageProps::new(dummy_assets::PRODUCT_SRC, "デスクマット Pro の商品画像")
        },
        vec![("data-blocks-product-overview-tabs-below-image", "")],
    )
}

/// 商品の評価（readonly の静的表示。無 JS のため操作不能、
/// `card_meta_cta::product_card` と同じ判断）。
fn product_rating() -> Node {
    let props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let state = RatingGroup::new(5, Some(4), true);
    let label = rating_group::label(
        &props,
        Some(RATING_LABEL_ID),
        vec![],
        vec![text("評価 4.0（86 件）")],
    );
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
    let control = rating_group::control(&props, Some(RATING_LABEL_ID), vec![], items);
    rating_group::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-product-overview-tabs-below-rating", "")],
        vec![label, control],
    )
}

/// 特徴リストの 1 行。
fn feature_item(label: &str) -> Node {
    list::item(vec![], vec![text(label)])
}

/// 共有リンク 1 件（架空ラベル、固定 href、モジュール doc「共有リンクは
/// 架空の固定 href」節）。
fn share_link(label: &str) -> Node {
    link::root(REPO, &LinkProps::default(), vec![], vec![text(label)])
}

/// 上段・情報列（商品名 + 評価 + 説明 + 購入ボタン + 特徴リスト + 共有
/// リンク）。
fn info_column() -> Node {
    div(
        vec![("class", "blocks-product-overview-tabs-below-info")],
        vec![
            heading(
                HeadingLevel::H2,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Semibold,
                },
                vec![],
                vec![text("デスクマット Pro")],
            ),
            product_rating(),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "手首の負担を抑える傾斜構造の作業用デスクマットです。撥水コーティング表面で日常使いにも適しています。",
                )],
            ),
            button(
                &ButtonProps {
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-product-overview-tabs-below-cta", "")],
                vec![text("カートに追加")],
            ),
            list::root(
                ListType::default(),
                ListVariant::Plain,
                vec![("data-blocks-product-overview-tabs-below-features", "")],
                vec![
                    feature_item("サイズ：90 × 40 cm"),
                    feature_item("素材：撥水コーティング表面"),
                    feature_item("お届け目安：3〜5 営業日"),
                ],
            ),
            div(
                vec![("class", "blocks-product-overview-tabs-below-share")],
                vec![share_link("共有 A"), share_link("共有 B")],
            ),
        ],
    )
}

/// 上段（商品画像 + 情報列、狭幅では縦積み、[`LAYOUT_CSS`] 参照）。
fn top_section() -> Node {
    div(
        vec![("class", "blocks-product-overview-tabs-below-top")],
        vec![product_image(), info_column()],
    )
}

/// レビュー 1 件分（アバター + 氏名 + 評価 + 本文）。
fn review_item((index, rating, body): (usize, u32, &str)) -> Node {
    let name = dummy_assets::PERSON_NAMES[index % dummy_assets::PERSON_NAMES.len()];
    let initials: String = name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .collect();
    let label_id = review_rating_label_id(index);

    let props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let state = RatingGroup::new(5, Some(rating), true);
    let label = rating_group::label(
        &props,
        Some(label_id.as_str()),
        vec![],
        vec![text(format!("評価 {rating}.0"))],
    );
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
    let control = rating_group::control(&props, Some(label_id.as_str()), vec![], items);
    let rating_node = rating_group::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![],
        vec![label, control],
    );

    div(
        vec![("class", "blocks-product-overview-tabs-below-review")],
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
                vec![("class", "blocks-product-overview-tabs-below-review-body")],
                vec![
                    div(
                        vec![("class", "blocks-product-overview-tabs-below-review-name")],
                        vec![text(name)],
                    ),
                    rating_node,
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(body)],
                    ),
                ],
            ),
        ],
    )
}

/// 下段・全幅タブ（静的タブ列 + レビュー一覧パネル、モジュール doc冒頭節
/// 参照。「よくある質問」「利用条件」パネルの併記は後半 #3075 で扱う）。
fn tabs_section() -> Node {
    div(
        vec![("class", "blocks-product-overview-tabs-below-tabs")],
        vec![
            static_tab_list("reviews"),
            div(
                vec![("class", "blocks-product-overview-tabs-below-reviews")],
                vec![
                    h3(vec![], vec![text("レビュー")]),
                    div(
                        vec![("class", "blocks-product-overview-tabs-below-review-list")],
                        REVIEWS.iter().copied().map(review_item).collect(),
                    ),
                ],
            ),
        ],
    )
}

/// `product-overview-tabs-below` の Demo 本体（上段 2 カラム + 下段全幅
/// タブ、モジュール doc冒頭節参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-product-overview-tabs-below-layout")],
        vec![top_section(), tabs_section()],
    )
}
```

## 原案差分メモ

- 本ページは全体計画の前半（骨格と主要領域）に相当します。「よくある
  質問」「利用条件」タブのパネル併記（非選択タブのプレビュー）・状態
  違いの並記（在庫切れ・評価なし等）・集約元差分の扱いは、後続のイシュー
  #3075 で仕上げます。
- タブは実物の `tabs::tabs` を使わず、非対話の視覚的タブ列 + 選択中
  パネルの常時可視表示にしています。理由は `faq-tabbed-accordion` と
  同様、無 JS の docs サイトでは実物のタブ切り替えが機能しないためです。
- 購入ボタンは `disabled` の静的表示にしています。無 JS のため押しても
  何も起きないボタンを操作可能なまま残さないための判断です。
- 配色・余白・角丸は既存のテーマトークンに従っています。

関連情報: [Image](../themes/image.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Rating Group](../themes/rating-group.md) /
[Button](../themes/button.md) / [List](../themes/list.md) /
[Link](../themes/link.md) / [Avatar](../themes/avatar.md)
