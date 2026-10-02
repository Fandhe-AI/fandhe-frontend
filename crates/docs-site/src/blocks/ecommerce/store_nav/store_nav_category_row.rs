//! `store-nav-category-row` block（イシュー #3092、主参照 R1317）。
//!
//! 既存 2 件（`store_nav_centered_logo`/`store_nav_mega_menu`）がいずれも
//! バー 1 行の中にカテゴリナビを収めるのに対し、本 block は 2 行構成を示す:
//! 1 行目にロゴ・検索・カートを置き、2 行目をカテゴリのトリガー行として
//! 幅に関係なく常時表示する（メニューボタンへの畳み込みを行わない）。狭い
//! 幅では 2 行目を横スクロールさせる。
//!
//! # 使用部品
//!
//! `navigation-menu` / `link` / `button` / `icon` / `scroll-area` の 5 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、`crates/docs-site/tests/
//! blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # パネルを `list` の外（`navigation_menu::root` 直下）へ置く理由
//!
//! [`crate::blocks::ecommerce::store_nav::store_nav_mega_menu`] は
//! `content` を `item` の中に置き `position: absolute` のまま使うが、本
//! block のトリガー行は [`fandhe_frontend_pre_styled_ui::scroll_area`] の
//! `viewport`（`overflow: auto`）の内側にあるため、`content` を `item` の
//! 中に置くと viewport の `overflow` と `scroll_area` root の
//! `position: relative; overflow: hidden` によってパネルが切り取られる
//! （固定 `min-block-size` で回避する手法は mega-menu のレビュー是正
//! `#3475` で指摘済みの当てずっぽうであり、ここでは採らない）。
//!
//! 代わりに [`navigation_menu::content`] を `navigation_menu::root` の直下、
//! `scroll_area::root` の兄弟として置き、[`LAYOUT_CSS`] が本 block のスコープ
//! 内でのみ `position: static` へ上書きして通常フローへ戻す。`root` 自体は
//! pre-styled-ui recipe の既定で行 flex（`display: flex`、
//! `crates/pre-styled-ui/src/navigation_menu.rs` の `recipe()` 参照）のため、
//! `position: static` だけでは `content` が `scroll_area::root` の隣に
//! 横並びのまま残ってしまう（指摘 #3540 の是正）。`LAYOUT_CSS` は
//! `.blocks-store-nav-category-row-shell [data-scope="navigation-menu"]
//! [data-part="root"]` へ `flex-direction: column`（`align-items: stretch` も
//! 併記）を追加で与え、`scroll_area::root`（カテゴリ行）→ `content`（パネル）
//! の順に縦積みさせる。クラス単体セレクタ（`.blocks-store-nav-category-row-nav`）
//! は recipe の `[data-scope="navigation-menu"][data-part="root"]`（2 属性）に
//! 詳細度で負け `align-items: center` のまま残っていたため（指摘 #3540 再指摘、
//! Bugbot Medium）、祖先クラス + 同型の属性セレクタで詳細度を揃えて上書きする。[`fandhe_frontend_pre_styled_ui::navigation_menu`] の recipe は `content`
//! 自身の `data-state`/`hidden` だけを見て祖先の構造を前提にしないため
//! （`crates/pre-styled-ui/src/navigation_menu.rs` 参照）、この配置は recipe
//! の契約を破らない。トリガーとパネルの関連付けは `aria-controls`/
//! `aria-labelledby` が担うため、DOM 上の位置を切り離しても対応関係は
//! 保たれる（Radix `NavigationMenu.Viewport` と同じ考え方で、寸法計測も
//! 行わない）。
//!
//! # トリガーとリンクの扱い（既存兄弟 block と同じ判断）
//!
//! 開いたカテゴリ（[`OpenState::Open`]）1 件のみ `disabled: true` の
//! trigger を持つ（押しても状態が変わらない no-op のため）。
//! `disabled_declarations()` が付ける不透明度低下は [`LAYOUT_CSS`] で
//! 打ち消す。残りのカテゴリは trigger を持たない
//! [`navigation_menu::item`] + [`navigation_menu::link`] のみで構成する
//! （閉じたまま操作しても何も起きない trigger を残さないため、
//! `store_nav_mega_menu` と同じ判断）。検索・カートの icon button は
//! no-op のため `disabled: true` のまま不透明度を打ち消さない。
//!
//! # 横スクロール
//!
//! `scroll_area::viewport` は既定で `overflow: auto` と `tabindex="0"` を
//! 持つ。[`LAYOUT_CSS`] は本 block スコープ内の `list`/`item`/`link`/
//! `trigger` へ `white-space: nowrap` を、`list` へ
//! `inline-size: max-content` を与え、幅が足りないときに横スクロール
//! バーが働く状態を作る。`viewport` には `role="region"` +
//! `aria-label` を付与する（素の `div` へ `aria-label` のみを付けるのは
//! ARIA 上不適切、[`crate::blocks::application::list::list_sticky_groups`]
//! と同じ判断）。
//!
//! 狭い幅での挙動はブラウザ幅を変えなくても確認できるよう、[`demo`] は
//! 広い幅・狭い幅（固定幅の枠）の 2 インスタンスを並記する。両方で
//! パネルを開いた状態にし、`id`/`aria-controls`/`aria-labelledby` は
//! `-wide-`/`-narrow-` 接尾辞で分けて一意にする（横断テスト
//! `demo_output_has_no_dangling_aria_references_or_duplicate_ids` 対応）。
//! `nav`（`navigation_menu::root` の `aria-label`）・`region`
//! （`viewport` の `aria-label`）も同じ理由で「広い幅」「狭い幅」を
//! 埋め込み、2 インスタンスでランドマーク名が重複しないようにする
//! （[`category_nav`] の `instance_label` 引数、指摘 #3540 対応）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `navigation_menu::*`/`scroll_area::*` は呼び出し側 `attrs` の `class` を
//! 除去しないため `.blocks-store-nav-category-row-*` の class フックが
//! そのまま使える。一方 `button::icon_button`/`link::root` は
//! `drop_class_attr` で `class` を常に除去するため、これらへのフックは
//! `data-blocks-store-nav-category-row-*` 属性で渡す。
//!
//! # href の方針
//!
//! `href="#"` は使わない。トリガー行・パネル項目はサイト内に実在する索引
//! ページ・既存 block ページへの相対パスを使う。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンは既定の `type="button"` のまま送信先を持たない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::scroll_area;
use fandhe_frontend_pre_styled_ui::Size;

