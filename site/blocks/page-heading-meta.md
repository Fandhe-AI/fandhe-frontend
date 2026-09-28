# page-heading-meta

`fandhe-frontend-pre-styled-ui` の `heading` / `badge` / `status` / `icon` /
`button` / `button-group` / `breadcrumb` / `clipboard` / `menu` 部品を合成
した、メタ情報行付きページ見出しの実例です。Blocks セクションは新規部品を
追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集
であることに注意してください（主参照は対応表 ID R1128、集約元は
R1132・R0187。出典の固有名・ファイル名は記載しません）。

見出しの下にアイコン付きメタ情報（状態・場所・日付・担当など）を横一列に
並べ、右側に操作ボタン群を配置した代表構成（例 A）に加え、上段に常時
パンくずを表示し状態をバッジで示す形（例 B）、上段に小さなラベルと
バッジを置き右側に共有 URL のコピー欄を配置した形（例 C）の 3 通りを
並べています。狭い幅（`40rem` 未満）ではメタ情報の行が折り返し、ボタン群は
見出しの下へ回ります。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・コピー・状態管理を行いません。ボタンは `type="button"` のまま送信先
を持たず、三点メニューは閉じた状態の固定表示、クリップボードは未コピー
（idle）状態の固定表示です（実際の開閉・コピー動作には
`fandhe-frontend-wasm-full` の JS 配線が必要で、docs サイトは JS
ハイドレーションを行いません）。文言はすべて独自に書いた架空のものであり、
実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::button_group;
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::status::{self, StatusProps};
use fandhe_frontend_pre_styled_ui::{Orientation, Size};

/// 自作の単純な幾何アイコン（`page_heading_actions.rs::geo_icon` と同型。
/// モジュール doc「アイコンは自作の単純幾何図形」参照）。開いた線分
/// パスのため `fill="none"` + `stroke="currentColor"` でアウトライン描画に
/// する。装飾用途のみのため `IconProps::default()`（`label: None`、
/// `aria-hidden`）を使う。
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

/// メタ行 1 項目（装飾アイコン + テキスト）。
fn meta_item(path_d: &'static str, label: &'static str) -> Node {
    div(
        vec![("data-blocks-page-heading-meta-meta-item", "")],
        vec![geo_icon(path_d), text(label)],
    )
}

/// 三点メニュー（無 JS のため閉じた状態で固定する。モジュール doc
/// 「メニューは無 JS のため閉じた状態で固定する」節参照）。
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
            menu::item("duplicate", false, false, vec![], vec![text("複製する")]),
            menu::separator(vec![], vec![]),
            menu::item(
                "archive",
                false,
                false,
                vec![],
                vec![text("アーカイブする")],
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

/// A: 代表構成（R1128）。状態 `status` + メタ行 3 項目 + `button_group` +
/// 主操作 + 三点メニュー。
fn instance_a() -> Node {
    let heading_group = div(
        vec![("data-blocks-page-heading-meta-heading-group", "")],
        vec![
            heading(
                HeadingLevel::H1,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("四半期レビュー資料")],
            ),
            div(
                vec![("data-blocks-page-heading-meta-meta-row", "")],
                vec![
                    status::root(
                        &StatusProps {
                            palette: ColorPalette::Success,
                            ..StatusProps::default()
                        },
                        vec![],
                        vec![status::indicator(vec![]), text("公開中")],
                    ),
                    meta_item("M12 2C8 2 5 5 5 9c0 5 7 13 7 13s7-8 7-13c0-4-3-7-7-7z", "本社 3F 会議室"),
                    meta_item(
                        "M7 3v3m10-3v3M4 9h16M5 6h14a1 1 0 011 1v12a1 1 0 01-1 1H5a1 1 0 01-1-1V7a1 1 0 011-1z",
                        "2026-10-01",
                    ),
                    meta_item(
                        "M12 12a4 4 0 100-8 4 4 0 000 8zm-7 8a7 7 0 0114 0",
                        "担当: 藤巻",
                    ),
                ],
            ),
        ],
    );
    let actions = div(
        vec![("data-blocks-page-heading-meta-actions", "")],
        vec![
            button_group::root(
                Orientation::Horizontal,
                "見出しの操作",
                vec![],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("編集")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("複製")],
                    ),
                ],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("公開する")]),
            overflow_menu("blocks-page-heading-meta-menu-a"),
        ],
    );
    div(
        vec![
            ("data-blocks-page-heading-meta-instance", ""),
            ("data-blocks-page-heading-meta-variant", "a"),
        ],
        vec![div(
            vec![("data-blocks-page-heading-meta-header", "")],
            vec![heading_group, actions],
        )],
    )
}

