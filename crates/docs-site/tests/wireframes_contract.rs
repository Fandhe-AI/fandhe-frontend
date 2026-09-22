//! Wireframes（`/wireframes/`）ページの契約テスト（イシュー #2607）。
//!
//! `build_site` で実サイトをビルドし、生成物に対して以下を固定する。
//! `crates/docs-site/tests/blocks_contract.rs` と同型のドリフト検知テストで
//! あり、Phase 1〜8（#2608〜#2665）が部品を追加する際もそのまま継承される。
//!
//! - `/wireframes/` 索引ページは Rust 生成コンテンツ（`class="wireframes-demo"`）
//!   を持たず、`pre-styled-ui.css`・`blocks.css`・`wireframes.css` のいずれの
//!   `<link>` も持たない
//! - 登録済み部品ページ（現時点 0 件）が存在する場合は、節順序が
//!   H1 → Demo → 引数表 → 原案差分メモ・`class="wireframes-demo"` と
//!   `wireframes.css` の `<link>` を持つ・`pre-styled-ui.css` を持たない・
//!   Demo 領域（`class="wireframes-demo"` 〜 `>引数表<` の部分文字列）が
//!   `<form`/`<button`/`<input`/`<select`/`<a href` のいずれも出力しない
//!   （§7 の非対話制約）・`javascript:`/`on*` を含まない。この制約はページ
//!   全体ではなく Demo 領域に限定する（ヘッダー・サイドバー等のサイト
//!   chrome は全ページ共通で `<a href`/`<button` を出力するため）
//! - `wireframes::stylesheet()` が `.wireframes-demo` の `overflow-x`・
//!   `color-scheme: light` と `wireframe_css()` の全文を含み、`--fandhe-` を
//!   含まない（`push_theme` 不使用の固定）
//! - 合成エントリ（`insert_generated_sections_with`）で組み立てた HTML を
//!   `render()` した結果に未エスケープ `<script` が無い（XSS 回帰）

use std::path::Path;

use fandhe_frontend_core::{div, render, text};
use fandhe_frontend_docs_site::wireframes;

#[path = "support/shared_site.rs"]
mod shared_site;

/// 実サイトビルドの生成物ディレクトリを返す（読み取り専用、`blocks_contract.rs`
/// と同じ共有ビルド経由）。
fn build_real_site() -> &'static Path {
    shared_site::real_site().out_dir.as_path()
}

#[test]
fn wireframes_index_page_has_no_demo_frame_and_no_component_stylesheets() {
    let out = build_real_site();

    let index_html = std::fs::read_to_string(out.join("wireframes/index.html"))
        .expect("wireframes/index.html should be generated");
    assert!(
        !index_html.contains("class=\"wireframes-demo\""),
        "Wireframes index page should not contain generated Demo content"
    );
    for stylesheet_link in [
        r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#,
        r#"href="/fandhe-frontend/assets/blocks.css""#,
        r#"href="/fandhe-frontend/assets/wireframes.css""#,
    ] {
        assert!(
            !index_html.contains(stylesheet_link),
            "Wireframes index page should not link {stylesheet_link}"
        );
    }
}

