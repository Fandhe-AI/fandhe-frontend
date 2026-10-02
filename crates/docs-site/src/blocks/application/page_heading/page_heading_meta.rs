//! `page-heading-meta` block（イシュー #2933。親トラッキング #2892
//! 「Blocks 目的別パーツ拡充」配下、phase:3）。見出しの下へアイコン付き
//! メタ情報を横一列に並べ、右側に操作ボタン群を置くページ見出しの合成例。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`page_heading_actions.rs`〔イシュー #2930〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `heading` / `badge` / `status` / `icon` / `button` / `button_group` /
//! `breadcrumb` / `clipboard` / `menu` の 9 部品のみを合成する（[`BLOCK`] の
//! `parts` に一致させる契約）。新しい UI 部品は追加しない。
//!
//! # 3 インスタンスで派生形を表現する（無 JS のため静的併記）
//!
//! 対応表 ID R1128（代表構成）を主参照に、R1132（常時表示のパンくず付き）・
//! R0187（上段ラベル + バッジ + 共有 URL）を集約する。
//!
//! - **A（R1128・代表構成）**: 見出し → メタ行（状態 `status` + 幾何アイコン
//!   付きの場所・日付・担当の 3 項目）→ 右側に `button_group`（編集・複製）
//!   + 主操作ボタン + 三点メニュー。
//! - **B（R1132・常時パンくず付き）**: 上段にパンくず、見出し → メタ行
//!   （状態は `badge` で示し `status` との使い分けを可視化）→ 右側は単発
//!   ボタン 2 個 + 三点メニュー。
//! - **C（R0187・上段ラベル + バッジ + 共有 URL）**: 上段に小さなラベル +
//!   `badge` → 見出し → メタ行（日付・担当）→ 右側に `clipboard`（共有
//!   URL）+ 主操作ボタン。
//!
//! # 狭幅ではメタ情報が折り返し、ボタン群は見出しの下へ回る
//!
//! `header` は既定で縦積み（`flex-direction: column`）、`40rem` 以上で
//! 左右配置（`row` + `space-between`）へ切り替える（`page_heading_actions`
//! と同じリテラル値。テーマの breakpoint トークンは `@media` 条件式の中では
//! 解決できない）。メタ行自体は `flex-wrap: wrap` で常に折り返す。
//!
//! # メニューは無 JS のため閉じた状態で固定する
//!
//! 開閉状態機械・実際のポップアップ表示はクライアント配線層（wasm-full）の
//! 責務であり、無 JS の docs サイトでは `OpenState::Closed` の静的表示のみを
//! 描画する（`page_heading_actions.rs::overflow_menu` と同型）。三点メニュー
//! 2 件（A・B）の `content` id はインスタンスごとに一意にする（id 重複・
//! 宙ぶらりん `aria-controls` 参照の回避、`blocks_contract.rs` の汎用
//! チェック参照）。
//!
//! # アイコンは自作の単純幾何図形
//!
//! `page_heading_actions.rs::geo_icon` と同型で、`icon::icon` へ独自の
//! `<path d="...">` を渡すのみ（実在ブランドのアイコンセットは使わない）。
//!
//! # `clipboard` は可視ラベル・trigger の 2 状態表示で構成する
//!
//! `section_heading_split.rs` と同型の合成（`root` → `label` + `control`
//! （`input` readonly + `trigger`（`indicator` idle/copied の 2 変種）））。
//! 実際のクリップボード書き込みはクライアント配線層の責務であり、本 Demo
//! は idle 状態の静的表示のみを描画する。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ボタンは `button::button` の既定 `type="button"` のまま送信先を
//! 持たない（`docs/policy/intentional-non-adoption.md` §3.25：バリデーション・
//! 送信処理は UI コンポーネント層の責務外）。リンクは使わない。文言はすべて
//! 独自の架空の日本語ダミー（実企業名・実クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::button_group;
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::status::{self, StatusProps};
use fandhe_frontend_pre_styled_ui::{Orientation, Size};

/// 自作の単純な幾何アイコン（`page_heading_actions.rs::geo_icon` と同型。
/// モジュール doc「アイコンは自作の単純幾何図形」参照）。開いた線分
/// パスのため `fill="none"` + `stroke="currentColor"` でアウトライン描画に
/// する。装飾用途のみのため `IconProps::default()`（`label: None`、
/// `aria-hidden`）を使う。
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

/// メタ行 1 項目（装飾アイコン + テキスト）。
fn meta_item(path_d: &'static str, label: &'static str) -> Node {
    div(
        vec![("data-blocks-page-heading-meta-meta-item", "")],
        vec![geo_icon(path_d), text(label)],
    )
}

/// 三点メニュー（無 JS のため閉じた状態で固定する。モジュール doc
/// 「メニューは無 JS のため閉じた状態で固定する」節参照）。
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
            menu::item("duplicate", false, false, vec![], vec![text("複製する")]),
            menu::separator(vec![], vec![]),
            menu::item(
                "archive",
                false,
                false,
                vec![],
                vec![text("アーカイブする")],
            ),
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

