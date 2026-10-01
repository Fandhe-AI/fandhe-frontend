//! `filter-sidebar` block（イシュー #3049。親 #3024「Blocks EC 系」配下）。
//! Ecommerce / Filter カテゴリ最初の block（`filter/mod.rs` の雛形を本 block
//! で卒業、イシュー #2734 §18）。
//!
//! # 使用部品
//!
//! `heading` / `menu` / `link` / `collapsible` / `fieldset` / `checkbox` /
//! `drawer` / `button` / `skeleton` の 9 部品のみを合成する（[`BLOCK`] の
//! `parts` に一致させる契約）。新しい UI 部品は追加しない。
//!
//! # 3 版の対応（イシュー本文の参照 ID）
//!
//! - **版 A（[`instance_accordion`]、主参照 R0818）**: カテゴリリンク +
//!   `collapsible` の開閉式フィルタ群 3 つ。無 JS では閉じた群を開けず
//!   選択肢に到達できなくなるため、静的表示は全群 open で描画する
//!   （「無 JS・到達性の方針」節参照）。
//! - **版 B（[`instance_expanded`]、R0819）**: `collapsible` を使わず
//!   `fieldset::root` + `legend` で 3 群を常時展開する。
//! - **版 C（[`instance_narrow`]、R0621）**: 版 A と同じ構成だが、
//!   インスタンス枠の幅を [`LAYOUT_CSS`] で固定し、ビューポートに関係なく
//!   狭い幅の表示（サイドバー非表示 + ドロワー）を見せる。
//!
//! # 無 JS・到達性の方針
//!
//! docs サイトは JS ハイドレーションを行わないため、開閉・並び替え・
//! ドロワーの実際の動作は表現できない。
//!
//! - `collapsible` の root/trigger/indicator/content は `disabled: true` で
//!   固定する（`form_layout_property_panel` と同じ判断。押しても何も
//!   起きない静的表示）。`disabled: true` のトリガーは開閉操作ができない
//!   ため、版 A/C は 3 群すべてを [`collapsible::OpenState::Open`] で
//!   固定する（`content` は closed のとき `hidden` 属性を付与する仕様
//!   `crates/headless-ui/src/collapsible.rs` のため、どれか 1 群でも
//!   closed にすると無 JS では当該群の選択肢へ到達できなくなる）。
//! - `checkbox` はすべて `CheckboxProps { disabled: true, .. }` にし、
//!   ネイティブの切り替えで見た目と `data-state` がずれるのを構造的に
//!   防ぐ（`form_layout_stacked` と同じ判断）。disabled の既定の薄い表示
//!   （opacity）は [`LAYOUT_CSS`] で打ち消す。
//! - `menu`（並び替え）は `OpenState::Closed` に固定する
//!   （`settings_page_tabs::key_actions_menu` と同じ判断）。trigger は
//!   `disabled: true` にし（押しても並び替えが実行できないため）、
//!   checkbox/collapsible と同様に既定の薄い表示（opacity）は
//!   [`LAYOUT_CSS`] で打ち消す。
//! - 広い幅ではサイドバーを表示しドロワー関連は非表示、狭い幅
//!   （`@container blocks-filter-sidebar (max-width: 40rem)`）ではその逆に
//!   する。これでどちらの幅でもフィルタ・カテゴリへ必ず到達できる
//!   （`app_shell_sidebar` と同じ方針）。各インスタンスにサイドバーと
//!   ドロワーの両方を持たせることで、個別インスタンスの幅に応じて
//!   実際にどちらが使われるかを決定的に示す。
//! - ドロワーは常に静的な開状態（[`drawer::OpenState::Open`]）で描画する
//!   （`cart_drawer`/`game_ui_modal` と同じ判断）。`modal: false` にして
//!   静的デモの外側の説明・コードを支援技術から隠さない。
//!
//! # ドロワーを枠内に収める理由
//!
//! `drawer::backdrop`/`drawer::positioner` の既定 CSS は
//! `position: fixed; inset: 0` のビューポート全体オーバーレイだが、Blocks
//! の掲示は Demo 枠内に収める必要がある。本 block スコープの属性セレクタで
//! `position: absolute; inset: 0; z-index: auto` へ差し替え、インスタンス
//! 自身を `position: relative` にして受け皿にする（`cart_drawer` と
//! 同型の手法）。この差し替えは「ドロワーが閉じて見える」既定状態
//! （`display: none`）向けであり、狭い幅の @container 規則内では次項の
//! 理由によりさらに上書きする。
//!
//! # 狭い幅でドロワーをオーバーレイにしない理由（PR #3504 レビュー指摘）
//!
//! ドロワーは常時 `OpenState::Open`（前項）のため、狭い幅で上記の
//! オーバーレイ配置のままだと商品領域を覆ったまま閉じる手段がなく
//! （無 JS のため close-trigger は操作できない）、カテゴリ「すべて」
//! リンクで商品領域へ移動しても見えなくなる。そのため狭い幅でのみ
//! `positioner` を `position: static` の通常フロー要素に戻す
//! （`backdrop` は常に非表示のまま。オーバーレイをやめた以上、暗幕を
//! 点ける意味がない）。`filter_drawer` は [`instance`] 内で商品グリッド
//! （`body`）の後ろに配置しているため、通常フローに戻すとドロワーの
//! 内容が商品領域の下に続けて表示され、両者が重ならず両方へ到達できる。
//!
//! # `href` の方針
//!
//! 商品領域は単一の固定カードセットでカテゴリ別の絞り込みを持たないため、
//! 全カテゴリを同一 href のリンクにすると「カテゴリを選んでも同じ場所へ
//! 移動する」という誤解を招く（PR #3504 レビュー指摘）。先頭の「すべて」
//! のみ、同じインスタンスの商品領域 id へのページ内フラグメント
//! （`#blocks-filter-sidebar-a-products` 等、`aria-current="page"` で選択中を
//! 示す）を持つ実際のリンクとし（`linkcheck` が同一ページの id 集合と
//! 突き合わせて検証できる、`content_article_toc` と同じ判断）、残りの
//! カテゴリは行き先を持たないプレーンテキスト（`span`）で示す
//! （[`category_nav`] 参照）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンはすべて既定の `type="button"` のまま用いる。
//!
//! # ダミー素材について
//!
//! カテゴリ名・商品名は独自の架空の文言であり、実在の商品・企業とは無関係。
//! 商品画像は [`skeleton`] の占位表示のみで、`data:` URI も外部 URL も
//! 使わない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{aside, div, li, nav, p, section, span, text, ul, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::collapsible;
use fandhe_frontend_pre_styled_ui::drawer::{
    self, CloseTriggerVariant, ContentIds, DrawerPlacement, OpenState,
};
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps, FieldsetRootProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState as MenuOpenState};
use fandhe_frontend_pre_styled_ui::skeleton::{skeleton, SkeletonProps, SkeletonVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// カテゴリ一覧（架空）。`(表示名,)` のみを持つ。
const CATEGORIES: &[&str] = &["すべて", "トップス", "ボトムス", "アウター", "小物"];

/// 1 件のチェックボックス選択肢（架空）。
struct Option_ {
    value: &'static str,
    label: &'static str,
    checked: bool,
}

/// カラー群の選択肢。
const COLOR_OPTIONS: &[Option_] = &[
    Option_ {
        value: "navy",
        label: "ネイビー",
        checked: true,
    },
    Option_ {
        value: "beige",
        label: "ベージュ",
        checked: false,
    },
    Option_ {
        value: "black",
        label: "ブラック",
        checked: false,
    },
];

/// サイズ群の選択肢。
const SIZE_OPTIONS: &[Option_] = &[
    Option_ {
        value: "s",
        label: "S",
        checked: false,
    },
    Option_ {
        value: "m",
        label: "M",
        checked: true,
    },
    Option_ {
        value: "l",
        label: "L",
        checked: false,
    },
];

/// 価格帯群の選択肢。
const PRICE_OPTIONS: &[Option_] = &[
    Option_ {
        value: "under-5000",
        label: "〜5,000 円",
        checked: false,
    },
    Option_ {
        value: "5000-to-15000",
        label: "5,000〜15,000 円",
        checked: false,
    },
    Option_ {
        value: "over-15000",
        label: "15,000 円〜",
        checked: false,
    },
];

/// チェックボックス 1 件（無 JS のためネイティブ `disabled` で固定、
/// モジュール doc「無 JS・到達性の方針」節）。
fn checkbox_item(name: &str, opt: &Option_) -> Node {
    let props = CheckboxProps {
        checked: if opt.checked {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        },
        disabled: true,
        ..CheckboxProps::default()
    };
    checkbox::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-filter-sidebar-checkbox", "")],
        vec![
            checkbox::hidden_input(&props, name, opt.value, vec![]),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
            checkbox::label(&props, vec![], vec![text(opt.label)]),
        ],
    )
}

