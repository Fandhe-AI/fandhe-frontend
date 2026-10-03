//! 部品ページ API 表の表記規則の契約テスト（イシュー #3621）。
//!
//! `fandhe_frontend_docs_site::component_page` が出す Arguments / Data Attributes /
//! CSS Variables 表の (1) コード化、(2) 空値プレースホルダ、(3) 縦積み用
//! `data-label`、(4) `site_theme` の縦積み CSS が Themes / Primitives 両方の
//! スタイルシートに載ること、を固定する。セルは常に `text()` 経由で
//! エスケープされる前提（REQ-1）も併せて確認する。

use fandhe_frontend_core::{div, el, p, render, text};
use fandhe_frontend_docs_site::component_page::{
    render_component_page, ArgRow, ComponentPageSpec, Layer,
};
use fandhe_frontend_docs_site::site_theme;

fn spec() -> ComponentPageSpec {
    ComponentPageSpec {
        features: &[],
        arguments: &[
            ArgRow {
                name: "disabled",
                kind: "bool",
                default: "false",
                description: "無効化する。",
            },
            ArgRow {
                name: "label",
                kind: "Option<Node>",
                default: "",
                description: "ラベル。",
            },
            ArgRow {
                name: "id",
                kind: "&str",
                default: "(必須)",
                description: "識別子。",
            },
        ],
        examples: &[],
        keyboard: &[],
        aria: &[],
        demo: None,
    }
}

fn demo() -> fandhe_frontend_core::Node {
    div(
        vec![("class", "pre-styled-showcase")],
        vec![el(
            "section",
            vec![],
            vec![
                el("h2", vec![], vec![text("Accordion")]),
                p(vec![], vec![text("説明文")]),
                el(
                    "div",
                    vec![
                        ("data-scope", "accordion"),
                        ("data-part", "root"),
                        ("data-disabled", ""),
                    ],
                    vec![],
                ),
                el(
                    "div",
                    vec![
                        ("data-scope", "accordion"),
                        ("data-part", "root"),
                        ("data-state", "open"),
                    ],
                    vec![],
                ),
            ],
        )],
    )
}

#[test]
fn api_tables_use_code_placeholders_and_data_labels() {
    let html = render(&render_component_page(
        "/themes/accordion/",
        demo(),
        &spec(),
        Layer::Themes,
    ));
    assert!(
        html.contains(r#"<table data-docs-api-table="arguments">"#),
        "{html}"
    );
    assert!(html.contains(r#"<th scope="row" data-label="Name"><code>disabled</code></th>"#));
    assert!(html.contains("<code>Option&lt;Node&gt;</code>"));
    assert!(html.contains("<code>false</code>"));
    assert!(html.contains("data-docs-api-placeholder"));
    assert!(html.contains("(必須)") && !html.contains("<code>(必須)</code>"));
    assert!(html.contains(r#"data-docs-api-table="data-attributes""#));
    assert!(html.contains("（値なし）"));
    assert!(html.contains("<code>open</code>"));
    assert!(!html.contains(">, "), "先頭カンマを出さない: {html}");
    assert!(html.contains(r#"data-docs-api-table="css-variables""#));
    // 本文セル（td と行見出し th）はすべて data-label を持つ（縦積み表示の前提）。
    assert_eq!(
        html.matches("<td").count() + html.matches(r#"<th scope="row""#).count(),
        html.matches("data-label=").count(),
        "{html}"
    );
}

#[test]
fn api_table_css_is_present_in_both_stylesheets_with_stacked_narrow_rules() {
    for css in [
        site_theme::stylesheet()
            .expect("stylesheet")
            .as_css()
            .to_string(),
        site_theme::stylesheet_without_recipes()
            .expect("stylesheet_without_recipes")
            .as_css()
            .to_string(),
    ] {
        let at = css
            .find(".docs-content table[data-docs-api-table] thead")
            .expect("API 表の縦積み規則が無い");
        let tail = &css[at..];
        let thead_rule = &tail[..tail.find('}').expect("thead 規則の終端")];
        assert!(
            !thead_rule.contains("display: none"),
            "thead は display: none にしない（列見出しを支援技術へ残す）"
        );
        assert!(
            thead_rule.contains("clip: rect(0, 0, 0, 0)"),
            "thead は visually-hidden で隠す"
        );
        assert!(tail.contains("content: attr(data-label)"));
    }
}
