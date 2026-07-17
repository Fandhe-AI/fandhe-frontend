//! `structure.toml`（REQ-13 のマニフェスト情報源）に必要な範囲のみを解釈する
//! 最小 TOML サブセットパーサ。
//!
//! [`structure`](crate::structure) モジュールから呼ばれ、`fw structure` / `fw impact` /
//! `fw gate`（TASK-13.1c 以降）が読む `structure.toml` を型付きモデルへ変換する前段の
//! 汎用パースを担う。`xtask/src/json.rs`（`cargo metadata` 用の外部依存ゼロ JSON パーサ）
//! と同じ契約に従う: `unwrap()` / `expect()` / `panic!` を使わず、すべての失敗を
//! [`TomlError`] として返す。
//!
//! # 対応する構文（サブセット）
//!
//! - `#` 始まりの行コメント（文字列リテラル内の `#` は除く）
//! - テーブルヘッダ `[a.b]`（bare key はハイフンを許容。`[[a]]` 形式の
//!   array-of-tables は非対応）
//! - `key = "basic string"`（`\"` `\\` `\n` `\t` のエスケープのみ対応）
//! - 文字列配列 `["a", "b"]`（末尾カンマ許容。要素は文字列のみ）
//! - 真偽値 `true` / `false`
//! - 10 進整数（先頭 `-` 許容）
//!
//! 複数行文字列・日時・inline table・array-of-tables 等、上記に含まれない構文は
//! 黙って無視せず明示的に [`TomlError`] を返す（fail-closed）。
//!
//! `structure.toml` は利用者プロジェクト側の入力であり信頼できないため、
//! 入力サイズ・テーブルパスのネスト深さに上限を設けて DoS を抑止する
//! （security.md A03）。
use std::fmt;

/// パース済み TOML の値。`structure.toml` のスキーマに必要な型のみを表現する。
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Integer(i64),
    Bool(bool),
    /// 文字列配列（`structure.toml` の `depends_on` 等で使用）。
    Array(Vec<Value>),
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// 文字列配列として取り出す。要素が 1 つでも文字列でなければ `None`。
    pub fn as_string_array(&self) -> Option<Vec<&str>> {
        match self {
            Value::Array(items) => items.iter().map(Value::as_str).collect(),
            _ => None,
        }
    }
}

/// パース失敗時のエラー。行番号（1 始まり）と理由のみを保持する。
///
/// 機微情報の露出防止（security.md）のため、入力全体の再掲や環境変数等は含めない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TomlError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for TomlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TOML parse error at line {}: {}",
            self.line, self.message
        )
    }
}

impl std::error::Error for TomlError {}

fn err(line: usize, message: impl Into<String>) -> TomlError {
    TomlError {
        line,
        message: message.into(),
    }
}

/// パース入力の最大バイト数（1 MiB）。巨大な `structure.toml` による DoS を抑止する。
const MAX_INPUT_BYTES: usize = 1024 * 1024;

/// テーブルヘッダのドット区切りセグメント数の上限。
const MAX_TABLE_PATH_DEPTH: usize = 16;

/// 1 テーブル分のキー・値の並び（宣言順）。
type Table = Vec<(String, Value)>;

/// テーブルパス（例: `["directories", "core"]`）と、そのテーブルの内容の組。
type TableEntry = (Vec<String>, Table);

/// パース済みドキュメント。テーブルパスごとに宣言順を保持したキー・値の並びを持つ。
///
/// トップレベル（どのテーブルヘッダの前にも書かれたキー）は空パス `[]` に属する。
/// `structure.toml` のスキーマはトップレベルキーを使わないため、
/// [`crate::structure`] 側でトップレベルキーの存在はエラーとして扱う。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Document {
    tables: Vec<TableEntry>,
}

impl Document {
    /// 指定パスのテーブルをキー・値の並び（宣言順）として返す。存在しなければ `None`。
    pub fn table(&self, path: &[&str]) -> Option<&[(String, Value)]> {
        self.tables
            .iter()
            .find(|(p, _)| p.len() == path.len() && p.iter().zip(path).all(|(a, b)| a == b))
            .map(|(_, kvs)| kvs.as_slice())
    }

