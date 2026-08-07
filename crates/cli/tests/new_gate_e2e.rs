//! `fw new` 生成直後に `fw gate` が PASS することを固定する e2e テスト
//! （イシュー #351、親イシュー #338「全プロジェクトが同一構成 = `fw gate`
//! がそのまま効く」の受け入れ条件）。
//!
//! `fw new`（イシュー #350、`cli/src/new.rs`）は `templates/default/` を
//! 決定的に展開するだけであり、`fw gate`（`cli/src/gate.rs::run_gate`）は
//! `structure.toml` を唯一の情報源として検証対象クレートを決定する。両者の
//! 前提（テンプレートの `structure.toml` / `clippy.toml` / `deny.toml` と
//! gate の実装）がドリフトすると、生成直後のプロジェクトが無編集で
//! BLOCKED になりかねない。本ファイルは
//! `fw new <name> --dir <scratch>` → `fw gate --project <scratch>/<name>`
//! を実バイナリとして直列実行し、7 チェック（type_check /
//! default_escape_check / url_validation_check / lint / lint_wasm32 / test /
//! policy）が出揃うこと、および policy を除く 6 チェックが常に PASS する
//! ことを断定する回帰テストである（イシュー #1174 で `lint_wasm32` を追加。
//! `templates/default/` は `role = "client-entrypoint"` を宣言しないため
//! not-applicable PASS で常に環境非依存に通過する）。
//!
//! `policy`（cargo-deny 依存）のみ実行環境（cargo-deny の導入有無）で結果が
//! 変わるため、`cli/tests/scenarios/bugfix_escape.rs::baseline_passes_gate`
//! が確立した「スキップ・`#[ignore]` を使わず両分岐を断定する」方針を
//! そのまま踏襲する（未導入環境では `policy` の failed 出力が
//! `environment error: ` で始まる＝コード起因ではなく環境起因のみで
//! あることまで確認し、BLOCKED を黙示的な想定外失敗と混同しない）。
//!
//! 本ファイルの削除・弱体化（アサーション削除・`#[ignore]` 付与等）は
//! `fw new`/`fw gate` 前提のドリフト検知回帰を失わせるため行わない
//! （`.claude/rules/coding-rust.md` 「テストの `#[ignore]` 追加でごまかさない」）。
//!
//! `cli/tests/support/mod.rs`（`negative_cases.rs` 等が使う `fw gate` 専用
//! フィクスチャ基盤）の `run_fw_gate`/`check_passed`/`cargo_deny_available`/
//! `ScratchProject` を再利用しつつ、`fw new` の起動と scratch ディレクトリの
//! 準備のみ `cli/tests/new_e2e.rs` と同方針の薄いローカルヘルパーを持つ
//! （`new_e2e.rs` 冒頭コメントが明文化する「テストターゲット独立の制約による
//! 意図的な複製」を踏襲）。
//!
//! イシュー #895: `fw_new_app_template_*` の 2 テスト（app テンプレート、
//! crates.io バージョン依存で完結する）は、version-bump-guard・
//! `template_vendor_drift`・本ファイルの三すくみ（`docs/ci/
//! version-bump-publish-order-gap.md` 参照）により、バンプ先バージョンが
//! crates.io へ未公開の間は必ず fail する構造的デッドロックを抱える
//! （`template-app-wasm-smoke` ジョブ（イシュー #885）と同型の問題）。
//! [`apply_patch_template_smoke`] が同ジョブと同じ `xtask
//! patch-template-smoke` サブコマンドをサブプロセスとして再利用し、未公開
//! 依存のみへ `[patch.crates-io]` フォールバックを適用する。全依存公開済み
//! の通常時はファイル無変更のまま `resolution=crates-io` のみが報告され、
//! crates.io 実 index での依存解決の検証能力は変わらない。

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Once;
use support::{
    cargo_deny_available, check_passed, run_fw, run_fw_gate, run_fw_gate_with_target_dir,
    ScratchProject,
};

/// `fw new` を実バイナリとして起動し (終了コード, stdout, stderr) を返す
/// （`cli/tests/new_e2e.rs::run_fw_new` と同一方針。テストターゲット独立の
/// 制約による意図的な複製）。
fn run_fw_new(extra_args: &[&str]) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_fw"))
        .arg("new")
        .args(extra_args)
        .output()
        .expect("failed to spawn `fw` binary");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// `fw new` の展開先となる一意な scratch ディレクトリ（`support::scratch_root`
/// 配下、`ScratchProject` の Drop ガードで後始末する）。
///
/// self-hosted runner の共有 `/tmp`・共有 `CARGO_TARGET_DIR` との衝突を避ける
/// ため、テスト名・PID・ナノ秒を含めて一意化する
/// （`cli/tests/support/mod.rs::scratch_root` と同一方針、`.claude/rules/ci.md`
/// 準拠）。呼び出しのたびに [`cleanup_stale_scratch`] を起動し、過去実行の
/// 残置物（プロセス kill で `ScratchProject` の Drop ガードが走らなかった
/// もの）を回収する（イシュー #637）。
///
/// # 単調カウンタによる同一プロセス内衝突対策（イシュー #729 PR #782 CI 障害）
///
/// 本ファイルの `fw_new_example_*_output_passes_fw_gate` 5 件はいずれも本関数を
/// 引数なしで呼ぶため、名前の一意性は「PID + ナノ秒タイムスタンプ」のみに依存
/// していた。cargo test の既定並列実行では libtest がこの 5 件をほぼ同時に
/// 別スレッドへディスパッチするため、プロセス起動直後の狭い時間窓に複数
/// スレッドが `SystemTime::now()` を呼ぶ状況が生じる。self-hosted runner
/// （コンテナ／VM）ではクロックソースの実効分解能がナノ秒を下回らない場合が
/// あり、この窓でナノ秒値が衝突すると 2 スレッドが同一パスを [`unique_scratch_dir`]
/// で同時に扱うことになり、一方の `remove_dir_all`（本関数冒頭の掃除、または
/// もう一方の `ScratchProject` Drop）がもう一方の `fw new`/`fw gate` 実行中の
/// ディレクトリを消してしまう（実際に PR #782 の CI で `fw gate` が
/// `Could not locate working directory` で BLOCKED になった障害と一致）。
/// `AtomicU64` の単調カウンタを名前へ追加することで、PID・ナノ秒が偶然一致
/// しても同一プロセス内で名前が重複しないことを保証する（クロック分解能に
/// 依存しない一意性）。
fn unique_scratch_dir() -> PathBuf {
    cleanup_stale_scratch();
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    let root = support::scratch_root();
    let dir = root.join(format!(
        "fw-new-gate-e2e-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        seq
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("failed to create scratch dir");
    dir
}

/// 過去実行の残置物を起動時に 1 プロセス 1 回だけ回収する
/// （イシュー #637 の根本原因: `CARGO_TARGET_TMPDIR` の実行時参照が cargo の
/// 仕様上常に失敗し `/tmp` へ落ちていたため、本ファイルが命名する 2 プレフィックス
/// （`fw-new-gate-e2e-*` / `fw-example-gate-shared-target-*`）のディレクトリが
/// `/tmp` へ恒久的に蓄積していた）。
///
/// 配置是正（[`unique_scratch_dir`]・[`example_shared_target_dir`] が
/// `support::scratch_root()` = コンパイル時 `CARGO_TARGET_TMPDIR` を使うよう
/// 修正済み）を根本対策としつつ、本関数は (1) 旧配置先 `std::env::temp_dir()`
/// に残る過去の `/tmp` 残置の自己回収、(2) 新配置先でも「所有者テストが
/// 不安定」（`example_shared_target_dir` doc 参照）なため蓄積し得る世代の
/// 上限管理、の 2 役を担う。`unique_scratch_dir`/`example_shared_target_dir`
/// の双方から呼ばれ、`Once` により本ファイルの全テストのどれが最初に走っても
/// 1 回だけ実行される。
fn cleanup_stale_scratch() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        for root in [support::scratch_root(), std::env::temp_dir()] {
            sweep_stale_scratch_dir(&root);
        }
    });
}

/// `root` 直下（非再帰）を走査し、本ファイルが命名する 2 プレフィックスに
/// 一致し、かつ名前中の PID が非生存のエントリのみを削除する。
///
/// 再帰的に子ディレクトリへ潜らない・プレフィックス完全一致のみを対象・
/// PID パース不能なら無視、という制約は「他プロセスが所有する無関係な
/// ディレクトリを誤って削除しない」ための安全弁（`cleanup_stale_scratch`
/// 呼び出し元の doc・イシュー #637 のセキュリティ考慮を参照）。
/// `unique_scratch_dir`/`example_shared_target_dir` からは `Once` 経由で、
/// 回帰テスト（`cleanup_stale_scratch_removes_dead_pid_entries_and_keeps_live_ones`）
/// からは `Once` を経由せず直接呼ばれる。
fn sweep_stale_scratch_dir(root: &std::path::Path) {
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        // symlink は削除対象外（パス操作の安全弁）。実ディレクトリのみを見る。
        let is_real_dir = std::fs::symlink_metadata(&path)
            .map(|meta| meta.is_dir())
            .unwrap_or(false);
        if !is_real_dir {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(pid) = extract_scratch_dir_pid(name) else {
            continue;
        };
        if scratch_dir_pid_is_stale(&path, pid) {
            let _ = std::fs::remove_dir_all(&path);
        }
    }
}

