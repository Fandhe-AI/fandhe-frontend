# navbar-with-search

`fandhe-frontend-pre-styled-ui` の `input-group` / `input` / `field` /
`navigation-menu` / `button` / `avatar` / `menu` / `icon` を合成した検索欄
付きの 1 段アプリナビバーです。[navbar-app-links](./navbar-app-links.md)
の「リンク群 + アクション」に対し、本 block は検索欄の置き方 3 通りを
並記します（主参照は対応表 ID R0162、出典の固有名は記載しません）。

Demo は 4 インスタンスを並記します: ロゴ → リンク群 → 検索欄 → 通知 →
アバターメニューの `links-end`、リンクなしで検索欄を中央に置く
`center-search`、12 列グリッドでロゴ・検索欄・操作を割り付ける `grid-12`、
そして Demo 枠を狭幅へ固定し検索欄が実際に見える状態を示す `narrow`。

狭い幅（Demo 枠基準の container query、48rem 未満）では、通常は検索欄
ラッパーを隠し代わりに検索アイコンボタン（無 JS のため無効化）を表示しま
すが、無 JS ではこのボタンから検索欄を開けず主要機能を確認できないため、
`narrow` インスタンスのみ Demo 枠を強制的に狭幅へ固定したうえで検索欄
ラッパーを常時表示し検索アイコンボタンを隠します。`links-end` のナビは
狭幅で折り返すのみとし、ハンバーガー化はしません（collapsible は使用部品
の契約外のため持ち込みません）。

本 Demo は静的表示例です。docs サイトは JS ハイドレーションを行わないため
操作系ボタンはすべて無効化しています。`<form>` は使わず、検索欄も送信先を
持ちません。文言はすべて架空のものです。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, header, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc 参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// 実在の自組織 URL。
const ORG: &str = "https://github.com/Fandhe-AI";

/// 装飾用の自作幾何アイコン（実在ブランドのロゴを模さない、`label: None`）。
fn geo_icon(d: &'static str) -> Node {
    icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", d)], vec![])],
    )
}

