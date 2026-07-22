//! `fandhe-frontend-pre-styled-ui` の骨格スモークテスト（イシュー #546）。
//!
//! 本クレートは現時点で公開 API を持たないため、`fandhe-frontend-headless-ui` への
//! path 依存が実体を持つこと・`fandhe-frontend-core`（dev-dependency）経由でも
//! 既定エスケープ（REQ-1）が効くことを固定する。#553（XSS 回帰テスト整備）の
//! 先行アンカーであり、テーマ・variant・styled 部品 API 実装（#547/#548/#550/#551）に
//! 合わせて拡充する。

use fandhe_frontend_core::{el, render, text};

// headless-ui への path 依存が実体を持つことの固定（linkage の存在確認）。
// 本クレートは現時点で headless-ui の公開 API を呼び出さないため `as _` で束縛のみ行う。
use fandhe_frontend_headless_ui as _;

#[test]
fn default_escape_holds_via_core_dev_dependency() {
    let node = el("div", vec![], vec![text("<script>alert('xss')</script>")]);
    let html = render(&node);

    assert!(
        !html.contains("<script>"),
        "既定エスケープが効いていない: {html}"
    );
    assert!(html.contains("&lt;script&gt;"));
}
