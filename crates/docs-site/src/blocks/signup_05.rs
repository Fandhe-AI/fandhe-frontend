//! `signup-05` block（イシュー #2095。shadcn/ui Blocks の `signup-05`
//! 〔ソーシャルプロバイダ付きサインアップフォーム〕に相当する合成例で、
//! `crate::blocks` モジュール doc の契約を `login_01`/`dashboard_01`/
//! `sidebar_07`/`sidebar_03` に続いて 5 件目に実装する）。
//!
//! # 使用部品
//!
//! `field`（`group`/`root`/`label`/`separator`）+ `input`（メール入力）/
//! `button`（Solid 送信・Outline プロバイダ代替登録・リンク風アクション）/
//! `heading`（ブランド見出し）/ `icon`（自作の単純幾何図形）の 5 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # shadcn 側との構成上の判断（Issue 見立てからの差し替え）
//!
//! shadcn/ui の `signup-05`（registry `new-york-v4/signup-05`）実物を確認
//! したところ、Issue 本文の見立てにある `card`/`separator`/`link` は実物には
//! 存在しない（Card なし・入力欄は Email 1 個のみ・区切りは
//! `FieldSeparator`・リンクは `<a href="#">` の死リンク）。本実装は実物に
//! 合わせて次のとおり差し替える:
//!
//! - **`card` は使わない**: 実物に Card がない。外枠は素の `div` で構成する。
//! - **ブランド見出しは `heading::heading`（H3）**: shadcn 実物は `<h1>` だが、
//!   Demo 内に `h1` を置くとページ本体の H1 と重複するため
//!   [`fandhe_frontend_pre_styled_ui::heading::HeadingLevel::H3`] を使う。
//!   `heading` は `data-scope="heading"` を持つため
//!   `crate::layout::with_heading_anchors` の TOC 収集対象外であり
//!   （`login_01`/`dashboard_01` の card `title`〔h3〕と同じ機構）、H1 →
//!   `## Demo` の下に正しくネストする。
//! - **ロゴは非リンクの `div` + `icon`**: 死リンク不使用方針
//!   （`crate::blocks` モジュール doc「`<form>` を使わない」節と同型の判断）
//!   のため `<a href="#">` を出力しない。`icon::IconProps.label` に
//!   `Some("Acme Inc.")` を渡すことで `role="img"` + `aria-label` が付与され、
//!   shadcn 側の sr-only span 相当のアクセシブルネームを代替する。
//! - **Sign in 行 / Terms 行は素の `div` + `ButtonVariant::Link`**:
//!   `field::helper_text` は `FieldProps` と aria 配線を要求し用途が異なる
//!   ため、`login_01` と同じ判断でリンク風ボタンを使う（`href` を持たない、
//!   下記「`<form>` を使わない」節参照）。
//! - **`FieldSeparator`「Or」は `field::separator`**: `separator::group`/
//!   `label`（`crate::separator`）ではなく、shadcn 側の `FieldSeparator` に
//!   対応する `field::separator`（イシュー #2276 で追加、`login_04` 系と
//!   同型）を使う。
//! - **Apple/Google ロゴ + ラベル**: 実企業名・実ブランド・商標ロゴは
//!   持ち込まない方針（`docs/design/docs-site-blocks-section.md` §8）の
//!   ため、ラベルを一般名詞「Continue with provider A」「Continue with
//!   provider B」へ置換し、アイコンは `sidebar_03`/`sidebar_07` と同型の
//!   自作幾何図形（[`geo_icon`]）を使う。
//! - **`link` は使わない**: 遷移先を持たないアクションはすべて
//!   `ButtonVariant::Link` に統一する（`login_01` と同一方針）。
//!
//! # `<form>` を使わない・認証処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力せず、送信ボタンは `button::button` の既定 `type="button"` のまま
//! 用いる。値は一切送信されず、認証処理・送信先を持たない静的な合成例
//! である。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `field::root`/`button::button`/`heading::heading`/`icon::icon` は
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有スタイルは `data-blocks-signup-05-*` 属性で
//! 渡し、[`LAYOUT_CSS`] 側も同じ属性セレクタで対応する
//! （`crate::blocks` モジュール doc「CSS フックが `class` と `[data-*]` で
//! 混在する理由」節参照）。一方 `field::group`/`field::separator`（headless
//! 再エクスポートで `drop_class_attr` を経由しない）と素の `div` には
//! `class` がそのまま効くため、それらは従来どおり class セレクタを使う。

