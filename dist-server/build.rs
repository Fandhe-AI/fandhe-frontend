//! `fandhe-frontend-dist-server` のビルドスクリプト。
//!
//! ワークスペース直下 `static/`（`wasm-thin`/`wasm-full` の埋め込み HTML 等が
//! 置かれる想定のディレクトリ、`.claude/rules/delegation-impl.md` 参照）を
//! 走査し、`(URL パス, ファイル内容)` の静的テーブルを `OUT_DIR` に生成する。
//! `src/assets.rs` がこの生成物を `include!` して配信に使う。
//!
//! # 外部依存ゼロの理由（REQ-3）
//!
//! `rust-embed` は依存グラフの深さを構造的に 8 まで押し上げ REQ-3（深さ 6 以内）
//! に違反する（`dist-server/Cargo.toml` の実測コメント参照）。本ファイルは
//! std のみで完結する自前実装とし、`build-dependencies` を一切追加しない
//! （WASM ビルドステージのサブプロセス起動も `std::process::Command` のみ）。
//!
//! # 埋め込みは常時有効（TASK-9.1b のスコープ）
//!
//! 開発時にファイルシステムから直接読み込む・埋め込みを強制する force-embed
//! 切り替えは TASK-10.1（イシュー #105/#106/#107）のスコープであり、本ファイル
//! （`static/` 走査部分）は debug/release を問わず常にコンパイル時埋め込み
//! テーブルを生成する。実行時にどちらを配信へ使うかは `assets.rs::lookup` の
//! `cfg` 分岐が担う。
//!
//! # WASM ビルドステージ（TASK-10.2b、イシュー #110、REQ-10 条件 3）
//!
//! `cargo build -p fandhe-frontend-dist-server` 単一コマンドでネイティブサーバーバイナリと
//! WASM クライアント成果物（`fandhe-frontend-wasm-full`）の双方が生成されるようにする。
//! 処理の流れは [`wasm_build_enabled`] → [`run_wasm_stage`]
//! （[`expected_wasm_bindgen_version`] によるバージョン整合検証 →
//! [`run_wasm_build`]（ネスト `cargo build --target wasm32-unknown-unknown`）→
//! [`run_wasm_bindgen`]（`wasm-bindgen --target web` 実行））。生成物は
//! `/static/wasm/<ファイル名>` の URL パスで [`EMBEDDED_ASSETS`] へ合流する。
//!
//! 生成物は **`OUT_DIR` 完結**とし、ソースツリー（`static/`）へは書き込まない
//! （`static/` は本ファイルが `rerun-if-changed` で監視しているため、書き込むと
//! 際限ない再ビルドループになる）。
//!
//! `RWS_WASM_BUILD=0`（`skip`/`false` も可）で本ステージ全体を明示的に無効化
//! できる（wasm ツールチェーン未整備環境向けの逃げ道。既定は有効 = フェイル
//! クローズ。Dockerfile ビルダーステージはこのオプトアウトを使う、TASK-10.3・
//! イシュー #114 で Docker 内 WASM 再ビルドが統合されるまでの暫定措置）。
//!
//! # キャッシュ・再ビルド制御（TASK-10.2c、イシュー #111）
//!
//! ネスト `cargo build --target wasm32-unknown-unknown`（[`run_wasm_build`]）は
//! cargo 自身の増分ビルドキャッシュ（`target/wasm-dist/`）が効くため常に実行
//! する。本ステージが独自にキャッシュ制御を行うのは、その後段の
//! `wasm-bindgen` 実行（[`run_wasm_bindgen`]）のみで、これはネストビルドが
//! 生成した `.wasm` の**内容ハッシュ**と `wasm-bindgen-cli` の実バージョンを
//! 束ねた fingerprint（`wasm_stage_cache::compute_wasm_stage_fingerprint`）を
//! `OUT_DIR/wasm-stage.fingerprint` に保存し、次回以降の実行で前回値と完全一致
//! する場合に限り再利用する（`wasm_stage_cache::wasm_stage_cache_hit`）。
//! これらキャッシュ判定の純粋関数は `src/wasm_stage_cache.rs` にソースレベルで
//! 切り出しており、`cargo test -p fandhe-frontend-dist-server` でユニットテストされる
//! （下記 `mod wasm_stage_cache` 宣言・当該ファイル冒頭コメント参照）。
//!
//! フェイルクローズ方針: fingerprint の読み取り失敗・欠落・不一致・成果物
//! （`OUT_DIR/wasm-assets/` 配下の `.js`/`_bg.wasm`）の欠落のいずれでも
//! 「再実行」側へ倒す。fingerprint ファイルの書き込みは `wasm-bindgen` が
//! 成功し成果物の存在を確認した**後**にのみ行う（[`run_wasm_bindgen`] の
//! 末尾）。失敗・中断時に不完全な成果物と一致する fingerprint が残る事故を
//! 避けるため。
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// キャッシュ判定の純粋関数群（fingerprint 計算・成果物完全性チェック等）は
// `src/wasm_stage_cache.rs` にソースレベルで切り出している。パッケージ自身の
// lib クレートを `build-dependencies` にはできない（循環依存）ため `#[path]`
// でこのファイルをそのまま取り込み、`cargo test -p fandhe-frontend-dist-server`
// （`lib.rs` 経由の通常モジュールとして）でユニットテストできるようにする
// （`bench_support.rs` と同型のパターン。詳細は当該ファイル冒頭コメント参照）。
#[path = "src/wasm_stage_cache.rs"]
mod wasm_stage_cache;

