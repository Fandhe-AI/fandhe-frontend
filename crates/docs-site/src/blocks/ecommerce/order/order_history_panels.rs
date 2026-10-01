//! `order-history-panels` block（イシュー #3056/#3057、親 #3055。
//! Ecommerce / Order カテゴリ）。注文ごとに枠付きパネルを縦に積む注文履歴の
//! 骨格・主要領域を合成する。主参照 R1116（サマリ帯 + 商品行の代表構成、
//! 狭幅でサマリ操作を三点メニューへ集約）を軸とする。`_/blocks-intake/` の
//! 対応ファイルは本イシュー着手時点で本 worktree に存在しないため、原稿・
//! 本コメントには対応表 ID のみを記す（`order_tracking_progress`〔イシュー
//! #3060/#3061〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `card` / `data-list` / `button` / `menu` / `image` / `text` / `heading` /
//! `link` / `separator` / `icon` の 10 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新しい UI 部品は追加しない。
//!
//! # 構成
//!
//! 注文 1 件 = `card` 1 枚。パネル上部はサマリ帯（注文番号・注文日・合計の
//! 3 項目 + 「注文を見る」「請求書を見る」ボタン・三点メニュー）、区切り線を
//! 挟んでパネル下部は商品行（画像・名称・価格・説明 + 「商品を見る」
//! リンク）の並びとする。本件では 2 注文（2〜3 商品ずつ）を縦に積む。
//!
//! # 狭幅ではサマリ操作をメニュー化し注文日を隠す（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため `@container`
//! （コンテナクエリ、`order_tracking_progress` と同型のパターン）で判定する。
//! [`LAYOUT_CSS`] のレイアウト root へ `container-type: inline-size` を
//! 宣言し、コンテナ幅が既定（40rem 未満）では「注文を見る」「請求書を見る」
//! ボタンと注文日の行を隠して三点メニューのみを見せ、`40rem` 以上では
//! ボタン・注文日を見せて三点メニューを隠す（R1116 の差分要件）。メニューの
//! 2 項目（「注文を見る」「請求書を見る」）はボタンと同じラベルの項目を
//! DOM 上に併せ持つ、という構成上の対応関係であり、実際に狭幅で操作に
//! 到達できることを示すものではない（次節のとおりメニュー自体も
//! `trigger: disabled` + `content: OpenState::Closed` の静的 Demo で
//! JS による開閉を持たず、対応するボタンも `40rem` 未満では CSS で
//! 非表示になる。狭幅時に実際に操作可能な手段は本 Demo に存在しない）。
//!
//! # 操作要素はすべて `disabled` で固定する
//!
//! 本 Demo は無 JS の docs サイトで静的な初期状態のみを示す（JS
//! ハイドレーションを行わない）。押しても何も起きない要素を操作可能に
//! 見せないため、2 個のボタン（`ButtonProps { disabled: true, .. }`）と
//! `menu::trigger` の `disabled: true`（第 2 引数）を固定する
//! （`list_title_meta`/`order_tracking_progress` と同型の判断）。
//!
//! # `menu` の id をページ内で一意にする理由
//!
//! `demo_output_has_no_dangling_aria_references_or_duplicate_ids`
//! （`crates/docs-site/tests/blocks_contract.rs`）が id 重複・参照切れを
//! fail-closed に検知するため、`menu::trigger`/`menu::content` の id は
//! 注文ごとに固定の `&'static str` 定数として持つ（[`Order::menu_id`]/
//! [`Order::trigger_id`]）。三点メニューの trigger は自作の線画アイコン
//! （装飾用途、`aria-hidden`）のみを子に持つため、`aria-label` で
//! 「その他の操作、注文 {番号}」を注文ごとに一意に供給する
//! （`list_title_meta` の可視テキスト + `visually_hidden` と異なり、本 block
//! は `visually_hidden` を `parts` に含めないため `aria-label` 属性で供給する）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、注文処理・決済・送信先を一切持たない。ボタンは
//! `button::button` の既定 `type="button"` のまま用いる。
//!
//! # `href` は外部絶対 URL のダミー値
//!
//! 商品リンクは `href="#"` を避け、ダミーでも意味の通る URL
//! （`https://example.com/products/…`）を使う。サイト内相対パスは
//! `linkcheck::check_links`（`crates/docs-site/tests/support/shared_site.rs`）が
//! 「実在しないページ」として fail-closed に検知するため使えない
//! （`order_tracking_progress` と同型の判断）。
//!
//! # ダミー素材について
//!
//! 注文番号・日付・商品名・価格はすべて架空の独自文言（実在の人物・企業・
//! 商品とは無関係）。商品画像はビルド時生成の同梱 SVG
//! （[`dummy_assets::PRODUCT_SRC`]）を使う（外部 URL・`data:` URI は使わない）。
//! 商品価格は円建てで固定し、各注文の商品価格合計と合計表示を一致させる
//! （注文 1: ¥8,200 + ¥1,400 = ¥9,600、注文 2: ¥12,800 + ¥3,200 + ¥1,000 =
//! ¥17,000）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応、イシュー #3057）
//!
//! 配送状況の状態表示（R1118）・商品行の「再購入」「類似品を見る」ボタン
//! （R1119）は本 block（#3056）では実装しない。後続イシュー #3057 で追加
//! する（詳細は `site/blocks/order-history-panels.md` の差分メモ節参照）。
//! 注文見出し（h3、R1119 の一部）はレビュー指摘対応（PR #3509）で本イシュー
//! にて先行実装済み。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::state::OpenState;
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::menu;
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 商品 1 件分のダミーデータ。
struct Item {
    name: &'static str,
    description: &'static str,
    price: &'static str,
    href: &'static str,
}

