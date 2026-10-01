//! `order-confirmation-split-image` block（イシュー #3053、親 #3024。
//! Ecommerce / Order カテゴリ）。注文確認ページを合成する。広い幅では左半分に
//! 大きな画像、右列に注文内容を縦積みする 2 カラム構成。主参照 R1121。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`profile_detail_datalist`〔イシュー #2937〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `image` / `heading` / `text` / `data-list` / `separator` / `link` の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::image::image`]・
//! [`fandhe_frontend_pre_styled_ui::heading::heading`]・
//! [`fandhe_frontend_pre_styled_ui::text::text`]・
//! [`fandhe_frontend_pre_styled_ui::data_list::root`]・
//! [`fandhe_frontend_pre_styled_ui::link::root`] はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant クラス
//! と合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-order-confirmation-split-image-*`）。レイアウト用ラッパー
//! （素の `<div>`）は `class="blocks-order-confirmation-split-image-*"` を
//! 使う（`profile_detail_datalist` と同型の判断）。
//!
//! # 狭い幅では画像を上部の帯にする（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため `@container`
//! （コンテナクエリ）で判定する（`profile_detail_datalist` と同型の
//! パターン）。[`LAYOUT_CSS`] のルート
//! `.blocks-order-confirmation-split-image-root` へ `container-type:
//! inline-size` を宣言し、コンテナ幅が `48rem` 以上のとき 2 カラム
//! grid へ、画像を左半分いっぱいに拡張する。未満では縦積みのまま
//! 画像を上部の帯にする。
//!
//! # 配送先と支払い情報は幅にかかわらず 2 列（`data-blocks-…-info`）
//!
//! [`fandhe_frontend_pre_styled_ui::data_list`] の `Vertical` orientation は
//! 1 列縦積みが既定のため、本 block は `data-blocks-order-confirmation-
//! split-image-info` を CSS フックに [`LAYOUT_CSS`] 側で常時 2 列 grid へ
//! 上書きする（`size` バリアントの詳細度に勝つよう `data-scope`/`data-part`
//! を併記、`profile_detail_datalist` の「区切り線・狭幅の縦積みは block 側
//! CSS が描く理由」節と同型の判断）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、決済処理・送信先は一切持たない。
//!
//! # ダミー素材について
//!
//! 商品画像は [`dummy_assets::PRODUCT_SRC`]（ビルド時生成の同梱 SVG、外部
//! URL・`data:` URI は使わない）、氏名は [`dummy_assets::PERSON_NAMES`]
//! （架空セット）を使う。追跡番号・住所・カード下 4 桁はすべて架空の
//! プレースホルダーであり、実在の人物・企業・PII・実クレデンシャルは
//! 含まない。参照元（R1121）の文言・配色・アイコンは持ち込まず、本 block
//! 独自の文言にしている。左半分の背景 [`dummy_assets::BACKGROUND_SRC`]
//! はドット柄の汎用プレースホルダー SVG（実際の梱包・商品の写真ではない）
//! のため、`contact_image_info`/`category_featured_banner` と同じ判断で
//! 空 `alt`（装飾扱い）にする（レビュー指摘: 実内容と異なる
//! 「梱包された注文商品のイメージ」という alt は誤り）。商品サムネイル
//! （[`dummy_assets::PRODUCT_SRC`]）は商品名と一致する alt を保つ。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// ラベル・値の 1 行（`data_list::item` + `item-label` + `item-value`）。
/// `value` は `impl Into<String>` で受け取り、`dummy_assets::PERSON_NAMES`
/// 等と組み立てた動的な値（`format!` の結果）もそのまま渡せるようにする
/// （レビュー指摘: 氏名を文字列直書きにすると共通ダミーデータ更新時に
/// 表示とモジュール説明がずれるため）。
fn row(label: &'static str, value: impl Into<String>) -> Node {
    row_with_attrs(label, value, vec![])
}