fn main() {
    // `CARGO_MANIFEST_DIR` は `dist-server/` を指す。埋め込み対象の `static/` や
    // 兄弟クレート（`wasm-full/` 等）はワークスペースルート直下にあるため
    // 一段上がる。
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("dist-server/ has a parent directory (workspace root)")
        .to_path_buf();
    let static_dir = workspace_root.join("static");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let dest_path = out_dir.join("embedded_assets.rs");

    let mut static_entries = Vec::new();
    collect_files(&static_dir, &static_dir, &mut static_entries);

    // `static/` 配下の追加・変更・削除でテーブルを再生成する（既存動作）。
    // WASM 成果物（OUT_DIR 側に生成される）はここに含めない —
    // OUT_DIR は本ファイル自身が毎回書き込む先であり、rerun-if-changed の
    // 対象にすると際限ない再ビルドループになる。
    println!("cargo:rerun-if-changed={}", static_dir.display());
    for (_, abs_path) in &static_entries {
        println!("cargo:rerun-if-changed={}", abs_path.display());
    }

    let mut entries = static_entries;

    // `wasm_assets_embedded` cfg は「このビルドで WASM 成果物が実際に埋め込み
    // テーブルへ合流したか」を `assets.rs`（テスト）へ伝える。`-D warnings` の
    // もとで `unexpected_cfgs` lint に引っかからないよう、使用しない場合も
    // 宣言だけは常に行う。
    println!("cargo::rustc-check-cfg=cfg(wasm_assets_embedded)");

    if wasm_build_enabled() {
        match run_wasm_stage(&workspace_root, &out_dir) {
            Ok(wasm_entries) => {
                entries.extend(wasm_entries);
                println!("cargo::rustc-cfg=wasm_assets_embedded");
            }
            // フェイルクローズ: オプトアウトしない限り WASM ステージの失敗は
            // ビルド全体を失敗させる（REQ-10 条件 3 を満たさないバイナリを
            // 静かに作らないため）。エラーメッセージは内部パス等の機微情報を
            // 含めず、対処方法（コマンド）のみを提示する（security.md
            // 「機微情報の露出」）。
            Err(message) => panic!("{message}"),
        }
    } else {
        println!(
            "cargo:warning=RWS_WASM_BUILD is disabled; skipping the WASM build stage \
             (/static/wasm/* will not be embedded or served)"
        );
    }

    // 生成物の並び順を実行間で安定させる（テーブルの diff が無意味に揺れない
    // ようにするため。lookup 自体は線形走査で順不同でも正しく動く）。
    entries.sort();

    let mut generated = String::new();
    generated.push_str(
        "/// build.rs が `static/`（および有効時は WASM ビルドステージ）から\n\
         /// 生成した埋め込みテーブル。`(URL パス, ファイル内容)` の組。\n\
         /// `assets.rs::lookup` からのみ参照される。\n\
         pub static EMBEDDED_ASSETS: &[(&str, &[u8])] = &[\n",
    );
    for (url_path, abs_path) in &entries {
        // `include_bytes!` はこの生成ファイル（OUT_DIR 側）からの相対パス解決に
        // なるため、埋め込み元ファイルは絶対パスで記述する（相対パスのまま
        // 埋め込むと `dist-server/` 基準と `OUT_DIR` 基準がずれてビルドが壊れる）。
        generated.push_str(&format!(
            "    ({url_path:?}, include_bytes!({abs_path:?}) as &[u8]),\n",
            url_path = url_path,
            abs_path = abs_path.display().to_string(),
        ));
    }
    generated.push_str("];\n");

    fs::write(&dest_path, generated).expect("write OUT_DIR/embedded_assets.rs");

    // WASM ビルドステージの再実行トリガー。これらのパスが変化すると
    // `cargo` は本 build.rs を再実行するが、そこから先の `wasm-bindgen`
    // 再実行要否は fingerprint 比較（`wasm_stage_cache_hit`、TASK-10.2c）が
    // 判断する。生成物自体（OUT_DIR/wasm-assets/*）は上記の理由で監視対象に
    // しない。
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("wasm-full/src").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("wasm-full/Cargo.toml").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("interactive/src").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("core/src").display()
    );
    // `Cargo.lock` の変更（wasm-bindgen 等の依存バージョン更新）も再実行
    // トリガーに含める。これを漏らすと、ロックファイルのみ更新された場合に
    // 本 build.rs が再実行されず、`expected_wasm_bindgen_version` が古い
    // バージョン整合性チェックのまま `/static/wasm/*` に stale な WASM 成果物
    // を埋め込み続けてしまう（Cursor Bugbot 指摘、PR #217 review 4719879204）。
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("Cargo.lock").display()
    );
    println!("cargo:rerun-if-env-changed=RWS_WASM_BUILD");
}

