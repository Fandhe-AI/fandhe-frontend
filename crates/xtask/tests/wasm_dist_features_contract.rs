//! イシュー #2329「dist-server 配布物の feature 集合を最小インタラクティブ
//! 構成へ縮小し bundle_size.rs の計測を同一構成へ更新する」の fail-closed
//! ドリフト検知テスト。
//!
//! `crates/dist-server/build.rs`（ネスト `cargo build -p
//! fandhe-frontend-wasm-full` の feature 引数）と
//! `crates/wasm-full/tests/bundle_size.rs`（REQ-11 gzip サイズ計測）は
//! `crates/dist-server/src/wasm_dist_features.rs` を `#[path]` で共有し、
//! `wasm_dist_features::WASM_DIST_FEATURES` を唯一の正として参照する契約
//! （同ファイル冒頭コメント参照）を、本テストが 2 つの観点で機械固定する:
//!
//! 1. 両消費者が実際に `#[path]` 取り込み + `nested_cargo_feature_args()`
//!    呼び出しを行っており、`"--no-default-features"`/`"--features"` の
//!    文字列リテラルを手書きで複製していないこと（「計測だけ縮小」
//!    「配布物だけ縮小」の両方を構造的に禁止する）。
//! 2. `WASM_DIST_FEATURES` の内容自体が想定どおりであること
//!    （`wasm-bindgen-exports` を含む・重複なし・`perf-assert` を含まない・
//!    全要素が `crates/wasm-full/Cargo.toml` の `default` に含まれる・
//!    `wasm-bindgen-exports`/`position` 以外の要素は `keynav.rs` に
//!    cfg 分岐を持たない ＝ click 操作のみで完結する）。
//!
//! 外部 TOML パーサは使わず（REQ-3）、`workflow_wasm_full_feature_matrix.rs`
//! と同型の行ベース文字列走査で `Cargo.toml`/`keynav.rs` を読む。

use std::path::PathBuf;

#[path = "../../dist-server/src/wasm_dist_features.rs"]
mod wasm_dist_features;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/xtask/ から 2 段上でワークスペースルートに到達する（イシュー #436）")
        .to_path_buf()
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(workspace_root().join(rel))
        .unwrap_or_else(|e| panic!("{rel} の読み込みに失敗した: {e}"))
}

/// `crates/wasm-full/Cargo.toml` の `[features]` `default = [...]` 配列に
/// 列挙された feature 名の集合を返す（`workflow_wasm_full_feature_matrix.rs
/// ::wasm_full_feature_names` の default 抽出部分と同型のロジック）。
fn wasm_full_default_features(cargo_toml: &str) -> Vec<String> {
    let marker = "\n[features]\n";
    let start = cargo_toml
        .find(marker)
        .expect("crates/wasm-full/Cargo.toml に `[features]` セクションが見つからない");
    let section = &cargo_toml[start + marker.len()..];

    let mut names = Vec::new();
    let mut in_default_array = false;
    for line in section.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            break;
        }
        if in_default_array {
            if trimmed.starts_with(']') {
                break;
            }
            // 行末コメント（`"foo", # comment` 形式）を先に切り落としてから
            // 引用符・カンマ・空白を剥がす。行頭コメント（`# ...`）はこの
            // 時点で空文字列になり自然に除外される。
            let without_comment = trimmed.split('#').next().unwrap_or("");
            let name =
                without_comment.trim_matches(|c: char| c == ',' || c == '"' || c.is_whitespace());
            if !name.is_empty() {
                names.push(name.to_string());
            }
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("default") {
            let rest = rest.trim_start();
            if rest.starts_with('=') {
                in_default_array = true;
            }
        }
    }
    names
}

#[test]
fn build_rs_and_bundle_size_share_the_single_definition_without_hand_duplication() {
    let build_rs = read("crates/dist-server/build.rs");
    let bundle_size = read("crates/wasm-full/tests/bundle_size.rs");

    for (label, contents) in [("build.rs", &build_rs), ("bundle_size.rs", &bundle_size)] {
        // rustdoc 中の言及（`wasm_dist_features` という語や
        // `src/wasm_dist_features.rs` というパス文字列）だけでは満たされない
        // よう、`#[path = "...wasm_dist_features.rs"]` の属性値そのものと
        // 直後の `mod wasm_dist_features;` 宣言の双方を要求する（`#[path]`
        // 行が消えても rustdoc だけ残っていればコンパイルは失敗するが、
        // 本テストがその削除を検知できないと fail-closed の意味がない）。
        assert!(
            contents.contains("wasm_dist_features.rs\"]"),
            "{label} に `#[path = \"...wasm_dist_features.rs\"]` 属性が見つからない"
        );
        assert!(
            contents.contains("mod wasm_dist_features;"),
            "{label} に `mod wasm_dist_features;` 宣言が見つからない"
        );
        assert!(
            contents.contains("nested_cargo_feature_args()"),
            "{label} が `wasm_dist_features::nested_cargo_feature_args()` を呼んでいない"
        );
        assert!(
            !contents.contains("\"--no-default-features\""),
            "{label} に手書きの `--no-default-features` リテラルが存在する（単一定義経由に統一すること）"
        );
        assert!(
            !contents.contains("\"--features\""),
            "{label} に手書きの `--features` リテラルが存在する（単一定義経由に統一すること）"
        );
    }
}

#[test]
fn wasm_dist_features_includes_wasm_bindgen_exports_and_has_no_duplicates_or_perf_assert() {
    let features = wasm_dist_features::WASM_DIST_FEATURES;
    assert!(features.contains(&"wasm-bindgen-exports"));
    assert!(!features.contains(&"perf-assert"));
    assert!(features.iter().all(|f| !f.is_empty()));

    let mut sorted = features.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        features.len(),
        "WASM_DIST_FEATURES に重複がある"
    );
}

#[test]
fn wasm_dist_features_are_all_declared_in_wasm_full_default() {
    let cargo_toml = read("crates/wasm-full/Cargo.toml");
    let default_features = wasm_full_default_features(&cargo_toml);

    for feature in wasm_dist_features::WASM_DIST_FEATURES {
        assert!(
            default_features.iter().any(|f| f == feature),
            "feature `{feature}` が crates/wasm-full/Cargo.toml の [features] default に存在しない"
        );
    }
}

#[test]
fn wasm_dist_features_other_than_wasm_bindgen_exports_and_position_are_click_complete() {
    // `keynav.rs` に `feature = "<name>"` の cfg 分岐が存在する scope は
    // キーボード操作を keynav が担う（click だけでは操作が完結しない）ため、
    // 最小インタラクティブ構成の対象外とする（本モジュール冒頭コメント
    // 「判断根拠」節参照）。この不変条件が破られた場合（keynav.rs へ
    // 新たに当該 feature の cfg 分岐が追加された場合）はテストを fail させ、
    // 集合の再検討を促す。
    let keynav_rs = read("crates/wasm-full/src/keynav.rs");

    for feature in wasm_dist_features::WASM_DIST_FEATURES {
        if *feature == "wasm-bindgen-exports" || *feature == "position" {
            continue;
        }
        let needle = format!("feature = \"{feature}\"");
        assert!(
            !keynav_rs.contains(&needle),
            "feature `{feature}` は crates/wasm-full/src/keynav.rs に cfg 分岐を持つため、\
             click 操作のみで完結する前提が崩れている。最小構成集合の再検討が必要（イシュー #2329）"
        );
    }
}
