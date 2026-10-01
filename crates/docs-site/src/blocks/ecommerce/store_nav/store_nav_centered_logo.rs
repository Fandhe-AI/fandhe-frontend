//! `store-nav-centered-logo` block（親トラッキング #3093、対応表 ID R1316
//! を主参照とする合成例。前半 #3094 で骨格・主要領域・Blocks 登録一式を
//! 仕上げ、本イシュー #3095（後半）で狭幅（メニュー展開時）状態の並記を
//! 仕上げる）。上部帯を持たない 1 行バーの中央にロゴ、左にカテゴリナビ、
//! 右に通貨・検索・アカウント・カートを配置するストアナビゲーション。
//!
//! # 使用部品
//!
//! `navigation-menu` / `link` / `button` / `icon` / `image` / `drawer` の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! `drawer` は狭幅インスタンスの並記（下記「狭幅インスタンスの並記」節）
//! でのみ使う。
//!
//! # 静的表示（無 JS、唯一のドロップダウンを常時 open で固定）
//!
//! [`super::super::header::header_mega_menu`] と同じ判断軸を踏襲する。
//! ドロップダウンを持つトップ項目は「レディース」1 件のみとし、
//! [`OpenState::Open`] で固定する。トリガーは押しても状態が変わらない
//! no-op になるため `disabled: true`（ネイティブ `disabled` 属性 +
//! `data-disabled`）にしてフォーカス・クリック不能を明示し、
//! `disabled_declarations()`（既定 `opacity: 0.5`）は [`LAYOUT_CSS`] で
//! 中和して通常のトリガーと同じ見た目に保つ。残りのトップ項目
//! （メンズ・ホーム用品）は [`navigation_menu::trigger`] を持たず
//! [`navigation_menu::item`] + [`navigation_menu::link`] のリンク項目
//! のみで構成する（閉じたままフォーカス可能だが操作しても何も起きない
//! trigger を作らないため）。
//!
//! # 全幅パネルの配置方法（`position: static` 上書きと包含ブロック）
//!
//! [`super::super::header::header_mega_menu`] と同型の判断。
//! [`fandhe_frontend_pre_styled_ui::navigation_menu`] の recipe は
//! `root`/`item` に `position: relative` を、`content` に `position:
//! absolute; top: 100%; left: 0;` を宣言するため、そのままではパネルが
//! ヘッダー全幅まで広がらない。[`LAYOUT_CSS`] は本 block のスコープ内に
//! 限定して `root`/`item` の `position` を `static` へ上書きし、
//! `.blocks-store-nav-centered-logo-bar-wrap`（`position: relative`）を
//! `content` の包含ブロックへ格上げする（`layout` 自身ではなく
//! `bar-wrap` を選ぶ理由は `header_mega_menu` と同じ: `layout` はバーの下に
//! ダミー本文まで含むため、`top: 100%` の基準がずれる）。`content` 自身は
//! `inset-inline: 0; min-width: 0;` を追加宣言してヘッダー全幅へ広げる。
//!
//! # id 接頭辞
//!
//! `id`/`aria-controls`/`aria-labelledby` は開いたドロップダウン（幅広・
//! 狭幅の各インスタンスの「レディース」1 件ずつ）と、狭幅の展開済み
//! drawer にのみ必要であり、`blocks-store-nav-centered-logo-women-
//! {trigger|content}`（幅広）・`blocks-store-nav-centered-logo-mobile-
//! women-{trigger|content}`（狭幅）・`blocks-store-nav-centered-logo-
//! mobile-drawer`（drawer の `content` 自体）・`blocks-store-nav-centered-
//! logo-mobile-drawer-title` の固定文字列で一意にする（項目が 1 件のみの
//! ため `format!` は使わない。`crate::blocks` モジュール doc「HTML 文字列
//! の直接組み立て禁止」節参照）。
//!
//! # href の方針
//!
//! `href="#"` は使わない（横断テストが禁止する）。パネル項目・中央ロゴは
//! サイト内に実在する索引ページへの相対パス（`../../`・`../../guides/`・
//! `../../themes/`・`../../primitives/`・`../../api/`・`../../examples/`）を
//! 使う（[`super::super::header::header_mega_menu`] と同型の判断）。
//!
//! # 狭幅インスタンスの並記（[`mobile_preview`]）
//!
//! 無 JS のため開閉するメニューボタンは作れない
//! （[`super::super::header::header_mega_menu`] モジュール doc「狭幅
//! インスタンスの並記」節と同じ制約）。本イシューでは、狭幅ビューポート
//! での見え方を「メニュー展開時」の状態として常時表示する第 2 の
//! インスタンスを並記する（`@media` によるビューポート幅連動の
//! `display` 切り替えは一切行わないため、`hidden` 属性も持たない）。
//! 親トラッキング #3093 の使用部品一覧に `drawer` が含まれ、かつ前半
//! #3094 のモジュール doc が「後半でドロワーを使って並記を追加する」旨を
//! 既に宣言していたため、狭幅の展開済みメニューは
//! [`super::super::header::header_mega_menu`] のような素の `nav`
//! 直置きではなく [`fandhe_frontend_pre_styled_ui::drawer`] の anatomy
//! （root/trigger/backdrop/positioner/content/title/close-trigger）で
//! 組む（[`mobile_drawer`]）。メニューボタン（[`mobile_bar`]）は
//! [`drawer::trigger`](fandhe_frontend_pre_styled_ui::drawer::trigger) を
//! 使い、押しても状態が変わらない no-op のため
//! `disabled: true`（ネイティブ `disabled` 属性 + `data-disabled`）に
//! する。`trigger` は `state`/`controls` 引数から `aria-haspopup="dialog"`
//! ・`aria-expanded`・`aria-controls` を自動で出力するため、これらを
//! 呼び出し側 `attrs` へ重複して渡さない。close-trigger も同じ理由で
//! `disabled: true` にする。`content` の `modal` 引数は `false` にする
//! （静的デモは閉じる機構を持たず外側に説明・コード・ナビゲーションが
//! あるため、表示実態と一致させる。
//! [`super::super::contact::contact_dialog_form`] と同じ判断）。狭幅の
//! カテゴリナビは幅広と同じ [`category_nav`] を
//! [`Orientation::Vertical`] で呼び出す（実際の縦並びは [`LAYOUT_CSS`]
//! の `flex-direction: column` 上書きが担い、`data-orientation` は SSR
//! 静的属性のみで視覚は担わない。`nav` ランドマークの `aria-label` は
//! 幅広側「カテゴリ」と重複しないよう「カテゴリ（狭幅）」にする）。
//! 命名に `hamburger` の語は使わない（`class`/`data-*` は `menu-toggle`/
//! `mobile` を使う。[`super::super::header::header_mega_menu`] と同型の
//! 判断）。
//!
//! # `.blocks-demo` のはみ出し対策
//!
//! `crate::blocks::stylesheet` の `.blocks-demo` は `overflow-x: auto` を
//! 持つ（`blocks_stylesheet_declares_demo_frame_overflow` 契約）。幅広時は
//! 絶対配置パネルがフローに寄与しないため、`.blocks-store-nav-centered-logo-page`
//! （ダミー本文枠）へ `min-block-size: 30rem` を持たせ、展開済みパネルが
//! レイアウトボックスの内側に収まるようにする。狭幅
//! （`@media (max-width: 47.99rem)`）では画像 1 列 + リンク列の実高さが
//! 固定枠を超えてはみ出し得るレビュー指摘（PR #3472）を踏まえ、`content`
//! パートを `position: static` へ上書きしてパネルをフロー内に戻す
//! （固定 `min-block-size` の当て推量に頼らず、実内容の高さぶんだけ
//! `.blocks-store-nav-centered-logo-page` の手前へ自然に積み増す構造）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `navigation_menu` のパート関数は呼び出し側 `attrs` の `class` を除去
//! しないため `.blocks-store-nav-centered-logo-*` の class フックがそのまま
//! 使える。一方 `button::button`/`button::icon_button`/`link::root`/
//! `icon::icon`/`image::image` は `drop_class_attr` で `class` を常に除去
//! する契約のため、これらへのフックは `data-blocks-store-nav-centered-logo-*`
//! 属性で渡す（`header_mega_menu` と同型の判断）。
//! [`fandhe_frontend_pre_styled_ui::drawer::root`] も `size` variant クラス
//! 付与のため呼び出し側 `attrs` の `class` を `drop_class_attr` で常に除去
//! する契約（`crates/pre-styled-ui/src/drawer.rs` の `root` rustdoc参照）
//! のため、`.blocks-store-nav-centered-logo-mobile` の class フックは
//! `drawer::root` の外側に置く `div` ラッパー（[`mobile_preview`]）へ付け、
//! `drawer::root` 自身へは渡さない。`drawer::trigger`/`backdrop`/
//! `positioner`/`content`/`title`/`close_trigger`（headless 直の再
//! エクスポート）は `class` を除去しないため、`content`/`positioner` の
//! CSS 上書きセレクタは `[data-scope="drawer"][data-part="..."]` 属性
//! セレクタで書く（[`super::super::contact::contact_dialog_form`] の
//! dialog 版と同型の判断）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）は本 block では使わないため衝突しない
//! （`fandhe_frontend_core::text` のみを import する）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンは既定の `type="button"`（`navigation_menu::trigger`
//! も `type="button"` 固定）のまま送信先を持たない。通貨・検索・
//! アカウント・カートは送信先・処理を持たない no-op のため
//! `disabled: true` にする。

