//! 属性値として出力する URL の安全性検証（イシュー #373）。
//!
//! `core/src/escape.rs` の既定エスケープは属性値コンテキストからの
//! breakout（`"` 等による脱出）を防ぐが、脱出を伴わない `javascript:` 等の
//! URL スキーム経由の XSS は防げない。本モジュールは [`render_into`]
//!（`lib.rs`）と `rws-wasm-client` の実 DOM 属性更新経路（`binding_dom.rs`）
//! の両方から呼ばれる契約の検証関数を提供し、両経路に同一の保証を与える
//! （`docs/policy/attribute-output-policy.md` 参照）。
//!
//! 外部依存ゼロ（std のみ）・`forbid(unsafe_code)` 域・panic なし
//! （[`is_safe_url`] は常に `bool` を返す）。
//!
//! [`render_into`]: crate::render

/// URL 値を受け取り得る属性名の正リスト。
///
/// `render_into`（`lib.rs`）・`rws-wasm-client` の `binding_dom.rs` の双方が
/// 本定数を単一の情報源として参照する（コピーを作らない）。属性名の照合は
/// ASCII 大文字小文字非依存で行う（[`is_url_attr`] 参照）。
pub const URL_ATTRS: &[&str] = &[
    "href",
    "src",
    "action",
    "formaction",
    "xlink:href",
    "poster",
    "cite",
    "data",
    "background",
    "ping",
    "dynsrc",
    "lowsrc",
];

/// 属性名が [`URL_ATTRS`] に該当するかを ASCII 大文字小文字非依存で判定する。
///
/// `srcset` は複数 URL 候補を含む特殊構文のため本関数の対象に含めず、
/// 呼び出し側（`render_into`・`binding_dom.rs`・`keyed_dom.rs`）で
/// [`is_safe_srcset`] を個別に適用する契約とする。
pub fn is_url_attr(name: &str) -> bool {
    URL_ATTRS.iter().any(|a| a.eq_ignore_ascii_case(name))
}

/// 属性名がイベントハンドラ属性（`on` で始まる）かどうかを判定する。
///
/// 本フレームワークのインタラクションモデルは `data-hydrate` /
/// `data-bind-*` によるマーキングと束縛点方式であり、インライン JS
/// （`onclick` 等）は設計上の正規経路に存在しない。`render_into` は
/// 該当属性を fail-closed で出力しない（意図的な制限の追加であり、
/// 既定エスケープの迂回経路の新設には当たらない）。
pub fn is_event_handler_attr(name: &str) -> bool {
    // `on` 単体はイベントハンドラ属性として無効（HTML 標準に `on` という
    // 名前のイベントは存在しない）。少なくとも 1 文字のイベント名が続く
    // ことを要求する（`onx` 以上の長さ）。
    name.len() > 2
        && name.as_bytes()[0].eq_ignore_ascii_case(&b'o')
        && name.as_bytes()[1].eq_ignore_ascii_case(&b'n')
}

/// URL 値が安全に出力してよいものかを判定する（許可リスト方式・deny by
/// default）。
///
/// # 判定規則
///
/// 1. ブラウザの寛容な URL パースを模倣し、判定前に ASCII タブ
///    （`\t`）・改行（`\n` `\r`）を値中の全位置から除去する。
/// 2. 先頭の ASCII 制御文字（C0）・空白文字をトリムする。
/// 3. 残った文字列の先頭からスキームを抽出する。スキームとみなすのは
///    `/` `?` `#` `\` のいずれよりも前に現れる `:` までの区間が
///    `[a-zA-Z][a-zA-Z0-9+.\-]*` に一致する場合のみ。`/` `?` `#` `\`
///    が `:` より先に現れる場合はスキームなし（相対 URL）として扱う
///    （例: `/path/a:b` はコロンの前に `/` があるため相対 URL）。
/// 4. スキームなし（相対 URL・`//host` の protocol-relative 含む）は許可する。
/// 5. スキームありの場合、`http` / `https` / `mailto` / `tel`
///    （大文字小文字非依存）のみ許可する。それ以外（`javascript:` /
///    `data:` / `vbscript:` / `blob:` / 未知スキーム等）はすべて拒否する。
///
/// 空文字列は相対 URL（許可）として扱う。
///
/// # Examples
///
/// ```
/// use rws_core::is_safe_url;
///
/// assert!(is_safe_url("/items/1"));
/// assert!(is_safe_url("https://example.com"));
/// assert!(!is_safe_url("javascript:alert(1)"));
/// assert!(!is_safe_url("java\tscript:alert(1)"));
/// ```
pub fn is_safe_url(value: &str) -> bool {
    // ステップ 1: タブ・改行を全位置から除去する（`java\tscript:` のような
    // 偽装形の遮断。ブラウザはこれらの制御文字を URL パース前に無視する
    // 挙動を持つため、それを模倣した過剰側安全な正規化を行う）。
    let stripped: String = value
        .chars()
        .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
        .collect();

    // ステップ 2: 先頭の C0 制御文字・空白をトリムする。
    let trimmed = stripped.trim_start_matches(|c: char| c.is_control() || c.is_whitespace());

    // ステップ 3〜5: スキーム抽出。
    match extract_scheme(trimmed) {
        None => true, // 相対 URL・空文字列は許可。
        Some(scheme) => {
            scheme.eq_ignore_ascii_case("http")
                || scheme.eq_ignore_ascii_case("https")
                || scheme.eq_ignore_ascii_case("mailto")
                || scheme.eq_ignore_ascii_case("tel")
        }
    }
}

