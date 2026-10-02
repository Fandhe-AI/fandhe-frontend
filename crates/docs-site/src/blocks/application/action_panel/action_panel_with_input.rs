//! `action-panel-with-input` block（イシュー #2955）。「見出し + 説明文の
//! 下に、メール入力欄と保存ボタンを横並びに置くカード状のパネル」の合成
//! 例。
//!
//! # 使用部品
//!
//! `card` / `heading` / `text` / `input` / `button` / `visually-hidden` の
//! 6 部品のみを合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい
//! UI 部品は追加しない。`field`/`FieldProps` は headless
//! `fandhe_frontend_headless_ui::field::input` の必須引数（アクセシビリティ
//! 状態の受け渡し）として使うのみで、`field::root`/`field::label` は
//! 呼ばない（構造は `card` の header/body で組む）。
//!
//! # 対応表参照
//!
//! 対応表の主参照 ID R0736（代表構成、集約元も同一のため差分インスタンス
//! は持たない）。`_/blocks-intake/` はローカル専用のため本 worktree には
//! 存在せず、`page_heading_avatar.rs`（#2931）・`list_title_meta.rs`
//! （#2925）と同じく対応表 ID のみを記す。文言・配色・アイコンは参照元を
//! 持ち込まず独自に書いた架空の文言。
//!
//! # ラベルを視覚的に隠す理由（`aria-label` ではなく実 `<label for>`）
//!
//! Issue 要件「入力欄のラベルは視覚的に隠す」を、`aria-label` によるラベル
//! 省略ではなく実 `<label for>` を [`visually_hidden::root`] で clip する
//! 構成で満たす。実 `<label>` はクリック時のフォーカス移動をブラウザが
//! ネイティブに提供する点で `aria-label` より堅牢（`pricing_comparison_table
//! .rs::select_view` の `<label for>` 前例と同型の判断、ただし本 block は
//! 視覚的に隠す点が異なる）。
//!
//! # 入力欄の `id` を `FieldIds::control` で固定する理由
//!
//! headless [`fandhe_frontend_headless_ui::field::FieldProps::id`] から導出
//! される既定コントロール id は `"{id}-control"` であり、`<label for>` を
//! 素の `el("label", ...)` で組み立てる本 block は導出前の生 id をそのまま
//! `for` 属性へ使いたい。[`FieldIds::control`] を明示指定して両者を一致
//! させる（`pricing_comparison_table.rs::select_view` と同型の判断、
//! `crates/pre-styled-ui/src/input.rs` モジュール doc 参照）。
//!
//! # `<form>` を使わない・初期状態を固定する
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。保存ボタンは `button::button` の既定 `type="button"` の
//! まま用い、送信処理・送信先を持たない。入力欄は `value` 属性を持たず、
//! 呼び出しごとに同一の未入力・未検証状態を返す（`demo()` は状態機械を
//! 持たない純関数）。
//!
//! # 狭幅ではボタンが入力欄の下へ折り返す（非表示にはしない）
//!
//! [`LAYOUT_CSS`] の行コンテナは `flex-wrap: wrap` の単純な折り返しのみで
//! 表現し、`page_heading_avatar.rs`「狭幅では操作列を折り返す（非表示には
//! しない）」節と同じ判断（`@container` によるボタン非表示は無 JS の docs
//! サイトでは実際に開閉・到達できなくなる部品向けの手段であり、単一の
//! 常時到達可能なボタンには不要）で `display: none` は使わない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// メール入力欄の `id`（`<label for>` と入力欄 `id` を一致させる固定
/// リテラル。モジュール doc「入力欄の `id` を `FieldIds::control` で固定
/// する理由」参照）。
const EMAIL_ID: &str = "blocks-action-panel-with-input-email";

/// メール入力欄のアクセシビリティ状態（初期状態＝未入力・未検証・
/// 有効・必須なし）。
fn email_field() -> FieldProps<'static> {
    FieldProps {
        id: EMAIL_ID,
        ids: FieldIds {
            control: Some(EMAIL_ID),
            ..FieldIds::default()
        },
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    }
}

