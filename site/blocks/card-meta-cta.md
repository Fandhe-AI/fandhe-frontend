# card-meta-cta

`card` / `badge` / `heading` / `text` / `list` / `icon` / `button` /
`rating-group` / `number-input` / `separator` の 10 部品を合成した、
題名・分類・説明・要点を縦に並べ下端に主操作ボタンを置く「情報＋CTA」
カードの合成例です。求人・料金プラン・商品の 3 例を同じ骨格で横並びに
表示し、狭い幅では 1 列に積みます。

いずれも `<form>` を持たず、送信処理・状態機械のない静的な表示例です。
商品版の数量入力・評価は readonly の固定値表示で、実行時の入力連動は
行いません（docs サイトは JS ハイドレーションを一切行わないため）。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::number_input::{self, NumberInputFlags};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::Size;

/// 評価ラベル（`rating_group::label` の `id`）。3 枚のカードを同一ページへ
/// 並記するため固定文字列にする（block 内では商品カード 1 枚のみが評価を
/// 持つため重複しない）。
const RATING_LABEL_ID: &str = "blocks-card-meta-cta-rating-label";
/// 数量入力の `id`（`number_input::label`/`input` が共有）。
const QUANTITY_INPUT_ID: &str = "blocks-card-meta-cta-quantity";

/// 装飾用の自作幾何アイコン（lucide 等の既存アイコンセットの path を
/// 複製しないための単純図形、`careers_card_grid::geo_icon` と同型）。
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

/// 位置ピンの幾何アイコン（求人カードの勤務地行）。
fn location_icon() -> Node {
    geo_icon("M12 21s-7-6.5-7-11a7 7 0 0114 0c0 4.5-7 11-7 11z M12 12a2 2 0 100-4 2 2 0 000 4z")
}

/// 時計の幾何アイコン（求人カードの雇用形態行）。
fn clock_icon() -> Node {
    geo_icon("M12 3a9 9 0 100 18 9 9 0 000-18z M12 7v5l4 2")
}

/// カレンダーの幾何アイコン（求人カードの募集期間行）。
fn calendar_icon() -> Node {
    geo_icon("M4 5h16v15H4z M4 9h16 M8 3v4 M16 3v4")
}

/// チェックの幾何アイコン（料金プランカードの機能一覧、装飾用）。
fn check_icon() -> Node {
    geo_icon("M4 12l5 5L20 6")
}

/// 仕様タグの幾何アイコン（商品カードの仕様一覧、装飾用）。
fn spec_icon() -> Node {
    geo_icon("M4 7h16 M4 12h10 M4 17h13")
}

/// 要点リストの 1 行（アイコン + 本文テキスト。3 例で共用する）。
fn meta_item(icon_node: Node, label: &str) -> Node {
    list::item(
        vec![],
        vec![list::indicator(vec![], vec![icon_node]), text(label)],
    )
}

/// 3 例が共有するカード骨格（分類 badge + 題名 + 説明 + 要点リスト +
/// 全幅 CTA ボタン）。`extras` は要点リストと CTA の間に挿入する追加
/// ノード（商品カードの評価・数量入力）。
fn meta_cta_card(
    badge_label: &str,
    title: &str,
    description: &str,
    items: Vec<(Node, &str)>,
    extras: Vec<Node>,
    cta_label: &str,
) -> Node {
    let list_items: Vec<Node> = items
        .into_iter()
        .map(|(icon_node, label)| meta_item(icon_node, label))
        .collect();

    let mut body_children = vec![
        styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(description)],
        ),
        separator::separator(&SeparatorProps::default(), vec![]),
        list::root(ListType::default(), ListVariant::Plain, vec![], list_items),
    ];
    body_children.extend(extras);

    card::root(
        CardProps::default(),
        vec![("data-blocks-card-meta-cta-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    badge::badge(&BadgeProps::default(), vec![], vec![text(badge_label)]),
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Lg,
                            weight: HeadingWeight::Semibold,
                        },
                        vec![],
                        vec![text(title)],
                    ),
                ],
            ),
            card::body(vec![("class", "blocks-card-meta-cta-body")], body_children),
            card::footer(
                vec![("class", "blocks-card-meta-cta-footer")],
                vec![button(
                    &ButtonProps::default(),
                    vec![("data-blocks-card-meta-cta-cta", "")],
                    vec![text(cta_label)],
                )],
            ),
        ],
    )
}

/// 求人カード（代表構成、対応表 ID R0024）。
fn job_card() -> Node {
    meta_cta_card(
        "エンジニアリング",
        "フロントエンドエンジニア",
        "描画コアと UI コンポーネント層の設計・実装を担当します。",
        vec![
            (location_icon(), "勤務地：リモート"),
            (clock_icon(), "雇用形態：正社員"),
            (calendar_icon(), "募集期間：通年"),
        ],
        vec![],
        "応募する",
    )
}

