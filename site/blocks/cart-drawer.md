# cart-drawer

`fandhe-frontend-pre-styled-ui` の `drawer` / `button` / `image` / `text` /
`separator` / `data-list` 部品を合成した、画面右端（inline-end）から出る
カートドロワーの実例です。Blocks セクションは新規部品を追加するものでは
なく、既存の Themes/Primitives 部品を組み合わせた実例集であることに注意
してください（主参照は対応表 ID R1250、集約元は R0679。出典の固有名・
ファイル名は記載しません）。

docs サイトは無 JS のため開閉のスライドインアニメーションは実演できず、
本 Demo はドロワーが**開いた状態**のみを静的に描きます。ヘッダー（タイトル
+ 閉じるボタン）・商品行がスクロールする本文・小計と操作ボタンの固定
フッターの 3 段構成です。固定オーバーレイ（`position: fixed`）は Demo 枠内
に収まるよう中和し、狭い幅（`40rem` 以下）ではドロワーを全幅にします。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。カートを開くボタン・閉じるボタン・購入手続き
ボタン・買い物を続けるボタンはいずれも `type="button"` のまま送信先・
クリック後の挙動を持ちません。商品名・バリエーション・数量・価格は
すべて独自に書いた架空のものであり、実企業名・実商品・PII を含みません。

## Rust コード

```rust
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
```

## 原案差分メモ

- 主参照 R1250（ヘッダ・スクロール本文・固定フッタの 3 段構成）を軸にし、
  集約元 R0679（画面右端からスライドするカート）からは配置（inline-end）
  とカートを開くトリガーボタンの構成要素を取り込んでいます。
- R0679 が持つスライドインアニメーションは、docs サイトが無 JS であり
  headless 層のドロワー開閉も開閉状態を同一フレームで切り替える契約
  （`crates/pre-styled-ui/src/drawer.rs` rustdoc 参照）のため描けません。
  本 Demo は開いた状態のみを静的に固定して示します。
- 狭い幅ではドロワーを全幅にする仕様を、コンテナクエリ（`@container`、
  Demo 枠の幅はビューポート幅と一致しないため `@media` は使いません）で
  実装しています。
- 行削除ボタン・購入手続きボタン・買い物を続けるボタンはいずれも押しても
  何も起きない静的な見本であり、クリック時の挙動（削除・遷移）は持ちません。
- ブラウザでの実機確認（`40rem` 前後のコンテナ幅切替・ライト/ダーク両
  テーマ）はサンドボックス制約により未実施です。cargo test による出力
  検証のみで代替しました。
