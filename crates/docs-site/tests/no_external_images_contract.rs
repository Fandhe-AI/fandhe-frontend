//! 生成サイト全 HTML の `img[src]` / `img[srcset]` が外部オリジンを指さないことを固定する契約テスト
//! （イシュー #3660）。
//!
//! 役割: docs サイトの閲覧時に `example.com` 等へのリクエストが飛ぶ（IP・Referer の外部漏えい、
//! オフライン時の読み込み失敗、全ページ走査での 404）経路の再混入を防ぐ。デモ・spec 例の画像は
//! `crate::showcase::IMAGE_DEMO_SRC`（同一オリジンの生成アセット）を使う。
//!
//! 走査範囲: `shared_site::real_site()` の出力ディレクトリ配下の全 `*.html` の `<img>` タグ。
//! `<source>` 等の他要素は対象外。読み取り専用の検査で描画経路は変更しない。

use std::path::{Path, PathBuf};

#[path = "support/shared_site.rs"]
mod shared_site;

/// 外部オリジン参照（`http://` / `https://` / scheme-relative `//`）か判定する。
fn is_external(url: &str) -> bool {
    let u = url.trim().to_ascii_lowercase();
    u.starts_with("http://") || u.starts_with("https://") || u.starts_with("//")
}

/// タグ文字列から属性値（二重引用符）を取り出す。
fn attr_value<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!(" {name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')? + start;
    Some(&tag[start..end])
}

/// HTML 中の全 `<img ...>` タグを返す。
fn img_tags(html: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(i) = rest.find("<img") {
        let tail = &rest[i..];
        let Some(end) = tail.find('>') else { break };
        out.push(&tail[..=end]);
        rest = &tail[end + 1..];
    }
    out
}

/// 外部オリジンを指す `src` / `srcset` 値を返す。
fn external_refs(html: &str) -> Vec<String> {
    let mut v = Vec::new();
    for tag in img_tags(html) {
        if let Some(s) = attr_value(tag, "src") {
            if is_external(s) {
                v.push(s.to_string());
            }
        }
        if let Some(set) = attr_value(tag, "srcset") {
            for cand in set.split(',') {
                let url = cand.split_whitespace().next().unwrap_or("");
                if is_external(url) {
                    v.push(url.to_string());
                }
            }
        }
    }
    v
}

fn collect_html(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).expect("read_dir") {
        let p = e.expect("entry").path();
        if p.is_dir() {
            collect_html(&p, out);
        } else if p.extension().is_some_and(|x| x == "html") {
            out.push(p);
        }
    }
}

#[test]
fn detector_flags_external_and_allows_same_origin() {
    let bad = r#"<img src="https://example.com/a.png" alt="x"><img src="//cdn.example/b.png">"#;
    assert_eq!(external_refs(bad).len(), 2);
    let srcset = r#"<img src="../a.svg" srcset="HTTP://e.com/a.png 2x">"#;
    assert_eq!(external_refs(srcset).len(), 1);
    assert!(external_refs(r#"<img src="../../assets/image-demo.svg" alt="x">"#).is_empty());
}

#[test]
fn no_generated_page_references_external_images() {
    let root = shared_site::real_site().out_dir.as_path();
    let mut files = Vec::new();
    collect_html(root, &mut files);
    assert!(!files.is_empty(), "no html generated");
    let mut violations = Vec::new();
    for f in &files {
        let html = std::fs::read_to_string(f).expect("read html");
        for r in external_refs(&html) {
            violations.push(format!("{}: {r}", f.strip_prefix(root).unwrap().display()));
        }
    }
    assert!(
        violations.is_empty(),
        "external img refs:\n{}",
        violations.join("\n")
    );
}

#[test]
fn avatar_and_image_cropper_pages_render_imgs() {
    let root = shared_site::real_site().out_dir.as_path();
    for page in [
        "primitives/avatar/index.html",
        "primitives/image-cropper/index.html",
    ] {
        let html = std::fs::read_to_string(root.join(page)).expect(page);
        assert!(!img_tags(&html).is_empty(), "{page}: no <img> found");
        assert!(external_refs(&html).is_empty(), "{page}: external img");
    }
}
