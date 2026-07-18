//! `fw structure` の JSON 出力（TASK-13.1c, #130）。
//!
//! REQ-13 受け入れ基準 1「ディレクトリ規約・ルート定義・コンポーネント境界・
//! 依存の 4 要素が人手の目視解釈に依存せずツール出力（構造化データ）として
//! 列挙できること」を満たす stdout 出力を組み立てる。`serde_json` 等の外部
//! クレートを使わず、`xtask/src/json.rs`（読み取り専用パーサ）とは独立に、
//! 手書きの最小シリアライザとして文字列を組み立てる（`cli` 外部依存ゼロ方針）。
//!
//! マニフェスト値（`description` 等）は利用者の `structure.toml` に由来する
//! ため、JSON 文字列として出力する際は必ず [`escape_str`] を通す
//! （security.md A08: JSON インジェクション対策。HTML 文字列直接組み立て禁止の
//! 精神をここでも踏襲し、値をそのまま埋め込まない）。

use crate::component_boundary::PublicSymbol;
use crate::routes::ExtractedRoute;
use crate::structure::StructureManifest;

/// JSON 文字列リテラルとして安全な形にエスケープする。
///
/// 制御文字（`\x00`-`\x1f`）・`"`・`\` をエスケープする。マニフェスト由来の
/// 任意文字列（`description` 等）がそのまま JSON 構造を破壊しないことを保証する
/// 唯一の経路であり、本モジュール内の全出力箇所がこの関数を通る契約とする。
///
/// `pub(crate)`: `gate.rs`（TASK-13.3, #138）が `cargo check`/`cargo clippy` 等の
/// 任意バイト列を含み得るコマンド出力を JSON へ埋め込む際にも同じ経路を再利用し、
/// JSON エスケープ実装を二重管理しない契約とする。
pub(crate) fn escape_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// `pub(crate)`: `gate.rs` の JSON レポート組み立てからも再利用する（上記 `escape_str` と同じ理由）。
pub(crate) fn quoted(s: &str) -> String {
    format!("\"{}\"", escape_str(s))
}

/// `pub(crate)`: `gate.rs` のチェック名一覧等、文字列配列を出力する箇所から再利用する。
pub(crate) fn string_array(items: &[String]) -> String {
    let parts: Vec<String> = items.iter().map(|s| quoted(s)).collect();
    format!("[{}]", parts.join(","))
}

/// 数値配列を JSON 配列として出力する（クォートしない）。
///
/// `pub(crate)`: `impact.rs`（TASK-13.2d, #136）の `render_report` が
/// `AffectedFile::lines`（1 始まりの行番号一覧）を出力する際に使う。
/// 文字列を経由しないため `escape_str` は不要（数値は JSON インジェクションの
/// 対象にならない）。
pub(crate) fn usize_array(items: &[usize]) -> String {
    let parts: Vec<String> = items.iter().map(usize::to_string).collect();
    format!("[{}]", parts.join(","))
}

/// `fw structure` が JSON 出力に含める 4 要素と、その生成に使った素材。
pub struct StructureOutput<'a> {
    pub manifest: &'a StructureManifest,
    /// (ディレクトリ名, ルート一覧)。`[routing]` を宣言していない場合は空。
    pub routes: Vec<(String, Vec<ExtractedRoute>)>,
    /// (ディレクトリ名, 公開シンボル一覧)。`role = "component"` のディレクトリのみ対象。
    pub component_boundary: Vec<(String, Vec<PublicSymbol>)>,
    /// (ディレクトリ名, 実体の normal workspace 依存名一覧)。`cargo metadata` から得た実測値。
    pub dependencies: Vec<(String, Vec<String>)>,
    /// `resolve.nodes` に現れた解決済みパッケージ総数（workspace member + 外部依存）。
    pub resolved_package_count: usize,
}

