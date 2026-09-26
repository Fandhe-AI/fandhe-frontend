//! `header-mega-menu` block（イシュー #2858。親トラッキング #2857 の前半、
//! 対応表 ID R0984 を主参照とする合成例）。幅を制限したバー（ブランド /
//! ナビ / アクション）の下に、ヘッダー全幅まで広がるメガメニューパネルを
//! 持つヘッダー。**Marketing / Header カテゴリで最初の block**（イシュー
//! #2734 の雛形を本 block 追加で卒業させた、`super`（`header/mod.rs`）
//! 参照）。
//!
//! パネル下部の補助 CTA 帯・狭幅で開いた状態の並記・原案差分メモは
//! 兄弟イシュー #2859（後半）の担当であり、本 block の範囲外
//! （`.claude/rules/out-of-scope-tracking.md`）。
//!
//! # 使用部品
//!
//! `navigation-menu` / `button` / `icon` / `link` の 4 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 静的表示（無 JS、唯一のドロップダウンを常時 open で固定）
//!
//! ドロップダウンを持つトップ項目は「プロダクト」1 件のみとし、
//! [`OpenState::Open`] で固定する（`disabled` は付与しない。無 JS の
//! docs サイトで `disabled: true` にすると本文が到達不能になる accordion
//! 系 block の教訓とは逆に、本 block は最初から `disabled: false` のまま
//! open 固定にする設計であり、閉じた状態を経由しないため到達性の問題は
//! 生じない）。残りのトップ項目（料金・ドキュメント）は
//! [`navigation_menu::trigger`] を持たず [`navigation_menu::item`] +
//! [`navigation_menu::link`] のリンク項目のみで構成する（閉じたまま
//! フォーカス可能だが操作しても何も起きない trigger を作らないため。
//! accordion 系 block が受けた「閉じた項目の本文へ到達できない」指摘を
//! 構造的に避ける）。
//!
//! # 全幅パネルの配置方法（`position: static` 上書きと包含ブロック）
//!
//! [`fandhe_frontend_pre_styled_ui::navigation_menu`] の recipe は
//! `root`/`item` に `position: relative` を、`content` に `position:
//! absolute; top: 100%; left: 0; min-width: 10rem;` を宣言する
//! （`crates/pre-styled-ui/src/navigation_menu.rs` 参照）。このままでは
//! `content` の包含ブロックが `item`（トリガー 1 個ぶんの幅）になり、
//! パネルがヘッダー全幅まで広がらない。[`LAYOUT_CSS`] は本 block の
//! スコープ内（`.blocks-header-mega-menu-layout` 子孫セレクタ、詳細度
//! (0,3,0)）に限定して `root`/`item` の `position` を `static` へ上書きし、
//! `.blocks-header-mega-menu-bar-wrap`（`position: relative`）を
//! `content` の包含ブロックへ格上げする（`.blocks-header-mega-menu-layout`
//! 自身ではなく `bar-wrap` を選ぶ理由: `layout` はバーの下にダミー本文
//! （[`page_placeholder`]）まで含むため、`layout` を包含ブロックにすると
//! `top: 100%` がダミー本文ぶんの高さを含めて計算され、パネルがバー直下
//! ではなくページ本文の下端に落ちてしまう。`bar-wrap` はバー 1 行のみを
//! 内包するため、`top: 100%` が常にバー直下を指す）。`content` 自身は
//! `inset-inline: 0; min-width: 0;` を追加宣言し、ヘッダー全幅へ広げる
//! （`top`/`left`/`z-index` は recipe 既定のまま変更不要）。中身は
//! `.blocks-header-mega-menu-panel-inner` で `max-inline-size` + 中央寄せ
//! を与え、上のバーと同じ幅に揃える。
//!
//! # `.blocks-demo` のはみ出し対策
//!
//! `crate::blocks::stylesheet` の `.blocks-demo` は `overflow-x: auto` を
//! 持つ（`blocks_stylesheet_declares_demo_frame_overflow` 契約）。絶対配置
//! パネルがフローに寄与しないため、`.blocks-header-mega-menu-page`
//! （ダミー本文枠）へ `min-block-size` を持たせ、展開済みパネルが
//! レイアウトボックスの内側に収まるようにする。
//!
//! # レスポンシブ（`@media (max-width: 47.99rem)`）
//!
//! [`super::super::footer::footer_newsletter`] と同じブレークポイントを
//! 使う。狭い幅ではナビ・アクション（`.blocks-header-mega-menu-nav`/
//! `.blocks-header-mega-menu-actions`）を非表示にし、ハンバーガー
//! （`[data-blocks-header-mega-menu-hamburger]`）を表示する。狭幅で開いた
//! 状態の並記は #2859 の担当のため、本 block は畳んだハンバーガー
//! （`aria-expanded="false"`）の静的表示のみを持つ。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `navigation_menu` のパート関数（`root`/`list`/`item`/`trigger`/
//! `item_indicator`/`content`/`link`）は呼び出し側 `attrs` の `class` を
//! 除去しない（`crates/headless-ui/src/navigation_menu.rs` の
//! `drop_reserved` は `data-*`/`aria-*`/`type`/`disabled` 系の予約キーのみを
//! 対象とし `class` を含まない）ため、`.blocks-header-mega-menu-nav` の
//! ような class フックがそのまま使える。一方 `button::button`/
//! `link::root`/`icon::icon` は `drop_class_attr` で呼び出し側 `class` を
//! 常に除去する契約のため、これらへのフックは
//! `data-blocks-header-mega-menu-*` 属性で渡す
//! （[`super::super::faq::faq_accordion_centered`] と同型の判断）。
//!
//! # href の方針
//!
//! `href="#"` は使わない（横断テストが禁止する）。サイト内に実在する
//! 索引ページへの相対パス（`../../guides/`・`../../themes/`・
//! `../../primitives/`・`../../api/`・`../../examples/`）を使う
//! （[`super::super::faq::faq_question_rows`] と同型の判断）。
//!
//! # id 接頭辞
//!
//! `id`/`aria-controls`/`aria-labelledby` は唯一の開いたドロップダウン
//! （プロダクト）にのみ必要であり、
//! `blocks-header-mega-menu-products-{trigger|content}` の固定文字列で
//! 一意にする（複数項目に添字展開する accordion 系 block とは異なり、
//! `format!` を使わない。`crate::blocks` モジュール doc「HTML 文字列の
//! 直接組み立て禁止」節参照）。
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
//! も `type="button"` 固定）のまま送信先を持たない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};

