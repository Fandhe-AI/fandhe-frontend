//! ルーティング統合テスト（TASK-7.2c、イシュー #57）。
//!
//! REQ-7（`docs/spec/04-requirements.md`）の受け入れ基準・
//! `docs/router-path-matching.md`（TASK-7.2a）の v1 仕様を、
//! `rws_server::router::Router` の公開 API（unit テストではなく外部クレート
//! と同じ利用経路）を通じて固定する。`server/src/router.rs` の unit テストと
//! 重複させず、以下の観点に絞る。
//!
//! - REQ-7 受け入れ基準の 3 ルート（`/`・`/items/:id`・`/search`）相当が
//!   公開 API 経由で解決できること
//! - `rws_server::ssr::respond`（SSR エントリ）との結合（`/` → 200、
//!   既知 `id` → 200、未知 `id` → 404）
//! - XSS 回帰: パスパラメータに XSS ペイロードを与えても `Params` は生文字列の
//!   ままであり、`respond` の 404 ボディに生の `<script>` 等が現れないこと
//!   （既定エスケープ、REQ-1）
//! - エッジケース（クエリ付きパス・末尾スラッシュ・連続スラッシュ・空パス・
//!   非 `/` 始まりパス）の非マッチ / マッチ挙動
//! - `RouterError` 全変種が公開 API 経由で再現でき、`Display` 出力が
//!   機微情報を含まないこと

use rws_server::router::{Router, RouterError};
use rws_server::ssr::respond;

/// REQ-7 受け入れ基準（PoC-3 の 3 ルート相当）が公開 API から解決できることを
/// 固定する。`/search` は `rws-app` に検索ページの凍結 API がないため
/// `respond()` へは配線されていない（`docs/router-path-matching.md` §7）。
/// ここでは router 単体としてのマッチング可否のみを確認する。
#[test]
fn resolves_req7_baseline_routes_via_public_api() {
    let router: Router<&str> = Router::new()
        .route("/", "home")
        .unwrap()
        .route("/items/:id", "item_detail")
        .unwrap()
        .route("/search", "search")
        .unwrap();

    assert_eq!(*router.resolve("/").unwrap().handler, "home");
    let item = router.resolve("/items/42").unwrap();
    assert_eq!(*item.handler, "item_detail");
    assert_eq!(item.params.get("id"), Some("42"));
    assert_eq!(*router.resolve("/search").unwrap().handler, "search");
}

/// SSR エントリ（`rws_server::ssr::respond`）との結合。ルーターが解決した
/// 結果がそのまま SSR のステータスコードへ反映されることを固定する。
#[test]
fn respond_uses_router_resolution_for_status_codes() {
    let root = respond("/").expect("\"/\" should be routed");
    assert_eq!(root.status, 200);

    let known = respond("/items/1").expect("\"/items/1\" should be routed");
    assert_eq!(known.status, 200);

    let unknown = respond("/items/does-not-exist").expect("\"/items/:id\" pattern should match");
    assert_eq!(unknown.status, 404);

    // ルーターに登録されていないパスは respond() でも None を返す
    // （router 自身が None を返すことの結合確認）。
    assert!(respond("/not-a-route").is_none());
}

