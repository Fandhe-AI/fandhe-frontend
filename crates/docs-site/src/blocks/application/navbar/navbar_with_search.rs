//! `navbar-with-search` block（イシュー #2929。親トラッキング #2892
//! 「Blocks アプリケーション A（phase:3）」配下）。対応表 ID R0162（代表）を
//! 軸に、集約元 6 件（R1090/R1091/R1094/R0161/R0162/R0580）の差分を、本
//! Demo の 3 レイアウト並記（+ 狭幅実演用の `narrow` インスタンス、後述）と
//! 原稿の「集約元との差分メモ」節（取得手段・ファイル名は記載しない、
//! [`super::navbar_app_links`] と同じライセンス上の転記制限）で読み取れる
//! ようにする。
//!
//! # 構成（検索欄付きの 1 段アプリナビバー）
//!
//! [`super::navbar_app_links`] の「リンク群 + アクション」に対し、本 block
//! は検索欄の置き方 3 通りを並記する。
//!
//! | variant | 構成 | 対応 ID |
//! |---|---|---|
//! | `links-end` | ロゴ → リンク群（navigation-menu）→ 検索欄 → 通知 → アバターメニュー | R0162（代表）+ R1090/R1091（配色違いはトークンで吸収） |
//! | `center-search` | ロゴ → 検索欄（中央寄せ、リンクなし）→ 通知 → アバターメニュー | R0161/R0580 |
//! | `grid-12` | 12 列グリッドでロゴ/検索欄/操作を割り付け（リンクなし） | R1094 |
//!
//! # 使用部品
//!
//! `input-group` / `input` / `navigation-menu` / `button` / `avatar` /
//! `menu` / `icon` の 7 部品に加え、検索欄のアクセシブルラベル付けに
//! `field`（`visually_hidden` と組み合わせる、[`super::super::hero`] の
//! `hero_search` と同型）を使う（[`BLOCK`] の `parts` に一致させる契約）。
//! `links-end` レイアウトの 2 インスタンス（`links-end`/`narrow`）のみ
//! `navigation-menu` を持つ。
//!
//! # 検索欄は狭幅でアイコンボタンへ縮む
//!
//! `< 48rem`（Demo 枠基準の container query、[`super::navbar_app_links`] と
//! 同じ判断）では検索欄ラッパーを隠し、代わりに検索アイコンボタン
//! （無 JS のため `disabled: true` 固定）を表示する。`links-end` のナビは
//! 狭幅で折り返すのみとし、ハンバーガー化はしない（使用部品 7 件の契約外
//! のため collapsible は持ち込まない）。狭幅は無 JS のため検索アイコン
//! ボタンだけでは実際に検索欄を開けず、主要機能を Demo 上で確認できない
//! （イシュー #2929 PR レビュー指摘）。このため `narrow` インスタンスのみ
//! [`super::navbar_app_links`] と同型の `data-blocks-navbar-with-search-
//! frame="narrow"` を付けて Demo 枠を強制的に `< 48rem` に固定しつつ、検索
//! 欄ラッパーを常時表示・検索アイコンボタンを非表示にするオーバーライドを
//! [`LAYOUT_CSS`] に持つ（狭幅でも検索欄そのものが操作可能であることを示す）。
//!
//! # id と ARIA の一意性
//!
//! 検索欄の `id`（`FieldProps::id`）とアバターメニューの `content_id` は
//! variant ごとに `blocks-navbar-with-search-query-{variant}` /
//! `blocks-navbar-with-search-profile-menu-{variant}` と一意にする
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` 契約）。
//!
//! # 無 JS のため全アクションを disabled 固定
//!
//! 検索トグル・通知・アバターメニュー trigger はいずれも `disabled: true`
//! で押しても何も起きないことを明示し、[`LAYOUT_CSS`] の
//! `[data-disabled]` 複合セレクタで `opacity: 1; cursor: default;` に
//! 中和して通常状態と同じ見た目に保つ（[`super::navbar_app_links`] と
//! 同じ判断）。
//!
//! # `<form>`/`href="#"`/`data:` を持たない
//!
//! `crate::blocks` モジュール doc「セキュリティ不変条件」節に従う静的な
//! 合成例。`href` は自リポジトリ・自組織の実在 URL に限る。検索欄は
//! `<form>` を持たず送信先も持たない。文言はすべて架空の日本語。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, header, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc 参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// 実在の自組織 URL。
const ORG: &str = "https://github.com/Fandhe-AI";

/// 装飾用の自作幾何アイコン（実在ブランドのロゴを模さない、`label: None`）。
fn geo_icon(d: &'static str) -> Node {
    icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", d)], vec![])],
    )
}

/// 検索アイコン（円 + 柄の 1 path、[`super::super::hero::hero_search`] と
/// 同型の自作幾何アイコン）。`geo_icon`（`fill="currentColor"` 固定）は
/// 塗りつぶし図形専用のため、輪郭のみで構成される虫眼鏡には使えない
/// （`fill="currentColor"` のまま柄を描くと直線はゼロ面積で消え、円弧は
/// 開始点への暗黙クローズで塗りつぶし円になってしまう）。`hero_search` と
/// 同じく `fill="none"` + `stroke="currentColor"` で明示的に上書きする。
fn search_icon() -> Node {
    icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zm9 17-5.2-5.2"),
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

/// ベル（通知）の幾何アイコン。
fn bell_icon() -> Node {
    geo_icon("M12 3a5 5 0 0 0-5 5v3l-2 4h14l-2-4V8a5 5 0 0 0-5-5zM10 18a2 2 0 0 0 4 0h-4z")
}

/// ロゴ（幾何図形 + ブランド名テキスト、リンクにしない）。
fn logo() -> Node {
    span(
        vec![("data-blocks-navbar-with-search-logo", "")],
        vec![
            geo_icon("M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z"),
            span(vec![], vec![text("Fandhe Console")]),
        ],
    )
}

/// メインナビ本体（`links-end` のみが持つ）。
fn nav() -> Node {
    let props = NavigationMenuProps::default();
    navigation_menu::root(
        &props,
        "メインナビゲーション",
        vec![("data-blocks-navbar-with-search-nav", "")],
        vec![navigation_menu::list(
            &props,
            vec![],
            vec![
                navigation_menu::item(
                    navigation_menu::OpenState::Closed,
                    false,
                    &props,
                    "dashboard",
                    vec![],
                    vec![navigation_menu::link(
                        "./",
                        true,
                        vec![],
                        vec![text("ダッシュボード")],
                    )],
                ),
                navigation_menu::item(
                    navigation_menu::OpenState::Closed,
                    false,
                    &props,
                    "projects",
                    vec![],
                    vec![navigation_menu::link(
                        REPO,
                        false,
                        vec![],
                        vec![text("プロジェクト")],
                    )],
                ),
                navigation_menu::item(
                    navigation_menu::OpenState::Closed,
                    false,
                    &props,
                    "reports",
                    vec![],
                    vec![navigation_menu::link(
                        ORG,
                        false,
                        vec![],
                        vec![text("レポート")],
                    )],
                ),
            ],
        )],
    )
}

/// 検索欄一式（可視ラベルの代わりに `visually_hidden` + `<label for>`、
/// [`super::super::hero::hero_search::search_group`] と同型）。`variant`
/// は `id` の suffix（モジュール doc「id と ARIA の一意性」節）。
fn search_group(variant: &'static str) -> Node {
    let field_id = format!("blocks-navbar-with-search-query-{variant}");
    let query_field = FieldProps {
        id: &field_id,
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
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &query_field,
        vec![("data-blocks-navbar-with-search-field", "")],
        vec![
            visually_hidden::root(
                vec![],
                vec![field::label(&query_field, vec![], vec![text("検索")])],
            ),
            input_group::root(
                &group_props,
                vec![("data-blocks-navbar-with-search-group", "")],
                vec![
                    input_group::addon(
                        InputGroupAlign::InlineStart,
                        &group_props,
                        vec![],
                        vec![search_icon()],
                    ),
                    input::input(
                        &InputProps::default(),
                        &query_field,
                        vec![
                            ("type", "search"),
                            ("autocomplete", "off"),
                            ("placeholder", "検索"),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 検索アイコンボタン（狭幅専用、無 JS のため `disabled: true` 固定）。
fn search_toggle() -> Node {
    button::icon_button(
        &ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        },
        "検索を開く",
        vec![("data-blocks-navbar-with-search-search-toggle", "")],
        vec![search_icon()],
    )
}

/// 通知ボタン（無 JS のため `disabled: true` 固定、アクセシブルネーム付き）。
fn notification_button() -> Node {
    button::icon_button(
        &ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        },
        "通知を表示",
        vec![("data-blocks-navbar-with-search-notify", "")],
        vec![bell_icon()],
    )
}

/// アバターメニュー（`menu::trigger` を `disabled: true` 固定にし、中に
/// avatar を入れる。`content_id` は variant ごとに一意にする）。
fn profile_menu(variant: &str) -> Node {
    let content_id = format!("blocks-navbar-with-search-profile-menu-{variant}");
    let trigger = menu::trigger(
        OpenState::Closed,
        true,
        Some(content_id.as_str()),
        vec![
            ("aria-label", "アカウントメニューを開く"),
            ("data-blocks-navbar-with-search-profile-trigger", ""),
        ],
        vec![avatar::root(
            &AvatarProps::default(),
            vec![],
            vec![avatar::fallback(
                ImageStatus::Error,
                vec![],
                vec![text("YK")],
            )],
        )],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item("account", false, false, vec![], vec![text("アカウント")]),
            menu::item("settings", false, false, vec![], vec![text("設定")]),
            menu::separator(vec![], vec![]),
            menu::item("logout", false, false, vec![], vec![text("ログアウト")]),
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

/// 1 variant 分のナビバー本体を組み立てる。`with_nav` はリンク群の有無、
/// `layout` は [`LAYOUT_CSS`] が分岐に使う `data-*` 値、`narrow` は
/// [`super::navbar_app_links`] と同型の「Demo 枠を `< 48rem` に固定する」
/// フラグ（閲覧者の画面幅に関係なく container query を狭幅側に倒し、検索欄
/// が実際に使える主要機能であることを Demo 上で示す。モジュール doc
/// 「検索欄は狭幅でアイコンボタンへ縮む」節の是正）。
fn bar(variant: &'static str, layout: &'static str, with_nav: bool, narrow: bool) -> Node {
    let mut children: Vec<Node> = vec![logo()];
    if with_nav {
        children.push(div(
            vec![("data-blocks-navbar-with-search-nav-wrap", "")],
            vec![nav()],
        ));
    }
    children.push(div(
        vec![("data-blocks-navbar-with-search-search-wrap", "")],
        vec![search_group(variant)],
    ));
    children.push(div(
        vec![("data-blocks-navbar-with-search-actions", "")],
        vec![
            search_toggle(),
            notification_button(),
            profile_menu(variant),
        ],
    ));

    let mut frame_attrs = vec![("data-blocks-navbar-with-search-shell", "")];
    if narrow {
        frame_attrs.push(("data-blocks-navbar-with-search-frame", "narrow"));
    }

    div(
        frame_attrs,
        vec![header(
            vec![
                ("data-blocks-navbar-with-search-root", ""),
                ("data-blocks-navbar-with-search-variant", variant),
                ("data-blocks-navbar-with-search-layout", layout),
            ],
            children,
        )],
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &'static str) -> Node {
    p(
        vec![("class", "blocks-navbar-with-search-caption")],
        vec![text(label)],
    )
}

/// `navbar-with-search` の Demo 本体。4 variant を縦に並記する純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-navbar-with-search-stack")],
        vec![
            caption("リンク群の右に検索欄を置く（links-end）"),
            bar("links-end", "links-end", true, false),
            caption("リンクなし・検索欄を中央に置く（center-search）"),
            bar("center-search", "center-search", false, false),
            caption("12 列グリッドでロゴ / 検索欄 / 操作を割り付ける（grid-12）"),
            bar("grid-12", "grid-12", false, false),
            caption("狭幅（< 48rem）でも検索欄が操作可能であること（narrow）"),
            bar("narrow", "links-end", true, true),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/navbar-with-search/",
    title: "navbar-with-search",
    category: BlockCategory::Navbar,
    rust_source: "crates/docs-site/src/blocks/application/navbar/navbar_with_search.rs",
    demo_class: "blocks-navbar-with-search",
    parts: &[
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Navigation Menu",
            path: "/themes/navigation-menu/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `navbar_with_search` 固有のレイアウト規則（`--fandhe-*` トークンのみ
/// 使用）。`links-end`/`center-search` の広幅（`>= 48rem`）オーバーライドは
/// `[data-blocks-navbar-with-search-actions]` の基準 `margin-inline-start:
/// auto` を `0` に打ち消す（検索欄側の auto margin と競合させると、flex の
/// 残り空間が検索欄前・検索欄後の 2 箇所へ分割され、検索欄が通知・
/// アバターから離れて見える。`grid-12` レイアウトの同名オーバーライドと
/// 同じ判断）。セレクタは `.blocks-navbar-with-search-*` /
/// `[data-blocks-navbar-with-search-*]`、および styled 部品の
/// `[data-scope][data-part]` セレクタとの複合セレクタのみを用い、他 block
/// や部品の素のセレクタへは影響させない。
const LAYOUT_CSS: &str = "\
.blocks-navbar-with-search-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-navbar-with-search-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-navbar-with-search-shell] {\n  container-type: inline-size;\n  container-name: blocks-navbar-with-search;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  overflow: hidden;\n}\n\
[data-blocks-navbar-with-search-shell][data-blocks-navbar-with-search-frame=\"narrow\"] {\n  max-inline-size: 22rem;\n}\n\
[data-blocks-navbar-with-search-shell][data-blocks-navbar-with-search-frame=\"narrow\"] [data-blocks-navbar-with-search-search-wrap] {\n  display: block;\n}\n\
[data-blocks-navbar-with-search-shell][data-blocks-navbar-with-search-frame=\"narrow\"] [data-scope=\"button\"][data-part=\"root\"][data-blocks-navbar-with-search-search-toggle] {\n  display: none;\n}\n\
[data-blocks-navbar-with-search-root] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2) var(--fandhe-space-4);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  background: var(--fandhe-color-bg-subtle);\n}\n\
[data-blocks-navbar-with-search-logo] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  white-space: nowrap;\n}\n\
[data-blocks-navbar-with-search-nav-wrap] {\n  flex-basis: 100%;\n}\n\
[data-blocks-navbar-with-search-search-wrap] {\n  display: none;\n}\n\
[data-blocks-navbar-with-search-actions] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  margin-inline-start: auto;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-navbar-with-search-search-toggle][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-navbar-with-search-notify][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"menu\"][data-part=\"trigger\"][data-blocks-navbar-with-search-profile-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
@container blocks-navbar-with-search (min-width: 48rem) {\n  \
[data-blocks-navbar-with-search-root][data-blocks-navbar-with-search-layout=\"links-end\"] [data-blocks-navbar-with-search-nav-wrap] {\n    flex-basis: auto;\n  }\n  \
[data-blocks-navbar-with-search-search-wrap] {\n    display: block;\n  }\n  \
[data-scope=\"button\"][data-part=\"root\"][data-blocks-navbar-with-search-search-toggle] {\n    display: none;\n  }\n  \
[data-blocks-navbar-with-search-root][data-blocks-navbar-with-search-layout=\"links-end\"] [data-blocks-navbar-with-search-search-wrap] {\n    flex: 0 1 20rem;\n    margin-inline-start: auto;\n  }\n  \
[data-blocks-navbar-with-search-root][data-blocks-navbar-with-search-layout=\"links-end\"] [data-blocks-navbar-with-search-actions] {\n    margin-inline-start: 0;\n  }\n  \
[data-blocks-navbar-with-search-root][data-blocks-navbar-with-search-layout=\"center-search\"] [data-blocks-navbar-with-search-search-wrap] {\n    flex: 1;\n    max-inline-size: 36rem;\n    margin-inline: auto;\n  }\n  \
[data-blocks-navbar-with-search-root][data-blocks-navbar-with-search-layout=\"center-search\"] [data-blocks-navbar-with-search-actions] {\n    margin-inline-start: 0;\n  }\n  \
[data-blocks-navbar-with-search-root][data-blocks-navbar-with-search-layout=\"grid-12\"] {\n    display: grid;\n    grid-template-columns: repeat(12, minmax(0, 1fr));\n    align-items: center;\n  }\n  \
[data-blocks-navbar-with-search-root][data-blocks-navbar-with-search-layout=\"grid-12\"] [data-blocks-navbar-with-search-logo] {\n    grid-column: 1 / span 3;\n  }\n  \
[data-blocks-navbar-with-search-root][data-blocks-navbar-with-search-layout=\"grid-12\"] [data-blocks-navbar-with-search-search-wrap] {\n    grid-column: 4 / span 6;\n    max-inline-size: none;\n    margin-inline: 0;\n  }\n  \
[data-blocks-navbar-with-search-root][data-blocks-navbar-with-search-layout=\"grid-12\"] [data-blocks-navbar-with-search-actions] {\n    grid-column: 10 / span 3;\n    justify-self: end;\n    margin-inline-start: 0;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, ORG, REPO};
    use fandhe_frontend_core::render;

    /// 8 部品の `data-scope` が揃い、`type="button"` があり、`<form>`・
    /// `href="#"`・`data:` src を持たないこと（`input` パーツは headless
    /// `field::input` へ委譲するため `data-scope="field"` として現れる、
    /// `crates/pre-styled-ui/src/input.rs` のドキュメント参照）。
    #[test]
    fn demo_composes_expected_parts_and_has_no_form() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"input-group\"",
            "data-scope=\"field\"",
            "data-scope=\"navigation-menu\"",
            "data-scope=\"button\"",
            "data-scope=\"avatar\"",
            "data-scope=\"menu\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"data-scope="field" data-part="input""#));
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 4 variant がそれぞれ 1 回ずつ描画され、caption が 4 件あること。
    #[test]
    fn demo_renders_all_four_variants() {
        let html = render(&demo());
        for variant in ["links-end", "center-search", "grid-12", "narrow"] {
            assert!(
                html.contains(&format!(
                    "data-blocks-navbar-with-search-variant=\"{variant}\""
                )),
                "variant {variant} should render"
            );
        }
        assert_eq!(html.matches("blocks-navbar-with-search-caption").count(), 4);
        assert_eq!(
            html.matches("data-blocks-navbar-with-search-shell").count(),
            4
        );
    }

    /// `links-end`/`narrow` の 2 variant が `navigation-menu` を持つこと。
    /// `data-scope` は `root`/`list`/`item`×3/`link`×3 の 8 パーツすべてに
    /// 付くため、1 nav あたり 8 件（root/list/item×3/link×3、アイテム数は
    /// モジュール doc の `nav()` 定義に従う）× 2 variant = 16 件が正しい
    /// 期待値である。
    #[test]
    fn links_end_and_narrow_have_navigation_menu() {
        let html = render(&demo());
        assert_eq!(html.matches("data-scope=\"navigation-menu\"").count(), 16);
        assert_eq!(html.matches("メインナビゲーション").count(), 2);
    }

    /// 検索欄の `id`/`for` が 4 variant すべてで一意であり、`type="search"`
    /// が 4 件あること。headless `field::label`/`field::input` は
    /// `FieldProps::id` からコントロール id を `"{id}-control"` として
    /// 決定的に派生させる（`crates/headless-ui/src/field.rs` の
    /// `control_id`/`label` 参照）ため、比較対象もその派生形にする。
    #[test]
    fn search_inputs_have_unique_ids_and_labels() {
        let html = render(&demo());
        for variant in ["links-end", "center-search", "grid-12", "narrow"] {
            let control_id = format!("blocks-navbar-with-search-query-{variant}-control");
            assert_eq!(html.matches(&format!(r#"id="{control_id}""#)).count(), 1);
            assert_eq!(html.matches(&format!(r#"for="{control_id}""#)).count(), 1);
        }
        assert_eq!(html.matches(r#"type="search""#).count(), 4);
    }

    /// 通知・検索トグル・アバターメニュー trigger が押しても何も起きない
    /// よう無効化されていること。
    #[test]
    fn action_controls_are_disabled() {
        let html = render(&demo());
        for hook in [
            "data-blocks-navbar-with-search-search-toggle",
            "data-blocks-navbar-with-search-notify",
            "data-blocks-navbar-with-search-profile-trigger",
        ] {
            assert_eq!(
                html.matches(hook).count(),
                4,
                "hook {hook} should render once per variant"
            );
        }
        for pos in html.match_indices("data-blocks-navbar-with-search-profile-trigger") {
            let tag_end = html[pos.0..]
                .find('>')
                .map(|rel| pos.0 + rel)
                .expect("profile trigger tag should close");
            let tag_start = html[..pos.0]
                .rfind("<button")
                .expect("profile trigger should be a button");
            assert!(html[tag_start..tag_end].contains("data-disabled"));
        }
    }

    /// アバターメニューの `aria-controls` と `content` の `id` が variant
    /// ごとに一意に対応すること。
    #[test]
    fn profile_menus_reference_their_content() {
        let html = render(&demo());
        for variant in ["links-end", "center-search", "grid-12", "narrow"] {
            let content_id = format!("blocks-navbar-with-search-profile-menu-{variant}");
            assert!(html.contains(&format!(r#"aria-controls="{content_id}""#)));
            assert!(html.contains(&format!(r#"id="{content_id}""#)));
        }
    }

    /// `narrow` インスタンスのみ frame 属性を持ち、Demo 枠を強制的に狭幅へ
    /// 固定して検索欄を常時表示すること（イシュー #2929 PR レビュー指摘の
    /// 是正: 狭幅でも検索欄が操作可能であることを示す）。
    #[test]
    fn only_narrow_instance_has_the_frame_attribute() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-navbar-with-search-frame=\"narrow\"")
                .count(),
            1
        );
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-navbar-with-search-shell][data-blocks-navbar-with-search-frame=\"narrow\"] [data-blocks-navbar-with-search-search-wrap] {\n  display: block;\n}"
        ));
    }

    /// リンクは現在地（`./`）と実在の自リポジトリ・自組織 URL のみを
    /// 指すこと。
    #[test]
    fn nav_links_point_to_self_and_real_urls() {
        let html = render(&demo());
        assert!(html.contains(r#"href="./""#));
        assert!(html.contains(&format!(r#"href="{REPO}""#)));
        assert!(html.contains(&format!(r#"href="{ORG}""#)));
    }

    /// [`LAYOUT_CSS`] が container query・レイアウト分岐・disabled 中和を
    /// 満たすこと。
    #[test]
    fn layout_css_has_responsive_rules() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-navbar-with-search (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: repeat(12, minmax(0, 1fr));"));
        assert!(LAYOUT_CSS.contains("grid-column: 4 / span 6;"));
        assert!(LAYOUT_CSS.contains("opacity: 1;"));
        assert!(LAYOUT_CSS.contains("display: none;"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-navbar-with-search-stack\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-navbar-with-search-stack");
    }
}
