//! `banner-email-signup` block（イシュー #2741。親トラッキング #2738
//! 「Phase 1: Blocks マーケティング A」配下、Marketing / Banner カテゴリ
//! 最初の block。対応表 ID R0012 の 1 件を参照する）。
//!
//! # 使用部品
//!
//! `callout`（帯の外枠）+ `heading`/`text`（タイトル・説明）+
//! `field`/`input`（メールアドレス入力）+ `button`（送信ボタン + 閉じる
//! ボタン）+ `visually_hidden`（可視ラベルを出さない代わりの
//! `<label for>` ラップ）を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約）。新規 UI 部品は作らない。
//!
//! # `<form>` を使わない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。登録ボタン・閉じるボタンはいずれも `type="button"`
//! （`button::button`/`button::close_button` の固定契約）のまま送信先・
//! 削除処理を持たず、実際のバリデーション・送信・閉じる動作は利用者自身の
//! Rust/JS コードで実装する（`docs/policy/intentional-non-adoption.md`
//! §3.25）。
//!
//! # 閉じるボタンは 1 DOM のみ（参照元 R0012 との差分）
//!
//! 参照元 R0012 は幅で出し分ける 2 DOM 構成だが、本実装は
//! `grid-template-areas` の切替のみで閉じるボタンを 1 DOM のまま
//! 再配置する（アクセシブル名の重複・フォーカス停止点の重複を避けるため。
//! 詳細差分は `site/blocks/banner-email-signup.md` の「差分メモ」節）。
//!
//! # 可視ラベルの代わりに `visually_hidden` + `<label for>`
//!
//! 帯レイアウドでは可視ラベルを出さず、[`visually_hidden::root`] で包んだ
//! [`field::label`] により入力欄のアクセシブル名を `<label for>` の関連付け
//! で確保する（`aria-label` 直書きより関連付けを優先する既存部品の作法に
//! 従う）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `callout::root`/`field::root`/`input::input`/`button::button`/
//! `button::close_button` は `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、Demo 固有スタイルは
//! `data-blocks-banner-email-signup-*` 属性で渡し、[`LAYOUT_CSS`] 側も
//! 同じ属性セレクタで対応する。素の `div` には `class` がそのまま効く。
//!
//! # レイアウト（`>= 48rem` は 3 カラム帯、`< 48rem` は 2 段）
//!
//! `>= 48rem`（`min-width` ではなく `LAYOUT_CSS` は `max-width: 47.99rem`
//! の `@media` で縮小側のみ上書きする、`footer_newsletter`/`login_04` 等の
//! 既存慣行）では `copy`（タイトル+説明）/`signup`（入力+ボタン）/`close`
//! の 3 領域を横並びの `grid-template-areas` で配置する。`< 48rem` では
//! `copy` の下に `signup` を回し、`close` は右上へ残す
//! （`site/blocks/banner-email-signup.md` の「差分メモ」節で言及するとおり、
//! 判定基準は Demo 枠幅ではなくビューポート幅である）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::callout::{self, CalloutProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// タイトル・説明（`copy` 領域）。
fn copy() -> Node {
    div(
        vec![("class", "blocks-banner-email-signup-copy")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Sm,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-banner-email-signup-title", "")],
                vec![text("Get release notes in your inbox")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-banner-email-signup-description", "")],
                vec![text("New blocks and components, once a month. No spam.")],
            ),
        ],
    )
}

/// メールアドレス入力 + 送信ボタン（`signup` 領域）。可視ラベルは出さず
/// `visually_hidden::root` で包んだ `field::label` が `<label for>` の
/// 関連付けを担う（モジュール doc「可視ラベルの代わりに」節参照）。
fn signup() -> Node {
    const EMAIL_FIELD_ID: &str = "blocks-banner-email-signup-email";
    let email_field = FieldProps {
        id: EMAIL_FIELD_ID,
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

    div(
        vec![("class", "blocks-banner-email-signup-signup")],
        vec![
            field::root(
                &orientation,
                &email_field,
                vec![("data-blocks-banner-email-signup-field", "")],
                vec![
                    visually_hidden::root(
                        vec![],
                        vec![field::label(
                            &email_field,
                            vec![],
                            vec![text("Email address")],
                        )],
                    ),
                    input::input(
                        &InputProps::default(),
                        &email_field,
                        vec![
                            ("type", "email"),
                            ("autocomplete", "email"),
                            ("placeholder", "you@example.com"),
                        ],
                    ),
                ],
            ),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-banner-email-signup-submit", "")],
                vec![text("Notify me")],
            ),
        ],
    )
}

