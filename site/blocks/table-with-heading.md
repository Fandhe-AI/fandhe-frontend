# table-with-heading

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `button` / `table` /
`card` / `badge` / `link` 部品を合成した、見出し＋追加ボタン付きテーブルの
実例です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R1319、集約元は R1320〜R1330・R1334。出典の固有名・
ファイル名は記載しません）。

上部に表題・説明・右端の「追加」ボタンを置き、その下に一覧テーブルを配置
する基本形（例 A）に加え、カード枠で包む形（例 B）、全幅表示（例 C）、
全幅＋内容だけ最大幅にする形（例 D）、縞模様（例 E）、小型大文字の列見出し
（例 F）、固定ヘッダー（例 G）、縦罫線（例 H）、詰めた行間（例 I）、外枠＋
現在プランのバッジ＋選択ボタンの料金表（例 J）の 10 通りを並べています。
役職・メール列（副次列）は狭い幅（`40rem` 未満）では非表示になります
（例 A〜I 共通。例 J の月額・上限列は料金表の主要情報のため対象外で、
狭い幅でも常に表示したままにしています）。縦罫線（例 H）は `table`
部品自体の機能ではなく本 block 固有の CSS フックで表現し、列間の区切りを
表すため先頭列（名前）には付けていません。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信を行いません。ボタンは `type="button"` のまま送信先を持たず、行末の
「編集」リンクは既存 block と同じリポジトリ URL へ張っています。氏名・
メール・プラン名・料金はすべて独自に書いた架空のものであり、実企業名・
実クレデンシャル・PII を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card;
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 既存 block と同じ架空リンク先（`href="#"` を使わないための定数）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// [`dummy_assets::PERSON_NAMES`] と対にした架空メールアドレス（`example.com`。
/// 実ドメイン・実クレデンシャルを含まない）。
const MEMBER_EMAILS: &[&str] = &[
    "haruto.f@example.com",
    "elena.v@example.com",
    "kwame.b@example.com",
    "mei.l@example.com",
    "noor.a@example.com",
    "ola.b@example.com",
    "priya.c@example.com",
    "theo.m@example.com",
];

/// 状態列に使う 3 状態（ラベル + palette）。行番号で周回させる。
const STATUSES: &[(&str, ColorPalette)] = &[
    ("稼働中", ColorPalette::Success),
    ("招待中", ColorPalette::Warning),
    ("停止", ColorPalette::Danger),
];

/// このインスタンスの見せ方を宣言するデータ（[`VARIANTS`] から駆動する）。
struct Variant {
    label: &'static str,
    card: bool,
    full_bleed: bool,
    content_max: bool,
    striped: bool,
    caps_header: bool,
    sticky: bool,
    rules: bool,
    size: Size,
    /// 行数（既定 5。sticky ヘッダーの実演〔G〕のみスクロールを起こすため
    /// 8 に増やす）。
    rows: usize,
}

const DEFAULT_ROWS: usize = 5;

const VARIANTS: &[Variant] = &[
    Variant {
        label: "代表構成",
        card: false,
        full_bleed: false,
        content_max: false,
        striped: false,
        caps_header: false,
        sticky: false,
        rules: false,
        size: Size::Md,
        rows: DEFAULT_ROWS,
    },
    Variant {
        label: "カード枠",
        card: true,
        full_bleed: false,
        content_max: false,
        striped: false,
        caps_header: false,
        sticky: false,
        rules: false,
        size: Size::Md,
        rows: DEFAULT_ROWS,
    },
    Variant {
        label: "全幅",
        card: false,
        full_bleed: true,
        content_max: false,
        striped: false,
        caps_header: false,
        sticky: false,
        rules: false,
        size: Size::Md,
        rows: DEFAULT_ROWS,
    },
    Variant {
        label: "全幅＋内容最大幅",
        card: false,
        full_bleed: true,
        content_max: true,
        striped: false,
        caps_header: false,
        sticky: false,
        rules: false,
        size: Size::Md,
        rows: DEFAULT_ROWS,
    },
    Variant {
        label: "縞模様",
        card: false,
        full_bleed: false,
        content_max: false,
        striped: true,
        caps_header: false,
        sticky: false,
        rules: false,
        size: Size::Md,
        rows: DEFAULT_ROWS,
    },
    Variant {
        label: "小型大文字見出し",
        card: false,
        full_bleed: false,
        content_max: false,
        striped: false,
        caps_header: true,
        sticky: false,
        rules: false,
        size: Size::Md,
        rows: DEFAULT_ROWS,
    },
    Variant {
        label: "固定ヘッダー",
        card: false,
        full_bleed: false,
        content_max: false,
        striped: false,
        caps_header: false,
        sticky: true,
        rules: false,
        size: Size::Md,
        rows: 8,
    },
    Variant {
        label: "縦罫線",
        card: false,
        full_bleed: false,
        content_max: false,
        striped: false,
        caps_header: false,
        sticky: false,
        rules: true,
        size: Size::Md,
        rows: DEFAULT_ROWS,
    },
    Variant {
        label: "詰めた行間",
        card: false,
        full_bleed: false,
        content_max: false,
        striped: false,
        caps_header: false,
        sticky: false,
        rules: false,
        size: Size::Sm,
        rows: DEFAULT_ROWS,
    },
];

