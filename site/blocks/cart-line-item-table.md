# cart-line-item-table

`fandhe-frontend-pre-styled-ui` の `heading` / `image` / `text` /
`native-select` / `button` / `separator` / `data-list` 部品を合成した、
列見出し付きの明細表を持つカート画面の実例です。Blocks セクションは新規
部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください（主参照は対応表 ID R0325、集約元は
1 件のみで差分なし。出典の固有名・ファイル名は記載しません）。

上部に見出し、続いて列見出し行（商品 / 数量 / 価格 / 合計）、同じ列割りの
商品行を 3 件、区切り線、明細下に小計・送料・合計と税込注記・購入手続き
ボタン（右寄せ）を配置しています。狭い幅（`40rem` 以下）では列見出し行を
隠し、各行を「商品画像 | 商品情報・数量・価格・合計」の 2 段組みへ組み
替えます（数量・価格・合計は商品情報と同じ列の開始位置に揃えます）。この
とき各セルには列名ラベルを表示し、列見出しが隠れても値の意味が失われない
ようにしています。列見出しが見えている広い幅でも、各セルの列名ラベルは
視覚的にのみ隠して DOM 上に残すため、スクリーンリーダーは値の直前に列名
（数量・価格・合計）を読み上げます。狭幅レイアウトとの両立のため
`<table>`/`<th>`/`<td>` へは置き換えず、`div`/`span` 構造のまま WAI-ARIA
`table`/`rowgroup`/`row`/`columnheader`/`cell` ロールを付与し、支援技術の
テーブルナビゲーションで行数・現在位置・列見出しとの関連を把握できるように
しています。商品情報には在庫状況（「在庫あり」「残り 2 点」等）と行削除
ボタンも並べて表示しています。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。docs-site は無 JS のため選択に応じた再計算は
できず、数量の `native-select` は `disabled` にして初期状態（1 個）の
まま固定表示します（変更可能に見えて金額が追従しない不整合を避けるため）。
行削除ボタンも同じ理由で `disabled` にしており、押しても行・小計は
変化しません。購入手続きボタンは `type="button"` のまま送信先を持ちません。
商品名・バリエーション・価格・在庫状況はすべて独自に書いた架空のもので
あり、実企業名・実商品・PII を含みません。空カート状態・割引行は本
block の対象外です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::native_select::{self, FieldIds, FieldProps, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self, TextProps, TextSize, TextVariant, TextWeight};

/// カート明細行 1 件分の架空データ。
struct LineItem {
    /// 行番号ラベル（`data-blocks-cart-line-item-table-row` の値。select の
    /// `id` 一意化にも同じ文字列を使う）。固定 3 行のため `&'static str`
    /// リテラルで持ち、実行時の文字列生成（`Box::leak` 等）を避ける。
    row: &'static str,
    name: &'static str,
    variant: &'static str,
    unit_price: &'static str,
    total_price: &'static str,
    /// 在庫状況の表示文言（[`stock_status`]。「在庫あり」/「残り N 点」等、
    /// 在庫わずかの状態を 1 行以上に混在させて状態違いを示す。合計値に
    /// 影響しないため在庫切れ行は作らない、#3029）。
    stock: &'static str,
}

/// 明細 3 行（小計はこの 3 行の `total_price` の和と手で一致させている）。
const LINE_ITEMS: &[LineItem] = &[
    LineItem {
        row: "1",
        name: "リネンシャツ",
        variant: "ネイビー / M",
        unit_price: "¥5,800",
        total_price: "¥5,800",
        stock: "在庫あり",
    },
    LineItem {
        row: "2",
        name: "コットンパンツ",
        variant: "ベージュ / L",
        unit_price: "¥6,200",
        total_price: "¥6,200",
        stock: "在庫あり",
    },
    LineItem {
        row: "3",
        name: "キャンバストートバッグ",
        variant: "オフホワイト",
        unit_price: "¥3,400",
        total_price: "¥3,400",
        stock: "残り 2 点",
    },
];

