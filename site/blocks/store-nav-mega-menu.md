# store-nav-mega-menu

`fandhe-frontend-pre-styled-ui` の `navigation-menu` / `link` / `button` /
`icon` / `image` / `native-select` / `badge` 部品を合成した、上部帯 +
メガメニュー付きストアナビゲーションです。Blocks セクションは新規部品を
追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集
であることに注意してください（主参照は対応表 ID R0711。出典の固有名・
ファイル名は記載しません）。

最上部の細い告知帯（告知文 + 言語/通貨切り替え）の下に、ロゴ・中央配置
ナビゲーション・検索/アカウント/カート操作を横一列に並べたバーがあります。
ナビゲーションのうち「新作」項目はヘッダー全幅まで広がるメガメニュー
パネルを常時展開しており、注目画像カード 1 件と項目リスト 3 列を持ちます。
無 JS の静的表示のためハンバーガーへの開閉切り替えは持たず、狭い幅では
バー・ナビ一覧・上部帯をいずれも折り返して常時到達可能なまま残します。

本 Demo は静的な表示例であり、唯一のドロップダウン（新作）を常時展開
（open）した状態で固定し、そのトリガーは `disabled` にして操作不能を
明示します。トリガーを持たないトップ項目（レディース・メンズ・小物・
セール）はリンクのみで構成し、無 JS のドキュメントサイトでも本文へ到達
できない閉じたトリガーを残しません。検索・アカウント・カートの操作
アイコンボタンは送信先・遷移先を持たない `disabled` 表示です。`<form>`
要素は一切持たず、データの取得・送信・状態管理を行いません。ボタンは
`type="button"` のまま送信先を持ちません。言語/通貨の `native-select` は
ネイティブ操作可能な部品として有効のまま残しますが、値の送信・保存は
行いません。文言・ブランド名はすべて独自に書いた架空のものであり、
実企業名・実クレデンシャル・PII を含みません。

