# list-container

`item` / `card` / `separator` の 3 部品を合成した、一覧リストの「容器」
バリエーションの実例です。Blocks セクションは新規部品を追加するものでは
なく、既存の Themes 部品を組み合わせた実例集であることに注意してください。

行データ（プレースホルダのタイトル + 説明のみ）は共通のまま、次の 4 通りの
容器へ並べます。

1. **区切り線付きの単純リスト**: `item::group` の中で行を
   `separator` で区切って並べます。
2. **外枠付きカード内のリスト**: 1 と同じ骨格を影付きの
   `card`（`Elevated`）の中に収めます。
3. **枠線のみのカード内リスト**: 2 と同じ骨格を枠線のみの `card`
   （`Outline`）に差し替えます。
4. **項目ごとに独立したカード**: 行それぞれを個別の `card` に包み、
   区切り線の代わりに余白で間隔を空けます。

本 docs サイトは JS ハイドレーションを行わない静的出力のため、実際の
コンテナ幅リサイズは示せません。1・2・4 は「通常幅」と「狭幅」の
2 インスタンスを横並びで示し、狭幅側では角丸の解除・全幅化・左右余白の
解除を CSS の `@container` クエリで静的に併記します（3 は狭幅対応の
集約元がないため単一インスタンスのみです）。

いずれも静的な表示例であり、`<form>` 要素・対話要素（ボタン・リンク）を
持ちません。行のタイトル・説明文は独自に書いた架空の文言です。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
use fandhe_frontend_pre_styled_ui::item::{self, ItemRootProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};

/// 各行のダミーデータ（タイトル + 説明、架空の文言）。
const ROWS: [(&str, &str); 4] = [
    (
        "週次レポートの自動生成",
        "毎週月曜 9 時に集計結果を配信します。",
    ),
    (
        "在庫アラートのしきい値見直し",
        "欠品率を抑えるため通知条件を再設定します。",
    ),
    (
        "サポート返信テンプレートの整理",
        "よくある問い合わせへの定型文を見直します。",
    ),
    (
        "バックアップ保存先の切り替え",
        "月次バックアップの保存先を新ストレージへ移行します。",
    ),
];

/// 1 行分の `item::root`。`variant` は
/// `[data-blocks-list-container-row]` フックの値で、`"plain"`
/// （単純リスト行、R1053 の左右余白解除対象）と `"framed"`（カード内・
/// 独立カード行）を区別する（モジュール doc「`item::root` の既定角丸を
/// 解除する理由」節参照）。
fn row(title: &str, description: &str, variant: &str) -> Node {
    item::root(
        ItemRootProps::default(),
        vec![("data-blocks-list-container-row", variant)],
        vec![item::content(
            vec![],
            vec![
                item::title(vec![], vec![text(title)]),
                item::description(vec![], vec![text(description)]),
            ],
        )],
    )
}

/// [`ROWS`] を `variant` フック付きの行 + [`separator::separator`] で
/// 交互に並べた `Vec<Node>`（行数 - 1 本の区切り線）。
fn separated_rows(variant: &str) -> Vec<Node> {
    let mut nodes = Vec::with_capacity(ROWS.len() * 2 - 1);
    for (i, (title, description)) in ROWS.iter().enumerate() {
        if i > 0 {
            nodes.push(separator::separator(&SeparatorProps::default(), vec![]));
        }
        nodes.push(row(title, description, variant));
    }
    nodes
}

/// R1047: 区切り線付きの単純リスト。
fn plain_list() -> Node {
    item::group("シンプルなリスト", vec![], separated_rows("plain"))
}

/// R1048/R1052: `variant` のカードで包んだ区切り線付きリスト。
fn framed_card_list(variant: CardVariant, label: &'static str) -> Node {
    card::root(
        variant,
        vec![("data-blocks-list-container-card", "")],
        vec![card::body(
            vec![("data-blocks-list-container-body", "")],
            vec![item::group(label, vec![], separated_rows("framed"))],
        )],
    )
}

/// R1050: 項目ごとに独立したカード（区切り線の代わりに `gap` で分離する）。
fn independent_cards() -> Node {
    let cards: Vec<Node> = ROWS
        .iter()
        .map(|(title, description)| {
            card::root(
                CardVariant::Elevated,
                vec![("data-blocks-list-container-card", "")],
                vec![card::body(
                    vec![("data-blocks-list-container-body", "")],
                    vec![row(title, description, "framed")],
                )],
            )
        })
        .collect();
    item::group(
        "項目ごとのカード",
        vec![("data-blocks-list-container-cards", "")],
        cards,
    )
}

/// パネル外枠（`@container` の名前付きコンテナ）。`narrow` が `true` なら
/// 幅を制限した狭幅インスタンス用クラスを追加する（モジュール doc「狭幅
/// 差分を `@container` + 幅制限インスタンスで静的に併記する理由」節）。
fn panel(narrow: bool, node: Node) -> Node {
    let class = if narrow {
        "blocks-list-container-panel blocks-list-container-narrow"
    } else {
        "blocks-list-container-panel"
    };
    div(vec![("class", class)], vec![node])
}

/// 通常幅・狭幅の 2 インスタンスを横並びで示す。
fn pair(normal: Node, narrow: Node) -> Node {
    div(
        vec![("class", "blocks-list-container-pair")],
        vec![panel(false, normal), panel(true, narrow)],
    )
}

/// 見出し付きセクションを組み立てる（`list_title_meta` と同型のヘルパ）。
fn section(label: &'static str, node: Node) -> Node {
    div(
        vec![("class", "blocks-list-container-section")],
        vec![
            el(
                "h3",
                vec![("class", "blocks-list-container-section-title")],
                vec![text(label)],
            ),
            node,
        ],
    )
}

/// `list-container` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
/// 主参照（R1047）を先頭に、R1048/R1052/R1050 をラベル付き見出しで区切って
/// 並べる。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-list-container-layout")],
        vec![
            section("区切り線付きの単純リスト", pair(plain_list(), plain_list())),
            section(
                "外枠付きカード内のリスト",
                pair(
                    framed_card_list(CardVariant::Elevated, "外枠付きカードのリスト"),
                    framed_card_list(CardVariant::Elevated, "外枠付きカードのリスト"),
                ),
            ),
            section(
                "枠線のみのカード内リスト",
                panel(
                    false,
                    framed_card_list(CardVariant::Outline, "枠線のみのカードのリスト"),
                ),
            ),
            section(
                "項目ごとに独立したカード",
                pair(independent_cards(), independent_cards()),
            ),
        ],
    )
}
```

## 集約元との差分メモ

- 主参照は R1047（区切り線付きの単純リスト）です。R1048（外枠付きカード）・
  R1052（枠線のみのカード）・R1050（項目ごとに独立したカード）を別
  インスタンスとして併記しています。
- R1049/R1051/R1053（狭幅時の角丸解除・全幅化・左右余白解除）は、実際の
  コンテナ幅リサイズを示せない静的出力の制約上、「通常幅」「狭幅」の
  2 インスタンスを横並びで示し CSS の `@container` クエリで表現しました。
- 行データ・見出しラベルはすべて独自の架空データです。
- ブラウザでの実機確認（コンテナ幅切替・ライト/ダーク両テーマ）は
  サンドボックス制約により未実施です。cargo test による出力検証のみで
  代替しました。
