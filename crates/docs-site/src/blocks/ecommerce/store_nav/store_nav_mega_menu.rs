//! `store-nav-mega-menu` block（親トラッキング #3096、主参照 R0711。
//! sub-issue 前半 #3097 で骨格・主要領域（上部帯・バー・メガメニュー・
//! 狭幅の到達性維持）を実装し、後半 #3098 が濃色帯 variant・アカウント
//! `menu`・狭幅ドロワー並記・原稿の差分メモを仕上げる）。最上部の細い帯
//! （告知文 + 言語/通貨切り替え）の下に、ロゴ・中央配置ナビ・検索/
//! アカウント/カート操作を横一列に並べたストアナビゲーション。Ecommerce /
//! Store Nav カテゴリの最初の block（`super`（`store_nav/mod.rs`）参照）。
//!
//! # 使用部品
//!
//! `navigation-menu` / `link` / `button` / `icon` / `image` /
//! `native-select` / `badge` の 7 部品を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。`menu`/`select`/`drawer`/`tabs` は
//! 後半 #3098 で使う場合にそこで追加する（本 block では宣言しない）。
//!
//! # 静的表示（無 JS、唯一のドロップダウンを常時 open で固定）
//!
//! ドロップダウンを持つトップ項目は「新作」1 件のみとし、
//! [`OpenState::Open`] で固定する。トリガーは押しても状態が変わらない
//! no-op になるため `disabled: true`（ネイティブ `disabled` 属性 +
//! `data-disabled`）にしてフォーカス・クリック不能を明示する
//! （[`crate::blocks::marketing::header::header_mega_menu`] と同じ判断。
//! `disabled_declarations()` は [`LAYOUT_CSS`] で中和し、開閉可能な見出し
//! として自然に見せる）。残りのトップ項目（レディース・メンズ・小物・
//! セール）は [`navigation_menu::trigger`] を持たず
//! [`navigation_menu::item`] + [`navigation_menu::link`] のリンク項目のみ
//! で構成する（閉じたままフォーカス可能だが操作しても何も起きない
//! trigger を作らないため）。検索・アカウント・カートの操作アイコン
//! ボタンは送信先・遷移先を持たない no-op のため `disabled: true` のまま
//! 中和しない（`header_mega_menu` の CTA ボタンと同じ判断）。
//!
//! # 全幅パネルの配置方法（`position: static` 上書きと包含ブロック）
//!
//! [`fandhe_frontend_pre_styled_ui::navigation_menu`] の recipe は
//! `root`/`item` に `position: relative` を、`content` に `position:
//! absolute; top: 100%; left: 0; min-width: 10rem;` を宣言する。このままで
//! はパネルがトリガー 1 個ぶんの幅にしか広がらない。[`LAYOUT_CSS`] は本
//! block のスコープ内（`.blocks-store-nav-mega-menu-shell` 子孫セレクタ）に
//! 限定して `root`/`item` の `position` を `static` へ上書きし、
//! `.blocks-store-nav-mega-menu-bar-wrap`（`position: relative`、上部帯を
//! 含まずバー 1 行のみを内包）を `content` の包含ブロックへ格上げする
//! （[`crate::blocks::marketing::header::header_mega_menu`] と同型の判断。上部帯まで
//! 含む `shell` 自身を包含ブロックにすると `top: 100%` が帯の高さぶん
//! ずれる）。`content` 自身は `inset-inline: 0; min-width: 0;` を追加
//! 宣言してヘッダー全幅へ広げる。
//!
//! # レスポンシブ（`@container`、狭幅でも内容を隠さない）
//!
//! [`crate::blocks::application::navbar::navbar_two_row`] と同じく `@container` を使い、
//! Demo 枠自体の幅（ビューポート幅ではない）に応じて折り返す
//! （`.blocks-store-nav-mega-menu-shell` に `container-type: inline-size`
//! を宣言）。狭幅ではバー・ナビ一覧・操作・上部帯をいずれも
//! `flex-wrap: wrap` で折り返すのみとし、`display: none` で隠さない
//! （ハンバーガーメニューへの畳み込みは行わない。無 JS では開閉処理を
//! 持たせられず、隠すと到達不能になる `header_mega_menu` と同じ教訓）。
//! 狭幅時の「メニュー展開時」ドロワー状態の並記は後半 #3098 のスコープ。
//! 狭幅では中央ナビを `order: 1`・操作領域を `order: 2` で明示し、DOM 順
//! （ブランド → ナビ → 操作）と表示順を一致させる（レビュー是正 #3475:
//! ナビのみへ `order: 2` を付け操作領域を既定 `order: 0` のままにしていた
//! ため、狭幅で操作領域がナビより先に表示され DOM 順と食い違っていた）。
//! 常時 open の「新作」項目（`[data-part="item"][data-state="open"]`）にも
//! `flex-basis: 100%` を与え、ナビ一覧の flex 兄弟として他の項目と並ぶ・
//! 折り返されるのを避けて専有行を確保する（レビュー是正 #3475）。
//!
//! # `.blocks-demo` のはみ出し対策
//!
//! `crate::blocks::stylesheet` の `.blocks-demo` は `overflow-x: auto` を
//! 持つ（CSS 仕様上 `overflow-y` も暗黙に `auto` へ計算される）。「新作」
//! パネルは常時 open だが `content` は `position: absolute` のままで
//! フローに寄与しないため、`.blocks-store-nav-mega-menu-page`（ダミー本文
//! 枠、[`crate::blocks::marketing::header::header_mega_menu`] の
//! `page_placeholder` と同型の判断）へ `min-block-size` を持たせ、常時
//! 展開済みパネルがレイアウトボックスの内側に収まるようにする（レビュー
//! 是正: 元の実装はこの枠を持たず、パネルが後続の「使用部品」見出し等と
//! 重なる・`.blocks-demo` 内でクリップされる不具合があった）。狭幅
//! （`@container` 50.99rem 以下）では列が 1 列積みになり注目画像カード＋
//! 列見出し＋リンクの合計高さが伸びるため、固定 `min-block-size` の加算
//! では不足しうる（レビュー是正 #3475: 後続ドキュメントレビューで固定
//! 40rem が内容合計を超える場合の重なり・クリップを指摘された）。固定値を
//! 積み増す代わりに `content`（`data-part="content"`）自体を
//! `position: static` へ上書きして通常フローへ戻し、後続の
//! [`page_placeholder`] がパネルの実高さぶん自然に押し下げられるようにする
//! （固定値の当てずっぽうをやめ、内容にかかわらず重なり・クリップが起きない
//! 構造にする判断）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `navigation_menu` のパート関数は呼び出し側 `attrs` の `class` を除去
//! しない（`crates/headless-ui/src/navigation_menu.rs` 参照）ため
//! `.blocks-store-nav-mega-menu-nav` のような class フックがそのまま使える。
//! 一方 `button::icon_button`/`native_select::native_select`/`image::image`/
//! `badge::badge` は `drop_class_attr` で呼び出し側 `class` を常に除去する
//! 契約のため、これらへのフックは `data-blocks-store-nav-mega-menu-*`
//! 属性で渡す（[`crate::blocks::marketing::header::header_mega_menu`] と同型の判断）。
//!
//! # href の方針
//!
//! `href="#"` は使わない（横断テストが禁止する）。パネル項目・トップ項目
//! はサイト内に実在する索引ページへの相対パス（`../../themes/`・
//! `../../primitives/`・`../../guides/`・`../../examples/`・`../../api/`）
//! を使う。
//!
//! # id 接頭辞
//!
//! `id`/`aria-controls`/`aria-labelledby` は開いたドロップダウン（新作）
//! と、帯の 2 つの `native-select`（言語・通貨）にのみ必要であり、
//! `blocks-store-nav-mega-menu-new-{trigger|content}`・
//! `blocks-store-nav-mega-menu-band-{language|currency}` の固定文字列で
//! 一意にする（複数項目に添字展開する block とは異なり `format!` を
//! 使わない）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンは既定の `type="button"` のまま送信先を持たない。
//! 言語/通貨の `native-select` はネイティブ操作可能な部品として enabled の
//! まま残す（値の送信・保存は行わない静的表示、
//! `crate::blocks::marketing::contact::contact_centered_form` と同じ
//! 判断軸）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/store-nav-mega-menu/",
    title: "store-nav-mega-menu",
    category: BlockCategory::StoreNav,
    rust_source: "crates/docs-site/src/blocks/ecommerce/store_nav/store_nav_mega_menu.rs",
    demo_class: "blocks-store-nav-mega-menu",
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
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Native Select",
            path: "/themes/native-select/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `store_nav_mega_menu` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。
