//! `hero-prompt-input` block（イシュー #2788。対応表 ID R0537 を主参照とする
//! 単独集約）。中央寄せのタグライン + 見出し + リード文の下に、複数行入力と
//! 送信ボタンを 1 つの枠に一体化した「プロンプト入力欄」を配置する
//! AI アシスタント向けヒーロー。取得手段・ファイル名・内部コンポーネント
//! 識別子は記載しない（`docs/design/motion-reference-adoption-policy.md`
//! §9 と同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `field` / `input-group` / `textarea` /
//! `button` の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # `<form>` を持たない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 状態機械を持たない静的な合成例である。送信ボタンは `button::button`
//! （既定 `type="button"`）のまま送信先・バリデーションを持たず、実際の
//! 送信処理は利用者自身の Rust/JS コードで実装する
//! （`docs/policy/intentional-non-adoption.md` §3.25）。
//!
//! # `aria-label` を使い `field::label` を置かない理由
//!
//! 参照レイアウトは可視ラベルを持たず、入力欄のアクセシブルネームを
//! `aria-label` で与える。`headless` の [`field`](fandhe_frontend_pre_styled_ui::field) は `label` を
//! 出力しない限り `aria-labelledby` を自動付与しないため、`aria-label`
//! との衝突は起きない。
//!
//! # `input_group::root` の直下に `textarea` を置く契約
//!
//! [`fandhe_frontend_pre_styled_ui::input_group::root`] は子として
//! headless `field::textarea` を直接受け取る契約（`root` 自身は
//! `<textarea>` を出力しない）。送信ボタンは `InputGroupAlign::BlockEnd`
//! の `addon`（`flex-basis: 100%` で改行配置）に収め、textarea の下段へ
//! 独立した行として置く。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge` / `heading::heading` / `text::text` / `field::root` /
//! `textarea::textarea` / `button::button` / `input_group::root` /
//! `input_group::addon` はいずれも `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有のスタイル
//! フックは `data-blocks-hero-prompt-input-*` 属性で渡す。素の `div` には
//! `class` がそのまま効くため、コンテナのみ `.blocks-hero-prompt-input-*`
//! クラスセレクタを使う（`contact_centered_form` と同型の判断）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # 見出しレベル（`H3`）
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! [`HeadingLevel::H3`] にする（`contact_centered_form` 等と同型の判断）。
//!
//! 文言はすべて架空のもの（実在の人物・企業・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::textarea::{self, FieldIds, FieldProps, TextareaProps};

