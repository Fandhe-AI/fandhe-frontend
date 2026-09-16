//! `footer-newsletter` block（イシュー #2551。親 #2530「Phase 7: Motion+
//! 部品化」配下、Motion+ `sections/footers` に相当する合成例で、
//! `crate::blocks` モジュール doc の契約を `footer-sticky-reveal` に続いて
//! 17 件目に実装する）。
//!
//! # 使用部品
//!
//! `field`/`input`（メールアドレス入力）+ `button`（Subscribe）+ `link`
//! （footer リンク列）+ `icon`（完了パネルのチェックマーク、自作幾何 SVG）
//! を合成する（[`BLOCK`] の `parts` に一致させる契約）。チェックマークは
//! `cta_signup_celebrate::checkmark_icon` と同型の自作アイコンであり、
//! 2 箇所目のためヘルパー共有はしない（YAGNI）。
//!
//! # 3 カラム footer + newsletter 列の presence（入力 ⇄ 完了）
//!
//! footer はブランド説明 / リンク列 2 種 / newsletter の 3 カラム grid。
//! newsletter 列は「入力」panel（`field`+`input`+`button`）と「完了」panel
//! （チェックマーク + 完了文）の 2 個を同一列内に置き、片方へ `hidden` を
//! 付ける。無 JS の docs サイトで両状態を可視化するため、`pricing_tiers_morph`
//! と同型の**2 インスタンス併記**を採る: 1 個目は完了 panel を `hidden`
//! （＝「入力」状態を表示）、2 個目は入力 panel を `hidden`（＝「完了」状態
//! を表示）。無 JS でも両状態が可視で現れ、実アプリでは JS が `hidden` の
//! 付け替えのみで遷移させる使い方を示す。
//!
//! # presence 同型 CSS（`SlotRecipe::presence_transition` を手書きで再現）
//!
//! `pre-styled-ui::recipe::presence_transition` は `data-scope`/`data-part`
//! を持つ部品 recipe 専用 builder であり、素の `div` へ直接適用できないため、
//! [`pricing_tiers_morph`](super::pricing_tiers_morph) と同じ判断で
//! `[data-blocks-footer-newsletter-panel]` へ同型の宣言を [`LAYOUT_CSS`] に
//! 手書きで再現する（`opacity`/`transform` + `transition-behavior:
//! allow-discrete` + `@starting-style`）。duration は
//! `var(--fandhe-motion-duration-normal)` のリテラル参照のため
//! `Theme::to_css` の `prefers-reduced-motion: reduce` 一括 0 化機構へ
//! 自動追従する（個別の `@media` は不要）。
//!
//! # `<form>` を使わない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。送信ボタンは `type="button"` のまま送信先を持たず、実際の
//! 送信処理・バリデーションは利用者自身の Rust コードで実装する
//! （`docs/policy/intentional-non-adoption.md` §3.25）。リンク先はすべて
//! `Fandhe-AI` の実在 GitHub リポジトリへの外部絶対 URL であり `href="#"`
//! は使わない（`demo()` が `base_path` を受け取らない制約〔[`Block::demo`]
//! は `fn() -> Node`〕の下で `linkcheck::check_links` の fail-closed 検証を
//! 満たしつつ実在の遷移先を示す唯一の実用的な手段。
//! [`footer_sticky_reveal`](super::footer_sticky_reveal) と同じ判断）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `field::root`/`button::button`/`link::root`/`icon::icon` は
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有スタイルは `data-blocks-footer-newsletter-*`
//! 属性で渡し、[`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。素の
//! `div`/`footer`/`p` には `class` がそのまま効く。

use super::{Block, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, footer, p, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 自作チェックマーク（`cta_signup_celebrate::checkmark_icon` と同型）。
fn checkmark_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Md,
            ..IconProps::default()
        },
        vec![("data-blocks-footer-newsletter-check", "")],
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

