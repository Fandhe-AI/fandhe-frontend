# auth-totp-enroll

`fandhe-frontend-pre-styled-ui` の `card` / `qr_code` / `clipboard` / `pin_input` / `field` /
`button` 部品を合成した、2 要素認証（TOTP）の登録画面の合成例です。認証アプリが読み取る
QR コード、手入力用の秘密鍵（コピー可）、数字 6 桁の確認コード入力を並べます。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、何も送信しません。コードの検証や
秘密鍵の生成はアプリケーション側の責務です（`docs/policy/intentional-non-adoption.md` §3.25）。
`otpauth` の URI・秘密鍵・発行者名はすべてダミー値で、実在の鍵やサービスではありません。
コピーは `fandhe-frontend-wasm-full` で hydrate した実アプリでのみ機能します
（無 JS の docs サイトでは静的表示です）。

## Rust コード

```rust
use fandhe_frontend_core::{code, div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::input::{FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::pin_input::{self, PinInputKind, PinInputProps};
use fandhe_frontend_pre_styled_ui::qr_code;
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// ダミーの発行者名（実在サービスではない）。
const ISSUER: &str = "DemoApp";
/// ダミーのアカウント名（`example.com` は RFC 2606 の予約ドメイン）。
const ACCOUNT: &str = "user@example.com";
/// ダミーの秘密鍵（base32 の文字集合内の明白な架空パターン）。実アプリでは
/// サーバー側で生成し、ログ・URL・属性・キャッシュへ出さない。
const SECRET: &str = "DEMODEMODEMODEMO";
/// QR に符号化するダミーの `otpauth` URI。マークアップには出力されず、
/// `qr_code::encode` を通した行列としてのみ描画される。
const OTPAUTH_URI: &str =
    "otpauth://totp/DemoApp:user@example.com?secret=DEMODEMODEMODEMO&issuer=DemoApp&digits=6&period=30";

const FIELD_ID: &str = "blocks-auth-totp-enroll-code";
const CONTROL_ID: &str = "blocks-auth-totp-enroll-code-control";
const HELPER_TEXT_ID: &str = "blocks-auth-totp-enroll-code-helper-text";
const LABEL_ID: &str = "blocks-auth-totp-enroll-code-label";

/// QR 領域。URI は行列へ変換するだけで、`aria_label` には説明だけを渡す。
/// encode 失敗時は説明テキストに差し替える。
fn qr_area() -> Node {
    match qr_code::encode(OTPAUTH_URI, qr_code::ErrorCorrectionLevel::M) {
        Ok(matrix) => qr_code::root(
            Size::Md,
            vec![("data-blocks-auth-totp-enroll-qr", "")],
            vec![qr_code::frame(
                &matrix,
                qr_code::DEFAULT_QUIET_ZONE,
                Some("QR code for authenticator app setup"),
                vec![],
                vec![qr_code::pattern(
                    &matrix,
                    qr_code::DEFAULT_QUIET_ZONE,
                    vec![],
                )],
            )],
        ),
        Err(_) => text("QR code could not be generated."),
    }
}

/// 6 桁とも空の確認コード入力群（桁位置を読み上げる visually-hidden テキスト付き）。
fn pin_digits(props: &PinInputProps) -> Vec<Node> {
    let total = 6;
    (0..total)
        .flat_map(|index| {
            let digit_id = format!("{FIELD_ID}-digit-{}", index + 1);
            let labelledby = format!("{LABEL_ID} {digit_id}");
            let mut attrs: Vec<(&str, &str)> = vec![
                ("aria-describedby", HELPER_TEXT_ID),
                ("aria-labelledby", labelledby.as_str()),
            ];
            if index == 0 {
                attrs.push(("id", CONTROL_ID));
            }
            let input = pin_input::input(
                index,
                total,
                "",
                PinInputKind::Numeric,
                false,
                true,
                props,
                false,
                attrs,
            );
            let digit_text = visually_hidden::root(
                vec![("id", digit_id.as_str())],
                vec![text(format!("digit {} of {}", index + 1, total))],
            );
            vec![input, digit_text]
        })
        .collect()
}

/// `auth-totp-enroll` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let field_props = FieldProps {
        id: FIELD_ID,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: true,
    };
    let orientation = FieldRootProps {
        orientation: FieldOrientation::Vertical,
    };
    let pin_props = PinInputProps {
        required: true,
        ..PinInputProps::default()
    };

    let manual = div(
        vec![("class", "blocks-auth-totp-enroll-manual")],
        vec![
            text("Can't scan? Enter this setup key manually."),
            div(
                vec![("class", "blocks-auth-totp-enroll-key")],
                vec![
                    code(vec![], vec![text(SECRET)]),
                    clipboard::root(
                        SECRET,
                        false,
                        vec![("data-blocks-auth-totp-enroll-copy", "")],
                        vec![clipboard::control(
                            false,
                            vec![],
                            vec![clipboard::trigger(
                                false,
                                vec![("aria-label", "Copy setup key")],
                                vec![
                                    clipboard::indicator(false, false, vec![], vec![text("Copy")]),
                                    clipboard::indicator(true, false, vec![], vec![text("Copied")]),
                                ],
                            )],
                        )],
                    ),
                ],
            ),
        ],
    );

    let code_field = field::root(
        &orientation,
        &field_props,
        vec![],
        vec![
            field::label(&field_props, vec![], vec![text("Verification code")]),
            pin_input::root(
                Size::Lg,
                false,
                false,
                vec![("data-blocks-auth-totp-enroll-pin", "")],
                vec![pin_input::control(vec![], pin_digits(&pin_props))],
            ),
            field::helper_text(
                &field_props,
                vec![],
                vec![text(
                    "Enter the 6-digit code shown in your authenticator app.",
                )],
            ),
        ],
    );

    card::root(
        CardProps::default(),
        vec![("data-blocks-auth-totp-enroll-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("Set up two-factor authentication")]),
                    card::description(
                        vec![],
                        vec![text(format!(
                            "Scan the QR code with your authenticator app to link {ISSUER} ({ACCOUNT})."
                        ))],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![div(
                    vec![("class", "blocks-auth-totp-enroll-stack")],
                    vec![
                        div(
                            vec![("class", "blocks-auth-totp-enroll-qr")],
                            vec![qr_area()],
                        ),
                        manual,
                        code_field,
                    ],
                )],
            ),
            card::footer(
                vec![],
                vec![
                    button::button(
                        &ButtonProps::default(),
                        vec![("data-blocks-auth-totp-enroll-verify", "")],
                        vec![text("Verify and enable")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("Cancel")],
                    ),
                ],
            ),
        ],
    )
}
```