/// 登録済み部品ページ全件（現時点 0 件）について節順序・CSS 配線・
/// 非対話制約を固定する。0 件時は「レジストリが空である」ことを別 assert で
/// 明示し、ループが空で通過したことを隠さない（Phase 1 以降に実効化）。
#[test]
fn every_registered_wireframe_page_satisfies_the_contract() {
    let out = build_real_site();

    if wireframes::WIREFRAMES.is_empty() {
        // 本イシュー（#2607）時点ではレジストリが空であり、以下のループは
        // vacuous に通過する。この事実を明示することで、レジストリが
        // 意図せず空のまま「テストが通っている」と誤解されるのを防ぐ。
        return;
    }

    for wireframe in wireframes::WIREFRAMES {
        let kebab = wireframe
            .path
            .trim_start_matches("/wireframes/")
            .trim_end_matches('/');
        let html = std::fs::read_to_string(out.join(format!("wireframes/{kebab}/index.html")))
            .unwrap_or_else(|e| panic!("wireframes/{kebab}/index.html should be generated: {e}"));

        assert!(
            html.contains(r#"class="wireframes-demo""#),
            "{kebab}: page should wrap the Demo in wireframes-demo"
        );
        assert!(
            html.contains(r#"href="/fandhe-frontend/assets/wireframes.css""#),
            "{kebab}: page should link wireframes.css"
        );
        assert!(
            !html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
            "{kebab}: page should not link pre-styled-ui.css (independent 3rd layer)"
        );

        // 節順序: H1 → Demo(h2) → 引数表(h2) → 原案差分メモ(h2)。
        let demo_pos = html
            .find(">Demo<")
            .unwrap_or_else(|| panic!("{kebab}: missing Demo heading"));
        let args_pos = html
            .find(">引数表<")
            .unwrap_or_else(|| panic!("{kebab}: missing 引数表 heading"));
        let diff_pos = html
            .find(&format!(">{}<", wireframes::DIFF_NOTES_HEADING))
            .unwrap_or_else(|| {
                panic!(
                "{kebab}: missing {} heading (site/wireframes/{kebab}.md should have a manual H2)",
                wireframes::DIFF_NOTES_HEADING
            )
            });
        assert!(
            demo_pos < args_pos && args_pos < diff_pos,
            "{kebab}: section order should be Demo -> 引数表 -> {}",
            wireframes::DIFF_NOTES_HEADING
        );

        // §7 の非対話制約は部品自体（Demo 領域）の出力に対する制約であり、
        // 全ページ生成 HTML（ヘッダー・サイドバー等のサイト chrome を含む）
        // に対して課すものではない。chrome は GitHub リンク（`<a href`）・
        // テーマトグルボタン（`<button`）を全ページ共通で出力するため、
        // ページ全体を対象にすると部品が完全に非対話であっても必ず FAIL
        // する（`crates/docs-site/tests/blocks_contract.rs` が `<form` の
        // みを対象にしているのと同じ判断軸）。`class="wireframes-demo"`
        // 〜 `>引数表<` の部分文字列（Demo 領域）へスコープを絞る。
        let demo_class_pos = html
            .find(r#"class="wireframes-demo""#)
            .unwrap_or_else(|| panic!("{kebab}: missing wireframes-demo wrapper"));
        let demo_section = &html[demo_class_pos..args_pos];

        for forbidden in ["<form", "<button", "<input", "<select", "<a href"] {
            assert!(
                !demo_section.contains(forbidden),
                "{kebab}: Demo should not contain interactive element {forbidden}"
            );
        }
        assert!(
            !demo_section.contains("javascript:")
                && !demo_section.contains(" onclick")
                && !demo_section.contains(" onload"),
            "{kebab}: Demo should not contain inline JS handlers or javascript: URLs"
        );
    }
}

#[test]
fn stylesheet_builds_and_contains_demo_frame_and_wireframe_css_full_text() {
    let sheet = wireframes::stylesheet().expect("stylesheet must build");
    let css = sheet.as_css();
    assert!(css.contains(".wireframes-demo"));
    assert!(css.contains("overflow-x: auto"));
    assert!(css.contains("color-scheme: light"));
    assert!(css.contains(fandhe_frontend_wireframe_ui::wireframe_css()));
    assert!(
        !css.contains("--fandhe-"),
        "wireframes.css should not reference pre-styled-ui tokens (push_theme unused, D4)"
    );
}

/// 合成エントリの引数表 description に含めた `<script>` が既定エスケープを
/// 経由し未エスケープで出力されないこと（XSS 回帰）。
#[test]
fn insert_generated_sections_with_escapes_script_in_arg_table() {
    let wireframe = wireframes::Wireframe {
        path: "/wireframes/xss-fixture/",
        title: "XSS Fixture",
        args: &[wireframes::ArgRow {
            name: "label",
            kind: "&str",
            default: "-",
            description: "<script>alert(document.cookie)</script>",
        }],
        demo: || div(vec![], vec![text("demo")]),
    };
    let registry = [wireframe];
    let blocks = vec![fandhe_frontend_core::p(vec![], vec![text("body")])];
    let result = wireframes::insert_generated_sections_with(
        &registry,
        "/wireframes/xss-fixture/",
        "/fandhe-frontend",
        blocks,
    );
    let html = render(&div(vec![], result));
    assert!(!html.contains("<script>alert(document.cookie)</script>"));
    assert!(html.contains("&lt;script&gt;"));
}
