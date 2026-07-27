//! `crate::highlight` の統合テスト（イシュー #1078）。
//!
//! `src/highlight.rs` 内 `#[cfg(test)]` の単体テスト（全域性・落とし穴系）を
//! 補完し、(1) `crate::markdown::parse_fence` からの入口（`highlight_children`）
//! の XSS 回帰、(2) CSS 契約（`TokenKind::ALL` を回した `token-*` セレクタの
//! 網羅性・ライト/ダーク両ブロックでの宣言）を検証する。

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::highlight::{highlight_children, Language, TokenKind};
use fandhe_frontend_docs_site::site_theme;

/// サイト骨格 CSS 全量。`tests/site_css_contract.rs::site_css` と同じ組み立て。
fn site_css() -> String {
    site_theme::stylesheet()
        .expect("site theme stylesheet should assemble")
        .as_css()
        .to_string()
}

// ---------------------------------------------------------------------
// フォールバック（受け入れ条件 3）
// ---------------------------------------------------------------------

#[test]
fn unsupported_languages_fall_back_to_none() {
    for lang in ["bash", "css", "json", "javascript", "sh", "yaml", "unknown"] {
        assert!(
            highlight_children("plain content", lang).is_none(),
            "language={lang} should fall back to plain (None)"
        );
    }
}

#[test]
fn empty_lang_token_falls_back_to_none() {
    assert!(highlight_children("plain content", "").is_none());
}

// ---------------------------------------------------------------------
// XSS 回帰（受け入れ条件 1）
// ---------------------------------------------------------------------

/// `highlight_children` が生成したノード列を `<pre><code>…</code></pre>` に
/// 包んで `render()` した HTML を返す（`parse_fence` の実際の組み立てを模す）。
fn render_highlighted(src: &str, lang_token: &str) -> String {
    use fandhe_frontend_core::{code, pre};
    let children = highlight_children(src, lang_token).expect("should tokenize");
    render(&pre(vec![], vec![code(vec![], children)]))
}

#[test]
fn xss_payload_bare_in_rust_source_is_escaped() {
    let out = render_highlighted("</code></pre><script>alert(1)</script>", "rust");
    assert!(!out.contains("<script"));
    // `1` はハイライト対象言語では token-number span で包まれるため、
    // "alert(1)" 全体の一致ではなく、脱出を狙ったタグ断片が個別にエスケープ
    // 済みであることを確認する。
    assert!(out.contains("&lt;script&gt;alert("));
    assert!(out.contains("&lt;/script&gt;"));
}

#[test]
fn xss_payload_in_rust_string_literal_is_escaped() {
    let out = render_highlighted(
        "let s = \"</code></pre><script>alert(1)</script>\";",
        "rust",
    );
    assert!(!out.contains("<script"));
    assert!(out.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
}

#[test]
fn xss_payload_in_rust_comment_is_escaped() {
    let out = render_highlighted("// </code></pre><script>alert(1)</script>", "rust");
    assert!(!out.contains("<script"));
    assert!(out.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
}

#[test]
fn xss_payload_in_html_fence_is_escaped() {
    let out = render_highlighted(
        "<div class=\"</code></pre><script>alert(1)</script>\">x</div>",
        "html",
    );
    assert!(!out.contains("<script"));
    assert!(out.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
}

#[test]
fn xss_payload_in_toml_fence_is_escaped() {
    let out = render_highlighted("key = \"</code></pre><script>alert(1)</script>\"", "toml");
    assert!(!out.contains("<script"));
    assert!(out.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
}

// ---------------------------------------------------------------------
// CSS 契約（受け入れ条件 2）
// ---------------------------------------------------------------------

#[test]
fn every_non_plain_token_kind_has_a_css_rule() {
    let css = site_css();
    for kind in TokenKind::ALL {
        if let Some(class) = kind.class() {
            let selector = format!(".{class}");
            assert!(
                css.contains(&selector),
                "css does not contain a selector for {class} ({kind:?})"
            );
        }
    }
}

#[test]
fn highlight_tokens_declared_in_both_dark_blocks() {
    let css = site_css();
    let media_start = css
        .find("@media (prefers-color-scheme: dark) {")
        .expect("media dark block should exist");
    let data_theme_start = css
        .find(":root[data-theme=\"dark\"] {")
        .expect("data-theme dark block should exist");

    let media_block = &css[media_start..data_theme_start];
    let data_theme_block = &css[data_theme_start..];

    for token in [
        "--fandhe-color-docs-code-keyword",
        "--fandhe-color-docs-code-string",
        "--fandhe-color-docs-code-comment",
        "--fandhe-color-docs-code-number",
        "--fandhe-color-docs-code-tag",
        "--fandhe-color-docs-code-attr",
    ] {
        assert!(
            media_block.contains(token),
            "{token} missing from @media (prefers-color-scheme: dark) block"
        );
        assert!(
            data_theme_block.contains(token),
            "{token} missing from :root[data-theme=\"dark\"] block"
        );
    }
}

// ---------------------------------------------------------------------
// 言語解決（Language::from_token）
// ---------------------------------------------------------------------

#[test]
fn language_from_token_resolves_supported_languages_case_insensitively() {
    assert_eq!(Language::from_token("rust"), Some(Language::Rust));
    assert_eq!(Language::from_token("RUST"), Some(Language::Rust));
    assert_eq!(Language::from_token("toml"), Some(Language::Toml));
    assert_eq!(Language::from_token("html"), Some(Language::Html));
}

#[test]
fn language_from_token_rejects_aliases_and_unknown() {
    // イシュー #1078 スコープ外: `rs` / `htm` エイリアスは意図的に非対応。
    assert_eq!(Language::from_token("rs"), None);
    assert_eq!(Language::from_token("htm"), None);
    assert_eq!(Language::from_token("bash"), None);
}
