// `server/src/ssg.rs` の unit test（`cfg(test)`）と
// `server/tests/three_mode_integration.rs`（integration test、別クレートと
// してリンクされるため unit test 側の `#[cfg(test)]` アイテムを参照できない）
// の双方から `include!` マクロ経由で読み込むテスト専用ヘルパー。
//
// `tempfile` 等の外部クレートを追加せず、`<target>/tmp` 配下 +
// プロセス固有サフィックスで一時ディレクトリを代用する
// （REQ-3: `fandhe-frontend-server` は外部依存ゼロを維持する）。
//
// 本ファイルは `src/ssg.rs` の unit test（ユニットテストバイナリ、
// `CARGO_TARGET_TMPDIR` はコンパイル時に設定されない）と
// `tests/three_mode_integration.rs`（統合テストバイナリ、cargo が
// コンパイル時のみ設定する、Cargo Book）の両方へ `include!` 展開される
// ため、`env!("CARGO_TARGET_TMPDIR")` を無条件には使えない（unit test
// 側でコンパイルエラーになる）。実行時 `CARGO_TARGET_DIR`
// （self-hosted runner の共有 `/cargo-target` 環境下では
// `/cargo-target/tmp` に収束し、統合テスト側の `env!` 既定と同一の
// 管理範囲に閉じる）→ `CARGO_MANIFEST_DIR` 基準のローカル既定レイアウト
// の順で解決し、`std::env::temp_dir()`（= `/tmp`）へは一切
// フォールバックしない（self-hosted runner の tmpfs を恒常的に消費して
// いたイシュー #637 の事実誤認の再発防止、#658）。
//
// 呼び出し文脈:
// - `include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/support/temp_dir.rs"))`
//   の形で各テストファイルへ本体をインライン展開して使う（別クレート化・
//   `pub` 公開はしない。テスト専用コードを本クレートの公開 API 面に
//   漏らさないため）。
// - 本ファイルは `std::path::PathBuf` / `std::fs` / `std::time` をすべて
//   完全修飾で参照するため、呼び出し側の `use` 状況に依存しない。
//
// 通常の doc コメント（`//!`）ではなく `//` を使うのは、`include!` は
// ファイル中腹に展開されるため `//!`（内部属性扱い）が
// `E0753: expected outer doc comment` になるため。

/// テスト専用の一時ディレクトリ。`Drop` でベストエフォート削除する
/// （削除失敗はテスト結果の正当性に影響しないため無視する）。
struct TempDir(std::path::PathBuf);

impl TempDir {
    /// `tag` を含む一意なパス（プロセス ID + ナノ秒タイムスタンプ）を
    /// `<target>/tmp` 配下に生成する。ディレクトリ自体の作成は
    /// 呼び出し先（`ssg::generate` 等）の `create_dir_all` に委ねる。
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let root = std::env::var("CARGO_TARGET_TMPDIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                std::env::var("CARGO_TARGET_DIR")
                    .map(|d| std::path::PathBuf::from(d).join("tmp"))
                    .unwrap_or_else(|_| {
                        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                            .join("../../target/tmp")
                    })
            });
        let _ = std::fs::create_dir_all(&root);
        let path = root.join(format!(
            "fandhe-frontend-server-test-{tag}-{}-{unique}",
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
