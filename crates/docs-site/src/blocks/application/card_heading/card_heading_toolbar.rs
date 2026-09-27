//! `card-heading-toolbar` block（イシュー #2902。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充」配下、phase:3）。「見出しの右にツールバー
//! （検索・操作ボタン・三点メニュー）を持つ区画」の合成例。Application /
//! Card Heading カテゴリ最初の block（`docs/design/docs-site-blocks-section.md`
//! §18 の卒業手順に従い `card_heading.rs` から本ディレクトリへ改名した）。
//!
//! # 使用部品
//!
//! `heading` / `badge` / `text` / `input_group` / `input` / `button` /
//! `menu` / `empty_state` / `separator` の 9 部品のみを合成する（[`BLOCK`]
//! の `parts` に一致させる契約）。新しい UI 部品は追加しない。
//!
//! # 2 インスタンスで本文の差分を表現する（無 JS のため静的併記）
//!
//! 1 block・複数インスタンス縦積みの構成で、ヘッダー（見出し・バッジ・
//! 説明・ツールバー）は同一のまま、本文だけを次の 2 通りで併記する。
//!
//! - `content` variant: 点線枠のプレースホルダ本文。
//! - `empty` variant: `empty_state` に差し替えた空状態表示。
//!
//! # レイアウト（`40rem` 未満でツールバーを見出しの下へ回す）
//!
//! ヘッダーは既定で縦積み（見出し群 → ツールバー）。`40rem` 以上で左右
//! 配置（見出し群 | ツールバー）へ切り替える（テーマの breakpoint トークン
//! は `@media` 条件式の中では解決できないため、他の block と同じく
//! リテラル値を直書きする）。
//!
//! # 三点メニューは無 JS のため閉じた状態で固定する
//!
//! 開閉状態機械・実際のポップアップ表示はクライアント配線層（wasm-full）の
//! 責務であり、無 JS の docs サイトでは `OpenState::Closed` の静的表示のみを
//! 描画する（`dashboard_01.rs` の `user_menu()` と同型の構成: `root` は
//! `trigger` と `positioner`（`content` を内包）を子に持つ）。
//!
//! # 可視ラベルの代わりに `aria-label`
//!
//! 検索欄・三点メニューの trigger は `field`/`visually_hidden` を使用部品
//! に含めないため、`aria-label` でアクセシブル名を確保する。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ボタンは `button::button` の既定 `type="button"` のまま送信先を
//! 持たない（`docs/policy/intentional-non-adoption.md` §3.25：バリデーション・
//! 送信処理は UI コンポーネント層の責務外）。文言はすべて独自の架空の
//! 日本語ダミー（実企業名・実クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps};
use fandhe_frontend_pre_styled_ui::field::{FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// ヘッダー左側（見出し群）。
fn heading_group() -> Node {
    div(
        vec![("data-blocks-card-heading-toolbar-heading-group", "")],
        vec![
            div(
                vec![("data-blocks-card-heading-toolbar-title-row", "")],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("進行中のタスク")],
                    ),
                    badge::badge(
                        &BadgeProps {
                            palette: ColorPalette::Neutral,
                            ..BadgeProps::default()
                        },
                        vec![],
                        vec![text("12 件")],
                    ),
                ],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("担当者ごとの進捗をまとめて確認できます。")],
            ),
        ],
    )
}