use crate::blocks::{dummy_assets, Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::drawer::{self, ContentIds, DrawerPlacement};
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::{Orientation, Size};

/// パネルの注目画像 1 件（見出し, 補足, href, alt）。href はサイト内に
/// 実在する索引ページへの相対パス（モジュール冒頭 rustdoc「href の方針」
/// 節）。画像は [`dummy_assets::PRODUCT_SRC`]（モノトーン抽象図形の
/// SVG）を使い回す。
type FeaturedItem = (&'static str, &'static str, &'static str);

/// パネルの列 1 件（列見出し, 項目〔表示名, href〕4 件）。
type PanelColumn = (&'static str, [(&'static str, &'static str); 4]);

/// 「レディース」パネルの注目画像 3 件。
const FEATURED_ITEMS: [FeaturedItem; 3] = [
    (
        "新作アウター",
        "今季の新作をまとめてチェック",
        "../../themes/",
    ),
    (
        "定番トップス",
        "毎日使えるベーシックアイテム",
        "../../guides/",
    ),
    (
        "アクセサリー",
        "コーディネートのアクセントに",
        "../../primitives/",
    ),
];

/// 「レディース」パネルの階層化された項目列 2 列。
const PANEL_COLUMNS: [PanelColumn; 2] = [
    (
        "カテゴリ",
        [
            ("トップス", "../../themes/"),
            ("ボトムス", "../../primitives/"),
            ("シューズ", "../../api/"),
            ("バッグ", "../../examples/"),
        ],
    ),
    (
        "ブランド",
        [
            ("アトリエノア", "../../guides/"),
            ("ルミエールベーシック", "../../themes/"),
            ("ノームアンドコー", "../../primitives/"),
            ("ソレイユスタジオ", "../../examples/"),
        ],
    ),
];

/// 唯一開いた状態で固定するトリガーの `id`（モジュール冒頭 rustdoc「id
/// 接頭辞」節。項目が 1 件のみのため `format!` による添字展開は行わない）。
const WOMEN_TRIGGER_ID: &str = "blocks-store-nav-centered-logo-women-trigger";
/// [`WOMEN_TRIGGER_ID`] と対になる `content` の `id`。
const WOMEN_CONTENT_ID: &str = "blocks-store-nav-centered-logo-women-content";

/// 狭幅インスタンス（[`mobile_preview`]）側のレディーストリガー `id`
/// （モジュール冒頭 rustdoc「狭幅インスタンスの並記」節）。
const MOBILE_WOMEN_TRIGGER_ID: &str = "blocks-store-nav-centered-logo-mobile-women-trigger";
/// [`MOBILE_WOMEN_TRIGGER_ID`] と対になる `content` の `id`。
const MOBILE_WOMEN_CONTENT_ID: &str = "blocks-store-nav-centered-logo-mobile-women-content";
/// 狭幅インスタンスの展開済み drawer 本体（`content` パート）の `id`
/// （メニュートグルボタンの `aria-controls` が参照する）。
const MOBILE_DRAWER_CONTENT_ID: &str = "blocks-store-nav-centered-logo-mobile-drawer";
/// [`MOBILE_DRAWER_CONTENT_ID`] の `aria-labelledby` が参照する `title`
/// の `id`。
const MOBILE_DRAWER_TITLE_ID: &str = "blocks-store-nav-centered-logo-mobile-drawer-title";

/// パネル項目・中央ロゴ・アクションで共有する線画アイコン（装飾用途、
/// `icon::icon` の `currentColor` 継承に任せ生の色リテラルは持ち込まない）。
fn line_icon(path_d: &str) -> Node {
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

/// 中央ロゴ領域（アイコン + 架空のブランド名、実在サイトの索引ページへの
/// リンク）。実在ブランドのロゴ・商標を模さない独自の単純図形
/// （`docs/design/wireframe-ui-architecture.md` と同じ判断軸）。
fn brand() -> Node {
    link::root(
        "../../",
        &LinkProps::default(),
        vec![("data-blocks-store-nav-centered-logo-brand", "")],
        vec![
            line_icon("M12 3a9 9 0 1 0 0 18a9 9 0 0 0 0-18zM8 12h8M12 8v8"),
            span(vec![], vec![text("Solaris Mart")]),
        ],
    )
}

/// 注目画像 1 件（画像 + 見出し + 補足）。
fn featured_link((title, note, href): &FeaturedItem) -> Node {
    navigation_menu::link(
        href,
        false,
        vec![("class", "blocks-store-nav-centered-logo-featured")],
        vec![
            image::image(
                &ImageProps::new(dummy_assets::PRODUCT_SRC, title),
                vec![("data-blocks-store-nav-centered-logo-featured-image", "")],
            ),
            span(
                vec![("class", "blocks-store-nav-centered-logo-featured-title")],
                vec![text(*title)],
            ),
            span(
                vec![("class", "blocks-store-nav-centered-logo-featured-note")],
                vec![text(*note)],
            ),
        ],
    )
}

/// パネル 1 列分（列見出し + リンク項目 4 件）。
fn panel_column((heading, items): &PanelColumn) -> Node {
    let links: Vec<Node> = items
        .iter()
        .map(|(label, href)| {
            navigation_menu::link(
                href,
                false,
                vec![("class", "blocks-store-nav-centered-logo-panel-link")],
                vec![text(*label)],
            )
        })
        .collect();
    let mut children = vec![span(
        vec![("class", "blocks-store-nav-centered-logo-panel-heading")],
        vec![text(*heading)],
    )];
    children.extend(links);
    div(
        vec![("class", "blocks-store-nav-centered-logo-panel-column")],
        children,
    )
}

/// 「レディース」トップ項目（唯一のドロップダウン、常時 open 固定）。
/// `trigger_id`/`content_id` は呼び出し側（幅広 [`category_nav`] 呼び出し・
/// 狭幅 [`category_nav`] 呼び出し）ごとに異なる固定文字列を渡す
/// （モジュール冒頭 rustdoc「id 接頭辞」節）。
fn women_item(props: &NavigationMenuProps, trigger_id: &str, content_id: &str) -> Node {
    let state = OpenState::Open;
    let featured: Vec<Node> = FEATURED_ITEMS.iter().map(featured_link).collect();
    let columns: Vec<Node> = PANEL_COLUMNS.iter().map(panel_column).collect();

    navigation_menu::item(
        state,
        false,
        props,
        "women",
        vec![],
        vec![
            navigation_menu::trigger(
                state,
                true,
                "women",
                Some(trigger_id),
                Some(content_id),
                vec![],
                vec![
                    text("レディース"),
                    navigation_menu::item_indicator(state, props, "women", vec![], vec![text("▾")]),
                ],
            ),
            navigation_menu::content(
                state,
                props,
                "women",
                Some(content_id),
                Some(trigger_id),
                vec![],
                vec![div(
                    vec![("class", "blocks-store-nav-centered-logo-panel-inner")],
                    vec![
                        div(
                            vec![("class", "blocks-store-nav-centered-logo-featured-row")],
                            featured,
                        ),
                        div(
                            vec![("class", "blocks-store-nav-centered-logo-panel-columns")],
                            columns,
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// トリガーを持たない、リンクのみのトップ項目（メンズ・ホーム用品）。
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

/// カテゴリナビ（「レディース」のメガメニュー + リンク項目 2 件）。幅広
/// バー（[`bar`]）・狭幅インスタンス（[`mobile_drawer`]）の双方から
/// `orientation`・`class`・`aria-label`・`id` 接頭辞のみを差し替えて
/// 呼ばれる共通部品（モジュール冒頭 rustdoc「狭幅インスタンスの並記」
/// 節）。
fn category_nav(
    props: &NavigationMenuProps,
    class: &str,
    aria_label: &str,
    trigger_id: &str,
    content_id: &str,
) -> Node {
    navigation_menu::root(
        props,
        aria_label,
        vec![("class", class)],
        vec![navigation_menu::list(
            props,
            vec![],
            vec![
                women_item(props, trigger_id, content_id),
                link_item(props, "men", "メンズ", "../../themes/"),
                link_item(props, "home", "ホーム用品", "../../guides/"),
            ],
        )],
    )
}

/// 右側のアクション（通貨・検索・アカウント・カート）。いずれも遷移先・
/// 送信処理を持たない no-op のため `disabled: true` にする（モジュール
/// 冒頭 rustdoc「`<form>` を持たない」節）。
fn actions() -> Node {
    div(
        vec![(
            "class",
            "blocks-store-nav-centered-logo-actions",
        )],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![(
                    "data-blocks-store-nav-centered-logo-currency",
                    "",
                )],
                vec![text("JPY")],
            ),
            button::icon_button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    disabled: true,
                    ..ButtonProps::default()
                },
                "検索",
                vec![(
                    "data-blocks-store-nav-centered-logo-search",
                    "",
                )],
                vec![line_icon(
                    "M10 4a6 6 0 1 0 0 12a6 6 0 0 0 0-12zM20 20l-4.35-4.35",
                )],
            ),
            button::icon_button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    disabled: true,
                    ..ButtonProps::default()
                },
                "アカウント",
                vec![(
                    "data-blocks-store-nav-centered-logo-account",
                    "",
                )],
                vec![line_icon("M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM4 20c0-4 4-6 8-6s8 2 8 6")],
            ),
            div(
                vec![(
                    "class",
                    "blocks-store-nav-centered-logo-cart",
                )],
                vec![
                    button::icon_button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            disabled: true,
                            ..ButtonProps::default()
                        },
                        "カート",
                        vec![(
                            "data-blocks-store-nav-centered-logo-cart-button",
                            "",
                        )],
                        vec![line_icon("M3 4h2l2 10h10l2-7H6M9 20a1 1 0 1 0 0-2a1 1 0 0 0 0 2ZM17 20a1 1 0 1 0 0-2a1 1 0 0 0 0 2Z")],
                    ),
                    span(
                        vec![(
                            "class",
                            "blocks-store-nav-centered-logo-cart-count",
                        )],
                        vec![text("2")],
                    ),
                ],
            ),
        ],
    )
}

/// 1 行バー（左: カテゴリナビ / 中央: ロゴ / 右: アクション）。
fn bar() -> Node {
    div(
        vec![("class", "blocks-store-nav-centered-logo-bar")],
        vec![
            category_nav(
                &NavigationMenuProps::default(),
                "blocks-store-nav-centered-logo-nav",
                "カテゴリ",
                WOMEN_TRIGGER_ID,
                WOMEN_CONTENT_ID,
            ),
            brand(),
            actions(),
        ],
    )
}

/// ダミーのページ本文（`.blocks-demo` のはみ出し対策、モジュール冒頭
/// rustdoc参照）。
fn page_placeholder() -> Node {
    div(
        vec![("class", "blocks-store-nav-centered-logo-page")],
        vec![text(
            "ページ本文（ダミー）。展開済みパネルの下に十分な高さを確保するための枠。",
        )],
    )
}

/// 幅広インスタンス（1 行バー + ダミー本文）。[`demo`] が狭幅インスタンス
/// （[`mobile_preview`]）と並べて描画する。
fn layout() -> Node {
    div(
        vec![("class", "blocks-store-nav-centered-logo-layout")],
        vec![
            div(
                vec![("class", "blocks-store-nav-centered-logo-bar-wrap")],
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
        vec![("class", "blocks-store-nav-centered-logo-state-label")],
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
        vec![("class", "blocks-store-nav-centered-logo-mobile-bar")],
        vec![
            brand(),
            drawer::trigger(
                OpenState::Open,
                Some(MOBILE_DRAWER_CONTENT_ID),
                vec![
                    ("aria-label", "メニュー"),
                    ("disabled", ""),
                    ("data-disabled", ""),
                    ("data-blocks-store-nav-centered-logo-menu-toggle", ""),
                ],
                vec![line_icon("M3 6h18M3 12h18M3 18h18")],
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
                vec![("class", "blocks-store-nav-centered-logo-mobile-panel-wrap")],
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
                                    vec![line_icon("M6 6l12 12M18 6L6 18")],
                                ),
                                drawer::title(
                                    Some(MOBILE_DRAWER_TITLE_ID),
                                    vec![],
                                    vec![text("メニュー")],
                                ),
                                category_nav(
                                    &NavigationMenuProps {
                                        orientation: Orientation::Vertical,
                                    },
                                    "blocks-store-nav-centered-logo-mobile-nav",
                                    "カテゴリ（狭幅）",
                                    MOBILE_WOMEN_TRIGGER_ID,
                                    MOBILE_WOMEN_CONTENT_ID,
                                ),
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
        vec![("class", "blocks-store-nav-centered-logo-mobile")],
        vec![mobile_drawer()],
    )
}

/// `store-nav-centered-logo` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「静的表示」節）。幅広インスタンス
/// （[`layout`]）と狭幅（メニュー展開時）インスタンス（[`mobile_preview`]）
/// を見出し付きで並記する（本イシューで追加、モジュール冒頭 rustdoc
/// 「狭幅インスタンスの並記」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-store-nav-centered-logo-states")],
        vec![
            state_label("幅広（レディースを展開）"),
            layout(),
            state_label("狭幅（メニュー展開時）"),
            mobile_preview(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/store-nav-centered-logo/",
    title: "store-nav-centered-logo",
    category: BlockCategory::StoreNav,
    rust_source: "crates/docs-site/src/blocks/ecommerce/store_nav/store_nav_centered_logo.rs",
    demo_class: "blocks-store-nav-centered-logo",
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
            label: "Drawer",
            path: "/themes/drawer/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `store_nav_centered_logo` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-store-nav-centered-logo-*`、`[data-blocks-store-nav-centered-logo-*]`、
/// および `.blocks-store-nav-centered-logo-layout` を祖先に持つ
/// `[data-scope="navigation-menu"]` 系セレクタへの子孫結合子付き上書き
/// （全幅パネル化、モジュール冒頭 rustdoc「全幅パネルの配置方法」節）
/// のみを用い、他 block や部品の素のセレクタへ影響させない。値はすべて
/// `var(--fandhe-*)` トークンで書き、生の色リテラルは使わない。
const LAYOUT_CSS: &str = "\
.blocks-store-nav-centered-logo-states {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-store-nav-centered-logo-state-label {\n  display: block;\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: 600;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-store-nav-centered-logo-bar-wrap {\n  position: relative;\n  background: var(--fandhe-color-bg);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-store-nav-centered-logo-bar {\n  display: grid;\n  grid-template-columns: 1fr auto 1fr;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n}\n\
.blocks-store-nav-centered-logo-nav[data-scope=\"navigation-menu\"][data-part=\"root\"] {\n  justify-self: start;\n}\n\
[data-blocks-store-nav-centered-logo-brand] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  justify-self: center;\n  font-weight: 600;\n  white-space: nowrap;\n  color: var(--fandhe-color-fg);\n}\n\
.blocks-store-nav-centered-logo-actions {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  justify-self: end;\n  white-space: nowrap;\n}\n\
.blocks-store-nav-centered-logo-cart {\n  position: relative;\n  display: inline-flex;\n}\n\
.blocks-store-nav-centered-logo-cart-count {\n  position: absolute;\n  top: -0.25rem;\n  right: -0.25rem;\n  min-inline-size: 1rem;\n  padding: 0 0.25rem;\n  border-radius: var(--fandhe-radius-full);\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);\n  font-size: var(--fandhe-font-font-size-xs);\n  line-height: 1rem;\n  text-align: center;\n}\n\
.blocks-store-nav-centered-logo-layout [data-scope=\"navigation-menu\"][data-part=\"root\"],\n\
.blocks-store-nav-centered-logo-layout [data-scope=\"navigation-menu\"][data-part=\"item\"] {\n  position: static;\n}\n\
.blocks-store-nav-centered-logo-layout [data-scope=\"navigation-menu\"][data-part=\"content\"] {\n  inset-inline: 0;\n  min-width: 0;\n}\n\
.blocks-store-nav-centered-logo-panel-inner {\n  max-inline-size: 64rem;\n  margin-inline: auto;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-4);\n}\n\
.blocks-store-nav-centered-logo-featured-row {\n  display: grid;\n  grid-template-columns: repeat(auto-fit, minmax(min(12rem, 100%), 1fr));\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-store-nav-centered-logo-featured[data-scope=\"navigation-menu\"][data-part=\"link\"] {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-1);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-store-nav-centered-logo-featured-image] {\n  display: block;\n  width: 100%;\n  height: 8rem;\n  object-fit: cover;\n  border-radius: var(--fandhe-radius-md);\n}\n\
.blocks-store-nav-centered-logo-featured-title {\n  font-weight: 600;\n}\n\
.blocks-store-nav-centered-logo-featured-note {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-store-nav-centered-logo-panel-columns {\n  display: grid;\n  grid-template-columns: repeat(auto-fit, minmax(min(10rem, 100%), 1fr));\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-store-nav-centered-logo-panel-column {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-store-nav-centered-logo-panel-heading {\n  display: block;\n  font-weight: 600;\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n  margin-block-end: var(--fandhe-space-2);\n}\n\
.blocks-store-nav-centered-logo-layout [data-scope=\"navigation-menu\"][data-part=\"trigger\"][data-disabled],\n\
.blocks-store-nav-centered-logo-mobile [data-scope=\"navigation-menu\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-store-nav-centered-logo-page {\n  min-block-size: 30rem;\n  padding: var(--fandhe-space-6) var(--fandhe-space-4);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-store-nav-centered-logo-mobile {\n  max-inline-size: 24rem;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  background: var(--fandhe-color-bg);\n  overflow: hidden;\n}\n\
.blocks-store-nav-centered-logo-mobile-bar {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-store-nav-centered-logo-mobile-panel-wrap {\n  position: relative;\n}\n\
.blocks-store-nav-centered-logo-mobile [data-scope=\"drawer\"][data-part=\"backdrop\"] {\n  position: absolute;\n  inset: 0;\n  z-index: auto;\n}\n\
.blocks-store-nav-centered-logo-mobile [data-scope=\"drawer\"][data-part=\"positioner\"] {\n  position: relative;\n  inset: auto;\n  z-index: auto;\n  padding: var(--fandhe-space-3);\n}\n\
.blocks-store-nav-centered-logo-mobile [data-scope=\"drawer\"][data-part=\"content\"] {\n  width: min(100%, var(--fandhe-drawer-size, 20rem));\n  height: auto;\n}\n\
.blocks-store-nav-centered-logo-mobile [data-scope=\"drawer\"] h2 {\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n  font-size: var(--fandhe-font-font-size-md);\n}\n\
.blocks-store-nav-centered-logo-mobile [data-scope=\"navigation-menu\"][data-part=\"item\"] {\n  position: static;\n  inline-size: 100%;\n}\n\
.blocks-store-nav-centered-logo-mobile [data-scope=\"navigation-menu\"][data-part=\"list\"] {\n  display: flex;\n  flex-direction: column;\n  align-items: stretch;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-store-nav-centered-logo-mobile [data-scope=\"navigation-menu\"][data-part=\"content\"] {\n  position: static;\n  inset-inline: auto;\n  padding: 0;\n  margin-top: var(--fandhe-space-2);\n  border: none;\n  box-shadow: none;\n}\n\
.blocks-store-nav-centered-logo-mobile .blocks-store-nav-centered-logo-actions {\n  flex-wrap: wrap;\n  justify-content: flex-start;\n}\n\
@media (max-width: 47.99rem) {\n  .blocks-store-nav-centered-logo-bar {\n    grid-template-columns: 1fr;\n    justify-items: center;\n  }\n  .blocks-store-nav-centered-logo-nav[data-scope=\"navigation-menu\"][data-part=\"root\"] {\n    justify-self: center;\n    inline-size: 100%;\n  }\n  .blocks-store-nav-centered-logo-layout [data-scope=\"navigation-menu\"][data-part=\"list\"] {\n    flex-wrap: wrap;\n    justify-content: center;\n  }\n  .blocks-store-nav-centered-logo-layout [data-scope=\"navigation-menu\"][data-part=\"item\"] {\n    inline-size: 100%;\n  }\n  .blocks-store-nav-centered-logo-layout [data-scope=\"navigation-menu\"][data-part=\"content\"] {\n    position: static;\n    inset-inline: auto;\n  }\n  .blocks-store-nav-centered-logo-actions {\n    justify-self: center;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{
        demo, LAYOUT_CSS, MOBILE_DRAWER_CONTENT_ID, MOBILE_DRAWER_TITLE_ID,
        MOBILE_WOMEN_CONTENT_ID, MOBILE_WOMEN_TRIGGER_ID, WOMEN_CONTENT_ID, WOMEN_TRIGGER_ID,
    };
    use fandhe_frontend_core::render;

    /// Demo が期待する 6 種の部品・非対話制約を満たすことの単体回帰
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
            "data-scope=\"drawer\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 唯一のドロップダウン（レディース）が幅広・狭幅の各インスタンスで
    /// 常時 open で固定され、`id`/`aria-controls`/`aria-labelledby` が
    /// 固定文字列の組で一致すること。
    #[test]
    fn open_trigger_and_content_are_wired() {
        let html = render(&demo());
        for (trigger_id, content_id) in [
            (WOMEN_TRIGGER_ID, WOMEN_CONTENT_ID),
            (MOBILE_WOMEN_TRIGGER_ID, MOBILE_WOMEN_CONTENT_ID),
        ] {
            assert!(html.contains(&format!("id=\"{trigger_id}\"")));
            assert!(html.contains(&format!("aria-controls=\"{content_id}\"")));
            assert!(html.contains(&format!("id=\"{content_id}\"")));
            assert!(html.contains(&format!("aria-labelledby=\"{trigger_id}\"")));
            let trigger_start = html
                .find(&format!("id=\"{trigger_id}\""))
                .expect("trigger id should be present");
            let tag_start = html[..trigger_start].rfind("<button").unwrap();
            let tag_end = html[tag_start..].find('>').unwrap() + tag_start;
            let tag = &html[tag_start..tag_end];
            assert!(tag.contains("disabled=\"\""), "tag={tag}");
            assert!(tag.contains(r#"data-disabled="""#), "tag={tag}");
            assert!(tag.contains(r#"aria-expanded="true""#), "tag={tag}");
        }
        assert!(html.contains(r#"data-part="item" data-state="open""#));
    }

    /// 狭幅インスタンスのメニュートグルボタンが展開済み drawer を指し、
    /// `disabled` で操作不能であることを固定する（モジュール冒頭
    /// rustdoc「狭幅インスタンスの並記」節）。
    #[test]
    fn narrow_preview_menu_trigger_is_expanded_and_disabled() {
        let html = render(&demo());
        let toggle_pos = html
            .find("data-blocks-store-nav-centered-logo-menu-toggle")
            .expect("menu toggle attr should be present");
        let tag_start = html[..toggle_pos].rfind("<button").unwrap();
        let tag_end = html[tag_start..].find('>').unwrap() + tag_start;
        let tag = &html[tag_start..tag_end];
        assert!(tag.contains(r#"aria-haspopup="dialog""#), "tag={tag}");
        assert!(tag.contains(r#"aria-expanded="true""#), "tag={tag}");
        assert!(
            tag.contains(&format!(r#"aria-controls="{MOBILE_DRAWER_CONTENT_ID}""#)),
            "tag={tag}"
        );
        assert!(tag.contains("disabled=\"\""), "tag={tag}");
        assert!(html.contains(&format!("id=\"{MOBILE_DRAWER_CONTENT_ID}\"")));
    }

    /// 狭幅インスタンスの展開済み drawer が `hidden` を持たず常時開いた
    /// 状態で、静的デモの表示実態に合わせ非モーダル（`aria-modal=false`）
    /// であることを固定する。
    #[test]
    fn narrow_preview_drawer_is_open_static_and_non_modal() {
        let html = render(&demo());
        for part in ["positioner", "content"] {
            let marker = format!(r#"data-scope="drawer" data-part="{part}""#);
            let pos = html
                .find(&marker)
                .unwrap_or_else(|| panic!("{marker} should be present"));
            let tag_start = html[..pos].rfind('<').unwrap();
            let tag_end = html[tag_start..].find('>').unwrap() + tag_start;
            let tag = &html[tag_start..tag_end];
            assert!(!tag.contains("hidden"), "tag={tag}");
            assert!(tag.contains(r#"data-state="open""#), "tag={tag}");
        }
        let content_pos = html
            .find(&format!("id=\"{MOBILE_DRAWER_CONTENT_ID}\""))
            .expect("drawer content id should be present");
        let tag_start = html[..content_pos].rfind('<').unwrap();
        let tag_end = html[tag_start..].find('>').unwrap() + tag_start;
        let content_tag = &html[tag_start..tag_end];
        assert!(
            content_tag.contains(r#"aria-modal="false""#),
            "tag={content_tag}"
        );
        assert!(
            content_tag.contains(&format!("aria-labelledby=\"{MOBILE_DRAWER_TITLE_ID}\"")),
            "tag={content_tag}"
        );
    }

    /// [`LAYOUT_CSS`] がトークン参照のみで構成され、生の色リテラル・
    /// `hamburger` の語を持たず、レスポンシブ上書き・`display: none` の
    /// 不在（狭幅インスタンスを常時表示する）を持つこと。
    #[test]
    fn layout_css_uses_tokens_and_no_hamburger() {
        assert!(!LAYOUT_CSS.contains("hamburger"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("rgb("));
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
        assert!(LAYOUT_CSS.contains("position: static;"));
        assert!(!LAYOUT_CSS.contains("display: none"));
    }

    /// 注目画像 6 件（幅広・狭幅の各インスタンス 3 件ずつ）がいずれも
    /// [`dummy_assets::PRODUCT_SRC`] を使うこと（実在ブランド画像を
    /// 持ち込まない、モジュール冒頭 rustdoc参照）。
    #[test]
    fn featured_images_use_dummy_asset_src() {
        let html = render(&demo());
        assert_eq!(
            html.matches(super::dummy_assets::PRODUCT_SRC).count(),
            6,
            "html={html}"
        );
    }
}
