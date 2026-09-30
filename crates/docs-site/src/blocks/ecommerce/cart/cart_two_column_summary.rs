//! `cart-two-column-summary` block（イシュー #3033、親 #3032 の前半。
//! Ecommerce / Cart カテゴリの最初の block）。左に商品行のリスト、右に
//! 注文サマリの 2 カラムで構成するカート画面の骨格と主要領域を実装する。
//! 主参照は対応表 ID R1247（集約元 R0324/R0681/R0682。送料無料進捗バー・
//! 数量選択のヘルプ吹き出し・買い物継続リンク・集計行ヘルプリンクは後半
//! #3034 で追加する、本ファイル末尾のモジュール doc 「後半 #3034 で追加する
//! 領域」節参照）。`_/blocks-intake/` の対応ファイルは本イシュー着手時点で
//! 本 worktree に存在しないため、原稿・本コメントには対応表 ID のみを記し、
//! レイアウトは親 issue #3032 の仕様文に従って独自に組む
//! （`profile-detail-datalist` #2937 と同じ扱い）。
//!
//! # 使用部品
//!
//! `heading` / `image` / `text` / `native-select`（フィールド系は
//! `fandhe_frontend_headless_ui::field` 経由）/ `button` / `separator` /
//! `data-list` / `card` の 8 部品を合成する（[`BLOCK`] の `parts` に一致
//! させる契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs`
//! が検証する）。新しい UI 部品は追加しない。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::card::root`]・
//! [`fandhe_frontend_pre_styled_ui::button::button`]・
//! [`fandhe_frontend_pre_styled_ui::native_select::native_select`]（内部で
//! `fandhe_frontend_headless_ui::field::select` を呼ぶ）・
//! [`fandhe_frontend_pre_styled_ui::data_list::root`]・
//! [`fandhe_frontend_pre_styled_ui::image::image`] はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-cart-two-column-summary-*`）。レイアウト用ラッパー
//! （見出し・カラム・商品行）は素の `<div>` のため
//! `class="blocks-cart-two-column-summary-*"` を使う
//! （`profile-detail-datalist` と同型の判断）。
//!
//! # 狭幅ではサマリが商品一覧の下へ回る（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`
//! （コンテナクエリ）で判定する（`profile-detail-datalist` と同型の
//! パターン）。[`LAYOUT_CSS`] のラッパー
//! `.blocks-cart-two-column-summary-stack` へ `container-type:
//! inline-size` を宣言し、コンテナ幅が `48rem` 未満のとき 2 カラムの
//! `grid-template-columns` を 1 カラムへ切り替える。
//!
//! # Demo 内では `position: sticky` を使わない
//!
//! `.blocks-demo` は `overflow-x: auto` の横スクロールコンテナのため、
//! Demo 内で右カラムを `position: sticky` 追従させても意図どおりに機能
//! しない（`content_article_toc` の判断を踏襲）。実アプリで組み込む際は
//! 呼び出し側で `sticky` を付与してよい旨を原稿の「原案差分メモ」節へ
//! 注記する。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。数量選択は
//! `select` の初期選択値のみを示す静的表示で、削除・購入手続きの各ボタンは
//! `button::button` の既定 `type="button"` のまま用いる。
//!
//! # 数量 `select` の id・アクセシブルネーム
//!
//! 商品行ごとに `FieldProps::id` を一意にし（`.../qty-<n>`）、
//! `aria-label="数量"` を `extra_attrs` で付与する。実際に出力される
//! `<select>` の `id` は `fandhe_frontend_headless_ui::field` の派生規則
//! により `"{id}-control"` になる。
//!
//! # ダミー素材について
//!
//! 商品名・属性・価格・在庫状態は本ファイル内の架空データ（実在の
//! ブランド・商品・PII を含まない）で持つ。商品画像はビルド時生成の
//! 同梱 SVG（[`dummy_assets::PRODUCT_SRC`]）を使う（外部 URL・`data:`
//! URI は使わない）。`alt` は空文字列（`""`）とし、商品名テキストが
//! 隣接して可視のためアクセシブルネームは商品名テキストが担う
//! （`profile-detail-datalist` のアバター画像 `alt` と異なり、
//! 装飾寄りのサムネイルのため空 `alt` を選ぶ判断は W3C の画像代替
//! テキスト決定木「隣接テキストが同じ情報を担うなら装飾扱い」に従う）。
//!
//! # 後半 #3034 で追加する領域（本 PR のスコープ外）
//!
//! 送料無料進捗バー（`progress`、R0682）・数量選択のヘルプ吹き出し
//! （`tooltip`、R0681）・買い物継続 `link`（R0324）・集計行ごとの
//! ヘルプリンク（R1247 の残り）・状態違い（在庫切れ行・空カート等）の
//! 並記は、本 PR の骨格・主要領域が固まった後に別イシュー #3034 で追加
//! する。放置ではなく `site/blocks/cart-two-column-summary.md` の
//! 「原案差分メモ」節に明記して追跡する。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::native_select::{
    native_select, FieldIds, FieldProps, NativeSelectProps,
};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};

