//! `hero-install-command` block（イシュー #2786。親トラッキング #2730/#2731
//! 「Blocks 目的別パーツ拡充ツリー」配下、対応表 ID R0123 を主参照とし
//! R0521〜R0523 の構成差分を集約した合成例。インストールコマンドの
//! コピー欄を備えたヒーロー）。取得手段・ファイル名・内部コンポーネント
//! 識別子は記載しない（`docs/design/motion-reference-adoption-policy.md`
//! §9 と同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `clipboard` / `code` / `input-group` /
//! `input` / `button` / `breadcrumb` の 9 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。`field`/`visually_hidden` は補助的な
//! 合成手段のため `parts` には列挙しない（`text_reveal`/`cursor` を
//! `parts` に列挙しない先例と同じ判断）。
//!
//! # 3 インスタンスの併記（配置差分・コピー欄形式差分・状態差分）
//!
//! 無 JS の docs サイトでは `data-copied` の実際の切り替えが起きないため、
//! 配置・コピー欄形式・コピー状態の差分を 1 つの Demo 内に 3 インスタンス
//! 静的に並記して示す（`contact_split_info` 等、他 block の「状態は別
//! インスタンスで併記」パターンと同型）。
//!
//! - **A（中央寄せ・`clipboard` 形式・idle）**: R0521/R0123 の基準形。
//!   [`fandhe_frontend_pre_styled_ui::clipboard`] の `root`/`control`/
//!   `input`/`trigger`/`indicator` を組み合わせ、`indicator` は idle 側の
//!   み可視。
//! - **B（左寄せ・`input-group` 形式）**: R0522。`field::root` +
//!   `input_group::root` + `input::input`（`readonly`）+
//!   `input_group::addon`（`InlineEnd`）+ `input_group::button` で 1 行の
//!   コピー欄を表現する。コピー状態そのものは持たない構成。addon
//!   ボタンは `clipboard` scope の外側にあり `headless_clipboard`
//!   配線が届かないため、実アプリでも押下時にコピーは起きない
//!   （下記「コピー配線の範囲」節参照）。`disabled: true` で押下不能を
//!   明示する。
//!
//! # コピー配線の範囲（A/C は実アプリで機能する・B は機能しない）
//!
//! A/C は [`fandhe_frontend_pre_styled_ui::clipboard`] の
//! `root`/`control`/`input`/`trigger` を組み合わせているため、
//! `fandhe-frontend-wasm-full` の `headless_clipboard` 配線
//! （`crates/wasm-full/src/headless_clipboard.rs`）が `mount`/`hydrate`
//! 時に自動で `navigator.clipboard.writeText` を配線する。無 JS の docs
//! サイト自体では他の全部品と同じく静的表示に留まる（`site/primitives/
//! clipboard.md` が明記する既存の site 全体の制約）が、この block を
//! 実アプリへ組み込めば A/C のコピー操作は実際に機能する。一方 B の
//! addon ボタンは `clipboard` scope の外側の `input-group` パーツで
//! あり、上記配線の対象にならないため、実アプリでも機能しない
//! （`disabled: true` で明示する理由）。
//! - **C（パンくず付き左寄せ・`clipboard` 形式・copied）**: R0523。
//!   `breadcrumb` を導入要素に置き、`clipboard` の `value_text` に `code`
//!   を重ねてコマンドを等幅表示し、`indicator` は copied 側のみ可視。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない（B の `field::root` は `<div>` を出力する headless 実装であり
//! `<form>` ではない）。ボタンはすべて `type="button"`（`button::button`/
//! `clipboard::trigger`/`input_group::button` いずれも既定・固定で
//! `type="button"`）。
//!
//! # コマンド文字列・遷移先はすべて無害
//!
//! コピー対象コマンドは本フレームワーク自身の CLI・一般的な Rust
//! ツールチェインのみ（実在の秘密情報・トークンらしき文字列を含まない）。
//! `breadcrumb::link` の遷移先は本リポジトリの公開 URL 固定 1 件のみで、
//! `href="#"`・`mailto:`・`data:` は使わない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge`/`heading::heading`/`text::text`/`clipboard::root`/
//! `field::root`/`input_group::root`/`input::input`/`button::button`/
//! `breadcrumb::root` はいずれも `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有のスタイル
//! フックは `data-blocks-hero-install-command-*` 属性で渡す。素の `div` は
//! `class` がそのまま効くため `.blocks-hero-install-command-*` クラス
//! セレクタを使う。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 本フレームワーク自身の公開リポジトリ URL（`breadcrumb::link` の唯一の
/// 遷移先。`href="#"`・死にリンクを避けるための固定値、`hero_terminal`
/// 等と同型の判断）。
const REPO_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// A（中央寄せ・`clipboard` 形式・idle）を組み立てる。
fn instance_a() -> Node {
    let value = "cargo install fandhe-frontend-cli";
    let input_id = "blocks-hero-install-command-a-input";
    div(
        vec![
            ("data-blocks-hero-install-command-hero", ""),
            ("data-align", "center"),
        ],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("v0.1 公開中")]),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl4,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("コマンド一つで始める")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("テンプレートをそのまま実行するだけで動きます。")],
            ),
            clipboard::root(
                value,
                false,
                vec![("data-blocks-hero-install-command-command", "")],
                vec![
                    visually_hidden::root(
                        vec![],
                        vec![clipboard::label(
                            false,
                            Some(input_id),
                            vec![],
                            vec![text("インストールコマンド")],
                        )],
                    ),
                    clipboard::control(
                        false,
                        vec![],
                        vec![
                            clipboard::input(value, false, vec![("id", input_id)]),
                            clipboard::trigger(
                                false,
                                vec![],
                                vec![
                                    clipboard::indicator(false, false, vec![], vec![text("Copy")]),
                                    clipboard::indicator(
                                        true,
                                        false,
                                        vec![],
                                        vec![text("Copied!")],
                                    ),
                                ],
                            ),
                        ],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-hero-install-command-actions")],
                vec![
                    button::button(
                        &ButtonProps::default(),
                        vec![("data-blocks-hero-install-command-cta", "")],
                        vec![text("はじめる")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![("data-blocks-hero-install-command-cta", "")],
                        vec![text("ドキュメントを見る")],
                    ),
                ],
            ),
        ],
    )
}