/// `hero-prompt-input` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    let field_id = "blocks-hero-prompt-input-prompt";
    let field_props = FieldProps {
        id: field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };

    let prompt = field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &field_props,
        vec![("data-blocks-hero-prompt-input-prompt", "")],
        vec![input_group::root(
            &InputGroupProps {
                disabled: false,
                invalid: false,
            },
            vec![],
            vec![
                textarea::textarea(
                    &TextareaProps::default(),
                    &field_props,
                    false,
                    vec![
                        ("placeholder", "コードとガイドについて質問する…"),
                        ("aria-label", "質問を入力"),
                        ("rows", "3"),
                    ],
                    vec![],
                ),
                input_group::addon(
                    InputGroupAlign::BlockEnd,
                    &InputGroupProps {
                        disabled: false,
                        invalid: false,
                    },
                    vec![("data-blocks-hero-prompt-input-actions", "")],
                    vec![button::button(
                        &ButtonProps::default(),
                        vec![("data-blocks-hero-prompt-input-submit", "")],
                        vec![text("送信")],
                    )],
                ),
            ],
        )],
    );

    div(
        vec![("class", "blocks-hero-prompt-input-inner")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-hero-prompt-input-tagline", "")],
                vec![text("AI アシスタント")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl4,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("ドキュメントに、そのまま質問する")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("コードとガイドを横断して、根拠付きで答えます。")],
            ),
            prompt,
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/hero-prompt-input/",
    title: "hero-prompt-input",
    category: BlockCategory::Hero,
    rust_source: "crates/docs-site/src/blocks/marketing/hero/hero_prompt_input.rs",
    demo_class: "blocks-hero-prompt-input",
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
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Textarea",
            path: "/themes/textarea/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `hero_prompt_input` 固有のレイアウト規則（`crate::blocks` モジュール doc
/// 「CSS の置き場」節）。他 block と同型に本ファイル内 private 定数として
/// [`BLOCK`] の `layout_css`（[`LayoutCss::Static`]）で自己申告し、
/// `crate::blocks::stylesheet` が `all_blocks()` 走査で `push_css` する。
///
/// セレクタは `.blocks-hero-prompt-input-*` と
/// `[data-blocks-hero-prompt-input-*]`、および styled `field`/`input-group`
/// の `[data-scope=...]` 系セレクタへの上書きのみを用い、他 block や部品の
/// 素のセレクタへ影響させない。狭幅（`@media (max-width: 47.99rem)`）では
/// 入力欄を全幅にする（`max-width: none`）。
const LAYOUT_CSS: &str = "\
.blocks-hero-prompt-input-inner {\n  max-width: 48rem;\n  margin-inline: auto;\n  text-align: center;\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n  padding-block: var(--fandhe-space-10);\n}\n\
[data-scope=\"field\"][data-part=\"root\"][data-blocks-hero-prompt-input-prompt] {\n  width: 100%;\n  max-width: 40rem;\n  text-align: start;\n}\n\
[data-scope=\"input-group\"][data-part=\"addon\"][data-blocks-hero-prompt-input-actions] {\n  justify-content: flex-end;\n}\n\
@media (max-width: 47.99rem) {\n  \
[data-scope=\"field\"][data-part=\"root\"][data-blocks-hero-prompt-input-prompt] {\n    max-width: none;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, BLOCK, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 7 種の部品を含むことを固定する（`crates/docs-site/
    /// tests/blocks_contract.rs` の横断検査と重複し過ぎない範囲での個別
    /// 固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"field\"",
            "data-scope=\"input-group\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"data-part="textarea""#));
    }

    /// `<form>` を出力しない・送信先を持たない静的表示で、`type="button"`
    /// がちょうど 1 個であること。
    #[test]
    fn demo_has_exactly_one_type_button_and_no_form() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"type="button""#).count(), 1);
        assert!(!html.contains("<form"));
        assert!(!html.contains(r#"type="submit""#));
        assert!(!html.contains("action="));
        assert!(!html.contains("src=\"data:"));
    }

    /// `aria-label` が `data-part="textarea"` を持つタグ自身の上にあること
    /// （CSS フック付け違えの再発防止、`contact_centered_form` と同型）。
    #[test]
    fn aria_label_is_on_the_textarea_part_element() {
        let html = render(&demo());
        let textarea_tags: Vec<&str> = html
            .split('<')
            .filter(|tag| tag.contains(r#"data-part="textarea""#))
            .collect();
        assert_eq!(
            textarea_tags.len(),
            1,
            "expected exactly one textarea part element in {html}"
        );
        assert!(
            textarea_tags[0].contains(r#"aria-label="質問を入力""#),
            "expected aria-label on the textarea part element, found tag: {}",
            textarea_tags[0]
        );
    }

    /// [`LAYOUT_CSS`] が狭幅ブレークポイントで入力欄を全幅にする規則を
    /// 持ち、`<` を含まないこと。
    #[test]
    fn layout_css_declares_narrow_breakpoint_full_width() {
        assert!(LAYOUT_CSS.contains("max-width: 40rem;"));
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
        assert!(LAYOUT_CSS.contains("max-width: none;"));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-hero-prompt-input-inner\""));
        assert_ne!(BLOCK.demo_class, "blocks-hero-prompt-input-inner");
    }
}
