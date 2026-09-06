//! `Dockerfile` / `.github/workflows/ci.yml` に散在する wasm-bindgen-cli の
//! 固定バージョン・SHA256 チェックサムと、`Cargo.lock` が解決した
//! `wasm-bindgen` クレートのバージョンとの同期ドリフトを検出する回帰テスト。
//!
//! TASK-10.3c（issue #117、`docs/design/docker-wasm-build-stage.md` §6 検証観点 2）の
//! ギャップ補完として追加した。`dist-server/build.rs::expected_wasm_bindgen_version`
//! は Cargo.lock 解決済みバージョンとの完全一致をフェイルクローズで要求するため、
//! `Dockerfile`・`ci.yml` の固定値がドリフトすると Docker ビルド・CI が
//! **実行時**（イメージビルド・ワークフロー実行）まで待たないと失敗が判明しない。
//! 本テストはそのドリフトを `cargo test` 時点で前倒し検出し、原因特定コストを
//! 下げる（検出自体は fail-closed のまま、検出タイミングを早める）。
//!
//! 外部 TOML/YAML/Dockerfile パーサは追加しない（REQ-3・xtask 外部依存ゼロ方針。
//! 依存追加のユーザー承認を得られない自動運転では「追加しない」側に倒す）。
//! そのため抽出は行ベースの文字列一致に留める。
//!
//! イシュー #1274（`Fandhe-AI/actions/wasm-tool-install` composite action への
//! 置換）以降、pin の宣言形式は Dockerfile（shell 代入 `KEY="value"`）と
//! ci.yml（ワークフローレベル `env:` ブロックの YAML mapping `KEY: "value"`）
//! で異なる。`extract_quoted_assignment` は両形式を受理する（`: "` / `="` の
//! いずれかを鍵の直後の区切りとして許容）。
//!
//! 4 テストの役割分担（イシュー #1218 で 3 本目、イシュー #1274 で 4 本目を追加）:
//! 1. `dockerfile_and_ci_pin_wasm_bindgen_version_in_sync_with_cargo_lock`:
//!    `WASM_BINDGEN_VERSION` の全出現（Dockerfile の x86_64/aarch64 両分岐 +
//!    ci.yml の env ブロック）と Cargo.lock 解決バージョンの突合。
//! 2. `dockerfile_and_ci_pin_matching_wasm_bindgen_sha256_for_x86_64_archive`:
//!    Dockerfile の x86_64 分岐 SHA256 と ci.yml（GitHub-hosted runner は
//!    x86_64 のみ）の SHA256 の突合。
//! 3. `dockerfile_pins_known_wasm_bindgen_sha256_for_aarch64_archive`:
//!    Dockerfile の aarch64 分岐 SHA256 は ci.yml に対応値が存在せず
//!    他ファイルとの突合ができないため、既知の正値（PR #1214 の aarch64
//!    実機 `sha256sum -c` 通過実績）とのハードコード突合で検知する
//!    （イシュー #1216 評価 §5 の解消方式、イシュー #1218 で解消）。
//! 4. `ci_yml_wasm_tool_install_steps_reference_env_pins_only`（イシュー #1274）:
//!    ci.yml の pin は env ブロックへ単一宣言点化されたため、
//!    `Fandhe-AI/actions/wasm-tool-install` 呼び出しステップが `with:` へ
//!    リテラル版数/SHA256 を直書きしていないこと（= env コンテキスト参照の
//!    みであること）を検証し、pin の二重管理の再発を防ぐ。加えて `tool:`
//!    の値（wasm-bindgen / wasm-pack）と `version:`/`sha256:` が参照する env
//!    ファミリー（WASM_BINDGEN_* / WASM_PACK_*）の対応も検証し、
//!    `tool: wasm-pack` に `WASM_BINDGEN_VERSION` を参照させるような
//!    取り違え（両方とも「env 参照である」ことだけを見る判定では検知でき
//!    ない）も検知する。呼び出し自体が 1 件も存在しない場合（全ステップの
//!    誤削除）も fail-closed に検知する。
//! 5. `wasm_subworkspace_locks_resolve_wasm_bindgen_in_sync_with_cargo_lock`
//!    （イシュー #1965）: ルートワークスペースから意図的に切り離された
//!    wasm サブワークスペース（`templates/app/wasm/`・
//!    `examples/interactive-view-transitions/wasm/`）と、それらの
//!    `crates/cli/` 同梱コピー（`include_str!` 用）が持つ独自
//!    `Cargo.lock` の `wasm-bindgen` 解決バージョンを、ルート
//!    `Cargo.lock`（テスト 1 が Dockerfile/ci.yml と突合済みの真値）と
//!    突合する。PR #1891（0.2.128 更新）で examples 側 2 lock が
//!    0.2.126 のまま取り残され、正本と同梱コピーが「揃って古い」ため
//!    `crates/cli/tests/example_publish_copy_drift.rs`（バイト一致検知）
//!    もすり抜けた実績（イシュー #1964・PR #1981 で是正済み）を踏まえ、
//!    このギャップを埋める。加えて、抽出関数
//!    `extract_wasm_bindgen_version_from_lock` 単体に対する否定ケース
//!    （前方一致パッケージの除外・フォーマット崩れの fail-closed 検知）
//!    を単体テストとして持つ。

