# page-heading-actions

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `badge` / `button` /
`button-group` / `menu` / `breadcrumb` / `link` / `input-group` / `input` /
`icon` / `avatar` 部品を合成した、操作ボタン付きのページ見出しの実例です。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R1126、集約元は R1224・R1225・R1231・R1127・R0593・R0590・R0189・R0592・
R0596・R1226・R0652・R0654。出典の固有名・ファイル名は記載しません）。

左に見出し・説明（任意でバッジ）、右に主操作・副操作ボタンと三点メニュー
（閉じた状態）を横並びに配置した基本形（例 A）に加え、下罫線付きの区画
見出し（例 B）、パンくず上段付き（例 C）、右の操作列を検索入力グループへ
差し替えた形（例 D）、戻るリンク上段とアカウントメニュー（例 E）の 5 通り
を並べています。狭い幅（`40rem` 未満）では見出しとボタン列が縦積みになり、
一部の操作ボタンは非表示になりますが、同じ操作は三点メニューからも常に
到達できます。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。ボタンは `type="button"` のまま送信先を持たず、
三点メニュー・アカウントメニューは閉じた状態の固定表示です（開閉には
`fandhe-frontend-wasm-full` の JS 配線が必要で、docs サイトは JS
ハイドレーションを行いません）。文言はすべて独自に書いた架空のものであり、
実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::button_group;
use fandhe_frontend_pre_styled_ui::field::{FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::{Orientation, Size};

/// 自作の単純な幾何アイコン（`cta_split_actions.rs::geo_icon` と同型。
/// モジュール doc「アイコンは自作の単純幾何図形」参照）。
///
/// 本ファイルの `path_d` はいずれも開いた線分（moveto/lineto のみで
/// `z` 閉曲線を持たない）ため、`icon::icon` 既定の `fill="currentColor"`
/// のままでは面積ゼロで何も描画されない（`sidebar_03.rs::geo_icon` が
/// 使う閉じた矩形パスとの差異）。`path` 要素へ `fill="none"` +
/// `stroke="currentColor"` を上書きしてアウトライン描画にする。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
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

/// 三点メニュー（無 JS のため閉じた状態で固定する。モジュール doc
/// 「メニューは無 JS のため閉じた状態で固定する」節参照）。
///
/// `collapsed_items`（`(id, ラベル)`）は `40rem` 未満で非表示になる
/// `data-blocks-page-heading-actions-collapsible` 付きボタンと同じ操作を
/// 表す項目で、共通の書き出す/複製する/削除するより前に挿入する。狭幅では
/// 三点メニュー経由でしか到達できないため、`export`/`duplicate`/`delete`
/// のみを常時列挙する構成では狭幅操作が失われる（codex/bugbot 指摘）。
fn overflow_menu(
    content_id: &'static str,
    collapsed_items: &[(&'static str, &'static str)],
) -> Node {
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id),
        vec![("aria-label", "その他の操作")],
        vec![text("\u{2026}")],
    );
    let mut items: Vec<Node> = collapsed_items
        .iter()
        .map(|(id, label)| menu::item(id, false, false, vec![], vec![text(*label)]))
        .collect();
    if !collapsed_items.is_empty() {
        items.push(menu::separator(vec![], vec![]));
    }
    items.extend([
        menu::item("export", false, false, vec![], vec![text("書き出す")]),
        menu::item("duplicate", false, false, vec![], vec![text("複製する")]),
        menu::separator(vec![], vec![]),
        menu::item("delete", false, false, vec![], vec![text("削除する")]),
    ]);
    let content = menu::content(OpenState::Closed, Some(content_id), None, vec![], items);
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// アカウントメニュー（trigger の子にアバターを持つ。モジュール doc
/// 「E（戻るリンク + アカウントメニュー）」節参照）。
fn account_menu(content_id: &'static str) -> Node {
    let avatar_node = avatar::root(
        &AvatarProps::default(),
        vec![],
        vec![avatar::fallback(
            ImageStatus::Error,
            vec![],
            vec![text("HF")],
        )],
    );
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id),
        vec![("aria-label", "アカウントメニューを開く（Haruto Fujimaki）")],
        vec![avatar_node],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id),
        None,
        vec![],
        vec![
            menu::item("profile", false, false, vec![], vec![text("プロフィール")]),
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

/// 見出し群（左側）。`badge`/`description` は任意（インスタンスごとに
/// 差し替える）。
fn heading_group(
    level: HeadingLevel,
    title: &'static str,
    badge: Option<Node>,
    description: Option<&'static str>,
) -> Node {
    let mut title_row_children = vec![heading(
        level,
        &HeadingProps {
            size: HeadingSize::Xl,
            ..HeadingProps::default()
        },
        vec![],
        vec![text(title)],
    )];
    if let Some(b) = badge {
        title_row_children.push(b);
    }
    let mut children = vec![div(
        vec![("data-blocks-page-heading-actions-title-row", "")],
        title_row_children,
    )];
    if let Some(d) = description {
        children.push(styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(d)],
        ));
    }
    div(
        vec![("data-blocks-page-heading-actions-heading-group", "")],
        children,
    )
}