/// ディレクトリ名からイシュー #637 が管理する 2 プレフィックス経由で PID を
/// 抽出する。`fw-new-gate-e2e-<pid>-<nanos>` と
/// `fw-example-gate-shared-target-<pid>` の 2 形式（[`unique_scratch_dir`]・
/// [`example_shared_target_dir`] が命名）のみに反応し、それ以外の名前
/// （無関係なディレクトリ）は `None` を返して素通りさせる。
fn extract_scratch_dir_pid(name: &str) -> Option<u32> {
    for prefix in ["fw-new-gate-e2e-", "fw-example-gate-shared-target-"] {
        if let Some(rest) = name.strip_prefix(prefix) {
            let pid_segment = rest.split('-').next().unwrap_or("");
            if let Ok(pid) = pid_segment.parse::<u32>() {
                return Some(pid);
            }
        }
    }
    None
}

/// 「非生存」の消去対象と判定するための最低経過時間（1 時間）。
///
/// self-hosted runner 上では `/cargo-target`（延いては本ファイルが使う
/// `support::scratch_root()` = コンパイル時 `CARGO_TARGET_TMPDIR`）が
/// コンテナ化された複数 CI ジョブ間で共有され得る。PID の生存確認
/// （`/proc/<pid>` の存在）は **PID 名前空間ローカル** の判定であり、
/// 別ジョブ（別 PID 名前空間）からは「このジョブ自身が現に使っている
/// scratch ディレクトリ」の PID が常に非存在＝stale と誤判定され得る
/// （イシュー #637 で発生した実障害: 直前まで `fw new`/`fw gate` を実行
/// 中だった scratch ディレクトリが別ジョブの sweep に削除され、
/// `fw_new_app_template_output_passes_fw_gate` が cargo の working
/// directory 消失で FAILED になった）。
///
/// このため PID 生存確認のみでは安全側判定として不十分であり、
/// 「PID 非生存**かつ** mtime が本閾値を超えている」の両条件を必須と
/// する（[`scratch_dir_pid_is_stale`] 参照）。本閾値はテストプロジェクト
/// 1 本の `fw gate`（cargo build/test/clippy 込み）が要する最大時間を
/// 十分に超え、かつ旧世代の回収を過度に遅延させない値として 1 時間とする。
const STALE_MIN_AGE: std::time::Duration = std::time::Duration::from_secs(60 * 60);

/// `path` の mtime が [`STALE_MIN_AGE`] を超えているかを判定する
/// （メタデータ取得・時刻計算に失敗した場合は「新しい」＝削除しない安全側
/// フォールバック）。
fn is_older_than_stale_threshold(path: &std::path::Path) -> bool {
    std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|modified| modified.elapsed().ok())
        .map(|elapsed| elapsed >= STALE_MIN_AGE)
        .unwrap_or(false)
}

/// `pid` が生存していないかを判定する（生存していれば削除しない安全側判定）。
///
/// Linux self-hosted runner を前提に `/proc/<pid>` の存在確認を第一の
/// シグナルとするが、これは PID 名前空間ローカルの判定であり、コンテナ化
/// された別 CI ジョブから見ると「他ジョブが現に使っている生存 PID」も
/// 「非存在」に見え得る（イシュー #637、[`STALE_MIN_AGE`] のコメント参照）。
/// そのため `/proc` 判定だけでは削除の根拠として不十分とし、いずれの分岐
/// （`/proc` 判定・`/proc` が使えない環境の mtime フォールバックの双方）でも
/// 「`path` の mtime が [`STALE_MIN_AGE`] を超えている」ことを削除の
/// **必須の追加条件**（AND 条件）とする。これにより、たとえ PID 判定が
/// 別名前空間との衝突で誤って「非生存」と示しても、直近に作成・更新された
/// （＝現に使用中の可能性が高い）ディレクトリは閾値時間が経過するまで
/// 保護される。
fn scratch_dir_pid_is_stale(path: &std::path::Path, pid: u32) -> bool {
    if !is_older_than_stale_threshold(path) {
        return false;
    }
    if std::path::Path::new("/proc/self").exists() {
        return !std::path::Path::new(&format!("/proc/{pid}")).exists();
    }
    // `/proc` が使えない環境（非 Linux 等）では PID 生存を判定できないため、
    // 上記の mtime 閾値チェック（既に通過済み）のみを根拠に stale 扱いにする。
    true
}

/// examples e2e 5 件（`fw_new_example_*_output_passes_fw_gate`）が共有する
/// `CARGO_TARGET_DIR`（イシュー #505・#609）。
///
/// # 背景
///
/// `run_fw_gate`（`support::run_fw` 既定）は `project_dir/target` を専用
/// `CARGO_TARGET_DIR` として起動するため、examples 5 例は毎回コールドで
/// fandhe-frontend-core/-app/-server 等の crates.io 依存を重複ビルドしていた。
/// 本ヘルパーが返す共有ディレクトリを [`run_fw_gate_with_target_dir`] と
/// `cargo run` smoke（各テスト末尾）の双方に明示指定することで、2 例目
/// 以降は依存クレートの再ビルドを避けられる。
///
/// # 安全性根拠（`support::run_fw` の偽陰性警告との関係）
///
/// `support::run_fw` doc コメントが警告する偽陰性リスク（`CARGO_TARGET_DIR`
/// 共有によりフィンガープリント衝突で直前フィクスチャの結果を誤って
/// 再利用する）は「同名パッケージを異内容で再利用する欠陥注入フィクスチャ」
/// （`negative_cases.rs` 等）に固有のリスクである。examples 5 例は
/// パッケージ名が相互に一意（`fandhe-frontend-example-ssr-routing` /
/// `-ssg-blog` / `-dist-server-docker` / `-interactive-view-transitions` /
/// `-headless-pre-styled-ui`）であり、リーフクレート自体は `fw new` が
/// [`unique_scratch_dir`] 配下へ毎回新規展開する（mtime が必ず新しくなる）
/// ため必ず再ビルドされる。crates.io 依存側もバージョン不変であり、cargo
/// 自身の build-dir ロック（複数 `cargo` 起動の直列化）により並行実行も安全。
///
/// # クリーンアップ方針（イシュー #637）
///
/// 特定のテストが所有者ではない（5 テストが共有し、最後に終わるテストの
/// 特定が不安定）ため `ScratchProject` の Drop ガードでは消さない。代わりに
/// 2 段構えで有界化する。
///
/// 1. **配置是正**: `support::scratch_root()` はコンパイル時に確定する
///    `env!("CARGO_TARGET_TMPDIR")`（CI では `/cargo-target/tmp`、ローカルでは
///    `target/tmp`）を既定とする（かつての実行時 `CARGO_TARGET_TMPDIR` 参照は
///    cargo の仕様上常に失敗し `/tmp` へ落ちていた）。これにより `cargo clean`
///    の既存管理範囲に収まる（旧 self-hosted 時代は
///    `.github/workflows/runner-maintenance.yml` の stale tmp 検査も併用
///    していたが、CI runner 方針のホステッドランナー既定への反転〔#1220〕に
///    伴い同ワークフローは廃止済み、イシュー #1237）
/// 2. **世代管理**: [`cleanup_stale_scratch`] が本関数呼び出し時に起動し、
///    PID サフィックスが非生存（dead）の旧世代ディレクトリを回収する。
///    「同一テストバイナリ内の 5 テスト間でのみ共有・並行 CI ジョブ／別回とは
///    隔離」という PID サフィックスによる分離（本関数）は不変であり、
///    ラン内キャッシュ共有の意図（イシュー #505）も維持される
fn example_shared_target_dir() -> PathBuf {
    cleanup_stale_scratch();
    support::scratch_root().join(format!(
        "fw-example-gate-shared-target-{}",
        std::process::id()
    ))
}

/// このテストバイナリ（`crates/cli`）が属するワークスペースのルートを返す
/// （イシュー #895）。`[patch.crates-io]` へ注入する path 参照は絶対パスで
/// なければならないため `canonicalize()` する。`CARGO_MANIFEST_DIR` は
/// クレートマニフェストのディレクトリ（`crates/cli`）であり、そこから
/// 2 階層上がリポジトリルート（`crates/*` の親）。
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to canonicalize repo root from CARGO_MANIFEST_DIR")
}

