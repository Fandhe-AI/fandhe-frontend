# store-nav-mega-menu

`fandhe-frontend-pre-styled-ui` の `navigation-menu` / `link` / `button` /
`icon` / `image` / `native-select` / `badge` / `drawer` 部品を合成した、
上部帯 + メガメニュー付きストアナビゲーションです。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わ
せた実例集であることに注意してください（主参照は対応表 ID R0711。出典の
固有名・ファイル名は記載しません）。

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

幅広・狭幅（メニュー展開時）の 2 状態をキャプション付きで並記します。
幅広インスタンスは狭い幅（Demo 枠基準の `@container` 50.99rem 以下）
でもバー・ナビ一覧・上部帯を折り返し、内容を非表示にせず常時到達可能な
まま残します（無 JS で開閉するメニューボタンは実装できないため、内容を
隠す構成は採りません）。狭幅インスタンスはメニューボタン（`disabled` +
`aria-expanded="true"` の静的状態）と、`drawer` 部品で組んだ展開済み
メニュー（`hidden` を持たず非モーダル）を常時表示し、狭い画面での見え方
を実演します。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::drawer::{self, ContentIds, DrawerPlacement};
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::input::{FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::{Orientation, Size};

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

/// 狭幅インスタンス（[`mobile_preview`]）側の「新作」トリガー `id`
/// （モジュール冒頭 rustdoc「狭幅インスタンスの並記」節）。
const MOBILE_NEW_TRIGGER_ID: &str = "blocks-store-nav-mega-menu-mobile-new-trigger";
/// [`MOBILE_NEW_TRIGGER_ID`] と対になる `content` の `id`。
const MOBILE_NEW_CONTENT_ID: &str = "blocks-store-nav-mega-menu-mobile-new-content";
/// 狭幅インスタンス側の言語 `native-select` の `id`。
const MOBILE_LANGUAGE_SELECT_ID: &str = "blocks-store-nav-mega-menu-mobile-band-language";
/// 狭幅インスタンス側の通貨 `native-select` の `id`。
const MOBILE_CURRENCY_SELECT_ID: &str = "blocks-store-nav-mega-menu-mobile-band-currency";
/// 狭幅インスタンスの展開済み drawer 本体（`content` パート）の `id`
/// （メニュートグルボタンの `aria-controls` が参照する）。
const MOBILE_DRAWER_CONTENT_ID: &str = "blocks-store-nav-mega-menu-mobile-drawer";
/// [`MOBILE_DRAWER_CONTENT_ID`] の `aria-labelledby` が参照する `title`
/// の `id`。
const MOBILE_DRAWER_TITLE_ID: &str = "blocks-store-nav-mega-menu-mobile-drawer-title";

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

/// 言語/通貨 `native-select` 2 件（帯 1 行分）。幅広の [`band`]・狭幅の
/// [`mobile_drawer`] の双方から `id` のみを差し替えて呼ばれる共通部品
/// （モジュール冒頭 rustdoc「狭幅インスタンスの並記」節）。
fn band_controls(language_id: &'static str, currency_id: &'static str) -> Node {
    let language_field = FieldProps {
        id: language_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let currency_field = FieldProps {
        id: currency_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
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
    )
}

/// 最上部の告知帯（告知文 + 言語/通貨 `native-select`、モジュール冒頭
/// rustdoc 参照）。
fn band() -> Node {
    div(
        vec![("class", "blocks-store-nav-mega-menu-band")],
        vec![
            span(vec![], vec![text("送料無料キャンペーン実施中")]),
            band_controls(LANGUAGE_SELECT_ID, CURRENCY_SELECT_ID),
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
/// `trigger_id`/`content_id` は呼び出し側（幅広 [`nav`] 呼び出し・狭幅
/// [`nav`] 呼び出し）ごとに異なる固定文字列を渡す（モジュール冒頭
/// rustdoc「id 接頭辞」節）。
fn new_item(
    props: &NavigationMenuProps,
    trigger_id: &'static str,
    content_id: &'static str,
) -> Node {
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
                Some(trigger_id),
                Some(content_id),
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
                Some(content_id),
                Some(trigger_id),
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

/// バー中央のナビゲーション（ストアメニュー本体）。幅広 [`bar`]・狭幅
/// [`mobile_drawer`] の双方から `orientation`・`class`・`aria_label`・
/// `trigger_id`/`content_id` のみを差し替えて呼ばれる共通部品（モジュール
/// 冒頭 rustdoc「狭幅インスタンスの並記」節）。
fn nav(
    orientation: Orientation,
    class: &'static str,
    aria_label: &str,
    trigger_id: &'static str,
    content_id: &'static str,
) -> Node {
    let props = NavigationMenuProps { orientation };
    let mut items = vec![new_item(&props, trigger_id, content_id)];
    items.extend(
        LINK_ITEMS
            .iter()
            .map(|(value, label, href)| link_item(&props, value, label, href)),
    );
    navigation_menu::root(
        &props,
        aria_label,
        vec![("class", class)],
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
        vec![
            brand(),
            nav(
                Orientation::Horizontal,
                "blocks-store-nav-mega-menu-nav",
                "ストアメニュー",
                NEW_TRIGGER_ID,
                NEW_CONTENT_ID,
            ),
            actions(),
        ],
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

/// 幅広インスタンス（告知帯 + バー + ダミー本文）。[`demo`] が狭幅インス
/// タンス（[`mobile_preview`]）と並べて描画する。
fn layout() -> Node {
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

/// 状態並記の見出し（モジュール冒頭 rustdoc「狭幅インスタンスの並記」
/// 節）。
fn state_label(label: &str) -> Node {
    span(
        vec![("class", "blocks-store-nav-mega-menu-state-label")],
        vec![text(label)],
    )
}

/// 狭幅インスタンスのバー（ブランド + メニュートグルボタン）。トリガーは
/// 押しても状態が変わらない no-op のため `disabled: true` にする
/// （モジュール冒頭 rustdoc「狭幅インスタンスの並記」節）。
/// `drawer::trigger` が `state`/`controls` から `aria-haspopup="dialog"`・
/// `aria-expanded="true"`・`aria-controls` を自動出力するため、これらを
/// `attrs` へ重複して渡さない。
fn mobile_bar() -> Node {
    div(
        vec![("class", "blocks-store-nav-mega-menu-mobile-bar")],
        vec![
            brand(),
            drawer::trigger(
                OpenState::Open,
                Some(MOBILE_DRAWER_CONTENT_ID),
                vec![
                    ("aria-label", "メニュー"),
                    ("disabled", ""),
                    ("data-disabled", ""),
                    ("data-blocks-store-nav-mega-menu-menu-toggle", ""),
                ],
                vec![stroke_icon("M3 6h18M3 12h18M3 18h18")],
            ),
        ],
    )
}

/// 狭幅インスタンスの展開済み drawer（`root` + backdrop/positioner/content
/// の anatomy、モジュール冒頭 rustdoc「狭幅インスタンスの並記」節）。
/// 常時 [`OpenState::Open`] で固定するため `hidden` 属性を持たない。
/// `modal` は `false`（静的デモは閉じる機構を持たず外側に説明・コード・
/// ナビゲーションがあるため、表示実態と一致させる）。`drawer::root` は
/// `drop_class_attr` で呼び出し側 `class` を除去するため、スコープ用の
/// class フックは呼び出し元（[`mobile_preview`]）の外側ラッパーへ付ける
/// （モジュール冒頭 rustdoc「CSS フックの選び方」節）。
fn mobile_drawer() -> Node {
    let state = OpenState::Open;
    let placement = DrawerPlacement::Start;
    drawer::root(
        Size::Sm,
        state,
        placement,
        vec![],
        vec![
            mobile_bar(),
            div(
                vec![("class", "blocks-store-nav-mega-menu-mobile-panel-wrap")],
                vec![
                    drawer::backdrop(state, vec![], vec![]),
                    drawer::positioner(
                        state,
                        placement,
                        vec![],
                        vec![drawer::content(
                            state,
                            placement,
                            false,
                            ContentIds {
                                id: Some(MOBILE_DRAWER_CONTENT_ID),
                                labelledby: Some(MOBILE_DRAWER_TITLE_ID),
                                describedby: None,
                            },
                            vec![],
                            vec![
                                drawer::close_trigger(
                                    vec![
                                        ("aria-label", "閉じる"),
                                        ("disabled", ""),
                                        ("data-disabled", ""),
                                    ],
                                    vec![stroke_icon("M6 6l12 12M18 6L6 18")],
                                ),
                                drawer::title(
                                    Some(MOBILE_DRAWER_TITLE_ID),
                                    vec![],
                                    vec![text("メニュー")],
                                ),
                                nav(
                                    Orientation::Vertical,
                                    "blocks-store-nav-mega-menu-mobile-nav",
                                    "ストアメニュー（狭幅）",
                                    MOBILE_NEW_TRIGGER_ID,
                                    MOBILE_NEW_CONTENT_ID,
                                ),
                                band_controls(MOBILE_LANGUAGE_SELECT_ID, MOBILE_CURRENCY_SELECT_ID),
                                actions(),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// 狭幅インスタンス全体（ブランド + メニュートグル + 展開済み drawer）。
/// [`demo`] が幅広インスタンス（[`layout`]）と並べて描画する。
fn mobile_preview() -> Node {
    div(
        vec![("class", "blocks-store-nav-mega-menu-mobile")],
        vec![mobile_drawer()],
    )
}

/// `store-nav-mega-menu` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「静的表示」節）。幅広インスタンス（[`layout`]）
/// と狭幅（メニュー展開時）インスタンス（[`mobile_preview`]）を見出し
/// 付きで並記する（本イシューで追加、モジュール冒頭 rustdoc「狭幅
/// インスタンスの並記」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-store-nav-mega-menu-states")],
        vec![
            state_label("幅広（新作を展開）"),
            layout(),
            state_label("狭幅（メニュー展開時）"),
            mobile_preview(),
        ],
    )
}
```

## 原案差分メモ

主参照（R0711、帯に言語・通貨）に対する無 JS の差分です。

- 唯一のドロップダウン（新作）を常時 open で固定し、狭幅はハンバーガー
  への自動切り替えではなく「狭幅（メニュー展開時）」インスタンスの並記
  で表現します。no-op のボタン（検索・アカウント・カート・メニュー
  トグル・drawer の閉じるボタン）はすべて `disabled` にします。
- R0710（告知文だけの帯 + ロゴ右寄せナビ）・R0712（ナビ中央配置）は、
  本 Demo はナビ中央配置を採用しています（親仕様はどちらでも可）。
- R0713（画像なしの項目 4 列のみ）は、本 Demo は注目画像カード 1 件 +
  項目列 3 列の 4 列グリッドで代表させています。
- R1313/R1314（濃色帯 + 画像のみのメガメニュー・アカウント操作）は、
  配色は既存トーンに揃えて淡色帯を採り、画像だけの構成は注目カードで
  代表させています。アカウントの操作は既存の `disabled` アイコン
  ボタンのまま残し、`menu` の静的展開は行いません。
- R1315（告知帯 + 画像 2 枚 + 3 節）は、画像 1 枚 + 項目列 3 列で代表
  させています。
- `menu`/`select`/`tabs` は Demo へ持ち込みません。親の使用部品候補には
  挙がっていますが、実際に使った部品だけを `parts` に宣言する契約のため
  です。
- 文言・ロゴ・画像はすべて独自に書いた架空のものです。

## 関連情報

[Navigation Menu](../themes/navigation-menu.md) /
[Link](../themes/link.md) / [Button](../themes/button.md) /
[Icon](../themes/icon.md) / [Image](../themes/image.md) /
[Native Select](../themes/native-select.md) / [Badge](../themes/badge.md) /
[Drawer](../themes/drawer.md)