/// WASM ビルドステージが有効かどうかを環境変数 `RWS_WASM_BUILD` から判定する。
///
/// 既定（未設定）は有効。`0`・`skip`・`false`（大文字小文字を区別しない）の
/// いずれかを設定した場合のみ無効化する。wasm ツールチェーン未整備環境
/// （Docker ビルダーステージ・一部 CI ジョブ）向けの明示オプトアウト
/// （設計 4.4 節。既定は統合ビルド有効という「安全側」を保つため、無効化は
/// 明示的な合言葉を要求する）。
fn wasm_build_enabled() -> bool {
    match env::var("RWS_WASM_BUILD") {
        Ok(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            !(normalized == "0" || normalized == "skip" || normalized == "false")
        }
        Err(_) => true,
    }
}

/// WASM ビルドステージ本体。バージョン整合検証 → ネスト `cargo build` →
/// `wasm-bindgen` の順に実行し、`/static/wasm/<ファイル名>` を URL パスとする
/// `(URL パス, 絶対パス)` の組を返す。呼び出し元（`main`）が
/// [`EMBEDDED_ASSETS`] へ合流させる。
///
/// 失敗時は `Err(メッセージ)` を返す（`main` 側で `panic!` してビルドを
/// 失敗させるかどうかを判断する。本関数自体はパニックしない設計とし、
/// 将来オプトアウト以外の回復経路が必要になった場合の拡張点を残す）。
fn run_wasm_stage(workspace_root: &Path, out_dir: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    // wasm-bindgen が生成する JS グルーコードは、生成に使った `wasm-bindgen`
    // クレートのバージョンと `wasm-bindgen-cli` のバージョンが一致しない場合
    // 実行時に壊れる（wasm-bindgen 自体の既知の制約）。Cargo.lock が解決した
    // バージョンと CLI の実バージョンを事前に突き合わせ、不一致は明確な
    // エラーで止める（REQ-3・security.md「脆弱な依存」観点の一環）。
    let expected_version = expected_wasm_bindgen_version(workspace_root)?;
    let installed_version = installed_wasm_bindgen_cli_version()?;
    if expected_version != installed_version {
        return Err(format!(
            "wasm-bindgen-cli version mismatch: Cargo.lock resolves wasm-bindgen {expected_version}, \
             but `wasm-bindgen --version` reports {installed_version}. \
             Install the matching CLI with: cargo install wasm-bindgen-cli --version {expected_version} --locked"
        ));
    }

    // ネストビルド自体は cargo 標準の増分キャッシュが効くため常に実行する。
    // キャッシュ制御の対象はこの後段の `wasm-bindgen` 実行のみ。
    let wasm_binary_path = run_wasm_build(workspace_root)?;
    let wasm_assets_dir = out_dir.join("wasm-assets");
    let fingerprint_path = out_dir.join("wasm-stage.fingerprint");

    if wasm_stage_cache::wasm_stage_cache_hit(
        &wasm_binary_path,
        &installed_version,
        &fingerprint_path,
        &wasm_assets_dir,
    ) {
        // SKIP 経路: `wasm-bindgen` の再実行のみを省略する。既存の
        // `OUT_DIR/wasm-assets/` をそのまま埋め込みテーブルの入力として使う
        // ため、呼び出し元（`main`）が期待する「合流エントリ・
        // `wasm_assets_embedded` cfg」の契約は SKIP でも HIT でも同一に保たれる
        // （契約を崩すとテスト側 `wasm_assets.rs` が静かにコンパイル対象外に
        // なる回帰を招くため）。
        eprintln!(
            "wasm-stage cache HIT: reusing {} (wasm-bindgen not re-run)",
            wasm_assets_dir.display()
        );
    } else {
        eprintln!("wasm-stage cache MISS: running wasm-bindgen");
        run_wasm_bindgen(&wasm_binary_path, &wasm_assets_dir)?;
        // fingerprint の書き込みは成果物確認後の最後の一手にする
        // （冒頭ドキュメンテーションコメントのフェイルクローズ方針参照）。
        wasm_stage_cache::write_wasm_stage_fingerprint(
            &wasm_binary_path,
            &installed_version,
            &fingerprint_path,
        )?;
    }

    let mut raw_entries = Vec::new();
    collect_files(&wasm_assets_dir, &wasm_assets_dir, &mut raw_entries);
    // `collect_files` は `"/static/" + root からの相対パス` を URL パスとして
    // 付与するため、配信先を `/static/wasm/` 配下へ付け替える。
    let entries = raw_entries
        .into_iter()
        .map(|(url_path, abs_path)| {
            let relative = url_path
                .strip_prefix("/static/")
                .expect("collect_files always prefixes with /static/")
                .to_string();
            (format!("/static/wasm/{relative}"), abs_path)
        })
        .collect();
    Ok(entries)
}