/// `fw new --template app` 生成直後のプロジェクトへ、`xtask
/// patch-template-smoke`（`crates/xtask/src/patch_template_smoke.rs`、
/// イシュー #885）をサブプロセスとして適用する（イシュー #895）。
///
/// # 背景・不変条件
///
/// `templates/app` は fandhe-frontend-core/-app/-wasm-client への crates.io
/// バージョン依存で完結する。version-bump-guard（バンプ強制）と
/// `template_vendor_drift`（テンプレート要求とバンプ後の正本 `version` の
/// 一致強制）を満たすバンプ PR では、crates.io 公開（`release.yml`、
/// `workflow_dispatch` + `mode: publish` の明示選択によりマージ後に運用）
/// までの間、生成プロジェクトの依存解決が crates.io 実 index では必ず失敗
/// する（三すくみ、`docs/ci/version-bump-publish-order-gap.md` 参照）。
///
/// `.github/workflows/ci.yml` の `template-app-wasm-smoke` ジョブが導入
/// 済みの同名フォールバックをここでも再利用することで、`fw
/// _new_app_template_*` e2e が同じデッドロックへ陥らないようにする。
/// crates.io sparse index への fail-closed 照会で「未公開」を確認できた
/// 依存のみ `[patch.crates-io]`（このテストが検証対象とする checkout 済み
/// コードそのものへの path 参照）へ切り替え、対応する `Cargo.lock` を
/// 削除する。全依存が公開済みの通常時はマニフェスト無変更のまま
/// `resolution=crates-io` のみが報告され、`fw gate --locked` は crates.io
/// 実解決の検証能力を保ったまま実行される（弱体化ではない）。
///
/// 緩和用の環境変数・CLI フラグは設けない（迂回禁止原則、`docs/ci/
/// version-bump-publish-order-gap.md` §5 項 4）。crates.io 到達不可・
/// 想定外応答は `environment error: ` プレフィックス付きで fail-closed に
/// panic する（テストの弱体化で吸収しない）。
///
/// # Panics
///
/// - `cargo run -p xtask -- patch-template-smoke` の起動・終了コード非 0
/// - フォールバック発動後の `cargo generate-lockfile` の起動・終了コード非 0
fn apply_patch_template_smoke(project_dir: &Path) {
    let repo_root = repo_root();

    // シェル非経由（`Command` への引数個別渡し）。project_dir/repo_root は
    // このテストバイナリが自ら生成した scratch パス・`canonicalize()` 済み
    // リポジトリルートであり、外部入力ではない（A03 インジェクション対策）。
    let output = Command::new("cargo")
        .args([
            "run",
            "--locked",
            "-p",
            "xtask",
            "--",
            "patch-template-smoke",
        ])
        .arg("--project-dir")
        .arg(project_dir)
        .arg("--repo-root")
        .arg(&repo_root)
        .current_dir(&repo_root)
        .output()
        .expect("failed to spawn `cargo run -p xtask -- patch-template-smoke`");

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    // CI 側の `grep '^template-app-wasm-smoke:'` 契約と同じ 1 行サマリを
    // このテストのログにも転送する（発動有無をテスト実行ログ上で追跡可能に
    // するため。行単位で転送しスレッド間の行分断を避ける）。
    for line in stdout.lines() {
        if line.starts_with("template-app-wasm-smoke: ") {
            println!("{line}");
        }
    }

    assert!(
        output.status.success(),
        "`cargo run -p xtask -- patch-template-smoke` failed \
         (project_dir={project_dir}, repo_root={repo_root}): \
         `environment error: ` プレフィックスがあれば crates.io sparse index \
         への到達性起因（runner/ネットワーク要因）、なければテンプレートの \
         依存宣言形式または repo-root 側クレート不整合などコード起因の \
         可能性が高い。stdout={stdout} stderr={stderr}",
        project_dir = project_dir.display(),
        repo_root = repo_root.display(),
    );

    if stdout.contains("resolution=path-override") {
        println!(
            "new_gate_e2e: patch-template-smoke フォールバックが発動した \
             (resolution=path-override)。[patch.crates-io] 注入に伴い削除された \
             ルート Cargo.lock を `cargo generate-lockfile` で再生成する \
             （再現性低下は既知・許容: `docs/ci/version-bump-publish-order-gap.md` \
             §5 項 2 と同一判断）。"
        );

        // `fw gate` は cargo 起動へ常に `--locked` を付与するため、削除済みの
        // Cargo.lock を残したままだと `fw gate` 自体が解決不能で fail する。
        // wasm/Cargo.lock は `fw gate` の対象外（`fw gate` はルートのみを
        // ビルド対象にする）のため再生成不要。
        let generate_output = Command::new("cargo")
            .arg("generate-lockfile")
            .current_dir(project_dir)
            .output()
            .expect("failed to spawn `cargo generate-lockfile`");

        assert!(
            generate_output.status.success(),
            "`cargo generate-lockfile` failed after patch-template-smoke fallback \
             (project_dir={project_dir}): crates.io（https://index.crates.io・\
             https://static.crates.io）への到達性が前提。stdout={stdout} stderr={stderr}",
            project_dir = project_dir.display(),
            stdout = String::from_utf8_lossy(&generate_output.stdout),
            stderr = String::from_utf8_lossy(&generate_output.stderr),
        );
    }
}

/// `fw new` で生成した直後のプロジェクトに対し `fw gate` を実行し、
/// チェックセット（7 件）が JSON にすべて現れること・環境に依存しない
/// 6 チェック（type_check / default_escape_check / url_validation_check /
/// lint / lint_wasm32 / test）が常に PASS することを断定する（イシュー
/// #1174）。
///
/// `policy` は cargo-deny の導入有無で分岐し、両分岐とも「コード起因ではなく
/// 環境要因でのみ BLOCKED になり得る」ことまで確認する
/// （`bugfix_escape.rs::baseline_passes_gate` と同一方針）。
#[test]
fn fw_new_output_passes_fw_gate() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) =
        run_fw_new(&["gate-pass-app", "--dir", &scratch.to_string_lossy()]);
    assert_eq!(
        new_code, 0,
        "fw new が失敗した: stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("gate-pass-app");
    let (gate_code, gate_stdout, gate_stderr) = run_fw_gate(&project_dir);

    // チェックセット自体のドリフト検知（gate.rs 側でチェックが増減しても
    // ここで検出できるよう、7 件すべてが JSON に現れることを断定する）。
    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "lint_wasm32",
        "test",
        "policy",
    ] {
        assert!(
            gate_stdout.contains(&format!("\"name\":\"{name}\"")),
            "fw gate のレポートにチェック `{name}` が現れない: stdout={gate_stdout}"
        );
    }

    // 環境（cargo-deny 導入有無）に依存しない 6 チェックは常に PASS する
    // はず（`lint_wasm32` は `templates/default/` が `role =
    // "client-entrypoint"` を宣言しないため not-applicable PASS で環境
    // 非依存、イシュー #1174）。ここが failed の場合はテンプレートと gate
    // 前提のドリフト（clippy.toml の disallowed-methods 欠落・
    // structure.toml の宣言クレート不一致・型不正コードの混入等）を意味する。
    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "lint_wasm32",
        "test",
    ] {
        assert_eq!(
            check_passed(&gate_stdout, name),
            Some(true),
            "fw new 生成直後のプロジェクトで `{name}` が失敗した（テンプレートと \
             fw gate の前提がドリフトしている）: stdout={gate_stdout} stderr={gate_stderr}"
        );
    }

    if cargo_deny_available() {
        assert_eq!(
            gate_code, 0,
            "cargo-deny 導入環境では fw new 生成直後は PASS するはず: \
             stdout={gate_stdout} stderr={gate_stderr}"
        );
        assert!(
            gate_stdout.contains("\"gate_result\":\"PASS\""),
            "stdout={gate_stdout}"
        );
        assert_eq!(
            check_passed(&gate_stdout, "policy"),
            Some(true),
            "stdout={gate_stdout}"
        );
    } else {
        // cargo-deny 未導入環境では policy のみ fail-closed で failed になり
        // 全体として BLOCKED になるが、その failed 出力が「環境エラーである
        // こと」を示す `environment error: ` プレフィックスで始まることまで
        // 確認する（`.claude/rules/ci.md` §ツール前提の明示・gate.rs
        // `ENVIRONMENT_ERROR_PREFIX` 参照）。これにより「テンプレート/gate
        // 前提のコード起因ドリフトによる BLOCKED」と「cargo-deny 未導入と
        // いう環境要因による BLOCKED」を取り違えない。
        assert_eq!(
            gate_code, 1,
            "cargo-deny 未導入環境では policy の fail-closed により BLOCKED \
             (終了コード 1) のはず: stdout={gate_stdout}"
        );
        assert!(
            gate_stdout.contains("\"gate_result\":\"BLOCKED\""),
            "stdout={gate_stdout}"
        );
        assert_eq!(
            check_passed(&gate_stdout, "policy"),
            Some(false),
            "stdout={gate_stdout}"
        );
        assert!(
            gate_stdout.contains("environment error: "),
            "policy の failed 出力は environment error であることを明示する \
             プレフィックスを含むはず（コード起因の FAIL と誤認させない）: \
             stdout={gate_stdout}"
        );
    }
}

