# text-split-reveal

`heading` / `text` / `button` の合成例（既存部品のみ、新規部品なし）。
見出しは文字単位、リード文は単語単位で分割し順にフェードイン
（`text_reveal::chars`/`words`、SSR のみで JS 不要）。`aria-hidden`
分割レイヤー + 隠しの完全テキストで読み上げは分断されません。
`prefers-reduced-motion: reduce` では最終状態のまま表示。`<form>` は
持ちません。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::text::{self, TextProps};
use fandhe_frontend_pre_styled_ui::text_reveal;

pub fn demo() -> Node {
    let title = heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl4,
            ..HeadingProps::default()
        },
        vec![],
        vec![text_reveal::chars("Built for clarity")],
    );

    let lead = div(
        vec![("class", "blocks-text-split-reveal-lead")],
        vec![text::text(
            &TextProps::default(),
            vec![],
            vec![text_reveal::words("Words fade in one by one, in order.")],
        )],
    );

    let cta = button::button(&ButtonProps::default(), vec![], vec![text("Try it out")]);

    div(
        vec![("class", "blocks-text-split-reveal-inner")],
        vec![title, lead, cta],
    )
}
```
