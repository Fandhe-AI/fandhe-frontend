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
/// 呼び出し元（`build.rs`）はこの戻り値を `PATH` 上の各エントリに対して
/// `Path::starts_with` で比較し、Cargo 自身の生成物ディレクトリを除外する
/// （直後のコメント参照）。`Path::starts_with` はコンポーネント単位の比較で
/// あり、比較対象の双方が絶対パスであることを前提とする（例えば相対パス
/// `"target"` は `PATH` エントリの絶対パスに対して `starts_with` が常に
/// `false` を返す）。`OUT_DIR` は cargo が常に絶対パスとして設定する
/// ため、この逆算結果は常に絶対パスになる。一方 `cargo_target_dir_env`
/// （`CARGO_TARGET_DIR` 環境変数）は `CARGO_TARGET_DIR=target` のような
/// 相対指定もそのまま渡ってくる契約であり、これを絶対パスかどうか
/// 検証せず優先して返すと、相対指定時に呼び出し元の除外判定が常に失敗し
/// Cargo の生成物ディレクトリが監視対象に残ってしまう（再ビルドループが
/// 再発する、codex P1 / Cursor Bugbot 指摘）。このため `OUT_DIR` からの
/// 逆算を常に優先し、逆算が `None`（想定より浅い `OUT_DIR` 階層。テスト
/// 環境等）の場合に限り `cargo_target_dir_env` へフォールバックする
/// （`--target-dir`/`CARGO_TARGET_DIR` で明示指定された値を cargo が
/// build script へそのまま伝える将来の挙動変化や、呼び出し元がテスト目的で
/// 明示指定するケースへの保険）。フォールバック時も渡された値をそのまま
/// 返すのみで絶対化はしない（呼び出し元が比較不能な相対パスを受け取る
/// リスクは残るが、逆算優先により通常経路では発生しない）。
///
/// 逆算・フォールバックのいずれでも解決できない場合は `None` を返し、
/// 呼び出し元は除外なし（フェイルオープン、監視漏れなし側）に
/// フォールバックする。
pub fn cargo_target_dir_from_out_dir(
    out_dir: &Path,
    target_triple: Option<&str>,
    cargo_target_dir_env: Option<&str>,
) -> Option<PathBuf> {
    if let Some(resolved) = out_dir.ancestors().nth(4) {
        let level4 = resolved.to_path_buf();

        if let Some(triple) = target_triple {
            if level4.file_name().and_then(|name| name.to_str()) == Some(triple) {
                return level4.parent().map(Path::to_path_buf);
            }
        }

        return Some(level4);
    }

    match cargo_target_dir_env {
        Some(dir) if !dir.is_empty() => Some(PathBuf::from(dir)),
        _ => None,
    }
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

    /// `OUT_DIR` から正しく逆算できる場合は、`CARGO_TARGET_DIR` 環境変数が
    /// 明示されていても逆算結果を優先する。`OUT_DIR` は cargo が常に絶対
    /// パスとして設定するため、呼び出し元の `Path::starts_with` 比較
    /// （双方が絶対パスであることが前提）が確実に機能する（codex P1 /
    /// Cursor Bugbot 指摘: 環境変数優先だと相対指定・`--target-dir` との
    /// 不一致で除外が効かなくなり再ビルドループが再発する）。
    #[test]
    fn out_dir_resolution_takes_priority_over_env() {
        let out_dir =
            PathBuf::from("/repo/target/wasm32-unknown-unknown/release/build/pkg-abcdef/out");
        let resolved = cargo_target_dir_from_out_dir(
            &out_dir,
            Some("wasm32-unknown-unknown"),
            Some("/custom/target"),
        );
        assert_eq!(resolved, Some(PathBuf::from("/repo/target")));
    }

    /// `CARGO_TARGET_DIR` 環境変数が相対パス（例: `CARGO_TARGET_DIR=target`）
    /// で指定されていても、`OUT_DIR` から逆算できる限りそちらが優先される
    /// ため、呼び出し元が相対パスを受け取って比較を失敗させることはない。
    #[test]
    fn relative_env_does_not_override_successful_resolution() {
        let out_dir = PathBuf::from("/repo/target/release/build/pkg-abcdef/out");
        let resolved = cargo_target_dir_from_out_dir(
            &out_dir,
            Some("x86_64-unknown-linux-gnu"),
            Some("target"),
        );
        assert_eq!(resolved, Some(PathBuf::from("/repo/target")));
    }

    /// `CARGO_TARGET_DIR` 環境変数と実際の `--target-dir` 指定が異なる
    /// （通常は起こらないが防御的に検証する）場合でも、`OUT_DIR` からの
    /// 逆算が優先されるため実際のビルド生成物ディレクトリと一致する。
    #[test]
    fn out_dir_resolution_wins_even_when_env_disagrees() {
        let out_dir =
            PathBuf::from("/actual/target/wasm32-unknown-unknown/release/build/pkg-abcdef/out");
        let resolved = cargo_target_dir_from_out_dir(
            &out_dir,
            Some("wasm32-unknown-unknown"),
            Some("/stale/target"),
        );
        assert_eq!(resolved, Some(PathBuf::from("/actual/target")));
    }

    /// 想定より浅い階層（テスト環境等で cargo の固定階層を満たさない `OUT_DIR`）
    /// では逆算できないため、`CARGO_TARGET_DIR` 環境変数へフォールバックする。
    #[test]
    fn shallow_out_dir_falls_back_to_env() {
        let out_dir = PathBuf::from("/out");
        let resolved = cargo_target_dir_from_out_dir(
            &out_dir,
            Some("wasm32-unknown-unknown"),
            Some("/custom/target"),
        );
        assert_eq!(resolved, Some(PathBuf::from("/custom/target")));
    }

    /// 想定より浅い階層かつ `CARGO_TARGET_DIR` 環境変数も未設定の場合は
    /// `None`（除外なし、フェイルオープン）を返す。
    #[test]
    fn shallow_out_dir_without_env_returns_none() {
        let out_dir = PathBuf::from("/out");
        let resolved =
            cargo_target_dir_from_out_dir(&out_dir, Some("wasm32-unknown-unknown"), None);
        assert_eq!(resolved, None);
    }
}
