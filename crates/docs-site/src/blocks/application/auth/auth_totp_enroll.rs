//! `auth-totp-enroll` block（イシュー #3830、親 #3824）。2 要素認証（TOTP）の
//! 登録画面を示す静的な合成例。認証アプリが読み取る QR コード、手入力用の
//! 秘密鍵（コピー可）、数字 6 桁の確認コード入力を並べる。
//!
//! # 使用部品
//!
//! `card`（骨格）/ `qr_code`（`otpauth` URI の SVG 表示）/ `clipboard`（秘密鍵の
//! コピー）/ `pin_input`（確認コード 6 桁）/ `field`（ラベル・補助文の配線）/
//! `button`（確認・キャンセル）の 6 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # URI・秘密鍵・発行者名はすべてダミー
//!
//! `ISSUER`/`ACCOUNT`/`SECRET`/`OTPAUTH_URI` は表示用の架空値であり、実在の鍵・
//! サービスではない（`example.com` は RFC 2606 の予約ドメイン）。実アプリでは
//! 秘密鍵をサーバー側で生成し、ログ・URL・属性・キャッシュへ出さない。
//!
//! # `otpauth` URI の出力経路
//!
//! URI は `qr_code::encode` で行列へ変換され、`pattern` の `d` 属性（数字と
//! パスコマンド文字だけ）に変わる 1 経路のみで出力される。レンダリング結果の
//! HTML に `otpauth` という文字列は現れない。QR の `aria_label` には URI を
//! 含めず、説明文だけを渡す。encode 失敗時は説明テキストへ差し替える
//! （`unwrap` は使わない）。
//!
//! # 秘密鍵が現れる箇所（`data-value` の例外）
//!
//! 秘密鍵がレンダリング結果に現れるのは (a) `code` 要素の表示テキスト、
//! (b) `clipboard::root` の `data-value` 属性の 2 箇所だけ。`data-value` は
//! クライアント側の配線へコピー対象を渡す部品の契約であり、アクセシブル
//! ネームの計算対象ではない。`aria-label` などの名前・説明に使う属性には
//! 秘密鍵も `otpauth` も入れない（テストで固定）。`value` 属性へ秘密を出す
//! `clipboard::input` は使わない。
//!
//! # `<form>` を使わない・送信処理を持たない
//!
//! `<form>` を出力せず、ボタンは既定 `type="button"`、桁入力も `name` を持たない。
//! 検証・送信は行わない静的な見本である
//! （`docs/policy/intentional-non-adoption.md` §3.25 の責務境界）。
//!
//! # `pin_input` の配線
//!
//! `auth_otp_verify` と同じ形。先頭桁に `field::label` の `for` 解決先 id、全桁に
//! `aria-describedby`、全桁に `aria-labelledby`（ラベル id + 桁位置の
//! visually-hidden id）を付ける。6 桁とも空の登録前状態で固定する。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! 部品は呼び出し側の `class` を除去するため、部品へ渡すフックは
//! `data-blocks-auth-totp-enroll-*` 属性、素の `div` には
//! `blocks-auth-totp-enroll-*` class を使う。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/auth-totp-enroll/",
    title: "auth-totp-enroll",
    category: BlockCategory::Auth,
    rust_source: "crates/docs-site/src/blocks/application/auth/auth_totp_enroll.rs",
    demo_class: "blocks-auth-totp-enroll",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Qr Code",
            path: "/themes/qr-code/",
        },
        Part {
            label: "Clipboard",
            path: "/themes/clipboard/",
        },
        Part {
            label: "Pin Input",
            path: "/themes/pin-input/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `auth_totp_enroll` 固有のレイアウト規則（`crate::blocks` モジュール doc
/// 「CSS の置き場」節。`super::stylesheet` から連結される）。
const LAYOUT_CSS: &str = "\
.blocks-auth-totp-enroll {\n  display: flex;\n  justify-content: center;\n  align-items: center;\n  min-height: 24rem;\n}\n\
[data-blocks-auth-totp-enroll-card] {\n  width: 100%;\n  max-width: 28rem;\n}\n\
.blocks-auth-totp-enroll-stack {\n  display: flex;\n  flex-direction: column;\n  gap: 1.5rem;\n}\n\
.blocks-auth-totp-enroll-qr {\n  display: flex;\n  justify-content: center;\n}\n\
.blocks-auth-totp-enroll-manual {\n  display: flex;\n  flex-direction: column;\n  gap: 0.5rem;\n  font-size: 0.875rem;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-auth-totp-enroll-key {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: 0.5rem;\n}\n\
.blocks-auth-totp-enroll-key code {\n  font-family: var(--fandhe-font-font-mono);\n  word-break: break-all;\n}\n\
[data-blocks-auth-totp-enroll-pin] [data-scope=\"pin-input\"][data-part=\"control\"] {\n  justify-content: center;\n  flex-wrap: wrap;\n}\n\
@media (max-width: 39.99rem) {\n  [data-blocks-auth-totp-enroll-card] {\n    max-width: 100%;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// 指定属性の値を全件抜き出す。
    fn attr_values(html: &str, name: &str) -> Vec<String> {
        let needle = format!(" {name}=\"");
        let mut out = Vec::new();
        let mut rest = html;
        while let Some(i) = rest.find(&needle) {
            let after = &rest[i + needle.len()..];
            let end = after.find('"').unwrap_or(after.len());
            out.push(after[..end].to_string());
            rest = &after[end..];
        }
        out
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for s in [
            "data-scope=\"card\"",
            "data-scope=\"qr-code\"",
            "data-scope=\"clipboard\"",
            "data-scope=\"pin-input\"",
            "data-scope=\"field\"",
            "data-scope=\"button\"",
            "role=\"img\"",
            "aria-label=\"QR code for authenticator app setup\"",
        ] {
            assert!(html.contains(s), "demo should contain {s}");
        }
    }

    /// URI は SVG パスへ変換されるだけで、マークアップには現れない。
    #[test]
    fn otpauth_uri_never_reaches_markup() {
        let html = demo_html();
        for s in ["otpauth", "secret=", "issuer="] {
            assert!(!html.contains(s), "markup must not contain {s}");
        }
    }

    /// 秘密鍵は表示テキストと clipboard の `data-value` の 2 箇所だけ。
    #[test]
    fn secret_appears_only_in_visible_text_and_clipboard_value() {
        let html = demo_html();
        assert_eq!(html.matches(SECRET).count(), 2);
        assert_eq!(html.matches(&format!("data-value=\"{SECRET}\"")).count(), 1);
        assert_eq!(html.matches(&format!(">{SECRET}<")).count(), 1);
    }

    #[test]
    fn accessible_name_attributes_never_contain_secrets() {
        let html = demo_html();
        for name in [
            "aria-label",
            "aria-description",
            "aria-roledescription",
            "title",
            "alt",
            "aria-valuetext",
        ] {
            for v in attr_values(&html, name) {
                for banned in [SECRET, "otpauth", ACCOUNT] {
                    assert!(!v.contains(banned), "{name}={v} must not contain {banned}");
                }
            }
        }
    }

    #[test]
    fn dummy_values_are_consistent() {
        assert!(OTPAUTH_URI.starts_with("otpauth://totp/"));
        assert!(OTPAUTH_URI.contains(&format!("secret={SECRET}")));
        assert!(OTPAUTH_URI.contains(&format!("issuer={ISSUER}")));
        assert!(OTPAUTH_URI.contains(ACCOUNT));
        assert!(SECRET.starts_with("DEMO"));
        assert!(SECRET.chars().all(|c| matches!(c, 'A'..='Z' | '2'..='7')));
    }

    /// fallback 経路に入らない（QR が実際に描画される）ことを固定する。
    #[test]
    fn qr_encode_succeeds_for_dummy_uri() {
        assert!(qr_code::encode(OTPAUTH_URI, qr_code::ErrorCorrectionLevel::M).is_ok());
        assert!(!demo_html().contains("could not be generated"));
    }

    #[test]
    fn no_form_or_dead_links() {
        let html = demo_html();
        for f in [
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "<script",
            " name=\"",
            "data:",
            "blob:",
        ] {
            assert!(!html.contains(f), "demo must not contain {f}");
        }
    }

    #[test]
    fn all_buttons_are_type_button() {
        let html = demo_html();
        let buttons = html.matches("<button").count();
        assert!(buttons >= 3);
        assert_eq!(html.matches("type=\"button\"").count(), buttons);
    }

    #[test]
    fn pin_digits_are_wired() {
        let html = demo_html();
        assert_eq!(html.matches("aria-labelledby=").count(), 6);
        assert_eq!(html.matches("aria-describedby=").count(), 6);
        assert_eq!(
            html.matches("id=\"blocks-auth-totp-enroll-code-control\"")
                .count(),
            1
        );
        let ids = attr_values(&html, "id");
        let unique: std::collections::BTreeSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len(), "id must be unique");
    }

    #[test]
    fn layout_css_is_safe() {
        for f in ["url(", "@import", "expression", "<"] {
            assert!(!LAYOUT_CSS.contains(f));
        }
    }
}
