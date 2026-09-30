//! `store-nav-centered-logo` block（親トラッキング #3093、対応表 ID R1316
//! を主参照とする合成例。本イシュー #3094（前半）で骨格・主要領域・
//! Blocks 登録一式を仕上げ、後半 #3095 で狭幅（メニュー展開時）状態の
//! 並記を仕上げる）。上部帯を持たない 1 行バーの中央にロゴ、左に
//! カテゴリナビ、右に通貨・検索・アカウント・カートを配置する
//! ストアナビゲーション。Ecommerce / Store Nav カテゴリの最初の block
//! （`super`〔`store_nav/mod.rs`〕参照）。
//!
//! # 使用部品
//!
//! `navigation-menu` / `link` / `button` / `icon` / `image` の 5 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 前半の範囲では `drawer` は使わない（狭幅の並記は後半 #3095 で追加）。
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
//! `id`/`aria-controls`/`aria-labelledby` は開いたドロップダウン
//! （「レディース」1 件）にのみ必要であり、
//! `blocks-store-nav-centered-logo-women-{trigger|content}` の固定文字列で
//! 一意にする（項目が 1 件のみのため `format!` は使わない。`crate::blocks`
//! モジュール doc「HTML 文字列の直接組み立て禁止」節参照）。
//!
//! # href の方針
//!
//! `href="#"` は使わない（横断テストが禁止する）。パネル項目・中央ロゴは
//! サイト内に実在する索引ページへの相対パス（`../../`・`../../guides/`・
//! `../../themes/`・`../../primitives/`・`../../api/`・`../../examples/`）を
//! 使う（[`super::super::header::header_mega_menu`] と同型の判断）。
//!
//! # レスポンシブ（前半の範囲、`@media (max-width: 47.99rem)`）
//!
//! 無 JS のため開閉するメニューボタンは作れない
//! （[`super::super::header::header_mega_menu`] モジュール doc「狭幅
//! インスタンスの並記」節と同じ制約）。前半では狭幅（メニュー展開時）の
//! 並記は行わず、既存インスタンスをこの幅で `flex-wrap` させてナビ・
//! アクションを非表示にせず折り返す（レビュー是正の教訓の先取り:
//! ハンバーガーで畳んで内容へ到達不能にする構成は採らない）。狭幅
//! （メニュー展開時）状態の並記は後半 #3095 で追加する。
//!
//! # `.blocks-demo` のはみ出し対策
//!
//! `crate::blocks::stylesheet` の `.blocks-demo` は `overflow-x: auto` を
//! 持つ（`blocks_stylesheet_declares_demo_frame_overflow` 契約）。絶対配置
//! パネルがフローに寄与しないため、`.blocks-store-nav-centered-logo-page`
//! （ダミー本文枠）へ `min-block-size` を持たせ、展開済みパネルが
//! レイアウトボックスの内側に収まるようにする。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `navigation_menu` のパート関数は呼び出し側 `attrs` の `class` を除去
//! しないため `.blocks-store-nav-centered-logo-*` の class フックがそのまま
//! 使える。一方 `button::button`/`button::icon_button`/`link::root`/
//! `icon::icon`/`image::image` は `drop_class_attr` で `class` を常に除去
//! する契約のため、これらへのフックは `data-blocks-store-nav-centered-logo-*`
//! 属性で渡す（`header_mega_menu` と同型の判断）。
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
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::Size;

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
fn women_item(props: &NavigationMenuProps) -> Node {
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
                Some(WOMEN_TRIGGER_ID),
                Some(WOMEN_CONTENT_ID),
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
                Some(WOMEN_CONTENT_ID),
                Some(WOMEN_TRIGGER_ID),
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

/// 左側のカテゴリナビ（「レディース」のメガメニュー + リンク項目 2 件）。
fn category_nav() -> Node {
    let props = NavigationMenuProps::default();
    navigation_menu::root(
        &props,
        "カテゴリ",
        vec![("class", "blocks-store-nav-centered-logo-nav")],
        vec![navigation_menu::list(
            &props,
            vec![],
            vec![
                women_item(&props),
                link_item(&props, "men", "メンズ", "../../themes/"),
                link_item(&props, "home", "ホーム用品", "../../guides/"),
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
        vec![category_nav(), brand(), actions()],
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

/// `store-nav-centered-logo` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「静的表示」節）。前半 #3094 の範囲では
/// 幅広インスタンス 1 種のみを描画する（狭幅の並記は後半 #3095 で追加）。
pub fn demo() -> Node {
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
.blocks-store-nav-centered-logo-featured-title {\n  font-weight: 600;\n}\n\
.blocks-store-nav-centered-logo-featured-note {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-store-nav-centered-logo-panel-columns {\n  display: grid;\n  grid-template-columns: repeat(auto-fit, minmax(min(10rem, 100%), 1fr));\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-store-nav-centered-logo-panel-column {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-store-nav-centered-logo-panel-heading {\n  display: block;\n  font-weight: 600;\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n  margin-block-end: var(--fandhe-space-2);\n}\n\
.blocks-store-nav-centered-logo-layout [data-scope=\"navigation-menu\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-store-nav-centered-logo-page {\n  min-block-size: 30rem;\n  padding: var(--fandhe-space-6) var(--fandhe-space-4);\n  color: var(--fandhe-color-fg-muted);\n}\n\
@media (max-width: 47.99rem) {\n  .blocks-store-nav-centered-logo-bar {\n    grid-template-columns: 1fr;\n    justify-items: center;\n  }\n  .blocks-store-nav-centered-logo-nav[data-scope=\"navigation-menu\"][data-part=\"root\"] {\n    justify-self: center;\n  }\n  .blocks-store-nav-centered-logo-layout [data-scope=\"navigation-menu\"][data-part=\"list\"] {\n    flex-wrap: wrap;\n    justify-content: center;\n  }\n  .blocks-store-nav-centered-logo-actions {\n    justify-self: center;\n  }\n  .blocks-store-nav-centered-logo-page {\n    min-block-size: 46rem;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, WOMEN_CONTENT_ID, WOMEN_TRIGGER_ID};
    use fandhe_frontend_core::render;

    /// Demo が期待する 5 種の部品・非対話制約を満たすことの単体回帰
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
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 唯一のドロップダウン（レディース）が常時 open で固定され、`id`/
    /// `aria-controls`/`aria-labelledby` が固定文字列の組で一致すること。
    #[test]
    fn open_trigger_and_content_are_wired() {
        let html = render(&demo());
        assert!(html.contains(r#"data-part="item" data-state="open""#));
        assert!(html.contains(&format!("id=\"{WOMEN_TRIGGER_ID}\"")));
        assert!(html.contains(&format!("aria-controls=\"{WOMEN_CONTENT_ID}\"")));
        assert!(html.contains(&format!("id=\"{WOMEN_CONTENT_ID}\"")));
        assert!(html.contains(&format!("aria-labelledby=\"{WOMEN_TRIGGER_ID}\"")));
        let trigger_start = html
            .find(&format!("id=\"{WOMEN_TRIGGER_ID}\""))
            .expect("trigger id should be present");
        let tag_start = html[..trigger_start].rfind("<button").unwrap();
        let tag_end = html[tag_start..].find('>').unwrap() + tag_start;
        let tag = &html[tag_start..tag_end];
        assert!(tag.contains("disabled=\"\""), "tag={tag}");
        assert!(tag.contains(r#"data-disabled="""#), "tag={tag}");
        assert!(tag.contains(r#"aria-expanded="true""#), "tag={tag}");
    }

    /// [`LAYOUT_CSS`] がトークン参照のみで構成され、生の色リテラル・
    /// `hamburger` の語を持たず、レスポンシブ上書きを持つこと。
    #[test]
    fn layout_css_uses_tokens_and_no_hamburger() {
        assert!(!LAYOUT_CSS.contains("hamburger"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("rgb("));
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
        assert!(LAYOUT_CSS.contains("position: static;"));
    }

    /// 注目画像 3 件がいずれも [`dummy_assets::PRODUCT_SRC`] を使うこと
    /// （実在ブランド画像を持ち込まない、モジュール冒頭 rustdoc参照）。
    #[test]
    fn featured_images_use_dummy_asset_src() {
        let html = render(&demo());
        assert_eq!(
            html.matches(super::dummy_assets::PRODUCT_SRC).count(),
            3,
            "html={html}"
        );
    }
}
