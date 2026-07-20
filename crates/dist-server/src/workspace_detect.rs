//! ワークスペース内ビルドかパッケージ単体ビルド（`cargo publish`/`cargo package`
//! の tarball 検証、crates.io からの利用者ビルド等、ワークスペース外での
//! ビルド）かを判定する純粋関数。
//!
//! `build.rs` 自身はパッケージ自身の lib を `build-dependencies` にできない
//! （循環依存）ため、`src/wasm_stage_cache.rs`・`src/wasm_build_gate.rs` と
//! 同型のパターンで `#[path]` によりこのファイルをソースレベル共有する
//! （`build.rs` 冒頭の `mod` 宣言参照）。`lib.rs` 側（通常のクレートモジュール）
//! では `cargo test -p fandhe-frontend-dist-server` によるユニットテスト対象と
//! する。

/// `workspace_root` 候補ディレクトリがワークスペースルートとして実在するかを、
/// 2 つの構造的事実の積で判定する純関数。
///
/// # 判定根拠（環境変数のアドホックな抑制フラグより構造的判定を優先する）
///
/// 1. ルート `Cargo.toml`（`root_cargo_toml`）の内容が `[workspace]` テーブルを
///    持つこと
/// 2. そのワークスペースが本クレート自身を `crates/dist-server/Cargo.toml` として
///    実体で含むこと（`dist_server_manifest_exists`）。1 だけでは
///    `CARGO_MANIFEST_DIR` から機械的に 2 段上がった先がたまたま無関係な
///    `[workspace]` を名乗るディレクトリだった場合の誤検知を防げないため、
///    「本クレートを実際に含む」という往復確認を必須にする
///
/// `cargo publish -p fandhe-frontend-dist-server --dry-run`・`cargo package` の
/// tarball 検証・crates.io からの利用者ビルドでは、パッケージは
/// `target/package/<name>-<version>/` のようなワークスペース外の一時
/// ディレクトリに単体展開されてビルドされる。呼び出し元
/// （[`crate::build`] 相当、実体は `build.rs`）が `CARGO_MANIFEST_DIR` から
/// 機械的に 2 段上がった先には、通常 `Cargo.toml` が存在しないか、存在しても
/// 上記いずれかの条件を満たさない。このいずれの場合も `false` を返し、
/// 呼び出し元は WASM ビルドステージ（Cargo.lock 読み取り・ネスト
/// `cargo build`・`wasm-bindgen` 実行）全体を静かにスキップする契約とする
/// （`wasm_assets_embedded` cfg は立たない）。
pub fn is_workspace_root(root_cargo_toml: Option<&str>, dist_server_manifest_exists: bool) -> bool {
    let Some(content) = root_cargo_toml else {
        return false;
    };
    content.contains("[workspace]") && dist_server_manifest_exists
}

#[cfg(test)]
mod tests {
    use super::is_workspace_root;

    /// ワークスペースルート Cargo.toml が読めない（存在しない）場合は、
    /// 内容を検証するまでもなく非ワークスペースと判定する
    /// （`cargo publish` の tarball 検証で `target/` 配下に展開された場合の
    /// 典型ケース）。
    #[test]
    fn missing_root_cargo_toml_is_not_workspace() {
        assert!(!is_workspace_root(None, true));
    }

    /// ルート Cargo.toml は存在するが `[workspace]` テーブルを持たない
    /// （＝単体クレートの Cargo.toml）場合は非ワークスペースと判定する。
    #[test]
    fn root_cargo_toml_without_workspace_table_is_not_workspace() {
        let content = "[package]\nname = \"dist-server\"\nversion = \"0.1.0\"\n";
        assert!(!is_workspace_root(Some(content), true));
    }

    /// `[workspace]` テーブルを持っていても、本クレート自身
    /// （`crates/dist-server/Cargo.toml`）が実在しなければ非ワークスペースと
    /// 判定する（機械的に 2 段上がった先がたまたま無関係な `[workspace]` を
    /// 名乗るディレクトリだった場合の誤検知防止）。
    #[test]
    fn workspace_table_without_dist_server_member_is_not_workspace() {
        let content = "[workspace]\nmembers = [\"crates/*\"]\n";
        assert!(!is_workspace_root(Some(content), false));
    }

    /// 両条件が揃って初めてワークスペース内ビルドと判定する
    /// （従来の WASM 埋め込み経路を維持する唯一のケース）。
    #[test]
    fn workspace_table_with_dist_server_member_is_workspace() {
        let content = "[workspace]\nmembers = [\"crates/*\"]\n";
        assert!(is_workspace_root(Some(content), true));
    }
}