/// 列見出し（副次列は `data-blocks-table-with-heading-secondary` を持つ）。
/// 縦罫線（`rule`）は列間の区切りを表すため先頭列には付けない
/// （先頭列に付けると存在しない左端の区切り線が描画されてしまう）。
fn column_headers(caps: bool, rules: bool) -> Node {
    let attrs_for = |secondary: bool, rule: bool| -> Vec<(&'static str, &'static str)> {
        let mut a = Vec::new();
        if secondary {
            a.push(("data-blocks-table-with-heading-secondary", ""));
        }
        if caps {
            a.push(("data-blocks-table-with-heading-caps", ""));
        }
        if rule {
            a.push(("data-blocks-table-with-heading-rule", ""));
        }
        a
    };
    table::row(
        vec![],
        vec![
            table::column_header(attrs_for(false, false), vec![text("名前")]),
            table::column_header(attrs_for(true, rules), vec![text("役職")]),
            table::column_header(attrs_for(true, rules), vec![text("メール")]),
            table::column_header(attrs_for(false, rules), vec![text("状態")]),
            table::column_header(
                attrs_for(false, rules),
                vec![span(
                    vec![("data-blocks-table-with-heading-sr-only", "")],
                    vec![text("編集")],
                )],
            ),
        ],
    )
}

/// 本文 1 行（`row_index` で氏名・メール・状態を周回させる）。
/// 縦罫線（`rule_attr`）は列間の区切りを表すため先頭列（名前）には付けない
/// （`column_headers` と同じ理由）。
fn body_row(row_index: usize, rules: bool) -> Node {
    let name_index = row_index % dummy_assets::PERSON_NAMES.len();
    let job_index = row_index % dummy_assets::JOB_TITLES.len();
    let (status_label, status_palette) = STATUSES[row_index % STATUSES.len()];
    let rule_attr: Vec<(&str, &str)> = if rules {
        vec![("data-blocks-table-with-heading-rule", "")]
    } else {
        vec![]
    };
    let mut role_attr = rule_attr.clone();
    role_attr.push(("data-blocks-table-with-heading-secondary", ""));
    let mut email_attr = rule_attr.clone();
    email_attr.push(("data-blocks-table-with-heading-secondary", ""));
    table::row(
        vec![],
        vec![
            table::cell(vec![], vec![text(dummy_assets::PERSON_NAMES[name_index])]),
            table::cell(role_attr, vec![text(dummy_assets::JOB_TITLES[job_index])]),
            table::cell(email_attr, vec![text(MEMBER_EMAILS[name_index])]),
            table::cell(
                rule_attr.clone(),
                vec![badge::badge(
                    &BadgeProps {
                        variant: BadgeVariant::Subtle,
                        palette: status_palette,
                        ..BadgeProps::default()
                    },
                    vec![],
                    vec![text(status_label)],
                )],
            ),
            table::cell(
                rule_attr,
                vec![link::root(
                    REPO,
                    &LinkProps::default(),
                    vec![],
                    vec![text("編集")],
                )],
            ),
        ],
    )
}

