//! `action-panel-with-well` block（イシュー #2956。Application / Action
//! Panel カテゴリ最初の block）。「タイトルの下に外側カードより一段濃い
//! 面色の内側枠（well）を置き、その中に支払手段の概要（カード種別アイコン・
//! 番号末尾・有効期限）と右端の編集ボタンを並べる」アクションパネルの
//! 合成例。
//!
//! # 使用部品
//!
//! `card` / `heading` / `text` / `icon` / `button` の 5 部品のみを合成する
//! （[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は追加しない。
//!
//! # well を新規部品ではなく `Card` の入れ子で表現する
//!
//! 「外側カードより一段濃い面色の内側枠」は、新規 UI 部品を追加せず
//! `card::root(CardVariant::Outline)` の `body` の中に
//! `card::root(CardVariant::Subtle)` を入れ子にして表現する。外側 Outline
//! の背景は `--fandhe-color-bg`、内側 Subtle の背景は
//! `--fandhe-color-bg-subtle` であり、この差が「一段濃い面色」に相当する
//! （`crates/pre-styled-ui/src/card.rs` の意図的非採用点「chakra
//! `bg.panel`」節参照。本トークン体系に panel 段がないため Subtle で
//! 近似する）。
//!
//! # `class` 属性の扱い（`drop_class_attr` 対策）
//!
//! `card::header`/`card::body`/`card::footer` は呼び出し側の `attrs` を
//! そのまま出力するため `class` を渡せるが、`text::text` は
//! `drop_class_attr` で `class` を無条件に除去する
//! （`page_heading_avatar.rs` モジュール doc「メタ行を素の `<p>` で組み立てる
//! 理由」と同じ制約）。本 block はレイアウト用 CSS フックを `card::body`
//! の `class`（`blocks-action-panel-with-well-row`）と、well 自体・編集
//! ボタンの識別に `data-blocks-action-panel-with-well-*` 属性で持たせ、
//! `text::text` へは `class` を渡さない。
//!
//! # 狭幅では折り返すのみで非表示にしない
//!
//! [`LAYOUT_CSS`] の `.blocks-action-panel-with-well-row` は
//! `flex-wrap: wrap` の単純な折り返しレイアウトであり、狭幅では編集ボタンが
//! 概要の下段へ回るだけで、`display: none` によって要素を隠すことはしない
//! （`page_heading_avatar.rs` と同型の判断）。
//!
//! # カード種別アイコンは自作の線画
//!
//! カード種別アイコンは `icon` 部品 + 自作の単純図形（角丸長方形 + 帯線
//! 1 本のカードシルエット）。実在ブランド（VISA 等）のロゴ・商標・名称は
//! 使わない。装飾用途のため `IconProps::default()`（`label: None`）のまま
//! 用い、隣接する概要テキストがアクセシブルネームを担う。
//!
//! # 番号末尾・有効期限はすべて架空値
//!
//! 「末尾 0187」「有効期限 2028/07」はいずれも独自に書いた無意味な架空値で
//! あり、実在のカード番号・実在の有効期限とは無関係。実在ブランド名も
//! 含まない（「クレジットカード」という一般名詞で表現する）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。編集ボタンは `button::button` の既定 `type="button"` の
//! まま用い、送信処理・送信先は一切持たない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};

/// カード種別アイコン（角丸長方形 + 帯線 1 本のカードシルエット、自作の
/// 単純図形）。装飾用途のため `IconProps::default()`（`label: None`）の
/// まま用いる。
fn card_icon() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M3 6h18v12H3z M3 10h18"),
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

/// 支払手段の概要（カード種別・番号末尾を 1 行、有効期限を 1 行）。
fn summary() -> Node {
    div(
        vec![("class", "blocks-action-panel-with-well-summary")],
        vec![
            styled_text::text(
                &TextProps {
                    weight: TextWeight::Medium,
                    ..TextProps::default()
                },
                vec![],
                vec![text("クレジットカード 末尾 0187")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("有効期限 2028/07")],
            ),
        ],
    )
}

/// 右端の編集ボタン。
fn edit_button() -> Node {
    button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![("data-blocks-action-panel-with-well-button", "")],
        vec![text("編集")],
    )
}

/// 内側 well（`CardVariant::Subtle` の入れ子）。カード種別アイコン・
/// 概要・編集ボタンを 1 行に並べる。
fn well() -> Node {
    card::root(
        CardProps {
            variant: CardVariant::Subtle,
            size: Size::Sm,
        },
        vec![("data-blocks-action-panel-with-well-well", "")],
        vec![card::body(
            vec![("class", "blocks-action-panel-with-well-row")],
            vec![card_icon(), summary(), edit_button()],
        )],
    )
}

