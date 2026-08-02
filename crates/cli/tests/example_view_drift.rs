//! `examples/interactive-view-transitions` の SSR 側 `src/main.rs` と CSR
//! 側 `wasm/src/lib.rs`（独立ワークスペースの glue クレート）間で、同一
//! マークアップを出力すべきビュー構築コードが重複定義されていることに
//! 起因するドリフトを機械検知するテスト（イシュー #1202）。
//!
//! # 背景
//!
//! 両ファイルは別クレート（別ワークスペース）に属し、`fandhe-frontend-core`
//! の `Node` を組み立てるコードを共有できない（`include!` による共有は
//! SSR/CSR で意図的に形が異なる root 包含 vs content-only の差異と教材性を
//! 損なうため不採用、詳細は本イシューの実装計画を参照）。従来は両ファイル
//! の doc コメントの注意書き（「片方だけ変更するとドリフトする。機械
//! テストはスコープ外」、PR #1200 の out-of-scope）だけが頼りだった。
//!
//! # 検証方式
//!
//! 正本 2 ファイル中に埋め込んだマーカーコメント
//! （`// fw-drift-guard:begin <name>` / `// fw-drift-guard:end <name>`、
//! 各 1 行完結）で囲んだ区間を抽出し、インデント正規化（[`dedent`]）後に
//! 完全一致することを assert する。区間の外（例: SSR 側は
//! `navigation_menu::root(...)`/`state.root(...)` で包み、CSR 側は content
//! ノード列のみを返す）は意図的な差分でありイシュー #1200 の修正内容その
//! ものなので、検証対象に含めない。
//!
//! `crates/cli/embedded-examples/interactive-view-transitions/`（`fw new
//! --example` 用の同梱コピー）は `example_publish_copy_drift.rs` の
//! バイト単位一致検証が既にカバーするため、本テストでは正本
//! （リポジトリルート `examples/`）のみを検証対象とする（二重検証しない）。

use std::path::{Path, PathBuf};

/// [`fandhe-frontend-server`/`fandhe-frontend-app` 等と異なり本クレートは
/// `crates/cli/` 配下でビルドされるため、workspace ルートは 2 段上）
/// `example_publish_copy_drift.rs::workspace_root` と同型のヘルパー。
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/cli/ has a workspace root two levels up")
        .to_path_buf()
}

/// `examples/interactive-view-transitions/src/main.rs`（SSR 側正本）の
/// 絶対パス。
fn main_rs_path() -> PathBuf {
    workspace_root().join("examples/interactive-view-transitions/src/main.rs")
}

/// `examples/interactive-view-transitions/wasm/src/lib.rs`（CSR glue 側
/// 正本）の絶対パス。
fn wasm_lib_rs_path() -> PathBuf {
    workspace_root().join("examples/interactive-view-transitions/wasm/src/lib.rs")
}

/// `source` から `// fw-drift-guard:begin <name>` 〜
/// `// fw-drift-guard:end <name>`（両マーカー行自体は含まない）の間の行を
/// 抽出する。
///
/// fail-closed 契約: begin/end のいずれかが 0 回・2 回以上出現する、
/// begin より end が先に出現する、区間が空（begin/end が隣接行）のいずれ
/// かに該当する場合は `Err` を返す。マーカーの削除・重複・空区間化による
/// サイレントな検証消失を防ぐ（検知漏れを許さない）。
fn extract_region(source: &str, name: &str) -> Result<Vec<String>, String> {
    let begin_marker = format!("fw-drift-guard:begin {name}");
    let end_marker = format!("fw-drift-guard:end {name}");

    let begin_indices: Vec<usize> = source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.trim_start().starts_with("//") && line.contains(&begin_marker))
        .map(|(i, _)| i)
        .collect();
    let end_indices: Vec<usize> = source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.trim_start().starts_with("//") && line.contains(&end_marker))
        .map(|(i, _)| i)
        .collect();

    if begin_indices.len() != 1 {
        return Err(format!(
            "region {name:?}: expected exactly 1 begin marker, found {}",
            begin_indices.len()
        ));
    }
    if end_indices.len() != 1 {
        return Err(format!(
            "region {name:?}: expected exactly 1 end marker, found {}",
            end_indices.len()
        ));
    }

    let begin = begin_indices[0];
    let end = end_indices[0];
    if end <= begin {
        return Err(format!(
            "region {name:?}: end marker (line {end}) must come after begin marker (line {begin})"
        ));
    }

    let lines: Vec<String> = source
        .lines()
        .skip(begin + 1)
        .take(end - begin - 1)
        .map(|s| s.to_string())
        .collect();

    if lines.is_empty() {
        return Err(format!("region {name:?}: region is empty"));
    }

    Ok(lines)
}

