//! `fw new`（TASK-13.4 相当、イシュー #350／複数テンプレート選択、
//! イシュー #378）が展開する標準プロジェクトテンプレート群のコンパイル時
//! 埋め込みマニフェスト。
//!
//! `fw` は単一実行ファイル配布（Docker 想定）を目標とするため、実行時に
//! `templates/<name>/` のファイルシステム配置へ依存させず、`include_str!`
//! でバイナリへ埋め込む。正本は引き続き `templates/<name>/`（`default` は
//! `xtask/tests/template_*.rs`、`app` は
//! `cli/tests/template_vendor_drift.rs` が正本として参照する）であり、
//! 本ファイルはその写しにすぎない。両者の乖離は `cli/tests/new_e2e.rs` の
//! ドリフト検知テストが `templates/<name>/` を再帰走査して機械的に検出する
//! （手動同期に頼らない。`.claude/rules/ci.md` の cargo-deny pin ドリフト検知と
//! 同じ運用方針）。
//!
//! `new.rs::run_new` から呼ばれ、[`Template::files`] の配列順（固定）で
//! 展開することが決定性（同一入力 → バイト単位で同一出力）を担保する。
//! `--template` 未指定時の既定は [`DEFAULT_TEMPLATE_NAME`]（`default`）で
//! あり、イシュー #378 以前の `fw new` 呼び出し（テンプレート選択なし）と
//! 完全後方互換（同一バイト出力）を保つ。

/// テンプレート 1 ファイル分のコンパイル時定数。
///
/// `rel_path` はテンプレートルート（`templates/<name>/`）からの相対パスで、
/// `new.rs` がターゲットディレクトリと結合してファイルを書き出す。
/// `executable` は git の実行ビット（mode 100755）をそのまま反映したもので、
/// Unix では `new.rs` がこれをもとに 0o755 を明示設定する。非 Unix
/// プラットフォームでは `new.rs::set_permissions` が no-op のため
/// `executable` は無視される（`docs/design/fw-new-design.md` 参照）。
pub(crate) struct TemplateFile {
    pub(crate) rel_path: &'static str,
    pub(crate) contents: &'static str,
    pub(crate) executable: bool,
}

/// 選択可能なテンプレート 1 件分の定義（イシュー #378 で `TemplateFile` の
/// 単一配列から一般化）。
///
/// `needle` はテンプレート内のパッケージ名プレースホルダー
/// （`Cargo.toml`/`Cargo.lock`/`structure.toml` に埋め込まれた仮パッケージ名）
/// で、`new.rs::expand_template` がプロジェクト名へ置換する際の対象文字列。
/// `substituted_files` はその置換を適用する `rel_path` の allowlist
/// （`new.rs::replace_exact` の fail-closed 出現回数検証と組み合わせて使う）。
pub(crate) struct Template {
    /// `--template <name>` で指定する識別子（allowlist 照合に使う固定文字列）。
    pub(crate) name: &'static str,
    /// このテンプレートが展開する全ファイル（配列順で決定的に展開）。
    pub(crate) files: &'static [TemplateFile],
    /// パッケージ名置換の対象文字列（テンプレートごとに異なる仮パッケージ名）。
    pub(crate) needle: &'static str,
    /// `needle` の置換を適用するファイルの `rel_path` allowlist。
    pub(crate) substituted_files: &'static [&'static str],
}

/// `--template` 省略時の既定テンプレート名。イシュー #378 以前の `fw new`
/// （テンプレート選択オプションなし）との後方互換性を保つため `default` の
/// まま固定する。
pub(crate) const DEFAULT_TEMPLATE_NAME: &str = "default";

