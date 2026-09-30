//! `cart-line-item-table` block（イシュー #3028。親 #3027 は規模 L のため
//! #3028（骨格・主要領域・登録一式）/ #3029（残り領域・状態表示・原稿
//! 仕上げ）へ 2 分割。本ファイルは #3028 範囲を担う）。
//! Ecommerce / Cart カテゴリ最初の block（列見出し付きの明細表を持つ
//! カート画面）。
//!
//! # 使用部品
//!
//! `heading` / `image` / `text` / `native-select` / `button` / `separator`
//! / `data-list` の 7 部品のみを合成する（[`BLOCK`] の `parts` に一致
//! させる契約）。新しい UI 部品は追加しない。
//!
//! # レイアウト（対応表 ID のみ記載、主参照 R0325）
//!
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、対応表 ID のみを記す（`page_heading_avatar.rs` と同じ
//! 扱い）。見出し → 列見出し行（商品 / 数量 / 価格 / 合計）→ 同じ列割りの
//! 商品行 3 件 → 区切り線 → 小計・合計の data-list + 購入手続きボタン
//! （右寄せ）。狭幅（`@container` 40rem 以下）では列見出し行を隠し、各行を
//! 「画像 | 情報」の 2 段へ組み替える。
//!
//! # 数量セレクトのアクセシブルネーム（`field::label` を使わない理由）
//!
//! 列見出し `<span>` は `<th>` ではなく `<select>` とプログラム的に
//! 関連付かないため、`native_select` 単体に `aria-label` を渡して
//! アクセシブルネームを持たせる。`field::label` を使うと使用部品に
//! `field` が増えるため採らない（計画で確定済みの判断）。
//!
//! # `text` の `class` 剥離への対応
//!
//! [`fandhe_frontend_pre_styled_ui::text::text`] は呼び出し側が渡した
//! `class` 属性を無条件に除去するため
//! （`page_heading_avatar.rs`「メタ行を素の `<p>` で組み立てる理由」と
//! 同じ制約）、レイアウトフックが要る要素（列見出し・狭幅用ラベル）は
//! `span`/`el` で直接組み立て、`text::text` は独立した内容表示にのみ使う。
//!
//! # 静的表示・`<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンは `button::button` 既定の `type="button"` のまま
//! 用い、送信処理・送信先は一切持たない。docs-site は無 JS のため選択に
//! 応じた再計算はできず、数量 `<select>` は `disabled` にして初期値のまま
//! 固定する（変更可能に見えて金額が追従しない不整合を避ける。PR #3461 レビュー
//! 指摘対応）。行削除・追加サマリ行・在庫状況等の状態表示は #3029 で追加。
//!
//! # ダミー素材について
//!
//! 商品画像は [`dummy_assets::PRODUCT_SRC`]（モノトーン抽象図形の SVG、
//! `build.rs` がビルド時に書き出す）を使う。商品名・バリエーション・価格は
//! 独自に書いた架空の文言であり、実在の商品・企業とは無関係。小計・合計は
//! 各行合計の和と手で一致させている。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::native_select::{self, FieldIds, FieldProps, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self, TextProps, TextVariant, TextWeight};

/// カート明細行 1 件分の架空データ。
struct LineItem {
    /// 行番号ラベル（`data-blocks-cart-line-item-table-row` の値。select の
    /// `id` 一意化にも同じ文字列を使う）。固定 3 行のため `&'static str`
    /// リテラルで持ち、実行時の文字列生成（`Box::leak` 等）を避ける。
    row: &'static str,
    name: &'static str,
    variant: &'static str,
    unit_price: &'static str,
    total_price: &'static str,
}

/// 明細 3 行（小計はこの 3 行の `total_price` の和と手で一致させている）。
const LINE_ITEMS: &[LineItem] = &[
    LineItem {
        row: "1",
        name: "リネンシャツ",
        variant: "ネイビー / M",
        unit_price: "¥5,800",
        total_price: "¥5,800",
    },
    LineItem {
        row: "2",
        name: "コットンパンツ",
        variant: "ベージュ / L",
        unit_price: "¥6,200",
        total_price: "¥6,200",
    },
    LineItem {
        row: "3",
        name: "キャンバストートバッグ",
        variant: "オフホワイト",
        unit_price: "¥3,400",
        total_price: "¥3,400",
    },
];