/// 検索アイコン（円 + 柄の 1 path、[`super::super::hero::hero_search`] と
/// 同型の自作幾何アイコン）。`geo_icon`（`fill="currentColor"` 固定）は
/// 塗りつぶし図形専用のため、輪郭のみで構成される虫眼鏡には使えない
/// （`fill="currentColor"` のまま柄を描くと直線はゼロ面積で消え、円弧は
/// 開始点への暗黙クローズで塗りつぶし円になってしまう）。`hero_search` と
/// 同じく `fill="none"` + `stroke="currentColor"` で明示的に上書きする。
fn search_icon() -> Node {
    icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zm9 17-5.2-5.2"),
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

/// ベル（通知）の幾何アイコン。
fn bell_icon() -> Node {
    geo_icon("M12 3a5 5 0 0 0-5 5v3l-2 4h14l-2-4V8a5 5 0 0 0-5-5zM10 18a2 2 0 0 0 4 0h-4z")
}

/// ロゴ（幾何図形 + ブランド名テキスト、リンクにしない）。
fn logo() -> Node {
    span(
        vec![("data-blocks-navbar-with-search-logo", "")],
        vec![
            geo_icon("M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z"),
            span(vec![], vec![text("Fandhe Console")]),
        ],
    )
}

/// メインナビ本体（`links-end` のみが持つ）。
fn nav() -> Node {
    let props = NavigationMenuProps::default();
    navigation_menu::root(
        &props,
        "メインナビゲーション",
        vec![("data-blocks-navbar-with-search-nav", "")],
        vec![navigation_menu::list(
            &props,
            vec![],
            vec![
                navigation_menu::item(
                    navigation_menu::OpenState::Closed,
                    false,
                    &props,
                    "dashboard",
                    vec![],
                    vec![navigation_menu::link(
                        "./",
                        true,
                        vec![],
                        vec![text("ダッシュボード")],
                    )],
                ),
                navigation_menu::item(
                    navigation_menu::OpenState::Closed,
                    false,
                    &props,
                    "projects",
                    vec![],
                    vec![navigation_menu::link(
                        REPO,
                        false,
                        vec![],
                        vec![text("プロジェクト")],
                    )],
                ),
                navigation_menu::item(
                    navigation_menu::OpenState::Closed,
                    false,
                    &props,
                    "reports",
                    vec![],
                    vec![navigation_menu::link(
                        ORG,
                        false,
                        vec![],
                        vec![text("レポート")],
                    )],
                ),
            ],
        )],
    )
}

/// 検索欄一式（可視ラベルの代わりに `visually_hidden` + `<label for>`、
/// [`super::super::hero::hero_search::search_group`] と同型）。`variant`
/// は `id` の suffix（モジュール doc「id と ARIA の一意性」節）。
fn search_group(variant: &'static str) -> Node {
    let field_id = format!("blocks-navbar-with-search-query-{variant}");
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
        vec![("data-blocks-navbar-with-search-field", "")],
        vec![
            visually_hidden::root(
                vec![],
                vec![field::label(&query_field, vec![], vec![text("検索")])],
            ),
            input_group::root(
                &group_props,
                vec![("data-blocks-navbar-with-search-group", "")],
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

/// 検索アイコンボタン（狭幅専用、無 JS のため `disabled: true` 固定）。
fn search_toggle() -> Node {
    button::icon_button(
        &ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        },
        "検索を開く",
        vec![("data-blocks-navbar-with-search-search-toggle", "")],
        vec![search_icon()],
    )
}

/// 通知ボタン（無 JS のため `disabled: true` 固定、アクセシブルネーム付き）。
fn notification_button() -> Node {
    button::icon_button(
        &ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        },
        "通知を表示",
        vec![("data-blocks-navbar-with-search-notify", "")],
        vec![bell_icon()],
    )
}

/// アバターメニュー（`menu::trigger` を `disabled: true` 固定にし、中に
/// avatar を入れる。`content_id` は variant ごとに一意にする）。
fn profile_menu(variant: &str) -> Node {
    let content_id = format!("blocks-navbar-with-search-profile-menu-{variant}");
    let trigger = menu::trigger(
        OpenState::Closed,
        true,
        Some(content_id.as_str()),
        vec![
            ("aria-label", "アカウントメニューを開く"),
            ("data-blocks-navbar-with-search-profile-trigger", ""),
        ],
        vec![avatar::root(
            &AvatarProps::default(),
            vec![],
            vec![avatar::fallback(
                ImageStatus::Error,
                vec![],
                vec![text("YK")],
            )],
        )],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item("account", false, false, vec![], vec![text("アカウント")]),
            menu::item("settings", false, false, vec![], vec![text("設定")]),
            menu::separator(vec![], vec![]),
            menu::item("logout", false, false, vec![], vec![text("ログアウト")]),
        ],
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// 1 variant 分のナビバー本体を組み立てる。`with_nav` はリンク群の有無、
/// `layout` は [`LAYOUT_CSS`] が分岐に使う `data-*` 値、`narrow` は
/// [`super::navbar_app_links`] と同型の「Demo 枠を `< 48rem` に固定する」
/// フラグ（閲覧者の画面幅に関係なく container query を狭幅側に倒し、検索欄
/// が実際に使える主要機能であることを Demo 上で示す。モジュール doc
/// 「検索欄は狭幅でアイコンボタンへ縮む」節の是正）。
fn bar(variant: &'static str, layout: &'static str, with_nav: bool, narrow: bool) -> Node {
    let mut children: Vec<Node> = vec![logo()];
    if with_nav {
        children.push(div(
            vec![("data-blocks-navbar-with-search-nav-wrap", "")],
            vec![nav()],
        ));
    }
    children.push(div(
        vec![("data-blocks-navbar-with-search-search-wrap", "")],
        vec![search_group(variant)],
    ));
    children.push(div(
        vec![("data-blocks-navbar-with-search-actions", "")],
        vec![
            search_toggle(),
            notification_button(),
            profile_menu(variant),
        ],
    ));

    let mut frame_attrs = vec![("data-blocks-navbar-with-search-shell", "")];
    if narrow {
        frame_attrs.push(("data-blocks-navbar-with-search-frame", "narrow"));
    }

    div(
        frame_attrs,
        vec![header(
            vec![
                ("data-blocks-navbar-with-search-root", ""),
                ("data-blocks-navbar-with-search-variant", variant),
                ("data-blocks-navbar-with-search-layout", layout),
            ],
            children,
        )],
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &'static str) -> Node {
    p(
        vec![("class", "blocks-navbar-with-search-caption")],
        vec![text(label)],
    )
}

/// `navbar-with-search` の Demo 本体。4 variant を縦に並記する純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-navbar-with-search-stack")],
        vec![
            caption("リンク群の右に検索欄を置く（links-end）"),
            bar("links-end", "links-end", true, false),
            caption("リンクなし・検索欄を中央に置く（center-search）"),
            bar("center-search", "center-search", false, false),
            caption("12 列グリッドでロゴ / 検索欄 / 操作を割り付ける（grid-12）"),
            bar("grid-12", "grid-12", false, false),
            caption("狭幅（< 48rem）でも検索欄が操作可能であること（narrow）"),
            bar("narrow", "links-end", true, true),
        ],
    )
}
```

## 集約元との差分メモ

- 主参照 R0162（`links-end`）を基準に、色調違いのみの集約元（R1090/R1091）
  はテーマの明暗切り替えへ統一し独立インスタンスにしていません。
- R0161/R0580（リンクなし・検索欄を中央に置く構成）は `center-search` に、
  R1094（12 列グリッド割り付け）は `grid-12` に対応します。
- 検索欄は `navbar-app-links` のハンバーガーパネルとは異なり、狭幅で
  アイコンボタンへ縮むのみで常時展開パネルは持ちません（使用部品 8 件の
  契約外のため collapsible は持ち込みません）。
- 操作系ボタン（検索トグル・通知・アバターメニュー trigger）はすべて
  無効化しています。

関連情報: [Input Group](../themes/input-group.md) /
[Input](../themes/input.md) / [Field](../themes/field.md) /
[Navigation Menu](../themes/navigation-menu.md) /
[Button](../themes/button.md) / [Avatar](../themes/avatar.md) /
[Menu](../themes/menu.md) / [Icon](../themes/icon.md)
