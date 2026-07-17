//! REQ-12（NPM 互換、`docs/spec/04-requirements.md`）の受け入れ基準 3
//! 「クライアント実行時（配布バイナリ・ブラウザ）に NPM パッケージのコード・
//! Node ランタイムが一切含まれないこと」を固定する回帰テスト（TASK-12.3、
//! イシュー #125。親イシュー #37）。
//!
//! 基準 1（`--ignore-scripts` 既定化、TASK-12.1・#38・イシュー #37 系列）・
//! 基準 2（実行可能コード非混入の機械検証、TASK-12.2・#123）は
//! `tools/npm-asset-build/` 側で担保済み（`docs/npm-asset-build.md` 参照）。
//! 本ファイルはその先、「配布物として最終的に出て行くもの」に対する
//! 最終ゲートを担う。
//!
//! # なぜ Docker イメージを直接 `docker build` しないか
//!
//! `Dockerfile`（ワークスペースルート）は最終ステージを `FROM scratch` とし、
//! ビルドステージで生成した `dist-server` バイナリ 1 つのみを `COPY` する
//! 構成である。したがって「Docker イメージの内容」は
//! 「配布バイナリの内容」と「その COPY 元・ステージ構成が本当にバイナリ
//! 1 つだけを持ち込む構造か」の 2 点に還元できる。本ファイルは前者を
//! [`embedded_assets_do_not_reference_npm_or_node`]・
//! [`distributed_binary_does_not_embed_npm_or_node_runtime`] で、後者を
//! [`dockerfile_final_stage_copies_only_the_binary_and_has_no_run_instruction`]
//! で検証し、実際に `docker build`（daemon 必須・CI 専用ジョブでのみ実行
//! 可能）を伴わずに self-hosted 環境・通常の `cargo test` からも実行できる
//! 形で「Docker イメージ内容の検査」と等価な保証を得る。実イメージのレイヤ
//! 走査までを検査する必要が生じた場合は、既存の
//! `.github/workflows/image-size.yml`（`docker build` を継続実測）を補完する
//! 形で別途 Issue 化を検討する（`.claude/rules/out-of-scope-tracking.md`）。
//!
//! # マーカー方式とフェイルクローズの証明
//!
//! 「NPM パッケージ・Node ランタイムが含まれない」ことの直接証明は困難な
//! ため、本ファイルは既知のシグネチャ（[`NPM_OR_NODE_MARKERS`]）の**不在**を
//! 検査する消極的検証（allowlist ではなく blocklist）である。blocklist 方式は
//! 検出器自体が実効性を失うと静かに骨抜きになるリスクがあるため、
//! [`marker_detector_flags_synthesized_npm_payload`]
//! （合成した NPM 風ペイロードを実際に検出できることを証明する負例テスト）
//! を同一ファイルに置き、PoC-7 以来の「検出器の実効性を負例で固定する」
//! 慣行に従う。
//!
//! マーカー・本ファイルの検査ロジックの削除・弱体化は、既存の XSS 回帰
//! テストと同様に「弱体化禁止」（`.claude/rules/coding-rust.md`）の対象。
//! マーカーを緩和する場合は、実バイナリ・実アセットでの誤検知が実測された
//! ときのみとし、その判断根拠をコメントに残すこと。

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rws_dist_server::assets::EMBEDDED_ASSETS;

