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
//! `FANDHE_FRONTEND_WASM_BUILD=0`（`skip`/`false` も可）で本ステージ全体を明示的に無効化
//! できる（wasm ツールチェーン未整備環境向けの逃げ道。既定は有効 = フェイル
//! クローズ。Dockerfile ビルダーステージはこのオプトアウトを使う、TASK-10.3・
//! イシュー #114 で Docker 内 WASM 再ビルドが統合されるまでの暫定措置）。
//!
//! # パッケージ単体ビルドでの WASM ステージ静的スキップ
//!
//! `cargo publish -p fandhe-frontend-dist-server`（tarball 検証を含む）・
//! crates.io から取得した利用者の `cargo build` は、本クレートを
//! `target/package/<name>-<version>/` のようなワークスペース外の一時
//! ディレクトリへ単体展開してビルドする。この経路では `workspace_root`
//! （`CARGO_MANIFEST_DIR` から機械的に 2 段上がった先）にワークスペース
//! ルート Cargo.toml も `static/` も存在せず、Cargo.lock も読めないため、
//! WASM ビルドステージ（バージョン整合検証・ネスト `cargo build`・
//! `wasm-bindgen` 実行）は原理的に成立しない。
//!
//! [`workspace_detect::is_workspace_root`]（判定根拠はその関数の doc 参照）で
//! この状態を構造的に検出し、非ワークスペース時は WASM ビルドステージ
//! 全体を静かにスキップする（`FANDHE_FRONTEND_WASM_BUILD` の明示オプトアウト
//! 時とは異なり `cargo:warning` は出さない — crates.io 利用者の通常ケースで
//! あり、環境不備ではないため）。`wasm_assets_embedded` cfg は立たず、
//! `assets.rs::lookup` は WASM 成果物なしの経路（既存の cfg 分岐）へ合流する。
//! ワークスペース内ビルド（従来経路）の挙動・fail-closed 検証は一切変えない。
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
//!
//! # 後処理ステージ（size optimization、イシュー #1971）
//!
//! `wasm-bindgen` 実行後、配布物サイズを縮める 2 段の後処理を行う
//! （採否判断・実測は `docs/ci/wasm-opt-adoption-evaluation.md` 参照）。
//!
//! 1. **`wasm-bindgen --remove-name-section --remove-producers-section`**
//!    （[`WASM_BINDGEN_ARGS`]）: 新規依存を追加せず、既存の `wasm-bindgen-cli`
//!    呼び出しへフラグを 2 つ足すだけで gzip 後サイズを大きく縮められる
//!    （イシュー #1969 実測）。トレードオフとして、本番配布物のブラウザ
//!    スタックトレースから関数名が読めなくなる（dist-server は本番配布物で
//!    あり、デバッグは `wasm-pack test` 等の別経路が担うため実害は小さいと
//!    判断している）。
//! 2. **`wasm-opt -Os`**（[`detect_wasm_opt`]・[`run_wasm_opt`]）:
//!    PATH 上に `wasm-opt`（binaryen）が見つかった場合のみ追加適用する
//!    **soft-skip** 設計。本ファイルは Cargo 依存追加・CI/Dockerfile への
//!    binaryen 導入は行わない（新規サプライチェーン依存の実導入は
//!    イシュー #1972 の担当）ため、本イシュー時点では CI にも Docker にも
//!    `wasm-opt` が存在せず、常に soft-skip 経路を通る。CI での fail-closed
//!    化（skip マーカーの不在を grep で assert する等）はイシュー #1972 が
//!    `test`/`bundle-size` ジョブ側で行う。`templates/app/tools/wasm/build.sh`
//!    の既存 soft-skip 実装（存在チェック → 一時ファイル経由の atomic 置換 →
//!    失敗時 hard fail）と同型の設計を踏襲する。
//!
//! 後処理の構成（`wasm-bindgen` の追加フラグ・`wasm-opt` の有無/バージョン）は
//! [`wasm_stage_cache::compute_wasm_stage_fingerprint`] の入力に織り込み、
//! 構成が変わればキャッシュを無効化する（後から `wasm-opt` を導入しても
//! 古いキャッシュ済み成果物が誤って再利用されないようにするため）。
use std::env;
use std::fs;
use std::io;
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

// WASM ビルドステージの有効・無効判定（`FANDHE_FRONTEND_WASM_BUILD`）。
// `wasm_stage_cache` と同型のパターンでソースレベル共有する
// （`src/wasm_build_gate.rs` 冒頭コメント参照）。
#[path = "src/wasm_build_gate.rs"]
mod wasm_build_gate;

