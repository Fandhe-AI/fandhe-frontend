# header-flyout-menu

`fandhe-frontend-pre-styled-ui` の `navigation-menu` / `button` / `icon` /
`link` 部品を合成した、ドロップダウン付きヘッダーです。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください（主参照は対応表 ID R0981。出典の固有名・
ファイル名は記載しません）。

ロゴ・ナビ・アクションを横並びに配置し、48rem 未満の狭い幅ではナビと
アクションを隠してハンバーガーボタンのみを表示します。ナビ項目のうち
「製品」はフライアウトを開いた状態、「リソース」は閉じた状態のまま固定
表示し、各項目はアイコン・ラベル・説明の 3 要素で構成しています。

本 Demo は静的な表示例であり、docs サイトは JS ハイドレーションを行わない
ため、押しても何も起きないトリガー（ナビ項目のトリガー・ハンバーガー
ボタン）はすべて `disabled` にして操作不能であることを明示しています。
`<form>` 要素は一切持たず、データの取得・送信・状態管理を行いません。
アクション行のボタンは `type="button"` のまま送信先を持ちません。文言は
すべて独自に書いた架空のものであり、実企業名・実クレデンシャル・PII を
含みません。実際に使うときはリンク先を差し替えてください。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, header, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::Size;

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// 実在の自組織 URL（Company リンク用）。
const ORG: &str = "https://github.com/Fandhe-AI";

/// 「Product」フライアウトの項目一覧（アイコン path・ラベル・説明・
/// href の組。件数を単体テストから参照するため `const` 配列にする）。
const PRODUCT_FLYOUT_ITEMS: &[(&str, &str, &str, &str)] = &[
    (
        "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
        "ダッシュボード",
        "主要な指標をひと目で確認できる管理画面",
        REPO,
    ),
    (
        "M12 2l2.4 7.2H22l-6 4.6 2.3 7.2L12 16.4 5.7 21l2.3-7.2-6-4.6h7.6z",
        "自動化",
        "繰り返し作業をワークフローとして自動化",
        REPO,
    ),
    (
        "M4 12a8 8 0 1116 0 8 8 0 01-16 0zm8-5v5l3 3",
        "レポート",
        "チームの成果を定期レポートとして共有",
        REPO,
    ),
    (
        "M12 2l9 4.9v9.6L12 22l-9-4.9V6.9z",
        "連携",
        "他ツールとの連携でデータを一元管理",
        REPO,
    ),
    (
        "M12 2l7 3v6c0 4.9-3 8.7-7 11-4-2.3-7-6.1-7-11V5z",
        "セキュリティ",
        "アクセス権限と監査ログを一元管理",
        REPO,
    ),
];

/// 「Resources」フライアウトの項目一覧。
const RESOURCES_FLYOUT_ITEMS: &[(&str, &str, &str, &str)] = &[
    (
        "M6 2h9l3 3v17H6zM15 2v3h3",
        "ドキュメント",
        "導入手順と API リファレンス",
        REPO,
    ),
    (
        "M12 2a5 5 0 015 5 5 5 0 01-10 0 5 5 0 015-5zM4 22a8 8 0 0116 0",
        "コミュニティ",
        "利用者同士で情報交換できる場",
        REPO,
    ),
];