/// 三点メニュー（無 JS のため閉じた状態で固定する。モジュール doc
/// 「三点メニューは無 JS のため閉じた状態で固定する」節参照）。
fn overflow_menu(content_id: &'static str) -> Node {
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id),
        vec![("aria-label", "その他の操作")],
        vec![text("\u{2026}")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id),
        None,
        vec![],
        vec![
            menu::item("export", false, false, vec![], vec![text("書き出す")]),
            menu::item(
                "archive",
                false,
                false,
                vec![],
                vec![text("アーカイブする")],
            ),
            menu::separator(vec![], vec![]),
            menu::item("delete", false, false, vec![], vec![text("削除する")]),
        ],
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// ヘッダー右側（ツールバー: 検索 + 操作ボタン + 三点メニュー）。
fn toolbar(search_field_id: &'static str, menu_content_id: &'static str) -> Node {
    let field = FieldProps {
        id: search_field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    div(
        vec![("data-blocks-card-heading-toolbar-toolbar", "")],
        vec![
            input_group::root(
                &group_props,
                vec![],
                vec![
                    input::input(
                        &InputProps::default(),
                        &field,
                        vec![
                            ("type", "search"),
                            ("placeholder", "検索"),
                            ("aria-label", "区画内を検索"),
                        ],
                    ),
                    input_group::addon(
                        InputGroupAlign::InlineEnd,
                        &group_props,
                        vec![],
                        vec![button::button(
                            &ButtonProps::default(),
                            vec![],
                            vec![text("検索")],
                        )],
                    ),
                ],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("書き出す")],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("新規作成")]),
            overflow_menu(menu_content_id),
        ],
    )
}

/// 本文（`variant` に応じてプレースホルダ / 空状態を切り替える。モジュール
/// doc「2 インスタンスで本文の差分を表現する」節参照）。
fn body(variant: &'static str) -> Node {
    let inner = if variant == "content" {
        div(
            vec![("data-blocks-card-heading-toolbar-placeholder", "")],
            vec![styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("ここに区画の本文が入ります。")],
            )],
        )
    } else {
        empty_state::root(
            &EmptyStateProps::default(),
            vec![],
            vec![empty_state::content(
                vec![],
                vec![
                    empty_state::title(vec![], vec![text("項目がまだありません")]),
                    empty_state::description(
                        vec![],
                        vec![text("最初の項目を追加すると、ここに一覧が表示されます。")],
                    ),
                    empty_state::actions(
                        vec![],
                        vec![button::button(
                            &ButtonProps::default(),
                            vec![],
                            vec![text("最初の項目を追加")],
                        )],
                    ),
                ],
            )],
        )
    };
    div(
        vec![
            ("data-blocks-card-heading-toolbar-body", ""),
            ("data-blocks-card-heading-toolbar-variant", variant),
        ],
        vec![inner],
    )
}

/// 操作行（左: 最終更新時刻、右: キャンセル/保存）。
fn footer() -> Node {
    div(
        vec![("data-blocks-card-heading-toolbar-footer", "")],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("最終更新: 5 分前")],
            ),
            div(
                vec![("data-blocks-card-heading-toolbar-footer-actions", "")],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("キャンセル")],
                    ),
                    button::button(&ButtonProps::default(), vec![], vec![text("保存")]),
                ],
            ),
        ],
    )
}

/// 区画 1 件（`variant` で本文を切り替える。id 接尾辞はパネル間で id が
/// 重複しないようにするための一意化。モジュール doc「id の一意性」節参照）。
fn panel(variant: &'static str, suffix: &'static str) -> Node {
    let search_field_id = if suffix == "content" {
        "blocks-card-heading-toolbar-search-content"
    } else {
        "blocks-card-heading-toolbar-search-empty"
    };
    let menu_content_id = if suffix == "content" {
        "blocks-card-heading-toolbar-menu-content"
    } else {
        "blocks-card-heading-toolbar-menu-empty"
    };
    div(
        vec![
            ("data-blocks-card-heading-toolbar-panel", ""),
            ("data-blocks-card-heading-toolbar-variant", variant),
        ],
        vec![
            div(
                vec![("data-blocks-card-heading-toolbar-header", "")],
                vec![heading_group(), toolbar(search_field_id, menu_content_id)],
            ),
            separator::separator(&SeparatorProps::default(), vec![]),
            body(variant),
            separator::separator(&SeparatorProps::default(), vec![]),
            footer(),
        ],
    )
}

