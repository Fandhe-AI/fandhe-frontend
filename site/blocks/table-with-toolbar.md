# table-with-toolbar

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `input-group` /
`input` / `button` / `icon` / `table` / `badge` / `pagination` /
`scroll-area` / `tabs` の 11 部品を合成した、ツールバー付きテーブルの実例
です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R0645、集約は R0715/R0716/R0717/R0718。出典の固有名・
ファイル名は記載しません）。

版 A（代表構成: 見出し帯「Invoices」・検索・絞り込み/新規作成ボタン・
横スクロール対応の表・フッターの件数表示/ページ送り）を基本形とし、
次の 3 版を並記して見せ方の違いを比較できるようにしています。

- **版 B（期間選択ボタン付き）**: ツールバーに「7 日間/30 日間/90 日間」
  の期間選択ボタン列を追加します（先頭のみ選択済み表示で固定）。
- **版 C（常時縦積み + エクスポート）**: 検索欄・ボタン行が Demo 枠の幅に
  関係なく常時縦積みで、操作列にエクスポートボタンを追加します。
- **版 D（タブ型の絞り込み付き）**: 「すべて/支払済み/未払い/期限超過」の
  4 タブで絞り込み条件を静的に表示します。タブのパネルはすべて空で、表は
  タブの外側に常時 1 つだけ表示します（絞り込みの実処理は UI コンポー
  ネント層の責務外のため行いません）。

版 A の見出し帯・テーブル・フッターは、検索欄・ボタン群が狭い幅では
コンテナクエリ（`@container`）により縦積みへ切り替わります。表・フッター
のデータは 4 版で共有し、各版で異なるのは見出し帯の操作要素のみです。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信を行いません。検索欄は空、ページ送りは 1 ページ目選択・前ページ無効の
固定表示、期間選択ボタンとタブも選択状態を固定表示するのみです。請求書
番号・顧客名・金額・期日はすべて独自に書いた架空のものであり、実企業名・
実クレデンシャル・PII を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, p, section, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::field::{FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::pagination::{self, ItemMode};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::scroll_area;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の幾何アイコン（線画）。`profile_detail_datalist::geo_icon` と同型
/// （モジュール doc「アイコンは自作の単純図形」節参照）。
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

/// 検索アイコン（虫眼鏡）。
fn search_icon() -> Node {
    geo_icon("M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14z M16.5 16.5L21 21")
}

/// 絞り込みアイコン（漏斗）。
fn filter_icon() -> Node {
    geo_icon("M4 5h16 M7 12h10 M10 19h4")
}

/// 新規作成アイコン（プラス）。
fn plus_icon() -> Node {
    geo_icon("M12 5v14 M5 12h14")
}

/// エクスポートアイコン（下向き矢印 + トレイ、版 C 専用）。
fn export_icon() -> Node {
    geo_icon("M12 4v12 M7 11l5 5 5-5 M4 20h16")
}

/// 請求書 1 行分のダミーデータ。
struct InvoiceRow {
    /// 請求書番号（`row_header` に出す一意なキー）。
    number: &'static str,
    /// 顧客名（[`dummy_assets::COMPANY_NAMES`] を周回参照する）。
    customer_index: usize,
    /// 状態ラベル + バッジ配色。
    status: (&'static str, ColorPalette),
    /// 金額（整形処理を書かず固定文字列、`.claude/rules/coding-rust.md`
    /// §3.23 相当の判断: 数値・日時整形は UI コンポーネント層/block の
    /// 責務外）。
    amount: &'static str,
    /// 支払期日（固定文字列）。
    due: &'static str,
}

const ROWS: &[InvoiceRow] = &[
    InvoiceRow {
        number: "INV-2041",
        customer_index: 0,
        status: ("支払済み", ColorPalette::Success),
        amount: "¥128,000",
        due: "2026-08-15",
    },
    InvoiceRow {
        number: "INV-2042",
        customer_index: 1,
        status: ("未払い", ColorPalette::Warning),
        amount: "¥64,500",
        due: "2026-09-01",
    },
    InvoiceRow {
        number: "INV-2043",
        customer_index: 2,
        status: ("支払済み", ColorPalette::Success),
        amount: "¥212,300",
        due: "2026-09-10",
    },
    InvoiceRow {
        number: "INV-2044",
        customer_index: 3,
        status: ("期限超過", ColorPalette::Danger),
        amount: "¥38,900",
        due: "2026-08-28",
    },
    InvoiceRow {
        number: "INV-2045",
        customer_index: 4,
        status: ("支払済み", ColorPalette::Success),
        amount: "¥95,000",
        due: "2026-09-20",
    },
    InvoiceRow {
        number: "INV-2046",
        customer_index: 5,
        status: ("未払い", ColorPalette::Warning),
        amount: "¥171,250",
        due: "2026-09-25",
    },
];

/// 見出し帯左側（表題 + 説明）。
fn title_group() -> Node {
    div(
        vec![("data-blocks-table-with-toolbar-title", "")],
        vec![
            heading(
                HeadingLevel::H2,
                &HeadingProps {
                    size: HeadingSize::Xl,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("Invoices")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("直近の請求書をまとめて確認できます。")],
            ),
        ],
    )
}

/// 期間選択ボタン 1 個（版 B 専用）。`feature_accordion_image::
/// category_button` と同型: 先頭のみ選択済み（`Solid` +
/// `aria-pressed="true"`）、他は非選択（`Outline` + `aria-pressed="false"`）。
/// 全ボタン `disabled: true` の静的固定（モジュール doc「B（期間選択
/// ボタン）」節参照）。
fn period_button(label: &'static str, selected: bool) -> Node {
    let (variant, pressed) = if selected {
        (ButtonVariant::Solid, "true")
    } else {
        (ButtonVariant::Outline, "false")
    };
    button::button(
        &ButtonProps {
            variant,
            size: Size::Sm,
            disabled: true,
            ..ButtonProps::default()
        },
        vec![("aria-pressed", pressed)],
        vec![text(label)],
    )
}

/// 期間選択ボタン列（版 B 専用。「7 日間」を初期選択として固定する）。
fn period_buttons() -> Node {
    div(
        vec![("data-blocks-table-with-toolbar-period", "")],
        vec![
            period_button("7 日間", true),
            period_button("30 日間", false),
            period_button("90 日間", false),
        ],
    )
}

/// 操作ボタン列（絞り込み → [エクスポート（版 C のみ）] → 新規作成）。
fn actions(export: bool) -> Node {
    let mut children = vec![button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            ..ButtonProps::default()
        },
        vec![],
        vec![filter_icon(), text("絞り込み")],
    )];
    if export {
        children.push(button::button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                ..ButtonProps::default()
            },
            vec![],
            vec![export_icon(), text("エクスポート")],
        ));
    }
    children.push(button::button(
        &ButtonProps::default(),
        vec![],
        vec![plus_icon(), text("新規作成")],
    ));
    div(
        vec![("data-blocks-table-with-toolbar-actions", "")],
        children,
    )
}