/// PR #358 Bugbot 指摘（イシュー #351）の e2e 回帰テスト:
/// `fw_new_output_passes_fw_gate` の `default_escape_check == Some(true)` は
/// 「未レビュー `raw_html()` が存在しないので PASS」と「走査対象パスが
/// 実在せず走査自体がスキップされたので PASS（無意味な PASS）」を区別
/// できない。本テストは `fw new` 生成直後のプロジェクトの実クレート配置先
/// （プロジェクトルート直下 `src/`）に未レビューの `raw_html()` 呼び出しを
/// 注入し、`default_escape_check` が実際に走査を実行して violation を
/// 検出（failed）することを断定する。これにより `structure.toml` の
/// `[directories.root]` 予約名規約と `fw gate`（`escape_check_src_dir`,
/// `cli/src/gate.rs`）の前提が一致していること（保険層が生成直後の
/// プロジェクトで機能していること）を固定する。
#[test]
fn fw_new_output_default_escape_check_detects_injected_violation_in_root_src() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) = run_fw_new(&[
        "gate-pass-violation-app",
        "--dir",
        &scratch.to_string_lossy(),
    ]);
    assert_eq!(
        new_code, 0,
        "fw new が失敗した: stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("gate-pass-violation-app");

    // `fw new` が展開したクレートは `<project_dir>/src/main.rs`
    // （プロジェクトルート直下、テンプレートの `[directories.root]` 規約）
    // に配置される。ここへ未レビューの `raw_html()` 呼び出しを追記する
    // （`#[expect(clippy::disallowed_methods, ...)]` を伴わないため
    // `default_escape_check` の違反として検出されるはず）。
    let main_rs = project_dir.join("src").join("main.rs");
    let mut content = std::fs::read_to_string(&main_rs).expect("failed to read src/main.rs");
    content.push_str("\nfn unreviewed_raw_html_probe() {\n    raw_html(\"x\");\n}\n");
    std::fs::write(&main_rs, content).expect("failed to write src/main.rs");

    let (gate_code, gate_stdout, gate_stderr) = run_fw_gate(&project_dir);

    assert_eq!(
        check_passed(&gate_stdout, "default_escape_check"),
        Some(false),
        "src/ 直下（`root` 規約）に注入した未レビュー raw_html() 呼び出しが \
         default_escape_check で検出されなかった（走査がスキップされている \
         疑い、PR #358 Bugbot 指摘のドリフト再発）: stdout={gate_stdout} \
         stderr={gate_stderr}"
    );
    assert!(
        gate_stdout.contains("main.rs"),
        "default_escape_check の failed 出力は違反ファイルを file:line で \
         列挙するはず: stdout={gate_stdout}"
    );
    assert_ne!(
        gate_code, 0,
        "default_escape_check が failed の場合 fw gate 全体も非ゼロで \
         終了するはず: stdout={gate_stdout}"
    );
}

/// イシュー #353: `fw new` 生成直後のプロジェクトで `fw structure` /
/// `fw impact` が「`root` 慣習未対応」由来の解析不能に陥らないことを固定する。
///
/// - `fw structure`: 旧実装は `project_dir.join("root")`（実在しないパス）を
///   見て「declared directory does not exist」で必ず exit 1 だった
///   （`structure::dir_fs_path` 導入により修正）。
/// - `fw impact`: 旧実装は `member_dir_name` が `manifest_dir == workspace_root`
///   を `ImpactError::Scan`（「manifest_dir equals workspace_root」）として
///   拒否するため `fw impact` 全体が exit 1 で失敗していた。テンプレートの
///   `find_item`/`Item`（`templates/default/src/main.rs`）はいずれも
///   トップレベル非公開宣言（`pub` ではない）であるため定義元が見つからず
///   `ImpactError::SymbolNotFound` になるのが**新実装での正しい**挙動である
///   （`component_boundary::extract_from_source` はトップレベル `pub` 宣言のみ
///   走査対象とする契約、`cli/src/component_boundary.rs` 参照）。本テストは
///   終了コードそのものではなく、エラーメッセージが新実装の
///   `SymbolNotFound`（"no definition found for symbol"）であり、旧実装の
///   `Scan` エラー（"manifest_dir equals workspace_root"）ではないことを
///   断定することで、`root` 慣習未対応の解析不能状態からの回復を固定する。
#[test]
fn fw_new_output_fw_structure_succeeds_and_fw_impact_does_not_hit_root_scan_error() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) =
        run_fw_new(&["structure-impact-app", "--dir", &scratch.to_string_lossy()]);
    assert_eq!(
        new_code, 0,
        "fw new が失敗した: stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("structure-impact-app");

    let (structure_code, structure_stdout, structure_stderr) =
        run_fw("structure", &[], &project_dir);
    assert_eq!(
        structure_code, 0,
        "fw new 生成直後のプロジェクトで fw structure は exit 0 のはず \
         （`root` 慣習のディレクトリ実在誤検知の非回帰）: \
         stdout={structure_stdout} stderr={structure_stderr}"
    );

    let (impact_code, impact_stdout, impact_stderr) =
        run_fw("impact", &["find_item"], &project_dir);
    assert_eq!(
        impact_code, 1,
        "find_item は非公開宣言のため symbol not found（検証違反、終了コード 1）が \
         正しい新実装の挙動: stdout={impact_stdout} stderr={impact_stderr}"
    );
    assert!(
        impact_stderr.contains("no definition found for symbol"),
        "新実装では SymbolNotFound として fail-closed するはず（旧実装は \
         root member の Scan エラーで別メッセージになっていた）: \
         stderr={impact_stderr}"
    );
    assert!(
        !impact_stderr.contains("manifest_dir equals workspace_root"),
        "旧実装の root member 解決エラー（member_dir_name の Scan エラー）が \
         再発していないこと（イシュー #353 のリグレッション封じ）: \
         stderr={impact_stderr}"
    );
}

/// イシュー #378 受け入れ条件 2: `fw new --template app`（fandhe-frontend-core/fandhe-frontend-app
/// 依存の拡充テンプレート、vendor 同梱）が生成直後に `fw gate` PASS する
/// ことを固定する。`fw_new_output_passes_fw_gate`（`default` テンプレート）
/// と同一の断定方針（環境依存の `policy` のみ両分岐を確認、他 6 チェックは
/// 常に PASS）を踏襲する。vendored crate 群のコンパイルを伴うため
/// `default` より実行時間が長い（PR 本文に記載する既知事項）。
#[test]
fn fw_new_app_template_output_passes_fw_gate() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) = run_fw_new(&[
        "gate-pass-app-template",
        "--template",
        "app",
        "--dir",
        &scratch.to_string_lossy(),
    ]);
    assert_eq!(
        new_code, 0,
        "fw new --template app が失敗した: stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("gate-pass-app-template");

    // イシュー #895: バンプ先バージョンが crates.io へ未公開の間の構造的
    // デッドロック（`template-app-wasm-smoke` ジョブと同型）を回避する。
    // 全依存公開済みの通常時はファイル無変更のまま素通りする
    // （`apply_patch_template_smoke` doc 参照）。
    apply_patch_template_smoke(&project_dir);

    let (gate_code, gate_stdout, gate_stderr) = run_fw_gate(&project_dir);

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "lint_wasm32",
        "test",
        "policy",
    ] {
        assert!(
            gate_stdout.contains(&format!("\"name\":\"{name}\"")),
            "fw gate のレポートにチェック `{name}` が現れない: stdout={gate_stdout}"
        );
    }

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "lint_wasm32",
        "test",
    ] {
        assert_eq!(
            check_passed(&gate_stdout, name),
            Some(true),
            "fw new --template app 生成直後のプロジェクトで `{name}` が失敗した \
             （app テンプレートと fw gate の前提がドリフトしている）: \
             stdout={gate_stdout} stderr={gate_stderr}"
        );
    }

    if cargo_deny_available() {
        assert_eq!(
            gate_code, 0,
            "cargo-deny 導入環境では fw new --template app 生成直後は PASS するはず: \
             stdout={gate_stdout} stderr={gate_stderr}"
        );
        assert!(
            gate_stdout.contains("\"gate_result\":\"PASS\""),
            "stdout={gate_stdout}"
        );
    } else {
        assert_eq!(
            gate_code, 1,
            "cargo-deny 未導入環境では policy の fail-closed により BLOCKED \
             (終了コード 1) のはず: stdout={gate_stdout}"
        );
        assert!(
            gate_stdout.contains("environment error: "),
            "policy の failed 出力は environment error であることを明示する \
             プレフィックスを含むはず: stdout={gate_stdout}"
        );
    }
}