/// `templates/default/` の全ファイル（13 件）を git の相対パス順・実行ビット
/// どおりに埋め込んだ固定配列。
///
/// 展開順はこの配列順であり、`fw new` の出力 JSON の `files` 一覧も同じ順序で
/// 並べる契約とする（`new.rs::run_new` 参照）。`structure.toml`（イシュー #351
/// で追加、`fw gate` が唯一の情報源として読む）は生成直後の `fw gate` PASS の
/// 前提条件であり、`Cargo.toml` / `Cargo.lock` と同様プロジェクト名への置換
/// 対象。
const DEFAULT_TEMPLATE_FILES: &[TemplateFile] = &[
    TemplateFile {
        rel_path: ".github/workflows/deny.yml",
        contents: include_str!("../../templates/default/.github/workflows/deny.yml"),
        executable: false,
    },
    TemplateFile {
        rel_path: ".github/workflows/npm-asset-gate.yml",
        contents: include_str!("../../templates/default/.github/workflows/npm-asset-gate.yml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "Cargo.lock",
        contents: include_str!("../../templates/default/Cargo.lock"),
        executable: false,
    },
    TemplateFile {
        rel_path: "Cargo.toml",
        contents: include_str!("../../templates/default/Cargo.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "clippy.toml",
        contents: include_str!("../../templates/default/clippy.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "deny.toml",
        contents: include_str!("../../templates/default/deny.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "src/main.rs",
        contents: include_str!("../../templates/default/src/main.rs"),
        executable: false,
    },
    TemplateFile {
        rel_path: "structure.toml",
        contents: include_str!("../../templates/default/structure.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "tests/negative_type_error.rs",
        contents: include_str!("../../templates/default/tests/negative_type_error.rs"),
        executable: false,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/allowlist.toml",
        contents: include_str!("../../templates/default/tools/npm-asset-build/allowlist.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/apply_exempt.py",
        contents: include_str!("../../templates/default/tools/npm-asset-build/apply_exempt.py"),
        executable: true,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/check_static_only.py",
        contents: include_str!(
            "../../templates/default/tools/npm-asset-build/check_static_only.py"
        ),
        executable: true,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/install.sh",
        contents: include_str!("../../templates/default/tools/npm-asset-build/install.sh"),
        executable: true,
    },
];

/// `templates/app/` の全ファイル（22 件）を埋め込んだ固定配列（イシュー #378）。
///
/// rws-core / rws-app（vendor 同梱、`vendor/rws-core` / `vendor/rws-app`）に
/// 依存する拡張テンプレート。`Loader` trait 実装・束縛点 API
/// （`bind_text`/`keyed_list`）・`rws_core::render` を使う実体サンプルを含む
/// （`templates/app/src/main.rs`）。`.github/workflows/*`・`clippy.toml`・
/// `deny.toml`・`tools/npm-asset-build/*` は `templates/default/` と共有
/// ファイル同一性を保つ（`cli/tests/template_vendor_drift.rs` が検証）。
const APP_TEMPLATE_FILES: &[TemplateFile] = &[
    TemplateFile {
        rel_path: ".github/workflows/deny.yml",
        contents: include_str!("../../templates/app/.github/workflows/deny.yml"),
        executable: false,
    },
    TemplateFile {
        rel_path: ".github/workflows/npm-asset-gate.yml",
        contents: include_str!("../../templates/app/.github/workflows/npm-asset-gate.yml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "Cargo.lock",
        contents: include_str!("../../templates/app/Cargo.lock"),
        executable: false,
    },
    TemplateFile {
        rel_path: "Cargo.toml",
        contents: include_str!("../../templates/app/Cargo.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "clippy.toml",
        contents: include_str!("../../templates/app/clippy.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "deny.toml",
        contents: include_str!("../../templates/app/deny.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "src/main.rs",
        contents: include_str!("../../templates/app/src/main.rs"),
        executable: false,
    },
    TemplateFile {
        rel_path: "static/embed.html",
        contents: include_str!("../../templates/app/static/embed.html"),
        executable: false,
    },
    TemplateFile {
        rel_path: "structure.toml",
        contents: include_str!("../../templates/app/structure.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "tests/escape_regression.rs",
        contents: include_str!("../../templates/app/tests/escape_regression.rs"),
        executable: false,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/allowlist.toml",
        contents: include_str!("../../templates/app/tools/npm-asset-build/allowlist.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/apply_exempt.py",
        contents: include_str!("../../templates/app/tools/npm-asset-build/apply_exempt.py"),
        executable: true,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/check_static_only.py",
        contents: include_str!("../../templates/app/tools/npm-asset-build/check_static_only.py"),
        executable: true,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/install.sh",
        contents: include_str!("../../templates/app/tools/npm-asset-build/install.sh"),
        executable: true,
    },
    TemplateFile {
        rel_path: "vendor/rws-app/Cargo.toml",
        contents: include_str!("../../templates/app/vendor/rws-app/Cargo.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "vendor/rws-app/src/lib.rs",
        contents: include_str!("../../templates/app/vendor/rws-app/src/lib.rs"),
        executable: false,
    },
    TemplateFile {
        rel_path: "vendor/rws-core/Cargo.toml",
        contents: include_str!("../../templates/app/vendor/rws-core/Cargo.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "vendor/rws-core/src/bind.rs",
        contents: include_str!("../../templates/app/vendor/rws-core/src/bind.rs"),
        executable: false,
    },
    TemplateFile {
        rel_path: "vendor/rws-core/src/escape.rs",
        contents: include_str!("../../templates/app/vendor/rws-core/src/escape.rs"),
        executable: false,
    },
    TemplateFile {
        rel_path: "vendor/rws-core/src/keyed.rs",
        contents: include_str!("../../templates/app/vendor/rws-core/src/keyed.rs"),
        executable: false,
    },
    TemplateFile {
        rel_path: "vendor/rws-core/src/lib.rs",
        contents: include_str!("../../templates/app/vendor/rws-core/src/lib.rs"),
        executable: false,
    },
    TemplateFile {
        rel_path: "vendor/rws-core/src/tags.rs",
        contents: include_str!("../../templates/app/vendor/rws-core/src/tags.rs"),
        executable: false,
    },
    TemplateFile {
        rel_path: "vendor/rws-core/src/url.rs",
        contents: include_str!("../../templates/app/vendor/rws-core/src/url.rs"),
        executable: false,
    },
];

/// `--template` の allowlist（イシュー #378）。
///
/// テンプレート名はここに列挙したコンパイル時定数との完全一致照合のみで
/// 解決し、ユーザー入力から動的にパス・`include_str!` 対象を組み立てない
/// （`security.md` A01/A03、`new.rs::run_new` のテンプレート解決を参照）。
/// 配列順は `fw new --template <unknown>` のエラーメッセージが提示する
/// 利用可能テンプレート一覧の表示順（固定）にもなる。
pub(crate) const TEMPLATES: &[Template] = &[
    Template {
        name: "default",
        files: DEFAULT_TEMPLATE_FILES,
        needle: "rws-template-default",
        substituted_files: &["Cargo.toml", "Cargo.lock", "structure.toml"],
    },
    Template {
        name: "app",
        files: APP_TEMPLATE_FILES,
        needle: "rws-template-app",
        substituted_files: &["Cargo.toml", "Cargo.lock", "structure.toml"],
    },
];

/// `name` に一致する [`Template`] を [`TEMPLATES`] から検索する。
///
/// 未知の名前は `None`（`new.rs::run_new` が使用法エラー・終了コード 2 へ
/// 変換する）。
pub(crate) fn find_template(name: &str) -> Option<&'static Template> {
    TEMPLATES.iter().find(|t| t.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_template_is_registered_and_matches_constant_name() {
        let t = find_template(DEFAULT_TEMPLATE_NAME).expect("default template must be registered");
        assert_eq!(t.name, "default");
        assert!(!t.files.is_empty());
    }

    #[test]
    fn app_template_is_registered() {
        let t = find_template("app").expect("app template must be registered");
        assert_eq!(t.name, "app");
        assert!(!t.files.is_empty());
    }

    #[test]
    fn unknown_template_name_resolves_to_none() {
        assert!(find_template("nonexistent").is_none());
    }

    /// 各テンプレートの実行可能ファイル集合（`executable: true` の
    /// `rel_path` 集合）が期待固定リストと一致することを確認する
    /// （テンプレート追加・改変時の意図しない実行ビット変化を検出する
    /// メタデータドリフト検知。プラットフォーム非依存: `set_permissions`
    /// 自体は Unix でのみ効果を持つが、本テストはメタデータの記述内容の
    /// みを検証するためどの OS でも実行できる）。
    #[test]
    fn executable_file_sets_match_expected_fixed_lists() {
        let expected_default: &[&str] = &[
            "tools/npm-asset-build/apply_exempt.py",
            "tools/npm-asset-build/check_static_only.py",
            "tools/npm-asset-build/install.sh",
        ];
        let expected_app: &[&str] = expected_default;

        for (name, expected) in [("default", expected_default), ("app", expected_app)] {
            let t = find_template(name).unwrap_or_else(|| panic!("template `{name}` must exist"));
            let mut actual: Vec<&str> = t
                .files
                .iter()
                .filter(|f| f.executable)
                .map(|f| f.rel_path)
                .collect();
            actual.sort_unstable();
            let mut expected_sorted = expected.to_vec();
            expected_sorted.sort_unstable();
            assert_eq!(
                actual, expected_sorted,
                "template `{name}` の実行可能ファイル集合が期待値と一致しない"
            );
        }
    }
}
