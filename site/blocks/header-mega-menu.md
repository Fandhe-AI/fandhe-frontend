# header-mega-menu

`fandhe-frontend-pre-styled-ui` の `navigation-menu` / `button` / `icon` /
`link` 部品を合成した、全幅メガメニュー付きヘッダーです。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0984。出典の固有名・ファイル名は記載しません）。

幅を制限したバー（ブランド / ナビ / アクション）の下に、ヘッダー全幅まで
広がるドロップダウンパネルを持ちます。パネルはアイコン付き項目を複数列に
並べた構成です。無 JS の静的表示のためハンバーガーへの開閉切り替えは
持たず、狭い幅ではバー内でナビ・アクションを折り返して常時到達可能な
まま残します。

本 Demo は静的な表示例であり、唯一のドロップダウン（プロダクト）を常時
展開（open）した状態で固定します。トリガーを持たないトップ項目（料金・
ドキュメント）はリンクのみで構成し、無 JS のドキュメントサイトでも
本文へ到達できない閉じたトリガーを残しません。`<form>` 要素は一切持たず、
データの取得・送信・状態管理を行いません。ボタンは `type="button"` の
まま送信先を持ちません。文言・ブランド名はすべて独自に書いた架空の
ものであり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::Size;

/// パネル項目 1 件（タイトル, 説明, href, アイコンの `path` `d`）。href は
/// サイト内に実在する索引ページへの相対パス（モジュール冒頭 rustdoc「href
/// の方針」節）。アイコンは PR #3273 レビュー指摘（P2）是正: モジュール
/// doc・本定数のコメントが言う「アイコン付き項目」を実際に描画する
/// （[`item_icon`] 参照。`error_page_popular_links::stroke_icon` と同型の
/// 装飾用線画）。
type PanelItem = (&'static str, &'static str, &'static str, &'static str);

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
                "M4 4h16v12H4zM8 20h8M12 16v4",
            ),
            (
                "レポート",
                "定期集計を自動で生成するレポート機能。",
                "../../guides/",
                "M6 3h9l3 3v15H6zM8 10h8M8 14h8M8 18h5",
            ),
            (
                "アラート",
                "しきい値超過を通知する監視機能。",
                "../../primitives/",
                "M12 3a6 6 0 0 0-6 6c0 5-2 6-2 6h16s-2-1-2-6a6 6 0 0 0-6-6zM10 19a2 2 0 0 0 4 0",
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
                "M8 6 3 12l5 6M16 6l5 6-5 6",
            ),
            (
                "サンプル集",
                "構成別の実装サンプルへの索引。",
                "../../examples/",
                "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
            ),
            (
                "導入ガイド",
                "はじめての導入手順をまとめたガイド。",
                "../../guides/",
                "M12 3v18M4 8l8-5 8 5M4 16l8 5 8-5",
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

/// リポジトリ実 URL（`href` の方針）。本サイトに実在するログインページは
/// 無いため、遷移先はこの実在の外部 URL を使う。ただし [`actions`] の
/// リンク文言は「ログイン」ではなく行き先どおり「GitHub」とする（PR #3273
/// レビュー指摘: 「ログイン」という文言のまま GitHub リポジトリへ飛ばすと
/// リンク名と実際の行き先が食い違い、ログイン画面に到達すると誤認させる。
/// [`super::header_flyout_menu`] の同型リンクは同じ不一致を抱えたまま
/// 既に main へマージ済みで、本 block の範囲外のため別途追跡する
/// （`.claude/rules/out-of-scope-tracking.md`）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

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

/// ブランド領域（アイコン + 架空のブランド名）。
fn brand() -> Node {
    div(
        vec![("class", "blocks-header-mega-menu-brand")],
        vec![brand_icon(), span(vec![], vec![text("Nimbus Studio")])],
    )
}

/// パネル項目のアイコン（線画、装飾用途。[`error_page_popular_links`] の
/// `stroke_icon` と同型のパターンで、`icon::icon` の `currentColor`
/// 継承に任せ生の色リテラルは持ち込まない。PR #3273 レビュー指摘（P2）
/// 是正: 従来ブランド領域にしか icon が無かった不整合を解消する）。
///
/// [`error_page_popular_links`]: crate::blocks::marketing::error_page::error_page_popular_links
fn item_icon(path_d: &str) -> Node {
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

/// パネル 1 列分（列見出し + アイコン付き項目 3 件）。
fn panel_column(heading: &str, items: &[PanelItem; 3]) -> Node {
    let links: Vec<Node> = items
        .iter()
        .map(|(title, description, href, icon_path_d)| {
            navigation_menu::link(
                href,
                false,
                vec![("class", "blocks-header-mega-menu-panel-link")],
                vec![
                    div(
                        vec![("class", "blocks-header-mega-menu-panel-link-header")],
                        vec![
                            item_icon(icon_path_d),
                            span(
                                vec![("class", "blocks-header-mega-menu-panel-link-title")],
                                vec![text(*title)],
                            ),
                        ],
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
                true,
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

/// バー右側のアクション（GitHub リンク + CTA ボタン）。PR #3273 レビュー
/// 指摘（P2）是正: 当初「ログイン」ラベルで [`REPO`]（GitHub リポジトリ）
/// へ遷移させていたが、本サイトに実在するログインページは無く、リンク名
/// （ログイン）と実際の行き先（GitHub）が食い違っていた。行き先を変えず
/// ラベルを実態（GitHub リポジトリ）に合わせて是正する（[`REPO`] の doc
/// コメント参照）。CTA（「無料で始める」）は遷移先・送信処理を持たない
/// no-op のため、`disabled: true` にしてフォーカス・クリック不能を明示する
/// （`disabled_declarations()` は [`LAYOUT_CSS`] で中和し通常の CTA と
/// 同じ見た目に保つ）。
fn actions() -> Node {
    div(
        vec![("class", "blocks-header-mega-menu-actions")],
        vec![
            link::root(REPO, &LinkProps::default(), vec![], vec![text("GitHub")]),
            button::button(
                &ButtonProps {
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-header-mega-menu-cta", "")],
                vec![text("無料で始める")],
            ),
        ],
    )
}

/// 幅を制限したバー（ブランド / ナビ / アクション）。狭い幅では
/// ハンバーガーで畳まず、[`LAYOUT_CSS`] の `flex-wrap` でバー内へ折り返す
/// （モジュール冒頭 rustdoc「レスポンシブ」節）。
fn bar() -> Node {
    div(
        vec![("class", "blocks-header-mega-menu-bar")],
        vec![brand(), nav(), actions()],
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