/// パネル項目 1 件（タイトル, href）。href はサイト内に実在する索引ページ・
/// 既存 block ページへの相対パス（モジュール冒頭 rustdoc「href の方針」節）。
type PanelItem = (&'static str, &'static str);

/// パネルの列 1 件（列見出し, 項目 4 件）。
type PanelColumn = (&'static str, [PanelItem; 4]);

/// 開いたカテゴリ（「コレクション」）が持つパネルの列一覧（列見出し +
/// 項目 4 件 × 3 列、画像なしの項目リスト）。
const PANEL_COLUMNS: [PanelColumn; 3] = [
    (
        "アウター",
        [
            ("コート", "../../themes/"),
            ("ジャケット", "../../primitives/"),
            ("ニット", "../../guides/"),
            ("パーカー", "../../examples/"),
        ],
    ),
    (
        "トップス",
        [
            ("シャツ", "../../themes/"),
            ("Tシャツ", "../../primitives/"),
            ("ブラウス", "../../guides/"),
            ("カットソー", "../../examples/"),
        ],
    ),
    (
        "アクセサリー",
        [
            ("バッグ", "../../api/"),
            ("ジュエリー", "../../themes/"),
            ("ベルト", "../../primitives/"),
            ("帽子", "../../guides/"),
        ],
    ),
];

/// 開いたカテゴリの value/ラベル。
const OPEN_CATEGORY: (&str, &str) = ("collection", "コレクション");

/// トリガーを持たない、リンクのみのカテゴリ一覧（value, ラベル, href）。
const LINK_CATEGORIES: [(&str, &str, &str); 7] = [
    ("women", "レディース", "../../themes/"),
    ("men", "メンズ", "../../primitives/"),
    ("kids", "キッズ", "../../guides/"),
    ("shoes", "シューズ", "../../examples/"),
    ("bags", "バッグ", "../../api/"),
    (
        "accessories",
        "アクセサリー",
        "../../blocks/promo-collection-cards/",
    ),
    ("sale", "セール", "../../blocks/category-split-panels/"),
];

/// 広い幅インスタンスのトリガー `id`。
const WIDE_TRIGGER_ID: &str = "blocks-store-nav-category-row-wide-trigger";
/// 広い幅インスタンスの `content` `id`。
const WIDE_CONTENT_ID: &str = "blocks-store-nav-category-row-wide-content";
/// 狭い幅インスタンスのトリガー `id`。
const NARROW_TRIGGER_ID: &str = "blocks-store-nav-category-row-narrow-trigger";
/// 狭い幅インスタンスの `content` `id`。
const NARROW_CONTENT_ID: &str = "blocks-store-nav-category-row-narrow-content";

/// ブランドロゴ（装飾用の幾何アイコン、実在ブランドを模さない単純図形）。
fn brand_icon() -> Node {
    icon::icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![("d", "M4 12l8-8 8 8M6 10v10h12V10")],
            vec![],
        )],
    )
}

