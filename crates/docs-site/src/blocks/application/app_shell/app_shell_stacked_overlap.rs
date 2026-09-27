//! `app-shell-stacked-overlap` block（イシュー #2897。親トラッキング #2892
//! 「Blocks アプリケーション A（phase:3）」配下、対応表 ID R1278（代表
//! 構成）を主参照とする合成例。集約元 R1279（帯の配色だけがブランド色に
//! なる版）・R1282（ナビが 2 段になり 2 行目に検索欄と主要リンクが入る版）
//! の差分は `site/blocks/app-shell-stacked-overlap.md` の「集約元との差分
//! メモ」節に記す）。
//!
//! # 使用部品
//!
//! `navigation-menu`（主要リンク）/ `input-group`・`input`（2 段版の検索
//! 欄）/ `avatar`（プロフィールボタン内のフォールバック）/
//! `menu`（プロフィールの閉じたドロップダウン）/ `button`（通知アイコン
//! ボタン、無 JS のため `disabled: true` 固定）/ `card`（重なる本文カード）/
//! `heading`（帯内のページ見出し）/ `collapsible`（狭幅の常時展開メニュー
//! パネル）/ `icon`（自作の幾何図形のみ、装飾用途）の 9 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # 2 variant の並記（配色・ナビ段数の差分を読み取れるようにする）
//!
//! 無 JS のため実際の開閉・レスポンシブ切り替えは示せない
//! （`crate::blocks` モジュール doc の一般方針）。同じ骨格（[`shell`]）を
//! 使う 2 variant を縦に並記する（`header_simple_bar`/`app_shell_navbar_columns`
//! と同型の判断）。
//!
//! | variant | 帯の配色 | ナビ | 対応 ID |
//! |---|---|---|---|
//! | `single-row` | 中立の濃色反転 | 1 段（ロゴ・メインナビ・通知・プロフィール） | R1278（代表） |
//! | `two-row-search` | ブランド色 | 2 段。2 行目に検索欄 + クイックリンク | R1279（配色）+ R1282（2 段+検索） |
//!
//! # 帯へ本文カードを重ねる仕組み
//!
//! 帯（[`band`]）は `padding-block-end` へ CSS 変数
//! `--blocks-app-shell-stacked-overlap-overlap` 分を上乗せし、本文
//! （`-body`）は `margin-block-start: calc(-1 * var(overlap))` で帯へ
//! 食い込む。重なり量は狭い幅で小さく、`@container
//! blocks-app-shell-stacked-overlap (min-width: 48rem)` で広げる（Demo 枠の
//! 幅基準、`header_simple_bar` 以降の block と同じコンテナクエリ方針）。
//!
//! # 狭幅ではナビをメニューボタンで畳む（`order` を使わない）
//!
//! `header_simple_bar` と同じ判断で、デスクトップ用ナビ（と 2 段版の
//! 2 行目）は専用ラッパー div で包み、狭い幅では隠す。代わりに
//! `collapsible::trigger`（`OpenState::Open` + `disabled: true` 固定 =
//! 常時展開でクリックしても何も起きない）と、常時展開パネル
//! （[`mobile_panel`]）を `aria-controls` で関連付ける。パネルの中身は
//! ナビ（`id` を持たないため `Node::clone()` で共有）と、アクション・
//! 検索欄（`menu::content`/`field` が固有 `id` を出力するため、別 suffix
//! で組み直した別インスタンス。同一 `id` の重複出力を禁じる契約テスト
//! `blocks_contract.rs::demo_output_has_no_dangling_aria_references_or_duplicate_ids`
//! に従う）。表示切り替えはラッパー自身の `data-*` 属性のみに CSS を
//! 効かせ、パネル内側の同一属性へは到達させない（`header_simple_bar` の
//! モジュール doc「CSS フックに data 属性を使う理由」節と同じ構造）。
//! `order` は使わない（DOM 順とキーボード操作順・
//! 視覚順を一致させる）。
//!
//! # 帯の中の部品の文字色・disabled の中和
//!
//! `navigation-menu`/`menu` の trigger/link・通知ボタンは styled recipe の
//! 既定色のままだと濃色・ブランド色の帯の上で読めなくなる。[`LAYOUT_CSS`]
//! は `[data-scope="…"][data-part="…"][data-blocks-app-shell-stacked-overlap-*]`
//! の複合セレクタで `color: inherit` を上書きし、通知ボタン・プロフィール
//! トリガーは disabled 固定のため `opacity: 1; cursor: default;` も添える
//! （`app_shell_navbar_columns`/`header_simple_bar` と同じ詳細度対策）。
//!
//! # `href` の方針・`<form>` を持たない・全データが架空
//!
//! `href` は実在する自リポジトリ・組織の URL に限る（`href="#"` は
//! `linkcheck` が拒否する死リンクのため使わない）。検索欄は送信先を
//! 持たない静的な入力（送信ボタンを置かない）。ブランド名・ユーザー名・
//! 本文はすべて架空のもの。`crate::blocks` モジュール doc の不変条件
//! どおり `<form>` は出力しない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{a, div, el, header, main_tag, nav as nav_el, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::collapsible;
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState as MenuOpenState};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc 参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// 実在の自組織 URL。
const ORG: &str = "https://github.com/Fandhe-AI";