use std::path::PathBuf;

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

/// 与えられた `Cargo.lock` 内容から `[[package]] name = "wasm-bindgen"`
/// ブロックの `version` を厳密抽出する純粋関数。
///
/// `wasm-bindgen-backend` / `wasm-bindgen-macro` 等、名前が前方一致する
/// 別パッケージのブロックを誤って拾わないよう、`name = "wasm-bindgen"` を
/// 完全一致（引用符込み）で探し、直後の `version = "..."` 行を読む。
/// `label` はパニックメッセージにどの lock を走査したかを示す表示名
/// （イシュー #1965 でルート `Cargo.lock` 専用の `cargo_lock_wasm_bindgen_version`
/// から抽出し、wasm サブワークスペース lock にも再利用できるようにした）。
fn extract_wasm_bindgen_version_from_lock(contents: &str, label: &str) -> String {
    let lines: Vec<&str> = contents.lines().collect();
    let name_line_index = lines
        .iter()
        .position(|line| line.trim() == "name = \"wasm-bindgen\"")
        .unwrap_or_else(|| {
            panic!("{label} に [[package]] name = \"wasm-bindgen\" ブロックが見つからない")
        });

    let version_line = lines
        .get(name_line_index + 1)
        .unwrap_or_else(|| panic!("{label} の name = \"wasm-bindgen\" の直後に行が存在する"))
        .trim();

    version_line
        .strip_prefix("version = \"")
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or_else(|| {
            panic!(
                "{label} の wasm-bindgen ブロックで name の直後が \
                 version 行になっていない（Cargo.lock のフォーマット変更 \
                 の可能性）: {version_line}"
            )
        })
        .to_owned()
}

/// 指定した workspace 相対パスの `Cargo.lock` を読み込み、`wasm-bindgen` の
/// 解決バージョンを返す。存在しないパスは `read_workspace_file` が
/// panic するため、パスのリネーム・削除は自動的に fail-closed になる。
fn wasm_bindgen_version_in_lock(relative_path: &str) -> String {
    let contents = read_workspace_file(relative_path);
    extract_wasm_bindgen_version_from_lock(&contents, relative_path)
}

/// ルートワークスペースの `Cargo.lock` が解決した `wasm-bindgen` の
/// バージョンを返す（既存呼び出し元向けの薄いラッパ）。
fn cargo_lock_wasm_bindgen_version() -> String {
    wasm_bindgen_version_in_lock("Cargo.lock")
}