/// 線画アイコン（検索・カートの装飾用途で共有する）。
fn stroke_icon(path_d: &str) -> Node {
    icon::icon(
        &IconProps {
            size: Size::Sm,
            label: None,
            ..IconProps::default()
        },
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

/// ブランド領域（アイコン + 架空のストア名、サイト内索引へのリンク）。
fn brand() -> Node {
    link::root(
        "../../blocks/",
        &LinkProps::default(),
        vec![("data-blocks-store-nav-category-row-brand", "")],
        vec![brand_icon(), span(vec![], vec![text("Meridian Goods")])],
    )
}

/// 1 行目右側の操作（検索・カート）。送信先・遷移先を持たない no-op の
/// ため `disabled: true` のまま中和しない（モジュール冒頭 rustdoc
/// 「トリガーとリンクの扱い」節）。
fn actions() -> Node {
    div(
        vec![("class", "blocks-store-nav-category-row-actions")],
        vec![
            button::icon_button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    disabled: true,
                    ..ButtonProps::default()
                },
                "検索",
                vec![],
                vec![stroke_icon("M10 4a6 6 0 1 0 0 12 6 6 0 0 0 0-12zM20 20l-5.5-5.5")],
            ),
            button::icon_button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    disabled: true,
                    ..ButtonProps::default()
                },
                "カート",
                vec![],
                vec![stroke_icon(
                    "M4 4h2l2.4 12h9.2L20 8H7M9 21a1 1 0 1 0 0-2 1 1 0 0 0 0 2zM17 21a1 1 0 1 0 0-2 1 1 0 0 0 0 2z",
                )],
            ),
        ],
    )
}

/// 1 行目（ブランド / 検索・カート）。
fn top_bar() -> Node {
    div(
        vec![("class", "blocks-store-nav-category-row-top-bar")],
        vec![brand(), actions()],
    )
}

/// パネル 1 列分（列見出し + 項目 4 件）。
fn panel_column(heading: &str, items: &[PanelItem; 4]) -> Node {
    let mut children = vec![span(
        vec![(
            "class",
            "blocks-store-nav-category-row-panel-column-heading",
        )],
        vec![text(heading)],
    )];
    children.extend(items.iter().map(|(title, href)| {
        navigation_menu::link(
            href,
            false,
            vec![("class", "blocks-store-nav-category-row-panel-link")],
            vec![text(*title)],
        )
    }));
    div(
        vec![("class", "blocks-store-nav-category-row-panel-column")],
        children,
    )
}

