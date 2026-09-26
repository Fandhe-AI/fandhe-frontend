//! `pricing-comparison-table` block（イシュー #2862。親トラッキング #2861
//! 「Blocks 目的別パーツ拡充ツリー」配下、プラン比較表を実装する目的別
//! パーツの前半。骨格・幅広表・狭幅カードの登録一式を担う。後半（#2863）
//! が native-select による 1 プラン選択の 2 列表示・状態の並記・原案差分
//! メモを追加する）。
//!
//! # 主参照・集約元の扱い
//!
//! 主参照は対応表 ID R0601（カテゴリ見出し行付きの比較表）。集約元
//! R1150（狭幅ティアカード）は本イシューで畳み込む。集約元 R0201（狭幅
//! select 切替）は #2863 が扱う（native-select の `id`/`label for` 配線を
//! 後半でまとめて扱うための意図的な分割、計画「前半と後半の分担」節）。
//! 参照元の文言・配色・アイコン・ファイル名は持ち込まない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じ転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `table` / `button` / `icon` / `card` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 見出しレベルに `H3` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、intro 領域の見出しは
//! [`fandhe_frontend_pre_styled_ui::heading::HeadingLevel::H3`] にする
//! （`comparison_table` 等と同じ判断）。カード内カテゴリ小見出しは見出し
//! レベルを飛ばさないよう `H4` にする。
//!
//! # 行見出しに `row_header` を使う理由（`<th scope="row">`）
//!
//! 機能名は [`fandhe_frontend_pre_styled_ui::table::row_header`]
//! （`<th scope="row">`）へ置く。値セルと同じく `<td>` へ置くと、
//! スクリーンリーダー利用者が値セル間を移動した際に列見出ししか読み
//! 上げられず「どの機能の可否・値か」が判別できない（`comparison_table`
//! と同じ codex-review P1 是正の踏襲）。
//!
//! # カテゴリ見出し行の組み方
//!
//! 各カテゴリの先頭行は `table::row_header` へ `colspan`（プラン数 + 1 の
//! 文字列表記）と `data-blocks-pricing-comparison-table-category` を渡した
//! 1 セル行にする。`row_header` が予約キーとして除去するのは `scope` の
//! みのため `colspan` はそのまま出力される（`ROW_HEADER_RESERVED` 契約、
//! ファイル内ユニットテストで固定）。カテゴリごとに独立した
//! `table::body`（tbody）を出すことで、視覚的な区切りと DOM 構造の区切り
//! を一致させる。
//!
//! # `drop_class_attr` と `data-*` フックの使い分け
//!
//! `heading` / `text` / `table::root` / `button` / `icon` / `card::root` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って
//! 除去する契約を持つため、これらの Demo 固有スタイルフックは
//! `data-blocks-pricing-comparison-table-*` 属性で渡す。素の `div`、
//! `table::header`/`body`/`row`/`cell`/`scroll_area`、
//! `card::header`/`body`/`footer` には `class` がそのまま効くため、名前
//! 空間分離のため `.blocks-pricing-comparison-table-*` クラスを使う。
//!
//! # `styled_text` という別名で import する理由
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`comparison_table`
//! 等と同じ回避方法）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節に従い、本 Demo
//! はフォーム・状態機械を持たない静的な合成例である。CTA ボタンは
//! [`fandhe_frontend_pre_styled_ui::button::button`] の既定
//! `type="button"` のまま使い、送信先・`href` は持たない。決済・契約処理は
//! 利用者自身の Rust コードで実装する
//! （`docs/policy/intentional-non-adoption.md` §3.25）。
//!
//! # 架空データであること
//!
//! プラン名・価格は `crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS`
//! （Starter $9 / Growth $29 / Scale $79）と同じ値を採用する。機能名・カテゴリ名
//! は本 block 固有の架空のものであり、実在の企業名・サービス名・PII は
//! 含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 表の列数（機能名列 + プラン数）を文字列表記した定数。プラン数が
/// [`PLANS`] 固定のため、`colspan` 属性値としてリテラルで持つ（ファイル内
/// ユニットテストで `PLANS.len() + 1` との一致を固定）。
const COL_COUNT_STR: &str = "4";

