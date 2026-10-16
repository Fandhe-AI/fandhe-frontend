# auth-device-approve

`fandhe-frontend-pre-styled-ui` の `card` / `pin_input` / `button` / `visually_hidden`
部品を合成した、別端末が表示したユーザーコードを確認して承認・拒否する画面の合成例です
（RFC 8628 の Device Authorization Grant で、TV や CLI のような入力に制約のあるデバイスが
`WDJB-MJHT` 形の短いコードを表示する場面を想定しています）。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください。

本 Demo は静的な表示例であり、`<form>` 要素を持ちません。承認・拒否の送信、認可判定、
トークン発行、レート制限はアプリケーション側の責務です
（`docs/policy/intentional-non-adoption.md` §3.25）。各桁は `name` を持たず、値を運ぶ経路はありません。
ハイフン入りコードの貼り付けや大文字化といった実際の入力挙動は Phase 4 で扱い、本 Block では扱いません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::pin_input::{self, PinInputKind, PinInputProps};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 表示するダミーのユーザーコード（母音を含まない架空パターン、4-4 区切りで表示）。
const CODE: [&str; 8] = ["B", "C", "D", "F", "G", "H", "J", "K"];

const LABEL_ID: &str = "blocks-auth-device-approve-label";
const CONTROL_ID: &str = "blocks-auth-device-approve-control";

/// `range` の桁を `input` + 読み上げ補助の組として並べる。index は通し番号、
/// count は常に全桁数（`aria-label` の "N of 8" と `data-index` を保つ）。
fn code_digits(range: std::ops::Range<usize>, props: &PinInputProps) -> Vec<Node> {
    range
        .flat_map(|index| {
            let digit_id = format!("blocks-auth-device-approve-digit-{}", index + 1);
            let labelledby = format!("{LABEL_ID} {digit_id}");
            let mut attrs: Vec<(&str, &str)> = vec![("aria-labelledby", labelledby.as_str())];
            if index == 0 {
                attrs.push(("id", CONTROL_ID));
            }
            let input = pin_input::input(
                index,
                CODE.len(),
                CODE[index],
                PinInputKind::Alphanumeric,
                false,
                false,
                props,
                true,
                attrs,
            );
            let digit_text = visually_hidden::root(
                vec![("id", digit_id.as_str())],
                vec![text(format!("character {} of {}", index + 1, CODE.len()))],
            );
            vec![input, digit_text]
        })
        .collect()
}

/// 接続元情報の 1 行（`dt` + `dd`）。
fn meta_row(term: &str, detail: &str) -> Vec<Node> {
    vec![
        el("dt", vec![], vec![text(term)]),
        el("dd", vec![], vec![text(detail)]),
    ]
}

/// `auth-device-approve` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let props = PinInputProps {
        required: true,
        ..PinInputProps::default()
    };
    let mut meta: Vec<Node> = Vec::new();
    meta.extend(meta_row("Device", "Demo Media Player"));
    meta.extend(meta_row("Requested", "2 minutes ago"));
    meta.extend(meta_row("IP address", "203.0.113.24"));

    card::root(
        CardProps::default(),
        vec![("data-blocks-auth-device-approve-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("Confirm device sign-in")]),
                    card::description(
                        vec![],
                        vec![text(
                            "A device is asking to sign in to your account. Approve only if the code below matches the one shown on that device.",
                        )],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![div(
                    vec![("class", "blocks-auth-device-approve-stack")],
                    vec![
                        el(
                            "dl",
                            vec![("class", "blocks-auth-device-approve-meta")],
                            meta,
                        ),
                        pin_input::root(
                            Size::Md,
                            true,
                            false,
                            vec![("data-blocks-auth-device-approve-code", "")],
                            vec![
                                pin_input::label(
                                    true,
                                    &props,
                                    vec![("id", LABEL_ID), ("for", CONTROL_ID)],
                                    vec![text("User code")],
                                ),
                                div(
                                    vec![("class", "blocks-auth-device-approve-groups")],
                                    vec![
                                        pin_input::control(vec![], code_digits(0..4, &props)),
                                        pin_input::separator(vec![], vec![text("-")]),
                                        pin_input::control(vec![], code_digits(4..8, &props)),
                                    ],
                                ),
                            ],
                        ),
                        el(
                            "p",
                            vec![("class", "blocks-auth-device-approve-help")],
                            vec![text(
                                "Make sure this code matches the one on your device. Never approve a code someone else sent you.",
                            )],
                        ),
                    ],
                )],
            ),
            card::footer(
                vec![("data-blocks-auth-device-approve-actions", "")],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("Deny")],
                    ),
                    button::button(&ButtonProps::default(), vec![], vec![text("Approve")]),
                ],
            ),
        ],
    )
}
```

## 原案差分メモ

- **4-4 区切り**: `pin_input::separator` を 1 個挟みます。`pin_input::root` は縦積みが既定のため、
  横並びにする `blocks-auth-device-approve-groups` ラッパーを呼び出し側で用意し、インライン style は使いません。
- **値はすべてダミー**: `WDJB-MJHT` は形式の例で、Demo の値には使いません。コードは母音を含まない架空パターン、
  IP は RFC 5737 の TEST-NET-3、デバイス名は架空の汎用名です。
- **`otp = false` / Alphanumeric**: ユーザーコードは SMS やメールで届くワンタイムコードではないため、
  `autocomplete="one-time-code"` による自動入力誘導を避けます。
- **全桁入力済みで固定**: 表示されたコードとの照合が目的の画面で、`verification_uri_complete` 経由の事前入力を想定します。
- **注意文**: 他人から送られたコードを承認しないよう促し、デバイスコードフィッシングを防ぐ観点を含めます。
- **ボタン**: Deny は Outline、Approve は Solid。どちらも `type="button"` です。
- **ロゴ・商標は持ち込まない**: サービスやデバイスのロゴは使いません。

関連情報: [Card](../themes/card.md) / [Pin Input](../themes/pin-input.md) /
[Button](../themes/button.md) / [Visually Hidden](../themes/visually-hidden.md)