/// A〜I の 1 インスタンス（表題 + 「追加」ボタン + 表）。
fn instance(v: &Variant) -> Node {
    let heading_row = div(
        vec![("data-blocks-table-with-heading-header", "")],
        vec![
            div(
                vec![],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Lg,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("メンバー一覧")],
                    ),
                    styled_text::text(
                        &fandhe_frontend_pre_styled_ui::text::TextProps {
                            variant: TextVariant::Muted,
                            ..Default::default()
                        },
                        vec![],
                        vec![text("チームに参加しているメンバーの一覧です。")],
                    ),
                ],
            ),
            button::button(
                &ButtonProps::default(),
                vec![],
                vec![text("メンバーを追加")],
            ),
        ],
    );
    let heading_row = if v.content_max {
        div(
            vec![("data-blocks-table-with-heading-content-max", "")],
            vec![heading_row],
        )
    } else {
        heading_row
    };

    let rows: Vec<Node> = (0..v.rows).map(|i| body_row(i, v.rules)).collect();
    let table_node = table::root(
        TableProps {
            variant: TableVariant::Line,
            size: v.size,
            striped: v.striped,
            sticky_header: v.sticky,
            ..TableProps::default()
        },
        vec![],
        vec![
            table::header(vec![], vec![column_headers(v.caps_header, v.rules)]),
            table::body(vec![], rows),
        ],
    );
    let mut scroll_attrs: Vec<(&str, &str)> = vec![
        ("role", "region"),
        ("aria-label", "メンバー一覧テーブル"),
        ("tabindex", "0"),
    ];
    if v.sticky {
        scroll_attrs.push(("data-blocks-table-with-heading-scroll", "sticky"));
    }
    let scroll = table::scroll_area(scroll_attrs, vec![table_node]);

    let body = if v.card {
        card::root(
            card::CardProps::default(),
            vec![],
            vec![
                card::header(vec![], vec![heading_row]),
                card::body(vec![], vec![scroll]),
            ],
        )
    } else {
        div(
            vec![("data-blocks-table-with-heading-body", "")],
            vec![heading_row, scroll],
        )
    };

    let mut frame_attrs: Vec<(&str, &str)> = vec![("data-blocks-table-with-heading-frame", "")];
    if v.full_bleed {
        frame_attrs.push(("data-blocks-table-with-heading-full-bleed", ""));
    }

    div(
        vec![("data-blocks-table-with-heading-instance", "")],
        vec![
            el(
                "h3",
                vec![("data-blocks-table-with-heading-section-title", "")],
                vec![text(v.label)],
            ),
            div(frame_attrs, vec![body]),
        ],
    )
}

/// J: 外枠 + 現在プラン + 選択ボタン（R1334）。他インスタンスと骨格が
/// 異なるため専用に組み立てる（モジュール doc「J」節参照）。
fn plans_instance() -> Node {
    const PLANS: &[(&str, &str, &str, bool)] = &[
        ("Starter", "¥0 / 月", "3 プロジェクト", false),
        ("Pro", "¥2,980 / 月", "無制限", true),
        ("Business", "¥9,800 / 月", "無制限 + 監査ログ", false),
    ];
    let heading_row = div(
        vec![("data-blocks-table-with-heading-header", "")],
        vec![div(
            vec![],
            vec![
                heading(
                    HeadingLevel::H3,
                    &HeadingProps {
                        size: HeadingSize::Lg,
                        ..HeadingProps::default()
                    },
                    vec![],
                    vec![text("プラン一覧")],
                ),
                styled_text::text(
                    &fandhe_frontend_pre_styled_ui::text::TextProps {
                        variant: TextVariant::Muted,
                        ..Default::default()
                    },
                    vec![],
                    vec![text("チームの利用状況に合わせてプランを選べます。")],
                ),
            ],
        )],
    );
    let rows: Vec<Node> = PLANS
        .iter()
        .map(|(name, price, limit, current)| {
            let status_cell = if *current {
                table::cell(
                    vec![],
                    vec![badge::badge(
                        &BadgeProps {
                            variant: BadgeVariant::Solid,
                            palette: ColorPalette::Accent,
                            ..BadgeProps::default()
                        },
                        vec![],
                        vec![text("現在のプラン")],
                    )],
                )
            } else {
                table::cell(vec![], vec![])
            };
            table::row(
                vec![],
                vec![
                    table::cell(vec![], vec![text(*name)]),
                    table::cell(vec![], vec![text(*price)]),
                    table::cell(vec![], vec![text(*limit)]),
                    status_cell,
                    table::cell(
                        vec![],
                        vec![button::button(
                            &ButtonProps {
                                variant: ButtonVariant::Outline,
                                size: Size::Sm,
                                disabled: *current,
                                ..ButtonProps::default()
                            },
                            vec![],
                            vec![text("選択")],
                        )],
                    ),
                ],
            )
        })
        .collect();
    let table_node = table::root(
        TableProps {
            variant: TableVariant::Outline,
            ..TableProps::default()
        },
        vec![],
        vec![
            table::header(
                vec![],
                vec![table::row(
                    vec![],
                    vec![
                        table::column_header(vec![], vec![text("プラン")]),
                        table::column_header(vec![], vec![text("月額")]),
                        table::column_header(vec![], vec![text("上限")]),
                        table::column_header(vec![], vec![text("状態")]),
                        table::column_header(
                            vec![],
                            vec![span(
                                vec![("data-blocks-table-with-heading-sr-only", "")],
                                vec![text("操作")],
                            )],
                        ),
                    ],
                )],
            ),
            table::body(vec![], rows),
        ],
    );
    let scroll = table::scroll_area(
        vec![
            ("role", "region"),
            ("aria-label", "プラン一覧テーブル"),
            ("tabindex", "0"),
        ],
        vec![table_node],
    );
    div(
        vec![("data-blocks-table-with-heading-instance", "")],
        vec![
            el(
                "h3",
                vec![("data-blocks-table-with-heading-section-title", "")],
                vec![text("外枠＋現在プラン＋選択ボタン")],
            ),
            div(
                vec![("data-blocks-table-with-heading-frame", "")],
                vec![div(
                    vec![("data-blocks-table-with-heading-body", "")],
                    vec![heading_row, scroll],
                )],
            ),
        ],
    )
}

