//! `auth-device-approve` block（イシュー #3829、親 #3824）。入力に制約のある
//! デバイス（TV・CLI 等）が表示した短いユーザーコードを、別の端末で確認して
//! 承認・拒否する画面（RFC 8628 の Device Authorization Grant 相当）の静的な合成例。
//!
//! # 使用部品
//!
//! `card`（骨格）/ `pin-input`（8 桁コード + 4-4 区切り）/ `button`（承認・拒否）/
//! `visually-hidden`（各桁の読み上げ補助）の 4 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # `<form>` を使わない・承認処理を持たない
//!
//! `<form>` を出力せず、ボタンは `button::button` の既定 `type="button"` のまま
//! 用いる。桁入力も `name` を持たず `hidden_input` も使わないため、値を運ぶ
//! 経路が存在しない。承認・拒否の送信、認可判定、トークン発行、レート制限は
//! アプリ側の責務である（`docs/policy/intentional-non-adoption.md` §3.25）。
//! ハイフン入りコードの貼り付け分配や大文字化といった実際の入力挙動は
//! 本 Block では扱わない静的見本である。
//!
//! # 区切りと横並びラッパー
//!
//! `pin_input::root` は `flex-direction: column` が既定のため、`control` /
//! `separator` / `control` を直接並べると縦積みになる。呼び出し側で
//! `blocks-auth-device-approve-groups` ラッパー（[`LAYOUT_CSS`] で横並び化）を
//! 置く。インライン `style` は CSP に依存するため使わない。部品側の CSS は変更しない。
//!
//! # 入力種別とワンタイムコード扱いにしない理由
//!
//! ユーザーコードは英数字のため `PinInputKind::Alphanumeric`（`inputmode` を出さない）。
//! SMS・メールで届くコードではないため `otp = false` とし、
//! `autocomplete="one-time-code"` による誤った自動入力誘導を避ける。照合が主目的の
//! ため `mask = false`、全桁入力済み（`verification_uri_complete` 経由で事前に
//! 埋まる想定）で固定する。`readonly` にはしない。
//!
//! # 各桁の `aria-labelledby`
//!
//! headless の各桁は固定の `aria-label="PIN digit N of M"` を持つが、これは
//! 見出しラベルを含まない。`auth_otp_verify` と同型に、全桁へ
//! `aria-labelledby="{ラベル id} {桁位置の visually-hidden id}"` を渡し、直後に
//! 桁位置を読み上げる `visually_hidden::root` を置く。4 桁ずつ 2 グループに
//! 分けても index は 0..8 の通し番号、count は 8 のまま渡す。
//!
//! # ダミー値の方針
//!
//! デバイス名は架空の汎用名、IP は RFC 5737 の TEST-NET-3（`203.0.113.0/24`）、
//! コードは RFC 8628 §6.1 推奨の母音なし文字集合から作った明白な架空パターン。
//! RFC の例示値は Demo の値に使わない。ロゴ・実在ブランドは持ち込まない。
//!
//! # CSS フックの選び方
//!
//! 部品は `class` を除去するため `data-blocks-auth-device-approve-*` 属性を、
//! 素の `div`/`dl` には `blocks-auth-device-approve-*` class を付ける。`card::footer`
//! の row 指定に勝つため、footer のセレクタは data-scope/part + 属性の
//! 詳細度 (0,3,0) で書く（`auth_oauth_consent` と同型）。
use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
                            Size::Lg,
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/auth-device-approve/",
    title: "auth-device-approve",
    category: BlockCategory::Auth,
    rust_source: "crates/docs-site/src/blocks/application/auth/auth_device_approve.rs",
    demo_class: "blocks-auth-device-approve",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Pin Input",
            path: "/themes/pin-input/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `auth_device_approve` 固有のレイアウト規則（`crate::blocks` モジュール doc
