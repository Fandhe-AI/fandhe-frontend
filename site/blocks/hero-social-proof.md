# hero-social-proof

`badge` / `heading` / `text` / `button` / `avatar` / `rating-group` の合成例
（既存部品のみ、新規部品なし）。タグライン・見出し・リード文・CTA の下に、
重なりアバター群 + readonly の星評価 + 利用者数の短文をまとめた社会的証明
行を置きます。星評価は他ユーザーの平均評価を表す静的表示（readonly）です。
`sm`（640px）未満では社会的証明行を縦積みにします。`<form>` は持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// [`avatar_stack`] が使う架空イニシャル 4 件（[`dummy_assets::PERSON_NAMES`]
/// 先頭 4 名から手書きした定数）。
const AVATAR_INITIALS: [&str; 4] = ["HF", "EV", "KB", "ML"];

/// [`rating`] の `label` に使う一意 id（全 block Demo 横断で衝突しない
/// よう `blocks-hero-social-proof-` 接頭辞を付ける）。
const RATING_LABEL_ID: &str = "blocks-hero-social-proof-rating-label";

/// 重なりアバター群（社会的証明の装飾。モジュール doc「アバター群を
/// `aria-hidden` にする a11y 判断」参照）。
fn avatar_stack() -> Node {
    let avatars: Vec<Node> = AVATAR_INITIALS
        .iter()
        .map(|initials| {
            avatar::root(
                &AvatarProps {
                    size: Size::Sm,
                    stacked: true,
                    ..AvatarProps::default()
                },
                vec![("data-blocks-hero-social-proof-avatar", "")],
                vec![
                    avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
                    avatar::fallback(ImageStatus::Loaded, vec![], vec![text(*initials)]),
                ],
            )
        })
        .collect();
    avatar::group(
        vec![
            ("class", "blocks-hero-social-proof-avatars"),
            ("aria-hidden", "true"),
        ],
        avatars,
    )
}

/// 星評価（readonly。他ユーザーの平均評価を表す静的表示。
/// `showcase.rs::rating_group_section` の readonly 構成と同型）。
///
/// `label`（"Average rating"）は隣接する [`text`]（"Rated 5.0/5 by
/// 12,000+ teams" の短文）と情報が重複するうえ、
/// `rating_group::label` の既定の可視表示は `.blocks-hero-social-proof-proof`
/// が横一列に並べる前提のレイアウトを崩す。`visually_hidden::root` で
/// テキストを包み、`aria-labelledby` で `control` と結びつく `id` を持つ
/// `<span>` 自体は DOM に残しつつ視覚的には隠す（`position: absolute` の
/// clip 手法のため他要素のレイアウトへ影響しない）。
fn rating() -> Node {
    let g = RatingGroup::new(5, Some(5), true);
    let props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let label = rating_group::label(
        &props,
        Some(RATING_LABEL_ID),
        vec![],
        vec![visually_hidden::root(vec![], vec![text("Average rating")])],
    );
    let items: Vec<Node> = (1..=g.count())
        .map(|i| {
            rating_group::item(
                i,
                RatingItemFlags {
                    checked: g.is_checked(i),
                    highlighted: g.is_highlighted(i),
                    disabled: false,
                    readonly: true,
                },
                &format!("{i} star{}", if i == 1 { "" } else { "s" }),
                vec![],
                vec![],
            )
        })
        .collect();
    let control = rating_group::control(&props, Some(RATING_LABEL_ID), vec![], items);
    rating_group::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-hero-social-proof-rating", "")],
        vec![label, control],
    )
}

pub fn demo() -> Node {
    let tagline = badge::badge(
        &BadgeProps::default(),
        vec![("data-blocks-hero-social-proof-tagline", "")],
        vec![text("New: workspace insights")],
    );

    let title = heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl4,
            ..HeadingProps::default()
        },
        vec![("data-blocks-hero-social-proof-heading", "")],
        vec![text("Ship features your team can trust")],
    );

    let lead = styled_text::text(
        &TextProps {
            size: TextSize::Lg,
            ..TextProps::default()
        },
        vec![("data-blocks-hero-social-proof-lead", "")],
        vec![text("Ship with confidence, review with ease.")],
    );

    let actions = div(
        vec![("class", "blocks-hero-social-proof-actions")],
        vec![
            button::button(&ButtonProps::default(), vec![], vec![text("Get started")]),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("View docs")],
            ),
        ],
    );

    let proof = div(
        vec![("class", "blocks-hero-social-proof-proof")],
        vec![
            avatar_stack(),
            rating(),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![("data-blocks-hero-social-proof-proof-text", "")],
                vec![text("Rated 5.0/5 by 12,000+ teams")],
            ),
        ],
    );

    div(
        vec![("class", "blocks-hero-social-proof-inner")],
        vec![tagline, title, lead, actions, proof],
    )
}
```
