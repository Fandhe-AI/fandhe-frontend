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

/// `Cargo.lock` から `[[package]] name = "wasm-bindgen"` ブロックの
/// `version` を厳密抽出する。
///
/// `wasm-bindgen-backend` / `wasm-bindgen-macro` 等、名前が前方一致する
/// 別パッケージのブロックを誤って拾わないよう、`name = "wasm-bindgen"` を
/// 完全一致（引用符込み）で探し、直後の `version = "..."` 行を読む。
fn cargo_lock_wasm_bindgen_version() -> String {
    let contents = read_workspace_file("Cargo.lock");
    let lines: Vec<&str> = contents.lines().collect();
    let name_line_index = lines
        .iter()
        .position(|line| line.trim() == "name = \"wasm-bindgen\"")
        .expect("Cargo.lock に [[package]] name = \"wasm-bindgen\" ブロックが見つからない");

    let version_line = lines
        .get(name_line_index + 1)
        .expect("name = \"wasm-bindgen\" の直後に行が存在する")
        .trim();

    version_line
        .strip_prefix("version = \"")
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or_else(|| {
            panic!(
                "Cargo.lock の wasm-bindgen ブロックで name の直後が \
                 version 行になっていない（Cargo.lock のフォーマット変更 \
                 の可能性）: {version_line}"
            )
        })
        .to_owned()
}

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
/// 現行既知値の出所: PR #1214（イシュー #450）の aarch64 実機 Docker
/// ビルドで `sha256sum -c` が実際に通過した実績
/// （`docs/reports/docker-wasm-rebuild-acceptance-report.md` §5a）。
#[test]
fn dockerfile_pins_known_wasm_bindgen_sha256_for_aarch64_archive() {
    const KNOWN_VERSION: &str = "0.2.126";
    const KNOWN_AARCH64_SHA256: &str =
        "2245120254a9f6c9a9adf3601f3d52bb31309219e9ceab7696e74e24885c440a";

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
