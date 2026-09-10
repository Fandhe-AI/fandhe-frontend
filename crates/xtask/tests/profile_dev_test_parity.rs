//! ルート `Cargo.toml` の `[profile.*]` package 上書きが「dev のみに置く」
//! 契約（イシュー #2308）を維持していることを fail-closed に固定する回帰テスト。
//!
//! 背景（イシュー #2307 の分析結果）: `wasm-pack test` は 1 回の呼び出しで
//! `cargo build --tests`（dev プロファイル）→ `cargo test`（test プロファイル）
//! の 2 段で cargo を起動する。dev/test の package 上書き対象クレート集合が
//! 一致しないと、依存先の cdylib+rlib クレート（wasm-client / wasm-full 等、
//! メタデータハッシュなしの固定名生成物）が dev/test で別ユニットへ解決され、
//! 互いを上書きし合って `wasm-pack test` の呼び出しごとに毎回二重コンパイルが
//! 発生する（`docs/ci/browser-test-duration-regression-analysis.md` 参照）。
//!
//! 本テストは 2 つの契約を検証する:
//! 1. **禁止契約**: ルート `Cargo.toml` に `[profile.test.*]`（`[profile.test]`
//!    本体・`[profile.test.package.*]`・`[profile.test.build-override]` 等の
//!    あらゆる表記揺れ）を一切置かない。ドット付きキーによる迂回（裸の
//!    `[profile]` ヘッダ配下の `test.package.x.opt-level = 1`、トップレベルの
//!    `profile.test.package.x.opt-level = 1`）も同様に禁止する。
//! 2. **維持契約**: docs-site とその描画チェーン 7 クレートへの
//!    `[profile.dev.package.<crate>] opt-level = 1` 上書きが存在すること
//!    （イシュー #2299 の `test-docs-site` ジョブ短縮効果が黙って失われて
//!    いないことの機械的裏付け）。
//!
//! xtask は外部依存ゼロ方針（REQ-3、`crates/xtask/Cargo.toml` 参照）のため
//! TOML パーサは導入せず、`wasm_bindgen_version_sync.rs` と同じく行単位の
//! 手書き解析で判定する。判定は「許容形の列挙」ではなく「禁止形への
//! 反転判定」とする（`crates/xtask/tests/workflow_runner_policy.rs` と同じ
//! 思想）: 認識できない未知の表記は禁止契約側では fail-closed に違反へ倒す。

use std::path::PathBuf;

/// 短縮効果を維持すべき 7 クレート（イシュー #2299 が opt-level 1 化した
/// docs-site とその描画チェーン）。
const DEV_OVERRIDE_CRATES: &[&str] = &[
    "fandhe-frontend-docs-site",
    "fandhe-frontend-core",
    "fandhe-frontend-interactive",
    "fandhe-frontend-app",
    "fandhe-frontend-server",
    "fandhe-frontend-headless-ui",
    "fandhe-frontend-pre-styled-ui",
];

/// workspace ルート（`xtask/` の親ディレクトリ）の絶対パスを返す。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/xtask/ から 2 段上でワークスペースルートに到達する（イシュー #436）")
        .to_path_buf()
}

fn read_workspace_file(relative_path: &str) -> String {
    let path = workspace_root().join(relative_path);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("{} の読み込みに失敗した: {}", relative_path, path.display()))
}

/// `#` 以降をコメントとして除去した行を返す（TOML の文字列リテラル内 `#` は
/// 本リポジトリの `Cargo.toml` では出現しないため、単純な `#` 分割で足りる）。
fn strip_comment(line: &str) -> &str {
    match line.find('#') {
        Some(idx) => &line[..idx],
        None => line,
    }
}

