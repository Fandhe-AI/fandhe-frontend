//! `cli/tests/scenarios/` 共有ハーネス（TASK-13.4a・#144、設計文書
//! `docs/design/scenario-regression-design.md` §4.2/§4.3）。
//!
//! PoC-7 が検証した代表的改修シナリオ（バグ修正・UI 改善・機能追加、
//! `docs/spec/03-poc/ai-self-maintenance/scenarios/`）を製品 CLI（`fw`）に
//! 対する統合テストとして再現するための、フィクスチャ生成・`fw` 起動・
//! JSON フィールド抽出の共通処理を提供する。
//!
//! `cli/tests/negative_cases.rs`（TASK-13.5・#262）が確立したヘルメチックな
//! フィクスチャ生成パターン（`ScratchProject` Drop ガード・
//! `cargo generate-lockfile --offline`・フィクスチャごとの `CARGO_TARGET_DIR`
//! 分離・cargo-deny 有無の環境差吸収）をそのまま踏襲する。統合テストは
//! ターゲット単位で独立コンパイルされるため cargo クレート間でのコード共有は
//! できず、`negative_cases.rs` とロジックが重複するが、これは意図的な複製
//! （テストターゲット独立の制約によるもの）であり、二重管理を避けるための
//! 抽出先は用意しない。
//!
//! 本ファイルはベースライン smoke test（`cli/tests/scenarios/main.rs`）に加え、
//! TASK-13.4b（#145、シナリオ 1: バグ修正）の
//! `cli/tests/scenarios/bugfix_escape.rs` が利用する。後続 TASK-13.4c/d
//! （#146〜#147）もシナリオ 2・3 固有のフィクスチャ拡張・`fw impact` JSON
//! 検証にこのハーネスを利用する契約（設計文書 §4.4）。

#![allow(dead_code)] // ベースライン smoke test・シナリオ 1 は一部のヘルパのみ使用する。残りは #146〜#147 が利用する契約。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// `write_scenario_project`/`write_scenario1_project` が書き出した一時プロジェクト
/// ディレクトリを保持し、スコープを抜けるタイミングで自身を削除するガード
/// （`negative_cases.rs::ScratchProject` と同一方針）。
pub struct ScenarioProject(PathBuf);

impl std::ops::Deref for ScenarioProject {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl Drop for ScenarioProject {
    fn drop(&mut self) {
        // 削除失敗（他プロセスによるロック等）はテスト結果の正当性に
        // 影響しないため、ベストエフォートとして無視する。
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// 一時プロジェクトを書き出すスクラッチルート。`CARGO_TARGET_TMPDIR`
/// （cargo がテストバイナリ実行時に設定する target 配下の一時ディレクトリ）が
/// あればそこに閉じ、未設定環境向けに OS 標準の一時領域へフォールバックする
/// （`negative_cases.rs` と同一パターン、パストラバーサル対策の一環）。
pub fn scratch_root() -> PathBuf {
    std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
}

/// ベースライン（正例）となる `app/src/main.rs` の内容。PoC-7
/// `target-project`（`Item` / `find_item`）相当の最小構成で、依存ゼロ・
/// clippy クリーン・`raw_html` 文字列を一切含まない
/// （`negative_cases.rs::baseline_main_rs` と同一内容。本サブタスクの
/// ベースライン smoke test 専用であり、シナリオ 1〜3 固有のフィクスチャ
/// 拡張（`server` クレート・ルート定義の追加等）は #145〜#147 が
/// このハーネスに追加する契約、設計文書 §4.2）。
pub fn baseline_main_rs() -> &'static str {
    r#"struct Item {
    id: String,
    name: String,
}

fn find_item<'a>(items: &'a [Item], target_id: &str) -> Option<&'a Item> {
    items.iter().find(|it| it.id == target_id)
}

fn main() {
    let items = vec![
        Item {
            id: "1".to_string(),
            name: "widget".to_string(),
        },
        Item {
            id: "2".to_string(),
            name: "gadget".to_string(),
        },
    ];
    if let Some(item) = find_item(&items, "1") {
        println!("found: {}", item.name);
    }
}
"#
}

