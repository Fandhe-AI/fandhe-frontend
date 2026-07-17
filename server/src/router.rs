//! v1 共通コアのパスマッチングルーター（TASK-7.2b）。
//!
//! PoC-3 では axum の `Router` を直接利用していたが、本モジュールは
//! **外部クレートに一切依存しないパスマッチング実装**として `rws-server` に
//! 製品化したものである（TASK-7.2a のパスマッチング仕様が
//! `docs/router-*.md` として未確定のため、TASK-7.2 系タスク分解の受け入れ
//! 基準（REQ-7: `/`・`/items/:id`・`/search` の 3 ルート相当）に基づく
//! フォールバック設計で実装している。7.2a 確定時に差分があれば追従修正する）。
//!
//! # 呼び出し文脈
//!
//! - HTTP・HTML を一切知らない。ハンドラ型 `H` はジェネリクスとして完全に
//!   分離しており、SSR（`rws-server` の axum 統合、TASK-6.1c）・SSG（静的
//!   書き出しバイナリ）・単一バイナリ配布（TASK-9.1）のいずれの上位層からも
//!   同一の [`Router`] / [`Router::resolve`] を呼び出せることを想定する。
//! - 本モジュールの出力（[`Params`]）は生文字列のまま返す。HTML へ出力する
//!   際は呼び出し元が必ず `rws_core::text` / `rws_core::el` の attrs 経由で
//!   既定エスケープ（REQ-1）を通すこと。本モジュール自身は `format!` 等で
//!   HTML 文字列を組み立てない。
//!
//! # マッチング仕様（v1）
//!
//! - セグメント単位の完全一致。`:name` は空でない 1 セグメントを捕捉する。
//! - 登録順の先勝ち（優先度規則は v1 対象外）。
//! - `?` 以降のクエリ文字列は照合前に切り落とす。
//! - 末尾スラッシュは正規化しない厳格一致（`/items/1/` と `/items/1` は別物）。
//! - ワイルドカード（`*path`）・パーセントデコード・HTTP メソッド別
//!   ディスパッチは v1 のスコープ外（PR にスコープ外事項として記録する）。
//!
//! # セキュリティ不変条件
//!
//! - 照合は文字列比較のみでファイルシステムへ一切触れない
//!   （パストラバーサルの影響面を持たない）。
//! - 登録ルート数 × リクエストパスのセグメント数に比例する線形走査のみ
//!   （正規表現・再帰・バックトラックを一切使わない。DoS への耐性）。
//! - 不正なパターン登録は `panic!` させず `Result::Err` として返す
//!   （ライブラリコードでの panic 回避規約、`coding-rust.md`）。

use std::fmt;

/// パターン文字列をパースした 1 セグメント分の内部表現。
///
/// `route()` 登録時にのみ生成し、`resolve()` の照合ループで使う。
#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    /// 固定文字列との完全一致を要求するセグメント（例: `"items"`）。
    Static(String),
    /// `:name` 形式。空でない 1 セグメントを捕捉し `Params` へ格納する。
    Param(String),
}

/// 登録済み 1 ルート分（パース済みパターン + ハンドラ）。
struct Route<H> {
    segments: Vec<Segment>,
    handler: H,
}

/// パターン登録済みルートの集合。
///
/// 登録順に先勝ちで解決する（`resolve()` 参照）。ハンドラ型 `H` は本モジュール
/// が一切関知しない不透明な値であり、HTTP レスポンス生成等の責務は呼び出し元
/// （`rws-server` の SSR エントリ等）が担う。
pub struct Router<H> {
    routes: Vec<Route<H>>,
}

impl<H> fmt::Debug for Router<H> {
    /// ハンドラの中身は表示しない（`H` に `Debug` 境界を強制しないための
    /// 簡略表示）。テストの `unwrap_err()` 等、`Result<Self, _>` を扱う
    /// コードが `Router<H>: Debug` を要求するために用意している。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Router")
            .field("route_count", &self.routes.len())
            .finish()
    }
}

impl<H> Default for Router<H> {
    fn default() -> Self {
        Self { routes: Vec::new() }
    }
}

impl<H> Router<H> {
    /// 空のルーターを作る。
    pub fn new() -> Self {
        Self::default()
    }

