# product-overview-tabs-below

`fandhe-frontend-pre-styled-ui` の `image` / `heading` / `text` /
`rating-group` / `button` / `list` / `link` / `avatar` 部品を合成した、
商品詳細ページの構成例です。Blocks セクションは新規部品を追加するもの
ではなく、既存の Themes/Primitives 部品を組み合わせた実例集であることに
注意してください（主参照は対応表 ID R1179。出典の固有名・ファイル名は
記載しません）。

上段 2 カラム（商品画像 + 商品名・評価・説明・購入ボタン・特徴リスト・
共有リンク）と、下段の全幅タブ（レビュー一覧 / よくある質問 / 利用条件）
から成ります。静的表示では「レビュー」を選択状態にし、「よくある質問」
「利用条件」は「『〜』タブを選択した場合のプレビュー」キャプション付きで
常時可視のまま併記します。狭い幅では上段が縦に積み重なり、タブは全幅の
まま下に表示されます。

無 JS の docs サイトでは実物の `tabs` でカテゴリを切り替える経路が
作れないため、下段のタブ列は見た目のみを示す装飾（クリック・キーボード
操作はできません、ラベルは `aria-hidden` で装飾扱いです）とし、選択中の
「レビュー」パネルを見出し付きで下に描画したうえで、残り 2 タブの内容も
プレビューとして併記します（`hidden` は一切使いません）。購入ボタンは
無 JS のため押しても何も起きないボタンを操作可能なまま残さない方針で
`disabled` の静的表示にしています。`<form>` 要素は一切持たず、データの
取得・送信・状態管理を行いません。文言・商品名・レビュー本文・人名は
すべて独自に書いた架空のものであり、実企業名・実クレデンシャル・PII を
含みません。

Demo は「代表構成」「在庫切れ・評価なし」の 2 版をキャプション付きで
縦に並べています。詳細は「原案差分メモ」節を参照してください。

## Rust コード

