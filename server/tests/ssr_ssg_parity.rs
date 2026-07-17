//! SSR / SSG 出力完全一致の回帰テストスイート（TASK-6.4、イシュー #50）。
//!
//! REQ-6 の中核保証「SSG が書き出す静的ファイルは、対応する SSR
//! （[`rws_server::ssr::respond`]）が返す文字列と完全一致する」ことを、
//! [`rws_server::ssg::generate`] の**戻り値 `Vec<PathBuf>` を起点にした
//! 網羅的な走査**で固定する。
//!
//! # `server/tests/three_mode_integration.rs` との役割分担
//!
//! `three_mode_integration.rs`（TASK-6.1d、イシュー #45）は三モード
//! （SSR/SSG/CSR）統合の最小固定であり、ハードコードしたルート表
//! （`/` と `demo_items()` の各 id）に対する一致確認のみを行う。
//! 本ファイルはそれを土台に、以下の観点で「拡充」する：
//!
//! - `generate()` の戻り値からルートを逆引きする網羅走査（ルートが増えても
//!   追従する構造。ハードコード表とは独立した経路で一致を検証する）
//! - 出力ファイル集合の過不足なき一致（書き漏らし・余計な書き出しがないこと）
//! - 決定性（同一入力を複数回実行しても同一バイト列になること）
//! - 再生成（同一ディレクトリへの再実行）シナリオでの一致維持
//! - 出力パスが `out_dir` 配下に限定されること（`ssg.rs` のパストラバーサル
//!   対策の外形回帰、OWASP A01）
//! - 200 以外のレスポンスを書き出さない契約の固定
//!
//! # セキュリティ不変条件
//!
//! - 比較はすべて文字列ではなくバイト列（`fs::read`）で行い、エンコーディング
//!   ・改行差異まで検知する（データ完全性）。
//! - XSS ペイロード（`demo_items()` の id "2"）については、SSR/SSG 両経路が
//!   バイト一致するだけでなく、両経路とも既定エスケープ済み
//!   （`&lt;script&gt;alert` を含み `<script>alert` を含まない）ことを固定
//!   する（REQ-1 回帰、`raw_html()` 等のエスケープ迂回 API は使用しない）。
//! - 一時ディレクトリはテスト専用ヘルパー（`support/temp_dir.rs`、
//!   `tempfile` 非依存）を再利用し、新規外部依存を追加しない（REQ-3）。

use rws_app::demo_items;
use rws_server::ssg::generate;
use rws_server::ssr::respond;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

// unit test（`server/src/ssg.rs`）と integration test は別クレートとして
// リンクされ `#[cfg(test)]` アイテムを共有できないため、`include!` で
// ソースを直接展開する（`server/tests/support/temp_dir.rs` 参照）。
include!("support/temp_dir.rs");

/// `generate()` が書き出したファイルパス（`out_dir` からの相対パス）から、
/// 対応する SSR リクエストパスを逆引きする。
///
/// 固定ルート表（`index.html` → `/`、`items/{id}/index.html` →
/// `/items/{id}`）は `server/src/ssg.rs` の `write_route` 呼び出し規約と
/// 対になっており、本関数はその逆写像として振る舞う。ルートが増えても
/// このマッピング規則自体は変わらない前提（`items/{id}/index.html` 型）で
/// 書かれているため、`demo_items()` が増減しても網羅走査が自動追従する。
fn request_path_for(out_dir: &Path, file_path: &Path) -> String {
    let relative = file_path
        .strip_prefix(out_dir)
        .expect("generate() が返すパスは常に out_dir 配下であるはず")
        .to_str()
        .expect("生成されるパスは常に UTF-8");

    if relative == "index.html" {
        return "/".to_string();
    }

    let id = relative
        .strip_prefix("items/")
        .and_then(|rest| rest.strip_suffix("/index.html"))
        .unwrap_or_else(|| panic!("未知のファイルレイアウト: {relative}"));
    format!("/items/{id}")
}

