//! `rws-server` の SSG エントリ（TASK-6.1c）。
//!
//! `--out <dir>` で指定したディレクトリ（既定 `target/ssg-out`）へ、
//! [`rws_server::ssg::generate`] を使って `/` と各デモアイテムの詳細ページを
//! 静的ファイルとして書き出す薄い CLI ラッパー。HTML 生成・出力パスの安全性
//! 検証は `rws_server::ssg` 側の責務であり、本ファイルは引数解析と結果表示
//! のみを担う。
//!
//! `#![forbid(unsafe_code)]` はクレートルートを跨いで継承されないため、
//! バイナリクレートルートである本ファイルにも明示的に付与する。

#![forbid(unsafe_code)]

use rws_server::ssg::generate;
use std::path::PathBuf;
use std::process::ExitCode;

const DEFAULT_OUT_DIR: &str = "target/ssg-out";

fn main() -> ExitCode {
    let out_dir = parse_out_dir(std::env::args().skip(1));

    match generate(&out_dir) {
        Ok(written) => {
            println!(
                "rws-server ssg: wrote {} file(s) to {out_dir:?}",
                written.len()
            );
            for path in &written {
                println!("  {path:?}");
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            // `SsgError` の `Display` は固定文言 + 入力パス/id のみで、内部
            // スタックトレース等は含まない（`security.md`「機微情報の露出」）。
            eprintln!("rws-server ssg: generation failed: {err}");
            ExitCode::FAILURE
        }
    }
}

/// `--out <dir>` 形式の引数から出力先ディレクトリを取り出す。未指定時は
/// [`DEFAULT_OUT_DIR`]。簡易パーサーのため CLI 引数解析ライブラリは追加しない
/// （REQ-3、外部依存ゼロを維持）。
fn parse_out_dir(mut args: impl Iterator<Item = String>) -> PathBuf {
    while let Some(arg) = args.next() {
        if arg == "--out" {
            if let Some(value) = args.next() {
                return PathBuf::from(value);
            }
        }
    }
    PathBuf::from(DEFAULT_OUT_DIR)
}

#[cfg(test)]
mod tests {
    use super::parse_out_dir;
    use std::path::PathBuf;

    #[test]
    fn parse_out_dir_reads_flag_value() {
        let args = vec!["--out".to_string(), "custom-dir".to_string()];
        assert_eq!(parse_out_dir(args.into_iter()), PathBuf::from("custom-dir"));
    }

    #[test]
    fn parse_out_dir_defaults_when_flag_missing() {
        let args: Vec<String> = vec![];
        assert_eq!(
            parse_out_dir(args.into_iter()),
            PathBuf::from(super::DEFAULT_OUT_DIR)
        );
    }
}
