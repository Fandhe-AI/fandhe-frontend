//! `reviews-summary-split` block（イシュー #3090。親トラッキング #3024。
//! Ecommerce / Reviews カテゴリ 2 件目）。左列に評価サマリ（平均評価・
//! 星別割合バー・導線）、右列にレビュー一覧を置く 2 カラム構成。
//!
//! # 使用部品
//!
//! `heading` / `rating-group` / `progress` / `avatar` / `text` / `link` /
//! `button` / `image` / `card` の 9 部品を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs`
//! が検証する）。新しい UI 部品は追加しない。
//!
//! # 2 版の対応（原稿「原案差分メモ」節と対になる索引）
//!
//! - **版 A（代表構成、対応表 ID R1219）**: 星別割合バー 5 本 + レビュー
//!   3 件（区切り線のみ、`card` は使わない）
//! - **版 B（集約元、対応表 ID R0215）**: 版 A のサマリにサムネイル画像
//!   8 枚のグリッドを追加し、レビュー 6 件を `card` に収める
//!
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`reviews-card-grid` #3088 と同じ扱い）。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::card::root`]・
//! [`fandhe_frontend_pre_styled_ui::button::button`]・
//! [`fandhe_frontend_pre_styled_ui::avatar::root`]・
//! [`fandhe_frontend_pre_styled_ui::progress::root`] はいずれも
//! `drop_class_attr` により呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-reviews-summary-split-*`、`reviews-card-grid` と同型の
//! 判断）。レイアウト用ラッパー（スタック・2 カラム・割合バー行・
//! サムネイルグリッド）は素の `<div>` のため `class` をそのまま使う。
//!
//! # 狭い幅では 2 カラムを縦積みにする（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`
//! （コンテナクエリ）で判定する（`profile_detail_datalist` と同型の
//! パターン）。DOM 順は常にサマリ → レビュー一覧のため、縦積み時も
//! `order` 指定なしでサマリが先に積まれる。
//!
//! # 平均評価は数値テキスト + 整数丸めの `rating-group`（readonly）
//!
//! [`fandhe_frontend_headless_ui::rating_group`] の API には半星の表現が
//! ないため、平均評価（4.3 等の小数）は星を切り捨ての整数（4）で表しつつ、
//! 隣に正確な数値テキストを並べて正確な値を示す。`rating_group::label` の
//! 可視テキストへ平均値・件数を明文化する方式は `reviews_card_grid.rs::
//! rating_row` と同型（`visually_hidden` は使わず、本 block の `parts` を
//! 9 件のまま保つ）。全 `rating-group` の label id は版・行ごとに一意にする
//! （`blocks_contract.rs` の重複 id 検査・`aria-labelledby` 参照先検査への
//! 適合）。
//!
//! # 星別割合バーは `progress`（linear）+ 明示 `aria-label`
//!
//! `feature_tabs_panel.rs::trigger_with_progress` と同型のパターン。
//! `aria-labelledby` の自動配線は headless/styled いずれの層の責務でもない
//! ため、各行で `aria-label`（例: 「5 つ星の割合」）を明示する。割合値は
//! 架空の固定値（5 本の合計が 100% になるよう固定）。
//!
//! # `<form>` を持たない・ボタンは disabled の静的表示
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。「レビューを書く」ボタンは無 JS のため押しても何も起きない
//! ことを明示する `disabled` の静的ボタン（`reviews_card_grid` と同型の
//! 判断）。`link` は実在の自リポジトリ URL のみとし `href="#"` は使わない。
//!
//! # ダミー素材について
//!
//! 投稿者名は [`dummy_assets::PERSON_NAMES`] を流用する。レビュー本文・
//! 評価内訳の割合・平均評価・件数はすべて独自に書いた架空の内容であり、
//! 実在の企業名・人物・PII・有料アセット名を含まない。サムネイル画像は
//! ビルド時生成の同梱 SVG（[`dummy_assets::PRODUCT_SRC`]）を使う（外部
//! URL・`data:` URI は使わない）。8 枚とも同一画像のため、`alt` は
//! 「購入者が投稿した写真 N」のように連番で一意にする（実在の商品写真では
//! ないため具体的な内容の記述はしない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, h3, text, Node};
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
    let tiles: Vec<Node> = (1..=8)
        .map(|i| {
            let alt = format!("購入者が投稿した写真 {i}");
            image::image(
                &ImageProps {
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, &alt)
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

/// caption（並記された各版の見出し）。
fn caption(label: &str) -> Node {
    h3(
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/reviews-summary-split/",
    title: "reviews-summary-split",
    category: BlockCategory::Reviews,
    rust_source: "crates/docs-site/src/blocks/ecommerce/reviews/reviews_summary_split.rs",
    demo_class: "blocks-reviews-summary-split",
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
            label: "Progress",
            path: "/themes/progress/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `reviews_summary_split` 固有のレイアウト規則（`--fandhe-*` トークンの
/// みを使用）。`40rem` のコンテナ幅境界で 2 カラム → 縦積みを切り替える
/// （モジュール doc「狭い幅では 2 カラムを縦積みにする」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-reviews-summary-split-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  container-type: inline-size;\n  container-name: blocks-reviews-summary-split;\n}\n\
.blocks-reviews-summary-split-split {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr) minmax(0, 2fr);\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-reviews-summary-split-summary {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-reviews-summary-split-rating-number {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-reviews-summary-split-breakdown {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  width: 100%;\n}\n\
.blocks-reviews-summary-split-breakdown-row {\n  display: grid;\n  grid-template-columns: auto 1fr auto;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-reviews-summary-split-thumbnails {\n  display: grid;\n  grid-template-columns: repeat(4, minmax(0, 1fr));\n  gap: var(--fandhe-space-2);\n  width: 100%;\n}\n\
.blocks-reviews-summary-split-list {\n  display: flex;\n  flex-direction: column;\n}\n\
.blocks-reviews-summary-split-review {\n  display: flex;\n  gap: var(--fandhe-space-3);\n  border-top: 1px solid var(--fandhe-color-border);\n  padding-block: var(--fandhe-space-4);\n}\n\
.blocks-reviews-summary-split-review-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-reviews-summary-split-cards {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-reviews-summary-split-card] > [data-scope=\"card\"][data-part=\"body\"] {\n  display: flex;\n  flex-direction: row;\n  gap: var(--fandhe-space-3);\n}\n\
@container blocks-reviews-summary-split (max-width: 40rem) {\n  \
.blocks-reviews-summary-split-split {\n    grid-template-columns: minmax(0, 1fr);\n  }\n\
}\n";

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
            "data-scope=\"progress\"",
            "data-scope=\"avatar\"",
            "data-scope=\"text\"",
            "data-scope=\"link\"",
            "data-scope=\"button\"",
            "data-scope=\"image\"",
            "data-scope=\"card\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn rating_group_root_count_is_eleven() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-scope=\"rating-group\" data-part=\"root\"")
                .count(),
            11
        );
    }

    #[test]
    fn progress_root_count_is_ten() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-scope=\"progress\" data-part=\"root\"")
                .count(),
            10
        );
    }

    #[test]
    fn thumbnail_image_count_is_eight() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-reviews-summary-split-thumbnail")
                .count(),
            8
        );
    }

    #[test]
    fn card_root_count_is_six() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-reviews-summary-split-card")
                .count(),
            6
        );
    }

    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = demo_html();
        for absent in [
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "src=\"data:",
            "<script",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
        assert!(html.contains("type=\"button\""));
    }

    #[test]
    fn demo_references_dummy_assets() {
        let html = demo_html();
        assert!(html.contains("../../assets/blocks-demo-avatar.svg"));
        assert!(html.contains("../../assets/blocks-demo-product.svg"));
    }

    #[test]
    fn rating_group_label_ids_are_unique() {
        let html = demo_html();
        let ids: Vec<&str> = html
            .split("id=\"")
            .skip(1)
            .filter(|rest| rest.contains("-rating-label"))
            .filter_map(|rest| rest.split('"').next())
            .collect();
        assert_eq!(ids.len(), 11);
        let unique: std::collections::BTreeSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "ids should all be unique: {ids:?}");
    }

    #[test]
    fn layout_css_is_safe_and_has_breakpoint() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-reviews-summary-split (max-width: 40rem)"));
        assert!(!LAYOUT_CSS.contains("display: none"));
    }

    #[test]
    fn demo_class_differs_from_layout_root_class() {
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-reviews-summary-split-stack"
        );
    }
}
