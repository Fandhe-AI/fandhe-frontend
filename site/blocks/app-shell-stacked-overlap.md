# app-shell-stacked-overlap

濃色のヒーロー帯（ナビバー + ページ見出し）の下端へ本文カードが重なる
アプリシェルの合成例です。帯にはロゴ・主要ナビ・通知・プロフィールを
まとめ、狭い幅ではメニューボタンで開く常時展開パネルへナビを畳みます。
新しい UI 部品は作らず、既存部品（navigation-menu/input-group/input/
avatar/menu/button/card/heading/collapsible/icon）のみで構成しています。
無 JS の静的な表示で `<form>` は出力しません。

主参照は対応表 ID R1278（代表構成）、集約元は対応表 ID R1279（帯の配色
だけがブランド色になる版）・R1282（ナビが 2 段になり 2 行目に検索欄と
主要リンクが入る版）です（出典の固有名・ファイル名は記載しません）。
ブランド名・ユーザー名・本文はすべて架空のサンプルです。

## Rust コード

```rust
use fandhe_frontend_core::{a, div, el, header, main_tag, nav as nav_el, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::collapsible;
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState as MenuOpenState};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc 参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// 実在の自組織 URL。
const ORG: &str = "https://github.com/Fandhe-AI";

/// メインナビの項目一覧（value, label, href）。両 variant で共有する。
const NAV_ITEMS: &[(&str, &str, &str)] = &[
    ("overview", "概要", REPO),
    ("projects", "プロジェクト", REPO),
    ("reports", "レポート", REPO),
    ("team", "メンバー", ORG),
];

/// 2 段版の 2 行目（クイックリンク、静的な素の `<a>`）。
const QUICK_LINKS: &[(&str, &str)] = &[("最近の更新", REPO), ("ヘルプ", ORG)];

/// 重なるカードのダミー本文行。
const CARD_ROWS: &[(&str, &str)] = &[
    ("今月の進捗", "先週比 12% 増。目標まで残り 3 タスクです。"),
    ("次のマイルストーン", "10 月 3 日にレビュー予定です。"),
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

/// ハンバーガー（3 本線）アイコン。
fn hamburger_icon() -> Node {
    geo_icon(Size::Md, "M3 6h18v2H3zM3 11h18v2H3zM3 16h18v2H3z")
}

/// 検索（虫眼鏡）アイコン。
fn search_icon() -> Node {
    geo_icon(
        Size::Sm,
        "M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zm9 17-5.2-5.2",
    )
}

/// ロゴ（幾何図形アイコン + ブランド名テキスト、リンクにしない。
/// `app_shell_navbar_columns::logo` と同型）。
fn logo() -> Node {
    div(
        vec![("data-blocks-app-shell-stacked-overlap-logo", "")],
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
        vec![],
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

/// 2 段版の 2 行目クイックリンク（部品化しない静的なダミーナビ、`href="#"`
/// は使わない）。
fn quick_links(aria_label: &str) -> Node {
    nav_el(
        vec![("aria-label", aria_label)],
        vec![div(
            vec![("data-blocks-app-shell-stacked-overlap-quick-links", "")],
            QUICK_LINKS
                .iter()
                .map(|(label, href)| a(vec![("href", href)], vec![text(*label)]))
                .collect(),
        )],
    )
}

/// 検索欄（可視ラベルの代わりに `visually_hidden` + `<label for>`、
/// `hero_search::search_group` と同型）。送信ボタンは置かない（`<form>`
/// を持たないデモのため）。
fn search_field(suffix: &str) -> Node {
    let field_id = format!("blocks-app-shell-stacked-overlap-search-{suffix}");
    let query_field = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &query_field,
        vec![("data-blocks-app-shell-stacked-overlap-search-field", "")],
        vec![
            visually_hidden::root(
                vec![],
                vec![field::label(
                    &query_field,
                    vec![],
                    vec![text("サイト内検索")],
                )],
            ),
            input_group::root(
                &group_props,
                vec![],
                vec![
                    input_group::addon(
                        InputGroupAlign::InlineStart,
                        &group_props,
                        vec![],
                        vec![search_icon()],
                    ),
                    input::input(
                        &InputProps::default(),
                        &query_field,
                        vec![
                            ("type", "search"),
                            ("autocomplete", "off"),
                            ("placeholder", "検索"),
                        ],
                    ),
                ],
            ),
        ],
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
        vec![("data-blocks-app-shell-stacked-overlap-notify", "")],
        vec![bell_icon()],
    )
}

/// プロフィールメニュー（閉じた `menu`、avatar フォールバック +
/// visually-hidden のアクセシブルネーム。`suffix` で variant 間の id 衝突を
/// 避ける。`sidebar_07::team_switcher`/`user_menu` と同型）。
fn profile_menu(suffix: &str) -> Node {
    let content_id = format!("blocks-app-shell-stacked-overlap-profile-{suffix}");
    let trigger = menu::trigger(
        MenuOpenState::Closed,
        true,
        Some(content_id.as_str()),
        vec![("data-blocks-app-shell-stacked-overlap-profile", "")],
        vec![
            avatar::root(
                &AvatarProps::default(),
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text("YT")],
                )],
            ),
            visually_hidden::root(vec![], vec![text("アカウントメニューを開く")]),
        ],
    );
    let content = menu::content(
        MenuOpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item("profile", false, false, vec![], vec![text("プロフィール")]),
            menu::item("settings", false, false, vec![], vec![text("設定")]),
            menu::separator(vec![], vec![]),
            menu::item("logout", false, false, vec![], vec![text("ログアウト")]),
        ],
    );
    let positioner = menu::positioner(MenuOpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        MenuOpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// アクション行（通知 + プロフィールメニュー）。
fn actions(suffix: &str) -> Node {
    div(
        vec![("data-blocks-app-shell-stacked-overlap-actions", "")],
        vec![notify_button(), profile_menu(suffix)],
    )
}

/// ハンバーガートリガー（狭い幅専用、常時展開の [`mobile_panel`] を
/// `aria-controls` で指す。`header_simple_bar::hamburger` と同型）。
fn hamburger(panel_id: &str) -> Node {
    collapsible::trigger(
        OpenState::Open,
        true,
        Some(panel_id),
        vec![
            ("aria-label", "メニューを開く"),
            ("data-blocks-app-shell-stacked-overlap-toggle", ""),
        ],
        vec![hamburger_icon()],
    )
}

/// 常時展開のドロップダウンパネル（狭い幅専用、[`shell`] の呼び出し元が
/// ナビ（clone）と、`id` 重複を避けて別 suffix で組み直したアクション・
/// 検索欄・行 2 を渡す）。
fn mobile_panel(panel_id: &str, children: Vec<Node>) -> Node {
    collapsible::content(
        OpenState::Open,
        true,
        Some(panel_id),
        vec![("data-blocks-app-shell-stacked-overlap-panel", "")],
        children,
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &str) -> Node {
    p(
        vec![("class", "blocks-app-shell-stacked-overlap-caption")],
        vec![text(label)],
    )
}

/// 重なる本文カード群。
fn body_cards() -> Node {
    div(
        vec![("data-blocks-app-shell-stacked-overlap-body", "")],
        CARD_ROWS
            .iter()
            .map(|(title, body)| {
                card::root(
                    CardProps::default(),
                    vec![],
                    vec![
                        card::header(vec![], vec![card::title(vec![], vec![text(*title)])]),
                        card::body(vec![], vec![p(vec![], vec![text(*body)])]),
                    ],
                )
            })
            .collect(),
    )
}

/// 1 本の shell（モジュール doc「2 variant の並記」節参照）。
fn shell(
    variant: &str,
    root_label: &str,
    nav_label: &str,
    quick_links_label: Option<&str>,
    page_title: &str,
    page_summary: &str,
) -> Node {
    let panel_suffix = format!("{variant}-panel");
    let panel_id = format!("blocks-app-shell-stacked-overlap-panel-{variant}");
    // nav・クイックリンクは `id` を持たないため clone で共有できるが、
    // プロフィールメニュー（`menu::content` の `id`）・検索欄（`field` の
    // `id`）はそれぞれ固有 `id` を出力するため、パネル側は clone せず
    // 別 suffix（`panel_suffix`）で独立して組み立てる（重複 `id` の禁止、
    // `blocks_contract.rs::demo_output_has_no_dangling_aria_references_or_duplicate_ids`
    // 参照）。
    let nav_node = nav(nav_label);
    let nav_wrap = div(
        vec![("data-blocks-app-shell-stacked-overlap-nav-wrap", "")],
        vec![nav_node.clone()],
    );
    let actions_wrap = div(
        vec![("data-blocks-app-shell-stacked-overlap-actions-wrap", "")],
        vec![actions(variant)],
    );

    let mut panel_children = vec![nav_node, actions(&panel_suffix)];

    let bar = header(
        vec![("data-blocks-app-shell-stacked-overlap-bar", "")],
        vec![logo(), nav_wrap, actions_wrap, hamburger(&panel_id)],
    );

    let mut band_children = vec![bar];

    if let Some(quick_links_label) = quick_links_label {
        let quick_links_node = quick_links(quick_links_label);
        let row2_wrap = div(
            vec![("data-blocks-app-shell-stacked-overlap-row2", "")],
            vec![search_field(variant), quick_links_node.clone()],
        );
        band_children.push(row2_wrap);
        panel_children.push(search_field(&panel_suffix));
        panel_children.push(quick_links_node);
    }

    band_children.push(mobile_panel(&panel_id, panel_children));

    band_children.push(div(
        vec![("data-blocks-app-shell-stacked-overlap-heading-row", "")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text(page_title)],
            ),
            p(vec![], vec![text(page_summary)]),
        ],
    ));

    div(
        vec![
            ("tabindex", "0"),
            ("role", "region"),
            ("aria-label", root_label),
            ("data-blocks-app-shell-stacked-overlap-root", ""),
            ("data-blocks-app-shell-stacked-overlap-variant", variant),
        ],
        vec![
            div(
                vec![("data-blocks-app-shell-stacked-overlap-band", "")],
                band_children,
            ),
            main_tag(vec![], vec![body_cards()]),
        ],
    )
}

/// `app-shell-stacked-overlap` の Demo 本体。2 variant を縦に並記する
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-app-shell-stacked-overlap-stack")],
        vec![
            caption("1 段ナビ・中立配色"),
            shell(
                "single-row",
                "アプリシェル（1 段ナビ）",
                "アプリ（1 段）",
                None,
                "ダッシュボード",
                "本日の概況をまとめています。",
            ),
            caption("2 段ナビ・ブランド配色 + 検索"),
            shell(
                "two-row-search",
                "アプリシェル（2 段ナビ + 検索）",
                "アプリ（2 段）",
                Some("クイックリンク"),
                "ダッシュボード（検索対応）",
                "検索欄からドキュメントを横断検索できます。",
            ),
        ],
    )
}
```

