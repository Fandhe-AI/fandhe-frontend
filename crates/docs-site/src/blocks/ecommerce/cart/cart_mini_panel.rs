//! `cart-mini-panel` block（イシュー #3030、親 #3024「Blocks EC」配下）。
//! 店舗ヘッダーのカートボタン直下に開くミニカートパネルの合成例。主参照は
//! 対応表 ID R1252（店舗ヘッダ + カートのポップオーバー）。集約元は R0326
//! （数量ステッパー + 合計込みボタン）・R0327（合計表示 + 2 ボタン）。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`cart_two_column_summary` と同じ扱い）。
//!
//! # 使用部品
//!
//! `popover` / `button` / `image` / `text` / `number-input` / `separator` /
//! `link` / `heading` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。新しい UI 部品は追加しない。
//!
//! # 2 インスタンスの並記（静的表示、無 JS）
//!
//! docs サイトは JS ハイドレーションを行わないため、パネルの開閉・数量
//! 変更は表現できない。[`demo`] は次の 2 インスタンスを縦に並記する
//! （`cart_two_column_summary::cart_with_items`/`empty_cart` と同型の並記
//! パターン）。
//!
//! - **basic**（主参照 R1252 + 集約元 R0327）: 商品行は [`CART_ITEMS`] の
//!   数量をそのまま固定テキストで表示するのみで、小計・補足・購入手続き/
//!   カートを見るの 2 操作を置く。
//! - **stepper**（集約元 R0326）: 各商品行の数量を readonly の数量
//!   ステッパーにする。購入ボタンの文言は送料・税が未確定である旨と
//!   矛盾しないよう basic と同一にし、確定額は含めない。
//!
//! いずれも `popover::root` を `OpenState::Open` 固定で描き、無 JS の静的
//! Demo として「パネルが開いた状態」のみを示す（`trigger` 自体は enabled の
//! まま、`page_heading_avatar` 等と同じ判断）。
//!
//! # `positioner` を通常フローへ戻す理由
//!
//! [`fandhe_frontend_pre_styled_ui::popover`] の `positioner` は既定で
//! `position: absolute; top: 100%; left: 0` のドロップダウン型オーバーレイ
//! だが、`.blocks-demo` は `overflow-x: auto` の横スクロールコンテナのため
//! 絶対配置のままだと縦方向にも切り取られる（`contact_dialog_form` の
//! dialog positioner 中和と同型の問題）。本 block スコープに限定した属性
//! セレクタで `position: static` へ中和し、トリガー直下の通常フローへ戻す
//! （[`LAYOUT_CSS`] 参照）。
//!
//! # `number_input` を readonly + トリガー disabled にする理由
//!
//! stepper インスタンスの数量ステッパーは、無 JS では押しても何も起きない
//! ため `NumberInputFlags::readonly` を `true` にし、増減トリガーも
//! `disabled: true` にする（`pricing_seats_split`/`header_flyout_menu` 等の
//! 前例と同じ判断）。`aria-valuenow`/表示値は固定値のまま変化しない。
//!
//! # 「カートを見る」リンク先
//!
//! `href="#"` は使わない（REQ-1 不変条件）。サイト内に実在する Cart 一覧
//! block `/blocks/cart-two-column-summary/` へ、[`crate::layout::asset_href`]
//! で base_path を反映したうえでリンクする（`navbar_docs_site` と同型の
//! 判断。linkcheck で参照整合が検証される）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。ボタンは
//! すべて `button::button` の既定 `type="button"` のまま用いる。
//!
//! # `class` と `data-*` の使い分け
//!
//! `button::button`/`image::image`/`text::text`/`heading::heading`/
//! `number_input::root`/`separator::separator`/`link::root` はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去するため、これらへの CSS
//! フックは `data-blocks-cart-mini-panel-*` 属性で渡す（`cart_two_column_
//! summary` と同じ判断）。`popover` の各パーツ・素の `div`/`p` は
//! `drop_class_attr` を経由しないため `class="blocks-cart-mini-panel-*"` を
//! 使う。行のメタ情報（属性・金額）は `text` 部品ではなく `el("p", ...)` で
//! 組み、`class` フックを直接付与する（`page_heading_avatar` と同じ判断。
//! ただし使用部品契約を満たすため `text` 部品も商品名・小計等で最低 1 回は
//! 使う）。
//!
//! # ダミー素材
//!
//! 店名は [`dummy_assets::COMPANY_NAMES`]、商品画像は
//! [`dummy_assets::PRODUCT_SRC`]（ビルド時生成の同梱 SVG、`data:` URI は
//! 使わない）を使う。商品名・金額は本ファイル内の架空データ（実在の
//! ブランド・商品・PII を含まない）。
use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::number_input::{self, NumberInputFlags};
use fandhe_frontend_pre_styled_ui::popover::{self, OpenState};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 架空の商品行データ（商品名, 属性表示, 単価表示, 数量）。3 行とも金額は
/// 小計（[`SUBTOTAL_TEXT`]）と整合する固定値。実在のブランド・商品・PII は
/// 含まない。
const CART_ITEMS: &[(&str, &str, &str, u32)] = &[
    ("ミニマル デスクランプ", "カラー: ホワイト", "¥4,800", 1),
    (
        "折りたたみ ブックスタンド",
        "素材: ウォルナット",
        "¥3,200",
        2,
    ),
    ("セラミック マグカップ", "カラー: グレー", "¥1,600", 1),
];