/// wasm サブワークスペース（ルートワークスペースから意図的に切り離された
/// 独立ワークスペース）の `Cargo.lock`。正本（`templates/app/wasm/`・
/// `examples/interactive-view-transitions/wasm/`）と `crates/cli/` 同梱
/// コピー（`fw new`/`fw new --example` の `include_str!` 用）の両方を
/// 列挙する。同梱コピー同士の一致は `crates/cli/tests/template_publish_copy_drift.rs`・
/// `crates/cli/tests/example_publish_copy_drift.rs` が担うが、「正本と
/// 同梱コピーが揃って古い」状態（PR #1891 の取り残し、イシュー #1964 で
/// 是正）はそれらのバイト一致検知ではすり抜けるため、ここで検知する。
///
/// 新しい wasm サブワークスペースを追加したときは本定数へ追記すること。
const WASM_SUBWORKSPACE_LOCKS: &[&str] = &[
    "templates/app/wasm/Cargo.lock",
    "examples/interactive-view-transitions/wasm/Cargo.lock",
    "crates/cli/templates/app/wasm/Cargo.lock",
    "crates/cli/embedded-examples/interactive-view-transitions/wasm/Cargo.lock",
];

/// `key="value"`（shell 代入形式）の代入から、キーの直後の `"..."` で
/// 囲まれた値のみを取り出す。
///
/// `Dockerfile` の宣言は行末に `; \`（シェル継続）が付くため、閉じ引用符の
/// 後ろに余分な文字が残る。`strip_suffix` で末尾一致を狙うと `Dockerfile`
/// 側で確実に外れるため、開き引用符の直後から次の引用符までを走査して
/// 値を切り出す。
fn extract_quoted_assignment<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let after_key = line.trim_start().strip_prefix(key)?;
    let after_open_quote = after_key.strip_prefix('"')?;
    let end = after_open_quote.find('"')?;
    Some(&after_open_quote[..end])
}

/// `key: "value"`（YAML mapping 形式）の代入から値を取り出す。
///
/// イシュー #1274 で ci.yml の pin 宣言をワークフローレベル `env:` ブロック
/// （YAML mapping）へ集約したため、Dockerfile の shell 代入形式
/// （`extract_quoted_assignment`）とは区切り文字（`=` vs `: `）が異なる。
/// 外部 YAML パーサは追加しない方針（REQ-3）のため、行頭一致 + 引用符内
/// 走査という同型の行ベース抽出に留める。
fn extract_yaml_quoted_assignment<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let after_key = line.trim_start().strip_prefix(key)?.strip_prefix(": ")?;
    let after_open_quote = after_key.strip_prefix('"')?;
    let end = after_open_quote.find('"')?;
    Some(&after_open_quote[..end])
}

/// 指定した内容から `WASM_BINDGEN_VERSION` 宣言の全出現を抽出する。
///
/// `Dockerfile`（x86_64/aarch64 の 2 分岐、shell 代入形式 `KEY="value"`）と
/// `ci.yml`（ワークフローレベル env ブロック、YAML mapping 形式
/// `KEY: "value"`）の両形式を受理する（イシュー #1274）。
fn extract_wasm_bindgen_versions(contents: &str) -> Vec<String> {
    contents
        .lines()
        .filter_map(|line| {
            extract_quoted_assignment(line, "WASM_BINDGEN_VERSION=")
                .or_else(|| extract_yaml_quoted_assignment(line, "WASM_BINDGEN_VERSION"))
        })
        .map(str::to_owned)
        .collect()
}

/// `WASM_BINDGEN_SHA256` 宣言の全出現を抽出する（両形式対応、イシュー #1274）。
fn extract_wasm_bindgen_sha256s(contents: &str) -> Vec<String> {
    contents
        .lines()
        .filter_map(|line| {
            extract_quoted_assignment(line, "WASM_BINDGEN_SHA256=")
                .or_else(|| extract_yaml_quoted_assignment(line, "WASM_BINDGEN_SHA256"))
        })
        .map(str::to_owned)
        .collect()
}