/// 見出し（H2、[`heading`] 部品）。docs ページ自体が H1 を持つため block 内
/// は H2 以下とする（`page_heading_avatar.rs` と同型の判断）。
fn page_heading() -> Node {
    heading::heading(
        HeadingLevel::H2,
        &HeadingProps {
            size: HeadingSize::Lg,
            ..HeadingProps::default()
        },
        vec![],
        vec![text("ショッピングカート")],
    )
}

/// 列見出しセル 1 個（素の `<span>`。[`LAYOUT_CSS`] のグリッド列割りに
/// フックする `data-blocks-cart-line-item-table-col` に加え、`role="columnheader"`
/// で表の列見出しとして支援技術に伝える）。
fn column_header(col: &'static str, label: &'static str) -> Node {
    span(
        vec![
            ("data-blocks-cart-line-item-table-col", col),
            ("role", "columnheader"),
        ],
        vec![text(label)],
    )
}

/// 列見出し行（広幅では通常表示、狭幅では [`LAYOUT_CSS`] の
/// `@container` で [`cell_label`] と同じ clip 手法により視覚的にのみ隠す。
/// `display: none` にすると `role="columnheader"` の見出しごとアクセシビ
/// リティツリーから消え、狭幅時に `role="table"` を付けた目的である列見出し
/// との関連付けがテーブルナビゲーションで得られなくなるため（PR #3461
/// レビュー指摘対応）。`role="row"` で [`row_group`] 配下の商品行と同じ行
/// として関連付ける。
fn column_headers() -> Node {
    div(
        vec![
            ("class", "blocks-cart-line-item-table-head"),
            ("role", "row"),
        ],
        vec![
            column_header("product", "商品"),
            column_header("qty", "数量"),
            column_header("price", "価格"),
            column_header("total", "合計"),
        ],
    )
}

/// 商品サムネイル画像。
fn product_thumbnail(name: &str) -> Node {
    image::image(
        &ImageProps {
            shape: ImageShape::Rounded,
            fit: ImageFit::Cover,
            ..ImageProps::new(dummy_assets::PRODUCT_SRC, name)
        },
        vec![("data-blocks-cart-line-item-table-thumb", "")],
    )
}

/// 商品名・バリエーションの情報列。
fn product_info(item: &LineItem) -> Node {
    div(
        vec![("class", "blocks-cart-line-item-table-info")],
        vec![
            text::text(
                &TextProps {
                    weight: TextWeight::Medium,
                    ..TextProps::default()
                },
                vec![],
                vec![text(item.name)],
            ),
            text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(item.variant)],
            ),
        ],
    )
}

/// 商品列（サムネイル + 情報）。`role="cell"` で表セルとして関連付ける。
fn product_cell(item: &LineItem) -> Node {
    div(
        vec![
            ("class", "blocks-cart-line-item-table-product"),
            ("role", "cell"),
        ],
        vec![product_thumbnail(item.name), product_info(item)],
    )
}

/// セルラベル（数量・価格・合計の各値に列の意味を関連付ける）。広幅では
/// [`LAYOUT_CSS`] の clip 手法で視覚的にのみ隠し（`display: none` ではなく
/// DOM 上に残すため、スクリーンリーダーは値の直前に読み上げる）、狭幅の
/// `@container` では通常表示へ切り替える（PR #3461 レビュー指摘対応。
/// `<table>`/`<th>` へ組み替えず、`@container` に応じたセル並べ替えを保った
/// まま列見出しとの意味的関連付けだけを補う判断）。
///
/// # 表としての行・列関連付け（WAI-ARIA `table` ロール、PR #3461 追加指摘対応）
///
/// 上記のセルラベルは各値の列名を補うのみで、行数・現在位置・列見出しとの
/// 関連付けといった表としての操作性までは支援技術に伝わらない
/// （Codex P2 指摘）。`<table>`/`<th>`/`<td>` への置き換えは、`@container`
/// による「狭幅で行を画像＋情報の 2 段へ組み替える」レイアウトと両立しない
/// （多くのブラウザは `display` が `table`/`table-row`/`table-cell` 以外へ
/// 上書きされた要素の暗黙 table ロールを外す）ため、既存の `div`/`span`
/// 構造を保ったまま [`role_table_wrapper`] 以下で `role="table"` /
/// `role="rowgroup"` / `role="row"` / `role="columnheader"` /
/// `role="cell"` を付与し、支援技術のテーブルナビゲーションで行・列を
/// 把握できるようにする。
fn cell_label(label: &'static str) -> Node {
    span(
        vec![("class", "blocks-cart-line-item-table-cell-label")],
        vec![text(label)],
    )
}