/// B: 常時パンくず付き（R1132）。状態は `badge` で示す。
fn instance_b() -> Node {
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
                    vec![breadcrumb::current_link(vec![], vec![text("下書き記事")])],
                ),
            ],
        )],
    );
    let heading_group = div(
        vec![("data-blocks-page-heading-meta-heading-group", "")],
        vec![
            div(
                vec![("data-blocks-page-heading-meta-title-row", "")],
                vec![
                    heading(
                        HeadingLevel::H1,
                        &HeadingProps {
                            size: HeadingSize::Xl2,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("新機能の紹介記事")],
                    ),
                    badge::badge(
                        &BadgeProps {
                            palette: ColorPalette::Warning,
                            ..BadgeProps::default()
                        },
                        vec![],
                        vec![text("下書き")],
                    ),
                ],
            ),
            div(
                vec![("data-blocks-page-heading-meta-meta-row", "")],
                vec![
                    meta_item(
                        "M7 3v3m10-3v3M4 9h16M5 6h14a1 1 0 011 1v12a1 1 0 01-1 1H5a1 1 0 01-1-1V7a1 1 0 011-1z",
                        "更新: 2026-09-20",
                    ),
                    meta_item(
                        "M12 12a4 4 0 100-8 4 4 0 000 8zm-7 8a7 7 0 0114 0",
                        "担当: 高橋",
                    ),
                ],
            ),
        ],
    );
    let actions = div(
        vec![("data-blocks-page-heading-meta-actions", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("プレビュー")],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("公開する")]),
            overflow_menu("blocks-page-heading-meta-menu-b"),
        ],
    );
    div(
        vec![
            ("data-blocks-page-heading-meta-instance", ""),
            ("data-blocks-page-heading-meta-variant", "b"),
        ],
        vec![
            div(
                vec![("data-blocks-page-heading-meta-top-row", "")],
                vec![breadcrumb_row],
            ),
            div(
                vec![("data-blocks-page-heading-meta-header", "")],
                vec![heading_group, actions],
            ),
        ],
    )
}

/// C: 上段ラベル + バッジ + 共有 URL（R0187）。バッジは上段（`top-row`）に
/// ラベルと並べて置く（モジュール doc 例 C 参照。見出しと同じ `title-row`
/// には置かない）。
fn instance_c() -> Node {
    let heading_group = div(
        vec![("data-blocks-page-heading-meta-heading-group", "")],
        vec![
            heading(
                HeadingLevel::H1,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("Fandhe 移行プロジェクト")],
            ),
            div(
                vec![("data-blocks-page-heading-meta-meta-row", "")],
                vec![
                    meta_item(
                        "M7 3v3m10-3v3M4 9h16M5 6h14a1 1 0 011 1v12a1 1 0 01-1 1H5a1 1 0 01-1-1V7a1 1 0 011-1z",
                        "開始: 2026-08-01",
                    ),
                    meta_item(
                        "M12 12a4 4 0 100-8 4 4 0 000 8zm-7 8a7 7 0 0114 0",
                        "担当: 吉原",
                    ),
                ],
            ),
        ],
    );
    const SHARE_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
    const SHARE_INPUT_ID: &str = "blocks-page-heading-meta-share-url";
    let share = clipboard::root(
        SHARE_URL,
        false,
        vec![("data-blocks-page-heading-meta-share", "")],
        vec![
            clipboard::label(false, Some(SHARE_INPUT_ID), vec![], vec![text("共有 URL")]),
            clipboard::control(
                false,
                vec![],
                vec![
                    clipboard::input(SHARE_URL, false, vec![("id", SHARE_INPUT_ID)]),
                    clipboard::trigger(
                        false,
                        vec![],
                        vec![
                            clipboard::indicator(false, false, vec![], vec![text("コピー")]),
                            clipboard::indicator(true, false, vec![], vec![text("コピー済み")]),
                        ],
                    ),
                ],
            ),
        ],
    );
    let actions = div(
        vec![("data-blocks-page-heading-meta-actions", "")],
        vec![
            share,
            button::button(&ButtonProps::default(), vec![], vec![text("設定を開く")]),
        ],
    );
    div(
        vec![
            ("data-blocks-page-heading-meta-instance", ""),
            ("data-blocks-page-heading-meta-variant", "c"),
        ],
        vec![
            div(
                vec![("data-blocks-page-heading-meta-top-row", "")],
                vec![
                    span(
                        vec![("data-blocks-page-heading-meta-eyebrow", "")],
                        vec![text("プロジェクト")],
                    ),
                    badge::badge(
                        &BadgeProps {
                            palette: ColorPalette::Accent,
                            ..BadgeProps::default()
                        },
                        vec![],
                        vec![text("ベータ")],
                    ),
                ],
            ),
            div(
                vec![("data-blocks-page-heading-meta-header", "")],
                vec![heading_group, actions],
            ),
        ],
    )
}

/// `page-heading-meta` の Demo 本体（3 インスタンスを縦積みで並記する。
/// 呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-page-heading-meta-layout")],
        vec![instance_a(), instance_b(), instance_c()],
    )
}
```

## 原案差分メモ

- 例 A（代表構成）は主参照（対応表 ID R1128）を軸に、状態表示・場所・日付・
  担当のメタ行と、`button-group`（編集・複製）+ 主操作ボタン + 三点
  メニューという操作列の組み合わせを表します。
- 例 B（常時パンくず付き）は R1132 に対応し、上段に常時表示のパンくずを
  置く形の差分を表します。状態表示は `status`（例 A）ではなく `badge`
  （下書き）で示し、両部品の使い分けを可視化しています。
- 例 C（上段ラベル + バッジ + 共有 URL）は R0187 に対応し、上段の小さな
  ラベル + `badge`（ベータ）、右側の操作列を `clipboard`（共有 URL の
  コピー欄）へ差し替えた形を表します。
- メタ情報の折り返し・ボタン群の縦積みへの切り替え（`40rem` 未満）は
  無 JS のため CSS のみで表現しています。実際のブラウザでの表示切り替え・
  クリップボードのコピー動作確認は本 Demo では行っていません。
- 文言・配色は既存のテーマトークンに従い、独自に書いた架空のものです。

関連情報: [Heading](../themes/heading.md) / [Badge](../themes/badge.md) /
[Status](../themes/status.md) / [Icon](../themes/icon.md) /
[Button](../themes/button.md) / [Button Group](../themes/button-group.md) /
[Breadcrumb](../themes/breadcrumb.md) / [Clipboard](../themes/clipboard.md) /
[Menu](../themes/menu.md)