/// ワークスペースの `Cargo.lock` を std の文字列処理でパースし、解決済みの
/// `wasm-bindgen` クレートのバージョンを取得する。
///
/// TOML パーサクレートを追加しない（REQ-3・`core は外部依存ゼロ` の精神を
/// `dist-server` の build-dependencies にも適用）。`Cargo.lock` の
/// `[[package]]` ブロック形式は cargo が生成する固定フォーマットであり、
/// `"[[package]]"` で分割して各ブロックの `name`/`version` 行を読むだけで
/// 十分に頑健（ブロック内のキー順序への依存を避けるため、名前・バージョンの
/// 両方が揃うまで走査する）。
fn expected_wasm_bindgen_version(workspace_root: &Path) -> Result<String, String> {
    let lock_path = workspace_root.join("Cargo.lock");
    let content = fs::read_to_string(&lock_path).map_err(|_| {
        "failed to read Cargo.lock to determine the required wasm-bindgen-cli version".to_string()
    })?;

    for block in content.split("[[package]]") {
        let mut name = None;
        let mut version = None;
        for line in block.lines() {
            let line = line.trim();
            if let Some(value) = line.strip_prefix("name = ") {
                name = Some(value.trim_matches('"'));
            } else if let Some(value) = line.strip_prefix("version = ") {
                version = Some(value.trim_matches('"'));
            }
            if name.is_some() && version.is_some() {
                break;
            }
        }
        if name == Some("wasm-bindgen") {
            return version.map(str::to_string).ok_or_else(|| {
                "found a wasm-bindgen entry in Cargo.lock but it has no version field".to_string()
            });
        }
    }

    Err(
        "wasm-bindgen package not found in Cargo.lock (is wasm-full's dependency on it intact?)"
            .to_string(),
    )
}