/// 正規化済み文字列からスキームを抽出する。
///
/// `/` `?` `#` `\` のいずれかが最初の `:` より前に現れる場合はスキーム
/// なし（`None`）として扱う。これにより `/path/a:b` のような相対 URL に
/// 含まれるコロンをスキーム区切りと誤認しない。
fn extract_scheme(s: &str) -> Option<&str> {
    let colon_idx = s.find(':')?;
    let candidate = &s[..colon_idx];

    // スキーム候補より前に breakout 文字（相対/フラグメント/クエリ/
    // バックスラッシュ）が含まれる場合はスキームとみなさない。
    if candidate.contains(['/', '?', '#', '\\']) {
        return None;
    }

    let mut chars = candidate.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return None,
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '.' || c == '-') {
        return None;
    }

    Some(candidate)
}

/// `srcset` 属性値（カンマ区切りの URL 候補 + 記述子）が安全に出力してよい
/// ものかを判定する。
///
/// `srcset` はカンマ区切りの複数 URL 候補（各候補は空白区切りで
/// `URL [記述子]` の形式）を持つ特殊構文であり、[`URL_ATTRS`] /
/// [`is_url_attr`] の対象外（単純な単一 URL 判定では表現できないため）。
/// 各候補の先頭トークン（URL 部分。記述子は無視）を [`is_safe_url`] で
/// 検証し、1 候補でも不合格なら属性全体を不合格として扱う（部分的な
/// 書き換えは決定性を損なうため行わない）。
///
/// `render_into`（`lib.rs`）・`rws-wasm-client` の `binding_dom.rs`・
/// `keyed_dom.rs` の 3 経路すべてが本関数を単一の情報源として参照する
/// 契約とする（イシュー #373 レビュー指摘: 従来は `render_into` にのみ
/// インライン実装されており、wasm-client の実 DOM 直接更新経路
/// （`apply_one`/`build_element`）では検証されない不整合があった）。
///
/// # Examples
///
/// ```
/// use rws_core::is_safe_srcset;
///
/// assert!(is_safe_srcset("/a.png 1x, /b.png 2x"));
/// assert!(!is_safe_srcset("/a.png 1x, javascript:alert(1) 2x"));
/// ```
pub fn is_safe_srcset(value: &str) -> bool {
    value.split(',').all(|candidate| {
        let url_part = candidate.split_whitespace().next().unwrap_or("");
        is_safe_url(url_part)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_urls_are_safe() {
        for v in ["/items/1", "./rel", "?q=1", "#frag", "//example.com/x", ""] {
            assert!(is_safe_url(v), "should be safe: {v}");
        }
    }

    #[test]
    fn relative_url_with_colon_in_path_is_safe() {
        // `/path/a:b` はコロンの前に `/` があるためスキームとみなさない。
        assert!(is_safe_url("/path/a:b"));
    }

    #[test]
    fn allowed_schemes_are_safe() {
        for v in [
            "https://example.com",
            "http://example.com",
            "mailto:a@example.com",
            "tel:+819012345678",
            "HTTPS://EXAMPLE.COM",
        ] {
            assert!(is_safe_url(v), "should be safe: {v}");
        }
    }

    #[test]
    fn dangerous_schemes_are_rejected() {
        for v in [
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "vbscript:msgbox(1)",
            "data:text/html;base64,PHNjcmlwdD4=",
            "foo:bar",
            "blob:https://example.com/uuid",
        ] {
            assert!(!is_safe_url(v), "should be rejected: {v}");
        }
    }

    #[test]
    fn control_char_and_whitespace_disguised_schemes_are_rejected() {
        for v in [
            "java\tscript:alert(1)",
            "java\nscript:alert(1)",
            "\u{0}javascript:alert(1)",
            " javascript:alert(1)",
            "\tjavascript:alert(1)",
        ] {
            assert!(!is_safe_url(v), "should be rejected: {v}");
        }
    }

    #[test]
    fn url_attr_matching_is_case_insensitive() {
        assert!(is_url_attr("href"));
        assert!(is_url_attr("HREF"));
        assert!(is_url_attr("Src"));
        assert!(!is_url_attr("class"));
    }

    #[test]
    fn event_handler_attr_detection() {
        assert!(is_event_handler_attr("onclick"));
        assert!(is_event_handler_attr("ONERROR"));
        assert!(is_event_handler_attr("OnMouseOver"));
        assert!(!is_event_handler_attr("data-on"));
        assert!(!is_event_handler_attr("on"));
        assert!(!is_event_handler_attr("open"));
    }
}
