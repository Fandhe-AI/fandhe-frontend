//! `order-confirmation-summary` block（イシュー #3054、親 #3024）。
//! Ecommerce / Order カテゴリ。注文確認（お礼 + 追跡番号）・商品明細・
//! 配送先/請求先/支払い/配送方法の 4 情報・集計（割引バッジ付き）の 4 領域
//! を合成する。主参照 R1124（単品 + 住所・支払い・集計）を軸に、R0587
//! （大見出し + 単品 + 縦区切り）・R0586（大きな商品画像の横並び + 右寄せ
//! 集計）の差分は本ファイルと原稿の差分メモ節で扱う。`_/blocks-intake/` の
//! 対応ファイルは着手時点で本 worktree に存在しないため、原稿・本コメントには
//! 対応表 ID のみを記す（`order_tracking_progress`〔イシュー #3060〕と
//! 同じ扱い）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `image` / `separator` / `data-list` / `badge` の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 構成
//!
//! お礼見出し（h3）+ 説明文 + 追跡番号のイントロ、区切り線、商品行
//! 2 件（画像・名称・説明・数量・価格。数量と価格の間に R0587 由来の
//! 縦区切りを挟む）、区切り線、配送先・請求先・支払い方法・配送方法の
//! 4 情報（2 列グリッド、R1124 主参照）、区切り線、集計（小計・割引
//! バッジ・合計、R0586 由来の右寄せ）の順に積む。
//!
//! # `class` と `data-*` の使い分け
//!
//! `image::image`/`badge`/`heading`/`text`/`data_list::root` はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-order-confirmation-summary-*`）。素の `div`/`ul`/`li` は
//! `class="blocks-order-confirmation-summary-*"` を使う
//! （`order_tracking_progress` と同型の判断）。レイアウト root の class は
//! `demo_class`（`blocks-order-confirmation-summary`）と別名
//! （`-layout`）にする（既存 block の Bugbot 教訓、`order_tracking_progress`
//! 系と同型）。
//!
//! # 狭い幅では 2 列の情報欄を 1 列に積む（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`
//! （コンテナクエリ）で判定する（`order_tracking_progress` と同型の
//! パターン）。[`LAYOUT_CSS`] のレイアウト root へ `container-type:
//! inline-size` を宣言し、コンテナ幅が `40rem` 未満のとき配送先・請求先・
//! 支払い方法・配送方法の 2 列グリッドを 1 列化し、商品行の画像を上に積み、
//! 右寄せ集計の `max-width` を解除する。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、注文処理・決済・送信先を一切持たない。
//! 操作可能なボタン・リンクも置かない（使用部品に含まれないため）。
//!
//! # ダミー素材について
//!
//! 宛名・住所・メール・追跡番号・カード番号はすべて架空
//! （`crate::blocks::dummy_assets` の人名を流用しつつ、住所・追跡番号・
//! カード末尾 4 桁・商品名・商品価格は本 block 独自の架空値）。メールアドレスは
//! `example.com` ドメイン。カード番号は末尾 4 桁の伏字表現のみとし、実在
//! パターンは使わない。商品画像はビルド時生成の同梱 SVG
//! （[`dummy_assets::PRODUCT_SRC`]）を使う（外部 URL・`data:` URI は使わない）。
//! 参照元の商品名・ブランド・アイコンは持ち込まない。商品価格は円建てで
//! 固定し（`crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS` はドル建てのため
//! 使わない）、商品 A ¥12,800 × 1 + 商品 B ¥2,400 × 2 = 小計 ¥17,600 →
//! 割引 10%（`AUTUMN10`）−¥1,760 → 合計 ¥15,840 と一致させる。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! R0587（大見出し + 単品 + 縦区切り）は縦区切りのみ本 block の商品行へ
//! 取り込み、大見出し自体は h3 の強調で代替する。R0586（大きな商品画像の
//! 横並び + 右寄せ集計）は右寄せ集計のみ取り込み、大きな商品画像の横並び
//! 表現は 1 block 内に画像サイズの異なる表現が混在すると差分の主眼が
//! 読み取れなくなるため Demo に別枠で並べず、原稿の差分メモ節で扱う。
//! `blocks_contract.rs` への block 固有ページテストの追加も見送る
//! （共通契約テストが `<form>`・`data:`・`raw_html`・重複 id を全 block で
//! 検証済みで、本 block が追加する CSS フックは素の `div`/`ul`/`li` の
//! `class` のみのため「フックが黙って効かない」リスクがない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::Orientation;
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// ラベル・値の 1 行（`data_list::item` + `item-label` + `item-value`）。
fn row(label: &'static str, value: &'static str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 割引コードをバッジで示す集計行（`row` と異なり label 側へ text + badge の
/// 2 ノードを並べる、[`demo`] の集計節専用ヘルパ）。
fn discount_row(code: &'static str, amount: &'static str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(
                vec![],
                vec![
                    text("割引"),
                    badge(
                        &BadgeProps {
                            variant: BadgeVariant::Subtle,
                            ..BadgeProps::default()
                        },
                        vec![],
                        vec![text(code)],
                    ),
                ],
            ),
            data_list::item_value(vec![], vec![text(amount)]),
        ],
    )
}