/// 料金プランカード（集約元 R0028: 機能一覧）。プラン名・価格は
/// `crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS` の 2 番目
/// （"Growth"/"$29"）を使う。
fn plan_card() -> Node {
    let price_row = styled_text::text(
        &TextProps {
            size: TextSize::Xl2,
            weight: TextWeight::Bold,
            ..TextProps::default()
        },
        vec![],
        vec![
            text(crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS[1].1),
            text(" / 月"),
        ],
    );
    // 価格行は要点リストの前に差し込むため、meta_cta_card の description
    // 直後（body_children 先頭）へ追加する代わりに items 経由ではなく
    // extras 経由で渡す（要点リストは機能一覧のみに限定するため）。
    let items = vec![
        (check_icon(), "エディタ統合"),
        (check_icon(), "無制限プロジェクト"),
        (check_icon(), "優先サポート"),
    ];
    meta_cta_card(
        "おすすめ",
        crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS[1].0,
        "チームでの本格運用に必要な機能をまとめたプランです。",
        items,
        vec![price_row],
        "このプランを選ぶ",
    )
}

/// 商品カード（集約元 R0025: 数量入力＋評価。readonly の静的表示、
/// モジュール doc「readonly 静的表示」節参照）。
fn product_card() -> Node {
    let price_row = styled_text::text(
        &TextProps {
            size: TextSize::Xl2,
            weight: TextWeight::Bold,
            ..TextProps::default()
        },
        vec![],
        vec![text("$48")],
    );

    let rating_props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let rating_group_state = RatingGroup::new(5, Some(4), true);
    let rating_label = rating_group::label(
        &rating_props,
        Some(RATING_LABEL_ID),
        vec![],
        vec![text("評価 4.0（128 件）")],
    );
    let rating_items: Vec<Node> = (1..=rating_group_state.count())
        .map(|i| {
            rating_group::item(
                i,
                RatingItemFlags {
                    checked: rating_group_state.is_checked(i),
                    highlighted: rating_group_state.is_highlighted(i),
                    disabled: false,
                    readonly: true,
                },
                &format!("{i} star{}", if i == 1 { "" } else { "s" }),
                vec![],
                vec![],
            )
        })
        .collect();
    let rating_control =
        rating_group::control(&rating_props, Some(RATING_LABEL_ID), vec![], rating_items);
    let rating = rating_group::root(
        Size::Sm,
        ColorPalette::Accent,
        &rating_props,
        vec![("data-blocks-card-meta-cta-rating", "")],
        vec![rating_label, rating_control],
    );

    let quantity_flags = NumberInputFlags {
        readonly: true,
        ..NumberInputFlags::default()
    };
    let quantity_input = number_input::root(
        Size::Md,
        false,
        false,
        true,
        vec![("data-blocks-card-meta-cta-quantity", "")],
        vec![
            number_input::label(
                quantity_flags,
                Some(QUANTITY_INPUT_ID),
                vec![],
                vec![text("数量")],
            ),
            number_input::control(
                quantity_flags,
                vec![],
                vec![
                    number_input::decrement_trigger(
                        Some(QUANTITY_INPUT_ID),
                        true,
                        vec![],
                        vec![text("-")],
                    ),
                    number_input::input(
                        "quantity",
                        Some(QUANTITY_INPUT_ID),
                        Some("1"),
                        "1",
                        "10",
                        quantity_flags,
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some(QUANTITY_INPUT_ID),
                        true,
                        vec![],
                        vec![text("+")],
                    ),
                ],
            ),
        ],
    );

    meta_cta_card(
        "新着",
        "デスクマット Pro",
        "手首の負担を抑える傾斜構造の作業用デスクマットです。",
        vec![
            (spec_icon(), "サイズ：90 × 40 cm"),
            (spec_icon(), "素材：撥水コーティング表面"),
        ],
        vec![price_row, rating, quantity_input],
        "カートに追加",
    )
}

/// `card-meta-cta` の Demo 本体（求人・料金プラン・商品の 3 例を横並びに
/// する。狭い幅では 1 列へ積む、[`LAYOUT_CSS`] 参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-card-meta-cta-layout")],
        vec![job_card(), plan_card(), product_card()],
    )
}
```

## 原案差分メモ

- 代表構成（求人）・商品版（数量入力＋評価）・料金プラン版（機能一覧）の
  3 バリエーションを 1 つの共有骨格へまとめ、3 枚並記で示しました。
- 数量入力と評価はいずれも readonly の静的表示にしています。
- 参照元のアイコン・配色は持ち込まず、線画の自作幾何アイコンにしています。
