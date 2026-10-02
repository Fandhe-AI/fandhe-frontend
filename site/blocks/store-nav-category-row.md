# store-nav-category-row

`fandhe-frontend-pre-styled-ui` の `navigation-menu` / `link` / `button` /
`icon` / `scroll-area` 部品を合成した、2 行構成のストアナビゲーションです。
Blocks セクションは新規部品を追加するものではなく、既存の Themes 部品を
組み合わせた実例集であることに注意してください（主参照は対応表 ID
R1317。出典の固有名・ファイル名は記載しません）。

1 行目にロゴ・検索・カートを置き、2 行目をカテゴリのトリガー行として
幅に関係なく常時表示します。既存の `store-nav-centered-logo`/
`store-nav-mega-menu` がいずれもバー 1 行の中にナビを収めるのに対し、
本 block は 2 行構成を示します。カテゴリ行はメニューボタンへの畳み込みを
行わず、狭い幅では横スクロールさせて常に到達可能なまま残します。

本 Demo は静的な表示例であり、カテゴリのうち「コレクション」1 件のみを
常時展開（open）した状態で固定し、そのトリガーは `disabled` にして操作
不能を明示します。トリガーを持たないその他のカテゴリ（レディース・
メンズ・キッズ・シューズ・バッグ・アクセサリー・セール）はリンクのみで
構成し、無 JS のドキュメントサイトでも本文へ到達できない閉じたトリガー
を残しません。検索・カートの操作アイコンボタンは送信先・遷移先を持たない
`disabled` 表示です。`<form>` 要素は一切持たず、データの取得・送信・状態
管理を行いません。ボタンは `type="button"` のまま送信先を持ちません。
文言・ブランド名はすべて独自に書いた架空のものであり、実企業名・実
クレデンシャル・PII を含みません。

広い幅・狭い幅（固定幅の枠）の 2 インスタンスを並記しており、ブラウザ幅を
変えなくても横スクロールの挙動を確認できます。

## Rust コード

```rust
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
fn category_nav(trigger_id: &'static str, content_id: &'static str) -> Node {
    let props = NavigationMenuProps::default();
    let mut items = vec![open_item(&props, trigger_id, content_id)];
    items.extend(
        LINK_CATEGORIES
            .iter()
            .map(|(value, label, href)| link_item(&props, value, label, href)),
    );
    navigation_menu::root(
        &props,
        "カテゴリ",
        vec![("class", "blocks-store-nav-category-row-nav")],
        vec![
            scroll_area::root(
                vec![("class", "blocks-store-nav-category-row-scroll")],
                vec![scroll_area::viewport(
                    vec![("role", "region"), ("aria-label", "カテゴリ一覧")],
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
fn instance(trigger_id: &'static str, content_id: &'static str) -> Node {
    div(
        vec![("class", "blocks-store-nav-category-row-shell")],
        vec![top_bar(), category_nav(trigger_id, content_id)],
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
            instance(WIDE_TRIGGER_ID, WIDE_CONTENT_ID),
            state_label("狭い幅（カテゴリ行を横スクロール）"),
            div(
                vec![("class", "blocks-store-nav-category-row-narrow-frame")],
                vec![instance(NARROW_TRIGGER_ID, NARROW_CONTENT_ID)],
            ),
        ],
    )
}
```

## 原案差分メモ

- 参照では全カテゴリがトリガーですが、本 block は開いた「コレクション」1
  件だけを trigger にし、他はリンクのみにしています。
- 参照は 2 列メニューですが、本 block は 3 列グループを auto-fit で並べ、
  幅に応じて列数が減ります。
- パネルはカテゴリ一覧（`list`）の外へ置き、通常フローで表示します。
- 画像は持ちません。

## 関連情報

[Navigation Menu](../themes/navigation-menu.md) /
[Link](../themes/link.md) / [Button](../themes/button.md) /
[Icon](../themes/icon.md) / [Scroll Area](../themes/scroll-area.md)
