//! ルーティング統合テスト（TASK-7.2c、イシュー #57）。
//!
//! REQ-7（`docs/spec/04-requirements.md`）の受け入れ基準・
//! `docs/api/router-path-matching.md`（TASK-7.2a）の v1 仕様を、
//! `fandhe_frontend_server::router::Router` の公開 API（unit テストではなく外部クレート
//! と同じ利用経路）を通じて固定する。`server/src/router.rs` の unit テストと
//! 重複させず、以下の観点に絞る。
//!
//! - REQ-7 受け入れ基準の 3 ルート（`/`・`/items/:id`・`/search`）相当が
//!   公開 API 経由で解決できること
//! - `fandhe_frontend_server::ssr::respond`（SSR エントリ）との結合（`/` → 200、
//!   既知 `id` → 200、未知 `id` → 404）
//! - XSS 回帰: パスパラメータに XSS ペイロードを与えても `Params` は生文字列の
//!   ままであること、および実際にルーティングされたパラメータが描画される
//!   経路（`respond` の 200 分岐、`fandhe_frontend_app::demo_items()[1]` の XSS ペイロード
//!   title）で生の `<script>` 等が現れないこと（既定エスケープ、REQ-1）
//! - エッジケース（クエリ付きパス・末尾スラッシュ・連続スラッシュ・空パス・
//!   非 `/` 始まりパス）の非マッチ / マッチ挙動
//! - `RouterError` 全変種が公開 API 経由で再現でき、`Display` 出力が
//!   機微情報を含まないこと
//!
//! 以下はイシュー #57 で `docs/api/router-path-matching.md` §3 の仕様表と
//! 既存カバレッジ（PR #239）を突き合わせて未固定と判明した観点を補う。
//!
//! - 「優先度規則なし・登録順の先勝ち」が、静的セグメントとパラメータが
//!   競合するパターン間（`/items/:id` と `/items/new`）でも成り立つこと
//!   （静的セグメント優先という暗黙規則が存在しないことの回帰）
//! - パーセントエンコードされたセグメントがデコードされず生文字列のまま
//!   保持されること（パストラバーサル再導入防止の回帰）
//! - 複数パラメータルートの解決と `Params::iter()` が公開 API 経由でも
//!   成り立つこと
//! - クエリ文字列のエッジケース（ルート + クエリ・空クエリ）

use fandhe_frontend_server::router::{Router, RouterError};
use fandhe_frontend_server::ssr::respond;

/// REQ-7 受け入れ基準（PoC-3 の 3 ルート相当）が公開 API から解決できることを
/// 固定する。`/search` は `fandhe-frontend-app` に検索ページの凍結 API がないため
/// `respond()` へは配線されていない（`docs/api/router-path-matching.md` §7）。
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

/// SSR エントリ（`fandhe_frontend_server::ssr::respond`）との結合。ルーターが解決した
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
/// `Router` は生文字列のまま `Params` へ格納する契約を公開 API 経由で固定する
/// （router 自体はエスケープを行わない契約）。
#[test]
fn xss_payload_in_path_param_is_captured_as_raw_string_by_router() {
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
}

/// XSS 回帰（削除・弱体化禁止）: `/items/:id` にルーティングされた
/// パスパラメータが実際に既知アイテムとして解決・描画される経路
/// （`fandhe_frontend_app::demo_items()[1]`, `id == "2"`、title に XSS ペイロードを含む）で、
/// `respond()` の 200 ボディが `fandhe-frontend-app` の既定エスケープを経由し生の
/// `<script>` タグを含まないことを確認する。
///
/// 前段の `xss_payload_in_path_param_is_captured_as_raw_string_by_router` は
/// router 自体が生文字列を保持する契約のみを固定するのに対し、本テストは
/// ルーティングされたパスパラメータが実際にレンダリングされる唯一の経路
/// （`respond()` の 200 分岐）でエスケープが機能することを検証する
/// （404 分岐は `detail_page(None)` の固定文言のみでパスパラメータを
/// 一切含まないため、エスケープ検証としては空虚になる。Bugbot 指摘対応）。
#[test]
fn respond_escapes_xss_payload_carried_by_matched_route_param() {
    let known_xss_item = respond("/items/2").expect("\"/items/2\" should match a known item");
    assert_eq!(known_xss_item.status, 200);
    assert!(!known_xss_item.body.contains("<script>alert"));
    assert!(known_xss_item.body.contains("&lt;script&gt;alert"));
}