/// 機能可否・値の表現。
enum FeatureValue {
    /// 含まれる（チェックアイコン）。
    Included,
    /// 含まれない（バツアイコン）。
    Excluded,
    /// 文字表示（例: 上限件数）。
    Text(&'static str),
}

/// 比較表の列見出しに置くプラン 1 件分（架空データ）。
struct Plan {
    name: &'static str,
    price: &'static str,
    cta_label: &'static str,
    /// 強調表示するプランかどうか（`accent-subtle` 背景、[`LAYOUT_CSS`]）。
    featured: bool,
}

/// 機能比較 1 行分。`values` の長さは [`PLANS`] の件数と一致させる
/// （ファイル内ユニットテストで固定する不変条件）。
struct FeatureRow {
    label: &'static str,
    values: &'static [FeatureValue],
}

/// カテゴリ見出し行 1 件分（機能行のグルーピング単位、`table::body` 1 個に
/// 対応する）。
struct Category {
    label: &'static str,
    rows: &'static [FeatureRow],
}

/// プラン列 3 件。価格は `crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS`
/// （Starter $9 / Growth $29 / Scale $79）と同じ値を直書きする（`const` の
/// 配列添字アクセスより明快なため）。
const PLANS: [Plan; 3] = [
    Plan {
        name: "Starter",
        price: "$9",
        cta_label: "Starter を選ぶ",
        featured: false,
    },
    Plan {
        name: "Growth",
        price: "$29",
        cta_label: "Growth を選ぶ",
        featured: true,
    },
    Plan {
        name: "Scale",
        price: "$79",
        cta_label: "Scale を選ぶ",
        featured: false,
    },
];

/// カテゴリ 2 件・機能行合計 5 件（架空データ）。
const CATEGORIES: [Category; 2] = [
    Category {
        label: "基本機能",
        rows: &[
            FeatureRow {
                label: "プロジェクト数",
                values: &[
                    FeatureValue::Text("3 件"),
                    FeatureValue::Text("無制限"),
                    FeatureValue::Text("無制限"),
                ],
            },
            FeatureRow {
                label: "ストレージ容量",
                values: &[
                    FeatureValue::Text("5GB"),
                    FeatureValue::Text("100GB"),
                    FeatureValue::Text("1TB"),
                ],
            },
            FeatureRow {
                label: "カスタムドメイン",
                values: &[
                    FeatureValue::Excluded,
                    FeatureValue::Included,
                    FeatureValue::Included,
                ],
            },
        ],
    },
    Category {
        label: "サポート",
        rows: &[
            FeatureRow {
                label: "優先サポート窓口",
                values: &[
                    FeatureValue::Excluded,
                    FeatureValue::Included,
                    FeatureValue::Included,
                ],
            },
            FeatureRow {
                label: "専任担当者",
                values: &[
                    FeatureValue::Excluded,
                    FeatureValue::Excluded,
                    FeatureValue::Included,
                ],
            },
        ],
    },
];

/// 可否を表す自作の抽象チェック/バツ図形。意味を持つアイコンのため
/// `label` を指定する（参照元のアイコン形状・内部識別子は持ち込まない）。
fn feature_value_icon(value: &FeatureValue) -> Node {
    match value {
        FeatureValue::Included => icon(
            &IconProps {
                size: Size::Sm,
                label: Some("含まれる"),
                ..IconProps::default()
            },
            vec![("data-blocks-pricing-comparison-table-value", "included")],
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
            vec![("data-blocks-pricing-comparison-table-value", "excluded")],
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

/// プラン 1 列分の列見出し（`<th scope="col">`）。プラン名・価格・CTA を
/// 縦に並べる（親 issue のレイアウト仕様）。
fn plan_column_header(plan: &Plan) -> Node {
    let col = if plan.featured {
        "featured"
    } else {
        "standard"
    };
    let cta_variant = if plan.featured {
        ButtonVariant::Solid
    } else {
        ButtonVariant::Outline
    };
    table::column_header(
        vec![("data-blocks-pricing-comparison-table-col", col)],
        vec![div(
            vec![("class", "blocks-pricing-comparison-table-plan-head")],
            vec![
                span(vec![], vec![text(plan.name)]),
                styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![("data-blocks-pricing-comparison-table-price", "")],
                    vec![text(plan.price)],
                ),
                button::button(
                    &ButtonProps {
                        variant: cta_variant,
                        ..ButtonProps::default()
                    },
                    vec![("data-blocks-pricing-comparison-table-cta", "")],
                    vec![text(plan.cta_label)],
                ),
            ],
        )],
    )
}

/// カテゴリ見出し行（`colspan` でプラン列をまたぐ 1 セル行）。
fn category_row(category: &Category) -> Node {
    table::row(
        vec![],
        vec![table::row_header(
            vec![
                ("colspan", COL_COUNT_STR),
                ("data-blocks-pricing-comparison-table-category", ""),
            ],
            vec![text(category.label)],
        )],
    )
}

/// 機能比較 1 行分（`table::row`）。
fn feature_table_row(row: &FeatureRow) -> Node {
    let mut cells: Vec<Node> = vec![table::row_header(
        vec![("data-blocks-pricing-comparison-table-feature", "")],
        vec![text(row.label)],
    )];
    for (value, plan) in row.values.iter().zip(PLANS.iter()) {
        let col = if plan.featured {
            "featured"
        } else {
            "standard"
        };
        let value_node = match value {
            FeatureValue::Text(label) => span(
                vec![("data-blocks-pricing-comparison-table-value", "text")],
                vec![text(*label)],
            ),
            included_or_excluded => feature_value_icon(included_or_excluded),
        };
        cells.push(table::cell(
            vec![("data-blocks-pricing-comparison-table-col", col)],
            vec![value_node],
        ));
    }
    table::row(vec![], cells)
}

/// 幅広表示（R0601）: カテゴリ見出し行付きの比較表。
fn table_view() -> Node {
    let mut header_cells: Vec<Node> = vec![table::column_header(vec![], vec![text("機能")])];
    header_cells.extend(PLANS.iter().map(plan_column_header));

    let bodies: Vec<Node> = CATEGORIES
        .iter()
        .map(|category| {
            let mut rows = vec![category_row(category)];
            rows.extend(category.rows.iter().map(feature_table_row));
            table::body(vec![], rows)
        })
        .collect();

    let mut table_children = vec![
        table::caption(vec![], vec![text("プラン別の機能比較")]),
        table::header(vec![], vec![table::row(vec![], header_cells)]),
    ];
    table_children.extend(bodies);

    let table_node = table::root(
        TableProps {
            variant: TableVariant::Outline,
            ..TableProps::default()
        },
        vec![],
        table_children,
    );

    div(
        vec![("data-blocks-pricing-comparison-table-view", "table")],
        vec![table::scroll_area(
            vec![
                ("role", "region"),
                ("aria-label", "プラン比較表"),
                ("tabindex", "0"),
            ],
            vec![table_node],
        )],
    )
}

/// 狭幅表示（R1150）: プランごとのカード。カテゴリごとに小見出し（`H4`）+
/// 機能一覧を並べる。
fn plan_card(plan: &Plan) -> Node {
    let cta_variant = if plan.featured {
        ButtonVariant::Solid
    } else {
        ButtonVariant::Outline
    };

    let mut body_children: Vec<Node> = Vec::new();
    for (i, category) in CATEGORIES.iter().enumerate() {
        body_children.push(heading(
            HeadingLevel::H4,
            &HeadingProps {
                size: HeadingSize::Sm,
                weight: HeadingWeight::Bold,
            },
            vec![],
            vec![text(category.label)],
        ));
        let items: Vec<Node> = category
            .rows
            .iter()
            .map(|row| {
                let value = &row.values[i];
                let value_node = match value {
                    FeatureValue::Text(label) => span(
                        vec![("data-blocks-pricing-comparison-table-value", "text")],
                        vec![text(format!("{}: {label}", row.label))],
                    ),
                    included_or_excluded => div(
                        vec![("class", "blocks-pricing-comparison-table-card-item")],
                        vec![feature_value_icon(included_or_excluded), text(row.label)],
                    ),
                };
                el("li", vec![], vec![value_node])
            })
            .collect();
        body_children.push(el("ul", vec![], items));
    }

    card::root(
        CardProps {
            variant: CardVariant::Outline,
            ..CardProps::default()
        },
        vec![("data-blocks-pricing-comparison-table-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text(plan.name)]),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![("data-blocks-pricing-comparison-table-price", "")],
                        vec![text(plan.price)],
                    ),
                ],
            ),
            card::body(vec![], body_children),
            card::footer(
                vec![],
                vec![button::button(
                    &ButtonProps {
                        variant: cta_variant,
                        ..ButtonProps::default()
                    },
                    vec![("data-blocks-pricing-comparison-table-cta", "")],
                    vec![text(plan.cta_label)],
                )],
            ),
        ],
    )
}

/// 狭幅表示（R1150）本体。プラン数分のカードを並べる。
fn cards_view() -> Node {
    div(
        vec![("data-blocks-pricing-comparison-table-view", "cards")],
        PLANS.iter().map(plan_card).collect(),
    )
}

/// `pricing-comparison-table` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。intro 領域 + 幅広表 + 狭幅カードの 3 領域を縦に並べる（表示
/// 切替は [`LAYOUT_CSS`] の `@media` が担う）。
pub fn demo() -> Node {
    let intro = div(
        vec![("class", "blocks-pricing-comparison-table-intro")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("プランを比較する")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-pricing-comparison-table-lead", "")],
                vec![text("機能ごとの違いを一目で確認できます。")],
            ),
        ],
    );

