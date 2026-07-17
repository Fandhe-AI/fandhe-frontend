//! コンポーネント境界抽出（TASK-13.1c, #130、REQ-13 受け入れ基準 1 の 4 要素目）。
//!
//! `role = "component"` のディレクトリ（本リポジトリでは `app/src`）配下の
//! トップレベル公開シンボル（`pub fn` / `pub struct` / `pub enum` / `pub const`）を
//! 文字列走査で列挙する。PoC-7（`docs/spec/03-poc/ai-self-maintenance/tools/poc7_tool.py`）
//! の `component_boundary` 抽出と同等の粒度（トップレベルのみ・AST 精密化はスコープ外、
//! `docs/structure-manifest.md` §5）を Rust で再実装したもの。
//!
//! パストラバーサル対策は [`crate::routes::resolve_within_root`] を共有する
//! （走査対象ディレクトリ名は `structure.toml` の検証済み `directories` キーのみ）。

use crate::routes::{self, ExtractError};
use std::path::Path;

/// 抽出したトップレベル公開シンボル 1 件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicSymbol {
    pub kind: SymbolKind,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Fn,
    Struct,
    Enum,
    Const,
}

impl SymbolKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            SymbolKind::Fn => "fn",
            SymbolKind::Struct => "struct",
            SymbolKind::Enum => "enum",
            SymbolKind::Const => "const",
        }
    }
}

/// `workspace_root / dir_name` 配下の `.rs` ファイルを走査し、行頭（インデントなし）
/// から始まる `pub fn|struct|enum|const <name>` 宣言を列挙する。
///
/// インデントされた（モジュール内・impl 内の）`pub` 宣言は「トップレベル」の
/// コンポーネント境界とはみなさず対象外とする（PoC-7 と同じ粒度）。
///
/// # Errors
///
/// 走査対象がワークスペースルート外を指す場合・存在しない場合・I/O に失敗した場合に
/// [`ExtractError`] を返す。
pub fn extract_public_symbols(
    workspace_root: &Path,
    dir_name: &str,
) -> Result<Vec<PublicSymbol>, ExtractError> {
    let target = routes::scan_root(workspace_root, dir_name)?;
    let mut symbols = Vec::new();
    for file in routes::list_rs_files(&target)? {
        let content = std::fs::read_to_string(&file)
            .map_err(|e| ExtractError::Io(format!("{:?}", e.kind())))?;
        symbols.extend(extract_from_source(&content));
    }
    Ok(symbols)
}

/// ソース文字列からトップレベル `pub` 宣言を抽出する（行頭一致・正規表現不使用）。
fn extract_from_source(content: &str) -> Vec<PublicSymbol> {
    const PREFIXES: &[(&str, SymbolKind)] = &[
        ("pub fn ", SymbolKind::Fn),
        ("pub struct ", SymbolKind::Struct),
        ("pub enum ", SymbolKind::Enum),
        ("pub const ", SymbolKind::Const),
    ];
    let mut symbols = Vec::new();
    for line in content.lines() {
        // インデントなし（行頭が空白でない）の宣言のみをトップレベルとみなす。
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }
        for (prefix, kind) in PREFIXES {
            if let Some(rest) = line.strip_prefix(prefix) {
                if let Some(name) = extract_identifier(rest) {
                    symbols.push(PublicSymbol { kind: *kind, name });
                }
                break;
            }
        }
    }
    symbols
}

/// 識別子（英数字・`_`）の先頭部分を取り出す。ジェネリクス・引数リスト等の
/// 後続トークンは無視する。
fn extract_identifier(rest: &str) -> Option<String> {
    let ident: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if ident.is_empty() {
        None
    } else {
        Some(ident)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_top_level_public_declarations_only() {
        let src = r#"
pub const LIKE_BUTTON_ID: &str = "like-btn";

pub struct Item {
    pub id: u32,
}

pub fn demo_items() -> Vec<Item> {
    Vec::new()
}

pub enum Mode {
    Ssr,
}

mod inner {
    pub fn nested_helper() {}
}
"#;
        let symbols = extract_from_source(src);
        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["LIKE_BUTTON_ID", "Item", "demo_items", "Mode"]);
        // インデントされた `pub fn nested_helper` はトップレベルではないため含まれない。
        assert!(!names.contains(&"nested_helper"));
    }

    #[test]
    fn extract_public_symbols_reads_real_app_crate() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("cli/ has a parent workspace root");
        let symbols = extract_public_symbols(workspace_root, "app").expect("scan should succeed");
        assert!(symbols.iter().any(|s| s.name == "demo_items"));
        assert!(symbols.iter().any(|s| s.name == "Item"));
    }
}