## 秘密鍵の扱い

- `otpauth` の URI は `qr_code::encode` で QR の SVG パスにだけ変換され、出力された HTML には
  残りません。QR の `aria-label` には「認証アプリ登録用の QR コード」という説明だけを入れ、
  URI は入れません。
- 秘密鍵が HTML に現れるのは、表示用の `code` 要素のテキストと、`clipboard` の
  `data-value` 属性（コピー対象を渡すための属性で、アクセシブルネームの計算対象ではない）
  の 2 箇所だけです。`value` 属性へ秘密を出す `clipboard::input` は使いません。
- 実アプリでは秘密鍵をサーバー側で生成し、ログ・URL・キャッシュへ出さないでください。
  このページには `Cache-Control: no-store` を付け、登録完了後は再表示しないでください。

## 原案差分メモ

- **`clipboard` は 1 個**: 1 root : 1 状態機械の前提に合わせ、秘密鍵のコピー用に 1 個だけ置きます。
- **確認コード入力は `auth-otp-verify` と同じ配線**: 6 桁とも空の登録前状態で固定します。
- **値はすべてダミー**: `DEMODEMODEMODEMO` と `user@example.com` の明白な架空パターンです。
- **ロゴ・商標は持ち込まない**: 認証アプリやサービスのロゴは使いません。

関連情報: [Card](../themes/card.md) / [Qr Code](../themes/qr-code.md) /
[Clipboard](../themes/clipboard.md) / [Pin Input](../themes/pin-input.md) /
[Field](../themes/field.md) / [Button](../themes/button.md)
