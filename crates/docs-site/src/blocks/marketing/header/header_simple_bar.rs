//! `header-simple-bar` block（イシュー #2860）。親トラッキング #2807
//! 「Blocks 目的別パーツ拡充（phase:2）」配下、対応表 ID R0982 を主参照
//! とする合成例。集約元 12 件の差分（R0572/R0573/R0159/R0985/R0986/
//! R0989/R0990/R0991/R0571 等）を 4 variant の並記で読み取れるように
//! する（詳細は `site/blocks/header-simple-bar.md` の「集約元との差分
//! メモ」節、対応表 ID のみを記す契約）。
//!
//! # 使用部品
//!
//! `navigation-menu` / `button` / `icon` / `link` の 4 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。既存の
//! [`super::header_flyout_menu`] と同じ組み合わせのため、実装はこの
//! block を雛形にした。
//!
//! # 4 variant の並記
//!
//! ナビ・アクションの配置だけが異なる 4 variant を 1 ページに縦に並べる
//! （`header_flyout_menu` の「3 形の並記」と同型）。DOM 順は固定し、CSS の
//! grid 配置・`order` だけで見た目の配置を変える。
//!
//! | variant | 配置 | アクション | 幅・装飾 |
//! |---|---|---|---|
//! | center | ロゴ左、ナビ中央 | ログイン + 登録 | 幅制限（中央寄せ） |
//! | start | ロゴ + ナビ左、アクション右 | ログイン + 登録 | 全幅、下境界線 |
//! | end | ロゴ左、ナビ + アクション右 | 登録のみ | — |
//! | logo-center | ナビ左、ロゴ中央、アクション右 | ログインのみ | 3 列 grid |
//!
//! # 静的表示（無 JS、disabled 固定）
//!
//! docs サイトは JS ハイドレーションを行わないため、登録ボタンは
//! 押しても何も起きない no-op になる。[`super::header_flyout_menu`] と
//! 同じ判断で `disabled: true`（ネイティブ `disabled` + `aria-disabled`）
//! にしてフォーカス不能・操作不能を明示し、[`LAYOUT_CSS`] で
//! `opacity: 1; cursor: default;` に中和して通常と同じ見た目に保つ。
//!
//! # ハンバーガーは開閉パネルを描画しない
//!
//! 狭い幅ではナビ・アクションを隠しハンバーガーボタンのみを表示する
//! （イシュー仕様）。開いた先のパネル自体は無 JS のため描画しないため、
//! `aria-controls` は付けない（`header_flyout_menu` のハンバーガーと同じ
//! 判断）。
//!
//! # CSS フックに data 属性を使う理由・詳細度対策
//!
//! `button::button`/`button::icon_button`/`link::root`/`icon::icon` は
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! ため、Demo 固有のスタイルフックは `data-blocks-header-simple-bar-*`
//! 属性で渡す。styled navigation-menu/button の recipe が持つ
//! `[data-scope][data-part]`（詳細度 `(0,2,0)`）・disabled state
//! （`(0,3,0)`）の base 宣言に単一属性セレクタでは詳細度で負けるため、
//! [`LAYOUT_CSS`] は `[data-scope="..."][data-part="..."][data-<hook>]`
//! の複合セレクタで上書きする（`header_flyout_menu` の「CSS 特異性」節と
//! 同じ教訓）。
//!
//! # `href` の方針・文言
//!
//! `href` は実在する自リポジトリ・組織の URL に限る（`href="#"` は
//! `linkcheck` が拒否する死リンクのため使わない）。文言は架空の日本語。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「セキュリティ不変条件」節に従い、本
//! Demo はフォーム・状態機械を持たない静的な合成例である。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, header, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::Size;

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc 参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// 実在の自組織 URL。
const ORG: &str = "https://github.com/Fandhe-AI";

