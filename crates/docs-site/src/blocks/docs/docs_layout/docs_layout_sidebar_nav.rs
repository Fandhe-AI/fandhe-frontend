//! `docs-layout-sidebar-nav` block（イシュー #3109、親 #3099）。
//!
//! # 使用部品
//!
//! `nav-list`（ナビグループの構造）/ `button`（検索トリガー・メニュー
//! ボタン）/ `kbd`（検索トリガーのショートカット表示）/ `menu`（ドキュメント
//! バージョン切替）/ `accordion`（塗り型インスタンスのナビグループ開閉）/
//! `collapsible`（線型インスタンスのナビグループ開閉）/ `switch`（カラー
//! モード切替）/ `link`（外部リンク）/ `icon`（ロゴ・検索・外部リンクの
//! 幾何アイコン）/ `drawer`（狭幅インスタンスのオーバーレイ）の 10 部品を
//! 合成する（`crate::blocks::Block::parts` に一致させる契約）。
//!
//! # 無 JS のため 2 つの開閉機構・3 インスタンスを静的に並置する
//!
//! docs サイトは JS ハイドレーションを行わない（CLAUDE.md）ため、本 Demo は
//! 開閉状態を実際に切り替えられない。アクティブ項目の示し方（線 / 塗り）と
//! 開閉機構（`collapsible` / `accordion`）の組み合わせを変えた 2 インスタンス
//! （`line`/`filled`）と、狭幅時に `drawer` でナビへ到達する形を示す
//! インスタンス（`narrow`）を縦に並べる（`sidebar-07`/`game-ui-modal` と
//! 同じ設計判断）。いずれも先頭のナビグループのみ開いた初期状態を描く。
//!
//! # `<form>` を使わない・全データが架空
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンはすべて `type="button"`。ブランド名・ドキュメント
//! バージョン名・ナビ項目名はすべて架空のものであり、実企業名・実在人物・
//! 実クレデンシャル・PII を含まない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `nav_list::root`/`button::button`/`menu::root`/`accordion::root`/
//! `switch::root`/`link::root`/`icon::icon`/`drawer::root` はいずれも
//! 呼び出し側 `attrs` の `class` を `drop_class_attr` により黙って除去する
//! （`crate::blocks::mod` モジュール doc「CSS フック」節参照）。これらへの
//! Demo 固有 CSS フックは `data-blocks-docs-layout-sidebar-nav-*` 属性で
//! 渡し、[`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。素の `div`/`span`・
//! `collapsible::*`（`class` 除去対象外）は `class` を使う。
//!
//! # `narrow` インスタンスの `drawer` 配置（`game-ui-modal` と同型）
//!
//! `drawer::positioner`/`backdrop` は本来 `position: fixed; inset: 0` の
//! ビューポート全体オーバーレイだが、Blocks の掲示は `.blocks-demo` 枠内へ
//! 収める必要がある。本 block スコープに限定した属性セレクタで
//! `position: absolute; inset: 0` へ差し替え、`[data-blocks-docs-layout-
//! sidebar-nav-frame="narrow"]`（`position: relative` を持つ）を包含
//! ブロックにする（[`LAYOUT_CSS`] 参照）。メニューボタンは常時展開の
//! `button::icon_button` とし（`app_shell_sidebar.rs` の「ハンバーガーは
//! 常時展開の trigger」と同じ判断）、`aria-expanded="true"` を固定で
//! 付与する静的表示に留める。
//!
//! # 狭幅フレーム（`18rem`）に対する `drawer::content` 幅の中和
//!
//! `drawer::content`（start placement）の既定 CSS は `width:
//! var(--fandhe-drawer-size, 20rem)` の固定幅を持つが、`narrow` インスタンス
//! の `[data-blocks-docs-layout-sidebar-nav-frame="narrow"]` は `18rem`・
//! `overflow: hidden` のため、そのままでは content の右端約 `2rem` が
//! 表示枠からはみ出して切れる（CI/codex(P1) 指摘）。[`LAYOUT_CSS`] の
//! `[data-blocks-docs-layout-sidebar-nav-frame="narrow"]
//! [data-scope="drawer"][data-part="content"]` へ `max-width: 100%` を
//! 追加し、フレーム幅を上限として収める（`--fandhe-drawer-size` 自体は
//! 変更せず、上限のみ掛ける）。
//!
//! # `drawer::title` のサイト共通 `h2` 装飾を中和する
//!
//! [`drawer::title`] は `h2` を描画するため、`.docs-content h2`
//! （`site_theme.rs`）の `border-top`/`padding-top`/`letter-spacing` を
//! 素のまま継承すると「Navigation」見出しに本文節区切りの罫線が漏れ出る
//! （`cart_drawer`/`settings_page_aside_nav` 等と同型の cursor(Medium)
//! 指摘）。[`LAYOUT_CSS`] は `[data-blocks-docs-layout-sidebar-nav-narrow-
//! content] [data-scope="drawer"][data-part="title"]` へ既存パターンと
//! 同じ `border-top: none; padding-top: 0; letter-spacing: normal;`
//! （+ `margin: 0`）を当てる。

