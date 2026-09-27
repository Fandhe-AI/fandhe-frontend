//! `app-shell-navbar-columns` block（イシュー #2893。親トラッキング #2892
//! 「Blocks アプリケーション A（phase:3）」配下、対応表 ID R1080 を主参照
//! とする合成例。集約元 R1081（左右カラム sticky 版）・R0134（3 セルの
//! ナビバー＋2 カラム骨格）・R0139（2 カラム＋フッター骨格）の差分は
//! `site/blocks/app-shell-navbar-columns.md` の「集約元との差分メモ」節に
//! 記す）。App Shell カテゴリ最初の block（イシュー #2734 の空雛形からの
//! 卒業、`super` の `mod.rs` 参照）。
//!
//! # 使用部品
//!
//! `navigation-menu`（主要リンク）/ `avatar`（プロフィールボタン内の
//! アイコン）/ `button`（通知・プロフィールの 2 個、いずれも無 JS のため
//! `disabled: true` 固定）/ `icon`（自作の幾何図形のみ、装飾用途）/
//! `separator`（通知とプロフィールの間の縦区切り、フッターの横区切り）/
//! `visually-hidden`（プロフィールボタンのアクセシブルネーム）の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、`blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。ロゴ・左右カラムの補助リストは
//! `link`/`card` を使わず素の `div`/`ul`/`li`/`p`（`href="#"`・クリック
//! ターゲットを持たないダミー表示のため部品化しない）。
//!
//! # 2 variant の並記（左右カラム sticky 版・2 列+フッター版）
//!
//! 無 JS のため開閉・レスポンシブ切り替えの実挙動を動的には示せない
//! （`crate::blocks` モジュール doc の一般方針）。同じ骨格（[`shell`]）を
//! 使う 2 variant を縦に並記して、集約元の差分をページ内で読み取れるように
//! する（`header_simple_bar`/`sidebar_07` と同型の判断）。
//!
//! | variant | 構成 | 対応 ID |
//! |---|---|---|
//! | `three-column` | 3 セルのナビバー + 左ナビカラム(sticky) / メイン / 右補助カラム(sticky) | R1080（代表）+ R1081（sticky） |
//! | `two-column-footer` | 3 セルのナビバー + メイン / 右補助カラム + 区切り線付きフッター | R0134（3 セルナビ+2 カラム）+ R0139（2 カラム+フッター） |
//!
//! # DOM 順はメイン → 左カラム → 右カラム（`order` を使わない、狭幅の
//! 読み上げ順・Tab 順を意味の通る順に保つ）
//!
//! 狭い幅では「補助カラムをメインの下に回す」仕様だが、これを CSS
//! `order` で実現すると視覚順と DOM 順（キーボード操作の Tab 順・
//! スクリーンリーダーの読み上げ順、WCAG 1.3.2 Meaningful Sequence）が
//! 食い違う。本実装は DOM 順自体をメイン → 左 → 右に固定し、広い幅での
//! 左右配置は [`LAYOUT_CSS`] の `grid-template-areas`（`order` を使わない
//! 明示的なグリッド配置）のみで行う。狭い幅では 1 列の縦積みとなり
//! DOM 順がそのまま視覚順になる。
//!
//! # ナビバーが `order` なしで狭幅に対応する仕組み
//!
//! ナビバーの DOM 順はロゴ → ナビ → アクションで固定する。狭い幅では
//! ナビ（`navigation-menu` を包む `data-blocks-app-shell-navbar-columns-nav`
//! ラッパー）へ `flex-basis: 100%` を与えて強制的に単独行へ折り返させる
//! （`flex-wrap: wrap` の自然な折り返しであり `order` は使わない）。広い幅
//! （`@container blocks-app-shell-navbar-columns (min-width: 48rem)`）では
//! `flex-basis: auto` に戻し、ロゴ・ナビ・アクションが 1 行に収まる。
//! アクションは常に `margin-inline-start: auto` で自身の行の末尾へ寄せる。
//!
//! # sticky を見せるための固定高スクロールコンテナ
//!
//! `footer_sticky_reveal`/`sidebar_07` と同じ判断で、各 shell ルート
//! （`data-blocks-app-shell-navbar-columns-root`）自体を固定高
//! （`max-block-size`）+ `overflow-y: auto` のスクロールコンテナにし、枠内で
//! ナビバー・左右カラムの sticky を再現する。キーボードでもスクロール
//! できるよう `tabindex="0"` + `role="region"` + variant ごとに一意な
//! `aria-label` を付与する。
//!
//! # 幅の判定はビューポートではなく Demo 枠の幅（`@container`）
//!
//! `header_flyout_menu` の先例に倣い、幅による列数・sticky 切り替えは
//! ビューポート幅ではなく Demo 枠の幅を基準にする。`container-type:
//! inline-size` は両 variant を包む `.blocks-app-shell-navbar-columns-stack`
//! （両 variant は常に同じ Demo 枠幅を共有するため、shell ルート個々では
//! なく共有ラッパーへ 1 回だけ宣言すれば足りる）へ付与し、`@container`
//! クエリは `data-blocks-app-shell-navbar-columns-variant` 属性との複合
//! セレクタで variant ごとに異なるグリッド構成を切り替える。
//!
//! # disabled ボタンの中和（`drop_class_attr` の契約）
//!
//! `button::button`/`button::icon_button`/`avatar::root`/`separator::separator`/
//! `visually_hidden::root`/`navigation_menu::*` はいずれも呼び出し側
//! `attrs` の `class` を `drop_class_attr` により黙って除去する（
//! `crate::blocks` モジュール doc「CSS フックが `class` と `[data-*]` で
//! 混在する理由」節参照）。本 block は統一して `data-blocks-app-shell-
//! navbar-columns-*` 属性で Demo 固有スタイルを渡す。通知・プロフィールの
//! 2 ボタンは無 JS デモのため `disabled: true`（ネイティブ `disabled` +
//! `aria-disabled`）にして操作不能を明示し、[`LAYOUT_CSS`] で
//! `opacity: 1; cursor: default;` に中和して通常と同じ見た目に保つ
//! （`header_simple_bar` と同じ判断）。styled recipe の `[data-scope]
//! [data-part]` 基底宣言・disabled state 宣言に単一属性セレクタでは詳細度
//! で負けるため、複合セレクタで上書きする。
//!
//! # `href` の方針・`<form>` を持たない・全データが架空
//!
//! ナビの `href` は実在する自リポジトリ・組織の URL に限る（`href="#"` は
//! `linkcheck` が拒否する死リンクのため使わない）。左カラムの補助ナビ・
//! 右カラムの補助情報はリンクを持たない静的なダミー表示（`href` 自体を
//! 持たないため `linkcheck` の対象外）。ブランド名・ユーザー名・本文は
//! すべて架空のもの。`crate::blocks` モジュール doc の不変条件どおり
//! `<form>` は出力しない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/app-shell-navbar-columns/",
    title: "app-shell-navbar-columns",
    category: BlockCategory::AppShell,
    rust_source: "crates/docs-site/src/blocks/application/app_shell/app_shell_navbar_columns.rs",
    demo_class: "blocks-app-shell-navbar-columns",
    parts: &[
        Part {
            label: "Navigation Menu",
            path: "/themes/navigation-menu/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
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
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `app_shell_navbar_columns` 固有のレイアウト規則（`--fandhe-*` トークン
/// のみ使用）。セレクタは `.blocks-app-shell-navbar-columns-*` と
/// `[data-blocks-app-shell-navbar-columns-*]`、および styled button の
/// `[data-scope]`/`[data-part]` セレクタとの複合セレクタのみを用い、他
/// block や部品の素のセレクタへは影響させない。`order` は使わない
/// （モジュール doc「DOM 順」「ナビバーが `order` なしで狭幅に対応する
/// 仕組み」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-app-shell-navbar-columns-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-app-shell-navbar-columns;\n}\n\
.blocks-app-shell-navbar-columns-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-app-shell-navbar-columns-root] {\n  --blocks-app-shell-navbar-columns-bar-h: 3.5rem;\n  display: flex;\n  flex-direction: column;\n  max-block-size: 32rem;\n  overflow-y: auto;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: 0.5rem;\n  background: var(--fandhe-color-bg);\n}\n\
[data-blocks-app-shell-navbar-columns-bar] {\n  position: sticky;\n  top: 0;\n  z-index: var(--fandhe-z-index-docked);\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  min-block-size: var(--blocks-app-shell-navbar-columns-bar-h);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  background: var(--fandhe-color-bg);\n  border-block-end: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-app-shell-navbar-columns-logo] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  white-space: nowrap;\n}\n\
[data-blocks-app-shell-navbar-columns-nav] {\n  flex: 1 1 100%;\n}\n\
[data-blocks-app-shell-navbar-columns-actions] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  margin-inline-start: auto;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-app-shell-navbar-columns-notify][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-app-shell-navbar-columns-profile] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-app-shell-navbar-columns-profile][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-blocks-app-shell-navbar-columns-body] {\n  display: grid;\n  gap: var(--fandhe-space-6);\n  grid-template-areas: \"main\" \"left\" \"right\";\n  max-inline-size: 72rem;\n  inline-size: 100%;\n  margin-inline: auto;\n  padding: var(--fandhe-space-6) var(--fandhe-space-4);\n}\n\
[data-blocks-app-shell-navbar-columns-main] {\n  grid-area: main;\n  min-inline-size: 0;\n}\n\
[data-blocks-app-shell-navbar-columns-left] {\n  grid-area: left;\n}\n\
[data-blocks-app-shell-navbar-columns-right] {\n  grid-area: right;\n}\n\
.blocks-app-shell-navbar-columns-side-list {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-app-shell-navbar-columns-side-item {\n  padding: var(--fandhe-space-2) var(--fandhe-space-3);\n  border-radius: 0.375rem;\n  background: var(--fandhe-color-bg-subtle);\n}\n\
.blocks-app-shell-navbar-columns-row {\n  padding: var(--fandhe-space-3);\n  margin-block-end: var(--fandhe-space-3);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: 0.375rem;\n}\n\
.blocks-app-shell-navbar-columns-row-title {\n  margin: 0 0 var(--fandhe-space-1);\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
[data-blocks-app-shell-navbar-columns-footer] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  padding: var(--fandhe-space-4);\n  border-block-start: 1px solid var(--fandhe-color-border);\n}\n\
@container blocks-app-shell-navbar-columns (min-width: 48rem) {\n  \
[data-blocks-app-shell-navbar-columns-nav] {\n    flex: 0 1 auto;\n  }\n  \
[data-blocks-app-shell-navbar-columns-variant=\"three-column\"] [data-blocks-app-shell-navbar-columns-body] {\n    grid-template-columns: 12rem minmax(0, 1fr) 14rem;\n    grid-template-areas: \"left main right\";\n  }\n  \
[data-blocks-app-shell-navbar-columns-variant=\"two-column-footer\"] [data-blocks-app-shell-navbar-columns-body] {\n    grid-template-columns: minmax(0, 1fr) 14rem;\n    grid-template-areas: \"main right\";\n  }\n  \
[data-blocks-app-shell-navbar-columns-left],\n  [data-blocks-app-shell-navbar-columns-right] {\n    position: sticky;\n    top: var(--blocks-app-shell-navbar-columns-bar-h);\n    align-self: start;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 6 部品を持ち、非対話制約（`<form>` なし・
    /// `href="#"` なし・`data:` src なし）を満たすこと。
    #[test]
    fn demo_composes_expected_parts_and_has_no_form() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"navigation-menu\"",
            "data-scope=\"avatar\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
            "data-scope=\"separator\"",
            "data-scope=\"visually-hidden\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 2 variant がそれぞれ 1 回ずつ描画され、フッターは
    /// `two-column-footer` にだけ現れ、左カラムは `three-column` にだけ
    /// 現れること。
    #[test]
    fn demo_renders_both_variants_with_expected_columns() {
        let html = render(&demo());
        for variant in ["three-column", "two-column-footer"] {
            assert_eq!(
                html.matches(&format!(
                    "data-blocks-app-shell-navbar-columns-variant=\"{variant}\""
                ))
                .count(),
                1,
                "variant {variant} should render exactly once, html={html}"
            );
        }
        assert_eq!(
            html.matches("data-blocks-app-shell-navbar-columns-footer")
                .count(),
            1,
            "footer should render exactly once (two-column-footer variant only)"
        );
        assert_eq!(
            html.matches("data-blocks-app-shell-navbar-columns-left")
                .count(),
            1,
            "left column should render exactly once (three-column variant only)"
        );
        assert_eq!(
            html.matches("data-blocks-app-shell-navbar-columns-right")
                .count(),
            2,
            "right column should render once per variant"
        );
    }

    /// 本体の DOM 順がメイン → 左 → 右であること（`three-column`
    /// variant、モジュール doc「DOM 順」節参照）。
    #[test]
    fn body_dom_order_is_main_then_left_then_right() {
        let html = render(&demo());
        let main_pos = html
            .find("data-blocks-app-shell-navbar-columns-main")
            .expect("main should render");
        let left_pos = html[main_pos..]
            .find("data-blocks-app-shell-navbar-columns-left")
            .map(|rel| main_pos + rel)
            .expect("left should follow main in DOM");
        let right_pos = html[left_pos..]
            .find("data-blocks-app-shell-navbar-columns-right")
            .map(|rel| left_pos + rel)
            .expect("right should follow left in DOM");
        assert!(main_pos < left_pos && left_pos < right_pos);
    }

    /// 通知・プロフィールボタンが `disabled`/`aria-disabled` を持ち
    /// `type="button"` であること。プロフィールボタンに visually-hidden の
    /// ラベル文言が入っていること。
    #[test]
    fn notify_and_profile_buttons_are_disabled_with_accessible_name() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-app-shell-navbar-columns-notify")
                .count(),
            2,
            "notify button should render once per variant"
        );
        assert_eq!(
            html.matches("data-blocks-app-shell-navbar-columns-profile")
                .count(),
            2,
            "profile button should render once per variant"
        );
        assert_eq!(html.matches(r#"aria-label="通知を表示""#).count(), 2);
        assert_eq!(
            html.matches("アカウントメニューを開く").count(),
            2,
            "profile button should carry the visually-hidden label"
        );
        let attr_start = html
            .find("data-blocks-app-shell-navbar-columns-profile")
            .expect("profile button should render");
        let tag_start = html[..attr_start]
            .rfind("<button")
            .expect("profile opening tag should precede its attribute");
        let tag_end = html[tag_start..]
            .find('>')
            .map(|rel| tag_start + rel)
            .expect("profile opening tag should close");
        let profile_tag = &html[tag_start..tag_end];
        assert!(
            profile_tag.contains("disabled"),
            "profile_tag={profile_tag}"
        );
        assert!(
            profile_tag.contains(r#"aria-disabled="true""#),
            "profile_tag={profile_tag}"
        );
    }

    /// スクロールコンテナのルートに `tabindex="0"`・`role="region"`・
    /// 一意な `aria-label` があり、nav/aside の `aria-label` が variant 間で
    /// 重複しないこと。
    #[test]
    fn root_is_keyboard_scrollable_region_with_unique_labels() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-blocks-app-shell-navbar-columns-root"#)
                .count(),
            2
        );
        assert_eq!(html.matches(r#"tabindex="0""#).count(), 2);
        assert_eq!(html.matches(r#"role="region""#).count(), 2);
        for label in [
            "アプリシェル（3 列レイアウト）",
            "アプリシェル（2 列 + フッター）",
            "アプリ（3 列）",
            "アプリ（2 列）",
            "メインコンテンツ（3 列）",
            "メインコンテンツ（2 列）",
            "サイドナビゲーション",
            "補足情報（3 列）",
            "補足情報（2 列）",
        ] {
            assert_eq!(
                html.matches(&format!("aria-label=\"{label}\"")).count(),
                1,
                "label={label} should be unique, html={html}"
            );
        }
    }

    /// [`LAYOUT_CSS`] が 48rem 境界・sticky・grid-template-areas・disabled
    /// 中和の各規則を持ち、`order` を使わないこと（モジュール doc「DOM
    /// 順」「ナビバーが `order` なしで狭幅に対応する仕組み」節参照）。
    #[test]
    fn layout_css_has_responsive_rules_without_order() {
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(
            LAYOUT_CSS.contains("@container blocks-app-shell-navbar-columns (min-width: 48rem)")
        );
        assert!(LAYOUT_CSS.contains("position: sticky;"));
        assert!(LAYOUT_CSS.contains("grid-template-areas: \"left main right\";"));
        assert!(LAYOUT_CSS.contains("grid-template-areas: \"main right\";"));
        assert!(LAYOUT_CSS.contains("opacity: 1;"));
        assert!(!LAYOUT_CSS.contains("{\n  order:"));
        assert!(!LAYOUT_CSS.contains(";\n  order:"));
    }
}
