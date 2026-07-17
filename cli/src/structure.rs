//! `structure.toml`（REQ-13 のプロジェクト構造マニフェスト）を型付き Rust 構造体へ
//! パース・検証するロジック（TASK-13.1b、親タスク TASK-13.1 の成果物本体）。
//!
//! [`crate::toml`] が返す汎用 TOML ドキュメントを、`fw structure` / `fw impact` /
//! `fw gate`（TASK-13.1c #130 以降、`cargo metadata` 連携・ルート抽出・JSON 出力を
//! 追加する予定）から使いやすい型（[`StructureManifest`]）へ変換し、
//! セマンティックな妥当性（必須キー・役割の列挙値・依存関係の参照整合性）を
//! 検証する。「宣言の機械的検証」という REQ-13 の趣旨に従い、未知キー・
//! 未知の役割・存在しないディレクトリへの参照はすべて fail-closed でエラーとする。
//!
//! # スキーマ（PoC-7 準拠、TASK-13.1a #128 未マージ時点の暫定合意）
//!
//! ```toml
//! [directories.<name>]
//! role = "core" | "component" | "server-entrypoint" | "client-entrypoint" | "asset"
//! crate = "<crate 名。アセットディレクトリ等は空文字列を許容>"
//! description = "<説明>"
//! depends_on = ["<directories 名>", ...]           # 任意。省略時は空配列
//! allowed_dependents = ["<directories 名>", ...]    # 任意。省略時は空配列
//!
//! [routing]
//! definition_file_pattern = "<glob パターン>"
//! handler_pattern = "<正規表現パターン。本イシューでは評価せず不透明な文字列として保持>"
//! ```
//!
//! `handler_pattern` の正規表現評価（ReDoS 対策含む）は TASK-13.1c のスコープであり、
//! 本モジュールは文字列として保持するのみで評価しない。
//!
//! #128（`13.1a: structure.toml スキーマ設計`）がマージされフィールド名・必須/任意の
//! 扱いに差異が生じた場合は、本モジュールの型定義・検証ルールのみが影響を受け、
//! [`crate::toml`] のパーサ自体には影響しない構造にしてある。
use crate::toml::{self, TomlError, Value};
use std::fmt;
use std::path::Path;

/// ディレクトリの役割。`structure.toml` の `role` キーが取り得る値のホワイトリスト。
///
/// 未知の役割文字列は [`StructureError`] としてエラーになる（fail-closed）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryRole {
    Core,
    Component,
    ServerEntrypoint,
    ClientEntrypoint,
    Asset,
}

impl DirectoryRole {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "core" => Some(Self::Core),
            "component" => Some(Self::Component),
            "server-entrypoint" => Some(Self::ServerEntrypoint),
            "client-entrypoint" => Some(Self::ClientEntrypoint),
            "asset" => Some(Self::Asset),
            _ => None,
        }
    }
}

/// `[directories.<name>]` テーブル 1 件分の宣言内容。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryRule {
    pub role: DirectoryRole,
    /// 対応する crate 名。`static/` 等アセット専用ディレクトリでは空文字列を許容する
    /// （キー自体は必須。「値が空」と「キー不在」を区別する）。
    pub crate_name: String,
    pub description: String,
    /// 省略時は空配列（TOML 側で `depends_on` キーを省略できる）。
    pub depends_on: Vec<String>,
    /// 省略時は空配列。
    pub allowed_dependents: Vec<String>,
}

/// `[routing]` テーブルの宣言内容。
///
/// パターン文字列は本イシューでは不透明な値として保持するのみで評価しない
/// （正規表現評価・ReDoS 対策は TASK-13.1c のスコープ）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingRule {
    pub definition_file_pattern: String,
    pub handler_pattern: String,
}

/// パース・検証済みの `structure.toml` 全体。
///
/// `directories` は宣言順を保持する（`Vec` を採用し `HashMap` は使わない。
/// 出力の再現性・テスト容易性のため）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureManifest {
    pub directories: Vec<(String, DirectoryRule)>,
    pub routing: RoutingRule,
}

impl StructureManifest {
    /// 宣言済みディレクトリ名からルールを引く。
    pub fn directory(&self, name: &str) -> Option<&DirectoryRule> {
        self.directories
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, rule)| rule)
    }
}

