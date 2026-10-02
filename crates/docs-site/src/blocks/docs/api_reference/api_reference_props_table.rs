//! `api-reference-props-table` block（イシュー #3103。親トラッキング #3099
//! 「Blocks 目的別パーツ拡充ツリー」配下、対応表 ID R0196（4 列構成）を
//! 構造の参照元とする合成例。API ドキュメントの一部として使う、
//! プロパティ名・型・既定値・説明の 4 列を持つ枠付きの表）。
//! 取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! # 使用部品
//!
//! `table` / `code` / `text` / `heading` の 4 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 見出しレベルに `H3` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、intro 領域の見出しは
//! [`fandhe_frontend_pre_styled_ui::heading::HeadingLevel::H3`] にする
//! （`comparison_table` 等、他の block と同じ判断）。
//!
//! # 行見出しに `table::row_header` を使う理由（`<th scope="row">`）
//!
//! プロパティ名（各行の見出し）は
//! [`fandhe_frontend_pre_styled_ui::table::row_header`]（`<th scope="row">`、
//! イシュー #2825 で新設された専用パーツ）へ置く。値セルと同じく `<td>` へ
//! 置くと、スクリーンリーダー利用者が値セル間を移動した際に列見出し
//! （「プロパティ」「型」等）しか読み上げられず「どのプロパティの値か」が
//! 判別できない（`comparison_table` と同じ a11y 教訓）。
//!
//! # `drop_class_attr` の契約（CSS フックの選び方）
//!
//! `heading::heading` / `text::text` / `code::code` / `table::root` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って
//! 除去する契約を持つため、これらの Demo 固有スタイルフックは
//! `data-blocks-api-reference-props-table-*` 属性で渡す。`table::header`/
//! `body`/`row`/`column_header`/`cell`/`row_header`/`caption`/
//! `scroll_area` は `drop_class_attr` を経由しないため `class` がそのまま
//! 効くが、他 block と同じく名前空間分離のため
//! `.blocks-api-reference-props-table-*` クラスを使う。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # `id`/`aria-labelledby` を出力しない理由
//!
//! 本 block はいずれの部品も `id` を要さない構成のため一切出力しない
//! （`comparison_table` と同じ判断、
//! `crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` の対象）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。プロパティ名・型・説明はすべて架空の UI 部品のもの（実在の
//! 製品名・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::ColorPalette;

/// プロパティ表 1 行分（架空コンポーネントのプロパティ）。
struct PropRow {
    name: &'static str,
    ty: &'static str,
    default: &'static str,
    description: &'static str,
}

/// 架空コンポーネント（`Toast`）のプロパティ一覧。`<` と `&` を含む型表記
/// （`Option<&str>`）を 1 件含め、既定エスケープ経路を実演する
/// （ファイル内ユニットテストで固定する不変条件）。
const PROPS: [PropRow; 5] = [
    PropRow {
        name: "title",
        ty: "Option<&str>",
        default: "None",
        description: "通知の見出し文言。省略時は本文のみ表示します。",
    },
    PropRow {
        name: "variant",
        ty: "ToastVariant",
        default: "Info",
        description: "見た目の種別（Info / Success / Warning / Danger）。",
    },
    PropRow {
        name: "dismissible",
        ty: "bool",
        default: "true",
        description: "閉じるボタンを表示するかどうか。",
    },
    PropRow {
        name: "duration_ms",
        ty: "u32",
        default: "4000",
        description: "自動で閉じるまでの表示時間（ミリ秒）。",
    },
    PropRow {
        name: "on_dismiss",
        ty: "Option<fn()>",
        default: "None",
        description: "閉じられた際に呼び出すコールバック。",
    },
];

/// プロパティ 1 行分（`table::row`）を組み立てる。
///
/// プロパティ名は `table::row_header`（`<th scope="row">`、モジュール doc
/// 「行見出しに `table::row_header` を使う理由」節）の中に accent palette の
/// `code` を置いて強調する。型・既定値は `table::cell` の中に既定
/// （Neutral）palette の `code`、説明は `styled_text::text`（Muted）。
fn prop_row(row: &PropRow) -> Node {
    table::row(
        vec![],
        vec![
            table::row_header(
                vec![("data-blocks-api-reference-props-table-name", "")],
                vec![code::code(
                    &CodeProps {
                        palette: ColorPalette::Accent,
                        ..CodeProps::default()
                    },
                    vec![],
                    vec![text(row.name)],
                )],
            ),
            table::cell(
                vec![("data-blocks-api-reference-props-table-type", "")],
                vec![code::code(
                    &CodeProps::default(),
                    vec![],
                    vec![text(row.ty)],
                )],
            ),
            table::cell(
                vec![("data-blocks-api-reference-props-table-default", "")],
                vec![code::code(
                    &CodeProps::default(),
                    vec![],
                    vec![text(row.default)],
                )],
            ),
            table::cell(
                vec![("data-blocks-api-reference-props-table-description", "")],
                vec![styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text(row.description)],
                )],
            ),
        ],
    )
}