/// XSS 回帰（削除・弱体化禁止）: パスパラメータに XSS ペイロードを与えても
/// `Router` は生文字列のまま `Params` へ格納する契約を公開 API 経由で固定し、
/// `respond()` の 404 ボディ（`rws-app` の既定エスケープ経由）に生の
/// ペイロード文字列が現れないことを確認する。
#[test]
fn xss_payload_in_path_param_is_not_rendered_raw_by_respond() {
    let router: Router<&str> = Router::new().route("/items/:id", "item_detail").unwrap();

    // パスセグメントは '/' で区切られるため、セグメント内に '/' を含まない
    // XSS ペイロード（onerror ハンドラ形式）を用いる（`server/src/router.rs`
    // の unit テスト `xss_payload_like_path_is_captured_as_raw_string` と
    // 同種のペイロード。`</script>` は閉じタグの '/' でセグメントが分割され
    // マッチしなくなるため使えない）。
    let payload = "<img src=x onerror=alert(1)>";
    let path = format!("/items/{payload}");
    let matched = router.resolve(&path).expect("should match");
    // router 自体は生文字列のまま返す契約（エスケープは呼び出し元の責務）。
    assert_eq!(matched.params.get("id"), Some(payload));

    // respond() は未知 id として 404 を返し、rws-app の既定エスケープ経由で
    // ボディを組み立てる。ペイロードそのものはボディに含まれないため、
    // 生の `<img` タグが混入しないことのみを確認する。
    let response = respond(&path).expect("\"/items/:id\" pattern should match");
    assert_eq!(response.status, 404);
    assert!(!response.body.contains("<img src=x onerror"));
}

/// クエリ文字列は照合前に切り落とされる（`docs/router-path-matching.md` §3）。
#[test]
fn query_string_is_ignored_when_matching() {
    let router: Router<&str> = Router::new().route("/items/:id", "item_detail").unwrap();

    let matched = router
        .resolve("/items/7?ref=list&utm=abc")
        .expect("should match ignoring query string");
    assert_eq!(matched.params.get("id"), Some("7"));
}

/// 末尾スラッシュ・連続スラッシュ・空パス・非 `/` 始まりパスはいずれも
/// 厳格一致の対象外として非マッチになる（`docs/router-path-matching.md` §3）。
#[test]
fn strict_matching_rejects_slash_variants() {
    let router: Router<&str> = Router::new()
        .route("/", "home")
        .unwrap()
        .route("/items/:id", "item_detail")
        .unwrap();

    assert!(
        router.resolve("/items/1/").is_none(),
        "末尾スラッシュは非マッチ"
    );
    assert!(
        router.resolve("/items//1").is_none(),
        "連続スラッシュは非マッチ"
    );
    assert!(router.resolve("").is_none(), "空パスは非マッチ");
    assert!(
        router.resolve("items").is_none(),
        "非 '/' 始まりパスは非マッチ"
    );
    // "/" 自体は一致するが "" とは別物であることも併せて確認する。
    assert_eq!(*router.resolve("/").unwrap().handler, "home");
}

/// `RouterError` の全変種が公開 API から再現でき、`Display` が開発者記述の
/// パターン文字列以外の機微情報（内部パス・実行時状態）を含まないことを
/// 固定する（`security.md` の機微情報露出対策）。
#[test]
fn router_error_variants_are_reachable_and_display_safely() {
    let missing_slash = Router::<&str>::new().route("items", "x").unwrap_err();
    assert_eq!(
        missing_slash,
        RouterError::MissingLeadingSlash("items".to_string())
    );
    assert_eq!(
        missing_slash.to_string(),
        "route pattern must start with '/': \"items\""
    );

    let empty_segment = Router::<&str>::new().route("/items//id", "x").unwrap_err();
    assert_eq!(
        empty_segment,
        RouterError::EmptySegment("/items//id".to_string())
    );

    let empty_param = Router::<&str>::new().route("/items/:", "x").unwrap_err();
    assert_eq!(
        empty_param,
        RouterError::EmptyParamName("/items/:".to_string())
    );

    let duplicate_param = Router::<&str>::new()
        .route("/items/:id/reviews/:id", "x")
        .unwrap_err();
    assert_eq!(
        duplicate_param,
        RouterError::DuplicateParamName {
            pattern: "/items/:id/reviews/:id".to_string(),
            name: "id".to_string(),
        }
    );
    // 機微な実行時情報（内部パス・スタックトレース等）を含まないことの確認。
    let display = duplicate_param.to_string();
    assert!(!display.contains("panic"));
    assert!(!display.to_lowercase().contains("backtrace"));
}
