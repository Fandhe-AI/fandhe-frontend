# Highlight Fixture Home

This fixture exists solely to give
`crates/docs-site/tests/search_index.rs` a deterministic Rust fence whose
tokenized output exercises the "keyword/literal adjacent to a non-space
character" regression class (イシュー #1078 レビュー指摘). Real
`docs/**` prose is not a stable substrate for this assertion because its
wording can change independently of the highlighter.

```rust
use crate::highlight;

fn foo(x: i32) -> i32 {
    x + 1
}

let result = foo(1);
```
