//! `rws_wasm_full::nav`（クライアント側ルーティング・イシュー #374）の
//! native テスト。
//!
//! `nav` モジュールの純粋層（[`ClientRoute`]/`resolve_path`/
//! `resolve_route_view_with`）は DOM 非依存のため、`wasm32` ターゲット・
//! 実 DOM を介さず native の `cargo test --workspace` から直接検証できる
//! （`wasm-full/tests/loader_csr.rs` と同じ 2 層構成方針）。配線層
//! （`start_router`、`#[cfg(target_arch = "wasm32")]`）の検証は
//! `wasm-full/tests/nav_browser.rs`（実ブラウザ）が担う。
//!
//! # 検証内容
//!
//! 1. ルート解決仕様（`docs/api/router-path-matching.md` v1 の代表ケース）:
//!    クエリ除去・末尾スラッシュ厳格一致・未登録パス `None`
//! 2. `resolve_route_view_with` が [`crate::csr`] 経由と同じ出力・タイトル
//!    契約（`server/src/ssr.rs::respond_with` と同一のリテラル）を返すこと
//! 3. loader fail-closed: 機微情報風文字列が出力へ混入しないこと
//! 4. XSS 回帰: XSS ペイロード id のノードが既定エスケープされること
//! 5. スクロール座標コーデック（[`encode_scroll_state`]/[`decode_scroll_state`]、
//!    イシュー #406）: 往復一致・不正値の fail-closed 拒否（DOM 非依存の
//!    純粋層のため、配線層の実ブラウザ検証は `wasm-full/tests/nav_browser.rs`
//!    が別途担う）

use rws_app::{demo_items, DemoItemDetailLoader, DemoItemsLoader, Item, Loader};
use rws_core::render;
use rws_wasm_full::nav::{
    decode_scroll_state, encode_scroll_state, resolve_path, resolve_route_view_with, ClientRoute,
};

/// 検証 1: `/` は `ClientRoute::List` に解決する。
#[test]
fn resolve_path_root_matches_list() {
    assert_eq!(resolve_path("/"), Some(ClientRoute::List));
}

/// 検証 1: `/items/:id` はクエリ文字列を切り落として解決する
/// （`docs/api/router-path-matching.md` v1 仕様の代表ケース）。
#[test]
fn resolve_path_strips_query_string_before_matching() {
    assert_eq!(
        resolve_path("/items/2?ref=top"),
        Some(ClientRoute::Detail("2".to_string()))
    );
}

/// 検証 1: 末尾スラッシュは正規化されず非一致となる
/// （`server/src/router.rs::trailing_slash_is_not_normalized_and_does_not_match`
/// と同じ v1 仕様の対称テスト）。
#[test]
fn resolve_path_does_not_normalize_trailing_slash() {
    assert_eq!(resolve_path("/items/1/"), None);
}

/// 検証 1: 未登録パスは `None`。
#[test]
fn resolve_path_returns_none_for_unregistered_path() {
    assert_eq!(resolve_path("/nope"), None);
}

/// 検証 1: `/items/:id` の `id` は空でない 1 セグメントのみを捕捉する。
#[test]
fn resolve_path_rejects_empty_id_segment() {
    assert_eq!(resolve_path("/items/"), None);
}

/// 検証 1: XSS ペイロード風の値もパスセグメントとして生文字列のまま捕捉する
/// （`server/src/router.rs::xss_payload_like_path_is_captured_as_raw_string`
/// と対になる回帰。URL デコード・HTML 解釈を一切行わない）。パスセグメントは
/// `/` で区切られるため、セグメント内に `/` を含まない onerror ハンドラ形式の
/// ペイロードを使う（server 側テストと同じ制約）。
#[test]
fn resolve_path_captures_xss_payload_like_id_as_raw_string() {
    let payload = "<img src=x onerror=alert(1)>";
    let path = format!("/items/{payload}");
    assert_eq!(
        resolve_path(&path),
        Some(ClientRoute::Detail(payload.to_string()))
    );
}

