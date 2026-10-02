//! `reviews-stacked-list` block（Ecommerce / Reviews カテゴリの block）。
//! レビューを区切り線で仕切り縦に並べる一覧の合成例。
//!
//! # 使用部品
//!
//! `heading` / `avatar` / `rating-group` / `text` / `separator` /
//! `input-group` / `input` / `badge` / `button` / `visually-hidden` の
//! 10 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 3 variant 併記（無 JS での静的表示）
//!
//! docs サイトは JS ハイドレーションを行わない設計（CLAUDE.md）のため、
//! 3 つの構成を 1 ページに縦へ併記する（`testimonial_split_image.rs` と
//! 同型）。
//!
//! - `three-column`: 視覚的に隠した見出し + 広幅で「投稿者・日付」/
//!   「星評価」/「タイトル・本文」の 3 列、狭幅で縦積みにする行。
//! - `author-split`: 視覚的に隠した見出し + アバター付きの行。左に
//!   アバター・投稿者名・日付、右に星評価・タイトル・本文を置く。
//! - `search-header`: 中央寄せの可視見出し + 平均評価 + 検索欄を上部に
//!   持つ行。行には購入者バッジを付ける。
//!
//! # 見出しの可視・非可視の使い分け
//!
//! `three-column`/`author-split` は一覧の直前に文脈（商品ページ等）がある
//! 想定のため見出しを `visually_hidden::root` で隠す。`search-header` は
//! 集計・検索欄を伴う独立した区画のため見出しを可視のまま中央寄せにする
//! （`card_heading_toolbar.rs` の判断と対称）。
//!
//! # 星評価は readonly、label id は行ごとに一意化
//!
//! `rating_group` は他ユーザーが付けた評価を表す静的表示であり、
//! `testimonial_split_image.rs::rating` と同型で `readonly: true`。
//! 1 ページに複数行出現するため label id は `variant` と行番号で一意化する
//! （[`rating_label_id`]、`blocks_contract.rs` の重複 id・ダングリング aria
//! 参照検査に適合させる）。
//!
//! # 検索欄は `aria-label`、送信処理を持たない
//!
//! `card_heading_toolbar.rs` と同型で、`field`/`visually_hidden` を検索欄の
//! アクセシブル名には使わず `aria-label` で確保する。検索ボタンは
//! `button::button` の既定 `type="button"` のまま送信先を持たない静的表示
//! （`docs/policy/intentional-non-adoption.md` §3.25：バリデーション・送信
//! 処理は UI コンポーネント層の責務外）。
//!
//! # アバターは装飾扱い（`alt=""`）
//!
//! 隣に投稿者名のテキストが常に出力されるため、アバター画像は装飾画像
//! として扱う（`testimonial_split_image.rs::instance_author_column` と
//! 同じ判断）。
//!
//! # レスポンシブはコンテナクエリ（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため `@container` で判定する
//! （`quickview_image_split.rs`/`promo_sale_products.rs` と同型）。ルート
//! `.blocks-reviews-stacked-list-stack` へ `container-type: inline-size` を
//! 宣言し、`40rem` 未満では各行を縦積みにする。
//!
//! # `<form>` を持たない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。文言はすべて独自の架空の日本語ダミー（実企業名・実
//! クレデンシャル・PII を含まない）で、投稿者名・アバターのみ
//! `crate::blocks::dummy_assets` を使う。
//!
//! # 参照について
//!
//! 主参照は対応表 ID R1218（広幅で投稿者・評価・本文の 3 列化）。集約元は
//! R1220（投稿者と本文の左右分離）・R1221（アバター付き）・R0216（中央
//! 見出し + 平均評価 + 検索欄）。取得手段・ファイル名・出典名・内部識別子は
//! 記載しない（他 block と同じライセンス上の転記制限）。取り込むのは領域の
//! 配置と部品構成という構造のみで、文言・配色は独自に書く。参照との差分は
//! `site/blocks/reviews-stacked-list.md` の「原案差分メモ」節に記載する。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, p, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 検索欄（`field`）の `id`。
const SEARCH_FIELD_ID: &str = "blocks-reviews-stacked-list-search";

