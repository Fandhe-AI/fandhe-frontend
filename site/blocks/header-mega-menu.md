# header-mega-menu

`fandhe-frontend-pre-styled-ui` の `navigation-menu` / `button` / `icon` /
`link` 部品を合成した、全幅メガメニュー付きヘッダーです。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0984。出典の固有名・ファイル名は記載しません）。

幅を制限したバー（ブランド / ナビ / アクション）の下に、ヘッダー全幅まで
広がるドロップダウンパネルを持ちます。パネルはアイコン付き項目を複数列に
並べた構成です。狭い幅ではナビ・アクションを畳み、ハンバーガーボタンへ
切り替わります（開いた状態の並記は後続の block で追加予定です）。

本 Demo は静的な表示例であり、唯一のドロップダウン（プロダクト）を常時
展開（open）した状態で固定します。トリガーを持たないトップ項目（料金・
ドキュメント）はリンクのみで構成し、無 JS のドキュメントサイトでも
本文へ到達できない閉じたトリガーを残しません。`<form>` 要素は一切持たず、
データの取得・送信・状態管理を行いません。ボタンは `type="button"` の
まま送信先を持ちません。文言・ブランド名はすべて独自に書いた架空の
ものであり、実企業名・実クレデンシャル・PII を含みません。

パネル下部の補助 CTA 帯・狭幅で開いた状態の並記・原案差分メモは
後続の block で追加予定です。

## Rust コード

```rust
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
```

関連情報: [Navigation Menu](../themes/navigation-menu.md) /
[Button](../themes/button.md) / [Icon](../themes/icon.md) /
[Link](../themes/link.md)