/// イシュー #378: `fw new --template app` 生成物への未レビュー `raw_html()`
/// 注入が `default_escape_check` で検出されることを固定する
/// （`fw_new_output_default_escape_check_detects_injected_violation_in_root_src`
/// の app テンプレート版）。app は fandhe-frontend-core に依存するため `raw_html()` が
/// 実際に解決可能な呼び出しになる点が `default`（fandhe-frontend-core 非依存）との
/// 差分であり、clippy.toml の disallowed-methods が依存追加によって
/// 初めて実効化されることを固定する（実装計画 §7 セキュリティ考慮）。
#[test]
fn fw_new_app_template_default_escape_check_detects_injected_violation() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) = run_fw_new(&[
        "gate-pass-app-violation",
        "--template",
        "app",
        "--dir",
        &scratch.to_string_lossy(),
    ]);
    assert_eq!(
        new_code, 0,
        "fw new --template app が失敗した: stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("gate-pass-app-violation");

    // イシュー #895: 未公開バージョンウィンドウ中は依存解決不能により
    // clippy の disallowed-methods チェック自体が走らないまま「失敗だから
    // PASS」となり検出能力の検証が空洞化する。デッドロックはしない構成だが
    // 安全側の判断として本テストにも適用する（実装計画 §4 ステップ 1-4）。
    apply_patch_template_smoke(&project_dir);

    let main_rs = project_dir.join("src").join("main.rs");
    let mut content = std::fs::read_to_string(&main_rs).expect("failed to read src/main.rs");
    content.push_str(
        "\nfn unreviewed_raw_html_probe() {\n    fandhe_frontend_core::raw_html(\"x\");\n}\n",
    );
    std::fs::write(&main_rs, content).expect("failed to write src/main.rs");

    let (gate_code, gate_stdout, gate_stderr) = run_fw_gate(&project_dir);

    assert_eq!(
        check_passed(&gate_stdout, "default_escape_check"),
        Some(false),
        "app テンプレート生成物の src/ 直下に注入した未レビュー raw_html() \
         呼び出しが default_escape_check で検出されなかった: \
         stdout={gate_stdout} stderr={gate_stderr}"
    );
    assert_ne!(
        gate_code, 0,
        "default_escape_check が failed の場合 fw gate 全体も非ゼロで \
         終了するはず: stdout={gate_stdout}"
    );
}

/// イシュー #410: `fw new --template embed`（静的単一ファイルテンプレート、
/// cargo パッケージを持たない）が生成する `structure.toml`（`[directories.root]`
/// `role = "asset"`、`crate` キーなし）は `fw gate`（`cli/src/gate.rs::
/// is_asset_only_project`）の静的専用モードの明示的オプトイン条件を満たす。
///
/// `default`/`app` の e2e（`fw_new_output_passes_fw_gate` /
/// `fw_new_app_template_output_passes_fw_gate`）は `policy`（cargo-deny 依存）
/// のみ実行環境（cargo-deny の導入有無）で結果が分岐するが、静的専用モードは
/// cargo を一切起動しないため cargo-deny の導入有無に依存せず、7 チェック
/// すべてが常に PASS・`gate_result: "PASS"`・終了コード 0 になることを
/// 無条件に断定する（計画 §4 ステップ 5 の断定方針差）。
#[test]
fn fw_new_embed_template_output_passes_fw_gate() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) = run_fw_new(&[
        "gate-pass-embed-template",
        "--template",
        "embed",
        "--dir",
        &scratch.to_string_lossy(),
    ]);
    assert_eq!(
        new_code, 0,
        "fw new --template embed が失敗した: stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("gate-pass-embed-template");
    let (gate_code, gate_stdout, gate_stderr) = run_fw_gate(&project_dir);

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "lint_wasm32",
        "test",
        "policy",
    ] {
        assert!(
            gate_stdout.contains(&format!("\"name\":\"{name}\"")),
            "fw gate のレポートにチェック `{name}` が現れない: stdout={gate_stdout}"
        );
        assert_eq!(
            check_passed(&gate_stdout, name),
            Some(true),
            "静的専用（asset-only）モードは cargo-deny の導入有無に関わらず \
             `{name}` が常に PASS するはず: stdout={gate_stdout} stderr={gate_stderr}"
        );
    }

    assert_eq!(
        gate_code, 0,
        "fw new --template embed 生成直後は環境に依存せず常に PASS するはず: \
         stdout={gate_stdout} stderr={gate_stderr}"
    );
    assert!(
        gate_stdout.contains("\"gate_result\":\"PASS\""),
        "stdout={gate_stdout}"
    );
}

/// イシュー #410: 静的専用（asset-only）モードは cargo 系 5 チェック
/// （イシュー #1174 で `lint_wasm32` を追加）を not-applicable PASS
/// 化するが、`default_escape_check`（保険層）は
/// バイパスしない。`embed` テンプレートは `src/` を生成しないため、
/// `[directories.root]` 予約名規約に従いプロジェクトルート直下へ
/// `src/injected.rs`（未レビュー `raw_html()` 呼び出し）を手動注入し、
/// `default_escape_check` が実際に走査を実行して violation を検出（failed）
/// することを断定する（`fw_new_output_default_escape_check_detects_injected_violation_in_root_src`
/// の embed 版。静的専用モードが検証の全面バイパスにならないことの回帰固定、
/// security.md A05）。
#[test]
fn fw_new_embed_template_gate_detects_injected_rust_violation() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) = run_fw_new(&[
        "gate-pass-embed-violation",
        "--template",
        "embed",
        "--dir",
        &scratch.to_string_lossy(),
    ]);
    assert_eq!(
        new_code, 0,
        "fw new --template embed が失敗した: stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("gate-pass-embed-violation");
    let injected_src_dir = project_dir.join("src");
    std::fs::create_dir_all(&injected_src_dir).expect("failed to create src/ for injection");
    std::fs::write(
        injected_src_dir.join("injected.rs"),
        "fn unreviewed_raw_html_probe() {\n    raw_html(\"x\");\n}\n",
    )
    .expect("failed to write src/injected.rs");

    let (gate_code, gate_stdout, gate_stderr) = run_fw_gate(&project_dir);

    assert_eq!(
        check_passed(&gate_stdout, "default_escape_check"),
        Some(false),
        "embed テンプレート（静的専用モード）の src/ に注入した未レビュー \
         raw_html() 呼び出しが default_escape_check で検出されなかった \
         （静的専用モードが保険層を全面バイパスしている疑い）: \
         stdout={gate_stdout} stderr={gate_stderr}"
    );
    assert!(
        gate_stdout.contains("injected.rs"),
        "default_escape_check の failed 出力は違反ファイルを file:line で \
         列挙するはず: stdout={gate_stdout}"
    );
    for name in ["type_check", "lint", "lint_wasm32", "test", "policy"] {
        assert_eq!(
            check_passed(&gate_stdout, name),
            Some(true),
            "静的専用モードの not-applicable 5 チェックは Rust コード混入時も \
             PASS のままのはず（cargo が起動されないため）: stdout={gate_stdout}"
        );
    }
    assert_ne!(
        gate_code, 0,
        "default_escape_check が failed の場合 fw gate 全体も非ゼロで \
         終了するはず: stdout={gate_stdout}"
    );
    assert!(
        gate_stdout.contains("\"gate_result\":\"BLOCKED\""),
        "stdout={gate_stdout}"
    );
}