/// 商品行 1 件（画像・名称・説明 + 数量・縦区切り・価格の `-meta` 行）。
/// 数量と価格の間の縦区切りは R0587（大見出し + 単品 + 縦区切り）由来
/// （モジュール冒頭「構成」節参照）。
fn item_row(
    name: &'static str,
    description: &'static str,
    quantity: &'static str,
    price: &'static str,
) -> Node {
    div(
        vec![("class", "blocks-order-confirmation-summary-item")],
        vec![
            image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, name)
                },
                vec![("data-blocks-order-confirmation-summary-image", "")],
            ),
            div(
                vec![("class", "blocks-order-confirmation-summary-item-info")],
                vec![
                    heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
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
                        vec![text(description)],
                    ),
                    div(
                        vec![("class", "blocks-order-confirmation-summary-meta")],
                        vec![
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(format!("数量 {quantity}"))],
                            ),
                            separator(
                                &SeparatorProps {
                                    orientation: Orientation::Vertical,
                                    ..SeparatorProps::default()
                                },
                                vec![],
                            ),
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(price)],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 見出し + 定義リストの 1 セクション（配送先・請求先・支払い方法・
/// 配送方法。R1124 主参照の 2 列グリッド、[`LAYOUT_CSS`] の `@container` が
/// 狭幅で 1 列化する）。
fn info_section(title: &'static str, rows: Vec<Node>) -> Node {
    div(
        vec![("class", "blocks-order-confirmation-summary-info-section")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(title)],
            ),
            data_list::root(
                DataListProps {
                    orientation: DataListOrientation::Vertical,
                    ..DataListProps::default()
                },
                vec![],
                rows,
            ),
        ],
    )
}