/// タブ型の絞り込み（版 D 専用）。実物 `tabs::tabs` を使い「すべて」を
/// 初期選択として固定する。全パネルを空にし、表は tabs の外へ常時可視で
/// 置く（モジュール doc「D（タブ型の絞り込み）」節参照）。`search_id` ごと
/// に呼び出し側の `id` が変わるのと同様、本関数は版 D でのみ呼ばれるため
/// 固定 id を持たせてよい（demo 内で 1 回しか呼ばれない契約）。
fn status_tabs() -> Node {
    let items = vec![
        TabItem {
            value: "all",
            trigger: vec![text("すべて")],
            content: vec![],
            disabled: false,
        },
        TabItem {
            value: "paid",
            trigger: vec![text("支払済み")],
            content: vec![],
            disabled: false,
        },
        TabItem {
            value: "unpaid",
            trigger: vec![text("未払い")],
            content: vec![],
            disabled: false,
        },
        TabItem {
            value: "overdue",
            trigger: vec![text("期限超過")],
            content: vec![],
            disabled: false,
        },
    ];
    div(
        vec![("data-blocks-table-with-toolbar-tabs", "")],
        vec![tabs::tabs(
            TabsVariant::Line,
            Size::Sm,
            ColorPalette::Accent,
            &TabsProps {
                id: "blocks-table-with-toolbar-tabs",
                selected: "all",
                orientation: Orientation::Horizontal,
                activation_mode: ActivationMode::Automatic,
                loop_focus: true,
                indicator: false,
            },
            items,
        )],
    )
}

