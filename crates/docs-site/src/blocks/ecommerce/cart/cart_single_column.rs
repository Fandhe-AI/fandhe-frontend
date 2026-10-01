//! `cart-single-column` block（イシュー #3031、親 #3032。Ecommerce / Cart
//! カテゴリ）。1 カラム構成のカート画面。主参照は対応表 ID R1248（小計のみの
//! 集計 + 削除テキストボタン）、集約元は R1249（淡い面で囲った 4 行の集計:
//! 小計・送料・税・合計）。`_/blocks-intake/` の対応ファイルは本イシュー
//! 着手時点で本 worktree に存在しないため、原稿・本コメントには対応表 ID
//! のみを記し、レイアウトはイシューの仕様文に従って独自に組む
//! （`cart-two-column-summary` #3033 と同じ扱い）。
//!
//! # 使用部品
//!
//! `heading` / `image` / `text` / `native-select` / `button` / `separator` /
//! `data-list` / `link` の 8 部品を合成する（[`BLOCK`] の `parts` に一致
//! させる契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs`
//! が検証する）。新しい UI 部品は追加しない。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::button::button`]・
//! [`fandhe_frontend_pre_styled_ui::native_select::native_select`]（内部で
//! `fandhe_frontend_headless_ui::field::select` を呼ぶ）・
//! [`fandhe_frontend_pre_styled_ui::data_list::root`]・
//! [`fandhe_frontend_pre_styled_ui::image::image`]・
//! [`fandhe_frontend_pre_styled_ui::link::root`] はいずれも `drop_class_attr`
//! で呼び出し側 `class` を除去してから内部 variant クラスと合成するため、
//! これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-cart-single-column-*`）。レイアウト用ラッパー（見出し・
//! 商品行・集計パネル）は素の `<div>` のため
//! `class="blocks-cart-single-column-*"` を使う
//! （`cart-two-column-summary` と同型の判断）。
//!
//! # `@container` による幅判定・広幅での字下げ
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`
//! （コンテナクエリ）で判定する。[`LAYOUT_CSS`] のラッパー
//! `.blocks-cart-single-column-stack` へ `container-type: inline-size` を
//! 宣言し、コンテナ幅が `36rem` 以上のときのみ集計パネルをサムネイル幅ぶん
//! 字下げする（狭幅では字下げしない）。グリッドによる多カラム化は一切
//! 行わず、どの幅でも 1 列構成を保つ。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。数量に応じた
//! 小計・合計の再計算は持たないため、数量 `select`・削除ボタン・購入手続き
//! ボタンはいずれもネイティブ `disabled`（+ ボタン側は `data-disabled`）で
//! 無効化し、操作しても `SUMMARY_ROWS_*` と矛盾しないようにする
//! （`cart_dialog.rs`/`cart_line_item_table.rs`/`cart_two_column_summary.rs`
//! と同型の判断、P2 指摘対応）。リンクはすべて固定のリポジトリ URL
//! （[`REPO_URL`]）を指し、`href="#"` は使わない。
//!
//! # 数量 `select` の id・アクセシブルネーム
//!
//! 商品行ごとに `FieldProps::id` を一意にする（`blocks-cart-single-column-
//! {a|b}-qty-<n>`。形 A・B の双方に同じ商品行を描くため、形のキーを含めて
//! id 衝突を避ける）。`aria-label` も商品名を含めて行ごとに区別する
//! （`format!("{name} の数量")`、`extra_attrs` で付与）。実際に出力される
//! `<select>` の `id` は `fandhe_frontend_headless_ui::field` の派生規則に
//! より `"{id}-control"` になる。
//!
//! # ダミー素材について
//!
//! 商品名・属性・価格は本ファイル内の架空データ（実在のブランド・商品・
//! PII を含まない）で持つ。商品画像はビルド時生成の同梱 SVG
//! （[`dummy_assets::PRODUCT_SRC`]）を使う（外部 URL・`data:` URI は使わ
//! ない）。`alt` は空文字列（`""`）とし、商品名テキストが隣接して可視の
//! ためアクセシブルネームは商品名テキストが担う（装飾寄りのサムネイルの
//! ため空 `alt` を選ぶ判断は `cart-two-column-summary` と同型）。
//!
//! # 形 A（R1248）・形 B（R1249）の並記
//!
//! [`demo`] は形 A（小計のみの集計 + 削除テキストボタン）と形 B（淡い面で
//! 囲った 4 行の集計）を縦に並記する（`contact_split_form_info.rs` の
//! 2 分割と同型のパターン、形ラベルは [`variant_label`]）。商品行・見出しは
//! 両形で共通のものを使い、集計部分だけを差し替える。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::native_select::{
    native_select, FieldIds, FieldProps, NativeSelectProps,
};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};