/// パネル項目 1 件（タイトル, 説明, href）。href はサイト内に実在する
/// 索引ページへの相対パス（モジュール冒頭 rustdoc「href の方針」節）。
type PanelItem = (&'static str, &'static str, &'static str);

/// パネルの列 1 件（列見出し, 項目 3 件）。
type PanelColumn = (&'static str, [PanelItem; 3]);

/// プロダクトパネルの列一覧（列見出し + アイコン付き項目 3 件 × 2 列）。
const PANEL_COLUMNS: [PanelColumn; 2] = [
    (
        "分析",
        [
            (
                "ダッシュボード",
                "利用状況をひと目で把握できる可視化パネル。",
                "../../themes/",
            ),
            (
                "レポート",
                "定期集計を自動で生成するレポート機能。",
                "../../guides/",
            ),
            (
                "アラート",
                "しきい値超過を通知する監視機能。",
                "../../primitives/",
            ),
        ],
    ),
    (
        "連携",
        [
            (
                "API",
                "外部システムと連携するための拡張ポイント。",
                "../../api/",
            ),
            (
                "サンプル集",
                "構成別の実装サンプルへの索引。",
                "../../examples/",
            ),
            (
                "導入ガイド",
                "はじめての導入手順をまとめたガイド。",
                "../../guides/",
            ),
        ],
    ),
];

/// 唯一開いた状態で固定するトリガーの `id`（モジュール冒頭 rustdoc
/// 「id 接頭辞」節。項目が 1 件のみのため `format!` による添字展開は
/// 行わない）。
const PRODUCTS_TRIGGER_ID: &str = "blocks-header-mega-menu-products-trigger";
/// [`PRODUCTS_TRIGGER_ID`] と対になる `content` の `id`。
const PRODUCTS_CONTENT_ID: &str = "blocks-header-mega-menu-products-content";

/// ブランドロゴ（装飾用の幾何アイコン、菱形）。実在ブランドのロゴ・
/// 商標を模さない独自の単純図形（`docs/design/wireframe-ui-architecture.md`
/// と同じ判断軸）。
fn brand_icon() -> Node {
    icon::icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", "M12 2L22 12L12 22L2 12Z")], vec![])],
    )
}

/// ハンバーガーアイコン（装飾用、3 本線）。アクセシブルネームはボタン側の
/// `aria-label` が担うため `label: None`（`aria-hidden` が付く）。
fn hamburger_icon() -> Node {
    icon::icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![
            el(
                "rect",
                vec![("x", "3"), ("y", "6"), ("width", "18"), ("height", "2")],
                vec![],
            ),
            el(
                "rect",
                vec![("x", "3"), ("y", "11"), ("width", "18"), ("height", "2")],
                vec![],
            ),
            el(
                "rect",
                vec![("x", "3"), ("y", "16"), ("width", "18"), ("height", "2")],
                vec![],
            ),
        ],
    )
}

