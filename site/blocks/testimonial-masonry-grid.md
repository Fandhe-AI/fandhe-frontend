# testimonial-masonry-grid

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `card` /
`blockquote` / `avatar` / `icon` の 6 部品を合成した、高さ不揃いの推薦文
グリッドです。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください。

長さの異なる 8 件の推薦文を、CSS Grid の `grid-auto-flow: row dense` で
高さを揃えずに敷き詰めます。列数は狭い幅で 1 列、`>= 40rem` で 2 列、
`>= 64rem` で 3 列、`>= 80rem` で 4 列に切り替わります。先頭と末尾の
カードは 2 行分の高さにまたがる featured カードとして強調表示されます。
本 Demo は静的な表示例であり、人名・役職・社名・推薦文はすべて架空の
固定値です。

大規模な集約元 issue を 2 分割した前半にあたり、後半（#2888）で他 2 案
（featured 1 枚 + 通常 10 枚の並記・CSS multi-column による詰め方）と、
それらの原案差分メモを追加する予定です。

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

/// 推薦文 1 件分（架空、実在の人物・企業とは無関係）。
struct Testimonial {
    quote: &'static str,
    /// 先頭・末尾のみ `true`（R0361 の featured 強調、モジュール doc
    /// 「先頭・末尾カードの featured 強調」節参照）。
    featured: bool,
}

/// 8 件の推薦文（長さをわざと不揃いにし、grid の高さ不揃いを Demo 上に
/// 出す。先頭〔index 0〕・末尾〔index 7〕が [`Testimonial::featured`]）。
const TESTIMONIALS: [Testimonial; 8] = [
    Testimonial {
        quote: "導入して最初の週から、チーム全員の作業状況が一目で分かるようになりました。以前は週次の進捗確認会議に時間を取られていましたが、今ではダッシュボードを見るだけで十分です。ドキュメントも整備されていて、新しいメンバーの立ち上げも驚くほど早くなりました。",
        featured: true,
    },
    Testimonial {
        quote: "サポートの反応が早く安心して使えます。",
        featured: false,
    },
    Testimonial {
        quote: "既定のエスケープのおかげで、レビューで指摘される脆弱性がほぼゼロになりました。",
        featured: false,
    },
    Testimonial {
        quote: "他のツールから乗り換えましたが、学習コストが低く、すぐに定着しました。",
        featured: false,
    },
    Testimonial {
        quote: "設定ファイルが一目で分かるので、運用の引き継ぎが楽になりました。",
        featured: false,
    },
    Testimonial {
        quote: "単一バイナリで配布できる点が、運用チームにとても好評です。",
        featured: false,
    },
    Testimonial {
        quote: "細部まで作り込まれた操作感で、初めて触ったメンバーもすぐに馴染めました。",
        featured: false,
    },
    Testimonial {
        quote: "料金プランの見直しを機に導入しましたが、機能面でも満足しています。特にレポート機能が充実していて、経営層への報告資料をそのまま出力できるのは大きな時短になりました。今後も長く使い続けたいツールです。",
        featured: true,
    },
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
                    "長さの異なる推薦文を高さを揃えずに敷き詰めて表示します。",
                )],
            ),
        ],
    )
}

/// 推薦文カード 1 枚（`index` は [`dummy_assets`] の人名・役職・社名を
/// 引くためのオフセット）。
fn testimonial_card(index: usize, item: &Testimonial) -> Node {
    let (variant, card_state) = if item.featured {
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
                        blockquote::content(vec![], vec![text(item.quote)]),
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

/// `testimonial-masonry-grid` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-testimonial-masonry-grid-layout")],
        vec![
            section_header(),
            div(
                vec![("class", "blocks-testimonial-masonry-grid-grid")],
                TESTIMONIALS
                    .iter()
                    .enumerate()
                    .map(|(index, item)| testimonial_card(index, item))
                    .collect(),
            ),
        ],
    )
}
```

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Card](../themes/card.md) / [Blockquote](../themes/blockquote.md) /
[Avatar](../themes/avatar.md) / [Icon](../themes/icon.md)
