# card-heading-toolbar

`fandhe-frontend-pre-styled-ui` の `heading` / `badge` / `text` /
`input-group` / `input` / `button` / `menu` / `empty-state` / `separator`
部品を合成した、ツールバー付きの区画の実例です。Blocks セクションは新規
部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください（主参照は対応表 ID R0643、集約元は
R0644。出典の固有名・ファイル名は記載しません）。

区画の上部に見出し・件数バッジ・説明文（左）と、検索欄・操作ボタン・
三点メニューのツールバー（右）を並べ、その下に本文、さらに下部へ最終
更新時刻とキャンセル/保存ボタンの操作行を配置した構成です。狭い幅
（`40rem` 未満）ではツールバーが見出しの下へ回り込みます。

本文はプレースホルダ表示の版（例 A）と、空状態表示に差し替えた版
（例 B）の 2 例を並べています。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。ボタンは `type="button"` のまま送信先を持たず、
三点メニューは閉じた状態の固定表示です（開閉には `fandhe-frontend-wasm-full`
の JS 配線が必要で、docs サイトは JS ハイドレーションを行いません）。文言は
すべて独自に書いた架空のものであり、実企業名・実クレデンシャル・PII を
含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps};
use fandhe_frontend_pre_styled_ui::field::{FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// ヘッダー左側（見出し群）。
fn heading_group() -> Node {
    div(
        vec![("data-blocks-card-heading-toolbar-heading-group", "")],
        vec![
            div(
                vec![("data-blocks-card-heading-toolbar-title-row", "")],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("進行中のタスク")],
                    ),
                    badge::badge(
                        &BadgeProps {
                            palette: ColorPalette::Neutral,
                            ..BadgeProps::default()
                        },
                        vec![],
                        vec![text("12 件")],
                    ),
                ],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("担当者ごとの進捗をまとめて確認できます。")],
            ),
        ],
    )
}

/// 三点メニュー（無 JS のため閉じた状態で固定する。モジュール doc
/// 「三点メニューは無 JS のため閉じた状態で固定する」節参照）。
fn overflow_menu(content_id: &'static str) -> Node {
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id),
        vec![("aria-label", "その他の操作")],
        vec![text("\u{2026}")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id),
        None,
        vec![],
        vec![
            menu::item("export", false, false, vec![], vec![text("書き出す")]),
            menu::item(
                "archive",
                false,
                false,
                vec![],
                vec![text("アーカイブする")],
            ),
            menu::separator(vec![], vec![]),
            menu::item("delete", false, false, vec![], vec![text("削除する")]),
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

/// ヘッダー右側（ツールバー: 検索 + 操作ボタン + 三点メニュー）。
fn toolbar(search_field_id: &'static str, menu_content_id: &'static str) -> Node {
    let field = FieldProps {
        id: search_field_id,
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
    div(
        vec![("data-blocks-card-heading-toolbar-toolbar", "")],
        vec![
            input_group::root(
                &group_props,
                vec![],
                vec![
                    input::input(
                        &InputProps::default(),
                        &field,
                        vec![
                            ("type", "search"),
                            ("placeholder", "検索"),
                            ("aria-label", "区画内を検索"),
                        ],
                    ),
                    input_group::addon(
                        InputGroupAlign::InlineEnd,
                        &group_props,
                        vec![],
                        vec![button::button(
                            &ButtonProps::default(),
                            vec![],
                            vec![text("検索")],
                        )],
                    ),
                ],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("書き出す")],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("新規作成")]),
            overflow_menu(menu_content_id),
        ],
    )
}

