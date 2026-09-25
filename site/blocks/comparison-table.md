# comparison-table

`heading` / `text` / `badge` / `table` / `icon` / `button` の 6 部品を合成
した比較表レイアウトです。上部に中央寄せのタグライン・見出し・説明文を
置き、その下に自社と競合を列、機能を行とする本物の表（`<table>`）を
並べます。列見出しには製品名と抽象図形のマークを表示し、セルには可否
アイコンまたは短い値（「無制限」「10 件」等）を入れます。表の下には
CTA ボタンを置きます。

- 静的表示です。`<form>` は使用せず、データ取得・送信も行いません。
- 文言・製品名はすべて架空のもので、実在の企業・サービス名ではありません。
- 自社と競合 1 社の 2 列表、競合 2 社の 3 列表の 2 インスタンスを並記して
  います。
- 狭い幅では表が横スクロールします（`table` の scroll-area パーツ）。
- チェック/バツのアイコンには読み上げ用ラベル（`aria-label`）を付けて
  います。

集約元は 2 件（対応表 ID R0433: 2 列表 / R0434: 3 列表）です。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 機能可否の表現。`Text` は「無制限」「10 件」のような文字値表示に使う
/// （モジュール doc「可否アイコンに `label` を指定する理由」節参照）。
enum FeatureValue {
    /// 含まれる（チェックアイコン）。
    Included,
    /// 含まれない（バツアイコン）。
    Excluded,
    /// 文字表示（例: 上限件数）。
    Text(&'static str),
}

/// 比較表の列見出しに置く製品 1 件分（架空データ）。
struct Product {
    name: &'static str,
    /// 自社列かどうか。`true` の列は強調表示（モジュール doc「自社列の
    /// 強調」節）される。
    ours: bool,
}

/// 機能比較 1 行分。`values` の長さは対象インスタンスの製品数と一致させる
/// （ファイル内ユニットテストで固定する不変条件）。
struct FeatureRow {
    label: &'static str,
    values: &'static [FeatureValue],
}

/// 2 列インスタンス（R0433）の製品列。自社 1 列 + 競合 1 社。
const TWO_PRODUCTS: [Product; 2] = [
    Product {
        name: "Fandhe Kit",
        ours: true,
    },
    Product {
        name: "Verdant Foundry",
        ours: false,
    },
];

/// 2 列インスタンス（R0433）の機能行。
const TWO_FEATURES: [FeatureRow; 4] = [
    FeatureRow {
        label: "プロジェクト数",
        values: &[FeatureValue::Text("無制限"), FeatureValue::Text("10 件")],
    },
    FeatureRow {
        label: "既定エスケープ",
        values: &[FeatureValue::Included, FeatureValue::Excluded],
    },
    FeatureRow {
        label: "外部依存ゼロの描画コア",
        values: &[FeatureValue::Included, FeatureValue::Excluded],
    },
    FeatureRow {
        label: "有償サポート窓口",
        values: &[FeatureValue::Excluded, FeatureValue::Included],
    },
];

/// 3 列インスタンス（R0434）の製品列。自社 1 列 + 競合 2 社（可否パターンを
/// 2 列インスタンスと変える）。
const THREE_PRODUCTS: [Product; 3] = [
    Product {
        name: "Fandhe Kit",
        ours: true,
    },
    Product {
        name: "Verdant Foundry",
        ours: false,
    },
    Product {
        name: "Trellisworks Co.",
        ours: false,
    },
];

/// 3 列インスタンス（R0434）の機能行。
const THREE_FEATURES: [FeatureRow; 4] = [
    FeatureRow {
        label: "プロジェクト数",
        values: &[
            FeatureValue::Text("無制限"),
            FeatureValue::Text("25 件"),
            FeatureValue::Text("10 件"),
        ],
    },
    FeatureRow {
        label: "既定エスケープ",
        values: &[
            FeatureValue::Included,
            FeatureValue::Excluded,
            FeatureValue::Excluded,
        ],
    },
    FeatureRow {
        label: "SSR/SSG 両対応",
        values: &[
            FeatureValue::Included,
            FeatureValue::Included,
            FeatureValue::Excluded,
        ],
    },
    FeatureRow {
        label: "有償サポート窓口",
        values: &[
            FeatureValue::Excluded,
            FeatureValue::Included,
            FeatureValue::Excluded,
        ],
    },
];

/// 可否を表す自作の抽象チェック/バツ図形（意味を持つアイコンのため
/// `label` を指定する。ロゴ相当のマーク〔[`product_mark`]〕と異なり
/// `None` にしない。参照元のアイコン形状・内部識別子は持ち込まない、
/// モジュール doc「可否アイコンに `label` を指定する理由」節）。
fn feature_value_icon(value: &FeatureValue) -> Node {
    match value {
        FeatureValue::Included => icon(
            &IconProps {
                size: Size::Sm,
                label: Some("含まれる"),
                ..IconProps::default()
            },
            vec![("data-blocks-comparison-table-value", "included")],
            vec![
                el(
                    "circle",
                    vec![
                        ("cx", "12"),
                        ("cy", "12"),
                        ("r", "9"),
                        ("fill", "none"),
                        ("stroke", "currentColor"),
                        ("stroke-width", "1.5"),
                    ],
                    vec![],
                ),
                el(
                    "path",
                    vec![
                        ("d", "M8 12.5l2.5 2.5L16 9"),
                        ("fill", "none"),
                        ("stroke", "currentColor"),
                        ("stroke-width", "1.5"),
                        ("stroke-linecap", "round"),
                        ("stroke-linejoin", "round"),
                    ],
                    vec![],
                ),
            ],
        ),
        FeatureValue::Excluded => icon(
            &IconProps {
                size: Size::Sm,
                label: Some("含まれない"),
                ..IconProps::default()
            },
            vec![("data-blocks-comparison-table-value", "excluded")],
            vec![
                el(
                    "path",
                    vec![
                        ("d", "M8 8l8 8"),
                        ("stroke", "currentColor"),
                        ("stroke-width", "1.5"),
                        ("stroke-linecap", "round"),
                    ],
                    vec![],
                ),
                el(
                    "path",
                    vec![
                        ("d", "M16 8l-8 8"),
                        ("stroke", "currentColor"),
                        ("stroke-width", "1.5"),
                        ("stroke-linecap", "round"),
                    ],
                    vec![],
                ),
            ],
        ),
        FeatureValue::Text(_) => span(vec![], vec![]),
    }
}

/// ロゴ相当の抽象図形（製品列見出しの装飾、モジュール doc「ロゴ相当の
/// マーク」節）。`seed` により円・六角形・菱形を切り替え、製品ごとに
/// 見た目を変える。装飾のため `label: None` を指定する。
fn product_mark(seed: usize) -> Node {
    let shape = match seed % 3 {
        0 => el(
            "circle",
            vec![
                ("cx", "12"),
                ("cy", "12"),
                ("r", "8"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "1.5"),
            ],
            vec![],
        ),
        1 => el(
            "polygon",
            vec![
                ("points", "12,3 20,8 20,16 12,21 4,16 4,8"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "1.5"),
            ],
            vec![],
        ),
        _ => el(
            "polygon",
            vec![
                ("points", "12,3 20,12 12,21 4,12"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "1.5"),
            ],
            vec![],
        ),
    };
    icon(
        &IconProps {
            size: Size::Sm,
            label: None,
            ..IconProps::default()
        },
        vec![("data-blocks-comparison-table-mark", "")],
        vec![shape],
    )
}

/// 製品 1 列分の列見出し（`<th scope="col">`）。自社列は accent 背景
/// （[`LAYOUT_CSS`]）+「自社」badge で強調する。
fn product_column_header(product: &Product, seed: usize) -> Node {
    let col = if product.ours { "ours" } else { "theirs" };
    let mut head_children: Vec<Node> =
        vec![product_mark(seed), span(vec![], vec![text(product.name)])];
    if product.ours {
        head_children.push(badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Solid,
                ..BadgeProps::default()
            },
            vec![("data-blocks-comparison-table-recommended", "")],
            vec![text("自社")],
        ));
    }
    table::column_header(
        vec![("data-blocks-comparison-table-col", col)],
        vec![div(
            vec![("class", "blocks-comparison-table-col-head")],
            head_children,
        )],
    )
}