    /// パターンを 1 件登録する。
    ///
    /// パターンは `/` から始まる必要があり、`:name` セグメントでパスパラメータ
    /// を宣言できる（例: `"/items/:id"`）。ビルダー形式で `?` チェーンできる
    /// よう `self` を消費して `Result<Self, _>` を返す。
    ///
    /// # Errors
    ///
    /// パターンが `/` から始まらない・空セグメントを含む（連続スラッシュ・
    /// 末尾スラッシュ）・`:` の直後にパラメータ名がない・同一パターン内で
    /// パラメータ名が重複する、のいずれかに該当する場合に
    /// [`RouterError`] を返す。`panic!` はしない。
    ///
    /// # Examples
    ///
    /// ```
    /// use rws_server::router::Router;
    ///
    /// let router: Router<&str> = Router::new()
    ///     .route("/", "home")?
    ///     .route("/items/:id", "item_detail")?
    ///     .route("/search", "search")?;
    /// # Ok::<(), rws_server::router::RouterError>(())
    /// ```
    pub fn route(mut self, pattern: &str, handler: H) -> Result<Self, RouterError> {
        let segments = parse_pattern(pattern)?;
        self.routes.push(Route { segments, handler });
        Ok(self)
    }

    /// リクエストパスを解決する。
    ///
    /// `?` 以降のクエリ文字列は照合前に切り落とす。登録順に先勝ちで走査し、
    /// 最初に一致したルートのハンドラ参照とパスパラメータを返す。一致する
    /// ルートがなければ `None`（`panic!` はしない）。
    ///
    /// # Examples
    ///
    /// ```
    /// use rws_server::router::Router;
    ///
    /// let router: Router<&str> = Router::new().route("/items/:id", "item_detail")?;
    /// let m = router.resolve("/items/42?ref=top").expect("matches");
    /// assert_eq!(*m.handler, "item_detail");
    /// assert_eq!(m.params.get("id"), Some("42"));
    /// # Ok::<(), rws_server::router::RouterError>(())
    /// ```
    pub fn resolve(&self, path: &str) -> Option<RouteMatch<'_, H>> {
        let path_without_query = match path.split_once('?') {
            Some((before, _)) => before,
            None => path,
        };
        let request_segments = split_path(path_without_query)?;

        for route in &self.routes {
            if route.segments.len() != request_segments.len() {
                continue;
            }
            if let Some(params) = match_segments(&route.segments, &request_segments) {
                return Some(RouteMatch {
                    handler: &route.handler,
                    params,
                });
            }
        }
        None
    }
}

/// ルート解決結果。一致したハンドラへの参照と抽出済みパスパラメータを返す。
pub struct RouteMatch<'a, H> {
    /// 一致したルートに登録されているハンドラへの参照。
    pub handler: &'a H,
    /// `:name` セグメントから抽出したパスパラメータ。
    pub params: Params,
}

/// `:name` セグメントから抽出したパスパラメータ。
///
/// 値は URL デコードされていない生文字列のまま保持する。HTML へ出力する際は
/// 呼び出し元が必ず `rws_core::text` / `rws_core::el` の attrs 経由で既定
/// エスケープ（REQ-1）を通すこと。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Params(Vec<(String, String)>);

impl Params {
    /// パラメータ名から値を取得する。登録されていなければ `None`。
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    /// 登録済みパラメータを `(name, value)` のイテレータとして返す。
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

/// パターン登録時の不正入力を表すエラー。
///
/// パターン文字列はフレームワーク利用者（開発者）が `route()` 呼び出し時に
/// 与えるものであり、エンドユーザー入力ではない。そのためメッセージに機微な
/// 実行時情報は含まない（該当パターン文字列そのものを添えるのみ）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouterError {
    /// パターンが `/` から始まっていない。
    MissingLeadingSlash(String),
    /// 連続スラッシュ・末尾スラッシュ等による空セグメントを含む。
    EmptySegment(String),
    /// `:` の直後にパラメータ名がない（例: `"/items/:"`）。
    EmptyParamName(String),
    /// 同一パターン内でパラメータ名が重複している。
    DuplicateParamName {
        /// 対象のパターン文字列。
        pattern: String,
        /// 重複していたパラメータ名。
        name: String,
    },
}