/// 数量 `native_select`（[`native_select`] 部品）。行ごとに `id` を一意化
/// し、`aria-label` でアクセシブルネームを持たせる（モジュール doc
/// 「数量セレクトのアクセシブルネーム」節参照）。option は 1〜3 個を表示
/// しつつ `disabled` にして選択値を 1 個に固定する（モジュール doc
/// 「静的表示・`<form>` を使わない」節参照。無 JS のため選択操作しても
/// 金額を追従できず、操作可能に見える不整合を避ける）。
fn quantity_select(item: &LineItem) -> Node {
    let id = format!("blocks-cart-line-item-table-qty-{}", item.row);
    let field = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: true,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let aria_label = format!("{} の数量", item.name);
    let options: Vec<Node> = (1..=3)
        .map(|n| {
            let value = n.to_string();
            let mut attrs = vec![("value", value.as_str())];
            if n == 1 {
                attrs.push(("selected", ""));
            }
            el("option", attrs, vec![text(n.to_string())])
        })
        .collect();
    native_select::native_select(
        &NativeSelectProps::default(),
        &field,
        vec![("aria-label", aria_label.as_str())],
        options,
    )
}

/// 数量セル（狭幅ラベル + select）。`role="cell"` で表セルとして関連付ける。
fn quantity_cell(item: &LineItem) -> Node {
    div(
        vec![
            ("class", "blocks-cart-line-item-table-cell"),
            ("data-blocks-cart-line-item-table-col", "qty"),
            ("role", "cell"),
        ],
        vec![cell_label("数量"), quantity_select(item)],
    )
}

/// 単価セル。`role="cell"` で表セルとして関連付ける。
fn price_cell(item: &LineItem) -> Node {
    div(
        vec![
            ("class", "blocks-cart-line-item-table-cell"),
            ("data-blocks-cart-line-item-table-col", "price"),
            ("role", "cell"),
        ],
        vec![cell_label("価格"), text(item.unit_price)],
    )
}

/// 行合計セル。`role="cell"` で表セルとして関連付ける。
fn total_cell(item: &LineItem) -> Node {
    div(
        vec![
            ("class", "blocks-cart-line-item-table-cell"),
            ("data-blocks-cart-line-item-table-col", "total"),
            ("role", "cell"),
        ],
        vec![cell_label("合計"), text(item.total_price)],
    )
}

/// 商品行 1 件。`role="row"` で [`row_group`] 配下の表行として関連付ける。
fn line_item_row(item: &LineItem) -> Node {
    div(
        vec![
            ("class", "blocks-cart-line-item-table-row"),
            ("data-blocks-cart-line-item-table-row", item.row),
            ("role", "row"),
        ],
        vec![
            product_cell(item),
            quantity_cell(item),
            price_cell(item),
            total_cell(item),
        ],
    )
}

/// 商品行一覧。`role="rowgroup"`（HTML `<tbody>` 相当）で
/// [`column_headers`] の行と区別しつつ表構造の一部として関連付ける。
fn line_item_rows() -> Node {
    div(
        vec![
            ("class", "blocks-cart-line-item-table-rows"),
            ("role", "rowgroup"),
        ],
        LINE_ITEMS.iter().map(line_item_row).collect(),
    )
}

