//! 実サイト（`site/nav.toml`）を統合テストバイナリ内で 1 回だけビルドし、
//! 複数のテスト関数へ共有するヘルパー（イシュー #2299）。
//!
//! テスト関数ごとに `build_site(&repo_root(), &out_dir)` を呼び直すと、
//! 実サイト 227 ページ分のフルビルドが繰り返される。CI 実測では
//! `search_index.rs`（実サイトビルド 4 回）・`site_showcase.rs`（5 回）・
//! `site_build.rs`（3 回）・`component_specs_1155.rs`（2 回）・
//! `blocks_contract.rs`（13 回）・`no_js_contract.rs`（8 回）が典型で、
//! これらの**プロセス内での重複ビルド**が `cargo test --workspace` 全体の
//! 所要時間（約 13 分）の大半を占めていた。本モジュールは
//! `std::sync::LazyLock` を使い、同一テストバイナリ（＝同一プロセス）内で
//! 実サイトビルドを 1 回だけ実行し、以降の呼び出しはキャッシュ済みの
//! 出力ディレクトリと [`fandhe_frontend_docs_site::build::BuildReport`] を
//! 再利用する。
//!
//! # 契約（重要）
//!
//! - 共有ビルドの出力ディレクトリは **読み取り専用** でのみ使うこと。
//!   出力ディレクトリへ書き込む・削除する・改変するテストは、この共有
//!   ビルドを使わず自前で `build_site` を呼ぶこと（各テストバイナリの
//!   既存 `TempDir` ヘルパーを使う）。
//! - 決定性検証（同じ入力を 2 回ビルドして出力が byte-identical であることを
//!   固定するテスト）は、共有ビルドを比較対象の一方として使いつつ、もう
//!   一方は独自に `build_site` を呼んでビルドすること（`search_index.rs`
//!   の実装を参照）。共有ビルド 1 回だけでは決定性を検証したことにならない。
//! - フィクスチャサイト（`tests/fixtures/` 配下）をビルドするテストは対象外
//!   （本モジュールは実リポジトリの `site/nav.toml` 専用）。
//! - 初期化（実サイトビルド）が失敗した場合、同一バイナリ内で
//!   [`real_site`] を呼ぶ全テストが FAIL する（`LazyLock` の初期化 panic は
//!   以降の呼び出しでも再 panic する）。失敗理由が複数のテスト名に重複
//!   表示されるが、実サイトが壊れていれば全テストが失敗すべきであり、
//!   意図的なトレードオフとして許容する。
//!
//! # 一時領域
//!
//! 出力ディレクトリは `env!("CARGO_TARGET_TMPDIR")` 基点（コンパイル時に
//! 確定する値のみを使い、実行時フォールバックで `/tmp` へリークしない、
//! `ci.md` イシュー #637 の一時領域配置方針）に「プロセス ID + 起動時刻
//! （ナノ秒）」を含む一意なパスとして作成する。同名ディレクトリが既に
//! 存在する場合（PID 再利用 + 時刻衝突という極端な場合）は中身ごと削除して
//! から空で作り直し、`build_site` が旧ファイルを清掃しないことによる
//! 削除済みページ等の混入（`no_js_contract.rs` のディレクトリ走査に
//! 影響する）を防ぐ。`static` に保持するため `Drop` は走らず（プロセス
//! 終了時に実行されない）、代わりに次回の初期化時に同じ prefix を持つ
//! 旧世代のうち mtime が [`STALE_MIN_AGE`]（1 時間）より古いものを回収して
//! 蓄積を有界化する（`crates/cli/tests/new_gate_e2e.rs::cleanup_stale_scratch`
//! と同じ「次回実行時に旧世代を回収する」方針。並走中の別テストバイナリの
//! 出力を誤って消さないよう、直近 1 時間以内に更新されたものは残す）。
//! CI runner はジョブごとに使い捨てのため、CI 上では恒久蓄積しない。
//!
//! `#[path = "support/shared_site.rs"] mod shared_site;` で取り込む各
//! テストバイナリは個別にコンパイル・実行される別プロセスであるため、
//! `LazyLock` のインスタンスもバイナリごとに独立する（バイナリをまたいだ
//! 共有は行わない・する必要もない）。

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use fandhe_frontend_docs_site::build::{build_site, BuildReport};