/// フィルタ群 1 つ（カラー/サイズ/価格帯）を `collapsible` で開閉式に
/// 包む（版 A/C、`form_layout_property_panel::panel_theme_collapsible` と
/// 同型）。
fn accordion_group(
    name: &str,
    label: &'static str,
    content_id: &str,
    open: bool,
    options: &[Option_],
) -> Node {
    let state = if open {
        collapsible::OpenState::Open
    } else {
        collapsible::OpenState::Closed
    };
    let items: Vec<Node> = options.iter().map(|opt| checkbox_item(name, opt)).collect();
    collapsible::root(
        state,
        true,
        vec![],
        vec![
            collapsible::trigger(
                state,
                true,
                Some(content_id),
                vec![],
                vec![
                    text(label),
                    collapsible::indicator(state, true, vec![], vec![]),
                ],
            ),
            collapsible::content(
                state,
                true,
                Some(content_id),
                vec![("data-blocks-filter-sidebar-group", "")],
                items,
            ),
        ],
    )
}

/// フィルタ群 1 つを `fieldset` + `legend` で常時展開する（版 B）。
fn fieldset_group(id: &str, name: &str, label: &'static str, options: &[Option_]) -> Node {
    let fieldset_props = FieldsetProps {
        id,
        disabled: false,
        invalid: false,
        has_helper_text: false,
    };
    let mut children: Vec<Node> =
        vec![fieldset::legend(&fieldset_props, vec![], vec![text(label)])];
    children.extend(options.iter().map(|opt| checkbox_item(name, opt)));
    fieldset::root(
        &FieldsetRootProps::default(),
        &fieldset_props,
        vec![("data-blocks-filter-sidebar-group", "")],
        children,
    )
}