/// 注文 1 件分のダミーデータ。[`menu_id`]/[`trigger_id`] は
/// `menu::content`/`menu::trigger` を紐づける注文固有の一意 id（モジュール
/// 冒頭「`menu` の id をページ内で一意にする理由」節参照）。
struct Order {
    number: &'static str,
    date: &'static str,
    total: &'static str,
    menu_id: &'static str,
    trigger_id: &'static str,
    items: &'static [Item],
}

const ORDERS: [Order; 2] = [
    Order {
        number: "FD-3102-220",
        date: "2026-09-18",
        total: "¥9,600",
        menu_id: "blocks-order-history-panels-menu-0",
        trigger_id: "blocks-order-history-panels-menu-trigger-0",
        items: &[
            Item {
                name: "ノイズキャンセリングイヤホン",
                description: "ブラック / Bluetooth 5.3",
                price: "¥8,200",
                href: "https://example.com/products/noise-cancelling-earbuds",
            },
            Item {
                name: "USB-C 急速充電ケーブル 1m",
                description: "グレー",
                price: "¥1,400",
                href: "https://example.com/products/usb-c-cable-1m",
            },
        ],
    },
    Order {
        number: "FD-3102-231",
        date: "2026-09-25",
        total: "¥17,000",
        menu_id: "blocks-order-history-panels-menu-1",
        trigger_id: "blocks-order-history-panels-menu-trigger-1",
        items: &[
            Item {
                name: "メカニカルキーボード",
                description: "茶軸 / 日本語配列",
                price: "¥12,800",
                href: "https://example.com/products/mechanical-keyboard",
            },
            Item {
                name: "静音マウス",
                description: "ホワイト",
                price: "¥3,200",
                href: "https://example.com/products/silent-mouse",
            },
            Item {
                name: "リストレスト",
                description: "低反発ウレタン",
                price: "¥1,000",
                href: "https://example.com/products/wrist-rest",
            },
        ],
    },
];

/// サマリ帯の定義リスト 1 行（ラベル + 値）。`extra_attrs` は注文日の行にだけ
/// 狭幅非表示フック（`data-blocks-order-history-panels-date`）を渡すために使う。
fn summary_row(
    label: &'static str,
    value: &'static str,
    extra_attrs: Vec<(&'static str, &'static str)>,
) -> Node {
    data_list::item(
        extra_attrs,
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 三点メニューの trigger に使う自作の線画アイコン（装飾用途、`aria-hidden`。
/// 参照元の絵柄アイコンは持ち込まない）。
fn more_actions_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![
            el(
                "circle",
                vec![("cx", "12"), ("cy", "5"), ("r", "1.6")],
                vec![],
            ),
            el(
                "circle",
                vec![("cx", "12"), ("cy", "12"), ("r", "1.6")],
                vec![],
            ),
            el(
                "circle",
                vec![("cx", "12"), ("cy", "19"), ("r", "1.6")],
                vec![],
            ),
        ],
    )
}

