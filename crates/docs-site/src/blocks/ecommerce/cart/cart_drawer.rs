//! `cart-drawer` block（イシュー #3026。親 #3024「Blocks EC 系」配下）。
//! Ecommerce / Cart カテゴリ 3 件目の block。画面右端（inline-end）から
//! 出るカートドロワーの合成例（主参照 R1250: ヘッダ・スクロール本文・
//! 固定フッタの 3 段構成、集約元 R0679: 右からスライドするカート。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、`cart_line_item_table.rs` と同じく対応表 ID のみを記す）。
//!
//! # 使用部品
//!
//! `drawer` / `button` / `image` / `text` / `separator` / `data-list` の
//! 6 部品のみを合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい
//! UI 部品は追加しない。
//!
//! # 静的開状態で固定する理由
//!
//! docs サイトは無 JS のため、スライドイン等の開閉アニメーションを
//! 実演できない（`fandhe-frontend-wasm-full` は drawer scope を未配線
//! でもある、`crates/pre-styled-ui/src/drawer.rs` rustdoc 参照）。本 Demo
//! は**開いた状態のみ**を静的に描く（`contact_dialog_form`/`game_ui_modal`
//! と同じ設計判断）。
//!
//! # trigger の `aria-expanded`/`aria-controls`
//!
//! [`drawer::trigger`] に `OpenState::Open` を渡すことで `aria-expanded="true"`
//! が出力される。無 JS 下でも「開いた状態」という表示の実態と一致するため
//! 嘘にならない。`controls` に content の id を渡し `aria-controls` で
//! 関連付ける。
//!
//! # 閉じるボタンを置く理由・`drawer::close_trigger` を使わない理由
//!
//! Issue のレイアウト仕様（ヘッダー行に閉じるボタンを含む）に従い、
//! [`button::close_button`]（`fandhe-frontend-pre-styled-ui` の通常のボタン
//! 部品）を置く。`drawer::close_trigger` ではなくこちらを使うのは、
//! アイコン子要素を自作する必要がなく既存部品をそのまま再利用できるため。
//! 押しても閉じない静的な見本であることは、他の購入手続きボタン等と同じ
//! 扱い（`contact_dialog_form` が close trigger 自体を省いた判断とは異なる
//! ケース。本 block は Issue がレイアウト仕様として閉じるボタンの配置を
//! 明示しているため、inert なボタンとして配置する）。
//!
//! # `aria-modal` を false にする理由
//!
//! 静的デモの外側に本文・ナビゲーションがあるため、表示の実態と一致させ
//! `aria-modal="false"` にする（`contact_dialog_form`/`game_ui_modal` と
//! 同じ判断）。
//!
//! # fixed オーバーレイの中和
//!
//! `drawer::backdrop`/`drawer::positioner` の既定 CSS は
//! `position: fixed; inset: 0` のビューポート全体オーバーレイだが、Blocks
//! の掲示は [`LAYOUT_CSS`] の `.blocks-cart-drawer-stage` 枠内へ収める
//! 必要がある。本 block スコープに限定した属性セレクタで `position:
//! absolute; inset: 0; z-index: auto` へ差し替え、stage 自身を
//! `position: relative; overflow: hidden` にして受け皿にする
//! （`contact_dialog_form` と同型の手法）。
//!
//! # 3 段固定レイアウトを block 側 div で組む理由
//!
//! `drawer` の anatomy には header/body/footer パートが存在しない
//! （`crates/pre-styled-ui/src/drawer.rs` rustdoc「本イシューのスコープ外」
//! 節に記録済みの既知の制約）。このため「ヘッダー固定・本文のみスクロール・
//! フッター固定」という Issue のレイアウト仕様は、`content` を `display:
//! flex; flex-direction: column` にした上で、ヘッダー・フッターに相当する
//! 素の `div`（`flex: none`）と、本文スクロール領域の `div`（`flex: 1 1
//! auto; min-height: 0; overflow-y: auto`）を block 側で組み立てることで
//! 実現する。headless/pre-styled 側の anatomy は変更しない。
//!
//! # 狭幅でのコンテナクエリ
//!
//! Issue のレイアウト仕様「狭い幅ではドロワーを全幅にする」に対し、
//! `@media` ではなく `@container`（`.blocks-cart-drawer-stage` が
//! `container-type: inline-size`）を使う。Demo 枠の幅はビューポート幅と
//! 一致しないため（`product_overview_*`/`page_heading_*` 等と同じ判断）。
//!
//! # スクロール領域のアクセシブルネーム
//!
//! 商品行のスクロール領域には `role="region"`・`aria-label`・
//! `tabindex="0"` を付与し、キーボード操作でスクロール位置にフォーカスを
//! 移せるようにする（支援技術の利用者がスクロール可能な領域の境界と役割を
//! 把握できるようにする目的）。
//!
//! # ダミー素材について
//!
//! 商品画像は [`dummy_assets::PRODUCT_SRC`]（モノトーン抽象図形の SVG、
//! `build.rs` がビルド時に書き出す）を使う。商品名・バリエーション・価格は
//! 独自に書いた架空の文言であり、実在の商品・企業とは無関係。小計は各行の
//! 単価 × 数量の和と手で一致させている。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンはすべて既定の `type="button"` のまま用い、送信処理・
//! 送信先は一切持たない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::drawer::{self, ContentIds, DrawerPlacement, OpenState};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self, TextProps, TextVariant, TextWeight};
use fandhe_frontend_pre_styled_ui::Size;

