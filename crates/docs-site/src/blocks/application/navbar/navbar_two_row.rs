//! `navbar-two-row` block（イシュー #2928。親トラッキング #2730 系
//! 「Blocks 目的別パーツ拡充（phase:3）」配下）。対応表 ID R1093（主参照）
//! を軸に、R1092（暗色配色のみ差）/ R0158（中央にセカンダリタブ）/
//! R0155（ハンバーガー左端の多階層・ドキュメントサイト向け 2 段版）の
//! 差分を、本 Demo の 2 variant 並記と本節の判断メモで読み取れるように
//! する（取得手段・ファイル名・内部コンポーネント識別子は記載しない、
//! `navbar_app_links` と同じライセンス上の転記制限。詳細は
//! `site/blocks/navbar-two-row.md` の「集約元との差分メモ」節）。
//!
//! # 構成（2 段構成のアプリ用ナビバー）
//!
//! 1 段目にロゴ・中央の検索欄・通知ボタン・プロフィールメニューを置き、
//! 2 段目にセカンダリナビを置く。広い幅では 2 段目を常時表示し、狭い幅
//! では [`super::navbar_app_links`] と同じ判断で 2 段目をハンバーガーの
//! 常時展開パネルへ畳む（狭幅でも到達可能にする、`header_simple_bar` と
//! 同じ確定パターン）。
//!
//! # 使用部品
//!
//! `input-group` / `input` / `navigation-menu` / `tab-nav` / `button` /
//! `avatar` / `menu` / `icon` / `collapsible` の 9 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。
//!
//! # 2 variant の並記
//!
//! | variant（`data-blocks-navbar-two-row-variant`） | 1 段目 DOM 順 | 2 段目 | 対応 ID |
//! |---|---|---|---|
//! | `pills` | ロゴ → 検索 → アクション（通知・プロフィールメニュー）→ ハンバーガー（右端） | `navigation-menu` のピル型リンク（左寄せ） | R1093（代表）+ R1092（暗色差は本節末尾の差分メモのみ） |
//! | `tabs-center` | ハンバーガー（左端）→ ロゴ → 検索 → アクション | `tab-nav` のタブ型リンク（中央寄せ） | R0158（中央タブ）+ R0155（ハンバーガー左端の多階層・ドキュメントサイト向け） |
//!
//! # 2 段目に `tabs` ではなく `tab-nav` を使う判断
//!
//! イシュー本文の部品案は `tabs` だが、無 JS の docs サイトでは
//! [`fandhe_frontend_pre_styled_ui::tabs::tabs`]（`role="tab"` + content
//! パネル必須の切替 UI）が実際には切り替わらない
//! （`crates/docs-site/src/component_specs_overlay.rs` の
//! `feature_tabs_panel` が同じ理由で実物 `tabs::tabs` を不採用にした先例）。
//! セカンダリ「ナビ」はページ遷移用のリンク集合であり、`aria-current`
//! を持つ [`fandhe_frontend_pre_styled_ui::tab_nav`] の方が意味論的にも
//! 正しいため、`tabs-center` variant では `tab-nav` を用いる。
//!
//! # 狭幅パネルは常時展開・2 段目行は広幅で非表示
//!
//! [`super::navbar_app_links`]/`header_simple_bar` と同じ確定パターンで、
//! ハンバーガーは `collapsible::trigger(OpenState::Open, disabled: true,
//! …)`（押しても何も起きないが常に到達可能）とし、
//! `collapsible::content(OpenState::Open, true, …)` の常時展開パネルへ
//! 2 段目ナビの `Node::clone()` を渡す。画面幅ではなく Demo 枠自体の幅で
//! 切り替えるため `@media` ではなく `@container`（`navbar_app_links` と
//! 同じ判断）を用い、`>= 48rem` ではパネル・ハンバーガーを非表示にし、
//! `< 48rem`（既定）では 2 段目の行自体を非表示にする（非表示側は
//! `display: none` で a11y ツリーから除外されるため、同一リンクの重複は
//! 実害を持たない）。
//!
//! # `id`/`aria-label` の一意性
//!
//! `menu` の `content_id`・ハンバーガーパネルの `id`・検索欄の `id`・
//! ナビ/タブ群の `aria-label` はいずれも variant ごとに suffix を分けて
//! 一意にする（`demo_output_has_no_dangling_aria_references_or_duplicate_ids`
//! 契約、`crate::blocks` モジュール doc 参照）。
//!
//! # 無 JS のため全アクションを disabled 固定
//!
//! ハンバーガー・通知・プロフィール trigger はいずれも `disabled: true`
//! で押しても何も起きないことを明示し、[`LAYOUT_CSS`] の
//! `[data-disabled]` 複合セレクタで `opacity: 1; cursor: default;` に
//! 中和して通常状態と同じ見た目に保つ。検索欄（[`search_box`]）も
//! `FieldProps`/`InputGroupProps` の両方を `disabled: true` にして入力
//! 不能にし、送信先を持たず `<form>` へは包まない。無効化に伴い
//! `field`/`input` パート・`input-group`/`addon` パート（検索アイコン）
//! が自前で持つ `opacity: 0.5` も、上記アクション群と同じく
//! [`LAYOUT_CSS`] の `[data-blocks-navbar-two-row-search]` 配下の
//! `[data-disabled]` 複合セレクタで `opacity: 1;` に中和する
//! （中和しないと検索欄だけ他のアクションより薄く見える不整合になる）。
//!
//! # `<form>`/`href="#"`/`data:` を持たない
//!
//! `crate::blocks` モジュール doc「セキュリティ不変条件」節に従う静的な
//! 合成例。`href` は自リポジトリ・自組織の実在 URL とサイト内相対パス
//! （現在地リンクの `./`）に限る。文言はすべて架空の日本語。
//!
//! # 暗色配色（R1092）はサイトのテーマトグルに追随させる
//!
//! R1092 は R1093 と暗色配色のみが異なる集約元であり、block 固有の色を
//! 追加で持ち込まない（[`LAYOUT_CSS`] は `--fandhe-*` トークンのみ参照
//! するため、docs サイトのヘッダーにあるテーマトグルで自動的に暗色へ
//! 追随する）。Demo へ暗色専用インスタンスは並記しない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, header, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::collapsible;
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps};
use fandhe_frontend_pre_styled_ui::tab_nav;
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

