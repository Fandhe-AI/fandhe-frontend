//! `fw new` サブコマンド本体（TASK-13.4 相当、イシュー #350／複数テンプレート
//! 選択、イシュー #378）。
//!
//! 親イシュー #338「決定的スキャフォールド — fw new」の第 1 タスク。
//! `templates/<name>/`（[`crate::new_template::TEMPLATES`] としてコンパイル
//! 時埋め込み）を決定的に展開し、AI エージェントが毎回 boilerplate を生成する
//! ことによる構成ドリフトを防ぐ。`fw gate` / `fw impact` / `structure.toml` が
//! そのまま効く「全プロジェクトが同一構成」を保証するのが目的。
//!
//! 終了コードは `main.rs` 冒頭の doc コメントが明文化する全サブコマンド共通の
//! 規約（0 = 成功 / 1 = 検証違反・実行失敗 / 2 = 使用法エラー）に従う。
//! テンプレートは `structure.toml`（イシュー #351）を同梱し、`fw gate` が
//! 唯一の情報源として読む宣言クレート名をプロジェクト名へ置換することで、
//! 生成直後の `fw gate` が無編集で PASS する構成を保証する
//! （`cli/tests/new_gate_e2e.rs` が e2e で固定する）。
//!
//! `--template <name>` はイシュー #378 で追加した選択 UI。未指定時は
//! [`crate::new_template::DEFAULT_TEMPLATE_NAME`]（`default`）を使い、
//! イシュー #378 以前の `fw new` 呼び出しと完全後方互換（同一バイト出力）を
//! 保つ。未知のテンプレート名は使用法エラー（終了コード 2）とし、stderr へ
//! 利用可能テンプレート一覧（固定順）を出す。

use crate::json_out::{quoted, string_array};
use crate::new_template::{find_template, Template, DEFAULT_TEMPLATE_NAME, TEMPLATES};
use std::fs;
use std::path::{Path, PathBuf};

const USAGE: &str =
    "fw new: usage: fw new <project-name> [--template <template>] [--dir <parent-dir>] [--force]";

/// `fw new` サブコマンド本体。
///
/// 1. 引数を解析する（第 1 位置引数 `<project-name>` 必須、`--template` /
///    `--dir` / `--force` はオプション）。引数の使い方が誤っている場合は
///    使用法エラー（終了コード 2）
/// 2. [`validate_project_name`] でプロジェクト名を検証する（違反は終了コード 2。
///    パストラバーサル対策として `/` `\` `..` 等を構造的に排除する）
/// 3. `--template` をコンパイル時 allowlist（[`TEMPLATES`]）と完全一致照合する。
///    未知の名前は使用法エラー（終了コード 2）
/// 4. ターゲットパス（`<parent-dir>/<project-name>`）の存在確認。`--force`
///    なしで既存の場合は fail-closed で拒否（終了コード 1）
/// 5. 選択したテンプレートの `files` を配列順（固定）で展開し、
///    `Cargo.toml` / `Cargo.lock` / `structure.toml` の package 名を
///    プロジェクト名へ置換する
/// 6. 成功時は展開したファイル一覧・使用テンプレート名を JSON で stdout へ
///    出力し終了コード 0
pub(crate) fn run_new(args: &[String]) -> i32 {
    let parsed = match parse_args(args) {
        Ok(p) => p,
        Err(code) => {
            eprintln!("{USAGE}");
            return code;
        }
    };

    if let Err(msg) = validate_project_name(&parsed.project_name) {
        eprintln!("fw new: {msg}");
        eprintln!("{USAGE}");
        return 2;
    }

    let template = match find_template(&parsed.template_name) {
        Some(t) => t,
        None => {
            let available: Vec<&str> = TEMPLATES.iter().map(|t| t.name).collect();
            eprintln!(
                "fw new: unknown template `{}` (available: {})",
                parsed.template_name,
                available.join(", ")
            );
            eprintln!("{USAGE}");
            return 2;
        }
    };

    let target = parsed.parent_dir.join(&parsed.project_name);

    // fail-closed: `--force` なしでターゲットが既存（ファイル・空ディレクトリ
    // 含む）の場合は展開せず拒否する。既存プロジェクトを気付かないまま
    // 上書き・混入させないための安全弁（security.md A05）。
    if target.exists() && !parsed.force {
        eprintln!(
            "fw new: target `{}` already exists (use --force to overwrite)",
            target.display()
        );
        return 1;
    }

    match expand_template(template, &target, &parsed.project_name) {
        Ok(files) => {
            let files_json = string_array(&files);
            println!(
                "{{\"created\":{},\"template\":{},\"files\":{}}}",
                quoted(&target.to_string_lossy()),
                quoted(template.name),
                files_json
            );
            0
        }
        Err(msg) => {
            eprintln!("fw new: {msg}");
            1
        }
    }
}