/// footer リンク列 1 群（見出し + リンク一覧）。
fn link_column(heading_text: &str, links: &[(&str, &str)]) -> Node {
    let items: Vec<Node> = links
        .iter()
        .map(|(href, label)| {
            div(
                vec![("class", "blocks-footer-newsletter-link-item")],
                vec![link::root(
                    href,
                    &LinkProps::default(),
                    vec![],
                    vec![text(*label)],
                )],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-footer-newsletter-column")],
        std::iter::once(p(
            vec![("class", "blocks-footer-newsletter-column-title")],
            vec![text(heading_text)],
        ))
        .chain(items)
        .collect(),
    )
}

/// 「入力」panel（email + Subscribe）。`field_id` は 2 インスタンス併記
/// での `id` 重複防止用。
fn subscribe_panel(field_id: &'static str, hidden: bool) -> Node {
    let email_field = FieldProps {
        id: field_id,
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

    let mut attrs = vec![("data-blocks-footer-newsletter-panel", "")];
    if hidden {
        attrs.push(("hidden", ""));
    }

    div(
        attrs,
        vec![
            field::root(
                &orientation,
                &email_field,
                vec![("data-blocks-footer-newsletter-field", "")],
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
                vec![("data-blocks-footer-newsletter-submit", "")],
                vec![text("Subscribe")],
            ),
        ],
    )
}

/// 「完了」panel（チェックマーク + 完了文）。
fn subscribed_panel(hidden: bool) -> Node {
    let mut attrs = vec![("data-blocks-footer-newsletter-panel", "")];
    if hidden {
        attrs.push(("hidden", ""));
    }

    div(
        attrs,
        vec![
            checkmark_icon(),
            p(
                vec![("class", "blocks-footer-newsletter-subscribed-text")],
                vec![text("Subscribed! Thanks for joining.")],
            ),
        ],
    )
}

/// newsletter 列（見出し + 2 panel。`subscribed` 側を可視にする）。
fn newsletter_column(field_id: &'static str, subscribed: bool) -> Node {
    div(
        vec![("class", "blocks-footer-newsletter-column")],
        vec![
            p(
                vec![("class", "blocks-footer-newsletter-column-title")],
                vec![text("Newsletter")],
            ),
            subscribe_panel(field_id, subscribed),
            subscribed_panel(!subscribed),
        ],
    )
}

/// footer 3 カラム本体。`field_id` は 2 インスタンス併記時の `id` 重複防止。
fn footer_instance(field_id: &'static str, subscribed: bool) -> Node {
    const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
    footer(
        vec![("data-blocks-footer-newsletter-root", "")],
        vec![
            div(
                vec![("class", "blocks-footer-newsletter-column")],
                vec![
                    p(
                        vec![("class", "blocks-footer-newsletter-column-title")],
                        vec![text("Fandhe Frontend")],
                    ),
                    p(
                        vec![("class", "blocks-footer-newsletter-description")],
                        vec![text("Rust 製フロントエンドフレームワーク。")],
                    ),
                ],
            ),
            link_column("Product", &[(REPO, "Guide"), (REPO, "API Reference")]),
            link_column("Community", &[(REPO, "GitHub")]),
            newsletter_column(field_id, subscribed),
        ],
    )
}

/// `footer-newsletter` の Demo 本体（モジュール doc「2 インスタンス併記」
/// 節参照）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-footer-newsletter-stack", "")],
        vec![
            p(
                vec![("data-blocks-footer-newsletter-caption", "")],
                vec![text("Before subscribe")],
            ),
            footer_instance("blocks-footer-newsletter-email-before", false),
            p(
                vec![("data-blocks-footer-newsletter-caption", "")],
                vec![text("After subscribe")],
            ),
            footer_instance("blocks-footer-newsletter-email-after", true),
        ],
    )
}
// blocks-code:end

/// [`super::BLOCKS`] へ登録するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/footer-newsletter/",
    title: "footer-newsletter",
    rust_source: "crates/docs-site/src/blocks/footer_newsletter.rs",
    demo_class: "blocks-footer-newsletter",
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
    ],
    demo,
};

/// `footer_newsletter` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。
///
/// `[data-blocks-footer-newsletter-panel]` の presence 同型宣言は
/// モジュール doc「presence 同型 CSS」節参照。ドリフト防止は本ファイル
/// 末尾の `#[cfg(test)]` が `var(--fandhe-motion-duration-normal)`
/// リテラルとの一致で固定する。
pub(super) const LAYOUT_CSS: &str = "\
[data-blocks-footer-newsletter-root] {\n  display: grid;\n  grid-template-columns: 2fr 1fr 1fr 1.5fr;\n  gap: 2rem;\n  padding: 1.5rem;\n  background: var(--fandhe-color-bg-subtle);\n  border-radius: 0.5rem;\n}\n\
.blocks-footer-newsletter-column {\n  display: flex;\n  flex-direction: column;\n  gap: 0.5rem;\n}\n\
.blocks-footer-newsletter-column-title {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n}\n\
.blocks-footer-newsletter-description {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-footer-newsletter-link-item {\n  display: block;\n}\n\
[data-blocks-footer-newsletter-field] {\n  display: flex;\n  flex-direction: column;\n  gap: 0.5rem;\n  margin-bottom: 0.75rem;\n}\n\
[data-blocks-footer-newsletter-submit] {\n  width: 100%;\n}\n\
[data-blocks-footer-newsletter-check] {\n  color: var(--fandhe-color-fg-success, var(--fandhe-color-fg));\n}\n\
.blocks-footer-newsletter-subscribed-text {\n  margin: 0.5rem 0 0;\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-footer-newsletter-stack] {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n}\n\
[data-blocks-footer-newsletter-caption] {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-footer-newsletter-panel] {\n  opacity: 1;\n  transition-property: opacity, transform, display;\n  transition-duration: var(--fandhe-motion-duration-normal);\n  transition-timing-function: var(--fandhe-motion-easing-standard);\n  transition-behavior: allow-discrete;\n}\n\
[data-blocks-footer-newsletter-panel][hidden] {\n  opacity: 0;\n  transform: scale(0.95);\n}\n\
@starting-style {\n  [data-blocks-footer-newsletter-panel] {\n    opacity: 0;\n    transform: scale(0.95);\n  }\n}\n\
@media (max-width: 47.99rem) {\n  [data-blocks-footer-newsletter-root] {\n    grid-template-columns: 1fr;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::LAYOUT_CSS;

    /// [`LAYOUT_CSS`] が motion トークン（`--fandhe-motion-duration-normal`）
    /// を参照していること（モジュール doc「presence 同型 CSS」節が言う
    /// 「`Theme::to_css` の reduced-motion 一括 0 化への自動追従」の前提を
    /// 手書き文字列のドリフトから固定する）。
    #[test]
    fn layout_css_references_the_shared_duration_token() {
        assert!(LAYOUT_CSS.contains("var(--fandhe-motion-duration-normal)"));
    }
}
