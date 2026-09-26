# header-flyout-menu

`fandhe-frontend-pre-styled-ui` の `navigation-menu` / `button` / `icon` /
`link` 部品を合成した、ドロップダウン付きヘッダーです。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください（主参照は対応表 ID R0981。出典の固有名・
ファイル名は記載しません）。

ロゴ・ナビ・アクションを横並びに配置し、48rem 未満の狭い幅ではナビと
アクションを隠してハンバーガーボタンのみを表示します。Demo は集約元の
バリエーション（対応表 ID R0575/R0576/R0577/R0987/R0988 を集約）を 3 形
並記しています。標準形は「製品」フライアウトを開いた状態（末尾に補助 CTA
行つき）・「リソース」は閉じた状態のまま固定表示し、各項目はアイコン・
ラベル・説明の 3 要素で構成しています。テキストのみ形はラベルだけの
ドロップダウン + ボタン 2 個、中央寄せ形は 48rem 以上でナビを中央へ寄せ、
フライアウト項目をアイコン付き 2 列グリッドで並べます。

本 Demo は静的な表示例であり、docs サイトは JS ハイドレーションを行わない
ため、押しても何も起きないトリガー（ナビ項目のトリガー・ハンバーガー
ボタン・主 CTA ボタン）はすべて `disabled` にして操作不能であることを
明示しています。`<form>` 要素は一切持たず、データの取得・送信・状態管理を
行いません。アクション行のボタンは `type="button"` のまま送信先を持ちません。
文言はすべて独自に書いた架空のものであり、実企業名・実クレデンシャル・PII を
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

/// centered 形フライアウトの項目一覧（2 列グリッドで並べる、R0577）。
const PLATFORM_FLYOUT_ITEMS: &[(&str, &str, &str, &str)] = &[
    (
        "M4 4h16v4H4zM4 12h16v8H4z",
        "ワークスペース",
        "チームで共有する作業スペース",
        REPO,
    ),
    (
        "M12 2l9 4.9v9.6L12 22l-9-4.9V6.9z",
        "API",
        "外部サービスと連携する REST/gRPC API",
        REPO,
    ),
    (
        "M4 12a8 8 0 1116 0 8 8 0 01-16 0zm8-5v5l3 3",
        "モニタリング",
        "稼働状況をリアルタイムに可視化",
        REPO,
    ),
    (
        "M12 2l7 3v6c0 4.9-3 8.7-7 11-4-2.3-7-6.1-7-11V5z",
        "アクセス制御",
        "ロールベースの権限管理",
        REPO,
    ),
];

/// simple 形フライアウトの項目一覧（テキストのみ、R0575）。
const SERVICES_TEXT_ITEMS: &[(&str, &str)] = &[
    ("導入支援", REPO),
    ("運用代行", REPO),
    ("トレーニング", REPO),
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

/// アイコン + ラベル + 説明の項目一覧を [`flyout_item`] へ写像する
/// （`nav_item_with_flyout` へ渡す `items` の組み立て用）。
fn flyout_items(items: &[(&str, &str, &str, &str)]) -> Vec<Node> {
    items
        .iter()
        .map(|(icon_path, label, description, href)| {
            flyout_item(icon_path, label, description, href)
        })
        .collect()
}

/// テキストのみの簡易フライアウト項目（R0575。アイコン・説明を持たない）。
fn flyout_text_item(label: &str, href: &str) -> Node {
    navigation_menu::link(
        href,
        false,
        vec![("data-blocks-header-flyout-menu-text-item", "")],
        vec![text(label)],
    )
}

/// フライアウト下部の補助 CTA 行 1 本（アイコン付き `link::root`、
/// モジュール doc「補助 CTA 行」節）。
fn footer_link(icon_path: &str, label: &str, href: &str) -> Node {
    link::root(
        href,
        &LinkProps::default(),
        vec![("data-blocks-header-flyout-menu-panel-footer-link", "")],
        vec![geo_icon(Size::Sm, icon_path), text(label)],
    )
}

/// 補助 CTA 行のコンテナ（2 本のリンクを横並びで持つ）。
fn panel_footer() -> Node {
    div(
        vec![("data-blocks-header-flyout-menu-panel-footer", "")],
        vec![
            footer_link("M5 12h14M13 6l6 6-6 6", "デモを見る", REPO),
            footer_link("M2 12h20M12 2v20", "営業に相談する", REPO),
        ],
    )
}

/// caption（並記された各形の見出し、`footer_inline_nav` 前例と同型）。
fn caption(label: &str) -> Node {
    p(
        vec![("class", "blocks-header-flyout-menu-caption")],
        vec![text(label)],
    )
}

/// トリガー付きナビ項目（フライアウトを持つ）。`disabled: true` 固定
/// （モジュール doc「静的表示」節）。`footer`（補助 CTA 行）・
/// `columns`（パネルの列数、centered 形の 2 列グリッド用）は任意。
fn nav_item_with_flyout(
    props: &NavigationMenuProps,
    value: &str,
    label: &str,
    state: OpenState,
    items: Vec<Node>,
    footer: Option<Node>,
    columns: Option<&'static str>,
) -> Node {
    let trigger_id = format!("blocks-header-flyout-menu-{value}-trigger");
    let content_id = format!("blocks-header-flyout-menu-{value}-content");

    let mut panel_attrs = vec![("data-blocks-header-flyout-menu-panel", "")];
    if let Some(cols) = columns {
        panel_attrs.push(("data-blocks-header-flyout-menu-columns", cols));
    }

    let mut panel_children = items;
    if let Some(footer_node) = footer {
        panel_children.push(footer_node);
    }

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
                panel_attrs,
                panel_children,
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

/// 横並びナビ本体（standard 形、R0981）。
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
                    flyout_items(PRODUCT_FLYOUT_ITEMS),
                    Some(panel_footer()),
                    None,
                ),
                nav_item_with_flyout(
                    &props,
                    "resources",
                    "リソース",
                    OpenState::Closed,
                    flyout_items(RESOURCES_FLYOUT_ITEMS),
                    None,
                    None,
                ),
                nav_link_item(&props, "pricing", "料金", REPO),
                nav_link_item(&props, "company", "会社概要", ORG),
            ],
        )],
    )
}

