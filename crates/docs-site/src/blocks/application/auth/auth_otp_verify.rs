//! `auth-otp-verify` block（イシュー #2964、親 #2951「Blocks
//! アプリケーション B」配下。Auth カテゴリ 5 件目、`login-01`/`login-04`/
//! `signup-01`/`signup-05` に続く）。
//!
//! # 使用部品
//!
//! `pin-input`（6 桁コード入力）+ `field`（label/helper_text 配線）/
//! `button`（Solid 確認・Link 再送信）/ `link`（戻る導線）/ `heading`
//! （見出し）/ `text`（説明文）の 6 部品を合成する（[`BLOCK`] の `parts`
//! に一致させる契約、`blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 参照・差分
//!
//! 主参照 R0144（集約元も同一 1 件のため差分なし。`_/blocks-intake/` は
//! 本 worktree に存在しないため、レイアウトはイシュー本文の仕様記述のみで
//! 確定した。`page-heading-avatar`〔#2931〕/ `list-title-meta`〔#2925〕と
//! 同型の扱い）。
//!
//! # `<form>` を使わない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり `<form>` を出力しない。
//! 確認ボタンは `button::button` の既定 `type="button"` のまま用い、
//! 桁入力欄も `name` を持たない（[`fandhe_frontend_pre_styled_ui::pin_input::hidden_input`]
//! を使わず送信値を運ぶ相手を作らない）。値は一切送信されず、認証・検証
//! ロジックを持たない静的な合成例である
//! （`docs/policy/intentional-non-adoption.md` §3.25 の責務境界）。
//!
//! # 桁入力の部分入力状態（固定表示）
//!
//! 6 桁のうち先頭 3 桁のみ値を持つ部分入力状態で固定する（`data-filled`
//! の見え方を静的に示す）。実際の値は意味を持たないダミー。
//!
//! # `field` との配線
//!
//! `field::label` の `for` は [`fandhe_frontend_headless_ui::field::FieldProps`]
//! の `id` から `"{id}-control"` を自動導出するため、先頭桁の `input` にのみ
//! 同じ値を `id` 属性として明示的に付与し解決させる。`field::helper_text`
//! も同様に `"{id}-helper-text"` を自動導出するため、全桁の `input` に
//! `aria-describedby` として明示的に同じ値を付与する（`field::root` の
//! 子として桁入力を直接並べるため、`field::label`/`field::helper_text`
//! 自身の自動配線対象〔input/textarea/select〕には該当せず、この手動
//! 配線が唯一の接続経路になる）。
//!
//! # 各桁の `aria-labelledby`（Codex 指摘 #3416 対応）
//!
//! [`fandhe_frontend_headless_ui::pin_input::input`] は `aria-label="PIN
//! digit N of M"` を固定で付与するが、これは桁位置のみを表し
//! `field::label` の「Verification code」を含まない。`field::label` の
//! `for`（`"{FIELD_ID}-control"`）は先頭桁のみを指すため、スクリーン
//! リーダーは残り 5 桁を「Verification code」の一部と認識できない。
//! 本 block は全桁の `input` に `aria-labelledby="{field label id}
//! {桁位置の visually-hidden id}"` を追加で渡し（headless 側の固定
//! `aria-label` はそのまま残す。WAI-ARIA では両方存在する場合
//! `aria-labelledby` が優先されるため、値の重複や矛盾は生じない）、
//! [`fandhe_frontend_pre_styled_ui::visually_hidden::root`] で桁ごとの
//! 「digit N of M」テキストを持つ非表示要素を各 `input` の直後に置く
//! ことで、各桁で「Verification code」+「digit N of M」の両方が
//! アクセシブルネームとして読み上げられるようにする。`field::label` の
//! `id` は `FieldProps` の既定導出（`"{FIELD_ID}-label"`、`FieldIds` を
//! 渡していないため上書きされない）にそのまま追随する。
//!
//! # 「戻る」導線は `link`、「再送信」は `ButtonVariant::Link`
//!
//! 「戻る」は実在する兄弟ページ `../login-01/` への遷移のため `link::root`
//! を使う（`href="#"` を使わない方針、`newsletter_stacked.rs` と同型）。
//! 「再送信」はページ遷移を伴わない状態変化アクションのため
//! `login_01`/`signup_05` と同判断で `ButtonVariant::Link` を使う。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `field::root`/`pin_input::root`/`button::button`/`heading::heading`/
//! `text::text`/`link::root` は `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有スタイルは
//! `data-blocks-auth-otp-verify-*` 属性で渡し、素の `div` には
//! `blocks-auth-otp-verify-*` class を使う（`signup_05` と同型の混在規則、
//! `crate::blocks` モジュール doc「CSS フックが `class` と `[data-*]` で
//! 混在する理由」節参照）。`pin-input` の桁枠の中央寄せは
//! `pin_input::control` 自体の既定 CSS を変えず、block 側スコープ限定
//! セレクタで対応する（他クレート契約への影響回避）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::input::{FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::pin_input::{self, PinInputKind, PinInputProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// `field` の `id` 派生元（`"{FIELD_ID}-control"`/`"{FIELD_ID}-helper-text"`
/// を [`field::label`]/[`field::helper_text`] が自動導出し、桁入力側は
/// リテラルで同じ値を明示配線する、モジュール doc「`field` との配線」節参照）。
const FIELD_ID: &str = "blocks-auth-otp-verify-code";
const CONTROL_ID: &str = "blocks-auth-otp-verify-code-control";
const HELPER_TEXT_ID: &str = "blocks-auth-otp-verify-code-helper-text";
/// `field::label` の既定 `id` 導出（`"{FIELD_ID}-label"`）と同じ値
/// （モジュール doc「各桁の `aria-labelledby`」節参照）。
const LABEL_ID: &str = "blocks-auth-otp-verify-code-label";

