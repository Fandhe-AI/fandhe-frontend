# header-simple-bar

`fandhe-frontend-pre-styled-ui` の `navigation-menu` / `button` / `icon` /
`link` / `collapsible` 部品を合成した、1 段構成のシンプルなヘッダーバーです。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0982。出典の固有名・ファイル名は記載しません）。

ロゴ・メインナビ（リンク列）・アクション（ログイン/登録）を横並びに配置し、
48rem 未満の狭い幅ではデスクトップ用のナビとアクションを隠してハンバーガー
トリガーのみを表示します。ハンバーガーは常時展開のドロップダウンパネルへ
`aria-controls` で関連付けられており、狭い幅でもナビへ到達できます（パネルは
デスクトップ用ナビ・アクションの複製で、無 JS のため開閉はできず常に展開
した状態です）。Demo は配置違いの 4 variant を並記します: 中央寄せ・幅制限
（ロゴ左・ナビ中央）、左寄せ・全幅・下境界線（ロゴ + ナビ左・アクション右）、
右寄せ・登録のみ（ロゴ左・ナビ + アクション右）、ロゴ中央（ナビ左・ロゴ中央・
アクション右の 3 列 grid）です。

本 Demo は静的な表示例であり、docs サイトは JS ハイドレーションを行わない
ため、押しても何も起きないボタン（登録ボタン・ハンバーガートリガー）は
すべて `disabled` にして操作不能であることを明示しています。`<form>`
要素は一切持たず、データの取得・送信・状態管理を行いません。ボタンは
`type="button"` のまま送信先を持ちません。文言はすべて独自に書いた架空の
ものであり、実企業名・実クレデンシャル・PII を含みません。実際に使うときは
リンク先を差し替えてください。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, header, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::collapsible;
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::Size;

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc 参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// 実在の自組織 URL。
const ORG: &str = "https://github.com/Fandhe-AI";

/// メインナビの項目一覧（value, label, href）。
const NAV_ITEMS: &[(&str, &str, &str)] = &[
    ("product", "製品", REPO),
    ("pricing", "料金", REPO),
    ("company", "会社概要", ORG),
];

/// 装飾用の幾何図形アイコン（`label: None`、実在ブランドのロゴを模さない
/// 自作 SVG）。
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

/// ハンバーガー（3 本線）アイコン。
fn hamburger_icon() -> Node {
    geo_icon(Size::Md, "M3 6h18v2H3zM3 11h18v2H3zM3 16h18v2H3z")
}

/// ロゴ（幾何図形アイコン + ブランド名テキスト）。
fn logo() -> Node {
    link::root(
        REPO,
        &LinkProps::default(),
        vec![("data-blocks-header-simple-bar-logo", "")],
        vec![
            geo_icon(Size::Md, "M4 4h16v6H4zM4 14h16v6H4z"),
            span(
                vec![("class", "blocks-header-simple-bar-brand")],
                vec![text("Fandhe Frontend")],
            ),
        ],
    )
}

/// メインナビ本体（`aria-label` は variant ごとに一意にし、同一ページ内で
/// ナビランドマーク名が重ならないようにする）。
fn nav(aria_label: &str) -> Node {
    let props = NavigationMenuProps::default();
    navigation_menu::root(
        &props,
        aria_label,
        vec![("data-blocks-header-simple-bar-nav", "")],
        vec![navigation_menu::list(
            &props,
            vec![],
            NAV_ITEMS
                .iter()
                .map(|(value, label, href)| {
                    navigation_menu::item(
                        OpenState::Closed,
                        false,
                        &props,
                        value,
                        vec![],
                        vec![navigation_menu::link(
                            href,
                            false,
                            vec![],
                            vec![text(*label)],
                        )],
                    )
                })
                .collect(),
        )],
    )
}

/// アクション行（ログイン link + 登録 button の 0〜2 個を任意組み合わせで
/// 持つ）。登録ボタンは無 JS デモのため `disabled: true` 固定。
fn actions(with_login: bool, with_signup: bool) -> Node {
    let mut children = Vec::new();
    if with_login {
        children.push(link::root(
            REPO,
            &LinkProps::default(),
            vec![],
            vec![text("ログイン")],
        ));
    }
    if with_signup {
        children.push(button::button(
            &ButtonProps {
                disabled: true,
                ..ButtonProps::default()
            },
            vec![("data-blocks-header-simple-bar-cta", "")],
            vec![text("登録")],
        ));
    }
    div(
        vec![("data-blocks-header-simple-bar-actions", "")],
        children,
    )
}

