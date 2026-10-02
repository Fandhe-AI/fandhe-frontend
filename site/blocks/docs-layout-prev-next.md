# docs-layout-prev-next

ドキュメント本文の末尾に置く、前後ページへの導線ブロックです。
`pagination` / `link` / `text` / `icon` / `card` の 5 部品を合成します。
Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0083（代表構成: 方向ラベル + タイトルの縦積み）で、
R0081（前後 1 件ずつの 2 ボタン）・R0082（淡色帯 + 次側に概要文）を
集約しています。ページタイトル・概要文はすべて架空の文言であり、実在の
人物・企業・PII は含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::pagination::{self, next_trigger, prev_trigger, ItemMode};
use fandhe_frontend_pre_styled_ui::recipe::{ColorPalette, Size};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};

/// 自作の幾何アイコン（線画。モジュール doc「アイコンは自作の単純図形」
/// 節参照）。`path` へ `fill="none"` + `stroke="currentColor"` を明示し、
/// `icon` の `<svg>` 側が固定で持つ `fill="currentColor"`（塗り面）を
/// 上書きして線画（ストローク）として描画する。
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

/// 左向き矢印アイコン（シェブロン）。
fn left_arrow_icon() -> Node {
    geo_icon("M15 4l-8 8 8 8")
}

/// 右向き矢印アイコン（シェブロン）。
fn right_arrow_icon() -> Node {
    geo_icon("M9 4l8 8-8 8")
}

/// 方向ラベル（`前へ`/`次へ`）+ ページタイトルの縦積み（版 A）。
fn meta_stack(direction_label: &'static str, title: &'static str) -> Node {
    div(
        vec![("class", "blocks-docs-layout-prev-next-meta")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Xs,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(direction_label)],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    weight: TextWeight::Medium,
                    ..TextProps::default()
                },
                vec![],
                vec![text(title)],
            ),
        ],
    )
}

/// A: 代表構成（R0083）。方向ラベル + タイトルの縦積みを矢印アイコンと
/// 併記する。
fn version_stacked_labels() -> Node {
    pagination::root(
        Size::Md,
        ColorPalette::Accent,
        "前後のページ（縦積み）",
        vec![("data-blocks-docs-layout-prev-next-nav", "")],
        vec![
            prev_trigger(
                ItemMode::Link { href: "../../" },
                false,
                vec![],
                vec![left_arrow_icon(), meta_stack("前へ", "はじめに")],
            ),
            next_trigger(
                ItemMode::Link {
                    href: "../../getting-started/quickstart/",
                },
                false,
                vec![],
                vec![meta_stack("次へ", "クイックスタート"), right_arrow_icon()],
            ),
        ],
    )
}

/// B: 前後 1 件ずつの 2 ボタン（R0081）。方向ラベルを省き、アイコン +
/// タイトルのみの 1 行にする。
fn version_two_buttons() -> Node {
    pagination::root(
        Size::Md,
        ColorPalette::Accent,
        "前後のページ（ボタン）",
        vec![("data-blocks-docs-layout-prev-next-nav", "")],
        vec![
            prev_trigger(
                ItemMode::Link {
                    href: "../../guides/",
                },
                false,
                vec![],
                vec![left_arrow_icon(), text("ガイド一覧")],
            ),
            next_trigger(
                ItemMode::Link {
                    href: "../../guides/component-authoring/",
                },
                false,
                vec![],
                vec![text("コンポーネント記述ガイド"), right_arrow_icon()],
            ),
        ],
    )
}

/// C: 淡色帯 + 次側に概要文（R0082）。帯の中へ `pagination::root` +
/// `link::root` を前後 2 件置く（`link` 部品の使用例）。
fn version_band() -> Node {
    card::root(
        CardProps {
            variant: CardVariant::Subtle,
            ..CardProps::default()
        },
        vec![("data-blocks-docs-layout-prev-next-band", "")],
        vec![card::body(
            vec![],
            vec![pagination::root(
                Size::Md,
                ColorPalette::Accent,
                "前後のページ（帯）",
                vec![],
                vec![
                    link::root(
                        "../../",
                        &LinkProps::default(),
                        vec![],
                        vec![left_arrow_icon(), meta_stack("前へ", "はじめに")],
                    ),
                    link::root(
                        "../../guides/",
                        &LinkProps::default(),
                        vec![("data-blocks-docs-layout-prev-next-next-summary", "")],
                        vec![
                            meta_stack("次へ", "ガイド一覧"),
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    variant: TextVariant::Muted,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(
                                    "部品の props と anatomy の書き方をまとめたガイドです。",
                                )],
                            ),
                            right_arrow_icon(),
                        ],
                    ),
                ],
            )],
        )],
    )
}

/// `docs-layout-prev-next` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-docs-layout-prev-next-stack")],
        vec![
            version_stacked_labels(),
            version_two_buttons(),
            version_band(),
        ],
    )
}
```

## 原案差分メモ

- **版 A（代表構成、R0083）**: `pagination` の `prev-trigger`/`next-trigger`
  （`ItemMode::Link`）の中へ、矢印アイコンと方向ラベル・タイトルの縦積みを
  併記します。
- **版 B（前後 1 件ずつの 2 ボタン、R0081）**: 同じトリガーですが方向ラベルを
  省き、アイコン + タイトルのみの 1 行にします。
- **版 C（淡色帯 + 次側に概要文、R0082）**: `card` の Subtle variant で帯を
  作り、中に `pagination::root` + `link::root`（版 A/B のトリガーではなく
  `link` 部品）を前後 2 件置きます。次側のみ概要文を添えます。
- 狭幅（コンテナ幅 32rem 未満）では各 `nav` が縦積みへ切り替わります
  （`@container` によるコンテナクエリ判定）。
- トリガーの固定高さは本 block の CSS で `height: auto` へ解除しています。
- 矢印アイコンは自作の線画（SVG path）で、参照元由来のアイコンセットでは
  ありません。

関連情報: [Pagination](../themes/pagination.md) / [Link](../themes/link.md) /
[Text](../themes/text.md) / [Icon](../themes/icon.md) /
[Card](../themes/card.md)