/// 小計表示（[`CART_ITEMS`] の単価 × 数量の合計と整合する固定値）。
const SUBTOTAL_TEXT: &str = "¥12,800";

/// stepper インスタンスの購入ボタン文言。送料・税は購入手続きで計算する旨を
/// [`subtotal_row`] の補足行で示しているため、確定していない合計額はボタン
/// 文言に含めない（基本形と同一文言）。
const STEPPER_CHECKOUT_LABEL: &str = "購入手続きへ";

/// 「カートを見る」の遷移先（モジュール doc参照）。
fn cart_summary_href() -> String {
    crate::layout::asset_href(
        &crate::blocks::demo_base_path(),
        "/blocks/cart-two-column-summary/",
    )
}

/// インスタンスの見出し（「基本形」「数量ステッパー付き」）。
fn variant_heading(label: &'static str) -> Node {
    heading(
        HeadingLevel::H3,
        &HeadingProps::default(),
        vec![],
        vec![text(label)],
    )
}

/// 店舗ヘッダーバー（架空の店名 + カートトリガー）。`trigger` は
/// `controls` で `content_id` と関連付け、無 JS の静的 Demo のため常に
/// `OpenState::Open` を渡す（モジュール doc「2 インスタンスの並記」節）。
fn shop_header(shop_name: &str, content_id: &str) -> Node {
    div(
        vec![("class", "blocks-cart-mini-panel-shop-header")],
        vec![
            el("p", vec![], vec![text(shop_name)]),
            popover::trigger(
                OpenState::Open,
                false,
                Some(content_id),
                vec![("data-blocks-cart-mini-panel-trigger", "")],
                vec![text("カート（3）")],
            ),
        ],
    )
}

/// 数量表示（basic: 固定テキスト、stepper: readonly ステッパー）。`name` は
/// stepper インスタンスのラベルへ商品名を含め、各行のアクセシブルネームを
/// 一意にするために使う（`number_input::label` の視覚文言、モジュール doc
/// 「`class` と `data-*` の使い分け」節参照）。
fn qty_display(prefix: &str, index: usize, name: &str, qty: u32, interactive: bool) -> Node {
    if !interactive {
        return el(
            "p",
            vec![("class", "blocks-cart-mini-panel-qty")],
            vec![text(format!("数量 {qty}"))],
        );
    }
    let field_id = format!("{prefix}-qty-{}", index + 1);
    let qty_value = qty.to_string();
    let flags = NumberInputFlags {
        readonly: true,
        ..NumberInputFlags::default()
    };
    number_input::root(
        Size::Sm,
        false,
        false,
        true,
        // `number_input::root`（pre-styled-ui）は `drop_class_attr` で
        // 呼び出し側 `class` を除去するため、CSS フックは `class` ではなく
        // `data-blocks-cart-mini-panel-qty` で渡す（モジュール doc
        // 「`class` と `data-*` の使い分け」節の前提どおり、`root` だけが
        // 本ファイル内で誤って `class` を使っていた是正）。
        vec![("data-blocks-cart-mini-panel-qty", "")],
        vec![
            number_input::label(
                flags,
                Some(&field_id),
                vec![],
                vec![text(format!("{name} の数量"))],
            ),
            number_input::control(
                flags,
                vec![],
                vec![
                    number_input::decrement_trigger(Some(&field_id), true, vec![], vec![text("-")]),
                    number_input::input(
                        "qty",
                        Some(&field_id),
                        Some(&qty_value),
                        "1",
                        "99",
                        flags,
                        vec![],
                    ),
                    number_input::increment_trigger(Some(&field_id), true, vec![], vec![text("+")]),
                ],
            ),
        ],
    )
}

