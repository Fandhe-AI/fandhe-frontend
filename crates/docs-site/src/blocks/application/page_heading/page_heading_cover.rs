//! `page-heading-cover` block（イシュー #2932。親トラッキング #2892
//! 配下、phase:3）。上部に横長カバー画像、その下端に重なる円形アバター、
//! 右に名前、さらに右に連絡系ボタン 2 個を配置するプロフィール見出しの
//! 合成例。Application / Page Heading カテゴリへ 2 件目の block として
//! 追加する（`page_heading_actions.rs` に続く。カテゴリはイシュー #2930 で
//! 既に卒業済みのため本ファイルは `mod` 宣言の追記のみで足りる）。
//!
//! # 使用部品
//!
//! `image` / `avatar` / `heading` / `button` / `icon` の 5 部品のみを合成
//! する（[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は追加
//! しない。
//!
//! # 狭幅ではアバター・名前・ボタンを縦積みにし、ボタンを全幅にする
//!
//! `header` は既定で縦積み（`flex-direction: column`）、`40rem` 以上で
//! 左右配置（`row` + `align-items: flex-end`）へ切り替える（他の block と
//! 同じリテラル値。テーマの breakpoint トークンは `@media` 条件式の中では
//! 解決できない）。ボタン列（`actions`）は狭幅で `flex-direction: column`
//! + `width: 100%` の全幅、`40rem` 以上で横並び + 自然幅に戻す。
//!
//! # アバターをカバー画像下端へ重ねる
//!
//! `avatar` に負の `margin-top` を与えてカバー画像の下端へ食い込ませ、
//! 背景色と同色の `box-shadow` を縁取りとして付与する（`border` は
//! avatar recipe 自身の枠線宣言と競合するため使わない）。
//!
//! # アイコンは自作の単純幾何図形
//!
//! `page_heading_actions.rs::geo_icon` と同型で、`icon::icon` へ独自の
//! `<path d="...">` を渡すのみ（実在ブランドのアイコンセットは使わない）。
//! ボタンはアイコン + 可視テキストの組で、`aria-label` を別途必要としない。
//!
//! # ダミー素材
//!
//! カバー画像は `crate::blocks::dummy_assets::BACKGROUND_SRC`、アバターは
//! 同 `AVATAR_SRC`、名前は同 `PERSON_NAMES[0]` を使う（いずれも装飾/
//! プレースホルダーのため `alt=""`。名前見出しがアクセシブル名を担う）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ボタンは `button::button` の既定 `type="button"` のまま送信先を
//! 持たない（`docs/policy/intentional-non-adoption.md` §3.25：バリデーション・
//! 送信処理は UI コンポーネント層の責務外）。文言・人名はすべて独自の架空
//! ダミー（実企業名・実クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の単純な幾何アイコン（`page_heading_actions.rs::geo_icon` と同型。
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

/// 封筒アイコン（メッセージ送信操作）。
fn mail_icon() -> Node {
    geo_icon("M4 6h16v12H4z M4 6l8 7 8-7")
}

/// 受話器アイコン（電話操作）。
fn phone_icon() -> Node {
    geo_icon(
        "M6 3h4l2 5-2.5 2a11 11 0 0 0 5 5l2-2.5 5 2v4a2 2 0 0 1-2 2A16 16 0 0 1 4 5a2 2 0 0 1 2-2z",
    )
}