/// drawer `content` の id（trigger の `aria-controls` が指す先）。
const CONTENT_ID: &str = "blocks-cart-drawer-content";
/// drawer `title` の id（content の `aria-labelledby` が指す先）。
const TITLE_ID: &str = "blocks-cart-drawer-title";

/// カート明細行 1 件分の架空データ。
struct CartItem {
    name: &'static str,
    variant: &'static str,
    qty: &'static str,
    price: &'static str,
}

/// 明細 3 行（小計 ¥21,600 はこの 3 行の単価 × 数量の和と手で一致させている:
/// 5,800×1 + 6,200×2 + 3,400×1 = 21,600）。
const ITEMS: &[CartItem] = &[
    CartItem {
        name: "リネンシャツ",
        variant: "ネイビー / M",
        qty: "数量: 1",
        price: "¥5,800",
    },
    CartItem {
        name: "コットンパンツ",
        variant: "ベージュ / L",
        qty: "数量: 2",
        price: "¥12,400",
    },
    CartItem {
        name: "キャンバストートバッグ",
        variant: "オフホワイト",
        qty: "数量: 1",
        price: "¥3,400",
    },
];

/// カートを開くトリガー（stage 上部の疑似ストアヘッダー行に置く）。
fn open_trigger() -> Node {
    drawer::trigger(
        OpenState::Open,
        Some(CONTENT_ID),
        vec![("data-blocks-cart-drawer-trigger", "")],
        vec![text("カート（3）")],
    )
}

/// ヘッダー（タイトル + 閉じるボタン）。`flex: none` で固定し、本文だけが
/// スクロールする（モジュール doc「3 段固定レイアウト」節参照）。
fn header() -> Node {
    div(
        vec![("class", "blocks-cart-drawer-header")],
        vec![
            drawer::title(Some(TITLE_ID), vec![], vec![text("ショッピングカート")]),
            button::close_button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    ..ButtonProps::default()
                },
                "カートを閉じる",
                vec![],
            ),
        ],
    )
}

/// 商品サムネイル画像。装飾的なダミー図形のため `alt=""` とし、同じ行内の
/// 可視の商品名との二重読み上げを避ける（`cart_line_item_table` と同じ扱い）。
fn item_thumbnail() -> Node {
    image::image(
        &ImageProps {
            shape: ImageShape::Rounded,
            fit: ImageFit::Cover,
            ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
        },
        vec![("data-blocks-cart-drawer-thumb", "")],
    )
}

/// 商品行 1 件（画像 + 商品名・バリエーション・数量 + 価格 + 削除ボタン）。
fn item_row(item: &CartItem) -> Node {
    li(
        vec![("class", "blocks-cart-drawer-item")],
        vec![
            item_thumbnail(),
            div(
                vec![("class", "blocks-cart-drawer-item-info")],
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
                    text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(item.qty)],
                    ),
                ],
            ),
            text::text(
                &TextProps {
                    weight: TextWeight::Medium,
                    ..TextProps::default()
                },
                vec![("class", "blocks-cart-drawer-item-price")],
                vec![text(item.price)],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("削除")],
            ),
        ],
    )
}

