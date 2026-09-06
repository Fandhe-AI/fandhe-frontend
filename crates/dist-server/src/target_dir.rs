//! `build.rs` の `OUT_DIR` から、そのビルドが使っている `CARGO_TARGET_DIR` を
//! 逆算する純粋関数。
//!
//! `build.rs` 自身はパッケージ自身の lib を `build-dependencies` にできない
//! （循環依存）ため、`src/wasm_stage_cache.rs`・`src/workspace_detect.rs` と
//! 同型のパターンで `#[path]` によりこのファイルをソースレベル共有する
//! （`build.rs` 冒頭の `mod` 宣言参照）。`lib.rs` 側（通常のクレートモジュール）
//! では `cargo test -p fandhe-frontend-dist-server` によるユニットテスト対象と
//! する。

use std::path::{Path, PathBuf};

/// `OUT_DIR`（cargo が保証する固定階層 `<CARGO_TARGET_DIR>/[<triple>/]<profile>/
/// build/<pkg>-<hash>/out`）から、このビルドが使っている `CARGO_TARGET_DIR` を
/// 逆算する。呼び出し元（`build.rs::run_wasm_stage`）が `PATH` 監視ループで
/// Cargo 自身の生成物ディレクトリ（`target/<profile>`・`target/<profile>/deps`、
/// `--target` 指定時は `target/<triple>/<profile>`・
/// `target/<triple>/<profile>/deps`）を除外するために使う（PR #1980 レビュー
/// 指摘）。
///
/// # ホストビルドと `--target` 指定ビルドの両階層に対応する
///
/// `OUT_DIR` の祖先を 4 段上がった位置（`out` → `<pkg>-<hash>` → `build` →
/// `<profile>` → 4 段目）は、ホストビルド（`cargo build`、`--target` なし）
/// では `CARGO_TARGET_DIR` そのものだが、`--target <triple>` 指定ビルドでは
/// cargo がさらに `<triple>` セグメントを挟むため `<CARGO_TARGET_DIR>/<triple>`
/// になる。区別するため、`target_triple` 引数（呼び出し元は `TARGET` 環境
/// 変数を渡す。cargo が build script に常に設定する）と 4 段目ディレクトリ名
/// が一致する場合はさらに 1 段上がる。一致しない場合はホストビルドとみなし
/// 4 段目をそのまま返す。
///
/// `cargo_target_dir_env`（呼び出し元は `CARGO_TARGET_DIR` 環境変数を渡す）が
/// `Some` の場合は、この逆算を行わずそちらを優先する（cargo が
/// `--target-dir`/`CARGO_TARGET_DIR` で明示指定された値をそのまま build
/// script へ伝えないため通常は `None` だが、将来 cargo の挙動が変わった
/// 場合や呼び出し元がテスト目的で明示指定した場合に備える）。
///
/// 想定より浅い階層（テスト環境等で `OUT_DIR` がこの形を満たさない場合）は
/// `None` を返し、呼び出し元は除外なし（フェイルオープン、監視漏れなし側）
/// にフォールバックする。
pub fn cargo_target_dir_from_out_dir(
    out_dir: &Path,
    target_triple: Option<&str>,
    cargo_target_dir_env: Option<&str>,
) -> Option<PathBuf> {
    if let Some(dir) = cargo_target_dir_env {
        if !dir.is_empty() {
            return Some(PathBuf::from(dir));
        }
    }

    let level4 = out_dir.ancestors().nth(4)?.to_path_buf();

    if let Some(triple) = target_triple {
        if level4.file_name().and_then(|name| name.to_str()) == Some(triple) {
            return level4.parent().map(Path::to_path_buf);
        }
    }

    Some(level4)
}

#[cfg(test)]
mod tests {
    use super::cargo_target_dir_from_out_dir;
    use std::path::PathBuf;

    /// ホストビルド（`--target` なし）の `OUT_DIR` 形状
    /// `<CARGO_TARGET_DIR>/<profile>/build/<pkg>-<hash>/out` からは、4 段上の
    /// 祖先がそのまま `CARGO_TARGET_DIR` になる。
    #[test]
    fn host_build_out_dir_resolves_to_target_dir() {
        let out_dir = PathBuf::from("/repo/target/release/build/pkg-abcdef/out");
        let resolved =
            cargo_target_dir_from_out_dir(&out_dir, Some("x86_64-unknown-linux-gnu"), None);
        assert_eq!(resolved, Some(PathBuf::from("/repo/target")));
    }

    /// `--target <triple>` 指定ビルドの `OUT_DIR` 形状
    /// `<CARGO_TARGET_DIR>/<triple>/<profile>/build/<pkg>-<hash>/out` では、
    /// 4 段上の祖先は triple ディレクトリでしかないため、triple 名が一致した
    /// 場合はさらに 1 段上がって `CARGO_TARGET_DIR` を返す
    /// （本 PR のレビュー指摘の再現ケース）。
    #[test]
    fn target_triple_build_out_dir_resolves_to_target_dir() {
        let out_dir =
            PathBuf::from("/repo/target/wasm32-unknown-unknown/release/build/pkg-abcdef/out");
        let resolved =
            cargo_target_dir_from_out_dir(&out_dir, Some("wasm32-unknown-unknown"), None);
        assert_eq!(resolved, Some(PathBuf::from("/repo/target")));
    }

    /// 4 段目ディレクトリ名が `target_triple` と一致しない場合
    /// （ホストビルド、あるいは profile 名がたまたま別の triple と同名の
    /// 場合はない前提）は、誤って余分に 1 段上がらずホスト扱いのまま返す。
    #[test]
    fn non_matching_triple_does_not_overshoot() {
        let out_dir = PathBuf::from("/repo/target/debug/build/pkg-abcdef/out");
        let resolved =
            cargo_target_dir_from_out_dir(&out_dir, Some("wasm32-unknown-unknown"), None);
        assert_eq!(resolved, Some(PathBuf::from("/repo/target")));
    }

    /// `target_triple` が渡されない（`None`）場合は triple 判定をスキップし、
    /// 常にホストビルド扱い（4 段上の祖先をそのまま返す）にフォールバックする。
    #[test]
    fn no_triple_provided_skips_triple_check() {
        let out_dir =
            PathBuf::from("/repo/target/wasm32-unknown-unknown/release/build/pkg-abcdef/out");
        let resolved = cargo_target_dir_from_out_dir(&out_dir, None, None);
        assert_eq!(
            resolved,
            Some(PathBuf::from("/repo/target/wasm32-unknown-unknown"))
        );
    }

    /// `CARGO_TARGET_DIR` 環境変数が明示されている場合は、逆算より優先して
    /// そちらをそのまま返す。
    #[test]
    fn explicit_cargo_target_dir_env_takes_priority() {
        let out_dir =
            PathBuf::from("/repo/target/wasm32-unknown-unknown/release/build/pkg-abcdef/out");
        let resolved = cargo_target_dir_from_out_dir(
            &out_dir,
            Some("wasm32-unknown-unknown"),
            Some("/custom/target"),
        );
        assert_eq!(resolved, Some(PathBuf::from("/custom/target")));
    }

    /// 想定より浅い階層（テスト環境等で cargo の固定階層を満たさない `OUT_DIR`）
    /// では `None`（除外なし、フェイルオープン）を返す。
    #[test]
    fn shallow_out_dir_returns_none() {
        let out_dir = PathBuf::from("/out");
        let resolved =
            cargo_target_dir_from_out_dir(&out_dir, Some("wasm32-unknown-unknown"), None);
        assert_eq!(resolved, None);
    }
}