/// メインナビの項目一覧（value, label, href）。両 variant で共有する。
const NAV_ITEMS: &[(&str, &str, &str)] = &[
    ("overview", "概要", REPO),
    ("projects", "プロジェクト", REPO),
    ("reports", "レポート", REPO),
    ("team", "メンバー", ORG),
];

/// 2 段版の 2 行目（クイックリンク、静的な素の `<a>`）。
const QUICK_LINKS: &[(&str, &str)] = &[("最近の更新", REPO), ("ヘルプ", ORG)];

/// 重なるカードのダミー本文行。
const CARD_ROWS: &[(&str, &str)] = &[
    ("今月の進捗", "先週比 12% 増。目標まで残り 3 タスクです。"),
    ("次のマイルストーン", "10 月 3 日にレビュー予定です。"),
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

/// ベル（通知）の幾何図形アイコン。
fn bell_icon() -> Node {
    geo_icon(
        Size::Sm,
        "M12 3a5 5 0 0 0-5 5v3l-2 4h14l-2-4V8a5 5 0 0 0-5-5zM10 18a2 2 0 0 0 4 0h-4z",
    )
}

/// ハンバーガー（3 本線）アイコン。
fn hamburger_icon() -> Node {
    geo_icon(Size::Md, "M3 6h18v2H3zM3 11h18v2H3zM3 16h18v2H3z")
}

/// 検索（虫眼鏡）アイコン。
fn search_icon() -> Node {
    geo_icon(
        Size::Sm,
        "M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zm9 17-5.2-5.2",
    )
}

/// ロゴ（幾何図形アイコン + ブランド名テキスト、リンクにしない。
/// `app_shell_navbar_columns::logo` と同型）。
fn logo() -> Node {
    div(
        vec![("data-blocks-app-shell-stacked-overlap-logo", "")],
        vec![
            geo_icon(Size::Md, "M4 4h16v6H4zM4 14h16v6H4z"),
            span(vec![], vec![text("Fandhe Frontend")]),
        ],
    )
}

/// メインナビ本体（`aria-label` は variant ごとに一意にする）。
fn nav(aria_label: &str) -> Node {
    let props = NavigationMenuProps::default();
    navigation_menu::root(
        &props,
        aria_label,
        vec![],
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

/// 2 段版の 2 行目クイックリンク（部品化しない静的なダミーナビ、`href="#"`
/// は使わない）。
fn quick_links(aria_label: &str) -> Node {
    nav_el(
        vec![("aria-label", aria_label)],
        vec![div(
            vec![("data-blocks-app-shell-stacked-overlap-quick-links", "")],
            QUICK_LINKS
                .iter()
                .map(|(label, href)| a(vec![("href", href)], vec![text(*label)]))
                .collect(),
        )],
    )
}

/// 検索欄（可視ラベルの代わりに `visually_hidden` + `<label for>`、
/// `hero_search::search_group` と同型）。送信ボタンは置かない（`<form>`
/// を持たないデモのため）。
fn search_field(suffix: &str) -> Node {
    let field_id = format!("blocks-app-shell-stacked-overlap-search-{suffix}");
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
        vec![("data-blocks-app-shell-stacked-overlap-search-field", "")],
        vec![
            visually_hidden::root(
                vec![],
                vec![field::label(
                    &query_field,
                    vec![],
                    vec![text("サイト内検索")],
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
    )
}

/// 通知ボタン（無 JS デモのため `disabled: true` 固定）。
fn notify_button() -> Node {
    button::icon_button(
        &ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        },
        "通知を表示",
        vec![("data-blocks-app-shell-stacked-overlap-notify", "")],
        vec![bell_icon()],
    )
}

/// プロフィールメニュー（閉じた `menu`、avatar フォールバック +
/// visually-hidden のアクセシブルネーム。`suffix` で variant 間の id 衝突を
/// 避ける。`sidebar_07::team_switcher`/`user_menu` と同型）。
fn profile_menu(suffix: &str) -> Node {
    let content_id = format!("blocks-app-shell-stacked-overlap-profile-{suffix}");
    let trigger = menu::trigger(
        MenuOpenState::Closed,
        true,
        Some(content_id.as_str()),
        vec![("data-blocks-app-shell-stacked-overlap-profile", "")],
        vec![
            avatar::root(
                &AvatarProps::default(),
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text("YT")],
                )],
            ),
            visually_hidden::root(vec![], vec![text("アカウントメニューを開く")]),
        ],
    );
    let content = menu::content(
        MenuOpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item("profile", false, false, vec![], vec![text("プロフィール")]),
            menu::item("settings", false, false, vec![], vec![text("設定")]),
            menu::separator(vec![], vec![]),
            menu::item("logout", false, false, vec![], vec![text("ログアウト")]),
        ],
    );
    let positioner = menu::positioner(MenuOpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        MenuOpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// アクション行（通知 + プロフィールメニュー）。
fn actions(suffix: &str) -> Node {
    div(
        vec![("data-blocks-app-shell-stacked-overlap-actions", "")],
        vec![notify_button(), profile_menu(suffix)],
    )
}

/// ハンバーガートリガー（狭い幅専用、常時展開の [`mobile_panel`] を
/// `aria-controls` で指す。`header_simple_bar::hamburger` と同型）。
fn hamburger(panel_id: &str) -> Node {
    collapsible::trigger(
        OpenState::Open,
        true,
        Some(panel_id),
        vec![
            ("aria-label", "メニューを開く"),
            ("data-blocks-app-shell-stacked-overlap-toggle", ""),
        ],
        vec![hamburger_icon()],
    )
}

/// 常時展開のドロップダウンパネル（狭い幅専用、[`shell`] の呼び出し元が
/// ナビ（clone）と、`id` 重複を避けて別 suffix で組み直したアクション・
/// 検索欄・行 2 を渡す）。
fn mobile_panel(panel_id: &str, children: Vec<Node>) -> Node {
    collapsible::content(
        OpenState::Open,
        true,
        Some(panel_id),
        vec![("data-blocks-app-shell-stacked-overlap-panel", "")],
        children,
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &str) -> Node {
    p(
        vec![("class", "blocks-app-shell-stacked-overlap-caption")],
        vec![text(label)],
    )
}

/// 重なる本文カード群。
fn body_cards() -> Node {
    div(
        vec![("data-blocks-app-shell-stacked-overlap-body", "")],
        CARD_ROWS
            .iter()
            .map(|(title, body)| {
                card::root(
                    CardProps::default(),
                    vec![],
                    vec![
                        card::header(vec![], vec![card::title(vec![], vec![text(*title)])]),
                        card::body(vec![], vec![p(vec![], vec![text(*body)])]),
                    ],
                )
            })
            .collect(),
    )
}

/// 1 本の shell（モジュール doc「2 variant の並記」節参照）。
fn shell(
    variant: &str,
    root_label: &str,
    nav_label: &str,
    quick_links_label: Option<&str>,
    page_title: &str,
    page_summary: &str,
) -> Node {
    let panel_suffix = format!("{variant}-panel");
    let panel_id = format!("blocks-app-shell-stacked-overlap-panel-{variant}");
    // nav・クイックリンクは `id` を持たないため clone で共有できるが、
    // プロフィールメニュー（`menu::content` の `id`）・検索欄（`field` の
    // `id`）はそれぞれ固有 `id` を出力するため、パネル側は clone せず
    // 別 suffix（`panel_suffix`）で独立して組み立てる（重複 `id` の禁止、
    // `blocks_contract.rs::demo_output_has_no_dangling_aria_references_or_duplicate_ids`
    // 参照）。
    let nav_node = nav(nav_label);
    let nav_wrap = div(
        vec![("data-blocks-app-shell-stacked-overlap-nav-wrap", "")],
        vec![nav_node.clone()],
    );
    let actions_wrap = div(
        vec![("data-blocks-app-shell-stacked-overlap-actions-wrap", "")],
        vec![actions(variant)],
    );

    let mut panel_children = vec![nav_node, actions(&panel_suffix)];

    let bar = header(
        vec![("data-blocks-app-shell-stacked-overlap-bar", "")],
        vec![logo(), nav_wrap, actions_wrap, hamburger(&panel_id)],
    );

    let mut band_children = vec![bar];

    if let Some(quick_links_label) = quick_links_label {
        let quick_links_node = quick_links(quick_links_label);
        let row2_wrap = div(
            vec![("data-blocks-app-shell-stacked-overlap-row2", "")],
            vec![search_field(variant), quick_links_node.clone()],
        );
        band_children.push(row2_wrap);
        panel_children.push(search_field(&panel_suffix));
        panel_children.push(quick_links_node);
    }

    band_children.push(mobile_panel(&panel_id, panel_children));

    band_children.push(div(
        vec![("data-blocks-app-shell-stacked-overlap-heading-row", "")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text(page_title)],
            ),
            p(vec![], vec![text(page_summary)]),
        ],
    ));

    div(
        vec![
            ("tabindex", "0"),
            ("role", "region"),
            ("aria-label", root_label),
            ("data-blocks-app-shell-stacked-overlap-root", ""),
            ("data-blocks-app-shell-stacked-overlap-variant", variant),
        ],
        vec![
            div(
                vec![("data-blocks-app-shell-stacked-overlap-band", "")],
                band_children,
            ),
            main_tag(vec![], vec![body_cards()]),
        ],
    )
}

/// `app-shell-stacked-overlap` の Demo 本体。2 variant を縦に並記する
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-app-shell-stacked-overlap-stack")],
        vec![
            caption("1 段ナビ・中立配色"),
            shell(
                "single-row",
                "アプリシェル（1 段ナビ）",
                "アプリ（1 段）",
                None,
                "ダッシュボード",
                "本日の概況をまとめています。",
            ),
            caption("2 段ナビ・ブランド配色 + 検索"),
            shell(
                "two-row-search",
                "アプリシェル（2 段ナビ + 検索）",
                "アプリ（2 段）",
                Some("クイックリンク"),
                "ダッシュボード（検索対応）",
                "検索欄からドキュメントを横断検索できます。",
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/app-shell-stacked-overlap/",
    title: "app-shell-stacked-overlap",
    category: BlockCategory::AppShell,
    rust_source: "crates/docs-site/src/blocks/application/app_shell/app_shell_stacked_overlap.rs",
    demo_class: "blocks-app-shell-stacked-overlap",
    parts: &[
        Part {
            label: "Navigation Menu",
            path: "/themes/navigation-menu/",
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
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Collapsible",
            path: "/themes/collapsible/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `app_shell_stacked_overlap` 固有のレイアウト規則（`--fandhe-*` トークン
/// のみ使用）。セレクタは `.blocks-app-shell-stacked-overlap-*` と
/// `[data-blocks-app-shell-stacked-overlap-*]`、および styled 部品の
/// `[data-scope]`/`[data-part]` セレクタとの複合セレクタのみを用い、他
/// block や部品の素のセレクタへは影響させない。`order` は使わない。
const LAYOUT_CSS: &str = "\
.blocks-app-shell-stacked-overlap-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-app-shell-stacked-overlap;\n}\n\
.blocks-app-shell-stacked-overlap-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-app-shell-stacked-overlap-root] {\n  --blocks-app-shell-stacked-overlap-overlap: 2rem;\n  display: flex;\n  flex-direction: column;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: 0.5rem;\n  overflow: hidden;\n  background: var(--fandhe-color-bg-subtle);\n}\n\
[data-blocks-app-shell-stacked-overlap-band] {\n  padding: var(--fandhe-space-4) var(--fandhe-space-4) calc(var(--fandhe-space-4) + var(--blocks-app-shell-stacked-overlap-overlap));\n}\n\
[data-blocks-app-shell-stacked-overlap-root][data-blocks-app-shell-stacked-overlap-variant=\"single-row\"] [data-blocks-app-shell-stacked-overlap-band] {\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);\n}\n\
[data-blocks-app-shell-stacked-overlap-root][data-blocks-app-shell-stacked-overlap-variant=\"two-row-search\"] [data-blocks-app-shell-stacked-overlap-band] {\n  background: var(--fandhe-color-accent);\n  color: var(--fandhe-color-accent-fg);\n}\n\
[data-blocks-app-shell-stacked-overlap-bar] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-app-shell-stacked-overlap-logo] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  white-space: nowrap;\n}\n\
[data-blocks-app-shell-stacked-overlap-nav-wrap] {\n  display: none;\n  flex-basis: 100%;\n}\n\
[data-blocks-app-shell-stacked-overlap-actions-wrap] {\n  display: none;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  margin-inline-start: auto;\n}\n\
[data-blocks-app-shell-stacked-overlap-row2] {\n  display: none;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n  margin-block-start: var(--fandhe-space-3);\n}\n\
[data-blocks-app-shell-stacked-overlap-quick-links] {\n  display: flex;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-app-shell-stacked-overlap-quick-links] a {\n  color: inherit;\n}\n\
[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-app-shell-stacked-overlap-toggle] {\n  color: inherit;\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"collapsible\"][data-part=\"content\"][data-blocks-app-shell-stacked-overlap-panel] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  margin-block-start: var(--fandhe-space-3);\n}\n\
[data-blocks-app-shell-stacked-overlap-heading-row] {\n  margin-block-start: var(--fandhe-space-4);\n}\n\
[data-scope=\"navigation-menu\"][data-part=\"link\"][data-blocks-app-shell-stacked-overlap-band]::before {\n  content: none;\n}\n\
[data-blocks-app-shell-stacked-overlap-band] [data-scope=\"navigation-menu\"] {\n  color: inherit;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-app-shell-stacked-overlap-notify] {\n  color: inherit;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-app-shell-stacked-overlap-notify][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"menu\"][data-part=\"trigger\"][data-blocks-app-shell-stacked-overlap-profile] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  color: inherit;\n}\n\
[data-scope=\"menu\"][data-part=\"trigger\"][data-blocks-app-shell-stacked-overlap-profile][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-blocks-app-shell-stacked-overlap-body] {\n  position: relative;\n  z-index: 1;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  margin-block-start: calc(-1 * var(--blocks-app-shell-stacked-overlap-overlap));\n  padding: 0 var(--fandhe-space-4) var(--fandhe-space-4);\n}\n\
@container blocks-app-shell-stacked-overlap (min-width: 48rem) {\n  \
[data-blocks-app-shell-stacked-overlap-root] {\n    --blocks-app-shell-stacked-overlap-overlap: 6rem;\n  }\n  \
[data-blocks-app-shell-stacked-overlap-nav-wrap] {\n    display: block;\n    flex-basis: auto;\n  }\n  \
[data-blocks-app-shell-stacked-overlap-actions-wrap] {\n    display: flex;\n  }\n  \
[data-blocks-app-shell-stacked-overlap-row2] {\n    display: flex;\n  }\n  \
[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-app-shell-stacked-overlap-toggle] {\n    display: none;\n  }\n  \
[data-scope=\"collapsible\"][data-part=\"content\"][data-blocks-app-shell-stacked-overlap-panel] {\n    display: none;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品を持ち、非対話制約（`<form>` なし・`href="#"`
    /// なし・`data:` src なし）を満たすこと。
    #[test]
    fn demo_composes_expected_parts_and_has_no_form() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"navigation-menu\"",
            "data-scope=\"input-group\"",
            "data-scope=\"field\"",
            "data-scope=\"avatar\"",
            "data-scope=\"menu\"",
            "data-scope=\"button\"",
            "data-scope=\"card\"",
            "data-scope=\"heading\"",
            "data-scope=\"collapsible\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 2 variant がそれぞれ 1 回ずつ描画され、2 行目の検索・クイックリンクは
    /// `two-row-search` にだけ現れること。
    #[test]
    fn demo_renders_both_variants_with_expected_rows() {
        let html = render(&demo());
        for variant in ["single-row", "two-row-search"] {
            assert_eq!(
                html.matches(&format!(
                    "data-blocks-app-shell-stacked-overlap-variant=\"{variant}\""
                ))
                .count(),
                1,
                "variant {variant} should render exactly once, html={html}"
            );
        }
        // row2 は two-row-search のデスクトップ用ラッパー + パネル内 clone
        // の計 2 回。
        assert_eq!(
            html.matches("data-blocks-app-shell-stacked-overlap-row2")
                .count(),
            1,
            "row2 wrapper should render once (two-row-search variant only)"
        );
    }

    /// ハンバーガーが 2 個で、`aria-label` を持ち、常時展開の
    /// [`super::mobile_panel`] を `aria-controls` で指すこと。
    #[test]
    fn hamburgers_have_label_and_control_a_panel() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-app-shell-stacked-overlap-toggle")
                .count(),
            2
        );
        assert_eq!(html.matches(r#"aria-label="メニューを開く""#).count(), 2);
        for variant in ["single-row", "two-row-search"] {
            let panel_id = format!("blocks-app-shell-stacked-overlap-panel-{variant}");
            assert!(
                html.contains(&format!(r#"aria-controls="{panel_id}""#)),
                "toggle for variant {variant} should reference {panel_id}"
            );
            assert!(
                html.contains(&format!(r#"id="{panel_id}""#)),
                "panel {panel_id} should render with matching id"
            );
        }
    }

    /// 通知ボタン・プロフィールメニュートリガーが disabled/aria-disabled で
    /// あること（無 JS デモのため押しても何も起きないことの明示）。デスクトップ
    /// 用ラッパー + 常時展開パネル内 clone の計 2 回 × 2 variant = 4 回ずつ
    /// 出現する（`header_simple_bar` と同型の clone 構造、モジュール doc
    /// 「狭幅ではナビをメニューボタンで畳む」節参照）。
    #[test]
    fn notify_and_profile_trigger_are_disabled() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-app-shell-stacked-overlap-notify")
                .count(),
            4
        );
        assert_eq!(
            html.matches("data-blocks-app-shell-stacked-overlap-profile")
                .count(),
            4
        );
        assert_eq!(
            html.matches("アカウントメニューを開く").count(),
            4,
            "profile trigger should carry the visually-hidden label"
        );
    }

    /// [`LAYOUT_CSS`] が 48rem 境界・重なり用 `calc(-1 *`・帯 2 配色・
    /// `order` 不使用を満たすこと。
    #[test]
    fn layout_css_has_overlap_and_two_band_colors_without_order() {
        assert!(
            LAYOUT_CSS.contains("@container blocks-app-shell-stacked-overlap (min-width: 48rem)")
        );
        assert!(LAYOUT_CSS.contains(
            "margin-block-start: calc(-1 * var(--blocks-app-shell-stacked-overlap-overlap));"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-app-shell-stacked-overlap-variant=\"single-row\"] [data-blocks-app-shell-stacked-overlap-band] {\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);\n}"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-app-shell-stacked-overlap-variant=\"two-row-search\"] [data-blocks-app-shell-stacked-overlap-band] {\n  background: var(--fandhe-color-accent);\n  color: var(--fandhe-color-accent-fg);\n}"
        ));
        assert!(!LAYOUT_CSS.contains("{\n  order:"));
        assert!(!LAYOUT_CSS.contains(";\n  order:"));
    }

    /// root（clone されない）の `aria-label` は variant ごとに一意であり、
    /// nav（デスクトップ用ラッパー + パネル内 clone の 2 回）は同一 variant
    /// 内で重複しても非表示側が `display: none` で a11y ツリーから除外され
    /// 実害を持たないこと（`header_simple_bar` と同じ根拠）。
    #[test]
    fn root_and_nav_labels_are_unique_per_variant() {
        let html = render(&demo());
        for label in [
            "アプリシェル（1 段ナビ）",
            "アプリシェル（2 段ナビ + 検索）",
        ] {
            assert_eq!(
                html.matches(&format!("aria-label=\"{label}\"")).count(),
                1,
                "root label={label} should be unique, html={html}"
            );
        }
        for label in ["アプリ（1 段）", "アプリ（2 段）"] {
            assert_eq!(
                html.matches(&format!("aria-label=\"{label}\"")).count(),
                2,
                "nav label={label} should render twice (wrap + panel clone), html={html}"
            );
        }
    }
}