/// 商品行のスクロール領域（`role="region"` + `aria-label` + `tabindex="0"`、
/// モジュール doc「スクロール領域のアクセシブルネーム」節参照）。
fn scroll_body() -> Node {
    div(
        vec![
            ("class", "blocks-cart-drawer-body"),
            ("role", "region"),
            ("aria-label", "カート内の商品"),
            ("tabindex", "0"),
        ],
        vec![ul(
            vec![("class", "blocks-cart-drawer-items")],
            ITEMS.iter().map(item_row).collect(),
        )],
    )
}

/// フッター（小計 + 注記 + 購入手続き + 買い物を続ける）。`flex: none` で
/// 固定する。
fn footer() -> Node {
    let subtotal_item = data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text("小計")]),
            data_list::item_value(vec![], vec![text("¥21,600")]),
        ],
    );
    let list = data_list::root(
        DataListProps {
            orientation: DataListOrientation::Horizontal,
            ..DataListProps::default()
        },
        vec![],
        vec![subtotal_item],
    );
    let note = text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text("送料と税は購入手続きで計算されます")],
    );
    let checkout_button = button::button(
        &ButtonProps {
            variant: ButtonVariant::Solid,
            ..ButtonProps::default()
        },
        vec![("data-blocks-cart-drawer-checkout", "")],
        vec![text("購入手続きへ")],
    );
    let continue_button = button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            ..ButtonProps::default()
        },
        vec![],
        vec![text("買い物を続ける")],
    );
    div(
        vec![("class", "blocks-cart-drawer-footer")],
        vec![list, note, checkout_button, continue_button],
    )
}

