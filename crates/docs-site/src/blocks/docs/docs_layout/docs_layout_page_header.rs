//! `docs-layout-page-header` block。ドキュメントページ上部のヘッダーを
//! 合成する実例で、Docs Layout カテゴリの block 登録点は
//! `docs_layout/mod.rs` を正とする。
//! `_/blocks-intake/` の対応ファイルは本 worktree に存在しないため、
//! 原稿・本コメントには対応表 ID のみを記す（`page_heading_meta.rs` 等と
//! 同じ扱い）。
//!
//! # 使用部品
//!
//! `breadcrumb` / `heading` / `text` / `button` / `clipboard` / `badge` /
//! `code` の 7 部品のみを合成する（[`BLOCK`] の `parts` に一致させる契約）。
//! 新しい UI 部品は追加しない。
//!
//! # 3 インスタンスで派生形を表現する（無 JS のため静的併記）
//!
//! 対応表 ID R0080（パンくず + 編集・ソース操作 + 説明）を主参照に、
//! R0091（小見出し + タイトル + コピー）・R0079（メソッドバッジ +
//! エンドポイント行）を集約する。
//!
//! - **A（R0080・代表構成）**: パンくず → 見出し行（左に見出し、右に
//!   「編集する」「ソースを表示」ボタン）→ 説明文（`text` の `Muted`）。
//! - **B（R0091・小見出し + コピー）**: 小見出し（eyebrow）→ 見出し行
//!   （右に `clipboard` の「ページをコピー」）→ 説明文。
//! - **C（R0079・API ページ版）**: パンくず → 見出し行（右に「ソースを
//!   表示」）→ エンドポイント行（`badge` の HTTP メソッド + `clipboard` +
//!   `code` のパス表示）→ 説明文。
//!
//! # 狭幅では操作ボタンが見出しの下へ回る
//!
//! `header` は既定で縦積み（`flex-direction: column`）、`40rem` 以上で
//! 左右配置（`row` + `space-between`）へ切り替える（`page_heading_meta`
//! と同じリテラル値。テーマの breakpoint トークンは `@media` 条件式の中
//! では解決できない）。
//!
//! # 見出しレベルは `H2`
//!
//! ページ自身の `<h1>` との重複を避けるため、本 Demo の見出しは
//! `HeadingLevel::H2`（見た目は `HeadingSize::Xl2`）で統一する。
//!
//! # クリップボードは未コピー（idle）状態の固定表示
//!
//! 実際のコピー動作はクライアント配線層（wasm-full）の責務であり、無 JS
//! の docs サイトでは idle 状態の静的表示のみを描画する（`page_heading_meta`
//! の clipboard 合成と同型）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ボタンは `button::button` の既定 `type="button"` のまま送信先を
//! 持たない（`docs/policy/intentional-non-adoption.md` §3.25：バリデーション・
//! 送信処理は UI コンポーネント層の責務外）。文言はすべて独自の架空の日本語
//! ダミー（実企業名・実クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// パンくず（「Blocks」→ 中間項目 → 現在ページ）。
fn breadcrumb_row(middle_label: &'static str, current_label: &'static str) -> Node {
    breadcrumb::root(
        Size::Md,
        BreadcrumbVariant::default(),
        Some("パンくずリスト"),
        vec![],
        vec![breadcrumb::list(
            vec![],
            vec![
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../", vec![], vec![text("Blocks")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../", vec![], vec![text(middle_label)])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(vec![], vec![text(current_label)])],
                ),
            ],
        )],
    )
}

/// A: 代表構成（R0080）。パンくず → 見出し行（編集・ソース操作）→ 説明文。
fn instance_a() -> Node {
    let top_row = div(
        vec![("data-blocks-docs-layout-page-header-top-row", "")],
        vec![breadcrumb_row("ガイド", "埋め込みガイド")],
    );
    let header = div(
        vec![("data-blocks-docs-layout-page-header-header", "")],
        vec![
            div(
                vec![("data-blocks-docs-layout-page-header-title-group", "")],
                vec![heading(
                    HeadingLevel::H2,
                    &HeadingProps {
                        size: HeadingSize::Xl2,
                        ..HeadingProps::default()
                    },
                    vec![],
                    vec![text("埋め込みガイド")],
                )],
            ),
            div(
                vec![("data-blocks-docs-layout-page-header-actions", "")],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("編集する")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("ソースを表示")],
                    ),
                ],
            ),
        ],
    );
    let description = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "最小構成の HTML への部分埋め込み手順をまとめた案内ページです。",
        )],
    );
    div(
        vec![
            ("data-blocks-docs-layout-page-header-instance", ""),
            ("data-blocks-docs-layout-page-header-variant", "a"),
        ],
        vec![top_row, header, description],
    )
}

