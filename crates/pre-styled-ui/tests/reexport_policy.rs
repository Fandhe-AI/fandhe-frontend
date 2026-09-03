//! headless 再エクスポートの**形式**（glob / 選択的 / shadowing）規約
//! （`crates/pre-styled-ui/src/lib.rs` 「headless 再エクスポートの形式規約
//! （イシュー #1062）」節）の機械検知テスト。
//!
//! 先例は `crates/core/tests/unsafe_boundary.rs`（ソース走査型の fail-closed
//! 契約テスト）と同型のアプローチを取る。本ファイルが検証するのは
//! 「再エクスポート**形式**が規約に沿っているか」のみであり、headless
//! 63 部品 / pre-styled 107 部品の対応関係（ラップ済み・pre-styled-only・
//! 未ラップ）には立ち入らない（そちらは `crates/docs-site/tests/` 側、
//! イシュー #1064 のスコープ）。
//!
//! **`REEXPORT-GLOB-REVIEWED:` マーカーはドキュメント規律のための装置であり、
//! セキュリティ制御ではない**（`.claude/rules/security.md` A05）。コメントは
//! 偽装可能であり、#157 で `// ESCAPE-REVIEWED:` コメント単体を根拠にできない
//! と確定した教訓と同型である。ただし本テストが対象とする glob 再エクス
//! ポートはエスケープ迂回面（`raw_html()` の使用可否）を一切左右しない
//! （出力される HTML 断片の生成経路そのものは変化しない到達性の話）ため、
//! `#[expect(clippy::disallowed_methods, reason = "...")]` のような属性強制
//! までは要求せず、コメント規律で比例的に扱う。検査 1（許可リストとの
//! 双方向一致）が実質的な fail-closed 境界を担保する。

use std::fs;
use std::path::{Path, PathBuf};

/// glob 再エクスポート（`pub use fandhe_frontend_headless_ui::<mod>::*;`）を
/// 規約 B の 4 条件（`lib.rs` 参照）に基づき維持すると判定済みのモジュール
/// 一覧（イシュー #1062 レビュー結果の正、
/// `docs/internal/pre-styled-ui-implementation-notes.md` §3c に来歴を記録）。
///
/// 追加・削除どちらの乖離も `glob_reexport_modules_match_reviewed_allowlist`
/// で fail-closed に検知する。
const GLOB_REEXPORT_MODULES: &[(&str, &str)] = &[
    (
        "action_bar",
        "stylesheet() のみ・variant 軸なし・属性セレクタのみ",
    ),
    (
        "popover",
        "stylesheet() のみ・#708 方針 3 で variant 軸非提供確定・属性セレクタのみ",
    ),
    (
        "hover_card",
        "stylesheet() のみ・variant 軸なし・属性セレクタのみ",
    ),
    (
        "tooltip",
        "stylesheet() のみ・#708 方針 3 で variant 軸非提供確定・属性セレクタのみ",
    ),
    (
        "toolbar",
        "stylesheet() のみ・variant 軸なし・属性セレクタのみ",
    ),
    (
        "scroll_area",
        "stylesheet() のみ・#825 で variant 軸非採用確定・属性セレクタのみ",
    ),
    (
        "toggle_tip",
        "stylesheet() のみ・variant 軸なし・属性セレクタのみ",
    ),
    (
        "menubar",
        "stylesheet() のみ・variant 軸なし・属性セレクタのみ",
    ),
    (
        "json_tree_view",
        "stylesheet() のみ・variant 軸なし・属性セレクタのみ",
    ),
    (
        "floating_panel",
        "stylesheet() のみ・variant 軸なし・属性セレクタのみ",
    ),
    (
        "timer",
        "stylesheet() のみ・variant 軸なし・属性セレクタのみ",
    ),
    (
        "navigation_menu",
        "stylesheet() のみ・variant 軸なし・属性セレクタのみ",
    ),
];

/// 規約 B-1 が許可する `pub` トップレベル項目名（glob モジュールが再定義して
/// よい styled 項目はこれのみ。それ以外が現れたら暗黙 shadowing/variant 型
/// 混入の兆候として FAIL する）。
const ALLOWED_TOP_LEVEL_PUB_ITEMS_IN_GLOB_MODULES: &[&str] = &["stylesheet", "css"];

/// workspace ルート（`crates/pre-styled-ui/` から 2 段上）の絶対パスを返す。
fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("crates/pre-styled-ui/ の 2 段上が workspace ルートであるはず")
        .to_path_buf()
}