/// レビュー本文の架空ダミー（タイトル・本文）。実企業名・PII を含まない。
const REVIEW_BODIES: &[(&str, &str)] = &[
    (
        "扱いやすくて助かりました",
        "届いてすぐに使い始めましたが、説明書を読まなくても迷わず操作できました。",
    ),
    (
        "期待以上の仕上がり",
        "写真で見るより質感が良く、普段使いにちょうど良いサイズ感です。",
    ),
    (
        "リピートしたい",
        "前回購入した色違いも良かったので、今回も安心して選べました。",
    ),
    (
        "梱包が丁寧でした",
        "配送中の傷みもなく、箱を開けた瞬間から気持ちよく使い始められました。",
    ),
    (
        "サイズ感がちょうど良い",
        "口コミを参考にワンサイズ上を選びましたが、ぴったりで満足しています。",
    ),
    (
        "少し時間がかかりました",
        "到着まで数日待ちましたが、品質を考えると十分納得できる内容でした。",
    ),
    (
        "贈り物にも良さそうです",
        "自分用に買いましたが、包装次第で贈り物にも使えそうな上品な見た目です。",
    ),
];

/// レビュー投稿日時（ISO 8601 の `datetime` 属性値, 表示文字列）の組。
const REVIEW_DATES: &[(&str, &str)] = &[
    ("2026-07-02", "2026年7月2日"),
    ("2026-07-18", "2026年7月18日"),
    ("2026-08-05", "2026年8月5日"),
    ("2026-08-21", "2026年8月21日"),
    ("2026-09-03", "2026年9月3日"),
    ("2026-09-15", "2026年9月15日"),
    ("2026-09-27", "2026年9月27日"),
];

/// レビューの星評価（5 段階）。[`REVIEW_BODIES`]/[`REVIEW_DATES`] と同じ
/// 長さ・添字で対応する。
const REVIEW_SCORES: &[u32] = &[5, 4, 5, 3, 5, 4, 5];

/// レビュー 1 件分のダミーデータへの参照（投稿者・日付・星評価・タイトル・
/// 本文を 1 つの添字 `index` から引けるようにまとめる）。
fn review_index(offset: usize, count: usize) -> Vec<usize> {
    (0..count)
        .map(|i| (offset + i) % REVIEW_BODIES.len())
        .collect()
}

/// 投稿者名（[`dummy_assets::PERSON_NAMES`] を周回利用）。
fn author_name(index: usize) -> &'static str {
    dummy_assets::PERSON_NAMES[index % dummy_assets::PERSON_NAMES.len()]
}

/// 星評価 label の `id`（variant・行番号ごとに一意化、モジュール doc
/// 「星評価は readonly、label id は行ごとに一意化」節参照）。
fn rating_label_id(variant: &str, row: usize) -> String {
    format!("blocks-reviews-stacked-list-rating-{variant}-{row}")
}

/// 星評価（readonly、[`rating_label_id`] が付与する一意ラベル）。
fn rating_block(label_id: &str, score: u32) -> Node {
    let props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let state = RatingGroup::new(5, Some(score), true);
    let label = rating_group::label(
        &props,
        Some(label_id),
        vec![],
        vec![visually_hidden::root(
            vec![],
            vec![text(format!("評価 {score} / 5"))],
        )],
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
    let control = rating_group::control(&props, Some(label_id), vec![], items);
    rating_group::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![],
        vec![label, control],
    )
}

/// 投稿日時（素の `<time>`、`el` は `&'static str` 属性のみのため
/// [`REVIEW_DATES`] の `&'static str` をそのまま渡せる）。
fn review_time(datetime: &'static str, display: &'static str) -> Node {
    el("time", vec![("datetime", datetime)], vec![text(display)])
}

/// レビュータイトル + 本文（タイトルは `h4`、`testimonial_split_image.rs`
/// の慣例どおり block 内見出し `h3` の 1 段下）。
fn review_body(title: &'static str, body: &'static str) -> Node {
    div(
        vec![("class", "blocks-reviews-stacked-list-body")],
        vec![
            el("h4", vec![], vec![text(title)]),
            styled_text::text(&TextProps::default(), vec![], vec![text(body)]),
        ],
    )
}

/// `three-column` variant の行（主参照 R1218。投稿者・日付 / 星評価 /
/// タイトル・本文の 3 列）。
fn row_three_column(row: usize, index: usize) -> Node {
    let (title, body) = REVIEW_BODIES[index];
    let (datetime, display) = REVIEW_DATES[index];
    div(
        vec![("class", "blocks-reviews-stacked-list-row")],
        vec![
            div(
                vec![("class", "blocks-reviews-stacked-list-meta")],
                vec![
                    styled_text::text(
                        &TextProps::default(),
                        vec![],
                        vec![text(author_name(index))],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![review_time(datetime, display)],
                    ),
                ],
            ),
            rating_block(&rating_label_id("three-column", row), REVIEW_SCORES[index]),
            review_body(title, body),
        ],
    )
}