struct ParsedArgs {
    project_name: String,
    parent_dir: PathBuf,
    template_name: String,
    force: bool,
}

/// `fw new <project-name> [--template <template>] [--dir <parent-dir>] [--force]`
/// を解析する。
///
/// `Err(2)` は使用法エラー（引数欠落・`--dir`/`--template` 値欠落・未知
/// フラグ）を表す。`--template` 未指定時は
/// [`DEFAULT_TEMPLATE_NAME`] を採用する（テンプレート名自体の allowlist
/// 照合は呼び出し元 [`run_new`] が行う。ここでは値の欠落のみを検査する）。
fn parse_args(args: &[String]) -> Result<ParsedArgs, i32> {
    let mut project_name: Option<String> = None;
    let mut parent_dir: Option<PathBuf> = None;
    let mut template_name: Option<String> = None;
    let mut force = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dir" => {
                let value = args.get(i + 1).ok_or(2)?;
                parent_dir = Some(PathBuf::from(value));
                i += 2;
            }
            "--template" => {
                let value = args.get(i + 1).ok_or(2)?;
                template_name = Some(value.clone());
                i += 2;
            }
            "--force" => {
                force = true;
                i += 1;
            }
            other if other.starts_with("--") => return Err(2),
            other => {
                if project_name.is_some() {
                    // 第 1 位置引数はすでに確定済み。第 2 の位置引数は使用法エラー。
                    return Err(2);
                }
                project_name = Some(other.to_string());
                i += 1;
            }
        }
    }

    let project_name = project_name.ok_or(2)?;
    let parent_dir = match parent_dir {
        Some(dir) => dir,
        None => std::env::current_dir().map_err(|_| 1)?,
    };
    let template_name = template_name.unwrap_or_else(|| DEFAULT_TEMPLATE_NAME.to_string());

    Ok(ParsedArgs {
        project_name,
        parent_dir,
        template_name,
        force,
    })
}

/// プロジェクト名の検証規則（cargo package name のサブセット）:
///
/// - 非空・64 文字以内
/// - 先頭は `[a-z]`、以降は `[a-z0-9_-]` のみ
///
/// パス区切り（`/` `\`）・`..`・先頭 `-` を構造的に排除し、ターゲットパス
/// 組み立て・テンプレート内文字列置換の双方でパストラバーサル・構文注入が
/// 起こり得ない文字集合に限定する（security.md A01/A03）。
fn validate_project_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("project name must not be empty".to_string());
    }
    if name.chars().count() > 64 {
        return Err("project name must be 64 characters or fewer".to_string());
    }
    let mut chars = name.chars();
    let first = chars.next().expect("checked non-empty above");
    if !first.is_ascii_lowercase() {
        return Err(format!(
            "project name must start with a lowercase ASCII letter (a-z): `{name}`"
        ));
    }
    if let Some(bad) =
        chars.find(|c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_' || *c == '-'))
    {
        return Err(format!(
            "project name may only contain lowercase ASCII letters, digits, `_`, `-` (found `{bad}`): `{name}`"
        ));
    }
    Ok(())
}

/// 一意な部分文字列 `needle` を `replacement` へちょうど `expected_count` 回
/// だけ置換する。出現回数が一致しない場合はテンプレート改変等で置換前提が
/// 崩れたことを示すため `Err` とする（`replace_all` 相当の一括置換ではなく、
/// 数を検証してから置換することで黙示的な置換漏れ・過剰置換を防ぐ）。
fn replace_exact(
    contents: &str,
    needle: &str,
    replacement: &str,
    expected_count: usize,
) -> Result<String, String> {
    let actual_count = contents.matches(needle).count();
    if actual_count != expected_count {
        return Err(format!(
            "template placeholder `{needle}` occurred {actual_count} time(s), expected {expected_count}"
        ));
    }
    Ok(contents.replace(needle, replacement))
}

