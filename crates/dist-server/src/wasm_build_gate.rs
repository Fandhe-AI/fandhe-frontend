//! WASM ビルドステージの有効・無効判定（`FANDHE_FRONTEND_WASM_BUILD`）。
//!
//! `build.rs` 自身はパッケージ自身の lib を `build-dependencies` にできない
//! （循環依存）ため、`src/wasm_stage_cache.rs` と同型のパターンで
//! `#[path]` によりこのファイルをソースレベル共有する（`build.rs` 冒頭の
//! `mod` 宣言参照）。`lib.rs` 側（通常のクレートモジュール）では
//! `cargo test -p fandhe-frontend-dist-server` によるユニットテスト対象とする。
//!
//! `crates/wasm-full/tests/bundle_size.rs` は同一契約の判定を独立実装として
//! 重複させている（テストクレートから本クレートの `#[doc(hidden)]` API を
//! 直接使わない既存方針、当該ファイル冒頭コメント参照）。契約を変更する場合は
//! 両ファイルを揃えて更新すること（#437 で `RWS_WASM_BUILD` →
//! `FANDHE_FRONTEND_WASM_BUILD` へ改名した際も両ファイルを同時更新した）。

/// WASM ビルドステージが有効かどうかを判定する純関数。
///
/// 環境変数の実読み取りを行わないことで、環境変数のミューテーションを伴わない
/// 決定的なユニットテスト（`None`＝未設定・`Some("0")` 等）を可能にする
/// （呼び出し元は `env::var("FANDHE_FRONTEND_WASM_BUILD").ok().as_deref()` を渡す）。
///
/// 既定（`None`）は有効。`0`・`skip`・`false`（大文字小文字を区別しない）の
/// いずれかを設定した場合のみ無効化する。wasm ツールチェーン未整備環境
/// （Docker ビルダーステージ・一部 CI ジョブ）向けの明示オプトアウト
/// （設計 4.4 節。既定は統合ビルド有効という「安全側」を保つため、無効化は
/// 明示的な合言葉を要求する）。
pub fn wasm_build_enabled_for(env_value: Option<&str>) -> bool {
    match env_value {
        Some(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            !(normalized == "0" || normalized == "skip" || normalized == "false")
        }
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::wasm_build_enabled_for;

    /// 未設定（新名 `FANDHE_FRONTEND_WASM_BUILD` を一切指定しない状態）は
    /// 既定で有効（安全側）であることを固定する回帰テスト（#437、旧名
    /// `RWS_WASM_BUILD` は新コードのどこからも参照されずフェイルオープンに
    /// ならないことの保証）。
    #[test]
    fn unset_defaults_to_enabled() {
        assert!(wasm_build_enabled_for(None));
    }

    #[test]
    fn explicit_disable_values_disable_the_stage() {
        for value in ["0", "skip", "false", "SKIP", "FALSE"] {
            assert!(
                !wasm_build_enabled_for(Some(value)),
                "expected {value:?} to disable the wasm build stage"
            );
        }
    }

    #[test]
    fn other_values_keep_the_stage_enabled() {
        for value in ["1", "true", "yes", ""] {
            assert!(
                wasm_build_enabled_for(Some(value)),
                "expected {value:?} to keep the wasm build stage enabled"
            );
        }
    }
}
