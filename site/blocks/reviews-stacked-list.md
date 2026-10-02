# reviews-stacked-list

レビューを区切り線で仕切って縦に並べる一覧ブロックです。投稿者名・
日付・星評価・タイトル・本文を持つ行を、`heading` / `avatar` /
`rating-group` / `text` / `separator` / `input-group` / `input` /
`badge` / `button` / `visually-hidden` の 10 部品で合成します。主参照は
対応表 ID R1218（広幅で投稿者・評価・本文の 3 列化）です。Blocks は
既存部品の合成例であり、新しい UI 部品は追加しません。

3 つの構成を縦に併記します。

- **three-column**: 広幅で「投稿者・日付」「星評価」「タイトル・本文」
  を 3 列に並べ、狭幅では縦積みにします。見出しは視覚的に隠します
- **author-split**: アバター付きの行で、左に投稿者列（アバター・
  氏名・日付）、右に星評価とレビュー本文を置きます。見出しは視覚的に
  隠します
- **search-header**: 中央寄せの可視見出しの下に平均評価と検索欄を
  持ち、各行に購入者バッジを付けます

星評価はすべて readonly（他ユーザーが付けた評価の表示）で、架空の
投稿者名・レビュー文言・日付のみを使います。実在の企業・ブランド・
PII・実クレデンシャルは含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。検索欄
の送信ボタンは `type="button"` の静的ボタンで、送信処理・絞り込み・
ページングは一切持ちません。

## Rust コード

```rust
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
                        vec![text("4.6 / 5（128 件のレビュー）")],
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
```

## 原案差分メモ

主参照 R1218 のほか、3 件の集約元を次のとおり割り当てています。

- **R1218（主参照・`three-column`）**: 広幅で投稿者・評価・本文を
  3 列に並べる構成をそのまま採用しています
- **R1220・R1221（`author-split` へ統合）**: 投稿者列と本文を左右に
  分離する構成（R1220）と、アバター付きで表示する構成（R1221）を
  1 つの variant に統合しました
- **R0216（`search-header`）**: 中央見出し + 平均評価 + 検索欄を上部
  に持つ構成を採用しています。検索欄は本 Demo では静的表示のみで
  送信先を持ちません
- **配色・文言**: 参照元の文言・配色・装飾・アイコンは持ち込まず、
  `--fandhe-color-*`/`--fandhe-space-*` トークンのみで組み直し、
  レビュー文言・投稿者名はすべて独自の架空データです
- **レイアウト**: `40rem` 未満のコンテナ幅では全 variant とも縦積みに
  し、`container-type: inline-size` によるコンテナクエリで
  `three-column`/`author-split` の行を grid 化します

関連情報: [Heading](../themes/heading.md) / [Avatar](../themes/avatar.md) /
[Rating Group](../themes/rating-group.md) / [Text](../themes/text.md) /
[Separator](../themes/separator.md) /
[Input Group](../themes/input-group.md) / [Input](../themes/input.md) /
[Badge](../themes/badge.md) / [Button](../themes/button.md) /
[Visually Hidden](../themes/visually-hidden.md)