/// 装飾用の幾何図形アイコン（`label: None`）を組み立てる（自作 SVG、実在
/// ブランドのロゴを模さない）。
fn geo_icon(size: Size, d: &str) -> Node {
    icon(
        &IconProps {
            size,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", d)], vec![])],
    )
}

/// シェブロン（開閉インジケータ）用の装飾アイコン。
fn chevron_icon() -> Node {
    geo_icon(Size::Sm, "M6 9l6 6 6-6")
}

/// ハンバーガー（3 本線）アイコン。
fn hamburger_icon() -> Node {
    geo_icon(Size::Md, "M3 6h18v2H3zM3 11h18v2H3zM3 16h18v2H3z")
}

/// ロゴ（幾何図形アイコン + ブランド名テキスト）。実在ブランドのロゴを
/// 模さない自作の SVG。
fn logo() -> Node {
    link::root(
        REPO,
        &LinkProps::default(),
        vec![("data-blocks-header-flyout-menu-logo", "")],
        vec![
            geo_icon(Size::Md, "M4 4h16v6H4zM4 14h16v6H4z"),
            span(
                vec![("class", "blocks-header-flyout-menu-brand")],
                vec![text("Fandhe Frontend")],
            ),
        ],
    )
}

/// フライアウト 1 件分の項目（アイコン + ラベル + 説明の 3 要素）。
fn flyout_item(icon_path: &str, label: &str, description: &str, href: &str) -> Node {
    navigation_menu::link(
        href,
        false,
        vec![("data-blocks-header-flyout-menu-item", "")],
        vec![
            geo_icon(Size::Md, icon_path),
            div(
                vec![("class", "blocks-header-flyout-menu-item-text")],
                vec![
                    span(
                        vec![("class", "blocks-header-flyout-menu-item-label")],
                        vec![text(label)],
                    ),
                    p(
                        vec![("class", "blocks-header-flyout-menu-item-description")],
                        vec![text(description)],
                    ),
                ],
            ),
        ],
    )
}

/// トリガー付きナビ項目（フライアウトを持つ）。`disabled: true` 固定
/// （モジュール doc「静的表示」節）。
fn nav_item_with_flyout(
    props: &NavigationMenuProps,
    value: &str,
    label: &str,
    state: OpenState,
    items: &[(&str, &str, &str, &str)],
) -> Node {
    let trigger_id = format!("blocks-header-flyout-menu-{value}-trigger");
    let content_id = format!("blocks-header-flyout-menu-{value}-content");

    let flyout_items: Vec<Node> = items
        .iter()
        .map(|(icon_path, label, description, href)| {
            flyout_item(icon_path, label, description, href)
        })
        .collect();

    navigation_menu::item(
        state,
        false,
        props,
        value,
        vec![],
        vec![
            navigation_menu::trigger(
                state,
                true,
                value,
                Some(trigger_id.as_str()),
                Some(content_id.as_str()),
                vec![],
                vec![
                    span(vec![], vec![text(label)]),
                    navigation_menu::item_indicator(
                        state,
                        props,
                        value,
                        vec![],
                        vec![chevron_icon()],
                    ),
                ],
            ),
            navigation_menu::content(
                state,
                props,
                value,
                Some(content_id.as_str()),
                Some(trigger_id.as_str()),
                vec![("data-blocks-header-flyout-menu-panel", "")],
                flyout_items,
            ),
        ],
    )
}

/// トリガーを持たない素のリンク項目（Pricing/Company）。
fn nav_link_item(props: &NavigationMenuProps, value: &str, label: &str, href: &str) -> Node {
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

/// 横並びナビ本体。
fn main_nav() -> Node {
    let props = NavigationMenuProps::default();
    navigation_menu::root(
        &props,
        "Main",
        vec![("data-blocks-header-flyout-menu-nav", "")],
        vec![navigation_menu::list(
            &props,
            vec![],
            vec![
                nav_item_with_flyout(
                    &props,
                    "product",
                    "製品",
                    OpenState::Open,
                    PRODUCT_FLYOUT_ITEMS,
                ),
                nav_item_with_flyout(
                    &props,
                    "resources",
                    "リソース",
                    OpenState::Closed,
                    RESOURCES_FLYOUT_ITEMS,
                ),
                nav_link_item(&props, "pricing", "料金", REPO),
                nav_link_item(&props, "company", "会社概要", ORG),
            ],
        )],
    )
}

/// アクション行（ログイン + 主 CTA）。
fn actions() -> Node {
    div(
        vec![("data-blocks-header-flyout-menu-actions", "")],
        vec![
            link::root(REPO, &LinkProps::default(), vec![], vec![text("ログイン")]),
            button::button(&ButtonProps::default(), vec![], vec![text("使ってみる")]),
        ],
    )
}

/// ハンバーガーボタン（狭い幅専用、押しても何も起きないため
/// `disabled: true` 固定。`aria-controls` は開く先を描画しないため付けない）。
fn hamburger() -> Node {
    button::icon_button(
        &ButtonProps {
            variant: ButtonVariant::Ghost,
            disabled: true,
            ..ButtonProps::default()
        },
        "Open main menu",
        vec![("data-blocks-header-flyout-menu-toggle", "")],
        vec![hamburger_icon()],
    )
}

/// `header-flyout-menu` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    header(
        vec![
            ("class", "blocks-header-flyout-menu-layout"),
            ("data-blocks-header-flyout-menu-root", ""),
        ],
        vec![logo(), main_nav(), actions(), hamburger()],
    )
}
```

関連情報: [Navigation Menu](../themes/navigation-menu.md) /
[Button](../themes/button.md) / [Icon](../themes/icon.md) /
[Link](../themes/link.md)