/// パース・検証失敗時のエラー。
///
/// メッセージには入力中の識別子（キー名・テーブル名）のみを含め、
/// ファイルパスや環境変数等の内部情報は含めない（security.md 機微情報露出防止）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructureError {
    /// 下層の TOML 構文パースに失敗した。
    Toml(TomlError),
    /// セマンティック検証（必須キー欠落・型不一致・未知キー・参照整合性等）に失敗した。
    Validation(String),
}

impl fmt::Display for StructureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StructureError::Toml(e) => write!(f, "structure.toml is not valid TOML: {e}"),
            StructureError::Validation(msg) => {
                write!(f, "structure.toml failed validation: {msg}")
            }
        }
    }
}

impl std::error::Error for StructureError {}

impl From<TomlError> for StructureError {
    fn from(e: TomlError) -> Self {
        StructureError::Toml(e)
    }
}

fn validation_err(message: impl Into<String>) -> StructureError {
    StructureError::Validation(message.into())
}

/// `input` を `structure.toml` としてパース・検証する。
///
/// `rws-cli` の唯一の TOML 解析エントリポイント。`load` からファイル読み込み後に
/// 呼ばれるほか、テストでも直接使用する。
///
/// 命名について: 標準ライブラリの `FromStr::from_str` と衝突しない自由関数として
/// 提供する（`clippy::should_implement_trait` を避けるため。`xtask/src/json.rs`
/// の `parse` と同じ方針）。
pub fn parse(input: &str) -> Result<StructureManifest, StructureError> {
    let doc = toml::parse(input)?;

    // 未知のトップレベル構成を拒否する: このスキーマで許可されるテーブルパスは
    // `["directories", <name>]` と `["routing"]` のみ。トップレベル直書きキー
    // （空パス）や、それ以外の未知テーブルは fail-closed でエラーとする。
    let mut directories: Vec<(String, DirectoryRule)> = Vec::new();
    let mut routing: Option<RoutingRule> = None;

    for (path, table) in doc.entries() {
        match path {
            [] => {
                if !table.is_empty() {
                    return Err(validation_err(
                        "top-level keys outside `[directories.*]` / `[routing]` are not supported",
                    ));
                }
            }
            [seg] if seg == "routing" => {
                routing = Some(parse_routing(table)?);
            }
            [seg, name] if seg == "directories" => {
                let rule = parse_directory(name, table)?;
                directories.push((name.clone(), rule));
            }
            other => {
                return Err(validation_err(format!(
                    "unknown table `[{}]` (expected `[directories.<name>]` or `[routing]`)",
                    other.join(".")
                )));
            }
        }
    }

    if directories.is_empty() {
        return Err(validation_err(
            "at least one `[directories.<name>]` table is required",
        ));
    }
    let routing = routing.ok_or_else(|| validation_err("`[routing]` table is required"))?;

    // 参照整合性: depends_on / allowed_dependents は宣言済みディレクトリ名のみを許す。
    let known_names: Vec<&str> = directories.iter().map(|(n, _)| n.as_str()).collect();
    for (name, rule) in &directories {
        for referenced in rule.depends_on.iter().chain(rule.allowed_dependents.iter()) {
            if !known_names.contains(&referenced.as_str()) {
                return Err(validation_err(format!(
                    "`directories.{name}` references unknown directory `{referenced}`"
                )));
            }
        }
    }

    Ok(StructureManifest {
        directories,
        routing,
    })
}

/// `structure.toml` を指定パスから読み込みパース・検証する。
///
/// パスはそのまま `std::fs::read_to_string` に渡すのみで、正規化・展開は行わない
/// （パストラバーサル対策・シンボリックリンク追跡の扱いはこの API を呼ぶ側の
/// 走査ロジック — TASK-13.1c 以降 — の責務とする）。
pub fn load(path: &Path) -> Result<StructureManifest, StructureError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        validation_err(format!(
            "failed to read structure.toml: {}",
            io_error_kind_message(&e)
        ))
    })?;
    parse(&content)
}