/// カテゴリナビ（同インスタンスの商品領域 id へのフラグメント、
/// モジュール doc「`href` の方針」節）。
///
/// 本 Demo は実際のカテゴリ別フィルタリングを持たず、商品領域は単一の
/// 固定カードセットのため、全カテゴリを同一 href のリンクにすると
/// 「カテゴリを選んでも同じ場所へ移動する」という誤解を招く
/// （PR #3504 レビュー指摘）。先頭の「すべて」のみ実際のリンク
/// （`aria-current="page"` で選択中を示す）とし、残りは行き先を持たない
/// プレーンテキストとして示す。
fn category_nav(products_id: &str) -> Node {
    let href = format!("#{products_id}");
    let items: Vec<Node> = CATEGORIES
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let child = if i == 0 {
                link::root(
                    href.as_str(),
                    &LinkProps::default(),
                    vec![("aria-current", "page")],
                    vec![text(*label)],
                )
            } else {
                span(vec![], vec![text(*label)])
            };
            li(vec![], vec![child])
        })
        .collect();
    nav(
        vec![
            ("aria-label", "カテゴリ"),
            ("class", "blocks-filter-sidebar-category-nav"),
        ],
        vec![ul(vec![], items)],
    )
}

/// フィルタパネル本体（カテゴリナビ + 3 群 + クリアボタン）。
/// `prefix` はサイドバー/ドロワーで異なる値を渡し id を一意にする
/// （モジュール doc「ID の一意性」節相当）。
fn filter_panel(prefix: &str, products_id: &str, accordion: bool) -> Node {
    let color_name = format!("{prefix}-color");
    let size_name = format!("{prefix}-size");
    let price_name = format!("{prefix}-price");
    let color_id = format!("{prefix}-color-content");
    let size_id = format!("{prefix}-size-content");
    let price_id = format!("{prefix}-price-content");
    let groups: Vec<Node> = if accordion {
        // 無 JS では disabled な trigger を開閉できず、closed な群は
        // `hidden` 属性で選択肢ごと到達不能になる（モジュール doc
        // 「無 JS・到達性の方針」節）。3 群すべてを open 固定する。
        vec![
            accordion_group(&color_name, "カラー", &color_id, true, COLOR_OPTIONS),
            accordion_group(&size_name, "サイズ", &size_id, true, SIZE_OPTIONS),
            accordion_group(&price_name, "価格帯", &price_id, true, PRICE_OPTIONS),
        ]
    } else {
        vec![
            fieldset_group(&color_id, &color_name, "カラー", COLOR_OPTIONS),
            fieldset_group(&size_id, &size_name, "サイズ", SIZE_OPTIONS),
            fieldset_group(&price_id, &price_name, "価格帯", PRICE_OPTIONS),
        ]
    };
    let mut children = vec![category_nav(products_id)];
    children.extend(groups);
    children.push(button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![],
        vec![text("条件をクリア")],
    ));
    div(vec![("class", "blocks-filter-sidebar-panel")], children)
}