#[test]
fn dockerfile_and_ci_pin_wasm_bindgen_version_in_sync_with_cargo_lock() {
    let expected_version = cargo_lock_wasm_bindgen_version();

    let dockerfile_contents = read_workspace_file("Dockerfile");
    let ci_contents = read_workspace_file(".github/workflows/ci.yml");

    let dockerfile_versions = extract_wasm_bindgen_versions(&dockerfile_contents);
    let ci_versions = extract_wasm_bindgen_versions(&ci_contents);

    // 検証そのものが形骸化（固定値の宣言が消えた）していないことを保証する。
    // 0 件はテストの誤 pass ではなく実際の欠落を意味するため fail-closed とする。
    assert!(
        !dockerfile_versions.is_empty(),
        "Dockerfile に WASM_BINDGEN_VERSION=\"...\" の宣言が見つからない \
         （wasm-bindgen-cli 導入ステップが削除・変更された可能性）"
    );
    assert!(
        !ci_versions.is_empty(),
        ".github/workflows/ci.yml に WASM_BINDGEN_VERSION=\"...\" の宣言が \
         見つからない（wasm-bindgen-cli 導入ステップが削除・変更された可能性）"
    );

    for version in &dockerfile_versions {
        assert_eq!(
            version, &expected_version,
            "Dockerfile の WASM_BINDGEN_VERSION（{version}）が Cargo.lock の \
             wasm-bindgen 解決バージョン（{expected_version}）とずれている。 \
             dist-server/build.rs::expected_wasm_bindgen_version がこの不一致を \
             フェイルクローズで検出し Docker ビルドを失敗させる前に、ここで \
             ドリフトを検出している。Dockerfile の WASM_BINDGEN_VERSION と \
             SHA256 を Cargo.lock の解決バージョンに合わせて更新すること"
        );
    }
    for version in &ci_versions {
        assert_eq!(
            version, &expected_version,
            "ci.yml の WASM_BINDGEN_VERSION（{version}）が Cargo.lock の \
             wasm-bindgen 解決バージョン（{expected_version}）とずれている。 \
             ci.yml の全ジョブの WASM_BINDGEN_VERSION と SHA256 を Cargo.lock \
             の解決バージョンに合わせて更新すること"
        );
    }
}

#[test]
fn dockerfile_and_ci_pin_matching_wasm_bindgen_sha256_for_x86_64_archive() {
    // Dockerfile は x86_64/aarch64 の 2 アーキ分岐で SHA256 を持つが、
    // ci.yml の GitHub-hosted runner は x86_64 のみのため、両ファイルで
    // 共通に検証できるのは x86_64-unknown-linux-musl archive の SHA256 のみ。
    // 同一 archive に異なるチェックサムが記載されている状態は、どちらかの
    // 改変・取得元の誤りを示す危険な兆候であり、サプライチェーン対策
    // （A08）としてここで一致を保証する。
    let dockerfile_contents = read_workspace_file("Dockerfile");
    let ci_contents = read_workspace_file(".github/workflows/ci.yml");

    let dockerfile_sha256s = extract_wasm_bindgen_sha256s(&dockerfile_contents);
    let ci_sha256s = extract_wasm_bindgen_sha256s(&ci_contents);

    assert!(
        !dockerfile_sha256s.is_empty(),
        "Dockerfile に WASM_BINDGEN_SHA256=\"...\" の宣言が見つからない \
         （チェックサム検証ステップが削除された可能性。A08 サプライ \
         チェーン対策の弱体化）"
    );
    assert!(
        !ci_sha256s.is_empty(),
        ".github/workflows/ci.yml に WASM_BINDGEN_SHA256=\"...\" の宣言が \
         見つからない（チェックサム検証ステップが削除された可能性。A08 \
         サプライチェーン対策の弱体化）"
    );

    // ci.yml は x86_64 archive のみを扱うため、その先頭値を x86_64 側の
    // 代表値とみなす（ci.yml 内の複数ジョブが同一値を宣言している前提は
    // 次のテストで別途検証する）。
    let ci_x86_64_sha256 = &ci_sha256s[0];

    // Dockerfile は case 文で x86_64 用の値を先に宣言している（現行実装）。
    // アーキ非依存に x86_64 用の値であることを明示するため、行の直前に
    // x86_64 archive 名が現れるブロックの SHA256 を対象とする。
    let dockerfile_x86_64_sha256 = dockerfile_contents
        .lines()
        .zip(dockerfile_contents.lines().skip(1))
        .find_map(|(archive_line, sha_line)| {
            if archive_line.contains("x86_64-unknown-linux-musl.tar.gz") {
                extract_quoted_assignment(sha_line, "WASM_BINDGEN_SHA256=")
            } else {
                None
            }
        })
        .expect(
            "Dockerfile 内で x86_64-unknown-linux-musl archive 行の直後に \
             WASM_BINDGEN_SHA256 が見つからない（Dockerfile の記述順序が \
             変わった可能性。テストの抽出ロジックの追随が必要）",
        );

    assert_eq!(
        dockerfile_x86_64_sha256, ci_x86_64_sha256,
        "Dockerfile と ci.yml の x86_64-unknown-linux-musl archive に対する \
         WASM_BINDGEN_SHA256 が一致していない（同一 archive への相違 \
         チェックサム。改変・改ざんの兆候として扱い、取得元の \
         https://github.com/rustwasm/wasm-bindgen/releases から正しい値を \
         再確認すること）"
    );

    for sha256 in &ci_sha256s {
        assert_eq!(
            sha256, ci_x86_64_sha256,
            "ci.yml 内で複数ジョブの WASM_BINDGEN_SHA256 が食い違っている \
             （同一 archive を指しているはずのジョブ間で不一致）"
        );
    }
}