/// ブランド領域（アイコン + 架空のブランド名）。
fn brand() -> Node {
    div(
        vec![("class", "blocks-header-mega-menu-brand")],
        vec![brand_icon(), span(vec![], vec![text("Nimbus Studio")])],
    )
}

/// パネル 1 列分（列見出し + アイコン付き項目 3 件）。
fn panel_column(heading: &str, items: &[PanelItem; 3]) -> Node {
    let links: Vec<Node> = items
        .iter()
        .map(|(title, description, href)| {
            navigation_menu::link(
                href,
                false,
                vec![("class", "blocks-header-mega-menu-panel-link")],
                vec![
                    span(
                        vec![("class", "blocks-header-mega-menu-panel-link-title")],
                        vec![text(*title)],
                    ),
                    span(
                        vec![("class", "blocks-header-mega-menu-panel-link-description")],
                        vec![text(*description)],
                    ),
                ],
            )
        })
        .collect();
    let mut children = vec![span(
        vec![("class", "blocks-header-mega-menu-panel-column-heading")],
        vec![text(heading)],
    )];
    children.extend(links);
    div(
        vec![("class", "blocks-header-mega-menu-panel-column")],
        children,
    )
}

/// 「プロダクト」トップ項目（唯一のドロップダウン、常時 open 固定）。
fn products_item(props: &NavigationMenuProps) -> Node {
    let state = OpenState::Open;
    let columns: Vec<Node> = PANEL_COLUMNS
        .iter()
        .map(|(heading, items)| panel_column(heading, items))
        .collect();

    navigation_menu::item(
        state,
        false,
        props,
        "products",
        vec![],
        vec![
            navigation_menu::trigger(
                state,
                false,
                "products",
                Some(PRODUCTS_TRIGGER_ID),
                Some(PRODUCTS_CONTENT_ID),
                vec![],
                vec![
                    text("プロダクト"),
                    navigation_menu::item_indicator(
                        state,
                        props,
                        "products",
                        vec![],
                        vec![text("▾")],
                    ),
                ],
            ),
            navigation_menu::content(
                state,
                props,
                "products",
                Some(PRODUCTS_CONTENT_ID),
                Some(PRODUCTS_TRIGGER_ID),
                vec![],
                vec![div(
                    vec![("class", "blocks-header-mega-menu-panel-inner")],
                    columns,
                )],
            ),
        ],
    )
}

/// トリガーを持たない、リンクのみのトップ項目（料金・ドキュメント）。
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

/// バー内のナビゲーション（メガメニュー本体）。
fn nav() -> Node {
    let props = NavigationMenuProps::default();
    navigation_menu::root(
        &props,
        "メインメニュー",
        vec![("class", "blocks-header-mega-menu-nav")],
        vec![navigation_menu::list(
            &props,
            vec![],
            vec![
                products_item(&props),
                link_item(&props, "pricing", "料金", "../../themes/"),
                link_item(&props, "docs", "ドキュメント", "../../guides/"),
            ],
        )],
    )
}

