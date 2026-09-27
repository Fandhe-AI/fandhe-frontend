# description-list-horizontal

`fandhe-frontend-pre-styled-ui` の `data-list` / `heading` / `text` /
`button` / `card` / `attachment` / `link` の 7 部品を合成した、左にラベル・
右に値が並ぶ横並び説明リストの実例です。Blocks セクションは新規部品を
追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集
であることに注意してください（主参照は対応表 ID R0888、他 5 件は
R0453/R0454/R0889/R0890/R0891。出典の固有名・ファイル名は記載しません）。

見出し・説明文・操作ボタンの行の下に、ラベル左・値右の Horizontal な
`data_list` を並べます。4 版を縦に並べており、いずれも架空のデモデータ
です。

- 代表構成: 見出し行に一括編集ボタン、値の 1 つとして添付ファイル一覧
  （リポジトリへのリンクつき）を持つ
- 値ごとに編集ボタンを置く版
- カード枠（`card::root`）に入れる版
- 偶数行へ背景色を敷く縞模様の版（区切り線は付けない）

区切り線・縞模様・狭い幅での縦積みはいずれも `data_list` 部品自体の
機能ではなく、本 block 固有の CSS（コンテナクエリ `@container` を含む）
が実現しています。本 Demo は静的な表示例であり、`<form>` 要素は一切
持たず、データの取得・送信・状態管理を行いません。添付ファイルの
リンクは実ファイルを持たないため死リンク（`href="#"`）を避けて本
リポジトリ自身を指し、文言も「リポジトリで確認」として実際の遷移先
どおりに表しています。文言はすべて独自に書いた架空のものであり、
実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::attachment::{
    self, AttachmentRootProps, AttachmentState, AttachmentVariant,
};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::link;
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 添付ファイルリンクの死リンク回避先（`href="#"` を使わない、他 block の
/// 慣例と同型）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 見出し・説明文・（任意の）右端操作を横並びに束ねる。`action` を渡さない
/// 版（B）は見出し行にボタンを出さない。
fn section_header(title: &str, description: &str, action: Option<Node>) -> Node {
    let mut children = vec![div(
        vec![("class", "blocks-description-list-horizontal-header-text")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![],
                vec![text(description)],
            ),
        ],
    )];
    if let Some(action) = action {
        children.push(action);
    }
    div(
        vec![("class", "blocks-description-list-horizontal-header")],
        children,
    )
}

/// ラベル・値の 1 行（`data_list::item` + `item-label` + `item-value`）を
/// 組み立てる。
fn row(label: &str, value_children: Vec<Node>) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], value_children),
        ],
    )
}

/// Horizontal 配置の `data_list::root` を組む。`extra_attrs` は
/// `data-blocks-description-list-horizontal-list`/`-striped` 等の CSS
/// フック用（`root` は `drop_class_attr` で `class` を除去するため）。
fn list<'a>(extra_attrs: Vec<(&'a str, &'a str)>, rows: Vec<Node>) -> Node {
    data_list::root(
        DataListProps {
            orientation: DataListOrientation::Horizontal,
            ..DataListProps::default()
        },
        extra_attrs,
        rows,
    )
}

/// 添付ファイル 1 件（種別ラベル・ファイル名・サイズ・ダウンロードリンク）
/// を組み立てる。
fn attachment_row(kind: &str, file_name: &str, size: &str) -> Node {
    attachment::root(
        AttachmentRootProps {
            variant: AttachmentVariant::File,
            state: AttachmentState::Idle,
            disabled: false,
        },
        vec![],
        vec![
            attachment::media(vec![], vec![text(kind)]),
            attachment::content(
                vec![],
                vec![
                    attachment::name(vec![], vec![text(file_name)]),
                    attachment::meta(vec![], vec![text(size)]),
                ],
            ),
            attachment::actions(
                vec![],
                vec![link::root(
                    REPO,
                    &link::LinkProps {
                        external: true,
                        ..link::LinkProps::default()
                    },
                    vec![("data-blocks-description-list-horizontal-download", "")],
                    vec![text("リポジトリで確認")],
                )],
            ),
        ],
    )
}

/// A: 代表構成（見出し行の一括編集ボタンは R0453 相当 + 添付ファイル一覧
/// は R0891 相当、主参照 R0888）。
fn version_representative() -> Node {
    let edit_button = button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![("data-blocks-description-list-horizontal-edit", "")],
        vec![text("編集")],
    );
    let header = section_header(
        "プロジェクト概要",
        "基本情報と進捗をまとめた説明です。",
        Some(edit_button),
    );
    let rows = vec![
        row("担当者", vec![text("佐藤 円香")]),
        row("役割", vec![text("プロジェクトリード")]),
        row("連絡先", vec![text("project-a@example.test")]),
        row("予算", vec![text("¥3,200,000")]),
        row(
            "概要",
            vec![text(
                "社内向けドキュメント基盤の刷新プロジェクト。既存資料の移行と検索性向上を目的とする。",
            )],
        ),
        row(
            "添付ファイル",
            vec![div(
                vec![(
                    "class",
                    "blocks-description-list-horizontal-attachments",
                )],
                vec![
                    attachment_row("PDF", "要件定義書.pdf", "820 KB"),
                    attachment_row("ZIP", "デザイン素材.zip", "4.1 MB"),
                ],
            )],
        ),
    ];
    div(
        vec![("class", "blocks-description-list-horizontal-section")],
        vec![
            header,
            list(
                vec![("data-blocks-description-list-horizontal-list", "")],
                rows,
            ),
        ],
    )
}