/// Dockerfile の aarch64 分岐 `WASM_BINDGEN_SHA256` が既知の正値と一致する
/// ことを保証する。
///
/// aarch64 archive の SHA256 は ci.yml（GitHub-hosted runner は x86_64 のみ）
/// に対応する値が存在しないため、`dockerfile_and_ci_pin_matching_wasm_bindgen_sha256_for_x86_64_archive`
/// のような他ファイルとの突合ができない（イシュー #1216 評価
/// `docs/ci/aarch64-docker-wasm-rebuild-ci-evaluation.md` §1.1・§3 論点 3 で
/// 特定した既知ギャップ）。本テストは既知値のハードコード突合でこの
/// ギャップを埋める（同文書 §5 の解消方式）。
///
/// # `WASM_BINDGEN_VERSION` バンプ時の期待値更新手順
///
/// 1. `https://github.com/rustwasm/wasm-bindgen/releases/download/<新バージョン>/wasm-bindgen-<新バージョン>-aarch64-unknown-linux-musl.tar.gz`
///    を取得し `sha256sum`（macOS は `shasum -a 256`）で算出する。取得元は
///    GitHub Releases に限定し、x86_64 側と著しく異なるファイルサイズ等の
///    改変兆候がないか確認する（A08 サプライチェーン対策、
///    `docs/ci/aarch64-docker-wasm-rebuild-ci-evaluation.md` §6 と同じ制約）。
/// 2. 算出値で Dockerfile の aarch64 分岐と本テストの
///    `KNOWN_AARCH64_SHA256`・`KNOWN_VERSION` を同時に更新する。
///
/// `KNOWN_VERSION` と `cargo_lock_wasm_bindgen_version()` の一致を assert
/// することで、`WASM_BINDGEN_VERSION` バンプ時（Cargo.lock 更新時）は
/// 本テストが必ず fail し、古い aarch64 SHA256 の残置がすり抜けない
/// （既知ペアの更新を強制する fail-closed 設計）。
///
/// 現行既知値（0.2.128）の出所: upstream で wasm-bindgen 0.2.128（web-sys
/// 0.3.105 / wasm-bindgen-futures 0.4.78 が `wasm-bindgen = "=0.2.128"` を
/// 厳密要求するようになったための追随バンプ）が公開されたため、GitHub
/// Releases から aarch64 archive を取得し `shasum -a 256` で算出した値
/// （aarch64 実機での `sha256sum -c` 通過検証は本バンプでは実施していない）。
/// これ以前の値（0.2.127）の出所はイシュー #1305（web-sys 0.3.104 が
/// wasm-bindgen =0.2.127 を厳密要求するようになったための追随バンプ）で
/// GitHub Releases から aarch64 archive を取得し `sha256sum` で算出した値。
#[test]
fn dockerfile_pins_known_wasm_bindgen_sha256_for_aarch64_archive() {
    const KNOWN_VERSION: &str = "0.2.128";
    const KNOWN_AARCH64_SHA256: &str =
        "079731dd1bc7798c1efa4f08fcc45130827cbcc9ff60a0b4c6047d64fc6fd25c";

    let cargo_lock_version = cargo_lock_wasm_bindgen_version();
    assert_eq!(
        cargo_lock_version, KNOWN_VERSION,
        "Cargo.lock の wasm-bindgen 解決バージョン（{cargo_lock_version}）が \
         本テストの既知値 KNOWN_VERSION（{KNOWN_VERSION}）とずれている。 \
         WASM_BINDGEN_VERSION がバンプされた可能性がある。上記 rustdoc の \
         手順に従い、Dockerfile の aarch64 分岐 WASM_BINDGEN_SHA256 と \
         本テストの KNOWN_VERSION・KNOWN_AARCH64_SHA256 を新バージョンの \
         正値へ同時に更新すること（更新を怠ると古い SHA256 の残置が \
         検知されずに残ってしまう）"
    );

    let dockerfile_contents = read_workspace_file("Dockerfile");

    let dockerfile_aarch64_sha256 = dockerfile_contents
        .lines()
        .zip(dockerfile_contents.lines().skip(1))
        .find_map(|(archive_line, sha_line)| {
            if archive_line.contains("aarch64-unknown-linux-musl.tar.gz") {
                extract_quoted_assignment(sha_line, "WASM_BINDGEN_SHA256=")
            } else {
                None
            }
        })
        .expect(
            "Dockerfile 内で aarch64-unknown-linux-musl archive 行の直後に \
             WASM_BINDGEN_SHA256 が見つからない（チェックサム検証ステップが \
             削除された、または Dockerfile の記述順序が変わった可能性。 \
             A08 サプライチェーン対策の弱体化、またはテストの抽出ロジックの \
             追随が必要）",
        );

    assert_eq!(
        dockerfile_aarch64_sha256, KNOWN_AARCH64_SHA256,
        "Dockerfile の aarch64-unknown-linux-musl archive に対する \
         WASM_BINDGEN_SHA256（{dockerfile_aarch64_sha256}）が既知の正値 \
         （{KNOWN_AARCH64_SHA256}）と一致しない。改変・改ざんの兆候として \
         扱い、取得元 https://github.com/rustwasm/wasm-bindgen/releases から \
         正しい値を再確認すること"
    );
}

