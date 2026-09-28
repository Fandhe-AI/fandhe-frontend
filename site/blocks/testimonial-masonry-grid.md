# testimonial-masonry-grid

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `card` /
`blockquote` / `avatar` / `icon` の 6 部品を合成した、高さ不揃いの推薦文
グリッドです。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください。

長さの異なる推薦文を CSS multi-column（`column-count`）で高さを揃えずに
敷き詰める 3 案を縦に並記しています。列数は狭い幅で 1 列、`>= 40rem` で
2 列、`>= 64rem` で 3 列、`>= 80rem` で 4 列に切り替わります（案 C のみ
`>= 80rem` でも最大 3 列に留めます）。本 Demo は静的な表示例であり、
人名・役職・社名・推薦文はすべて架空の固定値です。

- **案 A**: 8 枚を multi-column に敷き詰め、先頭・末尾のカードをアクセント
  カラーの枠線と拡大した引用文フォントサイズで featured 強調します。
- **案 B**: featured カード 1 枚を multi-column 容器の外（直前）に全幅で
  置き、その下の multi-column に通常カード 10 枚を敷き詰めます。
- **案 C**: featured なしの multi-column 9 枚です。`>= 80rem` でも最大
  3 列に留め、3 列 × 3 段で読める密度にしています。

各案の先頭には案の種別を示すラベルを添えています。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 11 件の推薦文（長さをわざと不揃いにし、multi-column の高さ不揃いを
/// Demo 上に出す。3 案（[`variant_a`]/[`variant_b`]/[`variant_c`]）が
/// それぞれ必要な件数だけスライスして使う）。
const QUOTES: [&str; 11] = [
    "導入して最初の週から、チーム全員の作業状況が一目で分かるようになりました。以前は週次の進捗確認会議に時間を取られていましたが、今ではダッシュボードを見るだけで十分です。ドキュメントも整備されていて、新しいメンバーの立ち上げも驚くほど早くなりました。",
    "サポートの反応が早く安心して使えます。",
    "既定のエスケープのおかげで、レビューで指摘される脆弱性がほぼゼロになりました。",
    "他のツールから乗り換えましたが、学習コストが低く、すぐに定着しました。",
    "設定ファイルが一目で分かるので、運用の引き継ぎが楽になりました。",
    "単一バイナリで配布できる点が、運用チームにとても好評です。",
    "細部まで作り込まれた操作感で、初めて触ったメンバーもすぐに馴染めました。",
    "料金プランの見直しを機に導入しましたが、機能面でも満足しています。特にレポート機能が充実していて、経営層への報告資料をそのまま出力できるのは大きな時短になりました。今後も長く使い続けたいツールです。",
    "導入前は複数ツールを併用していましたが、統合されたことで管理コストが大幅に減りました。",
    "ドキュメントが充実しているため、トラブル時も自己解決できることが多いです。",
    "チームの規模が大きくなっても、権限管理がシンプルなまま扱えています。",
];

/// 推薦文の装飾アイコン（引用符。参照元の形状は持ち込まない独自図形。
/// 常に `aria-hidden`、モジュール doc「装飾アイコン・チェックの a11y
/// 判断」節参照）。
fn quote_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Md,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                (
                    "d",
                    "M7 8c-1.7 0-3 1.3-3 3v5h5v-5H7c0-1.1.9-2 2-2V8zm9 0c-1.7 0-3 1.3-3 3v5h5v-5h-2c0-1.1.9-2 2-2V8z",
                ),
                ("fill", "currentColor"),
            ],
            vec![],
        )],
    )
}

/// セクション見出し（H3 見出し + リード文）。
fn section_header() -> Node {
    div(
        vec![("class", "blocks-testimonial-masonry-grid-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("利用者の声")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "長さの異なる推薦文を高さを揃えずに敷き詰める 3 案を並記しています。",
                )],
            ),
        ],
    )
}

/// 案の種別を示す状態並記の見出し（モジュール doc「3 案の並記」節、
/// `pricing_tiers_extra_row::state_label` と同型）。
fn state_label(label: &str) -> Node {
    span(
        vec![("class", "blocks-testimonial-masonry-grid-state-label")],
        vec![text(label)],
    )
}