/// 架空の商品行データ（商品名, 属性表示, 在庫状態, 価格表示, 初期選択数量）。
/// 実在のブランド・商品・PII は含まない。
const CART_ITEMS: &[(&str, &str, &str, &str, u8)] = &[
    (
        "エルゴノミック メッシュチェア",
        "カラー: グレー / サイズ: M",
        "在庫あり",
        "¥24,800",
        1,
    ),
    (
        "ノイズキャンセリング ヘッドホン",
        "カラー: ブラック",
        "在庫あり",
        "¥18,200",
        2,
    ),
    (
        "ステンレス タンブラー 500ml",
        "カラー: シルバー",
        "入荷待ち（2〜3 週間）",
        "¥3,600",
        1,
    ),
];

/// 注文サマリの集計行（ラベル, 値）。最終行（合計）は
/// [`summary_totals`] 側で強調用の `data-*` を追加する。
const SUMMARY_ROWS: &[(&str, &str)] = &[
    ("小計", "¥46,600"),
    ("送料", "¥600"),
    ("税", "¥4,660"),
    ("合計", "¥51,860"),
];

/// 指定した初期選択数量 `selected` の 1〜5 の `<option>` 列を組み立てる。
fn qty_options(selected: u8) -> Vec<Node> {
    (1..=5u8)
        .map(|n| {
            let mut attrs = vec![("value", n.to_string())];
            if n == selected {
                attrs.push(("selected", String::new()));
            }
            el(
                "option",
                attrs.iter().map(|(k, v)| (*k, v.as_str())).collect(),
                vec![text(n.to_string())],
            )
        })
        .collect()
}

/// 商品行 1 件（サムネイル + 名称/属性/在庫 + 価格/数量選択/削除）。
/// `index` は 0 始まりで、数量 `select` の一意な `id` の派生に使う。
fn item_row(index: usize, name: &str, attrs: &str, stock: &str, price: &str, qty: u8) -> Node {
    let field_id = format!("blocks-cart-two-column-summary-qty-{}", index + 1);
    let field = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    div(
        vec![("class", "blocks-cart-two-column-summary-item")],
        vec![
            image(
                &ImageProps {
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                },
                vec![("data-blocks-cart-two-column-summary-thumb", "")],
            ),
            div(
                vec![("class", "blocks-cart-two-column-summary-item-body")],
                vec![
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Medium,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(name)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(attrs)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(stock)],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-cart-two-column-summary-item-controls")],
                vec![
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Medium,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(price)],
                    ),
                    native_select(
                        &NativeSelectProps::default(),
                        &field,
                        vec![("aria-label", "数量")],
                        qty_options(qty),
                    ),
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            size: Size::Sm,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("削除")],
                    ),
                ],
            ),
        ],
    )
}

/// 商品行と行間の `separator` を束ねた左カラム（商品一覧）。
fn item_list() -> Node {
    let mut children = Vec::new();
    for (index, (name, attrs, stock, price, qty)) in CART_ITEMS.iter().enumerate() {
        if index > 0 {
            children.push(separator(&SeparatorProps::default(), vec![]));
        }
        children.push(item_row(index, name, attrs, stock, price, *qty));
    }
    div(
        vec![("class", "blocks-cart-two-column-summary-items")],
        children,
    )
}

