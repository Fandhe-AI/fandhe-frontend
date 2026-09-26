//! `newsletter-stacked` block（イシュー #3226。親トラッキング #2738
//! 「Blocks 目的別パーツ拡充ツリー・Phase 1（マーケティング A）」配下、
//! 対応表 ID R0517（基準形）・R0191/R1099（基準形と同形の素材違い）・
//! R0518（左寄せ）・R0519（パンくず付き左寄せ）・R1100（暗色カード中央
//! 寄せ）の集約元 6 件を 1 つの Demo にまとめた「タグライン → 大見出し →
//! 説明文 → メール入力 → 送信ボタン」の縦積み newsletter 登録セクション。
//! Marketing / Newsletter カテゴリ 3 件目の block。
//!
//! # 使用部品
//!
//! `heading` / `text` / `breadcrumb` / `field` / `input` / `button` /
//! `visually-hidden` / `card` の 8 部品のみを合成する（[`BLOCK`] の
//! `parts` に一致させる契約）。新しい UI 部品は追加しない。
//!
//! # 4 形を 1 つの Demo へ並べる
//!
//! - **A（基準形、R0517/R0191/R1099）**: タグライン + 見出し + 説明文 +
//!   登録フォーム、中央寄せ・素の面。
//! - **B（左寄せ、R0518）**: A と同じ構成のまま整列だけ左寄せへ変える。
//! - **C（パンくず付き、R0519）**: 先頭要素をタグラインからパンくずへ
//!   差し替えた左寄せ。
//! - **D（暗色カード、R1100）**: [`card::root`]（`CardVariant::Elevated`）
//!   を `fg`/`bg` 反転トークンで暗色化した中央寄せ（固定の暗色 hex は
//!   使わない。`newsletter_split`/`section_heading_stacked` と同じ判断）。
//!
//! # タグライン・パンくず・登録フォームの再利用元
//!
//! タグライン（`eyebrow` 相当）は
//! [`section_heading_stacked`](super::super::section_heading::section_heading_stacked)
//! の `eyebrow` ヘルパと同形（`styled_text::text` に `Sm`/`Semibold` +
//! `data-blocks-newsletter-stacked-tagline` 属性、色は
//! `--fandhe-color-accent`）。パンくず（Home → Blocks → 現在ページ）は
//! 同モジュールの `breadcrumb_nav` と同形。登録フォーム（メール入力 +
//! 送信ボタン、可視ラベルなし）は [`newsletter_split`](super::newsletter_split)
//! の `signup` と同形（`visually_hidden::root` で包んだ `field::label` が
//! `<label for>` の関連付けを担う）。共通ヘルパの新設は行わず、各 block
//! ファイルが自己完結する既存の慣行に従い本ファイル内へ複製する。
//!
//! # 狭い画面幅でも縦積みのまま（`@media` を設けない）
//!
//! `newsletter_split` の 2 列切り替えとは異なり、本 block は「先頭要素 →
//! 見出し → 説明文 → 入力欄 → 送信ボタン」の縦積みを画面幅によらず固定する
//! （イシュー本文の要件）。`LAYOUT_CSS` に `@media` を持たない。
//!
//! # 暗色カード（D）の色継承
//!
//! `card::root` へ `background: var(--fandhe-color-fg); color:
//! var(--fandhe-color-bg);` を上書きする。`heading` は個別の色宣言を
//! 持たないため `color: inherit` の恩恵をそのまま受けるが、タグライン
//! （`--fandhe-color-accent` 明示）は暗色面でコントラストを保証しない
//! ため `color: inherit` で上書きする（`section_heading_stacked` の暗色面
//! と同じ理由）。説明文は D のみ `TextVariant::Plain`（色宣言なし →
//! 継承）にし、他 3 形は `TextVariant::Muted` を使う（`newsletter_split`
//! と同じ判断: `Muted` は自身に `color:
//! var(--fandhe-color-fg-muted)` を持つため暗色面の反転配色を上書きして
//! コントラストを崩す）。送信ボタンも D のみ `background:
//! var(--fandhe-color-bg); color: var(--fandhe-color-fg);` へ反転する。
//!
//! # `<form>`/`href="#"`/`mailto:` を持たない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。送信ボタンは `button::button` の既定 `type="button"` のまま
//! 送信先を持たない。パンくずのリンク先はサイト内相対パス
//! （`"../../"`/`"../"`）のみとし `href="#"` は使わない。文言はすべて
//! 独自の架空の日本語ダミー（実企業名・実クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// Demo 内の各形の上に付ける区別ラベル（`section_heading_stacked` と
/// 同型）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: fandhe_frontend_pre_styled_ui::text::TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// タグライン（アクセント色の強調テキスト、A/B/D で使用）。暗色面（D）
/// では `color: inherit` で上書きする（モジュール doc「暗色カード（D）の
/// 色継承」節、`LAYOUT_CSS` 参照）。
fn tagline(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: fandhe_frontend_pre_styled_ui::text::TextSize::Sm,
            weight: fandhe_frontend_pre_styled_ui::text::TextWeight::Semibold,
            ..TextProps::default()
        },
        vec![("data-blocks-newsletter-stacked-tagline", "")],
        vec![text(label)],
    )
}

