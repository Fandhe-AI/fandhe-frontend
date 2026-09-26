# comparison-cards

`heading` / `text` / `badge` / `card` / `list` / `icon` の 6 部品を合成し
た、自社製品と競合製品を並べて比較する製品比較カードです。中央寄せの
見出し領域（eyebrow badge・見出し・リード文）の下に、製品カードを横並び
で置きます。各カードは製品名・短い説明・機能ごとの可否をアイコンで示す
リストを持ち、自社カードは枠線と badge で視覚的に強調します。

カード列は既定で 1 列（縦積み）で、幅が広がると 2 列・3 列へ切り替わり
ます。文言はすべて架空のもので、データ取得・送信は行わない静的な表示
例です。`<form>` は使用しません。

集約元は 2 件（対応表 ID R0435: カード 2 枚 / R0436: カード 3 枚）です。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 機能可否の表現。`Text` は「無制限」「10 件」のような文字値表示に使う
/// （参照元にある文字値行を残す判断、モジュール doc 参照）。
enum FeatureValue {
    /// 含まれる（チェックアイコン）。
    Included,
    /// 含まれない（バツアイコン）。
    Excluded,
    /// 文字表示（例: 上限件数）。
    Text(&'static str),
}

/// 機能比較 1 行分（架空、実在の製品・企業とは無関係）。
struct Feature {
    label: &'static str,
    value: FeatureValue,
}

/// 製品カード 1 枚分（架空データ）。
struct ProductCard {
    name: &'static str,
    description: &'static str,
    /// 自社カード（強調表示）かどうか。
    highlighted: bool,
    features: &'static [Feature],
}

/// 2 枚インスタンス（R0435）の製品カード。自社 1 枚 + 競合 1 社。
const TWO_CARDS: [ProductCard; 2] = [
    ProductCard {
        name: "Fandhe Kit",
        description: "既存部品だけで画面を組み立てる、静的サイト向けのツールキットです。",
        highlighted: true,
        features: &[
            Feature {
                label: "プロジェクト数",
                value: FeatureValue::Text("無制限"),
            },
            Feature {
                label: "既定エスケープ",
                value: FeatureValue::Included,
            },
            Feature {
                label: "外部依存ゼロの描画コア",
                value: FeatureValue::Included,
            },
            Feature {
                label: "有償サポート窓口",
                value: FeatureValue::Excluded,
            },
        ],
    },
    ProductCard {
        name: "Verdant Foundry",
        description: "汎用のページビルダーとテンプレート集を提供するツールです。",
        highlighted: false,
        features: &[
            Feature {
                label: "プロジェクト数",
                value: FeatureValue::Text("10 件"),
            },
            Feature {
                label: "既定エスケープ",
                value: FeatureValue::Excluded,
            },
            Feature {
                label: "外部依存ゼロの描画コア",
                value: FeatureValue::Excluded,
            },
            Feature {
                label: "有償サポート窓口",
                value: FeatureValue::Included,
            },
        ],
    },
];

/// 3 枚インスタンス（R0436）の製品カード。自社 1 枚 + 競合 2 社（可否
/// パターンを 2 枚インスタンスと変える）。
const THREE_CARDS: [ProductCard; 3] = [
    ProductCard {
        name: "Fandhe Kit",
        description: "既存部品だけで画面を組み立てる、静的サイト向けのツールキットです。",
        highlighted: true,
        features: &[
            Feature {
                label: "プロジェクト数",
                value: FeatureValue::Text("無制限"),
            },
            Feature {
                label: "既定エスケープ",
                value: FeatureValue::Included,
            },
            Feature {
                label: "SSR/SSG 両対応",
                value: FeatureValue::Included,
            },
            Feature {
                label: "有償サポート窓口",
                value: FeatureValue::Excluded,
            },
        ],
    },
    ProductCard {
        name: "Verdant Foundry",
        description: "汎用のページビルダーとテンプレート集を提供するツールです。",
        highlighted: false,
        features: &[
            Feature {
                label: "プロジェクト数",
                value: FeatureValue::Text("25 件"),
            },
            Feature {
                label: "既定エスケープ",
                value: FeatureValue::Excluded,
            },
            Feature {
                label: "SSR/SSG 両対応",
                value: FeatureValue::Included,
            },
            Feature {
                label: "有償サポート窓口",
                value: FeatureValue::Included,
            },
        ],
    },
    ProductCard {
        name: "Trellisworks Co.",
        description: "SPA 特化のフロントエンド基盤を提供するツールです。",
        highlighted: false,
        features: &[
            Feature {
                label: "プロジェクト数",
                value: FeatureValue::Text("10 件"),
            },
            Feature {
                label: "既定エスケープ",
                value: FeatureValue::Excluded,
            },
            Feature {
                label: "SSR/SSG 両対応",
                value: FeatureValue::Excluded,
            },
            Feature {
                label: "有償サポート窓口",
                value: FeatureValue::Excluded,
            },
        ],
    },
];

/// 可否を表す自作の抽象チェック/バツ図形（意味を持つアイコンのため
/// `label` を指定する。装飾用途の `bento_asymmetric_rows::feature_icon` と
/// 異なり `None` にしない。参照元のアイコン形状・内部識別子は持ち込まない、
/// モジュール doc「可否アイコンに `list::indicator` を使わない理由」節）。
fn feature_value_icon(value: &FeatureValue) -> Node {
    match value {
        FeatureValue::Included => icon(
            &IconProps {
                size: Size::Sm,
                label: Some("含まれる"),
                ..IconProps::default()
            },
            vec![("data-blocks-comparison-cards-value", "included")],
            vec![
                el(
                    "circle",
                    vec![
                        ("cx", "12"),
                        ("cy", "12"),
                        ("r", "9"),
                        ("fill", "none"),
                        ("stroke", "currentColor"),
                        ("stroke-width", "1.5"),
                    ],
                    vec![],
                ),
                el(
                    "path",
                    vec![
                        ("d", "M8 12.5l2.5 2.5L16 9"),
                        ("fill", "none"),
                        ("stroke", "currentColor"),
                        ("stroke-width", "1.5"),
                        ("stroke-linecap", "round"),
                        ("stroke-linejoin", "round"),
                    ],
                    vec![],
                ),
            ],
        ),
        FeatureValue::Excluded => icon(
            &IconProps {
                size: Size::Sm,
                label: Some("含まれない"),
                ..IconProps::default()
            },
            vec![("data-blocks-comparison-cards-value", "excluded")],
            vec![
                el(
                    "path",
                    vec![
                        ("d", "M8 8l8 8"),
                        ("stroke", "currentColor"),
                        ("stroke-width", "1.5"),
                        ("stroke-linecap", "round"),
                    ],
                    vec![],
                ),
                el(
                    "path",
                    vec![
                        ("d", "M16 8l-8 8"),
                        ("stroke", "currentColor"),
                        ("stroke-width", "1.5"),
                        ("stroke-linecap", "round"),
                    ],
                    vec![],
                ),
            ],
        ),
        FeatureValue::Text(_) => span(vec![], vec![]),
    }
}

/// 機能比較 1 行分（`list::item`）。
fn feature_row(feature: &Feature) -> Node {
    let value_node = match &feature.value {
        FeatureValue::Text(label) => span(
            vec![("data-blocks-comparison-cards-value", "text")],
            vec![text(*label)],
        ),
        included_or_excluded => feature_value_icon(included_or_excluded),
    };
    list::item(
        vec![("class", "blocks-comparison-cards-row")],
        vec![
            span(
                vec![("class", "blocks-comparison-cards-feature-label")],
                vec![text(feature.label)],
            ),
            value_node,
        ],
    )
}

/// 製品カード 1 枚（`card::root`）。
fn product_card(product: &ProductCard) -> Node {
    let (variant, state) = if product.highlighted {
        (CardVariant::Elevated, "highlight")
    } else {
        (CardVariant::Outline, "default")
    };

    let mut header_children: Vec<Node> = Vec::new();
    if product.highlighted {
        header_children.push(badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Solid,
                ..BadgeProps::default()
            },
            vec![("data-blocks-comparison-cards-recommended", "")],
            vec![text("おすすめ")],
        ));
    }
    header_children.push(heading(
        HeadingLevel::H4,
        &HeadingProps {
            size: HeadingSize::Lg,
            weight: HeadingWeight::Semibold,
        },
        vec![],
        vec![text(product.name)],
    ));
    header_children.push(styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-comparison-cards-description", "")],
        vec![text(product.description)],
    ));

    let features_list = list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![("data-blocks-comparison-cards-features", "")],
        product.features.iter().map(feature_row).collect(),
    );

    card::root(
        CardProps {
            variant,
            ..CardProps::default()
        },
        vec![("data-blocks-comparison-cards-card", state)],
        vec![
            card::header(
                vec![("class", "blocks-comparison-cards-header")],
                header_children,
            ),
            card::body(vec![], vec![features_list]),
        ],
    )
}