/// [`column_headers`] と [`line_item_rows`] を包む `role="table"` の外枠。
/// ARIA table ロールでは所有要素が `row`/`rowgroup` のみであることが
/// 期待されるため、見出し・区切り線・小計等を含む [`demo`] の最上位 `div`
/// とは別に、表本体だけを囲む専用ラッパーを設ける。
fn role_table_wrapper() -> Node {
    div(
        vec![
            ("class", "blocks-cart-line-item-table-table"),
            ("role", "table"),
            ("aria-label", "カート明細"),
        ],
        vec![column_headers(), line_item_rows()],
    )
}

/// 小計・合計 + 購入手続きボタン（右寄せ）。小計・合計は税送料を含まない
/// 明細合計のみ（送料・税・割引行は #3029 で追加予定）。
fn summary() -> Node {
    let subtotal_item = data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text("小計")]),
            data_list::item_value(vec![], vec![text("¥15,400")]),
        ],
    );
    let total_item = data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text("合計")]),
            data_list::item_value(vec![], vec![text("¥15,400")]),
        ],
    );
    let list = data_list::root(
        DataListProps {
            orientation: DataListOrientation::Horizontal,
            ..DataListProps::default()
        },
        vec![],
        vec![subtotal_item, total_item],
    );
    let checkout_button = button::button(
        &ButtonProps {
            variant: ButtonVariant::Solid,
            ..ButtonProps::default()
        },
        vec![],
        vec![text("購入手続きへ進む")],
    );
    div(
        vec![("class", "blocks-cart-line-item-table-summary")],
        vec![list, checkout_button],
    )
}

