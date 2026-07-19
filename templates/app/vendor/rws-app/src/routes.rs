//! server / client 単一定義からのルート生成（共有機構、イシュー #407）。
//!
//! # 背景・位置付け
//!
//! 従来、ルート定義は `server/src/ssr.rs`（`PageRoute` enum + `Router` 登録 +
//! ページタイトルリテラル）と `wasm-full/src/nav.rs`（`ClientRoute` enum +
//! 独自セグメント一致 + ページタイトルリテラル）に二重定義されており、
//! `wasm-full/tests/route_sync_static.rs`（静的ソース走査）によるドリフト
//! **検知**でのみ同期を担保していた（#374・PR #383 の申し送り）。
//!
//! 本モジュールは `rws-app`（`server`・`wasm-full` の双方から依存可能な
//! 唯一の層、`structure.toml` の `allowed_dependents` 参照）にルート表
//! （パスパターン + ハンドラ + ページタイトル）を**単一定義**し、
//! `server/src/ssr.rs`（SSR）・`wasm-full/src/nav.rs`（CSR）の双方が本モジュールの
//! [`resolve`] / [`title`] を呼ぶことで、パターンリテラル・タイトルリテラルの
//! 再定義を構造的に排除する。マッチングエンジンも [`crate::router::Router`]
//! （同じくイシュー #407 で `rws-server` から移設）を共有するため、
//! パスマッチング**意味論**（末尾スラッシュ・空セグメント・クエリ除去等）の
//! ドリフトも構造的に消滅する（`docs/design/route-definition-sharing.md` 案 B-1）。
//!
//! # `fw structure` の `rws-router-v1` 抽出器との関係
//!
//! `cli/src/routes.rs` の抽出器は `[routing] definition_dir` 配下の `.rs`
//! ファイルを文字列走査し `.route("<literal>", handler)` を抽出する
//! （AST 不使用・正規表現不使用）。本モジュールのルート表は同じ
//! `.route("<pattern>", AppRoute::Variant)` ビルダー DSL のまま [`crate::router::Router`]
//! へ登録するため、`structure.toml` の `definition_dir` を `"app"` へ変更する
//! だけで抽出器は無改修のまま追従できる（`docs/design/route-definition-sharing.md`
//! §4・採用判断基準 2）。
//!
//! # セキュリティ不変条件
//!
//! - [`resolve`] が返す `id`（`ResolvedRoute::Detail` の捕捉値）は
//!   [`crate::router::Params`] と同じく生文字列のままであり、HTML へ出力する
//!   際は呼び出し元（`server/src/ssr.rs`・`wasm-full/src/csr.rs` 経由）が必ず
//!   `rws_core::text` / `rws_core::el` の attrs 経由で既定エスケープ（REQ-1）を
//!   通すこと。本モジュール自身は loader への入力としてのみ `id` を渡し、
//!   HTML 文字列を一切組み立てない。
//! - [`title`] は固定 `&'static str` のみを返し、リクエスト由来の文字列を
//!   一切含まない。

use crate::router::Router;
use std::sync::OnceLock;

/// ルート表に登録するページ種別（`server/src/ssr.rs` の旧 `PageRoute`・
/// `wasm-full/src/nav.rs` の旧 `ClientRoute` を統合した単一定義）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppRoute {
    /// `/` — 一覧画面。
    List,
    /// `/items/:id` — 詳細画面。
    Detail,
}

/// [`resolve`] の返り値。マッチしたルート種別と（`Detail` の場合のみ）
/// 捕捉した `id` を保持する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRoute {
    /// マッチしたルート種別。
    pub route: AppRoute,
    /// `Detail` ルートで捕捉した `id`（生文字列、未エスケープ）。`List` では
    /// 常に `None`。
    pub id: Option<String>,
}

/// `/`・`/items/:id` を登録した [`Router`] を構築する。
///
/// パターン文字列は開発者がハードコードした定数であり、エンドユーザー入力
/// ではないため `expect` してよい（`coding-rust.md` のエラー処理規約は
/// エンドユーザー入力由来の失敗を panic させないことを求めるものであり、
/// コンパイル時定数の妥当性はこの限りでない）。`/search` は本クレートの
/// 凍結 API に search ページが存在しないため本 v1 では登録しない
/// （スコープ外、`docs/api/app-api.md` 追記）。
fn build_router() -> Router<AppRoute> {
    Router::new()
        .route("/", AppRoute::List)
        .expect("static pattern \"/\" is valid")
        .route("/items/:id", AppRoute::Detail)
        .expect("static pattern \"/items/:id\" is valid")
}

