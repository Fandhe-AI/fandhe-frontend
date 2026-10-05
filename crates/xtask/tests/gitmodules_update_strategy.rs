//! リポジトリ直下 `.gitmodules` の取得戦略に対する契約テスト（イシュー #3718）。
//!
//! 背景: `docs/spec` は private リポジトリを指す。cargo は git 依存の checkout 時に
//! submodule も取得するため、`update = none` を外すと認証のない環境の
//! `cargo install --git ... fandhe-frontend-docs-site` が
//! `failed to update submodule docs/spec` で失敗する。外部リポジトリが docs サイト
//! 生成器を導入できる前提を固定するのが本テストの役割である。
//!
//! 静的検証のみ（ネットワーク不使用）。匿名環境での実測は PR 記載の隔離計測が正。
//! 新しい submodule を足すと git 利用者全員の取得対象が増えるため、全 submodule に
//! `update = none` を要求して fail-closed にする。外部パーサは追加しない。

use std::collections::BTreeMap;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/xtask/ から 2 段上でワークスペースルートに到達する")
        .to_path_buf()
}

/// `.gitmodules` を行単位で読み、セクション名ごとの key/value を返す。
fn parse(text: &str) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut out: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut cur: Option<String> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with("[submodule ") && line.ends_with(']') {
            let name = line["[submodule ".len()..line.len() - 1].trim_matches('"');
            out.entry(name.to_string()).or_default();
            cur = Some(name.to_string());
        } else if let (Some(name), Some((k, v))) = (&cur, line.split_once('=')) {
            out.get_mut(name)
                .expect("section は登録済み")
                .insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    out
}

fn sections() -> BTreeMap<String, BTreeMap<String, String>> {
    let path = workspace_root().join(".gitmodules");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!(".gitmodules を読めない: {}: {e}", path.display()));
    parse(&text)
}

#[test]
fn docs_spec_submodule_is_declared() {
    let s = sections();
    let spec = s
        .get("docs/spec")
        .expect("docs/spec の submodule 宣言が必要");
    assert_eq!(spec.get("path").map(String::as_str), Some("docs/spec"));
}

#[test]
fn every_submodule_has_update_none() {
    for (name, kv) in sections() {
        assert_eq!(
            kv.get("update").map(String::as_str),
            Some("none"),
            "submodule `{name}` に update = none が無い。匿名の cargo install --git が \
             submodule 取得で失敗する（イシュー #3718）"
        );
    }
}