use crate::blocks::dummy_assets;
use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{
    self, item as accordion_item, item_content as accordion_item_content,
    item_indicator as accordion_item_indicator, item_trigger as accordion_item_trigger,
    AccordionProps,
};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::collapsible;
use fandhe_frontend_pre_styled_ui::drawer::{self, ContentIds, DrawerPlacement};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::kbd::{self, KbdProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::nav_list;
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 自作の単純な幾何アイコン（`sidebar-07::geo_icon` と同型。実在ブランド
/// 由来の path データは使わない、モジュール冒頭 rustdoc 参照）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// ストローク専用の幾何アイコン（`store_nav_mega_menu::stroke_icon` と
/// 同型）。ハンバーガーメニューの `path` は面積を持たないストローク線
/// データのため、常に `fill="currentColor"` で塗る [`geo_icon`] へ渡すと
/// 不可視になる（Bugbot 指摘対応）。`fill="none"` + `stroke="currentColor"`
/// を明示し、`icon` 側の既定塗りを打ち消す。
fn stroke_icon(path_d: &'static str) -> Node {
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

/// ロゴ + カラーモード切替の header 行。`switch` は
/// `pricing_seats_split.rs` と同じく `readonly`/`disabled` を固定した
/// 静的表示（無 JS のため切替不可）。
fn logo_row(suffix: &str) -> Node {
    let brand = dummy_assets::COMPANY_NAMES[0];
    let switch_props = SwitchProps {
        readonly: true,
        disabled: true,
        ..SwitchProps::default()
    };
    let input_name = format!("dark-mode-{suffix}");
    let mode_switch = switch::root(
        Size::Sm,
        ColorPalette::Accent,
        false,
        &switch_props,
        vec![("data-blocks-docs-layout-sidebar-nav-mode", "")],
        vec![
            switch::label(false, &switch_props, vec![], vec![text("ダークモード")]),
            switch::hidden_input(&input_name, "on", false, &switch_props, vec![]),
            switch::control(
                false,
                &switch_props,
                vec![],
                vec![switch::thumb(false, &switch_props, vec![], vec![])],
            ),
        ],
    );
    div(
        vec![("class", "blocks-docs-layout-sidebar-nav-logo-row")],
        vec![
            span(
                vec![("class", "blocks-docs-layout-sidebar-nav-logo")],
                vec![
                    geo_icon("M12 2 L22 8 L22 16 L12 22 L2 16 L2 8 Z"),
                    span(vec![], vec![text(brand)]),
                ],
            ),
            mode_switch,
        ],
    )
}

/// 検索トリガー（`button` + `icon` + `kbd::group`）。押しても何も起きない
/// 静的ボタン（`<form>` を持たず `<input>` も使わない、モジュール冒頭
/// rustdoc 参照）。
fn search_trigger() -> Node {
    button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![("data-blocks-docs-layout-sidebar-nav-search", "")],
        vec![
            geo_icon("M10 2a8 8 0 1 0 4.9 14.3l5.4 5.4 1.4-1.4-5.4-5.4A8 8 0 0 0 10 2Zm0 2a6 6 0 1 1 0 12 6 6 0 0 1 0-12Z"),
            span(vec![], vec![text("Search docs")]),
            kbd::group(
                vec![],
                vec![
                    kbd::kbd(&KbdProps::default(), vec![], vec![text("\u{2318}")]),
                    kbd::kbd(&KbdProps::default(), vec![], vec![text("K")]),
                ],
            ),
        ],
    )
}

/// ドキュメント／フレームワーク切替メニュー（`menu`、閉じた静的表示）。
fn switcher_menu(suffix: &str) -> Node {
    let content_id = format!("blocks-docs-layout-sidebar-nav-switcher-menu-{suffix}");
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id.as_str()),
        vec![
            ("aria-label", "Switch documentation version"),
            ("data-blocks-docs-layout-sidebar-nav-switcher", ""),
        ],
        vec![
            span(vec![], vec![text("Guides v2")]),
            span(
                vec![("data-blocks-docs-layout-sidebar-nav-chevron", "")],
                vec![text("\u{2195}")],
            ),
        ],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item("v2", true, false, vec![], vec![text("Guides v2")]),
            menu::item("v1", false, false, vec![], vec![text("Guides v1")]),
            menu::separator(vec![], vec![]),
            menu::item(
                "framework-wasm",
                false,
                false,
                vec![],
                vec![text("Framework: WASM")],
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

/// 外部リンク（`link` + `icon`）。遷移先は実在のリポジトリ URL固定
/// （`href="#"` は使わない、モジュール冒頭 rustdoc 参照）。
fn external_link_row() -> Node {
    div(
        vec![("class", "blocks-docs-layout-sidebar-nav-external")],
        vec![
            geo_icon("M14 3h7v7h-2V6.4l-9.3 9.3-1.4-1.4L17.6 5H14V3ZM5 5h6v2H7v10h10v-4h2v6H5V5Z"),
            link::root(
                "https://github.com/Fandhe-AI/fandhe-frontend",
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![],
                vec![text("GitHub で見る")],
            ),
        ],
    )
}

/// ナビグループ 1 件分の項目一覧（`(label, href, active)`）。
type NavItems = &'static [(&'static str, &'static str, bool)];

const GUIDES_ITEMS: NavItems = &[
    ("はじめに", "../../guides/", true),
    (
        "コンポーネント作成",
        "../../guides/component-authoring/",
        false,
    ),
    ("デプロイ", "../../guides/deployment/", false),
];

const API_ITEMS: NavItems = &[
    ("Component API", "../../api/component-api/", false),
    ("Server API", "../../api/server-api/", false),
];

/// `nav_list::list` + `nav_list::item`/`link` の組み立て。`nav_list::link`
/// の `current` 引数には常に `false` を渡し `aria-current="page"` を
/// 付けない（`settings_page_aside_nav::aside_nav` と同型の判断。本 Demo は
/// `/blocks/docs-layout-sidebar-nav/` 上に掲示される架空のサイドバーで
/// あり、リンク先（`../../guides/` 等）は実在する docs ページだが
/// 「現在表示中のページ」ではないため、`aria-current="page"` を付けると
/// 支援技術に偽の現在地を伝えてしまう。視覚的な強調は
/// `data-blocks-docs-layout-sidebar-nav-current` 属性（[`LAYOUT_CSS`]）
/// のみに反映する）。
fn nav_items(items: NavItems) -> Node {
    let children: Vec<Node> = items
        .iter()
        .map(|(label, href, active)| {
            let link_attrs = if *active {
                vec![("data-blocks-docs-layout-sidebar-nav-current", "")]
            } else {
                vec![]
            };
            nav_list::item(
                vec![],
                vec![nav_list::link(href, false, link_attrs, vec![text(*label)])],
            )
        })
        .collect();
    nav_list::list(vec![], children)
}

/// `collapsible` で開閉するナビグループ（`line` インスタンス、アクティブ
/// 項目は [`LAYOUT_CSS`] の線（border-inline-start）で示す）。
///
/// `nav_list::heading`（固定 `<h2>`）でラップしない: `<button>` の内側に
/// `<h2>` を置くと content model 違反になる（旧実装の codex/Bugbot 指摘）
/// ため見出しを外側に置く修正を一度行ったが、外側に置いても `line`/
/// `filled`/`narrow` の 3 インスタンス分「Guides」「API」が重複する固定
/// `<h2>` としてページのアウトラインに現れ続ける問題は残る
/// （cursor(Low) 指摘。`content_article_toc`/`footer_sticky_reveal` が
/// 同じ理由で `nav_list::heading` を使わない判断と同型）。本関数は
/// `collapsible::trigger`（`<button>`）をラップせずそのまま
/// `collapsible::root` の子にする（`nav_list::root`/`list`/`item`/`link`
/// は引き続き使うため「Nav List」部品の使用自体は変わらない）。
fn nav_group_collapsible(
    suffix: &str,
    key: &str,
    title: &'static str,
    items: NavItems,
    open: OpenState,
) -> Node {
    let content_id = format!("blocks-docs-layout-sidebar-nav-{key}-content-{suffix}");
    let trigger = collapsible::trigger(
        open,
        false,
        Some(content_id.as_str()),
        vec![("class", "blocks-docs-layout-sidebar-nav-group-trigger")],
        vec![
            span(vec![], vec![text(title)]),
            collapsible::indicator(open, false, vec![], vec![text("\u{25be}")]),
        ],
    );
    let content = collapsible::content(
        open,
        false,
        Some(content_id.as_str()),
        vec![],
        vec![nav_items(items)],
    );
    let label = format!("{title}（{suffix}）");
    nav_list::root(
        &label,
        vec![("data-blocks-docs-layout-sidebar-nav-group", "")],
        vec![collapsible::root(
            open,
            false,
            vec![],
            vec![trigger, content],
        )],
    )
}

/// `accordion` で開閉するナビグループ（`filled` インスタンス、アクティブ
/// 項目は [`LAYOUT_CSS`] の面色の塗りで示す）。
fn nav_group_accordion(
    suffix: &str,
    key: &str,
    title: &'static str,
    items: NavItems,
    open: OpenState,
) -> Node {
    let props = AccordionProps::default();
    let trigger_id = format!("blocks-docs-layout-sidebar-nav-{key}-trigger-{suffix}");
    let content_id = format!("blocks-docs-layout-sidebar-nav-{key}-content-{suffix}");
    let item_node = accordion_item(
        open,
        false,
        &props,
        vec![],
        vec![
            el(
                "h3",
                vec![(
                    "class",
                    "blocks-docs-layout-sidebar-nav-group-trigger-heading",
                )],
                vec![accordion_item_trigger(
                    open,
                    false,
                    &props,
                    key,
                    Some(trigger_id.as_str()),
                    Some(content_id.as_str()),
                    vec![("class", "blocks-docs-layout-sidebar-nav-group-trigger")],
                    vec![
                        span(vec![], vec![text(title)]),
                        accordion_item_indicator(
                            open,
                            false,
                            &props,
                            vec![],
                            vec![text("\u{25be}")],
                        ),
                    ],
                )],
            ),
            accordion_item_content(
                open,
                false,
                &props,
                Some(content_id.as_str()),
                Some(trigger_id.as_str()),
                vec![],
                vec![nav_items(items)],
            ),
        ],
    );
    let label = format!("{title}（{suffix}）");
    nav_list::root(
        &label,
        vec![("data-blocks-docs-layout-sidebar-nav-group", "")],
        vec![accordion::root(Size::Sm, &props, vec![], vec![item_node])],
    )
}

/// `line`/`filled` インスタンス共通のサイドバー本体。`use_accordion` で
/// 開閉機構を切り替える（モジュール冒頭「無 JS のため 2 つの開閉機構・
/// 3 インスタンスを静的に並置する」節参照）。
fn sidebar_body(suffix: &str, use_accordion: bool) -> Node {
    let groups = if use_accordion {
        vec![
            nav_group_accordion(suffix, "guides", "Guides", GUIDES_ITEMS, OpenState::Open),
            nav_group_accordion(suffix, "api", "API", API_ITEMS, OpenState::Closed),
        ]
    } else {
        vec![
            nav_group_collapsible(suffix, "guides", "Guides", GUIDES_ITEMS, OpenState::Open),
            nav_group_collapsible(suffix, "api", "API", API_ITEMS, OpenState::Closed),
        ]
    };
    let mut children = vec![
        logo_row(suffix),
        search_trigger(),
        switcher_menu(suffix),
        external_link_row(),
    ];
    children.extend(groups);
    div(
        vec![(
            "data-blocks-docs-layout-sidebar-nav-panel",
            if use_accordion { "filled" } else { "line" },
        )],
        children,
    )
}

/// 狭幅インスタンス（`drawer` で開いたサイドバー本体を静的に掲示する、
/// モジュール冒頭「`narrow` インスタンスの `drawer` 配置」節参照）。
fn narrow_frame() -> Node {
    let title_id = "blocks-docs-layout-sidebar-nav-narrow-title";
    let content_id = "blocks-docs-layout-sidebar-nav-narrow-content";
    let menu_button = button::icon_button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        "Open navigation",
        vec![
            ("aria-expanded", "true"),
            ("aria-controls", content_id),
            ("data-blocks-docs-layout-sidebar-nav-menu-button", ""),
        ],
        vec![stroke_icon("M3 6h18M3 12h18M3 18h18")],
    );
    let top_bar = div(
        vec![("class", "blocks-docs-layout-sidebar-nav-narrow-topbar")],
        vec![
            menu_button,
            span(vec![], vec![text(dummy_assets::COMPANY_NAMES[0])]),
        ],
    );
    let drawer_content = drawer::content(
        OpenState::Open,
        DrawerPlacement::Start,
        false,
        ContentIds {
            id: Some(content_id),
            labelledby: Some(title_id),
            describedby: None,
        },
        vec![("data-blocks-docs-layout-sidebar-nav-narrow-content", "")],
        vec![
            drawer::title(Some(title_id), vec![], vec![text("Navigation")]),
            sidebar_body("narrow", false),
        ],
    );
    let drawer_node = drawer::root(
        Size::Sm,
        OpenState::Open,
        DrawerPlacement::Start,
        vec![("data-blocks-docs-layout-sidebar-nav-narrow-root", "")],
        vec![
            drawer::backdrop(OpenState::Open, vec![], vec![]),
            drawer::positioner(
                OpenState::Open,
                DrawerPlacement::Start,
                vec![],
                vec![drawer_content],
            ),
        ],
    );
    div(
        vec![("data-blocks-docs-layout-sidebar-nav-frame", "narrow")],
        vec![top_bar, drawer_node],
    )
}

/// `docs-layout-sidebar-nav` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。`line`（主参照）→ `filled` → `narrow` の順に縦へ並べる。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-docs-layout-sidebar-nav-layout")],
        vec![
            p(
                vec![("class", "blocks-docs-layout-sidebar-nav-caption")],
                vec![text("Line（アクティブ項目を線で示す）")],
            ),
            sidebar_body("line", false),
            p(
                vec![("class", "blocks-docs-layout-sidebar-nav-caption")],
                vec![text("Filled（アクティブ項目を塗りで示す）")],
            ),
            sidebar_body("filled", true),
            p(
                vec![("class", "blocks-docs-layout-sidebar-nav-caption")],
                vec![text("Narrow（狭幅時は drawer で開く）")],
            ),
            narrow_frame(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/docs-layout-sidebar-nav/",
    title: "docs-layout-sidebar-nav",
    category: BlockCategory::DocsLayout,
    rust_source: "crates/docs-site/src/blocks/docs/docs_layout/docs_layout_sidebar_nav.rs",
    demo_class: "blocks-docs-layout-sidebar-nav",
    parts: &[
        Part {
            label: "Nav List",
            path: "/themes/nav-list/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Kbd",
            path: "/themes/kbd/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Accordion",
            path: "/themes/accordion/",
        },
        Part {
            label: "Collapsible",
            path: "/themes/collapsible/",
        },
        Part {
            label: "Switch",
            path: "/themes/switch/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Drawer",
            path: "/themes/drawer/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// [`super::mod@self`] の `stylesheet()` が `push_css` する block 固有の
/// レイアウト CSS（`docs/design/docs-site-blocks-section.md` §10 追記節に
/// 従い、並列進行する他 block との `mod.rs::LAYOUT_CSS` 追記衝突を避け
/// 本モジュール側の定数へ分離する）。
const LAYOUT_CSS: &str = "\
.blocks-docs-layout-sidebar-nav.blocks-demo {\n  overflow-x: auto;\n}\n\
.blocks-docs-layout-sidebar-nav-layout {\n  display: flex;\n  flex-direction: column;\n  gap: 1rem;\n}\n\
.blocks-docs-layout-sidebar-nav-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-docs-layout-sidebar-nav-panel] {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n  width: 18rem;\n  padding: 1rem;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg);\n}\n\
.blocks-docs-layout-sidebar-nav-logo-row {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: 0.5rem;\n}\n\
.blocks-docs-layout-sidebar-nav-logo {\n  display: flex;\n  align-items: center;\n  gap: 0.5rem;\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n}\n\
[data-blocks-docs-layout-sidebar-nav-search] {\n  justify-content: space-between;\n  width: 100%;\n}\n\
[data-blocks-docs-layout-sidebar-nav-switcher] {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  width: 100%;\n  padding: 0.5rem;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  background: transparent;\n  color: inherit;\n  font: inherit;\n  cursor: pointer;\n}\n\
.blocks-docs-layout-sidebar-nav-external {\n  display: flex;\n  align-items: center;\n  gap: 0.5rem;\n  padding: 0.25rem 0;\n}\n\
.blocks-docs-layout-sidebar-nav-group-trigger {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  width: 100%;\n  background: transparent;\n  border: none;\n  cursor: pointer;\n  padding: 0.25rem 0;\n  color: inherit;\n  font: inherit;\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n}\n\
.blocks-docs-layout-sidebar-nav-group-trigger-heading {\n  margin: 0;\n}\n\
[data-blocks-docs-layout-sidebar-nav-group] [data-scope=\"nav-list\"][data-part=\"list\"] {\n  display: flex;\n  flex-direction: column;\n  gap: 0.25rem;\n  margin: 0.25rem 0 0;\n  padding-inline-start: 0.5rem;\n}\n\
[data-blocks-docs-layout-sidebar-nav-panel=\"line\"] [data-scope=\"nav-list\"][data-part=\"link\"] {\n  display: block;\n  padding: 0.25rem 0.5rem;\n  border-inline-start: 2px solid transparent;\n  text-decoration: none;\n  color: inherit;\n}\n\
[data-blocks-docs-layout-sidebar-nav-panel=\"line\"] [data-scope=\"nav-list\"][data-part=\"link\"][data-blocks-docs-layout-sidebar-nav-current] {\n  border-inline-start-color: var(--fandhe-color-accent);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n}\n\
[data-blocks-docs-layout-sidebar-nav-panel=\"filled\"] [data-scope=\"nav-list\"][data-part=\"link\"] {\n  display: block;\n  padding: 0.25rem 0.5rem;\n  border-radius: var(--fandhe-radius-md);\n  text-decoration: none;\n  color: inherit;\n}\n\
[data-blocks-docs-layout-sidebar-nav-panel=\"filled\"] [data-scope=\"nav-list\"][data-part=\"link\"][data-blocks-docs-layout-sidebar-nav-current] {\n  background: var(--fandhe-color-bg-muted);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n}\n\
[data-blocks-docs-layout-sidebar-nav-frame=\"narrow\"] {\n  position: relative;\n  width: 18rem;\n  min-height: 24rem;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  overflow: hidden;\n}\n\
.blocks-docs-layout-sidebar-nav-narrow-topbar {\n  display: flex;\n  align-items: center;\n  gap: 0.5rem;\n  padding: 0.75rem;\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-docs-layout-sidebar-nav-frame=\"narrow\"] [data-scope=\"drawer\"][data-part=\"backdrop\"] {\n  position: absolute;\n  inset: 0;\n  z-index: 1;\n}\n\
[data-blocks-docs-layout-sidebar-nav-frame=\"narrow\"] [data-scope=\"drawer\"][data-part=\"positioner\"] {\n  position: absolute;\n  inset: 0;\n  z-index: 2;\n  display: flex;\n}\n\
[data-blocks-docs-layout-sidebar-nav-frame=\"narrow\"] [data-scope=\"drawer\"][data-part=\"content\"] {\n  height: 100%;\n  max-width: 100%;\n  box-shadow: var(--fandhe-shadow-lg);\n}\n\
[data-blocks-docs-layout-sidebar-nav-narrow-content] [data-blocks-docs-layout-sidebar-nav-panel] {\n  width: 100%;\n  height: 100%;\n  border: none;\n  border-radius: 0;\n}\n\
[data-blocks-docs-layout-sidebar-nav-narrow-content] [data-scope=\"drawer\"][data-part=\"title\"] {\n  margin: 0;\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_contains_all_ten_declared_parts() {
        let html = html();
        for scope in [
            "data-scope=\"nav-list\"",
            "data-scope=\"menu\"",
            "data-scope=\"accordion\"",
            "data-scope=\"collapsible\"",
            "data-scope=\"switch\"",
            "data-scope=\"link\"",
            "data-scope=\"drawer\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(
            html.contains("type=\"button\""),
            "demo should use button parts"
        );
        assert!(html.contains("<kbd"), "demo should use kbd part");
        assert!(html.contains("<svg"), "demo should use icon part");
    }

    #[test]
    fn demo_has_no_form_or_dead_links_or_data_uri() {
        let html = html();
        for absent in ["<form", "href=\"#\"", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should not contain {absent}");
        }
    }

    #[test]
    fn all_buttons_are_type_button() {
        let html = html();
        let button_open_count = html.matches("<button").count();
        let type_button_count = html.matches("type=\"button\"").count();
        assert_eq!(
            button_open_count, type_button_count,
            "every <button> should carry type=\"button\""
        );
    }

    #[test]
    fn external_link_has_safe_rel() {
        let html = html();
        assert!(html.contains("rel=\"noopener noreferrer\""));
        assert!(html.contains("target=\"_blank\""));
    }

    #[test]
    fn line_instance_has_exactly_one_open_collapsible_group() {
        let html = html();
        // `line` インスタンス（panel="line"）の範囲だけを切り出して数える。
        let start = html
            .find("data-blocks-docs-layout-sidebar-nav-panel=\"line\"")
            .unwrap();
        let end = html[start..]
            .find("data-blocks-docs-layout-sidebar-nav-panel=\"filled\"")
            .map(|i| start + i)
            .unwrap_or(html.len());
        let segment = &html[start..end];
        let open_count = segment
            .matches("data-scope=\"collapsible\" data-part=\"root\" data-state=\"open\"")
            .count();
        assert_eq!(
            open_count, 1,
            "line instance should have exactly one open collapsible group"
        );
    }

    #[test]
    fn demo_never_uses_aria_current_page() {
        // リンク先（`../../guides/` 等）は実在する docs ページだが「現在
        // 表示中のページ」ではないため `aria-current="page"` を付けない
        // （`settings_page_aside_nav` と同型の判断、モジュール冒頭
        // `nav_items` rustdoc 参照。codex P2 指摘対応）。
        let html = html();
        assert!(!html.contains("aria-current=\"page\""));
        assert_eq!(
            html.matches("data-blocks-docs-layout-sidebar-nav-current")
                .count(),
            3,
            "exactly one visually-highlighted current item per instance (line/filled/narrow)"
        );
    }

    #[test]
    fn each_nav_has_exactly_one_aria_current_page() {
        let html = html();
        // 各 `nav`（nav-list の root）内で aria-current="page" がちょうど 1 件。
        let navs: Vec<&str> = html.split("<nav").skip(1).collect();
        for nav in navs {
            let count = nav.matches("aria-current=\"page\"").count();
            assert!(
                count <= 1,
                "each nav should have at most one aria-current=\"page\", found {count}"
            );
        }
    }

    #[test]
    fn ids_are_unique_and_aria_references_resolve() {
        let html = html();
        let mut ids = Vec::new();
        let needle = "id=\"";
        let mut offset = 0usize;
        while let Some(rel) = html[offset..].find(needle) {
            let start = offset + rel + needle.len();
            let Some(end_rel) = html[start..].find('"') else {
                break;
            };
            ids.push(&html[start..start + end_rel]);
            offset = start + end_rel;
        }
        let id_set: std::collections::BTreeSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), id_set.len(), "ids should be unique: {ids:?}");
    }
}