/// A: 代表構成（R1128）。状態 `status` + メタ行 3 項目 + `button_group` +
/// 主操作 + 三点メニュー。
fn instance_a() -> Node {
    let heading_group = div(
        vec![("data-blocks-page-heading-meta-heading-group", "")],
        vec![
            heading(
                HeadingLevel::H1,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("四半期レビュー資料")],
            ),
            div(
                vec![("data-blocks-page-heading-meta-meta-row", "")],
                vec![
                    status::root(
                        &StatusProps {
                            palette: ColorPalette::Success,
                            ..StatusProps::default()
                        },
                        vec![],
                        vec![status::indicator(vec![]), text("公開中")],
                    ),
                    meta_item("M12 2C8 2 5 5 5 9c0 5 7 13 7 13s7-8 7-13c0-4-3-7-7-7z", "本社 3F 会議室"),
                    meta_item(
                        "M7 3v3m10-3v3M4 9h16M5 6h14a1 1 0 011 1v12a1 1 0 01-1 1H5a1 1 0 01-1-1V7a1 1 0 011-1z",
                        "2026-10-01",
                    ),
                    meta_item(
                        "M12 12a4 4 0 100-8 4 4 0 000 8zm-7 8a7 7 0 0114 0",
                        "担当: 藤巻",
                    ),
                ],
            ),
        ],
    );
    let actions = div(
        vec![("data-blocks-page-heading-meta-actions", "")],
        vec![
            button_group::root(
                Orientation::Horizontal,
                "見出しの操作",
                vec![],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("編集")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("複製")],
                    ),
                ],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("公開する")]),
            overflow_menu("blocks-page-heading-meta-menu-a"),
        ],
    );
    div(
        vec![
            ("data-blocks-page-heading-meta-instance", ""),
            ("data-blocks-page-heading-meta-variant", "a"),
        ],
        vec![div(
            vec![("data-blocks-page-heading-meta-header", "")],
            vec![heading_group, actions],
        )],
    )
}

/// B: 常時パンくず付き（R1132）。状態は `badge` で示す。
fn instance_b() -> Node {
    let breadcrumb_row = breadcrumb::root(
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
                    vec![breadcrumb::current_link(vec![], vec![text("下書き記事")])],
                ),
            ],
        )],
    );
    let heading_group = div(
        vec![("data-blocks-page-heading-meta-heading-group", "")],
        vec![
            div(
                vec![("data-blocks-page-heading-meta-title-row", "")],
                vec![
                    heading(
                        HeadingLevel::H1,
                        &HeadingProps {
                            size: HeadingSize::Xl2,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("新機能の紹介記事")],
                    ),
                    badge::badge(
                        &BadgeProps {
                            palette: ColorPalette::Warning,
                            ..BadgeProps::default()
                        },
                        vec![],
                        vec![text("下書き")],
                    ),
                ],
            ),
            div(
                vec![("data-blocks-page-heading-meta-meta-row", "")],
                vec![
                    meta_item(
                        "M7 3v3m10-3v3M4 9h16M5 6h14a1 1 0 011 1v12a1 1 0 01-1 1H5a1 1 0 01-1-1V7a1 1 0 011-1z",
                        "更新: 2026-09-20",
                    ),
                    meta_item(
                        "M12 12a4 4 0 100-8 4 4 0 000 8zm-7 8a7 7 0 0114 0",
                        "担当: 高橋",
                    ),
                ],
            ),
        ],
    );
    let actions = div(
        vec![("data-blocks-page-heading-meta-actions", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("プレビュー")],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("公開する")]),
            overflow_menu("blocks-page-heading-meta-menu-b"),
        ],
    );
    div(
        vec![
            ("data-blocks-page-heading-meta-instance", ""),
            ("data-blocks-page-heading-meta-variant", "b"),
        ],
        vec![
            div(
                vec![("data-blocks-page-heading-meta-top-row", "")],
                vec![breadcrumb_row],
            ),
            div(
                vec![("data-blocks-page-heading-meta-header", "")],
                vec![heading_group, actions],
            ),
        ],
    )
}