/// 部分入力状態（先頭 3 桁のみ値を持つ）で固定した 6 桁の pin-input 入力群。
/// 各桁 `input` の直後に、桁位置を読み上げる visually-hidden テキストを
/// 並べて出力する（`aria-labelledby` の参照先、モジュール doc 参照）。
fn pin_digits(props: &PinInputProps) -> Vec<Node> {
    let values = ["4", "2", "7", "", "", ""];
    values
        .into_iter()
        .enumerate()
        .flat_map(|(index, value)| {
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
                values.len(),
                value,
                PinInputKind::Numeric,
                false,
                true,
                props,
                false,
                attrs,
            );
            let digit_text = visually_hidden::root(
                vec![("id", digit_id.as_str())],
                vec![text(format!("digit {} of {}", index + 1, values.len()))],
            );
            vec![input, digit_text]
        })
        .collect()
}

/// `auth-otp-verify` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
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
    let link_button = ButtonProps {
        variant: ButtonVariant::Link,
        ..ButtonProps::default()
    };

    let header = div(
        vec![("class", "blocks-auth-otp-verify-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("Check your inbox")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-auth-otp-verify-lead", "")],
                vec![text("Enter the 6-digit code we sent to m•••@example.com.")],
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
                vec![("data-blocks-auth-otp-verify-pin", "")],
                vec![pin_input::control(vec![], pin_digits(&pin_props))],
            ),
            field::helper_text(
                &field_props,
                vec![],
                vec![text("The code expires in 10 minutes.")],
            ),
        ],
    );

    let submit = button::button(
        &ButtonProps::default(),
        vec![("data-blocks-auth-otp-verify-submit", "")],
        vec![text("Verify code")],
    );

    let footer = div(
        vec![("class", "blocks-auth-otp-verify-footer")],
        vec![
            div(
                vec![("class", "blocks-auth-otp-verify-resend")],
                vec![
                    text("Didn't get the code? "),
                    button::button(&link_button, vec![], vec![text("Resend code")]),
                ],
            ),
            link::root(
                "../login-01/",
                &LinkProps::default(),
                vec![("data-blocks-auth-otp-verify-back", "")],
                vec![text("Back to sign in")],
            ),
        ],
    );

    div(
        vec![("data-blocks-auth-otp-verify-stack", "")],
        vec![header, code_field, submit, footer],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/auth-otp-verify/",
    title: "auth-otp-verify",
    category: BlockCategory::Auth,
    rust_source: "crates/docs-site/src/blocks/application/auth/auth_otp_verify.rs",
    demo_class: "blocks-auth-otp-verify",
    parts: &[
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

/// `auth_otp_verify` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節。`signup_05` と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。
///
/// `[data-scope="pin-input"][data-part="control"]` の `justify-content`
/// 上書きは桁枠を中央寄せするためのもので、`pin_input::control` 自体の
/// 既定 CSS（`crates/pre-styled-ui/src/pin_input.rs`）は変更せず、本 block
/// のスコープ限定セレクタでのみ適用する。
const LAYOUT_CSS: &str = "\
.blocks-auth-otp-verify {\n  display: flex;\n  justify-content: center;\n  align-items: center;\n  min-height: 24rem;\n}\n\
[data-blocks-auth-otp-verify-stack] {\n  display: flex;\n  flex-direction: column;\n  gap: 1.5rem;\n  width: 100%;\n  max-width: 22rem;\n  margin: 0 auto;\n}\n\
.blocks-auth-otp-verify-header {\n  display: flex;\n  flex-direction: column;\n  gap: 0.5rem;\n  text-align: center;\n}\n\
[data-blocks-auth-otp-verify-pin] [data-scope=\"pin-input\"][data-part=\"control\"] {\n  justify-content: center;\n}\n\
[data-blocks-auth-otp-verify-submit] {\n  width: 100%;\n}\n\
.blocks-auth-otp-verify-footer {\n  display: flex;\n  flex-direction: column;\n  gap: 0.5rem;\n  align-items: center;\n  font-size: 0.875rem;\n  color: var(--fandhe-color-fg-muted);\n}\n\
@media (max-width: 39.99rem) {\n  [data-blocks-auth-otp-verify-stack] {\n    max-width: 100%;\n  }\n}\n";
