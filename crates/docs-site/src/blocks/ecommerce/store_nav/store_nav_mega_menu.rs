//! `store-nav-mega-menu` block（親トラッキング #3096、主参照 R0711。
//! sub-issue 前半 #3097 で骨格・主要領域（上部帯・バー・メガメニュー・
//! 狭幅の到達性維持）を実装し、本イシュー #3098（後半）が狭幅
//! （メニュー展開時）状態の並記・原稿の差分メモを仕上げる）。最上部の
//! 細い帯（告知文 + 言語/通貨切り替え）の下に、ロゴ・中央配置ナビ・
//! 検索/アカウント/カート操作を横一列に並べたストアナビゲーション。
//!
//! # 使用部品
//!
//! `navigation-menu` / `link` / `button` / `icon` / `image` /
//! `native-select` / `badge` / `drawer` の 8 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。`drawer` は狭幅インスタンスの並記
//! （下記「狭幅インスタンスの並記」節）でのみ使う。濃色帯 variant・
//! アカウント `menu` の静的 open は Demo へ並記せず、集約元との差分として
//! 「原案差分メモ」節（`site/blocks/store-nav-mega-menu.md`）で扱う
//! （詳細判断は同節参照）。
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
//! 狭幅時の「メニュー展開時」ドロワー状態の並記は下記「狭幅インスタンス
//! の並記」節を参照。
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
//! # 狭幅インスタンスの並記（[`mobile_preview`]）
//!
//! 無 JS のため開閉するメニューボタンは作れない
//! （[`crate::blocks::marketing::header::header_mega_menu`] モジュール doc
//! 「狭幅インスタンスの並記」節と同じ制約）。本イシューでは、狭幅
//! ビューポートでの見え方を「メニュー展開時」の状態として常時表示する
//! 第 2 のインスタンスを並記する（`@media` によるビューポート幅連動の
//! `display` 切り替えは一切行わないため、`hidden` 属性も持たない）。
//! 狭幅の展開済みメニューは [`crate::blocks::ecommerce::store_nav::store_nav_centered_logo`]
//! と同じく [`fandhe_frontend_pre_styled_ui::drawer`] の anatomy
//! （root/trigger/backdrop/positioner/content/title/close-trigger）で組む
//! （[`mobile_drawer`]）。メニューボタン（[`mobile_bar`]）は
//! [`drawer::trigger`](fandhe_frontend_pre_styled_ui::drawer::trigger) を
//! 使い、押しても状態が変わらない no-op のため `disabled: true`
//! （ネイティブ `disabled` 属性 + `data-disabled`）にする。`trigger` は
//! `state`/`controls` 引数から `aria-haspopup="dialog"`・
//! `aria-expanded`・`aria-controls` を自動で出力するため、これらを呼び
//! 出し側 `attrs` へ重複して渡さない。close-trigger も同じ理由で
//! `disabled: true` にする。`content` の `modal` 引数は `false` にする
//! （静的デモは閉じる機構を持たず外側に説明・コード・ナビゲーションが
//! あるため、表示実態と一致させる）。狭幅の [`nav`] は幅広と同じ関数を
//! [`Orientation::Vertical`] で呼び出す（実際の縦並びは [`LAYOUT_CSS`]
//! の `flex-direction: column` 上書きが担い、`data-orientation` は SSR
//! 静的属性のみで視覚は担わない。`nav` ランドマークの `aria-label` は
//! 幅広側「ストアメニュー」と重複しないよう「ストアメニュー（狭幅）」に
//! する）。帯の言語/通貨 `native-select` も [`band_controls`] を別 `id`
//! で再利用し、drawer 内へ複製する。
//!
//! `menu`/`select`/`tabs` は Demo に持ち込まない: 親の使用部品候補には
//! 挙がっているが、[`BLOCK`] の `parts` には実際に使った部品だけを宣言
//! するのが契約であり、アカウントの操作は既存の `disabled` アイコン
//! ボタンのまま残す（`site/blocks/store-nav-mega-menu.md` の
//! 「原案差分メモ」節参照）。命名に `hamburger` の語は使わない（`class`/
//! `data-*` は `menu-toggle`/`mobile` を使う）。
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
//! [`fandhe_frontend_pre_styled_ui::drawer::root`] も `size` variant
//! クラス付与のため呼び出し側 `attrs` の `class` を `drop_class_attr` で
//! 常に除去する契約（`crates/pre-styled-ui/src/drawer.rs` の `root`
//! rustdoc 参照）のため、`.blocks-store-nav-mega-menu-mobile` の class
//! フックは `drawer::root` の外側に置く `div` ラッパー（[`mobile_preview`]）
//! へ付け、`drawer::root` 自身へは渡さない。`drawer::trigger`/`backdrop`/
//! `positioner`/`content`/`title`/`close_trigger`（headless 直の再
//! エクスポート）は `class` を除去しないため、`content`/`positioner` の
//! CSS 上書きセレクタは `[data-scope="drawer"][data-part="..."]` 属性
//! セレクタで書く。
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
//! と、帯の 2 つの `native-select`（言語・通貨）、狭幅の展開済み drawer
//! にのみ必要であり、`blocks-store-nav-mega-menu-new-{trigger|content}`・
//! `blocks-store-nav-mega-menu-band-{language|currency}`（幅広）・
//! `blocks-store-nav-mega-menu-mobile-new-{trigger|content}`・
//! `blocks-store-nav-mega-menu-mobile-band-{language|currency}`・
//! `blocks-store-nav-mega-menu-mobile-drawer`（drawer の `content` 自体）・
//! `blocks-store-nav-mega-menu-mobile-drawer-title`（狭幅）の固定文字列で
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
        Part {
            label: "Drawer",
            path: "/themes/drawer/",
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
.blocks-store-nav-mega-menu-states {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-store-nav-mega-menu-state-label {\n  display: block;\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: 600;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-store-nav-mega-menu-mobile {\n  max-inline-size: 24rem;\n  border-width: 1px;\n  border-style: solid;\n  border-color: var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  background: var(--fandhe-color-bg);\n  overflow: hidden;\n}\n\
.blocks-store-nav-mega-menu-mobile-bar {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-store-nav-mega-menu-mobile-panel-wrap {\n  position: relative;\n}\n\
.blocks-store-nav-mega-menu-mobile [data-scope=\"drawer\"][data-part=\"backdrop\"] {\n  position: absolute;\n  inset: 0;\n  z-index: auto;\n}\n\
.blocks-store-nav-mega-menu-mobile [data-scope=\"drawer\"][data-part=\"positioner\"] {\n  position: relative;\n  inset: auto;\n  z-index: auto;\n  padding: var(--fandhe-space-3);\n}\n\
.blocks-store-nav-mega-menu-mobile [data-scope=\"drawer\"][data-part=\"content\"] {\n  width: min(100%, var(--fandhe-drawer-size, 20rem));\n  height: auto;\n}\n\
.blocks-store-nav-mega-menu-mobile [data-scope=\"drawer\"] h2 {\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n  font-size: var(--fandhe-font-font-size-md);\n}\n\
.blocks-store-nav-mega-menu-mobile [data-scope=\"navigation-menu\"][data-part=\"item\"] {\n  position: static;\n  inline-size: 100%;\n}\n\
.blocks-store-nav-mega-menu-mobile [data-scope=\"navigation-menu\"][data-part=\"list\"] {\n  display: flex;\n  flex-direction: column;\n  align-items: stretch;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-store-nav-mega-menu-mobile [data-scope=\"navigation-menu\"][data-part=\"content\"] {\n  position: static;\n  inset-inline: auto;\n  padding: 0;\n  margin-top: var(--fandhe-space-2);\n  border-style: none;\n  box-shadow: none;\n}\n\
.blocks-store-nav-mega-menu-mobile [data-scope=\"navigation-menu\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-store-nav-mega-menu-mobile .blocks-store-nav-mega-menu-panel-inner {\n  grid-template-columns: 1fr;\n}\n\
.blocks-store-nav-mega-menu-mobile .blocks-store-nav-mega-menu-band-controls {\n  flex-wrap: wrap;\n  justify-content: flex-start;\n}\n\
.blocks-store-nav-mega-menu-mobile .blocks-store-nav-mega-menu-actions {\n  flex-wrap: wrap;\n  justify-content: flex-start;\n}\n\
@container blocks-store-nav-mega-menu (max-width: 50.99rem) {\n  .blocks-store-nav-mega-menu-nav[data-scope=\"navigation-menu\"][data-part=\"root\"] {\n    order: 1;\n    flex-basis: 100%;\n  }\n  .blocks-store-nav-mega-menu-actions {\n    order: 2;\n  }\n  .blocks-store-nav-mega-menu-shell [data-scope=\"navigation-menu\"][data-part=\"content\"] {\n    position: static;\n  }\n  .blocks-store-nav-mega-menu-shell [data-scope=\"navigation-menu\"][data-part=\"item\"][data-state=\"open\"] {\n    flex-basis: 100%;\n  }\n  .blocks-store-nav-mega-menu-panel-inner {\n    grid-template-columns: 1fr;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{
        demo, BLOCK, CURRENCY_SELECT_ID, LANGUAGE_SELECT_ID, LAYOUT_CSS, MOBILE_CURRENCY_SELECT_ID,
        MOBILE_DRAWER_CONTENT_ID, MOBILE_DRAWER_TITLE_ID, MOBILE_LANGUAGE_SELECT_ID,
        MOBILE_NEW_CONTENT_ID, MOBILE_NEW_TRIGGER_ID, NEW_CONTENT_ID, NEW_TRIGGER_ID,
    };
    use fandhe_frontend_core::render;

    /// Demo が期待する 8 種の部品・非対話制約を満たすことの単体回帰
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
            "data-scope=\"drawer\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 唯一のドロップダウン（新作）が幅広・狭幅の各インスタンスで常時
    /// open で固定され、`hidden` を持たないこと。閉じた trigger（リンクの
    /// みの項目）は存在しないこと。drawer の trigger/content は
    /// `data-scope="navigation-menu"` を持たないため、本テストは
    /// `data-scope="navigation-menu"` で絞って数える。
    #[test]
    fn single_dropdown_is_open_and_trigger_disabled() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-part="item" data-state="open""#)
                .count(),
            2,
            "html={html}"
        );
        assert_eq!(
            html.matches("data-scope=\"navigation-menu\" data-part=\"content\"")
                .count(),
            2
        );
        assert_eq!(
            html.matches(r#"data-scope="navigation-menu" data-part="trigger""#)
                .count(),
            2
        );
        assert!(!html.contains(r#"data-part="trigger" aria-expanded="false""#));

        for (trigger_id, content_id) in [
            (NEW_TRIGGER_ID, NEW_CONTENT_ID),
            (MOBILE_NEW_TRIGGER_ID, MOBILE_NEW_CONTENT_ID),
        ] {
            let trigger_start = html
                .find(&format!("id=\"{trigger_id}\""))
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
            assert!(html.contains(&format!("aria-controls=\"{content_id}\"")));
            assert!(html.contains(&format!("id=\"{content_id}\"")));
            assert!(html.contains(&format!("aria-labelledby=\"{trigger_id}\"")));
        }
    }

    /// 狭幅インスタンスのメニュートグルボタンが展開済み drawer を指し、
    /// `disabled` で操作不能であることを固定する（モジュール冒頭
    /// rustdoc「狭幅インスタンスの並記」節）。
    #[test]
    fn mobile_drawer_is_open_non_modal_and_toggle_disabled() {
        let html = render(&demo());
        let toggle_pos = html
            .find("data-blocks-store-nav-mega-menu-menu-toggle")
            .expect("menu toggle attr should be present");
        let tag_start = html[..toggle_pos].rfind("<button").unwrap();
        let tag_end = html[tag_start..].find('>').unwrap() + tag_start;
        let toggle_tag = &html[tag_start..tag_end];
        assert!(
            toggle_tag.contains(r#"aria-haspopup="dialog""#),
            "{toggle_tag}"
        );
        assert!(
            toggle_tag.contains(r#"aria-expanded="true""#),
            "{toggle_tag}"
        );
        assert!(
            toggle_tag.contains(&format!(r#"aria-controls="{MOBILE_DRAWER_CONTENT_ID}""#)),
            "{toggle_tag}"
        );
        assert!(toggle_tag.contains("disabled=\"\""), "{toggle_tag}");

        let content_marker = r#"data-scope="drawer" data-part="content""#;
        let content_pos = html
            .find(content_marker)
            .expect("drawer content should be present");
        let tag_start = html[..content_pos].rfind('<').unwrap();
        let tag_end = html[tag_start..].find('>').unwrap() + tag_start;
        let content_tag = &html[tag_start..tag_end];
        assert!(!content_tag.contains("hidden"), "{content_tag}");
        assert!(
            content_tag.contains(r#"aria-modal="false""#),
            "{content_tag}"
        );
        assert!(
            content_tag.contains(&format!("aria-labelledby=\"{MOBILE_DRAWER_TITLE_ID}\"")),
            "{content_tag}"
        );
        assert!(html.contains(&format!("id=\"{MOBILE_DRAWER_CONTENT_ID}\"")));
    }

    /// 狭幅インスタンスの `id`（新作トリガー/content・言語/通貨 select）が
    /// 幅広側の `id` と重複しないことを固定する。
    #[test]
    fn mobile_ids_differ_from_wide_ids() {
        let html = render(&demo());
        assert_ne!(MOBILE_NEW_TRIGGER_ID, NEW_TRIGGER_ID);
        assert_ne!(MOBILE_NEW_CONTENT_ID, NEW_CONTENT_ID);
        assert_ne!(MOBILE_LANGUAGE_SELECT_ID, LANGUAGE_SELECT_ID);
        assert_ne!(MOBILE_CURRENCY_SELECT_ID, CURRENCY_SELECT_ID);
        assert!(html.contains(&format!("id=\"{MOBILE_LANGUAGE_SELECT_ID}-control\"")));
        assert!(html.contains(&format!("id=\"{MOBILE_CURRENCY_SELECT_ID}-control\"")));
    }

    /// [`demo`] が幅広・狭幅の各状態見出しを含むこと（モジュール冒頭
    /// rustdoc「狭幅インスタンスの並記」節）。
    #[test]
    fn demo_shows_both_state_labels() {
        let html = render(&demo());
        assert!(html.contains("幅広（新作を展開）"));
        assert!(html.contains("狭幅（メニュー展開時）"));
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

    /// カートの badge が幅広・狭幅の各インスタンス 1 件ずつ、計 2 件
    /// 描画され、件数文字列を含むこと。
    #[test]
    fn cart_badge_renders_count() {
        let html = render(&demo());
        assert_eq!(html.matches("data-scope=\"badge\"").count(), 2, "{html}");
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
