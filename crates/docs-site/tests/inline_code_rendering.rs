//! 部品ページの生成文でバッククォート囲みがインラインコードとして描画される
//! ことの契約テスト（イシュー #3601）。
//!
//! Themes / Primitives（`component_page::generated_content`）と Wireframes
//! （`wireframes::insert_generated_sections`）の全ページについて、生成された
//! ノード木の中に「閉じたバッククォートの組を含むテキスト」が残っていない
//! ことを検査する。判定関数は `markdown::inline_code_nodes` を流用せず本ファイル
//! 内で独立に実装する（自己参照による空振り防止）。

use std::path::{Path, PathBuf};

use fandhe_frontend_core::{render, Node};
use fandhe_frontend_docs_site::component_page::{
    render_component_page, ArgRow, AriaRow, ComponentPageSpec, ExampleEntry, KeyRow, Layer,
};
use fandhe_frontend_docs_site::nav::parse_nav;
use fandhe_frontend_docs_site::{component_page, wireframes};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo_root should resolve from CARGO_MANIFEST_DIR")
}

/// 長さ n のバッククォート連続の後に同じ長さ n の連続が現れるか。
fn has_closed_backtick_pair(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    let mut runs: Vec<(usize, usize)> = Vec::new(); // (開始, 長さ)
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '`' {
            let st = i;
            while i < chars.len() && chars[i] == '`' {
                i += 1;
            }
            runs.push((st, i - st));
        } else {
            i += 1;
        }
    }
    for (a, (_, la)) in runs.iter().enumerate() {
        if runs[a + 1..].iter().any(|(_, lb)| lb == la) {
            return true;
        }
    }
    false
}

/// コード系要素の外側にある `Node::Text` の違反断片を集める。
fn collect_violations(node: &Node, in_literal: bool, out: &mut Vec<String>) {
    match node {
        Node::Text(t) => {
            if !in_literal && has_closed_backtick_pair(t) {
                out.push(t.chars().take(80).collect());
            }
        }
        Node::RawHtml(_) => {}
        Node::Element { tag, children, .. } => {
            let lit = in_literal
                || matches!(
                    *tag,
                    "code" | "pre" | "kbd" | "samp" | "script" | "style" | "textarea"
                );
            for c in children {
                collect_violations(c, lit, out);
            }
        }
    }
}

#[test]
fn closed_backtick_pair_detector_self_check() {
    assert!(has_closed_backtick_pair("a `b` c"));
    assert!(has_closed_backtick_pair("``a`b``"));
    assert!(!has_closed_backtick_pair("a `b c"));
    assert!(!has_closed_backtick_pair("plain"));
}

#[test]
fn no_closed_backticks_remain_in_generated_component_pages() {
    let input = std::fs::read_to_string(repo_root().join("site/nav.toml"))
        .expect("site/nav.toml should be readable");
    let nav = parse_nav(&input).expect("site/nav.toml should parse");

    let (mut themes, mut prims, mut wires) = (0usize, 0usize, 0usize);
    let mut violations: Vec<String> = Vec::new();
    let mut button_html = String::new();

    for page in nav.all_pages() {
        let path = page.path.as_str();
        let mut found = Vec::new();
        if path.starts_with("/themes/") || path.starts_with("/primitives/") {
            let Some(node) = component_page::generated_content(path) else {
                continue;
            };
            if path.starts_with("/themes/") {
                themes += 1;
            } else {
                prims += 1;
            }
            collect_violations(&node, false, &mut found);
            if path == "/themes/button/" {
                button_html = render(&node);
            }
        } else if path.starts_with("/wireframes/") {
            let nodes = wireframes::insert_generated_sections(path, "", vec![]);
            if nodes.is_empty() {
                continue;
            }
            wires += 1;
            for n in &nodes {
                collect_violations(n, false, &mut found);
            }
        } else {
            continue;
        }
        for f in found {
            violations.push(format!("{path}: {f}"));
        }
    }

    assert!(
        violations.is_empty(),
        "closed backtick pairs remain as plain text:\n{}",
        violations.join("\n")
    );
    // 空振り防止: 各層で十分な数のページを検査している。
    assert!(themes >= 100, "too few themes pages checked: {themes}");
    assert!(prims >= 60, "too few primitives pages checked: {prims}");
    assert_eq!(wires, 49, "wireframes pages checked: {wires}");
    assert!(
        button_html.contains("<code>"),
        "/themes/button/ should contain inline code"
    );
}

/// バッククォートで囲んだ XSS ペイロードが生成文の全入口で無害化される。
#[test]
fn backticked_payloads_are_escaped_in_every_generated_field() {
    const A: &str = "`<script>alert(1)</script>`";
    const B: &str = "`\"><img src=x onerror=alert(1)>`";
    let spec = ComponentPageSpec {
        features: &[A],
        arguments: &[ArgRow {
            name: A,
            kind: B,
            default: A,
            description: B,
        }],
        examples: &[ExampleEntry {
            title: A,
            description: B,
            render: || Node::Text("ex".to_string()),
        }],
        keyboard: &[KeyRow {
            key: A,
            description: B,
        }],
        aria: &[AriaRow {
            attribute: A,
            description: B,
        }],
        demo: None,
    };
    let demo = Node::Text("demo".to_string());
    let page = render_component_page("/themes/widget/", demo, &spec, Layer::Themes);
    let html = render(&page);
    assert!(!html.contains("<script>alert(1)</script>"), "{html}");
    assert!(!html.contains("<img"), "{html}");
    assert!(html.contains("<code>&lt;script&gt;alert(1)&lt;/script&gt;</code>"));
    assert!(html.contains("<code>&quot;&gt;&lt;img src=x onerror=alert(1)&gt;</code>"));
}