/// 見出し帯右側（検索 + [期間選択（版 B）] + 操作ボタン列）。`search_id` は
/// 呼び出し側（[`variant`]）が版ごとに一意な値を渡し、複数版並記時の
/// `id` 重複（`tests/blocks_contract.rs`）を避ける。
fn toolbar(search_id: &'static str, period: bool, export: bool) -> Node {
    let field = FieldProps {
        id: search_id,
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
    let mut children = vec![input_group::root(
        &group_props,
        vec![("data-blocks-table-with-toolbar-search", "")],
        vec![
            input_group::addon(
                InputGroupAlign::InlineStart,
                &group_props,
                vec![],
                vec![search_icon()],
            ),
            input::input(
                &InputProps::default(),
                &field,
                vec![
                    ("type", "search"),
                    ("placeholder", "請求書を検索"),
                    ("aria-label", "請求書を検索"),
                ],
            ),
        ],
    )];
    if period {
        children.push(period_buttons());
    }
    children.push(actions(export));
    div(
        vec![("data-blocks-table-with-toolbar-toolbar", "")],
        children,
    )
}

/// 見出し帯全体（表題群 + ツールバー）。
fn header(search_id: &'static str, period: bool, export: bool) -> Node {
    div(
        vec![("data-blocks-table-with-toolbar-header", "")],
        vec![title_group(), toolbar(search_id, period, export)],
    )
}

/// 列見出し行（Amount は右寄せの目印として `data-align="end"` を持つ。
/// `table` 部品自体は列寄せの variant を持たないため block 固有の CSS
/// フックとして扱う）。
fn column_headers() -> Node {
    table::row(
        vec![],
        vec![
            table::column_header(vec![], vec![text("請求書番号")]),
            table::column_header(vec![], vec![text("顧客")]),
            table::column_header(vec![], vec![text("状態")]),
            table::column_header(vec![("data-align", "end")], vec![text("金額")]),
            table::column_header(vec![], vec![text("期日")]),
        ],
    )
}

/// 本文 1 行。
fn body_row(row: &InvoiceRow) -> Node {
    let customer =
        dummy_assets::COMPANY_NAMES[row.customer_index % dummy_assets::COMPANY_NAMES.len()];
    let (status_label, status_palette) = row.status;
    table::row(
        vec![],
        vec![
            table::row_header(vec![], vec![text(row.number)]),
            table::cell(vec![], vec![text(customer)]),
            table::cell(
                vec![],
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
            table::cell(vec![("data-align", "end")], vec![text(row.amount)]),
            table::cell(vec![], vec![text(row.due)]),
        ],
    )
}

/// テーブル本体（横スクロール対応の `scroll_area` 包み）。
fn table_section() -> Node {
    let table_node = table::root(
        TableProps {
            variant: TableVariant::Line,
            size: Size::Md,
            ..TableProps::default()
        },
        vec![("data-blocks-table-with-toolbar-table", "")],
        vec![
            table::header(vec![], vec![column_headers()]),
            table::body(vec![], ROWS.iter().map(body_row).collect()),
        ],
    );
    scroll_area::root(
        vec![("data-blocks-table-with-toolbar-scroll", "")],
        vec![scroll_area::viewport(
            vec![("role", "region"), ("aria-label", "請求書一覧")],
            vec![scroll_area::content(vec![], vec![table_node])],
        )],
    )
}

/// フッター（件数表示 + ページ送り。1 ページ目選択・前ページ無効で固定）。
fn footer() -> Node {
    div(
        vec![("data-blocks-table-with-toolbar-footer", "")],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("42 件中 1–6 件を表示")],
            ),
            pagination::root(
                Size::Sm,
                ColorPalette::Accent,
                "請求書ページ",
                vec![("data-blocks-table-with-toolbar-pagination", "")],
                vec![
                    pagination::prev_trigger(ItemMode::Button, true, vec![], vec![text("前へ")]),
                    pagination::item(ItemMode::Button, 1, true, false, vec![], vec![text("1")]),
                    pagination::item(ItemMode::Button, 2, false, false, vec![], vec![text("2")]),
                    pagination::item(ItemMode::Button, 3, false, false, vec![], vec![text("3")]),
                    pagination::ellipsis(vec![], vec![text("…")]),
                    pagination::item(ItemMode::Button, 7, false, false, vec![], vec![text("7")]),
                    pagination::next_trigger(ItemMode::Button, false, vec![], vec![text("次へ")]),
                ],
            ),
        ],
    )
}