/// 見出し（H2、[`heading`] 部品）。docs ページ自体が H1 を持つため block 内
/// は H2 以下とする（`page_heading_avatar.rs` と同型の判断）。
fn page_heading() -> Node {
    heading::heading(
        HeadingLevel::H2,
        &HeadingProps {
            size: HeadingSize::Lg,
            ..HeadingProps::default()
        },
        vec![],
        vec![text("ショッピングカート")],
    )
}

/// 列見出しセル 1 個（素の `<span>`。[`LAYOUT_CSS`] のグリッド列割りに
/// フックする `data-blocks-cart-line-item-table-col` に加え、`role="columnheader"`
/// で表の列見出しとして支援技術に伝える）。
fn column_header(col: &'static str, label: &'static str) -> Node {
    span(
        vec![
            ("data-blocks-cart-line-item-table-col", col),
            ("role", "columnheader"),
        ],
        vec![text(label)],
    )
}

/// 列見出し行（広幅では通常表示、狭幅では [`LAYOUT_CSS`] の
/// `@container` で [`cell_label`] と同じ clip 手法により視覚的にのみ隠す。
/// `display: none` にすると `role="columnheader"` の見出しごとアクセシビ
/// リティツリーから消え、狭幅時に `role="table"` を付けた目的である列見出し
/// との関連付けがテーブルナビゲーションで得られなくなるため（PR #3461
/// レビュー指摘対応）。`role="row"` で [`row_group`] 配下の商品行と同じ行
/// として関連付ける。
fn column_headers() -> Node {
    div(
        vec![
            ("class", "blocks-cart-line-item-table-head"),
            ("role", "row"),
        ],
        vec![
            column_header("product", "商品"),
            column_header("qty", "数量"),
            column_header("price", "価格"),
            column_header("total", "合計"),
        ],
    )
}

/// 商品サムネイル画像。装飾的なダミー図形のため `alt=""` とし、同じセル内の
/// 可視の商品名との二重読み上げを避ける（`cart_two_column_summary.rs` と同じ扱い）。
fn product_thumbnail() -> Node {
    image::image(
        &ImageProps {
            shape: ImageShape::Rounded,
            fit: ImageFit::Cover,
            ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
        },
        vec![("data-blocks-cart-line-item-table-thumb", "")],
    )
}

/// 在庫状況（`text::text` Muted/Sm）。`class` は剥離されるため
/// （モジュール doc「`text` の `class` 剥離への対応」節参照）レイアウト
/// フックには使わず、代わりに `data-blocks-cart-line-item-table-stock`
/// 属性（`attrs` 経由、`class` と異なり剥離されない）を付けて、他部品と
/// 同様にテスト・将来の CSS フックの対象として識別できるようにする。
fn stock_status(stock: &'static str) -> Node {
    text::text(
        &TextProps {
            variant: TextVariant::Muted,
            size: TextSize::Sm,
            ..TextProps::default()
        },
        vec![("data-blocks-cart-line-item-table-stock", "")],
        vec![text(stock)],
    )
}

/// 行削除ボタン（[`button::button`]）。モジュール doc「静的表示・`<form>`
/// を使わない」節のとおり `disabled` にし、無 JS で押しても行・小計が
/// 追従しない不整合を避ける。可視ラベル「削除」に加え、行ごとに異なる
/// `aria-label` を持たせて同一ページ内の複数「削除」ボタンを区別する
/// （数量 select の `aria-label` と同じ判断軸）。
fn remove_button(item: &LineItem) -> Node {
    let aria_label = format!("{} をカートから削除", item.name);
    button::button(
        &ButtonProps {
            variant: ButtonVariant::Ghost,
            size: Size::Sm,
            disabled: true,
            ..ButtonProps::default()
        },
        vec![("aria-label", aria_label.as_str())],
        vec![text("削除")],
    )
}

/// 商品名・バリエーション・在庫状況・削除ボタンの情報列。`role="table"`
/// の 4 列・12 セル構造（[`role_table_wrapper`]）を変えないため、削除
/// ボタンは独立セルにせず商品セル内のこの列へまとめて置く（#3029）。
fn product_info(item: &LineItem) -> Node {
    div(
        vec![("class", "blocks-cart-line-item-table-info")],
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
            stock_status(item.stock),
            remove_button(item),
        ],
    )
}

