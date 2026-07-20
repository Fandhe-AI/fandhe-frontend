//! `fandhe-frontend-docs-site` の起動エントリ。
//!
//! 公式 docs サイト（`docs/` 配下のドキュメントを本フレームワーク自身の SSG で
//! 静的サイトへ変換するもの）を生成するジェネレータの骨格。イシュー #465 の
//! スコープは骨格のみであり、`nav.toml` 読込・Markdown レンダリング・`dist/`
//! 出力（親イシュー #458 の Phase 2-3 以降）は後続イシューで実装する。
//!
//! `fandhe-frontend-core` / `fandhe-frontend-app` / `fandhe-frontend-server`
//! への依存（`structure.toml` の `depends_on` 宣言）は、後続実装で
//! `fandhe-frontend-server::ssg` 経由の静的生成を利用する前提として本クレートに
//! 事前宣言してある（本ファイル自体はまだ未参照）。
//!
//! 開発者・CI 用ツールであり、生成物は配布物（`fandhe-frontend-dist-server` 等）
//! には含めない（`structure.toml` の `role = "tooling"` 参照）。

#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    // 未実装のまま成功終了して空サイトを黙って公開する事故を防ぐため、
    // fail-closed（`security.md` A05 セキュリティ設定ミスの趣旨）で
    // 非 0 終了する。内部パス・スタックトレース等の機微情報は出力しない。
    eprintln!("fandhe-frontend-docs-site: not yet implemented (see issue #465 / #458)");
    ExitCode::FAILURE
}
