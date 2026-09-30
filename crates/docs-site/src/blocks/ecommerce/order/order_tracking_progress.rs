//! `order-tracking-progress` block（イシュー #3060。Ecommerce / Order
//! カテゴリ、区分初の block。親 #3059 の前半部分）。注文ヘッダ・商品カード
//! 列（配送状況の進捗バー + 到達段階ラベル）・サマリの 3 領域を合成する。
//! 主参照 R1122（代表構成）を軸に実装する。集約元 R1123（大画像・枠なし
//! 版）・R0585（商品ごと配送先強調）・R0588（`steps` によるアイコン付き
//! タイムライン版）の差分並記は後半 #3061 へ送る（PR を小さく保つ方針、
//! 親 #3059 実装メモ参照）。`_/blocks-intake/` の対応ファイルは本イシュー
//! 着手時点で本 worktree に存在しないため、原稿・本コメントには対応表 ID
//! のみを記す（`profile_detail_datalist`〔イシュー #2937〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `image` / `progress` / `data-list` / `button` /
//! `link` / `card` / `separator` / `badge` の 10 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! `steps`（R0588 のタイムライン版）は後半 #3061 で追加するため、本イシュー
//! の `parts` には含めない。新しい UI 部品は追加しない。
//!
//! # 構成
//!
//! ヘッダ（注文番号・注文日の見出し + 請求書リンク・確認ボタン）の下に、
//! 商品カード 2 件（画像・名称・価格・配送先 + 配送状況テキスト・
//! 進捗バー・4 段階の到達ラベル列）、区切り線を挟んでサマリ 3 カラム
//! （請求先・支払い情報・集計）を積む。2 商品は到達段階を違えることで
//! 進捗バーの差を見せる（状態違い＝配達完了等の並記は #3061 へ送る）。
//!
//! # 進捗バーと段階ラベルを同じ値から導く（[`shipment_progress`]）
//!
//! `Progress` の `value`（0〜100）と到達段階数（0〜3）が別々の定数から
//! 食い違って書かれることを構造で防ぐため、到達段階 index（0〜3）だけを
//! 呼び出し側で決め、[`shipment_progress`] が `value = index * 100 / 3` を
//! 導出する。段階ラベル列（[`stage_list`]）も同じ `reached` index を受け
//! 取って `data-reached` を切り替えるため、2 つの表現は常に一致する。
//!
//! # `class` と `data-*` の使い分け
//!
//! `card::root`/`button::button`/`image::image`/`progress::root`/
//! `link::root`/`badge`/`heading`/`text`/`data_list::root` はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-order-tracking-progress-*`）。素の `div`/`ul`/`li` と
//! `card::body`/`card::footer`（variant を持たず `attrs` をそのまま連結
//! する）は `class="blocks-order-tracking-progress-*"` を使う
//! （`profile_detail_datalist` と同型の判断）。レイアウト root の class は
//! `demo_class`（`blocks-order-tracking-progress`）と別名（`-layout`）に
//! する（既存 block の Bugbot 教訓、`profile_detail_datalist` 系と同型）。
//!
//! # 狭い幅では段階ラベルを縦並び・商品グリッド/サマリを 1 カラムにする
//! （`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`
//! （コンテナクエリ）で判定する（`profile_detail_datalist` と同型の
//! パターン）。[`LAYOUT_CSS`] のレイアウト root へ `container-type:
//! inline-size` を宣言し、コンテナ幅が `40rem` 未満のときヘッダ操作群を
//! 下段へ折り返し、商品グリッドとサマリを 1 カラム化し、4 段階の段階
//! ラベル列を 1 カラム（縦並び）へ切り替える（親仕様「狭い幅では段階
//! ラベルを縦並び」節）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、注文処理・決済・送信先を一切持たない。
//! ボタンは `button::button` の既定 `type="button"` のまま用いる。
//!
//! # `href` は外部絶対 URL のダミー値
//!
//! 請求書リンクは `href="#"` を避け、ダミーでも意味の通る URL
//! （`https://example.com/invoices/…`）を使う。サイト内相対パス
//! （`/invoices/…`）は `linkcheck::check_links`（`crates/docs-site/
//! tests/support/shared_site.rs`）が「実在しないページ」として
//! fail-closed に検知するため使えない（`footer_newsletter_band` の
//! `REPO` 定数と同型の判断、`crate::blocks` モジュール doc
//! 「`href` に絶対 URL を使う理由」節参照）。
//!
//! # ダミー素材について
//!
//! 注文番号・住所・氏名・決済情報はすべて架空（`crate::blocks::
//! dummy_assets` の人名・社名を流用しつつ、注文番号・住所・カード末尾 4
//! 桁・商品価格は本 block 独自の架空値）。メールアドレスは
//! `example.com` ドメイン、電話番号・住所は架空パターンとし、実在の
//! 人物・企業・PII・実クレデンシャルは含まない。カード番号は末尾 4 桁の
//! 伏字表現のみとし、実在パターンは使わない。商品画像はビルド時生成の
//! 同梱 SVG（[`dummy_assets::PRODUCT_SRC`]）を使う（外部 URL・`data:` URI
//! は使わない）。参照元の商品名・ブランド・アイコンは持ち込まない。商品
//! 価格は円建てで固定し（`crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS`
//! はドル建てのため使わない）、小計・送料・税・合計と通貨・金額を一致
//! させる（2 商品の価格合計 ¥18,400 + 送料 ¥600 + 税 ¥1,900 = 合計
//! ¥20,900）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::Orientation;
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::progress::{self, ProgressProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 配送の 4 段階（表示ラベル）。
const STAGES: [&str; 4] = ["注文受付", "発送準備", "配送中", "配達完了"];

/// 到達段階 index（0〜3）から `Progress`（`value` は 0〜100 の 0〜3 等分）を
/// 導く。段階ラベル列（[`stage_list`]）と進捗バーが同じ `reached` から
/// 導出されるため値の食い違いが構造的に起きない
/// （モジュール冒頭「進捗バーと段階ラベルを同じ値から導く」節参照）。
fn shipment_progress(reached: usize) -> Progress {
    let value = (reached as f64) * 100.0 / ((STAGES.len() - 1) as f64);
    Progress::new(0.0, 100.0, Some(value), Orientation::Horizontal)
}

/// 4 段階の到達ラベル列。`reached` 未満の index は `data-reached` を持つ
/// （狭幅時は [`LAYOUT_CSS`] が 1 カラム＝縦並びへ切り替える）。
fn stage_list(reached: usize) -> Node {
    ul(
        vec![("class", "blocks-order-tracking-progress-stages")],
        STAGES
            .iter()
            .enumerate()
            .map(|(index, label)| {
                let mut attrs = vec![];
                if index <= reached {
                    attrs.push(("data-reached", ""));
                }
                li(attrs, vec![text(*label)])
            })
            .collect(),
    )
}

/// 注文番号・注文日の見出しと、請求書リンク・確認ボタンの操作群を束ねる
/// ヘッダ行。
fn order_header(order_number: &'static str, order_date: &'static str) -> Node {
    div(
        vec![("class", "blocks-order-tracking-progress-header")],
        vec![
            div(
                vec![("class", "blocks-order-tracking-progress-identity")],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(format!("注文 #{order_number}"))],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(format!("注文日 {order_date}"))],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-order-tracking-progress-actions")],
                vec![
                    link::root(
                        "https://example.com/invoices/fd-2048-113",
                        &LinkProps::default(),
                        vec![],
                        vec![text("請求書を表示")],
                    ),
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            size: Size::Sm,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("注文内容を確認")],
                    ),
                ],
            ),
        ],
    )
}