/// `wasm-bindgen --version` を実行し、インストール済み `wasm-bindgen-cli` の
/// バージョン文字列を返す。未インストール・実行失敗時は、内部パス等を含まない
/// 対処方法つきのエラーメッセージを返す。
fn installed_wasm_bindgen_cli_version() -> Result<String, String> {
    let output = Command::new("wasm-bindgen")
        .arg("--version")
        .output()
        .map_err(|_| {
            "wasm-bindgen-cli not found on PATH. Install it with: \
         cargo install wasm-bindgen-cli --version <version-matching-Cargo.lock> --locked"
                .to_string()
        })?;

    if !output.status.success() {
        return Err("`wasm-bindgen --version` exited with a non-zero status".to_string());
    }

    // 出力形式は `wasm-bindgen <version>`。末尾トークンをバージョンとして扱う。
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .rsplit(' ')
        .next()
        .map(str::to_string)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "unexpected `wasm-bindgen --version` output format".to_string())
}

/// ネストした `cargo build -p fandhe-frontend-wasm-full --target wasm32-unknown-unknown
/// --release` を実行し、生成された `.wasm` バイナリの絶対パスを返す。
///
/// # 環境の分離（決定性確保）
///
/// - `--target-dir` を親ビルドと別ディレクトリ（`target/wasm-dist`）にする。
///   親 cargo と同一 target dir を共有すると、親プロセスが保持する
///   ディレクトリロックとネストプロセスがデッドロックする
/// - `Command::env_clear()` で外部環境を一旦すべて遮断し、ビルドに最低限
///   必要な変数（`PATH`/`HOME`/`CARGO_HOME`/`RUSTUP_HOME`/
///   `RUSTUP_TOOLCHAIN`）のみを明示的に許可リストで引き継ぐ。CI の
///   `RUSTFLAGS='-F unsafe_code' cargo check`（forbid-unsafe ジョブ参照）等が
///   ネストビルドへ伝播すると、`wasm-bindgen`/`web-sys` の FFI 境界が
///   正当に使う `unsafe` と衝突して無関係な失敗を起こすため、
///   `RUSTFLAGS`/`CARGO_ENCODED_RUSTFLAGS`/`CARGO_TARGET_DIR` 等の外部設定は
///   一切引き継がない
/// - プロファイルは常に release 固定（配布用埋め込み成果物。REQ-11 の
///   バンドルサイズ基準に沿う。debug wasm 対応は TASK-10.2a/c（#109/#111）の
///   検討の結果、release 固定を維持する設計判断とした。開発時の高速確認は
///   `--target nodejs` のオプトイン経路（`docs/design/wasm-build-integration.md`
///   §6.4）が別途担う）
fn run_wasm_build(workspace_root: &Path) -> Result<PathBuf, String> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let wasm_target_dir = workspace_root.join("target").join("wasm-dist");

    let mut command = Command::new(&cargo);
    command.env_clear();
    for key in [
        "PATH",
        "HOME",
        "CARGO_HOME",
        "RUSTUP_HOME",
        "RUSTUP_TOOLCHAIN",
    ] {
        if let Ok(value) = env::var(key) {
            command.env(key, value);
        }
    }
    command
        .current_dir(workspace_root)
        .args([
            "build",
            "-p",
            "fandhe-frontend-wasm-full",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
            "--target-dir",
        ])
        .arg(&wasm_target_dir);

    let status = command.status().map_err(|e| {
        format!("failed to spawn nested `cargo build -p fandhe-frontend-wasm-full`: {e}")
    })?;
    if !status.success() {
        return Err(
            "nested `cargo build -p fandhe-frontend-wasm-full --target wasm32-unknown-unknown --release` failed. \
             Ensure the wasm32-unknown-unknown target is installed: rustup target add wasm32-unknown-unknown"
                .to_string(),
        );
    }

    Ok(wasm_target_dir
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("fandhe_frontend_wasm_full.wasm"))
}