// パッケージ単体ビルド（ワークスペース外）かどうかの構造的判定。
// `wasm_stage_cache`・`wasm_build_gate` と同型のパターンでソースレベル共有する
// （`src/workspace_detect.rs` 冒頭コメント参照）。
#[path = "src/workspace_detect.rs"]
mod workspace_detect;

fn main() {
    // `CARGO_MANIFEST_DIR` は `crates/dist-server/` を指す。埋め込み対象の
    // `static/`（ワークスペースルート直下、クレートではないため移設対象外）や
    // 兄弟クレート（`crates/wasm-full/` 等）へは 2 段上がってワークスペース
    // ルートに到達する（イシュー #436、`crates/` 配下移設）。
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/dist-server/ has a workspace root two levels up")
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

    // パッケージ単体ビルド（ワークスペース外）の構造的判定。判定根拠は
    // `workspace_detect::is_workspace_root` の doc コメント参照。
    // 「非ワークスペース」への縮退はファイル不在（NotFound）のみを根拠とする。
    // 権限エラー等の予期しない読み取り失敗まで静かに非ワークスペース扱いに
    // すると、fail-closed であるべき WASM 検証ステージが気づかれずに欠落する
    // ため、その場合はビルドを失敗させる。
    let root_cargo_toml = match fs::read_to_string(workspace_root.join("Cargo.toml")) {
        Ok(contents) => Some(contents),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
        Err(err) => panic!(
            "failed to read the workspace root Cargo.toml (not a missing file): {}. \
             Fix the file permissions or filesystem state and rebuild.",
            err.kind()
        ),
    };
    let dist_server_manifest_exists = workspace_root
        .join("crates/dist-server/Cargo.toml")
        .exists();
    let in_workspace = workspace_detect::is_workspace_root(
        root_cargo_toml.as_deref(),
        dist_server_manifest_exists,
    );

    if !in_workspace {
        // crates.io からの利用者ビルド・`cargo publish`/`cargo package` の
        // tarball 検証の通常ケース。環境不備ではないため `cargo:warning` は
        // 出さず静かにスキップする（冒頭ドキュメンテーションコメント参照）。
        // `wasm_assets_embedded` cfg は立てない。
    } else if wasm_build_enabled() {
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
            "cargo:warning=FANDHE_FRONTEND_WASM_BUILD is disabled; skipping the WASM build stage \
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
        workspace_root.join("crates/wasm-full/src").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("crates/wasm-full/Cargo.toml").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("crates/interactive/src").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("crates/core/src").display()
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
    println!("cargo:rerun-if-env-changed=FANDHE_FRONTEND_WASM_BUILD");
}

/// WASM ビルドステージが有効かどうかを環境変数 `FANDHE_FRONTEND_WASM_BUILD` から
/// 判定する薄いラッパ。判定ロジック本体は `wasm_build_gate::wasm_build_enabled_for`
/// （純関数、`src/wasm_build_gate.rs`）に分離してあり、そちらは
/// `cargo test -p fandhe-frontend-dist-server` のユニットテスト対象、かつ
/// `crates/wasm-full/tests/bundle_size.rs` が同一契約の判定を独立実装として
/// 重複させている（契約を変更する場合は両ファイルを揃えて更新すること）。
fn wasm_build_enabled() -> bool {
    wasm_build_gate::wasm_build_enabled_for(env::var("FANDHE_FRONTEND_WASM_BUILD").ok().as_deref())
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

    // `wasm-opt` の検出は fingerprint の入力になるため、キャッシュ HIT 判定
    // より前に 1 回だけ行う（冒頭ドキュメンテーションコメント「後処理
    // ステージ」参照）。
    let wasm_opt_version = detect_wasm_opt()?;

    // `wasm-opt` の導入・更新・削除を検知する再実行条件（PR #1980 レビュー
    // 指摘）。fingerprint（`wasm_stage_cache`）は `wasm-opt` のバージョンを
    // 入力に含めるが、その比較自体が「本 build.rs が再実行されること」を
    // 前提にしている。cargo は明示的な `rerun-if-changed`/
    // `rerun-if-env-changed` 以外の理由では build.rs を再実行しないため、
    // `static/`・ワークスペースソース等のトリガー（`main` 末尾）が一切
    // 変化しないまま「未導入 → 導入」「バージョン更新」「削除」が起きても
    // 検知できず、fingerprint 比較そのものに到達しない（PATH やツール実体は
    // cargo の標準トリガーの監視対象外なため）。以下の 3 種のトリガーで
    // これを補う:
    // 1. `PATH` 環境変数自体の変更（新しいディレクトリの追加・既存の
    //    再配置等）。
    println!("cargo:rerun-if-env-changed=PATH");
    // 2. 現在 `PATH` から解決できる `wasm-opt` の実体そのもの（同じ場所への
    //    バイナリ入れ替え・削除で mtime が変わるケースを拾う）。
    if let Some(wasm_opt_path) = locate_on_path("wasm-opt") {
        println!("cargo:rerun-if-changed={}", wasm_opt_path.display());
    }
    // 3. `PATH` 上の各ディレクトリ（`wasm-opt` が新規に追加された場合、
    //    ディレクトリ自体の mtime が変わるため検知できる。「未導入 → 導入」
    //    への変化は 2. では拾えず、この経路でしか検知できない）。
    //    `PATH` の要素は初回ビルド時点で未作成のディレクトリ（例:
    //    `~/.local/bin`）を含みうる。`dir.is_dir()` が false のまま
    //    監視登録をスキップすると、後日そのディレクトリ自体と `wasm-opt`
    //    が新規作成されても `PATH` 文字列も既存の監視対象ファイルも
    //    変化しないため build.rs が再実行されず、導入を検知できない
    //    （PR #1980 レビュー指摘）。Cargo は `rerun-if-changed` に存在しない
    //    パスを指定した場合、そのパスが出現するまで毎回ビルドスクリプトを
    //    再実行する仕様を持つ（`cargo::rerun-if-changed` のドキュメント
    //    「If a path pointing to a file... doesn't exist and its ancestor
    //    directory doesn't exist... the build script is always rerun.」）ため、
    //    存在しないディレクトリもそのまま監視対象に含めることで作成を
    //    確実に検知できる。ネストビルド自体は cargo 標準の増分キャッシュが
    //    効くため（直後のコメント参照）、この間 build.rs が毎回再実行されて
    //    も後段の `wasm_stage_cache_hit` 判定でコストの大部分は回避される。
    if let Some(path_var) = env::var_os("PATH") {
        for dir in env::split_paths(&path_var) {
            println!("cargo:rerun-if-changed={}", dir.display());
        }
    }

    // ネストビルド自体は cargo 標準の増分キャッシュが効くため常に実行する。
    // キャッシュ制御の対象はこの後段の `wasm-bindgen`/`wasm-opt` 実行のみ。
    let wasm_binary_path = run_wasm_build(workspace_root)?;
    let wasm_assets_dir = out_dir.join("wasm-assets");
    let fingerprint_path = out_dir.join("wasm-stage.fingerprint");

    if wasm_stage_cache::wasm_stage_cache_hit(
        &wasm_binary_path,
        &installed_version,
        wasm_opt_version.as_deref(),
        &fingerprint_path,
        &wasm_assets_dir,
    ) {
        // SKIP 経路: `wasm-bindgen`/`wasm-opt` の再実行のみを省略する。既存の
        // `OUT_DIR/wasm-assets/` をそのまま埋め込みテーブルの入力として使う
        // ため、呼び出し元（`main`）が期待する「合流エントリ・
        // `wasm_assets_embedded` cfg」の契約は SKIP でも HIT でも同一に保たれる
        // （契約を崩すとテスト側 `wasm_assets.rs` が静かにコンパイル対象外に
        // なる回帰を招くため）。
        eprintln!(
            "wasm-stage cache HIT: reusing {} (wasm-bindgen/wasm-opt not re-run)",
            wasm_assets_dir.display()
        );
    } else {
        // 成果物を上書きする前に旧 fingerprint を無効化する（PR #1980 レビュー
        // 指摘）。無効化せずに `wasm-bindgen` が両成果物を復元すると、直後の
        // `wasm-opt` が失敗・中断して `write_wasm_stage_fingerprint` に
        // 到達しなかった場合でも、旧 fingerprint が偶然「現在の入力」と
        // 一致するケース（wasm バイナリ内容・CLI バージョン・wasm-opt
        // バージョンのいずれも前回成功時から変わっていない）で次回ビルドが
        // HIT と誤判定し、未最適化のまま残った成果物を最適化済みとして
        // 再利用してしまう。削除自体が存在しない（`NotFound`）場合は無視して
        // よい（そもそも旧 fingerprint が無ければ次回比較は自然に MISS
        // になる）。しかし権限エラー等の `NotFound` 以外の削除失敗は無視
        // してはならない（PR #1980 レビュー指摘）: 削除できないファイルが
        // 読み取りもできないとは限らないため、旧 fingerprint が現在の入力と
        // 一致したまま残り、かつこの後 `wasm-bindgen` が両成果物を復元した
        // 状態で `wasm-opt` が失敗・中断して `write_wasm_stage_fingerprint`
        // に到達しなかった場合、次回ビルドで旧 fingerprint が偶然「現在の
        // 入力」と一致し HIT と誤判定して未最適化の成果物を最適化済みとして
        // 再利用してしまう。フェイルクローズを維持するため、成果物を変更
        // する前に削除エラー（`NotFound` 以外）で即座に停止する。
        match fs::remove_file(&fingerprint_path) {
            Ok(()) => {}
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(format!(
                    "failed to invalidate stale wasm-stage fingerprint {}: {err}",
                    fingerprint_path.display()
                ));
            }
        }

        eprintln!("wasm-stage cache MISS: running wasm-bindgen");
        run_wasm_bindgen(&wasm_binary_path, &wasm_assets_dir)?;

        if let Some(version) = &wasm_opt_version {
            eprintln!("wasm-stage: running wasm-opt -Os ({version})");
            run_wasm_opt(&wasm_binary_path, &wasm_assets_dir, out_dir)?;
        } else {
            println!(
                "cargo:warning=wasm-opt not found on PATH; skipping size optimization \
                 (output correctness unaffected; see docs/ci/wasm-opt-adoption-evaluation.md)"
            );
        }

        // fingerprint の書き込みは成果物確認後の最後の一手にする
        // （冒頭ドキュメンテーションコメントのフェイルクローズ方針参照）。
        wasm_stage_cache::write_wasm_stage_fingerprint(
            &wasm_binary_path,
            &installed_version,
            wasm_opt_version.as_deref(),
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

/// `wasm-bindgen` に渡す固定引数。`--target web --no-typescript` は従来からの
/// 構成、`--remove-name-section --remove-producers-section` はイシュー #1971
/// で追加した size optimization（冒頭ドキュメンテーションコメント「後処理
/// ステージ」参照。イシュー #1969 実測で新規依存ゼロのまま gzip 後サイズを
/// 大きく縮められることを確認済み）。
///
/// `crates/wasm-full/tests/bundle_size.rs::WASM_BINDGEN_ARGS` が同一配列を
/// 独立実装として複製しており（`bundle_size.rs` は `dist-server` に依存
/// させない設計のため）、この配列を変更する場合は両ファイルを揃えて
/// 更新すること。
const WASM_BINDGEN_ARGS: &[&str] = &[
    "--target",
    "web",
    "--no-typescript",
    "--remove-name-section",
    "--remove-producers-section",
];

/// `wasm-bindgen` を実行し、生成された JS グルーコード・`_bg.wasm` を
/// `wasm_assets_dir`（`OUT_DIR/wasm-assets/`、呼び出し元 [`run_wasm_stage`] が
/// 用意する）へ出力する。フラグ構成は [`WASM_BINDGEN_ARGS`] 参照。
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
        .args(WASM_BINDGEN_ARGS)
        .arg("--out-dir")
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

/// `PATH` 上から `exe_name`（拡張子なし）という名前の実行可能ファイルを
/// 探索し、見つかった最初の絶対パスを返す。サブプロセスは起動しない
/// 純粋な探索であり、`detect_wasm_opt` とは別目的（cargo の再実行トリガー
/// 登録用に実体の場所を特定するだけ。存在確認のみで実行可能性までは
/// 検証しない）。見つからなければ `None`。
fn locate_on_path(exe_name: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(exe_name);
        if candidate.is_file() {
            return Some(candidate);
        }
        if cfg!(windows) {
            let candidate_exe = dir.join(format!("{exe_name}.exe"));
            if candidate_exe.is_file() {
                return Some(candidate_exe);
            }
        }
    }
    None
}

/// PATH 上の `wasm-opt`（binaryen）を検出する。
///
/// 3 分岐（冒頭ドキュメンテーションコメント「後処理ステージ」参照）:
/// - 見つからない（`Command::spawn` が `NotFound`）→ `Ok(None)`（soft-skip）
/// - 見つかったが起動・実行に失敗（`NotFound` 以外の spawn エラー、非 0
///   終了、空出力）→ `Err`（hard fail。壊れた環境を黙って素通りしない）
/// - 見つかって正常終了 → `Ok(Some(バージョン文字列))`（`wasm-opt --version`
///   の出力そのもの。fingerprint の入力として使う）
fn detect_wasm_opt() -> Result<Option<String>, String> {
    let output = match Command::new("wasm-opt").arg("--version").output() {
        Ok(output) => output,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(format!(
                "failed to spawn `wasm-opt --version` (found on PATH but could not execute it): {err}"
            ));
        }
    };

    if !output.status.success() {
        return Err("`wasm-opt --version` exited with a non-zero status".to_string());
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if version.is_empty() {
        return Err("`wasm-opt --version` produced no output".to_string());
    }

    Ok(Some(version))
}

/// `wasm-opt -Os --strip-producers` を `wasm_binary_path` の file stem から
/// 求めた `<stem>_bg.wasm`（`wasm_assets_dir` 配下、`wasm-bindgen` が既に
/// 生成済み）に適用し、意味論を変えずにサイズのみ縮める。
///
/// `--strip-producers` は `wasm-opt` 自身が producers custom section を
/// 書き戻すのを防ぐ（`run_wasm_bindgen` の `--remove-producers-section` で
/// 除去済みの section が `wasm-opt` 実行によって復活すると、
/// `crates/dist-server/tests/wasm_assets.rs` の
/// `served_wasm_binary_has_no_name_or_producers_custom_sections` が
/// binaryen 導入環境で偽陽性 PASS のまま実体が崩れる/導入直後に FAIL する
/// 事態になる。イシュー #1971 PR #1980 レビュー指摘）。
///
/// 一時ファイルは `out_dir` の**兄弟ディレクトリ**（`OUT_DIR/wasm-opt-tmp/`）
/// に置く。`OUT_DIR/wasm-assets/` の中に置くと `collect_files` が
/// ディレクトリ全体を埋め込みテーブルへ合流させるため、残置した一時ファイル
/// まで配信対象になってしまう（`OUT_DIR` 配下の兄弟ディレクトリなので
/// `fs::rename` は同一ファイルシステム内の atomic 置換のまま成立する）。
///
/// 出力が非空かつ `\0asm` マジックナンバーで始まることを確認してから
/// `fs::rename` で置換する。`wasm-opt` が失敗した場合は一時ファイルを削除し
/// `Err` を返す（`_bg.wasm` は無傷のまま残る、フェイルクローズ）。
fn run_wasm_opt(
    wasm_binary_path: &Path,
    wasm_assets_dir: &Path,
    out_dir: &Path,
) -> Result<(), String> {
    let stem = wasm_binary_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            format!(
                "could not determine file stem of {}",
                wasm_binary_path.display()
            )
        })?;
    let bg_wasm = wasm_assets_dir.join(format!("{stem}_bg.wasm"));

    let tmp_dir = out_dir.join("wasm-opt-tmp");
    fs::create_dir_all(&tmp_dir)
        .map_err(|e| format!("failed to create {}: {e}", tmp_dir.display()))?;
    let tmp_path = tmp_dir.join(format!("{stem}_bg.wasm"));

    let status = Command::new("wasm-opt")
        .arg("-Os")
        .arg("--strip-producers")
        .arg(&bg_wasm)
        .arg("-o")
        .arg(&tmp_path)
        .status()
        .map_err(|e| format!("failed to spawn wasm-opt: {e}"))?;
    if !status.success() {
        let _ = fs::remove_file(&tmp_path);
        return Err(format!(
            "wasm-opt failed while optimizing {}",
            bg_wasm.display()
        ));
    }

    let optimized = fs::read(&tmp_path)
        .map_err(|e| format!("failed to read wasm-opt output {}: {e}", tmp_path.display()))?;
    if !wasm_stage_cache::looks_like_wasm(&optimized) {
        let _ = fs::remove_file(&tmp_path);
        return Err(format!(
            "wasm-opt produced an unexpected (empty or non-wasm) output at {}",
            tmp_path.display()
        ));
    }

    fs::rename(&tmp_path, &bg_wasm).map_err(|e| {
        format!(
            "failed to replace {} with wasm-opt output: {e}",
            bg_wasm.display()
        )
    })?;

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
