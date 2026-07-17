//! `fw`: AI 自己保守フック（REQ-13）向けの開発者・エージェント用 CLI エントリポイント。
//!
//! TASK-13.1（親 #127）の製品化対象。本ファイルは TASK-13.1a（#128, 本クレート
//! 骨格 + `structure` モジュールのスキーマ型定義）の時点ではサブコマンド
//! ディスパッチの骨格のみを提供し、`structure` サブコマンドは未実装スタブ
//! （終了コード 2 + 英語メッセージ）とする。実処理は以下へ段階的に実装される:
//! - TASK-13.1b（#129）: `structure.toml`（TOML サブセット）の手書きパーサ実装
//! - TASK-13.1c（#130）: `cargo metadata` 連携によるマニフェスト生成・突き合わせ
//! - TASK-13.1d（#131）: ルートの `structure.toml` を用いた統合テスト整備
//!
//! スキーマ型定義・整合性検証（`validate()`）は [`structure`] モジュールを参照。

#![forbid(unsafe_code)]

mod structure;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let exit_code = run(&args);
    std::process::exit(exit_code);
}

/// サブコマンドディスパッチ本体。`main` からテスト容易性のため分離する。
///
/// 戻り値はプロセスの終了コード。未知のサブコマンド・引数不足は 2、
/// 正常終了は 0 とする（xtask の `check-deps` 等と終了コード規約を統一）。
fn run(args: &[String]) -> i32 {
    match args.first().map(String::as_str) {
        Some("structure") => run_structure(&args[1..]),
        Some(other) => {
            eprintln!("fw: unknown subcommand `{other}`");
            print_usage();
            2
        }
        None => {
            eprintln!("fw: a subcommand is required");
            print_usage();
            2
        }
    }
}

fn print_usage() {
    eprintln!("Usage: fw <subcommand>");
    eprintln!("Subcommands:");
    eprintln!("  structure    generate/validate the machine-readable project structure manifest");
}

/// `structure` サブコマンド: TASK-13.1a 時点では未実装。
///
/// `structure.toml` のパース（TASK-13.1b）とマニフェスト生成（TASK-13.1c）が
/// 揃うまでは常に非 0 終了で「未実装」であることを明示する。呼び出し元
/// （CI・AI 自己保守フック）が誤って「構造チェック PASS」と解釈しないよう、
/// フォールバックの黙示的成功は返さない。
fn run_structure(_args: &[String]) -> i32 {
    eprintln!(
        "fw structure: not implemented yet (TASK-13.1b/13.1c, see docs/structure-manifest.md)"
    );
    2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_subcommand_is_an_error() {
        assert_eq!(run(&[]), 2);
    }

    #[test]
    fn unknown_subcommand_is_an_error() {
        assert_eq!(run(&["bogus".to_string()]), 2);
    }

    #[test]
    fn structure_subcommand_is_a_stub_for_now() {
        assert_eq!(run(&["structure".to_string()]), 2);
    }
}