/// 一意な一時プロジェクトディレクトリに以下を書き出す:
///
/// ```text
/// <fixture>/
/// ├── structure.toml   ([directories.app], role = "component")
/// ├── Cargo.toml       (virtual workspace, members = ["app"])
/// ├── deny.toml        (templates/default/deny.toml と同ポリシーの最小版)
/// ├── clippy.toml      (disallowed-methods: fandhe_frontend_core::raw_html)
/// └── app/
///     ├── Cargo.toml   (name = "scenario-fixture-app", 依存ゼロ)
///     └── src/main.rs  (main_rs_content)
/// ```
///
/// `cargo generate-lockfile --offline` で `Cargo.lock` を生成する（依存ゼロの
/// ため決定的・ネットワーク不要）。`fw gate` は `--locked` で `cargo`
/// サブコマンドを起動するため、ロックファイルなしでは各チェックがロック
/// ファイル欠落自体で failed になり、注入した欠陥とは無関係な失敗理由に
/// なってしまう（ケースの特定性を損なう）ため、ここで確実に用意する
/// （`negative_cases.rs::write_case_project` と同一方針）。
///
/// `scenario_name` はスクラッチディレクトリ名の一意化のみに使う（ファイル内容には
/// 影響しない）。
pub fn write_scenario_project(scenario_name: &str, main_rs_content: &str) -> ScenarioProject {
    let dest = scratch_root().join(format!(
        "scenario-{scenario_name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dest);
    let app_src = dest.join("app").join("src");
    fs::create_dir_all(&app_src).expect("一時プロジェクトディレクトリの作成に失敗した");

    fs::write(
        dest.join("structure.toml"),
        r#"
[manifest]
version = 1

[directories.app]
role = "component"
crate = "scenario-fixture-app"
description = "TASK-13.4 scenario regression fixture"
"#,
    )
    .expect("structure.toml の書き込みに失敗した");

    fs::write(
        dest.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    )
    .expect("workspace Cargo.toml の書き込みに失敗した");

    // `templates/default/deny.toml` と同じ主要ポリシー（bans/licenses/sources）
    // を持つ最小版。`policy` チェックが `deny.toml` 実在確認の先で実際に
    // `cargo deny check bans licenses sources` を走らせられるようにする
    // （`negative_cases.rs` と同一内容）。
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

    fs::write(
        dest.join("app").join("Cargo.toml"),
        "[package]\nname = \"scenario-fixture-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n",
    )
    .expect("app/Cargo.toml の書き込みに失敗した");

    // イシュー #157/#263（`gate.rs::clippy_policy_check`）: `lint` チェックは
    // `project_dir` 直下の `clippy.toml` に `disallowed-methods` の
    // `fandhe_frontend_core::raw_html` エントリが存在することを fail-closed で前提とする
    // （欠落時は cargo clippy を起動する前に `lint` を failed とする）。
    fs::write(
        dest.join("clippy.toml"),
        r#"disallowed-methods = [
    { path = "fandhe_frontend_core::raw_html", reason = "REQ-1 の唯一のエスケープ迂回経路。レビュー済みの呼び出しには `#[expect(clippy::disallowed_methods, reason = \"ESCAPE-REVIEWED: <根拠>\")]` を呼び出し文へ直接付与すること（`#[allow(...)]` によるブランケット抑止は禁止、docs/policy/raw-html-review-gate.md 参照）" },
]
"#,
    )
    .expect("clippy.toml の書き込みに失敗した");

    fs::write(app_src.join("main.rs"), main_rs_content).expect("main.rs の書き込みに失敗した");

    // 依存ゼロのためネットワークアクセスなしで決定的にロックファイルを生成できる。
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

    ScenarioProject(dest)
}

/// シナリオ 1（バグ修正、TASK-13.4b）用フィクスチャの `core/src/lib.rs`
/// ベースライン内容。
///
/// PoC-7 `target-project`（`docs/spec/03-poc/ai-self-maintenance/scenarios/
/// bugfix-escape-regression/`）が実測した「`fandhe-frontend-core` 相当のレンダリングコア」
/// を、依存ゼロ・ネイティブビルド可能な最小構成で再現する。`render`/`text`/
/// `escape_html` の 3 点を、実際の `fandhe-frontend-core` の責務（ノード木 API・
/// render・既定エスケープ、`docs/policy/unsafe-boundary.md` の対象外＝安全な純 Rust）
/// と同じ形で持つ。
///
/// [`SINGLE_QUOTE_ESCAPE_ARM`] を注入対象の一意な部分文字列として公開し、
/// [`bugfix_escape`](crate::bugfix_escape) シナリオがここへの置換で
/// エスケープ回帰を再現する。
pub fn scenario1_core_lib_rs() -> &'static str {
    r#"//! シナリオ 1（TASK-13.4b, #145）フィクスチャ: `fandhe-frontend-core` 相当の最小
//! レンダリングコア。ノード木 API（`text`/`render`）と既定エスケープ
//! （REQ-1）である `escape_html` のみを持つ。
//!
//! `fandhe-frontend-app`（`app/`）・`fandhe-frontend-wasm-client`（`wasm-client/`）相当のフィクスチャ
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
/// （`render`、ひいては `fandhe-frontend-app`/`fandhe-frontend-wasm-client` 相当の各フィクスチャ）が
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

/// ノード木を HTML 文字列へレンダリングする。`fandhe-frontend-app`/`fandhe-frontend-wasm-client`
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
/// match アームの一意な部分文字列。[`bugfix_escape`](crate::bugfix_escape)
/// シナリオが [`replace_unique`] でここを「無変換で素通しする」内容へ置換し、
/// PoC-7 `bugfix-escape-regression` の回帰（`docs/spec/03-poc/
/// ai-self-maintenance/scenarios/bugfix-escape-regression/`）を再現する。
pub const SINGLE_QUOTE_ESCAPE_ARM: &str = "'\\'' => out.push_str(\"&#x27;\"),";

/// シングルクォートのエスケープを欠落させた（無変換で素通しする）置換後の
/// 内容。型・lint は無傷のまま `text_node_is_escaped_by_default` のみが
/// 失敗する構造にする（PoC-7 `gate-before-fix.json` と同じ失敗モード:
/// `type_check`/`lint`/`default_escape_check` は通過、`test` のみ failed）。
pub const SINGLE_QUOTE_ESCAPE_ARM_REGRESSED: &str = "'\\'' => out.push(c),";

/// シナリオ 1 フィクスチャ専用の `core/src/url.rs`（イシュー #401、
/// `url_validation_check` U2/U3 の充足専用）。`core/src/url.rs`（実 `fandhe-frontend-core`）
/// の `URL_ATTRS`（12 属性ピン）・ガード関数 4 種の定義・呼び出しを最小構成で
/// 再現する。`lib.rs` から `mod url;` されないため実際のクレートには含まれず
/// （[`write_scenario1_project`] のコメント参照）、シナリオ 1 の型・lint・
/// テストの挙動には一切影響しない。
const SCENARIO1_CORE_URL_VALIDATION_FIXTURE: &str = r#"//! イシュー #401 対応: `url_validation_check` の U2/U3 充足専用フィクスチャ。
//! `lib.rs` から `mod` 宣言されないため実クレートには含まれない
//! （`write_scenario1_project` コメント参照）。

pub const URL_ATTRS: &[&str] = &[
    "href",
    "src",
    "action",
    "formaction",
    "xlink:href",
    "poster",
    "cite",
    "data",
    "background",
    "ping",
    "dynsrc",
    "lowsrc",
];

pub fn is_url_attr(name: &str) -> bool {
    URL_ATTRS.iter().any(|a| a.eq_ignore_ascii_case(name))
}

pub fn is_event_handler_attr(name: &str) -> bool {
    name.len() > 2
        && name.as_bytes()[0].eq_ignore_ascii_case(&b'o')
        && name.as_bytes()[1].eq_ignore_ascii_case(&b'n')
}

pub fn is_safe_url(value: &str) -> bool {
    match extract_scheme(value) {
        None => true,
        Some(scheme) => {
            scheme.eq_ignore_ascii_case("http")
                || scheme.eq_ignore_ascii_case("https")
                || scheme.eq_ignore_ascii_case("mailto")
                || scheme.eq_ignore_ascii_case("tel")
        }
    }
}

fn extract_scheme(s: &str) -> Option<&str> {
    let colon_idx = s.find(':')?;
    Some(&s[..colon_idx])
}

pub fn is_safe_srcset(value: &str) -> bool {
    value.split(',').all(|candidate| {
        let url_part = candidate.split_whitespace().next().unwrap_or("");
        is_safe_url(url_part)
    })
}

/// U3（ガード呼び出し実在チェック）充足用の自己完結した呼び出し口。
pub fn __gate_self_check(name: &str, value: &str) -> bool {
    if is_event_handler_attr(name) {
        return false;
    }
    if is_url_attr(name) {
        return is_safe_url(value);
    }
    is_safe_srcset(value)
}
"#;

/// シナリオ 1 用 `app/src/lib.rs`（`fandhe-frontend-app` 相当）。`fandhe-frontend-core` 相当の
/// `render`/`text` を呼び出す薄いコンポーネント層。`render` の使用箇所として
/// `fw impact render` の `affected_files`/`affected_crates` に現れる契約。
pub fn scenario1_app_lib_rs() -> &'static str {
    r#"//! シナリオ 1（TASK-13.4b, #145）フィクスチャ: `fandhe-frontend-app` 相当の
//! コンポーネント層。`fandhe-frontend-core` 相当クレート（`core/`）の `render`/`text` を
//! 呼び出し、一覧ページ相当の文字列を組み立てる。

use fandhe_frontend_core::{render, text};

/// 一覧ページ相当のレンダリング関数。`render` の直接の呼び出し元。
pub fn list_page(name: &str) -> String {
    render(&text(name))
}
"#
}

