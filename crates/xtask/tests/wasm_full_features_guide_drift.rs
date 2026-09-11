//! イシュー #2330「wasm-full feature 一覧・移行手順・dist-server 最小
//! 構成の定義を利用者向けドキュメントと examples へ反映する」の
//! fail-closed ドリフト検知テスト。
//!
//! `docs/guides/wasm-full-features.md` は
//! `crates/wasm-full/src/lib.rs` クレート doc（一次情報）を利用者向けに
//! 再構成した文書であり、手動転記のため一次情報から乖離しうる。本テストは
//! 次の 2 点を機械固定する:
//!
//! 1. ガイド §7 移行手順コード例（`省略できます）。` の直後の fenced code
//!    block）にある `features = [...]` の列挙が、
//!    `crates/wasm-full/Cargo.toml` の `[features]` `default` 配列と
//!    順序込みで完全一致すること（`wasm_dist_features_contract.rs`
//!    ::wasm_full_default_features と同型の行ベース抽出を独立実装する）。
//! 2. ガイド §8 冒頭の `WASM_DIST_FEATURES` への言及直後にある fenced
//!    code block のカンマ区切り feature 列挙が、
//!    `crates/dist-server/src/wasm_dist_features.rs` の
//!    `WASM_DIST_FEATURES` と順序込みで完全一致すること。
//!
//! HTML コメントのマーカー（`<!-- ... -->`）は使わない: このリポジトリの
//! docs サイト Markdown レンダラ（`crate::markdown`、外部依存ゼロ・REQ-1
//! 既定エスケープ）は HTML コメントを特別扱いせず、既定エスケープ経路で
//! `&lt;!-- ... --&gt;` として本文にそのまま出力してしまう（実際に
//! ビルドして確認済み）。そのため本テストはガイド内の一意な文字列
//! （fenced code block の先頭行・直前の言及）を目印に fail-closed
//! （見つからなければ panic）で該当ブロックを特定する。
//!
//! 外部 TOML パーサは使わず（REQ-3）、行ベース文字列走査のみで完結する。

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
/// 列挙された feature 名を順序込みで返す
/// （`wasm_dist_features_contract.rs::wasm_full_default_features` と
/// 同型の行ベース抽出を、テストクレート間の依存を避けるため独立実装する）。
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
    assert!(
        !names.is_empty(),
        "crates/wasm-full/Cargo.toml の `default` 配列が空、または抽出に失敗した"
    );
    names
}

/// `anchor` の出現位置から本文末尾までを返す（見つからなければ fail-closed
/// に panic する）。
fn slice_from_anchor<'a>(haystack: &'a str, anchor: &str) -> &'a str {
    let start = haystack
        .find(anchor)
        .unwrap_or_else(|| panic!("目印 `{anchor}` がガイド内に見つからない"));
    &haystack[start..]
}

/// `after` 以降で最初に現れる fenced code block（```` ``` ```` 区切り）の
/// 本文を返す（見つからなければ fail-closed に panic する）。
fn first_fenced_block_after(after: &str) -> &str {
    let fence_start = after
        .find("```")
        .expect("目印の直後に fenced code block（```）が見つからない");
    let body_start = after[fence_start..]
        .find('\n')
        .map(|i| fence_start + i + 1)
        .expect("fenced code block の開始行に改行が見つからない");
    let body_end = after[body_start..]
        .find("```")
        .map(|i| body_start + i)
        .expect("fenced code block の閉じ ``` が見つからない");
    &after[body_start..body_end]
}

/// ガイド内 `features = [...]`（複数行、1 行 1 `"feature-name",`）から
/// feature 名を順序込みで抽出する。
fn extract_guide_feature_array(block: &str) -> Vec<String> {
    let array_marker = "features = [";
    let start = block
        .find(array_marker)
        .expect("ガイドの `[dependencies.fandhe-frontend-wasm-full]` ブロックに `features = [` が見つからない");
    let after = &block[start + array_marker.len()..];
    let end = after
        .find(']')
        .expect("ガイドの `features = [...]` の閉じ `]` が見つからない");
    let body = &after[..end];

    let mut names = Vec::new();
    for line in body.lines() {
        let name = line.trim_matches(|c: char| c == ',' || c == '"' || c.is_whitespace());
        if !name.is_empty() {
            names.push(name.to_string());
        }
    }
    assert!(
        !names.is_empty(),
        "ガイドの `features = [...]` からの feature 名抽出に失敗した"
    );
    names
}

/// ガイド内のカンマ区切り 1 行（`a, b, c` 形式）から feature 名を
/// 順序込みで抽出する。
fn extract_guide_comma_list(block: &str) -> Vec<String> {
    block
        .lines()
        .find(|line| line.contains(','))
        .unwrap_or_else(|| panic!("ガイドの dist-server 最小構成のカンマ区切り一覧が見つからない"))
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[test]
fn guide_default_features_match_wasm_full_cargo_toml() {
    let cargo_toml = read("crates/wasm-full/Cargo.toml");
    let expected = wasm_full_default_features(&cargo_toml);

    let guide = read("docs/guides/wasm-full-features.md");
    // ガイド §7 本文末尾（`省略できます）。`）の直後に移行手順コード例の
    // fenced code block が続く（本文中に他の出現箇所はない一意な文字列）。
    let anchor_slice = slice_from_anchor(&guide, "省略できます）。");
    let block = first_fenced_block_after(anchor_slice);
    let actual = extract_guide_feature_array(block);

    assert_eq!(
        actual, expected,
        "docs/guides/wasm-full-features.md の `features = [...]` 一覧が \
         crates/wasm-full/Cargo.toml の `default` 配列と一致しない（\
         feature 追加・削除時はガイドも更新すること）"
    );
}

#[test]
fn guide_dist_server_minimal_set_matches_wasm_dist_features() {
    let guide = read("docs/guides/wasm-full-features.md");
    // ガイド §8 冒頭が `WASM_DIST_FEATURES` に言及した直後の fenced code
    // block に最小構成のカンマ区切り一覧を掲載している。
    let anchor_slice = slice_from_anchor(&guide, "WASM_DIST_FEATURES");
    let block = first_fenced_block_after(anchor_slice);
    let actual = extract_guide_comma_list(block);

    let expected: Vec<String> = wasm_dist_features::WASM_DIST_FEATURES
        .iter()
        .map(|s| s.to_string())
        .collect();

    assert_eq!(
        actual, expected,
        "docs/guides/wasm-full-features.md の dist-server 最小構成一覧が \
         crates/dist-server/src/wasm_dist_features.rs::WASM_DIST_FEATURES \
         と一致しない"
    );
}
