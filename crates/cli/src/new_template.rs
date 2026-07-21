//! `fw new`（TASK-13.4 相当、イシュー #350／複数テンプレート選択、
//! イシュー #378）が展開する標準プロジェクトテンプレート群のコンパイル時
//! 埋め込みマニフェスト。
//!
//! `fw` は単一実行ファイル配布（Docker 想定）を目標とするため、実行時に
//! `templates/<name>/` のファイルシステム配置へ依存させず、`include_str!`
//! でバイナリへ埋め込む。正本は引き続きリポジトリルート `templates/<name>/`
//! であり、ルート `templates/<name>/` と `crates/cli/templates/<name>/`
//! 同梱コピーの乖離検知は `cli/tests/template_publish_copy_drift.rs` が担う
//! （`app` テンプレート固有のバージョン依存整合性・共有ファイル同一性は
//! `cli/tests/template_vendor_drift.rs`、`default` は `xtask/tests/template_*.rs`
//! が別途検証する）。
//!
//! `include_str!` はクレートディレクトリ外を参照できない（`cargo package` の
//! tarball 検証で拒否される）ため、本クレートは `crates/cli/templates/` に
//! 正本のバイト単位同梱コピーを置き、そちらを参照する
//! （`templates/default/tools/npm-asset-build/` が `tools/npm-asset-build/` の
//! 同梱コピーであるのと同じ「正本 + 同梱コピー + ドリフト検知テスト」運用、
//! イシュー #316）。ルート `templates/<name>/` と `crates/cli/templates/<name>/`
//! の乖離は `cli/tests/template_publish_copy_drift.rs` が両ディレクトリを
//! 再帰走査してバイト単位比較し機械的に検出する（手動同期に頼らない。
//! `.claude/rules/ci.md` の cargo-deny pin ドリフト検知と同じ運用方針）。
//!
//! `crates/cli/templates/` 配下の `Cargo.toml` は `Cargo.toml.embed` に
//! リネームしてコピーする。`cargo package` はサブディレクトリに
//! `Cargo.toml` が存在すると、それをネストした別パッケージとみなし
//! `include`/`exclude` の指定に関わらずファイル列挙から機械的に除外する
//! ため（cargo の既知の仕様）、元ファイル名のままでは同梱できない。内容は
//! 正本 `templates/<name>/(.../)?Cargo.toml` とバイト単位で同一であり、
//! `template_publish_copy_drift.rs` が拡張子差異を許容した上で内容一致を
//! 検証する。`fw new` が生成するプロジェクトへの書き出しファイル名
//! （`rel_path`）は `Cargo.toml` のまま変わらない（[`TemplateFile::rel_path`]
//! 参照）。
//!
//! `new.rs::run_new` から呼ばれ、[`Template::files`] の配列順（固定）で
//! 展開することが決定性（同一入力 → バイト単位で同一出力）を担保する。
//! `--template` 未指定時の既定は [`DEFAULT_TEMPLATE_NAME`]（`default`）で
//! あり、イシュー #378 以前の `fw new` 呼び出し（テンプレート選択なし）と
//! 完全後方互換（同一バイト出力）を保つ。
//!
//! ## `fw new --example`（イシュー #500）
//!
//! 上記の `TEMPLATES`（`fw new --template <name>`、雛形の生成でパッケージ名を
//! プロジェクト名へ置換する）とは別に、[`EXAMPLES`]（`fw new --example <name>`）
//! を提供する。`examples/<name>/`（イシュー #499 で確立した「crates.io 公開済み
//! クレートを実際に使う正本サンプル」規約）を `templates/` と同じ「正本 +
//! `include_str!` 同梱コピー + ドリフト検知テスト」運用で取得できるようにする
//! 機構であり、`crates/cli/embedded-examples/` に正本のバイト単位同梱コピーを
//! 置く（`crates/cli/embedded-examples/README.md` 参照）。乖離検知は
//! `cli/tests/example_publish_copy_drift.rs` が担う。
//!
//! `Template` / `TemplateFile` struct を完全再利用するが、`EXAMPLES` の各
//! `Template::substituted_files` は空配列に固定する（パッケージ名置換を行わない）。
//! `examples/ssr-routing/tests/routing.rs` が
//! `env!("CARGO_BIN_EXE_fandhe-frontend-example-ssr-routing")` でバイナリ名を
//! 直接参照するため、`Cargo.toml` のパッケージ名だけを置換すると
//! `CARGO_BIN_EXE_*` が未定義になり生成直後の `cargo test` がコンパイル不能に
//! なる。examples は「雛形の生成」ではなく「正本サンプルの取得」であり、生成物が
//! 正本 `examples/<name>/` と全ファイルバイト一致になることが決定性・ドリフト
//! 検知の観点で最も強い保証になる（`EMBED_TEMPLATE_FILES` と同じ整理）。

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