/// リンク先固定 URL（`cart-two-column-summary` 等と同じ判断で、実アプリ
/// では呼び出し側が実 URL に差し替える前提のダミー値）。
const REPO_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 架空の商品行データ（商品名, 属性表示, 価格表示, 初期選択数量）。実在の
/// ブランド・商品・PII は含まない。小計 ¥23,600 と整合する。
const CART_ITEMS: &[(&str, &str, &str, u8)] = &[
    ("リネン トートバッグ", "ナチュラル", "¥6,800", 1),
    ("セラミック マグカップ", "ホワイト", "¥2,400", 2),
    ("ウール ブランケット", "グレー", "¥12,000", 1),
];

/// 形 A（R1248、小計のみ）の集計行。
const SUMMARY_ROWS_A: &[(&str, &str, bool)] = &[("小計", "¥23,600", false)];

/// 形 B（R1249、淡色面 + 4 行）の集計行。最終行（合計）のみ強調する。
const SUMMARY_ROWS_B: &[(&str, &str, bool)] = &[
    ("小計", "¥23,600", false),
    ("送料", "¥800", false),
    ("税（10%）", "¥2,360", false),
    ("合計", "¥26,760", true),
];

/// 形ラベル（`contact_split_form_info.rs` の `variant_label` と同型）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

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

/// 数量 `select`（形ごとに一意な id、モジュール doc「数量 `select` の id・
/// アクセシブルネーム」節参照）。
fn qty_control(variant_key: &str, index: usize, name: &str, qty: u8) -> Node {
    let field_id = format!("blocks-cart-single-column-{variant_key}-qty-{}", index + 1);
    let qty_aria_label = format!("{name} の数量");
    let field = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        // 数量に応じた小計・合計の再計算を本 Demo は持たない（静的表示の
        // ダミーデータのため）。操作可能なまま残すと選択を変えても
        // `SUMMARY_ROWS_*` と矛盾するため、他の cart block
        // （`cart_dialog.rs`/`cart_line_item_table.rs`/
        // `cart_two_column_summary.rs`）と同型で `disabled` にして
        // 初期選択値のまま固定する。
        disabled: true,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    native_select(
        &NativeSelectProps::default(),
        &field,
        vec![("aria-label", qty_aria_label.as_str())],
        qty_options(qty),
    )
}

/// 商品行 1 件（サムネイル + 名称/属性 + 価格/数量選択/削除）。`variant_key`
/// は数量 `select` の id を形ごとに区別するために使う。
fn item_row(
    variant_key: &str,
    index: usize,
    name: &str,
    attrs: &str,
    price: &str,
    qty: u8,
) -> Node {
    let remove_aria_label = format!("{name} を削除");
    div(
        vec![("class", "blocks-cart-single-column-item")],
        vec![
            image(
                &ImageProps {
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                },
                vec![("data-blocks-cart-single-column-thumb", "")],
            ),
            div(
                vec![("class", "blocks-cart-single-column-item-body")],
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
                ],
            ),
            div(
                vec![("class", "blocks-cart-single-column-item-controls")],
                vec![
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Medium,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(price)],
                    ),
                    qty_control(variant_key, index, name, qty),
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            size: Size::Sm,
                            ..ButtonProps::default()
                        },
                        // 送信処理を持たない静的 Demo のため、操作可能な
                        // まま残すと押しても行・集計が消えず矛盾する。
                        // ネイティブ `disabled` + `data-disabled` で
                        // フォーカス・クリックの双方を抑止する
                        // （`cart_dialog.rs` と同型の判断）。
                        vec![
                            ("aria-label", remove_aria_label.as_str()),
                            ("disabled", ""),
                            ("data-disabled", ""),
                        ],
                        vec![text("削除")],
                    ),
                ],
            ),
        ],
    )
}