/// B（左寄せ・`input-group` 形式）を組み立てる。コピー状態そのものは
/// 持たない構成（モジュール doc「3 インスタンスの併記」節参照）。
fn instance_b() -> Node {
    let field_id = "blocks-hero-install-command-b-input";
    let field_props = FieldProps {
        id: field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: true,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };

    div(
        vec![
            ("data-blocks-hero-install-command-hero", ""),
            ("data-align", "start"),
        ],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![],
                vec![text("既存プロジェクトに追加")],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl4,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("依存クレートを追加する")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("`Cargo.toml` に 1 行追加するだけで導入できます。")],
            ),
            field::root(
                &FieldRootProps {
                    orientation: FieldOrientation::Vertical,
                },
                &field_props,
                vec![],
                vec![
                    visually_hidden::root(
                        vec![],
                        vec![field::label(
                            &field_props,
                            vec![],
                            vec![text("追加コマンド")],
                        )],
                    ),
                    input_group::root(
                        &group_props,
                        vec![("data-blocks-hero-install-command-command", "")],
                        vec![
                            input::input(
                                &InputProps::default(),
                                &field_props,
                                vec![("value", "cargo add fandhe-frontend-core")],
                            ),
                            input_group::addon(
                                InputGroupAlign::InlineEnd,
                                &group_props,
                                vec![],
                                vec![input_group::button(
                                    // レビュー指摘対応（P1、イシュー #2786
                                    // codex 指摘）: この addon ボタンは
                                    // `clipboard` scope の外側にあるため
                                    // `fandhe-frontend-wasm-full` の
                                    // `headless_clipboard` 配線が届かず、
                                    // 実アプリに組み込んでも押下時に
                                    // コピーは起きない。`gallery_carousel`
                                    // の prev/next trigger・
                                    // `feature_tabs_panel` の CTA と同型の
                                    // 判断で `disabled: true`
                                    // （ボタン単体のみ、`group_props` 自体は
                                    // 変更せず addon/input の見た目は保つ）
                                    // にして「押しても何も起きない」ことを
                                    // 明示する。
                                    &InputGroupProps {
                                        disabled: true,
                                        ..group_props
                                    },
                                    vec![],
                                    vec![text("コピー")],
                                )],
                            ),
                        ],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-hero-install-command-actions")],
                vec![button::button(
                    &ButtonProps::default(),
                    vec![("data-blocks-hero-install-command-cta", "")],
                    vec![text("詳しく見る")],
                )],
            ),
        ],
    )
}