/// イシュー #500: `fw new --example ssr-routing` 生成直後のプロジェクトに対し
/// `fw gate` を実行し、`app`/`default` テンプレートと同じ 7 チェック断定方針
/// （`policy` のみ cargo-deny 導入有無で分岐、他 6 チェックは常に PASS）を
/// 適用する。さらに受け入れ条件 1（`fw new demo --example ssr-routing && cd
/// demo && cargo run -- /items/1` が動作する）を `cargo run` 実行で直接固定
/// する。
///
/// `--example` はパッケージ名を置換しない（`new_template.rs` モジュール doc
/// コメント参照）ため、生成プロジェクトのパッケージ名は正本と同じ
/// `fandhe-frontend-example-ssr-routing` のまま。
///
/// # 前提（`.claude/rules/ci.md` 準拠）
///
/// `examples/ssr-routing` は fandhe-frontend-core/-app/-server への crates.io
/// バージョン依存で完結する（vendor 同梱なし、イシュー #499）。本テストの
/// `cargo build`/`cargo run`/`fw gate` はいずれも crates.io
/// （`https://index.crates.io`・`https://static.crates.io`）への到達性を
/// 前提とする。到達不可の場合は環境エラーとして扱い、テストの弱体化で
/// 対処しない（`fw_new_app_template_output_passes_fw_gate` と同じ前提）。
#[test]
fn fw_new_example_ssr_routing_output_passes_fw_gate() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) = run_fw_new(&[
        "gate-pass-example-ssr-routing",
        "--example",
        "ssr-routing",
        "--dir",
        &scratch.to_string_lossy(),
    ]);
    assert_eq!(
        new_code, 0,
        "fw new --example ssr-routing が失敗した: stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("gate-pass-example-ssr-routing");
    let shared_target = example_shared_target_dir();
    let (gate_code, gate_stdout, gate_stderr) =
        run_fw_gate_with_target_dir(&project_dir, &shared_target);

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "lint_wasm32",
        "test",
        "policy",
    ] {
        assert!(
            gate_stdout.contains(&format!("\"name\":\"{name}\"")),
            "fw gate のレポートにチェック `{name}` が現れない: stdout={gate_stdout}"
        );
    }

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "lint_wasm32",
        "test",
    ] {
        assert_eq!(
            check_passed(&gate_stdout, name),
            Some(true),
            "fw new --example ssr-routing 生成直後のプロジェクトで `{name}` が \
             失敗した（ssr-routing サンプルと fw gate の前提がドリフトしている）: \
             stdout={gate_stdout} stderr={gate_stderr}"
        );
    }

    if cargo_deny_available() {
        assert_eq!(
            gate_code, 0,
            "cargo-deny 導入環境では fw new --example ssr-routing 生成直後は \
             PASS するはず: stdout={gate_stdout} stderr={gate_stderr}"
        );
        assert!(
            gate_stdout.contains("\"gate_result\":\"PASS\""),
            "stdout={gate_stdout}"
        );
    } else {
        assert_eq!(
            gate_code, 1,
            "cargo-deny 未導入環境では policy の fail-closed により BLOCKED \
             (終了コード 1) のはず: stdout={gate_stdout}"
        );
        assert!(
            gate_stdout.contains("environment error: "),
            "policy の failed 出力は environment error であることを明示する \
             プレフィックスを含むはず: stdout={gate_stdout}"
        );
    }

    // 受け入れ条件 1（実装計画 §2.2 の起点）: `fw new demo --example
    // ssr-routing && cd demo && cargo run -- /items/1` が動作すること。
    // `examples/ssr-routing/tests/routing.rs::run_cli` と同一の呼び出し方
    // （引数 1 個の CLI、標準出力にステータス行 + body）を生成プロジェクト
    // 直下で `cargo run` 経由で再現する。
    let run_output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--")
        .arg("/items/1")
        .current_dir(&project_dir)
        .env("CARGO_TARGET_DIR", &shared_target)
        .output()
        .expect("failed to spawn `cargo run` in generated example project");
    assert!(
        run_output.status.success(),
        "cargo run -- /items/1 が生成直後の ssr-routing サンプルで失敗した: \
         stdout={} stderr={}",
        String::from_utf8_lossy(&run_output.stdout),
        String::from_utf8_lossy(&run_output.stderr)
    );
    let run_stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(run_stdout.starts_with("200\n"), "stdout was: {run_stdout}");
    assert!(
        run_stdout.contains("Content-Type: text/html"),
        "stdout was: {run_stdout}"
    );
}

/// イシュー #501: `fw new --example ssg-blog` 生成直後のプロジェクトに対し
/// `fw gate` を実行し、`ssr-routing` 分（`fw_new_example_ssr_routing_output_passes_fw_gate`）
/// と同じ 7 チェック断定方針（`policy` のみ cargo-deny 導入有無で分岐、他 6
/// チェックは常に PASS）を適用する。さらに受け入れ条件 1（`cargo run` で
/// `dist/` に静的サイトが生成される）を `cargo run` 実行で直接固定する。
///
/// `--example` はパッケージ名を置換しない（`new_template.rs` モジュール doc
/// コメント参照）ため、生成プロジェクトのパッケージ名は正本と同じ
/// `fandhe-frontend-example-ssg-blog` のまま。
///
/// # 前提（`.claude/rules/ci.md` 準拠）
///
/// `examples/ssg-blog` は fandhe-frontend-core/-server への crates.io
/// バージョン依存で完結する（vendor 同梱なし、イシュー #501）。本テストの
/// `cargo build`/`cargo run`/`fw gate` はいずれも crates.io
/// （`https://index.crates.io`・`https://static.crates.io`）への到達性を
/// 前提とする。到達不可の場合は環境エラーとして扱い、テストの弱体化で
/// 対処しない（`fw_new_example_ssr_routing_output_passes_fw_gate` と同じ前提）。
#[test]
fn fw_new_example_ssg_blog_output_passes_fw_gate() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) = run_fw_new(&[
        "gate-pass-example-ssg-blog",
        "--example",
        "ssg-blog",
        "--dir",
        &scratch.to_string_lossy(),
    ]);
    assert_eq!(
        new_code, 0,
        "fw new --example ssg-blog が失敗した: stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("gate-pass-example-ssg-blog");
    let shared_target = example_shared_target_dir();
    let (gate_code, gate_stdout, gate_stderr) =
        run_fw_gate_with_target_dir(&project_dir, &shared_target);

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "lint_wasm32",
        "test",
        "policy",
    ] {
        assert!(
            gate_stdout.contains(&format!("\"name\":\"{name}\"")),
            "fw gate のレポートにチェック `{name}` が現れない: stdout={gate_stdout}"
        );
    }

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "lint_wasm32",
        "test",
    ] {
        assert_eq!(
            check_passed(&gate_stdout, name),
            Some(true),
            "fw new --example ssg-blog 生成直後のプロジェクトで `{name}` が \
             失敗した（ssg-blog サンプルと fw gate の前提がドリフトしている）: \
             stdout={gate_stdout} stderr={gate_stderr}"
        );
    }

    if cargo_deny_available() {
        assert_eq!(
            gate_code, 0,
            "cargo-deny 導入環境では fw new --example ssg-blog 生成直後は \
             PASS するはず: stdout={gate_stdout} stderr={gate_stderr}"
        );
        assert!(
            gate_stdout.contains("\"gate_result\":\"PASS\""),
            "stdout={gate_stdout}"
        );
    } else {
        assert_eq!(
            gate_code, 1,
            "cargo-deny 未導入環境では policy の fail-closed により BLOCKED \
             (終了コード 1) のはず: stdout={gate_stdout}"
        );
        assert!(
            gate_stdout.contains("environment error: "),
            "policy の failed 出力は environment error であることを明示する \
             プレフィックスを含むはず: stdout={gate_stdout}"
        );
    }

    // 受け入れ条件 1: `cargo run` で `dist/` に静的サイトが生成されること。
    let run_output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .current_dir(&project_dir)
        .env("CARGO_TARGET_DIR", &shared_target)
        .output()
        .expect("failed to spawn `cargo run` in generated example project");
    assert!(
        run_output.status.success(),
        "cargo run が生成直後の ssg-blog サンプルで失敗した: stdout={} stderr={}",
        String::from_utf8_lossy(&run_output.stdout),
        String::from_utf8_lossy(&run_output.stderr)
    );

    let dist = project_dir.join("dist");
    assert!(
        dist.join("index.html").is_file(),
        "dist/index.html が生成されていない"
    );
    for slug in ["hello-ssg", "default-escaping", "view-transitions"] {
        assert!(
            dist.join("posts").join(slug).join("index.html").is_file(),
            "dist/posts/{slug}/index.html が生成されていない"
        );
    }
    // イシュー #1135: generate_assets による sitemap.xml / robots.txt の
    // 生成も、生成直後のプロジェクトで正本と同じ実演が動くことを確認する。
    assert!(
        dist.join("sitemap.xml").is_file(),
        "dist/sitemap.xml が生成されていない"
    );
    assert!(
        dist.join("robots.txt").is_file(),
        "dist/robots.txt が生成されていない"
    );
}