/// A: 基本形（ページ見出し）。
fn instance_a() -> Node {
    let heading_group_node = heading_group(
        HeadingLevel::H1,
        "プロジェクト設定",
        Some(badge::badge(
            &BadgeProps {
                palette: ColorPalette::Success,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("公開中")],
        )),
        Some("チームの権限とインテグレーションをここで管理します。"),
    );
    let actions = div(
        vec![("data-blocks-page-heading-actions-actions", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-page-heading-actions-collapsible", "")],
                vec![text("下書きを保存")],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("変更を公開")]),
            overflow_menu(
                "blocks-page-heading-actions-menu-a",
                &[("draft-save", "下書きを保存")],
            ),
        ],
    );
    div(
        vec![
            ("data-blocks-page-heading-actions-instance", ""),
            ("data-blocks-page-heading-actions-variant", "a"),
        ],
        vec![div(
            vec![("data-blocks-page-heading-actions-header", "")],
            vec![heading_group_node, actions],
        )],
    )
}

/// B: 区画見出し（下罫線）。
fn instance_b() -> Node {
    let heading_group_node = heading_group(
        HeadingLevel::H2,
        "メンバー",
        None,
        Some("このワークスペースに参加しているメンバーの一覧です。"),
    );
    let actions = div(
        vec![("data-blocks-page-heading-actions-actions", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    ..ButtonProps::default()
                },
                vec![("aria-label", "並べ替え")],
                vec![geo_icon("M4 6h16M4 12h10M4 18h6")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    ..ButtonProps::default()
                },
                vec![("aria-label", "表示設定")],
                vec![geo_icon("M12 4v16M4 12h16")],
            ),
        ],
    );
    div(
        vec![
            ("data-blocks-page-heading-actions-instance", ""),
            ("data-blocks-page-heading-actions-variant", "b"),
        ],
        vec![div(
            vec![
                ("data-blocks-page-heading-actions-header", ""),
                ("data-blocks-page-heading-actions-section-heading", ""),
            ],
            vec![heading_group_node, actions],
        )],
    )
}

/// C: パンくず上段 + ボタン群。
fn instance_c() -> Node {
    let breadcrumb_row = breadcrumb::root(
        Size::Md,
        BreadcrumbVariant::default(),
        Some("パンくずリスト"),
        vec![],
        vec![breadcrumb::list(
            vec![],
            vec![
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../", vec![], vec![text("Blocks")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(vec![], vec![text("請求設定")])],
                ),
            ],
        )],
    );
    let heading_group_node = heading_group(HeadingLevel::H1, "請求設定", None, None);
    let button_group_node = button_group::root(
        Orientation::Horizontal,
        "請求の操作",
        vec![],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    ..ButtonProps::default()
                },
                vec![
                    ("aria-label", "詳細設定"),
                    ("data-blocks-page-heading-actions-collapsible", ""),
                ],
                vec![geo_icon(
                    "M12 15a3 3 0 100-6 3 3 0 000 6zM4 12h2m12 0h2M12 4v2m0 12v2",
                )],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-page-heading-actions-collapsible", "")],
                vec![text("請求書をダウンロード")],
            ),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-page-heading-actions-primary-action", "")],
                vec![text("プランを変更")],
            ),
        ],
    );
    let actions = div(
        vec![("data-blocks-page-heading-actions-actions", "")],
        vec![
            button_group_node,
            overflow_menu(
                "blocks-page-heading-actions-menu-c",
                &[
                    ("detail-settings", "詳細設定"),
                    ("invoice-download", "請求書をダウンロード"),
                ],
            ),
        ],
    );
    div(
        vec![
            ("data-blocks-page-heading-actions-instance", ""),
            ("data-blocks-page-heading-actions-variant", "c"),
        ],
        vec![
            div(
                vec![("data-blocks-page-heading-actions-top-row", "")],
                vec![breadcrumb_row],
            ),
            div(
                vec![("data-blocks-page-heading-actions-header", "")],
                vec![heading_group_node, actions],
            ),
        ],
    )
}

