//! Markdown フェンスコードブロックへコピーボタンの骨格を差し込む（イシュー #3605）。
//!
//! # 役割・呼び出し文脈
//!
//! `crate::build::build_site` が `render_markdown` の直後（blocks / wireframes の
//! 生成節挿入より前）に [`wrap_code_blocks`] を適用する。対象は Markdown 由来の
//! `pre` に限られ、Blocks の demo や部品ページの Anatomy の `pre` は包まない。
//! `crate::markdown::parse_fence` の出力（`pre > code`）は変えない。
//!
//! 出力構造:
//!
//! ```text
//! div.docs-code-block
//!   pre > code           … 既存のまま
//!   button.docs-code-copy[type=button][hidden][aria-label][data-copy-state=idle]
//!   span.docs-code-copy-status[role=status][aria-live=polite]
//! ```
//!
//! ボタンは既定 `hidden` で、可視化・ラベル（Copy / Copied / Failed）の付与は
//! `crate::script::SITE_JS` が配線完了後に行う（テーマトグルと同じ契約）。
//! SSG 時にラベル文字列を出さないのは、検索インデックス
//! （`crate::search_index`）へ全ページ共通の語が混入するのを避けるため。
//! 後続 Phase（ヒーロー・コードブロックヘッダー）もこのラッパー構造を再利用する。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! 属性値はすべて `&'static str` 定数で、入力由来の値を属性へ流さない。
//! `pre` 以下のノードは所有権ごと移すのみで、既定エスケープ経路を変えない。
//! `raw_html()` は使わない。

use fandhe_frontend_core::{button, div, span, Node};

/// コードブロックのラッパー class。
pub const CODE_BLOCK_CLASS: &str = "docs-code-block";
/// コピーボタンの class。
pub const COPY_BUTTON_CLASS: &str = "docs-code-copy";
/// 完了通知用ステータス領域の class。
pub const COPY_STATUS_CLASS: &str = "docs-code-copy-status";
/// コピー状態（`idle` / `copied` / `failed`）を表す属性名。
pub const COPY_STATE_ATTR: &str = "data-copy-state";

/// Node 木を再帰的に辿り、`pre` 要素をコピーボタン付きラッパーで包む。
///
/// `pre` の内側は辿らない（フェンス本文は不変）。`Text` / `RawHtml` は素通し。
pub fn wrap_code_blocks(nodes: Vec<Node>) -> Vec<Node> {
    nodes.into_iter().map(wrap_node).collect()
}

fn wrap_node(node: Node) -> Node {
    match node {
        Node::Element {
            tag: "pre",
            attrs,
            children,
        } => div(
            vec![("class", CODE_BLOCK_CLASS)],
            vec![
                Node::Element {
                    tag: "pre",
                    attrs,
                    children,
                },
                button(
                    vec![
                        ("type", "button"),
                        ("class", COPY_BUTTON_CLASS),
                        ("hidden", ""),
                        ("aria-label", "Copy code"),
                        (COPY_STATE_ATTR, "idle"),
                    ],
                    vec![],
                ),
                span(
                    vec![
                        ("class", COPY_STATUS_CLASS),
                        ("role", "status"),
                        ("aria-live", "polite"),
                    ],
                    vec![],
                ),
            ],
        ),
        Node::Element {
            tag,
            attrs,
            children,
        } => Node::Element {
            tag,
            attrs,
            children: wrap_code_blocks(children),
        },
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::render_markdown;
    use fandhe_frontend_core::render;

    fn render_all(nodes: Vec<Node>) -> String {
        nodes.iter().map(render).collect()
    }

    #[test]
    fn wraps_plain_fence_with_hidden_button() {
        let nodes = render_markdown("```\nhello\n```\n");
        let before = render_all(nodes.clone());
        let html = render_all(wrap_code_blocks(nodes));
        assert!(html.contains(r#"<div class="docs-code-block"><pre>"#));
        assert!(html.contains(r#"class="docs-code-copy" hidden"#));
        assert!(html.contains(r#"class="docs-code-copy-status" role="status""#));
        let pre = &before[before.find("<pre>").unwrap()..before.find("</pre>").unwrap() + 6];
        assert!(html.contains(pre), "pre must be byte-identical");
    }

    #[test]
    fn wraps_highlighted_and_nested_fences() {
        let md = "```rust\nfn main() {}\n```\n\n> ```\n> quoted\n> ```\n";
        let html = render_all(wrap_code_blocks(render_markdown(md)));
        assert_eq!(html.matches(r#"class="docs-code-copy" hidden"#).count(), 2);
        assert_eq!(html.matches(r#"class="docs-code-block""#).count(), 2);
    }

    #[test]
    fn leaves_input_without_pre_unchanged() {
        let nodes = render_markdown("just a paragraph\n");
        assert_eq!(wrap_code_blocks(nodes.clone()), nodes);
    }

    #[test]
    fn keeps_default_escaping_in_fence_body() {
        let html = render_all(wrap_code_blocks(render_markdown(
            "```\n<script>alert(1)</script>\n```\n",
        )));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn button_and_status_have_no_text() {
        let html = render_all(wrap_code_blocks(render_markdown("```\nx\n```\n")));
        assert!(html.contains("data-copy-state=\"idle\"></button>"));
        assert!(html.contains("aria-live=\"polite\"></span>"));
    }
}
