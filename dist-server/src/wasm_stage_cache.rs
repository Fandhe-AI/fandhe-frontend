//! `dist-server/build.rs` の WASM ビルドステージ（TASK-10.2c、イシュー #111）が
//! 使うキャッシュ判定の純粋関数群。
//!
//! # なぜこのファイルが `src/` にあるのか
//!
//! `build.rs` はパッケージ自身の lib クレートを `build-dependencies` に
//! 指定できない（循環依存になるため）。一方でこのファイルの関数群
//! （ファイル内容ハッシュ・fingerprint 比較・成果物完全性チェック）は
//! I/O はあっても外部プロセスを起動しない純粋なロジックであり、
//! `cargo test` で単体テストしたい。
//!
//! そこで `bench_support.rs`（TASK-10.4a）と同じ「ロジックをテスト可能な
//! クレート側モジュールへ切り出す」パターンを採り、`build.rs` 側は
//! `#[path = "src/wasm_stage_cache.rs"] mod wasm_stage_cache;` で本ファイルを
//! ソースレベルで取り込む（リンクではなくソース共有のため、
//! `build-dependencies` の循環問題を回避できる）。
//!
//! `lib.rs` からは `#[doc(hidden)] pub mod wasm_stage_cache;` として通常の
//! モジュールとして取り込まれ、下記 `#[cfg(test)]` のユニットテストは
//! `cargo test -p rws-dist-server` で実行される。`build.rs` 側の
//! ビルドでは `#[cfg(test)]` は当然コンパイルされない
//! （build スクリプトは `cargo test` の対象ではないため）。
//!
//! ここに置く関数は「サブプロセスを起動しない・環境変数を読まない」ものに
//! 限定する（`wasm-bindgen`/ネスト `cargo build` の起動、バージョン照合等は
//! 引き続き `build.rs` 本体が担う）。

use std::fs;
use std::path::Path;

/// std のみで完結する自前 FNV-1a（64bit）実装。`build-dependencies`／通常の
/// 依存へハッシュ用クレートを追加しないための選択（REQ-3・`build.rs`
/// 冒頭ドキュメンテーションコメント「外部依存ゼロの理由」参照）。暗号学的
/// 強度は不要（ビルド成果物の変化検知が目的で、敵対的な衝突耐性は要求しない
/// 用途）。
pub fn fnv1a_hash(bytes: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn is_non_empty_file(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.len() > 0)
        .unwrap_or(false)
}

/// `wasm_assets_dir` に、`wasm_binary_path` のファイル幹名（`file_stem`）から
/// 想定される `wasm-bindgen --target web` の 2 大成果物
/// （`<stem>.js`・`<stem>_bg.wasm`）が両方とも存在し、かつ空ファイルでないかを
/// 確認する。キャッシュ HIT 判定・`run_wasm_bindgen` 成功確認の両方から使う
/// 共通ガード（呼び出し元は `build.rs`）。
pub fn wasm_assets_look_complete(wasm_assets_dir: &Path, wasm_binary_path: &Path) -> bool {
    let Some(stem) = wasm_binary_path.file_stem().and_then(|s| s.to_str()) else {
        return false;
    };
    let js_path = wasm_assets_dir.join(format!("{stem}.js"));
    let wasm_path = wasm_assets_dir.join(format!("{stem}_bg.wasm"));
    is_non_empty_file(&js_path) && is_non_empty_file(&wasm_path)
}

/// 現在の WASM ステージ入力から fingerprint 文字列を計算する。
///
/// 構成要素は「ネストビルドが生成した `.wasm` の内容ハッシュ（FNV-1a）」と
/// 「インストール済み `wasm-bindgen-cli` の実バージョン」。前者は wasm-full の
/// ソース変更を、後者は CLI 入れ替え（stale なグルーコード再利用の防止、
/// PR #217 review 4719879204 と同種の懸念）をそれぞれ捉える。`Cargo.lock` が
/// 解決する `wasm-bindgen` バージョンは `build.rs::run_wasm_stage` の
/// バージョン整合検証で既に CLI 実バージョンと一致していることが保証されて
/// いるため、fingerprint へ別途含める必要はない。
pub fn compute_wasm_stage_fingerprint(
    wasm_binary_path: &Path,
    installed_wasm_bindgen_version: &str,
) -> Result<String, String> {
    let bytes = fs::read(wasm_binary_path).map_err(|e| {
        format!(
            "failed to read {} for fingerprinting: {e}",
            wasm_binary_path.display()
        )
    })?;
    let hash = fnv1a_hash(&bytes);
    Ok(format!("{hash:016x}:{installed_wasm_bindgen_version}"))
}

