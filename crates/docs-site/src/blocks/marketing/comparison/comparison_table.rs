//! `comparison-table` block（イシュー #2825。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、対応表 ID R0433/R0434 の 2 件を
//! 構造の参照元とする合成例。自社と競合を列、機能を行とする本物の表
//! （[`fandhe_frontend_pre_styled_ui::table`]）で比較する製品比較 UI）。
//! 取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! **Marketing / Comparison カテゴリで 3 番目の block**（`super`
//! （`comparison/mod.rs`）参照。1 番目は [`super::comparison_feature_rows`]
//! （イシュー #2823）、2 番目は [`super::comparison_cards`]（イシュー
//! #2822））。
//!
//! [`super::comparison_cards`] がカード並びで機能可否を示すのに対し、本
//! block は列見出しに製品を並べ、機能を行とする表形式で比較する点が異なる。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `table` / `icon` / `button` の 6 部品を合成
//! する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 2 参照 ID の畳み込み方（Demo は 2 インスタンス）
//!
//! - **2 列インスタンス**（R0433 が主参照）: 自社 1 列 + 競合 1 社の
//!   計 2 列。
//! - **3 列インスタンス**（R0434 が主参照）: 自社 1 列 + 競合 2 社の
//!   計 3 列（可否パターンを 2 列インスタンスと変え、比較として読める
//!   ようにする）。
//!
//! `comparison_cards` と同じく、集約元の差分は Demo の縦並記で示す
//! （[`site/blocks/comparison-table.md`] の「原案差分メモ」参照）。
//!
//! # 見出しレベルに `H3` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、intro 領域の見出しは
//! [`fandhe_frontend_pre_styled_ui::heading::HeadingLevel::H3`] にする
//! （`comparison_cards` 等と同じ判断）。表のセル内には `heading` を置かない
//! （列見出しは `<th>` そのものの意味論に任せる、下記「行見出しパーツが
//! 無い」節参照）。
//!
//! # 行見出しパーツが無い理由（`<th scope="row">` を使わない）
//!
//! [`fandhe_frontend_pre_styled_ui::table`] には行見出し用パーツ
//! （`<th scope="row">`）が存在しない。[`fandhe_frontend_pre_styled_ui::
//! table::column_header`] は `scope="col"` を固定で付与し、呼び出し側が
//! `attrs` に `scope` を含めても `drop_reserved` により除去される契約
//! （`table.rs` モジュール doc「セキュリティ不変条件」節）である。このため
//! 機能名は [`fandhe_frontend_pre_styled_ui::table::cell`]（`<td>`）へ置き、
//! 太字は [`LAYOUT_CSS`] のフックで付ける。
//!
//! # ロゴ相当のマーク（実在ブランドを模さない）
//!
//! 列見出しの製品名の上に、自前の抽象図形（円・六角形・菱形）を
//! [`fandhe_frontend_pre_styled_ui::icon::icon`]（`label: None`）で描く。
//! 隣に製品名テキストがあるため装飾扱いとし、`label` を `None` にする
//! （`comparison_cards::feature_value_icon` の可否アイコンとは異なり
//! 情報を運ばないため）。参照元の SVG path・実在ブランドのロゴ・商標は
//! 持ち込まない。
//!
//! # 可否アイコンに `label` を指定する理由（a11y）
//!
//! 可否は本 block の中核情報であり装飾ではないため、
//! [`fandhe_frontend_pre_styled_ui::icon::IconProps::label`]
//! （`Some("含まれる")`/`Some("含まれない")`）を指定して `role="img"` +
//! `aria-label` による意味のある代替テキストを付与する（`comparison_cards`
//! の可否アイコンと同じ判断）。ロゴ相当のマークとは `label` の有無で明確に
//! 区別する。
//!
//! # 自社列の強調
//!
//! 自社列（`ours`）の `column-header`/`cell` は
//! `[data-blocks-comparison-table-col="ours"]`（[`LAYOUT_CSS`] が accent
//! subtle 背景を追加）+ 列見出し内の badge（「自社」）で強調する。競合列は
//! `data-blocks-comparison-table-col="theirs"` のみを持つ（複数競合列でも
//! 同一値、列を個別区別する必要が無いため）。
//!
//! # 横スクロールの実現方法
//!
//! 各表を [`fandhe_frontend_pre_styled_ui::table::scroll_area`]（素の
//! `overflow: auto` ボックス）で包む。3 列インスタンスの表には
//! [`LAYOUT_CSS`] で `min-width`（`42rem`）を与え、狭い幅で確実にはみ出して
//! スクロールさせる。2 列インスタンスにも小さめの `min-width`（`28rem`）を
//! 付ける。`scroll_area` はキーボードで到達できるよう `role="region"` +
//! `aria-label` + `tabindex="0"` を渡す（`table.rs` モジュール doc
//! 「`scroll-area` パーツ」節、フォーカスリングは recipe の Inset 規約で
//! 表示される）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `badge::badge` / `icon::icon` /
//! `button::button` / `table::root` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、これらの
//! Demo 固有スタイルフックは `data-blocks-comparison-table-*` 属性で渡す。
//! `table::header`/`body`/`row`/`column_header`/`cell`/`caption`/
//! `scroll_area` は `drop_class_attr` を経由しないため `class` がそのまま
//! 効くが、他 block と同じく名前空間分離のため `.blocks-comparison-table-*`
//! クラスを使う。ルート class（`blocks-comparison-table-layout`）は
//! [`Block::demo_class`]（`blocks-comparison-table`）とは意図的に別名にする
//! （`comparison_cards` 等と同じ Bugbot 教訓の回避）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # `id`/`aria-labelledby` を出力しない理由
//!
//! 2 インスタンス（2 列/3 列）を同一ページへ並記するため、`id`/
//! `aria-controls`/`aria-labelledby` を出力すると id 重複や宙に浮いた
//! ARIA 参照を生みやすい。本 block はいずれの部品も `id` を要さない構成
//! のため一切出力しない（`comparison_cards` と同じ判断、
//! `crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` の対象）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。CTA ボタンは [`fandhe_frontend_pre_styled_ui::button::button`]
//! の既定 `type="button"` のまま使い、送信先・`href` は持たない。製品名・
//! 機能名・説明はすべて架空のもの（実在の企業名・サービス名・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/comparison-table/",
    title: "comparison-table",
    category: BlockCategory::Comparison,
    rust_source: "crates/docs-site/src/blocks/marketing/comparison/comparison_table.rs",
    demo_class: "blocks-comparison-table",
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
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `comparison_table` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。
///
/// 色はすべて既存トークン（`--fandhe-color-accent`/`-accent-subtle`/
/// `-border`/`-fg-muted`）のみを使う。自社列の強調は
/// `[data-scope="table"]` を前置した複合セレクタで書く（table recipe の
/// `column-header`/`cell` base 規則に詳細度で勝つため、モジュール doc
/// 「自社列の強調」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-comparison-table-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
[data-blocks-comparison-table-instance] {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-comparison-table-intro {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  text-align: center;\n  max-width: 36rem;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-comparison-table-lead] {\n  margin: 0;\n}\n\
[data-scope=\"table\"][data-part=\"scroll-area\"][data-blocks-comparison-table-scroll] {\n  width: 100%;\n  max-width: 56rem;\n}\n\
[data-blocks-comparison-table-table=\"two\"] {\n  min-width: 28rem;\n}\n\
[data-blocks-comparison-table-table=\"three\"] {\n  min-width: 42rem;\n}\n\
.blocks-comparison-table-col-head {\n  display: inline-flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n}\n\
[data-scope=\"table\"][data-part=\"column-header\"][data-blocks-comparison-table-col],\n[data-scope=\"table\"][data-part=\"cell\"][data-blocks-comparison-table-col] {\n  text-align: center;\n}\n\
[data-scope=\"table\"][data-part=\"cell\"][data-blocks-comparison-table-feature] {\n  text-align: left;\n  font-weight: 600;\n}\n\
[data-scope=\"table\"][data-part=\"column-header\"][data-blocks-comparison-table-col=\"ours\"],\n[data-scope=\"table\"][data-part=\"cell\"][data-blocks-comparison-table-col=\"ours\"] {\n  background: var(--fandhe-color-accent-subtle);\n}\n\
[data-scope=\"icon\"][data-part=\"root\"][data-blocks-comparison-table-value=\"excluded\"] {\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-comparison-table-cta {\n  display: flex;\n  justify-content: center;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 6 種の部品を出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"badge\"",
            "data-scope=\"table\"",
            "data-scope=\"icon\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// 2 列・3 列の両インスタンスの表が出力され、`<th scope="col">` の総数が
    /// (1+2)+(1+3)=7 であること。
    #[test]
    fn demo_renders_two_and_three_column_tables() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-blocks-comparison-table-table="two""#)
                .count(),
            1
        );
        assert_eq!(
            html.matches(r#"data-blocks-comparison-table-table="three""#)
                .count(),
            1
        );
        assert_eq!(html.matches(r#"scope="col""#).count(), 7);
    }

    /// 各インスタンスで本文行数 × 製品列数のセル数が一致すること
    /// （`FeatureRow.values.len()` が製品数と一致する不変条件の間接検証）。
    #[test]
    fn feature_rows_match_product_counts() {
        for row in super::TWO_FEATURES.iter() {
            assert_eq!(row.values.len(), super::TWO_PRODUCTS.len());
        }
        for row in super::THREE_FEATURES.iter() {
            assert_eq!(row.values.len(), super::THREE_PRODUCTS.len());
        }
    }

    /// CTA ボタンが `type="button"` で計 2 個、`type="submit"` が 0 個で
    /// あること。
    #[test]
    fn cta_buttons_are_type_button() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"type="button""#).count(), 2);
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

    /// 非対話・安全性の不変条件（`<form>`・`id=` 属性・`data:` URI を
    /// 持たないこと）。`<button` はここでは禁止しない（CTA が正しく
    /// `button` 部品を使っていることの裏付けのため）。
    #[test]
    fn demo_never_contains_forbidden_markup() {
        let html = render(&demo());
        for absent in ["<form", "id=\"", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`comparison_cards` 等と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-comparison-table-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-comparison-table-layout");
    }

    /// [`LAYOUT_CSS`] が横スクロール・中央寄せ・自社列強調の規則を持つこと。
    #[test]
    fn layout_css_declares_scroll_and_centering() {
        assert!(LAYOUT_CSS.contains("min-width: 42rem;"));
        assert!(LAYOUT_CSS.contains("text-align: center;"));
        assert!(LAYOUT_CSS.contains(
            r#"[data-scope="table"][data-part="column-header"][data-blocks-comparison-table-col="ours"],"#
        ));
        assert!(LAYOUT_CSS.contains("background: var(--fandhe-color-accent-subtle);"));
    }

    /// [`LAYOUT_CSS`] のルート規則（2 インスタンス間の縦方向ギャップ）が、
    /// `demo()` が実際に出力するルート class（`.blocks-comparison-table-layout`）
    /// をセレクタとして参照していること。属性セレクタ
    /// （`[data-blocks-comparison-table-root]`）へ誤って書くと、
    /// その属性がどこにも出力されないため常にマッチしない死んだ CSS
    /// ルールになる（レビュー指摘、`comparison_cards` と同じクラス
    /// セレクタ方式に統一する）。
    #[test]
    fn layout_css_root_rule_matches_demo_root_class() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-comparison-table-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}"
        ));
        assert!(!LAYOUT_CSS.contains("[data-blocks-comparison-table-root]"));
    }
}