/// 商品列（サムネイル + 情報）。`role="cell"` で表セルとして関連付ける。
fn product_cell(item: &LineItem) -> Node {
    div(
        vec![
            ("class", "blocks-cart-line-item-table-product"),
            ("role", "cell"),
        ],
        vec![product_thumbnail(), product_info(item)],
    )
}

/// セルラベル（数量・価格・合計の各値に列の意味を関連付ける）。広幅では
/// [`LAYOUT_CSS`] の clip 手法で視覚的にのみ隠し（`display: none` ではなく
/// DOM 上に残すため、スクリーンリーダーは値の直前に読み上げる）、狭幅の
/// `@container` では通常表示へ切り替える（PR #3461 レビュー指摘対応。
/// `<table>`/`<th>` へ組み替えず、`@container` に応じたセル並べ替えを保った
/// まま列見出しとの意味的関連付けだけを補う判断）。
///
/// # 表としての行・列関連付け（WAI-ARIA `table` ロール、PR #3461 追加指摘対応）
///
/// 上記のセルラベルは各値の列名を補うのみで、行数・現在位置・列見出しとの
/// 関連付けといった表としての操作性までは支援技術に伝わらない
/// （Codex P2 指摘）。`<table>`/`<th>`/`<td>` への置き換えは、`@container`
/// による「狭幅で行を画像＋情報の 2 段へ組み替える」レイアウトと両立しない
/// （多くのブラウザは `display` が `table`/`table-row`/`table-cell` 以外へ
/// 上書きされた要素の暗黙 table ロールを外す）ため、既存の `div`/`span`
/// 構造を保ったまま [`role_table_wrapper`] 以下で `role="table"` /
/// `role="rowgroup"` / `role="row"` / `role="columnheader"` /
/// `role="cell"` を付与し、支援技術のテーブルナビゲーションで行・列を
/// 把握できるようにする。
fn cell_label(label: &'static str) -> Node {
    span(
        vec![("class", "blocks-cart-line-item-table-cell-label")],
        vec![text(label)],
    )
}

/// 数量 `native_select`（[`native_select`] 部品）。行ごとに `id` を一意化
/// し、`aria-label` でアクセシブルネームを持たせる（モジュール doc
/// 「数量セレクトのアクセシブルネーム」節参照）。option は 1〜3 個を表示
/// しつつ `disabled` にして選択値を 1 個に固定する（モジュール doc
/// 「静的表示・`<form>` を使わない」節参照。無 JS のため選択操作しても
/// 金額を追従できず、操作可能に見える不整合を避ける）。
fn quantity_select(item: &LineItem) -> Node {
    let id = format!("blocks-cart-line-item-table-qty-{}", item.row);
    let field = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: true,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let aria_label = format!("{} の数量", item.name);
    let options: Vec<Node> = (1..=3)
        .map(|n| {
            let value = n.to_string();
            let mut attrs = vec![("value", value.as_str())];
            if n == 1 {
                attrs.push(("selected", ""));
            }
            el("option", attrs, vec![text(n.to_string())])
        })
        .collect();
    native_select::native_select(
        &NativeSelectProps::default(),
        &field,
        vec![("aria-label", aria_label.as_str())],
        options,
    )
}

/// 数量セル（狭幅ラベル + select）。`role="cell"` で表セルとして関連付ける。
fn quantity_cell(item: &LineItem) -> Node {
    div(
        vec![
            ("class", "blocks-cart-line-item-table-cell"),
            ("data-blocks-cart-line-item-table-col", "qty"),
            ("role", "cell"),
        ],
        vec![cell_label("数量"), quantity_select(item)],
    )
}

/// 単価セル。`role="cell"` で表セルとして関連付ける。
fn price_cell(item: &LineItem) -> Node {
    div(
        vec![
            ("class", "blocks-cart-line-item-table-cell"),
            ("data-blocks-cart-line-item-table-col", "price"),
            ("role", "cell"),
        ],
        vec![cell_label("価格"), text(item.unit_price)],
    )
}

/// 行合計セル。`role="cell"` で表セルとして関連付ける。
fn total_cell(item: &LineItem) -> Node {
    div(
        vec![
            ("class", "blocks-cart-line-item-table-cell"),
            ("data-blocks-cart-line-item-table-col", "total"),
            ("role", "cell"),
        ],
        vec![cell_label("合計"), text(item.total_price)],
    )
}