```rust
use fandhe_frontend_core::{div, el_owned, h3, p, section, text, Node};
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

/// 「よくある質問」パネルのダミー Q&A（架空文、モジュール doc「使用部品」
/// 節）。
const FAQS: [(&str, &str); 3] = [
    (
        "サイズは選べますか",
        "現在は 90 × 40 cm の 1 サイズのみの展開です。",
    ),
    (
        "水洗いはできますか",
        "表面は撥水コーティングのため軽い汚れは水拭きで対応できます。丸洗いは推奨していません。",
    ),
    (
        "滑り止めは付いていますか",
        "裏面全体に滑り止め加工を施しており、デスク上でずれにくい仕様です。",
    ),
];

/// 「利用条件」パネルの箇条書き項目（架空文）。
const TERMS: [&str; 3] = [
    "返品は到着後 14 日以内、未使用・未開封の商品に限り受け付けます。",
    "初期不良は 1 年間の保証対象です。通常使用による経年劣化は対象外です。",
    "配送は国内のみの対応です。離島・一部地域は追加日数がかかる場合があります。",
];

/// Demo 版ごとの状態フラグ（モジュール doc「状態違いの並記」節）。
/// `in_stock`/`rated` の 2 bool のみを持ち、列挙型・builder は作らない
/// （現状 2 版のみのため）。
struct Variant {
    /// 在庫あり（false のとき購入ボタンを「在庫切れ」に差し替える）。
    in_stock: bool,
    /// 評価集計あり（false のとき「まだ評価はありません」に差し替え、
    /// レビューパネルも「レビューはまだありません。」に差し替える）。
    rated: bool,
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
/// `card_meta_cta::product_card` と同じ判断）。`rated` が false の版
/// （モジュール doc「状態違いの並記」節）は `rating_group::root` を
/// 一切出力せず「まだ評価はありません」の文だけを返す。
fn product_rating(rated: bool) -> Node {
    if !rated {
        return styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![("data-blocks-product-overview-tabs-below-rating", "")],
            vec![text("まだ評価はありません")],
        );
    }
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

/// 在庫切れ注記（`variant.in_stock` が false のときのみ [`info_column`]
/// 冒頭へ挿入する、モジュール doc「状態違いの並記」節）。
fn stock_notice() -> Node {
    styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            size: TextSize::Sm,
            ..TextProps::default()
        },
        vec![],
        vec![text("現在在庫切れです。入荷時期は未定です。")],
    )
}

/// 上段・情報列（商品名 + 評価 + 説明 + 購入ボタン + 特徴リスト + 共有
/// リンク）。`variant` により評価表示・購入ボタンのラベルが切り替わる
/// （モジュール doc「状態違いの並記」節）。
fn info_column(variant: &Variant) -> Node {
    let mut children = vec![heading(
        HeadingLevel::H2,
        &HeadingProps {
            size: HeadingSize::Xl2,
            weight: HeadingWeight::Semibold,
        },
        vec![],
        vec![text("デスクマット Pro")],
    )];
    if !variant.in_stock {
        children.push(stock_notice());
    }
    children.push(product_rating(variant.rated));
    children.push(styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "手首の負担を抑える傾斜構造の作業用デスクマットです。撥水コーティング表面で日常使いにも適しています。",
        )],
    ));
    children.push(button(
        &ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        },
        vec![("data-blocks-product-overview-tabs-below-cta", "")],
        vec![text(if variant.in_stock {
            "カートに追加"
        } else {
            "在庫切れ"
        })],
    ));
    children.push(list::root(
        ListType::default(),
        ListVariant::Plain,
        vec![("data-blocks-product-overview-tabs-below-features", "")],
        vec![
            feature_item("サイズ：90 × 40 cm"),
            feature_item("素材：撥水コーティング表面"),
            feature_item("お届け目安：3〜5 営業日"),
        ],
    ));
    children.push(div(
        vec![("class", "blocks-product-overview-tabs-below-share")],
        vec![share_link("共有 A"), share_link("共有 B")],
    ));
    div(
        vec![("class", "blocks-product-overview-tabs-below-info")],
        children,
    )
}

/// 上段（商品画像 + 情報列、狭幅では縦積み、[`LAYOUT_CSS`] 参照）。
fn top_section(variant: &Variant) -> Node {
    div(
        vec![("class", "blocks-product-overview-tabs-below-top")],
        vec![product_image(), info_column(variant)],
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

/// 版キャプション（`ai_chat_code_preview::caption` と同型、素の `<p>` で
/// `heading` 部品を使わない）。
fn caption(label: &str) -> Node {
    p(
        vec![("class", "blocks-product-overview-tabs-below-caption")],
        vec![text(label)],
    )
}

/// 非選択タブ（「よくある質問」「利用条件」）のプレビュー併記
/// （`faq_tabbed_accordion::category_preview` と同型の
/// 「『〜』タブを選択した場合のプレビュー」キャプション付き常時可視節、
/// モジュール doc「実物の `tabs::tabs` を使わない」節参照）。
fn panel_preview(label: &str, body: Node) -> Node {
    div(
        vec![("class", "blocks-product-overview-tabs-below-preview")],
        vec![
            h3(
                vec![],
                vec![text(format!("「{label}」タブを選択した場合のプレビュー"))],
            ),
            body,
        ],
    )
}

/// 「よくある質問」パネル本体（質問は `H4`/`Sm` の見出し、回答は
/// `styled_text::text`、[`FAQS`] 定数参照）。
fn faq_panel() -> Node {
    div(
        vec![("class", "blocks-product-overview-tabs-below-faq")],
        FAQS.iter()
            .map(|(question, answer)| {
                div(
                    vec![("class", "blocks-product-overview-tabs-below-faq-item")],
                    vec![
                        heading(
                            HeadingLevel::H4,
                            &HeadingProps {
                                size: HeadingSize::Sm,
                                ..HeadingProps::default()
                            },
                            vec![],
                            vec![text(*question)],
                        ),
                        styled_text::text(
                            &TextProps {
                                size: TextSize::Sm,
                                ..TextProps::default()
                            },
                            vec![],
                            vec![text(*answer)],
                        ),
                    ],
                )
            })
            .collect(),
    )
}

/// 「利用条件」パネル本体（導入文 + 箇条書き 3 項目、[`TERMS`] 定数参照）。
fn terms_panel() -> Node {
    div(
        vec![],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("ご購入前に以下の利用条件をご確認ください。")],
            ),
            list::root(
                ListType::default(),
                ListVariant::Marker,
                vec![],
                TERMS
                    .iter()
                    .map(|term| list::item(vec![], vec![text(*term)]))
                    .collect(),
            ),
        ],
    )
}

/// 下段・全幅タブ（静的タブ列 + レビュー一覧パネル + 「よくある質問」
/// 「利用条件」プレビュー併記、モジュール doc「実物の `tabs::tabs` を
/// 使わない」節参照）。`variant.rated` が false のときレビューパネルは
/// 一覧の代わりに「レビューはまだありません。」の文を返す。
fn tabs_section(variant: &Variant) -> Node {
    let reviews_body: Node = if variant.rated {
        div(
            vec![("class", "blocks-product-overview-tabs-below-review-list")],
            REVIEWS.iter().copied().map(review_item).collect(),
        )
    } else {
        styled_text::text(
            &TextProps::default(),
            vec![],
            vec![text("レビューはまだありません。")],
        )
    };
    div(
        vec![("class", "blocks-product-overview-tabs-below-tabs")],
        vec![
            static_tab_list("reviews"),
            div(
                vec![("class", "blocks-product-overview-tabs-below-reviews")],
                vec![h3(vec![], vec![text("レビュー")]), reviews_body],
            ),
            panel_preview("よくある質問", faq_panel()),
            panel_preview("利用条件", terms_panel()),
        ],
    )
}

/// 1 版分の shell（上段 2 カラム + 下段全幅タブ）を組み立てる
/// （モジュール doc「状態違いの並記」節）。
fn shell(variant: &Variant) -> Node {
    div(
        vec![("class", "blocks-product-overview-tabs-below-layout")],
        vec![top_section(variant), tabs_section(variant)],
    )
}

/// `product-overview-tabs-below` の Demo 本体。2 版（代表構成・在庫切れ・
/// 評価なし）をキャプション付きで縦に並べる（モジュール doc「状態違いの
/// 並記」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-product-overview-tabs-below-demo")],
        vec![
            caption("代表構成"),
            section(
                vec![],
                vec![shell(&Variant {
                    in_stock: true,
                    rated: true,
                })],
            ),
            caption("在庫切れ・評価なし"),
            section(
                vec![],
                vec![shell(&Variant {
                    in_stock: false,
                    rated: false,
                })],
            ),
        ],
    )
}
```

