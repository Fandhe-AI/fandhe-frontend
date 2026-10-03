//! Markdown フェンスコードブロックへコピーボタンの骨格を差し込む（イシュー #3605）。
//!
//! # 役割・呼び出し文脈
//!
//! `crate::build::build_site` が `render_markdown` の直後（blocks / wireframes の
//! 生成節挿入より前）に [`wrap_code_blocks`] を適用する。対象は Markdown 由来の
//! `pre` に限られ、Blocks の demo や部品ページの Anatomy の `pre` は包まない。
//! `crate::markdown::parse_fence` の出力（`pre > code`）は変えない。
//!
//! 出力構造（イシュー #3620 でヘッダー帯を追加し、ボタンをその中へ移した）:
//!
//! ```text
//! div.docs-code-block
//!   div.docs-code-header
//!     span.docs-code-lang    … 言語ラベル（allow-list の静的文字列。無指定・未知なら出さない）
//!     button.docs-code-copy[type=button][hidden][aria-label][data-copy-state=idle]
//!   pre > code           … 既存のまま
//!   span.docs-code-copy-status[role=status][aria-live=polite]
//! ```
//!
//! ラベルは `code` の `language-<token>` class を [`language_label`] の静的対応表で引くだけで、
//! 入力由来の文字列は HTML へ出さない。ヘッダー内の語は検索インデックスから除外される
//! （`crate::search_index` が [`CODE_HEADER_CLASS`] の部分木を落とす）。
//! 部品ページ Examples の `pre` は [`wrap_code_blocks_outside_scopes`] で包む。
//!
//! ボタンは既定 `hidden` で、可視化・ラベル（Copy / Copied）の付与と、失敗時の再 hidden 化は
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

use fandhe_frontend_core::{button, code, div, pre, span, text, Node};

/// コードブロックヘッダー帯の class。検索インデックスの除外判定にも使う。
pub const CODE_HEADER_CLASS: &str = "docs-code-header";
/// 言語ラベルの class。
pub const CODE_LANG_CLASS: &str = "docs-code-lang";

/// コードブロックのラッパー class。
pub const CODE_BLOCK_CLASS: &str = "docs-code-block";
/// コピーボタンの class。
pub const COPY_BUTTON_CLASS: &str = "docs-code-copy";
/// 完了通知用ステータス領域の class。
pub const COPY_STATUS_CLASS: &str = "docs-code-copy-status";
/// コピー状態（`idle` / `copied` / `failed`）を表す属性名。
pub const COPY_STATE_ATTR: &str = "data-copy-state";

/// フェンス言語トークン（`language-` 接頭辞の後ろ）から表示名を引く静的対応表。
///
/// 戻り値は `&'static str` のみで、入力由来の文字列を HTML へ流さない（REQ-1）。
/// 未知の言語は `None`（ラベルなし）。ハイライト対応（`crate::highlight`）とは独立。
pub fn language_label(token: &str) -> Option<&'static str> {
    Some(match token.to_ascii_lowercase().as_str() {
        "rust" => "Rust",
        "toml" => "TOML",
        "bash" => "Bash",
        "sh" | "shell" => "Shell",
        "html" => "HTML",
        "css" => "CSS",
        "js" | "javascript" => "JavaScript",
        "json" => "JSON",
        "jsonc" => "JSONC",
        "text" => "Text",
        _ => return None,
    })
}

/// `pre` の子 `code` の `class="language-<token>"` からラベルを取り出す。
fn fence_label(children: &[Node]) -> Option<&'static str> {
    children.iter().find_map(|c| match c {
        Node::Element {
            tag: "code", attrs, ..
        } => attrs
            .iter()
            .filter(|(k, _)| k == "class")
            .flat_map(|(_, v)| v.split_whitespace())
            .find_map(|t| t.strip_prefix("language-").and_then(language_label)),
        _ => None,
    })
}

/// 部品ページ Examples 用の CSS スニペットブロック（`pre > code.language-css`）。
///
/// 本文は `text()` を通る（既定エスケープ）。ラベル `CSS` は [`wrap_code_blocks_outside_scopes`] が付ける。
pub fn css_snippet_block(src: &'static str) -> Node {
    pre(
        vec![],
        vec![code(vec![("class", "language-css")], vec![text(src)])],
    )
}

/// Node 木を再帰的に辿り、`pre` 要素をヘッダー（言語ラベル + コピーボタン）付きラッパーで包む。
///
/// `pre` の内側は辿らない（フェンス本文は不変）。`Text` / `RawHtml` は素通し。
pub fn wrap_code_blocks(nodes: Vec<Node>) -> Vec<Node> {
    nodes.into_iter().map(|n| wrap_node(n, true)).collect()
}