/// 見出し（H2、[`Heading`](heading) 部品）。docs ページ自体が H1 を持つため
/// block 内は H2 以下とする（`page_heading_avatar.rs` と同型の判断）。
fn heading_node() -> Node {
    heading::heading(
        HeadingLevel::H2,
        &HeadingProps {
            size: HeadingSize::Md,
            ..HeadingProps::default()
        },
        vec![],
        vec![text("通知メールの送信先")],
    )
}

/// 説明文（[`Text`](styled_text) 部品、`Muted` variant）。
fn description() -> Node {
    styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text("週次レポートと重要なお知らせの送信先を設定します。")],
    )
}

/// 視覚的に隠したラベル + メール入力欄 + 保存ボタンの横並び行。
fn input_row() -> Node {
    let label = visually_hidden::root(
        vec![],
        vec![fandhe_frontend_core::el(
            "label",
            vec![("for", EMAIL_ID)],
            vec![text("メールアドレス")],
        )],
    );
    let email_input = input::input(
        &InputProps::default(),
        &email_field(),
        vec![
            ("type", "email"),
            ("autocomplete", "email"),
            ("placeholder", "you@example.com"),
        ],
    );
    let save_button = button::button(
        &ButtonProps::default(),
        vec![("data-blocks-action-panel-with-input-submit", "")],
        vec![text("保存する")],
    );
    div(
        vec![("class", "blocks-action-panel-with-input-row")],
        vec![label, email_input, save_button],
    )
}

/// `action-panel-with-input` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。レイアウト用 class は `card::root` へ渡さず最外を素の
/// `div` で包む（`card::root` は `drop_class_attr` で呼び出し側 `class` を
/// 除去するため、[`LAYOUT_CSS`] を効かせるには外側にもう 1 段必要。
/// `page_heading_avatar.rs::demo` と同型の判断）。
#[must_use]
pub fn demo() -> Node {
    let panel = card::root(
        CardProps::default(),
        vec![],
        vec![
            card::header(vec![], vec![heading_node(), description()]),
            card::body(vec![], vec![input_row()]),
        ],
    );
    div(
        vec![("class", "blocks-action-panel-with-input-layout")],
        vec![panel],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/action-panel-with-input/",
    title: "action-panel-with-input",
    category: BlockCategory::ActionPanel,
    rust_source: "crates/docs-site/src/blocks/application/action_panel/action_panel_with_input.rs",
    demo_class: "blocks-action-panel-with-input",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
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

/// `action_panel_with_input` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節と同型）。狭幅では行コンテナが
/// `flex-wrap` で折り返すのみで、ボタンを非表示にはしない（モジュール doc
/// 「狭幅ではボタンが入力欄の下へ折り返す（非表示にはしない）」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-action-panel-with-input-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-action-panel-with-input-row {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-action-panel-with-input-row [data-scope=\"field\"][data-part=\"input\"] {\n  flex: 1 1 16rem;\n  min-width: 0;\n  width: auto;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, EMAIL_ID, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"heading\"",
            "data-scope=\"field\"",
            "data-scope=\"button\"",
            "data-scope=\"visually-hidden\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(html.contains("<label"));
        assert!(html.contains(&format!("for=\"{EMAIL_ID}\"")));
        assert!(html.contains("type=\"email\""));
    }

    #[test]
    fn no_form_and_no_prefilled_value() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("value=\""));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn all_buttons_are_type_button() {
        let html = demo_html();
        let count_open = html.matches("<button").count();
        let count_typed = html.matches("type=\"button\"").count();
        assert!(count_open > 0);
        assert!(count_typed >= count_open, "html={html}");
    }

    #[test]
    fn label_is_visually_hidden_and_targets_input_id() {
        let html = demo_html();
        assert_eq!(
            html.matches(&format!("for=\"{EMAIL_ID}\"")).count(),
            1,
            "html={html}"
        );
        assert_eq!(
            html.matches(&format!("id=\"{EMAIL_ID}\"")).count(),
            1,
            "html={html}"
        );
    }

    #[test]
    fn layout_css_is_safe_and_wraps_without_hiding() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("flex-wrap: wrap;"));
        assert!(!LAYOUT_CSS.contains("display: none"));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-action-panel-with-input-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-action-panel-with-input-layout"
        );
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }
}