/// C で使うパンくず（Home → Blocks → 現在ページ）。`aria_label` は必ず
/// 一意にし `<label for>` の重複関連付けを避ける（他 block と同様）。
fn breadcrumb_nav(aria_label: &'static str) -> Node {
    breadcrumb::root(
        Size::Md,
        BreadcrumbVariant::default(),
        Some(aria_label),
        vec![],
        vec![breadcrumb::list(
            vec![],
            vec![
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../../", vec![], vec![text("Home")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../", vec![], vec![text("Blocks")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(vec![], vec![text("Newsletter")])],
                ),
            ],
        )],
    )
}

/// メールアドレス入力 + 送信ボタン（`signup` 領域、`newsletter_split` と
/// 同形）。可視ラベルは出さず `visually_hidden::root` で包んだ
/// `field::label` が `<label for>` の関連付けを担う。`email_field_id` は
/// 4 インスタンスで一意にする。
fn signup(email_field_id: &'static str) -> Node {
    let email_field = FieldProps {
        id: email_field_id,
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
        vec![("class", "blocks-newsletter-stacked-signup")],
        vec![
            field::root(
                &orientation,
                &email_field,
                vec![],
                vec![
                    visually_hidden::root(
                        vec![],
                        vec![field::label(
                            &email_field,
                            vec![],
                            vec![text("メールアドレス")],
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
                vec![("data-blocks-newsletter-stacked-submit", "")],
                vec![text("登録する")],
            ),
        ],
    )
}

/// 縦積みの newsletter 登録 1 件を組み立てる（本 block の中核ヘルパ）。
///
/// - `align`: `"center"`/`"start"`。
/// - `tone`: `None`（通常面）/`Some("card")`（暗色カード、D 専用）。
/// - `lead`: 見出しの直前に置く先頭要素（タグライン/パンくず）。
fn stacked(
    align: &'static str,
    tone: Option<&'static str>,
    lead: Node,
    title: &'static str,
    description: &'static str,
    email_field_id: &'static str,
) -> Node {
    let mut attrs = vec![("data-blocks-newsletter-stacked-align", align)];
    if let Some(tone) = tone {
        attrs.push(("data-blocks-newsletter-stacked-tone", tone));
    }

    let description_variant = if tone.is_some() {
        TextVariant::Plain
    } else {
        TextVariant::Muted
    };

    div(
        attrs,
        vec![
            lead,
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: description_variant,
                    ..TextProps::default()
                },
                vec![],
                vec![text(description)],
            ),
            signup(email_field_id),
        ],
    )
}

/// `newsletter-stacked` の Demo 本体（4 形を縦に並べる。呼び出しごとに
/// 同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    let a = stacked(
        "center",
        None,
        tagline("お知らせ"),
        "縦積みの newsletter 登録",
        "タグライン・大見出し・説明文・登録フォームを中央に積む、最も基本的な形です。",
        "blocks-newsletter-stacked-email-center",
    );

    let b = stacked(
        "start",
        None,
        tagline("お知らせ"),
        "左寄せの newsletter 登録",
        "同じ構成のまま、常に左寄せで表示します。",
        "blocks-newsletter-stacked-email-start",
    );

    let c = stacked(
        "start",
        None,
        breadcrumb_nav("Breadcrumb example"),
        "パンくず付きの newsletter 登録",
        "現在地を示すパンくずを先頭に置き、左寄せで表示します。",
        "blocks-newsletter-stacked-email-breadcrumb",
    );

    let d = stacked(
        "center",
        Some("card"),
        tagline("お知らせ"),
        "暗色カードの newsletter 登録",
        "固定の暗色カードに収め、中央に積みます。",
        "blocks-newsletter-stacked-email-card",
    );

    let card_d = card::root(
        CardProps {
            variant: CardVariant::Elevated,
            ..CardProps::default()
        },
        vec![("data-blocks-newsletter-stacked-tone", "card")],
        vec![card::body(vec![], vec![d])],
    );

    div(
        vec![("class", "blocks-newsletter-stacked-layout")],
        vec![
            div(vec![], vec![variant_label("A. 基準形（中央寄せ）"), a]),
            div(vec![], vec![variant_label("B. 左寄せ"), b]),
            div(vec![], vec![variant_label("C. パンくず付き（左寄せ）"), c]),
            div(
                vec![],
                vec![variant_label("D. 暗色カード（中央寄せ）"), card_d],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/newsletter-stacked/",
    title: "newsletter-stacked",
    category: BlockCategory::Newsletter,
    rust_source: "crates/docs-site/src/blocks/marketing/newsletter/newsletter_stacked.rs",
    demo_class: "blocks-newsletter-stacked",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Breadcrumb",
            path: "/themes/breadcrumb/",
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
        Part {
            label: "Card",
            path: "/themes/card/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `newsletter_stacked` 固有のレイアウト規則。セレクタは
/// `.blocks-newsletter-stacked-*` と `[data-blocks-newsletter-stacked-*]`
/// のみを用いる（`contact_split_info` と同じ名前空間分離）。`@media` は
/// 持たない（モジュール doc「狭い画面幅でも縦積みのまま」節、常に縦積み
/// を固定する）。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-newsletter-stacked` だが、`demo()` が
/// 返すルート `div` の class は `blocks-newsletter-stacked-layout` という
/// 別名にする（`newsletter_split`/`section_heading_stacked` と同じ Bugbot
/// 教訓の回避）。
const LAYOUT_CSS: &str = "\
.blocks-newsletter-stacked-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
[data-blocks-newsletter-stacked-align] {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  text-align: center;\n  gap: var(--fandhe-space-4);\n  max-width: 36rem;\n  margin: 0 auto;\n}\n\
[data-blocks-newsletter-stacked-align=\"start\"] {\n  align-items: flex-start;\n  text-align: left;\n  margin: 0;\n}\n\
.blocks-newsletter-stacked-signup {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  width: 100%;\n  max-width: 24rem;\n}\n\
.blocks-newsletter-stacked-signup [data-scope=\"button\"][data-part=\"root\"] {\n  align-self: stretch;\n}\n\
[data-blocks-newsletter-stacked-tagline] {\n  color: var(--fandhe-color-accent);\n}\n\
.blocks-newsletter-stacked [data-scope=\"card\"][data-part=\"root\"][data-blocks-newsletter-stacked-tone=\"card\"] {\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);\n}\n\
.blocks-newsletter-stacked [data-scope=\"card\"][data-part=\"body\"] [data-blocks-newsletter-stacked-align] {\n  padding: 0;\n  margin: 0 auto;\n}\n\
[data-blocks-newsletter-stacked-tone=\"card\"] [data-blocks-newsletter-stacked-tagline] {\n  color: inherit;\n}\n\
[data-blocks-newsletter-stacked-tone=\"card\"] [data-scope=\"button\"][data-part=\"root\"][data-blocks-newsletter-stacked-submit] {\n  background: var(--fandhe-color-bg);\n  color: var(--fandhe-color-fg);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（heading/text/breadcrumb/field/input/button/
    /// visually-hidden/card）の anatomy をすべて実際に出力していることと、
    /// 4 形・整列・見出しレベルを固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"breadcrumb\"",
            "data-scope=\"field\" data-part=\"root\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"button\"",
            "data-scope=\"visually-hidden\"",
            "data-scope=\"card\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("<h3").count(),
            4,
            "demo should render exactly 4 headings"
        );
        assert!(
            !html.contains("<h1"),
            "demo should not render a page-level h1"
        );
        assert!(
            !html.contains("<h2"),
            "demo should not render a section-level h2"
        );
        assert_eq!(
            html.matches("data-blocks-newsletter-stacked-align=\"center\"")
                .count(),
            2,
            "A/D should be centered"
        );
        assert_eq!(
            html.matches("data-blocks-newsletter-stacked-align=\"start\"")
                .count(),
            2,
            "B/C should be left-aligned"
        );
        assert_eq!(
            html.matches("data-blocks-newsletter-stacked-tone=\"card\"")
                .count(),
            2,
            "D should have the card tone on both the card::root wrapper and its inner row"
        );
        assert_eq!(
            html.matches("data-scope=\"breadcrumb\" data-part=\"root\"")
                .count(),
            1,
            "only C uses breadcrumb"
        );
    }

    /// `<form>`/`type="submit"`/`href="#"`/`data:`/`mailto:` を出力しない・
    /// XSS 回帰の不変条件を固定する（`crate::blocks` モジュール doc）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in [
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "src=\"data:",
            "mailto:",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// 送信ボタンはちょうど 4 個、いずれも `type="button"`。
    #[test]
    fn demo_has_exactly_four_type_button_buttons() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"type="button""#).count(), 4);
    }

    /// visually-hidden な `<label for>` と同じ `id` を持つ input が 4 組
    /// 存在し、`id` の重複がない（アクセシブル名の関連付けを固定）。
    #[test]
    fn demo_label_for_matches_input_id() {
        let html = render(&demo());
        for id in [
            "blocks-newsletter-stacked-email-center",
            "blocks-newsletter-stacked-email-start",
            "blocks-newsletter-stacked-email-breadcrumb",
            "blocks-newsletter-stacked-email-card",
        ] {
            let for_attr = format!(r#"for="{id}-control""#);
            let id_attr = format!(r#"id="{id}-control""#);
            assert!(html.contains(&for_attr), "{for_attr} should be present");
            assert!(html.contains(&id_attr), "{id_attr} should be present");
        }
    }

    /// [`LAYOUT_CSS`] が `<` を含まず（REQ-1: `</style>` によるスタイル
    /// 脱出を防ぐ）、`@media` を持たない（狭い画面幅でも縦積みのまま固定
    /// する契約）ことと、card tone の上書きセレクタを持つことを固定する。
    #[test]
    fn layout_css_has_no_angle_bracket_and_no_media() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(!LAYOUT_CSS.contains("@media"));
        assert!(LAYOUT_CSS.contains("[data-blocks-newsletter-stacked-tone=\"card\"]"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「ルート class を `demo_class` と別名に
    /// する理由」節の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-newsletter-stacked-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-newsletter-stacked-layout");
    }
}