/// サマリ帯の三点メニュー（「注文を見る」「請求書を見る」。狭幅で隠れる
/// ボタン 2 個と同じラベルの項目を DOM 上に持つのみで、メニュー自体も
/// 静的 Demo のため実際の開閉操作は提供しない。モジュール冒頭「狭幅では
/// サマリ操作をメニュー化し注文日を隠す」節参照）。
fn action_menu(order: &Order) -> Node {
    let aria_label = format!("その他の操作、注文 {}", order.number);
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![
            menu::trigger(
                OpenState::Closed,
                true,
                Some(order.menu_id),
                vec![
                    ("id", order.trigger_id),
                    ("aria-label", aria_label.as_str()),
                ],
                vec![more_actions_icon()],
            ),
            menu::positioner(
                OpenState::Closed,
                vec![],
                vec![menu::content(
                    OpenState::Closed,
                    Some(order.menu_id),
                    Some(order.trigger_id),
                    vec![],
                    vec![
                        menu::item("view", false, false, vec![], vec![text("注文を見る")]),
                        menu::separator(vec![], vec![]),
                        menu::item("invoice", false, false, vec![], vec![text("請求書を見る")]),
                    ],
                )],
            ),
        ],
    )
}

/// パネル上部のサマリ帯（`card::header`。注文見出し + 定義リスト + 操作群）。
/// 注文見出し（`<h3>`）を商品見出し（`<h4>`、[`item_row`]）より前に置き、
/// 見出し一覧（スクリーンリーダーの見出しナビゲーション）で商品がどの注文に
/// 属するか判別できるようにする（レビュー指摘対応、イシュー #3056）。
/// 定義リスト + 操作群は `card::header` 本体ではなく内側の
/// `.blocks-order-history-panels-summary-row` へ横並びレイアウトを持たせる
/// （`card::header` 自体に付けると recipe の `[data-scope="card"]
/// [data-part="header"]` セレクタ〔attribute 2 個、本 block の単一 class
/// より高い詳細度〕に `flex-direction: column` で負け、横並びにならない。
/// レビュー指摘対応、イシュー #3056）。
fn summary(order: &Order) -> Node {
    card::header(
        vec![("class", "blocks-order-history-panels-summary")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![("class", "blocks-order-history-panels-order-heading")],
                vec![text(format!("注文 {}", order.number))],
            ),
            div(
                vec![("class", "blocks-order-history-panels-summary-row")],
                vec![
                    data_list::root(
                        DataListProps {
                            orientation: DataListOrientation::Vertical,
                            ..DataListProps::default()
                        },
                        vec![("data-blocks-order-history-panels-summary-list", "")],
                        vec![
                            summary_row("注文番号", order.number, vec![]),
                            summary_row(
                                "注文日",
                                order.date,
                                vec![("data-blocks-order-history-panels-date", "")],
                            ),
                            summary_row("合計", order.total, vec![]),
                        ],
                    ),
                    div(
                        vec![("class", "blocks-order-history-panels-actions")],
                        vec![
                            button(
                                &ButtonProps {
                                    variant: ButtonVariant::Outline,
                                    size: Size::Sm,
                                    disabled: true,
                                    ..ButtonProps::default()
                                },
                                vec![("data-blocks-order-history-panels-action-button", "")],
                                vec![text("注文を見る")],
                            ),
                            button(
                                &ButtonProps {
                                    variant: ButtonVariant::Outline,
                                    size: Size::Sm,
                                    disabled: true,
                                    ..ButtonProps::default()
                                },
                                vec![("data-blocks-order-history-panels-action-button", "")],
                                vec![text("請求書を見る")],
                            ),
                            action_menu(order),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 商品 1 行分（画像・名称・価格・説明 + 「商品を見る」リンク）。可視テキストは
/// 5 件とも「商品を見る」で同じになるため、`aria-label` で商品名を含む
/// アクセシブルネーム（例: 「ノイズキャンセリングイヤホンを見る」）を供給し、
/// リンク一覧での行き先判別を可能にする（レビュー指摘対応、イシュー #3056）。
fn item_row(item: &Item) -> Node {
    div(
        vec![("class", "blocks-order-history-panels-item")],
        vec![
            image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, item.name)
                },
                vec![("data-blocks-order-history-panels-image", "")],
            ),
            div(
                vec![("class", "blocks-order-history-panels-item-body")],
                vec![
                    div(
                        vec![("class", "blocks-order-history-panels-item-heading")],
                        vec![
                            heading(
                                HeadingLevel::H4,
                                &HeadingProps::default(),
                                vec![],
                                vec![text(item.name)],
                            ),
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(item.price)],
                            ),
                        ],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(item.description)],
                    ),
                ],
            ),
            {
                let aria_label = format!("{}を見る", item.name);
                link::root(
                    item.href,
                    &LinkProps::default(),
                    vec![
                        ("data-blocks-order-history-panels-item-link", ""),
                        ("aria-label", aria_label.as_str()),
                    ],
                    vec![text("商品を見る")],
                )
            },
        ],
    )
}