/// バー右側のアクション（ログインリンク + CTA ボタン）。
fn actions() -> Node {
    div(
        vec![("class", "blocks-header-mega-menu-actions")],
        vec![
            link::root(
                "../../guides/",
                &LinkProps::default(),
                vec![],
                vec![text("ログイン")],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("無料で始める")]),
        ],
    )
}

/// 狭い幅で表示するハンバーガーボタン（畳んだ状態の静的表示のみ、開いた
/// 状態の並記は #2859 の担当）。
fn hamburger() -> Node {
    button::button(
        &ButtonProps {
            variant: ButtonVariant::Ghost,
            ..ButtonProps::default()
        },
        vec![
            ("data-blocks-header-mega-menu-hamburger", ""),
            ("aria-label", "メニューを開く"),
            ("aria-expanded", "false"),
        ],
        vec![hamburger_icon()],
    )
}

/// 幅を制限したバー（ブランド / ナビ / アクション / ハンバーガー）。
fn bar() -> Node {
    div(
        vec![("class", "blocks-header-mega-menu-bar")],
        vec![brand(), nav(), actions(), hamburger()],
    )
}

/// ダミーのページ本文（`.blocks-demo` のはみ出し対策、モジュール冒頭
/// rustdoc 参照）。
fn page_placeholder() -> Node {
    div(
        vec![("class", "blocks-header-mega-menu-page")],
        vec![text(
            "ページ本文（ダミー）。展開済みパネルの下に十分な高さを確保するための枠。",
        )],
    )
}