/// `action-panel-with-well` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。外側 `Outline` カードのタイトル下に [`well`] を置く。
#[must_use]
pub fn demo() -> Node {
    card::root(
        CardProps::from(CardVariant::Outline),
        vec![("data-blocks-action-panel-with-well-panel", "")],
        vec![
            card::header(
                vec![],
                vec![heading::heading(
                    HeadingLevel::H2,
                    &HeadingProps {
                        size: HeadingSize::Md,
                        ..HeadingProps::default()
                    },
                    vec![],
                    vec![text("支払方法")],
                )],
            ),
            card::body(vec![], vec![well()]),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/action-panel-with-well/",
    title: "action-panel-with-well",
    category: BlockCategory::ActionPanel,
    rust_source: "crates/docs-site/src/blocks/application/action_panel/action_panel_with_well.rs",
    demo_class: "blocks-action-panel-with-well",
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
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `action_panel_with_well` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。狭幅では `flex-wrap` で
/// 折り返すのみで、編集ボタンを非表示にはしない（モジュール doc「狭幅では
/// 折り返すのみで非表示にしない」節参照）。
///
/// `.blocks-action-panel-with-well-row` は `card::body`（well 内側カードの
/// body）に付与するクラスだが、`card` recipe の `[data-scope="card"]
/// [data-part="body"]` セレクタが `flex-direction: column` を
/// 詳細度（0,2,0）で先に固定しているため、クラス単体（詳細度 0,1,0）の
/// `flex-direction: row` 指定は打ち消される。本セレクタは同じ属性 2 つ
/// （`data-scope`/`data-part`）にクラスを重ねて詳細度 (0,3,0) にし、確実に
/// row を上書きする（レビュー指摘、イシュー #2956）。
const LAYOUT_CSS: &str = "\
[data-blocks-action-panel-with-well-panel] {\n  max-inline-size: 40rem;\n}\n\
[data-scope=\"card\"][data-part=\"body\"].blocks-action-panel-with-well-row {\n  display: flex;\n  flex-direction: row;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-action-panel-with-well-summary {\n  display: flex;\n  flex: 1 1 12rem;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
[data-blocks-action-panel-with-well-well] [data-scope=\"icon\"] {\n  flex-shrink: 0;\n}\n\
[data-blocks-action-panel-with-well-well] [data-scope=\"button\"] {\n  margin-inline-start: auto;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
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
            "data-scope=\"text\"",
            "data-scope=\"icon\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn outer_and_inner_card_variants_both_present() {
        let html = demo_html();
        assert!(html.contains("fd-card--variant-outline"));
        assert!(html.contains("fd-card--variant-subtle"));
    }

    #[test]
    fn no_form_semantics_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
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
    fn layout_css_is_safe_and_reflows_without_hiding() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("flex-wrap: wrap;"));
        // 狭幅でも編集ボタンへ到達できることの回帰ガード（`page_heading_avatar.rs`
        // と同型の意図的判断）。
        assert!(!LAYOUT_CSS.contains("display: none"));
    }

    #[test]
    fn row_selector_outranks_card_body_column_specificity() {
        // レビュー指摘の回帰ガード（イシュー #2956）: `card::body` の
        // `[data-scope="card"][data-part="body"]`（詳細度 0,2,0）に
        // `flex-direction: column` が先に固定されているため、行レイアウト
        // 側のセレクタは同じ 2 属性 + クラスの詳細度 (0,3,0) で
        // `flex-direction: row` を持たなければ打ち消される。
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"card\"][data-part=\"body\"].blocks-action-panel-with-well-row {\n  display: flex;\n  flex-direction: row;"
        ));
    }

    #[test]
    fn summary_class_is_not_stripped() {
        // `div` は core のノード木 API を直接使うため `drop_class_attr` の
        // 対象外である回帰ガード。
        let html = demo_html();
        assert!(html.contains("class=\"blocks-action-panel-with-well-summary\""));
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }

    #[test]
    fn no_real_brand_names_or_full_card_numbers() {
        let html = demo_html();
        assert!(!html.to_ascii_lowercase().contains("visa"));
        assert!(!html.to_ascii_lowercase().contains("mastercard"));
        assert!(html.contains("末尾 0187"));
    }
}