/// `build_router()` の結果をプロセス生存期間中 1 回だけ構築してキャッシュする。
/// ルート定義は固定（開発者がハードコードしたパターンのみ）であり実行時に
/// 変化しないため、`OnceLock`（`std` のみ・追加依存なし）で使い回す
/// （`server/src/ssr.rs` の従前実装を踏襲）。
fn router() -> &'static Router<AppRoute> {
    static ROUTER: OnceLock<Router<AppRoute>> = OnceLock::new();
    ROUTER.get_or_init(build_router)
}

/// リクエストパスを解決する。`server/src/ssr.rs`（SSR）・`wasm-full/src/nav.rs`
/// （CSR）の双方が本関数を呼ぶ（`docs/design/route-definition-sharing.md`
/// 案 B-1、単一定義の強制）。
///
/// クエリ文字列除去・末尾スラッシュ厳格一致・空セグメント拒否等の意味論は
/// [`crate::router::Router::resolve`] に委譲する（`docs/api/router-path-matching.md`
/// v1 仕様どおり）。一致しないパスは `None`（呼び出し側がフォールバック挙動
/// ——server は 404 応答、wasm-full はブラウザ既定遷移への委譲——を決める）。
pub fn resolve(path: &str) -> Option<ResolvedRoute> {
    let route_match = router().resolve(path)?;
    let id = match route_match.handler {
        AppRoute::List => None,
        AppRoute::Detail => Some(route_match.params.get("id")?.to_string()),
    };
    Some(ResolvedRoute {
        route: *route_match.handler,
        id,
    })
}

/// ルート種別からページタイトル（`<title>` 相当）を返す。SSR 出力
/// （`server/src/ssr.rs::respond_with` が `page_shell` へ渡すタイトル）と
/// クライアント遷移（`wasm-full/src/nav.rs::resolve_route_view_with` が
/// `document.title` へ設定する値）が単一定義から常に一致することを保証する
/// （受け入れ条件「三モード整合」の一部、リテラルの二重管理を排除）。
pub fn title(route: AppRoute) -> &'static str {
    match route {
        AppRoute::List => "記事一覧",
        AppRoute::Detail => "記事詳細",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_list_route() {
        let resolved = resolve("/").expect("\"/\" should match");
        assert_eq!(resolved.route, AppRoute::List);
        assert_eq!(resolved.id, None);
    }

    #[test]
    fn resolves_detail_route_and_captures_id() {
        let resolved = resolve("/items/42").expect("should match");
        assert_eq!(resolved.route, AppRoute::Detail);
        assert_eq!(resolved.id, Some("42".to_string()));
    }

    /// v1 仕様（`docs/api/router-path-matching.md`）: クエリ文字列は照合前に
    /// 切り落とす。
    #[test]
    fn query_string_is_stripped_before_matching() {
        let resolved = resolve("/items/2?ref=top").expect("should match ignoring query");
        assert_eq!(resolved.id, Some("2".to_string()));
    }

    /// v1 仕様: 末尾スラッシュは正規化しない厳格一致。
    #[test]
    fn trailing_slash_does_not_match() {
        assert_eq!(resolve("/items/1/"), None);
    }

    /// v1 仕様: 連続スラッシュ（空セグメント）は一致しない。
    #[test]
    fn empty_id_segment_does_not_match() {
        assert_eq!(resolve("/items/"), None);
    }

    /// v1 仕様: 非 `/` 始まりのパスは一致しない。
    #[test]
    fn non_root_relative_path_does_not_match() {
        assert_eq!(resolve("items/1"), None);
    }

    #[test]
    fn unregistered_path_does_not_match() {
        assert_eq!(resolve("/no-such-page"), None);
    }

    /// XSS ペイロード風パスセグメントも捕捉値として生文字列のまま返る
    /// （エスケープは描画側の責務、`crate::router` の契約を継承）ことを固定する。
    #[test]
    fn xss_payload_like_id_is_captured_as_raw_string() {
        let payload = "<img src=x onerror=alert(1)>";
        let path = format!("/items/{payload}");
        let resolved = resolve(&path).expect("should match");
        assert_eq!(resolved.id, Some(payload.to_string()));
    }

    #[test]
    fn titles_are_distinct_and_fixed() {
        assert_eq!(title(AppRoute::List), "記事一覧");
        assert_eq!(title(AppRoute::Detail), "記事詳細");
    }
}