/// 注文 1 件分のパネル（`card::root`。サマリ帯 + 区切り線 + 商品行の並び）。
fn panel(order: &Order) -> Node {
    let mut body_children: Vec<Node> = Vec::new();
    for (index, item) in order.items.iter().enumerate() {
        if index > 0 {
            body_children.push(separator(&SeparatorProps::default(), vec![]));
        }
        body_children.push(item_row(item));
    }
    card::root(
        CardProps::default(),
        vec![("data-blocks-order-history-panels-panel", "")],
        vec![
            summary(order),
            separator(&SeparatorProps::default(), vec![]),
            card::body(
                vec![("class", "blocks-order-history-panels-items")],
                body_children,
            ),
        ],
    )
}

/// `order-history-panels` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-order-history-panels-layout")],
        ORDERS.iter().map(panel).collect(),
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/order-history-panels/",
    title: "order-history-panels",
    category: BlockCategory::Order,
    rust_source: "crates/docs-site/src/blocks/ecommerce/order/order_history_panels.rs",
    demo_class: "blocks-order-history-panels",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
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
            label: "Menu",
            path: "/themes/menu/",
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
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `order_history_panels` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。既定（40rem 未満）ではサマリの
/// ボタン・注文日を隠して三点メニューのみを見せ、`40rem` 以上では逆に
/// ボタン・注文日を見せてメニューを隠す（モジュール冒頭「狭幅では
/// サマリ操作をメニュー化し注文日を隠す」節参照）。
///
/// `.blocks-order-history-panels-summary-row`（定義リスト + 操作群の横並び）は
/// `card::header` 本体（`[data-scope="card"][data-part="header"]`、attribute
/// セレクタ 2 個で詳細度 `(0,2,0)`）ではなく内側の子 `div` へ適用する
/// （単一 class セレクタ `(0,1,0)` は header 本体に直接付けると recipe の
/// `flex-direction: column` に負けて縦積みのまま戻ってしまうため、
/// recipe が触れない子要素側へレイアウトを持たせて詳細度勝負を避ける。
/// レビュー指摘対応、イシュー #3056）。
///
/// 狭幅用の非表示セレクタ（action-button / date）は、フック属性単体
/// （`(0,1,0)`）ではなく `[data-scope][data-part]` を併記して詳細度を
/// 上げる（`[data-blocks-order-history-panels-action-button]
/// [data-scope="button"][data-part="root"]` / `[data-blocks-order-history-panels-date]
/// [data-scope="data-list"][data-part="item"]`、ともに `(0,3,0)`）。Button
/// recipe の `[data-scope="button"][data-part="root"] { display:
/// inline-flex }` と Data List recipe の `[data-scope="data-list"]
/// [data-part="item"] { display: var(...) }` が同じ `(0,2,0)` を持つため、
/// フック属性単体では負けて 40rem 未満でも隠れない（レビュー指摘対応、
/// イシュー #3056）。
///
/// サマリの定義リスト（`[data-blocks-order-history-panels-summary-list]
/// [data-scope="data-list"][data-part="root"]`）は `display` と並べて
/// `flex-direction: row` も明示する。Data List recipe の `[data-scope=
/// "data-list"][data-part="root"] { flex-direction: column }` は同じ
/// `(0,2,0)` のため、`display` だけ上書きしても `flex-direction` は
/// recipe 側が勝ち続け、3 フィールドが縦積みのまま戻ってしまう
/// （レビュー指摘対応、PR #3509）。
///
/// 商品画像の `height: 5rem` は、フック属性単体（`(0,1,0)`）のままでは
/// Image recipe の `[data-scope="image"][data-part="root"] { height: auto }`
/// （`(0,2,0)`）に負けて `object-fit: cover` のクロップが効かないため、
/// 上記 action-button / date と同じ `[data-scope][data-part]` 併記
/// （`(0,3,0)`）に分離して明示する。`grid-row`/`width` は hook 属性のみ
/// （`(0,1,0)`）のままで recipe と競合しないため分離不要
/// （レビュー指摘対応、PR #3509）。
const LAYOUT_CSS: &str = "\
.blocks-order-history-panels-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-order-history-panels;\n}\n\
.blocks-order-history-panels-summary-row {\n  display: flex;\n  flex-direction: row;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-order-history-panels-summary-list][data-scope=\"data-list\"][data-part=\"root\"] {\n  display: flex;\n  flex-direction: row;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-order-history-panels-actions {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  margin-inline-start: auto;\n}\n\
[data-blocks-order-history-panels-action-button][data-scope=\"button\"][data-part=\"root\"] {\n  display: none;\n}\n\
[data-blocks-order-history-panels-date][data-scope=\"data-list\"][data-part=\"item\"] {\n  display: none;\n}\n\
.blocks-order-history-panels-items {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-order-history-panels-item {\n  display: grid;\n  grid-template-columns: 5rem minmax(0, 1fr);\n  gap: var(--fandhe-space-2) var(--fandhe-space-4);\n  align-items: start;\n}\n\
[data-blocks-order-history-panels-image] {\n  grid-row: 1 / 3;\n  width: 100%;\n}\n\
[data-blocks-order-history-panels-image][data-scope=\"image\"][data-part=\"root\"] {\n  height: 5rem;\n}\n\
.blocks-order-history-panels-item-body {\n  grid-column: 2;\n  grid-row: 1;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-order-history-panels-item-heading {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: baseline;\n  justify-content: space-between;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-order-history-panels-item-link] {\n  grid-column: 2;\n  grid-row: 2;\n  justify-self: start;\n}\n\
@container blocks-order-history-panels (min-width: 40rem) {\n  \
[data-blocks-order-history-panels-action-button][data-scope=\"button\"][data-part=\"root\"] {\n    display: inline-flex;\n  }\n  \
[data-blocks-order-history-panels-date][data-scope=\"data-list\"][data-part=\"item\"] {\n    display: flex;\n  }\n  \
.blocks-order-history-panels-actions [data-scope=\"menu\"][data-part=\"root\"] {\n    display: none;\n  }\n  \
.blocks-order-history-panels-item {\n    grid-template-columns: 5rem minmax(0, 1fr) auto;\n  }\n  \
[data-blocks-order-history-panels-image] {\n    grid-row: 1;\n  }\n  \
[data-blocks-order-history-panels-item-link] {\n    grid-column: 3;\n    grid-row: 1;\n    justify-self: end;\n  }\n\
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
            "data-scope=\"card\"",
            "data-scope=\"data-list\"",
            "data-scope=\"button\"",
            "data-scope=\"menu\"",
            "data-scope=\"image\"",
            "data-scope=\"text\"",
            "data-scope=\"heading\"",
            "data-scope=\"link\"",
            "data-scope=\"separator\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(
            html.matches("data-scope=\"card\" data-part=\"root\"")
                .count(),
            2
        );
        assert_eq!(html.matches("<button").count(), 6);
        assert_eq!(
            html.matches("data-scope=\"menu\" data-part=\"root\"")
                .count(),
            2
        );
        // 注文見出し（`<h3>`）2 件（注文ごと）+ 商品見出し（`<h4>`）5 件
        // （2 + 3 商品）。h3 が h4 より前に出て注文→商品の見出し階層になる
        // ことは `order_heading_precedes_item_headings` が別途検証する。
        assert_eq!(html.matches("<h3").count(), 2);
        assert_eq!(html.matches("<h4").count(), 5);
    }

    #[test]
    fn order_heading_precedes_item_headings() {
        let html = demo_html();
        let h3_0 = html.find("<h3").expect("h3 should exist");
        let h4_0 = html.find("<h4").expect("h4 should exist");
        assert!(
            h3_0 < h4_0,
            "注文見出し（h3）は商品見出し（h4）より前に出るべき"
        );
        assert!(html.contains(">注文 FD-3102-220<"));
        assert!(html.contains(">注文 FD-3102-231<"));
    }

    #[test]
    fn item_links_have_distinct_accessible_names() {
        let html = demo_html();
        let mut labels: Vec<&str> = Vec::new();
        for chunk in html.split("aria-label=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                let label = &chunk[..end];
                // メニューの「その他の操作」aria-label は商品リンクとは別物
                // なので除外する（`action_menu` が注文ごとに供給、本テストは
                // 商品リンクの判別可否のみを検証する）。
                if label.ends_with("を見る") && !label.starts_with("その他の操作") {
                    labels.push(label);
                }
            }
        }
        assert_eq!(labels.len(), 5, "商品リンクは 5 件: {labels:?}");
        let mut sorted = labels.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            labels.len(),
            sorted.len(),
            "商品リンクのアクセシブルネームが重複している: {labels:?}"
        );
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
    fn menu_ids_are_unique_and_referenced() {
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
        assert!(html.contains("id=\"blocks-order-history-panels-menu-trigger-0\""));
        assert!(html.contains("id=\"blocks-order-history-panels-menu-trigger-1\""));
        assert!(html.contains("aria-controls=\"blocks-order-history-panels-menu-0\""));
        assert!(html.contains("aria-controls=\"blocks-order-history-panels-menu-1\""));
        assert!(html.contains("aria-labelledby=\"blocks-order-history-panels-menu-trigger-0\""));
        assert!(html.contains("aria-labelledby=\"blocks-order-history-panels-menu-trigger-1\""));
    }

    #[test]
    fn order_total_matches_item_sum() {
        let html = demo_html();
        // 注文 1: 8,200 + 1,400 = 9,600。注文 2: 12,800 + 3,200 + 1,000 = 17,000。
        assert!(html.contains("¥9,600"));
        assert!(html.contains("¥17,000"));
    }

    #[test]
    fn action_controls_are_natively_disabled() {
        let html = demo_html();
        // ボタン 2 個 × 注文 2 件 = 4、menu::trigger × 2 = 2、計 6 個の
        // disabled 操作要素（モジュール冒頭「操作要素はすべて disabled で
        // 固定する」節参照）。
        assert_eq!(html.matches(" disabled=\"\"").count(), 6);
    }

    #[test]
    fn layout_css_is_safe_and_collapses_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-order-history-panels (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-order-history-panels-action-button][data-scope=\"button\"][data-part=\"root\"] {\n  display: none;\n}"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-order-history-panels-action-button][data-scope=\"button\"][data-part=\"root\"] {\n    display: inline-flex;\n  }"
        ));
    }

    /// レビュー指摘対応（PR #3509）: サマリ一覧の `flex-direction: row` が
    /// Data List recipe の `column`（同じ詳細度 `(0,2,0)`）に負けず明示
    /// されていることを固定する。
    #[test]
    fn summary_list_overrides_flex_direction_to_row() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-order-history-panels-summary-list][data-scope=\"data-list\"][data-part=\"root\"] {\n  display: flex;\n  flex-direction: row;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-6);\n}"
        ));
    }

    /// レビュー指摘対応（PR #3509）: 商品画像の `height: 5rem` が
    /// `[data-scope="image"][data-part="root"]` 併記で Image recipe の
    /// `height: auto`（`(0,2,0)`）より高い詳細度（`(0,3,0)`）を持つことを
    /// 固定する。
    #[test]
    fn item_image_height_outranks_image_recipe() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-order-history-panels-image][data-scope=\"image\"][data-part=\"root\"] {\n  height: 5rem;\n}"
        ));
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }
}