/// `banner-email-signup` の Demo 本体（全幅の帯。`copy`/`signup`/`close`
/// の 3 領域を [`LAYOUT_CSS`] の `grid-template-areas` で配置する）。
pub fn demo() -> Node {
    callout::root(
        &CalloutProps::default(),
        vec![("data-blocks-banner-email-signup-root", "")],
        vec![div(
            vec![("class", "blocks-banner-email-signup-bar")],
            vec![
                copy(),
                signup(),
                button::close_button(
                    &ButtonProps {
                        variant: ButtonVariant::Ghost,
                        size: Size::Sm,
                        ..ButtonProps::default()
                    },
                    "Dismiss banner",
                    vec![("data-blocks-banner-email-signup-close", "")],
                ),
            ],
        )],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/banner-email-signup/",
    title: "banner-email-signup",
    category: BlockCategory::Banner,
    rust_source: "crates/docs-site/src/blocks/marketing/banner/banner_email_signup.rs",
    demo_class: "blocks-banner-email-signup",
    parts: &[
        Part {
            label: "Callout",
            path: "/themes/callout/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
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
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `banner_email_signup` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。
///
/// callout root 自体のプロパティ（`display`/`padding` 等、recipe base が
/// `[data-scope="callout"][data-part="root"]` の詳細度 0,2,0 で宣言済み）は
/// 上書きしない。帯としてのレイアウトはすべて内側の
/// `.blocks-banner-email-signup-bar`（素の `div`、`class` がそのまま効く）
/// に寄せる（モジュール doc「CSS フックの選び方」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-banner-email-signup-bar {\n  flex: 1;\n  min-width: 0;\n  display: grid;\n  grid-template-columns: 1fr auto auto;\n  grid-template-areas: \"copy signup close\";\n  align-items: center;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-banner-email-signup-copy {\n  grid-area: copy;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-banner-email-signup-signup {\n  grid-area: signup;\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-banner-email-signup-field] {\n  min-width: 14rem;\n}\n\
[data-blocks-banner-email-signup-close] {\n  grid-area: close;\n}\n\
@media (max-width: 47.99rem) {\n  .blocks-banner-email-signup-bar {\n    grid-template-columns: 1fr auto;\n    grid-template-areas: \"copy close\" \"signup signup\";\n    align-items: start;\n  }\n  .blocks-banner-email-signup-signup {\n    flex-direction: column;\n    align-items: stretch;\n  }\n  [data-blocks-banner-email-signup-field] {\n    min-width: 0;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// `<form>` を出力しない（`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_does_not_output_a_form_element() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
    }

    /// 登録・閉じるボタンはいずれも `type="button"`（ちょうど 2 個）。
    #[test]
    fn demo_has_exactly_two_type_button_buttons() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"type="button""#).count(), 2);
    }

    /// 閉じるボタンの `aria-label` はちょうど 1 個（重複なし）。
    #[test]
    fn demo_has_exactly_one_aria_label() {
        let html = render(&demo());
        assert_eq!(html.matches("aria-label=").count(), 1);
    }

    /// メール入力は `type="email"` を持ち、`<h3>` を含み `<h2>` は含まない。
    #[test]
    fn demo_has_email_input_and_h3_heading_only() {
        let html = render(&demo());
        assert!(html.contains(r#"type="email""#));
        assert!(html.contains("<h3"));
        assert!(!html.contains("<h2"));
    }

    /// visually-hidden な `<label for>` と同じ `id` を持つ input が存在する
    /// （アクセシブル名の関連付けを固定）。
    #[test]
    fn demo_label_for_matches_input_id() {
        let html = render(&demo());
        assert!(html.contains(r#"data-scope="visually-hidden""#));
        assert!(html.contains(r#"for="blocks-banner-email-signup-email-control""#));
        assert!(html.contains(r#"id="blocks-banner-email-signup-email-control""#));
    }

    /// CSS フック属性がすべて出力 HTML に現れる。
    #[test]
    fn demo_css_hooks_present_in_html() {
        let html = render(&demo());
        for hook in [
            "data-blocks-banner-email-signup-root",
            "data-blocks-banner-email-signup-field",
            "data-blocks-banner-email-signup-submit",
            "data-blocks-banner-email-signup-close",
        ] {
            assert!(html.contains(hook), "html に {hook} が無い");
        }
    }

    /// レイアウト（配置・レスポンシブ切替）を担う CSS フックは
    /// `LAYOUT_CSS` 側にも同じ属性セレクタで現れる（`-root`/`-submit` は
    /// レイアウト上書きを持たないため対象外。callout recipe base の
    /// 詳細度負けを避ける判断、モジュール doc「CSS フックの選び方」節）。
    #[test]
    fn layout_hooks_present_in_layout_css() {
        for hook in [
            "data-blocks-banner-email-signup-field",
            "data-blocks-banner-email-signup-close",
        ] {
            assert!(LAYOUT_CSS.contains(hook), "LAYOUT_CSS に {hook} が無い");
        }
    }

    /// [`LAYOUT_CSS`] がレスポンシブ切替とグリッド配置を含み、`<` を含まない
    /// （`StyleSheet::push_css` の禁則文字チェック、CSS 文字列組み立ての
    /// 安全側確認）。
    #[test]
    fn layout_css_has_media_query_and_grid_areas_and_no_angle_bracket() {
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
        assert!(LAYOUT_CSS.contains("grid-template-areas"));
        assert!(!LAYOUT_CSS.contains('<'));
    }
}
