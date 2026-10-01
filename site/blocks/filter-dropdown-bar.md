# filter-dropdown-bar

見出しの下に、並び替えメニューと商品フィルタ（カテゴリ・色・サイズ・素材など
のドロップダウン）を横一列に並べる目的別パーツです。`heading` / `menu` /
`popover` / `checkbox` / `button` / `badge` の 6 部品を合成します。Blocks は
既存部品の合成例であり、新しい UI 部品は追加しません。

フィルタのうち 1 つは popover を開いた状態で描き、中に checkbox 一覧を表示
します。選択中の件数はバッジで示します。狭い幅では、フィルタ群を「フィルタ」
ボタン 1 個へ畳みます。

主参照は対応表 ID R0815（中央見出し + 4 フィルタ）で、R0816（左見出し +
3 フィルタ + 件数バッジ）を集約しています。文言・配色・装飾は参照元から持ち
込まず、カテゴリ・フィルタの値はすべて架空のデータです。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。上の Demo は
「中央見出し + 4 フィルタ」（`centered`、R0815）・「左寄せ見出し + 3 フィル
タ + 件数バッジ」（`left`、R0816）・「狭幅での畳み表示」（`narrow`）の
3 variant で構成します。

## Rust コード

```rust
use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::menu;
use fandhe_frontend_pre_styled_ui::popover::{self, OpenState};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 並び替えメニュー（常時閉、`disabled: true`）。`variant` は id の suffix
/// （モジュール doc「id と ARIA の一意性」節）。
fn sort_menu(variant: &str) -> Node {
    let trigger_id = format!("blocks-filter-dropdown-bar-sort-trigger-{variant}");
    let content_id = format!("blocks-filter-dropdown-bar-sort-menu-{variant}");
    let trigger = menu::trigger(
        OpenState::Closed,
        true,
        Some(content_id.as_str()),
        vec![
            ("id", trigger_id.as_str()),
            ("data-blocks-filter-dropdown-bar-sort-trigger", ""),
        ],
        vec![text("並び替え")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        Some(trigger_id.as_str()),
        vec![],
        vec![
            menu::item(
                "recommended",
                false,
                false,
                vec![],
                vec![text("おすすめ順")],
            ),
            menu::item("newest", false, false, vec![], vec![text("新着順")]),
            menu::item(
                "price-asc",
                false,
                false,
                vec![],
                vec![text("価格の安い順")],
            ),
            menu::item(
                "price-desc",
                false,
                false,
                vec![],
                vec![text("価格の高い順")],
            ),
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

/// 選択済みチェックボックス 1 件を組み立てる。`disabled: true` の理由は
/// モジュール doc「checkbox をネイティブ `disabled` にする理由」節参照。
fn filter_checkbox(
    name: &str,
    value: &'static str,
    label_text: &'static str,
    checked: bool,
) -> Node {
    let props = CheckboxProps {
        checked: if checked {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        },
        disabled: true,
        ..CheckboxProps::default()
    };
    checkbox::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-filter-dropdown-bar-checkbox", "")],
        vec![
            checkbox::hidden_input(&props, name, value, vec![]),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
            checkbox::label(&props, vec![], vec![text(label_text)]),
        ],
    )
}

/// 開いたフィルタ（`popover`、`trigger` は `disabled: true`）。`badge_count`
/// が `Some` のときトリガーへ件数バッジを添える。`variant`/`filter_key` は
/// id の suffix・接頭辞。
#[allow(clippy::too_many_arguments)]
fn open_filter(
    variant: &str,
    filter_key: &str,
    label_text: &'static str,
    options: &[(&'static str, &'static str, bool)],
    badge_count: Option<u8>,
) -> Node {
    let trigger_id = format!("blocks-filter-dropdown-bar-{filter_key}-trigger-{variant}");
    let panel_id = format!("blocks-filter-dropdown-bar-{filter_key}-panel-{variant}");

    let mut trigger_children = vec![text(label_text)];
    let mut trigger_attrs = vec![
        ("id", trigger_id.as_str()),
        ("data-blocks-filter-dropdown-bar-trigger", ""),
    ];
    let aria_label_value;
    if let Some(count) = badge_count {
        aria_label_value = format!("{label_text}（{count} 件選択中）");
        trigger_attrs.push(("aria-label", aria_label_value.as_str()));
        trigger_children.push(badge(
            &BadgeProps {
                variant: BadgeVariant::Subtle,
                ..BadgeProps::default()
            },
            vec![],
            vec![text(count.to_string())],
        ));
    }

    let trigger = popover::trigger(
        OpenState::Open,
        true,
        Some(panel_id.as_str()),
        trigger_attrs,
        trigger_children,
    );

    let checkboxes: Vec<Node> = options
        .iter()
        .map(|(value, label_text, checked)| {
            filter_checkbox(
                &format!("filter-{filter_key}-{variant}"),
                value,
                label_text,
                *checked,
            )
        })
        .collect();

    let content = popover::content(
        OpenState::Open,
        Some(panel_id.as_str()),
        Some(trigger_id.as_str()),
        None,
        vec![("data-blocks-filter-dropdown-bar-panel", "")],
        checkboxes,
    );
    let positioner = popover::positioner(
        OpenState::Open,
        vec![("data-blocks-filter-dropdown-bar-positioner", "")],
        vec![content],
    );

    popover::root(
        OpenState::Open,
        vec![("data-blocks-filter-dropdown-bar-popover", "")],
        vec![trigger, positioner],
    )
}

/// 閉じたフィルタ（`popover`、`trigger` は `disabled: true`）。パネルを
/// 描画しないため `aria-controls` に `None` を渡し、参照切れを作らない
/// （モジュール doc「id と ARIA の一意性」節参照）。
fn closed_filter(variant: &str, filter_key: &str, label_text: &'static str) -> Node {
    let trigger_id = format!("blocks-filter-dropdown-bar-{filter_key}-trigger-{variant}");
    let trigger = popover::trigger(
        OpenState::Closed,
        true,
        None,
        vec![
            ("id", trigger_id.as_str()),
            ("data-blocks-filter-dropdown-bar-trigger", ""),
        ],
        vec![text(label_text)],
    );
    popover::root(
        OpenState::Closed,
        vec![("data-blocks-filter-dropdown-bar-popover", "")],
        vec![trigger],
    )
}

/// 狭幅で畳んだ「フィルタ」ボタン（`disabled: true`）。件数バッジを子に
/// 置く（「サイズ」フィルタのチェック数と一致させる、モジュール doc
/// 「構成」節の `narrow` variant 参照）。
fn collapsed_filter_button(variant: &str, count: u8) -> Node {
    button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            disabled: true,
            ..ButtonProps::default()
        },
        vec![
            ("data-blocks-filter-dropdown-bar-collapsed-trigger", ""),
            ("data-blocks-filter-dropdown-bar-variant-scoped", variant),
        ],
        vec![
            text("フィルタ"),
            badge(
                &BadgeProps {
                    variant: BadgeVariant::Subtle,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text(count.to_string())],
            ),
        ],
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &'static str) -> Node {
    p(
        vec![("class", "blocks-filter-dropdown-bar-caption")],
        vec![text(label)],
    )
}

/// `centered`（R0815）: 中央寄せ見出し + 4 フィルタ、「色」を開状態にする。
fn centered_bar() -> Node {
    let variant = "centered";
    let heading_node = heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Lg,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text("すべての商品")],
    );
    let bar = div(
        vec![("data-blocks-filter-dropdown-bar-bar", "")],
        vec![
            sort_menu(variant),
            div(
                vec![("data-blocks-filter-dropdown-bar-filters", "")],
                vec![
                    closed_filter(variant, "category", "カテゴリ"),
                    open_filter(
                        variant,
                        "color",
                        "色",
                        &[
                            ("black", "ブラック", false),
                            ("white", "ホワイト", true),
                            ("navy", "ネイビー", false),
                            ("beige", "ベージュ", false),
                            ("green", "グリーン", false),
                        ],
                        None,
                    ),
                    closed_filter(variant, "size", "サイズ"),
                    closed_filter(variant, "material", "素材"),
                ],
            ),
        ],
    );
    div(
        vec![
            ("data-blocks-filter-dropdown-bar-shell", ""),
            ("data-blocks-filter-dropdown-bar-variant", variant),
        ],
        vec![heading_node, bar],
    )
}

/// `left`/`narrow` 共通の「サイズ」フィルタ選択肢（`value`, `label`,
/// `checked`）。両 variant の選択状態を単一の正から導出し、畳みボタンの
/// 件数バッジ（[`narrow_bar`]）が `left` の実チェック数（`open_filter` の
/// checkbox 一覧）と食い違わないようにする（Codex P2 指摘の是正）。
const SIZE_OPTIONS: &[(&str, &str, bool)] = &[
    ("s", "S", true),
    ("m", "M", true),
    ("l", "L", false),
    ("xl", "XL", false),
];

/// [`SIZE_OPTIONS`] のうちチェック済みの件数。
fn size_options_checked_count() -> u8 {
    SIZE_OPTIONS
        .iter()
        .filter(|(_, _, checked)| *checked)
        .count() as u8
}

/// `left`（R0816）: 左寄せ見出し + 3 フィルタ、「サイズ」を開状態にし
/// [`SIZE_OPTIONS`] のチェック数を件数バッジに添える。
fn left_bar() -> Node {
    let variant = "left";
    let heading_node = heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Lg,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text("アウター")],
    );
    let bar = div(
        vec![("data-blocks-filter-dropdown-bar-bar", "")],
        vec![
            sort_menu(variant),
            div(
                vec![("data-blocks-filter-dropdown-bar-filters", "")],
                vec![
                    closed_filter(variant, "category", "カテゴリ"),
                    closed_filter(variant, "color", "色"),
                    open_filter(
                        variant,
                        "size",
                        "サイズ",
                        SIZE_OPTIONS,
                        Some(size_options_checked_count()),
                    ),
                ],
            ),
        ],
    );
    div(
        vec![
            ("data-blocks-filter-dropdown-bar-shell", ""),
            ("data-blocks-filter-dropdown-bar-variant", variant),
        ],
        vec![heading_node, bar],
    )
}

/// `narrow`: `left` と同じ構成を `max-inline-size: 22rem` のシェルで
/// 再現し、`@container` でフィルタ群を畳みボタンへ切り替える（モジュール
/// doc「`@container` の閾値の根拠」節参照）。畳みボタンの件数バッジは
/// [`SIZE_OPTIONS`] のチェック数（`left` と同一の正）から導出し、実際の
/// 選択状態と常に一致させる。
fn narrow_bar() -> Node {
    let variant = "narrow";
    let heading_node = heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Lg,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text("アウター")],
    );
    let bar = div(
        vec![("data-blocks-filter-dropdown-bar-bar", "")],
        vec![
            sort_menu(variant),
            div(
                vec![("data-blocks-filter-dropdown-bar-filters", "")],
                vec![
                    closed_filter(variant, "category", "カテゴリ"),
                    closed_filter(variant, "color", "色"),
                    closed_filter(variant, "size", "サイズ"),
                ],
            ),
            collapsed_filter_button(variant, size_options_checked_count()),
        ],
    );
    div(
        vec![
            ("data-blocks-filter-dropdown-bar-shell", ""),
            ("data-blocks-filter-dropdown-bar-variant", variant),
            ("data-blocks-filter-dropdown-bar-frame", "narrow"),
        ],
        vec![heading_node, bar],
    )
}

/// `filter-dropdown-bar` の Demo 本体。3 variant を caption 付きで縦に
/// 並記する純関数。ルート class は `demo_class`（`BLOCK` 参照）とは別名
/// にする（二重適用防止、`auth_dropdown_panel.rs` と同じ判断）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-filter-dropdown-bar-stack")],
        vec![
            caption("中央見出し + 4 フィルタ（「色」を展開）"),
            centered_bar(),
            caption("左寄せ見出し + 3 フィルタ + 件数バッジ（「サイズ」を展開）"),
            left_bar(),
            caption("狭幅（< 30rem、フィルタ群を「フィルタ」ボタン 1 個へ畳む）"),
            narrow_bar(),
        ],
    )
}
```

## 原案差分メモ

- **R0815（主参照、中央見出し + 4 フィルタ）**: `centered` variant として
  取り込みました。件数バッジは付けず、「色」フィルタの popover を開いた状態
  にしています。
- **R0816（左見出し + 3 フィルタ + 件数バッジ）**: `left` variant として取り
  込みました。「サイズ」フィルタの popover を開いた状態にし、選択件数バッジ
  を付けています。狭幅での畳み表示（`narrow` variant）は、この R0816 の構成
  を `max-inline-size: 22rem` のシェルで再現して実演しています。
- トリガー・checkbox はすべてネイティブ `disabled` にしています（無 JS 環境
  で開閉・チェック状態が静的な見た目と食い違うのを防ぐため）。

関連情報: [Heading](../themes/heading.md) / [Menu](../themes/menu.md) /
[Popover](../themes/popover.md) / [Checkbox](../themes/checkbox.md) /
[Button](../themes/button.md) / [Badge](../themes/badge.md)