/// 検証 2: `resolve_route_view_with(List)` は `server/src/ssr.rs::respond_with`
/// と同じタイトル（"記事一覧"）を返し、`list_page` 直呼びと render 結果が
/// 一致する。
#[test]
fn resolve_route_view_with_list_matches_direct_call_and_title() {
    let (title, node) =
        resolve_route_view_with(&DemoItemsLoader, &DemoItemDetailLoader, &ClientRoute::List);
    assert_eq!(title, "記事一覧");
    let direct = render(&rws_app::list_page(&demo_items()));
    assert_eq!(render(&node), direct);
}

/// 検証 2: `resolve_route_view_with(Detail)` は既知 id で "記事詳細" タイトル
/// を返し、`detail_page(Some(_))` 直呼びと render 結果が一致する。
#[test]
fn resolve_route_view_with_detail_matches_direct_call_for_known_id() {
    let (title, node) = resolve_route_view_with(
        &DemoItemsLoader,
        &DemoItemDetailLoader,
        &ClientRoute::Detail("1".to_string()),
    );
    assert_eq!(title, "記事詳細");
    let expected_item = demo_items().into_iter().find(|it| it.id == "1");
    let direct = render(&rws_app::detail_page(expected_item.as_ref()));
    assert_eq!(render(&node), direct);
}

/// 検証 2: 未知 id（`Output = None`）も "記事詳細" タイトルのまま
/// `detail_page(None)` の固定文言ノードへ収束する（見つからない、を
/// `Error` として扱わない設計書 §3.3 の契約、`server/src/ssr.rs` の
/// 404 応答が本文は変えずステータスのみ変える契約と対応）。
#[test]
fn resolve_route_view_with_detail_matches_direct_call_for_unknown_id() {
    let (title, node) = resolve_route_view_with(
        &DemoItemsLoader,
        &DemoItemDetailLoader,
        &ClientRoute::Detail("does-not-exist".to_string()),
    );
    assert_eq!(title, "記事詳細");
    let direct = render(&rws_app::detail_page(None));
    assert_eq!(render(&node), direct);
}

/// 検証 4: XSS ペイロード item（id="2"）への遷移でも既定エスケープされ、
/// 生の `<script>` タグが出力に現れない。
#[test]
fn resolve_route_view_with_detail_escapes_xss_payload_title() {
    let (_, node) = resolve_route_view_with(
        &DemoItemsLoader,
        &DemoItemDetailLoader,
        &ClientRoute::Detail("2".to_string()),
    );
    let html = render(&node);
    assert!(
        !html.contains("<script>"),
        "XSS ペイロード item の title が既定エスケープされずに出力へ混入した: {html}"
    );
    assert!(
        html.contains("&lt;script&gt;"),
        "既定エスケープ済みの文字列が出力に含まれていること: {html}"
    );
}

/// 検証 3 用の失敗フィクスチャ（一覧）。
struct FailingListLoader;

impl Loader for FailingListLoader {
    type Input = ();
    type Output = Vec<Item>;
    type Error = String;

    fn load(&self, _input: &()) -> Result<Vec<Item>, String> {
        Err("secret://db-password@internal-host".to_string())
    }
}

/// 検証 3 用の失敗フィクスチャ（詳細）。
struct FailingDetailLoader;

impl Loader for FailingDetailLoader {
    type Input = String;
    type Output = Option<Item>;
    type Error = String;

    fn load(&self, _input: &String) -> Result<Option<Item>, String> {
        Err("secret://db-password@internal-host".to_string())
    }
}

/// 検証 3: loader 失敗時も機微情報風文字列は出力へ混入せず、
/// `rws_wasm_full::csr::loader_error_view` の render 結果へ収束する。
/// タイトルはルート由来（"記事一覧"）のまま変わらない（`resolve_route_view_with`
/// は `Ok`/`Err` に関わらずタイトルをルートから決定する契約、`Loader::Error`
/// 型自体の相違を吸収するため一覧・詳細は個別の失敗 loader 型を要する）。
#[test]
fn resolve_route_view_with_list_converges_to_fixed_error_view_without_leaking_error_value() {
    let (title, node) = resolve_route_view_with(
        &FailingListLoader,
        &DemoItemDetailLoader,
        &ClientRoute::List,
    );
    assert_eq!(title, "記事一覧");
    let html = render(&node);
    let expected = render(&rws_wasm_full::csr::loader_error_view());
    assert_eq!(html, expected);
    assert!(
        !html.contains("secret://db-password@internal-host"),
        "loader の Error 値（機微情報風文字列）が出力へ混入してはならない: {html}"
    );
}

