//! JSON-LD（構造化データ）を安全に `<script type="application/ld+json">`
//! として埋め込む正規 API（イシュー #1117）。
//!
//! # 背景
//!
//! `<script>` 要素の内容は HTML パーサ上「raw text」であり実体参照
//! （`&quot;` 等）は復号されない。そのため本クレートの既定エスケープ
//! （[`crate::escape_html`]、不変条件 1）をそのまま JSON 文字列へ適用すると
//! `"` が `&quot;` に化けて JSON として壊れる。かといって [`crate::raw_html`]
//! をそのまま利用者に使わせると、既定エスケープ迂回の監査対象
//! （`clippy::disallowed_methods`、`docs/policy/raw-html-review-gate.md`）を
//! 利用者コード側へ転嫁してしまい、`json_ld` のたびに個別の `#[expect]` を
//! 書かせることになる。本モジュールはこの迂回を **フレームワーク内部で
//! 一度だけ審査済みの安全な wrapper** として提供し、利用者コードには
//! 通常の関数呼び出しとして公開する。
//!
//! # 設計: 中立化（HTML 活性文字の `\uXXXX` エスケープ）
//!
//! [`json_ld`] はシリアライズ済み JSON 文字列を受け取り、`<` `>` `&` と
//! U+2028 / U+2029（JS 行終端子。将来 JSON-LD が JS コンテキストへ流用
//! される変更が入っても安全なように多層防御として同時に処理する）を
//! `\uXXXX` 形式へ全置換してから [`crate::raw_html`] へ渡す。`"` `\` や
//! 制御文字には触れない（JSON 構文そのものであり、文字列リテラル向けの
//! エスケープを文書全体へ適用すると JSON が壊れる）。
//!
//! `crates/docs-site/src/search_index.rs::escape_json_string` と処理対象の
//! 文字集合は同じだが、あちらは「文字列リテラル 1 個」を組み立てる関数
//! （`"` `\` も含めて自前シリアライズする）であるのに対し、本モジュールは
//! **シリアライズ済みの JSON 文書全体**を受け取って HTML 活性文字だけを
//! 中立化する点が異なる。
//!
//! ## 意味保存の根拠
//!
//! 妥当な JSON 文書において `<` `>` `&` および U+2028/U+2029 は文字列
//! リテラルの内部にしか出現し得ない（JSON 文法上、リテラル外に許容される
//! 文字は構造文字 `{} [] : ,`・数値・`true`/`false`/`null`・空白
//! （space/tab/CR/LF）のみ）。したがって上記の全置換は `JSON.parse` の
//! 結果を変えない。
//!
//! ## JSON バリデーションを行わない理由（fail-safe 設計）
//!
//! [`json_ld`] は入力が妥当な JSON かどうかを検証しない。不正な入力
//! （非 JSON文字列）が渡された場合でも、中立化後の出力には `<` `>` `&`
//! が一切残らないため、`<script>` 要素からの breakout は構造的に
//! 不可能である（最悪でもクローラが解釈できない不活性テキストが出力
//! されるだけで、XSS には至らない）。この非対称性は本クレートの URL
//! 属性検証（[`crate::is_safe_url`]、不変条件 8。不正な値を許可すると
//! 危険になり得るため fail-closed に出力をスキップする）とは前提が
//! 異なる。JSON-LD の中身は script 要素の外へ影響しないため fail-closed
//! にする必要がなく、手書き JSON パーサを追加する保守コスト・攻撃面拡大
//! を避ける判断とした。

use crate::{el, raw_html, Node};

/// シリアライズ済みの JSON 文字列を中立化して
/// `<script type="application/ld+json">...</script>` ノードを組み立てる。
///
/// 引数はすでに JSON としてシリアライズ済みの文字列を渡す
/// （HTML エスケープ済みの文字列を渡さないこと。エスケープすると JSON が
/// 壊れる）。`serde_json` 等を利用する場合は
/// `serde_json::to_string(&value)` の結果をそのまま渡せる
/// （本クレートは外部依存ゼロ契約のため `serde_json` を re-export・
/// 依存はしない。呼び出し側の任意依存として利用する）。
///
/// # Security
///
/// 中立化処理により出力には生の `<` `>` `&` が一切含まれない
/// （`</script>` による script 要素の早期終了・`<!--` コメント注入が
/// 構造的に不可能）。詳細はモジュール doc を参照。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{json_ld, render};
///
/// let node = json_ld(r#"{"@context":"https://schema.org","name":"A"}"#);
/// assert_eq!(
///     render(&node),
///     r#"<script type="application/ld+json">{"@context":"https://schema.org","name":"A"}</script>"#
/// );
/// ```
///
/// `serde_json` と組み合わせる例（利用者側の任意依存）:
///
/// ```ignore
/// use fandhe_frontend_core::json_ld;
///
/// let value = serde_json::json!({ "@context": "https://schema.org", "@type": "Article" });
/// let node = json_ld(serde_json::to_string(&value).expect("serialize"));
/// ```
pub fn json_ld(json: impl Into<String>) -> Node {
    let neutralized = escape_json_for_script(&json.into());
    // ESCAPE-REVIEWED: `escape_json_for_script` が `<` `>` `&` および
    // U+2028/U+2029 を `\uXXXX` 化済みであり、出力に HTML 活性文字が
    // 一切残らないことをモジュール内テスト（xss 回帰含む）が固定する。
    // json_ld() はこの中立化を経由する唯一の呼び出し元であり、
    // raw_html() の新たな迂回経路を追加するものではない（不変条件 2）。
    #[expect(
        clippy::disallowed_methods,
        reason = "ESCAPE-REVIEWED: 中立化済み JSON（< > & U+2028/2029 を \\uXXXX 化済みで HTML 活性文字を含まない）のみを渡す。json_ld のテストが出力に生の < > & が残らないことを固定する"
    )]
    let content = raw_html(neutralized);
    el(
        "script",
        vec![("type", "application/ld+json")],
        vec![content],
    )
}