/// シナリオ 1 用 `wasm-client/src/lib.rs`（`fandhe-frontend-wasm-client` 相当）。
///
/// `cli/src/impact.rs::CLIENT_BOUNDARY_CRATES` に含まれるクレート名
/// （`fandhe-frontend-wasm-client`）と完全一致させることで、`render` の変更が
/// クライアント境界へ波及した場合の `breaking_risk: high` 判定
/// （`judge_breaking_risk`）を再現する。実際の `fandhe-frontend-wasm-full`/`fandhe-frontend-wasm-thin`
/// と異なり `wasm-bindgen` は使わない純ネイティブ lib とし、`fw gate` が
/// ネイティブ `cargo test`/`cargo check` で検証できるようにする
/// （wasm32 ターゲットのクロスビルドは本シナリオのスコープ外）。
pub fn scenario1_wasm_client_lib_rs() -> &'static str {
    r#"//! シナリオ 1（TASK-13.4b, #145）フィクスチャ: `fandhe-frontend-wasm-client` 相当の
//! クライアント境界層（ハイドレーション等の CSR 経路を模した薄い関数のみ）。
//! `wasm-bindgen` は使わず純ネイティブ lib として構成する
//! （`fw gate`/`fw impact` のネイティブ実行対象に含めるため）。