/// 商品行 1 件（サムネイル + 名称/属性 + 数量 + 価格）。`prefix` は
/// stepper インスタンスの数量入力 `id` の一意化に使う。
fn item_row(
    prefix: &str,
    index: usize,
    name: &str,
    attrs: &str,
    price: &str,
    qty: u32,
    interactive: bool,
) -> Node {
    div(
        vec![("class", "blocks-cart-mini-panel-item")],
        vec![
            image(
                &ImageProps {
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                },
                vec![("data-blocks-cart-mini-panel-thumb", "")],
            ),
            div(
                vec![("class", "blocks-cart-mini-panel-item-body")],
                vec![
                    styled_text::text(&TextProps::default(), vec![], vec![text(name)]),
                    el(
                        "p",
                        vec![("class", "blocks-cart-mini-panel-item-attrs")],
                        vec![text(attrs)],
                    ),
                    qty_display(prefix, index, name, qty, interactive),
                ],
            ),
            el(
                "p",
                vec![("class", "blocks-cart-mini-panel-item-price")],
                vec![text(price)],
            ),
        ],
    )
}

/// 小計 + 補足行（モジュール doc「2 インスタンスの並記」節参照）。
fn subtotal_row() -> Node {
    div(
        vec![("class", "blocks-cart-mini-panel-subtotal")],
        vec![
            div(
                vec![("class", "blocks-cart-mini-panel-subtotal-line")],
                vec![
                    el("span", vec![], vec![text("小計")]),
                    styled_text::text(&TextProps::default(), vec![], vec![text(SUBTOTAL_TEXT)]),
                ],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("送料・税は購入手続きで計算します。")],
            ),
        ],
    )
}

/// 下端の操作 2 つ（購入手続き・カートを見る）。
fn panel_actions(checkout_label: &str) -> Node {
    div(
        vec![("class", "blocks-cart-mini-panel-actions")],
        vec![
            button(
                &ButtonProps::default(),
                vec![("data-blocks-cart-mini-panel-checkout", "")],
                vec![text(checkout_label)],
            ),
            link::root(
                &cart_summary_href(),
                &LinkProps {
                    variant: LinkVariant::Underline,
                    ..LinkProps::default()
                },
                vec![("data-blocks-cart-mini-panel-view-cart", "")],
                vec![text("カートを見る")],
            ),
        ],
    )
}

/// 1 インスタンス分のミニカートパネル（ヘッダー・見出し・パネル本体）を
/// 組み立てる。`prefix` は本インスタンス内の id を一意にする接頭辞。
fn panel_instance(
    prefix: &str,
    variant_title: &'static str,
    shop_name: &str,
    checkout_label: &str,
    interactive: bool,
) -> Vec<Node> {
    let content_id = format!("{prefix}-content");
    let title_id = format!("{prefix}-title");

    let mut rows = vec![];
    for (index, (name, attrs, price, qty)) in CART_ITEMS.iter().enumerate() {
        if index > 0 {
            rows.push(separator(&SeparatorProps::default(), vec![]));
        }
        rows.push(item_row(
            prefix,
            index,
            name,
            attrs,
            price,
            *qty,
            interactive,
        ));
    }
    rows.push(separator(&SeparatorProps::default(), vec![]));
    rows.push(subtotal_row());
    rows.push(panel_actions(checkout_label));

    vec![
        variant_heading(variant_title),
        div(
            vec![("class", "blocks-cart-mini-panel-instance")],
            vec![popover::root(
                OpenState::Open,
                vec![("data-blocks-cart-mini-panel-variant", prefix)],
                vec![
                    shop_header(shop_name, &content_id),
                    popover::positioner(
                        OpenState::Open,
                        vec![],
                        vec![popover::content(
                            OpenState::Open,
                            Some(&content_id),
                            Some(&title_id),
                            None,
                            vec![("data-blocks-cart-mini-panel-content", "")],
                            vec![
                                popover::title(Some(&title_id), vec![], vec![text("カート")]),
                                div(vec![("class", "blocks-cart-mini-panel-items")], rows),
                            ],
                        )],
                    ),
                ],
            )],
        ),
    ]
}