/// クエリ文字列は照合前に切り落とされる（`docs/api/router-path-matching.md` §3）。
#[test]
fn query_string_is_ignored_when_matching() {
    let router: Router<&str> = Router::new().route("/items/:id", "item_detail").unwrap();

    let matched = router
        .resolve("/items/7?ref=list&utm=abc")
        .expect("should match ignoring query string");
    assert_eq!(matched.params.get("id"), Some("7"));
}

/// 末尾スラッシュ・連続スラッシュ・空パス・非 `/` 始まりパスはいずれも
/// 厳格一致の対象外として非マッチになる（`docs/api/router-path-matching.md` §3）。
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

/// 登録順の先勝ち（`docs/api/router-path-matching.md` §3）は、静的セグメントと
/// パラメータが競合するパターン間でも成り立つことを固定する。「静的セグメント
/// を優先する」という暗黙の優先度規則は v1 に存在しないため、後から登録した
/// 静的パターンが先に登録したパラメータパターンを上書きすることはない。
#[test]
fn registration_order_wins_over_static_vs_param_ambiguity() {
    // パラメータパターンを先に登録した場合、"/items/new" もパラメータ側が勝つ。
    let param_first: Router<&str> = Router::new()
        .route("/items/:id", "item_detail")
        .unwrap()
        .route("/items/new", "item_new")
        .unwrap();
    let matched = param_first.resolve("/items/new").unwrap();
    assert_eq!(*matched.handler, "item_detail");
    assert_eq!(matched.params.get("id"), Some("new"));

    // 登録順を逆にすると、静的パターンを先に登録した側が勝つ。
    let static_first: Router<&str> = Router::new()
        .route("/items/new", "item_new")
        .unwrap()
        .route("/items/:id", "item_detail")
        .unwrap();
    let matched = static_first.resolve("/items/new").unwrap();
    assert_eq!(*matched.handler, "item_new");
}

/// パーセントエンコードされたセグメント（`%2F`・`%2e%2e%2f` 等）はデコードせず
/// 生文字列のまま `Params` へ格納されることを固定する
/// （`docs/api/router-path-matching.md` §3・§5、パストラバーサル再導入防止の回帰）。
/// デコードするとセグメント数が変化しパストラバーサルの面が再導入され得るため、
/// router が一切デコードしないことをここで固定する。
#[test]
fn percent_encoded_segments_are_not_decoded() {
    let router: Router<&str> = Router::new().route("/items/:id", "item_detail").unwrap();

    let matched = router
        .resolve("/items/%2e%2e%2fsecret")
        .expect("should match as a single opaque segment");
    assert_eq!(matched.params.get("id"), Some("%2e%2e%2fsecret"));

    let matched = router
        .resolve("/items/a%2Fb")
        .expect("%2F must not be decoded into an actual '/' separator");
    assert_eq!(matched.params.get("id"), Some("a%2Fb"));
}

/// 複数パラメータルートの解決と `Params::iter()` の全ペア列挙を、
/// unit テスト（`server/src/router.rs`）と重複させず公開 API 経由でも固定する。
#[test]
fn multi_param_route_and_params_iter_via_public_api() {
    let router: Router<&str> = Router::new()
        .route("/items/:id/reviews/:review_id", "review_detail")
        .unwrap();

    let matched = router.resolve("/items/2/reviews/9").expect("should match");
    assert_eq!(*matched.handler, "review_detail");
    assert_eq!(matched.params.get("id"), Some("2"));
    assert_eq!(matched.params.get("review_id"), Some("9"));

    let pairs: Vec<(&str, &str)> = matched.params.iter().collect();
    assert_eq!(pairs, vec![("id", "2"), ("review_id", "9")]);
}

/// クエリ文字列のエッジケース（`docs/api/router-path-matching.md` §3）:
/// ルートパス自身にクエリが付く場合、および空クエリ（`?` のみで値なし）の
/// いずれも `?` 以降が切り落とされてマッチすることを固定する。
#[test]
fn query_string_edge_cases() {
    let router: Router<&str> = Router::new()
        .route("/", "home")
        .unwrap()
        .route("/items/:id", "item_detail")
        .unwrap();

    let root_with_query = router
        .resolve("/?ref=x")
        .expect("root path with query string should match");
    assert_eq!(*root_with_query.handler, "home");

    let empty_query = router
        .resolve("/items/1?")
        .expect("trailing '?' with no query value should still match");
    assert_eq!(*empty_query.handler, "item_detail");
    assert_eq!(empty_query.params.get("id"), Some("1"));
}