/// 機能比較 1 行分（`table::row`）。`products` は列の `ours`/`theirs`
/// フックを決めるためだけに使う（機能名列を除いた列数と `row.values` の
/// 長さが一致する契約、ファイル内ユニットテストで固定）。
fn feature_table_row(row: &FeatureRow, products: &[Product]) -> Node {
    let mut cells: Vec<Node> = vec![table::cell(
        vec![("data-blocks-comparison-table-feature", "")],
        vec![text(row.label)],
    )];
    for (value, product) in row.values.iter().zip(products.iter()) {
        let col = if product.ours { "ours" } else { "theirs" };
        let value_node = match value {
            FeatureValue::Text(label) => span(
                vec![("data-blocks-comparison-table-value", "text")],
                vec![text(*label)],
            ),
            included_or_excluded => feature_value_icon(included_or_excluded),
        };
        cells.push(table::cell(
            vec![("data-blocks-comparison-table-col", col)],
            vec![value_node],
        ));
    }
    table::row(vec![], cells)
}

/// 変種 1 件分（見出し領域 + 比較表 + CTA）。`id` は
/// `data-blocks-comparison-table-instance`/`-scroll`/`-table` 値
/// （`"two"`/`"three"`）。
#[allow(clippy::too_many_arguments)]
fn table_instance(
    id: &'static str,
    products: &'static [Product],
    features: &'static [FeatureRow],
    heading_text: &str,
    lead: &str,
    caption_text: &str,
    scroll_label: &'static str,
    cta_label: &str,
) -> Node {
    let intro = div(
        vec![("class", "blocks-comparison-table-intro")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-comparison-table-eyebrow", "")],
                vec![text("比較")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text(heading_text)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-comparison-table-lead", "")],
                vec![text(lead)],
            ),
        ],
    );

    let mut header_cells: Vec<Node> = vec![table::column_header(vec![], vec![text("機能")])];
    header_cells.extend(
        products
            .iter()
            .enumerate()
            .map(|(i, product)| product_column_header(product, i)),
    );

    let body_rows: Vec<Node> = features
        .iter()
        .map(|row| feature_table_row(row, products))
        .collect();

    let table_node = table::root(
        TableProps {
            variant: TableVariant::Outline,
            ..TableProps::default()
        },
        vec![("data-blocks-comparison-table-table", id)],
        vec![
            table::caption(vec![], vec![text(caption_text)]),
            table::header(vec![], vec![table::row(vec![], header_cells)]),
            table::body(vec![], body_rows),
        ],
    );

    let scroll = table::scroll_area(
        vec![
            ("data-blocks-comparison-table-scroll", id),
            ("role", "region"),
            ("aria-label", scroll_label),
            ("tabindex", "0"),
        ],
        vec![table_node],
    );

    let cta = div(
        vec![("class", "blocks-comparison-table-cta")],
        vec![button::button(
            &ButtonProps::default(),
            vec![("data-blocks-comparison-table-cta", "")],
            vec![text(cta_label)],
        )],
    );

    div(
        vec![("data-blocks-comparison-table-instance", id)],
        vec![intro, scroll, cta],
    )
}