/// B: 値ごとの編集ボタン（R0454）。
fn version_per_row_action() -> Node {
    let header = section_header("連絡先情報", "各項目を個別に編集できます。", None);
    let field = |label: &str, value: &str| {
        let aria_label = format!("{label}を変更");
        row(
            label,
            vec![
                text(value),
                button(
                    &ButtonProps {
                        variant: ButtonVariant::Ghost,
                        size: Size::Sm,
                        ..ButtonProps::default()
                    },
                    vec![
                        ("data-blocks-description-list-horizontal-row-action", ""),
                        ("aria-label", aria_label.as_str()),
                    ],
                    vec![text("変更")],
                ),
            ],
        )
    };
    let rows = vec![
        field("氏名", "山田 太郎"),
        field("部署", "開発本部"),
        field("内線番号", "1234"),
    ];
    div(
        vec![("class", "blocks-description-list-horizontal-section")],
        vec![
            header,
            list(
                vec![("data-blocks-description-list-horizontal-list", "")],
                rows,
            ),
        ],
    )
}

/// C: カード枠（R0889）。
fn version_card() -> Node {
    let rows = vec![
        row("プラン", vec![text("Business")]),
        row("契約期間", vec![text("2026-04-01〜2027-03-31")]),
        row("次回更新", vec![text("2027-03-31")]),
    ];
    card::root(
        CardProps::from(CardVariant::Outline),
        vec![],
        vec![
            card::header(
                vec![("class", "blocks-description-list-horizontal-header")],
                vec![div(
                    vec![("class", "blocks-description-list-horizontal-header-text")],
                    vec![
                        heading(
                            HeadingLevel::H3,
                            &HeadingProps::default(),
                            vec![],
                            vec![text("契約情報")],
                        ),
                        styled_text::text(
                            &TextProps {
                                variant: TextVariant::Muted,
                                size: TextSize::Sm,
                                ..TextProps::default()
                            },
                            vec![],
                            vec![text("現在の契約状況です。")],
                        ),
                    ],
                )],
            ),
            card::body(
                vec![],
                vec![list(
                    vec![("data-blocks-description-list-horizontal-list", "")],
                    rows,
                )],
            ),
        ],
    )
}

/// D: 縞模様（R0890。区切り線は付けない）。
fn version_striped() -> Node {
    let header = section_header(
        "在庫サマリー",
        "偶数行に背景色を敷いて視認性を高めています。",
        None,
    );
    let rows = vec![
        row("商品コード", vec![text("SKU-2048")]),
        row("在庫数", vec![text("128 個")]),
        row("倉庫", vec![text("第 2 倉庫")]),
        row("最終棚卸日", vec![text("2026-09-01")]),
    ];
    div(
        vec![("class", "blocks-description-list-horizontal-section")],
        vec![
            header,
            list(
                vec![("data-blocks-description-list-horizontal-striped", "")],
                rows,
            ),
        ],
    )
}

/// `description-list-horizontal` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-description-list-horizontal-stack")],
        vec![
            version_representative(),
            version_per_row_action(),
            version_card(),
            version_striped(),
        ],
    )
}
```

## 原案差分メモ

- **代表構成（主参照 R0888）**: 見出し行の右端に一括編集ボタン（R0453
  相当）を置き、値の 1 項目として添付ファイル一覧（R0891 相当。種別
  ラベル・ファイル名・サイズ・ダウンロードリンクを持つ）を並べた構成
  です。
- **値ごとの編集ボタン版（R0454）**: 各行の値の隣に Ghost/Sm の小さな
  ボタンを配置し、項目単位で編集操作を示します。
- **カード枠版（R0889）**: `card::root`（Outline）で全体を囲み、
  `card::header` に見出し・説明文を、`card::body` に `data_list` を
  収めます。
- **縞模様版（R0890）**: `data_list` の偶数行へ背景色（`--fandhe-color-
  bg-muted`。Demo 枠自体の背景〔`.blocks-demo` の `--fandhe-color-
  bg-subtle`〕と同色にならないよう 1 段濃い色を使う）を敷き、区切り線は
  付けません。
- 区切り線（罫線）・縞模様・値の右寄せボタンの配置はいずれも
  `data_list` 部品自体の機能ではなく、`data_list` が意図的に非採用として
  いる `divideY`（区切り線ユーティリティ）を block 側 CSS で補う形です。
- 狭い幅（コンテナ幅 36rem 未満相当）では、ラベル・値を横並びから縦積み
  へ切り替えます。判定はビューポート幅ではなく Demo 枠自体の幅を基準に
  する `@container` コンテナクエリで行います。
- 添付ファイルの種別表示（PDF/ZIP）はモノトーンの文字ラベルのみで、
  絵文字や参照元由来のアイコンは持ち込んでいません。添付ファイルの
  リンクは実ファイルを持たないため死リンク回避で本リポジトリ自身を
  指し、文言も「リポジトリで確認」として遷移先どおりに表しています。

関連情報: [Data List](../themes/data-list.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Button](../themes/button.md) /
[Card](../themes/card.md) / [Attachment](../themes/attachment.md) /
[Link](../themes/link.md)
