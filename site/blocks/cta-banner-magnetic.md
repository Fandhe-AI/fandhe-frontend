# cta-banner-magnetic

`fandhe-frontend-pre-styled-ui` の `button` / `heading` 部品を合成した、
Motion+ `sections/cta-sections` の magnetic banner に相当する合成例です。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、CTA ボタンは
`type="button"` のまま送信先を持ちません。CTA ボタンに付与している
`data-fandhe-magnetic`（値なし存在属性）は `fandhe-frontend-wasm-full` の
`magnetic` feature（既定 on）が消費する opt-in マーカーであり、実アプリで
JS ハイドレーションが有効な場合、ポインタの移動に追従してボタンが吸い付く
ように動きます（`fandhe-frontend-animation::magnetic::compute_pull`/
`write_offset` が `--fandhe-motion-magnetic-x`/`-y` の 2 個の CSS カスタム
プロパティを計算・書き込み、CSS 側の `transform: translate(var(..))` が
それを消費します）。本 docs サイトは JS ハイドレーションを一切行わないため、
本 Demo ではポインタ追従は発生しません（マークアップと opt-in 属性の使い方
のみを示す静的な実例です）。

## Rust コード

```rust
use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::Size;

/// `cta-banner-magnetic` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    let title = heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl,
            weight: HeadingWeight::Bold,
        },
        vec![("data-blocks-cta-banner-magnetic-title", "")],
        vec![text("Ready to get started?")],
    );
    let description = p(
        vec![("class", "blocks-cta-banner-magnetic-description")],
        vec![text(
            "Join teams already shipping faster with a framework built for AI-era security.",
        )],
    );
    let cta = button::button(
        &ButtonProps {
            size: Size::Lg,
            ..ButtonProps::default()
        },
        vec![
            ("data-blocks-cta-banner-magnetic-cta", ""),
            ("data-fandhe-magnetic", ""),
        ],
        vec![text("Get started")],
    );

    div(
        vec![("data-blocks-cta-banner-magnetic-banner", "")],
        vec![title, description, cta],
    )
}
```