/// `CARGO_MANIFEST_DIR`（`crates/docs-site`）から repo_root を解決する
/// （各テストバイナリの既存 `repo_root` 実装と同一規約）。
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo_root should resolve from CARGO_MANIFEST_DIR")
}

/// 一時領域の基点。cargo が `CARGO_TARGET_TMPDIR` を設定するのはテスト
/// バイナリのコンパイル時のみ（実行時の `std::env::var` 参照は常に失敗する、
/// `ci.md` イシュー #637）ため、コンパイル時に確定する `env!` のみを使う。
fn scratch_root() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let _ = std::fs::create_dir_all(&root);
    root
}

/// 実サイトビルドの共有結果。
pub struct SharedSite {
    /// ビルド出力ディレクトリ（読み取り専用で使うこと）。
    pub out_dir: PathBuf,
    /// `build_site` が返したビルド結果のサマリ。
    pub report: BuildReport,
}

/// 共有ビルド出力ディレクトリ名の prefix（旧世代回収の対象判定にも使う）。
const OUT_DIR_PREFIX: &str = "fandhe-frontend-docs-site-shared-real-site-";

/// 旧世代の共有ビルド出力を回収する際の最小経過時間。これより新しいものは
/// 並走中の別テストバイナリが使用中の可能性があるため残す。
const STALE_MIN_AGE: std::time::Duration = std::time::Duration::from_secs(60 * 60);

/// `scratch_root()` 直下の旧世代の共有ビルド出力（同じ prefix を持ち、
/// mtime が [`STALE_MIN_AGE`] より古いディレクトリ）を回収する。失敗は
/// 無視する（回収はベストエフォートであり、テストの合否に影響させない）。
fn reclaim_stale_out_dirs(root: &Path) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    let now = std::time::SystemTime::now();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !name.starts_with(OUT_DIR_PREFIX) {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let is_stale = meta
            .modified()
            .ok()
            .and_then(|m| now.duration_since(m).ok())
            .is_some_and(|age| age >= STALE_MIN_AGE);
        if meta.is_dir() && is_stale {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

static SHARED_SITE: LazyLock<SharedSite> = LazyLock::new(|| {
    let root = scratch_root();
    reclaim_stale_out_dirs(&root);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let out_dir = root.join(format!("{OUT_DIR_PREFIX}{}-{nanos}", std::process::id()));
    // 万一同名ディレクトリが残っていても旧ファイルを引き継がない（空で作り直す）。
    if out_dir.exists() {
        std::fs::remove_dir_all(&out_dir)
            .unwrap_or_else(|e| panic!("clear stale shared real-site out dir {out_dir:?}: {e}"));
    }
    std::fs::create_dir_all(&out_dir)
        .unwrap_or_else(|e| panic!("create shared real-site out dir {out_dir:?}: {e}"));
    let report = build_site(&repo_root(), &out_dir)
        .expect("real site/nav.toml should build cleanly (shared build, issue #2299)");
    SharedSite { out_dir, report }
});

/// テストバイナリ内で共有される実サイトビルド結果を返す。初回呼び出し時
/// のみ実際にビルドし、以降はキャッシュ済みの結果を返す（同一プロセス内の
/// 複数テストスレッドからの並行呼び出しは `LazyLock` が直列化する）。
///
/// # 契約
///
/// 戻り値の `out_dir` は読み取り専用で使うこと（モジュール doc参照）。
pub fn real_site() -> &'static SharedSite {
    &SHARED_SITE
}