/// 商品行と行間の `separator` を束ねたリスト（両形で共通利用）。
fn item_list(variant_key: &str) -> Node {
    let mut children = vec![];
    for (index, (name, attrs, price, qty)) in CART_ITEMS.iter().enumerate() {
        if index > 0 {
            children.push(separator(&SeparatorProps::default(), vec![]));
        }
        children.push(item_row(variant_key, index, name, attrs, price, *qty));
    }
    div(vec![("class", "blocks-cart-single-column-items")], children)
}

/// 集計行 1 件（`data_list::item` + `item-label` + `item-value`）。
/// `emphasize` の行（合計）だけ強調用の
/// `data-blocks-cart-single-column-total` を付与する（[`LAYOUT_CSS`] が
/// 罫線・フォントウェイトで強調するフック）。
fn summary_row(label: &str, value: &str, emphasize: bool) -> Node {
    let attrs = if emphasize {
        vec![("data-blocks-cart-single-column-total", "")]
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

/// 集計リスト（横並び）を組み立てる。
fn summary_list(rows: &[(&str, &str, bool)]) -> Node {
    let data_rows = rows
        .iter()
        .map(|(label, value, emphasize)| summary_row(label, value, *emphasize))
        .collect();
    data_list::root(
        DataListProps {
            orientation: DataListOrientation::Horizontal,
            ..DataListProps::default()
        },
        vec![],
        data_rows,
    )
}

/// 購入手続きボタンと「買い物を続ける」リンクの縦積み（両形で共通利用）。
fn checkout_actions() -> Node {
    div(
        vec![("class", "blocks-cart-single-column-actions")],
        vec![
            button(
                &ButtonProps::default(),
                // 送信先を持たない静的 Demo のため、操作可能なまま
                // 残すと押しても何も起きず実際の状態と矛盾する。
                // ネイティブ `disabled` + `data-disabled` で無効化する
                // （`cart_dialog.rs` と同型の判断）。
                vec![
                    ("data-blocks-cart-single-column-checkout", ""),
                    ("disabled", ""),
                    ("data-disabled", ""),
                ],
                vec![text("購入手続きへ")],
            ),
            link::root(
                REPO_URL,
                &LinkProps {
                    external: true,
                    variant: LinkVariant::Underline,
                    ..LinkProps::default()
                },
                vec![],
                vec![text("買い物を続ける")],
            ),
        ],
    )
}

/// 注記（送料・税の扱い）。形 A は小計のみの表示のため未確定である旨、
/// 形 B は送料・税を計上済みの確定合計を表示しているため、表示額が
/// 確定額であることを示す文言にする（形ごとに文言を分ける理由は P2
/// 指摘: 形 B の注記が計上済み金額と矛盾しないようにするため）。
fn shipping_note(text_content: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(text_content)],
    )
}

/// 形 A（R1248）。中央寄せの見出し → 商品行 → 小計のみの集計 → 注記 →
/// 購入手続き。[`cart_with_summary_b`] と同じ理由（`stack` の直接の子へ
/// 平坦に並べる）で `Vec<Node>` を返す。
fn cart_with_summary_a() -> Vec<Node> {
    vec![
        variant_label("A: 小計のみの集計（R1248）"),
        div(
            vec![("class", "blocks-cart-single-column-cart")],
            vec![
                div(
                    vec![("class", "blocks-cart-single-column-heading")],
                    vec![heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text("ショッピングカート")],
                    )],
                ),
                item_list("a"),
                summary_list(SUMMARY_ROWS_A),
                shipping_note("送料と税は購入手続き時に計算されます。"),
                checkout_actions(),
            ],
        ),
    ]
}

