//! `fw` バイナリのエントリポイント。
//!
//! `structure` / `impact` / `gate` サブコマンドの実体は TASK-13.1c（#130）以降で
//! 接続される。本イシュー（TASK-13.1b #129）のスコープは `rws_cli::structure` の
//! パース・検証 API までであるため、ここでは usage 表示のみを行う骨格として置く。
#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args();
    let _program = args.next();
    let subcommand = args.next();

    match subcommand.as_deref() {
        // TASK-13.1c (#130) 以降でサブコマンドを接続する。
        Some("structure") | Some("impact") | Some("gate") | None => {
            print_usage();
            ExitCode::FAILURE
        }
        Some(other) => {
            eprintln!("fw: unknown subcommand `{other}`");
            print_usage();
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    eprintln!("usage: fw <structure|impact|gate> (not yet implemented)");
}