/// B: 小見出し + コピー（R0091）。小見出し → 見出し行（ページをコピー）→
/// 説明文。
fn instance_b() -> Node {
    const COPY_VALUE: &str = "/guides/npm-asset-build";
    const COPY_INPUT_ID: &str = "blocks-docs-layout-page-header-copy-b";
    let top_row = div(
        vec![("data-blocks-docs-layout-page-header-top-row", "")],
        vec![span(
            vec![("data-blocks-docs-layout-page-header-eyebrow", "")],
            vec![text("ガイド")],
        )],
    );
    let copy = clipboard::root(
        COPY_VALUE,
        false,
        vec![("data-blocks-docs-layout-page-header-copy", "")],
        vec![
            clipboard::label(
                false,
                Some(COPY_INPUT_ID),
                vec![],
                vec![text("ページをコピー")],
            ),
            clipboard::control(
                false,
                vec![],
                vec![
                    clipboard::input(COPY_VALUE, false, vec![("id", COPY_INPUT_ID)]),
                    clipboard::trigger(
                        false,
                        vec![],
                        vec![
                            clipboard::indicator(false, false, vec![], vec![text("コピー")]),
                            clipboard::indicator(true, false, vec![], vec![text("コピー済み")]),
                        ],
                    ),
                ],
            ),
        ],
    );
    let header = div(
        vec![("data-blocks-docs-layout-page-header-header", "")],
        vec![
            div(
                vec![("data-blocks-docs-layout-page-header-title-group", "")],
                vec![heading(
                    HeadingLevel::H2,
                    &HeadingProps {
                        size: HeadingSize::Xl2,
                        ..HeadingProps::default()
                    },
                    vec![],
                    vec![text("NPM アセットビルド")],
                )],
            ),
            div(
                vec![("data-blocks-docs-layout-page-header-actions", "")],
                vec![copy],
            ),
        ],
    );
    let description = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "NPM 互換の静的アセットビルドを `--ignore-scripts` 前提で組み込む手順です。",
        )],
    );
    div(
        vec![
            ("data-blocks-docs-layout-page-header-instance", ""),
            ("data-blocks-docs-layout-page-header-variant", "b"),
        ],
        vec![top_row, header, description],
    )
}