/// `cart-mini-panel` の Demo 本体。呼び出しごとに同一の `Node` を返す純
/// 関数。basic（固定数量表示）・stepper（readonly 数量ステッパー）の 2
/// インスタンスを縦に並記する（モジュール doc「2 インスタンスの並記」節
/// 参照）。
pub fn demo() -> Node {
    let mut children = panel_instance(
        "blocks-cart-mini-panel-basic",
        "基本形",
        dummy_assets::COMPANY_NAMES[0],
        "購入手続きへ",
        false,
    );
    children.extend(panel_instance(
        "blocks-cart-mini-panel-stepper",
        "数量ステッパー付き",
        dummy_assets::COMPANY_NAMES[1],
        STEPPER_CHECKOUT_LABEL,
        true,
    ));
    div(vec![("class", "blocks-cart-mini-panel-layout")], children)
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/cart-mini-panel/",
    title: "cart-mini-panel",
    category: BlockCategory::Cart,
    rust_source: "crates/docs-site/src/blocks/ecommerce/cart/cart_mini_panel.rs",
    demo_class: "blocks-cart-mini-panel",
    parts: &[
        Part {
            label: "Popover",
            path: "/themes/popover/",
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
            label: "Number Input",
            path: "/themes/number-input/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `cart_mini_panel` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-cart-mini-panel-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-cart-mini-panel-instance {\n  max-width: 24rem;\n  margin-left: auto;\n}\n\
.blocks-cart-mini-panel-shop-header {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  background: var(--fandhe-color-bg-subtle);\n}\n\
.blocks-cart-mini-panel-layout [data-scope=\"popover\"][data-part=\"positioner\"] {\n  position: static;\n  margin-top: var(--fandhe-space-2);\n}\n\
[data-blocks-cart-mini-panel-content] {\n  width: 100%;\n  max-width: none;\n}\n\
.blocks-cart-mini-panel-layout [data-scope=\"popover\"][data-part=\"content\"] h2 {\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n  margin-block-end: var(--fandhe-space-3);\n}\n\
.blocks-cart-mini-panel-items {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-cart-mini-panel-item {\n  display: grid;\n  grid-template-columns: 3rem 1fr auto;\n  gap: var(--fandhe-space-3);\n  align-items: start;\n}\n\
img[data-scope=\"image\"][data-blocks-cart-mini-panel-thumb] {\n  width: 3rem;\n  height: 3rem;\n  flex: none;\n}\n\
.blocks-cart-mini-panel-item-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-cart-mini-panel-item-attrs {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-cart-mini-panel-qty, [data-blocks-cart-mini-panel-qty] {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-cart-mini-panel-item-price {\n  margin: 0;\n  font-weight: var(--fandhe-font-weight-medium, 500);\n  white-space: nowrap;\n}\n\
.blocks-cart-mini-panel-subtotal {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-cart-mini-panel-subtotal-line {\n  display: flex;\n  justify-content: space-between;\n  font-weight: var(--fandhe-font-weight-bold, 700);\n}\n\
.blocks-cart-mini-panel-actions {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  margin-block-start: var(--fandhe-space-1);\n}\n\
[data-blocks-cart-mini-panel-checkout] {\n  flex: 1 1 auto;\n}\n\
[data-blocks-cart-mini-panel-view-cart] {\n  flex: none;\n}\n";

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
            "data-scope=\"popover\"",
            "data-scope=\"button\"",
            "data-scope=\"image\"",
            "data-scope=\"text\"",
            "data-scope=\"number-input\"",
            "data-scope=\"separator\"",
            "data-scope=\"link\"",
            "data-scope=\"heading\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn both_variants_are_rendered() {
        let html = demo_html();
        assert!(
            html.contains("data-blocks-cart-mini-panel-variant=\"blocks-cart-mini-panel-basic\"")
        );
        assert!(
            html.contains("data-blocks-cart-mini-panel-variant=\"blocks-cart-mini-panel-stepper\"")
        );
        assert!(html.contains("基本形"));
        assert!(html.contains("数量ステッパー付き"));
    }

    #[test]
    fn panels_are_open_and_trigger_expanded() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-scope=\"popover\" data-part=\"trigger\"")
                .count(),
            2
        );
        assert_eq!(html.matches("aria-expanded=\"true\"").count(), 2);
        assert_eq!(
            html.matches("data-scope=\"popover\" data-part=\"content\"")
                .count(),
            2
        );
        // 開いた content/positioner は `hidden` 存在属性を持たない。
        assert!(!html.contains(" hidden>"));
        assert!(!html.contains(" hidden "));
    }

    #[test]
    fn no_form_dead_links_or_unescaped_html_opt_in() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
        assert!(html.contains("../../assets/blocks-demo-product.svg"));
    }

    #[test]
    fn checkout_buttons_are_type_button() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-cart-mini-panel-checkout").count(),
            2
        );
        assert!(html.contains("購入手続きへ</button>") || html.contains("購入手続きへ（"));
    }

    #[test]
    fn view_cart_link_points_at_cart_two_column_summary() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-cart-mini-panel-view-cart")
                .count(),
            2
        );
        assert!(html.contains("/blocks/cart-two-column-summary/"));
    }

    #[test]
    fn stepper_number_inputs_are_readonly_and_triggers_disabled() {
        let html = demo_html();
        let qty_count = CART_ITEMS.len();
        assert_eq!(
            html.matches("data-scope=\"number-input\" data-part=\"root\"")
                .count(),
            qty_count
        );
        assert_eq!(
            html.matches("data-scope=\"number-input\" data-part=\"input\"")
                .count(),
            qty_count
        );
        // `data-readonly=""` も部分文字列 `readonly=""` を含むため、native
        // `readonly` 属性（直前が空白）のみを先頭空白込みで数える。
        assert_eq!(html.matches(" readonly=\"\"").count(), qty_count);
        // 増減トリガー 2 × 行数ぶんが disabled。
        assert_eq!(
            html.matches("data-scope=\"number-input\" data-part=\"increment-trigger\"")
                .count(),
            qty_count
        );
        assert_eq!(
            html.matches("data-scope=\"number-input\" data-part=\"decrement-trigger\"")
                .count(),
            qty_count
        );
    }

    #[test]
    fn number_input_ids_are_unique() {
        let html = demo_html();
        for n in 1..=CART_ITEMS.len() {
            let needle = format!("id=\"blocks-cart-mini-panel-stepper-qty-{n}\"");
            assert_eq!(
                html.matches(needle.as_str()).count(),
                1,
                "expected exactly one {needle}"
            );
        }
    }

    #[test]
    fn item_names_and_prices_are_visible() {
        let html = demo_html();
        for (name, _, price, _) in CART_ITEMS {
            assert!(html.contains(name), "missing visible product name: {name}");
            assert!(html.contains(price), "missing visible price: {price}");
        }
    }

    #[test]
    fn qty_stepper_css_hook_survives_drop_class_attr() {
        // Codex/Cursor Bugbot 指摘（PR #3524）の回帰テスト:
        // `number_input::root`（pre-styled-ui）は `drop_class_attr` で呼び
        // 出し側 `class` を除去するため、`class="blocks-cart-mini-panel-qty"`
        // を渡しても LAYOUT_CSS の規則が適用されなかった。CSS フックを
        // `data-blocks-cart-mini-panel-qty` へ切り替え、`root` へ実際に
        // その属性が出力されることを固定する。
        let html = demo_html();
        let qty_count = CART_ITEMS.len();
        assert_eq!(
            html.matches("data-blocks-cart-mini-panel-qty=\"\"").count(),
            qty_count
        );
        assert!(LAYOUT_CSS.contains("[data-blocks-cart-mini-panel-qty]"));
    }

    #[test]
    fn stepper_qty_labels_have_distinct_accessible_names() {
        // Cursor Bugbot 指摘（PR #3524）の回帰テスト: 各商品行の数量入力
        // ラベルが固定文言「数量」のみだと全行で同一になりアクセシブル
        // ネームで区別できないため、商品名を含めて一意にする。
        let html = demo_html();
        for (name, ..) in CART_ITEMS {
            let needle = format!("{name} の数量");
            assert!(
                html.contains(&needle),
                "missing distinct qty label for {name}: {html}"
            );
        }
    }

    #[test]
    fn layout_css_is_safe_and_neutralizes_positioner() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(!LAYOUT_CSS.contains("display: none"));
        assert!(LAYOUT_CSS
            .contains("[data-scope=\"popover\"][data-part=\"positioner\"] {\n  position: static;"));
    }
}
