//! `comparison-cards` block（イシュー #2822。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、対応表 ID R0435/R0436 の 2 件を
//! 構造の参照元とする合成例。中央寄せの見出し領域の下に、自社製品と
//! 競合製品のカードを横並びで並べ、機能ごとの可否をアイコンで示す製品
//! 比較 UI）。取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! **Marketing / Comparison カテゴリで 2 番目の block**（`super`
//! （`comparison/mod.rs`）参照。最初の block〔カテゴリ雛形からの卒業〕は
//! イシュー #2823 の `comparison_feature_rows`）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `card` / `list` / `icon` の 6 部品を合成
//! する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 2 参照 ID の畳み込み方（Demo は 2 インスタンス）
//!
//! - **2 枚インスタンス**（R0435 が主参照）: 自社カード + 競合 1 社の
//!   計 2 枚。
//! - **3 枚インスタンス**（R0436 が主参照）: 自社カード + 競合 2 社の
//!   計 3 枚（可否パターンを 2 枚インスタンスと変え、比較として読める
//!   ようにする）。
//!
//! `changelog_timeline`/`content_columns_screenshot` と同じく、集約元の
//! 差分は Demo の縦並記で示す（[`site/blocks/comparison-cards.md`] の
//! 「原案差分メモ」参照）。
//!
//! # レイアウトとブレークポイント
//!
//! カード列は既定で 1 列（縦積み）とし、[`fandhe_frontend_pre_styled_ui::
//! recipe::Breakpoint::Md`]（768px = 48rem）以上で 2 枚インスタンスを
//! 2 列へ、[`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]
//! （1024px = 64rem）以上で 3 枚インスタンスを 3 列へ切り替える
//! mobile-first の `min-width` メディアクエリで書く（`login_04` 等の
//! 既存 block が使う `max-width: 47.99rem` 形式とは意図的に逆方向にする
//! 判断。段数が増えるほど広い幅を要求する構造のため「基本 1 列、広がったら
//! 増やす」の方が素直に書けると判断した。閾値の値〔48rem/64rem〕自体は
//! テーマ breakpoint トークンと一致させる）。
//!
//! # 自社カードの強調
//!
//! 自社カードは [`fandhe_frontend_pre_styled_ui::card::CardVariant::
//! Elevated`] + `data-blocks-comparison-cards-card="highlight"`（
//! [`LAYOUT_CSS`] が accent 色の 2px 枠を追加）+ 見出し badge
//! （「おすすめ」）で強調する。競合カードは
//! [`fandhe_frontend_pre_styled_ui::card::CardVariant::Outline`] + 同属性
//! `"default"` のみを持つ。
//!
//! # 見出しレベルに `H3`/`H4` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため変種見出しは
//! [`fandhe_frontend_pre_styled_ui::heading::HeadingLevel::H3`] にする
//! （`content_columns_screenshot` 等と同じ判断）。製品名は [`crate::blocks::
//! marketing::comparison::comparison_cards`] 内で `card::title`（`<h3>`
//! 固定）を使わず [`fandhe_frontend_pre_styled_ui::heading::heading`]
//! （[`fandhe_frontend_pre_styled_ui::heading::HeadingLevel::H4`]）を直接
//! 呼ぶことで、H3 の下の見出しレベルとして正しく 1 段深くする
//! （`changelog_timeline` が `timeline::title` の代わりに `heading` H4 を
//! 使う判断と同型）。
//!
//! # 可否アイコンに `list::indicator` を使わない理由（a11y）
//!
//! [`fandhe_frontend_pre_styled_ui::list::indicator`] は常に
//! `aria-hidden="true"` を強制する装飾用パーツであり、中に可否アイコンを
//! 入れるとその情報がスクリーンリーダーに一切伝わらない（可否は本 block の
//! 中核情報であり装飾ではない）。そのため可否アイコンは `list::item` の
//! 直下に置き、[`fandhe_frontend_pre_styled_ui::icon::IconProps::label`]
//! （`Some("含まれる")`/`Some("含まれない")`）を指定して `role="img"` +
//! `aria-label` による意味のある代替テキストを付与する
//! （`bento_asymmetric_rows::feature_icon` の自作アイコンと同じ着想だが、
//! 本 block は装飾ではなく情報を運ぶため `label` を `None` にしない点が
//! 異なる）。
//!
//! # `id`/`aria-labelledby` を出力しない理由
//!
//! 2 インスタンス（2 枚/3 枚）を同一ページへ並記するため、`id`/
//! `aria-controls`/`aria-labelledby` を出力すると id 重複や宙に浮いた
//! ARIA 参照を生みやすい。本 block はいずれの部品も `id` を要さない構成
//! のため一切出力しない（`changelog_timeline` と同じ判断、
//! `crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` の対象）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `badge::badge` / `card::root` /
//! `list::root` / `icon::icon` はいずれも `drop_class_attr` により呼び出し
//! 側 `attrs` の `class` を黙って除去する契約を持つため、Demo 固有の
//! スタイルフックは `data-blocks-comparison-cards-*` 属性で渡す。
//! `card::header`/`card::body`/`list::item` は `drop_class_attr` を経由
//! しないため `class` がそのまま効くが、他 block と同じく名前空間分離の
//! ため `.blocks-comparison-cards-*` クラスを使う。ルート class
//! （`blocks-comparison-cards-layout`）は [`Block::demo_class`]
//! （`blocks-comparison-cards`）とは意図的に別名にする（`blog_list_image`
//! 等と同じ Bugbot 教訓の回避）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。製品名・機能名・説明はすべて架空のもの（実在の企業名・
//! サービス名・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/comparison-cards/",
    title: "comparison-cards",
    category: BlockCategory::Comparison,
    rust_source: "crates/docs-site/src/blocks/marketing/comparison/comparison_cards.rs",
    demo_class: "blocks-comparison-cards",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `comparison_cards` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。