/// `cart-line-item-table` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-cart-line-item-table-layout")],
        vec![
            page_heading(),
            role_table_wrapper(),
            separator::separator(&SeparatorProps::default(), vec![]),
            summary(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/cart-line-item-table/",
    title: "cart-line-item-table",
    category: BlockCategory::Cart,
    rust_source: "crates/docs-site/src/blocks/ecommerce/cart/cart_line_item_table.rs",
    demo_class: "blocks-cart-line-item-table",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Native Select",
            path: "/themes/native-select/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Data List",
            path: "/themes/data-list/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `cart_line_item_table` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。`container-type: inline-size`
/// を持つ独自コンテナ名で `@container` を切り替える。狭幅では列見出し行を
/// 隠し、各行を「画像 | 情報」の 2 段へ組み替え、セル内ラベルを表示して
/// 列の意味を保つ。
const LAYOUT_CSS: &str = "\
.blocks-cart-line-item-table-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-cart-line-item-table;\n}\n\
.blocks-cart-line-item-table-head,\n.blocks-cart-line-item-table-row {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr) 6rem 6rem 6rem;\n  gap: var(--fandhe-space-4);\n  align-items: center;\n}\n\
.blocks-cart-line-item-table-head {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n  border-bottom: 1px solid var(--fandhe-color-border);\n  padding-block: var(--fandhe-space-2);\n}\n\
.blocks-cart-line-item-table-row {\n  border-bottom: 1px solid var(--fandhe-color-border);\n  padding-block: var(--fandhe-space-4);\n}\n\
.blocks-cart-line-item-table-product {\n  display: flex;\n  gap: var(--fandhe-space-3);\n  align-items: center;\n  min-width: 0;\n}\n\
.blocks-cart-line-item-table-info {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
img[data-scope=\"image\"][data-blocks-cart-line-item-table-thumb] {\n  width: 4rem;\n  height: 4rem;\n  flex-shrink: 0;\n}\n\
.blocks-cart-line-item-table-cell-label {\n  position: absolute;\n  width: 1px;\n  height: 1px;\n  padding: 0;\n  margin: -1px;\n  overflow: hidden;\n  clip: rect(0, 0, 0, 0);\n  white-space: nowrap;\n  border-width: 0;\n}\n\
.blocks-cart-line-item-table-summary {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-end;\n  gap: var(--fandhe-space-4);\n}\n\
@container blocks-cart-line-item-table (max-width: 40rem) {\n  .blocks-cart-line-item-table-head {\n    position: absolute;\n    width: 1px;\n    height: 1px;\n    padding: 0;\n    margin: -1px;\n    overflow: hidden;\n    clip: rect(0, 0, 0, 0);\n    white-space: nowrap;\n    border-width: 0;\n  }\n  .blocks-cart-line-item-table-row {\n    grid-template-columns: 4rem minmax(0, 1fr);\n  }\n  .blocks-cart-line-item-table-product {\n    grid-column: 1 / -1;\n  }\n  .blocks-cart-line-item-table-cell {\n    grid-column: 2;\n  }\n  .blocks-cart-line-item-table-cell-label {\n    position: static;\n    width: auto;\n    height: auto;\n    padding: 0;\n    margin: 0 var(--fandhe-space-1) 0 0;\n    overflow: visible;\n    clip: auto;\n    white-space: normal;\n    color: var(--fandhe-color-fg-muted);\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"image\"",
            "data-scope=\"text\"",
            "data-scope=\"field\" data-part=\"select\"",
            "data-scope=\"button\"",
            "data-scope=\"separator\"",
            "data-scope=\"data-list\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn demo_renders_three_rows_and_column_headers() {
        let html = demo_html();
        for row in ["1", "2", "3"] {
            assert!(html.contains(&format!("data-blocks-cart-line-item-table-row=\"{row}\"")));
        }
        for label in ["商品", "数量", "価格", "合計"] {
            assert!(html.contains(label));
        }
    }

    #[test]
    fn no_form_semantics_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn all_buttons_are_type_button() {
        let html = demo_html();
        let count_open = html.matches("<button").count();
        let count_typed = html.matches("type=\"button\"").count();
        assert!(count_open > 0);
        assert!(count_typed >= count_open, "html={html}");
    }

    #[test]
    fn select_ids_are_unique_and_labelled() {
        let html = demo_html();
        let mut ids: Vec<&str> = Vec::new();
        for chunk in html.split("id=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                ids.push(&chunk[..end]);
            }
        }
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(ids.len(), sorted.len(), "id が重複している: {ids:?}");
        // 3 行分の select（数量ラベル）+ role="table" ラッパーの
        // aria-label="カート明細" の計 4 件（role_table_structure_associates_rows_and_columns
        // 参照）。
        assert_eq!(html.matches("aria-label=\"").count(), 4);
    }

    #[test]
    fn layout_css_is_safe_and_switches_at_narrow_width() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@container blocks-cart-line-item-table"));
        assert!(LAYOUT_CSS.contains(".blocks-cart-line-item-table-head {\n    position: absolute;"));
        assert!(!LAYOUT_CSS.contains(".blocks-cart-line-item-table-head {\n    display: none;"));
        assert!(LAYOUT_CSS.contains("grid-template-columns:"));
    }

    #[test]
    fn thumb_selector_specificity_beats_image_recipe_base() {
        assert!(LAYOUT_CSS
            .contains("img[data-scope=\"image\"][data-blocks-cart-line-item-table-thumb]"));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-cart-line-item-table-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-cart-line-item-table-layout"
        );
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }

    #[test]
    fn uses_shared_dummy_assets() {
        let html = demo_html();
        assert!(html.contains(super::dummy_assets::PRODUCT_SRC));
    }

    /// PR #3461 追加指摘（Codex P2）対応: `<table>`/`<th>`/`<td>` へ組み替え
    /// ずとも、WAI-ARIA `table` ロールで行・列の関連付けが支援技術に伝わる
    /// ことを固定する。列見出し行 1 + 商品行 3 の計 4 行、商品行 1 件あたり
    /// 4 セル（商品・数量・価格・合計）で計 12 セルとなる。
    #[test]
    fn role_table_structure_associates_rows_and_columns() {
        let html = demo_html();
        assert_eq!(html.matches("role=\"table\"").count(), 1);
        assert_eq!(html.matches("role=\"rowgroup\"").count(), 1);
        assert_eq!(html.matches("role=\"row\"").count(), 4, "html={html}");
        assert_eq!(html.matches("role=\"columnheader\"").count(), 4);
        assert_eq!(html.matches("role=\"cell\"").count(), 12, "html={html}");
        assert!(html.contains("aria-label=\"カート明細\""));
    }
}
