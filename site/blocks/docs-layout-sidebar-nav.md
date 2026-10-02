# docs-layout-sidebar-nav

`fandhe-frontend-pre-styled-ui` の `nav-list` / `button` / `kbd` / `menu` /
`accordion` / `collapsible` / `switch` / `link` / `icon` / `drawer` 部品を
合成した、ドキュメントサイト用サイドバーの実例です。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください（主参照は対応表 ID R0085、集約元は
R0084・R0086・R0087・R0088。出典の固有名・ファイル名は記載しません）。

ロゴ + カラーモード切替、検索トリガー（ショートカット表示付き）、
ドキュメント／フレームワーク切替メニュー、外部リンク、ナビグループの順に
縦へ並べた構成です。アクティブ項目を線で示す形（line）と塗りで示す形
（filled）、狭い幅でメニューボタンからドロワーで開く形（narrow）の 3
インスタンスを並記しています。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。ボタンは `type="button"` のまま送信先を持たず、
メニュー・ドロワーは無 JS のため初期状態（閉 / 開のいずれか固定）を
掲示するのみです。文言はすべて独自に書いた架空のものであり、実企業名・
実クレデンシャル・PII を含みません。

## Rust コード

```rust
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

/// `nav_list::list` + `nav_list::item`/`link` の組み立て。
fn nav_items(items: NavItems) -> Node {
    let children: Vec<Node> = items
        .iter()
        .map(|(label, href, active)| {
            nav_list::item(
                vec![],
                vec![nav_list::link(href, *active, vec![], vec![text(*label)])],
            )
        })
        .collect();
    nav_list::list(vec![], children)
}

/// `collapsible` で開閉するナビグループ（`line` インスタンス、アクティブ
/// 項目は [`LAYOUT_CSS`] の線（border-inline-start）で示す）。
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
            nav_list::heading(vec![], vec![text(title)]),
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
        vec![geo_icon("M3 6h18M3 12h18M3 18h18")],
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
```

## 原案差分メモ

- 主参照は R0085（ロゴ + 検索 + 切替メニュー + 線型の現在位置表示）です。
  R0084（外部リンク + 塗り型のグループ）・R0086（モード切替 + アコーディオン）
  を `filled` インスタンスへ、R0087（開閉グループ閉じた状態）・R0088
  （フレームワーク切替 + 開閉 4 節）の要素を `line` インスタンスの構成
  （先頭グループのみ開く・切替メニューを持つ）へまとめて反映しています。
- 無 JS 制約のため、ナビグループの開閉・メニューの開閉・ドロワーの開閉は
  いずれも固定の初期状態（先頭グループのみ開く・メニュー/切替は閉じる・
  ドロワーのみ開いた状態を静的に示す）です。
- カラーモード切替・ドキュメント切替・検索・外部リンク以外のボタン・
  メニュー項目はすべて静的な初期状態の掲示にとどまり、実際のデータ取得・
  画面遷移は行いません。
- ブラウザでの実機確認（ドロワーの枠内収まり・ライト/ダーク両テーマ・
  狭幅ビューポートでの横スクロール）はサンドボックス制約により未実施
  です。cargo test による出力検証のみで代替しました。
