//! TASK-13.4 シナリオ共通ヘルパー（`cli/tests/scenarios/main.rs` 経由でのみ
//! コンパイルされる、統合テスト専用モジュール）。
//!
//! `cli/tests/negative_cases.rs`（TASK-13.5）で確立済みのヘルメチック・
//! フィクスチャパターン（`ScratchProject` による自動削除・`CARGO_TARGET_TMPDIR`
//! 配下への隔離・`CARGO_TARGET_DIR` 分離・`replace_unique` による欠陥注入・
//! `cargo_deny_available` による環境差吸収）をシナリオ統合テスト向けに
//! 踏襲する。シナリオ 2（#146）・シナリオ 3（#147）は本ファイルの関数群を
//! そのまま再利用してモジュール追加のみで合流できる契約とする
//! （重複統合は本イシューのスコープ外、`out-of-scope-tracking.md`）。
//!
//! # ヘルメチック性
//!
//! 全ヘルパーはネットワークアクセスを行わない。フィクスチャは外部依存ゼロの
//! path 依存クレートのみで構成し、`cargo generate-lockfile --offline` で
//! ロックファイルを決定的に生成する。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// `fw` バイナリを起動し、(終了コード, stdout, stderr) を返す。
///
/// `args` にはサブコマンドとその固有引数（例: `["gate"]` / `["impact", "render"]`）
/// を渡す。`--project <project_dir>` は本関数が付与するため呼び出し側は含めない。
///
/// `CARGO_TARGET_DIR` を `project_dir` 配下の専用 `target/` へ明示上書きする
/// （`negative_cases.rs::run_fw_gate` と同一方針）。self-hosted runner では
/// `CARGO_TARGET_DIR` がプロセス環境に既定で設定されており、これを継承した
/// まま `cargo` を起動すると、同名クレート（`rws-core` 等）を持つ他フィクス
/// チャとビルドキャッシュ/フィンガープリントが衝突し、直前に生成した別
/// フィクスチャのビルド結果を誤って再利用してしまう（注入したはずの欠陥が
/// 再コンパイルされず誤って PASS する偽陰性）。ここで上書きすることで各
/// フィクスチャを独立させる（`fw` から起動される `cargo` 子プロセスにも env は
/// 継承されるため、`gate.rs`/`impact.rs` 側の変更は不要）。
pub fn run_fw(args: &[&str], project_dir: &Path) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_fw"))
        .args(args)
        .arg("--project")
        .arg(project_dir)
        .env("CARGO_TARGET_DIR", project_dir.join("target"))
        .output()
        .expect("failed to spawn `fw` binary");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// `stdout`（`fw gate` の JSON レポート）中の `"name":"<name>"` エントリの
/// `passed` 値を判定する。該当エントリが見つからない場合は `None`
/// （「チェック自体が JSON に現れていない」ことと「passed:false」を区別する
/// ため、`bool` ではなく `Option<bool>` を返す。`negative_cases.rs` と同一実装）。
pub fn check_passed(stdout: &str, name: &str) -> Option<bool> {
    if stdout.contains(&format!("\"name\":\"{name}\",\"passed\":true")) {
        Some(true)
    } else if stdout.contains(&format!("\"name\":\"{name}\",\"passed\":false")) {
        Some(false)
    } else {
        None
    }
}