## 集約元との差分メモ

- 集約元 2 件を 2 variant（1 段ナビ・中立配色 / 2 段ナビ・ブランド配色 +
  検索）へ集約しました。1 段目が R1278（主参照）、2 段目が R1279（帯の
  配色だけがブランド色になる版）・R1282（ナビが 2 段になり 2 行目に検索欄
  と主要リンクが入る版）に対応します。
- 帯の下端には常に本文カードが重なります。重なり量は Demo 枠の幅
  （コンテナクエリ）が広いほど大きくなります。
- 集約元の配色・文言・アイコン・ハンバーガーメニュー・ドロップダウン
  開閉は持ち込んでいません。ナビバーは常に主要リンクが並んだ状態を表示し、
  狭い幅では常時展開のメニューパネルへ切り替わります。
- 通知ボタン・プロフィールメニューのトリガーは押しても何も起きないため、
  いずれも `disabled` にして固定し、フォーカス・操作不能であることを明示
  しています。
- 検索欄は送信先を持たない静的な入力で、送信ボタンは置いていません。
- 列切り替え・重なり量の判定はビューポート幅ではなく Demo 枠自体の幅
  （コンテナクエリ）を基準にしています。Demo 枠の幅を 48rem（768px 相当）
  以上に広げると、重なりが大きくなりデスクトップ用ナビが表示されます。
- 文言・アイコン・ユーザー名はすべて独自に書いた架空のものです。配色・
  余白・角丸は既存のテーマトークンに従っています。

関連情報: [Navigation Menu](../themes/navigation-menu.md) /
[Input Group](../themes/input-group.md) / [Input](../themes/input.md) /
[Avatar](../themes/avatar.md) / [Menu](../themes/menu.md) /
[Button](../themes/button.md) / [Card](../themes/card.md) /
[Heading](../themes/heading.md) / [Collapsible](../themes/collapsible.md) /
[Icon](../themes/icon.md)