/// D: 操作列差し替え（検索入力グループ）。
fn instance_d() -> Node {
    let heading_group_node = heading_group(
        HeadingLevel::H1,
        "問い合わせ一覧",
        Some(badge::badge(
            &BadgeProps {
                palette: ColorPalette::Neutral,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("128 件")],
        )),
        None,
    );
    let search_field = FieldProps {
        id: "blocks-page-heading-actions-search",
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
    let search_group = input_group::root(
        &group_props,
        vec![("data-blocks-page-heading-actions-search-group", "")],
        vec![
            input_group::addon(
                InputGroupAlign::InlineStart,
                &group_props,
                vec![],
                vec![geo_icon(
                    "M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zm9 17-5.2-5.2",
                )],
            ),
            input::input(
                &InputProps::default(),
                &search_field,
                vec![
                    ("type", "search"),
                    ("placeholder", "問い合わせを検索"),
                    ("aria-label", "問い合わせを検索"),
                ],
            ),
            input_group::addon(
                InputGroupAlign::InlineEnd,
                &group_props,
                vec![],
                vec![input_group::button(
                    &group_props,
                    vec![],
                    vec![text("並べ替え")],
                )],
            ),
        ],
    );
    let actions = div(
        vec![("data-blocks-page-heading-actions-actions", "")],
        vec![
            search_group,
            overflow_menu("blocks-page-heading-actions-menu-d", &[]),
        ],
    );
    div(
        vec![
            ("data-blocks-page-heading-actions-instance", ""),
            ("data-blocks-page-heading-actions-variant", "d"),
        ],
        vec![div(
            vec![("data-blocks-page-heading-actions-header", "")],
            vec![heading_group_node, actions],
        )],
    )
}

/// E: 戻るリンク上段 + アカウントメニュー。
fn instance_e() -> Node {
    let back_link = link::root(
        "../",
        &LinkProps::default(),
        vec![],
        vec![geo_icon("M15 18l-6-6 6-6"), text("一覧へ戻る")],
    );
    let heading_group_node = heading_group(HeadingLevel::H1, "APIキー", None, None);
    let actions = div(
        vec![("data-blocks-page-heading-actions-actions", "")],
        vec![account_menu("blocks-page-heading-actions-menu-e")],
    );
    div(
        vec![
            ("data-blocks-page-heading-actions-instance", ""),
            ("data-blocks-page-heading-actions-variant", "e"),
        ],
        vec![
            div(
                vec![("data-blocks-page-heading-actions-top-row", "")],
                vec![back_link],
            ),
            div(
                vec![("data-blocks-page-heading-actions-header", "")],
                vec![heading_group_node, actions],
            ),
        ],
    )
}

/// `page-heading-actions` の Demo 本体（5 インスタンスを縦積みで並記する。
/// 呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-page-heading-actions-layout")],
        vec![
            instance_a(),
            instance_b(),
            instance_c(),
            instance_d(),
            instance_e(),
        ],
    )
}
```

## 原案差分メモ

- 例 A（基本形）は主参照（対応表 ID R1126）を代表構成とし、集約元
  R1224・R1225・R1231（見出し + バッジ + 説明、副操作/主操作ボタン + 三点
  メニューの組み合わせ）をこの 1 例に集約しています。
- 例 B（区画見出し）は R0593 に対応し、下罫線付きの区画見出しとアイコン
  のみの Ghost ボタン列という構成の差分を表します。
- 例 C（パンくず上段 + ボタン群）は R1127（パンくず）・R0590・R0189
  （ボタン群 + 三点メニュー）に対応します。
- 例 D（検索入力グループ）は R0592・R1226・R0652・R0654 に対応し、右の
  操作列を「検索アイコン付き入力 + 並べ替えボタン」の `input-group` へ
  差し替えた形を表します。
- 例 E（戻るリンク + アカウントメニュー）は R1127（戻るリンク）・R0596
  （アカウントメニュー）に対応します。
- 狭い幅（`40rem` 未満）での「ボタンの一部をメニューへ集約する」挙動は
  無 JS のため CSS のみで表現しています（`data-blocks-page-heading-
  actions-collapsible` を持つボタンを非表示にし、同じ操作を三点メニューの
  項目としても常に用意する静的な集約）。実際のブラウザでの表示切り替え
  確認は本 Demo では行っていません。
- 文言・配色は既存のテーマトークンに従い、独自に書いた架空のものです。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Badge](../themes/badge.md) / [Button](../themes/button.md) /
[Button Group](../themes/button-group.md) / [Menu](../themes/menu.md) /
[Breadcrumb](../themes/breadcrumb.md) / [Link](../themes/link.md) /
[Input Group](../themes/input-group.md) / [Input](../themes/input.md) /
[Icon](../themes/icon.md) / [Avatar](../themes/avatar.md)