/// fingerprint をファイルへ書き込む。呼び出しは `build.rs::run_wasm_stage` の
/// `wasm-bindgen` 成功パスからのみ行う（呼び出し元が成果物の存在を確認済み
/// であることが前提）。ここで初めて「次回はこの成果物を再利用してよい」と
/// いう記録が残るため、呼び出し順序を崩さないこと。
pub fn write_wasm_stage_fingerprint(
    wasm_binary_path: &Path,
    installed_wasm_bindgen_version: &str,
    fingerprint_path: &Path,
) -> Result<(), String> {
    let fingerprint =
        compute_wasm_stage_fingerprint(wasm_binary_path, installed_wasm_bindgen_version)?;
    fs::write(fingerprint_path, fingerprint)
        .map_err(|e| format!("failed to write {}: {e}", fingerprint_path.display()))
}

/// `wasm-bindgen` の再実行をスキップしてよいかを判定する（TASK-10.2c）。
///
/// 判定は次の全条件が揃った場合のみ `true`（HIT）を返す。1 つでも欠ければ
/// `false`（MISS = 再実行）に倒すフェイルクローズ方針を取る。
///
/// - `fingerprint_path` に前回の fingerprint が保存されている（読めない・
///   存在しない場合は即 MISS）
/// - 現在の入力（ネストビルドが生成した `.wasm` の内容ハッシュ + インストール
///   済み `wasm-bindgen-cli` の実バージョン）から計算した fingerprint と完全
///   一致する
/// - `wasm_assets_dir` に前回の成果物一式（`<stem>.js`/`<stem>_bg.wasm`）が
///   実際に残っている（fingerprint だけ一致してディレクトリが空、という
///   事故を防ぐ）
pub fn wasm_stage_cache_hit(
    wasm_binary_path: &Path,
    installed_wasm_bindgen_version: &str,
    fingerprint_path: &Path,
    wasm_assets_dir: &Path,
) -> bool {
    let Some(stored) = fs::read_to_string(fingerprint_path).ok() else {
        return false;
    };
    let Ok(current) =
        compute_wasm_stage_fingerprint(wasm_binary_path, installed_wasm_bindgen_version)
    else {
        return false;
    };
    stored.trim() == current && wasm_assets_look_complete(wasm_assets_dir, wasm_binary_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 一時ディレクトリを 1 つ作り、後始末までまとめて面倒を見るテスト
    /// ヘルパー（`assets.rs::with_temp_static_root` と同じ手法。`tempfile`
    /// 等の外部クレートを追加しない REQ-3 方針）。
    fn with_temp_dir(test_name: &str, body: impl FnOnce(&Path)) {
        let temp_root = std::env::temp_dir().join(format!(
            "rws-dist-server-wasm-stage-cache-{test_name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&temp_root).expect("create temp dir");

        body(&temp_root);

        let _ = fs::remove_dir_all(&temp_root);
    }

    /// FNV-1a の既知ベクタ（空文字列のオフセットベーシスそのもの、および
    /// "a" 1 バイトの手計算結果）で実装の正しさを固定する。
    #[test]
    fn fnv1a_hash_matches_known_vectors() {
        assert_eq!(fnv1a_hash(b""), 0xcbf29ce484222325);
        // offset_basis(0xcbf29ce484222325) XOR 'a'(0x61) を FNV prime で乗算した値。
        let expected = (0xcbf29ce484222325u64 ^ 0x61).wrapping_mul(0x100000001b3);
        assert_eq!(fnv1a_hash(b"a"), expected);
    }

    #[test]
    fn fnv1a_hash_differs_for_different_inputs() {
        assert_ne!(fnv1a_hash(b"wasm-full v1"), fnv1a_hash(b"wasm-full v2"));
    }

    #[test]
    fn wasm_assets_look_complete_true_when_both_artifacts_present_and_non_empty() {
        with_temp_dir("assets-complete", |dir| {
            let wasm_binary_path = dir.join("rws_wasm_full.wasm");
            fs::write(dir.join("rws_wasm_full.js"), b"glue code").unwrap();
            fs::write(dir.join("rws_wasm_full_bg.wasm"), b"\0asm...").unwrap();
            assert!(wasm_assets_look_complete(dir, &wasm_binary_path));
        });
    }

    #[test]
    fn wasm_assets_look_complete_false_when_wasm_artifact_missing() {
        with_temp_dir("assets-missing-wasm", |dir| {
            let wasm_binary_path = dir.join("rws_wasm_full.wasm");
            fs::write(dir.join("rws_wasm_full.js"), b"glue code").unwrap();
            // `_bg.wasm` を書かない。
            assert!(!wasm_assets_look_complete(dir, &wasm_binary_path));
        });
    }

    #[test]
    fn wasm_assets_look_complete_false_when_artifact_is_empty() {
        with_temp_dir("assets-empty-artifact", |dir| {
            let wasm_binary_path = dir.join("rws_wasm_full.wasm");
            fs::write(dir.join("rws_wasm_full.js"), b"").unwrap();
            fs::write(dir.join("rws_wasm_full_bg.wasm"), b"\0asm...").unwrap();
            assert!(!wasm_assets_look_complete(dir, &wasm_binary_path));
        });
    }

    /// レビュー指摘（false HIT 回避の自動検証不在）に対応する中核テスト:
    /// fingerprint 保存後に `.wasm` の内容が変化した（= wasm-full のソースが
    /// 変わった）場合、キャッシュは必ず MISS を返し stale な成果物を再利用
    /// しないことを固定する。
    #[test]
    fn cache_hit_is_false_when_wasm_binary_content_changed_after_fingerprint_was_written() {
        with_temp_dir("cache-miss-on-content-change", |dir| {
            let wasm_binary_path = dir.join("rws_wasm_full.wasm");
            let wasm_assets_dir = dir.join("wasm-assets");
            let fingerprint_path = dir.join("wasm-stage.fingerprint");
            let version = "0.2.126";

            fs::write(&wasm_binary_path, b"wasm bytes v1").unwrap();
            fs::create_dir_all(&wasm_assets_dir).unwrap();
            fs::write(wasm_assets_dir.join("rws_wasm_full.js"), b"glue v1").unwrap();
            fs::write(wasm_assets_dir.join("rws_wasm_full_bg.wasm"), b"\0asm v1").unwrap();
            write_wasm_stage_fingerprint(&wasm_binary_path, version, &fingerprint_path).unwrap();

            // 変更前は HIT のはず。
            assert!(wasm_stage_cache_hit(
                &wasm_binary_path,
                version,
                &fingerprint_path,
                &wasm_assets_dir,
            ));

            // wasm-full のソース変更を模した内容変更。
            fs::write(&wasm_binary_path, b"wasm bytes v2 (different source)").unwrap();

            assert!(
                !wasm_stage_cache_hit(
                    &wasm_binary_path,
                    version,
                    &fingerprint_path,
                    &wasm_assets_dir,
                ),
                "stale fingerprint must not HIT after the underlying .wasm content changed"
            );
        });
    }

    /// `wasm-bindgen-cli` の入れ替え（PR #217 review 4719879204 と同種の懸念）
    /// でも、`.wasm` の内容が同一であれば fingerprint のバージョン部分の
    /// 不一致だけで MISS に倒れることを固定する。
    #[test]
    fn cache_hit_is_false_when_wasm_bindgen_cli_version_changed() {
        with_temp_dir("cache-miss-on-version-change", |dir| {
            let wasm_binary_path = dir.join("rws_wasm_full.wasm");
            let wasm_assets_dir = dir.join("wasm-assets");
            let fingerprint_path = dir.join("wasm-stage.fingerprint");

            fs::write(&wasm_binary_path, b"wasm bytes (unchanged)").unwrap();
            fs::create_dir_all(&wasm_assets_dir).unwrap();
            fs::write(wasm_assets_dir.join("rws_wasm_full.js"), b"glue").unwrap();
            fs::write(wasm_assets_dir.join("rws_wasm_full_bg.wasm"), b"\0asm").unwrap();
            write_wasm_stage_fingerprint(&wasm_binary_path, "0.2.126", &fingerprint_path).unwrap();

            assert!(!wasm_stage_cache_hit(
                &wasm_binary_path,
                "0.2.127",
                &fingerprint_path,
                &wasm_assets_dir,
            ));
        });
    }

    /// fingerprint は一致していても成果物ディレクトリが空・欠落している
    /// 場合は MISS に倒れることを固定する（レビューコメントが指摘する
    /// 「fingerprint だけ一致してディレクトリが空」の事故防止）。
    #[test]
    fn cache_hit_is_false_when_fingerprint_matches_but_assets_are_missing() {
        with_temp_dir("cache-miss-on-missing-assets", |dir| {
            let wasm_binary_path = dir.join("rws_wasm_full.wasm");
            let wasm_assets_dir = dir.join("wasm-assets");
            let fingerprint_path = dir.join("wasm-stage.fingerprint");
            let version = "0.2.126";

            fs::write(&wasm_binary_path, b"wasm bytes").unwrap();
            write_wasm_stage_fingerprint(&wasm_binary_path, version, &fingerprint_path).unwrap();
            // `wasm_assets_dir` を作らない（成果物欠落を模す）。

            assert!(!wasm_stage_cache_hit(
                &wasm_binary_path,
                version,
                &fingerprint_path,
                &wasm_assets_dir,
            ));
        });
    }

    /// fingerprint ファイル自体が存在しない（初回ビルド）場合も MISS。
    #[test]
    fn cache_hit_is_false_when_fingerprint_file_is_absent() {
        with_temp_dir("cache-miss-no-fingerprint", |dir| {
            let wasm_binary_path = dir.join("rws_wasm_full.wasm");
            let wasm_assets_dir = dir.join("wasm-assets");
            let fingerprint_path = dir.join("wasm-stage.fingerprint");

            fs::write(&wasm_binary_path, b"wasm bytes").unwrap();
            fs::create_dir_all(&wasm_assets_dir).unwrap();
            fs::write(wasm_assets_dir.join("rws_wasm_full.js"), b"glue").unwrap();
            fs::write(wasm_assets_dir.join("rws_wasm_full_bg.wasm"), b"\0asm").unwrap();

            assert!(!wasm_stage_cache_hit(
                &wasm_binary_path,
                "0.2.126",
                &fingerprint_path,
                &wasm_assets_dir,
            ));
        });
    }
}