/// `comparison-table` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「2 参照 ID の畳み込み方」節）。2 列（R0433）・
/// 3 列（R0434）の 2 インスタンスを縦に並べる。
pub fn demo() -> Node {
    let two = table_instance(
        "two",
        &TWO_PRODUCTS,
        &TWO_FEATURES,
        "自社と競合を比較する",
        "主要な機能の違いを一目で確認できます。",
        "自社と競合 1 社の機能比較",
        "自社と競合の比較表",
        "無料で試す",
    );
    let three = table_instance(
        "three",
        &THREE_PRODUCTS,
        &THREE_FEATURES,
        "3 社で比較する",
        "競合 2 社との違いをまとめました。",
        "自社と競合 2 社の機能比較",
        "自社と競合 2 社の比較表",
        "無料で試す",
    );

    div(
        vec![("class", "blocks-comparison-table-layout")],
        vec![two, three],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0433/R0434。出典の固有名・ファイル名は記載しません）
からの意図的な差分は次のとおりです。

- 集約元 2 件（2 列表・3 列表）を 1 ページに縦並記しました。
- ロゴは実在ブランドを模さず、円・六角形・菱形の抽象図形にしました。
- `table` に行見出しパーツ（`<th scope="row">`）が無いため、機能名は
  `<td>`（`table::cell`）に置き、太字は block 固有 CSS で表現しました。
- 見出しレベルは `h3` にしました（ページ側が `## Demo` として `h2` を
  出すため）。
- 自社列の強調は既存トークン（`--fandhe-color-accent-subtle`）と「自社」
  badge のみで表現し、参照元の配色・装飾は持ち込みませんでした。
- CTA ボタンは `button` 部品の既定 `type="button"` のまま使い、送信は
  行いません。
- 2 インスタンスを並記するため `id`/`aria-labelledby`/`aria-controls` は
  出力しません。
- 文言・製品名・機能名はすべて独自に書き下ろしたもので、実在の企業・
  サービス名ではありません。配色は既存テーマトークンにそのまま従います。
