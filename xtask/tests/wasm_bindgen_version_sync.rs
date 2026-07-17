//! `Dockerfile` / `.github/workflows/ci.yml` に散在する wasm-bindgen-cli の
//! 固定バージョン・SHA256 チェックサムと、`Cargo.lock` が解決した
//! `wasm-bindgen` クレートのバージョンとの同期ドリフトを検出する回帰テスト。
//!
//! TASK-10.3c（issue #117、`docs/docker-wasm-build-stage.md` §6 検証観点 2）の
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

use std::path::PathBuf;

/// workspace ルート（`xtask/` の親ディレクトリ）の絶対パスを返す。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask/ には親ディレクトリ（workspace ルート）が存在する")
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

/// `key="value"` 形式の代入から、キーの直後の `"..."` で囲まれた値のみを
/// 取り出す。
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

/// 指定した内容から `WASM_BINDGEN_VERSION="..."` の全出現を抽出する。
///
/// `Dockerfile`（x86_64/aarch64 の 2 分岐）・`ci.yml`（複数ジョブ）のいずれも
/// この形式で固定値を宣言している前提（両ファイルの現行実装を参照）。
fn extract_wasm_bindgen_versions(contents: &str) -> Vec<String> {
    contents
        .lines()
        .filter_map(|line| extract_quoted_assignment(line, "WASM_BINDGEN_VERSION="))
        .map(str::to_owned)
        .collect()
}

/// `WASM_BINDGEN_SHA256="..."` の全出現を抽出する。
fn extract_wasm_bindgen_sha256s(contents: &str) -> Vec<String> {
    contents
        .lines()
        .filter_map(|line| extract_quoted_assignment(line, "WASM_BINDGEN_SHA256="))
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