/// 形 B（R1249）。見出し・商品行は形 A と共通。集計は淡色面のパネルで
/// 4 行を囲む。`demo` の `stack`（`display: flex; gap: ...`）へ直接の子
/// として並べる契約のため `Vec<Node>` を返す
/// （`contact_split_form_info::demo` と同型。無地の `div` で包むと `stack`
/// の `gap` が包み `div` 間にしか効かず要素間の余白が消えるため包まない）。
fn cart_with_summary_b() -> Vec<Node> {
    vec![
        variant_label("B: 淡色面に 4 行の集計（R1249）"),
        div(
            vec![("class", "blocks-cart-single-column-cart")],
            vec![
                div(
                    vec![("class", "blocks-cart-single-column-heading")],
                    vec![heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text("ショッピングカート")],
                    )],
                ),
                item_list("b"),
                div(
                    vec![("class", "blocks-cart-single-column-summary-panel")],
                    vec![summary_list(SUMMARY_ROWS_B)],
                ),
                shipping_note("送料と税を含む確定金額です。"),
                checkout_actions(),
            ],
        ),
    ]
}

/// `cart-single-column` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。形 A・形 B を縦に並記する（モジュール doc「形 A（R1248）・形 B
/// （R1249）の並記」節参照）。
pub fn demo() -> Node {
    let mut children = cart_with_summary_a();
    children.extend(cart_with_summary_b());
    div(vec![("class", "blocks-cart-single-column-stack")], children)
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/cart-single-column/",
    title: "cart-single-column",
    category: BlockCategory::Cart,
    rust_source: "crates/docs-site/src/blocks/ecommerce/cart/cart_single_column.rs",
    demo_class: "blocks-cart-single-column",
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
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `cart_single_column` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。1 列構成を保つため
/// `grid-template-columns` は一切使わない。
const LAYOUT_CSS: &str = "\
.blocks-cart-single-column-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  container-type: inline-size;\n  container-name: blocks-cart-single-column;\n}\n\
.blocks-cart-single-column-cart {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  max-width: 42rem;\n  margin-inline: auto;\n}\n\
.blocks-cart-single-column-heading {\n  text-align: center;\n}\n\
.blocks-cart-single-column-items {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-cart-single-column-item {\n  display: flex;\n  gap: var(--fandhe-space-4);\n  flex-wrap: wrap;\n  align-items: flex-start;\n}\n\
[data-blocks-cart-single-column-thumb] {\n  width: 6rem;\n  flex: none;\n}\n\
.blocks-cart-single-column-item-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  flex: 1 1 auto;\n  min-width: 0;\n}\n\
.blocks-cart-single-column-item-controls {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-end;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-cart-single-column-summary-panel {\n  background: var(--fandhe-color-bg-subtle);\n  border-radius: var(--fandhe-radius-md);\n  padding: var(--fandhe-space-4);\n}\n\
[data-blocks-cart-single-column-total] {\n  border-top: 1px solid var(--fandhe-color-border);\n  padding-top: var(--fandhe-space-2);\n  font-weight: var(--fandhe-font-weight-bold, 700);\n}\n\
.blocks-cart-single-column-actions {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-cart-single-column-checkout] {\n  width: 100%;\n}\n\
@container blocks-cart-single-column (min-width: 36rem) {\n  \
.blocks-cart-single-column-summary-panel {\n    margin-inline-start: calc(6rem + var(--fandhe-space-4));\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, CART_ITEMS, LAYOUT_CSS};
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
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<select").count(), 6);
        assert_eq!(html.matches("<h3").count(), 2);
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
        for variant in ["a", "b"] {
            for n in 1..=3 {
                let needle = format!("id=\"blocks-cart-single-column-{variant}-qty-{n}-control\"");
                assert_eq!(
                    html.matches(needle.as_str()).count(),
                    1,
                    "expected exactly one {needle}"
                );
            }
        }
        for (name, ..) in CART_ITEMS {
            let needle = format!("aria-label=\"{name} の数量\"");
            assert_eq!(
                html.matches(needle.as_str()).count(),
                2,
                "expected exactly two {needle} (形 A + 形 B)"
            );
        }
    }

    /// P1 指摘回帰: 削除ボタンのアクセシブルネームが商品名ごとに区別される
    /// ことを固定する（修正前はどの商品行も `aria-label="削除"` で同一だった）。
    #[test]
    fn remove_buttons_have_per_item_aria_label() {
        let html = demo_html();
        for (name, ..) in CART_ITEMS {
            let needle = format!("aria-label=\"{name} を削除\"");
            assert_eq!(
                html.matches(needle.as_str()).count(),
                2,
                "expected exactly two {needle} (形 A + 形 B)"
            );
        }
    }

    #[test]
    fn summary_variants_differ() {
        let html = demo_html();
        assert!(html.contains("小計のみの集計（R1248）"));
        assert!(html.contains("淡色面に 4 行の集計（R1249）"));
        assert!(html.contains("blocks-cart-single-column-summary-panel"));
        // P2 指摘回帰: 形 B は送料・税を計上済みの確定合計を表示するため、
        // 「購入手続き時に計算されます」という未確定を示す注記文言を持たない。
        assert!(html.contains("送料と税は購入手続き時に計算されます。"));
        assert!(html.contains("送料と税を含む確定金額です。"));
        // 「小計」は集計ラベル × 2（形 A + 形 B）に加え、形 A の variant
        // label 文言「小計のみの集計（R1248）」内の部分一致が 1 件乗るため
        // 計 3。「送料」は集計ラベル × 1（形 B のみ）に加え、形 A の注記
        // 「送料と税は購入手続き時に計算されます。」・形 B の注記「送料と税
        // を含む確定金額です。」内の部分一致が 1 件ずつ乗るため計 3。
        assert_eq!(html.matches("小計").count(), 3);
        assert_eq!(html.matches("送料").count(), 3);
        assert_eq!(html.matches("税（10%）").count(), 1);
        assert_eq!(html.matches("合計").count(), 1);
    }

    #[test]
    fn layout_css_is_safe_and_single_column() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-cart-single-column"));
        assert!(!LAYOUT_CSS.contains("grid-template-columns"));
    }

    #[test]
    fn item_thumbnail_alt_is_empty_and_name_is_visible_text() {
        let html = demo_html();
        assert!(html.contains(r#"alt="""#));
        for (name, ..) in CART_ITEMS {
            assert!(html.contains(name), "missing visible product name: {name}");
        }
    }

    /// Bugbot 指摘回帰: 数量 select・削除・購入手続きの各操作可能要素が
    /// 送信処理・再計算を持たない静的 Demo で操作可能なまま残ると、操作
    /// しても `SUMMARY_ROWS_*` の表示額と矛盾する。`cart_dialog.rs` と
    /// 同型の判断で全て `disabled` にする（モジュール doc「`<form>` を
    /// 使わない」節参照）。
    #[test]
    fn all_interactive_controls_are_disabled_noop() {
        let html = demo_html();
        // 数量 select 6（商品 3 件 × 形 A/B）+ 削除ボタン 6 + 購入手続き
        // ボタン 2（形 A/B）= 14。
        assert_eq!(html.matches(" disabled=\"\"").count(), 14);
        // native_select は disabled の field から data-disabled も自動で
        // 付与する（`field.rs` 参照）ため、select 6 + 削除ボタン 6 +
        // 購入手続きボタン 2 = 14（" disabled=\"\"" と同数）。
        assert_eq!(html.matches("data-disabled=\"\"").count(), 14);
    }
}