/// I/O エラーの種別のみを文字列化する。`std::io::Error` の `Display` は
/// OS 依存のメッセージ（まれに内部パスを含み得る）を出すため、
/// 機微情報露出を避けて `ErrorKind` ベースの定型メッセージに丸める。
fn io_error_kind_message(e: &std::io::Error) -> String {
    format!("{:?}", e.kind())
}

fn get_str<'a>(
    table: &'a [(String, Value)],
    key: &str,
    context: &str,
) -> Result<&'a str, StructureError> {
    let value = table
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
        .ok_or_else(|| validation_err(format!("`{context}` is missing required key `{key}`")))?;
    value
        .as_str()
        .ok_or_else(|| validation_err(format!("`{context}.{key}` must be a string")))
}

fn get_optional_string_array(
    table: &[(String, Value)],
    key: &str,
    context: &str,
) -> Result<Vec<String>, StructureError> {
    match table.iter().find(|(k, _)| k == key).map(|(_, v)| v) {
        None => Ok(Vec::new()),
        Some(value) => value
            .as_string_array()
            .map(|items| items.into_iter().map(str::to_string).collect())
            .ok_or_else(|| validation_err(format!("`{context}.{key}` must be a string array"))),
    }
}

/// `[directories.<name>]` テーブル 1 件を検証しつつ [`DirectoryRule`] へ変換する。
fn parse_directory(name: &str, table: &[(String, Value)]) -> Result<DirectoryRule, StructureError> {
    let context = format!("directories.{name}");
    const ALLOWED_KEYS: &[&str] = &[
        "role",
        "crate",
        "description",
        "depends_on",
        "allowed_dependents",
    ];
    reject_unknown_keys(table, ALLOWED_KEYS, &context)?;

    let role_str = get_str(table, "role", &context)?;
    let role = DirectoryRole::parse(role_str).ok_or_else(|| {
        validation_err(format!(
            "`{context}.role` has unknown value `{role_str}` (expected one of: core, component, server-entrypoint, client-entrypoint, asset)"
        ))
    })?;
    // `crate` は空文字列を許容する（`static/` 等）。キー自体は必須。
    let crate_name = get_str(table, "crate", &context)?.to_string();
    let description = get_str(table, "description", &context)?.to_string();
    let depends_on = get_optional_string_array(table, "depends_on", &context)?;
    let allowed_dependents = get_optional_string_array(table, "allowed_dependents", &context)?;

    Ok(DirectoryRule {
        role,
        crate_name,
        description,
        depends_on,
        allowed_dependents,
    })
}

/// `[routing]` テーブルを検証しつつ [`RoutingRule`] へ変換する。
fn parse_routing(table: &[(String, Value)]) -> Result<RoutingRule, StructureError> {
    const ALLOWED_KEYS: &[&str] = &["definition_file_pattern", "handler_pattern"];
    reject_unknown_keys(table, ALLOWED_KEYS, "routing")?;

    Ok(RoutingRule {
        definition_file_pattern: get_str(table, "definition_file_pattern", "routing")?.to_string(),
        handler_pattern: get_str(table, "handler_pattern", "routing")?.to_string(),
    })
}

