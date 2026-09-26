# faq-static-grid

常時表示の FAQ グリッド。文言は架空です。

## Rust コード

```rust
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// Q&A。
const FAQS: [(&str, &str); 3] = [
    ("招待上限は", "プラン次第"),
    ("無料期間は", "14日間無料"),
    ("解約方法は", "いつでも。"),
];

/// 導入部。
fn header() -> Node {
    div(
        vec![("class", "blocks-faq-static-grid-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("FAQ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![link::root(
                    REPO,
                    &LinkProps {
                        variant: LinkVariant::Underline,
                        ..LinkProps::default()
                    },
                    vec![],
                    vec![text("GitHub")],
                )],
            ),
        ],
    )
}

/// FAQ 1 件分。
fn faq_entry(question: &str, answer: &str) -> Node {
    div(
        vec![("class", "blocks-faq-static-grid-item")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(question)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(answer)],
            ),
        ],
    )
}

/// FAQ グリッド本体。
fn faq_grid() -> Node {
    div(
        vec![("class", "blocks-faq-static-grid-grid")],
        FAQS.iter()
            .map(|(question, answer)| faq_entry(question, answer))
            .collect(),
    )
}

/// ボタン行。
fn contact() -> Node {
    div(
        vec![("class", "blocks-faq-static-grid-actions")],
        vec![
            button::button(&ButtonProps::default(), vec![], vec![text("問合せ")]),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("資料")],
            ),
        ],
    )
}

/// Demo 本体。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-faq-static-grid-layout")],
        vec![header(), faq_grid(), contact()],
    )
}
```