    div(
        vec![("class", "blocks-pricing-comparison-table-layout")],
        vec![intro, table_view(), cards_view()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/pricing-comparison-table/",
    title: "pricing-comparison-table",
    category: BlockCategory::Pricing,
    rust_source: "crates/docs-site/src/blocks/marketing/pricing/pricing_comparison_table.rs",
    demo_class: "blocks-pricing-comparison-table",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `pricing_comparison_table` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。色・余白はすべて既存トークン
/// （`--fandhe-color-*`・`--fandhe-space-*`）のみを使う。狭幅（48rem 未満）で
/// 表からカードへ切り替える。
const LAYOUT_CSS: &str = "\
.blocks-pricing-comparison-table-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-pricing-comparison-table-intro {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"table\"][data-part=\"scroll-area\"] {\n  width: 100%;\n}\n\
[data-blocks-pricing-comparison-table-view=\"table\"] [data-scope=\"table\"][data-part=\"root\"] {\n  min-width: 40rem;\n}\n\
.blocks-pricing-comparison-table-plan-head {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"table\"][data-part=\"column-header\"][data-blocks-pricing-comparison-table-col],\n[data-scope=\"table\"][data-part=\"cell\"][data-blocks-pricing-comparison-table-col] {\n  text-align: center;\n}\n\
[data-scope=\"table\"][data-part=\"row-header\"][data-blocks-pricing-comparison-table-feature] {\n  text-align: left;\n  font-weight: 600;\n}\n\
[data-scope=\"table\"][data-part=\"row-header\"][data-blocks-pricing-comparison-table-category] {\n  background: var(--fandhe-color-bg-subtle);\n  text-align: left;\n  font-weight: 600;\n}\n\
[data-scope=\"table\"][data-part=\"column-header\"][data-blocks-pricing-comparison-table-col=\"featured\"],\n[data-scope=\"table\"][data-part=\"cell\"][data-blocks-pricing-comparison-table-col=\"featured\"] {\n  background: var(--fandhe-color-accent-subtle);\n}\n\
[data-scope=\"icon\"][data-part=\"root\"][data-blocks-pricing-comparison-table-value=\"excluded\"] {\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-pricing-comparison-table-card-item {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-pricing-comparison-table-view=\"cards\"] {\n  display: none;\n  grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr));\n  gap: var(--fandhe-space-4);\n}\n\
@media (max-width: 47.99rem) {\n  [data-blocks-pricing-comparison-table-view=\"table\"] {\n    display: none;\n  }\n  [data-blocks-pricing-comparison-table-view=\"cards\"] {\n    display: grid;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, CATEGORIES, PLANS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 6 種の部品を出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"table\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
            "data-scope=\"card\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// `scope="col"` の数が 1 + プラン数（3）= 4 であること。
    #[test]
    fn column_header_count_matches_plans_plus_one() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"scope="col""#).count(), PLANS.len() + 1);
    }

    /// カテゴリ見出し行の `colspan="4"` がカテゴリ数（2）件あること。
    #[test]
    fn category_rows_have_colspan_matching_col_count() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"colspan="4""#).count(),
            CATEGORIES.len(),
            "colspan should equal PLANS.len() + 1"
        );
    }

    /// `scope="row"` の総数がカテゴリ数 + 機能行の総数であること。
    #[test]
    fn row_header_count_matches_categories_plus_feature_rows() {
        let html = render(&demo());
        let feature_row_count: usize = CATEGORIES.iter().map(|c| c.rows.len()).sum();
        assert_eq!(
            html.matches(r#"scope="row""#).count(),
            CATEGORIES.len() + feature_row_count
        );
    }

    /// すべての `FeatureRow.values.len()` が `PLANS.len()` と一致すること。
    #[test]
    fn feature_rows_match_plan_count() {
        for category in CATEGORIES.iter() {
            for row in category.rows.iter() {
                assert_eq!(row.values.len(), PLANS.len());
            }
        }
    }

    /// CTA ボタンが `type="button"` で計 `PLANS.len() * 2`（表とカード）個、
    /// `type="submit"` が 0 個であること。
    #[test]
    fn cta_buttons_are_type_button() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"type="button""#).count(), PLANS.len() * 2);
        assert_eq!(html.matches(r#"type="submit""#).count(), 0);
    }

    /// 可否アイコンが `role="img"` + 意味のある `aria-label` を持つこと。
    #[test]
    fn value_icons_are_labelled() {
        let html = render(&demo());
        assert!(html.contains(r#"aria-label="含まれる""#));
        assert!(html.contains(r#"aria-label="含まれない""#));
        assert!(html.contains(r#"role="img""#));
    }

    /// 非対話・安全性の不変条件（`<form>`・`id=` 属性・`data:` URI・
    /// `<script` を持たないこと）。
    #[test]
    fn demo_never_contains_forbidden_markup() {
        let html = render(&demo());
        for absent in ["<form", " id=\"", "data:", "<script"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`comparison_table` 等と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-pricing-comparison-table-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-pricing-comparison-table-layout"
        );
    }

    /// [`super::LAYOUT_CSS`] が `@media` による表/カード切替を両方の view
    /// セレクタで宣言していること。
    #[test]
    fn layout_css_declares_view_switch() {
        assert!(super::LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
        assert!(
            super::LAYOUT_CSS.contains(r#"[data-blocks-pricing-comparison-table-view="table"]"#)
        );
        assert!(
            super::LAYOUT_CSS.contains(r#"[data-blocks-pricing-comparison-table-view="cards"]"#)
        );
    }

    /// [`super::LAYOUT_CSS`] が `<` を含まないこと（`push_css` の検証観点の
    /// 事前固定）。
    #[test]
    fn layout_css_has_no_angle_bracket() {
        assert!(!super::LAYOUT_CSS.contains('<'));
    }
}
