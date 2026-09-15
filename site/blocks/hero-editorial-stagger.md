# hero-editorial-stagger

`badge` / `heading` / `text` / `button` の合成例（既存部品のみ、新規部品
なし）。上から順に遅延フェードイン（JS 不要、CSS `animation-delay`）し、
`prefers-reduced-motion: reduce` では消えます。`<form>` は持ちません。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::recipe::stagger_index_style;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};

const ITEM_ATTR: &str = "data-blocks-hero-editorial-stagger-item";

pub fn demo() -> Node {
    let eyebrow_style = stagger_index_style(0);
    let eyebrow = badge::badge(
        &BadgeProps::default(),
        vec![(ITEM_ATTR, ""), ("style", eyebrow_style.as_str())],
        vec![text("New: workspace insights")],
    );

    let heading_style = stagger_index_style(1);
    let title = heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl4,
            ..HeadingProps::default()
        },
        vec![(ITEM_ATTR, ""), ("style", heading_style.as_str())],
        vec![text("Ship features your team can trust")],
    );

    let lead_style = stagger_index_style(2);
    let lead = styled_text::text(
        &TextProps {
            size: TextSize::Lg,
            ..TextProps::default()
        },
        vec![(ITEM_ATTR, ""), ("style", lead_style.as_str())],
        vec![text("Ship with confidence, review with ease.")],
    );

    let actions_style = stagger_index_style(3);
    let actions = div(
        vec![
            ("class", "blocks-hero-editorial-stagger-actions"),
            (ITEM_ATTR, ""),
            ("style", actions_style.as_str()),
        ],
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

    div(
        vec![("class", "blocks-hero-editorial-stagger-inner")],
        vec![eyebrow, title, lead, actions],
    )
}
```