/// テーブル内に許可リスト外のキーがないか検証する（未知キーの fail-closed 拒否）。
fn reject_unknown_keys(
    table: &[(String, Value)],
    allowed: &[&str],
    context: &str,
) -> Result<(), StructureError> {
    for (key, _) in table {
        if !allowed.contains(&key.as_str()) {
            return Err(validation_err(format!(
                "`{context}` has unknown key `{key}`"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PoC-7 フィクスチャ相当。[`crate::toml`] のテストと同じ内容をここでも
    /// インラインで保持する（サブモジュール未初期化環境への実行時依存を作らない）。
    const FIXTURE: &str = r#"
[directories.core]
role = "core"
crate = "rws-core"
description = "外部依存ゼロのレンダリングコア"
allowed_dependents = ["app", "server", "wasm-client"]

[directories.app]
role = "component"
crate = "rws-app"
description = "共通コンポーネント"
depends_on = ["core"]
allowed_dependents = ["server", "wasm-client"]

[directories.server]
role = "server-entrypoint"
crate = "rws-server"
description = "SSR/SSG エントリポイント"
depends_on = ["core", "app"]

[directories.wasm-client]
role = "client-entrypoint"
crate = "rws-wasm-client"
description = "CSR エントリポイント"
depends_on = ["core", "app"]

[directories.static]
role = "asset"
crate = ""
description = "静的アセット"

[routing]
definition_file_pattern = "server/src/**/*.rs"
handler_pattern = "\\.route\\(\\s*\"([^\"]+)\"\\s*,\\s*get\\(([a-zA-Z0-9_]+)\\)\\)"
"#;

    #[test]
    fn parses_poc7_fixture_successfully() {
        let manifest = parse(FIXTURE).expect("フィクスチャは検証済みスキーマを満たす");
        assert_eq!(manifest.directories.len(), 5);
        let core = manifest.directory("core").unwrap();
        assert_eq!(core.role, DirectoryRole::Core);
        assert_eq!(core.crate_name, "rws-core");
        // depends_on を省略したディレクトリは空配列になる。
        assert!(core.depends_on.is_empty());
        assert_eq!(
            core.allowed_dependents,
            vec!["app", "server", "wasm-client"]
        );

        let static_dir = manifest.directory("static").unwrap();
        assert_eq!(static_dir.role, DirectoryRole::Asset);
        // crate = "" はキーが「存在してかつ空文字列」であることを意味する。
        assert_eq!(static_dir.crate_name, "");
        assert!(static_dir.depends_on.is_empty());
        assert!(static_dir.allowed_dependents.is_empty());

        assert_eq!(
            manifest.routing.handler_pattern,
            r#"\.route\(\s*"([^"]+)"\s*,\s*get\(([a-zA-Z0-9_]+)\)\)"#
        );
    }

    #[test]
    fn rejects_missing_required_key() {
        let missing_role = r#"
[directories.core]
crate = "rws-core"
description = "desc"

[routing]
definition_file_pattern = "x"
handler_pattern = "y"
"#;
        assert!(parse(missing_role).is_err());
    }

    #[test]
    fn rejects_unknown_key() {
        let unknown_key = r#"
[directories.core]
role = "core"
crate = "rws-core"
description = "desc"
unexpected = "value"

[routing]
definition_file_pattern = "x"
handler_pattern = "y"
"#;
        assert!(parse(unknown_key).is_err());
    }

    #[test]
    fn rejects_unknown_role() {
        let unknown_role = r#"
[directories.core]
role = "not-a-role"
crate = "rws-core"
description = "desc"

[routing]
definition_file_pattern = "x"
handler_pattern = "y"
"#;
        assert!(parse(unknown_role).is_err());
    }

    #[test]
    fn rejects_dangling_reference() {
        let dangling = r#"
[directories.core]
role = "core"
crate = "rws-core"
description = "desc"
depends_on = ["nonexistent"]

[routing]
definition_file_pattern = "x"
handler_pattern = "y"
"#;
        assert!(parse(dangling).is_err());
    }

    #[test]
    fn rejects_empty_directories() {
        let no_dirs = r#"
[routing]
definition_file_pattern = "x"
handler_pattern = "y"
"#;
        assert!(parse(no_dirs).is_err());
    }

    #[test]
    fn rejects_missing_routing() {
        let no_routing = r#"
[directories.core]
role = "core"
crate = "rws-core"
description = "desc"
"#;
        assert!(parse(no_routing).is_err());
    }

    #[test]
    fn rejects_unsupported_syntax_propagated_from_toml_layer() {
        let bad_toml = "[[directories]]\nrole = \"core\"\n";
        assert!(matches!(parse(bad_toml), Err(StructureError::Toml(_))));
    }

    #[test]
    fn load_reports_error_for_missing_file() {
        let result = load(Path::new("/nonexistent/path/does-not-exist/structure.toml"));
        assert!(result.is_err());
        // エラーメッセージに絶対パス文字列そのものを含めないこと（機微情報露出の抑止）。
        let message = result.unwrap_err().to_string();
        assert!(!message.contains("/nonexistent/path"));
    }

    #[test]
    fn does_not_panic_on_arbitrary_input() {
        // fuzz 的な軽い健全性確認: 明らかに壊れた入力でも panic せず Err を返す。
        let inputs = [
            "",
            "[",
            "]",
            "[directories]",
            "[directories.]",
            "key",
            "\0\0\0",
        ];
        for input in inputs {
            let _ = parse(input);
        }
    }
}