/// 強調等の CSS フックを `data_list::item` 自身に付けたい行
/// （`data_list::root` の `<dl>` 直下は `div`〔item〕のみを子に持てるため、
/// 合計行の強調フックは `item` を追加の `div` で包まず本関数で直接付与する。
/// `data_list(dl)` 配下で `item` をさらに `div` で包むと `dt`/`dd` が項目
/// グループ直下にない無効な入れ子になる、というレビュー指摘の是正）。
fn row_with_attrs(
    label: &'static str,
    value: impl Into<String>,
    attrs: Vec<(&'static str, &'static str)>,
) -> Node {
    data_list::item(
        attrs,
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 「買い物を続ける」リンクの遷移先（固定のリポジトリ URL）。
/// 本 Demo は商品一覧・ストアページを持たないため、レビュー指摘
/// （`href="../"` は `/blocks/` 直下へ戻るだけで文言の期待先と不一致）を
/// 受け、`cart_two_column_summary`/`cart_single_column` 等の同種リンクと
/// 同じ判断（固定のリポジトリ URL + `external: true`、`href="#"` は
/// 使わない）に揃える。
const REPO_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 商品行 1 件（サムネイル・商品名・オプション・価格）。
fn product_line(name: &'static str, option: &'static str, price: &'static str) -> Node {
    div(
        vec![("class", "blocks-order-confirmation-split-image-line")],
        vec![
            image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    aspect_ratio: AspectRatio::Square,
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, name)
                },
                vec![("data-blocks-order-confirmation-split-image-thumb", "")],
            ),
            div(
                vec![("class", "blocks-order-confirmation-split-image-line-text")],
                vec![
                    styled_text::text(&TextProps::default(), vec![], vec![text(name)]),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(option)],
                    ),
                ],
            ),
            styled_text::text(&TextProps::default(), vec![], vec![text(price)]),
        ],
    )
}

