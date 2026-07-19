// `wasm-client/tests/three_mode_integration.rs` から `include!` マクロ経由で
// 読み込むテスト専用ヘルパー。`server/tests/support/temp_dir.rs`（イシュー
// #45・#50）の複製。
//
// integration test は各ファイルが独立バイナリとしてコンパイルされ、かつ
// 別クレート（`rws-server`）の test-only コードは参照できないため、この
// クレートにも同型のヘルパーを複製する（`core/tests/no_branching_across_modes.rs`
// の `collect_rs_files`/`strip_comments` 複製と同じ理由）。
//
// `tempfile` 等の外部クレートを追加せず、`std::env::temp_dir()` +
// プロセス固有サフィックスで一時ディレクトリを代用する（REQ-3。本ヘルパーは
// dev-dependency 経由でのみ使われ、`rws-wasm-client` の製品面依存を増やさない）。
//
// 呼び出し文脈:
// - `include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/support/temp_dir.rs"))`
//   の形で各テストファイルへ本体をインライン展開して使う（別クレート化・
//   `pub` 公開はしない。テスト専用コードを本クレートの公開 API 面に
//   漏らさないため）。
//
// 通常の doc コメント（`//!`）ではなく `//` を使うのは、`include!` は
// ファイル中腹に展開されるため `//!`（内部属性扱い）が
// `E0753: expected outer doc comment` になるため。

/// テスト専用の一時ディレクトリ。`Drop` でベストエフォート削除する
/// （削除失敗はテスト結果の正当性に影響しないため無視する）。
struct TempDir(std::path::PathBuf);

impl TempDir {
    /// `tag` を含む一意なパス（プロセス ID + ナノ秒タイムスタンプ）を
    /// `std::env::temp_dir()` 配下に生成する。ディレクトリ自体の作成は
    /// 呼び出し先（`ssg::generate` 等）の `create_dir_all` に委ねる。
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!(
            "rws-wasm-client-test-{tag}-{}-{unique}",
            std::process::id()
        ));
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // 後片付け失敗はテスト失敗にしない（一時ディレクトリの残留は
        // テスト結果の正当性に影響しない）。
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