    /// 宣言済みの全テーブルを宣言順で (パス, キー・値の並び) として返す
    /// （`structure.toml` の `[directories.*]` を列挙する用途）。
    /// パスから再度 [`Document::table`] を引き直す必要がないため、
    /// 呼び出し側で「列挙したパスのテーブルが見つからない」という
    /// 到達不能ケースの `unwrap()`/`expect()` を書かずに済む。
    pub fn entries(&self) -> impl Iterator<Item = (&[String], &[(String, Value)])> {
        self.tables
            .iter()
            .map(|(p, kvs)| (p.as_slice(), kvs.as_slice()))
    }
}

/// `input` を TOML サブセットとしてパースする。
///
/// `structure.toml` の読み込みが唯一の呼び出し経路（[`crate::structure::parse`]）。
pub fn parse(input: &str) -> Result<Document, TomlError> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(err(
            0,
            format!("input exceeds maximum size of {MAX_INPUT_BYTES} bytes"),
        ));
    }

    let mut doc = Document::default();
    let mut current_path: Vec<String> = Vec::new();
    // ルート（トップレベル）テーブルを常に用意しておく。
    doc.tables.push((Vec::new(), Vec::new()));

    for (idx, raw_line) in input.lines().enumerate() {
        let line_no = idx + 1;
        let line = strip_comment(raw_line);
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('[') {
            let path = parse_table_header(trimmed, line_no)?;
            if !doc.tables.iter().any(|(p, _)| *p == path) {
                doc.tables.push((path.clone(), Vec::new()));
            } else {
                return Err(err(
                    line_no,
                    format!("duplicate table header `[{}]`", path.join(".")),
                ));
            }
            current_path = path;
            continue;
        }

        let eq_pos = trimmed.find('=').ok_or_else(|| {
            err(
                line_no,
                "expected `key = value` or `[table]` header".to_string(),
            )
        })?;
        let key = trimmed[..eq_pos].trim();
        let value_raw = trimmed[eq_pos + 1..].trim();
        validate_bare_key(key, line_no)?;
        let value = parse_value(value_raw, line_no)?;

        // current_path はテーブルヘッダ処理時に必ず doc.tables へ登録されるが、
        // 呼び出し元コードの契約に頼らず、万一見つからない場合も panic せず
        // エラーとして返す（`unwrap()`/`expect()` 不使用の方針、coding-rust.md）。
        let table = doc
            .tables
            .iter_mut()
            .find(|(p, _)| *p == current_path)
            .ok_or_else(|| {
                err(
                    line_no,
                    "internal error: current table not found".to_string(),
                )
            })?;
        if table.1.iter().any(|(k, _)| k == key) {
            return Err(err(line_no, format!("duplicate key `{key}` in table")));
        }
        table.1.push((key.to_string(), value));
    }

    Ok(doc)
}

/// 行コメント（`#` 以降）を取り除く。文字列リテラル内の `#` は除去しない。
fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut escaped = false;
    for (i, c) in line.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
        } else if c == '#' {
            return &line[..i];
        }
    }
    line
}

/// `key` が bare key（英数字・`_`・`-` のみ、非空）であることを検証する。
fn validate_bare_key(key: &str, line_no: usize) -> Result<(), TomlError> {
    if key.is_empty() {
        return Err(err(line_no, "empty key".to_string()));
    }
    if !key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(err(line_no, format!("invalid characters in key `{key}`")));
    }
    Ok(())
}

