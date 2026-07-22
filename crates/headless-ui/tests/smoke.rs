//! `fandhe-frontend-headless-ui` の骨格スモークテスト（イシュー #522）。
//!
//! 本クレートは現時点で公開 API を持たないため、`fandhe-frontend-core` への
//! path 依存が実体を持つこと・本クレート経由でも既定エスケープ（REQ-1）が
//! 効くことを固定する。#553（XSS 回帰テスト整備）の先行アンカーであり、
//! anatomy / data-* / ARIA API 実装（#523）に合わせて拡充する。

use fandhe_frontend_core::{el, render, text};

#[test]
fn default_escape_holds_via_core_dependency() {
    let node = el("div", vec![], vec![text("<script>alert('xss')</script>")]);
    let html = render(&node);

    assert!(
        !html.contains("<script>"),
        "既定エスケープが効いていない: {html}"
    );
    assert!(html.contains("&lt;script&gt;"));
}