use fandhe_frontend_core::{render, text};

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
/// <scratch>/scenario1-<label>-<pid>-<nanos>/
/// ├── structure.toml   ([directories.core]/[directories.app]/[directories.wasm-client])
/// ├── Cargo.toml       (virtual workspace, members = ["core", "app", "wasm-client"])
/// ├── deny.toml        (write_scenario_project と同一ポリシーの最小版)
/// ├── clippy.toml      (fandhe_frontend_core::raw_html の disallowed-methods エントリ)
/// ├── core/            (name = "fandhe-frontend-core")
/// ├── app/              (name = "fandhe-frontend-app", core へ path 依存)
/// └── wasm-client/      (name = "fandhe-frontend-wasm-client", core へ path 依存)
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
/// なってしまう（ケースの特定性を損なう）ため、ここで確実に用意する
/// （`write_scenario_project`/`negative_cases.rs::write_case_project` と
/// 同一方針）。
pub fn write_scenario1_project(label: &str, core_lib_rs: &str) -> ScenarioProject {
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
crate = "fandhe-frontend-core"
description = "TASK-13.4b scenario1 fixture: rendering core"
allowed_dependents = ["app", "wasm-client"]

[directories.app]
role = "component"
crate = "fandhe-frontend-app"
description = "TASK-13.4b scenario1 fixture: component layer"
depends_on = ["core"]

[directories.wasm-client]
role = "client-entrypoint"
crate = "fandhe-frontend-wasm-client"
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
    // を持つ最小版（`write_scenario_project`/`negative_cases.rs` と同一内容）。
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
    // `disallowed-methods` の `fandhe_frontend_core::raw_html` エントリが存在することを
    // fail-closed で前提とする（`write_scenario_project` と同一内容）。
    fs::write(
        dest.join("clippy.toml"),
        r#"disallowed-methods = [
    { path = "fandhe_frontend_core::raw_html", reason = "REQ-1 の唯一のエスケープ迂回経路。レビュー済みの呼び出しには `#[expect(clippy::disallowed_methods, reason = \"ESCAPE-REVIEWED: <根拠>\")]` を呼び出し文へ直接付与すること（`#[allow(...)]` によるブランケット抑止は禁止、docs/policy/raw-html-review-gate.md 参照）" },
]
"#,
    )
    .expect("clippy.toml の書き込みに失敗した");

    fs::write(
        dest.join("core").join("Cargo.toml"),
        "[package]\nname = \"fandhe-frontend-core\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n",
    )
    .expect("core/Cargo.toml の書き込みに失敗した");
    fs::write(core_src.join("lib.rs"), core_lib_rs).expect("core/src/lib.rs の書き込みに失敗した");

    // イシュー #401（`url_validation_check`）: `[directories.core]` に
    // `role = "core"` を宣言すると、`fw gate` は当該ディレクトリの src/ に
    // `URL_ATTRS` 定義・URL 検証ガード関数 4 種の定義/呼び出しが実在する
    // ことを要求する（U2/U3、fail-closed）。本シナリオ 1 フィクスチャは
    // イシュー #373 以前から存在するエスケープ回帰シナリオ専用の最小
    // `fandhe-frontend-core` スタンドインであり、`core_lib_rs`（シングルクォート
    // エスケープの注入対象、[`SINGLE_QUOTE_ESCAPE_ARM`]）を汚染せずに
    // 新しい gate 不変条件を満たす必要がある。そのため `lib.rs` からは
    // `mod` 宣言しない独立ファイルとして `url.rs` を追加する: `mod` 宣言が
    // ないため `cargo check`/`clippy`/`test`（型チェック・lint・テスト
    // チェック）のコンパイル対象には含まれず（Rust はクレートルートから
    // 到達可能な `mod` のみをコンパイルする）、`url_validation_check`
    // （ファイルシステム走査ベースで `mod` 宣言の有無を問わない）のみが
    // これを検出する。
    fs::write(
        core_src.join("url.rs"),
        SCENARIO1_CORE_URL_VALIDATION_FIXTURE,
    )
    .expect("core/src/url.rs の書き込みに失敗した");

    fs::write(
        dest.join("app").join("Cargo.toml"),
        "[package]\nname = \"fandhe-frontend-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n\n[dependencies]\nfandhe-frontend-core = { path = \"../core\" }\n",
    )
    .expect("app/Cargo.toml の書き込みに失敗した");
    fs::write(app_src.join("lib.rs"), scenario1_app_lib_rs())
        .expect("app/src/lib.rs の書き込みに失敗した");

    fs::write(
        dest.join("wasm-client").join("Cargo.toml"),
        "[package]\nname = \"fandhe-frontend-wasm-client\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n\n[dependencies]\nfandhe-frontend-core = { path = \"../core\" }\n",
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

    ScenarioProject(dest)
}