/// 推薦文カード 1 枚（`index` は [`dummy_assets`] の人名・役職・社名を
/// 引くためのオフセット、`quote` は表示する推薦文、`featured` は強調
/// 表示の有無）。
fn testimonial_card(index: usize, quote: &str, featured: bool) -> Node {
    let (variant, card_state) = if featured {
        (CardVariant::Elevated, "featured")
    } else {
        (CardVariant::Outline, "default")
    };
    let name = dummy_assets::PERSON_NAMES[index % dummy_assets::PERSON_NAMES.len()];
    let job_title = dummy_assets::JOB_TITLES[index % dummy_assets::JOB_TITLES.len()];
    let company = dummy_assets::COMPANY_NAMES[index % dummy_assets::COMPANY_NAMES.len()];

    card::root(
        CardProps {
            variant,
            ..CardProps::default()
        },
        vec![("data-blocks-testimonial-masonry-grid-card", card_state)],
        vec![card::body(
            vec![],
            vec![
                quote_icon(),
                blockquote::root(
                    BlockquoteVariant::default(),
                    ColorPalette::default(),
                    vec![("data-blocks-testimonial-masonry-grid-quote", "")],
                    vec![
                        blockquote::content(vec![], vec![text(quote)]),
                        blockquote::caption(
                            vec![("class", "blocks-testimonial-masonry-grid-meta")],
                            vec![
                                avatar::root(
                                    &AvatarProps::default(),
                                    vec![("data-blocks-testimonial-masonry-grid-avatar", "")],
                                    vec![avatar::image(
                                        ImageStatus::Loaded,
                                        dummy_assets::AVATAR_SRC,
                                        "",
                                        vec![],
                                    )],
                                ),
                                div(
                                    vec![("class", "blocks-testimonial-masonry-grid-byline")],
                                    vec![
                                        span(vec![], vec![text(name)]),
                                        span(
                                            vec![("class", "blocks-testimonial-masonry-grid-role")],
                                            vec![text(format!("{job_title}, {company}"))],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                    ],
                ),
            ],
        )],
    )
}

/// 案 A（主参照 R0361、モジュール doc「3 案の並記」節参照）。前半 #2887
/// の実装そのまま: 8 枚を multi-column に敷き詰め、先頭・末尾を
/// featured にする。
fn variant_a() -> Node {
    let cards: Vec<Node> = QUOTES[0..8]
        .iter()
        .enumerate()
        .map(|(index, quote)| testimonial_card(index, quote, index == 0 || index == 7))
        .collect();
    div(
        vec![
            ("class", "blocks-testimonial-masonry-grid-variant"),
            ("data-blocks-testimonial-masonry-grid-variant", "a"),
        ],
        vec![
            state_label("案 A: 先頭と末尾を強調"),
            div(
                vec![("class", "blocks-testimonial-masonry-grid-grid")],
                cards,
            ),
        ],
    )
}

/// 案 B（R1365、モジュール doc「3 案の並記」節参照）。featured カード
/// 1 枚を multi-column 容器の外（直前）に全幅で置き、その下の
/// multi-column に通常カード 10 枚を敷き詰める。
fn variant_b() -> Node {
    let featured = testimonial_card(0, QUOTES[0], true);
    let cards: Vec<Node> = QUOTES[1..11]
        .iter()
        .enumerate()
        .map(|(offset, quote)| testimonial_card(offset + 1, quote, false))
        .collect();
    div(
        vec![
            ("class", "blocks-testimonial-masonry-grid-variant"),
            ("data-blocks-testimonial-masonry-grid-variant", "b"),
        ],
        vec![
            state_label("案 B: featured 1 枚 + 通常 10 枚"),
            div(
                vec![(
                    "class",
                    "blocks-testimonial-masonry-grid-featured-standalone",
                )],
                vec![featured],
            ),
            div(
                vec![("class", "blocks-testimonial-masonry-grid-grid")],
                cards,
            ),
        ],
    )
}

/// 案 C（R1366、モジュール doc「3 案の並記」節参照）。featured なしの
/// multi-column 9 枚。`>= 80rem` でも最大 3 列に留める（[`LAYOUT_CSS`] の
/// 案 C 専用オーバーライドセレクタ参照）。
fn variant_c() -> Node {
    let cards: Vec<Node> = QUOTES[0..9]
        .iter()
        .enumerate()
        .map(|(index, quote)| testimonial_card(index, quote, false))
        .collect();
    div(
        vec![
            ("class", "blocks-testimonial-masonry-grid-variant"),
            ("data-blocks-testimonial-masonry-grid-variant", "c"),
        ],
        vec![
            state_label("案 C: 強調なしの 9 枚"),
            div(
                vec![("class", "blocks-testimonial-masonry-grid-grid")],
                cards,
            ),
        ],
    )
}

/// `testimonial-masonry-grid` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。見出しの後に 3 案（[`variant_a`]/[`variant_b`]/
/// [`variant_c`]）を縦に並べる。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-testimonial-masonry-grid-layout")],
        vec![section_header(), variant_a(), variant_b(), variant_c()],
    )
}
```

## 原案差分メモ

参照（主参照は対応表 ID R0361、集約元は対応表 ID R1365/R1366。出典の
固有名・ファイル名は記載しません）から取り込んだのは構造（列数・強調の
有無・カードの配置）のみであり、次の点を独自に設計・変更しています。

- R0361 が持つ「先頭・末尾カードが 2 行分の高さにまたがる」形は
  CSS multi-column では表現できない（`column-count` は行の概念を持たず、
  複数列にまたがる `span` 指定は列方向のみで行方向には存在しない）ため、
  枠線とフォントサイズによる視覚的な強調差別化に置き換えています。
- R1365 の featured 1 枚 + 通常 10 枚の構成は、featured カードを
  multi-column 容器の外（直前）に全幅で置くことで表現しています。
  CSS Grid の 2×2 span は行の高さが揃ってしまい masonry にならず、
  `column-span: all` は inline-block のカードには効かないため、いずれも
  採用していません。
- R1366 の CSS multi-column 9 枚は featured なしとしていますが、
  `>= 80rem` でも列数を 3 列に留める点は R1366 の意図（3 列 × 3 段で
  読める密度）を汲んだ独自の調整です。
- 人名・役職・社名・推薦文の文言はすべて架空のもの（実在の企業名・PII
  を含まない）であり、参照元の配色・装飾は持ち込んでいません。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Card](../themes/card.md) / [Blockquote](../themes/blockquote.md) /
[Avatar](../themes/avatar.md) / [Icon](../themes/icon.md)
