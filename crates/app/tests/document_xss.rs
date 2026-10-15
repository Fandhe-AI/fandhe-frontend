//! `Document` の XSS 回帰テスト（削除・弱体化禁止、`crates/core/tests/xss_escape.rs` と同流儀）。
use fandhe_frontend_app::Document;
use fandhe_frontend_core::{el, text};

const PAYLOADS: &[&str] = &[
    "\"><script>alert(1)</script>",
    "<img src=x onerror=alert(1)>",
    "'&<>\"",
];

fn assert_no_raw(s: &str) {
    assert!(!s.contains("<script>alert"), "{s}");
    assert!(!s.contains("<img src=x"), "{s}");
}

#[test]
fn title_lang_body_head_text_escaped() {
    for p in PAYLOADS {
        let s = Document::new(*p, el("p", vec![], vec![text(*p)]))
            .lang(*p)
            .head_node(el("style", vec![], vec![text(*p)]))
            .render();
        assert_no_raw(&s);
        assert!(s.contains("&lt;") || s.contains("&amp;"), "{s}");
    }
}

#[test]
fn href_src_breakout_is_escaped() {
    let p = "\"><script>alert(1)</script>";
    let s = Document::new("T", text("x"))
        .stylesheet(p)
        .script(p)
        .render();
    assert_no_raw(&s);
    assert!(s.contains("&quot;&gt;&lt;script&gt;"), "{s}");
}

#[test]
fn dangerous_schemes_omit_element() {
    for u in [
        "javascript:alert(1)",
        "JaVaScRiPt:alert(1)",
        "java\tscript:alert(1)",
        "java\nscript:alert(1)",
        "data:text/html,<script>alert(1)</script>",
        "vbscript:msgbox(1)",
    ] {
        let s = Document::new("T", text("x"))
            .stylesheet(u)
            .script_module(u)
            .stylesheet("/ok.css")
            .script("/ok.js")
            .render();
        assert_eq!(s.matches("<link").count(), 1, "{u}: {s}");
        assert_eq!(s.matches("<script").count(), 1, "{u}: {s}");
        assert!(!s.to_lowercase().contains("script:"), "{u}: {s}");
        assert!(!s.contains("data:"), "{u}: {s}");
    }
}

#[test]
fn safe_urls_kept() {
    let s = Document::new("T", text("x"))
        .stylesheet("https://example.com/a.css")
        .script("//cdn.example/x.js")
        .render();
    assert!(s.contains("https://example.com/a.css") && s.contains("//cdn.example/x.js"));
}
