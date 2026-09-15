# cta-signup-celebrate

`fandhe-frontend-pre-styled-ui` の `card` / `field` / `input` / `button`
部品を合成した、Motion+ `sections/cta-sections` の signup celebrate に相当
する合成例です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、送信ボタンは
`type="button"` のまま送信先を持ちません。メールアドレスの入力値・完了
メッセージはいずれも架空のものであり、実企業名・実クレデンシャル・PII を
含みません。送信ボタンに付与している `data-fandhe-confetti-trigger` と
canvas に付与している `data-fandhe-confetti-canvas` は、`fandhe-frontend-
wasm-full` の `confetti` feature（既定 on、イシュー #2533 で実装済み）が
そのまま消費する既存の opt-in 属性の組であり、本 block では confetti 自体を
新規実装せず既存機構を「使う側」として合成しています。本 docs サイトは JS
ハイドレーションを一切行わないため、本 Demo ではクリック委譲・confetti の
発火は発生しません。代わりに「送信前」カードと「送信完了（celebrate）」
カードを静的に併記し、実アプリで両属性が付与されたときの見た目を示します。
実際の送信処理・バリデーションは利用者自身の Rust コードで書いてください
（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポー
ネント層はアプリケーションロジックを内包しません）。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::Size;

/// confetti 発火対象 canvas の `id`（`data-fandhe-confetti-trigger` の値と
/// 一致させる、モジュール doc「confetti の使用」節参照）。
const CANVAS_ID: &str = "cta-signup-celebrate-canvas";

/// 自作のチェックマークアイコン（著作物を複製しない、`signup_05`/
/// `sidebar_03` の `geo_icon` と同型）。
fn checkmark_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Lg,
            ..IconProps::default()
        },
        vec![("data-blocks-cta-signup-celebrate-check", "")],
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

/// 「送信前」カード（email 入力 + confetti トリガー付き送信ボタン +
/// canvas）。
fn before_submit_card() -> Node {
    let email_field = FieldProps {
        id: "blocks-cta-signup-celebrate-email",
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

    card::root(
        CardProps::default(),
        vec![("data-blocks-cta-signup-celebrate-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("Get notified at launch")]),
                    card::description(
                        vec![],
                        vec![text("We will only email you once, when we ship.")],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![
                    field::root(
                        &orientation,
                        &email_field,
                        vec![("data-blocks-cta-signup-celebrate-field", "")],
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
                        vec![
                            ("data-blocks-cta-signup-celebrate-submit", ""),
                            ("data-fandhe-confetti-trigger", CANVAS_ID),
                        ],
                        vec![text("Notify me")],
                    ),
                    el(
                        "canvas",
                        vec![
                            ("id", CANVAS_ID),
                            ("data-fandhe-confetti-canvas", ""),
                            ("data-blocks-cta-signup-celebrate-canvas", ""),
                            ("aria-hidden", "true"),
                        ],
                        vec![],
                    ),
                ],
            ),
        ],
    )
}

/// 「送信完了（celebrate）」カード（チェックマーク + 完了メッセージ）。
fn after_submit_card() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-cta-signup-celebrate-card", "")],
        vec![card::body(
            vec![("data-blocks-cta-signup-celebrate-celebrate", "")],
            vec![
                checkmark_icon(),
                card::title(vec![], vec![text("You're all set!")]),
                card::description(vec![], vec![text("We'll email you the moment we launch.")]),
            ],
        )],
    )
}

/// `cta-signup-celebrate` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-cta-signup-celebrate-stack", "")],
        vec![
            div(
                vec![("data-blocks-cta-signup-celebrate-caption", "")],
                vec![text("Before submission")],
            ),
            before_submit_card(),
            div(
                vec![("data-blocks-cta-signup-celebrate-caption", "")],
                vec![text("After submission (celebrate)")],
            ),
            after_submit_card(),
        ],
    )
}
```