/// `api-reference-props-table` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ設計）。
pub fn demo() -> Node {
    let intro = div(
        vec![("class", "blocks-api-reference-props-table-intro")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("Toast props")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-api-reference-props-table-lead", "")],
                vec![text(
                    "Toast コンポーネントが受け付けるプロパティの一覧です。",
                )],
            ),
        ],
    );

    let header = table::header(
        vec![],
        vec![table::row(
            vec![],
            vec![
                table::column_header(vec![], vec![text("プロパティ")]),
                table::column_header(vec![], vec![text("型")]),
                table::column_header(vec![], vec![text("既定値")]),
                table::column_header(vec![], vec![text("説明")]),
            ],
        )],
    );

    let body = table::body(vec![], PROPS.iter().map(prop_row).collect());

    let table_node = table::root(
        TableProps {
            variant: TableVariant::Outline,
            ..TableProps::default()
        },
        vec![("data-blocks-api-reference-props-table-table", "")],
        vec![
            table::caption(vec![], vec![text("Toast コンポーネントの props 一覧")]),
            header,
            body,
        ],
    );

    let scroll = table::scroll_area(
        vec![
            ("data-blocks-api-reference-props-table-scroll", ""),
            ("role", "region"),
            ("aria-label", "Toast props 一覧表"),
            ("tabindex", "0"),
        ],
        vec![table_node],
    );

    div(
        vec![("class", "blocks-api-reference-props-table-layout")],
        vec![intro, scroll],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/api-reference-props-table/",
    title: "api-reference-props-table",
    category: BlockCategory::ApiReference,
    rust_source: "crates/docs-site/src/blocks/docs/api_reference/api_reference_props_table.rs",
    demo_class: "blocks-api-reference-props-table",
    parts: &[
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Code",
            path: "/themes/code/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `api_reference_props_table` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節）。
///
/// 色はすべて既存トークン（`--fandhe-color-fg-muted` 等）のみを使う。
/// セルの上揃えは `[data-scope="table"]` を前置した複合セレクタで書く
/// （table recipe の `cell`/`row-header` base 規則に詳細度で勝つため、
/// `comparison_table` と同じ手法）。
const LAYOUT_CSS: &str = "\
.blocks-api-reference-props-table-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-api-reference-props-table-intro {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-api-reference-props-table-lead] {\n  margin: 0;\n}\n\
[data-scope=\"table\"][data-part=\"scroll-area\"][data-blocks-api-reference-props-table-scroll] {\n  width: 100%;\n}\n\
[data-blocks-api-reference-props-table-table] {\n  min-width: 40rem;\n}\n\
[data-scope=\"table\"][data-part=\"row-header\"][data-blocks-api-reference-props-table-name],\n[data-scope=\"table\"][data-part=\"cell\"][data-blocks-api-reference-props-table-type],\n[data-scope=\"table\"][data-part=\"cell\"][data-blocks-api-reference-props-table-default],\n[data-scope=\"table\"][data-part=\"cell\"][data-blocks-api-reference-props-table-description] {\n  vertical-align: top;\n}\n\
[data-scope=\"table\"][data-part=\"row-header\"][data-blocks-api-reference-props-table-name] {\n  white-space: nowrap;\n}\n\
[data-scope=\"table\"][data-part=\"cell\"][data-blocks-api-reference-props-table-description] {\n  min-width: 16rem;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, PROPS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 4 種の部品を出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"table\"",
            "data-scope=\"code\"",
            "data-scope=\"text\"",
            "data-scope=\"heading\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// `scope="col"` が 4 件、`scope="row"`（プロパティ数）が
    /// [`PROPS`] の件数と一致すること。
    #[test]
    fn demo_renders_expected_table_shape() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"scope="col""#).count(), 4);
        assert_eq!(html.matches(r#"scope="row""#).count(), PROPS.len());
        assert_eq!(
            html.matches(r#"data-scope="table" data-part="row-header""#)
                .count(),
            PROPS.len()
        );
    }

    /// プロパティ名の `code` が accent palette の variant class を持つこと。
    #[test]
    fn name_code_uses_accent_palette() {
        let html = render(&demo());
        assert!(html.contains("fd-code--color-palette-accent"));
    }

    /// `Option<&str>` がエスケープされて出力され、生の `<`/`&` が
    /// 出力に含まれないこと（既定エスケープの実演、REQ-1）。
    #[test]
    fn type_annotations_are_escaped() {
        let html = render(&demo());
        assert!(html.contains("Option&lt;&amp;str&gt;"));
        assert!(!html.contains("Option<&str>"));
    }

    /// 非対話・安全性の不変条件（`<form>`・`id=` 属性・`data:` URI を
    /// 持たないこと）。
    #[test]
    fn demo_never_contains_forbidden_markup() {
        let html = render(&demo());
        for absent in ["<form", "id=\"", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`comparison_table` 等と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-api-reference-props-table-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-api-reference-props-table-layout"
        );
    }

    /// [`LAYOUT_CSS`] が上揃え・横スクロール確保の規則を持つこと。
    #[test]
    fn layout_css_declares_vertical_align_and_min_width() {
        assert!(LAYOUT_CSS.contains("vertical-align: top;"));
        assert!(LAYOUT_CSS.contains("min-width: 40rem;"));
    }

    /// [`LAYOUT_CSS`] のルート規則が、`demo()` が実際に出力するルート class
    /// をセレクタとして参照していること（死んだ CSS ルール防止、
    /// `comparison_table` と同じレビュー教訓）。
    #[test]
    fn layout_css_root_rule_matches_demo_root_class() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-api-reference-props-table-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}"
        ));
    }
}
