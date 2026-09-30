//! `cart-two-column-summary` block（イシュー #3033 が骨格・#3034 が残り領域を
//! 実装、親 #3032。Ecommerce / Cart カテゴリの最初の block）。左に商品行の
//! リスト、右に注文サマリの 2 カラムで構成するカート画面。主参照は対応表
//! ID R1247（集約元 R0324/R0681/R0682）。`_/blocks-intake/` の対応ファイルは
//! 本イシュー着手時点で本 worktree に存在しないため、原稿・本コメントには
//! 対応表 ID のみを記し、レイアウトは親 issue #3032 の仕様文に従って独自に
//! 組む（`profile-detail-datalist` #2937 と同じ扱い）。
//!
//! # 使用部品
//!
//! `heading` / `image` / `text` / `native-select` / `button` / `separator` /
//! `data-list` / `card` / `progress` / `link` / `tooltip` の 11 部品を合成
//! する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::card::root`]・
//! [`fandhe_frontend_pre_styled_ui::button::button`]・
//! [`fandhe_frontend_pre_styled_ui::native_select::native_select`]（内部で
//! `fandhe_frontend_headless_ui::field::select` を呼ぶ）・
//! [`fandhe_frontend_pre_styled_ui::data_list::root`]・
//! [`fandhe_frontend_pre_styled_ui::image::image`]・
//! [`fandhe_frontend_pre_styled_ui::link::root`] はいずれも
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
//! `button::button` の既定 `type="button"` のまま用いる。リンクはすべて
//! 固定のリポジトリ URL（[`REPO_URL`]）を指し、`href="#"` は使わない。
//!
//! # 数量 `select` の id・アクセシブルネーム
//!
//! 商品行ごとに `FieldProps::id` を一意にし（`.../qty-<n>`）、
//! `aria-label` も商品名を含めて行ごとに区別する
//! （`format!("{name} の数量")`、`extra_attrs` で付与。
//! `settings_api_keys_table` の失効ボタン `aria-label` と同型の判断）。
//! 実際に出力される `<select>` の `id` は
//! `fandhe_frontend_headless_ui::field` の派生規則により
//! `"{id}-control"` になる。
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
//! # 送料無料までの進捗バー（イシュー #3034、対応表 ID R0682）
//!
//! 左カラム上部に `progress` を 1 本置き、小計（[`SUBTOTAL_TEXT`]）と
//! 送料無料ライン（[`SHIPPING_FREE_THRESHOLD`]）から算出した消費率を示す。
//! `settings_billing_usage.rs` の `usage_bar` と同型に
//! `Progress::new` + styled `root`/`range` を組み、`aria-label` を明示する
//! （`fandhe_frontend_pre_styled_ui::progress::root` は
//! `aria-labelledby` を自動配線しないため）。「あと ¥3,400」の文言は
//! [`SUMMARY_ROWS`] の小計・送料無料ラインと整合させた固定値であり、
//! 実アプリでは呼び出し側が動的に計算する。
//!
//! # 数量選択のヘルプ吹き出し（イシュー #3034、対応表 ID R0681）
//!
//! 1 行目の数量 `select` の隣にのみ `tooltip` を添える
//! （`form_layout_property_panel.rs` の行間欄 tooltip と同型に
//! `OpenState::Open` 固定 + `trigger` を `disabled: true` にし、無 JS の
//! 静的 Demo で常時可視にする）。3 行すべてに常時 open の吹き出しを出すと
//! 縦に重なって読めなくなるため、代表として 1 行目のみに絞る（原稿
//! 「原案差分メモ」節に明記）。`select` 側には `aria-describedby` で
//! tooltip の `content` id を関連付ける（`form_layout_property_panel.rs`
//! イシュー #2914 是正と同型の判断）。tooltip の `positioner` は
//! pre-styled-ui 既定で `position: absolute; bottom: 100%` のため、常時
//! 表示の静的 Demo では価格・削除ボタンと重ならないよう
//! `.blocks-cart-two-column-summary-qty` スコープ内で `position: static`
//! へ中和し、select の直下にインライン表示する
//! （`settings_billing_usage.rs` の toggle tip 中和と同型）。
//!
//! # 入荷待ち行は数量変更不可（状態違いの並記）
//!
//! [`CART_ITEMS`] 3 行目（「ステンレス タンブラー」）は入荷待ちのため、
//! 数量 `select` を `FieldProps::disabled = true` で無効化する（ネイティブ
//! `disabled` 属性がそのまま操作不能化する、`native_select.rs` の
//! `disabled_declarations` 参照）。在庫テキストには
//! `data-blocks-cart-two-column-summary-stock` を、行ルートには
//! `data-blocks-cart-two-column-summary-item-state="backorder"` を付与し、
//! [`LAYOUT_CSS`] が警告色（`--fandhe-color-warning-fg-subtle`）を当てる。
//!
//! # 集計行ごとのヘルプリンク・買い物継続リンク（対応表 ID R1247 の残り・R0324）
//!
//! 小計・送料・税の各集計行ラベルの隣に `link`（`?`、対応表 ID R1247）を
//! 添える。合計行には付けない（合計に説明は不要なため）。サマリカードの
//! `footer` には購入手続きボタンの下へ「買い物を続ける」`link`
//! （R0324、`LinkVariant::Underline`）を積む。いずれも固定のリポジトリ
//! URL（[`REPO_URL`]、`external: true`）を指す（`footer_newsletter.rs` 等
//! 既存 block と同じ判断、`href="#"` は使わない）。
//!
//! # 空カート状態の並記
//!
//! [`demo`] は通常のカート（形 A）に続けて空カート状態（形 B）を縦に並記
//! する（`contact_split_form_info.rs` の 2 分割と同型のパターン、形ラベル
//! は [`variant_label`]）。空カートでは見出しを共有せず、`card` 1 枚に
//! 「カートに商品はありません。」の文言と「買い物を続ける」`link` のみを
//! 置き、2 カラムのサマリは出さない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::native_select::{
    native_select, FieldIds, FieldProps, NativeSelectProps,
};
use fandhe_frontend_pre_styled_ui::progress::{self, Orientation, ProgressProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::tooltip::{self, OpenState};

/// リンク先固定 URL（`footer_newsletter.rs` 等と同じ判断で、実アプリでは
/// 呼び出し側が実 URL に差し替える前提のダミー値）。
const REPO_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 数量ヘルプ tooltip の `content` id（ページ内一意、1 行目のみ使用）。
const QTY_HELP_ID: &str = "blocks-cart-two-column-summary-qty-help";

/// 送料無料ライン（円）。[`shipping_progress`] の進捗バーの `max`。
const SHIPPING_FREE_THRESHOLD: f64 = 50_000.0;

/// [`SUMMARY_ROWS`] の小計と整合させた現在の小計（円、進捗バーの `value`）。
const SUBTOTAL: f64 = 46_600.0;

/// 架空の商品行データ（商品名, 属性表示, 在庫状態, 価格表示, 初期選択数量,
/// 入荷待ちで数量変更不可か）。3 行目のみ `true`（モジュール doc「入荷待ち
/// 行は数量変更不可」節参照）。実在のブランド・商品・PII は含まない。
const CART_ITEMS: &[(&str, &str, &str, &str, u8, bool)] = &[
    (
        "エルゴノミック メッシュチェア",
        "カラー: グレー / サイズ: M",
        "在庫あり",
        "¥24,800",
        1,
        false,
    ),
    (
        "ノイズキャンセリング ヘッドホン",
        "カラー: ブラック",
        "在庫あり",
        "¥18,200",
        1,
        false,
    ),
    (
        "ステンレス タンブラー 500ml",
        "カラー: シルバー",
        "入荷待ち（2〜3 週間）",
        "¥3,600",
        1,
        true,
    ),
];

/// 注文サマリの集計行（ラベル, 値, ヘルプリンクを添えるか）。最終行
/// （合計）だけ強調用の `data-*` を [`summary_totals`] 側で追加し、
/// ヘルプリンクは付けない。
const SUMMARY_ROWS: &[(&str, &str, bool)] = &[
    ("小計", "¥46,600", true),
    ("送料", "¥600", true),
    ("税", "¥4,660", true),
    ("合計", "¥51,860", false),
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

/// 数量ヘルプの tooltip（1 行目のみ。モジュール doc「数量選択のヘルプ
/// 吹き出し」節参照）。`OpenState::Open` 固定・`trigger` は `disabled: true`
/// で無 JS の静的 Demo に合わせる。
fn qty_help_tooltip() -> Node {
    tooltip::root(
        OpenState::Open,
        vec![],
        vec![
            tooltip::trigger(
                OpenState::Open,
                true,
                Some(QTY_HELP_ID),
                vec![("aria-label", "数量についてのヘルプ")],
                vec![text("?")],
            ),
            tooltip::positioner(
                OpenState::Open,
                vec![],
                vec![tooltip::content(
                    OpenState::Open,
                    Some(QTY_HELP_ID),
                    vec![],
                    vec![text("1 回のご注文で選べる数量は最大 5 個です。")],
                )],
            ),
        ],
    )
}

/// 数量 `select`（+ 1 行目のみ隣にヘルプ tooltip）をまとめたコントロール。
fn qty_control(index: usize, name: &str, qty: u8, disabled: bool) -> Node {
    let field_id = format!("blocks-cart-two-column-summary-qty-{}", index + 1);
    let qty_aria_label = format!("{name} の数量");
    let field = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        disabled,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let show_help = index == 0;
    let mut extra_attrs = vec![("aria-label", qty_aria_label.as_str())];
    if show_help {
        extra_attrs.push(("aria-describedby", QTY_HELP_ID));
    }
    let mut children = vec![native_select(
        &NativeSelectProps::default(),
        &field,
        extra_attrs,
        qty_options(qty),
    )];
    if show_help {
        children.push(qty_help_tooltip());
    }
    div(
        vec![("class", "blocks-cart-two-column-summary-qty")],
        children,
    )
}

/// 商品行 1 件（サムネイル + 名称/属性/在庫 + 価格/数量選択/削除）。
/// `index` は 0 始まりで、数量 `select` の一意な `id` の派生に使う。
fn item_row(
    index: usize,
    name: &str,
    attrs: &str,
    stock: &str,
    price: &str,
    qty: u8,
    backorder: bool,
) -> Node {
    let mut row_attrs = vec![];
    if backorder {
        row_attrs.push((
            "data-blocks-cart-two-column-summary-item-state",
            "backorder",
        ));
    }
    div(
        {
            let mut a = vec![("class", "blocks-cart-two-column-summary-item")];
            a.extend(row_attrs);
            a
        },
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
                        vec![("data-blocks-cart-two-column-summary-stock", "")],
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
                    qty_control(index, name, qty, backorder),
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

/// 商品行と行間の `separator` を束ねた左カラム（送料無料進捗 + 商品一覧）。
fn item_list() -> Node {
    let mut children = vec![shipping_progress()];
    for (index, (name, attrs, stock, price, qty, backorder)) in CART_ITEMS.iter().enumerate() {
        if index > 0 {
            children.push(separator(&SeparatorProps::default(), vec![]));
        }
        children.push(item_row(index, name, attrs, stock, price, *qty, *backorder));
    }
    div(
        vec![("class", "blocks-cart-two-column-summary-items")],
        children,
    )
}

/// 送料無料までの進捗バー（モジュール doc「送料無料までの進捗バー」節
/// 参照）。
fn shipping_progress() -> Node {
    let p = Progress::new(
        0.0,
        SHIPPING_FREE_THRESHOLD,
        Some(SUBTOTAL),
        Orientation::Horizontal,
    );
    div(
        vec![("class", "blocks-cart-two-column-summary-shipping")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![],
                vec![text("送料無料まであと ¥3,400")],
            ),
            progress::root(
                &p,
                &ProgressProps {
                    size: Size::Sm,
                    ..ProgressProps::default()
                },
                None,
                vec![("aria-label", "送料無料までの進捗")],
                vec![p.track(vec![], vec![progress::range(&p, vec![])])],
            ),
        ],
    )
}

/// 集計行ラベルへ添えるヘルプリンク（`?`、対応表 ID R1247。合計行には
/// 付けない、モジュール doc「集計行ごとのヘルプリンク」節参照）。
fn summary_help_link(label: &str) -> Node {
    link::root(
        REPO_URL,
        &LinkProps {
            external: true,
            ..LinkProps::default()
        },
        vec![("aria-label", &format!("{label}についてのヘルプ"))],
        vec![text("?")],
    )
}

/// 集計行 1 件（`data_list::item` + `item-label` + `item-value`）。最終行
/// （合計）だけ強調用の `data-blocks-cart-two-column-summary-total` を
/// `item` へ付与する（[`LAYOUT_CSS`] が罫線・フォントウェイトで強調する
/// フック）。`with_help` の行はラベルの隣にヘルプリンクを添える。
fn summary_row(label: &str, value: &str, emphasize: bool, with_help: bool) -> Node {
    let attrs = if emphasize {
        vec![("data-blocks-cart-two-column-summary-total", "")]
    } else {
        vec![]
    };
    let mut label_children = vec![text(label)];
    if with_help {
        label_children.push(summary_help_link(label));
    }
    data_list::item(
        attrs,
        vec![
            data_list::item_label(vec![], label_children),
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
        .map(|(i, (label, value, with_help))| summary_row(label, value, i == last, *with_help))
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

/// 右カラム（注文サマリカード）。`footer` に購入手続きボタンと買い物継続
/// リンクを縦に積む（モジュール doc「集計行ごとのヘルプリンク・買い物継続
/// リンク」節参照）。
fn summary_card() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-cart-two-column-summary-summary", "")],
        vec![
            card::header(vec![], vec![card::title(vec![], vec![text("注文サマリ")])]),
            card::body(vec![], vec![summary_totals()]),
            card::footer(
                vec![],
                vec![
                    button(
                        &ButtonProps::default(),
                        vec![("data-blocks-cart-two-column-summary-checkout", "")],
                        vec![text("購入手続きへ")],
                    ),
                    link::root(
                        REPO_URL,
                        &LinkProps {
                            external: true,
                            variant: LinkVariant::Underline,
                            ..LinkProps::default()
                        },
                        vec![("data-blocks-cart-two-column-summary-continue", "")],
                        vec![text("買い物を続ける")],
                    ),
                ],
            ),
        ],
    )
}

/// 通常のカート（形 A）。送料無料進捗 + 商品一覧 + 注文サマリの 2 カラム。
fn cart_with_items() -> Node {
    div(
        vec![],
        vec![
            variant_label("A: 商品あり"),
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

/// 空カート状態（形 B、モジュール doc「空カート状態の並記」節参照）。
/// 見出しは共有せず、`card` 1 枚に文言 + 買い物継続リンクのみを置く。
fn empty_cart() -> Node {
    div(
        vec![],
        vec![
            variant_label("B: カートが空の状態"),
            card::root(
                CardProps::default(),
                vec![("class", "blocks-cart-two-column-summary-empty")],
                vec![card::body(
                    vec![],
                    vec![
                        styled_text::text(
                            &TextProps::default(),
                            vec![("data-blocks-cart-two-column-summary-variant", "empty")],
                            vec![text("カートに商品はありません。")],
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
                )],
            ),
        ],
    )
}

/// `cart-two-column-summary` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。形 A（商品あり）・形 B（空カート）を縦に並記する
/// （モジュール doc「空カート状態の並記」節参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-cart-two-column-summary-stack")],
        vec![cart_with_items(), empty_cart()],
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
        Part {
            label: "Progress",
            path: "/themes/progress/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Tooltip",
            path: "/themes/tooltip/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `cart_two_column_summary` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-cart-two-column-summary-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  container-type: inline-size;\n  container-name: blocks-cart-two-column-summary;\n}\n\
.blocks-cart-two-column-summary-columns {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr) minmax(16rem, 22rem);\n  gap: var(--fandhe-space-8);\n  align-items: start;\n  margin-block-start: var(--fandhe-space-4);\n}\n\
.blocks-cart-two-column-summary-items {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}\n\
.blocks-cart-two-column-summary-shipping {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-cart-two-column-summary-item {\n  display: flex;\n  gap: var(--fandhe-space-4);\n  align-items: flex-start;\n}\n\
[data-blocks-cart-two-column-summary-thumb] {\n  width: 6rem;\n  height: 6rem;\n  flex: none;\n}\n\
.blocks-cart-two-column-summary-item-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  flex: 1 1 auto;\n  min-width: 0;\n}\n\
.blocks-cart-two-column-summary-item-controls {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-end;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-cart-two-column-summary-qty {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  position: relative;\n}\n\
.blocks-cart-two-column-summary-qty [data-scope=\"tooltip\"][data-part=\"positioner\"] {\n  position: static;\n}\n\
[data-blocks-cart-two-column-summary-item-state=\"backorder\"] [data-blocks-cart-two-column-summary-stock] {\n  color: var(--fandhe-color-warning-fg-subtle);\n}\n\
[data-blocks-cart-two-column-summary-totals] [data-blocks-cart-two-column-summary-total] {\n  border-top: 1px solid var(--fandhe-color-border);\n  padding-top: var(--fandhe-space-2);\n  font-weight: var(--fandhe-font-weight-bold, 700);\n}\n\
[data-blocks-cart-two-column-summary-checkout] {\n  width: 100%;\n}\n\
[data-blocks-cart-two-column-summary-summary] [data-scope=\"card\"][data-part=\"footer\"] {\n  flex-direction: column;\n  align-items: stretch;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-cart-two-column-summary-continue] {\n  align-self: center;\n}\n\
.blocks-cart-two-column-summary-empty {\n  max-width: 32rem;\n}\n\
[data-blocks-cart-two-column-summary-variant=\"empty\"] {\n  display: block;\n  margin-block-end: var(--fandhe-space-3);\n}\n\
@container blocks-cart-two-column-summary (max-width: 48rem) {\n  \
.blocks-cart-two-column-summary-columns {\n    grid-template-columns: minmax(0, 1fr);\n  }\n  \
.blocks-cart-two-column-summary-item-controls {\n    align-items: flex-start;\n  }\n\
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
            "data-scope=\"card\"",
            "data-scope=\"progress\"",
            "data-scope=\"link\"",
            "data-scope=\"tooltip\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<select").count(), 3);
        assert_eq!(html.matches("<option").count(), 15);
        assert_eq!(html.matches("selected=\"\"").count(), 3);
        assert_eq!(html.matches("data-scope=\"separator\"").count(), 2);
        // 削除ボタン 3 + 購入手続きボタン 1 + tooltip trigger 1 = 5。
        assert_eq!(html.matches("<button").count(), 5);
        // ネイティブ `disabled=""`（3 行目の select + tooltip trigger）の 2 件。
        // `data-disabled=""` は末尾が `disabled=""` と一致するため先頭の空白を
        // 含めて区別する。
        assert_eq!(html.matches(" disabled=\"\"").count(), 2);
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
        for (name, _, _, _, _, _) in CART_ITEMS {
            let needle = format!("aria-label=\"{name} の数量\"");
            assert_eq!(
                html.matches(needle.as_str()).count(),
                1,
                "expected exactly one {needle}"
            );
        }
    }

    #[test]
    fn qty_help_tooltip_is_open_and_described() {
        let html = demo_html();
        assert_eq!(html.matches(r#"role="tooltip""#).count(), 1);
        assert_eq!(
            html.matches("id=\"blocks-cart-two-column-summary-qty-help\"")
                .count(),
            1
        );
        assert_eq!(
            html.matches("aria-describedby=\"blocks-cart-two-column-summary-qty-help\"")
                .count(),
            2,
            "trigger + 1 行目 select の 2 箇所"
        );
        assert!(!html.contains("hidden"));
    }

    #[test]
    fn shipping_progress_has_label_and_percent() {
        let html = demo_html();
        assert_eq!(html.matches(r#"role="progressbar""#).count(), 1);
        assert!(html.contains(r#"aria-label="送料無料までの進捗""#));
        assert!(html.contains("--fandhe-progress-percent"));
    }

    #[test]
    fn links_use_fixed_repo_url_not_dead_anchor() {
        let html = demo_html();
        assert!(!html.contains("href=\"#\""));
        // ヘルプ 3（小計/送料/税）+ 継続 1 + 空カート 1 = 5。
        assert_eq!(html.matches("data-scope=\"link\"").count(), 5);
        assert!(!html.contains("data-scope=\"link\" data-part=\"root\" href=\"#\""));
    }

    #[test]
    fn empty_cart_variant_is_rendered() {
        let html = demo_html();
        assert!(html.contains("カートに商品はありません。"));
        assert!(html.contains(r#"data-blocks-cart-two-column-summary-variant="empty""#));
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-cart-two-column-summary (max-width: 48rem)"));
        assert!(LAYOUT_CSS.contains(".blocks-cart-two-column-summary-shipping"));
        assert!(LAYOUT_CSS.contains(".blocks-cart-two-column-summary-qty"));
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