/// テーブルヘッダ行（`[...]`）のドット区切りセグメント列を正規化して返す。
/// クォート・空白を除去するため `[ profile . test . package . "x" ]` の
/// ような表記揺れも `["profile", "test", "package", "x"]` へ正規化される。
/// ヘッダ行でなければ `None`。
fn normalized_header_segments(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if !(trimmed.starts_with('[') && trimmed.ends_with(']')) {
        return None;
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    // 配列テーブル `[[x]]` は本リポジトリの profile 宣言に存在しないため
    // 対象外（内側の `[`/`]` が残ると空文字列セグメントになり、後段の
    // 先頭セグメント比較が `profile` と一致せず素通りするだけで安全側）。
    let segments: Vec<String> = inner
        .split('.')
        .map(|seg| seg.trim().trim_matches('"').trim_matches('\'').to_string())
        .collect();
    Some(segments)
}

/// ルート `Cargo.toml` の内容に `[profile.test.*]` 系の禁止表記、または
/// ドット付きキーによる同等の迂回が含まれていないかを判定する。
/// 違反があれば理由文字列を返し、なければ `None`。
///
/// 反転判定（許容形を列挙せず禁止形を検知する）で書く: 未知の表記は
/// 「禁止に該当しない」側へ倒さず、`profile.test` に触れる行はすべて
/// 違反として扱う。これにより裸の `[profile]` + `test.package.x = ...` の
/// ようなドット付きキー迂回も取りこぼさない。
fn find_test_profile_violation(contents: &str) -> Option<String> {
    for (line_no, raw_line) in contents.lines().enumerate() {
        let line = strip_comment(raw_line);
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // テーブルヘッダ行: 先頭 2 セグメントが profile/test なら違反
        // （`[profile.testfoo]` のような別名は誤検知しない: 完全一致のみ）。
        if let Some(segments) = normalized_header_segments(trimmed) {
            if segments.first().map(String::as_str) == Some("profile")
                && segments.get(1).map(String::as_str) == Some("test")
            {
                return Some(format!(
                    "{}行目: 禁止された [profile.test.*] ヘッダを検出した: {}",
                    line_no + 1,
                    raw_line.trim()
                ));
            }
            // 裸の `[profile]` ヘッダ配下でドット付きキー `test.package.x = ...`
            // を書く迂回を塞ぐため、裸の `[profile]` ヘッダ自体も禁止する
            // （本リポジトリはこの形を使わないため fail-closed のコストはゼロ）。
            if segments.len() == 1 && segments[0] == "profile" {
                return Some(format!(
                    "{}行目: 裸の [profile] ヘッダは禁止（配下でのドット付きキー test.* 迂回を防ぐため）: {}",
                    line_no + 1,
                    raw_line.trim()
                ));
            }
            continue;
        }

        // 非ヘッダ行（key = value 形式）: トップレベルのドット付きキー
        // `profile.test.package.x.opt-level = 1`、およびインラインテーブル
        // `profile = { test = { package = { x = { opt-level = 1 } } } }`
        // による迂回を塞ぐ。`profile` トークンの直後が識別子の続き（英数字・
        // `_`・`-`）でなければ迂回とみなす。`strip_prefix` の残余が空文字列
        // （行全体が `profile` のみ）・`.` 始まり（ドット付きキー）・`=` 始まり
        // （インラインテーブル代入）・空白始まり（`profile =` 等）のいずれかを
        // 違反として検知する（`profiled_x = 1` のような無関係な識別子は
        // 残余が英数字で始まるため誤検知しない）。
        if let Some(rest) = trimmed.strip_prefix("profile") {
            let is_boundary = rest.is_empty()
                || rest.starts_with('.')
                || rest.starts_with('=')
                || rest.starts_with(char::is_whitespace);
            if is_boundary {
                return Some(format!(
                    "{}行目: profile をトップレベルキー/インラインテーブルとして使う記述は禁止（[profile.test.*] 迂回防止）: {}",
                    line_no + 1,
                    raw_line.trim()
                ));
            }
        }
    }
    None
}

/// `[profile.dev.package.<crate>]` ヘッダ配下（次のヘッダ行まで）に
/// `opt-level = 1` が存在するかを判定する。
fn has_dev_opt_level_one(contents: &str, crate_name: &str) -> bool {
    let target_header = format!("profile.dev.package.{crate_name}");
    let lines: Vec<&str> = contents.lines().collect();
    let mut in_target_section = false;
    for raw_line in &lines {
        let line = strip_comment(raw_line);
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(segments) = normalized_header_segments(trimmed) {
            let joined = segments.join(".");
            in_target_section = joined == target_header;
            continue;
        }
        if in_target_section {
            let normalized: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
            if normalized == "opt-level=1" {
                return true;
            }
        }
    }
    false
}

#[test]
fn root_cargo_toml_has_no_test_profile_section() {
    let contents = read_workspace_file("Cargo.toml");
    assert!(
        find_test_profile_violation(&contents).is_none(),
        "{}",
        find_test_profile_violation(&contents).unwrap_or_default()
    );
}

#[test]
fn root_cargo_toml_keeps_dev_opt_level_overrides_for_docs_site_chain() {
    let contents = read_workspace_file("Cargo.toml");
    for crate_name in DEV_OVERRIDE_CRATES {
        assert!(
            has_dev_opt_level_one(&contents, crate_name),
            "[profile.dev.package.{crate_name}] opt-level = 1 が見つからない \
             （イシュー #2299 の test-docs-site 短縮効果が失われている可能性）"
        );
    }
}

#[test]
fn find_test_profile_violation_detects_known_forms() {
    assert!(find_test_profile_violation("[profile.test]\nopt-level = 1\n").is_some());
    assert!(find_test_profile_violation(
        "[profile.test.package.fandhe-frontend-core]\nopt-level = 1\n"
    )
    .is_some());
    assert!(find_test_profile_violation(
        "[profile.test.package.\"fandhe-frontend-core\"]\nopt-level = 1\n"
    )
    .is_some());
    assert!(find_test_profile_violation(
        "[ profile . test . package . fandhe-frontend-core ]\nopt-level = 1\n"
    )
    .is_some());
    assert!(find_test_profile_violation("[profile.\"test\".package.x]\nopt-level = 1\n").is_some());
    assert!(
        find_test_profile_violation("[profile.test.build-override]\nopt-level = 1\n").is_some()
    );
    assert!(find_test_profile_violation("[profile]\ntest.package.x.opt-level = 1\n").is_some());
    assert!(find_test_profile_violation("profile.test.package.x.opt-level = 1\n").is_some());
    // インラインテーブルによる迂回（ヘッダ行を経由せず `profile = { ... }`
    // で丸ごと代入する形）。
    assert!(find_test_profile_violation(
        "profile = { test = { package = { x = { opt-level = 1 } } } }\n"
    )
    .is_some());
    assert!(find_test_profile_violation("profile={test={package={x={opt-level=1}}}}\n").is_some());
}

#[test]
fn find_test_profile_violation_does_not_false_positive() {
    assert!(find_test_profile_violation("[profile.testfoo]\nopt-level = 1\n").is_none());
    assert!(find_test_profile_violation("# [profile.test.package.x]\n# opt-level = 1\n").is_none());
    assert!(find_test_profile_violation(
        "[profile.dev.package.fandhe-frontend-core]\nopt-level = 1\n"
    )
    .is_none());
    assert!(find_test_profile_violation(
        "[profile.release.package.fandhe-frontend-core]\ncodegen-units = 1\n"
    )
    .is_none());
    // `profile` を前方一致で含むだけの無関係な識別子は誤検知しない
    // （`profile` トークン直後が識別子の続きである限り境界とみなさない）。
    assert!(find_test_profile_violation("profiled_x = 1\n").is_none());
    assert!(find_test_profile_violation("profile_name = \"release\"\n").is_none());
}

#[test]
fn has_dev_opt_level_one_detects_missing_or_wrong_value() {
    assert!(!has_dev_opt_level_one(
        "[profile.dev.package.fandhe-frontend-core]\nopt-level = 0\n",
        "fandhe-frontend-core"
    ));
    assert!(!has_dev_opt_level_one(
        "[profile.release.package.fandhe-frontend-core]\nopt-level = 1\n",
        "fandhe-frontend-core"
    ));
    assert!(has_dev_opt_level_one(
        "[profile.dev.package.\"fandhe-frontend-core\"]\nopt-level = 1\n",
        "fandhe-frontend-core"
    ));
}
