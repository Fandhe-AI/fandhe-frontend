# app-shell-navbar-columns

上部に固定表示のナビバー（ロゴ・主要リンク・通知・プロフィール）を持ち、
その下に最大幅で中央寄せした 2〜3 列の本体を配置するアプリシェルの合成例
です。左右の補助カラムはスクロールしてもナビバーの下端に貼り付いたまま
（sticky）表示され、メインカラムだけが独立してスクロールします。狭い幅
では 1 列に積み、補助カラムはメインカラムの下に回ります。新しい UI 部品
は作らず、既存部品（navigation-menu/avatar/button/icon/separator/
visually-hidden）のみで構成しています。無 JS の静的な表示で `<form>` は
出力しません。

主参照は対応表 ID R1080、集約元は対応表 ID R1081・R0134・R0139 の 3 件
です（出典の固有名・ファイル名は記載しません）。ブランド名・ユーザー名・
本文はすべて架空のサンプルです。

## Rust コード

```rust
use fandhe_frontend_core::{aside, div, el, footer, header, li, p, section, span, text, ul, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::{Orientation, Size};

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc 参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// 実在の自組織 URL。
const ORG: &str = "https://github.com/Fandhe-AI";

/// メインナビの項目一覧（value, label, href）。両 variant で共有する
/// （`header_simple_bar::NAV_ITEMS` と同型、別々の `navigation-menu` root
/// インスタンスへ渡すため value の重複は実害を持たない）。
const NAV_ITEMS: &[(&str, &str, &str)] = &[
    ("dashboard", "ダッシュボード", REPO),
    ("projects", "プロジェクト", REPO),
    ("reports", "レポート", REPO),
    ("settings", "設定", ORG),
];

/// 左カラムのダミー補助ナビ項目（架空、静的表示のみ）。
const SIDE_NAV_ITEMS: &[&str] = &["概要", "アクティビティ", "メンバー", "アーカイブ"];

/// メイン本文のダミー行（架空、スクロールを見せるための分量確保）。
const CONTENT_ROWS: &[&str] = &[
    "四半期の売上サマリを更新しました。",
    "新規メンバーが 2 名参加しました。",
    "レポート #128 がレビュー待ちです。",
    "バックアップジョブが正常に完了しました。",
    "ストレージ使用量が 68% に達しました。",
    "週次ダイジェストを送信しました。",
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

/// ベル（通知）の幾何図形アイコン。
fn bell_icon() -> Node {
    geo_icon(
        Size::Sm,
        "M12 3a5 5 0 0 0-5 5v3l-2 4h14l-2-4V8a5 5 0 0 0-5-5zM10 18a2 2 0 0 0 4 0h-4z",
    )
}

/// ロゴ（幾何図形アイコン + ブランド名テキスト、リンクにしない。
/// モジュール doc「使用部品」節参照）。
fn logo() -> Node {
    div(
        vec![("data-blocks-app-shell-navbar-columns-logo", "")],
        vec![
            geo_icon(Size::Md, "M4 4h16v6H4zM4 14h16v6H4z"),
            span(vec![], vec![text("Fandhe Frontend")]),
        ],
    )
}

/// メインナビ本体（`aria-label` は variant ごとに一意にする）。
fn nav(aria_label: &str) -> Node {
    let props = NavigationMenuProps::default();
    navigation_menu::root(
        &props,
        aria_label,
        vec![("data-blocks-app-shell-navbar-columns-nav-list", "")],
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

/// 通知ボタン（無 JS デモのため `disabled: true` 固定）。
fn notify_button() -> Node {
    button::icon_button(
        &ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        },
        "通知を表示",
        vec![("data-blocks-app-shell-navbar-columns-notify", "")],
        vec![bell_icon()],
    )
}

/// プロフィールボタン（avatar フォールバック + 名前 + visually-hidden の
/// アクセシブルネーム、無 JS デモのため `disabled: true` 固定）。
fn profile_button() -> Node {
    button::button(
        &ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        },
        vec![("data-blocks-app-shell-navbar-columns-profile", "")],
        vec![
            avatar::root(
                &AvatarProps::default(),
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text("AL")],
                )],
            ),
            visually_hidden::root(vec![], vec![text("アカウントメニューを開く")]),
        ],
    )
}

/// アクション行（縦区切り + 通知 + プロフィール）。
fn actions() -> Node {
    div(
        vec![("data-blocks-app-shell-navbar-columns-actions", "")],
        vec![
            notify_button(),
            separator::separator(
                &SeparatorProps {
                    orientation: Orientation::Vertical,
                    ..SeparatorProps::default()
                },
                vec![],
            ),
            profile_button(),
        ],
    )
}

/// ナビバー本体（DOM 順: ロゴ → ナビ → アクション、`order` を使わず
/// 狭幅では [`LAYOUT_CSS`] のナビラッパー `flex-basis: 100%` のみで折返す。
/// モジュール doc「ナビバーが `order` なしで狭幅に対応する仕組み」参照）。
fn navbar(aria_label: &str) -> Node {
    header(
        vec![("data-blocks-app-shell-navbar-columns-bar", "")],
        vec![
            logo(),
            div(
                vec![("data-blocks-app-shell-navbar-columns-nav", "")],
                vec![nav(aria_label)],
            ),
            actions(),
        ],
    )
}

/// 左カラム（サイドナビゲーション、静的なダミーリスト。`three-column`
/// variant のみが持つ）。
fn left_column(aria_label: &str) -> Node {
    let items: Vec<Node> = SIDE_NAV_ITEMS
        .iter()
        .map(|label| {
            li(
                vec![("class", "blocks-app-shell-navbar-columns-side-item")],
                vec![text(*label)],
            )
        })
        .collect();
    aside(
        vec![
            ("aria-label", aria_label),
            ("data-blocks-app-shell-navbar-columns-left", ""),
        ],
        vec![ul(
            vec![("class", "blocks-app-shell-navbar-columns-side-list")],
            items,
        )],
    )
}

/// 右カラム（補足情報、静的なダミーカード）。
fn right_column(aria_label: &str) -> Node {
    aside(
        vec![
            ("aria-label", aria_label),
            ("data-blocks-app-shell-navbar-columns-right", ""),
        ],
        vec![div(
            vec![("class", "blocks-app-shell-navbar-columns-row")],
            vec![
                p(
                    vec![("class", "blocks-app-shell-navbar-columns-row-title")],
                    vec![text("ストレージ使用量")],
                ),
                p(vec![], vec![text("68% 使用中（100GB 中 68GB）")]),
            ],
        )],
    )
}

/// メインカラム（スクロールを見せるためのダミー行群）。
fn main_column(aria_label: &str) -> Node {
    let rows: Vec<Node> = CONTENT_ROWS
        .iter()
        .map(|row| {
            div(
                vec![("class", "blocks-app-shell-navbar-columns-row")],
                vec![p(vec![], vec![text(*row)])],
            )
        })
        .collect();
    section(
        vec![
            ("tabindex", "0"),
            ("role", "region"),
            ("aria-label", aria_label),
            ("data-blocks-app-shell-navbar-columns-main", ""),
        ],
        rows,
    )
}

/// フッター（`two-column-footer` variant のみが持つ、区切り線 + 架空の
/// コピーライト表記）。
fn shell_footer() -> Node {
    footer(
        vec![("data-blocks-app-shell-navbar-columns-footer", "")],
        vec![
            separator::separator(&SeparatorProps::default(), vec![]),
            p(vec![], vec![text("© Fandhe Frontend Demo")]),
        ],
    )
}

/// 1 本の shell（モジュール doc「2 variant の並記」節参照）。DOM 順は
/// メイン → 左（あれば）→ 右（モジュール doc「DOM 順」節参照）。
fn shell(
    variant: &str,
    root_label: &str,
    nav_label: &str,
    main_label: &str,
    left_label: Option<&str>,
    right_label: &str,
    with_footer: bool,
) -> Node {
    let mut body_children = vec![main_column(main_label)];
    if let Some(left_label) = left_label {
        body_children.push(left_column(left_label));
    }
    body_children.push(right_column(right_label));

    let mut children = vec![
        navbar(nav_label),
        div(
            vec![("data-blocks-app-shell-navbar-columns-body", "")],
            body_children,
        ),
    ];
    if with_footer {
        children.push(shell_footer());
    }

    div(
        vec![
            ("tabindex", "0"),
            ("role", "region"),
            ("aria-label", root_label),
            ("data-blocks-app-shell-navbar-columns-root", ""),
            ("data-blocks-app-shell-navbar-columns-variant", variant),
        ],
        children,
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &str) -> Node {
    p(
        vec![("class", "blocks-app-shell-navbar-columns-caption")],
        vec![text(label)],
    )
}

/// `app-shell-navbar-columns` の Demo 本体。2 variant を縦に並記する
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-app-shell-navbar-columns-stack")],
        vec![
            caption("3 列（左右カラム sticky）"),
            shell(
                "three-column",
                "アプリシェル（3 列レイアウト）",
                "アプリ（3 列）",
                "メインコンテンツ（3 列）",
                Some("サイドナビゲーション"),
                "補足情報（3 列）",
                false,
            ),
            caption("2 列 + フッター"),
            shell(
                "two-column-footer",
                "アプリシェル（2 列 + フッター）",
                "アプリ（2 列）",
                "メインコンテンツ（2 列）",
                None,
                "補足情報（2 列）",
                true,
            ),
        ],
    )
}
```