/// NPM パッケージ・Node ランタイムに由来すると判断できる既知シグネチャ。
///
/// 各マーカーは「配布物中に存在すれば NPM/Node 由来と判断してよい」ことを
/// 目的にでき得る限り特異的な文字列を選定している（`node` 単独等の短い
/// トークンは、Rust の識別子・ファイルパス・コメント中の日本語文の英字部分
/// 等と衝突しやすいため採用しない）。採用根拠:
///
/// - `"node_modules"`: NPM パッケージ群が格納されるディレクトリ名。パス
///   断片としてのみ現れる想定で、他の意味を持つ英単語との組み合わせでは
///   ないため偽陽性が起きにくい。
/// - `"#!/usr/bin/env node"` / `"#!/usr/bin/node"`: Node 実行可能スクリプトの
///   shebang。CLI ツールのエントリポイントに特有。
/// - `"process.binding("`: Node の内部（非公開）ネイティブバインディング API。
///   ブラウザ環境の JS には存在しない Node 固有 API。
/// - `"module.exports"`: CommonJS のエクスポート構文。ブラウザ／WASM 向け
///   ES モジュール（`export function` 等、本リポジトリの WASM グルーが
///   採用する形式）には現れない。
/// - `"require(\""` / `"require('"`: CommonJS の `require` 呼び出し。引用符
///   直後までを含めることで、英単語 "requires" 等との衝突を避ける。
/// - `"npm_package_"` / `"npm_config_"`: npm がライフサイクルスクリプトに
///   注入する環境変数プレフィックス。
/// - `"libnode"`: Node ランタイムを埋め込み配布する際の共有ライブラリ名。
///
/// 本テストファイル内の [`marker_detector_flags_synthesized_npm_payload`]
/// が、これらのマーカーが実際に検出可能であることを合成ペイロードで証明する。
const NPM_OR_NODE_MARKERS: &[&str] = &[
    "node_modules",
    "#!/usr/bin/env node",
    "#!/usr/bin/node",
    "process.binding(",
    "module.exports",
    "require(\"",
    "require('",
    "npm_package_",
    "npm_config_",
    "libnode",
];

/// バイト列 `haystack` の中に、いずれかの `markers` が部分文字列として
/// 含まれていれば、最初に一致したマーカーを返す。
///
/// バイナリ（非 UTF-8 混在）に対しても安全に走査できるよう、文字列化を
/// 経由せずバイト列の `windows` 一致で判定する（`str::contains` はバイナリ
/// 全体が有効な UTF-8 であることを要求してしまうため使えない）。
fn find_marker<'a>(haystack: &[u8], markers: &'a [&'a str]) -> Option<&'a str> {
    markers
        .iter()
        .find(|marker| {
            let needle = marker.as_bytes();
            !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
        })
        .copied()
}

/// 層 1（ブラウザ配信物）: コンパイル時埋め込みアセットテーブルに
/// NPM パッケージ由来ファイル・NPM/Node マーカーが混入していないことを検証する。
///
/// `EMBEDDED_ASSETS` は `dist-server/build.rs` が `static/`（および WASM
/// ビルドステージ生成物）を走査して生成する固定テーブルであり、ブラウザへ
/// 実際に配信され得るファイルの全量である（`assets.rs::embedded_lookup`
/// 参照）。ここでの検査はビルド条件（debug/release/`force-embed`）を問わず
/// 常にコンパイル時に確定しているため cfg ゲート不要。
#[test]
fn embedded_assets_do_not_reference_npm_or_node() {
    assert!(
        !EMBEDDED_ASSETS.is_empty(),
        "embedded assets table must not be empty (build.rs must have embedded at least static/view-transitions.js)"
    );

    for (path, bytes) in EMBEDDED_ASSETS {
        assert!(
            !path.contains("node_modules"),
            "embedded asset path must not originate from node_modules: {path}"
        );
        assert!(
            !path.ends_with("package.json") && !path.ends_with("package-lock.json"),
            "embedded asset must not be an NPM package manifest: {path}"
        );

        // JS 系アセットのみ内容マーカー検査の対象とする（wasm/html/css 等は
        // NPM/Node の実行コード形式ではないため対象外。`mime.rs` の既知拡張子
        // 表と一致する範囲に限定する）。
        let is_js_asset = path.ends_with(".js") || path.ends_with(".mjs") || path.ends_with(".cjs");
        if is_js_asset {
            if let Some(marker) = find_marker(bytes, NPM_OR_NODE_MARKERS) {
                panic!("embedded JS asset {path} contains NPM/Node marker: {marker}");
            }
        }
    }
}

/// 開発モード配信ルート `static/`（ワークスペース直下）に、NPM パッケージ
/// インストールの痕跡（`node_modules/` ディレクトリ・`package.json`）が
/// 存在しないことを検証する。
///
/// dev モード（`assets::AssetMode::DevFilesystem`）はこのディレクトリを
/// 実行時に直接読むため（`assets.rs::dev_fs` 参照）、埋め込みテーブル検査
/// （層 1）だけでは dev 配信経路をカバーできない。本テストがその隙間を埋める。
#[test]
fn dev_static_root_has_no_npm_install_artifacts() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let static_root = manifest_dir
        .parent()
        .expect("dist-server/ has a parent directory (workspace root)")
        .join("static");

    assert!(
        static_root.is_dir(),
        "workspace static/ directory must exist: {}",
        static_root.display()
    );
    assert!(
        !static_root.join("node_modules").exists(),
        "static/ must not contain an installed node_modules/ directory"
    );
    assert!(
        !static_root.join("package.json").exists(),
        "static/ must not contain a package.json (npm project marker)"
    );
    assert!(
        !static_root.join("package-lock.json").exists(),
        "static/ must not contain a package-lock.json (npm project marker)"
    );
}