/// C: API ページ版（R0079）。パンくず → 見出し行（ソース表示）→
/// エンドポイント行（メソッドバッジ + コピー可能なパス）→ 説明文。
fn instance_c() -> Node {
    const ENDPOINT: &str = "/v1/projects/{id}";
    const ENDPOINT_INPUT_ID: &str = "blocks-docs-layout-page-header-endpoint-c";
    let top_row = div(
        vec![("data-blocks-docs-layout-page-header-top-row", "")],
        vec![breadcrumb_row("API Reference", "プロジェクト取得")],
    );
    let header = div(
        vec![("data-blocks-docs-layout-page-header-header", "")],
        vec![
            div(
                vec![("data-blocks-docs-layout-page-header-title-group", "")],
                vec![heading(
                    HeadingLevel::H2,
                    &HeadingProps {
                        size: HeadingSize::Xl2,
                        ..HeadingProps::default()
                    },
                    vec![],
                    vec![text("プロジェクトを取得する")],
                )],
            ),
            div(
                vec![("data-blocks-docs-layout-page-header-actions", "")],
                vec![button::button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("ソースを表示")],
                )],
            ),
        ],
    );
    let endpoint_row = div(
        vec![("data-blocks-docs-layout-page-header-endpoint", "")],
        vec![
            badge::badge(
                &BadgeProps {
                    palette: ColorPalette::Success,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text("GET")],
            ),
            clipboard::root(
                ENDPOINT,
                false,
                vec![],
                vec![clipboard::control(
                    false,
                    vec![],
                    vec![
                        clipboard::value_text(
                            vec![],
                            vec![code::code(
                                &CodeProps::default(),
                                vec![("id", ENDPOINT_INPUT_ID)],
                                vec![text(ENDPOINT)],
                            )],
                        ),
                        clipboard::trigger(
                            false,
                            vec![],
                            vec![
                                clipboard::indicator(false, false, vec![], vec![text("コピー")]),
                                clipboard::indicator(true, false, vec![], vec![text("コピー済み")]),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    );
    let description = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text("指定した ID のプロジェクトを取得します。")],
    );
    div(
        vec![
            ("data-blocks-docs-layout-page-header-instance", ""),
            ("data-blocks-docs-layout-page-header-variant", "c"),
        ],
        vec![top_row, header, endpoint_row, description],
    )
}

/// `docs-layout-page-header` の Demo 本体（3 インスタンスを縦積みで並記
/// する。呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-docs-layout-page-header-layout")],
        vec![instance_a(), instance_b(), instance_c()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/docs-layout-page-header/",
    title: "docs-layout-page-header",
    category: BlockCategory::DocsLayout,
    rust_source: "crates/docs-site/src/blocks/docs/docs_layout/docs_layout_page_header.rs",
    demo_class: "blocks-docs-layout-page-header",
    parts: &[
        Part {
            label: "Breadcrumb",
            path: "/themes/breadcrumb/",
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
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Clipboard",
            path: "/themes/clipboard/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Code",
            path: "/themes/code/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `docs_layout_page_header` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型で、本ファイル内 private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-docs-layout-page-header-*` と
/// `[data-blocks-docs-layout-page-header-*]` のみを用いる。ルート class を
/// `demo_class` と別名にする（`page_heading_meta` 等と同じ Bugbot 教訓の
/// 回避）。
const LAYOUT_CSS: &str = "\
.blocks-docs-layout-page-header-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
[data-blocks-docs-layout-page-header-instance] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-docs-layout-page-header-top-row] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-docs-layout-page-header-eyebrow] {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
[data-blocks-docs-layout-page-header-header] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-docs-layout-page-header-title-group] {\n  display: flex;\n  flex-direction: column;\n  min-width: 0;\n}\n\
[data-blocks-docs-layout-page-header-actions] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-docs-layout-page-header-endpoint] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
@media (min-width: 40rem) {\n  [data-blocks-docs-layout-page-header-header] {\n    flex-direction: row;\n    justify-content: space-between;\n    align-items: flex-start;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（breadcrumb/heading/text/button/clipboard/badge/code）
    /// の anatomy をすべて実際に出力していること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"breadcrumb\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"button\"",
            "data-scope=\"clipboard\"",
            "data-scope=\"badge\"",
            "data-scope=\"code\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// インスタンス数はちょうど 3 件（A〜C）。
    #[test]
    fn demo_instance_count() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-docs-layout-page-header-instance")
                .count(),
            3
        );
    }

    /// clipboard はすべて未コピー（idle）状態の固定表示
    /// （`aria-live` 等の実装詳細ではなく `data-copied` が false 固定である
    /// ことを確認する）。
    #[test]
    fn clipboard_is_static_idle() {
        let html = render(&demo());
        assert!(!html.contains("data-copied=\"true\""));
    }

    /// `<form>`・`type="submit"`・`href="#"`・`data:` URI を出力しない
    /// （`crate::blocks` モジュール doc）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in ["<form", "type=\"submit\"", "href=\"#\"", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイントを持ち、`<` を含まない
    /// （REQ-1: `</style>` によるスタイル脱出を防ぐ）。
    #[test]
    fn layout_css_declares_breakpoint_and_no_angle_bracket() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-docs-layout-page-header-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-docs-layout-page-header-layout"
        );
    }

    /// C（API ページ版）は "GET" バッジと `data-scope="code"` のエンドポイント
    /// 表示を持つ。
    #[test]
    fn api_instance_has_method_badge_and_endpoint_code() {
        let html = render(&demo());
        assert!(html.contains("GET"));
        assert!(html.contains("data-scope=\"code\""));
    }
}
