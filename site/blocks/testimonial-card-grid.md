# testimonial-card-grid

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `card` / `blockquote` /
`avatar` / `button` / `icon` 部品を合成した、推薦文カードグリッドのセクション
です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R0360、集約元は R0355・R0727。出典の固有名・ファイル名は
記載しません）。

中央寄せの見出し + リード文の下に、推薦文カードを 3 列（狭い幅では 1 列）で
並べます。カード 1 枚は任意のロゴ・引用文・著者行（アバター・氏名・役職）で
構成します。主参照（見出しの下に CTA ボタン、カード 6 枚）と集約元（カード内
にロゴ、カード 3 枚）の 2 バリエーションを静的インスタンスとして並べています。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。CTA ボタンは `type="button"` のまま送信先を
持ちません。文言・氏名・社名はすべて独自に書いた架空のものであり、実企業名・
実クレデンシャル・PII を含みません。ロゴは実在ブランドを模さない自作の
幾何図形です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::ColorPalette;

/// 推薦文 1 件分の静的データ（架空、実企業名・PII を含まない）。
struct Testimonial {
    quote: &'static str,
    name: &'static str,
    role: &'static str,
    company: &'static str,
}

/// `with-cta`（R0360）用の 6 件。
const TESTIMONIALS_WITH_CTA: [Testimonial; 6] = [
    Testimonial {
        quote: "導入初日から操作に迷うことがなく、チーム全員がすぐに使いこなせました。",
        name: "Haruto Fujimaki",
        role: "Product Designer",
        company: "Lumenbridge Systems",
    },
    Testimonial {
        quote: "サポートの返信が早く、細かな要望にも丁寧に対応してもらえます。",
        name: "Elena Vasquez",
        role: "Engineering Lead",
        company: "Verdant Foundry",
    },
    Testimonial {
        quote: "画面構成がシンプルで、新しいメンバーの立ち上がりが早くなりました。",
        name: "Kwame Boateng",
        role: "Customer Success Manager",
        company: "Quill & Meridian",
    },
    Testimonial {
        quote: "既定の設定だけで十分に安心して使えるところが気に入っています。",
        name: "Mei Lindqvist",
        role: "Data Analyst",
        company: "Trellisworks Co.",
    },
    Testimonial {
        quote: "他社製品からの乗り換えでしたが、移行の手間はほとんどありませんでした。",
        name: "Noor Al-Sayed",
        role: "Marketing Strategist",
        company: "Aurelia Dynamics",
    },
    Testimonial {
        quote: "運用チームからも「管理がしやすくなった」と好評です。",
        name: "Ola Bergström",
        role: "Operations Coordinator",
        company: "Northshelf Logistics",
    },
];

/// `with-logo`（R0355）用の 3 件。
const TESTIMONIALS_WITH_LOGO: [Testimonial; 3] = [
    Testimonial {
        quote: "導入からわずか数週間で、チーム全体の作業が驚くほど整理されました。",
        name: "Priya Chandran",
        role: "Customer Success Manager",
        company: "Lumenbridge Systems",
    },
    Testimonial {
        quote: "細部まで作り込まれた操作感で、初めて触ったメンバーもすぐに馴染めました。",
        name: "Théo Marchetti",
        role: "Data Analyst",
        company: "Verdant Foundry",
    },
    Testimonial {
        quote: "他のツールと比べて圧倒的にシンプルで、迷わず使い続けられています。",
        name: "Haruto Fujimaki",
        role: "Marketing Strategist",
        company: "Quill & Meridian",
    },
];

/// Demo が並記する 1 インスタンス分。
struct Instance {
    /// `data-blocks-testimonial-card-grid-variant` の値。
    variant: &'static str,
    /// 状態ラベル（見出しにはしない、モジュール doc「見出しレベル」節）。
    label: &'static str,
    testimonials: &'static [Testimonial],
    /// カード先頭にロゴ（装飾用 icon）を置くか。
    with_logo: bool,
}

const INSTANCES: [Instance; 2] = [
    Instance {
        variant: "with-cta",
        label: "見出し下に CTA・カード 6 枚（主参照。R0727 の基本形を包含）",
        testimonials: &TESTIMONIALS_WITH_CTA,
        with_logo: false,
    },
    Instance {
        variant: "with-logo",
        label: "カード先頭にロゴ・カード 3 枚（集約元）",
        testimonials: &TESTIMONIALS_WITH_LOGO,
        with_logo: true,
    },
];