/// 層 2（配布バイナリ）: 実際にビルドされた `dist-server` バイナリのバイト列
/// 全体を走査し、NPM パッケージコード・Node ランタイムのマーカーが 1 つも
/// 含まれないことを検証する。
///
/// `env!("CARGO_BIN_EXE_dist-server")` は cargo がテストバイナリのビルドに
/// 先立って実ビルドした `dist-server` バイナリの絶対パスを指す
/// （`tests/isolated_run.rs` と同じ取得方法）。debug/release いずれの
/// プロファイルでも実行可能で、force-embed フィーチャー有無を問わず常に
/// 実行される（実バイナリの内容検査であり `AssetMode` の分岐に依存しない
/// ため、他のテストファイルのような embedded-mode 限定 cfg ゲートは不要）。
#[test]
fn distributed_binary_does_not_embed_npm_or_node_runtime() {
    let binary_path = Path::new(env!("CARGO_BIN_EXE_dist-server"));
    let binary_bytes = fs::read(binary_path).expect("dist-server binary must be readable");

    assert!(
        !binary_bytes.is_empty(),
        "dist-server binary must not be empty"
    );

    if let Some(marker) = find_marker(&binary_bytes, NPM_OR_NODE_MARKERS) {
        panic!(
            "dist-server binary ({}) contains NPM/Node marker: {marker}",
            binary_path.display()
        );
    }
}

