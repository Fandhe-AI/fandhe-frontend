# cursor-hover-cards

`fandhe-frontend-pre-styled-ui` の `card` 部品を合成した、Motion+ Cursor
（ポインタに spring で追従するカスタムカーソル）に相当する合成例です。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください。

本 Demo は静的な表示例であり、`<form>` 要素を持ちません。カード 3 枚には
それぞれ `data-fandhe-cursor-target`（バリアント名 `"ring"`）を付与し、
2 枚目にはさらに `data-fandhe-cursor-target-label`（hover 中にカーソルへ
表示するラベル）、3 枚目には `data-fandhe-cursor-target-magnetic`
（値なし存在属性、カーソルをカード中心へ吸着させる）を付与しています。
これらはいずれも `fandhe-frontend-wasm-full` の `cursor` feature
（既定 on）が消費する opt-in マーカーです。実アプリで JS ハイドレーションが
有効な場合、カードへポインタが乗るとカスタムカーソルがリング形状へ変化し、
ラベルを表示し、3 枚目では吸着します（`fandhe-frontend-animation::cursor`
の spring 追従演算が `--fandhe-motion-cursor-x`/`-y` の 2 個の CSS カスタム
プロパティを計算・書き込みます）。本 docs サイトは JS ハイドレーションを
一切行わないため、本 Demo ではカーソルの追従・hover バリアント変化は発生
しません（マークアップと opt-in 属性の使い方のみを示す静的な実例です）。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::cursor;

/// `cursor-hover-cards` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    let plain_card = card::root(
        CardProps::from(CardVariant::Outline),
        vec![("data-fandhe-cursor-target", "ring")],
        vec![
            card::title(vec![], vec![text("Explore")]),
            card::description(vec![], vec![text("Hover to see the cursor change shape.")]),
        ],
    );
    let labeled_card = card::root(
        CardProps::from(CardVariant::Outline),
        vec![
            ("data-fandhe-cursor-target", "ring"),
            ("data-fandhe-cursor-target-label", "View"),
        ],
        vec![
            card::title(vec![], vec![text("View details")]),
            card::description(
                vec![],
                vec![text("Hover to see a label attached to the cursor.")],
            ),
        ],
    );
    let magnetic_card = card::root(
        CardProps::from(CardVariant::Outline),
        vec![
            ("data-fandhe-cursor-target", "ring"),
            ("data-fandhe-cursor-target-label", "Focus"),
            ("data-fandhe-cursor-target-magnetic", ""),
        ],
        vec![
            card::title(vec![], vec![text("Magnetic focus")]),
            card::description(
                vec![],
                vec![text("Hover to see the cursor snap to the card center.")],
            ),
        ],
    );

    div(
        vec![("data-blocks-cursor-hover-cards-grid", "")],
        vec![
            plain_card,
            labeled_card,
            magnetic_card,
            cursor::cursor(vec![]),
        ],
    )
}
```