/// `cart-drawer` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
/// 疑似ページ枠（stage）の中に、カートを開くボタンと、開いた状態の
/// drawer（backdrop + positioner + content）を配置する。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-cart-drawer-stage")],
        vec![
            div(
                vec![("class", "blocks-cart-drawer-topbar")],
                vec![open_trigger()],
            ),
            drawer::root(
                Size::Md,
                OpenState::Open,
                DrawerPlacement::End,
                vec![("data-blocks-cart-drawer-root", "")],
                vec![
                    drawer::backdrop(OpenState::Open, vec![], vec![]),
                    drawer::positioner(
                        OpenState::Open,
                        DrawerPlacement::End,
                        vec![],
                        vec![drawer::content(
                            OpenState::Open,
                            DrawerPlacement::End,
                            false,
                            ContentIds {
                                id: Some(CONTENT_ID),
                                labelledby: Some(TITLE_ID),
                                describedby: None,
                            },
                            vec![("data-blocks-cart-drawer-panel", "")],
                            vec![
                                header(),
                                separator::separator(&SeparatorProps::default(), vec![]),
                                scroll_body(),
                                separator::separator(&SeparatorProps::default(), vec![]),
                                footer(),
                            ],
                        )],
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
    path: "/blocks/cart-drawer/",
    title: "cart-drawer",
    category: BlockCategory::Cart,
    rust_source: "crates/docs-site/src/blocks/ecommerce/cart/cart_drawer.rs",
    demo_class: "blocks-cart-drawer",
    parts: &[
        Part {
            label: "Drawer",
            path: "/themes/drawer/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
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

/// `cart_drawer` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節と同型）。モジュール doc「fixed オーバー
/// レイの中和」「3 段固定レイアウト」「狭幅でのコンテナクエリ」節の実装。
const LAYOUT_CSS: &str = "\
.blocks-cart-drawer-stage {\n  position: relative;\n  height: 34rem;\n  overflow: hidden;\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-subtle);\n  container-type: inline-size;\n  container-name: blocks-cart-drawer;\n}\n\
.blocks-cart-drawer-topbar {\n  display: flex;\n  justify-content: flex-end;\n  padding: var(--fandhe-space-4);\n}\n\
.blocks-cart-drawer [data-scope=\"drawer\"][data-part=\"backdrop\"] {\n  position: absolute;\n  inset: 0;\n  z-index: auto;\n}\n\
.blocks-cart-drawer [data-scope=\"drawer\"][data-part=\"positioner\"] {\n  position: absolute;\n  inset: 0;\n  z-index: auto;\n}\n\
.blocks-cart-drawer [data-scope=\"drawer\"][data-part=\"content\"][data-placement] {\n  display: flex;\n  flex-direction: column;\n  height: 100%;\n  padding: 0;\n  overflow: hidden;\n}\n\
.blocks-cart-drawer-header {\n  flex: none;\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-4) var(--fandhe-space-6);\n}\n\
.blocks-cart-drawer [data-scope=\"drawer\"][data-part=\"title\"] {\n  margin: 0;\n  padding-inline-end: 0;\n}\n\
.blocks-cart-drawer-body {\n  flex: 1 1 auto;\n  min-height: 0;\n  overflow-y: auto;\n  padding: var(--fandhe-space-4) var(--fandhe-space-6);\n}\n\
.blocks-cart-drawer-items {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-cart-drawer-item {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  padding-block-end: var(--fandhe-space-4);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-cart-drawer-item:last-child {\n  border-bottom: none;\n  padding-block-end: 0;\n}\n\
img[data-scope=\"image\"][data-blocks-cart-drawer-thumb] {\n  width: 4rem;\n  height: 4rem;\n  flex-shrink: 0;\n}\n\
.blocks-cart-drawer-item-info {\n  flex: 1 1 auto;\n  min-width: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-cart-drawer-footer {\n  flex: none;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  padding: var(--fandhe-space-4) var(--fandhe-space-6) var(--fandhe-space-6);\n}\n\
[data-blocks-cart-drawer-checkout] {\n  width: 100%;\n}\n\
@container blocks-cart-drawer (max-width: 40rem) {\n  .blocks-cart-drawer [data-scope=\"drawer\"][data-part=\"content\"][data-placement] {\n    width: 100%;\n  }\n}\n";

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
            "data-scope=\"drawer\"",
            "data-scope=\"button\"",
            "data-scope=\"image\"",
            "data-scope=\"text\"",
            "data-scope=\"separator\"",
            "data-scope=\"data-list\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn no_form_and_all_buttons_are_type_button() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        let count_open = html.matches("<button").count();
        let count_typed = html.matches("type=\"button\"").count();
        assert!(count_open > 0);
        assert_eq!(count_typed, count_open, "html={html}");
    }

    #[test]
    fn drawer_is_open_static_non_modal_at_end() {
        let html = demo_html();
        assert!(html.contains("data-state=\"open\""));
        assert!(html.contains("data-placement=\"end\""));
        assert!(html.contains("aria-modal=\"false\""));
        // backdrop の `aria-hidden="true"` に "hidden" の部分文字列が
        // 含まれるため、単独の存在属性 `hidden`（closed 時のみ出力される）
        // だけを精確に検知する（`contact_dialog_form` と同じ手法）。
        assert!(!html.contains(" hidden>"));
        assert!(!html.contains(" hidden "));
    }

    #[test]
    fn trigger_and_title_wire_aria_ids() {
        let html = demo_html();
        assert!(html.contains("aria-controls=\"blocks-cart-drawer-content\""));
        assert!(html.contains("id=\"blocks-cart-drawer-content\""));
        assert!(html.contains("aria-labelledby=\"blocks-cart-drawer-title\""));
        assert!(html.contains("id=\"blocks-cart-drawer-title\""));
        assert!(html.contains("aria-expanded=\"true\""));
    }

    #[test]
    fn scroll_region_is_focusable_and_labelled() {
        let html = demo_html();
        assert!(html.contains("role=\"region\""));
        assert!(html.contains("aria-label=\"カート内の商品\""));
        assert!(html.contains("tabindex=\"0\""));
    }

    #[test]
    fn product_images_have_alt_and_bundled_src() {
        let html = demo_html();
        assert_eq!(html.matches("alt=\"\"").count(), 3, "html={html}");
        assert!(html.contains(super::dummy_assets::PRODUCT_SRC));
    }

    #[test]
    fn layout_css_neutralizes_fixed_overlay_and_goes_full_width_on_narrow() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("position: absolute;\n  inset: 0;\n  z-index: auto;"));
        assert!(LAYOUT_CSS.contains("@container blocks-cart-drawer (max-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("width: 100%;"));
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-cart-drawer-stage\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-cart-drawer-stage");
    }
}