/// 開いたカテゴリのトリガー項目（`list` 内、disabled 固定）。
fn open_item(
    props: &NavigationMenuProps,
    trigger_id: &'static str,
    content_id: &'static str,
) -> Node {
    let state = OpenState::Open;
    let (value, label) = OPEN_CATEGORY;
    navigation_menu::item(
        state,
        false,
        props,
        value,
        vec![],
        vec![navigation_menu::trigger(
            state,
            true,
            value,
            Some(trigger_id),
            Some(content_id),
            vec![],
            vec![
                text(label),
                navigation_menu::item_indicator(state, props, value, vec![], vec![text("▾")]),
            ],
        )],
    )
}

/// トリガーを持たない、リンクのみのカテゴリ項目。
fn link_item(props: &NavigationMenuProps, value: &str, label: &str, href: &str) -> Node {
    navigation_menu::item(
        OpenState::Closed,
        false,
        props,
        value,
        vec![],
        vec![navigation_menu::link(
            href,
            false,
            vec![],
            vec![text(label)],
        )],
    )
}

/// 開いたカテゴリのパネル（`list` の外、`navigation_menu::root` 直下に
/// 置く。モジュール冒頭 rustdoc「パネルを `list` の外へ置く理由」節）。
fn panel(props: &NavigationMenuProps, trigger_id: &'static str, content_id: &'static str) -> Node {
    let (value, _) = OPEN_CATEGORY;
    let columns: Vec<Node> = PANEL_COLUMNS
        .iter()
        .map(|(heading, items)| panel_column(heading, items))
        .collect();
    navigation_menu::content(
        OpenState::Open,
        props,
        value,
        Some(content_id),
        Some(trigger_id),
        vec![("class", "blocks-store-nav-category-row-panel")],
        vec![div(
            vec![("class", "blocks-store-nav-category-row-panel-inner")],
            columns,
        )],
    )
}

/// 2 行目（横スクロール可能なカテゴリのトリガー行 + 開いたパネル）。
/// `instance_label`（「広い幅」「狭い幅」）を `nav`/`region` のランドマーク名へ
/// 織り込み、広い幅・狭い幅の 2 インスタンスでアクセシブルネームが重複しない
/// ようにする（モジュール冒頭 rustdoc「横スクロール」節、指摘 #3540 対応）。
fn category_nav(trigger_id: &'static str, content_id: &'static str, instance_label: &str) -> Node {
    let props = NavigationMenuProps::default();
    let mut items = vec![open_item(&props, trigger_id, content_id)];
    items.extend(
        LINK_CATEGORIES
            .iter()
            .map(|(value, label, href)| link_item(&props, value, label, href)),
    );
    navigation_menu::root(
        &props,
        &format!("カテゴリ（{instance_label}）"),
        vec![("class", "blocks-store-nav-category-row-nav")],
        vec![
            scroll_area::root(
                vec![("class", "blocks-store-nav-category-row-scroll")],
                vec![scroll_area::viewport(
                    vec![
                        ("role", "region"),
                        ("aria-label", &format!("カテゴリ一覧（{instance_label}）")),
                    ],
                    vec![scroll_area::content(
                        vec![],
                        vec![navigation_menu::list(&props, vec![], items)],
                    )],
                )],
            ),
            panel(&props, trigger_id, content_id),
        ],
    )
}

/// 1 インスタンス分（1 行目 + 2 行目、`id` は呼び出し側が指定する接尾辞で
/// 一意にする）。
fn instance(trigger_id: &'static str, content_id: &'static str, instance_label: &str) -> Node {
    div(
        vec![("class", "blocks-store-nav-category-row-shell")],
        vec![
            top_bar(),
            category_nav(trigger_id, content_id, instance_label),
        ],
    )
}

/// 幅の状態ラベル（広い幅・狭い幅の並記を見分けるための見出し）。
fn state_label(label: &str) -> Node {
    span(
        vec![("class", "blocks-store-nav-category-row-state-label")],
        vec![text(label)],
    )
}