/// テスト観点 1: `generate()` の戻り値を起点に全ファイルを走査し、各ファイルの
/// バイト列が対応する SSR レスポンスのバイト列と完全一致することを検証する。
/// あわせて UTF-8 妥当性・`<!DOCTYPE html>` 開始を確認する。
#[test]
fn all_generated_files_match_ssr_bytes_exhaustively() {
    let dir = TempDir::new("exhaustive");
    let written = generate(&dir.0).expect("generate should succeed");
    assert!(
        !written.is_empty(),
        "少なくとも 1 ファイルは書き出されるはず"
    );

    for file_path in &written {
        let request_path = request_path_for(&dir.0, file_path);
        let ssr_body = respond(&request_path)
            .unwrap_or_else(|| panic!("SSR route not found for {request_path}"))
            .body;

        let ssg_bytes = fs::read(file_path).expect("生成されたファイルは読み取れるはず");
        assert_eq!(
            ssg_bytes,
            ssr_body.as_bytes(),
            "path={request_path} で SSR/SSG のバイト列が不一致"
        );

        let ssg_text = String::from_utf8(ssg_bytes)
            .unwrap_or_else(|_| panic!("path={request_path} の出力は UTF-8 であるはず"));
        assert!(
            ssg_text.starts_with("<!DOCTYPE html>"),
            "path={request_path} は <!DOCTYPE html> で開始するはず"
        );
    }
}

/// テスト観点 2: 出力ディレクトリを再帰走査したファイル集合が、
/// 「`index.html` + `demo_items()` 件数分の `items/{id}/index.html`」と
/// 過不足なく一致すること（書き漏らし・余計な書き出しの回帰）。
#[test]
fn generated_file_set_is_exactly_the_route_table() {
    let dir = TempDir::new("file-set");
    generate(&dir.0).expect("generate should succeed");

    let mut expected: BTreeSet<PathBuf> = BTreeSet::new();
    expected.insert(dir.0.join("index.html"));
    for item in demo_items() {
        expected.insert(dir.0.join("items").join(&item.id).join("index.html"));
    }

    let mut actual: BTreeSet<PathBuf> = BTreeSet::new();
    collect_files(&dir.0, &mut actual);

    assert_eq!(actual, expected);
}

/// `root` 配下のファイル（ディレクトリを除く）を再帰的に集める素朴なヘルパー。
/// 標準ライブラリのみを使用し（REQ-3）、テスト専用の走査用途に限定する。
fn collect_files(root: &Path, out: &mut BTreeSet<PathBuf>) {
    let entries = fs::read_dir(root).expect("read_dir should succeed");
    for entry in entries {
        let entry = entry.expect("dir entry should be readable");
        let path = entry.path();
        let file_type = entry.file_type().expect("file_type should be readable");
        if file_type.is_dir() {
            collect_files(&path, out);
        } else {
            out.insert(path);
        }
    }
}

/// テスト観点 3: XSS ペイロード fixture（id "2"）の詳細ページについて、
/// SSR/SSG のバイト一致に加え、両経路とも既定エスケープされていることを
/// 固定する（REQ-1、モード間でエスケープ処理の差異がないことの回帰）。
#[test]
fn xss_payload_page_parity_is_byte_exact_and_escaped() {
    let dir = TempDir::new("xss-parity");
    generate(&dir.0).expect("generate should succeed");

    let payload_item = demo_items()
        .into_iter()
        .find(|it| it.id == "2")
        .expect("demo_items() の id \"2\" が XSS ペイロード fixture のはず");

    let ssr_body = respond(&format!("/items/{}", payload_item.id))
        .expect("payload item route should match")
        .body;
    let ssg_path = dir
        .0
        .join("items")
        .join(&payload_item.id)
        .join("index.html");
    let ssg_bytes = fs::read(&ssg_path).expect("payload item file should exist");

    assert_eq!(ssg_bytes, ssr_body.as_bytes());

    for body in [ssr_body.as_str(), std::str::from_utf8(&ssg_bytes).unwrap()] {
        assert!(!body.contains("<script>alert"));
        assert!(body.contains("&lt;script&gt;alert"));
    }
}