/// 商品カード 1 枚（画像占位 + テキスト占位 2 行）。
fn product_card() -> Node {
    div(
        vec![("class", "blocks-filter-sidebar-card")],
        vec![
            skeleton(
                &SkeletonProps {
                    variant: SkeletonVariant::Rect,
                    ..SkeletonProps::default()
                },
                vec![("data-blocks-filter-sidebar-card-image", "")],
            ),
            skeleton(&SkeletonProps::default(), vec![]),
            skeleton(&SkeletonProps::default(), vec![]),
        ],
    )
}

/// 商品領域（6 枚のカード）。
fn product_area(products_id: &str) -> Node {
    section(
        vec![
            ("id", products_id),
            ("aria-label", "商品一覧"),
            ("class", "blocks-filter-sidebar-products"),
        ],
        (0..6).map(|_| product_card()).collect(),
    )
}

/// 並び替えメニュー（`settings_page_tabs::key_actions_menu` と同型、
/// `OpenState::Closed` 固定）。
fn sort_menu(prefix: &str) -> Node {
    let content_id = format!("{prefix}-sort-content");
    let trigger = menu::trigger(
        MenuOpenState::Closed,
        true,
        Some(content_id.as_str()),
        vec![],
        vec![text("並び替え")],
    );
    let content = menu::content(
        MenuOpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item(
                "recommended",
                false,
                false,
                vec![],
                vec![text("おすすめ順")],
            ),
            menu::item("newest", false, false, vec![], vec![text("新着順")]),
            menu::item(
                "price-asc",
                false,
                false,
                vec![],
                vec![text("価格の安い順")],
            ),
            menu::item(
                "price-desc",
                false,
                false,
                vec![],
                vec![text("価格の高い順")],
            ),
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

/// フィルタドロワーのトリガー（ヘッダー行に置く）。
fn drawer_trigger(content_id: &str) -> Node {
    drawer::trigger(
        OpenState::Open,
        Some(content_id),
        vec![("class", "blocks-filter-sidebar-drawer-trigger")],
        vec![text("フィルタ")],
    )
}

/// ヘッダー行（見出し + ドロワートリガー + 並び替えメニュー）。
fn header_row(prefix: &str, heading_text: &'static str, drawer_content_id: &str) -> Node {
    div(
        vec![("class", "blocks-filter-sidebar-header")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text(heading_text)],
            ),
            div(
                vec![("class", "blocks-filter-sidebar-header-actions")],
                vec![drawer_trigger(drawer_content_id), sort_menu(prefix)],
            ),
        ],
    )
}