/// `author-split` variant の行（集約元 R1220・R1221。左にアバター・
/// 投稿者名・日付、右に星評価・タイトル・本文）。
fn row_author_split(row: usize, index: usize) -> Node {
    let (title, body) = REVIEW_BODIES[index];
    let (datetime, display) = REVIEW_DATES[index];
    let author_column = div(
        vec![("class", "blocks-reviews-stacked-list-author")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Md,
                    ..AvatarProps::default()
                },
                vec![],
                vec![avatar::image(
                    ImageStatus::Loaded,
                    dummy_assets::AVATAR_SRC,
                    "",
                    vec![],
                )],
            ),
            div(
                vec![],
                vec![
                    styled_text::text(
                        &TextProps::default(),
                        vec![],
                        vec![text(author_name(index))],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![review_time(datetime, display)],
                    ),
                ],
            ),
        ],
    );
    let content = div(
        vec![("class", "blocks-reviews-stacked-list-content")],
        vec![
            rating_block(&rating_label_id("author-split", row), REVIEW_SCORES[index]),
            review_body(title, body),
        ],
    );
    div(
        vec![("class", "blocks-reviews-stacked-list-row")],
        vec![author_column, content],
    )
}

/// `search-header` variant の行（集約元 R0216。購入者バッジ付き）。
fn row_search_header(row: usize, index: usize) -> Node {
    let (title, body) = REVIEW_BODIES[index];
    let (datetime, display) = REVIEW_DATES[index];
    div(
        vec![("class", "blocks-reviews-stacked-list-row")],
        vec![
            div(
                vec![("class", "blocks-reviews-stacked-list-meta")],
                vec![
                    div(
                        vec![("class", "blocks-reviews-stacked-list-meta-name")],
                        vec![
                            styled_text::text(
                                &TextProps::default(),
                                vec![],
                                vec![text(author_name(index))],
                            ),
                            badge::badge(&BadgeProps::default(), vec![], vec![text("購入者")]),
                        ],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![review_time(datetime, display)],
                    ),
                ],
            ),
            rating_block(&rating_label_id("search-header", row), REVIEW_SCORES[index]),
            review_body(title, body),
        ],
    )
}