/// ci.yml の `Fandhe-AI/actions/wasm-tool-install` 呼び出しステップが、
/// pin 値を `with:` へリテラル直書きせず、ワークフロー冒頭の env ブロック
/// （`${{ env.WASM_BINDGEN_VERSION }}` 等）への参照のみで注入していることを
/// 検証する（イシュー #1274）。
///
/// 背景: pin の正を env ブロックへ単一宣言点化した設計は、新規ステップが
/// リテラル版数/SHA256 を直書きすると同期検知（本ファイル冒頭 2 テスト）の
/// 対象から漏れてしまう構造的リスクを持つ（env ブロックの値だけを追随
/// 更新しても、直書きされた古い値が気づかれずに残り得るため）。本テストは
/// この「二重管理の再発」を fail-closed に検知する。
///
/// 呼び出し自体が 1 件も見つからない場合（置換の全体的な巻き戻し・誤削除）
/// も fail-closed に検知する（`wasm-tool-install` 導入そのものの消失防止）。
#[test]
fn ci_yml_wasm_tool_install_steps_reference_env_pins_only() {
    let ci_contents = read_workspace_file(".github/workflows/ci.yml");
    let lines: Vec<&str> = ci_contents.lines().collect();

    let uses_indices: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| {
            line.trim_start()
                .starts_with("uses: Fandhe-AI/actions/wasm-tool-install")
        })
        .map(|(idx, _)| idx)
        .collect();

    assert!(
        !uses_indices.is_empty(),
        "ci.yml に Fandhe-AI/actions/wasm-tool-install の呼び出しが \
         1 件も見つからない（イシュー #1274 の置換が巻き戻された、または \
         誤って全削除された可能性）"
    );

    for &idx in &uses_indices {
        // `uses:` の直後は `with:` → `tool:` → `version:` → `sha256:` の
        // 固定順で並ぶ（本ファイル冒頭のリファレンス実装を参照）。範囲を
        // 直後 6 行に限定し、無関係な後続ステップのコメント等を誤って
        // 拾わないようにする。
        let block: Vec<&str> = lines.iter().skip(idx + 1).take(6).copied().collect();

        let tool_line = block
            .iter()
            .find(|line| line.trim_start().starts_with("tool:"))
            .unwrap_or_else(|| {
                panic!(
                    "ci.yml の {} 行目付近の wasm-tool-install 呼び出しに \
                     tool: フィールドが見つからない",
                    idx + 1
                )
            });
        let version_line = block
            .iter()
            .find(|line| line.trim_start().starts_with("version:"))
            .unwrap_or_else(|| {
                panic!(
                    "ci.yml の {} 行目付近の wasm-tool-install 呼び出しに \
                     version: フィールドが見つからない",
                    idx + 1
                )
            });
        let sha256_line = block
            .iter()
            .find(|line| line.trim_start().starts_with("sha256:"))
            .unwrap_or_else(|| {
                panic!(
                    "ci.yml の {} 行目付近の wasm-tool-install 呼び出しに \
                     sha256: フィールドが見つからない",
                    idx + 1
                )
            });

        // `tool:` の値ごとに正しい env ファミリー（WASM_BINDGEN_* /
        // WASM_PACK_*）へのみ参照が許される。`tool: wasm-pack` に対して
        // `version: ${{ env.WASM_BINDGEN_VERSION }}` を書くような取り違えは、
        // 単純な「env 参照であるか」だけの判定では見逃す（両方とも env
        // 参照ではあるため）。これを防ぐため、tool の値で許容する env 名を
        // 決定してから照合する。
        let (expected_version_env, expected_sha256_env) = if tool_line.contains("wasm-bindgen") {
            ("WASM_BINDGEN_VERSION", "WASM_BINDGEN_SHA256")
        } else if tool_line.contains("wasm-pack") {
            ("WASM_PACK_VERSION", "WASM_PACK_SHA256")
        } else {
            panic!(
                "ci.yml の {} 行目付近の wasm-tool-install 呼び出しの tool: \
                 が wasm-bindgen / wasm-pack のいずれでもない（{tool_line}）",
                idx + 1
            );
        };

        let expected_version_ref = format!("${{{{ env.{expected_version_env} }}}}");
        let expected_sha256_ref = format!("${{{{ env.{expected_sha256_env} }}}}");

        assert!(
            version_line.contains(&expected_version_ref),
            "ci.yml の {} 行目付近の wasm-tool-install 呼び出しの version: \
             が期待される env 参照（{expected_version_ref}）になっていない \
             （{version_line}）。tool: と version: の env ファミリーの \
             取り違え（例: wasm-pack に WASM_BINDGEN_VERSION）、またはpin \
             値のリテラル直書きの可能性がある",
            idx + 1
        );
        assert!(
            sha256_line.contains(&expected_sha256_ref),
            "ci.yml の {} 行目付近の wasm-tool-install 呼び出しの sha256: \
             が期待される env 参照（{expected_sha256_ref}）になっていない \
             （{sha256_line}）。tool: と sha256: の env ファミリーの \
             取り違え、またはpin 値のリテラル直書きの可能性がある",
            idx + 1
        );
    }
}
/// wasm サブワークスペース 4 lock（正本 2 件・`crates/cli/` 同梱コピー 2 件）
/// が解決する `wasm-bindgen` バージョンが、ルート `Cargo.lock` の解決
/// バージョン（テスト 1 が Dockerfile/ci.yml と突合済みの真値）と一致する
/// ことを検証する（イシュー #1965）。
///
/// PR #1891（0.2.128 更新）で examples 側 2 lock が 0.2.126 のまま取り残され
/// た際、正本と `crates/cli/` 同梱コピーが「揃って古い」状態だったため
/// `crates/cli/tests/example_publish_copy_drift.rs`（バイト一致検知）も
/// すり抜けた（イシュー #1964・PR #1981 で是正済み）。本テストはこの
/// ギャップ（正本と同梱コピーが揃って上流ドリフトから取り残される状態）
/// を `cargo test` 時点で fail-closed に検知する。
#[test]
fn wasm_subworkspace_locks_resolve_wasm_bindgen_in_sync_with_cargo_lock() {
    // 検証そのものが形骸化（対象リストが空になった）していないことを保証
    // する。0 件はテストの誤 pass ではなく実際の欠落を意味するため
    // fail-closed とする。
    assert!(
        !WASM_SUBWORKSPACE_LOCKS.is_empty(),
        "WASM_SUBWORKSPACE_LOCKS が空になっている（wasm サブワークスペース \
         lock の検証対象が消失した可能性）"
    );

    let expected_version = cargo_lock_wasm_bindgen_version();

    for relative_path in WASM_SUBWORKSPACE_LOCKS {
        let actual_version = wasm_bindgen_version_in_lock(relative_path);
        assert_eq!(
            actual_version, expected_version,
            "{relative_path} が解決する wasm-bindgen のバージョン \
             （{actual_version}）が、ルート Cargo.lock の解決バージョン \
             （{expected_version}）とずれている。是正するには \
             `cargo update --manifest-path <サブワークスペース>/Cargo.toml \
             -p wasm-bindgen --precise {expected_version}` で正本の lock を \
             再生成し（`templates/app/tools/wasm/build.sh` の自動再ピン \
             機構でも代替可）、`crates/cli/` 配下の同梱コピーへバイト一致で \
             同期すること。同期漏れは \
             crates/cli/tests/template_publish_copy_drift.rs・ \
             crates/cli/tests/example_publish_copy_drift.rs が別途検知する"
        );
    }
}