use super::{Block, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, icon_size_for, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の単純な矩形アイコン（著作物を複製しない、`sidebar_03`/`sidebar_07`
/// と同型。モジュール doc「Apple/Google ロゴ + ラベル」節参照）。
fn geo_icon(path_d: &'static str, size: Size) -> Node {
    icon(
        &IconProps {
            size,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// ブランド行（ロゴ枠 + 見出し + Sign in 導線）。
fn brand() -> Node {
    let logo = div(
        vec![("class", "blocks-signup-05-logo")],
        vec![icon(
            &IconProps {
                size: Size::Lg,
                label: Some("Acme Inc."),
                ..IconProps::default()
            },
            vec![],
            vec![el("path", vec![("d", "M4 4h16v16H4z")], vec![])],
        )],
    );
    let title = heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl,
            weight: HeadingWeight::Bold,
        },
        vec![("data-blocks-signup-05-title", "")],
        vec![text("Welcome to Acme Inc.")],
    );
    let signin_row = div(
        vec![("class", "blocks-signup-05-signin-row")],
        vec![
            text("Already have an account? "),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Link,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("Sign in")],
            ),
        ],
    );
    div(
        vec![("class", "blocks-signup-05-brand")],
        vec![logo, title, signin_row],
    )
}

/// `signup-05` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let email_field = FieldProps {
        id: "blocks-signup-05-email",
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
    let link_button = ButtonProps {
        variant: ButtonVariant::Link,
        ..ButtonProps::default()
    };
    let provider_icon_size = icon_size_for(Size::Md);

    let field_group = field::group(
        vec![],
        vec![
            brand(),
            field::root(
                &orientation,
                &email_field,
                vec![("data-blocks-signup-05-field", "")],
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
                vec![("data-blocks-signup-05-submit", "")],
                vec![text("Create Account")],
            ),
            field::separator(
                vec![("data-blocks-signup-05-separator", "")],
                vec![text("Or")],
            ),
            div(
                vec![("data-blocks-signup-05-providers", "")],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![("data-blocks-signup-05-provider", "")],
                        vec![
                            geo_icon("M12 3l9 8h-3v9H6v-9H3z", provider_icon_size),
                            text("Continue with provider A"),
                        ],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![("data-blocks-signup-05-provider", "")],
                        vec![
                            geo_icon("M4 4h16v16H4z", provider_icon_size),
                            text("Continue with provider B"),
                        ],
                    ),
                ],
            ),
        ],
    );

    let terms = div(
        vec![("class", "blocks-signup-05-terms")],
        vec![
            text("By clicking continue, you agree to our "),
            button::button(&link_button, vec![], vec![text("Terms of Service")]),
            text(" and "),
            button::button(&link_button, vec![], vec![text("Privacy Policy")]),
            text("."),
        ],
    );

    div(
        vec![("data-blocks-signup-05-stack", "")],
        vec![field_group, terms],
    )
}
// blocks-code:end

/// [`super::BLOCKS`] へ登録するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/signup-05/",
    title: "signup-05",
    rust_source: "crates/docs-site/src/blocks/signup_05.rs",
    demo_class: "blocks-signup-05",
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
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    demo,
};

/// `signup_05` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節。`login_01`/`sidebar_03`/`sidebar_07`/
/// `dashboard_01` と同型で `pub(super)` として `super::stylesheet` から
/// 連結される）。
pub(super) const LAYOUT_CSS: &str = "\
.blocks-signup-05 {\n  display: flex;\n  justify-content: center;\n  align-items: center;\n  min-height: 24rem;\n}\n\
[data-blocks-signup-05-stack] {\n  display: flex;\n  flex-direction: column;\n  gap: 1.5rem;\n  width: 100%;\n  max-width: 24rem;\n  margin: 0 auto;\n}\n\
.blocks-signup-05-brand {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: 0.5rem;\n  text-align: center;\n}\n\
.blocks-signup-05-logo {\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  width: 2rem;\n  height: 2rem;\n  border-radius: 0.375rem;\n}\n\
.blocks-signup-05-signin-row {\n  font-size: 0.875rem;\n  text-align: center;\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-signup-05-field] {\n  display: flex;\n  flex-direction: column;\n  gap: 0.5rem;\n}\n\
[data-blocks-signup-05-submit] {\n  width: 100%;\n}\n\
[data-blocks-signup-05-providers] {\n  display: grid;\n  grid-template-columns: repeat(2, 1fr);\n  gap: 1rem;\n}\n\
[data-blocks-signup-05-provider] {\n  width: 100%;\n}\n\
.blocks-signup-05-terms {\n  font-size: 0.75rem;\n  text-align: center;\n  color: var(--fandhe-color-fg-muted);\n  padding: 0 1.5rem;\n}\n\
@media (max-width: 39.99rem) {\n  [data-blocks-signup-05-providers] {\n    grid-template-columns: 1fr;\n  }\n}\n";