/// JSON 文書全体から HTML/JS コンテキストで活性化しうる文字のみを
/// `\uXXXX` 形式へ全置換する（`"` `\` 制御文字には触れない。モジュール
/// doc の「意味保存の根拠」参照）。
fn escape_json_for_script(json: &str) -> String {
    let mut out = String::with_capacity(json.len());
    for c in json.chars() {
        match c {
            '<' => out.push_str("\\u003C"),
            '>' => out.push_str("\\u003E"),
            '&' => out.push_str("\\u0026"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render;

    #[test]
    fn renders_expected_script_element_without_mangling_quotes() {
        let node = json_ld(r#"{"@context":"https://schema.org","name":"A"}"#);
        assert_eq!(
            render(&node),
            r#"<script type="application/ld+json">{"@context":"https://schema.org","name":"A"}</script>"#
        );
    }

    #[test]
    fn neutralizes_script_breakout_attempt() {
        let payload = r#"{"name":"</script><script>alert(1)</script>"}"#;
        let node = json_ld(payload);
        let html = render(&node);
        assert!(
            !html.contains('<') || html.matches("<script").count() == 1,
            "生の < が script 開始タグ以外に出現した: {html}"
        );
        assert!(
            !html.contains("</script><script>"),
            "script breakout 断片がそのまま出力された: {html}"
        );
        // 出力全体で `<` `>` `&` が使われるのは開始・終了タグそのものだけ
        // （中身は \uXXXX 化済み）であることを固定する。
        let inner_start = html.find('>').unwrap() + 1;
        let inner_end = html.rfind("</script>").unwrap();
        let inner = &html[inner_start..inner_end];
        assert!(!inner.contains('<'));
        assert!(!inner.contains('>'));
        assert!(!inner.contains('&'));
    }

    #[test]
    fn neutralizes_html_comment_injection_attempt() {
        let payload = r#"{"name":"<!--><script>alert(1)</script>-->"}"#;
        let html = render(&json_ld(payload));
        let inner_start = html.find('>').unwrap() + 1;
        let inner_end = html.rfind("</script>").unwrap();
        let inner = &html[inner_start..inner_end];
        assert!(!inner.contains('<'));
        assert!(!inner.contains('>'));
    }

    #[test]
    fn neutralizes_ampersand() {
        let payload = r#"{"name":"Tom & Jerry"}"#;
        let html = render(&json_ld(payload));
        assert!(!html.contains("Tom & Jerry"));
        assert!(html.contains(r"Tom \u0026 Jerry"));
    }

    #[test]
    fn neutralizes_line_terminators() {
        let payload = "{\"name\":\"a\u{2028}b\u{2029}c\"}";
        let html = render(&json_ld(payload));
        assert!(!html.contains('\u{2028}'));
        assert!(!html.contains('\u{2029}'));
        assert!(html.contains("a\\u2028b\\u2029c"));
    }

    #[test]
    fn empty_input_renders_empty_script_element() {
        let html = render(&json_ld(""));
        assert_eq!(html, r#"<script type="application/ld+json"></script>"#);
    }

    #[test]
    fn preserves_json_syntax_characters_untouched() {
        // `"` `\` および JSON エスケープ済み表記（`\n` 等）は中立化の
        // 対象外であり、JSON 構文を壊さずそのまま透過する。
        let payload = r#"{"quote":"a\"b","backslash":"a\\b","newline":"a\nb"}"#;
        let html = render(&json_ld(payload));
        assert!(html.contains(r#"a\"b"#));
        assert!(html.contains(r"a\\b"));
        assert!(html.contains(r"a\nb"));
    }

    #[test]
    fn invalid_json_input_still_produces_html_inert_output() {
        // json_ld は JSON バリデーションを行わない（fail-safe 設計）。
        // 不正な入力でも出力に生の HTML 活性文字が残らないことを固定する。
        let payload = "<script>alert(1)</script> not valid json &";
        let html = render(&json_ld(payload));
        let inner_start = html.find('>').unwrap() + 1;
        let inner_end = html.rfind("</script>").unwrap();
        let inner = &html[inner_start..inner_end];
        assert!(!inner.contains('<'));
        assert!(!inner.contains('>'));
        assert!(!inner.contains('&'));
    }
}