/// 検証 3（詳細）: 上記と同様の fail-closed 検証を詳細ルートで固定する。
#[test]
fn resolve_route_view_with_detail_converges_to_fixed_error_view_without_leaking_error_value() {
    let (title, node) = resolve_route_view_with(
        &DemoItemsLoader,
        &FailingDetailLoader,
        &ClientRoute::Detail("1".to_string()),
    );
    assert_eq!(title, "記事詳細");
    let html = render(&node);
    let expected = render(&rws_wasm_full::csr::loader_error_view());
    assert_eq!(html, expected);
    assert!(
        !html.contains("secret://db-password@internal-host"),
        "loader の Error 値（機微情報風文字列）が出力へ混入してはならない: {html}"
    );
}

/// イシュー #403: 遷移先（Detail ルート）の解決結果ノードが
/// `data-hydrate="like"` を持つことを純粋層で固定する。`nav::wiring::render_route`
/// （wasm32 配線層、ブラウザテストは `nav_browser.rs` が担当）が
/// `rws_wasm_client::wire_hydrate_targets` で再配線する対象が実際に
/// 存在することの契約テスト（`rws_core::find_attr_values` で
/// `rws-wasm-client::HYDRATE_ATTR` と同一の属性名契約を検証する）。
#[test]
fn resolve_route_view_with_detail_node_has_hydrate_target() {
    let (_, node) = resolve_route_view_with(
        &DemoItemsLoader,
        &DemoItemDetailLoader,
        &ClientRoute::Detail("1".to_string()),
    );
    let values = rws_core::find_attr_values(&node, rws_wasm_client::HYDRATE_ATTR);
    assert_eq!(
        values,
        vec![rws_wasm_client::LIKE_HYDRATE_VALUE.to_string()],
        "Detail ルートの解決結果ノードは data-hydrate=\"like\" を含むこと（再配線対象の存在契約）"
    );
}

// ---------------------------------------------------------------------
// 検証 5: スクロール座標コーデック（イシュー #406）
// ---------------------------------------------------------------------

/// エンコード→デコードが座標を保存すること（往復一致）。
#[test]
fn scroll_state_codec_round_trips() {
    assert_eq!(
        decode_scroll_state(&encode_scroll_state(0.0, 0.0)),
        Some((0.0, 0.0))
    );
    assert_eq!(
        decode_scroll_state(&encode_scroll_state(240.0, 1980.5)),
        Some((240.0, 1980.5))
    );
}

/// history state は同一オリジンから改ざん可能な前提のため、固定プレフィックス
/// と一致しない値は fail-closed で `None` になること。
#[test]
fn decode_scroll_state_rejects_values_without_the_fixed_prefix() {
    assert_eq!(decode_scroll_state("0,0"), None);
    assert_eq!(decode_scroll_state("evil"), None);
}

/// 非数・非有限（`NaN`/`Inf`）・負値のいずれも `None` になること
/// （`Window::scroll_to_with_x_and_y` へ渡してよい値の範囲を構造的に限定する）。
#[test]
fn decode_scroll_state_rejects_non_numeric_non_finite_and_negative_values() {
    assert_eq!(decode_scroll_state("rws-scroll:a,b"), None);
    assert_eq!(decode_scroll_state("rws-scroll:NaN,0"), None);
    assert_eq!(decode_scroll_state("rws-scroll:0,-5"), None);
}

/// history state が XSS ペイロード風文字列で改ざんされていても、復号は必ず
/// 失敗し（`None`）、`Window::scroll_to_with_x_and_y`（数値専用 API）以外へ
/// 値が流出しないことを直接固定する（DOM・HTML への流入経路を持たない設計の
/// 回帰テスト）。
#[test]
fn decode_scroll_state_rejects_xss_payload_like_string() {
    assert_eq!(
        decode_scroll_state("rws-scroll:<script>alert(1)</script>,0"),
        None
    );
}
