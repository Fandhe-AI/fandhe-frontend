//! `static/view-transitions.js`（イシュー #61, TASK-8.2a）の構造回帰テスト。
//!
//! CI に Node ランタイムが存在しないため、JS 成果物の検証は本 workspace の
//! `cargo test --workspace` 経路（既存 `.github/workflows/ci.yml`）に乗せる
//! 構造チェックとして実装する。JS の実行結果ではなくソーステキストの契約
//! （公開 API 名・フォールバック分岐の存在）を固定する。
//!
//! 実効 LOC（10 行以内、REQ-8 受け入れ基準）の機械カウントと CI ゲート化は
//! 兄弟イシュー #62（TASK-8.2b）のスコープであり、本テストには含めない。

use std::path::PathBuf;

/// workspace ルート（`xtask/` の親ディレクトリ）から見た対象ファイルの絶対パスを返す。
fn view_transitions_js_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/xtask/ から 2 段上でワークスペースルートに到達する（イシュー #436）")
        .join("static")
        .join("view-transitions.js")
}

#[test]
fn view_transitions_js_exists_and_is_readable() {
    let path = view_transitions_js_path();
    assert!(
        path.is_file(),
        "static/view-transitions.js が存在しない: {}",
        path.display()
    );
    std::fs::read_to_string(&path).expect("static/view-transitions.js の読み取りに失敗した");
}

#[test]
fn view_transitions_js_exports_with_view_transition() {
    let source = std::fs::read_to_string(view_transitions_js_path())
        .expect("static/view-transitions.js の読み取りに失敗した");
    assert!(
        source.contains("export function withViewTransition("),
        "公開 API `export function withViewTransition(...)` が見つからない（呼び出し側との契約）"
    );
}

#[test]
fn view_transitions_js_falls_back_when_unsupported() {
    let source = std::fs::read_to_string(view_transitions_js_path())
        .expect("static/view-transitions.js の読み取りに失敗した");

    // View Transitions API 非対応ブラウザ向けのフィーチャー検出分岐（REQ-8 受け入れ基準）。
    assert!(
        source.contains("typeof document.startViewTransition"),
        "非対応ブラウザ検出（typeof document.startViewTransition）が見つからない"
    );
    // 非対応時は update() を直接呼び即時実行にフォールバックする契約。
    assert!(
        source.contains("update();"),
        "非対応時の即時実行フォールバック（update() の直接呼び出し）が見つからない"
    );
    // 対応時は document.startViewTransition() に update をそのまま委譲する契約
    // （呼び出し側が ViewTransition オブジェクトを受け取れるようにする）。
    assert!(
        source.contains("document.startViewTransition(update)"),
        "対応ブラウザでの委譲呼び出し（document.startViewTransition(update)）が見つからない"
    );
}