/// 商品カード 1 件（画像・名称・価格・配送先 + 配送状況・進捗バー・
/// 段階ラベル列）。
fn product_card(
    name: &'static str,
    price: &'static str,
    destination: &'static str,
    status_label: &'static str,
    eta: &'static str,
    reached: usize,
) -> Node {
    let progress_state = shipment_progress(reached);
    card::root(
        CardProps::default(),
        vec![("data-blocks-order-tracking-progress-card", "")],
        vec![
            card::body(
                vec![("class", "blocks-order-tracking-progress-product")],
                vec![
                    image(
                        &ImageProps {
                            fit: ImageFit::Cover,
                            shape: ImageShape::Rounded,
                            ..ImageProps::new(dummy_assets::PRODUCT_SRC, name)
                        },
                        vec![("data-blocks-order-tracking-progress-image", "")],
                    ),
                    div(
                        vec![("class", "blocks-order-tracking-progress-product-info")],
                        vec![
                            heading(
                                HeadingLevel::H4,
                                &HeadingProps::default(),
                                vec![],
                                vec![text(name)],
                            ),
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(price)],
                            ),
                            styled_text::text(
                                &TextProps {
                                    variant: TextVariant::Muted,
                                    size: TextSize::Sm,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(format!("配送先: {destination}"))],
                            ),
                        ],
                    ),
                ],
            ),
            card::footer(
                vec![("class", "blocks-order-tracking-progress-shipping")],
                vec![
                    div(
                        vec![("class", "blocks-order-tracking-progress-status")],
                        vec![
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(format!("配送状況: {status_label}"))],
                            ),
                            badge(
                                &BadgeProps {
                                    variant: BadgeVariant::Subtle,
                                    ..BadgeProps::default()
                                },
                                vec![],
                                vec![text(eta)],
                            ),
                        ],
                    ),
                    progress::root(
                        &progress_state,
                        &ProgressProps::default(),
                        Some(status_label),
                        vec![
                            ("data-blocks-order-tracking-progress-bar", ""),
                            ("aria-label", "配送の進捗"),
                        ],
                        vec![progress_state
                            .track(vec![], vec![progress::range(&progress_state, vec![])])],
                    ),
                    stage_list(reached),
                ],
            ),
        ],
    )
}

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