/// `page-heading-cover` の Demo 本体（呼び出しごとに同一の `Node` を返す
/// 純関数）。カバー画像 → アバター重なり + 名前 + 操作ボタン 2 個の 1
/// インスタンス構成（集約元が R1129 単独のため併記なし）。
pub fn demo() -> Node {
    let cover = div(
        vec![("data-blocks-page-heading-cover-cover", "")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                ..ImageProps::new(crate::blocks::dummy_assets::BACKGROUND_SRC, "")
            },
            vec![("data-blocks-page-heading-cover-cover-image", "")],
        )],
    );
    let avatar_node = avatar::root(
        &AvatarProps {
            size: Size::Xl,
            ..AvatarProps::default()
        },
        vec![("data-blocks-page-heading-cover-avatar", "")],
        vec![avatar::image(
            ImageStatus::Loaded,
            crate::blocks::dummy_assets::AVATAR_SRC,
            "",
            vec![],
        )],
    );
    let name = heading(
        HeadingLevel::H1,
        &HeadingProps {
            size: HeadingSize::Xl,
            ..HeadingProps::default()
        },
        vec![("data-blocks-page-heading-cover-name", "")],
        vec![text(crate::blocks::dummy_assets::PERSON_NAMES[0])],
    );
    let actions = div(
        vec![("data-blocks-page-heading-cover-actions", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![mail_icon(), text("メッセージ")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![phone_icon(), text("電話")],
            ),
        ],
    );
    let header = div(
        vec![("data-blocks-page-heading-cover-header", "")],
        vec![avatar_node, name, actions],
    );
    div(
        vec![("class", "blocks-page-heading-cover-layout")],
        vec![cover, header],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/page-heading-cover/",
    title: "page-heading-cover",
    category: BlockCategory::PageHeading,
    rust_source: "crates/docs-site/src/blocks/application/page_heading/page_heading_cover.rs",
    demo_class: "blocks-page-heading-cover",
    parts: &[
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `page_heading_cover` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型で、本ファイル内 private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-page-heading-cover-*` と
/// `[data-blocks-page-heading-cover-*]` のみを用いる。ルート class は
/// `demo_class`（`blocks-page-heading-cover`）とは別名の
/// `blocks-page-heading-cover-layout` にする（`page_heading_actions` 等と
/// 同じ Bugbot 教訓の回避）。
const LAYOUT_CSS: &str = "\
.blocks-page-heading-cover-layout {\n  display: flex;\n  flex-direction: column;\n}\n\
[data-blocks-page-heading-cover-cover] {\n  height: 8rem;\n  overflow: hidden;\n  border-radius: var(--fandhe-radius-lg);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-page-heading-cover-cover-image] {\n  width: 100%;\n  height: 100%;\n}\n\
[data-blocks-page-heading-cover-header] {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-4);\n  padding: 0 var(--fandhe-space-4);\n}\n\
[data-blocks-page-heading-cover-avatar] {\n  margin-top: -2.5rem;\n  box-shadow: 0 0 0 4px var(--fandhe-color-bg-subtle);\n}\n\
[data-blocks-page-heading-cover-actions] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  width: 100%;\n}\n\
[data-blocks-page-heading-cover-actions] [data-scope=\"button\"] {\n  width: 100%;\n}\n\
@media (min-width: 40rem) {\n  [data-blocks-page-heading-cover-cover] {\n    height: 12rem;\n  }\n  [data-blocks-page-heading-cover-header] {\n    flex-direction: row;\n    align-items: flex-end;\n  }\n  [data-blocks-page-heading-cover-avatar] {\n    margin-top: -3rem;\n  }\n  [data-blocks-page-heading-cover-actions] {\n    flex-direction: row;\n    width: auto;\n    margin-inline-start: auto;\n  }\n  [data-blocks-page-heading-cover-actions] [data-scope=\"button\"] {\n    width: auto;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（image/avatar/heading/button/icon）の anatomy を
    /// すべて実際に出力していること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"image\"",
            "data-scope=\"avatar\"",
            "data-scope=\"heading\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// 操作ボタンはちょうど 2 件、いずれも既定 `type="button"`。
    #[test]
    fn demo_has_two_action_buttons() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-scope=\"button\" data-part=\"root\"")
                .count(),
            2
        );
        assert_eq!(html.matches(r#"type="button""#).count(), 2);
    }

    /// カバー画像・アバターのダミー素材 `src` を出力していること。
    #[test]
    fn demo_uses_dummy_assets() {
        let html = render(&demo());
        assert!(html.contains(crate::blocks::dummy_assets::BACKGROUND_SRC));
        assert!(html.contains(crate::blocks::dummy_assets::AVATAR_SRC));
        assert!(html.contains(crate::blocks::dummy_assets::PERSON_NAMES[0]));
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
        assert!(html.contains("class=\"blocks-page-heading-cover-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-page-heading-cover-layout");
    }
}
