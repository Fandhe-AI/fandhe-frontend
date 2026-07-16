//! `rws-core` の既定エスケープ実装（TASK-1.1a）。
//!
//! ノード木 API（TASK-1.1b で `lib.rs` に追加予定）がテキストノード・属性値を
//! HTML へシリアライズする際、**必ずこのモジュールの関数を経由してエスケープする**
//! ことを製品仕様として固定する。エスケープを迂回できる経路は `raw_html()`
//! （明示的オプトイン API、TASK-1.1b で設計）のみとし、それ以外の新規迂回経路を
//! 作らないことが `rws-core` 全体の不変条件（REQ-1）となる。
//!
//! # 対象文字仕様（OWASP XSS Prevention Cheat Sheet Rule #1 準拠）
//!
//! テキストノード・属性値の両コンテキストで、以下の 5 文字を同一の規則で
//! エスケープする（コンテキストごとに対象文字を変えない過剰側の安全設計）。
//!
//! | 文字 | 置換後    | 根拠                                                         |
//! |------|-----------|--------------------------------------------------------------|
//! | `&`  | `&amp;`   | エンティティ再解釈（二重エスケープ・エンティティ偽装）防止。**必ず最初に処理する** |
//! | `<`  | `&lt;`    | タグ開始の無効化（テキストコンテキストの XSS 主経路）        |
//! | `>`  | `&gt;`    | タグ終端の無効化・コメント脱出対策                            |
//! | `"`  | `&quot;`  | 二重引用符属性値からの脱出防止                                |
//! | `'`  | `&#x27;`  | 単一引用符属性値からの脱出防止（`&apos;` は HTML4 非対応のため数値参照を採用） |
//!
//! `&` を先頭で処理することで、他の文字が生成するエンティティ（`&lt;` 等）自体が
//! 再度 `&` として二重エスケープされることを防ぐ。処理順序はテストで固定する。
//!
//! # 契約（レンダリング側 = TASK-1.1b が前提とする不変条件）
//!
//! - 属性値は常に二重引用符（`"`）で囲むことをレンダリング側の責務とする。
//!   単一引用符・引用符なし属性は本フレームワークでは使用しない設計とし、
//!   その前提のもとで上表 5 文字のエスケープのみで属性値コンテキストの
//!   脱出を防止できる。
//! - 非 ASCII（マルチバイト UTF-8）文字は対象外文字として変更せず透過する。
//!
//! # スコープ外（本モジュールでは扱わない）
//!
//! URL / JavaScript / CSS コンテキストのエスケープ、void 要素処理、
//! タグ名・属性名の妥当性検証。必要になった時点で
//! `.claude/rules/out-of-scope-tracking.md` に従い Issue 化を検討する。

/// 入力文字列を既定のエスケープ規則に従って HTML エンティティ化した新しい
/// `String` を返す。
///
/// テキストノード・属性値の両コンテキストで安全に埋め込める文字列を返す
/// （上表 5 文字を置換し、それ以外の文字はそのまま透過する）。
///
/// `rws-core` のノード木 API（TASK-1.1b）が、ユーザー由来の文字列を
/// テキストノード・属性値としてシリアライズする際に呼び出すことを想定する。
///
/// # Examples
///
/// ```
/// use rws_core::escape_html;
///
/// assert_eq!(escape_html("<script>"), "&lt;script&gt;");
/// assert_eq!(escape_html("plain text"), "plain text");
/// ```
pub fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    escape_html_into(input, &mut out);
    out
}

/// [`escape_html`] の書き込み先を呼び出し側の `String` バッファに委ねる版。
///
/// `render_into` のような呼び出し元が、ノード木のシリアライズ中に確保済みの
/// バッファへ直接追記できるようにし、中間 `String` の割り当てを避けるための
/// 契約を提供する。エスケープ規則・処理順序は [`escape_html`] と同一。
///
/// # Examples
///
/// ```
/// use rws_core::escape_html_into;
///
/// let mut buf = String::from("prefix:");
/// escape_html_into("<b>", &mut buf);
/// assert_eq!(buf, "prefix:&lt;b&gt;");
/// ```
pub fn escape_html_into(input: &str, out: &mut String) {
    // エスケープ不要文字が続く区間はまとめて push_str し、置換が必要な
    // 1 文字ごとに都度 push_str するオーバーヘッドを避ける。
    let mut last_end = 0;
    for (idx, ch) in input.char_indices() {
        // `&` を最初に判定することが仕様上重要: 他の分岐が生成するエンティティ
        // （例: `&lt;`）自体を後から再エスケープしてしまう（二重エスケープ）
        // 事態を避けるため、判定順序ではなく元入力の走査順で `&` を検出した
        // 時点でそのまま `&amp;` に置換する（走査は 1 パスなので二重エスケープは発生しない）。
        let entity = match ch {
            '&' => "&amp;",
            '<' => "&lt;",
            '>' => "&gt;",
            '"' => "&quot;",
            '\'' => "&#x27;",
            _ => continue,
        };
        out.push_str(&input[last_end..idx]);
        out.push_str(entity);
        last_end = idx + ch.len_utf8();
    }
    out.push_str(&input[last_end..]);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 対象 5 文字それぞれが仕様どおりのエンティティに置換されることを固定する。
    #[test]
    fn escapes_each_target_character_to_its_specified_entity() {
        assert_eq!(escape_html("&"), "&amp;");
        assert_eq!(escape_html("<"), "&lt;");
        assert_eq!(escape_html(">"), "&gt;");
        assert_eq!(escape_html("\""), "&quot;");
        assert_eq!(escape_html("'"), "&#x27;");
    }

    /// PoC-3 と同型の XSS ペイロードが無害化されることを確認する
    /// （script タグ・属性脱出用の引用符を含む代表ケース）。
    #[test]
    fn neutralizes_script_tag_xss_payload() {
        assert_eq!(
            escape_html("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"
        );
    }

    /// `&` の先行処理を固定する: 入力中の `<` が生成した `&lt;` の `&` を
    /// 再度エスケープして `&amp;lt;` になってしまう二重エスケープ不全がないことを保証する。
    #[test]
    fn does_not_double_escape_generated_entities() {
        let escaped = escape_html("<");
        assert_eq!(escaped, "&lt;");
        assert_ne!(escaped, "&amp;lt;");
    }

    /// 非 ASCII（日本語・絵文字などマルチバイト UTF-8）は対象外文字として
    /// 破壊されず透過することを確認する。
    #[test]
    fn passes_through_non_ascii_characters_unchanged() {
        assert_eq!(escape_html("日本語テキスト"), "日本語テキスト");
        assert_eq!(escape_html("絵文字🎉です"), "絵文字🎉です");
        assert_eq!(escape_html("<日本語>"), "&lt;日本語&gt;");
    }

    /// 空文字列・エスケープ不要文字列はそのまま返る（無駄な確保・変更がない）ことを確認する。
    #[test]
    fn returns_input_unchanged_when_no_escaping_needed() {
        assert_eq!(escape_html(""), "");
        assert_eq!(
            escape_html("plain text without special chars"),
            "plain text without special chars"
        );
    }

    /// `escape_html_into` が呼び出し側バッファへの追記として機能し、
    /// 既存内容を破壊しないことを確認する（render_into からの利用契約）。
    #[test]
    fn escape_html_into_appends_to_existing_buffer() {
        let mut buf = String::from("prefix:");
        escape_html_into("<b>&\"'</b>", &mut buf);
        assert_eq!(buf, "prefix:&lt;b&gt;&amp;&quot;&#x27;&lt;/b&gt;");
    }
}
