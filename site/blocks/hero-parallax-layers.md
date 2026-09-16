# hero-parallax-layers

`heading` / `text` / `button` の合成例（既存部品のみ）。背景・中景・
前景の 3 レイヤー（抽象図形）が `SlotRecipe::parallax` で異なる速度で
視差移動（対応ブラウザのみ、非対応は静止。JS 不要）。
`prefers-reduced-motion: reduce` で無効化。`data-fandhe-scroll-progress`
は無 JS のため付与しません。`<form>` は持ちません。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};

fn parallax_layer(part: &'static str) -> Node {
    div(
        vec![("data-scope", "blocks-hero-parallax"), ("data-part", part)],
        vec![],
    )
}

pub fn demo() -> Node {
    let content = div(
        vec![
            ("data-scope", "blocks-hero-parallax"),
            ("data-part", "content"),
        ],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("A workspace that moves with you")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    ..TextProps::default()
                },
                vec![],
                vec![text("Layers drift at different speeds as you scroll.")],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("Explore")]),
        ],
    );

    div(
        vec![("class", "blocks-hero-parallax-stage")],
        vec![
            parallax_layer("layer-back"),
            parallax_layer("layer-mid"),
            parallax_layer("layer-front"),
            content,
        ],
    )
}
```