/// ロゴ（幾何図形 + ブランド名テキスト）。
fn logo() -> Node {
    span(
        vec![("data-blocks-navbar-two-row-logo", "")],
        vec![
            geo_icon("M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z"),
            span(vec![], vec![text("Fandhe Console")]),
        ],
    )
}

/// 検索アイコン（円 + 柄の 2 path、装飾のため `aria-hidden="true"`）。
fn search_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
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

/// 中央の検索欄（可視ラベルの代わりに `visually_hidden` + `field::label`
/// で `<label for>` の関連付けによりアクセシブル名を確保する、
/// `hero_search::search_group` と同じ判断）。`variant` は `id` の suffix。
fn search_box(variant: &'static str) -> Node {
    let field_id = format!("blocks-navbar-two-row-search-{variant}");
    let query_field = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        disabled: true,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: true,
        invalid: false,
    };
    div(
        vec![("data-blocks-navbar-two-row-search", "")],
        vec![field::root(
            &FieldRootProps {
                orientation: FieldOrientation::Vertical,
            },
            &query_field,
            vec![],
            vec![
                visually_hidden::root(
                    vec![],
                    vec![field::label(
                        &query_field,
                        vec![],
                        vec![text("サイト内を検索")],
                    )],
                ),
                input_group::root(
                    &group_props,
                    vec![],
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
        )],
    )
}

/// 通知ボタン（無 JS のため `disabled: true` 固定、アクセシブルネーム付き）。
fn notification_button() -> Node {
    button::icon_button(
        &ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        },
        "Notifications",
        vec![("data-blocks-navbar-two-row-notify", "")],
        vec![geo_icon("M6 8a6 6 0 0 1 12 0v4l2 4H4l2-4z")],
    )
}