## 原案差分メモ

- 対応表の主参照は R1179 の 1 件のみで、参照由来の差分はありません
  （取得手段・出典の固有名は記載しない契約）。参照素材の対応表 ID の
  みを記録し、本ページの Demo が持つ 2 版の並記は参照由来ではなく
  **本 block 固有の状態違い**（在庫あり/なし・評価あり/なし）です。
- タブは実物の `tabs::tabs` を使わず、非対話の視覚的タブ列 + 全パネル
  常時可視表示にしています。理由は `faq-tabbed-accordion` と同様、
  無 JS の docs サイトでは実物のタブ切り替えが機能しないためです。
  選択中の「レビュー」パネルは見出し付きで直下に描画し、「よくある
  質問」「利用条件」は「『〜』タブを選択した場合のプレビュー」
  キャプション付きで縦に併記します（`hidden` は一切使いません）。
- Demo は「代表構成」（在庫あり・評価 4.0・レビュー 3 件）と「在庫切れ・
  評価なし」（購入ボタンを「在庫切れ」に差し替え、評価は「まだ評価は
  ありません」、レビューは「レビューはまだありません。」に差し替え）の
  2 版をキャプション付きで縦に並べています。狭幅版（3 版目）は意図的に
  並記しません: 本 block の上段 2 カラム切替は `@media (min-width: 48rem)`
  の viewport メディアクエリであり、`ai-chat-code-preview` が使う
  `@container` クエリと異なりページ内の並記だけではリサイズなしに
  視覚確認できないためです。
- 購入ボタンは `disabled` の静的表示にしています。無 JS のため押しても
  何も起きないボタンを操作可能なまま残さないための判断です。
- 配色・余白・角丸は既存のテーマトークンに従っています。

関連情報: [Image](../themes/image.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Rating Group](../themes/rating-group.md) /
[Button](../themes/button.md) / [List](../themes/list.md) /
[Link](../themes/link.md) / [Avatar](../themes/avatar.md)