/// [`wrap_code_blocks`] と同じだが、`data-scope` を持つ要素（部品デモ本体・anatomy）の
/// 部分木へは降りない。部品ページ Examples 用（#3620）。
pub fn wrap_code_blocks_outside_scopes(nodes: Vec<Node>) -> Vec<Node> {
    nodes.into_iter().map(|n| wrap_node(n, false)).collect()
}

/// `pre` 1 個をヘッダー（言語ラベル + コピーボタン）付きラッパー（`div.docs-code-block`）で包む。
///
/// [`wrap_code_blocks`] の各 `pre` と、Markdown を経由せず Rust 側で組む
/// ヒーローのインストールコマンド（`crate::landing`、イシュー #3612）が同じ
/// 骨格を共有するための単一実装点。`pre` の中身は所有権ごと移すのみ。
#[must_use]
pub fn copy_block(pre: Node) -> Node {
    let mut header = Vec::new();
    if let Node::Element { children, .. } = &pre {
        if let Some(label) = fence_label(children) {
            header.push(span(vec![("class", CODE_LANG_CLASS)], vec![text(label)]));
        }
    }
    header.push(button(
        vec![
            ("type", "button"),
            ("class", COPY_BUTTON_CLASS),
            ("hidden", ""),
            ("aria-label", "Copy code"),
            (COPY_STATE_ATTR, "idle"),
        ],
        vec![],
    ));
    div(
        vec![("class", CODE_BLOCK_CLASS)],
        vec![
            div(vec![("class", CODE_HEADER_CLASS)], header),
            pre,
            span(
                vec![
                    ("class", COPY_STATUS_CLASS),
                    ("role", "status"),
                    ("aria-live", "polite"),
                ],
                vec![],
            ),
        ],
    )
}

fn wrap_node(node: Node, enter_scopes: bool) -> Node {
    match node {
        Node::Element {
            tag: "pre",
            attrs,
            children,
        } => copy_block(Node::Element {
            tag: "pre",
            attrs,
            children,
        }),
        Node::Element {
            tag,
            attrs,
            children,
        } => {
            if !enter_scopes && attrs.iter().any(|(k, _)| k == "data-scope") {
                return Node::Element {
                    tag,
                    attrs,
                    children,
                };
            }
            Node::Element {
                tag,
                attrs,
                children: children
                    .into_iter()
                    .map(|n| wrap_node(n, enter_scopes))
                    .collect(),
            }
        }
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
        assert!(
            html.contains(r#"<div class="docs-code-block"><div class="docs-code-header"><button"#)
        );
        assert!(!html.contains("docs-code-lang"));
        assert!(html.contains("</div><pre>"));
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

    fn labels(md: &str) -> String {
        render_all(wrap_code_blocks(render_markdown(md)))
    }

    #[test]
    fn labels_known_languages() {
        for (tok, want) in [
            ("rust", "Rust"),
            ("toml", "TOML"),
            ("bash", "Bash"),
            ("sh", "Shell"),
            ("html", "HTML"),
            ("css", "CSS"),
            ("js", "JavaScript"),
            ("jsonc", "JSONC"),
            ("text", "Text"),
            ("RUST", "Rust"),
        ] {
            let html = labels(&format!("```{tok}\nx\n```\n"));
            assert!(
                html.contains(&format!(r#"<span class="docs-code-lang">{want}</span>"#)),
                "{tok}: {html}"
            );
        }
    }

    #[test]
    fn unknown_language_has_no_label() {
        assert!(!labels("```mermaid\nx\n```\n").contains("docs-code-lang"));
        assert!(!labels("```\nx\n```\n").contains("docs-code-lang"));
    }

    #[test]
    fn button_is_inside_header_and_pre_unchanged_when_highlighted() {
        let nodes = render_markdown("```rust\nfn main() {}\n```\n");
        let before = render_all(nodes.clone());
        let html = render_all(wrap_code_blocks(nodes));
        let pre = &before[before.find("<pre>").unwrap()..before.find("</pre>").unwrap() + 6];
        assert!(html.contains(pre));
        let h = html.find("docs-code-header").unwrap();
        let b = html.find("docs-code-copy\"").unwrap();
        let p = html.find("<pre>").unwrap();
        assert!(h < b && b < p);
    }

    #[test]
    fn header_does_not_unescape_body() {
        let html = labels("```rust\n<script>alert(1)</script>\n```\n");
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn outside_scopes_skips_data_scope_subtrees() {
        let inner = css_snippet_block("a{}");
        let scoped = div(vec![("data-scope", "x")], vec![inner.clone()]);
        let out = wrap_code_blocks_outside_scopes(vec![scoped.clone(), inner]);
        assert_eq!(out[0], scoped);
        let html = render_all(out);
        assert_eq!(html.matches("docs-code-block").count(), 1);
        assert!(html.contains(r#"<span class="docs-code-lang">CSS</span>"#));
    }
}