/// 「CSS の置き場」節。`super::stylesheet` から連結される）。
const LAYOUT_CSS: &str = "\
.blocks-auth-device-approve {\n  display: flex;\n  justify-content: center;\n  align-items: center;\n  min-height: 22rem;\n}\n\
[data-blocks-auth-device-approve-card] {\n  width: 100%;\n  max-width: 28rem;\n}\n\
.blocks-auth-device-approve-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-auth-device-approve-meta {\n  display: grid;\n  grid-template-columns: max-content 1fr;\n  gap: var(--fandhe-space-1) var(--fandhe-space-4);\n  margin: 0;\n}\n\
.blocks-auth-device-approve-meta dt {\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-auth-device-approve-meta dd {\n  margin: 0;\n}\n\
.blocks-auth-device-approve-groups {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  justify-content: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-auth-device-approve-help {\n  margin: 0;\n  font-size: 0.875rem;\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-scope=\"card\"][data-part=\"footer\"][data-blocks-auth-device-approve-actions] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
@media (min-width: 40rem) {\n  [data-scope=\"card\"][data-part=\"footer\"][data-blocks-auth-device-approve-actions] {\n    flex-direction: row;\n    justify-content: flex-end;\n  }\n}\n\
@media (max-width: 39.99rem) {\n  [data-blocks-auth-device-approve-card] {\n    max-width: 100%;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// 使用部品の anatomy が出力されることを固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for s in [
            "data-scope=\"card\"",
            "data-scope=\"pin-input\"",
            "data-scope=\"button\"",
            "data-scope=\"visually-hidden\"",
        ] {
            assert!(html.contains(s), "demo should contain {s}");
        }
    }

    /// 送信経路・ワンタイムコード誘導・外部参照を出さないことを固定する。
    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        for f in [
            "<form",
            "type=\"submit\"",
            " name=\"",
            "href=\"#\"",
            "<script",
            "data:",
            "blob:",
            "one-time-code",
            "inputmode=",
            "style=\"",
        ] {
            assert!(!html.contains(f), "demo must not contain {f}");
        }
    }

    /// 全ボタンが `type="button"` であることを固定する。
    #[test]
    fn all_buttons_are_type_button() {
        let html = demo_html();
        let buttons = html.matches("<button").count();
        assert!(buttons >= 2);
        assert_eq!(html.matches("type=\"button\"").count(), buttons);
    }

    /// 8 桁の英数字と 1 個の区切り、2 グループで構成されることを固定する。
    #[test]
    fn code_is_eight_alphanumeric_chars_with_one_separator() {
        assert_eq!(CODE.len(), 8);
        assert!(CODE
            .iter()
            .all(|c| c.len() == 1 && c.chars().all(|ch| ch.is_ascii_alphanumeric())));
        let html = demo_html();
        assert_eq!(html.matches("data-part=\"separator\"").count(), 1);
        assert_eq!(html.matches("data-part=\"control\"").count(), 2);
        assert_eq!(html.matches("data-part=\"input\"").count(), 8);
    }

    /// 全桁が見出しラベルと桁位置で名前付けされ、参照先が存在することを固定する。
    #[test]
    fn every_digit_is_labelled() {
        let html = demo_html();
        assert_eq!(html.matches("aria-labelledby=").count(), 8);
        for n in 1..=8 {
            assert!(html.contains(&format!("id=\"blocks-auth-device-approve-digit-{n}\"")));
        }
        assert!(html.contains(&format!("id=\"{LABEL_ID}\"")));
    }

    /// 外部参照を作る CSS 構文を含まないことを固定する。
    #[test]
    fn layout_css_is_safe() {
        for f in ["url(", "@import", "expression", "<"] {
            assert!(!LAYOUT_CSS.contains(f));
        }
    }

    /// ダミー値が RFC 5737 の TEST-NET と架空デバイス名であることを固定する。
    #[test]
    fn dummy_values_are_not_real() {
        let html = demo_html();
        assert!(html.contains("203.0.113."));
        assert!(html.contains("Demo Media Player"));
    }
}