/// 一意な部分文字列 `from` を `to` へちょうど 1 箇所だけ置換する。複数箇所・
/// 0 箇所にマッチした場合は panic し、フィクスチャのリファクタリングで
/// 注入前提が崩れたことをテスト失敗として顕在化させる
/// （`negative_cases.rs::replace_unique` と同一方針。シナリオ 1〜3 の
/// before/after 変更適用に #145〜#147 が使う契約）。
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

/// `fw` バイナリを任意のサブコマンド（`structure` / `gate` / `impact`）で
/// 起動し、(終了コード, stdout, stderr) を返す。`--project <dir>` を固定で
/// 付与し、`CARGO_TARGET_DIR` をフィクスチャ配下の専用ディレクトリへ上書きする
/// （`negative_cases.rs::run_fw_gate` と同一方針。self-hosted runner 等で
/// 継承された `CARGO_TARGET_DIR` をそのまま使うと、同名パッケージを使う
/// 複数フィクスチャ間でビルドキャッシュ/フィンガープリントが衝突し、直前の
/// フィクスチャの結果を誤って再利用してしまう偽陰性を招くため、フィクスチャ
/// ごとに独立させる）。
///
/// `extra_args` はサブコマンド固有の追加引数（例: `fw impact <symbol>` の
/// `<symbol>`）を `--project` より前に渡す。
pub fn run_fw(subcommand: &str, extra_args: &[&str], project_dir: &Path) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_fw"))
        .arg(subcommand)
        .args(extra_args)
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

/// `write_workspace_project` に渡す 1 crate 分のフィクスチャ内容。
///
/// シナリオ 2（UI 改善、TASK-13.4c・#146）で複数クレート（`fandhe-frontend-app` /
/// `fandhe-frontend-server` / `fandhe-frontend-wasm-client` 等）にまたがるワークスペースを組み立てる
/// ために導入した汎用型。設計文書 §4.4「`common.rs` をシナリオ数だけ
/// 分岐させない」方針に従い、シナリオ固有の特殊化を持ち込まず、ソース内容・
/// クレート構成は呼び出し側（`scenario{1,2,3}_*.rs`）からパラメータとして
/// 渡す（後続 #145/#147 も同じ型を再利用する契約）。
pub struct MemberFixture<'a> {
    /// ワークスペースルート直下のディレクトリ名（`structure.toml` の
    /// `directories` キー・cargo workspace member 名と一致させる）。
    pub dir_name: &'a str,
    /// `<dir_name>/Cargo.toml` の内容（`[package] name = "..."` を含む）。
    pub cargo_toml: &'a str,
    /// `<dir_name>/src/` 配下に書き出す `(相対パス, 内容)` の一覧
    /// （例: `("lib.rs", "pub fn ...")`）。`'static` に固定しないのは、
    /// `common::replace_unique` が返す所有 `String`（`before`/`after` 変更
    /// 適用後の内容）をリークせずそのまま借用させるため（呼び出し側の
    /// スタックフレーム内で完結させる、設計文書 §4.4）。
    pub src_files: &'a [(&'a str, &'a str)],
}