impl fmt::Display for RouterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouterError::MissingLeadingSlash(pattern) => {
                write!(f, "route pattern must start with '/': {pattern:?}")
            }
            RouterError::EmptySegment(pattern) => {
                write!(f, "route pattern contains an empty segment: {pattern:?}")
            }
            RouterError::EmptyParamName(pattern) => {
                write!(
                    f,
                    "route pattern has a ':' segment with no parameter name: {pattern:?}"
                )
            }
            RouterError::DuplicateParamName { pattern, name } => {
                write!(
                    f,
                    "route pattern declares parameter {name:?} more than once: {pattern:?}"
                )
            }
        }
    }
}

impl std::error::Error for RouterError {}

/// パターン文字列を [`Segment`] 列にパースする。
///
/// `route()` からのみ呼ばれる。`"/"`（ルート）は空のセグメント列を返す。
fn parse_pattern(pattern: &str) -> Result<Vec<Segment>, RouterError> {
    let rest = pattern
        .strip_prefix('/')
        .ok_or_else(|| RouterError::MissingLeadingSlash(pattern.to_string()))?;

    if rest.is_empty() {
        // "/" 単体はセグメントなしのルートパターンとして扱う。
        return Ok(Vec::new());
    }

    let mut segments = Vec::new();
    let mut seen_params: Vec<&str> = Vec::new();

    for part in rest.split('/') {
        if part.is_empty() {
            return Err(RouterError::EmptySegment(pattern.to_string()));
        }
        if let Some(name) = part.strip_prefix(':') {
            if name.is_empty() {
                return Err(RouterError::EmptyParamName(pattern.to_string()));
            }
            if seen_params.contains(&name) {
                return Err(RouterError::DuplicateParamName {
                    pattern: pattern.to_string(),
                    name: name.to_string(),
                });
            }
            seen_params.push(name);
            segments.push(Segment::Param(name.to_string()));
        } else {
            segments.push(Segment::Static(part.to_string()));
        }
    }

    Ok(segments)
}

/// リクエストパス（クエリ除去済み）をセグメント列に分解する。
///
/// パターンと異なり、リクエストパスは利用者が任意の文字列を送り得るため
/// `Result` ではなく `Option` で表現する（`/` で始まらない場合や空セグメントを
/// 含む場合は、単に一致するルートがない = `None` として扱い、`panic!` は
/// しない）。空セグメントを許容しないことで「末尾スラッシュ・連続スラッシュを
/// 含むパスはどのパターンとも一致しない」という厳格一致（v1 仕様）を実現する。
fn split_path(path: &str) -> Option<Vec<&str>> {
    let rest = path.strip_prefix('/')?;
    if rest.is_empty() {
        return Some(Vec::new());
    }
    let segments: Vec<&str> = rest.split('/').collect();
    if segments.iter().any(|s| s.is_empty()) {
        return None;
    }
    Some(segments)
}