/// フィルタドロワー（狭い幅のみ表示。モジュール doc「ドロワーを枠内に
/// 収める理由」節）。
fn filter_drawer(
    prefix: &str,
    products_id: &str,
    content_id: &str,
    title_id: &str,
    accordion: bool,
) -> Node {
    drawer::root(
        Size::Sm,
        OpenState::Open,
        DrawerPlacement::Start,
        vec![("class", "blocks-filter-sidebar-drawer")],
        vec![
            drawer::backdrop(OpenState::Open, vec![], vec![]),
            drawer::positioner(
                OpenState::Open,
                DrawerPlacement::Start,
                vec![],
                vec![drawer::content(
                    OpenState::Open,
                    DrawerPlacement::Start,
                    false,
                    ContentIds {
                        id: Some(content_id),
                        labelledby: Some(title_id),
                        describedby: None,
                    },
                    vec![],
                    vec![
                        drawer::title(Some(title_id), vec![], vec![text("フィルタ")]),
                        filter_panel(&format!("{prefix}-drawer"), products_id, accordion),
                        drawer::close_trigger_with_variant(
                            CloseTriggerVariant::Text,
                            vec![],
                            vec![text("閉じる")],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// 1 インスタンス分の組み立て（見出し行 + 本体グリッド + ドロワー）。
fn instance(prefix: &str, heading_text: &'static str, accordion: bool) -> Node {
    let products_id = format!("blocks-filter-sidebar-{prefix}-products");
    let drawer_content_id = format!("blocks-filter-sidebar-{prefix}-drawer-content");
    let drawer_title_id = format!("blocks-filter-sidebar-{prefix}-drawer-title");
    div(
        vec![("class", "blocks-filter-sidebar-instance")],
        vec![
            header_row(prefix, heading_text, &drawer_content_id),
            div(
                vec![("class", "blocks-filter-sidebar-body")],
                vec![
                    aside(
                        vec![
                            ("aria-label", "商品フィルタ"),
                            ("class", "blocks-filter-sidebar-aside"),
                        ],
                        vec![filter_panel(prefix, &products_id, accordion)],
                    ),
                    product_area(&products_id),
                ],
            ),
            filter_drawer(
                prefix,
                &products_id,
                &drawer_content_id,
                &drawer_title_id,
                accordion,
            ),
        ],
    )
}

/// 版キャプション（素の `p` + class、styled `text` は `class` を落とす
/// ため使わない）。
fn version_caption(label: &'static str) -> Node {
    p(
        vec![("class", "blocks-filter-sidebar-caption")],
        vec![text(label)],
    )
}

/// `filter-sidebar` の Demo 本体。呼び出しごとに決定的な `Node` を返す
/// 純関数。版 A/B/C（モジュール doc「3 版の対応」節）を縦に並べる。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-filter-sidebar-wrap")],
        vec![
            version_caption("開閉式フィルタ（R0818、代表構成）"),
            instance("a", "新着アイテム", true),
            version_caption("常時展開フィルタ（R0819）"),
            instance("b", "新着アイテム", false),
            version_caption("狭い幅: サイドバーが隠れドロワーに切り替わる（R0621）"),
            div(
                vec![("class", "blocks-filter-sidebar-narrow-stage")],
                vec![instance("c", "新着アイテム", true)],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/filter-sidebar/",
    title: "filter-sidebar",
    category: BlockCategory::Filter,
    rust_source: "crates/docs-site/src/blocks/ecommerce/filter/filter_sidebar.rs",
    demo_class: "blocks-filter-sidebar",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Collapsible",
            path: "/themes/collapsible/",
        },
        Part {
            label: "Fieldset",
            path: "/themes/fieldset/",
        },
        Part {
            label: "Checkbox",
            path: "/themes/checkbox/",
        },
        Part {
            label: "Drawer",
            path: "/themes/drawer/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Skeleton",
            path: "/themes/skeleton/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `filter_sidebar` 固有のレイアウト規則（モジュール doc「無 JS・到達性の
/// 方針」「ドロワーを枠内に収める理由」節の実装）。
const LAYOUT_CSS: &str = "\
.blocks-filter-sidebar-wrap {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-filter-sidebar .blocks-filter-sidebar-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-filter-sidebar-instance {\n  position: relative;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  container-type: inline-size;\n  container-name: blocks-filter-sidebar;\n}\n\
.blocks-filter-sidebar-narrow-stage {\n  max-width: 24rem;\n}\n\
.blocks-filter-sidebar-header {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-filter-sidebar-header-actions {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-filter-sidebar-body {\n  display: grid;\n  grid-template-columns: 14rem 1fr;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-filter-sidebar-panel {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-filter-sidebar-category-nav ul {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-filter-sidebar [data-blocks-filter-sidebar-group] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-filter-sidebar [data-scope=\"checkbox\"][data-part=\"root\"][data-blocks-filter-sidebar-checkbox][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-filter-sidebar [data-scope=\"collapsible\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-filter-sidebar [data-scope=\"collapsible\"][data-part=\"content\"][data-disabled] {\n  color: var(--fandhe-color-fg);\n}\n\
.blocks-filter-sidebar [data-scope=\"menu\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-filter-sidebar-products {\n  display: grid;\n  grid-template-columns: repeat(3, 1fr);\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-filter-sidebar-card {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-filter-sidebar [data-scope=\"skeleton\"][data-blocks-filter-sidebar-card-image] {\n  width: 100%;\n  height: 6rem;\n}\n\
.blocks-filter-sidebar-drawer-trigger {\n  display: none;\n}\n\
.blocks-filter-sidebar [data-scope=\"drawer\"][data-part=\"backdrop\"] {\n  position: absolute;\n  inset: 0;\n  z-index: auto;\n  display: none;\n}\n\
.blocks-filter-sidebar [data-scope=\"drawer\"][data-part=\"positioner\"] {\n  position: absolute;\n  inset: 0;\n  z-index: auto;\n  display: none;\n}\n\
.blocks-filter-sidebar [data-scope=\"drawer\"][data-part=\"title\"] {\n  margin: 0;\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n\
@container blocks-filter-sidebar (max-width: 40rem) {\n  .blocks-filter-sidebar-aside {\n    display: none;\n  }\n  .blocks-filter-sidebar-drawer-trigger {\n    display: inline-flex;\n  }\n  .blocks-filter-sidebar-body {\n    grid-template-columns: 1fr;\n  }\n  .blocks-filter-sidebar [data-scope=\"drawer\"][data-part=\"positioner\"] {\n    position: static;\n    display: block;\n  }\n  .blocks-filter-sidebar [data-scope=\"drawer\"][data-part=\"content\"] {\n    width: min(100%, var(--fandhe-drawer-size, 20rem));\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"menu\"",
            "data-scope=\"link\"",
            "data-scope=\"collapsible\"",
            "data-scope=\"fieldset\"",
            "data-scope=\"checkbox\"",
            "data-scope=\"drawer\"",
            "data-scope=\"button\"",
            "data-scope=\"skeleton\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn no_form_and_all_buttons_are_type_button() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        let count_open = html.matches("<button").count();
        let count_typed = html.matches("type=\"button\"").count();
        assert!(count_open > 0);
        assert_eq!(count_typed, count_open, "html={html}");
    }

    #[test]
    fn version_a_has_all_collapsible_sections_open() {
        let html = demo_html();
        // 版 A・版 C はそれぞれサイドバー + ドロワーの 2 パネル × 3 群を
        // 持つ。disabled な trigger は開閉操作ができず、closed な群は
        // `hidden` 属性で選択肢へ到達不能になるため（モジュール doc
        // 「無 JS・到達性の方針」節）、3 群すべてを open 固定する
        // （版 B は fieldset のため 0）。
        assert_eq!(
            html.matches(r#"data-scope="collapsible" data-part="content" data-state="open""#)
                .count(),
            12,
            "html={html}"
        );
        assert_eq!(
            html.matches(r#"data-scope="collapsible" data-part="content" data-state="closed""#)
                .count(),
            0,
            "html={html}"
        );
    }

    #[test]
    fn version_b_has_no_collapsible_and_three_legends() {
        let html = demo_html();
        // 版 B のみ fieldset legend を持つ（サイドバー + ドロワーの
        // 2 パネル × 3 群）。
        assert_eq!(
            html.matches(r#"data-scope="fieldset" data-part="legend""#)
                .count(),
            6,
            "html={html}"
        );
    }

    #[test]
    fn each_instance_has_one_open_non_modal_drawer() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-scope=\"drawer\" data-part=\"content\"")
                .count(),
            3
        );
        assert_eq!(html.matches("aria-modal=\"false\"").count(), 3);
        assert_eq!(html.matches("data-placement=\"start\"").count(), 9); // root/positioner/content × 3
    }

    #[test]
    fn trigger_and_title_ids_are_unique_and_resolved() {
        let html = demo_html();
        for prefix in ["a", "b", "c"] {
            let content_id = format!("blocks-filter-sidebar-{prefix}-drawer-content");
            let title_id = format!("blocks-filter-sidebar-{prefix}-drawer-title");
            assert_eq!(
                html.matches(format!("id=\"{content_id}\"").as_str())
                    .count(),
                1
            );
            assert_eq!(
                html.matches(format!("id=\"{title_id}\"").as_str()).count(),
                1
            );
            assert!(html.contains(format!("aria-controls=\"{content_id}\"").as_str()));
            assert!(html.contains(format!("aria-labelledby=\"{title_id}\"").as_str()));
        }
    }

    #[test]
    fn category_links_point_to_existing_fragment_ids() {
        let html = demo_html();
        for prefix in ["a", "b", "c"] {
            let products_id = format!("blocks-filter-sidebar-{prefix}-products");
            assert!(html.contains(format!("href=\"#{products_id}\"").as_str()));
            assert!(html.contains(format!("id=\"{products_id}\"").as_str()));
        }
        assert!(!html.contains("href=\"#\""));
    }

    #[test]
    fn checkboxes_are_disabled_and_css_neutralizes_opacity() {
        let html = demo_html();
        assert!(html.contains("data-scope=\"checkbox\""));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"checkbox\"][data-part=\"root\"][data-blocks-filter-sidebar-checkbox][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}"
        ));
    }

    #[test]
    fn layout_css_has_no_html_breakout_and_declares_container_query() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size"));
        assert!(LAYOUT_CSS.contains("@container blocks-filter-sidebar (max-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("position: absolute;\n  inset: 0;\n  z-index: auto;"));
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }

    #[test]
    fn drawer_backdrop_is_always_hidden() {
        // レビュー指摘（PR #3504 初回）: backdrop は常に Open で描画される
        // ため、positioner のみを非表示にすると広い幅で backdrop が
        // インスタンス全体を覆いマウス操作を妨げる。既定で非表示にする。
        //
        // レビュー指摘（PR #3504 再指摘）: 狭い幅で positioner を
        // オーバーレイのまま表示へ戻すと、常時 Open なドロワーが商品領域を
        // 塞いで無 JS では閉じられなくなる。狭い幅では positioner を
        // 通常フローへ戻す（下記 narrow_container_query テスト）ため、
        // オーバーレイ専用だった backdrop はどの幅でも表示に戻さない。
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"drawer\"][data-part=\"backdrop\"] {\n  position: absolute;\n  inset: 0;\n  z-index: auto;\n  display: none;\n}"
        ));
        let narrow_block = LAYOUT_CSS
            .split("@container blocks-filter-sidebar (max-width: 40rem) {")
            .nth(1)
            .expect("narrow container query block");
        assert!(!narrow_block.contains("data-part=\"backdrop\""));
    }

    #[test]
    fn drawer_positioner_is_static_in_narrow_container_query() {
        // レビュー指摘（PR #3504 再指摘）: 狭い幅で常時 Open なドロワーが
        // `position: absolute; inset: 0` のオーバーレイのままだと商品領域を
        // 塞ぎ、無 JS では閉じる手段がない。狭い幅でのみ `positioner` を
        // `position: static` の通常フロー要素に戻し、`instance()` 内で
        // 商品グリッドより後ろに配置されている `filter_drawer` の内容が
        // 商品領域の下に続けて表示されるようにする（重ならず両方へ
        // 到達できる）。
        let narrow_block = LAYOUT_CSS
            .split("@container blocks-filter-sidebar (max-width: 40rem) {")
            .nth(1)
            .expect("narrow container query block");
        assert!(narrow_block.contains(
            "[data-scope=\"drawer\"][data-part=\"positioner\"] {\n    position: static;\n    display: block;\n  }"
        ));
    }

    #[test]
    fn narrow_container_query_collapses_body_grid_to_single_column() {
        // レビュー指摘（PR #3504）: `.blocks-filter-sidebar-aside` を隠しても
        // `.blocks-filter-sidebar-body` の grid-template-columns が
        // `14rem 1fr` のままだと狭い幅で商品グリッドが全幅にならず隙間が
        // 残る。
        let narrow_block = LAYOUT_CSS
            .split("@container blocks-filter-sidebar (max-width: 40rem) {")
            .nth(1)
            .expect("narrow container query block");
        assert!(narrow_block
            .contains(".blocks-filter-sidebar-body {\n    grid-template-columns: 1fr;\n  }"));
    }

    #[test]
    fn drawer_close_trigger_uses_text_variant_with_visible_label() {
        // レビュー指摘（PR #3504）: `close_trigger`（アイコン専用契約）に
        // 複数文字の日本語テキストを渡すと、既定の 2rem 固定ボックス +
        // overflow: hidden でラベルが切り詰められる。平文ボタンの
        // `close_trigger_with_variant(CloseTriggerVariant::Text, ..)` を使う。
        let html = demo_html();
        assert!(html.contains("data-variant=\"text\""));
        assert!(html.contains(">閉じる<"));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-filter-sidebar-wrap\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-filter-sidebar-wrap");
    }
}