///
/// セレクタは `.blocks-store-nav-mega-menu-*`、
/// `[data-blocks-store-nav-mega-menu-*]`、および
/// `.blocks-store-nav-mega-menu-shell` を祖先に持つ
/// `[data-scope="navigation-menu"]` 系セレクタへの子孫結合子付き上書き
/// （全幅パネル化、モジュール冒頭 rustdoc「全幅パネルの配置方法」節）
/// のみを用いる。値はすべて `var(--fandhe-*)` トークンで書き、生の色
/// リテラルは使わない。狭幅の折り返しは `@container`（Demo 枠幅基準、
/// モジュール冒頭 rustdoc「レスポンシブ」節）で行う。4 列パネル
/// （`panel-inner`）の列最小幅合計は 14rem + 10rem × 3 = 44rem、列間
/// gap（`--fandhe-space-6` = 1.5rem × 3）+ 左右 padding
/// （`--fandhe-space-4` = 1rem × 2）を足すと 50.5rem になる。切替幅を
/// これより低い `47.99rem` にすると約 48〜50.5rem の Demo 枠で 4 列の
/// まま最小幅合計がコンテナ幅を超えパネルが横にはみ出す（レビュー是正
/// #3475）ため、切替幅は 50.5rem 以上の `50.99rem` にする。
const LAYOUT_CSS: &str = "\
.blocks-store-nav-mega-menu-shell {\n  container-type: inline-size;\n  container-name: blocks-store-nav-mega-menu;\n}\n\
.blocks-store-nav-mega-menu-band {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n  padding: var(--fandhe-space-2) var(--fandhe-space-4);\n  background: var(--fandhe-color-bg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-store-nav-mega-menu-band-controls {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-store-nav-mega-menu-bar-wrap {\n  position: relative;\n  background: var(--fandhe-color-bg);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-store-nav-mega-menu-bar {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  max-inline-size: 64rem;\n  margin-inline: auto;\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n}\n\
.blocks-store-nav-mega-menu-brand {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  font-weight: 600;\n  white-space: nowrap;\n}\n\
.blocks-store-nav-mega-menu-nav[data-scope=\"navigation-menu\"][data-part=\"root\"] {\n  flex: 1 1 auto;\n  display: flex;\n  justify-content: center;\n}\n\
.blocks-store-nav-mega-menu-shell [data-scope=\"navigation-menu\"][data-part=\"root\"],\n\
.blocks-store-nav-mega-menu-shell [data-scope=\"navigation-menu\"][data-part=\"item\"] {\n  position: static;\n}\n\
.blocks-store-nav-mega-menu-shell [data-scope=\"navigation-menu\"][data-part=\"list\"] {\n  display: flex;\n  flex-wrap: wrap;\n  justify-content: center;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-store-nav-mega-menu-shell [data-scope=\"navigation-menu\"][data-part=\"content\"] {\n  inset-inline: 0;\n  min-width: 0;\n}\n\
.blocks-store-nav-mega-menu-shell [data-scope=\"navigation-menu\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-store-nav-mega-menu-panel-inner {\n  max-inline-size: 64rem;\n  margin-inline: auto;\n  display: grid;\n  grid-template-columns: minmax(min(14rem, 100%), 1fr) repeat(3, minmax(min(10rem, 100%), 1fr));\n  gap: var(--fandhe-space-6);\n  padding: var(--fandhe-space-4);\n}\n\
[data-blocks-store-nav-mega-menu-featured] {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-store-nav-mega-menu-featured-title {\n  font-weight: 600;\n}\n\
.blocks-store-nav-mega-menu-featured-description {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-store-nav-mega-menu-panel-column {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-store-nav-mega-menu-panel-column-heading {\n  display: block;\n  font-weight: 600;\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n  margin-block-end: var(--fandhe-space-1);\n}\n\
.blocks-store-nav-mega-menu-actions {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  white-space: nowrap;\n}\n\
.blocks-store-nav-mega-menu-cart {\n  position: relative;\n  display: inline-flex;\n}\n\
[data-blocks-store-nav-mega-menu-cart-badge] {\n  position: absolute;\n  top: -0.25rem;\n  right: -0.25rem;\n}\n\
.blocks-store-nav-mega-menu-page {\n  min-block-size: 22rem;\n  padding: var(--fandhe-space-6) var(--fandhe-space-4);\n  color: var(--fandhe-color-fg-muted);\n}\n\
@container blocks-store-nav-mega-menu (max-width: 50.99rem) {\n  .blocks-store-nav-mega-menu-nav[data-scope=\"navigation-menu\"][data-part=\"root\"] {\n    order: 1;\n    flex-basis: 100%;\n  }\n  .blocks-store-nav-mega-menu-actions {\n    order: 2;\n  }\n  .blocks-store-nav-mega-menu-shell [data-scope=\"navigation-menu\"][data-part=\"content\"] {\n    position: static;\n  }\n  .blocks-store-nav-mega-menu-shell [data-scope=\"navigation-menu\"][data-part=\"item\"][data-state=\"open\"] {\n    flex-basis: 100%;\n  }\n  .blocks-store-nav-mega-menu-panel-inner {\n    grid-template-columns: 1fr;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{
        demo, BLOCK, CURRENCY_SELECT_ID, LANGUAGE_SELECT_ID, LAYOUT_CSS, NEW_CONTENT_ID,
        NEW_TRIGGER_ID,
    };
    use fandhe_frontend_core::render;

    /// Demo が期待する 7 種の部品・非対話制約を満たすことの単体回帰
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"navigation-menu\"",
            "data-scope=\"link\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
            "data-scope=\"image\"",
            "data-scope=\"field\"",
            "data-scope=\"badge\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 唯一のドロップダウン（新作）が常時 open で固定され、`hidden` を
    /// 持たないこと。閉じた trigger（リンクのみの項目）は存在しないこと。
    #[test]
    fn single_dropdown_is_open_and_trigger_disabled() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-part="item" data-state="open""#)
                .count(),
            1,
            "html={html}"
        );
        assert_eq!(html.matches("data-part=\"content\"").count(), 1);
        assert_eq!(html.matches(r#"data-part="trigger""#).count(), 1);
        assert!(!html.contains(r#"data-part="trigger" aria-expanded="false""#));

        let trigger_start = html
            .find(&format!("id=\"{NEW_TRIGGER_ID}\""))
            .expect("trigger id should be present");
        let tag_start = html[..trigger_start].rfind("<button").unwrap();
        let tag_end = html[tag_start..].find('>').unwrap() + tag_start;
        let trigger_tag = &html[tag_start..tag_end];
        assert!(trigger_tag.contains("disabled=\"\""), "{trigger_tag}");
        assert!(trigger_tag.contains(r#"data-disabled="""#), "{trigger_tag}");
        assert!(
            trigger_tag.contains(r#"aria-expanded="true""#),
            "{trigger_tag}"
        );
        assert!(html.contains(&format!("aria-controls=\"{NEW_CONTENT_ID}\"")));
        assert!(html.contains(&format!("id=\"{NEW_CONTENT_ID}\"")));
        assert!(html.contains(&format!("aria-labelledby=\"{NEW_TRIGGER_ID}\"")));
    }

    /// 帯の 2 つの `native-select` が異なる `id`（`field::select` の既定
    /// 派生規則により `"{id}-control"`）を持ち、それぞれ
    /// `aria-label="言語"`/`"通貨"` を持つこと。
    #[test]
    fn band_selects_have_distinct_ids_and_labels() {
        let html = render(&demo());
        assert_ne!(LANGUAGE_SELECT_ID, CURRENCY_SELECT_ID);
        assert!(html.contains(&format!("id=\"{LANGUAGE_SELECT_ID}-control\"")));
        assert!(html.contains(&format!("id=\"{CURRENCY_SELECT_ID}-control\"")));
        assert!(html.contains(r#"aria-label="言語""#));
        assert!(html.contains(r#"aria-label="通貨""#));
    }

    /// カートの badge が 1 件のみ描画され、件数文字列を含むこと。
    #[test]
    fn cart_badge_renders_count() {
        let html = render(&demo());
        assert_eq!(html.matches("data-scope=\"badge\"").count(), 1, "{html}");
        assert!(html.contains(">3<"));
    }

    /// 注目画像カードが `href` ではなく共有ダミー画像アセットを使うこと。
    #[test]
    fn featured_card_uses_shared_product_asset() {
        let html = render(&demo());
        assert!(html.contains("src=\"../../assets/blocks-demo-product.svg\""));
    }

    /// 狭幅でも内容を隠さないこと（`@container`・`display: none`/
    /// `hamburger` 不使用の固定）。
    #[test]
    fn narrow_layout_keeps_everything_reachable() {
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-store-nav-mega-menu"));
        assert!(!LAYOUT_CSS.contains("display: none"));
        assert!(!LAYOUT_CSS.contains("hamburger"));
        // `hidden` 存在属性そのもの（`aria-hidden`（装飾アイコン）は対象外）
        // を持つ要素が無いこと。
        assert!(!render(&demo()).contains(" hidden"));
    }

    /// 全幅パネル化（`position: static` 上書き・`inset-inline: 0`）を
    /// 持つこと。
    #[test]
    fn layout_css_makes_panel_full_width() {
        assert!(LAYOUT_CSS.contains("position: static;"));
        assert!(LAYOUT_CSS.contains("inset-inline: 0;"));
        assert!(LAYOUT_CSS.contains("minmax(min(14rem, 100%), 1fr)"));
    }

    /// 常時 open のパネルが `.blocks-demo` のはみ出し・クリップを起こさぬ
    /// よう、ダミー本文枠（`min-block-size`）で高さを確保すること（レビュー
    /// 是正、モジュール冒頭 rustdoc「`.blocks-demo` のはみ出し対策」節）。
    #[test]
    fn page_placeholder_reserves_space_for_open_panel() {
        assert!(LAYOUT_CSS.contains(".blocks-store-nav-mega-menu-page"));
        assert!(LAYOUT_CSS.contains("min-block-size: 22rem;"));
        assert!(render(&demo()).contains("class=\"blocks-store-nav-mega-menu-page\""));
    }

    /// 狭幅（`@container` 50.99rem 以下）ではパネルの内容合計が固定高さを
    /// 超えうるため、固定 `min-block-size` を積み増すのではなく `content`
    /// を通常フローへ戻す（レビュー是正 #3475、モジュール冒頭 rustdoc
    /// 「`.blocks-demo` のはみ出し対策」節）。固定 40rem 加算のような
    /// 当てずっぽうの数値には戻さないことを固定する。
    #[test]
    fn narrow_panel_returns_to_normal_flow_instead_of_fixed_height() {
        let (_, narrow) = LAYOUT_CSS
            .split_once("@container")
            .expect("LAYOUT_CSS should declare narrow @container rules");
        assert!(
            narrow.contains("[data-part=\"content\"]") && narrow.contains("position: static;"),
            "narrow rules should return the panel content to normal flow: {narrow}"
        );
        assert!(
            !narrow.contains("min-block-size"),
            "narrow rules should not reserve a fixed magic-number height: {narrow}"
        );
    }

    /// バーの `order` 上書きが幅広時に存在しないこと（DOM 順＝ブランド →
    /// ナビ → 操作のまま、余計な `order` を付けない）。
    #[test]
    fn wide_bar_order_matches_dom_order() {
        let (wide, _) = LAYOUT_CSS
            .split_once("@container")
            .expect("LAYOUT_CSS should declare narrow @container rules");
        assert!(
            !wide.contains("order:"),
            "wide bar should rely on DOM order (brand, nav, actions) without `order` overrides: {wide}"
        );
    }

    /// 狭幅 `@container` 内の `order` 上書きが DOM 順（ブランド → ナビ →
    /// 操作）と一致すること（レビュー是正 #3475: ナビのみへ `order: 2` を
    /// 付け操作領域を既定 `order: 0` のまま残していたため、狭幅で操作領域
    /// がナビより先に表示され DOM 順と食い違っていた。ナビを `order: 1`・
    /// 操作領域を `order: 2` にして DOM 順（ブランド=0 < ナビ=1 <
    /// 操作=2）を保ったまま、ナビだけが `flex-basis: 100%` で専有行に
    /// 折り返るようにする）。
    #[test]
    fn narrow_bar_order_matches_dom_order() {
        let (_, narrow) = LAYOUT_CSS
            .split_once("@container")
            .expect("LAYOUT_CSS should declare narrow @container rules");
        assert!(
            narrow.contains(".blocks-store-nav-mega-menu-nav") && narrow.contains("order: 1;"),
            "narrow nav should use order: 1 (after brand, before actions): {narrow}"
        );
        assert!(
            narrow.contains(".blocks-store-nav-mega-menu-actions") && narrow.contains("order: 2;"),
            "narrow actions should use order: 2 (after nav) to match DOM order: {narrow}"
        );
    }

    /// 常時 open の「新作」項目（`[data-part="item"][data-state="open"]`）
    /// が狭幅で `flex-basis: 100%` を持ち、ナビ一覧の flex 兄弟として他の
    /// 項目と並ぶ・折り返されるのを避けて専有行を確保すること（レビュー
    /// 是正 #3475、Bugbot 指摘: in-flow panel breaks narrow nav layout）。
    #[test]
    fn narrow_open_item_reserves_own_flex_line() {
        let (_, narrow) = LAYOUT_CSS
            .split_once("@container")
            .expect("LAYOUT_CSS should declare narrow @container rules");
        assert!(
            narrow.contains(r#"[data-part="item"][data-state="open"]"#)
                && narrow.contains("flex-basis: 100%;"),
            "narrow open item should reserve its own flex line: {narrow}"
        );
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-store-nav-mega-menu-shell\""));
        assert_ne!(BLOCK.demo_class, "blocks-store-nav-mega-menu-shell");
    }
}
