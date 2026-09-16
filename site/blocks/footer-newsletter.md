# footer-newsletter

`field` / `input` / `button` / `link` を合成した、Motion+
`sections/footers` の newsletter 相当の合成例です。

`<form>` を持たず、送信ボタンは `type="button"` のまま送信先を持ちません。
実際の送信処理は利用者の Rust コードで実装してください
（`docs/policy/intentional-non-adoption.md` §3.25）。

newsletter 列は「入力」panel と「完了」panel を同一列に持ち、無 JS のため
Before/After の 2 インスタンスを静的に併記します。リンク先は GitHub への
外部 URL です。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, footer, p, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 自作チェックマーク（`cta_signup_celebrate::checkmark_icon` と同型）。
fn checkmark_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Md,
            ..IconProps::default()
        },
        vec![("data-blocks-footer-newsletter-check", "")],
        vec![el(
            "path",
            vec![
                ("d", "M20 6L9 17l-5-5"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// footer リンク列 1 群（見出し + リンク一覧）。
fn link_column(heading_text: &str, links: &[(&str, &str)]) -> Node {
    let items: Vec<Node> = links
        .iter()
        .map(|(href, label)| {
            div(
                vec![("class", "blocks-footer-newsletter-link-item")],
                vec![link::root(
                    href,
                    &LinkProps::default(),
                    vec![],
                    vec![text(*label)],
                )],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-footer-newsletter-column")],
        std::iter::once(p(
            vec![("class", "blocks-footer-newsletter-column-title")],
            vec![text(heading_text)],
        ))
        .chain(items)
        .collect(),
    )
}

/// 「入力」panel（email + Subscribe）。`field_id` は 2 インスタンス併記
/// での `id` 重複防止用。
fn subscribe_panel(field_id: &'static str, hidden: bool) -> Node {
    let email_field = FieldProps {
        id: field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let orientation = FieldRootProps {
        orientation: FieldOrientation::Vertical,
    };

    let mut attrs = vec![("data-blocks-footer-newsletter-panel", "")];
    if hidden {
        attrs.push(("hidden", ""));
    }

    div(
        attrs,
        vec![
            field::root(
                &orientation,
                &email_field,
                vec![("data-blocks-footer-newsletter-field", "")],
                vec![
                    field::label(&email_field, vec![], vec![text("Email")]),
                    input::input(
                        &InputProps::default(),
                        &email_field,
                        vec![("type", "email"), ("placeholder", "m@example.com")],
                    ),
                ],
            ),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-footer-newsletter-submit", "")],
                vec![text("Subscribe")],
            ),
        ],
    )
}

/// 「完了」panel（チェックマーク + 完了文）。
fn subscribed_panel(hidden: bool) -> Node {
    let mut attrs = vec![("data-blocks-footer-newsletter-panel", "")];
    if hidden {
        attrs.push(("hidden", ""));
    }

    div(
        attrs,
        vec![
            checkmark_icon(),
            p(
                vec![("class", "blocks-footer-newsletter-subscribed-text")],
                vec![text("Subscribed! Thanks for joining.")],
            ),
        ],
    )
}

/// newsletter 列（見出し + 2 panel。`subscribed` 側を可視にする）。
fn newsletter_column(field_id: &'static str, subscribed: bool) -> Node {
    div(
        vec![("class", "blocks-footer-newsletter-column")],
        vec![
            p(
                vec![("class", "blocks-footer-newsletter-column-title")],
                vec![text("Newsletter")],
            ),
            subscribe_panel(field_id, subscribed),
            subscribed_panel(!subscribed),
        ],
    )
}

/// footer 3 カラム本体。`field_id` は 2 インスタンス併記時の `id` 重複防止。
fn footer_instance(field_id: &'static str, subscribed: bool) -> Node {
    const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
    footer(
        vec![("data-blocks-footer-newsletter-root", "")],
        vec![
            div(
                vec![("class", "blocks-footer-newsletter-column")],
                vec![
                    p(
                        vec![("class", "blocks-footer-newsletter-column-title")],
                        vec![text("Fandhe Frontend")],
                    ),
                    p(
                        vec![("class", "blocks-footer-newsletter-description")],
                        vec![text("Rust 製フロントエンドフレームワーク。")],
                    ),
                ],
            ),
            link_column("Product", &[(REPO, "Guide"), (REPO, "API Reference")]),
            link_column("Community", &[(REPO, "GitHub")]),
            newsletter_column(field_id, subscribed),
        ],
    )
}

/// `footer-newsletter` の Demo 本体（モジュール doc「2 インスタンス併記」
/// 節参照）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-footer-newsletter-stack", "")],
        vec![
            p(
                vec![("data-blocks-footer-newsletter-caption", "")],
                vec![text("Before subscribe")],
            ),
            footer_instance("blocks-footer-newsletter-email-before", false),
            p(
                vec![("data-blocks-footer-newsletter-caption", "")],
                vec![text("After subscribe")],
            ),
            footer_instance("blocks-footer-newsletter-email-after", true),
        ],
    )
}
```