/// `lines` から共通の先頭空白（インデント）を剥がして単一文字列へ連結する
/// （SSR 側とグルー側でネスト深さが異なるため、比較前にインデント差を
/// 吸収する）。空行は最小インデント計算から除外する。
fn dedent(lines: &[String]) -> String {
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                line.chars().skip(min_indent).collect::<String>()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `fw-drift-guard` 区間の一致検証対象となる区間名の固定リスト（本テスト
/// における単一の情報源）。
///
/// - `nav-menu-items`: `NAV_MENU_ITEMS` 定数宣言
/// - `menubar-menus`: `MENUBAR_MENUS` 定数宣言
/// - `nav-menu-item-nodes`: navigation-menu の項目ノード列を組み立てる文
/// - `menubar-menu-map`: menubar のメニューノード列を組み立てるクロージャ
const REGION_NAMES: &[&str] = &[
    "nav-menu-items",
    "menubar-menus",
    "nav-menu-item-nodes",
    "menubar-menu-map",
];

/// `examples/interactive-view-transitions` の SSR/CSR 間で `fw-drift-guard`
/// 区間がすべて（インデント正規化後）完全一致することを検証する。
///
/// 片方のファイルのみを変更して区間の内容がドリフトした場合、この
/// テストは FAIL する（受け入れ条件）。
#[test]
fn ssr_csr_view_regions_match() {
    let main_rs = std::fs::read_to_string(main_rs_path()).expect("failed to read src/main.rs");
    let wasm_lib_rs =
        std::fs::read_to_string(wasm_lib_rs_path()).expect("failed to read wasm/src/lib.rs");

    for name in REGION_NAMES {
        let main_region = extract_region(&main_rs, name).unwrap_or_else(|e| {
            panic!(
                "src/main.rs: failed to extract fw-drift-guard region {name:?}: {e}\n\
                 (both src/main.rs and wasm/src/lib.rs must declare exactly one \
                 begin/end marker pair for each region in REGION_NAMES)"
            )
        });
        let wasm_region = extract_region(&wasm_lib_rs, name).unwrap_or_else(|e| {
            panic!(
                "wasm/src/lib.rs: failed to extract fw-drift-guard region {name:?}: {e}\n\
                 (both src/main.rs and wasm/src/lib.rs must declare exactly one \
                 begin/end marker pair for each region in REGION_NAMES)"
            )
        });

        let main_dedented = dedent(&main_region);
        let wasm_dedented = dedent(&wasm_region);

        assert_eq!(
            main_dedented, wasm_dedented,
            "fw-drift-guard region {name:?} has drifted between \
             examples/interactive-view-transitions/src/main.rs and \
             examples/interactive-view-transitions/wasm/src/lib.rs. \
             Both files define this view-construction code independently \
             (separate crates in separate workspaces cannot share code) and \
             must be updated together. Fix: apply the same change to the \
             `// fw-drift-guard:begin {name}` .. `// fw-drift-guard:end {name}` \
             region in both files."
        );
    }
}

// --- fail-closed 実証テスト（合成フィクスチャ）--------------------------
//
// `extract_region`/`dedent` の fail-closed 挙動そのものを検証する。実際の
// example ファイルに触れず、インラインの合成 Rust ソース文字列で「begin
// マーカー欠落」「マーカー重複」「空区間」「内容乖離」の 4 パターンを
// カバーする（受け入れ条件「乖離した場合にテストが FAIL する（fail-closed
// 実証を含む）」）。

#[test]
fn extract_region_detects_content_drift() {
    let a = "// fw-drift-guard:begin x\nfoo\n// fw-drift-guard:end x\n";
    let b = "// fw-drift-guard:begin x\nbar\n// fw-drift-guard:end x\n";
    let ra = extract_region(a, "x").expect("region x should extract from a");
    let rb = extract_region(b, "x").expect("region x should extract from b");
    assert_ne!(dedent(&ra), dedent(&rb));
}

#[test]
fn extract_region_errors_on_missing_begin_marker() {
    let source = "foo\n// fw-drift-guard:end x\n";
    assert!(extract_region(source, "x").is_err());
}

#[test]
fn extract_region_errors_on_missing_end_marker() {
    let source = "// fw-drift-guard:begin x\nfoo\n";
    assert!(extract_region(source, "x").is_err());
}

#[test]
fn extract_region_errors_on_duplicate_begin_marker() {
    let source =
        "// fw-drift-guard:begin x\nfoo\n// fw-drift-guard:begin x\nbar\n// fw-drift-guard:end x\n";
    assert!(extract_region(source, "x").is_err());
}

#[test]
fn extract_region_errors_on_duplicate_end_marker() {
    let source =
        "// fw-drift-guard:begin x\nfoo\n// fw-drift-guard:end x\nbar\n// fw-drift-guard:end x\n";
    assert!(extract_region(source, "x").is_err());
}

#[test]
fn extract_region_errors_on_empty_region() {
    let source = "// fw-drift-guard:begin x\n// fw-drift-guard:end x\n";
    assert!(extract_region(source, "x").is_err());
}

#[test]
fn extract_region_errors_when_end_precedes_begin() {
    let source = "// fw-drift-guard:end x\nfoo\n// fw-drift-guard:begin x\n";
    assert!(extract_region(source, "x").is_err());
}

#[test]
fn dedent_normalizes_indentation_differences() {
    let nested = vec!["        foo".to_string(), "            bar".to_string()];
    let toplevel = vec!["    foo".to_string(), "        bar".to_string()];
    assert_eq!(dedent(&nested), dedent(&toplevel));
}