/// 変種 1 件分（見出し領域 + カード列）。`columns` はカード列の
/// `data-blocks-comparison-cards-columns` 値（`"2"`/`"3"`）。
fn section_variant(
    cards: &[ProductCard],
    columns: &'static str,
    heading_text: &str,
    lead: &str,
) -> Node {
    let header = div(
        vec![("class", "blocks-comparison-cards-header-area")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-comparison-cards-eyebrow", "")],
                vec![text("比較")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text(heading_text)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-comparison-cards-lead", "")],
                vec![text(lead)],
            ),
        ],
    );

    let grid = div(
        vec![
            ("class", "blocks-comparison-cards-grid"),
            ("data-blocks-comparison-cards-columns", columns),
        ],
        cards.iter().map(product_card).collect(),
    );

    div(
        vec![("class", "blocks-comparison-cards-variant")],
        vec![header, grid],
    )
}

/// `comparison-cards` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「2 参照 ID の畳み込み方」節）。2 枚（R0435）・
/// 3 枚（R0436）の 2 インスタンスを縦に並べる。
pub fn demo() -> Node {
    let two = section_variant(
        &TWO_CARDS,
        "2",
        "プランを比較する",
        "主要な機能の違いを一目で確認できます。",
    );
    let three = section_variant(
        &THREE_CARDS,
        "3",
        "3 社で比較する",
        "競合 2 社との違いをまとめました。",
    );

    div(
        vec![("class", "blocks-comparison-cards-layout")],
        vec![two, three],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0435/R0436。出典の固有名・ファイル名は記載しません）
からの意図的な差分は次のとおりです。

- 集約元 2 件（カード 2 枚・カード 3 枚）を 1 ページに縦並記しました。
- 自社カードの強調（参照元にはない意匠）を加えました。`CardVariant::
  Elevated` + accent 色の 2px 枠 + 「おすすめ」badge です。
- 見出しを `h3`、製品名を `h4` に下げました（ページ側が `## Demo` として
  `h2` を出すため）。
- 2 インスタンスを並記するため `id`/`aria-labelledby` は出力しません。
- 可否アイコンは自作の図形にし、`role="img"` + `aria-label`（「含まれる」/
  「含まれない」）で可読性を確保しました。装飾用の `list::indicator` は
  可否という中核情報を隠してしまうため使いません。
- eyebrow は tagline ではなく `badge` にしました。
- 列数切り替えの閾値は「基本 1 列、広がったら増やす」の mobile-first
  （`min-width`）で書きました。
- 背景・大きな余白は `.blocks-demo` 枠に任せました。
- 文言・製品名・機能名はすべて架空のもので、実在の企業・サービス名では
  ありません。配色は既存テーマトークンにそのまま従います。