/// 本文（`variant` に応じてプレースホルダ / 空状態を切り替える。モジュール
/// doc「2 インスタンスで本文の差分を表現する」節参照）。
fn body(variant: &'static str) -> Node {
    let inner = if variant == "content" {
        div(
            vec![("data-blocks-card-heading-toolbar-placeholder", "")],
            vec![styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("ここに区画の本文が入ります。")],
            )],
        )
    } else {
        empty_state::root(
            &EmptyStateProps::default(),
            vec![],
            vec![empty_state::content(
                vec![],
                vec![
                    empty_state::title(vec![], vec![text("項目がまだありません")]),
                    empty_state::description(
                        vec![],
                        vec![text("最初の項目を追加すると、ここに一覧が表示されます。")],
                    ),
                    empty_state::actions(
                        vec![],
                        vec![button::button(
                            &ButtonProps::default(),
                            vec![],
                            vec![text("最初の項目を追加")],
                        )],
                    ),
                ],
            )],
        )
    };
    div(
        vec![
            ("data-blocks-card-heading-toolbar-body", ""),
            ("data-blocks-card-heading-toolbar-variant", variant),
        ],
        vec![inner],
    )
}

/// 操作行（左: 最終更新時刻、右: キャンセル/保存）。
fn footer() -> Node {
    div(
        vec![("data-blocks-card-heading-toolbar-footer", "")],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("最終更新: 5 分前")],
            ),
            div(
                vec![("data-blocks-card-heading-toolbar-footer-actions", "")],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("キャンセル")],
                    ),
                    button::button(&ButtonProps::default(), vec![], vec![text("保存")]),
                ],
            ),
        ],
    )
}

/// 区画 1 件（`variant` で本文を切り替える。id 接尾辞はパネル間で id が
/// 重複しないようにするための一意化。モジュール doc「id の一意性」節参照）。
fn panel(variant: &'static str, suffix: &'static str) -> Node {
    let search_field_id = if suffix == "content" {
        "blocks-card-heading-toolbar-search-content"
    } else {
        "blocks-card-heading-toolbar-search-empty"
    };
    let menu_content_id = if suffix == "content" {
        "blocks-card-heading-toolbar-menu-content"
    } else {
        "blocks-card-heading-toolbar-menu-empty"
    };
    div(
        vec![
            ("data-blocks-card-heading-toolbar-panel", ""),
            ("data-blocks-card-heading-toolbar-variant", variant),
        ],
        vec![
            div(
                vec![("data-blocks-card-heading-toolbar-header", "")],
                vec![heading_group(), toolbar(search_field_id, menu_content_id)],
            ),
            separator::separator(&SeparatorProps::default(), vec![]),
            body(variant),
            separator::separator(&SeparatorProps::default(), vec![]),
            footer(),
        ],
    )
}

/// `card-heading-toolbar` の Demo 本体（プレースホルダ版・空状態版の 2
/// パネルを縦積みで並記する。呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-card-heading-toolbar-layout")],
        vec![panel("content", "content"), panel("empty", "empty")],
    )
}
```

## 集約元との差分メモ

- 主参照（対応表 ID R0643）は本文にプレースホルダを表示する代表構成
  （例 A）です。集約元（対応表 ID R0644）は本文を空状態表示に差し替えた
  版で、本 Demo では例 A・例 B の 2 インスタンスとして並べています
  （`data-blocks-card-heading-toolbar-variant` で区別できます）。
- 三点メニューは閉じた状態の固定表示です。トリガーを押しても開きません
  （wasm-full の JS 配線がある実アプリでは操作できます）。
- 検索欄には可視ラベルを置かず、`aria-label` でアクセシブル名を確保して
  います（本 block の使用部品に `field`/`visually_hidden` を含めないため）。
- 狭い幅（`40rem` 未満）ではヘッダーのツールバーが見出し群の下へ回り込み、
  `40rem` 以上で左右配置に切り替わります。
- 文言・配色は既存のテーマトークンに従い、独自に書いた架空のものです。

関連情報: [Heading](../themes/heading.md) / [Badge](../themes/badge.md) /
[Text](../themes/text.md) / [Input Group](../themes/input-group.md) /
[Input](../themes/input.md) / [Button](../themes/button.md) /
[Menu](../themes/menu.md) / [Empty State](../themes/empty-state.md) /
[Separator](../themes/separator.md)
