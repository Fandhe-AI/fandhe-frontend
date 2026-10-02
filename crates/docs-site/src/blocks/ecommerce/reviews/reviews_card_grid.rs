//! `reviews-card-grid` block（イシュー #3088。親トラッキング #3024。
//! 対応表 ID R0214（主参照・集約元ともに R0214 の 1 件のみ）を構造の
//! 参照元とする合成例。取得手段・ファイル名・内部コンポーネント識別子は
//! 記載しない（`docs/design/motion-reference-adoption-policy.md` §9 と
//! 同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `rating-group` / `button` / `separator` / `card` / `text` の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # レイアウト（ヘッダ行 + 区切り線 + カードグリッド + もっと見る）
//!
//! ヘッダ行（見出し + 平均評価 + 投稿ボタン）→ 区切り線 → レビューカード
//! 6 枚のグリッド → 末尾中央の「さらに読み込む」ボタンの順。狭幅では
//! ヘッダ行が折り返し、カードグリッドは 1 列に畳む。
//!
//! # 列数は `@container` で切り替える
//!
//! `product_list_rich_cards`「列数は `@container` で切り替える」節と同型の
//! 判断。Demo 枠（`.docs-content`、最大 46rem。`.blocks-demo` の左右
//! padding `1.5rem`×2 を差し引くと実効上限は約 43rem）の幅はビューポート幅
//! と一致しないため、`@media` ではなくコンテナクエリで判定する。境界値
//! `32rem` は実効上限 43rem 以下に収め、Demo 内で実際に切り替わる値を選ぶ。
//! `container-type`/`container-name` はレイアウトルートへ宣言し、
//! `@container` では名前付きコンテナを介してグリッドを判定対象にする。
//!
//! # 評価を `rating-group`（readonly）+ 件数込みラベルで表現する理由
//!
//! `product_list_rich_cards.rs::rating_row` と同型。ヘッダ行の平均評価・
//! カードごとの個別評価のいずれも、現在の評価と件数をラベルで明文化し、
//! ラベル `id` はカードごとに一意にする
//! （`crates/docs-site/tests/blocks_contract.rs` の重複 id 検査対応）。
//!
//! # 投稿ボタン・もっと見るボタンは disabled の静的表示
//!
//! `product_overview_gallery_split.rs`「購入ボタンは disabled の静的表示」
//! 節と同じ判断。無 JS のため押しても何も起きないボタンを操作可能なまま
//! 残さない。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。`href="#"` は使わない。
//!
//! # ダミー素材について
//!
//! 投稿者名は [`dummy_assets::PERSON_NAMES`] を流用する。レビュー件名・
//! 本文・日付はすべて独自に書いた架空の内容であり、実在の企業名・人物・
//! PII・有料アセット名を含まない。
//!
//! # 原案差分メモ・スコープ外
//!
//! R0214（ヘッダ + 6 件グリッド + もっと見る）の構成をそのまま採用した。
//! ボタン（投稿・もっと見る）は無 JS のため disabled の静的表示にした。
//! `_/blocks-intake/` の参照ファイルは本イシュー着手時点で worktree に
//! 存在しないため、対応表 ID のみを記して実装した（見た目の細部調整は
//! スコープ外、`site/blocks/reviews-card-grid.md` の「原案差分メモ」節も
//! 参照）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/reviews-card-grid/",
    title: "reviews-card-grid",
    category: BlockCategory::Reviews,
    rust_source: "crates/docs-site/src/blocks/ecommerce/reviews/reviews_card_grid.rs",
    demo_class: "blocks-reviews-card-grid",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Rating Group",
            path: "/themes/rating-group/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `reviews_card_grid` 固有のレイアウト規則（`crate::blocks` モジュール doc
/// 「block 固有 CSS の置き場」節と同型）。モジュール doc「列数は
/// `@container` で切り替える」節のとおり、`32rem` のコンテナ幅境界
/// （Demo 枠の実効上限約 43rem 以下）で列数を切り替える。
/// `display: none` は使わない。
const LAYOUT_CSS: &str = "\
.blocks-reviews-card-grid-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-reviews-card-grid;\n}\n\
.blocks-reviews-card-grid-header {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-reviews-card-grid-header-summary {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-reviews-card-grid-grid {\n  display: grid;\n  gap: var(--fandhe-space-4);\n  grid-template-columns: minmax(0, 1fr);\n}\n\
@container blocks-reviews-card-grid (min-width: 32rem) {\n  .blocks-reviews-card-grid-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n\
[data-blocks-reviews-card-grid-card] > [data-scope=\"card\"][data-part=\"body\"] {\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-reviews-card-grid-more {\n  display: flex;\n  justify-content: center;\n}\n\
";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"rating-group\"",
            "data-scope=\"button\"",
            "data-scope=\"separator\"",
            "data-scope=\"card\"",
            "data-scope=\"text\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn all_buttons_are_type_button() {
        let html = demo_html();
        let count_open = html.matches("<button").count();
        let count_typed = html.matches("type=\"button\"").count();
        assert!(count_open > 0);
        assert!(count_typed >= count_open, "html={html}");
    }

    #[test]
    fn demo_has_no_form_or_unsafe_href() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }

    #[test]
    fn rating_labels_have_unique_ids() {
        let html = demo_html();
        let ids: Vec<&str> = html
            .split("id=\"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .collect();
        let unique: std::collections::BTreeSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "ids should all be unique: {ids:?}");
    }

    #[test]
    fn grid_has_six_cards() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-reviews-card-grid-card").count(),
            6
        );
    }

    #[test]
    fn exactly_one_h2_heading() {
        let html = demo_html();
        assert_eq!(html.matches("<h2").count(), 1);
    }

    #[test]
    fn layout_css_is_safe_and_has_breakpoint() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size"));
        assert!(LAYOUT_CSS.contains("@container blocks-reviews-card-grid (min-width: 32rem)"));
        assert!(!LAYOUT_CSS.contains("display: none"));
    }

    #[test]
    fn demo_class_differs_from_layout_root_class() {
        assert_ne!(super::BLOCK.demo_class, "blocks-reviews-card-grid-layout");
    }
}