/// 商品行 1 件。`role="row"` で [`row_group`] 配下の表行として関連付ける。
fn line_item_row(item: &LineItem) -> Node {
    div(
        vec![
            ("class", "blocks-cart-line-item-table-row"),
            ("data-blocks-cart-line-item-table-row", item.row),
            ("role", "row"),
        ],
        vec![
            product_cell(item),
            quantity_cell(item),
            price_cell(item),
            total_cell(item),
        ],
    )
}

/// 商品行一覧。`role="rowgroup"`（HTML `<tbody>` 相当）で
/// [`column_headers`] の行と区別しつつ表構造の一部として関連付ける。
fn line_item_rows() -> Node {
    div(
        vec![
            ("class", "blocks-cart-line-item-table-rows"),
            ("role", "rowgroup"),
        ],
        LINE_ITEMS.iter().map(line_item_row).collect(),
    )
}

/// [`column_headers`] と [`line_item_rows`] を包む `role="table"` の外枠。
/// ARIA table ロールでは所有要素が `row`/`rowgroup` のみであることが
/// 期待されるため、見出し・区切り線・小計等を含む [`demo`] の最上位 `div`
/// とは別に、表本体だけを囲む専用ラッパーを設ける。
fn role_table_wrapper() -> Node {
    div(
        vec![
            ("class", "blocks-cart-line-item-table-table"),
            ("role", "table"),
            ("aria-label", "カート明細"),
        ],
        vec![column_headers(), line_item_rows()],
    )
}

/// 小計・送料・合計 + 税込注記 + 購入手続きボタン（右寄せ）。小計
/// （¥15,400）+ 送料（¥600）= 合計（¥16,000）で値を手で一致させている。
/// 割引行は親仕様（#3027）にないため追加しない（#3029）。
fn summary() -> Node {
    let subtotal_item = data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text("小計")]),
            data_list::item_value(vec![], vec![text("¥15,400")]),
        ],
    );
    let shipping_item = data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text("送料")]),
            data_list::item_value(vec![], vec![text("¥600")]),
        ],
    );
    let total_item = data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text("合計")]),
            data_list::item_value(vec![], vec![text("¥16,000")]),
        ],
    );
    let list = data_list::root(
        DataListProps {
            orientation: DataListOrientation::Horizontal,
            ..DataListProps::default()
        },
        vec![],
        vec![subtotal_item, shipping_item, total_item],
    );
    let tax_note = text::text(
        &TextProps {
            variant: TextVariant::Muted,
            size: TextSize::Sm,
            ..TextProps::default()
        },
        vec![],
        vec![text("価格はすべて税込です。")],
    );
    let checkout_button = button::button(
        &ButtonProps {
            variant: ButtonVariant::Solid,
            ..ButtonProps::default()
        },
        vec![],
        vec![text("購入手続きへ進む")],
    );
    div(
        vec![("class", "blocks-cart-line-item-table-summary")],
        vec![list, tax_note, checkout_button],
    )
}

/// `cart-line-item-table` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-cart-line-item-table-layout")],
        vec![
            page_heading(),
            role_table_wrapper(),
            separator::separator(&SeparatorProps::default(), vec![]),
            summary(),
        ],
    )
}
```

## 原案差分メモ

- 主参照 R0325 のみを集約元としており、他ファイルとの差分はありません。
- 親 #3027 は規模 L のため #3028（骨格・主要領域・登録一式）/ #3029
  （行削除・送料行・在庫状況等の残り領域と原稿仕上げ）へ 2 分割して実装
  しました。空カート状態・割引行は親仕様にないため対象外です。
- 実データ取得・数量変更・行削除・購入手続きは行わず、静的な初期状態の
  みを示します。商品名・バリエーション・価格・在庫状況・小計・送料・
  合計はすべて独自の架空データであり、小計（¥15,400）+ 送料（¥600）=
  合計（¥16,000）と手で一致させています。
- ブラウザでの実機確認（`40rem` 前後のコンテナ幅切替・ライト/ダーク両
  テーマ）はサンドボックス制約により未実施です。cargo test による出力
  検証のみで代替しました。