/// プロフィールメニュー（`menu::trigger` を `disabled: true` 固定にし、
/// 中に [`avatar`] を入れる。`content_id` は variant ごとに一意にする）。
fn profile_menu(variant: &str) -> Node {
    let content_id = format!("blocks-navbar-two-row-profile-menu-{variant}");
    let trigger = menu::trigger(
        OpenState::Closed,
        true,
        Some(content_id.as_str()),
        vec![
            ("aria-label", "アカウントメニューを開く"),
            ("data-blocks-navbar-two-row-profile-trigger", ""),
        ],
        vec![avatar::root(
            &AvatarProps::default(),
            vec![],
            vec![avatar::fallback(
                ImageStatus::Error,
                vec![],
                vec![text("YS")],
            )],
        )],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item("profile", false, false, vec![], vec![text("プロフィール")]),
            menu::item("settings", false, false, vec![], vec![text("設定")]),
            menu::item("help", false, false, vec![], vec![text("ヘルプ")]),
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

/// ハンバーガートリガー（狭い幅専用、[`mobile_panel`] を `aria-controls`
/// で指す。押しても何も起きないため `disabled: true` 固定だが、
/// `OpenState::Open` によりパネル自体は常に到達可能）。
fn hamburger(panel_id: &str) -> Node {
    collapsible::trigger(
        collapsible::OpenState::Open,
        true,
        Some(panel_id),
        vec![
            ("aria-label", "Open main menu"),
            ("data-blocks-navbar-two-row-toggle", ""),
        ],
        vec![geo_icon("M3 6h18v2H3zM3 11h18v2H3zM3 16h18v2H3z")],
    )
}

/// 常時展開のハンバーガーパネル（2 段目ナビの clone のみを持つ、
/// モジュール doc「`id`/`aria-label` の一意性」節参照）。
fn mobile_panel(panel_id: &str, secondary_nav: Node) -> Node {
    collapsible::content(
        collapsible::OpenState::Open,
        true,
        Some(panel_id),
        vec![("data-blocks-navbar-two-row-panel", "")],
        vec![secondary_nav],
    )
}

/// `pills` variant の 2 段目（`navigation-menu` のピル型リンク、
/// [`LAYOUT_CSS`] の `current-style="pill"` 複合セレクタで角丸を付ける）。
fn secondary_nav_pills(aria_label: &str) -> Node {
    let props = NavigationMenuProps::default();
    let items: &[(&str, &str, &str, bool)] = &[
        ("overview", "概要", "./", true),
        ("projects", "プロジェクト", REPO, false),
        ("reports", "レポート", REPO, false),
        ("members", "メンバー", ORG, false),
    ];
    navigation_menu::root(
        &props,
        aria_label,
        vec![("data-blocks-navbar-two-row-secondary", "")],
        vec![navigation_menu::list(
            &props,
            vec![],
            items
                .iter()
                .map(|(value, label, href, current)| {
                    navigation_menu::item(
                        navigation_menu::OpenState::Closed,
                        false,
                        &props,
                        value,
                        vec![],
                        vec![navigation_menu::link(
                            href,
                            *current,
                            vec![],
                            vec![text(*label)],
                        )],
                    )
                })
                .collect(),
        )],
    )
}

/// `tabs-center` variant の 2 段目（`tab-nav` のタブ型リンク、先頭 1 件を
/// 現在地とする。モジュール doc「2 段目に `tabs` ではなく `tab-nav` を
/// 使う判断」節参照）。
fn secondary_nav_tabs(aria_label: &str) -> Node {
    let items: &[(&str, &str, bool)] = &[
        ("./", "概要", true),
        (REPO, "プロジェクト", false),
        (REPO, "レポート", false),
        (ORG, "メンバー", false),
    ];
    tab_nav::root(
        Size::Md,
        aria_label,
        vec![("data-blocks-navbar-two-row-secondary", "")],
        items
            .iter()
            .map(|(href, label, current)| tab_nav::link(href, *current, vec![], vec![text(*label)]))
            .collect(),
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &'static str) -> Node {
    p(
        vec![("class", "blocks-navbar-two-row-caption")],
        vec![text(label)],
    )
}

/// 1 variant 分の 2 段構成ナビバー一式（1 段目 + 2 段目 + 常時展開パネル）を
/// 組み立てる。`variant` は `"pills"`/`"tabs-center"`、
/// `hamburger_at_start` は 1 段目でハンバーガーを先頭（左端）へ置くか
/// 末尾（右端）へ置くか（R0155 の多階層・ドキュメントサイト向け配置差）。
fn bar(
    variant: &'static str,
    secondary_nav_for: fn(&str) -> Node,
    hamburger_at_start: bool,
) -> Node {
    let panel_id = format!("blocks-navbar-two-row-panel-{variant}");
    let aria_label = format!("セカンダリナビゲーション（{variant}）");
    let secondary_nav = secondary_nav_for(&aria_label);
    let secondary_row = div(
        vec![("data-blocks-navbar-two-row-row-secondary", "")],
        vec![secondary_nav.clone()],
    );

    let actions = div(
        vec![("data-blocks-navbar-two-row-actions", "")],
        vec![notification_button(), profile_menu(variant)],
    );
    let search = search_box(variant);
    let toggle = hamburger(&panel_id);

    let primary_children = if hamburger_at_start {
        vec![toggle, logo(), search, actions]
    } else {
        vec![logo(), search, actions, toggle]
    };

    div(
        vec![
            ("data-blocks-navbar-two-row-shell", ""),
            ("data-blocks-navbar-two-row-variant", variant),
        ],
        vec![
            header(
                vec![("data-blocks-navbar-two-row-row-primary", "")],
                primary_children,
            ),
            secondary_row,
            mobile_panel(&panel_id, secondary_nav),
        ],
    )
}

/// `navbar-two-row` の Demo 本体。2 variant を縦に並記する純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-navbar-two-row-stack")],
        vec![
            caption("ピル型セカンダリナビ・アクション末尾のハンバーガー（pills）"),
            bar("pills", secondary_nav_pills, false),
            caption("中央寄せタブ・左端のハンバーガー（tabs-center）"),
            bar("tabs-center", secondary_nav_tabs, true),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/navbar-two-row/",
    title: "navbar-two-row",
    category: BlockCategory::Navbar,
    rust_source: "crates/docs-site/src/blocks/application/navbar/navbar_two_row.rs",
    demo_class: "blocks-navbar-two-row",
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
            label: "Navigation Menu",
            path: "/themes/navigation-menu/",
        },
        Part {
            label: "Tab Nav",
            path: "/themes/tab-nav/",
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
        Part {
            label: "Collapsible",
            path: "/themes/collapsible/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `navbar_two_row` 固有のレイアウト規則（`--fandhe-*` トークンのみ使用）。
/// セレクタは `.blocks-navbar-two-row-*` / `[data-blocks-navbar-two-row-*]`、
/// および styled 部品の `[data-scope][data-part]` セレクタとの複合セレクタ
/// のみを用い、他 block や部品の素のセレクタへは影響させない。
const LAYOUT_CSS: &str = "\
.blocks-navbar-two-row-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-navbar-two-row-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-navbar-two-row-shell] {\n  container-type: inline-size;\n  container-name: blocks-navbar-two-row;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  overflow: hidden;\n  background: var(--fandhe-color-bg);\n}\n\
[data-blocks-navbar-two-row-row-primary] {\n  display: grid;\n  grid-template-columns: auto minmax(0, 1fr) auto auto;\n  align-items: center;\n  gap: var(--fandhe-space-3) var(--fandhe-space-4);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  background: var(--fandhe-color-bg-subtle);\n}\n\
[data-blocks-navbar-two-row-variant=\"tabs-center\"] [data-blocks-navbar-two-row-row-primary] {\n  grid-template-columns: auto auto minmax(0, 1fr) auto;\n}\n\
[data-blocks-navbar-two-row-logo] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  white-space: nowrap;\n}\n\
[data-blocks-navbar-two-row-search] {\n  max-inline-size: 32rem;\n  margin-inline: auto;\n  inline-size: 100%;\n}\n\
[data-blocks-navbar-two-row-search] [data-scope=\"field\"][data-part=\"input\"][data-disabled] {\n  opacity: 1;\n  cursor: not-allowed;\n}\n\
[data-blocks-navbar-two-row-search] [data-scope=\"input-group\"][data-part=\"addon\"][data-disabled] {\n  opacity: 1;\n  cursor: not-allowed;\n}\n\
[data-blocks-navbar-two-row-actions] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-navbar-two-row-notify][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"menu\"][data-part=\"trigger\"][data-blocks-navbar-two-row-profile-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-navbar-two-row-toggle] {\n  display: inline-flex;\n}\n\
[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-navbar-two-row-toggle][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-blocks-navbar-two-row-row-secondary] {\n  display: none;\n  padding: var(--fandhe-space-2) var(--fandhe-space-4);\n  border-block-start: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-navbar-two-row-variant=\"pills\"] [data-blocks-navbar-two-row-row-secondary] {\n  justify-content: flex-start;\n}\n\
[data-blocks-navbar-two-row-variant=\"tabs-center\"] [data-blocks-navbar-two-row-row-secondary] {\n  justify-content: center;\n}\n\
[data-blocks-navbar-two-row-variant=\"pills\"] [data-scope=\"navigation-menu\"][data-part=\"link\"] {\n  border-radius: var(--fandhe-radius-full);\n}\n\
[data-scope=\"collapsible\"][data-part=\"content\"][data-blocks-navbar-two-row-panel] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  padding: 0 var(--fandhe-space-4) var(--fandhe-space-3);\n}\n\
[data-blocks-navbar-two-row-panel] [data-scope=\"navigation-menu\"][data-part=\"list\"] {\n  flex-direction: column;\n}\n\
[data-blocks-navbar-two-row-panel] [data-scope=\"tab-nav\"][data-part=\"root\"] {\n  flex-direction: column;\n}\n\
@container blocks-navbar-two-row (min-width: 48rem) {\n  \
[data-blocks-navbar-two-row-row-primary] {\n    grid-template-columns: auto minmax(0, 1fr) auto;\n  }\n  \
[data-blocks-navbar-two-row-variant=\"tabs-center\"] [data-blocks-navbar-two-row-row-primary] {\n    grid-template-columns: auto minmax(0, 1fr) auto;\n  }\n  \
[data-blocks-navbar-two-row-row-secondary] {\n    display: flex;\n  }\n  \
[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-navbar-two-row-toggle] {\n    display: none;\n  }\n  \
[data-scope=\"collapsible\"][data-part=\"content\"][data-blocks-navbar-two-row-panel] {\n    display: none;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, ORG, REPO};
    use fandhe_frontend_core::render;

    /// 9 部品の `data-scope` が揃い、`type="button"` があり、`<form>`・
    /// `href="#"`・`data:` src を持たないこと。
    #[test]
    fn demo_composes_expected_parts_and_has_no_form() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"input-group\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"navigation-menu\"",
            "data-scope=\"tab-nav\"",
            "data-scope=\"button\"",
            "data-scope=\"avatar\"",
            "data-scope=\"menu\"",
            "data-scope=\"icon\"",
            "data-scope=\"collapsible\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 2 variant がそれぞれ 1 回ずつ描画され、caption が 2 件あること。
    #[test]
    fn demo_renders_both_variants() {
        let html = render(&demo());
        for variant in ["pills", "tabs-center"] {
            assert!(
                html.contains(&format!("data-blocks-navbar-two-row-variant=\"{variant}\"")),
                "variant {variant} should render"
            );
        }
        assert_eq!(html.matches("blocks-navbar-two-row-caption").count(), 2);
        assert_eq!(html.matches("data-blocks-navbar-two-row-shell").count(), 2);
    }

    /// 通知・プロフィール trigger・ハンバーガーが押しても何も起きないよう
    /// 無効化されていること（`disabled`/`data-disabled` を持つ）。
    #[test]
    fn action_controls_are_disabled() {
        let html = render(&demo());
        for hook in [
            "data-blocks-navbar-two-row-notify",
            "data-blocks-navbar-two-row-profile-trigger",
            "data-blocks-navbar-two-row-toggle",
        ] {
            assert_eq!(
                html.matches(hook).count(),
                2,
                "hook {hook} should render once per variant"
            );
        }
    }

    /// 検索欄が variant ごとに 1 回、入力欄自体（`data-scope="field"
    /// data-part="input"`）へ `disabled=""` が付き、`data-scope="input-group"
    /// data-part="root"` へ `data-disabled` が付くこと（無 JS のため操作
    /// 不能な静的 Demo である契約）。
    #[test]
    fn search_box_is_disabled() {
        let html = render(&demo());
        assert_eq!(html.matches("data-blocks-navbar-two-row-search").count(), 2);
        for variant in ["pills", "tabs-center"] {
            let control_id = format!("blocks-navbar-two-row-search-{variant}-control");
            let input_pos = html
                .find(&format!(r#"id="{control_id}""#))
                .unwrap_or_else(|| panic!("search input id={control_id} should render"));
            let tag_end = html[input_pos..]
                .find('>')
                .map(|rel| input_pos + rel)
                .expect("input tag should close");
            let tag_start = html[..input_pos]
                .rfind("<input")
                .expect("search input should have an opening tag");
            let input_tag = &html[tag_start..tag_end];
            assert!(input_tag.contains("disabled=\"\""), "input_tag={input_tag}");
        }
        assert_eq!(
            html.matches(
                "data-scope=\"input-group\" data-part=\"root\" role=\"group\" data-disabled=\"\""
            )
            .count(),
            2
        );
    }

    /// 検索欄の `field`/`input` パート・`input-group`/`addon`
    /// パート（検索アイコン）が、無効化に伴う自前の `opacity: 0.5`
    /// フェードを [`LAYOUT_CSS`] で中和していること（Cursor Bugbot
    /// 指摘、検索欄だけ他のアクションより薄く見える不整合の回帰防止）。
    #[test]
    fn search_box_disabled_fade_is_counteracted() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-navbar-two-row-search] [data-scope=\"field\"][data-part=\"input\"][data-disabled] {\n  opacity: 1;\n  cursor: not-allowed;\n}"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-navbar-two-row-search] [data-scope=\"input-group\"][data-part=\"addon\"][data-disabled] {\n  opacity: 1;\n  cursor: not-allowed;\n}"
        ));
    }

    /// ハンバーガーが variant 数だけあり、`aria-controls`/`id` が
    /// 対応し、パネルに `hidden` が付かないこと（常時展開）。
    #[test]
    fn hamburgers_control_an_always_open_panel() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"aria-label="Open main menu""#).count(), 2);
        for variant in ["pills", "tabs-center"] {
            let panel_id = format!("blocks-navbar-two-row-panel-{variant}");
            assert!(html.contains(&format!(r#"aria-controls="{panel_id}""#)));
            assert!(html.contains(&format!(r#"id="{panel_id}""#)));
        }
        assert_eq!(html.matches("data-blocks-navbar-two-row-panel").count(), 2);
        for panel_attr_pos in html.match_indices("data-blocks-navbar-two-row-panel") {
            let tag_start = html[..panel_attr_pos.0]
                .rfind("<div")
                .expect("panel div should have an opening tag");
            let tag_end = html[tag_start..]
                .find('>')
                .map(|rel| tag_start + rel)
                .expect("panel opening tag should close");
            let panel_tag = &html[tag_start..tag_end];
            assert!(!panel_tag.contains("hidden"), "panel_tag={panel_tag}");
        }
    }

    /// プロフィールメニューの `content_id` が variant ごとに一意であること
    /// （`aria-controls` が対応する `id` を持つ）。
    #[test]
    fn profile_menu_ids_are_unique_per_variant() {
        let html = render(&demo());
        for variant in ["pills", "tabs-center"] {
            let content_id = format!("blocks-navbar-two-row-profile-menu-{variant}");
            assert!(html.contains(&format!(r#"aria-controls="{content_id}""#)));
            assert!(html.contains(&format!(r#"id="{content_id}""#)));
        }
    }

    /// `tab-nav` scope 内の現在地リンク（`aria-current="page"`）が 2 回
    /// 出ること（表示行 + ハンバーガーパネル内 clone の 1 回ずつ、
    /// `navbar_app_links`/`header_simple_bar` と同じ理由で非表示側の
    /// 重複は実害を持たない）。
    #[test]
    fn tab_nav_has_current_link_in_visible_row_and_panel_clone() {
        let html = render(&demo());
        let tab_nav_pos = html
            .find("data-scope=\"tab-nav\" data-part=\"root\"")
            .expect("tab-nav root should render");
        let tab_nav_section = &html[tab_nav_pos..];
        assert_eq!(tab_nav_section.matches(r#"aria-current="page""#).count(), 2);
    }

    /// `BLOCK.parts` の `path` が全件 kebab-case の `/themes/…/` であること。
    #[test]
    fn parts_point_to_themes_pages() {
        for part in super::BLOCK.parts {
            assert!(part.path.starts_with("/themes/"));
            assert!(part.path.ends_with('/'));
        }
    }

    /// [`LAYOUT_CSS`] が container query の境界・2 段目非表示・disabled 中和・
    /// ピル型 CSS を持つこと（DOM 順自体を variant ごとに並べ替える設計の
    /// ため CSS `order` は使わない、`header_simple_bar` と同じ判断。DOM 順
    /// 自体の検証は別テストが担う）。`navbar_app_links` と同じく画面幅では
    /// なく Demo 枠自体の幅で切り替えるため `@media` ではなく `@container`
    /// を用いる（狭幅パネル内のナビリンクを縦積みにする規則も併せて検証）。
    #[test]
    fn layout_css_has_container_query_and_style_rules() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-navbar-two-row (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-navbar-two-row-toggle] {\n    display: none;\n  }"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"collapsible\"][data-part=\"content\"][data-blocks-navbar-two-row-panel] {\n    display: none;\n  }"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-navbar-two-row-panel] [data-scope=\"navigation-menu\"][data-part=\"list\"] {\n  flex-direction: column;\n}"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-navbar-two-row-panel] [data-scope=\"tab-nav\"][data-part=\"root\"] {\n  flex-direction: column;\n}"
        ));
        assert!(LAYOUT_CSS.contains("border-radius: var(--fandhe-radius-full);"));
    }

    /// リンクは自リポジトリ・自組織の実在 URL と現在地 `./` のみを指す
    /// こと。
    #[test]
    fn links_point_to_real_urls_or_self() {
        let html = render(&demo());
        assert!(html.contains(r#"href="./""#));
        assert!(html.contains(&format!(r#"href="{REPO}""#)));
        assert!(html.contains(&format!(r#"href="{ORG}""#)));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-navbar-two-row-stack\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-navbar-two-row-stack");
    }
}
