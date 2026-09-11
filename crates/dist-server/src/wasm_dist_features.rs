//! `fandhe-frontend-dist-server` が配布する `fandhe-frontend-wasm-full`
//! の feature 集合（イシュー #2329）。
//!
//! `build.rs`（ネスト `cargo build -p fandhe-frontend-wasm-full`）と
//! `crates/wasm-full/tests/bundle_size.rs`（REQ-11 gzip サイズ計測）の
//! **両方がこのファイルを唯一の正として参照する**。手書きで
//! `--no-default-features`/`--features` 引数を複製すると「計測だけ縮小」
//! 「配布物だけ縮小」のどちらかに乖離しゲートが形骸化しうるため
//! （`docs/design/wasm-full-feature-gating-evaluation.md` §14）、
//! `crates/xtask/tests/wasm_dist_features_contract.rs` が両消費者の
//! ソースを機械走査し、本ファイルの `nested_cargo_feature_args` を
//! 実際に呼んでいること・手書きの `"--features"` リテラルが存在しない
//! ことを fail-closed に固定する。
//!
//! `build.rs` 自身はパッケージ自身の lib を `build-dependencies` に
//! できない（循環依存）ため、`wasm_stage_cache`・`wasm_build_gate`・
//! `workspace_detect` と同型のパターンで `#[path]` によりこのファイルを
//! ソースレベル共有する。`bundle_size.rs`（`fandhe-frontend-wasm-full` の
//! 統合テスト）も同じ `#[path]` 手法で取り込む: Cargo 依存としては
//! `dist-server` に依存しない設計を保ったまま（`cargo publish --dry-run`
//! の tarball 検証は lib/bin のみをコンパイルしテストは対象外のため、
//! wasm-full 側のパッケージ検証にも影響しない）コンパイル時にソースを
//! 共有する。
//!
//! # 「最小インタラクティブコンポーネント」集合の判断根拠
//!
//! 採用する集合（`WASM_DIST_FEATURES`）は次の 6 件:
//! `wasm-bindgen-exports` / `collapsible` / `dialog` / `popover` /
//! `tooltip` / `position`。
//!
//! - REQ-11 本文が定義するワークロード（カウンター・フォーム入力・
//!   動的リスト更新相当）は、常時配線される `events::wire_events`
//!   （`data-action` の click/input/change 委譲）と束縛点更新のみで
//!   成立し、scope feature を 1 つも要求しない（理論下限は
//!   `wasm-bindgen-exports` のみ）。
//! - 本 issue が出発点とする「button / input / dialog 系」に、
//!   クリック操作のみで機能が完結する disclosure / overlay 部品を
//!   加える。`crates/wasm-full/src/keynav.rs` に `feature = "collapsible"`
//!   / `"dialog"` / `"popover"` / `"tooltip"` の cfg 分岐は存在しない
//!   （= これら 4 scope は keynav off でも WAI-ARIA 上のキーボード操作が
//!   欠けない）。他の scope feature（accordion / calendar / combobox /
//!   listbox / menu / menubar / navigation-menu / radio-group / select /
//!   tabs / toggle-group / tree-view）は keynav の match arm を持つため
//!   除外し、「click 行だけを配布して keyboard 操作を欠いた部品」を
//!   最小構成として出荷しない。
//! - `keynav` 自体・`focus-visible` は除外する（評価文書 §5 で単体
//!   14〜28 KB を占める支配的なサイズレバー）。
//! - `position` は popover / tooltip の表示位置決め
//!   （`headless::wire_headless_component` 内の
//!   `ensure_global_controller`/`reposition_within`、イシュー #2209）に
//!   必要なため含める。
//! - 配線群別 feature（avatar / clipboard / timer / angle-slider /
//!   splitter / signature-pad / number-input / command / sidebar /
//!   chart / chart-range / questionnaire）は対象外。
//!
//! 集合を変更する場合は `crates/wasm-full/tests/bundle_size.rs` の
//! `bundle-size:` 1 行サマリ実測値を PR 本文に記録すること
//! （`.claude/rules/ci.md` 参照）。

/// dist-server が配布する `fandhe-frontend-wasm-full` の feature 集合
/// （順序固定）。全要素は `crates/wasm-full/Cargo.toml` の `[features]`
/// `default` 配列に含まれる（`crates/xtask/tests/wasm_dist_features_contract.rs`
/// が機械検証する）。
pub const WASM_DIST_FEATURES: &[&str] = &[
    "wasm-bindgen-exports",
    "collapsible",
    "dialog",
    "popover",
    "tooltip",
    "position",
];

/// ネスト `cargo build -p fandhe-frontend-wasm-full` / feature 検証ビルド
/// へ渡す `--no-default-features --features <集合>` の 3 引数を生成する。
///
/// [`WASM_DIST_FEATURES`] を必ず経由することで、呼び出し元が手書きで
/// `--features` 文字列を複製する経路を作らない（本モジュール冒頭の
/// 契約参照）。
pub fn nested_cargo_feature_args() -> [String; 3] {
    [
        "--no-default-features".to_string(),
        "--features".to_string(),
        WASM_DIST_FEATURES.join(","),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // パス非依存の純粋不変条件のみを検証する（このファイルは
    // `crates/wasm-full/tests/bundle_size.rs` からも `#[path]` で
    // 取り込まれるため、`CARGO_MANIFEST_DIR` に依存する処理をここへ
    // 置かない）。

    #[test]
    fn includes_wasm_bindgen_exports() {
        assert!(WASM_DIST_FEATURES.contains(&"wasm-bindgen-exports"));
    }

    #[test]
    fn has_no_duplicates() {
        let mut sorted = WASM_DIST_FEATURES.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), WASM_DIST_FEATURES.len());
    }

    #[test]
    fn excludes_perf_assert() {
        assert!(!WASM_DIST_FEATURES.contains(&"perf-assert"));
    }

    #[test]
    fn has_no_empty_entries() {
        assert!(WASM_DIST_FEATURES.iter().all(|f| !f.is_empty()));
    }

    #[test]
    fn nested_cargo_feature_args_matches_fixed_shape() {
        let args = nested_cargo_feature_args();
        assert_eq!(args[0], "--no-default-features");
        assert_eq!(args[1], "--features");
        assert_eq!(
            args[2],
            "wasm-bindgen-exports,collapsible,dialog,popover,tooltip,position"
        );
    }
}