/// `fw new --example dist-server-docker` で生成した直後のプロジェクトが
/// `fw gate` を PASS すること（イシュー #502、`fw_new_example_ssr_routing_output_passes_fw_gate`
/// と同型のモデル）。
///
/// `cargo build`/`fw gate` はいずれも crates.io
/// （`https://index.crates.io`・`https://static.crates.io`）への到達性を
/// 前提とする。到達不可の場合は環境エラーとして扱い、テストの弱体化で
/// 対処しない（`fw_new_app_template_output_passes_fw_gate` と同じ前提）。
///
/// `dist-server-docker` は常駐型サーバー（`accept()` ループが戻らない設計、
/// `src/main.rs` 参照）のため、`fw_new_example_ssr_routing_output_passes_fw_gate`
/// 末尾のような `cargo run` 追撃は行わない。HTTP 応答検証（GET / ・
/// GET /static/style.css ・404）は生成プロジェクト内 `tests/boot.rs`
/// （実プロセス起動 + 素の TCP）が担い、下記の `test` チェック PASS 断定を
/// もって検証済みとする。
#[test]
fn fw_new_example_dist_server_docker_output_passes_fw_gate() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) = run_fw_new(&[
        "gate-pass-example-dist-server-docker",
        "--example",
        "dist-server-docker",
        "--dir",
        &scratch.to_string_lossy(),
    ]);
    assert_eq!(
        new_code, 0,
        "fw new --example dist-server-docker が失敗した: stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("gate-pass-example-dist-server-docker");
    let shared_target = example_shared_target_dir();
    let (gate_code, gate_stdout, gate_stderr) =
        run_fw_gate_with_target_dir(&project_dir, &shared_target);

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "lint_wasm32",
        "test",
        "policy",
    ] {
        assert!(
            gate_stdout.contains(&format!("\"name\":\"{name}\"")),
            "fw gate のレポートにチェック `{name}` が現れない: stdout={gate_stdout}"
        );
    }

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "lint_wasm32",
        "test",
    ] {
        assert_eq!(
            check_passed(&gate_stdout, name),
            Some(true),
            "fw new --example dist-server-docker 生成直後のプロジェクトで `{name}` が \
             失敗した（dist-server-docker サンプルと fw gate の前提がドリフトしている）: \
             stdout={gate_stdout} stderr={gate_stderr}"
        );
    }

    if cargo_deny_available() {
        assert_eq!(
            gate_code, 0,
            "cargo-deny 導入環境では fw new --example dist-server-docker 生成直後は \
             PASS するはず: stdout={gate_stdout} stderr={gate_stderr}"
        );
        assert!(
            gate_stdout.contains("\"gate_result\":\"PASS\""),
            "stdout={gate_stdout}"
        );
    } else {
        assert_eq!(
            gate_code, 1,
            "cargo-deny 未導入環境では policy の fail-closed により BLOCKED \
             (終了コード 1) のはず: stdout={gate_stdout}"
        );
        assert!(
            gate_stdout.contains("environment error: "),
            "policy の failed 出力は environment error であることを明示する \
             プレフィックスを含むはず: stdout={gate_stdout}"
        );
    }
}

/// `fw new <name> --example interactive-view-transitions` が生成直後の
/// プロジェクトに対しても `fw gate` PASS を保証し、`cargo run` が native
/// デモ出力 + `dist/index.html`（`data-hydrate-*`・`@view-transition` を含む
/// SSR HTML）を生成することを固定する（イシュー #503、
/// `fw_new_example_ssr_routing_output_passes_fw_gate` と同型の e2e）。
///
/// `--example` はパッケージ名を置換しない（`new_template.rs` モジュール doc
/// コメント参照）ため、生成プロジェクトのパッケージ名は正本と同じ
/// `fandhe-frontend-example-interactive-view-transitions` のまま。
///
/// `wasm/`（独立ワークスペースの glue クレート `interactive-vt-wasm`）と
/// `tools/wasm/build.sh` によるブラウザ実動作確認は本テストのスコープ外
/// （wasm ビルド + ブラウザ操作の smoke テスト CI 化は後続 issue、README.md
/// 参照）。`structure.toml` が `wasm/` を宣言しないため `fw gate` の
/// 検証対象クレート決定にも影響しない（`templates/app` と同じ方針）。
///
/// # 前提（`.claude/rules/ci.md` 準拠）
///
/// `examples/interactive-view-transitions` は fandhe-frontend-core/-app/
/// -interactive への crates.io バージョン依存で完結する（vendor 同梱なし、
/// イシュー #499/#503）。本テストの `cargo build`/`cargo run`/`fw gate` は
/// いずれも crates.io（`https://index.crates.io`・`https://static.crates.io`）
/// への到達性を前提とする。到達不可の場合は環境エラーとして扱い、テストの
/// 弱体化で対処しない（`fw_new_example_ssr_routing_output_passes_fw_gate` と
/// 同じ前提）。
#[test]
fn fw_new_example_interactive_view_transitions_output_passes_fw_gate() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) = run_fw_new(&[
        "gate-pass-example-interactive-view-transitions",
        "--example",
        "interactive-view-transitions",
        "--dir",
        &scratch.to_string_lossy(),
    ]);
    assert_eq!(
        new_code, 0,
        "fw new --example interactive-view-transitions が失敗した: \
         stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("gate-pass-example-interactive-view-transitions");
    let shared_target = example_shared_target_dir();
    let (gate_code, gate_stdout, gate_stderr) =
        run_fw_gate_with_target_dir(&project_dir, &shared_target);

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "lint_wasm32",
        "test",
        "policy",
    ] {
        assert!(
            gate_stdout.contains(&format!("\"name\":\"{name}\"")),
            "fw gate のレポートにチェック `{name}` が現れない: stdout={gate_stdout}"
        );
    }

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "lint_wasm32",
        "test",
    ] {
        assert_eq!(
            check_passed(&gate_stdout, name),
            Some(true),
            "fw new --example interactive-view-transitions 生成直後のプロジェクトで \
             `{name}` が失敗した（サンプルと fw gate の前提がドリフトしている）: \
             stdout={gate_stdout} stderr={gate_stderr}"
        );
    }

    if cargo_deny_available() {
        assert_eq!(
            gate_code, 0,
            "cargo-deny 導入環境では fw new --example interactive-view-transitions \
             生成直後は PASS するはず: stdout={gate_stdout} stderr={gate_stderr}"
        );
        assert!(
            gate_stdout.contains("\"gate_result\":\"PASS\""),
            "stdout={gate_stdout}"
        );
    } else {
        assert_eq!(
            gate_code, 1,
            "cargo-deny 未導入環境では policy の fail-closed により BLOCKED \
             (終了コード 1) のはず: stdout={gate_stdout}"
        );
        assert!(
            gate_stdout.contains("environment error: "),
            "policy の failed 出力は environment error であることを明示する \
             プレフィックスを含むはず: stdout={gate_stdout}"
        );
    }

    // 受け入れ条件 1: `fw new demo --example interactive-view-transitions &&
    // cd demo && cargo run` が native デモ出力（`AppState::dispatch` 実演）と
    // `dist/index.html`（`data-hydrate-*`・`@view-transition` を含む SSR
    // HTML）を生成すること。
    let run_output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .current_dir(&project_dir)
        .env("CARGO_TARGET_DIR", &shared_target)
        .output()
        .expect("failed to spawn `cargo run` in generated example project");
    assert!(
        run_output.status.success(),
        "cargo run が生成直後の interactive-view-transitions サンプルで失敗した: \
         stdout={} stderr={}",
        String::from_utf8_lossy(&run_output.stdout),
        String::from_utf8_lossy(&run_output.stderr)
    );
    let run_stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(
        run_stdout.contains("native state machine demo"),
        "stdout was: {run_stdout}"
    );
    assert!(
        run_stdout.contains("wrote dist/index.html"),
        "stdout was: {run_stdout}"
    );

    let dist_html = std::fs::read_to_string(project_dir.join("dist/index.html"))
        .expect("cargo run should have written dist/index.html");
    assert!(
        dist_html.contains("data-hydrate-"),
        "dist/index.html was: {dist_html}"
    );
    assert!(
        dist_html.contains("@view-transition"),
        "dist/index.html was: {dist_html}"
    );
}