///
/// mobile-first（`min-width`）でカード列の列数を増やす（モジュール doc
/// 「レイアウトとブレークポイント」節参照）。色はすべて既存トークン
/// （`--fandhe-color-accent`/`-border`/`-fg-muted`）のみを使う。
const LAYOUT_CSS: &str = "\
.blocks-comparison-cards-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
.blocks-comparison-cards-variant {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-comparison-cards-header-area {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  text-align: center;\n  max-width: 36rem;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-comparison-cards-lead] {\n  margin: 0;\n}\n\
.blocks-comparison-cards-grid {\n  display: grid;\n  width: 100%;\n  gap: var(--fandhe-space-4);\n  grid-template-columns: 1fr;\n}\n\
[data-blocks-comparison-cards-columns=\"2\"] {\n  max-width: 48rem;\n}\n\
[data-blocks-comparison-cards-columns=\"3\"] {\n  max-width: 64rem;\n}\n\
@media (min-width: 48rem) {\n  [data-blocks-comparison-cards-columns=\"2\"] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n\
@media (min-width: 64rem) {\n  [data-blocks-comparison-cards-columns=\"3\"] {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-comparison-cards-card=\"highlight\"] {\n  border: 2px solid var(--fandhe-color-accent);\n}\n\
.blocks-comparison-cards-header {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-comparison-cards-description] {\n  margin: 0;\n}\n\
[data-scope=\"list\"][data-part=\"root\"][data-blocks-comparison-cards-features] {\n  margin: 0;\n  padding: 0;\n}\n\
.blocks-comparison-cards-row {\n  display: flex;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  padding-block: var(--fandhe-space-3);\n  border-top: 1px solid var(--fandhe-color-border);\n}\n\
[data-scope=\"list\"][data-part=\"root\"].fd-list--variant-plain > [data-scope=\"list\"][data-part=\"item\"].blocks-comparison-cards-row {\n  align-items: center;\n}\n\
[data-scope=\"list\"][data-part=\"item\"].blocks-comparison-cards-row:first-child {\n  border-top: none;\n  padding-top: 0;\n}\n\
[data-scope=\"list\"][data-part=\"item\"].blocks-comparison-cards-row:last-child {\n  padding-bottom: 0;\n}\n\
[data-blocks-comparison-cards-value=\"excluded\"] {\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-comparison-cards-row:has([data-blocks-comparison-cards-value=\"excluded\"]) .blocks-comparison-cards-feature-label {\n  color: var(--fandhe-color-fg-muted);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 6 種の部品を出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"badge\"",
            "data-scope=\"card\"",
            "data-scope=\"list\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// 2 枚・3 枚の両インスタンスの列数フックが出力されていること。
    #[test]
    fn demo_wires_both_column_counts() {
        let html = render(&demo());
        assert!(html.contains(r#"data-blocks-comparison-cards-columns="2""#));
        assert!(html.contains(r#"data-blocks-comparison-cards-columns="3""#));
    }

    /// 自社カード（highlight）が各インスタンスに 1 枚ずつ、計 2 枚
    /// あること。
    #[test]
    fn demo_has_one_highlighted_card_per_instance() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-blocks-comparison-cards-card="highlight""#)
                .count(),
            2
        );
    }

    /// 可否アイコンが `role="img"` + 意味のある `aria-label` を持つこと
    /// （モジュール doc「可否アイコンに `list::indicator` を使わない理由」
    /// 節）。
    #[test]
    fn demo_labels_included_and_excluded_icons_for_accessibility() {
        let html = render(&demo());
        assert!(html.contains(r#"aria-label="含まれる""#));
        assert!(html.contains(r#"aria-label="含まれない""#));
        assert!(html.contains(r#"role="img""#));
    }

    /// 可否表現には `list::indicator`（装飾用、常に `aria-hidden`）を
    /// 使っていないこと。
    #[test]
    fn demo_never_uses_list_indicator() {
        let html = render(&demo());
        assert!(!html.contains(r#"data-part="indicator""#));
    }

    /// 非対話・安全性の不変条件（`<form>`・`id=` 属性・`data:` URI を
    /// 持たないこと）。
    #[test]
    fn demo_never_contains_forbidden_markup() {
        let html = render(&demo());
        for absent in ["<form", "id=\"", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` 等と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-comparison-cards-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-comparison-cards-layout");
    }

    /// [`LAYOUT_CSS`] が 48rem/64rem のブレークポイント条件と accent 枠を
    /// 持つこと。
    #[test]
    fn layout_css_has_breakpoints_and_accent_border() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("border: 2px solid var(--fandhe-color-accent);"));
    }
}