## 集約元との差分メモ

- 集約元 3 件を 2 variant（左右カラム sticky/2 列 + フッター）へ集約
  しました。左右カラム sticky 形が R1080（主参照）・R1081（sticky）、
  2 列 + フッター形が R0134（3 セルのナビバー + 2 カラム骨格）・R0139
  （2 カラム + フッター骨格）に対応します。
- 集約元の配色・文言・アイコン・ハンバーガーメニュー・ドロップダウン
  開閉は持ち込んでいません。ナビバーは常に主要リンクが並んだ状態を表示し、
  狭い幅でも折り返しのみでリンクへ到達できます。
- 通知・プロフィールの 2 ボタンは押しても何も起きないため、いずれも
  `disabled` にして固定し、フォーカス・操作不能であることを明示して
  います。
- sticky・列切り替えの判定はビューポート幅ではなく Demo 枠自体の幅
  （コンテナクエリ）を基準にしています。実際にスクロールして sticky を
  確認するには、Demo 枠の幅を 48rem（768px 相当）以上に広げてください。
- 文言・アイコン・ユーザー名はすべて独自に書いた架空のものです。配色・
  余白・角丸は既存のテーマトークンに従っています。

関連情報: [Navigation Menu](../themes/navigation-menu.md) /
[Avatar](../themes/avatar.md) / [Button](../themes/button.md) /
[Icon](../themes/icon.md) / [Separator](../themes/separator.md) /
[Visually Hidden](../themes/visually-hidden.md)