/// C: 上段ラベル + バッジ + 共有 URL（R0187）。バッジは上段（`top-row`）に
/// ラベルと並べて置く（モジュール doc 例 C 参照。見出しと同じ `title-row`
/// には置かない）。
fn instance_c() -> Node {
    let heading_group = div(
        vec![("data-blocks-page-heading-meta-heading-group", "")],
        vec![
            heading(
                HeadingLevel::H1,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("Fandhe 移行プロジェクト")],
            ),
            div(
                vec![("data-blocks-page-heading-meta-meta-row", "")],
                vec![
                    meta_item(
                        "M7 3v3m10-3v3M4 9h16M5 6h14a1 1 0 011 1v12a1 1 0 01-1 1H5a1 1 0 01-1-1V7a1 1 0 011-1z",
                        "開始: 2026-08-01",
                    ),
                    meta_item(
                        "M12 12a4 4 0 100-8 4 4 0 000 8zm-7 8a7 7 0 0114 0",
                        "担当: 吉原",
                    ),
                ],
            ),
        ],
    );
    const SHARE_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
    const SHARE_INPUT_ID: &str = "blocks-page-heading-meta-share-url";
    let share = clipboard::root(
        SHARE_URL,
        false,
        vec![("data-blocks-page-heading-meta-share", "")],
        vec![
            clipboard::label(false, Some(SHARE_INPUT_ID), vec![], vec![text("共有 URL")]),
            clipboard::control(
                false,
                vec![],
                vec![
                    clipboard::input(SHARE_URL, false, vec![("id", SHARE_INPUT_ID)]),
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
    let actions = div(
        vec![("data-blocks-page-heading-meta-actions", "")],
        vec![
            share,
            button::button(&ButtonProps::default(), vec![], vec![text("設定を開く")]),
        ],
    );
    div(
        vec![
            ("data-blocks-page-heading-meta-instance", ""),
            ("data-blocks-page-heading-meta-variant", "c"),
        ],
        vec![
            div(
                vec![("data-blocks-page-heading-meta-top-row", "")],
                vec![
                    span(
                        vec![("data-blocks-page-heading-meta-eyebrow", "")],
                        vec![text("プロジェクト")],
                    ),
                    badge::badge(
                        &BadgeProps {
                            palette: ColorPalette::Accent,
                            ..BadgeProps::default()
                        },
                        vec![],
                        vec![text("ベータ")],
                    ),
                ],
            ),
            div(
                vec![("data-blocks-page-heading-meta-header", "")],
                vec![heading_group, actions],
            ),
        ],
    )
}

/// `page-heading-meta` の Demo 本体（3 インスタンスを縦積みで並記する。
/// 呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-page-heading-meta-layout")],
        vec![instance_a(), instance_b(), instance_c()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/page-heading-meta/",
    title: "page-heading-meta",
    category: BlockCategory::PageHeading,
    rust_source: "crates/docs-site/src/blocks/application/page_heading/page_heading_meta.rs",
    demo_class: "blocks-page-heading-meta",
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
            label: "Status",
            path: "/themes/status/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Button Group",
            path: "/themes/button-group/",
        },
        Part {
            label: "Breadcrumb",
            path: "/themes/breadcrumb/",
        },
        Part {
            label: "Clipboard",
            path: "/themes/clipboard/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `page_heading_meta` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節と同型で、本ファイル内 private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-page-heading-meta-*` と
/// `[data-blocks-page-heading-meta-*]` のみを用いる。ルート class を
/// `demo_class` と別名にする（`page_heading_actions` 等と同じ Bugbot 教訓の
/// 回避）。
const LAYOUT_CSS: &str = "\
.blocks-page-heading-meta-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
[data-blocks-page-heading-meta-instance] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-page-heading-meta-top-row] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-page-heading-meta-eyebrow] {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
[data-blocks-page-heading-meta-header] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-page-heading-meta-heading-group] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  min-width: 0;\n}\n\
[data-blocks-page-heading-meta-title-row] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-page-heading-meta-meta-row] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2) var(--fandhe-space-4);\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
[data-blocks-page-heading-meta-meta-item] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n}\n\
[data-blocks-page-heading-meta-actions] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
@media (min-width: 40rem) {\n  [data-blocks-page-heading-meta-header] {\n    flex-direction: row;\n    justify-content: space-between;\n    align-items: flex-start;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（heading/badge/status/icon/button/button-group/
    /// breadcrumb/clipboard/menu）の anatomy をすべて実際に出力していること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"badge\"",
            "data-scope=\"status\"",
            "data-scope=\"icon\"",
            "data-scope=\"button\"",
            "data-scope=\"button-group\"",
            "data-scope=\"breadcrumb\"",
            "data-scope=\"clipboard\"",
            "data-scope=\"menu\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// インスタンス数はちょうど 3 件（A〜C）。
    #[test]
    fn demo_instance_count() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-page-heading-meta-instance")
                .count(),
            3
        );
    }

    /// メニューはちょうど 2 件（A・B）、いずれも閉じた状態
    /// （`aria-expanded="false"`）で固定し、trigger の `aria-controls` が
    /// content の id と一致する。
    #[test]
    fn menus_are_static_closed() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"aria-expanded="false""#).count(), 2);
        for suffix in ["a", "b"] {
            let id = format!("blocks-page-heading-meta-menu-{suffix}");
            assert!(html.contains(&format!(r#"aria-controls="{id}""#)));
            assert!(html.contains(&format!(r#"id="{id}""#)));
        }
    }

    /// `<form>`・`href="#"`・`data:` URI・`type="submit"` を出力しない
    /// （`crate::blocks` モジュール doc）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in ["<form", "type=\"submit\"", "href=\"#\"", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// アイコンのみのトリガー（三点メニュー）は `aria-label` を持つ。
    #[test]
    fn icon_only_controls_have_aria_label() {
        let html = render(&demo());
        assert!(html.contains(r#"aria-label="その他の操作""#));
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
        assert!(html.contains("class=\"blocks-page-heading-meta-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-page-heading-meta-layout");
    }
}