/// アクション行（ログイン + 主 CTA）。無 JS デモのため主 CTA も
/// ハンバーガーと同じ理由（モジュール doc「静的表示」節）で
/// `disabled: true` 固定にし、フォーカス・クリック不能を明示する。
fn actions() -> Node {
    div(
        vec![("data-blocks-header-flyout-menu-actions", "")],
        vec![
            link::root(REPO, &LinkProps::default(), vec![], vec![text("ログイン")]),
            button::button(
                &ButtonProps {
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-header-flyout-menu-cta", "")],
                vec![text("使ってみる")],
            ),
        ],
    )
}

/// simple 形のアクション行（R0575。ボタン 2 個、どちらも `disabled: true`）。
fn actions_simple() -> Node {
    div(
        vec![("data-blocks-header-flyout-menu-actions", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-header-flyout-menu-cta-secondary", "")],
                vec![text("ログイン")],
            ),
            button::button(
                &ButtonProps {
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-header-flyout-menu-cta", "")],
                vec![text("使ってみる")],
            ),
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

/// standard 形（R0981 主参照）。
fn standard_header() -> Node {
    header(
        vec![
            ("class", "blocks-header-flyout-menu-layout"),
            ("data-blocks-header-flyout-menu-root", ""),
            ("data-blocks-header-flyout-menu-variant", "standard"),
        ],
        vec![logo(), main_nav(), actions(), hamburger()],
    )
}

/// simple 形（R0575。テキストのみのドロップダウン + ボタン 2 個）。
fn simple_header() -> Node {
    let props = NavigationMenuProps::default();
    let nav = navigation_menu::root(
        &props,
        "Main (text only)",
        vec![("data-blocks-header-flyout-menu-nav", "")],
        vec![navigation_menu::list(
            &props,
            vec![],
            vec![nav_item_with_flyout(
                &props,
                "services",
                "サービス",
                OpenState::Open,
                SERVICES_TEXT_ITEMS
                    .iter()
                    .map(|(label, href)| flyout_text_item(label, href))
                    .collect(),
                None,
                None,
            )],
        )],
    );
    header(
        vec![
            ("class", "blocks-header-flyout-menu-layout"),
            ("data-blocks-header-flyout-menu-root", ""),
            ("data-blocks-header-flyout-menu-variant", "simple"),
        ],
        vec![logo(), nav, actions_simple(), hamburger()],
    )
}

/// centered 形（R0577。48rem 以上でナビを中央寄せ、フライアウトは
/// アイコン付き項目を 2 列グリッドで並べ末尾に補助 CTA を持つ）。
fn centered_header() -> Node {
    let props = NavigationMenuProps::default();
    let nav = navigation_menu::root(
        &props,
        "Main (centered)",
        vec![("data-blocks-header-flyout-menu-nav", "")],
        vec![navigation_menu::list(
            &props,
            vec![],
            vec![
                nav_item_with_flyout(
                    &props,
                    "platform",
                    "プラットフォーム",
                    OpenState::Open,
                    flyout_items(PLATFORM_FLYOUT_ITEMS),
                    Some(panel_footer()),
                    Some("2"),
                ),
                nav_link_item(&props, "docs", "ドキュメント", REPO),
                nav_link_item(&props, "about", "会社概要", ORG),
            ],
        )],
    );
    header(
        vec![
            ("class", "blocks-header-flyout-menu-layout"),
            ("data-blocks-header-flyout-menu-root", ""),
            ("data-blocks-header-flyout-menu-variant", "centered"),
        ],
        vec![logo(), nav, actions(), hamburger()],
    )
}

/// `header-flyout-menu` の Demo 本体。集約元のバリエーション（standard/
/// simple/centered）を縦に並記する（モジュール doc「3 形の並記」節、
/// `footer_inline_nav` 前例と同型）。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-header-flyout-menu-stack")],
        vec![
            caption("標準形"),
            standard_header(),
            caption("テキストのみ"),
            simple_header(),
            caption("中央寄せ"),
            centered_header(),
        ],
    )
}
```

## 原案差分メモ

- 集約元のバリエーションは 3 形（標準形/テキストのみ形/中央寄せ形）で
  並記しました。標準形が R0981（主参照）、テキストのみ形が R0575、
  中央寄せ形が R0577 に対応します。
- R0576/R0988（モバイル drawer トリガー）は Demo に描画していません。
  狭幅でのハンバーガーボタンがトリガー相当ですが、docs サイトは無 JS の
  ため開いた先の drawer 自体を表現できないためです。
- R0987（2 項目のフライアウト）は、標準形の「リソース」（2 項目・閉状態）
  と同じ構造であるため、独立したインスタンスにはしていません。
- 押しても何も起きないトリガー・ボタン（各形のフライアウトトリガー・
  ハンバーガー・アクション行のボタン）はすべて `disabled` にして固定し、
  フォーカス・操作不能であることを明示しています。
- 文言・アイコンはすべて独自に書いた架空のものです。配色・余白・角丸は
  既存のテーマトークンに従っています。

関連情報: [Navigation Menu](../themes/navigation-menu.md) /
[Button](../themes/button.md) / [Icon](../themes/icon.md) /
[Link](../themes/link.md)
