//! WASM ビルドステージ（TASK-10.2b、イシュー #110、`dist-server/build.rs`）の
//! 配信検証。
//!
//! `build.rs` は WASM ステージが実際に埋め込みテーブルへ合流したときのみ
//! `wasm_assets_embedded` cfg を有効にする（`FANDHE_FRONTEND_WASM_BUILD=0` でオプトアウト
//! した場合や、wasm ツールチェーン不在でビルド自体が失敗する経路では
//! 立たない）。本ファイル全体をこの cfg でゲートすることで、
//! - オプトアウトしたジョブ（例: forbid-unsafe。self-hosted で `RUSTFLAGS`
//!   を設定するため WASM ステージ自体を無効化する運用、`.github/workflows/ci.yml`
//!   参照）では本テストは存在しないもの（空バイナリ）として扱われ
//!   `cargo test --workspace` を壊さない
//! - WASM ステージを実行するジョブ（`.github/workflows/ci.yml` の `test`）では
//!   本テストが必ずコンパイル・実行され、REQ-10 条件 3（単一 `cargo build` で
//!   ネイティブ + WASM 双方の成果物が生成される）を静かにスキップせず固定する
//!
//! （`.claude/rules/coding-rust.md`「テストの `#[ignore]` 追加でごまかさない」
//! に対応する、実行時スキップではなくコンパイル時ゲートの選択）。
#![cfg(wasm_assets_embedded)]

use fandhe_frontend_dist_server::routes::route_request;

#[test]
fn wasm_bindgen_js_glue_is_served_with_javascript_content_type() {
    let response = route_request("/static/wasm/fandhe_frontend_wasm_full.js");
    assert_eq!(response.status, 200);
    assert_eq!(response.content_type, "text/javascript; charset=utf-8");
    assert!(!response.body.is_empty());
}

#[test]
fn wasm_binary_is_served_with_wasm_content_type() {
    let response = route_request("/static/wasm/fandhe_frontend_wasm_full_bg.wasm");
    assert_eq!(response.status, 200);
    assert_eq!(response.content_type, "application/wasm");
    assert!(!response.body.is_empty());
    // WASM バイナリのマジックナンバー（`\0asm`）を確認し、空ファイル・破損
    // ファイルの埋め込みを検知する。
    assert_eq!(&response.body[..4], b"\0asm");
}

#[test]
fn unknown_wasm_path_still_returns_404() {
    // 既存のパストラバーサル・未知パス防御を WASM 資産追加後も維持することを
    // 固定する回帰テスト。
    assert_eq!(
        route_request("/static/wasm/does-not-exist.wasm").status,
        404
    );
}

/// WASM バイナリ形式（LEB128 セクション列）を std のみで走査し、custom
/// section の名前一覧を返す。TOML パーサ等を追加しない REQ-3 方針
/// （`build.rs::expected_wasm_bindgen_version` と同じ「固定フォーマットを
/// 自前で読む」選択）を踏襲する。
///
/// フォーマット: 8 バイトヘッダ（`\0asm` + version）→ セクションの繰り返し
/// （`id`(1B) + `size`(ULEB128) + `size` バイトの内容）。`id == 0` は custom
/// section で、内容の先頭が `name`(ULEB128 長 + UTF-8 バイト列)。
fn custom_section_names(bytes: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    // 8 バイトヘッダ（マジックナンバー 4B + バージョン 4B）をスキップする。
    let mut pos = 8usize;

    fn read_uleb128(bytes: &[u8], pos: &mut usize) -> Option<u64> {
        let mut result: u64 = 0;
        let mut shift = 0u32;
        loop {
            let byte = *bytes.get(*pos)?;
            *pos += 1;
            result |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Some(result);
            }
            shift += 7;
            if shift >= 64 {
                return None;
            }
        }
    }

    while pos < bytes.len() {
        let Some(&section_id) = bytes.get(pos) else {
            break;
        };
        pos += 1;
        let Some(section_size) = read_uleb128(bytes, &mut pos) else {
            break;
        };
        let section_size = section_size as usize;
        let Some(section_end) = pos.checked_add(section_size) else {
            break;
        };
        if section_end > bytes.len() {
            break;
        }

        if section_id == 0 {
            let mut name_pos = pos;
            if let Some(name_len) = read_uleb128(bytes, &mut name_pos) {
                let name_len = name_len as usize;
                if let Some(name_end) = name_pos.checked_add(name_len) {
                    if name_end <= section_end {
                        if let Ok(name) = std::str::from_utf8(&bytes[name_pos..name_end]) {
                            names.push(name.to_string());
                        }
                    }
                }
            }
        }

        pos = section_end;
    }

    names
}

#[test]
fn served_wasm_binary_has_no_name_or_producers_custom_sections() {
    // イシュー #1971 Step A（`wasm-bindgen --remove-name-section
    // --remove-producers-section`）の end-to-end 検証。`build.rs` が
    // 実際に配信対象へ埋め込む `_bg.wasm` から、除去対象の custom section
    // （`name`・`producers`）が消えていることを固定する。
    let response = route_request("/static/wasm/fandhe_frontend_wasm_full_bg.wasm");
    assert_eq!(response.status, 200);

    let names = custom_section_names(&response.body);
    assert!(
        !names.iter().any(|n| n == "name"),
        "expected no `name` custom section, found sections: {names:?}"
    );
    assert!(
        !names.iter().any(|n| n == "producers"),
        "expected no `producers` custom section, found sections: {names:?}"
    );
}