/// 層 3（Docker イメージ構造）: ワークスペースルートの `Dockerfile` を
/// 行ベースで解析し、最終ステージ（`FROM scratch`）が
/// 「ビルダーステージからのバイナリ 1 件のみを `COPY` し、`RUN` を含まない」
/// ことを検証する。
///
/// この構造が保たれる限り、最終イメージへ Node ランタイム・NPM パッケージを
/// 持ち込む経路（`RUN npm install` 等の実行）が構造的に存在しない。
/// ビルダーステージ自体は変更対象外（TASK-10.3・イシュー #114 で
/// wasm-bindgen 導入等の正当な変更が入る予定のため）だが、`nodejs`/`npm ` の
/// インストールを示す明示的なトークンがビルダーステージにも存在しないことを
/// 併せて確認し、将来の改変を過度に制約しない形で検査する。
#[test]
fn dockerfile_final_stage_copies_only_the_binary_and_has_no_run_instruction() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dockerfile_path = manifest_dir
        .parent()
        .expect("dist-server/ has a parent directory (workspace root)")
        .join("Dockerfile");

    let content = fs::read_to_string(&dockerfile_path).expect("Dockerfile must be readable");

    // `FROM` 行でステージに分割する（`FROM ... AS builder` / `FROM scratch`
    // の 2 ステージ構成を前提とする本 Dockerfile の実際の構造に合わせる。
    // 3 ステージ以上への拡張時は「最終ステージ」の判定のみで足りるため
    // この分割方法のままで対応できる）。
    let mut stages: Vec<Vec<&str>> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("FROM ") {
            stages.push(Vec::new());
        }
        if let Some(current) = stages.last_mut() {
            current.push(line);
        }
    }
    assert!(
        stages.len() >= 2,
        "Dockerfile must have at least a builder stage and a final stage"
    );

    let final_stage = stages.last().expect("stages must not be empty");
    let final_stage_header = final_stage
        .first()
        .expect("each stage has a FROM header line")
        .trim();
    assert!(
        final_stage_header.starts_with("FROM scratch"),
        "final stage must be `FROM scratch`, got: {final_stage_header}"
    );

    let mut copy_instruction_count = 0;
    let mut entrypoint_found = false;
    for line in final_stage.iter().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let instruction = trimmed.split_whitespace().next().unwrap_or("");
        match instruction {
            "COPY" => {
                copy_instruction_count += 1;
                assert!(
                    trimmed.contains("--from=builder"),
                    "final stage COPY must come from the builder stage, got: {trimmed}"
                );
            }
            "USER" | "ENV" | "EXPOSE" => {
                // 非 root 実行・ポート公開・実行時設定は許可された命令
                // （Node/NPM ランタイムを持ち込まない）。
            }
            "ENTRYPOINT" => {
                entrypoint_found = true;
                assert!(
                    trimmed.contains("/dist-server"),
                    "ENTRYPOINT must point at the dist-server binary, got: {trimmed}"
                );
            }
            "RUN" => {
                panic!(
                    "final stage must not contain RUN (no code execution at image-build time in the final stage): {trimmed}"
                );
            }
            other => {
                panic!("unexpected instruction in final stage: {other} (line: {trimmed})");
            }
        }
    }

    assert_eq!(
        copy_instruction_count, 1,
        "final stage must COPY exactly one artifact (the dist-server binary)"
    );
    assert!(
        entrypoint_found,
        "final stage must declare ENTRYPOINT for the dist-server binary"
    );

    // ビルダーステージを含む全行に、Node/npm インストールを示す明示的な
    // トークンがないことを確認する。単語境界を意識した特異的なトークンに
    // 限定し、`wasm-bindgen` 等の将来導入（TASK-10.3）とは衝突しないようにする。
    let dockerfile_wide_markers = ["nodejs", "npm install", "npm ci", "apt-get install -y npm"];
    for marker in dockerfile_wide_markers {
        assert!(
            !content.contains(marker),
            "Dockerfile must not install a Node/NPM runtime (found marker: {marker})"
        );
    }
}

/// マーカー検出器（[`find_marker`]）の実効性を、合成した NPM 風ペイロードで
/// 証明する負例テスト。
///
/// blocklist 方式の検出器は「マーカーが古くなって何も検出できなくなる」
/// 形で静かに骨抜きになり得る（例: 生成ツールのバージョンが変わりマーカーの
/// 綴りが変化した等）。本テストはそれを防ぐフェイルクローズの証明であり、
/// 上記の実アセット・実バイナリ検査（正例が「クリーンである」ことしか
/// 示さない）を補完する。
#[test]
fn marker_detector_flags_synthesized_npm_payload() {
    let synthetic_payloads: &[&[u8]] = &[
        b"// generated glue\nrequire(\"node_modules/foo/index.js\");\n",
        b"#!/usr/bin/env node\nconsole.log('hello');\n",
        b"module.exports = { run() {} };\n",
        b"process.binding('fs');\n",
        b"process.env.npm_package_name;\n",
    ];

    for payload in synthetic_payloads {
        assert!(
            find_marker(payload, NPM_OR_NODE_MARKERS).is_some(),
            "detector failed to flag synthesized NPM/Node payload: {:?}",
            std::str::from_utf8(payload).unwrap_or("<non-utf8>")
        );
    }

    // 対照実験: NPM/Node と無関係な第一者 JS 相当のペイロードは検出されない
    // こと（誤検知していないことの確認）。
    let benign_payload = b"export function withViewTransition(update) { return update(); }\n";
    assert!(
        find_marker(benign_payload, NPM_OR_NODE_MARKERS).is_none(),
        "detector must not flag benign first-party ES module code"
    );
}