/// `fw new --example headless-pre-styled-ui` で生成した直後のプロジェクトが
/// `fw gate` を PASS すること（イシュー #609、
/// `fw_new_example_ssg_blog_output_passes_fw_gate` と同型の e2e）。
///
/// `--example` はパッケージ名を置換しない（`new_template.rs` モジュール doc
/// コメント参照）ため、生成プロジェクトのパッケージ名は正本と同じ
/// `fandhe-frontend-example-headless-pre-styled-ui` のまま。
///
/// # 前提（`.claude/rules/ci.md` 準拠）
///
/// `examples/headless-pre-styled-ui` は当初 `fandhe-frontend-headless-ui` が
/// crates.io 未公開のため `fw new --example` 非対応だった（イシュー #552）が、
/// 前提クレート公開（イシュー #608）を受けて fandhe-frontend-core/
/// -headless-ui（推移的に -interactive）への crates.io バージョン依存へ
/// 切り替え、本テストで初めて登録した（イシュー #609）。本テストの
/// `cargo build`/`cargo run`/`fw gate` はいずれも crates.io
/// （`https://index.crates.io`・`https://static.crates.io`）への到達性を
/// 前提とする。到達不可の場合は環境エラーとして扱い、テストの弱体化で
/// 対処しない（`fw_new_example_ssr_routing_output_passes_fw_gate` と同じ前提）。
#[test]
fn fw_new_example_headless_pre_styled_ui_output_passes_fw_gate() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) = run_fw_new(&[
        "gate-pass-example-headless-pre-styled-ui",
        "--example",
        "headless-pre-styled-ui",
        "--dir",
        &scratch.to_string_lossy(),
    ]);
    assert_eq!(
        new_code, 0,
        "fw new --example headless-pre-styled-ui が失敗した: \
         stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("gate-pass-example-headless-pre-styled-ui");
    let shared_target = example_shared_target_dir();
    let (gate_code, gate_stdout, gate_stderr) =
        run_fw_gate_with_target_dir(&project_dir, &shared_target);

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "lint_wasm32",
        "test",
        "policy",
    ] {
        assert!(
            gate_stdout.contains(&format!("\"name\":\"{name}\"")),
            "fw gate のレポートにチェック `{name}` が現れない: stdout={gate_stdout}"
        );
    }

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "lint_wasm32",
        "test",
    ] {
        assert_eq!(
            check_passed(&gate_stdout, name),
            Some(true),
            "fw new --example headless-pre-styled-ui 生成直後のプロジェクトで \
             `{name}` が失敗した（サンプルと fw gate の前提がドリフトしている）: \
             stdout={gate_stdout} stderr={gate_stderr}"
        );
    }

    if cargo_deny_available() {
        assert_eq!(
            gate_code, 0,
            "cargo-deny 導入環境では fw new --example headless-pre-styled-ui \
             生成直後は PASS するはず: stdout={gate_stdout} stderr={gate_stderr}"
        );
        assert!(
            gate_stdout.contains("\"gate_result\":\"PASS\""),
            "stdout={gate_stdout}"
        );
    } else {
        assert_eq!(
            gate_code, 1,
            "cargo-deny 未導入環境では policy の fail-closed により BLOCKED \
             (終了コード 1) のはず: stdout={gate_stdout}"
        );
        assert!(
            gate_stdout.contains("environment error: "),
            "policy の failed 出力は environment error であることを明示する \
             プレフィックスを含むはず: stdout={gate_stdout}"
        );
    }

    // 受け入れ条件: `cargo run` で `dist/index.html` と `dist/assets/ui.css`
    // が生成されること（正本 README.md「動かし方」参照）。
    let run_output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .current_dir(&project_dir)
        .env("CARGO_TARGET_DIR", &shared_target)
        .output()
        .expect("failed to spawn `cargo run` in generated example project");
    assert!(
        run_output.status.success(),
        "cargo run が生成直後の headless-pre-styled-ui サンプルで失敗した: \
         stdout={} stderr={}",
        String::from_utf8_lossy(&run_output.stdout),
        String::from_utf8_lossy(&run_output.stderr)
    );

    let dist = project_dir.join("dist");
    assert!(
        dist.join("index.html").is_file(),
        "dist/index.html が生成されていない"
    );
    assert!(
        dist.join("assets").join("ui.css").is_file(),
        "dist/assets/ui.css が生成されていない"
    );
}

/// イシュー #637 の回帰テスト（受け入れ条件 1）: `support::scratch_root()` と
/// [`example_shared_target_dir`] が、実行時 env 上書きがない前提でコンパイル
/// 時に確定する `env!("CARGO_TARGET_TMPDIR")` 配下に固定され、`/tmp`
/// （`std::env::temp_dir()`）へ一切置かれないことを断定する。かつての実行時
/// `CARGO_TARGET_TMPDIR` 参照は cargo の仕様上常に失敗し `/tmp` へ落ちて
/// いた（根本原因）ため、配置の是正自体をコードで固定する。
#[test]
fn scratch_root_is_pinned_under_cargo_target_tmpdir() {
    let compiled_default = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));

    // テスト実行環境が実行時 `CARGO_TARGET_TMPDIR` を明示上書きしていない
    // 通常運用でのみ、既定値との一致を断定する（上書き運用は許容するが
    // その場合は本テストの対象外＝「実行時上書き」自体の正当性を検証する
    // テストではない）。
    if std::env::var("CARGO_TARGET_TMPDIR").is_err() {
        assert_eq!(
            support::scratch_root(),
            compiled_default,
            "scratch_root はコンパイル時 CARGO_TARGET_TMPDIR に固定されるべき"
        );
        assert_eq!(
            example_shared_target_dir().parent(),
            Some(compiled_default.as_path()),
            "example_shared_target_dir も同じルート配下に置かれるべき"
        );
    }

    let system_tmp = std::env::temp_dir();
    assert_ne!(
        support::scratch_root(),
        system_tmp,
        "scratch_root は OS 標準の一時領域（/tmp 等）と一致してはならない"
    );
}

/// イシュー #637 の回帰テスト（受け入れ条件 1、および CI #648 障害を受けた
/// 追加断定）: [`sweep_stale_scratch_dir`] が「本ファイルが命名する 2
/// プレフィックスに一致し、かつ PID が非生存**かつ** mtime が
/// [`STALE_MIN_AGE`] を超えている」エントリのみを削除し、生存 PID の
/// エントリ・**PID 非生存でも mtime が新しい（＝現に使用中の可能性が高い）
/// エントリ**・プレフィックス不一致の無関係なエントリは残置することを
/// 断定する。mtime AND 条件を欠いた旧実装は、PID 名前空間が別のコンテナ化
/// CI ジョブから見て生存 PID を「非存在」と誤判定し、他ジョブが現に
/// `fw new`/`fw gate` を実行中の scratch ディレクトリを削除してしまう
/// 実障害（`fw_new_app_template_output_passes_fw_gate` の cargo working
/// directory 消失 FAILED）を CI #648 で引き起こした。
#[test]
fn cleanup_stale_scratch_removes_dead_pid_entries_and_keeps_live_ones() {
    let test_root = support::scratch_root().join(format!(
        "fw-637-sweep-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&test_root);
    std::fs::create_dir_all(&test_root).expect("failed to create sweep test root");

    // Linux の pid_max 上限（4194304）を超え、いかなる生存プロセスにも
    // 割り当てられ得ない値を dead PID として使う。
    let dead_pid = u32::MAX;
    let live_pid = std::process::id();

    let dead_new_gate = test_root.join(format!("fw-new-gate-e2e-{dead_pid}-123456789"));
    let dead_shared_target = test_root.join(format!("fw-example-gate-shared-target-{dead_pid}"));
    let live_new_gate = test_root.join(format!("fw-new-gate-e2e-{live_pid}-987654321"));
    let dead_pid_fresh_mtime = test_root.join(format!("fw-new-gate-e2e-{dead_pid}-555555555"));
    let unrelated = test_root.join("some-unrelated-directory");

    for dir in [
        &dead_new_gate,
        &dead_shared_target,
        &live_new_gate,
        &dead_pid_fresh_mtime,
        &unrelated,
    ] {
        std::fs::create_dir_all(dir).expect("failed to create fixture entry");
    }

    // dead pid の 2 エントリは [`STALE_MIN_AGE`] の mtime AND 条件（イシュー
    // #637 の PID 名前空間跨ぎ誤判定対策）を満たすよう、作成直後の mtime を
    // 閾値超過（2 時間前）へ巻き戻す。`live_new_gate`（生存 pid）はここで
    // 触らず作成直後の新しい mtime のまま残し、「新しいディレクトリは
    // PID 判定だけでは削除されない」ことも本テストで併せて固定する。
    let stale_mtime = std::time::SystemTime::now() - (STALE_MIN_AGE * 2);
    for dir in [&dead_new_gate, &dead_shared_target] {
        let file = std::fs::File::open(dir).expect("failed to open fixture entry for mtime set");
        file.set_modified(stale_mtime)
            .expect("failed to backdate fixture entry mtime");
    }

    sweep_stale_scratch_dir(&test_root);

    assert!(
        !dead_new_gate.exists(),
        "dead pid の fw-new-gate-e2e-* エントリは回収されるべき"
    );
    assert!(
        !dead_shared_target.exists(),
        "dead pid の fw-example-gate-shared-target-* エントリは回収されるべき"
    );
    assert!(
        live_new_gate.exists(),
        "生存 pid の fw-new-gate-e2e-* エントリは残置されるべき"
    );
    assert!(
        dead_pid_fresh_mtime.exists(),
        "PID 非生存判定でも mtime が STALE_MIN_AGE 未満（新しい＝現に使用中の \
         可能性が高い）エントリは削除されてはならない（CI #648 障害の \
         再発防止、PID 名前空間跨ぎの誤判定に対する mtime AND 条件）"
    );
    assert!(
        unrelated.exists(),
        "プレフィックス不一致のエントリは sweep 対象外であるべき"
    );

    let _ = std::fs::remove_dir_all(&test_root);
}