/// `[a.b]` 形式のテーブルヘッダをパースし、ドット区切りセグメント列を返す。
fn parse_table_header(trimmed: &str, line_no: usize) -> Result<Vec<String>, TomlError> {
    if trimmed.starts_with("[[") {
        return Err(err(
            line_no,
            "array-of-tables (`[[...]]`) is not supported".to_string(),
        ));
    }
    if !trimmed.ends_with(']') {
        return Err(err(line_no, "unterminated table header".to_string()));
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    if inner.contains('[') || inner.contains(']') {
        return Err(err(line_no, "malformed table header".to_string()));
    }
    if inner.trim().is_empty() {
        return Err(err(line_no, "empty table header".to_string()));
    }
    let segments: Vec<String> = inner.split('.').map(|s| s.trim().to_string()).collect();
    if segments.len() > MAX_TABLE_PATH_DEPTH {
        return Err(err(line_no, "table header nesting too deep".to_string()));
    }
    for seg in &segments {
        validate_bare_key(seg, line_no)?;
    }
    Ok(segments)
}

/// キーの右辺（コメント除去・trim 済み）を値としてパースする。
fn parse_value(raw: &str, line_no: usize) -> Result<Value, TomlError> {
    if raw.is_empty() {
        return Err(err(line_no, "missing value after `=`".to_string()));
    }
    if raw.starts_with('"') {
        return parse_basic_string(raw, line_no).map(Value::String);
    }
    if raw.starts_with('[') {
        return parse_array(raw, line_no);
    }
    if raw == "true" {
        return Ok(Value::Bool(true));
    }
    if raw == "false" {
        return Ok(Value::Bool(false));
    }
    if is_integer_literal(raw) {
        let n: i64 = raw
            .parse()
            .map_err(|_| err(line_no, format!("invalid integer literal `{raw}`")))?;
        return Ok(Value::Integer(n));
    }
    Err(err(
        line_no,
        format!("unsupported value syntax `{raw}` (this TOML subset supports only basic strings, string arrays, bool, integer)"),
    ))
}

fn is_integer_literal(raw: &str) -> bool {
    let digits = raw.strip_prefix('-').unwrap_or(raw);
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

/// `"..."` 形式の basic string をパースする。`\"` `\\` `\n` `\t` のみ対応。
fn parse_basic_string(s: &str, line_no: usize) -> Result<String, TomlError> {
    let chars: Vec<char> = s.chars().collect();
    if chars.first() != Some(&'"') {
        return Err(err(line_no, "expected string literal".to_string()));
    }
    let mut out = String::new();
    let mut i = 1usize;
    loop {
        let c = *chars
            .get(i)
            .ok_or_else(|| err(line_no, "unterminated string literal".to_string()))?;
        match c {
            '"' => {
                if i + 1 != chars.len() {
                    return Err(err(
                        line_no,
                        "unexpected trailing data after string literal".to_string(),
                    ));
                }
                return Ok(out);
            }
            '\\' => {
                i += 1;
                let esc = *chars
                    .get(i)
                    .ok_or_else(|| err(line_no, "unterminated escape sequence".to_string()))?;
                match esc {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    other => {
                        return Err(err(line_no, format!("invalid escape sequence `\\{other}`")))
                    }
                }
                i += 1;
            }
            c if (c as u32) < 0x20 => {
                return Err(err(
                    line_no,
                    "unescaped control character in string literal".to_string(),
                ))
            }
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }
}

/// `["a", "b"]` 形式の文字列配列をパースする。要素は文字列のみ・末尾カンマ許容。
fn parse_array(raw: &str, line_no: usize) -> Result<Value, TomlError> {
    if !raw.ends_with(']') {
        return Err(err(line_no, "unterminated array literal".to_string()));
    }
    let inner = &raw[1..raw.len() - 1];
    let parts = split_top_level_commas(inner);
    let mut items = Vec::new();
    for (i, part) in parts.iter().enumerate() {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            // 末尾カンマのみ許容する（最後の要素の後の空要素）。
            if i == parts.len() - 1 {
                continue;
            }
            return Err(err(line_no, "empty element in array literal".to_string()));
        }
        if !trimmed.starts_with('"') {
            return Err(err(
                line_no,
                "array elements must be string literals (this TOML subset supports only string arrays)".to_string(),
            ));
        }
        items.push(Value::String(parse_basic_string(trimmed, line_no)?));
    }
    Ok(Value::Array(items))
}

/// 文字列リテラル内のカンマを無視して、トップレベルのカンマで分割する。
fn split_top_level_commas(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut start = 0usize;
    for (i, c) in s.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            ',' => {
                result.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    result.push(&s[start..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PoC-7 `docs/spec/03-poc/ai-self-maintenance/target-project/structure.toml` 相当の
    /// フィクスチャ。実行時にサブモジュール（`docs/spec/`）へ依存しないよう、
    /// テスト内にインラインで保持する（サブモジュール未初期化環境でも通ること）。
    const FIXTURE: &str = r#"
# 機械可読なプロジェクト構造マニフェスト（PoC-7）
[directories.core]
role = "core"
crate = "rws-core"
description = "外部依存ゼロのレンダリングコア（Node木・render・escape）"
allowed_dependents = ["app", "server", "wasm-client"]

[directories.app]
role = "component"
crate = "rws-app"
description = "モード非依存の共通コンポーネント（SSR/CSR/SSG から共通利用）"
depends_on = ["core"]
allowed_dependents = ["server", "wasm-client"]

[directories.server]
role = "server-entrypoint"
crate = "rws-server"
description = "SSR/SSG エントリポイント（axum ルーティング）"
depends_on = ["core", "app"]
allowed_dependents = []

[directories.wasm-client]
role = "client-entrypoint"
crate = "rws-wasm-client"
description = "CSR/ハイドレーション エントリポイント（wasm32 ターゲット）"
depends_on = ["core", "app"]
allowed_dependents = []

[directories.static]
role = "asset"
crate = ""
description = "静的アセット（HTML/CSS/JS グルー）"
depends_on = []
allowed_dependents = []

[routing]
definition_file_pattern = "server/src/**/*.rs"
handler_pattern = "\\.route\\(\\s*\"([^\"]+)\"\\s*,\\s*get\\(([a-zA-Z0-9_]+)\\)\\)"
"#;

    #[test]
    fn parses_poc7_fixture() {
        let doc = parse(FIXTURE).expect("フィクスチャは正常にパースできる想定");
        let core = doc.table(&["directories", "core"]).unwrap();
        assert_eq!(
            core.iter().find(|(k, _)| k == "role").unwrap().1.as_str(),
            Some("core")
        );
        assert_eq!(
            core.iter()
                .find(|(k, _)| k == "allowed_dependents")
                .unwrap()
                .1
                .as_string_array(),
            Some(vec!["app", "server", "wasm-client"])
        );
        // `crate = ""` は「値が空文字列」であり「キー不在」ではない。
        let static_dir = doc.table(&["directories", "static"]).unwrap();
        assert_eq!(
            static_dir
                .iter()
                .find(|(k, _)| k == "crate")
                .unwrap()
                .1
                .as_str(),
            Some("")
        );
        let routing = doc.table(&["routing"]).unwrap();
        // TOML エスケープが正しく解決され、二重バックスラッシュ (\\) が単一の
        // バックスラッシュへ、\" が " へ変換されていること。
        assert_eq!(
            routing
                .iter()
                .find(|(k, _)| k == "handler_pattern")
                .unwrap()
                .1
                .as_str(),
            Some(r#"\.route\(\s*"([^"]+)"\s*,\s*get\(([a-zA-Z0-9_]+)\)\)"#)
        );
    }

    #[test]
    fn rejects_array_of_tables() {
        assert!(parse("[[directories]]\nrole = \"core\"\n").is_err());
    }

    #[test]
    fn rejects_unknown_syntax() {
        assert!(parse("key = 2024-01-01T00:00:00Z\n").is_err()); // 日時（非対応）
        assert!(parse("key = { inline = \"table\" }\n").is_err()); // inline table（非対応）
        assert!(parse("key = \"\"\"multi\nline\"\"\"\n").is_err()); // 複数行文字列（非対応）
    }

    #[test]
    fn rejects_malformed_table_header() {
        assert!(parse("[a.b\nkey = \"v\"\n").is_err());
        assert!(parse("[]\nkey = \"v\"\n").is_err());
    }

    #[test]
    fn rejects_duplicate_key_and_table() {
        assert!(parse("[a]\nkey = \"1\"\nkey = \"2\"\n").is_err());
        assert!(parse("[a]\nkey = \"1\"\n[a]\nkey2 = \"2\"\n").is_err());
    }

    #[test]
    fn does_not_panic_on_huge_input() {
        let huge = "#".repeat(MAX_INPUT_BYTES + 1);
        assert!(parse(&huge).is_err());
    }

    #[test]
    fn allows_trailing_comma_in_array() {
        let doc = parse("[a]\nkey = [\"x\", \"y\",]\n").unwrap();
        let table = doc.table(&["a"]).unwrap();
        assert_eq!(
            table
                .iter()
                .find(|(k, _)| k == "key")
                .unwrap()
                .1
                .as_string_array(),
            Some(vec!["x", "y"])
        );
    }

    #[test]
    fn ignores_hash_inside_string_literal() {
        let doc = parse("[a]\nkey = \"not a # comment\"\n").unwrap();
        let table = doc.table(&["a"]).unwrap();
        assert_eq!(
            table.iter().find(|(k, _)| k == "key").unwrap().1.as_str(),
            Some("not a # comment")
        );
    }
}
