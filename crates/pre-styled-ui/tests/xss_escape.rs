//! `fandhe-frontend-pre-styled-ui` の XSS 回帰テスト（イシュー #553、REQ-1）。
//!
//! # 本ファイルのスコープ（フォールバック方針）
//!
//! 実装時点（イシュー #553）で本クレートの `src/lib.rs` は公開 API を持たない
//! （テーマトークン・styled 部品実装はイシュー #547/#548/#550/#551 が並行スコープ、
//! `crates/pre-styled-ui/src/lib.rs` 冒頭コメント参照）。そのため本ファイルは
//! `crates/headless-ui/tests/helpers_escape.rs`・`crates/headless-ui/tests/xss_escape.rs`
//! と同型のテキスト・属性値・URL 属性の 3 経路を、本クレートが実際に依存する
//! 経路（`[dependencies]` の `fandhe-frontend-headless-ui`・
//! `[dev-dependencies]` の `fandhe-frontend-core`、`crates/pre-styled-ui/Cargo.toml`
//! 参照）を通して固定する。styled 部品（Button/Badge/Card/Alert 等）経由の
//! 回帰テスト拡充は #550/#551 側のスコープであり、本イシューでは追跡提案に
//! 留める（`.claude/rules/out-of-scope-tracking.md`）。
//!
//! 既存の `tests/smoke.rs`（headless-ui への path 依存の実体確認 + core
//! dev-dependency 経由の既定エスケープ 1 件）は削除・変更しない。本ファイルは
//! それを 3 経路（テキスト・属性値・URL 属性）へ拡充する独立ファイルである。
//!
//! # 削除・弱体化の禁止
//!
//! `.claude/rules/coding-rust.md` の規約により、本ファイルの XSS 回帰
//! テストは以後の削除・弱体化・`#[ignore]` 化を禁止する。

use fandhe_frontend_core::{escape_html, render, text};
use fandhe_frontend_headless_ui::{aria_label, avatar, dialog, ImageStatus};

/// OWASP XSS Prevention Cheat Sheet Rule #1 系の共有ペイロード集合。
///
/// `crates/headless-ui/tests/xss_escape.rs::payloads` と観点を揃えるが、
/// クレート境界をまたいで共有しない既存方針に従い本ファイル内で独立に
/// 定義する。
mod payloads {
    /// タグ注入。
    pub const SCRIPT_TAG: &str = "<script>alert('xss')</script>";
    /// 二重引用符属性値からの breakout。
    pub const DOUBLE_QUOTE_BREAKOUT: &str = "\"><script>alert(1)</script>";
    /// 単一引用符属性値からの breakout（イベントハンドラ注入込み）。
    pub const SINGLE_QUOTE_BREAKOUT: &str = "' onmouseover='alert(1)";
    /// 非 ASCII 混在文字列（マルチバイト透過の確認）。
    pub const NON_ASCII_MIXED: &str = "こんにちは<script>alert(1)</script>世界";

    /// 全ペイロードをまとめて返す（網羅的にループ検証する用途）。
    pub fn all() -> Vec<&'static str> {
        vec![
            SCRIPT_TAG,
            DOUBLE_QUOTE_BREAKOUT,
            SINGLE_QUOTE_BREAKOUT,
            NON_ASCII_MIXED,
        ]
    }
}

/// (1) テキスト経路 + (2) 属性値経路の共通アサーション
/// （`crates/headless-ui/tests/xss_escape.rs::assert_payload_is_escaped` と同型）。
fn assert_payload_is_escaped(payload: &str, html: &str, context_label: &str) {
    let expected_escaped = escape_html(payload);
    assert!(
        html.contains(&expected_escaped),
        "{context_label}で期待されるエスケープ済み表現が出力に見当たらない: \
         payload={payload:?}, expected_escaped={expected_escaped:?}, html={html}"
    );
    assert!(
        !html.contains(payload),
        "{context_label}で生ペイロードが出力にそのまま残っている: payload={payload:?}, html={html}"
    );
    assert!(
        !html.contains("<script>"),
        "{context_label}で実タグとしての <script> が出力に出現している: html={html}"
    );
}

/// (1) テキスト経路: `fandhe_frontend_core::text`（本クレートの
/// `[dev-dependencies]` 経由）+ headless-ui の `dialog::title` children へ
/// 全ペイロードを注入し、既定エスケープが本クレートの依存グラフ末端まで
/// 貫通することを固定する。
#[test]
fn text_children_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let node = dialog::title(None, vec![], vec![text(payload)]);
        let html = render(&node);
        assert_payload_is_escaped(payload, &html, "headless-ui 経由のテキストコンテキスト");
    }
}

/// (2) 属性値経路: headless-ui の `aria_label` ヘルパ + 呼び出し側 attrs へ
/// 属性 breakout 系ペイロードを注入し、エスケープが貫通することを固定する。
#[test]
fn attribute_value_paths_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let aria_label_node = dialog::title(None, vec![aria_label(payload)], vec![]);
        let html = render(&aria_label_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "headless-ui 経由の aria_label 属性値コンテキスト",
        );

        let caller_attrs_node = dialog::title(None, vec![("data-testid", payload)], vec![]);
        let html = render(&caller_attrs_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "headless-ui 経由の呼び出し側 attrs 属性値コンテキスト",
        );
    }
}

/// (3) URL 属性経路: headless-ui の `avatar::image` の `src` へ危険 URL
/// スキームを渡し、core の許可リスト方式（deny by default）が本クレートの
/// 依存経路経由でも貫通することを固定する（拒否時に属性ごとスキップされる
/// 契約は `crates/core/src/lib.rs::render_into` 参照）。
#[test]
fn avatar_image_src_rejects_dangerous_url_schemes_via_headless_ui() {
    let dangerous_urls = [
        "javascript:alert(1)",
        "JaVaScRiPt:alert(1)",
        "data:text/html;base64,PHNjcmlwdD4=",
        "vbscript:msgbox(1)",
    ];

    for url in dangerous_urls {
        let node = avatar::image(ImageStatus::Loaded, url, "safe alt text", vec![]);
        let html = render(&node);
        assert!(
            !html.contains("src="),
            "危険な URL スキームなのに src 属性が出力されている: url={url:?}, html={html}"
        );
        assert!(
            html.contains(r#"alt="safe alt text""#),
            "src 属性の拒否によって兄弟属性 alt まで欠落している: html={html}"
        );
    }
}

/// (3) URL 属性経路: 安全な URL は `src="..."` として透過することを固定する
/// （陽性・陰性の両建て、vacuous pass 防止）。
#[test]
fn avatar_image_src_passes_through_safe_urls_via_headless_ui() {
    for url in ["/avatars/1.png", "https://example.com/a.png"] {
        let node = avatar::image(ImageStatus::Loaded, url, "avatar", vec![]);
        let html = render(&node);
        let expected = format!(r#"src="{}""#, escape_html(url));
        assert!(
            html.contains(&expected),
            "安全な URL が src 属性として透過していない: url={url:?}, html={html}"
        );
    }
}