/// `header-mega-menu` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-header-mega-menu-layout")],
        vec![
            div(
                vec![("class", "blocks-header-mega-menu-bar-wrap")],
                vec![bar()],
            ),
            page_placeholder(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/header-mega-menu/",
    title: "header-mega-menu",
    category: BlockCategory::Header,
    rust_source: "crates/docs-site/src/blocks/marketing/header/header_mega_menu.rs",
    demo_class: "blocks-header-mega-menu",
    parts: &[
        Part {
            label: "Navigation Menu",
            path: "/themes/navigation-menu/",
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
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `header_mega_menu` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` ではなく
/// 本ファイル内 private 定数として `super::stylesheet` 経由の `push_css`
/// で連結される）。
///
/// セレクタは `.blocks-header-mega-menu-*`、
/// `[data-blocks-header-mega-menu-*]`、および `.blocks-header-mega-menu-
/// layout` を祖先に持つ `[data-scope="navigation-menu"]` 系セレクタへの
/// 子孫結合子付き上書き（全幅パネル化、モジュール冒頭 rustdoc「全幅パネル
/// の配置方法」節）のみを用い、他 block や部品の素のセレクタへ影響させ
/// ない。値はすべて `var(--fandhe-*)` トークンで書き、生の色リテラルは
/// 使わない。
const LAYOUT_CSS: &str = "\
.blocks-header-mega-menu-bar-wrap {\n  position: relative;\n  background: var(--fandhe-color-bg);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-header-mega-menu-bar {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  max-inline-size: 64rem;\n  margin-inline: auto;\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n}\n\
.blocks-header-mega-menu-brand {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  font-weight: 600;\n  white-space: nowrap;\n}\n\
.blocks-header-mega-menu-layout [data-scope=\"navigation-menu\"][data-part=\"root\"] {\n  flex: 1;\n}\n\
.blocks-header-mega-menu-layout [data-scope=\"navigation-menu\"][data-part=\"root\"],\n\
.blocks-header-mega-menu-layout [data-scope=\"navigation-menu\"][data-part=\"item\"] {\n  position: static;\n}\n\
.blocks-header-mega-menu-layout [data-scope=\"navigation-menu\"][data-part=\"content\"] {\n  inset-inline: 0;\n  min-width: 0;\n}\n\
.blocks-header-mega-menu-panel-inner {\n  max-inline-size: 64rem;\n  margin-inline: auto;\n  display: grid;\n  grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));\n  gap: var(--fandhe-space-6);\n  padding: var(--fandhe-space-2) var(--fandhe-space-4);\n}\n\
.blocks-header-mega-menu-panel-column {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-header-mega-menu-panel-column-heading {\n  display: block;\n  font-weight: 600;\n  font-size: var(--fandhe-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n  margin-block-end: var(--fandhe-space-2);\n}\n\
.blocks-header-mega-menu-panel-link {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-header-mega-menu-panel-link-title {\n  font-weight: 600;\n}\n\
.blocks-header-mega-menu-panel-link-description {\n  font-size: var(--fandhe-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-header-mega-menu-actions {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  white-space: nowrap;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-header-mega-menu-hamburger] {\n  display: none;\n}\n\
.blocks-header-mega-menu-page {\n  min-block-size: 28rem;\n  padding: var(--fandhe-space-6) var(--fandhe-space-4);\n  color: var(--fandhe-color-fg-muted);\n}\n\
@media (max-width: 47.99rem) {\n  .blocks-header-mega-menu-nav[data-scope=\"navigation-menu\"][data-part=\"root\"] {\n    display: none;\n  }\n  .blocks-header-mega-menu-actions {\n    display: none;\n  }\n  [data-scope=\"button\"][data-part=\"root\"][data-blocks-header-mega-menu-hamburger] {\n    display: inline-flex;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, PRODUCTS_CONTENT_ID, PRODUCTS_TRIGGER_ID};
    use fandhe_frontend_core::render;

    /// Demo が期待する 4 種の部品・非対話制約を満たすことの単体回帰
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"navigation-menu\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 唯一のドロップダウン（プロダクト）が常時 open で固定され、`hidden`
    /// を持たないこと（モジュール doc「静的表示」節）。閉じた
    /// trigger（`aria-expanded="false"` の navigation-menu trigger）は
    /// 存在しないこと（リンクのみの項目は trigger を持たないため）。
    #[test]
    fn demo_renders_single_open_dropdown() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-part="item" data-state="open""#)
                .count(),
            1,
            "html={html}"
        );
        assert_eq!(html.matches("data-part=\"content\"").count(), 1);
        assert!(!html.contains(r#"data-part="content" data-state="closed""#));
        assert_eq!(
            html.matches(r#"data-part="trigger""#).count(),
            1,
            "リンクのみの項目は trigger を持たない: html={html}"
        );
        // ハンバーガーボタン自身は `aria-expanded="false"` を持つため
        // （畳んだ状態の静的表示）、ここでは navigation-menu の trigger
        // パートに限定して aria-expanded="false" が存在しないことを見る。
        assert!(!html.contains(r#"data-part="trigger" aria-expanded="false""#));
        assert!(html.contains(&format!("id=\"{PRODUCTS_TRIGGER_ID}\"")));
        assert!(html.contains(&format!("aria-controls=\"{PRODUCTS_CONTENT_ID}\"")));
        assert!(html.contains(&format!("id=\"{PRODUCTS_CONTENT_ID}\"")));
        assert!(html.contains(&format!("aria-labelledby=\"{PRODUCTS_TRIGGER_ID}\"")));
    }

    /// ハンバーガーボタンがアクセシブルネームを持つこと。
    #[test]
    fn hamburger_has_accessible_name() {
        let html = render(&demo());
        assert!(html.contains(r#"aria-label="メニューを開く""#));
        assert!(html.contains("data-blocks-header-mega-menu-hamburger"));
    }

    /// [`LAYOUT_CSS`] が全幅パネル化（`position: static` 上書き・
    /// `inset-inline: 0`）とレスポンシブ切り替えを持つこと。
    #[test]
    fn layout_css_makes_panel_full_width() {
        assert!(LAYOUT_CSS.contains("position: static;"));
        assert!(LAYOUT_CSS.contains("inset-inline: 0;"));
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-header-mega-menu-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-header-mega-menu-layout");
    }
}