/// `extract_wasm_bindgen_version_from_lock` が、前方一致する別パッケージ
/// （`wasm-bindgen-backend` 等）のブロックを誤って拾わず、`name = "wasm-bindgen"`
/// に完全一致するブロック直後の `version` のみを正しく読み取ることを検証
/// する（受け入れ基準 2 のテスト内恒久化、イシュー #1965）。
#[test]
fn extract_wasm_bindgen_version_from_lock_reads_exact_package_block() {
    let synthetic_lock = "\
[[package]]
name = \"wasm-bindgen-backend\"
version = \"9.9.9\"

[[package]]
name = \"wasm-bindgen\"
version = \"0.0.0\"

[[package]]
name = \"wasm-bindgen-macro\"
version = \"8.8.8\"
";

    let version = extract_wasm_bindgen_version_from_lock(synthetic_lock, "synthetic-lock");

    assert_eq!(
        version, "0.0.0",
        "前方一致パッケージ（wasm-bindgen-backend / wasm-bindgen-macro）の \
         version を誤って拾っている。name の完全一致判定が壊れている \
         可能性がある"
    );
}

/// `extract_wasm_bindgen_version_from_lock` は、`name = "wasm-bindgen"` の
/// 直後が `version` 行でない（Cargo.lock のフォーマットが崩れた）場合に
/// fail-closed に panic することを検証する（受け入れ基準 2 のテスト内
/// 恒久化、イシュー #1965）。
#[test]
#[should_panic(expected = "version 行になっていない")]
fn extract_wasm_bindgen_version_from_lock_rejects_block_without_version() {
    let malformed_lock = "\
[[package]]
name = \"wasm-bindgen\"
source = \"registry+https://github.com/rust-lang/crates.io-index\"
";

    extract_wasm_bindgen_version_from_lock(malformed_lock, "malformed-lock");
}