/// `templates/default/` の全ファイル（14 件）を git の相対パス順・実行ビット
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
        contents: include_str!("../templates/default/.github/workflows/deny.yml"),
        executable: false,
    },
    TemplateFile {
        rel_path: ".github/workflows/npm-asset-gate.yml",
        contents: include_str!("../templates/default/.github/workflows/npm-asset-gate.yml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "Cargo.lock",
        contents: include_str!("../templates/default/Cargo.lock"),
        executable: false,
    },
    TemplateFile {
        rel_path: "Cargo.toml",
        contents: include_str!("../templates/default/Cargo.toml.embed"),
        executable: false,
    },
    TemplateFile {
        rel_path: "README.md",
        contents: include_str!("../templates/default/README.md"),
        executable: false,
    },
    TemplateFile {
        rel_path: "clippy.toml",
        contents: include_str!("../templates/default/clippy.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "deny.toml",
        contents: include_str!("../templates/default/deny.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "src/main.rs",
        contents: include_str!("../templates/default/src/main.rs"),
        executable: false,
    },
    TemplateFile {
        rel_path: "structure.toml",
        contents: include_str!("../templates/default/structure.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "tests/negative_type_error.rs",
        contents: include_str!("../templates/default/tests/negative_type_error.rs"),
        executable: false,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/allowlist.toml",
        contents: include_str!("../templates/default/tools/npm-asset-build/allowlist.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/apply_exempt.py",
        contents: include_str!("../templates/default/tools/npm-asset-build/apply_exempt.py"),
        executable: true,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/check_static_only.py",
        contents: include_str!("../templates/default/tools/npm-asset-build/check_static_only.py"),
        executable: true,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/install.sh",
        contents: include_str!("../templates/default/tools/npm-asset-build/install.sh"),
        executable: true,
    },
];

/// `templates/app/` の全ファイル（18 件）を埋め込んだ固定配列（イシュー #378、
/// イシュー #411 で wasm ビルド込み CSR 完全実体を追加、イシュー #412 で
/// vendor 同梱から crates.io バージョン依存へ切替）。
///
/// fandhe-frontend-core / fandhe-frontend-app（crates.io バージョン依存）に
/// 依存する拡張テンプレート。`Loader` trait 実装・束縛点 API
/// （`bind_text`/`keyed_list`）・`fandhe_frontend_core::render` を使う実体サンプルを含む
/// （`templates/app/src/main.rs`）。`.github/workflows/*`・`clippy.toml`・
/// `deny.toml`・`tools/npm-asset-build/*` は `templates/default/` と共有
/// ファイル同一性を保つ（`cli/tests/template_vendor_drift.rs` が検証）。
///
/// イシュー #411: fandhe-frontend-interactive / fandhe-frontend-wasm-client
/// （crates.io バージョン依存、正本は `interactive/` / `wasm-client/`）に
/// 依存する独立ワークスペース `wasm/`（glue クレート `app-csr-wasm`）・
/// `tools/wasm/build.sh`（wasm ビルド手順）を追加し、生成プロジェクトが
/// 自力で CSR（`mount_csr`/`hydrate`）の wasm 成果物をビルドできる完全実体を
/// 同梱する（root（`fandhe-frontend-template-app`）の依存グラフ・`fw gate`
/// 対象は不変、実装計画 §2.2 参照）。イシュー #412（vendor 同梱 → バージョン
/// 依存への切替）により、root・wasm/ いずれも自前のクレートソースを持たなく
/// なった（`crates.io` から取得）。
const APP_TEMPLATE_FILES: &[TemplateFile] = &[
    TemplateFile {
        rel_path: ".github/workflows/deny.yml",
        contents: include_str!("../templates/app/.github/workflows/deny.yml"),
        executable: false,
    },
    TemplateFile {
        rel_path: ".github/workflows/npm-asset-gate.yml",
        contents: include_str!("../templates/app/.github/workflows/npm-asset-gate.yml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "Cargo.lock",
        contents: include_str!("../templates/app/Cargo.lock"),
        executable: false,
    },
    TemplateFile {
        rel_path: "Cargo.toml",
        contents: include_str!("../templates/app/Cargo.toml.embed"),
        executable: false,
    },
    TemplateFile {
        rel_path: "clippy.toml",
        contents: include_str!("../templates/app/clippy.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "deny.toml",
        contents: include_str!("../templates/app/deny.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "src/main.rs",
        contents: include_str!("../templates/app/src/main.rs"),
        executable: false,
    },
    TemplateFile {
        rel_path: "static/embed.html",
        contents: include_str!("../templates/app/static/embed.html"),
        executable: false,
    },
    TemplateFile {
        rel_path: "structure.toml",
        contents: include_str!("../templates/app/structure.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "tests/escape_regression.rs",
        contents: include_str!("../templates/app/tests/escape_regression.rs"),
        executable: false,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/allowlist.toml",
        contents: include_str!("../templates/app/tools/npm-asset-build/allowlist.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/apply_exempt.py",
        contents: include_str!("../templates/app/tools/npm-asset-build/apply_exempt.py"),
        executable: true,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/check_static_only.py",
        contents: include_str!("../templates/app/tools/npm-asset-build/check_static_only.py"),
        executable: true,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/install.sh",
        contents: include_str!("../templates/app/tools/npm-asset-build/install.sh"),
        executable: true,
    },
    // イシュー #411: CSR wasm ビルド込みの完全実体。fandhe-frontend-interactive /
    // fandhe-frontend-wasm-client（crates.io バージョン依存、イシュー #412 で
    // vendor 同梱から切替）をビルドする独立ワークスペース wasm/・手順スクリプト
    // tools/wasm/build.sh を追加する（実装計画 §2.2・§2.3）。
    TemplateFile {
        rel_path: "wasm/Cargo.lock",
        contents: include_str!("../templates/app/wasm/Cargo.lock"),
        executable: false,
    },
    TemplateFile {
        rel_path: "wasm/Cargo.toml",
        contents: include_str!("../templates/app/wasm/Cargo.toml.embed"),
        executable: false,
    },
    TemplateFile {
        rel_path: "wasm/src/lib.rs",
        contents: include_str!("../templates/app/wasm/src/lib.rs"),
        executable: false,
    },
    TemplateFile {
        rel_path: "tools/wasm/build.sh",
        contents: include_str!("../templates/app/tools/wasm/build.sh"),
        executable: true,
    },
];

/// `templates/embed/` の全ファイル（2 件）を埋め込んだ固定配列
/// （イシュー #410）。
///
/// cargo パッケージを持たない「静的単一ファイルの部分埋め込み構成」
/// （REQ-7）。`embed.html` は TASK-7.1a（#52）の正本をバイト無変更で流用し
/// （`xtask/tests/template_embed_html.rs`・`cli/tests/template_vendor_drift.rs`
/// が参照する正本と同一であることが前提のため、本テンプレート追加時に
/// 一切変更しない）、`structure.toml` は `fw gate` が唯一の情報源として読む
/// 静的専用（asset-only）マニフェスト（`cli/src/gate.rs::is_asset_only_project`
/// が明示宣言として認識、`templates/embed/structure.toml` 冒頭コメント参照）。
const EMBED_TEMPLATE_FILES: &[TemplateFile] = &[
    TemplateFile {
        rel_path: "embed.html",
        contents: include_str!("../templates/embed/embed.html"),
        executable: false,
    },
    TemplateFile {
        rel_path: "structure.toml",
        contents: include_str!("../templates/embed/structure.toml"),
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
        needle: "fandhe-frontend-template-default",
        substituted_files: &["Cargo.toml", "Cargo.lock", "structure.toml"],
    },
    Template {
        name: "app",
        files: APP_TEMPLATE_FILES,
        needle: "fandhe-frontend-template-app",
        substituted_files: &["Cargo.toml", "Cargo.lock", "structure.toml"],
    },
    Template {
        name: "embed",
        files: EMBED_TEMPLATE_FILES,
        // cargo パッケージが存在しないテンプレートのため置換対象の仮パッケージ名
        // 自体を持たない。この `needle` はどのファイルにも出現しないダミー
        // 文字列であり（`substituted_files` が空のため置換ループは素通りする、
        // `new.rs::expand_template` 参照）、生成物はテンプレートと全ファイル
        // バイト一致になる（`cli/tests/new_e2e.rs` が固定）。
        needle: "fandhe-frontend-template-embed",
        substituted_files: &[],
    },
];

/// `name` に一致する [`Template`] を [`TEMPLATES`] から検索する。
///
/// 未知の名前は `None`（`new.rs::run_new` が使用法エラー・終了コード 2 へ
/// 変換する）。
pub(crate) fn find_template(name: &str) -> Option<&'static Template> {
    TEMPLATES.iter().find(|t| t.name == name)
}

/// `examples/ssr-routing/` の全ファイル（8 件）を git の相対パス順・実行ビット
/// どおりに埋め込んだ固定配列（イシュー #500）。
///
/// `crates/cli/embedded-examples/ssr-routing/` は正本 `examples/ssr-routing/`
/// のバイト単位同梱コピーであり、乖離は
/// `cli/tests/example_publish_copy_drift.rs` が検知する。全ファイル
/// `executable: false`（正本側に実行ビット付きファイルが存在しないため）。
const SSR_ROUTING_EXAMPLE_FILES: &[TemplateFile] = &[
    TemplateFile {
        rel_path: "Cargo.lock",
        contents: include_str!("../embedded-examples/ssr-routing/Cargo.lock"),
        executable: false,
    },
    TemplateFile {
        rel_path: "Cargo.toml",
        contents: include_str!("../embedded-examples/ssr-routing/Cargo.toml.embed"),
        executable: false,
    },
    TemplateFile {
        rel_path: "README.md",
        contents: include_str!("../embedded-examples/ssr-routing/README.md"),
        executable: false,
    },
    TemplateFile {
        rel_path: "clippy.toml",
        contents: include_str!("../embedded-examples/ssr-routing/clippy.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "deny.toml",
        contents: include_str!("../embedded-examples/ssr-routing/deny.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "src/main.rs",
        contents: include_str!("../embedded-examples/ssr-routing/src/main.rs"),
        executable: false,
    },
    TemplateFile {
        rel_path: "structure.toml",
        contents: include_str!("../embedded-examples/ssr-routing/structure.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "tests/routing.rs",
        contents: include_str!("../embedded-examples/ssr-routing/tests/routing.rs"),
        executable: false,
    },
];

/// `examples/interactive-view-transitions/` の全ファイル（13 件）を git の
/// 相対パス順・実行ビットどおりに埋め込んだ固定配列（イシュー #503）。
///
/// `crates/cli/embedded-examples/interactive-view-transitions/` は正本
/// `examples/interactive-view-transitions/` のバイト単位同梱コピーであり、
/// 乖離は `cli/tests/example_publish_copy_drift.rs` が検知する。
/// `tools/wasm/build.sh` のみ実行ビット付き（`git ls-files -s` の mode
/// 100755、正本側の実行ビットをそのまま反映）。
const INTERACTIVE_VIEW_TRANSITIONS_EXAMPLE_FILES: &[TemplateFile] = &[
    TemplateFile {
        rel_path: "Cargo.lock",
        contents: include_str!("../embedded-examples/interactive-view-transitions/Cargo.lock"),
        executable: false,
    },
    TemplateFile {
        rel_path: "Cargo.toml",
        contents: include_str!(
            "../embedded-examples/interactive-view-transitions/Cargo.toml.embed"
        ),
        executable: false,
    },
    TemplateFile {
        rel_path: "README.md",
        contents: include_str!("../embedded-examples/interactive-view-transitions/README.md"),
        executable: false,
    },
    TemplateFile {
        rel_path: "clippy.toml",
        contents: include_str!("../embedded-examples/interactive-view-transitions/clippy.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "deny.toml",
        contents: include_str!("../embedded-examples/interactive-view-transitions/deny.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "src/main.rs",
        contents: include_str!("../embedded-examples/interactive-view-transitions/src/main.rs"),
        executable: false,
    },
    TemplateFile {
        rel_path: "static/embed.html",
        contents: include_str!(
            "../embedded-examples/interactive-view-transitions/static/embed.html"
        ),
        executable: false,
    },
    TemplateFile {
        rel_path: "structure.toml",
        contents: include_str!("../embedded-examples/interactive-view-transitions/structure.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "tests/state_machine.rs",
        contents: include_str!(
            "../embedded-examples/interactive-view-transitions/tests/state_machine.rs"
        ),
        executable: false,
    },
    TemplateFile {
        rel_path: "tools/wasm/build.sh",
        contents: include_str!(
            "../embedded-examples/interactive-view-transitions/tools/wasm/build.sh"
        ),
        executable: true,
    },
    TemplateFile {
        rel_path: "wasm/Cargo.lock",
        contents: include_str!("../embedded-examples/interactive-view-transitions/wasm/Cargo.lock"),
        executable: false,
    },
    TemplateFile {
        rel_path: "wasm/Cargo.toml",
        contents: include_str!(
            "../embedded-examples/interactive-view-transitions/wasm/Cargo.toml.embed"
        ),
        executable: false,
    },
    TemplateFile {
        rel_path: "wasm/src/lib.rs",
        contents: include_str!("../embedded-examples/interactive-view-transitions/wasm/src/lib.rs"),
        executable: false,
    },
];

/// `--example` の allowlist（イシュー #500・#503）。
///
/// サンプル名はここに列挙したコンパイル時定数との完全一致照合のみで解決し、
/// ユーザー入力から動的にパス・`include_str!` 対象を組み立てない
/// （`security.md` A01/A03、[`TEMPLATES`] と同じ方針）。配列順は
/// `fw new --example <unknown>` のエラーメッセージが提示する利用可能サンプル
/// 一覧の表示順（固定）にもなる。
pub(crate) const EXAMPLES: &[Template] = &[
    Template {
        name: "ssr-routing",
        files: SSR_ROUTING_EXAMPLE_FILES,
        // examples はパッケージ名を置換しない（モジュール doc コメント参照）。
        // この `needle` はどのファイルにも出現しないダミー文字列であり
        // （`substituted_files` が空のため置換ループは素通りする、
        // `new.rs::expand_template` 参照）、生成物はサンプルと全ファイル
        // バイト一致になる（`cli/tests/new_e2e.rs` が固定）。
        needle: "fandhe-frontend-example-placeholder-unused",
        substituted_files: &[],
    },
    Template {
        name: "interactive-view-transitions",
        files: INTERACTIVE_VIEW_TRANSITIONS_EXAMPLE_FILES,
        // 上記 "ssr-routing" と同じ理由でパッケージ名置換を行わない
        // （このサンプルは wasm/ 独立ワークスペースを持つが、そちらの
        // パッケージ名 `interactive-vt-wasm` も同様に置換対象外）。
        needle: "fandhe-frontend-example-placeholder-unused-interactive-view-transitions",
        substituted_files: &[],
    },
];

/// `name` に一致する [`Template`]（`EXAMPLES` 由来）を検索する。
///
/// 未知の名前は `None`（`new.rs::run_new` が使用法エラー・終了コード 2 へ
/// 変換する）。
pub(crate) fn find_example(name: &str) -> Option<&'static Template> {
    EXAMPLES.iter().find(|t| t.name == name)
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
    fn embed_template_is_registered() {
        let t = find_template("embed").expect("embed template must be registered");
        assert_eq!(t.name, "embed");
        assert_eq!(
            t.files.len(),
            2,
            "embed template must contain exactly embed.html and structure.toml"
        );
        assert!(
            t.substituted_files.is_empty(),
            "embed template has no cargo package, so no package-name substitution applies"
        );
    }

    #[test]
    fn unknown_template_name_resolves_to_none() {
        assert!(find_template("nonexistent").is_none());
    }

    #[test]
    fn ssr_routing_example_is_registered() {
        let e = find_example("ssr-routing").expect("ssr-routing example must be registered");
        assert_eq!(e.name, "ssr-routing");
        assert_eq!(
            e.files.len(),
            8,
            "ssr-routing example must contain exactly 8 files"
        );
        assert!(
            e.substituted_files.is_empty(),
            "examples do not substitute package names (see module doc comment, issue #500)"
        );
        assert!(
            e.files.iter().all(|f| !f.executable),
            "ssr-routing example has no executable files in the source"
        );
    }

    #[test]
    fn unknown_example_name_resolves_to_none() {
        assert!(find_example("nonexistent").is_none());
    }

    #[test]
    fn interactive_view_transitions_example_is_registered() {
        let e = find_example("interactive-view-transitions")
            .expect("interactive-view-transitions example must be registered");
        assert_eq!(e.name, "interactive-view-transitions");
        assert_eq!(
            e.files.len(),
            13,
            "interactive-view-transitions example must contain exactly 13 files"
        );
        assert!(
            e.substituted_files.is_empty(),
            "examples do not substitute package names (see module doc comment, issue #500)"
        );
        let mut executable: Vec<&str> = e
            .files
            .iter()
            .filter(|f| f.executable)
            .map(|f| f.rel_path)
            .collect();
        executable.sort_unstable();
        assert_eq!(
            executable,
            vec!["tools/wasm/build.sh"],
            "interactive-view-transitions example must have exactly one executable file \
             (tools/wasm/build.sh)"
        );
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
        // イシュー #411: `tools/wasm/build.sh` は app テンプレート固有の
        // 実行可能ファイル（wasm ビルド手順、default テンプレートには存在しない）。
        let expected_app: &[&str] = &[
            "tools/npm-asset-build/apply_exempt.py",
            "tools/npm-asset-build/check_static_only.py",
            "tools/npm-asset-build/install.sh",
            "tools/wasm/build.sh",
        ];
        let expected_embed: &[&str] = &[];

        for (name, expected) in [
            ("default", expected_default),
            ("app", expected_app),
            ("embed", expected_embed),
        ] {
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