/// 見出し + 定義リストの 1 セクション（請求先・支払い情報・集計）。
fn summary_section(title: &'static str, orientation: DataListOrientation, rows: Vec<Node>) -> Node {
    div(
        vec![("class", "blocks-order-tracking-progress-summary-section")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(title)],
            ),
            data_list::root(
                DataListProps {
                    orientation,
                    ..DataListProps::default()
                },
                vec![],
                rows,
            ),
        ],
    )
}

/// `order-tracking-progress` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-order-tracking-progress-layout")],
        vec![
            order_header("FD-2048-113", "2026-09-24"),
            separator(&SeparatorProps::default(), vec![]),
            div(
                vec![("class", "blocks-order-tracking-progress-items")],
                vec![
                    product_card(
                        dummy_assets::COMPANY_NAMES[0],
                        "¥12,800",
                        "東京都渋谷区 1-2-3",
                        "配送中",
                        "9/27 到着予定",
                        2,
                    ),
                    product_card(
                        dummy_assets::COMPANY_NAMES[1],
                        "¥5,600",
                        "大阪府大阪市 4-5-6",
                        "発送準備中",
                        "9/29 到着予定",
                        1,
                    ),
                ],
            ),
            separator(&SeparatorProps::default(), vec![]),
            div(
                vec![("class", "blocks-order-tracking-progress-summary")],
                vec![
                    summary_section(
                        "請求先",
                        DataListOrientation::Vertical,
                        vec![
                            row("宛名", dummy_assets::PERSON_NAMES[0]),
                            row("住所", "東京都渋谷区 1-2-3"),
                            row("メール", "haruto.fujimaki@example.com"),
                        ],
                    ),
                    summary_section(
                        "支払い情報",
                        DataListOrientation::Vertical,
                        vec![
                            row("支払方法", "クレジットカード"),
                            row("カード番号", "**** **** **** 4242"),
                            row("請求日", "2026-09-24"),
                        ],
                    ),
                    summary_section(
                        "集計",
                        DataListOrientation::Horizontal,
                        vec![
                            row("小計", "¥18,400"),
                            row("送料", "¥600"),
                            row("税", "¥1,900"),
                            row("合計", "¥20,900"),
                        ],
                    ),
                ],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/order-tracking-progress/",
    title: "order-tracking-progress",
    category: BlockCategory::Order,
    rust_source: "crates/docs-site/src/blocks/ecommerce/order/order_tracking_progress.rs",
    demo_class: "blocks-order-tracking-progress",
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
            label: "Progress",
            path: "/themes/progress/",
        },
        Part {
            label: "Data List",
            path: "/themes/data-list/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `order_tracking_progress` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-order-tracking-progress-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-order-tracking-progress;\n}\n\
.blocks-order-tracking-progress-header {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: flex-start;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-order-tracking-progress-identity {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-order-tracking-progress-actions {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-order-tracking-progress-items {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-order-tracking-progress-product {\n  display: grid;\n  grid-template-columns: 8rem 1fr;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-order-tracking-progress-image] {\n  width: 100%;\n  height: 6rem;\n}\n\
.blocks-order-tracking-progress-product-info {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-order-tracking-progress-shipping {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-order-tracking-progress-status {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-order-tracking-progress-stages {\n  display: grid;\n  grid-template-columns: repeat(4, minmax(0, 1fr));\n  gap: var(--fandhe-space-2);\n  margin: 0;\n  padding: 0;\n  list-style: none;\n}\n\
.blocks-order-tracking-progress-stages > li {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n  text-align: center;\n}\n\
.blocks-order-tracking-progress-stages > li[data-reached] {\n  color: var(--fandhe-color-fg);\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
.blocks-order-tracking-progress-summary {\n  display: grid;\n  grid-template-columns: repeat(3, minmax(0, 1fr));\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-order-tracking-progress-summary-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
@container blocks-order-tracking-progress (max-width: 40rem) {\n  \
.blocks-order-tracking-progress-actions {\n    width: 100%;\n  }\n  \
.blocks-order-tracking-progress-product {\n    grid-template-columns: 1fr;\n  }\n  \
.blocks-order-tracking-progress-stages {\n    grid-template-columns: 1fr;\n  }\n  \
.blocks-order-tracking-progress-summary {\n    grid-template-columns: 1fr;\n  }\n\
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
            "data-scope=\"progress\"",
            "data-scope=\"data-list\"",
            "data-scope=\"button\"",
            "data-scope=\"link\"",
            "data-scope=\"card\"",
            "data-scope=\"separator\"",
            "data-scope=\"badge\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<h3").count(), 1);
        assert_eq!(
            html.matches("data-scope=\"card\" data-part=\"root\"")
                .count(),
            2
        );
        assert_eq!(
            html.matches("data-scope=\"progress\" data-part=\"root\"")
                .count(),
            2
        );
        assert_eq!(html.matches("<li").count(), 8);
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
    fn progress_value_matches_reached_stage_count() {
        let html = demo_html();
        // 商品 1: reached = 2 → value = 2 * 100 / 3 ≈ 66.67%
        assert!(html.contains("--fandhe-progress-percent: 66.66666666666667%"));
        // 商品 2: reached = 1 → value = 1 * 100 / 3 ≈ 33.33%
        assert!(html.contains("--fandhe-progress-percent: 33.333333333333336%"));
        assert_eq!(html.matches("data-reached=\"\"").count(), 5);
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-order-tracking-progress (max-width: 40rem)"));
        assert!(LAYOUT_CSS.contains(
            ".blocks-order-tracking-progress-stages {\n    grid-template-columns: 1fr;\n  }"
        ));
    }
}
