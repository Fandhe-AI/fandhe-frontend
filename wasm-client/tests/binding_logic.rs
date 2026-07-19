//! 束縛点ベースの最小更新（イシュー #343）の native テスト。
//!
//! 1. ソースガード（受け入れ条件 2 のソースレベル固定）: `binding.rs` /
//!    `binding_dom.rs` が `set_inner_html` / `insert_adjacent_html` /
//!    `raw_html` を含まないことを固定する。`binding_dom.rs`（wasm32 専用）は
//!    native ビルドではコンパイル対象外のため、`include_str!` によるソース
//!    文字列走査で browser-test 未実行環境でも回帰を検出できるようにする
//!    （`wasm-client` 既存の 2 層構成と同じ「native で常時実行されるソース
//!    ガード」パターン）。
//! 2. `wasm-client/src/binding.rs` のパース・トークン化ロジックの追加網羅
//!    （`src/binding.rs` 内 `#[cfg(test)]` の単体テストを補完する結合的な
//!    確認）。

const BINDING_RS: &str = include_str!("../src/binding.rs");
const BINDING_DOM_RS: &str = include_str!("../src/binding_dom.rs");

/// DOM への HTML 挿入は `rws_core::render` の出力のみを経由する不変条件
/// （`lib.rs` クレート docs 不変条件 1・2・4）が、束縛点ベースの最小更新
/// 経路にも及んでいることをソースレベルで固定する。
///
/// 呼び出し構文（`.set_inner_html(` 等、末尾に `(` を伴う実呼び出しの形）で
/// 走査する。モジュール docs（`//!`）が禁止 API 名そのものを不変条件の説明
/// 文中に含む（本ファイルの `binding_dom.rs` doc コメントも含む）ため、
/// 裸の API 名の部分文字列一致では doc コメント中の言及を偽陽性として
/// 検出してしまう。呼び出し構文まで見ることで「実際に呼んでいるか」だけを
/// 判定する。
#[test]
fn binding_sources_do_not_contain_forbidden_dom_rebuild_apis() {
    for forbidden_call in [
        "set_inner_html(",
        ".set_inner_html(",
        "insert_adjacent_html(",
        "raw_html(",
    ] {
        assert!(
            !BINDING_RS.contains(forbidden_call),
            "wasm-client/src/binding.rs は `{forbidden_call}` 呼び出しを含まないこと（受け入れ条件 2 のソースガード）"
        );
        assert!(
            !BINDING_DOM_RS.contains(forbidden_call),
            "wasm-client/src/binding_dom.rs は `{forbidden_call}` 呼び出しを含まないこと（受け入れ条件 2 のソースガード）"
        );
    }
}

/// `binding_dom.rs` のテキスト更新経路が `set_text_content` を使っている
/// ことを合わせて固定する（受け入れ条件 2 の肯定的側面）。
#[test]
fn binding_dom_uses_set_text_content_for_text_updates() {
    assert!(
        BINDING_DOM_RS.contains("set_text_content"),
        "wasm-client/src/binding_dom.rs はテキスト更新に set_text_content を使うこと"
    );
}

use rws_wasm_client::{element_binding_specs, parse_binding_tokens, BindingKind, BindingSpec};

#[test]
fn parse_binding_tokens_is_reexported_and_behaves_as_documented() {
    assert_eq!(
        parse_binding_tokens("aria-pressed:liked disabled:busy"),
        vec![
            ("aria-pressed".to_string(), "liked".to_string()),
            ("disabled".to_string(), "busy".to_string()),
        ]
    );
    assert_eq!(parse_binding_tokens("onclick:draft"), Vec::new());
}

#[test]
fn element_binding_specs_is_reexported_and_orders_text_then_attr_then_class() {
    let specs = element_binding_specs(
        Some("counter"),
        Some("aria-pressed:liked"),
        Some("liked:liked"),
    );
    assert_eq!(
        specs,
        vec![
            BindingSpec {
                field: "counter".to_string(),
                kind: BindingKind::Text,
            },
            BindingSpec {
                field: "liked".to_string(),
                kind: BindingKind::Attr("aria-pressed".to_string()),
            },
            BindingSpec {
                field: "liked".to_string(),
                kind: BindingKind::Class("liked".to_string()),
            },
        ]
    );
}