濃色帯 variant・アカウント導線・狭幅（メニュー展開時）状態の並記は
後続の `store-nav-mega-menu`（後半）で追加します。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::input::{FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::Size;

/// パネル項目 1 件（タイトル, href）。href はサイト内に実在する索引ページへ
/// の相対パス（モジュール冒頭 rustdoc「href の方針」節）。
type PanelItem = (&'static str, &'static str);

/// パネルの列 1 件（列見出し, 項目 4 件）。
type PanelColumn = (&'static str, [PanelItem; 4]);

/// 「新作」パネルの列一覧（列見出し + 項目 4 件 × 3 列）。
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

/// トリガーを持たないトップ項目一覧（value, ラベル, href）。
const LINK_ITEMS: [(&str, &str, &str); 4] = [
    ("women", "レディース", "../../themes/"),
    ("men", "メンズ", "../../primitives/"),
    ("accessories", "小物", "../../guides/"),
    ("sale", "セール", "../../blocks/promo-collection-cards/"),
];

/// 「新作」トリガーの `id`（モジュール冒頭 rustdoc「id 接頭辞」節）。
const NEW_TRIGGER_ID: &str = "blocks-store-nav-mega-menu-new-trigger";
/// [`NEW_TRIGGER_ID`] と対になる `content` の `id`。
const NEW_CONTENT_ID: &str = "blocks-store-nav-mega-menu-new-content";
/// 言語 `native-select` の `id`。
const LANGUAGE_SELECT_ID: &str = "blocks-store-nav-mega-menu-band-language";
/// 通貨 `native-select` の `id`。
const CURRENCY_SELECT_ID: &str = "blocks-store-nav-mega-menu-band-currency";

/// ブランドロゴ（装飾用の幾何アイコン、実在ブランドを模さない単純図形）。
fn brand_icon() -> Node {
    icon::icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", "M4 4h16v16H4zM4 12h16")], vec![])],
    )
}

/// ブランド領域（アイコン + 架空のストア名）。
fn brand() -> Node {
    div(
        vec![("class", "blocks-store-nav-mega-menu-brand")],
        vec![brand_icon(), span(vec![], vec![text("Nimbus Store")])],
    )
}

/// 線画アイコン（装飾用途、パネル外の操作アイコンで共有する）。
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

/// 最上部の告知帯（告知文 + 言語/通貨 `native-select`、モジュール冒頭
/// rustdoc 参照）。
fn band() -> Node {
    let language_field = FieldProps {
        id: LANGUAGE_SELECT_ID,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let currency_field = FieldProps {
        id: CURRENCY_SELECT_ID,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    div(
        vec![("class", "blocks-store-nav-mega-menu-band")],
        vec![
            span(vec![], vec![text("送料無料キャンペーン実施中")]),
            div(
                vec![("class", "blocks-store-nav-mega-menu-band-controls")],
                vec![
                    native_select::native_select(
                        &NativeSelectProps::default(),
                        &language_field,
                        vec![("aria-label", "言語")],
                        vec![
                            el("option", vec![("value", "ja")], vec![text("日本語")]),
                            el("option", vec![("value", "en")], vec![text("English")]),
                        ],
                    ),
                    native_select::native_select(
                        &NativeSelectProps::default(),
                        &currency_field,
                        vec![("aria-label", "通貨")],
                        vec![
                            el("option", vec![("value", "jpy")], vec![text("JPY")]),
                            el("option", vec![("value", "usd")], vec![text("USD")]),
                            el("option", vec![("value", "eur")], vec![text("EUR")]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// パネル 1 列分（列見出し + 項目 4 件）。
fn panel_column(heading: &str, items: &[PanelItem; 4]) -> Node {
    let mut children = vec![span(
        vec![("class", "blocks-store-nav-mega-menu-panel-column-heading")],
        vec![text(heading)],
    )];
    children.extend(items.iter().map(|(title, href)| {
        navigation_menu::link(
            href,
            false,
            vec![("class", "blocks-store-nav-mega-menu-panel-link")],
            vec![text(*title)],
        )
    }));
    div(
        vec![("class", "blocks-store-nav-mega-menu-panel-column")],
        children,
    )
}

/// パネル左側の注目画像カード（`image` + `link::root`。パネル列項目
/// （`navigation_menu::link`）とは異なりナビゲーション一覧の一部ではない
/// 促進カードのため、`drop_class_attr` の契約に従い CSS フックは
/// `data-blocks-store-nav-mega-menu-featured` 属性で渡す）。
fn featured_card() -> Node {
    link::root(
        "../../blocks/promo-collection-cards/",
        &LinkProps::default(),
        vec![("data-blocks-store-nav-mega-menu-featured", "")],
        vec![
            image::image(
                &ImageProps {
                    fit: fandhe_frontend_pre_styled_ui::image::ImageFit::Cover,
                    ..ImageProps::new(crate::blocks::dummy_assets::PRODUCT_SRC, "")
                },
                vec![],
            ),
            span(
                vec![("class", "blocks-store-nav-mega-menu-featured-title")],
                vec![text("新作コレクション")],
            ),
            span(
                vec![("class", "blocks-store-nav-mega-menu-featured-description")],
                vec![text("今季の新作アイテムをまとめてチェック。")],
            ),
        ],
    )
}

/// 「新作」トップ項目（唯一のドロップダウン、常時 open 固定）。
fn new_item(props: &NavigationMenuProps) -> Node {
    let state = OpenState::Open;
    let columns: Vec<Node> = PANEL_COLUMNS
        .iter()
        .map(|(heading, items)| panel_column(heading, items))
        .collect();
    let mut panel_children = vec![featured_card()];
    panel_children.extend(columns);

    navigation_menu::item(
        state,
        false,
        props,
        "new",
        vec![],
        vec![
            navigation_menu::trigger(
                state,
                true,
                "new",
                Some(NEW_TRIGGER_ID),
                Some(NEW_CONTENT_ID),
                vec![],
                vec![
                    text("新作"),
                    navigation_menu::item_indicator(state, props, "new", vec![], vec![text("▾")]),
                ],
            ),
            navigation_menu::content(
                state,
                props,
                "new",
                Some(NEW_CONTENT_ID),
                Some(NEW_TRIGGER_ID),
                vec![],
                vec![div(
                    vec![("class", "blocks-store-nav-mega-menu-panel-inner")],
                    panel_children,
                )],
            ),
        ],
    )
}

/// トリガーを持たない、リンクのみのトップ項目。
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

/// バー中央のナビゲーション（ストアメニュー本体）。
fn nav() -> Node {
    let props = NavigationMenuProps::default();
    let mut items = vec![new_item(&props)];
    items.extend(
        LINK_ITEMS
            .iter()
            .map(|(value, label, href)| link_item(&props, value, label, href)),
    );
    navigation_menu::root(
        &props,
        "ストアメニュー",
        vec![("class", "blocks-store-nav-mega-menu-nav")],
        vec![navigation_menu::list(&props, vec![], items)],
    )
}

/// バー右側の操作（検索・アカウント・カート）。いずれも送信先・遷移先を
/// 持たない no-op のため `disabled: true` のまま中和しない
/// （モジュール冒頭 rustdoc「静的表示」節）。
fn actions() -> Node {
    div(
        vec![("class", "blocks-store-nav-mega-menu-actions")],
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
                "アカウント",
                vec![],
                vec![stroke_icon(
                    "M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM4 20a8 8 0 0 1 16 0",
                )],
            ),
            div(
                vec![("class", "blocks-store-nav-mega-menu-cart")],
                vec![
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
                    badge::badge(
                        &BadgeProps {
                            size: Size::Sm,
                            ..BadgeProps::default()
                        },
                        vec![("data-blocks-store-nav-mega-menu-cart-badge", "")],
                        vec![text("3")],
                    ),
                ],
            ),
        ],
    )
}

/// バー（ブランド / 中央ナビ / 操作）。
fn bar() -> Node {
    div(
        vec![("class", "blocks-store-nav-mega-menu-bar")],
        vec![brand(), nav(), actions()],
    )
}

/// ダミーのページ本文（`.blocks-demo` のはみ出し対策、モジュール冒頭
/// rustdoc「`.blocks-demo` のはみ出し対策」節参照）。
fn page_placeholder() -> Node {
    div(
        vec![("class", "blocks-store-nav-mega-menu-page")],
        vec![text(
            "ページ本文（ダミー）。常時展開済みパネルの下に十分な高さを確保するための枠。",
        )],
    )
}

/// `store-nav-mega-menu` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-store-nav-mega-menu-shell")],
        vec![
            band(),
            div(
                vec![("class", "blocks-store-nav-mega-menu-bar-wrap")],
                vec![bar()],
            ),
            page_placeholder(),
        ],
    )
}
```

## 関連情報

[Navigation Menu](../themes/navigation-menu.md) /
[Link](../themes/link.md) / [Button](../themes/button.md) /
[Icon](../themes/icon.md) / [Image](../themes/image.md) /
[Native Select](../themes/native-select.md) / [Badge](../themes/badge.md)