/// 実行環境に `cargo-deny` サブコマンドが導入済みかを判定する。
///
/// 本リポジトリ自身の CI（`.github/workflows/ci.yml`）は cargo-deny を
/// インストールしないため、`policy` チェックは CI 上では「cargo-deny 起動失敗
/// → failed（fail-closed）」となる。ローカル開発環境（導入済み）との差を
/// 吸収し、どちらの環境でも弱体化なしで取れる最強のアサーションを常時実行
/// するために使う（`negative_cases.rs` と同一実装、スキップ・`#[ignore]` は
/// 行わない）。
pub fn cargo_deny_available() -> bool {
    Command::new("cargo")
        .args(["deny", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 一時プロジェクトディレクトリを保持し、スコープを抜けるタイミングで
/// 自身を削除するガード（`negative_cases.rs::ScratchProject` と同一方針）。
pub struct ScratchProject(PathBuf);

impl std::ops::Deref for ScratchProject {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl Drop for ScratchProject {
    fn drop(&mut self) {
        // 削除失敗（他プロセスによるロック等）はテスト結果の正当性に
        // 影響しないため、ベストエフォートとして無視する。
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// 一時プロジェクトを書き出すスクラッチルート。`CARGO_TARGET_TMPDIR`
/// （cargo がテストバイナリ実行時に設定する target 配下の一時ディレクトリ）が
/// あればそこに閉じ、未設定環境向けに OS 標準の一時領域へフォールバックする
/// （パストラバーサル対策の一環、`negative_cases.rs` と同一パターン）。
pub fn scratch_root() -> PathBuf {
    std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
}

/// 一意な部分文字列 `from` を `to` へちょうど 1 箇所だけ置換する。複数箇所・
/// 0 箇所にマッチした場合は panic し、フィクスチャのリファクタリングで
/// 注入前提が崩れたことをテスト失敗として顕在化させる
/// （`negative_cases.rs::replace_unique` と同一実装）。
pub fn replace_unique(content: &str, from: &str, to: &str) -> String {
    assert_eq!(
        content.matches(from).count(),
        1,
        "注入対象の部分文字列 `{from}` が一意に見つからない（ベースラインの \
         リファクタリングでこのテストの前提が崩れている）"
    );
    let injected = content.replacen(from, to, 1);
    assert_ne!(content, injected, "置換後の内容が変化していない");
    injected
}

/// シナリオ 1（バグ修正、TASK-13.4b）用フィクスチャの `core/src/lib.rs`
/// ベースライン内容。
///
/// PoC-7 `target-project`（`docs/spec/03-poc/ai-self-maintenance/scenarios/
/// bugfix-escape-regression/`）が実測した「`rws-core` 相当のレンダリングコア」
/// を、依存ゼロ・ネイティブビルド可能な最小構成で再現する。`render`/`text`/
/// `escape_html` の 3 点を、実際の `rws-core` の責務（ノード木 API・
/// render・既定エスケープ、`docs/unsafe-boundary.md` の対象外＝安全な純 Rust）
/// と同じ形で持つ。
///
/// [`SINGLE_QUOTE_ESCAPE_ARM`] を注入対象の一意な部分文字列として公開し、
/// [`bugfix_escape`] シナリオがここへの置換でエスケープ回帰を再現する。
pub fn scenario1_core_lib_rs() -> &'static str {
    r#"//! シナリオ 1（TASK-13.4b, #145）フィクスチャ: `rws-core` 相当の最小
//! レンダリングコア。ノード木 API（`text`/`render`）と既定エスケープ
//! （REQ-1）である `escape_html` のみを持つ。
//!
//! `rws-app`（`app/`）・`rws-wasm-client`（`wasm-client/`）相当のフィクスチャ
//! クレートがここの `render`/`text` を呼び出す契約。`escape_html` の
//! エスケープ漏れは `text_node_is_escaped_by_default` の失敗として顕在化する
//! （`fw gate` の `test` チェックが検出、PoC-7 gate-before-fix.json 相当）。

pub enum Node {
    Text(String),
}

/// テキストノードを構築する。`render` を通した時点で必ず [`escape_html`]
/// を経由する（本フィクスチャに `raw_html()` 相当の迂回 API は存在しない）。
pub fn text(s: &str) -> Node {
    Node::Text(s.to_string())
}

/// 既定エスケープ（REQ-1）: `&` `<` `>` `"` `'` の 5 文字を HTML 実体参照へ
/// 変換する。この関数の出力はエスケープ済みであることを呼び出し元
/// （`render`、ひいては `rws-app`/`rws-wasm-client` 相当の各フィクスチャ）が
/// 前提とする契約。
pub fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            other => out.push(other),
        }
    }
    out
}

/// ノード木を HTML 文字列へレンダリングする。`rws-app`/`rws-wasm-client`
/// 相当のフィクスチャから呼ばれ、既定エスケープ済みの文字列を返す。
pub fn render(node: &Node) -> String {
    match node {
        Node::Text(s) => escape_html(s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// REQ-1 既定エスケープの回帰テスト。シングルクォートが `&#x27;` へ
    /// 変換されることを検証する（PoC-7 `bugfix-escape-regression` シナリオが
    /// 実測した回帰そのもの）。`fw gate` の `test` チェックがこの単体テストを
    /// 実行するため、シングルクォートのエスケープ漏れが混入すると
    /// 本テストの失敗として `fw gate` 全体を BLOCKED にする。
    #[test]
    fn text_node_is_escaped_by_default() {
        let rendered = render(&text("a'b<c>&\"d"));
        assert_eq!(rendered, "a&#x27;b&lt;c&gt;&amp;&quot;d");
    }
}
"#
}

/// [`scenario1_core_lib_rs`] 中の、シングルクォートを `&#x27;` へ変換する
/// match アームの一意な部分文字列。[`bugfix_escape`] シナリオが
/// [`replace_unique`] でここを「無変換で素通しする」内容へ置換し、
/// PoC-7 `bugfix-escape-regression` の回帰（`docs/spec/03-poc/
/// ai-self-maintenance/scenarios/bugfix-escape-regression/`）を再現する。
pub const SINGLE_QUOTE_ESCAPE_ARM: &str = "'\\'' => out.push_str(\"&#x27;\"),";

/// シングルクォートのエスケープを欠落させた（無変換で素通しする）置換後の
/// 内容。型・lint は無傷のまま `text_node_is_escaped_by_default` のみが
/// 失敗する構造にする（PoC-7 `gate-before-fix.json` と同じ失敗モード:
/// `type_check`/`lint`/`default_escape_check` は通過、`test` のみ failed）。
pub const SINGLE_QUOTE_ESCAPE_ARM_REGRESSED: &str = "'\\'' => out.push(c),";

/// シナリオ 1 用 `app/src/lib.rs`（`rws-app` 相当）。`rws-core` 相当の
/// `render`/`text` を呼び出す薄いコンポーネント層。`render` の使用箇所として
/// `fw impact render` の `affected_files`/`affected_crates` に現れる契約。
pub fn scenario1_app_lib_rs() -> &'static str {
    r#"//! シナリオ 1（TASK-13.4b, #145）フィクスチャ: `rws-app` 相当の
//! コンポーネント層。`rws-core` 相当クレート（`core/`）の `render`/`text` を
//! 呼び出し、一覧ページ相当の文字列を組み立てる。

use rws_core::{render, text};

/// 一覧ページ相当のレンダリング関数。`render` の直接の呼び出し元。
pub fn list_page(name: &str) -> String {
    render(&text(name))
}
"#
}

/// シナリオ 1 用 `wasm-client/src/lib.rs`（`rws-wasm-client` 相当）。
///
/// `cli/src/impact.rs::CLIENT_BOUNDARY_CRATES` に含まれるクレート名
/// （`rws-wasm-client`）と完全一致させることで、`render` の変更が
/// クライアント境界へ波及した場合の `breaking_risk: high` 判定
/// （`judge_breaking_risk`）を再現する。実際の `rws-wasm-full`/`rws-wasm-thin`
/// と異なり `wasm-bindgen` は使わない純ネイティブ lib とし、`fw gate` が
/// ネイティブ `cargo test`/`cargo check` で検証できるようにする
/// （wasm32 ターゲットのクロスビルドは本シナリオのスコープ外）。
pub fn scenario1_wasm_client_lib_rs() -> &'static str {
    r#"//! シナリオ 1（TASK-13.4b, #145）フィクスチャ: `rws-wasm-client` 相当の
//! クライアント境界層（ハイドレーション等の CSR 経路を模した薄い関数のみ）。
//! `wasm-bindgen` は使わず純ネイティブ lib として構成する
//! （`fw gate`/`fw impact` のネイティブ実行対象に含めるため）。

use rws_core::{render, text};

/// ハイドレーション時にラベルを再描画する相当の関数。`render` の直接の
/// 呼び出し元であり、`cli/src/impact.rs::CLIENT_BOUNDARY_CRATES` 判定の
/// 対象クレートからの利用を再現する。
pub fn hydrate_label(name: &str) -> String {
    render(&text(name))
}
"#
}

/// シナリオ 1 の 3 クレートワークスペース一式を一意な一時プロジェクト
/// ディレクトリへ書き出す:
///
/// ```text
/// <scratch>/scenario1-<pid>-<nanos>/
/// ├── structure.toml   ([directories.core]/[directories.app]/[directories.wasm-client])
/// ├── Cargo.toml       (virtual workspace, members = ["core", "app", "wasm-client"])
/// ├── deny.toml        (negative_cases.rs と同一ポリシーの最小版)
/// ├── clippy.toml      (rws_core::raw_html の disallowed-methods エントリ)
/// ├── core/            (name = "rws-core")
/// ├── app/              (name = "rws-app", core へ path 依存)
/// └── wasm-client/      (name = "rws-wasm-client", core へ path 依存)
/// ```
///
/// `core_lib_rs` を呼び出し側から差し替え可能にすることで、ベースライン
/// フィクスチャ書き出しと、エスケープ回帰を注入済みのフィクスチャ書き出しの
/// 両方に本関数 1 つで対応する。
///
/// `cargo generate-lockfile --offline` で `Cargo.lock` を生成する（path 依存
/// のみのため決定的・ネットワーク不要）。`fw gate` は `--locked` で `cargo`
/// サブコマンドを起動するため、ロックファイルなしでは各チェックがロック
/// ファイル欠落自体で failed になり、注入した欠陥とは無関係な失敗理由に
/// なってしまう（ケースの特定性を損なう）ため、ここで確実に用意する。
pub fn write_scenario1_project(label: &str, core_lib_rs: &str) -> ScratchProject {
    let dest = scratch_root().join(format!(
        "scenario1-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dest);

    let core_src = dest.join("core").join("src");
    let app_src = dest.join("app").join("src");
    let wasm_client_src = dest.join("wasm-client").join("src");
    fs::create_dir_all(&core_src).expect("core/src ディレクトリの作成に失敗した");
    fs::create_dir_all(&app_src).expect("app/src ディレクトリの作成に失敗した");
    fs::create_dir_all(&wasm_client_src).expect("wasm-client/src ディレクトリの作成に失敗した");

    fs::write(
        dest.join("structure.toml"),
        r#"
[manifest]
version = 1

[directories.core]
role = "core"
crate = "rws-core"
description = "TASK-13.4b scenario1 fixture: rendering core"
allowed_dependents = ["app", "wasm-client"]

[directories.app]
role = "component"
crate = "rws-app"
description = "TASK-13.4b scenario1 fixture: component layer"
depends_on = ["core"]

[directories.wasm-client]
role = "client-entrypoint"
crate = "rws-wasm-client"
description = "TASK-13.4b scenario1 fixture: client boundary layer"
depends_on = ["core"]
"#,
    )
    .expect("structure.toml の書き込みに失敗した");

    fs::write(
        dest.join("Cargo.toml"),
        "[workspace]\nmembers = [\"core\", \"app\", \"wasm-client\"]\nresolver = \"2\"\n",
    )
    .expect("workspace Cargo.toml の書き込みに失敗した");

    // `templates/default/deny.toml` と同じ主要ポリシー（bans/licenses/sources）
    // を持つ最小版（`negative_cases.rs::write_case_project` と同一内容）。
    fs::write(
        dest.join("deny.toml"),
        r#"[graph]
targets = []

[bans]
multiple-versions = "warn"
deny = [
    { name = "openssl-sys" },
]

[licenses]
allow = ["MIT", "Apache-2.0", "Unicode-3.0", "BSD-3-Clause"]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
"#,
    )
    .expect("deny.toml の書き込みに失敗した");

    // `gate.rs::clippy_policy_check` は `project_dir` 直下の `clippy.toml` に
    // `disallowed-methods` の `rws_core::raw_html` エントリが存在することを
    // fail-closed で前提とする（`negative_cases.rs` と同一内容）。
    fs::write(
        dest.join("clippy.toml"),
        r#"disallowed-methods = [
    { path = "rws_core::raw_html", reason = "REQ-1 の唯一のエスケープ迂回経路。レビュー済みの呼び出しには `#[expect(clippy::disallowed_methods, reason = \"ESCAPE-REVIEWED: <根拠>\")]` を呼び出し文へ直接付与すること（`#[allow(...)]` によるブランケット抑止は禁止、docs/raw-html-review-gate.md 参照）" },
]
"#,
    )
    .expect("clippy.toml の書き込みに失敗した");

    fs::write(
        dest.join("core").join("Cargo.toml"),
        "[package]\nname = \"rws-core\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n",
    )
    .expect("core/Cargo.toml の書き込みに失敗した");
    fs::write(core_src.join("lib.rs"), core_lib_rs).expect("core/src/lib.rs の書き込みに失敗した");

    fs::write(
        dest.join("app").join("Cargo.toml"),
        "[package]\nname = \"rws-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n\n[dependencies]\nrws-core = { path = \"../core\" }\n",
    )
    .expect("app/Cargo.toml の書き込みに失敗した");
    fs::write(app_src.join("lib.rs"), scenario1_app_lib_rs())
        .expect("app/src/lib.rs の書き込みに失敗した");

    fs::write(
        dest.join("wasm-client").join("Cargo.toml"),
        "[package]\nname = \"rws-wasm-client\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n\n[dependencies]\nrws-core = { path = \"../core\" }\n",
    )
    .expect("wasm-client/Cargo.toml の書き込みに失敗した");
    fs::write(
        wasm_client_src.join("lib.rs"),
        scenario1_wasm_client_lib_rs(),
    )
    .expect("wasm-client/src/lib.rs の書き込みに失敗した");

    // 依存は path 依存のみのためネットワークアクセスなしで決定的にロック
    // ファイルを生成できる。
    let lockfile_output = Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(&dest)
        .output()
        .expect("cargo generate-lockfile の起動に失敗した");
    assert!(
        lockfile_output.status.success(),
        "cargo generate-lockfile --offline に失敗した（フィクスチャ自体が壊れている）: {}",
        String::from_utf8_lossy(&lockfile_output.stderr)
    );

    ScratchProject(dest)
}
