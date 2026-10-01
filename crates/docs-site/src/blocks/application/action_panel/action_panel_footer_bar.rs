//! `action-panel-footer-bar` block（イシュー #2952。親トラッキング #2951
//! 配下）。セクション末尾に上罫線付きの細い帯を置き、左に補足テキスト、
//! 右にボタン群（表示・編集・保存等）を並べる操作バーの合成例。
//!
//! # 使用部品
//!
//! `button` / `button-group` / `text` / `separator` / `icon` の 5 部品のみを
//! 合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は
//! 追加しない。
//!
//! # 3 インスタンス併記（主 R0647・R0648・R0649）
//!
//! - A（R0647、代表構成）: 補足テキストが上・操作列が下（狭幅）、右へ
//!   Outline「表示」「編集」の button-group + 単体 Solid「保存」+
//!   アイコンのみの「その他の操作」`icon_button`
//! - B（R0648、積み順・ボタン順違い）: 狭幅で操作列を先に積み、主ボタン
//!   （「保存」）を先頭に置く並び順違い
//! - C（R0649、主・副ボタンの購入導線）: Solid「購入する」+ Outline
//!   「カートに追加」の 2 ボタンのみ。狭幅では各ボタンを全幅にして縦積み
//!
//! 狭幅（`40rem` 未満）の積み順・全幅化は [`LAYOUT_CSS`] の
//! `data-blocks-action-panel-footer-bar-stack` 値で切り替える
//! （`note-first`/`actions-first`/`full-width`）。
//!
//! # 狭幅判定は `@container`（デモ枠の実幅基準）
//!
//! この block は `.docs-content` 内の `.blocks-demo` に表示され、
//! ビューポート幅（`@media`）はデモ枠自体の幅と一致しない
//! （`profile_detail_datalist.rs` と同じ教訓）。ルート
//! `.blocks-action-panel-footer-bar-layout` に `container-type:
//! inline-size` を設定し、`@container blocks-action-panel-footer-bar
//! (min-width: 40rem)` でデモ枠の実幅を基準に切り替える。
//!
//! # アイコンのみのボタンには `aria-label` を付与する
//!
//! 「その他の操作」ボタンは `button::icon_button` 経由で組み立て、
//! アクセシブルネームを必須の `aria-label` として出力する（装飾アイコン
//! 自体は `IconProps::default()` のまま `aria-hidden="true"`）。
//!
//! # アイコンは自作の単純幾何図形
//!
//! `page_heading_cover.rs::geo_icon` と同型で、`icon::icon` へ独自の
//! `<path d="...">` を渡すのみ（実在ブランドのアイコンセットは使わない）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ボタンは `button::button`/`button::icon_button` の既定
//! `type="button"` のまま送信先を持たない（`docs/policy/intentional-non-adoption.md`
//! §3.25：バリデーション・送信処理は UI コンポーネント層の責務外）。
//! 文言はすべて独自の架空ダミー（実企業名・実クレデンシャル・PII を
//! 含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::button_group::{self, Orientation};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// 自作の単純な幾何アイコン（`page_heading_cover.rs::geo_icon` と同型。
/// モジュール doc「アイコンは自作の単純幾何図形」参照）。開いた線分のみの
/// パスのため `fill="none"` + `stroke="currentColor"` でアウトライン描画
/// にする。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
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

/// 縦三点アイコン（「その他の操作」の装飾。実在アイコンセットは使わない）。
fn more_icon() -> Node {
    geo_icon("M12 6v.01 M12 12v.01 M12 18v.01")
}

/// 1 インスタンス分の帯を組み立てる（上罫線 + 補足テキスト + 操作列）。
fn instance(stack: &'static str, note: &'static str, actions: Vec<Node>) -> Node {
    let rule = separator::separator(
        &SeparatorProps::default(),
        vec![("data-blocks-action-panel-footer-bar-rule", "")],
    );
    let note_node = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-action-panel-footer-bar-note", "")],
        vec![fandhe_frontend_core::text(note)],
    );
    let actions_node = div(
        vec![("data-blocks-action-panel-footer-bar-actions", "")],
        actions,
    );
    let bar = div(
        vec![
            ("data-blocks-action-panel-footer-bar-bar", ""),
            ("data-blocks-action-panel-footer-bar-stack", stack),
        ],
        vec![note_node, actions_node],
    );
    div(
        vec![("data-blocks-action-panel-footer-bar-instance", "")],
        vec![rule, bar],
    )
}