/// `card-heading-toolbar` の Demo 本体（プレースホルダ版・空状態版の 2
/// パネルを縦積みで並記する。呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-card-heading-toolbar-layout")],
        vec![panel("content", "content"), panel("empty", "empty")],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/card-heading-toolbar/",
    title: "card-heading-toolbar",
    category: BlockCategory::CardHeading,
    rust_source: "crates/docs-site/src/blocks/application/card_heading/card_heading_toolbar.rs",
    demo_class: "blocks-card-heading-toolbar",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
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
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Empty State",
            path: "/themes/empty-state/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `card_heading_toolbar` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型で、本ファイル内 private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-card-heading-toolbar-*` と
/// `[data-blocks-card-heading-toolbar-*]` のみを用いる。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-card-heading-toolbar` だが、`demo()`
/// が返すルート `div` の class は `blocks-card-heading-toolbar-layout`
/// という別名にする（`section_heading_split` 等と同じ Bugbot 教訓の回避）。
const LAYOUT_CSS: &str = "\
.blocks-card-heading-toolbar-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
[data-blocks-card-heading-toolbar-panel] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-6);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n}\n\
[data-blocks-card-heading-toolbar-header] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-card-heading-toolbar-heading-group] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-card-heading-toolbar-title-row] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-card-heading-toolbar-toolbar] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-card-heading-toolbar-toolbar] [data-scope=\"input-group\"][data-part=\"root\"] {\n  flex: 1 1 12rem;\n  min-width: 0;\n}\n\
[data-blocks-card-heading-toolbar-placeholder] {\n  min-height: 6rem;\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  border: 1px dashed var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  padding: var(--fandhe-space-4);\n}\n\
[data-blocks-card-heading-toolbar-footer] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-card-heading-toolbar-footer-actions] {\n  display: flex;\n  gap: var(--fandhe-space-2);\n}\n\
@media (min-width: 40rem) {\n  [data-blocks-card-heading-toolbar-header] {\n    flex-direction: row;\n    justify-content: space-between;\n    align-items: flex-start;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（heading/badge/text/input-group/input/button/menu/
    /// empty-state/separator）の anatomy をすべて実際に出力していること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"badge\"",
            "data-scope=\"text\"",
            "data-scope=\"input-group\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"button\"",
            "data-scope=\"menu\"",
            "data-scope=\"empty-state\"",
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// パネルはちょうど 2 件（content 版・empty 版）。
    #[test]
    fn demo_panel_count() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-card-heading-toolbar-panel")
                .count(),
            2
        );
    }

    /// `empty-state` の root はちょうど 1 件（empty 版だけ）。
    #[test]
    fn demo_empty_state_count() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-scope=\"empty-state\" data-part=\"root\"")
                .count(),
            1
        );
    }

    /// `<form>` を出力しない・XSS 回帰の不変条件を固定する
    /// （`crate::blocks` モジュール doc）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in ["<form", "type=\"submit\"", "href=\"#\"", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// menu は閉じた状態（`aria-expanded="false"`）で固定し、trigger の
    /// `aria-controls` が content の id と一致する。
    #[test]
    fn menu_is_static_closed() {
        let html = render(&demo());
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(html.contains(r#"aria-controls="blocks-card-heading-toolbar-menu-content""#));
        assert!(html.contains(r#"id="blocks-card-heading-toolbar-menu-content""#));
        assert!(html.contains(r#"aria-controls="blocks-card-heading-toolbar-menu-empty""#));
        assert!(html.contains(r#"id="blocks-card-heading-toolbar-menu-empty""#));
    }

    /// 検索欄は `aria-label` を持つ。
    #[test]
    fn search_input_has_aria_label() {
        let html = render(&demo());
        assert!(html.contains(r#"aria-label="区画内を検索""#));
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
        assert!(html.contains("class=\"blocks-card-heading-toolbar-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-card-heading-toolbar-layout"
        );
    }
}