/// C（パンくず付き左寄せ・`clipboard` 形式・copied）を組み立てる。
fn instance_c() -> Node {
    let value = "fw new my-app";
    div(
        vec![
            ("data-blocks-hero-install-command-hero", ""),
            ("data-align", "start"),
        ],
        vec![
            breadcrumb::root(
                Size::Md,
                BreadcrumbVariant::default(),
                Some("Breadcrumb"),
                vec![],
                vec![breadcrumb::list(
                    vec![],
                    vec![
                        breadcrumb::item(
                            vec![],
                            vec![breadcrumb::link(REPO_URL, vec![], vec![text("Docs")])],
                        ),
                        breadcrumb::separator(vec![], vec![text("/")]),
                        breadcrumb::item(
                            vec![],
                            vec![breadcrumb::current_link(vec![], vec![text("Quick Start")])],
                        ),
                    ],
                )],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl4,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("新規プロジェクトを作る")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "コピーしたコマンドをターミナルに貼り付けて実行します。",
                )],
            ),
            clipboard::root(
                value,
                true,
                vec![("data-blocks-hero-install-command-command", "")],
                vec![clipboard::control(
                    true,
                    vec![],
                    vec![
                        clipboard::value_text(
                            vec![],
                            vec![code::code(&CodeProps::default(), vec![], vec![text(value)])],
                        ),
                        clipboard::trigger(
                            true,
                            vec![],
                            vec![
                                clipboard::indicator(false, true, vec![], vec![text("Copy")]),
                                clipboard::indicator(true, true, vec![], vec![text("Copied!")]),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    )
}

pub fn demo() -> Node {
    div(
        vec![("class", "blocks-hero-install-command-layout")],
        vec![instance_a(), instance_b(), instance_c()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/hero-install-command/",
    title: "hero-install-command",
    category: BlockCategory::Hero,
    rust_source: "crates/docs-site/src/blocks/marketing/hero/hero_install_command.rs",
    demo_class: "blocks-hero-install-command",
    parts: &[
        Part {
            label: "Badge",
            path: "/themes/badge/",
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
            label: "Clipboard",
            path: "/themes/clipboard/",
        },
        Part {
            label: "Code",
            path: "/themes/code/",
        },
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
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
            label: "Breadcrumb",
            path: "/themes/breadcrumb/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `hero_install_command` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節、他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。`md` 未満（`< 48rem`）で CTA を
/// 全幅縦積みにする要件をここで実装する。
const LAYOUT_CSS: &str = "\
.blocks-hero-install-command-layout {\n  display: flex;\n  flex-direction: column;\n  gap: 3rem;\n  padding-block: 2rem;\n}\n\
[data-blocks-hero-install-command-hero] {\n  max-width: 48rem;\n  display: flex;\n  flex-direction: column;\n  gap: 1rem;\n}\n\
[data-blocks-hero-install-command-hero][data-align=\"center\"] {\n  margin-inline: auto;\n  text-align: center;\n  align-items: center;\n}\n\
[data-blocks-hero-install-command-hero][data-align=\"start\"] {\n  align-items: flex-start;\n}\n\
[data-blocks-hero-install-command-command] {\n  width: 100%;\n  max-width: 28rem;\n  font-family: var(--fandhe-font-font-mono);\n}\n\
.blocks-hero-install-command-actions {\n  display: flex;\n  flex-direction: column;\n  width: 100%;\n  gap: 0.75rem;\n}\n\
.blocks-hero-install-command-actions [data-blocks-hero-install-command-cta] {\n  width: 100%;\n}\n\
@media (min-width: 48rem) {\n  \
.blocks-hero-install-command-actions {\n    flex-direction: row;\n    width: auto;\n    flex-wrap: wrap;\n  }\n  \
.blocks-hero-install-command-actions [data-blocks-hero-install-command-cta] {\n    width: auto;\n  }\n  \
[data-blocks-hero-install-command-hero][data-align=\"center\"] .blocks-hero-install-command-actions {\n    justify-content: center;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// [`LAYOUT_CSS`] が md ブレークポイントと主要セレクタを含むこと
    /// （「md 未満で CTA 全幅縦積み」要件の固定）。
    #[test]
    fn layout_css_declares_md_breakpoint_and_cta_selectors() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("[data-blocks-hero-install-command-cta]"));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// Demo が `<form>` を出力せず、`data-copied` は C（`clipboard::root`/
    /// `control`/`trigger` の 3 箇所）のみに現れること（A/B は idle/
    /// input-group 形式のため `data-copied` を持たない）。
    #[test]
    fn demo_has_no_form_and_exactly_one_copied_instance() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
        assert_eq!(html.matches("data-copied").count(), 3);
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("mailto:"));
        assert!(!html.contains("src=\"data:"));
    }
}
