# footer-newsletter-band

帯が全幅の footer。A: 帯下。B: 帯上。C: 簡素形。`<form>` なし。

## Rust コード

```rust
use fandhe_frontend_core::{div, footer, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};

const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

fn link(label: &'static str) -> Node {
    link::root(REPO, &LinkProps::default(), vec![], vec![text(label)])
}

fn top() -> Node {
    div(vec![("class", "fnb-t")], vec![link("製品")])
}

fn band(id: &'static str) -> Node {
    let f = FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let signup = div(
        vec![],
        vec![
            field::root(
                &FieldRootProps {
                    orientation: FieldOrientation::Vertical,
                },
                &f,
                vec![],
                vec![
                    field::label(&f, vec![], vec![text("メール")]),
                    input::input(&InputProps::default(), &f, vec![("type", "email")]),
                ],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("購読")]),
        ],
    );
    div(vec![("data-fnb-band", "")], vec![text("最新情報"), signup])
}

fn variant(order: u8, id: &'static str) -> Node {
    let mut body = match order {
        0 => vec![top(), band(id)],
        1 => vec![band(id), top()],
        _ => vec![div(vec![("class", "fnb-c")], vec![link("製品"), band(id)])],
    };
    body.push(separator(&SeparatorProps::default(), vec![]));
    body.push(div(vec![], vec![text("© 2026")]));
    footer(vec![], body)
}

pub fn demo() -> Node {
    div(
        vec![("class", "fnb-l")],
        vec![
            text("A"),
            variant(0, "fnb-a"),
            text("B"),
            variant(1, "fnb-b"),
            text("C"),
            variant(2, "fnb-c"),
        ],
    )
}
```

関連情報: [Field](../themes/field.md) / [Input](../themes/input.md) /
[Button](../themes/button.md) / [Link](../themes/link.md) /
[Separator](../themes/separator.md)
