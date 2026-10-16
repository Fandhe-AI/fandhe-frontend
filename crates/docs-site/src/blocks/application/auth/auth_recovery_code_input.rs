//! `auth-recovery-code-input` block（イシュー #3828、親 #3824）。2 要素認証の
//! 復旧コードを単一の入力欄で受け取る画面の静的な合成例。
//!
//! # 使用部品
//!
//! `field`（label/helper_text 配線）+ `input`（単一の入力欄）/ `button`
//! （Solid 確認）/ `link`（別手段・戻る導線）/ `heading`（見出し）/ `text`
//! （説明文）の 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 単一の入力欄を使う理由
//!
//! 桁ごとの入力欄（`pin_input`）は貼り付けやフォーカス移動といった入力挙動を
//! クライアント側で配線する前提の部品であり、JS なしの認証画面では単一の
//! 入力欄が最も確実に動く。復旧コードは桁数・区切りが可変でもあるため
//! `field` + `input` で組む。
//!
//! # `<form>` を使わない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり `<form>` を出力しない。
//! 確認ボタンは `button::button` の既定 `type="button"` のままで、入力欄は
//! `name` も `value` も持たず送信経路と事前入力値を作らない。認証・検証
//! ロジックを持たない静的な合成例である
//! （`docs/policy/intentional-non-adoption.md` §3.25 の責務境界）。
//!
//! # `autocomplete` 等の属性値（固定リテラル）
//!
//! 型付き `autocomplete`（`input_with_autocomplete`）の語彙は
//! username/email/current-password/new-password のみで `off` を表せないため、
//! 既存 block と同じく `extra_attrs` へ `&'static str` のリテラルを渡す
//! （任意文字列の経路は作らない）。`autocomplete="off"` は、1 回しか使えない
//! 復旧コードがフォーム履歴へ残ることを避け、`one-time-code` による SMS・
//! メール由来コードの自動入力候補が別の値を誘導することも避けるため。
//! `autocapitalize="none"`/`spellcheck="false"` はコードの書き換えと下線を
//! 防ぐ。英数字とハイフンの文字列なので `inputmode="numeric"` は付けない。
//!
//! # 「別手段」「戻る」導線は `link`
//!
//! どちらも実在する兄弟ページ（`../auth-otp-verify/`・`../login-01/`）への
//! 遷移のため `link::root` を使う（`href="#"` を使わない方針）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `field::root`/`input::input`/`button::button`/`heading::heading`/
//! `text::text`/`link::root` は呼び出し側 `attrs` の `class` を除去する契約を
//! 持つため、Demo 固有スタイルは `data-blocks-auth-recovery-code-input-*`
//! 属性で渡し、素の `div` には `blocks-auth-recovery-code-input-*` class を
//! 使う（`crate::blocks` モジュール doc「CSS フックが `class` と `[data-*]`
//! で混在する理由」節参照）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// `field` の `id` 派生元（`field::label` の `for` と `field::helper_text` の
/// `id` は `FieldProps` から自動導出され、`input::input` も同じ値で配線される）。
const FIELD_ID: &str = "blocks-auth-recovery-code-input-code";