/// `table-with-heading` の Demo 本体（A〜I + J の 10 インスタンスを縦積みで
/// 並記する。呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    let mut children: Vec<Node> = VARIANTS.iter().map(instance).collect();
    children.push(plans_instance());
    div(
        vec![("class", "blocks-table-with-heading-layout")],
        children,
    )
}
```

## 原案差分メモ

- 例 A（代表構成）は主参照（対応表 ID R1319）を軸に、既定 variant・既定
  size のテーブルを表します。
- 例 B（カード枠）は R1320 に対応し、表題行と表を `card` で包む形の差分を
  表します。
- 例 C（全幅）は R1321 に対応し、フレームの左右余白を 0 にして表を全幅に
  広げる形を表します。
- 例 D（全幅＋内容最大幅）は R1322 に対応し、C に加えて表題行だけを最大幅
  で中央寄せする形を表します。
- 例 E（縞模様）は R1323 に対応し、`table` の `striped` variant を有効化
  した形を表します。
- 例 F（小型大文字見出し）は R1324 に対応し、列見出しへ独自フックで
  小型・大文字表示を適用した形を表します。
- 例 G（固定ヘッダー）は R1328 に対応し、`table` の `sticky_header`
  variant とスクロール枠の最大高を組み合わせた形を表します（行数を 8 に
  増やし、確実にスクロールが発生するようにしています）。
- 例 H（縦罫線）は R1329 に対応し、列間へ独自フックで縦罫線を足した形を
  表します。`table` 部品自体は列区切り線の機能を持たないため、この見せ方
  は本 block 固有の CSS で表現しています。
- 例 I（詰めた行間）は R1330 に対応し、`table` の `size` を `Sm` にした
  形を表します。
- 例 J（外枠＋現在プラン＋選択ボタン）は R1334 に対応し、`Outline`
  variant のテーブルで料金表を表現し、現在プランの行だけ状態列に
  `badge`（Solid）を置き、操作列のボタンを無効化した形を表します。表題行
  右端のボタンは持たず（プラン選択は行ごとの「選択」ボタンが担う）、
  月額・上限列は副次列扱いにせず狭い幅でも常に表示します。
- 役職・メール列を副次列として、狭い幅（`40rem` 未満）では非表示にする
  挙動（対応表 ID R1326）は例 A〜I 共通です（例 J の月額・上限列は対象
  外）。実際のブラウザでの表示切り替え確認は Chrome の DevTools で
  `40rem` 未満・以上の 2 幅を目視確認しています。
- 文言・配色は既存のテーマトークンに従い、独自に書いた架空のものです。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Button](../themes/button.md) / [Table](../themes/table.md) /
[Card](../themes/card.md) / [Badge](../themes/badge.md) /
[Link](../themes/link.md)