/// 集計行 1 件（`data_list::item` + `item-label` + `item-value`）。最終行
/// （合計）だけ強調用の `data-blocks-cart-two-column-summary-total` を
/// `item` へ付与する（[`LAYOUT_CSS`] が罫線・フォントウェイトで強調する
/// フック）。
fn summary_row(label: &str, value: &str, emphasize: bool) -> Node {
    let attrs = if emphasize {
        vec![("data-blocks-cart-two-column-summary-total", "")]
    } else {
        vec![]
    };
    data_list::item(
        attrs,
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 注文サマリの集計リスト（横並び、最終行のみ強調）。
fn summary_totals() -> Node {
    let last = SUMMARY_ROWS.len() - 1;
    let rows = SUMMARY_ROWS
        .iter()
        .enumerate()
        .map(|(i, (label, value))| summary_row(label, value, i == last))
        .collect();
    data_list::root(
        DataListProps {
            orientation: DataListOrientation::Horizontal,
            ..DataListProps::default()
        },
        vec![("data-blocks-cart-two-column-summary-totals", "")],
        rows,
    )
}

/// 右カラム（注文サマリカード）。
fn summary_card() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-cart-two-column-summary-summary", "")],
        vec![
            card::header(
                vec![],
                vec![card::title(
                    vec![],
                    vec![heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![text("注文サマリ")],
                    )],
                )],
            ),
            card::body(vec![], vec![summary_totals()]),
            card::footer(
                vec![],
                vec![button(
                    &ButtonProps::default(),
                    vec![("data-blocks-cart-two-column-summary-checkout", "")],
                    vec![text("購入手続きへ")],
                )],
            ),
        ],
    )
}

/// `cart-two-column-summary` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-cart-two-column-summary-stack")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("ショッピングカート")],
            ),
            div(
                vec![("class", "blocks-cart-two-column-summary-columns")],
                vec![item_list(), summary_card()],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/cart-two-column-summary/",
    title: "cart-two-column-summary",
    category: BlockCategory::Cart,
    rust_source: "crates/docs-site/src/blocks/ecommerce/cart/cart_two_column_summary.rs",
    demo_class: "blocks-cart-two-column-summary",
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
        Part {
            label: "Card",
            path: "/themes/card/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `cart_two_column_summary` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-cart-two-column-summary-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-cart-two-column-summary;\n}\n\
.blocks-cart-two-column-summary-columns {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr) minmax(16rem, 22rem);\n  gap: var(--fandhe-space-8);\n  align-items: start;\n}\n\
.blocks-cart-two-column-summary-items {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}\n\
.blocks-cart-two-column-summary-item {\n  display: flex;\n  gap: var(--fandhe-space-4);\n  align-items: flex-start;\n}\n\
[data-blocks-cart-two-column-summary-thumb] {\n  width: 6rem;\n  height: 6rem;\n  flex: none;\n}\n\
.blocks-cart-two-column-summary-item-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  flex: 1 1 auto;\n  min-width: 0;\n}\n\
.blocks-cart-two-column-summary-item-controls {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-end;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-cart-two-column-summary-totals] [data-blocks-cart-two-column-summary-total] {\n  border-top: 1px solid var(--fandhe-color-border);\n  padding-top: var(--fandhe-space-2);\n}\n\
[data-blocks-cart-two-column-summary-checkout] {\n  width: 100%;\n}\n\
@container blocks-cart-two-column-summary (max-width: 48rem) {\n  \
.blocks-cart-two-column-summary-columns {\n    grid-template-columns: minmax(0, 1fr);\n  }\n  \
.blocks-cart-two-column-summary-item-controls {\n    align-items: flex-start;\n  }\n\
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
            "data-scope=\"heading\"",
            "data-scope=\"image\"",
            "data-scope=\"text\"",
            "data-scope=\"field\"",
            "data-scope=\"button\"",
            "data-scope=\"separator\"",
            "data-scope=\"data-list\"",
            "data-scope=\"card\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<select").count(), 3);
        assert_eq!(html.matches("<option").count(), 15);
        assert_eq!(html.matches("selected=\"\"").count(), 3);
        assert_eq!(html.matches("data-scope=\"separator\"").count(), 2);
        assert_eq!(html.matches("<button").count(), 4);
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("href=\"#\""));
        assert!(html.contains("../../assets/blocks-demo-product.svg"));
    }

    #[test]
    fn select_ids_are_unique_and_labelled() {
        let html = demo_html();
        for n in 1..=3 {
            let needle = format!("id=\"blocks-cart-two-column-summary-qty-{n}-control\"");
            assert_eq!(
                html.matches(needle.as_str()).count(),
                1,
                "expected exactly one {needle}"
            );
        }
        assert_eq!(html.matches("aria-label=\"数量\"").count(), 3);
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-cart-two-column-summary (max-width: 48rem)"));
    }

    #[test]
    fn item_thumbnail_alt_is_empty_and_name_is_visible_text() {
        let html = demo_html();
        assert!(html.contains(r#"alt="""#));
        for (name, ..) in super::CART_ITEMS {
            assert!(html.contains(name), "missing visible product name: {name}");
        }
    }
}