/// パースされたパターンとリクエストパスのセグメント列を照合する。
///
/// 呼び出し元（`resolve()`）が事前に長さ一致を確認済みであることを前提とする。
fn match_segments(pattern: &[Segment], request: &[&str]) -> Option<Params> {
    let mut params = Vec::new();
    for (segment, actual) in pattern.iter().zip(request.iter()) {
        match segment {
            Segment::Static(expected) => {
                if expected != actual {
                    return None;
                }
            }
            Segment::Param(name) => {
                // ルール上パターン側の空パラメータ名は route() で拒否済みだが、
                // request 側の空セグメントは split_path で既に除去されている
                // ため、ここでの actual は常に非空。
                params.push((name.clone(), (*actual).to_string()));
            }
        }
    }
    Some(Params(params))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// REQ-7 の受け入れ基準（PoC-3 の 3 ルート相当）が解決できることを固定する。
    #[test]
    fn resolves_req7_baseline_routes() {
        let router: Router<&str> = Router::new()
            .route("/", "home")
            .unwrap()
            .route("/items/:id", "item_detail")
            .unwrap()
            .route("/search", "search")
            .unwrap();

        let home = router.resolve("/").expect("root should match");
        assert_eq!(*home.handler, "home");
        assert_eq!(home.params.get("id"), None);

        let search = router.resolve("/search").expect("search should match");
        assert_eq!(*search.handler, "search");
    }

    #[test]
    fn extracts_param_from_items_id() {
        let router: Router<&str> = Router::new().route("/items/:id", "item_detail").unwrap();

        let matched = router.resolve("/items/2").expect("should match");
        assert_eq!(*matched.handler, "item_detail");
        assert_eq!(matched.params.get("id"), Some("2"));
    }

    #[test]
    fn query_string_is_stripped_before_matching() {
        let router: Router<&str> = Router::new().route("/items/:id", "item_detail").unwrap();

        let matched = router
            .resolve("/items/2?ref=list&utm=abc")
            .expect("should match ignoring query string");
        assert_eq!(matched.params.get("id"), Some("2"));
    }

    #[test]
    fn unregistered_path_does_not_match() {
        let router: Router<&str> = Router::new()
            .route("/", "home")
            .unwrap()
            .route("/items/:id", "item_detail")
            .unwrap();

        assert!(router.resolve("/nope").is_none());
    }

    #[test]
    fn extra_trailing_segment_does_not_match() {
        let router: Router<&str> = Router::new().route("/items/:id", "item_detail").unwrap();

        assert!(router.resolve("/items/1/extra").is_none());
    }

    #[test]
    fn trailing_slash_is_not_normalized_and_does_not_match() {
        let router: Router<&str> = Router::new().route("/items/:id", "item_detail").unwrap();

        // v1 は厳格一致。"/items/1/" と "/items/1" は別物として扱う。
        assert!(router.resolve("/items/1/").is_none());
    }

    #[test]
    fn xss_payload_like_path_is_captured_as_raw_string() {
        // rws-app の demo_items()[1] と同種の XSS ペイロードをパスパラメータに
        // 見立てたテスト。router は生文字列のまま返すのみでエスケープは行わ
        // ない契約であることを固定する（既定エスケープは描画側の責務）。
        let router: Router<&str> = Router::new().route("/items/:id", "item_detail").unwrap();

        // パスセグメントは '/' で区切られるため、セグメント内に '/' を含まない
        // XSS ペイロード（onerror ハンドラ形式）を用いる。
        let payload = "<img src=x onerror=alert(1)>";
        let path = format!("/items/{payload}");
        let matched = router.resolve(&path).expect("should match");
        assert_eq!(matched.params.get("id"), Some(payload));

        // 描画側（rws-core::text）を通すと既定エスケープされることを確認し、
        // router がエスケープ責務を持たないことの実証にする。
        let escaped = rws_core::render(&rws_core::text(matched.params.get("id").unwrap()));
        assert!(!escaped.contains("<img"));
        assert!(escaped.contains("&lt;img"));
    }

    #[test]
    fn duplicate_items_first_registration_wins() {
        let router: Router<&str> = Router::new()
            .route("/items/:id", "first")
            .unwrap()
            .route("/items/:id", "second")
            .unwrap();

        let matched = router.resolve("/items/9").expect("should match");
        assert_eq!(*matched.handler, "first");
    }

    #[test]
    fn rejects_pattern_without_leading_slash() {
        let router: Router<&str> = Router::new();
        let err = router.route("items", "x").unwrap_err();
        assert_eq!(err, RouterError::MissingLeadingSlash("items".to_string()));
    }

    #[test]
    fn rejects_pattern_with_empty_segment() {
        let router: Router<&str> = Router::new();
        let err = router.route("/items//id", "x").unwrap_err();
        assert_eq!(err, RouterError::EmptySegment("/items//id".to_string()));
    }

    #[test]
    fn rejects_pattern_with_empty_param_name() {
        let router: Router<&str> = Router::new();
        let err = router.route("/items/:", "x").unwrap_err();
        assert_eq!(err, RouterError::EmptyParamName("/items/:".to_string()));
    }

    #[test]
    fn rejects_pattern_with_duplicate_param_name() {
        let router: Router<&str> = Router::new();
        let err = router.route("/items/:id/reviews/:id", "x").unwrap_err();
        assert_eq!(
            err,
            RouterError::DuplicateParamName {
                pattern: "/items/:id/reviews/:id".to_string(),
                name: "id".to_string(),
            }
        );
    }

    /// rws-app の Item / demo_items() を実データに見立て、router が抽出した
    /// id で一覧から該当データを引けることを確認する（rws-server が SSR で
    /// 行う想定の一連の流れの縮小版）。
    #[test]
    fn resolved_param_can_look_up_rws_app_item() {
        let router: Router<&str> = Router::new().route("/items/:id", "item_detail").unwrap();
        let items = rws_app::demo_items();

        let matched = router.resolve("/items/2").expect("should match");
        let found = items
            .iter()
            .find(|item| Some(item.id.as_str()) == matched.params.get("id"));

        assert!(found.is_some());
    }
}
