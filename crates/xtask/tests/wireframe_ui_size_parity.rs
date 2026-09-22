//! `fandhe-frontend-wireframe-ui::Size` と `fandhe-frontend-pre-styled-ui::recipe::Size`
//! の段階名（variant 名）パリティを固定する回帰テスト（イシュー #2605）。
//!
//! wireframe-ui は headless-ui / pre-styled-ui への依存を持たない独立した
//! 第 3 の UI 層であり（`docs/design/wireframe-ui-architecture.md` §1）、
//! `Size` は両クレートで別々に定義された型である。段階名（`Xs`/`Sm`/`Md`/
//! `Lg`/`Xl`）が将来どちらか一方だけ変更されてドリフトしないよう、
//! `crates/xtask/tests/profile_dev_test_parity.rs` と同じ「外部 TOML/Rust
//! パーサなし・行走査」方式でソースから variant 識別子を抽出し完全一致を
//! 固定する（xtask は REQ-3 外部依存ゼロ方針のため構文解析ライブラリは
//! 導入しない）。
//!
//! **注意**: この sibling ファイル参照は xtask（`publish = false`）側にのみ
//! 置く。wireframe-ui 側のテストに同種の参照を置くと、初回公開（#2668）
//! 時の `cargo package` を壊すため（イシュー #2605 実装計画 §4 Step 2）。

use std::path::PathBuf;

/// `crates/xtask/` から 2 段上でワークスペースルートに到達する（イシュー #436）。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/xtask/ から 2 段上でワークスペースルートに到達する")
        .to_path_buf()
}

/// 指定ファイルから `pub enum Size {` ブロックの variant 識別子を宣言順で
/// 抽出する。`#[default]` 等の属性行・コメント行・空行は無視する。
/// ブロックが見つからない、または variant が 0 件の場合は fail-closed で
/// panic する（呼び出し元が assert 前に検知できるよう明示的にエラーメッセージを持つ）。
fn extract_size_variants(path: &PathBuf) -> Vec<String> {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));

    let lines = content.lines();
    let mut found_header = false;
    let mut depth: i32 = 0;
    let mut variants = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if !found_header {
            // `pub enum Size {`（末尾にトレイト由来の空白・タブ差異があり得るため
            // 前方一致で判定する）。
            if trimmed.starts_with("pub enum Size {") {
                found_header = true;
                depth = 1;
            }
            continue;
        }

        // コメント・属性行はブロック終端判定（`{`/`}` カウント）より先に
        // スキップする。`///` doc コメント等に `{`/`}` を含む文言が
        // 混入しても深さ計算を乱さないための順序（堅牢性）。
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }

        // ブロック終端の判定（ネストした `{`/`}` を数える。variant 自体は
        // 単純な識別子のみで `{`/`}` を含まないため、ここでの深さ計算は
        // 属性・derive 等がブロック内に現れない本ファイル群の実態に対して
        // 十分である）。
        depth += trimmed.matches('{').count() as i32;
        depth -= trimmed.matches('}').count() as i32;
        if depth <= 0 {
            break;
        }

        // `Xs,` / `Xs` のように末尾カンマの有無を許容し、識別子部分のみを取る。
        let ident = trimmed.trim_end_matches(',').trim();
        if !ident.is_empty() && ident.chars().next().is_some_and(|c| c.is_uppercase()) {
            variants.push(ident.to_string());
        }
    }

    assert!(
        found_header,
        "`pub enum Size {{` ブロックが見つからない: {}",
        path.display()
    );
    assert!(
        !variants.is_empty(),
        "Size の variant が 0 件: {}",
        path.display()
    );
    variants
}

#[test]
fn wireframe_ui_size_matches_pre_styled_ui_size_variant_names() {
    let root = workspace_root();
    let pre_styled_ui_path = root.join("crates/pre-styled-ui/src/recipe.rs");
    let wireframe_ui_path = root.join("crates/wireframe-ui/src/size.rs");

    let pre_styled_ui_variants = extract_size_variants(&pre_styled_ui_path);
    let wireframe_ui_variants = extract_size_variants(&wireframe_ui_path);

    assert_eq!(
        wireframe_ui_variants, pre_styled_ui_variants,
        "wireframe-ui::Size と pre-styled-ui::recipe::Size の variant 名・順序が一致しない"
    );
    assert_eq!(
        wireframe_ui_variants,
        vec!["Xs", "Sm", "Md", "Lg", "Xl"],
        "Size の段階名が想定（Xs/Sm/Md/Lg/Xl）と異なる"
    );
}