/// テスト観点 4: `generate()` を別ディレクトリへ 2 回実行しても全ファイルが
/// バイト一致し、`respond()` の反復呼び出しも一致すること（時刻・乱数・
/// グローバル状態への非依存の固定）。
#[test]
fn generate_is_deterministic_across_runs() {
    let dir_a = TempDir::new("determinism-a");
    let dir_b = TempDir::new("determinism-b");

    let written_a = generate(&dir_a.0).expect("first generate should succeed");
    let written_b = generate(&dir_b.0).expect("second generate should succeed");

    assert_eq!(written_a.len(), written_b.len());

    for file_a in &written_a {
        let request_path = request_path_for(&dir_a.0, file_a);
        let relative = file_a
            .strip_prefix(&dir_a.0)
            .expect("file_a should be under dir_a");
        let file_b = dir_b.0.join(relative);

        let bytes_a = fs::read(file_a).expect("file_a should be readable");
        let bytes_b = fs::read(&file_b).expect("file_b should be readable");
        assert_eq!(
            bytes_a, bytes_b,
            "path={request_path} で実行間の出力が不一致"
        );
    }

    // respond() 自体の反復呼び出しも副作用・グローバル状態を持たないことを確認する。
    assert_eq!(respond("/").unwrap().body, respond("/").unwrap().body);
}

/// テスト観点 5: 同一ディレクトリへ再実行（上書き）した後も SSR とバイト一致
/// すること（差分ビルド・再生成シナリオの回帰）。
#[test]
fn regenerate_into_same_dir_preserves_parity() {
    let dir = TempDir::new("regenerate");
    generate(&dir.0).expect("first generate should succeed");
    let written = generate(&dir.0).expect("regenerate into same dir should succeed");

    for file_path in &written {
        let request_path = request_path_for(&dir.0, file_path);
        let ssr_body = respond(&request_path).unwrap().body;
        let ssg_bytes = fs::read(file_path).unwrap();
        assert_eq!(ssg_bytes, ssr_body.as_bytes());
    }
}

/// テスト観点 6: `generate()` の戻り値の全パスが、`canonicalize()` 後に
/// `out_dir` 配下に収まること（`ssg.rs` の id ホワイトリスト検証・
/// パストラバーサル対策の外形回帰、OWASP A01）。
#[test]
fn generated_paths_stay_within_out_dir() {
    let dir = TempDir::new("path-containment");
    let written = generate(&dir.0).expect("generate should succeed");

    let canonical_out_dir = dir
        .0
        .canonicalize()
        .expect("out_dir should be canonicalizable after generate() creates it");

    for file_path in &written {
        let canonical_file = file_path
            .canonicalize()
            .unwrap_or_else(|e| panic!("{file_path:?} should be canonicalizable: {e}"));
        assert!(
            canonical_file.starts_with(&canonical_out_dir),
            "{canonical_file:?} は out_dir {canonical_out_dir:?} 配下に収まらない"
        );
    }
}

/// テスト観点 7: 未知 id は SSR で 404 になり、`generate()` の出力集合にも
/// 対応するファイルが存在しないこと（「200 応答ボディのみ書き出す」契約の固定）。
#[test]
fn ssg_never_writes_non_200_bodies() {
    let unknown_response = respond("/items/does-not-exist").expect("pattern should still match");
    assert_eq!(unknown_response.status, 404);

    let dir = TempDir::new("no-404-files");
    generate(&dir.0).expect("generate should succeed");

    assert!(!dir.0.join("items/does-not-exist/index.html").exists());
}