/// `action-panel-footer-bar` の Demo 本体（呼び出しごとに同一の `Node` を
/// 返す純関数）。3 インスタンス（R0647/R0648/R0649）を縦に併記する。
pub fn demo() -> Node {
    // A: 代表構成（R0647）。補足が上・操作が下、Outline 2 個の button-group
    // + 単体 Solid「保存」+ アイコンのみ「その他の操作」。
    let example_a = instance(
        "note-first",
        "最終保存: 2 分前（下書き）",
        vec![
            button_group::root(
                Orientation::Horizontal,
                "レコードの操作",
                vec![],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![fandhe_frontend_core::text("表示")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![fandhe_frontend_core::text("編集")],
                    ),
                ],
            ),
            button::button(
                &ButtonProps::default(),
                vec![],
                vec![fandhe_frontend_core::text("保存")],
            ),
            button::icon_button(
                &ButtonProps::default(),
                "その他の操作",
                vec![],
                vec![more_icon()],
            ),
        ],
    );

    // B: 積み順・ボタン順違い（R0648）。狭幅では操作列が先、主ボタン
    // （保存）を先頭に置く。
    let example_b = instance(
        "actions-first",
        "変更は自動で保存されません",
        vec![
            button::button(
                &ButtonProps::default(),
                vec![],
                vec![fandhe_frontend_core::text("保存")],
            ),
            button_group::root(
                Orientation::Horizontal,
                "レコードの操作",
                vec![],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![fandhe_frontend_core::text("編集")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![fandhe_frontend_core::text("表示")],
                    ),
                ],
            ),
        ],
    );

    // C: 主・副ボタンの購入導線（R0649）。狭幅では各ボタンを全幅にして
    // 縦積み。
    let example_c = instance(
        "full-width",
        "合計 ¥4,800（税込）・送料無料",
        vec![
            button::button(
                &ButtonProps::default(),
                vec![],
                vec![fandhe_frontend_core::text("購入する")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![fandhe_frontend_core::text("カートに追加")],
            ),
        ],
    );

    div(
        vec![("class", "blocks-action-panel-footer-bar-layout")],
        vec![example_a, example_b, example_c],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/action-panel-footer-bar/",
    title: "action-panel-footer-bar",
    category: BlockCategory::ActionPanel,
    rust_source: "crates/docs-site/src/blocks/application/action_panel/action_panel_footer_bar.rs",
    demo_class: "blocks-action-panel-footer-bar",
    parts: &[
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Button Group",
            path: "/themes/button-group/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `action_panel_footer_bar` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型で、本ファイル内 private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-action-panel-footer-bar-*` と
/// `[data-blocks-action-panel-footer-bar-*]` のみを用いる。ルート class は
/// `demo_class`（`blocks-action-panel-footer-bar`）とは別名の
/// `blocks-action-panel-footer-bar-layout` にする（既存 block と同じ
/// Bugbot 教訓の回避）。
const LAYOUT_CSS: &str = "\
.blocks-action-panel-footer-bar-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  container-type: inline-size;\n  container-name: blocks-action-panel-footer-bar;\n}\n\
[data-blocks-action-panel-footer-bar-bar] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  padding-top: var(--fandhe-space-3);\n}\n\
[data-blocks-action-panel-footer-bar-bar][data-blocks-action-panel-footer-bar-stack=\"actions-first\"] {\n  flex-direction: column-reverse;\n}\n\
[data-blocks-action-panel-footer-bar-actions] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-action-panel-footer-bar-stack=\"full-width\"] [data-blocks-action-panel-footer-bar-actions] {\n  flex-direction: column;\n  align-items: stretch;\n}\n\
[data-blocks-action-panel-footer-bar-stack=\"full-width\"] [data-blocks-action-panel-footer-bar-actions] [data-scope=\"button\"][data-part=\"root\"] {\n  width: 100%;\n}\n\
@container blocks-action-panel-footer-bar (min-width: 40rem) {\n  [data-blocks-action-panel-footer-bar-bar] {\n    flex-direction: row;\n    align-items: center;\n    justify-content: space-between;\n  }\n  [data-blocks-action-panel-footer-bar-bar][data-blocks-action-panel-footer-bar-stack=\"actions-first\"] {\n    flex-direction: row;\n  }\n  [data-blocks-action-panel-footer-bar-actions] {\n    flex-direction: row;\n    width: auto;\n    margin-inline-start: auto;\n  }\n  [data-blocks-action-panel-footer-bar-stack=\"full-width\"] [data-blocks-action-panel-footer-bar-actions] {\n    flex-direction: row;\n    align-items: center;\n  }\n  [data-blocks-action-panel-footer-bar-stack=\"full-width\"] [data-blocks-action-panel-footer-bar-actions] [data-scope=\"button\"][data-part=\"root\"] {\n    width: auto;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（button/button-group/text/separator/icon）の
    /// anatomy をすべて実際に出力していること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"button\"",
            "data-scope=\"button-group\"",
            "data-scope=\"text\"",
            "data-scope=\"separator\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// 全ボタン（button-group 内含む）が `type="button"` 固定で、個数が
    /// 一致すること（A: 4 + B: 3 + C: 2 = 9）。
    #[test]
    fn demo_buttons_are_type_button_and_expected_count() {
        let html = render(&demo());
        let root_count = html
            .matches("data-scope=\"button\" data-part=\"root\"")
            .count();
        assert_eq!(root_count, 9);
        assert_eq!(html.matches(r#"type="button""#).count(), 9);
    }

    /// アイコンのみのボタンに `aria-label` があり、button-group root に
    /// `role="group"` + `aria-label` があること。
    #[test]
    fn demo_has_accessible_names() {
        let html = render(&demo());
        assert!(html.contains(r#"aria-label="その他の操作""#));
        assert!(html.contains(r#"role="group""#));
        assert!(html.contains(r#"aria-label="レコードの操作""#));
    }

    /// `<form>`・`type="submit"`・`href="#"`・`data:` URI・`<script` を
    /// 出力しない（`crate::blocks` モジュール doc）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in [
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "src=\"data:",
            "<script",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイントを持ち、`<` を含まない
    /// （REQ-1: `</style>` によるスタイル脱出を防ぐ）。3 つの `stack` 値が
    /// すべて `demo()` 出力の属性値として現れること。デモ枠の実幅基準の
    /// `@container`（ビューポート幅基準の `@media` ではない）であることも
    /// 固定する。
    #[test]
    fn layout_css_declares_breakpoint_and_stack_values_present() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-action-panel-footer-bar (min-width: 40rem)"));
        let html = render(&demo());
        for stack in ["note-first", "actions-first", "full-width"] {
            let needle = format!("data-blocks-action-panel-footer-bar-stack=\"{stack}\"");
            assert!(
                html.contains(&needle),
                "demo output should contain {needle}"
            );
        }
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-action-panel-footer-bar-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-action-panel-footer-bar-layout"
        );
    }
}