/// `order-confirmation-split-image` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数。
pub fn demo() -> Node {
    let media = div(
        vec![("class", "blocks-order-confirmation-split-image-media")],
        vec![image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Auto,
                ..ImageProps::new(dummy_assets::BACKGROUND_SRC, "")
            },
            vec![("data-blocks-order-confirmation-split-image-media", "")],
        )],
    );

    let content = div(
        vec![("class", "blocks-order-confirmation-split-image-content")],
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
                    "商品の発送準備が整い次第、登録済みのメールアドレスへ発送通知をお送りします。",
                )],
            ),
            data_list::root(
                DataListProps {
                    orientation: DataListOrientation::Horizontal,
                    ..DataListProps::default()
                },
                vec![],
                vec![row("追跡番号", "FD-2026-0000-1234")],
            ),
            separator(&SeparatorProps::default(), vec![]),
            div(
                vec![("class", "blocks-order-confirmation-split-image-products")],
                vec![
                    product_line("リネンのトートバッグ", "カラー: サンド", "¥4,800"),
                    product_line("セラミックマグカップ", "数量: 2", "¥2,400"),
                    product_line("オーガニックコットンタオル", "サイズ: M", "¥1,600"),
                ],
            ),
            separator(&SeparatorProps::default(), vec![]),
            data_list::root(
                DataListProps {
                    orientation: DataListOrientation::Horizontal,
                    ..DataListProps::default()
                },
                vec![],
                vec![
                    row("小計", "¥8,800"),
                    row("送料", "¥500"),
                    row("消費税", "¥880"),
                    row_with_attrs(
                        "合計",
                        "¥10,180",
                        vec![("data-blocks-order-confirmation-split-image-total", "")],
                    ),
                ],
            ),
            separator(&SeparatorProps::default(), vec![]),
            data_list::root(
                DataListProps {
                    orientation: DataListOrientation::Vertical,
                    ..DataListProps::default()
                },
                vec![("data-blocks-order-confirmation-split-image-info", "")],
                vec![
                    row(
                        "配送先",
                        format!(
                            "{} / 東京都渋谷区桜丘町1-2-3",
                            dummy_assets::PERSON_NAMES[0]
                        ),
                    ),
                    row("支払い方法", "クレジットカード（末尾 0000）"),
                ],
            ),
            link::root(
                REPO_URL,
                &LinkProps {
                    external: true,
                    variant: LinkVariant::Underline,
                    ..LinkProps::default()
                },
                vec![("data-blocks-order-confirmation-split-image-continue", "")],
                vec![text("買い物を続ける →")],
            ),
        ],
    );

    div(
        vec![("class", "blocks-order-confirmation-split-image-root")],
        vec![div(
            vec![("class", "blocks-order-confirmation-split-image-layout")],
            vec![media, content],
        )],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/order-confirmation-split-image/",
    title: "order-confirmation-split-image",
    category: BlockCategory::Order,
    rust_source: "crates/docs-site/src/blocks/ecommerce/order/order_confirmation_split_image.rs",
    demo_class: "blocks-order-confirmation-split-image",
    parts: &[
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Data List",
            path: "/themes/data-list/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `order_confirmation_split_image` 固有のレイアウト規則
/// （`crate::blocks::LAYOUT_CSS` doc「block 固有 CSS の置き場」節と同型）。
///
/// 合計行の強調・配送先/支払い情報の 2 列 grid は `data-scope`/`data-part`
/// を併記して size バリアントの詳細度（0,3,0）に勝つ
/// （`profile_detail_datalist` と同型の判断）。
const LAYOUT_CSS: &str = "\
.blocks-order-confirmation-split-image-root {\n  container-type: inline-size;\n  container-name: blocks-order-confirmation-split-image;\n}\n\
.blocks-order-confirmation-split-image-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-order-confirmation-split-image-media [data-blocks-order-confirmation-split-image-media] {\n  display: block;\n  width: 100%;\n  height: 12rem;\n}\n\
.blocks-order-confirmation-split-image-content {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}\n\
.blocks-order-confirmation-split-image-products {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-order-confirmation-split-image-line {\n  display: grid;\n  grid-template-columns: 4rem minmax(0, 1fr) auto;\n  gap: var(--fandhe-space-3);\n  align-items: center;\n}\n\
.blocks-order-confirmation-split-image-line [data-blocks-order-confirmation-split-image-thumb] {\n  width: 4rem;\n}\n\
.blocks-order-confirmation-split-image-line-text {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
[data-blocks-order-confirmation-split-image-total] [data-scope=\"data-list\"][data-part=\"item-value\"] {\n  font-weight: var(--fandhe-font-font-weight-semibold);\n}\n\
[data-scope=\"data-list\"][data-part=\"root\"][data-blocks-order-confirmation-split-image-info] {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-4);\n}\n\
@container blocks-order-confirmation-split-image (min-width: 48rem) {\n  \
.blocks-order-confirmation-split-image-layout {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    gap: var(--fandhe-space-8);\n    align-items: stretch;\n  }\n  \
.blocks-order-confirmation-split-image-media {\n    height: 100%;\n  }\n  \
.blocks-order-confirmation-split-image-media [data-blocks-order-confirmation-split-image-media] {\n    height: 100%;\n    min-height: 24rem;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, REPO_URL};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"image\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"data-list\"",
            "data-scope=\"separator\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<h3").count(), 1);
        assert_eq!(
            html.matches("blocks-order-confirmation-split-image-line\"")
                .count(),
            3
        );
        assert_eq!(
            html.matches("data-blocks-order-confirmation-split-image-info=\"\"")
                .count(),
            1
        );
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"submit\""));
        assert!(html.contains("../../assets/blocks-demo-product.svg"));
        assert!(html.contains("../../assets/blocks-demo-background.svg"));
        assert!(html.contains(&format!("href=\"{REPO_URL}\"")));
    }

    #[test]
    fn layout_css_is_safe_and_switches_on_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS
            .contains("@container blocks-order-confirmation-split-image (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: repeat(2, minmax(0, 1fr));"));
    }

    #[test]
    fn product_thumbnails_have_non_empty_alt_and_background_is_decorative() {
        let html = demo_html();
        // 商品サムネイル 3 件は商品名と一致する alt を持つ。
        assert!(html.contains("alt=\"リネンのトートバッグ\""));
        assert!(html.contains("alt=\"セラミックマグカップ\""));
        assert!(html.contains("alt=\"オーガニックコットンタオル\""));
        // 左半分の背景はドット柄の汎用プレースホルダーのため装飾扱い（空 alt）。
        assert_eq!(html.matches("alt=\"\"").count(), 1);
    }
}
