//! `order-history-table` block（イシュー #3058、親 #3024。Ecommerce / Order
//! カテゴリ、`order-tracking-progress` に続く 2 件目）。注文ごとに
//! サマリ帯（注文番号・注文日・合計金額・請求書リンク）を置き、その下へ
//! 商品・価格・状態・操作の 4 列を持つ商品明細表を積む合成例。注文 2 件分を
//! 縦に並べる。主参照は R1117（注文ごとのサマリ帯+商品表）1 件のみで、
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`order_tracking_progress`/`profile_detail_datalist` と同じ扱い）。
//!
//! # 使用部品
//!
//! `table` / `data-list` / `link` / `image` / `text` / `visually-hidden` の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。`heading`/`button` は使わない
//! （注文番号はサマリ帯の `data-list` 項目として表示する）。
//!
//! # レイアウト仕様（R1117）
//!
//! 注文 1 件 = [`order_block`] が返す「サマリ帯 + 商品明細表」の組。
//! 商品明細表は `商品`/`価格`/`状態`/`操作` の 4 列を持ち、注文 2 件分
//! （3 行・2 行）を縦に並べる（[`orders`]）。狭い幅では価格・状態の 2 列を
//! 隠し、価格・状態の両方を商品セル内へ表示する（[`LAYOUT_CSS`] の
//! `@container` 規則。価格のみを再表示し状態を再表示しないと、配送状態が
//! ビジュアル・アクセシビリティツリー双方から消失するため、PR #3510 レビュー
//! 指摘を踏まえ両方を商品セル内へ出す）。
//!
//! # 2 状態の並記（無 JS 制約下での折り畳みデモ）
//!
//! docs サイトは JS ハイドレーションを行わないため、ビューポート連動の
//! リサイズ実演はできない。[`table_responsive_stacked`]
//! （`crate::blocks::application::table::table_responsive_stacked`）と同型で、
//! 同一構造の 2 インスタンス（状態 A: 通常幅、状態 B:
//! `max-inline-size: 24rem` で強制的に狭幅化）を並記し、`@container`
//! （コンテナクエリ）で判定することで挙動の違いを静的に見せる。
//!
//! # 価格・状態の二重出力と a11y
//!
//! 価格は「価格セル（広幅用）」と「商品セル内の狭幅用価格テキスト」の、
//! 状態も同様に「状態セル（広幅用）」と「商品セル内の狭幅用状態テキスト」
//! の 2 か所にそれぞれ出力するが、[`LAYOUT_CSS`] の `@container` 条件で
//! 常にどちらか一方だけが `display: none` になる。`display: none` は
//! アクセシビリティツリーからも除外されるため、スクリーンリーダーによる
//! 二重読み上げは起きない（`table_responsive_stacked` モジュール doc
//! 「副次列の二重出力と a11y」節と同じ判断）。
//!
//! # `visually-hidden` の使用箇所
//!
//! 4 箇所で使う: (1) 「操作」列見出しは可視テキストを持たず列名のみを
//! スクリーンリーダーへ供給する（`table_responsive_stacked` と同型）、
//! (2) 各 `<table>` に `caption` 経由でアクセシブルネーム
//! 「注文 #… の商品明細」を与える、(3) 請求書リンク・商品リンクは注文番号・
//! 商品名ごとに同一文言が複数回出現するため、リンク内に補足テキストを足し
//! て一意な名前にする（WCAG 2.4.4）、(4) 商品セル内の狭幅用価格・状態
//! テキストへ「価格 」「状態 」の接頭辞を補い、単位のないテキストの羅列に
//! ならないようにする。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::table::root`]・
//! [`fandhe_frontend_pre_styled_ui::data_list::root`]・
//! [`fandhe_frontend_pre_styled_ui::link::root`]・
//! [`fandhe_frontend_pre_styled_ui::image::image`]・
//! [`fandhe_frontend_pre_styled_ui::text::text`] はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-order-history-table-*`）。レイアウト用ラッパー（フレーム・
//! サマリ帯のコンテナ）は素の `<div>` のため
//! `class="blocks-order-history-table-*"` を使う。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。
//!
//! # `href` は外部絶対 URL のダミー値
//!
//! 請求書リンク・商品リンクの `href` はいずれも `https://example.com/`
//! 配下の架空 URL とする。`href="#"` とサイト内相対パスは
//! `crates/docs-site/tests/blocks_contract.rs`/linkcheck が fail-closed に
//! 検知するため使わない（`order_tracking_progress` と同じ方針）。
//!
//! # ダミー素材について
//!
//! 商品画像は [`dummy_assets::PRODUCT_SRC`] のみを使う（`data:` URI・外部
//! URL は使わない）。商品名・価格・注文番号・日付はすべて架空の値で、
//! 実在の商品・企業・PII は含まない。各注文の合計金額は商品価格の和と
//! 手で一致させている（注文 1: 4,800 + 3,200 + 1,600 = 9,600円、注文 2:
//! 6,400 + 2,000 = 8,400円）。
//!
//! # スコープ外
//!
//! `blocks_contract.rs` への本 block 固有ページテストの追加は見送る。
//! 共通の契約テストが全 block を検証済みで、固有の CSS フックは本ファイル
//! 末尾の単体テストで固定するため（`.claude/rules/out-of-scope-tracking.md`
//! 対応）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// ラベル・値の 1 項目（サマリ帯の `data-list` 項目）。
fn summary_item(label: &'static str, value: &'static str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 商品 1 行分（商品名・価格・状態・商品ページへのリンク先 slug）。
fn item_row(
    name: &'static str,
    price: &'static str,
    status: &'static str,
    slug: &'static str,
) -> Node {
    let product_href = format!("https://example.com/products/{slug}");
    table::row(
        vec![],
        vec![
            table::row_header(
                vec![],
                vec![
                    div(
                        vec![("class", "blocks-order-history-table-product")],
                        vec![
                            image(
                                &ImageProps::new(dummy_assets::PRODUCT_SRC, ""),
                                vec![("data-blocks-order-history-table-product", "")],
                            ),
                            styled_text::text(&TextProps::default(), vec![], vec![text(name)]),
                        ],
                    ),
                    styled_text::text(
                        &TextProps::default(),
                        vec![("data-blocks-order-history-table-inline-price", "")],
                        vec![
                            visually_hidden::root(vec![], vec![text("価格 ")]),
                            text(price),
                        ],
                    ),
                    styled_text::text(
                        &TextProps::default(),
                        vec![("data-blocks-order-history-table-inline-status", "")],
                        vec![
                            visually_hidden::root(vec![], vec![text("状態 ")]),
                            text(status),
                        ],
                    ),
                ],
            ),
            table::cell(
                vec![("data-blocks-order-history-table-secondary", "")],
                vec![text(price)],
            ),
            table::cell(
                vec![("data-blocks-order-history-table-secondary", "")],
                vec![text(status)],
            ),
            table::cell(
                vec![],
                vec![link::root(
                    &product_href,
                    &LinkProps::default(),
                    vec![],
                    vec![
                        text("商品を見る"),
                        visually_hidden::root(vec![], vec![text(format!("（{name}）"))]),
                    ],
                )],
            ),
        ],
    )
}

/// 注文 1 件（サマリ帯 + 商品明細表）を組み立てる。`items` は
/// `(商品名, 価格, 状態, 商品ページ slug)` の組。
fn order_block(
    order_number: &'static str,
    order_date: &'static str,
    total: &'static str,
    items: &[(&'static str, &'static str, &'static str, &'static str)],
) -> Node {
    let invoice_href = format!("https://example.com/invoices/{order_number}");
    let caption_label = format!("注文 #{order_number} の商品明細");
    div(
        vec![("class", "blocks-order-history-table-order")],
        vec![
            div(
                vec![("class", "blocks-order-history-table-summary")],
                vec![
                    data_list::root(
                        DataListProps {
                            orientation: DataListOrientation::Horizontal,
                            size: Size::Sm,
                            ..DataListProps::default()
                        },
                        vec![("data-blocks-order-history-table-summary-list", "")],
                        vec![
                            summary_item("注文番号", order_number),
                            summary_item("注文日", order_date),
                            summary_item("合計金額", total),
                        ],
                    ),
                    link::root(
                        &invoice_href,
                        &LinkProps::default(),
                        vec![],
                        vec![
                            text("請求書を表示"),
                            visually_hidden::root(
                                vec![],
                                vec![text(format!("（注文 #{order_number}）"))],
                            ),
                        ],
                    ),
                ],
            ),
            table::root(
                TableProps::default(),
                vec![("data-blocks-order-history-table-table", "")],
                vec![
                    table::caption(
                        vec![],
                        vec![visually_hidden::root(vec![], vec![text(caption_label)])],
                    ),
                    table::header(
                        vec![],
                        vec![table::row(
                            vec![],
                            vec![
                                table::column_header(vec![], vec![text("商品")]),
                                table::column_header(
                                    vec![("data-blocks-order-history-table-secondary", "")],
                                    vec![text("価格")],
                                ),
                                table::column_header(
                                    vec![("data-blocks-order-history-table-secondary", "")],
                                    vec![text("状態")],
                                ),
                                table::column_header(
                                    vec![],
                                    vec![visually_hidden::root(vec![], vec![text("操作")])],
                                ),
                            ],
                        )],
                    ),
                    table::body(
                        vec![],
                        items
                            .iter()
                            .map(|(name, price, status, slug)| item_row(name, price, status, slug))
                            .collect(),
                    ),
                ],
            ),
        ],
    )
}

/// 注文 2 件分のダミーデータ（状態 A/B で共有する）。注文 1 は全行
/// 「配達済み」、注文 2 は「配送中」と「発送準備中」を混ぜる
/// （モジュール doc「ダミー素材について」節参照）。
fn orders() -> Vec<Node> {
    vec![
        order_block(
            "FD-2026-0912",
            "2026年09月12日",
            "¥9,600",
            &[
                (
                    "折りたたみデスクライト",
                    "¥4,800",
                    "配達済み（9月15日）",
                    "desk-lamp",
                ),
                (
                    "コットンブランケット",
                    "¥3,200",
                    "配達済み（9月15日）",
                    "cotton-blanket",
                ),
                (
                    "セラミックマグ",
                    "¥1,600",
                    "配達済み（9月15日）",
                    "ceramic-mug",
                ),
            ],
        ),
        order_block(
            "FD-2026-0827",
            "2026年08月27日",
            "¥8,400",
            &[
                ("ノートブックカバー", "¥6,400", "配送中", "notebook-cover"),
                (
                    "ワイヤレス充電パッド",
                    "¥2,000",
                    "発送準備中",
                    "wireless-charger",
                ),
            ],
        ),
    ]
}

/// `order-history-table` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-order-history-table-layout")],
        vec![
            div(
                vec![("class", "blocks-order-history-table-frame")],
                orders(),
            ),
            div(
                vec![(
                    "class",
                    "blocks-order-history-table-frame blocks-order-history-table-frame--narrow",
                )],
                orders(),
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/order-history-table/",
    title: "order-history-table",
    category: BlockCategory::Order,
    rust_source: "crates/docs-site/src/blocks/ecommerce/order/order_history_table.rs",
    demo_class: "blocks-order-history-table",
    parts: &[
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Data List",
            path: "/themes/data-list/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
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
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `order_history_table` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
///
/// 閾値 `40rem` は `table_responsive_stacked`/`order_tracking_progress` と
/// 同系統の「4〜5 列テーブルが自然に収まる下限」として選んだ。
const LAYOUT_CSS: &str = "\
.blocks-order-history-table-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-order-history-table-frame {\n  container-type: inline-size;\n  container-name: blocks-order-history-table;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-order-history-table-frame--narrow {\n  max-inline-size: 24rem;\n}\n\
.blocks-order-history-table-order {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-order-history-table-summary {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-4);\n  background: var(--fandhe-color-bg-subtle);\n  border-radius: var(--fandhe-radius-md);\n}\n\
[data-scope=\"data-list\"][data-part=\"root\"][data-blocks-order-history-table-summary-list] {\n  flex-direction: row;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-order-history-table-product {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-order-history-table-product] {\n  width: 4rem;\n  height: 4rem;\n  object-fit: cover;\n  border-radius: var(--fandhe-radius-md);\n  flex-shrink: 0;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-order-history-table-inline-price] {\n  display: none;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-order-history-table-inline-status] {\n  display: none;\n}\n\
@container blocks-order-history-table (max-width: 40rem) {\n  \
[data-blocks-order-history-table-secondary] {\n    display: none;\n  }\n  \
[data-scope=\"text\"][data-part=\"root\"][data-blocks-order-history-table-inline-price] {\n    display: block;\n    color: var(--fandhe-color-fg-muted);\n    font-size: var(--fandhe-font-font-size-sm);\n  }\n  \
[data-scope=\"text\"][data-part=\"root\"][data-blocks-order-history-table-inline-status] {\n    display: block;\n    color: var(--fandhe-color-fg-muted);\n    font-size: var(--fandhe-font-font-size-sm);\n  }\n  \
.blocks-order-history-table-summary {\n    flex-direction: column;\n    align-items: flex-start;\n  }\n\
}\n";

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
            "data-scope=\"table\"",
            "data-scope=\"data-list\"",
            "data-scope=\"link\"",
            "data-scope=\"image\"",
            "data-scope=\"text\"",
            "data-scope=\"visually-hidden\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<table").count(), 4); // 2 states * 2 orders
        assert_eq!(html.matches(r#"scope="row""#).count(), 10); // 2 states * 5 rows (3 + 2)
        assert_eq!(
            html.matches("data-blocks-order-history-table-secondary")
                .count(),
            2 * (2 * 2 + 2 * 5) // 2 states * (2 orders * 2 column headers + 2 secondary cells * 5 rows)
        );
        assert_eq!(html.matches("<caption").count(), 4); // 2 states * 2 orders
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn layout_css_is_safe_and_hides_secondary_columns_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-order-history-table (max-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("max-inline-size: 24rem;"));
    }

    /// `table_responsive_stacked`（PR #3396）の教訓を踏まえた回帰: 狭幅用
    /// 価格テキストの表示切替は `data-scope`/`data-part` を併記したセレクタ
    /// で行う（単一属性セレクタだけでは recipe base に詳細度で負けて
    /// 常時表示のままになり得るため。`text` recipe は base で `display` を
    /// 宣言しないため実害は小さいが、将来 recipe 側が `display` を追加
    /// しても壊れないよう、他 block と同じ堅牢なセレクタ形へ統一する）。
    #[test]
    fn inline_price_toggle_wins_specificity_against_recipe_base() {
        let scoped_selector = r#"[data-scope="text"][data-part="root"][data-blocks-order-history-table-inline-price]"#;
        assert!(
            LAYOUT_CSS.contains(&format!("{scoped_selector} {{\n  display: none;")),
            "wide state must hide the inline price via a scoped selector"
        );
        assert!(
            LAYOUT_CSS.contains(&format!("{scoped_selector} {{\n    display: block;")),
            "narrow state must show the inline price as block (the <p> element default)"
        );
    }

    #[test]
    fn invoice_and_product_links_use_absolute_example_urls() {
        let html = demo_html();
        let mut found_href = false;
        for captured in html.split("href=\"").skip(1) {
            let href_end = captured.find('"').expect("href attribute must be closed");
            let href = &captured[..href_end];
            assert!(
                href.starts_with("https://example.com/"),
                "unexpected href: {href}"
            );
            found_href = true;
        }
        assert!(found_href, "demo should contain at least one href");
    }

    #[test]
    fn operations_column_header_is_screen_reader_only() {
        let html = demo_html();
        // 状態 1 件あたり: 操作列見出し 2 件（注文 2 件分）+ caption 2 件
        // + 請求書リンク補足 2 件 + 商品リンク補足 5 件 + 狭幅用価格の
        // 「価格 」接頭辞 5 件 + 狭幅用状態の「状態 」接頭辞 5 件
        // （いずれも商品行数ぶん）= 21 件。これが広幅・狭幅の 2 状態ぶんで
        // 計 42 件。
        assert_eq!(html.matches("data-scope=\"visually-hidden\"").count(), 42);
    }

    /// PR #3510 レビュー指摘の回帰: 狭幅（40rem 以下）では価格列・状態列の
    /// 両方が隠れるが、商品セル内へ価格だけでなく状態も再表示されるため、
    /// 配送状態がビジュアル・アクセシビリティツリー双方から消えない。
    #[test]
    fn inline_status_is_rendered_alongside_inline_price_for_narrow_width() {
        let html = demo_html();
        assert!(html.contains("data-blocks-order-history-table-inline-status"));
        // 2 状態 * 5 行（3 + 2）= 10 件。
        assert_eq!(
            html.matches("data-blocks-order-history-table-inline-status")
                .count(),
            10
        );
    }

    /// PR #3510 レビュー指摘の回帰: `image()` に `data-*` 属性を渡さないと
    /// `LAYOUT_CSS` の `[data-scope="image"][data-part="root"]
    /// [data-blocks-order-history-table-product]` セレクタが一切マッチせず、
    /// 4rem 固定サイズ等が適用されない。
    #[test]
    fn product_image_carries_css_hook_attribute() {
        let html = demo_html();
        assert!(html.contains("data-blocks-order-history-table-product"));
        let has_img_with_hook = html
            .split("<img")
            .skip(1)
            .all(|segment| match segment.find('>') {
                Some(end) => segment[..end].contains("data-blocks-order-history-table-product"),
                None => false,
            });
        assert!(
            has_img_with_hook,
            "every <img> must carry the product CSS hook attribute"
        );
    }
}
