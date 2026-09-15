# hero-terminal

`code` / `kbd` の合成例（既存部品のみ）。架空コマンド 3 行が上から順に
遅延フェードイン（JS 不要）。最終行の `text_reveal::typewriter` は
opt-in マーカーのみで、実際の文字送りは `wasm-full` が JS
ハイドレーション後に担うため無 JS の本サイトでは静的表示です。
`prefers-reduced-motion: reduce` で消えます。`<form>` は持ちません。

## Rust コード

```rust
use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::kbd::{self, KbdProps};
use fandhe_frontend_pre_styled_ui::recipe::stagger_index_style;
use fandhe_frontend_pre_styled_ui::text_reveal;

const LINE_ATTR: &str = "data-blocks-hero-terminal-line";

fn prompt() -> Node {
    span(
        vec![("class", "blocks-hero-terminal-prompt")],
        vec![text("$ ")],
    )
}

fn command_line(index: usize, command: &str) -> Node {
    let style = stagger_index_style(index);
    div(
        vec![(LINE_ATTR, ""), ("style", style.as_str())],
        vec![
            prompt(),
            code::code(&CodeProps::default(), vec![], vec![text(command)]),
        ],
    )
}

pub fn demo() -> Node {
    let dot = || span(vec![("class", "blocks-hero-terminal-dot")], vec![]);
    let titlebar = div(
        vec![("class", "blocks-hero-terminal-titlebar")],
        vec![dot(), dot(), dot()],
    );

    let mut lines = vec![
        command_line(0, "fw new my-app"),
        command_line(1, "cd my-app"),
        command_line(2, "cargo run"),
    ];
    let typewriter_style = stagger_index_style(3);
    lines.push(div(
        vec![(LINE_ATTR, ""), ("style", typewriter_style.as_str())],
        vec![
            prompt(),
            text_reveal::typewriter("Ready in 42ms", None),
            kbd::kbd(
                &KbdProps::default(),
                vec![("class", "blocks-hero-terminal-hint")],
                vec![text("⌘ K")],
            ),
        ],
    ));

    div(
        vec![("data-blocks-hero-terminal-panel", "")],
        vec![
            titlebar,
            div(vec![("class", "blocks-hero-terminal-body")], lines),
        ],
    )
}
```
