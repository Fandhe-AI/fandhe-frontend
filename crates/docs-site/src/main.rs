//! `fandhe-frontend-docs-site` の起動エントリ。
//!
//! 公式 docs サイト（`docs/` 配下のドキュメントを本フレームワーク自身の SSG で
//! 静的サイトへ変換するもの）を生成するバイナリ。イシュー #470 でビルド
//! エントリを統合し、`--out <dir>` を渡せば `site/nav.toml` に基づく全ページと
//! アセットが `<dir>` へ書き出される（受け入れ条件 1）。内部リンク検証
//! （`crate::linkcheck`）で 1 件でもリンク切れが見つかれば、書き出しを一切
//! 行わずエラー内容を報告して非 0 終了する（受け入れ条件 2）。
//!
//! 本ファイル自体は引数パースと終了コード変換のみを担う薄いラッパーであり、
//! ビルドロジック本体は [`fandhe_frontend_docs_site::build::build_site`]
//! （`src/build.rs`）に置く。`tests/site_build.rs`（E2E テスト、
//! `env!("CARGO_BIN_EXE_docs-site")` 経由でこのバイナリを起動する）と
//! `build_site` を直接呼ぶ単体テストの双方から同一のビルドロジックを共有
//! するための分離である。
//!
//! CLI 引数（外部クレート clap 等は追加しない、REQ-3）:
//! - `--out <dir>`（必須）: 出力先ディレクトリ
//! - `--root <dir>`（任意、既定 `.`）: `<root>/site/nav.toml` を読むリポジトリ
//!   ルート。フィクスチャルートを渡す E2E テストのために存在する
//!
//! `--out` 欠落・未知の引数は usage を stderr に出して非 0 終了する
//! （黙って既定値へフォールバックしない、fail-closed。`security.md` A05）。
//!
//! CI ワークフロー（イシュー #474 のスコープ）から `cargo run -p
//! fandhe-frontend-docs-site -- --out dist/` の形で呼ばれる想定。
//!
//! 開発者・CI 用ツールであり、生成物は配布物（`fandhe-frontend-dist-server` 等）
//! には含めない（`structure.toml` の `role = "tooling"` 参照）。

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use fandhe_frontend_docs_site::build::build_site;

/// パース済み CLI 引数。
#[derive(Debug)]
struct Args {
    root: PathBuf,
    out: PathBuf,
}

/// `std::env::args` を手動パースする（外部クレート非依存、REQ-3）。
///
/// 未知のフラグ・`--out` 欠落は `Err(usage メッセージ)` を返す。呼び出し元
/// （[`main`]）がそのまま stderr へ出力して非 0 終了する契約。
fn parse_args<I: Iterator<Item = String>>(mut args: I) -> Result<Args, String> {
    const USAGE: &str =
        "usage: docs-site --out <dir> [--root <dir>]\n\n  --out <dir>   output directory (required)\n  --root <dir>  repository root containing site/nav.toml (default: \".\")";

    let mut root: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => {
                let value = args
                    .next()
                    .ok_or_else(|| format!("--out requires a value\n\n{USAGE}"))?;
                out = Some(PathBuf::from(value));
            }
            "--root" => {
                let value = args
                    .next()
                    .ok_or_else(|| format!("--root requires a value\n\n{USAGE}"))?;
                root = Some(PathBuf::from(value));
            }
            other => {
                return Err(format!("unknown argument `{other}`\n\n{USAGE}"));
            }
        }
    }

    let out = out.ok_or_else(|| format!("missing required argument --out\n\n{USAGE}"))?;
    Ok(Args {
        root: root.unwrap_or_else(|| PathBuf::from(".")),
        out,
    })
}

fn main() -> ExitCode {
    // `args().skip(1)`: 先頭要素（実行ファイルパス）は引数パースの対象外。
    let parsed = match parse_args(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("fandhe-frontend-docs-site: {message}");
            return ExitCode::FAILURE;
        }
    };

    match build_site(&parsed.root, &parsed.out) {
        Ok(report) => {
            println!(
                "fandhe-frontend-docs-site: wrote {} page(s) and {} asset(s) to {}",
                report.written.len(),
                report.assets.len(),
                parsed.out.display()
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("fandhe-frontend-docs-site: build failed: {err}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_requires_out() {
        let err = parse_args(std::iter::empty()).unwrap_err();
        assert!(err.contains("missing required argument --out"));
    }

    #[test]
    fn parse_args_accepts_out_and_defaults_root() {
        let args = parse_args(vec!["--out".to_string(), "dist".to_string()].into_iter()).unwrap();
        assert_eq!(args.out, PathBuf::from("dist"));
        assert_eq!(args.root, PathBuf::from("."));
    }

    #[test]
    fn parse_args_accepts_out_and_root() {
        let args = parse_args(
            vec![
                "--root".to_string(),
                "fixture".to_string(),
                "--out".to_string(),
                "dist".to_string(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(args.root, PathBuf::from("fixture"));
        assert_eq!(args.out, PathBuf::from("dist"));
    }

    #[test]
    fn parse_args_rejects_unknown_flag() {
        let err = parse_args(vec!["--bogus".to_string()].into_iter()).unwrap_err();
        assert!(err.contains("unknown argument"));
    }

    #[test]
    fn parse_args_rejects_missing_value() {
        let err = parse_args(vec!["--out".to_string()].into_iter()).unwrap_err();
        assert!(err.contains("--out requires a value"));
    }
}