/// `wasm-bindgen --target web --no-typescript` を実行し、生成された JS グルー
/// コード・`_bg.wasm` を `wasm_assets_dir`（`OUT_DIR/wasm-assets/`、呼び出し元
/// [`run_wasm_stage`] が用意する）へ出力する。
///
/// `--no-typescript` は `.d.ts` が実行時配信に不要という PoC-4 の発見事項に
/// 対応する（型定義ファイルはこのサーバーが配信する対象に含めない）。
/// `--target web` は本番配布用の ES module 形式（`<script type="module">` から
/// 直接 import 可能）を選択する。`nodejs`/`bundler` 等の使い分けは
/// イシュー #161 のスコープ。
///
/// キャッシュ HIT 時（TASK-10.2c）は呼び出されない。呼び出し元が
/// `wasm_stage_cache::wasm_stage_cache_hit` で再利用可否を判定する。
fn run_wasm_bindgen(wasm_binary_path: &Path, wasm_assets_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(wasm_assets_dir)
        .map_err(|e| format!("failed to create {}: {e}", wasm_assets_dir.display()))?;

    let status = Command::new("wasm-bindgen")
        .args(["--target", "web", "--no-typescript", "--out-dir"])
        .arg(wasm_assets_dir)
        .arg(wasm_binary_path)
        .status()
        .map_err(|e| format!("failed to spawn wasm-bindgen: {e}"))?;
    if !status.success() {
        return Err(
            "wasm-bindgen failed to generate JS bindings for fandhe-frontend-wasm-full".to_string(),
        );
    }

    if !wasm_stage_cache::wasm_assets_look_complete(wasm_assets_dir, wasm_binary_path) {
        // wasm-bindgen が成功ステータスを返したにも関わらず期待した成果物
        // （`<stem>.js`/`<stem>_bg.wasm`）が揃っていない異常系。fingerprint を
        // 書かず、次回ビルドで確実に再実行させる（フェイルクローズ）。
        return Err(format!(
            "wasm-bindgen reported success but expected artifacts are missing under {}",
            wasm_assets_dir.display()
        ));
    }

    Ok(())
}

// `wasm_stage_cache_hit` / `compute_wasm_stage_fingerprint` /
// `write_wasm_stage_fingerprint` / `wasm_assets_look_complete` /
// `fnv1a_hash` はいずれも `wasm_stage_cache` モジュール（`src/wasm_stage_cache.rs`
// を `#[path]` でソースレベル共有、本ファイル冒頭の `mod` 宣言参照）へ移した。
// ユニットテストは `cargo test -p fandhe-frontend-dist-server`（`src/wasm_stage_cache.rs`
// の `#[cfg(test)]`）が担う。

/// `dir` 以下を再帰的に走査し、`(URL パス, 絶対パス)` を `out` へ積む。
///
/// `root` は URL パス片（`"/static/" + root からの相対パス`）の算出基準。
/// シンボリックリンクは `fs::metadata`（リンク先を辿る）で判定するため、
/// ワークスペース外を指すリンクを埋め込む事故を避けるには `static/` 配下に
/// 外部リンクを置かない運用を前提とする（現時点で `static/` は単一ファイル
/// のみで実害なし。将来の運用注意点としてここに残す）。
///
/// WASM ビルドステージ（[`run_wasm_stage`]）でも `OUT_DIR/wasm-assets/` を
/// 走査する用途に再利用する（`root` を差し替えるだけで URL パス算出ロジックを
/// 共有できるため）。
fn collect_files(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, out);
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .expect("walked path is under root")
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        let url_path = format!("/static/{relative}");
        out.push((url_path, path));
    }
}