/// メインナビの項目一覧（value, label, href）。
const NAV_ITEMS: &[(&str, &str, &str)] = &[
    ("product", "製品", REPO),
    ("pricing", "料金", REPO),
    ("company", "会社概要", ORG),
];

/// 装飾用の幾何図形アイコン（`label: None`、実在ブランドのロゴを模さない
/// 自作 SVG）。
fn geo_icon(size: Size, d: &str) -> Node {
    icon(
        &IconProps {
            size,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", d)], vec![])],
    )
}

/// ハンバーガー（3 本線）アイコン。
fn hamburger_icon() -> Node {
    geo_icon(Size::Md, "M3 6h18v2H3zM3 11h18v2H3zM3 16h18v2H3z")
}

/// ロゴ（幾何図形アイコン + ブランド名テキスト）。
fn logo() -> Node {
    link::root(
        REPO,
        &LinkProps::default(),
        vec![("data-blocks-header-simple-bar-logo", "")],
        vec![
            geo_icon(Size::Md, "M4 4h16v6H4zM4 14h16v6H4z"),
            span(
                vec![("class", "blocks-header-simple-bar-brand")],
                vec![text("Fandhe Frontend")],
            ),
        ],
    )
}

/// メインナビ本体（`aria-label` は variant ごとに一意にし、同一ページ内で
/// ナビランドマーク名が重ならないようにする）。
fn nav(aria_label: &str) -> Node {
    let props = NavigationMenuProps::default();
    navigation_menu::root(
        &props,
        aria_label,
        vec![("data-blocks-header-simple-bar-nav", "")],
        vec![navigation_menu::list(
            &props,
            vec![],
            NAV_ITEMS
                .iter()
                .map(|(value, label, href)| {
                    navigation_menu::item(
                        OpenState::Closed,
                        false,
                        &props,
                        value,
                        vec![],
                        vec![navigation_menu::link(
                            href,
                            false,
                            vec![],
                            vec![text(*label)],
                        )],
                    )
                })
                .collect(),
        )],
    )
}

/// アクション行（ログイン link + 登録 button の 0〜2 個を任意組み合わせで
/// 持つ）。登録ボタンは無 JS デモのため `disabled: true` 固定。
fn actions(with_login: bool, with_signup: bool) -> Node {
    let mut children = Vec::new();
    if with_login {
        children.push(link::root(
            REPO,
            &LinkProps::default(),
            vec![],
            vec![text("ログイン")],
        ));
    }
    if with_signup {
        children.push(button::button(
            &ButtonProps {
                disabled: true,
                ..ButtonProps::default()
            },
            vec![("data-blocks-header-simple-bar-cta", "")],
            vec![text("登録")],
        ));
    }
    div(
        vec![("data-blocks-header-simple-bar-actions", "")],
        children,
    )
}

/// ハンバーガーボタン（狭い幅専用、押しても何も起きないため
/// `disabled: true` 固定。開閉パネルを描画しないため `aria-controls` は
/// 付けない）。
fn hamburger() -> Node {
    button::icon_button(
        &ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        },
        "Open main menu",
        vec![("data-blocks-header-simple-bar-toggle", "")],
        vec![hamburger_icon()],
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &str) -> Node {
    p(
        vec![("class", "blocks-header-simple-bar-caption")],
        vec![text(label)],
    )
}

/// 1 本のバー（DOM 順は常に ロゴ・ナビ・アクション・ハンバーガー固定。
/// 見た目の配置差は [`LAYOUT_CSS`] の `data-blocks-header-simple-bar-variant`
/// セレクタが担う）。
fn bar(variant: &str, aria_label: &str, with_login: bool, with_signup: bool) -> Node {
    header(
        vec![
            ("class", "blocks-header-simple-bar-layout"),
            ("data-blocks-header-simple-bar-root", ""),
            ("data-blocks-header-simple-bar-variant", variant),
        ],
        vec![
            logo(),
            nav(aria_label),
            actions(with_login, with_signup),
            hamburger(),
        ],
    )
}

/// `header-simple-bar` の Demo 本体。4 variant を縦に並記する純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-header-simple-bar-stack")],
        vec![
            caption("中央寄せ・幅制限"),
            bar("center", "メイン（中央寄せ）", true, true),
            caption("左寄せ・全幅・下境界線"),
            bar("start", "メイン（左寄せ）", true, true),
            caption("右寄せ・登録のみ"),
            bar("end", "メイン（右寄せ）", false, true),
            caption("ロゴ中央・ログインのみ"),
            bar("logo-center", "メイン（ロゴ中央）", true, false),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/header-simple-bar/",
    title: "header-simple-bar",
    category: BlockCategory::Header,
    rust_source: "crates/docs-site/src/blocks/marketing/header/header_simple_bar.rs",
    demo_class: "blocks-header-simple-bar",
    parts: &[
        Part {
            label: "Navigation Menu",
            path: "/themes/navigation-menu/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `header_simple_bar` 固有のレイアウト規則（`--fandhe-*` トークンのみ
/// 使用）。セレクタは `.blocks-header-simple-bar-*` と
/// `[data-blocks-header-simple-bar-*]`、および styled navigation-menu /
/// styled button の `[data-scope]`/`[data-part]` セレクタとの複合セレクタ
/// のみを用い、他 block や部品の素のセレクタへは影響させない。
const LAYOUT_CSS: &str = "\
.blocks-header-simple-bar-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-header-simple-bar-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-header-simple-bar-layout {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  background: var(--fandhe-color-bg);\n}\n\
.blocks-header-simple-bar-brand {\n  font-weight: var(--fandhe-font-font-weight-medium);\n  white-space: nowrap;\n}\n\
[data-scope=\"navigation-menu\"][data-part=\"root\"][data-blocks-header-simple-bar-nav] {\n  display: none;\n}\n\
[data-blocks-header-simple-bar-actions] {\n  display: none;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-header-simple-bar-toggle] {\n  display: inline-flex;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-header-simple-bar-toggle][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-header-simple-bar-cta][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
@media (min-width: 48rem) {\n  \
[data-scope=\"navigation-menu\"][data-part=\"root\"][data-blocks-header-simple-bar-nav] {\n    display: block;\n  }\n  \
[data-blocks-header-simple-bar-actions] {\n    display: flex;\n  }\n  \
[data-scope=\"button\"][data-part=\"root\"][data-blocks-header-simple-bar-toggle] {\n    display: none;\n  }\n  \
[data-blocks-header-simple-bar-root][data-blocks-header-simple-bar-variant=\"start\"] {\n    border-block-end: 1px solid var(--fandhe-color-border);\n  }\n  \
[data-blocks-header-simple-bar-root][data-blocks-header-simple-bar-variant=\"start\"] [data-scope=\"navigation-menu\"][data-part=\"root\"][data-blocks-header-simple-bar-nav] {\n    margin-inline-end: auto;\n  }\n  \
[data-blocks-header-simple-bar-root][data-blocks-header-simple-bar-variant=\"end\"] [data-scope=\"navigation-menu\"][data-part=\"root\"][data-blocks-header-simple-bar-nav] {\n    margin-inline-start: auto;\n  }\n  \
[data-blocks-header-simple-bar-root][data-blocks-header-simple-bar-variant=\"center\"] {\n    max-inline-size: 64rem;\n    margin-inline: auto;\n  }\n  \
[data-blocks-header-simple-bar-root][data-blocks-header-simple-bar-variant=\"center\"] [data-scope=\"navigation-menu\"][data-part=\"root\"][data-blocks-header-simple-bar-nav] {\n    flex: 1;\n    text-align: center;\n  }\n  \
[data-blocks-header-simple-bar-root][data-blocks-header-simple-bar-variant=\"logo-center\"] {\n    display: grid;\n    grid-template-columns: 1fr auto 1fr;\n    align-items: center;\n  }\n  \
[data-blocks-header-simple-bar-root][data-blocks-header-simple-bar-variant=\"logo-center\"] [data-blocks-header-simple-bar-logo] {\n    justify-self: center;\n    order: 2;\n  }\n  \
[data-blocks-header-simple-bar-root][data-blocks-header-simple-bar-variant=\"logo-center\"] [data-scope=\"navigation-menu\"][data-part=\"root\"][data-blocks-header-simple-bar-nav] {\n    justify-self: start;\n    order: 1;\n  }\n  \
[data-blocks-header-simple-bar-root][data-blocks-header-simple-bar-variant=\"logo-center\"] [data-blocks-header-simple-bar-actions] {\n    justify-self: end;\n    order: 3;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 4 部品を持ち、非対話制約（`<form>` なし・
    /// `href="#"` なし・`data:` src なし）を満たすこと。
    #[test]
    fn demo_composes_expected_parts_and_has_no_form() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"navigation-menu\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 4 variant がそれぞれ 1 回ずつ出ること。
    #[test]
    fn demo_renders_all_four_variants() {
        let html = render(&demo());
        for variant in ["center", "start", "end", "logo-center"] {
            assert!(
                html.contains(&format!(
                    "data-blocks-header-simple-bar-variant=\"{variant}\""
                )),
                "variant {variant} should render, html={html}"
            );
        }
        assert_eq!(html.matches("blocks-header-simple-bar-caption").count(), 4);
    }

    /// ハンバーガーが 4 個で、`aria-label` を持ち `aria-controls` を持たない
    /// こと（開閉パネル非描画）。
    #[test]
    fn hamburgers_have_label_but_no_aria_controls() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-header-simple-bar-toggle").count(),
            4
        );
        assert_eq!(html.matches(r#"aria-label="Open main menu""#).count(), 4);
        assert!(!html.contains("aria-controls"));
    }

    /// 登録ボタンが disabled + aria-disabled で描画されること。
    #[test]
    fn signup_button_is_disabled() {
        let html = render(&demo());
        let attr_start = html
            .find("data-blocks-header-simple-bar-cta")
            .expect("cta button should render");
        let tag_start = html[..attr_start]
            .rfind("<button")
            .expect("cta opening tag should precede its attribute");
        let tag_end = html[tag_start..]
            .find('>')
            .map(|rel| tag_start + rel)
            .expect("cta opening tag should close");
        let cta_tag = &html[tag_start..tag_end];
        assert!(cta_tag.contains("disabled"), "cta_tag={cta_tag}");
        assert!(
            cta_tag.contains(r#"aria-disabled="true""#),
            "cta_tag={cta_tag}"
        );
    }

    /// [`LAYOUT_CSS`] が 48rem 境界・ハンバーガー切り替え・disabled 中和・
    /// ロゴ中央 3 列 grid を持つこと。
    #[test]
    fn layout_css_has_responsive_rules() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"button\"][data-part=\"root\"][data-blocks-header-simple-bar-toggle] {\n    display: none;\n  }"
        ));
        assert!(LAYOUT_CSS.contains("opacity: 1;"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: 1fr auto 1fr;"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-header-simple-bar-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-header-simple-bar-layout");
    }

    /// ナビの `aria-label` が variant ごとに一意であること（4 種とも
    /// 異なる文言で 1 回ずつ現れる）。
    #[test]
    fn nav_aria_label_is_unique_per_variant() {
        let html = render(&demo());
        for label in [
            "メイン（中央寄せ）",
            "メイン（左寄せ）",
            "メイン（右寄せ）",
            "メイン（ロゴ中央）",
        ] {
            assert_eq!(
                html.matches(&format!("aria-label=\"{label}\"")).count(),
                1,
                "label={label} html={html}"
            );
        }
    }
}