/// `auth-recovery-code-input` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
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

    let header = div(
        vec![("class", "blocks-auth-recovery-code-input-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("Use a recovery code")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-auth-recovery-code-input-lead", "")],
                vec![text(
                    "Enter one of the recovery codes you saved when you set up two-step verification.",
                )],
            ),
        ],
    );

    let code_field = field::root(
        &orientation,
        &field_props,
        vec![],
        vec![
            field::label(&field_props, vec![], vec![text("Recovery code")]),
            input::input(
                &InputProps::default(),
                &field_props,
                vec![
                    ("type", "text"),
                    ("autocomplete", "off"),
                    ("autocapitalize", "none"),
                    ("spellcheck", "false"),
                    ("placeholder", "xxxxx-xxxxx"),
                ],
            ),
            field::helper_text(
                &field_props,
                vec![],
                vec![text("Each recovery code can be used only once.")],
            ),
        ],
    );

    let submit = button::button(
        &ButtonProps::default(),
        vec![("data-blocks-auth-recovery-code-input-submit", "")],
        vec![text("Verify")],
    );

    let footer = div(
        vec![("class", "blocks-auth-recovery-code-input-footer")],
        vec![
            link::root(
                "../auth-otp-verify/",
                &LinkProps::default(),
                vec![("data-blocks-auth-recovery-code-input-alt", "")],
                vec![text("Use a verification code instead")],
            ),
            link::root(
                "../login-01/",
                &LinkProps::default(),
                vec![("data-blocks-auth-recovery-code-input-back", "")],
                vec![text("Back to sign in")],
            ),
        ],
    );

    div(
        vec![("data-blocks-auth-recovery-code-input-stack", "")],
        vec![header, code_field, submit, footer],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/auth-recovery-code-input/",
    title: "auth-recovery-code-input",
    category: BlockCategory::Auth,
    rust_source: "crates/docs-site/src/blocks/application/auth/auth_recovery_code_input.rs",
    demo_class: "blocks-auth-recovery-code-input",
    parts: &[
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `auth_recovery_code_input` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節。`auth_otp_verify` と同型）。
const LAYOUT_CSS: &str = "\
.blocks-auth-recovery-code-input {\n  display: flex;\n  justify-content: center;\n  align-items: center;\n  min-height: 24rem;\n}\n\
[data-blocks-auth-recovery-code-input-stack] {\n  display: flex;\n  flex-direction: column;\n  gap: 1.5rem;\n  width: 100%;\n  max-width: 22rem;\n  margin: 0 auto;\n}\n\
.blocks-auth-recovery-code-input-header {\n  display: flex;\n  flex-direction: column;\n  gap: 0.5rem;\n  text-align: center;\n}\n\
[data-blocks-auth-recovery-code-input-submit] {\n  width: 100%;\n}\n\
.blocks-auth-recovery-code-input-footer {\n  display: flex;\n  flex-direction: column;\n  gap: 0.5rem;\n  align-items: center;\n  font-size: 0.875rem;\n  color: var(--fandhe-color-fg-muted);\n}\n\
@media (max-width: 39.99rem) {\n  [data-blocks-auth-recovery-code-input-stack] {\n    max-width: 100%;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// 使用部品の anatomy と label/input の配線が出力されることを固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"field\"",
            "data-scope=\"button\"",
            "<label",
            "for=\"blocks-auth-recovery-code-input-code-control\"",
            "blocks-auth-recovery-code-input-code-helper-text",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    /// 固定リテラルの入力属性がちょうど 1 回ずつ出ることを固定する。
    #[test]
    fn outputs_fixed_input_attributes() {
        let html = demo_html();
        assert_eq!(html.matches("autocomplete=\"off\"").count(), 1);
        assert_eq!(html.matches("autocomplete=").count(), 1);
        for expected in [
            "autocapitalize=\"none\"",
            "spellcheck=\"false\"",
            "type=\"text\"",
        ] {
            assert!(html.contains(expected), "demo should contain {expected}");
        }
        assert!(!html.contains("inputmode="));
    }

    /// `<form>`・`name`・事前入力 `value`・`data:` URI・ダミーリンクを出さないことを固定する。
    #[test]
    fn no_form_no_name_no_prefilled_value() {
        let html = demo_html();
        for forbidden in [
            "<form",
            " name=\"",
            " value=\"",
            "src=\"data:",
            "href=\"#\"",
        ] {
            assert!(
                !html.contains(forbidden),
                "demo must not contain {forbidden}"
            );
        }
    }

    /// 全ボタンが `type="button"` であることを固定する。
    #[test]
    fn all_buttons_are_type_button() {
        let html = demo_html();
        let buttons = html.matches("<button").count();
        assert_eq!(buttons, 1);
        assert_eq!(html.matches("type=\"button\"").count(), buttons);
    }

    /// 外部参照を作る CSS 構文を含まないことを固定する。
    #[test]
    fn layout_css_is_safe() {
        for forbidden in ["url(", "@import", "expression"] {
            assert!(!LAYOUT_CSS.contains(forbidden));
        }
    }
}