/// ロゴ相当マーク（装飾、モジュール doc「ロゴ・アバターの a11y」節）。
/// 実在ブランドのロゴ・商標は模さない独自の六角形。
fn logo_icon() -> Node {
    icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![("data-blocks-testimonial-card-grid-logo", "")],
        vec![el(
            "path",
            vec![
                ("d", "M12 3l7.5 4.5v9L12 21l-7.5-4.5v-9L12 3z"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "1.5"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 推薦文カード 1 枚を組み立てる（`with_logo` が真ならロゴ行を先頭へ）。
fn testimonial_card(item: &Testimonial, with_logo: bool) -> Node {
    let mut body_children = Vec::new();
    if with_logo {
        body_children.push(div(
            vec![("class", "blocks-testimonial-card-grid-logo-row")],
            vec![
                logo_icon(),
                styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text(item.company)],
                ),
            ],
        ));
    }
    body_children.push(blockquote::root(
        BlockquoteVariant::default(),
        ColorPalette::default(),
        vec![],
        vec![
            blockquote::content(vec![], vec![text(item.quote)]),
            blockquote::caption(
                vec![("class", "blocks-testimonial-card-grid-meta")],
                vec![
                    avatar::root(
                        &AvatarProps::default(),
                        vec![("data-blocks-testimonial-card-grid-avatar", "")],
                        vec![avatar::image(
                            ImageStatus::Loaded,
                            dummy_assets::AVATAR_SRC,
                            "",
                            vec![],
                        )],
                    ),
                    div(
                        vec![("class", "blocks-testimonial-card-grid-byline")],
                        vec![
                            div(vec![], vec![text(item.name)]),
                            div(vec![], vec![text(item.role)]),
                        ],
                    ),
                ],
            ),
        ],
    ));

    card::root(
        CardProps::default(),
        vec![("data-blocks-testimonial-card-grid-card", "")],
        vec![card::body(
            vec![("class", "blocks-testimonial-card-grid-body")],
            body_children,
        )],
    )
}

/// header 領域（見出し・リード文・CTA。全インスタンス共有のため 1 回だけ出す）。
/// CTA は R0360「見出しの下に CTA ボタン」に対応するが、共有 header へ
/// 混ぜるとどのインスタンスの装飾か区別しにくくなるため、実際には
/// `with-cta` インスタンスのラベル行の直下に置く（モジュール doc
/// 「インスタンス並記の理由」節、原案差分メモにも補足を記載）。
fn header() -> Node {
    div(
        vec![("class", "blocks-testimonial-card-grid-header")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("お客様の声")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("導入いただいたチームから寄せられた感想の一部です。")],
            ),
        ],
    )
}

/// `with-cta` インスタンスのラベル行の直下に置く CTA ボタン
/// （モジュール doc「header」節参照）。
fn cta_row() -> Node {
    div(
        vec![("class", "blocks-testimonial-card-grid-cta-row")],
        vec![button::button(
            &ButtonProps::default(),
            vec![],
            vec![text("導入事例をもっと見る")],
        )],
    )
}

/// `testimonial-card-grid` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。header を 1 回だけ出したあと、2 バリエーションを縦に並べる
/// （モジュール doc「インスタンス並記の理由」節）。
pub fn demo() -> Node {
    let mut children = vec![header()];
    for instance in &INSTANCES {
        let mut section_children = vec![styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(instance.label)],
        )];
        if instance.variant == "with-cta" {
            section_children.push(cta_row());
        }
        let cards: Vec<Node> = instance
            .testimonials
            .iter()
            .map(|item| testimonial_card(item, instance.with_logo))
            .collect();
        section_children.push(div(
            vec![("class", "blocks-testimonial-card-grid-grid")],
            cards,
        ));

        children.push(div(
            vec![
                ("class", "blocks-testimonial-card-grid-state"),
                (
                    "data-blocks-testimonial-card-grid-variant",
                    instance.variant,
                ),
            ],
            section_children,
        ));
    }

    div(
        vec![("class", "blocks-testimonial-card-grid-layout")],
        children,
    )
}
```

## 原案差分メモ

- 主参照（対応表 ID R0360、見出しの下に CTA ボタン）は `with-cta`
  インスタンスに対応します。参照では CTA が見出し直下にありますが、
  Demo では共有 header と状態ラベルの区別を保つため、CTA ボタンは
  `with-cta` インスタンスのラベル行の直下（カード grid の上）に配置して
  います。カードは 6 枚、ロゴなしです。
- 集約元（対応表 ID R0355、カード内にロゴ）は `with-logo` インスタンス
  に対応します。カードの先頭に、装飾用の自作幾何図形（六角形）とテキスト
  の社名を並べています。アイコンは `aria-hidden="true"` の装飾扱いで、
  社名はテキストとして併記するため情報は失われません。
- 集約元（対応表 ID R0727、特記事項なし）は独立インスタンス化せず、
  基本の 3 列カードグリッドとして `with-cta` インスタンスに包含していま
  す（同じ骨格の装飾差分のみのため、`pricing-tier-cards` が鏡像の
  R1141 を扱ったのと同じ方針）。
- avatar の画像は `alt=""`（装飾扱い）にしています。隣接する氏名テキスト
  が同じ情報を伝えるためです。
- 文言・氏名・社名・ロゴ図形はすべて独自に書いた架空のものです。参照元の
  文言・配色・アイコン形状は持ち込んでいません。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Card](../themes/card.md) / [Blockquote](../themes/blockquote.md) /
[Avatar](../themes/avatar.md) / [Button](../themes/button.md) /
[Icon](../themes/icon.md)
