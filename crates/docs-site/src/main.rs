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
//! （本番登録表）と `build_site_with`（`--no-page-sections` 時に空の登録表を渡す）
//! （`src/build.rs`）に置く。`tests/site_build.rs`（E2E テスト、
//! `env!("CARGO_BIN_EXE_docs-site")` 経由でこのバイナリを起動する）と
//! `build_site` を直接呼ぶ単体テストの双方から同一のビルドロジックを共有
//! するための分離である。
//!
//! CLI 引数（外部クレート clap 等は追加しない、REQ-3）:
//! - `--out <dir>`（必須）: 出力先ディレクトリ
//! - `--root <dir>`（任意、既定 `.`）: `<root>/site/nav.toml` を読むリポジトリ
//!   ルート。フィクスチャルートを渡す E2E テストのために存在する
//! - `--no-page-sections`（任意）: 本番の生成節登録表（本サイト専用のヒーロー・
//!   索引カード等）を使わず、空の登録表（`EMPTY_REGISTRY`）でビルドする。
//!   外部リポジトリの nav 向け（イシュー #3716）。未指定時は従来どおり本番登録表
//!   で、登録ページを持たない nav は書き出し前に fail-closed で失敗する。
//!   部品ページ・Blocks・Wireframes のショーケース注入と専用アセット出力も
//!   止まる（イシュー #3717）
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

use fandhe_frontend_docs_site::build::{build_site, build_site_with};
use fandhe_frontend_docs_site::page_sections::EMPTY_REGISTRY;

/// パース済み CLI 引数。
#[derive(Debug)]
struct Args {
    root: PathBuf,
    out: PathBuf,
    /// `true` のとき空の登録表でビルドする（既定 `false` = 本番登録表）。
    no_page_sections: bool,
}

/// `std::env::args` を手動パースする（外部クレート非依存、REQ-3）。
///
/// 未知のフラグ・`--out` 欠落は `Err(usage メッセージ)` を返す。呼び出し元
/// （[`main`]）がそのまま stderr へ出力して非 0 終了する契約。
fn parse_args<I: Iterator<Item = String>>(mut args: I) -> Result<Args, String> {
    const USAGE: &str =
        "usage: docs-site --out <dir> [--root <dir>] [--no-page-sections]\n\n  --out <dir>         output directory (required)\n  --root <dir>        repository root containing site/nav.toml (default: \".\")\n  --no-page-sections  build without the built-in page section registry and without the built-in showcases (component/blocks/wireframes pages) (for sites other than the fandhe-frontend docs)";

    let mut root: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut no_page_sections = false;

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
            "--no-page-sections" => no_page_sections = true,
            other => {
                return Err(format!("unknown argument `{other}`\n\n{USAGE}"));
            }
        }
    }

    let out = out.ok_or_else(|| format!("missing required argument --out\n\n{USAGE}"))?;
    Ok(Args {
        root: root.unwrap_or_else(|| PathBuf::from(".")),
        out,
        no_page_sections,
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

    // フラグなしは従来どおり `build_site` を呼び、本サイトの経路を一切変えない。
    let result = if parsed.no_page_sections {
        build_site_with(&parsed.root, &parsed.out, &EMPTY_REGISTRY)
    } else {
        build_site(&parsed.root, &parsed.out)
    };
    match result {
        Ok(report) => {
            println!(
                "fandhe-frontend-docs-site: wrote {} page(s), {} redirect(s) and {} asset(s) to {}",
                report.written.len(),
                report.redirects.len(),
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
        assert!(!args.no_page_sections);
    }

    #[test]
    fn parse_args_no_page_sections_does_not_consume_next_value() {
        let a = |v: &[&str]| parse_args(v.iter().map(|s| s.to_string()));
        let args = a(&["--no-page-sections", "--out", "dist", "--root", "fixture"]).unwrap();
        assert!(args.no_page_sections);
        assert_eq!(args.out, PathBuf::from("dist"));
        assert_eq!(args.root, PathBuf::from("fixture"));
        assert!(
            a(&["--out", "dist", "--no-page-sections"])
                .unwrap()
                .no_page_sections
        );
    }

    #[test]
    fn parse_args_rejects_valued_no_page_sections() {
        let err = parse_args(
            ["--out", "d", "--no-page-sections=false"]
                .iter()
                .map(|s| s.to_string()),
        )
        .unwrap_err();
        assert!(err.contains("unknown argument"));
    }

    #[test]
    fn usage_mentions_no_page_sections() {
        let err = parse_args(std::iter::empty()).unwrap_err();
        assert!(err.contains("--no-page-sections"));
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