/// 選択された [`Template`] の `files` を配列順（固定）で `target` 配下へ
/// 展開する。
///
/// タイムスタンプ・乱数・環境変数由来の値を出力ファイルへ一切書き込まない
/// ため、同一引数の 2 回実行はバイト単位で同一出力になる（決定性の担保）。
/// 書き込み途中の失敗は該当パスを含めて `Err` を返す（部分生成物を残すが、
/// 成功と誤認させる 0 終了は返さない）。
fn expand_template(
    template: &Template,
    target: &Path,
    project_name: &str,
) -> Result<Vec<String>, String> {
    let mut written: Vec<String> = Vec::with_capacity(template.files.len());

    for file in template.files {
        let dest = target.join(file.rel_path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create directory `{}`: {e}", parent.display()))?;
        }

        let contents = if template.substituted_files.contains(&file.rel_path) {
            replace_exact(file.contents, template.needle, project_name, 1)?
        } else {
            file.contents.to_string()
        };

        fs::write(&dest, contents)
            .map_err(|e| format!("failed to write `{}`: {e}", dest.display()))?;

        set_permissions(&dest, file.executable)?;

        written.push(file.rel_path.to_string());
    }

    Ok(written)
}

/// `executable` なテンプレートファイルに Unix の実行ビット（0o755）を設定する。
///
/// 非 Unix プラットフォームではパーミッションモデルが異なるため設定をスキップ
/// する（`docs/design/fw-new-design.md` に明記）。
#[cfg(unix)]
fn set_permissions(path: &Path, executable: bool) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    if !executable {
        return Ok(());
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("failed to set permissions on `{}`: {e}", path.display()))
}

#[cfg(not(unix))]
fn set_permissions(_path: &Path, _executable: bool) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- validate_project_name ---

    #[test]
    fn accepts_valid_names() {
        assert!(validate_project_name("demo-app").is_ok());
        assert!(validate_project_name("demo_app_2").is_ok());
        assert!(validate_project_name("a").is_ok());
        assert!(validate_project_name(&"a".repeat(64)).is_ok());
    }

    #[test]
    fn rejects_empty_name() {
        assert!(validate_project_name("").is_err());
    }

    #[test]
    fn rejects_names_over_64_chars() {
        assert!(validate_project_name(&"a".repeat(65)).is_err());
    }

    #[test]
    fn rejects_uppercase() {
        assert!(validate_project_name("DemoApp").is_err());
    }

    #[test]
    fn rejects_leading_digit_or_dash() {
        assert!(validate_project_name("1demo").is_err());
        assert!(validate_project_name("-demo").is_err());
    }

    #[test]
    fn rejects_path_separators_and_traversal() {
        assert!(validate_project_name("a/b").is_err());
        assert!(validate_project_name("a\\b").is_err());
        assert!(validate_project_name("..").is_err());
        assert!(validate_project_name("../evil").is_err());
    }

    // --- replace_exact ---

    #[test]
    fn replace_exact_succeeds_on_expected_count() {
        let out = replace_exact("name = \"x\"\nname = \"x\"", "x", "y", 2).unwrap();
        assert_eq!(out, "name = \"y\"\nname = \"y\"");
    }

    #[test]
    fn replace_exact_errors_on_count_mismatch() {
        assert!(replace_exact("no match here", "needle", "x", 1).is_err());
        assert!(replace_exact("needle needle", "needle", "x", 1).is_err());
    }

    // --- parse_args / run_new usage errors ---

    #[test]
    fn parse_args_requires_project_name() {
        assert!(parse_args(&[]).is_err());
    }

    #[test]
    fn parse_args_rejects_unknown_flag() {
        assert!(parse_args(&["demo".to_string(), "--bogus".to_string()]).is_err());
    }

    #[test]
    fn parse_args_rejects_missing_dir_value() {
        assert!(parse_args(&["demo".to_string(), "--dir".to_string()]).is_err());
    }

    #[test]
    fn parse_args_rejects_second_positional() {
        assert!(parse_args(&["demo".to_string(), "extra".to_string()]).is_err());
    }

    #[test]
    fn parse_args_accepts_force_and_dir() {
        let parsed = parse_args(&[
            "demo".to_string(),
            "--dir".to_string(),
            "/tmp/x".to_string(),
            "--force".to_string(),
        ])
        .unwrap();
        assert_eq!(parsed.project_name, "demo");
        assert_eq!(parsed.parent_dir, PathBuf::from("/tmp/x"));
        assert!(parsed.force);
    }

    #[test]
    fn run_new_without_args_is_usage_error() {
        assert_eq!(run_new(&[]), 2);
    }

    #[test]
    fn run_new_rejects_invalid_project_name() {
        assert_eq!(run_new(&["../evil".to_string()]), 2);
        assert_eq!(run_new(&["UPPER".to_string()]), 2);
    }
}
