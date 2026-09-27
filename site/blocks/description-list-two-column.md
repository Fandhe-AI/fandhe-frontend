# description-list-two-column

`fandhe-frontend-pre-styled-ui` の `data-list` / `heading` / `text` /
`attachment` / `button` 部品を合成した、2 カラムの説明リストです。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0893。出典の固有名・ファイル名は記載しません）。

各項目はラベルを値の上に積み、2 列グリッドに並べます。長文の項目
（概要）と添付ファイル一覧は 2 列をまたぐ全幅表示にします。狭い幅
（40rem 未満）では 1 列に戻ります。Demo は集約元 2 件の違いを読み取れる
よう、2 インスタンスを縦に並記します: 見出し + リード文のみの構成
（R0893、主参照）と、見出し行の右側に操作ボタンを置いた構成（R0456）
です。

本 Demo は静的な表示例であり、docs サイトは JS ハイドレーションを行わない
ため、`<form>` 要素は一切持たず、データの取得・送信・状態管理を行いません。
文言・氏名・所属・連絡先・添付ファイル名はすべて独自に書いた架空のもの
であり、実在人物・実企業名・実クレデンシャル・PII を含みません（連絡先は
予約ドメイン `example.com` を使用します）。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, li, p, text, ul, Node};
use fandhe_frontend_pre_styled_ui::attachment::{self, AttachmentRootProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 概要欄の架空の紹介文（検索インデックス容量対策で 2〜3 文に抑える）。
const SUMMARIES: &[&str] = &[
    "在宅勤務を中心とした働き方への切り替えを希望しています。\
     チーム内の定例は既存のオンライン会議のまま継続する想定です。",
    "隣接チームとの兼務を解消し、現チームの業務に専念したいという申請です。\
     引き継ぎ期間として 2 週間を見込んでいます。",
];

/// 添付ファイル 1 件（`media` に拡張子表記、`content` に名前・サイズ、
/// `actions` にダウンロードボタンを置く）。
fn attachment_item(extension: &str, name: &str, size: &str) -> Node {
    li(
        vec![],
        vec![attachment::root(
            AttachmentRootProps::default(),
            vec![],
            vec![
                attachment::media(
                    vec![],
                    vec![styled_text::text(
                        &TextProps {
                            size: TextSize::Xs,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(extension)],
                    )],
                ),
                attachment::content(
                    vec![],
                    vec![
                        attachment::name(vec![], vec![text(name)]),
                        attachment::meta(vec![], vec![text(size)]),
                    ],
                ),
                attachment::actions(
                    vec![],
                    vec![button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            size: Size::Sm,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("ダウンロード")],
                    )],
                ),
            ],
        )],
    )
}

/// 半幅の項目（ラベル + 値）1 件。
fn field(label: &str, value: &str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 全幅の項目（`data-blocks-description-list-two-column-span="full"` 付き）。
fn field_full(label: &str, value_children: Vec<Node>) -> Node {
    data_list::item(
        vec![("data-blocks-description-list-two-column-span", "full")],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], value_children),
        ],
    )
}

/// [`dummy_assets::PERSON_NAMES`] の氏名から架空メールアドレスを導出する
/// （空白をピリオドへ・小文字化。実在するメールとの一致を避けるための
/// 機械的な変換であり、氏名との対応が取れた実例にする）。
fn applicant_email(name: &str) -> String {
    format!("{}@example.com", name.to_lowercase().replace(' ', "."))
}

/// インスタンス 1 件分（`index` で人物・文言をずらす。`with_action` で
/// 見出し行の右側に操作ボタンを持つか切り替える、モジュール doc「2
/// インスタンス併記」参照）。
fn instance(index: usize, with_action: bool) -> Node {
    let applicant = dummy_assets::PERSON_NAMES[index];
    let department = dummy_assets::JOB_TITLES[index % dummy_assets::JOB_TITLES.len()];
    let email = applicant_email(applicant);
    let start_date = "2026-11-01";
    let summary = SUMMARIES[index % SUMMARIES.len()];

    let mut header_children: Vec<Node> = vec![div(
        vec![("data-blocks-description-list-two-column-header-text", "")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("申請内容")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("勤務条件の変更申請です。")],
            ),
        ],
    )];
    // with_action が false のときは第 2 子（ボタン）を挿入しない。空の
    // div を置くと header の `gap` により見出し下へ余分な隙間が生じるため
    // （Cursor Bugbot 指摘、PR #3350）。
    if with_action {
        header_children.push(button::button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                size: Size::Sm,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("編集")],
        ));
    }

    div(
        vec![("data-blocks-description-list-two-column-layout", "")],
        vec![
            div(
                vec![("data-blocks-description-list-two-column-header", "")],
                header_children,
            ),
            data_list::root(
                DataListProps {
                    orientation: DataListOrientation::Vertical,
                    ..DataListProps::default()
                },
                vec![("data-blocks-description-list-two-column-list", "")],
                vec![
                    field("申請者", applicant),
                    field("所属", department),
                    field("連絡先", &email),
                    field("希望開始日", start_date),
                    field_full("概要", vec![text(summary)]),
                    field_full(
                        "添付ファイル",
                        vec![ul(
                            vec![("data-blocks-description-list-two-column-attachments", "")],
                            vec![
                                attachment_item("PDF", "勤務条件変更申請書.pdf", "1.2 MB"),
                                attachment_item("PNG", "現行シフト表.png", "480 KB"),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// caption（並記された各インスタンスの見出し）。
fn caption(label: &str) -> Node {
    p(
        vec![("data-blocks-description-list-two-column-caption", "")],
        vec![text(label)],
    )
}

/// `description-list-two-column` の Demo 本体（2 インスタンス併記）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-description-list-two-column-stack", "")],
        vec![
            caption("2 列の説明リスト（主参照 R0893）"),
            instance(0, false),
            caption("見出し行に操作ボタン（集約元 R0456）"),
            instance(1, true),
        ],
    )
}
```

## 集約元との差分メモ

- R0893（主参照）は見出し + リード文のみの構成です。本 Demo のインスタンス
  A（`with_action: false`）に対応します。
- R0456 は見出し行の右側に「編集」操作ボタンを持つ構成です。本 Demo の
  インスタンス B（`with_action: true`）に対応します。
- 狭い幅ではどちらのインスタンスも項目を 1 列に戻します。
- 文言・配色・アイコンは参照元から持ち込まず、すべて独自に書いた架空の
  ものです。

関連情報: [Data List](../themes/data-list.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Attachment](../themes/attachment.md) /
[Button](../themes/button.md)