/// `store-nav-category-row` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「横スクロール」節）。広い幅・狭い幅の 2
/// インスタンスを並記し、`id` 系はそれぞれ一意にする。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-store-nav-category-row-states")],
        vec![
            state_label("広い幅"),
            instance(WIDE_TRIGGER_ID, WIDE_CONTENT_ID, "広い幅"),
            state_label("狭い幅（カテゴリ行を横スクロール）"),
            div(
                vec![("class", "blocks-store-nav-category-row-narrow-frame")],
                vec![instance(NARROW_TRIGGER_ID, NARROW_CONTENT_ID, "狭い幅")],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/store-nav-category-row/",
    title: "store-nav-category-row",
    category: BlockCategory::StoreNav,
    rust_source: "crates/docs-site/src/blocks/ecommerce/store_nav/store_nav_category_row.rs",
    demo_class: "blocks-store-nav-category-row",
    parts: &[
        Part {
            label: "Navigation Menu",
            path: "/themes/navigation-menu/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
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
            label: "Scroll Area",
            path: "/themes/scroll-area/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `store_nav_category_row` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節）。
///
/// セレクタは `.blocks-store-nav-category-row-*`、
/// `[data-blocks-store-nav-category-row-*]`、および
/// `.blocks-store-nav-category-row-shell` を祖先に持つ
/// `[data-scope="navigation-menu"]`/`[data-scope="scroll-area"]` 系セレクタ
/// への子孫結合子付き上書き（パネルを通常フローへ戻す配置、モジュール
/// 冒頭 rustdoc「パネルを `list` の外へ置く理由」節）のみを用いる。値は
/// すべて `var(--fandhe-*)` トークンで書き、生の色リテラルは使わない。
/// 狭い幅は `@container` ではなく固定幅の枠（`narrow-frame`）で示す
/// （2 インスタンス並記、モジュール冒頭 rustdoc「横スクロール」節）。
const LAYOUT_CSS: &str = "\
.blocks-store-nav-category-row-states {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-store-nav-category-row-state-label {\n  display: block;\n  font-weight: 600;\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-store-nav-category-row-shell {\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  background: var(--fandhe-color-bg);\n  overflow: hidden;\n}\n\
.blocks-store-nav-category-row-narrow-frame {\n  max-inline-size: 22rem;\n  margin-inline: auto;\n}\n\
.blocks-store-nav-category-row-top-bar {\n  display: flex;\n  flex-wrap: nowrap;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-store-nav-category-row-shell [data-scope=\"navigation-menu\"][data-part=\"root\"] {\n  display: flex;\n  flex-direction: column;\n  align-items: stretch;\n}\n\
[data-blocks-store-nav-category-row-brand] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  font-weight: 600;\n  white-space: nowrap;\n}\n\
.blocks-store-nav-category-row-actions {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  white-space: nowrap;\n}\n\
.blocks-store-nav-category-row-shell [data-scope=\"navigation-menu\"][data-part=\"list\"] {\n  display: flex;\n  flex-wrap: nowrap;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  inline-size: max-content;\n  padding: var(--fandhe-space-2) var(--fandhe-space-4);\n}\n\
.blocks-store-nav-category-row-shell [data-scope=\"navigation-menu\"][data-part=\"item\"],\n\
.blocks-store-nav-category-row-shell [data-scope=\"navigation-menu\"][data-part=\"link\"],\n\
.blocks-store-nav-category-row-shell [data-scope=\"navigation-menu\"][data-part=\"trigger\"] {\n  white-space: nowrap;\n}\n\
.blocks-store-nav-category-row-shell [data-scope=\"navigation-menu\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-store-nav-category-row-shell [data-scope=\"navigation-menu\"][data-part=\"content\"] {\n  position: static;\n  min-width: 0;\n  inset: auto;\n  border-top: 1px solid var(--fandhe-color-border);\n  padding: var(--fandhe-space-4);\n}\n\
.blocks-store-nav-category-row-panel-inner {\n  display: grid;\n  grid-template-columns: repeat(auto-fit, minmax(min(10rem, 100%), 1fr));\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-store-nav-category-row-panel-column {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-store-nav-category-row-panel-column-heading {\n  display: block;\n  font-weight: 600;\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n  margin-block-end: var(--fandhe-space-1);\n}\n\
";

#[cfg(test)]
mod tests {
    use super::{
        demo, BLOCK, LAYOUT_CSS, NARROW_CONTENT_ID, NARROW_TRIGGER_ID, WIDE_CONTENT_ID,
        WIDE_TRIGGER_ID,
    };
    use fandhe_frontend_core::render;

    /// Demo が期待する 5 種の部品・非対話制約を満たすことの単体回帰。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"navigation-menu\"",
            "data-scope=\"link\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
            "data-scope=\"scroll-area\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 広い幅・狭い幅の両インスタンスで、開いたカテゴリのパネルが常時
    /// 展開済み（`hidden` なし）であり、閉じたトリガー
    /// （`aria-expanded="false"`）が存在しないこと。
    #[test]
    fn single_dropdown_is_open_in_both_instances() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-part="item" data-state="open""#)
                .count(),
            2,
            "html={html}"
        );
        // `scroll_area::content`（横スクロールコンテナ側のパーツ）も同じ
        // `data-part="content"` を名乗るため、scope で絞り込んで
        // navigation-menu 側の `content`（パネル）だけを数える。
        assert_eq!(
            html.matches(r#"data-scope="navigation-menu" data-part="content""#)
                .count(),
            2
        );
        assert_eq!(html.matches(r#"data-part="trigger""#).count(), 2);
        assert!(!html.contains(r#"data-part="trigger" aria-expanded="false""#));

        for (trigger_id, content_id) in [
            (WIDE_TRIGGER_ID, WIDE_CONTENT_ID),
            (NARROW_TRIGGER_ID, NARROW_CONTENT_ID),
        ] {
            assert!(html.contains(&format!("id=\"{trigger_id}\"")));
            assert!(html.contains(&format!("id=\"{content_id}\"")));
            assert!(html.contains(&format!("aria-controls=\"{content_id}\"")));
            assert!(html.contains(&format!("aria-labelledby=\"{trigger_id}\"")));
        }
        assert!(!html.contains(" hidden"));
    }

    /// 開いたパネルが `list`（`</ul>`）より後ろに置かれていること
    /// （「パネルを `list` の外へ置く」構造の回帰ガード、モジュール冒頭
    /// rustdoc「パネルを `list` の外へ置く理由」節）。`viewport` に
    /// `role="region"` があることも併せて固定する。
    #[test]
    fn panel_sits_after_list_in_dom_order() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"role="region""#).count(), 2, "{html}");

        let mut search_from = 0usize;
        for _ in 0..2 {
            let list_end = html[search_from..]
                .find("</ul>")
                .expect("list should close with </ul>")
                + search_from;
            // `scroll_area::content`（list を包む横スクロールコンテナ側）も
            // 同じ `data-part="content"` を名乗り `</ul>` より前に現れるため、
            // navigation-menu scope で絞り込んでパネル側の `content` を探す。
            let content_start = html[search_from..]
                .find(r#"data-scope="navigation-menu" data-part="content""#)
                .expect("navigation-menu content (panel) should be present")
                + search_from;
            assert!(
                content_start > list_end,
                "panel content should come after the list's closing </ul>: list_end={list_end} content_start={content_start}"
            );
            search_from = content_start + 1;
        }
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-store-nav-category-row-states\""));
        assert_ne!(BLOCK.demo_class, "blocks-store-nav-category-row-states");
    }

    /// nav ルートの縦積み上書きが recipe の
    /// `[data-scope="navigation-menu"][data-part="root"]`（0,2,0）より高い
    /// 詳細度（祖先クラス + 同型の属性セレクタ）で書かれ、クラス単体
    /// セレクタへ退行していないこと（指摘 #3540、Bugbot Medium）。
    #[test]
    fn nav_root_stretch_outranks_recipe_specificity() {
        let rule = ".blocks-store-nav-category-row-shell [data-scope=\"navigation-menu\"][data-part=\"root\"] {\n  display: flex;\n  flex-direction: column;\n  align-items: stretch;\n}";
        assert!(LAYOUT_CSS.contains(rule));
        assert!(!LAYOUT_CSS.contains(".blocks-store-nav-category-row-nav {"));
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-store-nav-category-row-shell\""));
    }
}