/// 4 要素（directories / routes / component_boundary / dependencies）を含む
/// JSON オブジェクトを 1 行で出力する（`fw structure` の stdout。パイプ処理・
/// 他ツールからの読み込みを想定し pretty-print はしない）。
pub fn render(output: &StructureOutput<'_>) -> String {
    let mut buf = String::new();
    buf.push('{');

    buf.push_str("\"directories\":[");
    for (i, dir) in output.manifest.directories.iter().enumerate() {
        if i > 0 {
            buf.push(',');
        }
        buf.push('{');
        buf.push_str("\"name\":");
        buf.push_str(&quoted(&dir.name));
        buf.push_str(",\"role\":");
        buf.push_str(&quoted(dir.role.as_str()));
        buf.push_str(",\"crate\":");
        match &dir.crate_name {
            Some(c) => buf.push_str(&quoted(c)),
            None => buf.push_str("null"),
        }
        buf.push_str(",\"description\":");
        buf.push_str(&quoted(&dir.description));
        buf.push_str(",\"depends_on\":");
        buf.push_str(&string_array(&dir.depends_on));
        buf.push_str(",\"allowed_dependents\":");
        buf.push_str(&string_array(&dir.allowed_dependents));
        buf.push('}');
    }
    buf.push(']');

    buf.push_str(",\"routes\":[");
    let mut first = true;
    for (dir_name, routes) in &output.routes {
        for route in routes {
            if !first {
                buf.push(',');
            }
            first = false;
            buf.push('{');
            buf.push_str("\"directory\":");
            buf.push_str(&quoted(dir_name));
            buf.push_str(",\"path\":");
            buf.push_str(&quoted(&route.path));
            buf.push_str(",\"handler\":");
            buf.push_str(&quoted(&route.handler));
            buf.push('}');
        }
    }
    buf.push(']');

    buf.push_str(",\"component_boundary\":[");
    let mut first = true;
    for (dir_name, symbols) in &output.component_boundary {
        for symbol in symbols {
            if !first {
                buf.push(',');
            }
            first = false;
            buf.push('{');
            buf.push_str("\"directory\":");
            buf.push_str(&quoted(dir_name));
            buf.push_str(",\"kind\":");
            buf.push_str(&quoted(symbol.kind.as_str()));
            buf.push_str(",\"name\":");
            buf.push_str(&quoted(&symbol.name));
            buf.push('}');
        }
    }
    buf.push(']');

    buf.push_str(",\"dependencies\":{");
    buf.push_str("\"resolved_package_count\":");
    buf.push_str(&output.resolved_package_count.to_string());
    buf.push_str(",\"workspace_edges\":[");
    for (i, (dir_name, deps)) in output.dependencies.iter().enumerate() {
        if i > 0 {
            buf.push(',');
        }
        buf.push('{');
        buf.push_str("\"directory\":");
        buf.push_str(&quoted(dir_name));
        buf.push_str(",\"depends_on\":");
        buf.push_str(&string_array(deps));
        buf.push('}');
    }
    buf.push_str("]}");

    buf.push('}');
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_str_handles_quotes_backslashes_and_control_chars() {
        assert_eq!(escape_str("a\"b\\c\nd"), "a\\\"b\\\\c\\nd");
        assert_eq!(escape_str("\u{0007}"), "\\u0007");
    }

    #[test]
    fn quoted_wraps_escaped_value_in_double_quotes() {
        assert_eq!(quoted("a\"b"), "\"a\\\"b\"");
    }

    #[test]
    fn string_array_renders_comma_separated_quoted_items() {
        assert_eq!(
            string_array(&["a".to_string(), "b\"c".to_string()]),
            "[\"a\",\"b\\\"c\"]"
        );
    }

    #[test]
    fn usize_array_renders_comma_separated_unquoted_numbers() {
        assert_eq!(usize_array(&[1, 2, 10]), "[1,2,10]");
        assert_eq!(usize_array(&[]), "[]");
    }
}