/// ハンバーガートリガー（狭い幅専用、常時展開の [`mobile_panel`] を
/// `aria-controls` で指す。押しても何も起きないため `disabled: true` 固定
/// だが、`OpenState::Open` によりパネル自体は常に到達可能）。
fn hamburger(panel_id: &str) -> Node {
    collapsible::trigger(
        OpenState::Open,
        true,
        Some(panel_id),
        vec![
            ("aria-label", "Open main menu"),
            ("data-blocks-header-simple-bar-toggle", ""),
        ],
        vec![hamburger_icon()],
    )
}

/// 常時展開のドロップダウンパネル（狭い幅専用、[`bar`] の呼び出し元が
/// デスクトップ用ナビ・アクションを `Node::clone()` して渡す）。
fn mobile_panel(panel_id: &str, nav_node: Node, actions_node: Node) -> Node {
    collapsible::content(
        OpenState::Open,
        true,
        Some(panel_id),
        vec![("data-blocks-header-simple-bar-panel", "")],
        vec![nav_node, actions_node],
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &str) -> Node {
    p(
        vec![("class", "blocks-header-simple-bar-caption")],
        vec![text(label)],
    )
}

/// 1 本のバー（DOM 順は variant ごとに視覚順と一致させる。`logo-center`
/// のみナビ・ロゴ・アクションの順、他 3 variant はロゴ・ナビ・アクション
/// の順。見た目の配置差は [`LAYOUT_CSS`] の
/// `data-blocks-header-simple-bar-variant` セレクタが担う）。デスクトップ
/// 用ナビ・アクションは専用ラッパー div（`-nav-wrap`/`-actions-wrap`）で
/// 包み、狭い幅では [`mobile_panel`]（同じノードの clone）へ表示を譲る。
fn bar(variant: &str, aria_label: &str, with_login: bool, with_signup: bool) -> Node {
    let panel_id = format!("hsb-panel-{variant}");
    let nav_node = nav(aria_label);
    let actions_node = actions(with_login, with_signup);
    let nav_wrap = div(
        vec![("data-blocks-header-simple-bar-nav-wrap", "")],
        vec![nav_node.clone()],
    );
    let actions_wrap = div(
        vec![("data-blocks-header-simple-bar-actions-wrap", "")],
        vec![actions_node.clone()],
    );
    let main_children = if variant == "logo-center" {
        vec![nav_wrap, logo(), actions_wrap, hamburger(&panel_id)]
    } else {
        vec![logo(), nav_wrap, actions_wrap, hamburger(&panel_id)]
    };
    div(
        vec![("data-blocks-header-simple-bar-block", "")],
        vec![
            header(
                vec![
                    ("class", "blocks-header-simple-bar-layout"),
                    ("data-blocks-header-simple-bar-root", ""),
                    ("data-blocks-header-simple-bar-variant", variant),
                ],
                main_children,
            ),
            mobile_panel(&panel_id, nav_node, actions_node),
        ],
    )
}

/// `header-simple-bar` の Demo 本体。4 variant を縦に並記する純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-header-simple-bar-stack")],
        vec![
            caption("中央寄せ・幅制限"),
            bar("center", "メイン（中央寄せ）", true, true),
            caption("左寄せ・全幅・下境界線"),
            bar("start", "メイン（左寄せ）", true, true),
            caption("右寄せ・登録のみ"),
            bar("end", "メイン（右寄せ）", false, true),
            caption("ロゴ中央・ログインのみ"),
            bar("logo-center", "メイン（ロゴ中央）", true, false),
        ],
    )
}
```

## 集約元との差分メモ

- 集約元 12 件を 4 variant（中央寄せ/左寄せ/右寄せ/ロゴ中央）へ集約しました。
  中央寄せ形が R0982（主参照）、左寄せ形が R0573/R0989/R0985/R0159、
  右寄せ形が R0571/R0990、ロゴ中央形が R0991 に対応します。
- R0572/R0986（CTA 1 個 + 下境界線・リンク中央寄せ等）は左寄せ形・中央寄せ形の
  装飾差分として扱い、独立したインスタンスにはしていません。
- R0160（ナビ内ボタン）・R0983（ブランド色背景）は本 Demo では扱いません。
  ナビ項目はすべてテキストリンクとし、背景色はテーマの既定トークンの
  ままにしています。
- 押しても何も起きないボタン（登録ボタン・ハンバーガートリガー）はすべて
  `disabled` にして固定し、フォーカス・操作不能であることを明示しています。
  ハンバーガートリガーは常時展開のドロップダウンパネルへ `aria-controls`
  で関連付けられており、狭い幅でもナビ・アクションへ到達できます。
- 文言・アイコンはすべて独自に書いた架空のものです。配色・余白・角丸は
  既存のテーマトークンに従っています。

関連情報: [Navigation Menu](../themes/navigation-menu.md) /
[Button](../themes/button.md) / [Icon](../themes/icon.md) /
[Link](../themes/link.md) / [Collapsible](../themes/collapsible.md)