/// 複数クレートワークスペースの一時プロジェクトを書き出す汎用ビルダー
/// （`write_scenario_project` の単一クレート版を拡張したもの。設計文書 §4.2
/// 「拡張指針」に従い、既存のベースライン smoke test 用フィクスチャ生成とは
/// 独立させ、両者を同時に保守可能にする）。
///
/// ```text
/// <fixture>/
/// ├── structure.toml   (呼び出し側が渡す structure_toml_content)
/// ├── Cargo.toml       (virtual workspace, members = members の dir_name 一覧)
/// ├── deny.toml / clippy.toml  (write_scenario_project と同一内容)
/// └── <member.dir_name>/
///     ├── Cargo.toml   (member.cargo_toml)
///     └── src/<file>   (member.src_files の各エントリ)
/// ```
///
/// `write_scenario_project` と同じく `cargo generate-lockfile --offline` で
/// `Cargo.lock` を生成する（各 member 間の依存は path 依存のみのため registry
/// 不要・決定的に成功する）。
pub fn write_workspace_project(
    scenario_name: &str,
    structure_toml: &str,
    members: &[MemberFixture<'_>],
) -> ScenarioProject {
    let dest = scratch_root().join(format!(
        "scenario-{scenario_name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dest);
    fs::create_dir_all(&dest).expect("一時プロジェクトディレクトリの作成に失敗した");

    fs::write(dest.join("structure.toml"), structure_toml)
        .expect("structure.toml の書き込みに失敗した");

    let members_list = members
        .iter()
        .map(|m| format!("\"{}\"", m.dir_name))
        .collect::<Vec<_>>()
        .join(", ");
    fs::write(
        dest.join("Cargo.toml"),
        format!("[workspace]\nmembers = [{members_list}]\nresolver = \"2\"\n"),
    )
    .expect("workspace Cargo.toml の書き込みに失敗した");

    // `deny.toml` / `clippy.toml` は `write_scenario_project` と同一内容
    // （`gate.rs::policy_check` / `clippy_policy_check` が前提とするポリシー
    // 設定。フィクスチャ間で二重管理しないよう、内容はここで固定する）。
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

    fs::write(
        dest.join("clippy.toml"),
        r#"disallowed-methods = [
    { path = "fandhe_frontend_core::raw_html", reason = "REQ-1 の唯一のエスケープ迂回経路。レビュー済みの呼び出しには `#[expect(clippy::disallowed_methods, reason = \"ESCAPE-REVIEWED: <根拠>\")]` を呼び出し文へ直接付与すること（`#[allow(...)]` によるブランケット抑止は禁止、docs/policy/raw-html-review-gate.md 参照）" },
]
"#,
    )
    .expect("clippy.toml の書き込みに失敗した");

    for member in members {
        let member_src = dest.join(member.dir_name).join("src");
        fs::create_dir_all(&member_src).expect("member src ディレクトリの作成に失敗した");
        fs::write(
            dest.join(member.dir_name).join("Cargo.toml"),
            member.cargo_toml,
        )
        .expect("member Cargo.toml の書き込みに失敗した");
        for (rel_path, content) in member.src_files {
            fs::write(member_src.join(rel_path), content)
                .expect("member ソースファイルの書き込みに失敗した");
        }
    }

    // 各 member 間は path 依存のみのためネットワークアクセスなしで決定的に
    // ロックファイルを生成できる（`write_scenario_project` と同一方針）。
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

    ScenarioProject(dest)
}

/// `stdout`（`fw gate` の JSON レポート）中の `"name":"<name>"` エントリの
/// `passed` 値を判定する。該当エントリが見つからない場合は `None`
/// （「チェック自体が JSON に現れていない」ことと「passed:false」を区別する
/// ため、`bool` ではなく `Option<bool>` を返す。`negative_cases.rs::check_passed`
/// と同一実装）。
pub fn check_passed(stdout: &str, name: &str) -> Option<bool> {
    if stdout.contains(&format!("\"name\":\"{name}\",\"passed\":true")) {
        Some(true)
    } else if stdout.contains(&format!("\"name\":\"{name}\",\"passed\":false")) {
        Some(false)
    } else {
        None
    }
}

/// 実行環境に `cargo-deny` サブコマンドが導入済みかを判定する
/// （リポジトリ自身の CI には未導入、ローカル開発環境には導入済みという
/// 差を吸収するための補助関数。`negative_cases.rs::cargo_deny_available`
/// と同一実装。設計文書 §4.3 の環境差吸収方針を参照）。
pub fn cargo_deny_available() -> bool {
    Command::new("cargo")
        .args(["deny", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// `fw impact` の JSON レポート中の文字列フィールド
/// `"<field>":"<value>"` を抽出する。専用 JSON パーサ依存を持ち込まず、
/// `check_passed` と同じ「文字列走査による軽量抽出」方針を踏襲する
/// （`cli` の外部依存ゼロを維持、設計文書 §4.3）。フィールドが見つからない
/// 場合は `None`（欠落と空文字列を区別する）。
///
/// `breaking_risk`（`"high"`/`"medium"`/`"low"`）等、値に `"` を含まない
/// フィールドの抽出に使う。#145〜#147 が `fw impact` の JSON アサーションに
/// 利用する契約。
pub fn json_string_field(stdout: &str, field: &str) -> Option<String> {
    let needle = format!("\"{field}\":\"");
    let start = stdout.find(&needle)? + needle.len();
    let rest = &stdout[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// `fw impact` の JSON レポート中の真偽値フィールド
/// `"<field>":true`/`"<field>":false` を抽出する
/// （`requires_human_approval` / `ambiguous` 等）。フィールドが見つからない
/// 場合は `None`。
pub fn json_bool_field(stdout: &str, field: &str) -> Option<bool> {
    if stdout.contains(&format!("\"{field}\":true")) {
        Some(true)
    } else if stdout.contains(&format!("\"{field}\":false")) {
        Some(false)
    } else {
        None
    }
}

/// 複数クレート構成フィクスチャの workspace member 1 件分の仕様
/// （TASK-13.4d・#147 が導入。設計文書 §4.2「フィクスチャ拡張指針」に基づく
/// 汎用形で、シナリオ固有の特殊化を持ち込まない契約とする。#145/#146
/// （シナリオ 1・2）が複数クレート構成を必要とする場合はこの型・
/// [`write_scenario_workspace`] を再利用し、`common.rs` を複製・分岐させない）。
pub struct MemberSpec {
    /// ワークスペースルート直下のディレクトリ名（`structure.toml` の
    /// `directories.<dir>` キーと一致させる、`^[a-z0-9_-]+$` を満たすこと）。
    pub dir: &'static str,
    /// `Cargo.toml` の `package.name`（`structure.toml` の `crate` フィールドと
    /// 一致させる）。
    pub package_name: &'static str,
    /// `structure.toml` の `role`（`"component"` / `"server-entrypoint"` 等、
    /// `cli/src/structure.rs::Role` が受理する文字列）。
    pub role: &'static str,
    /// `true` なら `src/main.rs`（bin クレート）、`false` なら `src/lib.rs`
    /// （lib クレート）として書き出す。
    pub is_bin: bool,
    /// このメンバーが path 依存する他メンバーの `dir` 一覧（同じ `members`
    /// スライス内に存在する必要がある）。
    pub path_deps: &'static [&'static str],
    /// `src/{lib,main}.rs` に書き出すソース全文。呼び出し側（シナリオ固有の
    /// テストファイル）が `replace_unique` 等で before/after を組み立てる。
    pub source: String,
}

/// [`write_scenario_project`]（単一 `app` クレート構成）の複数クレート版。
/// 以下を書き出す:
///
/// ```text
/// <fixture>/
/// ├── structure.toml   ([directories.<dir>] を members の数だけ、
/// │                     routing が Some なら [routing] も追加)
/// ├── Cargo.toml       (virtual workspace, members = members の dir 一覧)
/// ├── deny.toml / clippy.toml  (write_scenario_project と同一内容)
/// └── <dir>/           (members の各エントリにつき 1 クレート)
///     ├── Cargo.toml   (path_deps は `../<dep_dir>` として依存宣言)
///     └── src/{lib,main}.rs
/// ```
///
/// `routing` は `(definition_dir, extractor)` のペア。シナリオ 1〜3
/// （#145〜#147）はいずれもルート定義を伴う複数クレート構成を必要とするため
/// 引数化するが、ルート定義が不要なシナリオは `None` を渡せる。
///
/// `write_scenario_project` と同じく `cargo generate-lockfile --offline` で
/// ロックファイルを生成する（path 依存のみのためネットワーク不要・決定的）。
pub fn write_scenario_workspace(
    scenario_name: &str,
    members: &[MemberSpec],
    routing: Option<(&str, &str)>,
) -> ScenarioProject {
    let dest = scratch_root().join(format!(
        "scenario-ws-{scenario_name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dest);
    fs::create_dir_all(&dest).expect("一時プロジェクトディレクトリの作成に失敗した");

    let mut structure_toml = String::from("\n[manifest]\nversion = 1\n\n");
    for m in members {
        structure_toml.push_str(&format!(
            "[directories.{dir}]\nrole = \"{role}\"\ncrate = \"{pkg}\"\ndescription = \"TASK-13.4 scenario regression fixture (multi-crate)\"\n\n",
            dir = m.dir,
            role = m.role,
            pkg = m.package_name,
        ));
    }
    if let Some((definition_dir, extractor)) = routing {
        structure_toml.push_str(&format!(
            "[routing]\ndefinition_dir = \"{definition_dir}\"\nextractor = \"{extractor}\"\n"
        ));
    }
    fs::write(dest.join("structure.toml"), structure_toml)
        .expect("structure.toml の書き込みに失敗した");

    let members_list = members
        .iter()
        .map(|m| format!("\"{}\"", m.dir))
        .collect::<Vec<_>>()
        .join(", ");
    fs::write(
        dest.join("Cargo.toml"),
        format!("[workspace]\nmembers = [{members_list}]\nresolver = \"2\"\n"),
    )
    .expect("workspace Cargo.toml の書き込みに失敗した");

    // deny.toml / clippy.toml は write_scenario_project と同一内容（二重管理を
    // 避けるため本来は共有したいが、テストターゲット独立の制約により複製する）。
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

    fs::write(
        dest.join("clippy.toml"),
        r#"disallowed-methods = [
    { path = "fandhe_frontend_core::raw_html", reason = "REQ-1 の唯一のエスケープ迂回経路。レビュー済みの呼び出しには `#[expect(clippy::disallowed_methods, reason = \"ESCAPE-REVIEWED: <根拠>\")]` を呼び出し文へ直接付与すること（`#[allow(...)]` によるブランケット抑止は禁止、docs/policy/raw-html-review-gate.md 参照）" },
]
"#,
    )
    .expect("clippy.toml の書き込みに失敗した");

    for m in members {
        let member_dir = dest.join(m.dir);
        let src_dir = member_dir.join("src");
        fs::create_dir_all(&src_dir).expect("member src ディレクトリの作成に失敗した");

        let mut cargo_toml = format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n",
            m.package_name
        );
        if !m.path_deps.is_empty() {
            cargo_toml.push_str("\n[dependencies]\n");
            for dep_dir in m.path_deps {
                let dep = members
                    .iter()
                    .find(|candidate| &candidate.dir == dep_dir)
                    .unwrap_or_else(|| panic!("path_deps が未知の dir `{dep_dir}` を参照している"));
                cargo_toml.push_str(&format!(
                    "{} = {{ path = \"../{}\" }}\n",
                    dep.package_name, dep_dir
                ));
            }
        }
        fs::write(member_dir.join("Cargo.toml"), cargo_toml)
            .expect("member Cargo.toml の書き込みに失敗した");

        let file_name = if m.is_bin { "main.rs" } else { "lib.rs" };
        fs::write(src_dir.join(file_name), &m.source).expect("member ソースの書き込みに失敗した");
    }

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

    ScenarioProject(dest)
}

/// `fw impact` の JSON レポート中の文字列配列フィールド
/// `"<field>":["a","b"]` の要素をそのまま（クォート込みの生テキストとして）
/// 含むかを判定する軽量ヘルパ。配列全体をパースせず、期待する要素
/// （例: 新設ルート `/search`）が配列内に文字列として現れるかどうかの
/// 部分一致検証に使う（`affected_routes` / `affected_crates` の非空・
/// 特定要素含有の検証、設計文書 §4.3）。
pub fn json_array_contains_str(stdout: &str, field: &str, expected_element: &str) -> bool {
    let needle = format!("\"{field}\":[");
    let Some(start) = stdout.find(&needle) else {
        return false;
    };
    let rest = &stdout[start + needle.len()..];
    let Some(end) = rest.find(']') else {
        return false;
    };
    let array_body = &rest[..end];
    array_body.contains(&format!("\"{expected_element}\""))
}