/// 版 1 つ分（見出し帯 + [タブ行] + テーブル + フッター）。`kind` は
/// `data-blocks-table-with-toolbar-variant` の値（`"standard"`/`"period"`/
/// `"stacked"`/`"tabs"`）。`search_id` は版ごとに一意な検索欄 `id`
/// （[`toolbar`] rustdoc「`id` 重複を避ける」節参照）。
fn variant(
    kind: &'static str,
    search_id: &'static str,
    period: bool,
    export: bool,
    show_tabs: bool,
) -> Node {
    let mut children = vec![header(search_id, period, export)];
    if show_tabs {
        children.push(status_tabs());
    }
    children.push(table_section());
    children.push(footer());
    section(
        vec![("data-blocks-table-with-toolbar-variant", kind)],
        children,
    )
}

/// `table-with-toolbar` の Demo 本体（版 A〜D を並記。呼び出しごとに同一の
/// `Node` を返す純関数。モジュール doc「4 版の並記」節参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-table-with-toolbar-layout")],
        vec![
            p(
                vec![("class", "blocks-table-with-toolbar-caption")],
                vec![text("代表構成")],
            ),
            variant(
                "standard",
                "blocks-table-with-toolbar-search-a",
                false,
                false,
                false,
            ),
            p(
                vec![("class", "blocks-table-with-toolbar-caption")],
                vec![text("期間選択ボタン付き")],
            ),
            variant(
                "period",
                "blocks-table-with-toolbar-search-b",
                true,
                false,
                false,
            ),
            p(
                vec![("class", "blocks-table-with-toolbar-caption")],
                vec![text("常時縦積み + エクスポート")],
            ),
            variant(
                "stacked",
                "blocks-table-with-toolbar-search-c",
                false,
                true,
                false,
            ),
            p(
                vec![("class", "blocks-table-with-toolbar-caption")],
                vec![text("タブ型の絞り込み付き")],
            ),
            variant(
                "tabs",
                "blocks-table-with-toolbar-search-d",
                false,
                false,
                true,
            ),
        ],
    )
}
```

## 原案差分メモ

- 版 A（代表構成、対応表 ID R0645/R0715）は、見出し帯（表題・説明・検索・
  絞り込み/新規作成ボタン）・テーブル・フッター（件数表示・ページ送り）の
  基本形を表します。検索欄・絞り込み/新規作成ボタンの行と見出し行は、
  Demo 枠の幅が `40rem` 未満のコンテナクエリで縦積みに切り替わります
  （`@media` のビューポート幅ではなく block 自体の描画幅で判定します）。
- 版 B（期間選択ボタン付き、対応表 ID R0716）は、ツールバーへ期間選択
  ボタン列を追加した見せ方です。各ボタンは `aria-pressed` で選択状態を
  示しつつ `disabled` の静的固定とし、参照元の期間区切り定義（日数の
  境界値等）は持ち込まず独自の文言を使っています。
- 版 C（常時縦積み + エクスポート、対応表 ID R0717）は、`@container` の
  横並び切り替えを持たず幅に関係なく常時縦積みにした見せ方です。操作列に
  エクスポートボタン（自作の下矢印 + トレイ線画）を追加しています。
- 版 D（タブ型の絞り込み付き、対応表 ID R0718）は、実物の `tabs` 部品で
  絞り込み条件を静的に表示した見せ方です。**全パネルを空にし、表はタブの
  外側に常時可視で 1 つだけ置いています**。これは非選択パネルへ内容を
  閉じ込める構造（同じ Blocks セクションの `faq-tabbed-accordion` で
  見つかった課題）を避けるための判断です。
- テーブルは `scroll_area` で包み、横にはみ出す幅（`min-width: 42rem`）を
  与えて横スクロールを確認できるようにしています（4 版共通）。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Input Group](../themes/input-group.md) / [Input](../themes/input.md) /
[Button](../themes/button.md) / [Icon](../themes/icon.md) /
[Table](../themes/table.md) / [Badge](../themes/badge.md) /
[Pagination](../themes/pagination.md) /
[Scroll Area](../themes/scroll-area.md) / [Tabs](../themes/tabs.md)