/// 平均評価 + 検索欄（`search-header` variant 上部）。
fn summary_and_search() -> Node {
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    let field = FieldProps {
        id: SEARCH_FIELD_ID,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    div(
        vec![("class", "blocks-reviews-stacked-list-summary")],
        vec![
            div(
                vec![("class", "blocks-reviews-stacked-list-summary-score")],
                vec![
                    rating_block("blocks-reviews-stacked-list-rating-summary", 4),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("4 / 5（128 件のレビュー）")],
                    ),
                ],
            ),
            input_group::root(
                &group_props,
                vec![],
                vec![
                    input::input(
                        &InputProps::default(),
                        &field,
                        vec![
                            ("type", "search"),
                            ("placeholder", "レビューを検索"),
                            ("aria-label", "レビューを検索"),
                        ],
                    ),
                    input_group::addon(
                        InputGroupAlign::InlineEnd,
                        &group_props,
                        vec![],
                        vec![button::button(
                            &ButtonProps::default(),
                            vec![],
                            vec![text("検索")],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// 行の間に挟む水平区切り線。
fn row_separator() -> Node {
    separator::separator(
        &SeparatorProps::default(),
        vec![("data-blocks-reviews-stacked-list-separator", "")],
    )
}

/// 複数行を [`row_separator`] で区切って並べる。
fn rows_with_separators(rows: Vec<Node>) -> Vec<Node> {
    let mut out = Vec::with_capacity(rows.len() * 2);
    for (i, row) in rows.into_iter().enumerate() {
        if i > 0 {
            out.push(row_separator());
        }
        out.push(row);
    }
    out
}

/// 視覚的に隠した `h3` 見出し（`three-column`/`author-split` 用、モジュール
/// doc「見出しの可視・非可視の使い分け」節参照）。
///
/// `heading()`（pre-styled-ui）は呼び出し側 `class` を `drop_class_attr` で
/// 無言で除去するため、`.blocks-reviews-stacked-list-layout`（`gap` 付き
/// flex コンテナ）の直接の子にすると視覚的に隠していてもレイアウト上の
/// flex スロットと gap を消費してしまう。`position: absolute` を当てる
/// class はラッパー `div` へ付け、flex フローそのものから除外する。
fn hidden_heading(label: &str) -> Node {
    div(
        vec![("class", "blocks-reviews-stacked-list-hidden-heading")],
        vec![heading(
            HeadingLevel::H3,
            &HeadingProps::default(),
            vec![],
            vec![visually_hidden::root(vec![], vec![text(label)])],
        )],
    )
}

/// 可視の中央寄せ `h3` 見出し（`search-header` 用）。
///
/// `heading()` は呼び出し側 `class` を `drop_class_attr` で除去するため、
/// `class="blocks-reviews-stacked-list-heading"`（`text-align: center`）を
/// `heading()` 自体へ渡しても適用されない（`cta_centered.rs` と同じく、
/// 中央寄せの `class` は `heading()` を包むラッパー `div` へ付け、
/// `text-align` の継承で内側の見出しテキストへ反映させる）。
fn visible_heading(label: &str) -> Node {
    div(
        vec![("class", "blocks-reviews-stacked-list-heading")],
        vec![heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl,
                ..HeadingProps::default()
            },
            vec![],
            vec![text(label)],
        )],
    )
}

/// `three-column` variant 本体（3 行）。
fn instance_three_column() -> Node {
    let indices = review_index(0, 3);
    let rows = indices
        .iter()
        .enumerate()
        .map(|(row, &index)| row_three_column(row, index))
        .collect();
    div(
        vec![
            ("class", "blocks-reviews-stacked-list-layout"),
            ("data-blocks-reviews-stacked-list-variant", "three-column"),
        ],
        {
            let mut children = vec![hidden_heading("お客様のレビュー")];
            children.extend(rows_with_separators(rows));
            children
        },
    )
}

/// `author-split` variant 本体（3 行）。
fn instance_author_split() -> Node {
    let indices = review_index(3, 3);
    let rows = indices
        .iter()
        .enumerate()
        .map(|(row, &index)| row_author_split(row, index))
        .collect();
    div(
        vec![
            ("class", "blocks-reviews-stacked-list-layout"),
            ("data-blocks-reviews-stacked-list-variant", "author-split"),
        ],
        {
            let mut children = vec![hidden_heading("お客様のレビュー")];
            children.extend(rows_with_separators(rows));
            children
        },
    )
}

/// `search-header` variant 本体（平均評価 + 検索欄 + 2 行）。
fn instance_search_header() -> Node {
    let indices = review_index(0, 2);
    let rows = indices
        .iter()
        .enumerate()
        .map(|(row, &index)| row_search_header(row, index))
        .collect();
    div(
        vec![
            ("class", "blocks-reviews-stacked-list-layout"),
            ("data-blocks-reviews-stacked-list-variant", "search-header"),
        ],
        {
            let mut children = vec![visible_heading("お客様のレビュー"), summary_and_search()];
            children.extend(rows_with_separators(rows));
            children
        },
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &str) -> Node {
    p(
        vec![("class", "blocks-reviews-stacked-list-caption")],
        vec![text(label)],
    )
}

/// `reviews-stacked-list` の Demo 本体（3 variant 併記）。呼び出しごとに
/// 同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-reviews-stacked-list-stack")],
        vec![
            caption("投稿者・評価・本文を 3 列化（広幅）"),
            instance_three_column(),
            caption("アバター付き・投稿者列と本文を左右分離"),
            instance_author_split(),
            caption("中央見出し + 平均評価 + 検索欄"),
            instance_search_header(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/reviews-stacked-list/",
    title: "reviews-stacked-list",
    category: BlockCategory::Reviews,
    rust_source: "crates/docs-site/src/blocks/ecommerce/reviews/reviews_stacked_list.rs",
    demo_class: "blocks-reviews-stacked-list",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Rating Group",
            path: "/themes/rating-group/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `reviews_stacked_list` 固有のレイアウト規則（`--fandhe-*` トークンの
/// みを使用）。既定（狭幅）は縦積み、`40rem` 以上でコンテナクエリにより
/// `three-column`/`author-split` を grid 化する（モジュール doc
/// 「レスポンシブはコンテナクエリ」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-reviews-stacked-list-stack {\n  container-type: inline-size;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-reviews-stacked-list-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-reviews-stacked-list-layout {\n  position: relative;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-reviews-stacked-list-hidden-heading {\n  position: absolute;\n}\n\
.blocks-reviews-stacked-list-heading {\n  text-align: center;\n}\n\
.blocks-reviews-stacked-list-summary {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  text-align: center;\n}\n\
.blocks-reviews-stacked-list-summary-score {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-reviews-stacked-list-summary [data-scope=\"input-group\"][data-part=\"root\"] {\n  max-inline-size: 20rem;\n}\n\
.blocks-reviews-stacked-list-row {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-reviews-stacked-list-meta {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-reviews-stacked-list-meta-name {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-reviews-stacked-list-author {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-reviews-stacked-list-content {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-reviews-stacked-list-body h4 {\n  margin: 0 0 var(--fandhe-space-1) 0;\n}\n\
.blocks-reviews-stacked-list-body p {\n  margin: 0;\n}\n\
@container (min-width: 40rem) {\n  \
.blocks-reviews-stacked-list-layout[data-blocks-reviews-stacked-list-variant=\"three-column\"] .blocks-reviews-stacked-list-row {\n    display: grid;\n    grid-template-columns: minmax(0, 12rem) auto minmax(0, 1fr);\n    align-items: start;\n    gap: var(--fandhe-space-4);\n  }\n  \
.blocks-reviews-stacked-list-layout[data-blocks-reviews-stacked-list-variant=\"author-split\"] .blocks-reviews-stacked-list-row {\n    display: grid;\n    grid-template-columns: minmax(0, 14rem) minmax(0, 1fr);\n    align-items: start;\n    gap: var(--fandhe-space-4);\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use std::collections::HashSet;

    /// [`demo`] が 10 部品すべてを正しい属性・文言で出力すること
    /// （`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"avatar\"",
            "data-scope=\"rating-group\"",
            "data-scope=\"text\"",
            "data-scope=\"separator\"",
            "data-scope=\"input-group\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"badge\"",
            "data-scope=\"button\"",
            "data-scope=\"visually-hidden\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"alt="""#));
        assert!(html.contains(r#"aria-label="レビューを検索""#));
        assert!(html.contains("<time"));
        assert!(html.contains("購入者"));
    }

    /// [`demo`] が `<form>`・不正リンク・`data:` URI・`<script` のいずれも
    /// 含まないこと（`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
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

    /// 星評価 label の `id` が重複しないこと（モジュール doc「星評価は
    /// readonly、label id は行ごとに一意化」節参照）。
    #[test]
    fn rating_label_ids_are_unique() {
        let html = render(&demo());
        let mut ids = HashSet::new();
        for part in html.split("id=\"").skip(1) {
            if let Some(end) = part.find('"') {
                let id = &part[..end];
                if id.starts_with("blocks-reviews-stacked-list-rating-") {
                    assert!(
                        ids.insert(id.to_string()),
                        "duplicate rating label id: {id}"
                    );
                }
            }
        }
        assert!(
            ids.len() >= 8,
            "expected at least 8 rating label ids, got {}",
            ids.len()
        );
    }

    /// [`LAYOUT_CSS`] が全主要セレクタ・コンテナクエリ・トークン参照を
    /// 持ち、色リテラルを含まないこと。
    #[test]
    fn layout_css_uses_tokens_and_container_query() {
        for selector in [
            ".blocks-reviews-stacked-list-stack {",
            ".blocks-reviews-stacked-list-layout {",
            ".blocks-reviews-stacked-list-row {",
            ".blocks-reviews-stacked-list-author {",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-space-4)"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(!LAYOUT_CSS.contains("black"));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// ルート class が出力に現れ、かつ `BLOCK.demo_class` とは異なること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("blocks-reviews-stacked-list-stack"));
        assert_ne!("blocks-reviews-stacked-list-stack", BLOCK.demo_class);
    }

    /// 3 variant がそれぞれ 1 回ずつ出ること。
    #[test]
    fn demo_renders_three_variants() {
        let html = render(&demo());
        assert!(html.contains("data-blocks-reviews-stacked-list-variant=\"three-column\""));
        assert!(html.contains("data-blocks-reviews-stacked-list-variant=\"author-split\""));
        assert!(html.contains("data-blocks-reviews-stacked-list-variant=\"search-header\""));
    }
}