/// `order-confirmation-summary` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-order-confirmation-summary-layout")],
        vec![
            div(
                vec![("class", "blocks-order-confirmation-summary-intro")],
                vec![
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("お支払いが完了しました")],
                    ),
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text("ご注文ありがとうございます")],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(
                            "発送が完了次第、登録のメールアドレス宛にご連絡いたします。",
                        )],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("追跡番号: FD-7Q2K-0915")],
                    ),
                ],
            ),
            separator(&SeparatorProps::default(), vec![]),
            div(
                vec![("class", "blocks-order-confirmation-summary-items")],
                vec![
                    item_row(
                        "リネントートバッグ",
                        "リネン素材のトートバッグ。マチ広で普段使いしやすいサイズ感。",
                        "1",
                        "¥12,800",
                    ),
                    item_row(
                        "陶器マグカップ",
                        "陶器のマグカップ。電子レンジ・食洗機対応。",
                        "2",
                        "¥2,400",
                    ),
                ],
            ),
            separator(&SeparatorProps::default(), vec![]),
            div(
                vec![("class", "blocks-order-confirmation-summary-info")],
                vec![
                    info_section(
                        "配送先",
                        vec![
                            row("宛名", dummy_assets::PERSON_NAMES[0]),
                            row("住所", "東京都渋谷区 1-2-3"),
                        ],
                    ),
                    info_section(
                        "請求先",
                        vec![
                            row("宛名", dummy_assets::PERSON_NAMES[0]),
                            row("メール", "haruto.fujimaki@example.com"),
                        ],
                    ),
                    info_section(
                        "支払い方法",
                        vec![
                            row("支払方法", "クレジットカード"),
                            row("カード番号", "**** **** **** 4242"),
                        ],
                    ),
                    info_section(
                        "配送方法",
                        vec![
                            row("配送方法", "通常配送"),
                            row("お届け予定", "9/27 到着予定"),
                        ],
                    ),
                ],
            ),
            separator(&SeparatorProps::default(), vec![]),
            div(
                vec![("class", "blocks-order-confirmation-summary-totals")],
                vec![data_list::root(
                    DataListProps {
                        orientation: DataListOrientation::Horizontal,
                        ..DataListProps::default()
                    },
                    vec![],
                    vec![
                        row("小計", "¥17,600"),
                        discount_row("AUTUMN10", "−¥1,760"),
                        row("合計", "¥15,840"),
                    ],
                )],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/order-confirmation-summary/",
    title: "order-confirmation-summary",
    category: BlockCategory::Order,
    rust_source: "crates/docs-site/src/blocks/ecommerce/order/order_confirmation_summary.rs",
    demo_class: "blocks-order-confirmation-summary",
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
            label: "Image",
            path: "/themes/image/",
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
            label: "Badge",
            path: "/themes/badge/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `order_confirmation_summary` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-order-confirmation-summary-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-order-confirmation-summary;\n}\n\
.blocks-order-confirmation-summary-intro {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-order-confirmation-summary-items {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-order-confirmation-summary-item {\n  display: grid;\n  grid-template-columns: 6rem 1fr;\n  gap: var(--fandhe-space-4);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-order-confirmation-summary-image] {\n  width: 100%;\n  height: 6rem;\n}\n\
.blocks-order-confirmation-summary-item-info {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-order-confirmation-summary-meta {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-order-confirmation-summary-info {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-order-confirmation-summary-info-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-order-confirmation-summary-totals {\n  margin-inline-start: auto;\n  max-width: 24rem;\n  width: 100%;\n}\n\
@container blocks-order-confirmation-summary (max-width: 40rem) {\n  \
.blocks-order-confirmation-summary-item {\n    grid-template-columns: 1fr;\n  }\n  \
.blocks-order-confirmation-summary-info {\n    grid-template-columns: 1fr;\n  }\n  \
.blocks-order-confirmation-summary-totals {\n    max-width: none;\n  }\n\
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
            "data-scope=\"text\"",
            "data-scope=\"image\"",
            "data-scope=\"separator\"",
            "data-scope=\"data-list\"",
            "data-scope=\"badge\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<h3").count(), 1);
        assert_eq!(html.matches("<img").count(), 2);
        assert!(html.contains("data-orientation=\"vertical\""));
        assert!(html.contains("AUTUMN10"));
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(html.contains("../../assets/blocks-demo-product.svg"));
    }

    #[test]
    fn totals_match_fixed_amounts() {
        let html = demo_html();
        // 商品 A ¥12,800 × 1 + 商品 B ¥2,400 × 2 = 小計 ¥17,600。
        // 割引 10%（AUTUMN10）−¥1,760 → 合計 ¥15,840（モジュール冒頭
        // 「ダミー素材について」節参照）。
        assert!(html.contains("¥17,600"));
        assert!(html.contains("−¥1,760"));
        assert!(html.contains("¥15,840"));
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(
            LAYOUT_CSS.contains("@container blocks-order-confirmation-summary (max-width: 40rem)")
        );
        assert!(LAYOUT_CSS.contains(
            ".blocks-order-confirmation-summary-info {\n    grid-template-columns: 1fr;\n  }"
        ));
        assert!(LAYOUT_CSS.contains(
            ".blocks-order-confirmation-summary-item {\n    grid-template-columns: 1fr;\n  }"
        ));
    }
}