fn pre_styled_ui_src_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// `src/*.rs` を走査し `(モジュール名, ファイル相対パス, 行番号, 行内容)` の
/// 一覧を返す。シンボリックリンクは辿らない（`unsafe_boundary.rs` と同型の
/// 安全側走査、`.claude/rules/security.md` A01）。
fn scan_rs_files() -> Vec<PathBuf> {
    let dir = pre_styled_ui_src_dir();
    let mut files = Vec::new();
    for entry in fs::read_dir(&dir).expect("crates/pre-styled-ui/src を読めること") {
        let entry = entry.expect("read_dir エントリを読めること");
        let file_type = entry.file_type().expect("file_type を取得できること");
        if !file_type.is_file() {
            // シンボリックリンク・ディレクトリはスキップし、実ファイルのみ対象とする。
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    files
}

/// 行がコメント（`//`/`///`/`//!` のいずれか）で始まるかを判定する
/// （トリム後の先頭一致、`unsafe_boundary.rs` と同型の行ベース判定）。
/// ブロックコメント（`/* */`）は本クレートで未使用であり本判定の対象外
/// （`lib.rs` の rustdoc は `//!` のみで構成されるため誤検知しない）。
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with("//")
}

/// `pub use fandhe_frontend_headless_ui::<mod>::*;` 形式の行から `<mod>` を
/// 抽出する（コメント行は除外済みの入力を前提とする）。
fn extract_glob_module(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let prefix = "pub use fandhe_frontend_headless_ui::";
    let suffix = "::*;";
    if let Some(rest) = trimmed.strip_prefix(prefix) {
        if let Some(module) = rest.strip_suffix(suffix) {
            // `state::*`/`data_attrs::*` のような glob は本規約の対象外
            // （規約 A の選択的個別再エクスポートと併用される内部モジュール
            // への glob であり、headless の当該部品モジュール自体への glob
            // ではない）。対象は単純な単一セグメントのモジュール名のみ。
            if !module.contains("::") {
                return Some(module.to_string());
            }
        }
    }
    None
}

/// 検査 1: glob 一覧の双方向一致（追加・削除の双方で FAIL）。
///
/// レビュー未了のまま glob を追加した場合（規約 B の 4 条件確認漏れ）と、
/// レビュー済みリストが実装から乖離した場合（リスト更新漏れ）の双方を
/// 検知する。
#[test]
fn glob_reexport_modules_match_reviewed_allowlist() {
    let mut found_modules: Vec<String> = Vec::new();
    for path in scan_rs_files() {
        let content = fs::read_to_string(&path).expect("*.rs を読めること");
        for line in content.lines() {
            if is_comment_line(line) {
                continue;
            }
            if let Some(module) = extract_glob_module(line) {
                found_modules.push(module);
            }
        }
    }
    found_modules.sort();
    found_modules.dedup();

    let mut reviewed: Vec<&str> = GLOB_REEXPORT_MODULES.iter().map(|(m, _)| *m).collect();
    reviewed.sort_unstable();

    let extra: Vec<&String> = found_modules
        .iter()
        .filter(|m| !reviewed.contains(&m.as_str()))
        .collect();
    let missing: Vec<&&str> = reviewed
        .iter()
        .filter(|m| !found_modules.iter().any(|f| f == *m))
        .collect();

    assert!(
        extra.is_empty() && missing.is_empty(),
        "glob 再エクスポートの一覧が GLOB_REEXPORT_MODULES（イシュー #1062 レビュー \
         済みリスト）と一致しません。\n\
         未レビューの追加（規約 B の 4 条件を満たすなら GLOB_REEXPORT_MODULES へ \
         追加し満たさないなら選択的 re-export へ切り替える。crates/pre-styled-ui/src/lib.rs \
         「headless 再エクスポートの形式規約」参照）: {extra:?}\n\
         リストに残存するが実装から消えた項目（GLOB_REEXPORT_MODULES から削除する \
         こと）: {missing:?}"
    );
}

/// 検査 2: マーカーコメントの存在（規約 B-4）。
///
/// glob 行の直前の連続コメント行（空行で打ち切り）に `REEXPORT-GLOB-REVIEWED:`
/// が含まれることを要求する。
#[test]
fn glob_reexport_lines_have_reviewed_marker_comment() {
    let mut missing_marker: Vec<String> = Vec::new();

    for path in scan_rs_files() {
        let content = fs::read_to_string(&path).expect("*.rs を読めること");
        let lines: Vec<&str> = content.lines().collect();
        for (idx, line) in lines.iter().enumerate() {
            if is_comment_line(line) {
                continue;
            }
            if extract_glob_module(line).is_none() {
                continue;
            }
            // 直前のコメントブロック（空行で打ち切り）を上へ辿って収集する。
            let mut has_marker = false;
            let mut cursor = idx;
            while cursor > 0 {
                let prev = lines[cursor - 1];
                if !is_comment_line(prev) {
                    break;
                }
                if prev.contains("REEXPORT-GLOB-REVIEWED:") {
                    has_marker = true;
                    break;
                }
                cursor -= 1;
            }
            if !has_marker {
                let rel = path
                    .strip_prefix(workspace_root())
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                missing_marker.push(format!("{rel}:{}", idx + 1));
            }
        }
    }

    assert!(
        missing_marker.is_empty(),
        "以下の glob 再エクスポート行の直前に `REEXPORT-GLOB-REVIEWED: <条件 1〜3 \
         を満たす理由>`（イシュー #1062 規約 B-4）コメントがありません: \
         {missing_marker:?}"
    );
}

/// 検査 3: glob モジュールの pub 項目許可リスト（規約 B-1 / 規約 C の近似検査）。
///
/// glob 再エクスポートを維持する 13 モジュールがトップレベルで定義する
/// `pub` 項目が `stylesheet`/`css` 以外に増えていないかを検知する。これは
/// 「glob と同名ローカル定義による暗黙 shadowing」と「variant 型の後付け
/// 追加（規約 B-2 逸脱）」の両方を捉える。
#[test]
fn glob_reexport_modules_define_only_allowed_top_level_pub_items() {
    let mut violations: Vec<String> = Vec::new();

    for (module, _) in GLOB_REEXPORT_MODULES {
        let path = pre_styled_ui_src_dir().join(format!("{module}.rs"));
        let content =
            fs::read_to_string(&path).unwrap_or_else(|_| panic!("{} を読めること", path.display()));
        for (idx, line) in content.lines().enumerate() {
            if is_comment_line(line) {
                continue;
            }
            let trimmed = line.trim_start();
            for kind in [
                "pub fn ",
                "pub struct ",
                "pub enum ",
                "pub trait ",
                "pub type ",
                "pub const ",
                "pub static ",
            ] {
                if let Some(rest) = trimmed.strip_prefix(kind) {
                    let name = rest
                        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                        .next()
                        .unwrap_or("");
                    if !ALLOWED_TOP_LEVEL_PUB_ITEMS_IN_GLOB_MODULES.contains(&name) {
                        violations.push(format!(
                            "{module}.rs:{} `{}` は glob モジュールの許可項目 \
                             （stylesheet/css）外です（規約 B-1 逸脱、選択的 \
                             re-export への切り替えを検討）",
                            idx + 1,
                            name
                        ));
                    }
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "glob 再エクスポートモジュールが stylesheet()/css() 以外の pub 項目を \
         定義しています（イシュー #1062 規約 B-1）:\n{violations:?}"
    );
}

/// 検査 4: 規約 C の対偶検査。
///
/// `pub fn root` を定義するモジュール（styled パーツ関数を再定義している
/// ＝規約 A 対象）が glob 行を同時に持たないことを要求する。glob と
/// `root` 再定義の併存は、規約 C が禁止する「暗黙 shadowing」の典型形
/// （glob 由来の `root` を同名ローカル定義が上書きし、読み手に差分が
/// 見えない状態）そのものであり、独立した検査として固定する。
#[test]
fn modules_defining_root_do_not_use_glob_reexport() {
    let glob_modules: Vec<&str> = GLOB_REEXPORT_MODULES.iter().map(|(m, _)| *m).collect();
    let mut violations: Vec<String> = Vec::new();

    for path in scan_rs_files() {
        let content = fs::read_to_string(&path).expect("*.rs を読めること");
        let module_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();

        let defines_root = content
            .lines()
            .any(|line| !is_comment_line(line) && line.trim_start().starts_with("pub fn root"));
        let has_glob = content
            .lines()
            .any(|line| !is_comment_line(line) && extract_glob_module(line).is_some());

        if defines_root && has_glob {
            violations.push(module_name.clone());
        }
        // glob_modules に登録されているモジュールが root を再定義していない
        // ことも同時に確認する（検査 3 とは独立に、規約 A/B の二律背反を
        // 直接固定する）。
        if glob_modules.contains(&module_name.as_str()) && defines_root {
            violations.push(format!("{module_name}（GLOB_REEXPORT_MODULES 登録済み）"));
        }
    }

    assert!(
        violations.is_empty(),
        "`pub fn root` を再定義しつつ glob 再エクスポートも行っているモジュールが \
         あります（イシュー #1062 規約 C 違反、選択的 re-export へ統一すること）: \
         {violations:?}"
    );
}
