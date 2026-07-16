//! `xtask` 内部専用の最小 JSON パーサ。
//!
//! `cargo metadata --format-version 1` の出力（cargo が生成する整形式 JSON）を
//! 読み取るためだけに存在する。`check_deps` から呼ばれ、パース結果は
//! [`Json`] の木として返る。
//!
//! 外部依存ゼロ（`coding-rust.md` / `xtask/Cargo.toml` 参照）の制約下で
//! `serde_json` を使わずに `cargo metadata` を解析するための代替実装であり、
//! 汎用 JSON 実装ではない（コメント・仕様は最小限）。
//!
//! # パニックしない
//!
//! 不正な入力（途中切断・不正エスケープ・数値形式不正・深すぎるネスト）は
//! すべて [`JsonError`] として返し、`unwrap()` / `panic!` は使わない。
//! `cargo metadata` の出力は外部プロセスの出力であり、信頼しきらない
//! （security.md: A08 データ整合性）。

use std::fmt;

/// パース済み JSON 値。
///
/// `cargo metadata` の出力に必要な型のみを表現する（top-level はオブジェクト想定）。
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    /// キー順序を保持する（`cargo metadata` の出力順に依存する処理はしないが、
    /// デバッグ時の再現性のため Vec で保持し HashMap は使わない）。
    Object(Vec<(String, Json)>),
}

impl Json {
    /// オブジェクトからキーを引く。オブジェクトでない場合・キーが存在しない場合は `None`。
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Object(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Array(items) => Some(items.as_slice()),
            _ => None,
        }
    }
}

/// パース失敗時のエラー。位置（バイトオフセット）と理由のみを保持する。
///
/// 機微情報の露出防止（security.md）のため、入力全体の再掲や環境変数等は含めない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonError {
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "JSON parse error at byte {}: {}",
            self.offset, self.message
        )
    }
}

impl std::error::Error for JsonError {}

/// 再帰下降パーサの最大ネスト深さ。
///
/// スタックオーバーフロー（DoS）を防ぐための防御的上限であり、
/// `cargo metadata` の実出力（数十段程度）を十分に超える値を設定する。
const MAX_DEPTH: usize = 128;

/// `input` を JSON としてパースする。
///
/// `check_deps` の `cargo metadata` 実行結果の解析元。先頭・末尾の空白は許容し、
/// 末尾に余分なトークンがある場合はエラーとする。
pub fn parse(input: &str) -> Result<Json, JsonError> {
    let bytes = input.as_bytes();
    let mut pos = 0usize;
    skip_whitespace(bytes, &mut pos);
    let value = parse_value(bytes, &mut pos, 0)?;
    skip_whitespace(bytes, &mut pos);
    if pos != bytes.len() {
        return Err(JsonError {
            offset: pos,
            message: "trailing data after JSON value".to_string(),
        });
    }
    Ok(value)
}

fn skip_whitespace(bytes: &[u8], pos: &mut usize) {
    while *pos < bytes.len() && matches!(bytes[*pos], b' ' | b'\t' | b'\n' | b'\r') {
        *pos += 1;
    }
}

fn peek(bytes: &[u8], pos: usize) -> Result<u8, JsonError> {
    bytes.get(pos).copied().ok_or(JsonError {
        offset: pos,
        message: "unexpected end of input".to_string(),
    })
}

fn parse_value(bytes: &[u8], pos: &mut usize, depth: usize) -> Result<Json, JsonError> {
    if depth > MAX_DEPTH {
        return Err(JsonError {
            offset: *pos,
            message: "nesting too deep".to_string(),
        });
    }
    skip_whitespace(bytes, pos);
    let c = peek(bytes, *pos)?;
    match c {
        b'{' => parse_object(bytes, pos, depth),
        b'[' => parse_array(bytes, pos, depth),
        b'"' => parse_string(bytes, pos).map(Json::String),
        b't' => parse_literal(bytes, pos, "true", Json::Bool(true)),
        b'f' => parse_literal(bytes, pos, "false", Json::Bool(false)),
        b'n' => parse_literal(bytes, pos, "null", Json::Null),
        b'-' | b'0'..=b'9' => parse_number(bytes, pos),
        _ => Err(JsonError {
            offset: *pos,
            message: format!("unexpected byte 0x{c:02x}"),
        }),
    }
}

fn parse_literal(bytes: &[u8], pos: &mut usize, lit: &str, value: Json) -> Result<Json, JsonError> {
    let end = *pos + lit.len();
    if end > bytes.len() || &bytes[*pos..end] != lit.as_bytes() {
        return Err(JsonError {
            offset: *pos,
            message: format!("expected literal `{lit}`"),
        });
    }
    *pos = end;
    Ok(value)
}

fn parse_object(bytes: &[u8], pos: &mut usize, depth: usize) -> Result<Json, JsonError> {
    // '{' を消費
    *pos += 1;
    let mut entries = Vec::new();
    skip_whitespace(bytes, pos);
    if peek(bytes, *pos)? == b'}' {
        *pos += 1;
        return Ok(Json::Object(entries));
    }
    loop {
        skip_whitespace(bytes, pos);
        if peek(bytes, *pos)? != b'"' {
            return Err(JsonError {
                offset: *pos,
                message: "expected string key".to_string(),
            });
        }
        let key = parse_string(bytes, pos)?;
        skip_whitespace(bytes, pos);
        if peek(bytes, *pos)? != b':' {
            return Err(JsonError {
                offset: *pos,
                message: "expected `:` after object key".to_string(),
            });
        }
        *pos += 1;
        let value = parse_value(bytes, pos, depth + 1)?;
        entries.push((key, value));
        skip_whitespace(bytes, pos);
        match peek(bytes, *pos)? {
            b',' => {
                *pos += 1;
            }
            b'}' => {
                *pos += 1;
                break;
            }
            _ => {
                return Err(JsonError {
                    offset: *pos,
                    message: "expected `,` or `}` in object".to_string(),
                })
            }
        }
    }
    Ok(Json::Object(entries))
}

fn parse_array(bytes: &[u8], pos: &mut usize, depth: usize) -> Result<Json, JsonError> {
    // '[' を消費
    *pos += 1;
    let mut items = Vec::new();
    skip_whitespace(bytes, pos);
    if peek(bytes, *pos)? == b']' {
        *pos += 1;
        return Ok(Json::Array(items));
    }
    loop {
        let value = parse_value(bytes, pos, depth + 1)?;
        items.push(value);
        skip_whitespace(bytes, pos);
        match peek(bytes, *pos)? {
            b',' => {
                *pos += 1;
            }
            b']' => {
                *pos += 1;
                break;
            }
            _ => {
                return Err(JsonError {
                    offset: *pos,
                    message: "expected `,` or `]` in array".to_string(),
                })
            }
        }
    }
    Ok(Json::Array(items))
}

fn parse_string(bytes: &[u8], pos: &mut usize) -> Result<String, JsonError> {
    // '"' を消費
    *pos += 1;
    let mut out = String::new();
    // サロゲートペア（\uD800-\uDBFF に続く \uDC00-\uDFFF）を組み立てるための保留領域。
    let mut pending_high_surrogate: Option<u16> = None;
    loop {
        let c = peek(bytes, *pos)?;
        match c {
            b'"' => {
                *pos += 1;
                if pending_high_surrogate.is_some() {
                    return Err(JsonError {
                        offset: *pos,
                        message: "unpaired UTF-16 surrogate".to_string(),
                    });
                }
                return Ok(out);
            }
            b'\\' => {
                *pos += 1;
                let esc = peek(bytes, *pos)?;
                match esc {
                    b'"' => {
                        out.push('"');
                        *pos += 1;
                    }
                    b'\\' => {
                        out.push('\\');
                        *pos += 1;
                    }
                    b'/' => {
                        out.push('/');
                        *pos += 1;
                    }
                    b'b' => {
                        out.push('\u{0008}');
                        *pos += 1;
                    }
                    b'f' => {
                        out.push('\u{000C}');
                        *pos += 1;
                    }
                    b'n' => {
                        out.push('\n');
                        *pos += 1;
                    }
                    b'r' => {
                        out.push('\r');
                        *pos += 1;
                    }
                    b't' => {
                        out.push('\t');
                        *pos += 1;
                    }
                    b'u' => {
                        *pos += 1;
                        let unit = parse_hex4(bytes, pos)?;
                        if let Some(high) = pending_high_surrogate.take() {
                            if (0xDC00..=0xDFFF).contains(&unit) {
                                let c = 0x10000
                                    + (u32::from(high) - 0xD800) * 0x400
                                    + (u32::from(unit) - 0xDC00);
                                match char::from_u32(c) {
                                    Some(ch) => out.push(ch),
                                    None => {
                                        return Err(JsonError {
                                            offset: *pos,
                                            message: "invalid surrogate pair".to_string(),
                                        })
                                    }
                                }
                            } else {
                                return Err(JsonError {
                                    offset: *pos,
                                    message: "expected low surrogate".to_string(),
                                });
                            }
                        } else if (0xD800..=0xDBFF).contains(&unit) {
                            pending_high_surrogate = Some(unit);
                        } else if (0xDC00..=0xDFFF).contains(&unit) {
                            return Err(JsonError {
                                offset: *pos,
                                message: "unexpected low surrogate".to_string(),
                            });
                        } else {
                            match char::from_u32(u32::from(unit)) {
                                Some(ch) => out.push(ch),
                                None => {
                                    return Err(JsonError {
                                        offset: *pos,
                                        message: "invalid \\u escape".to_string(),
                                    })
                                }
                            }
                        }
                    }
                    other => {
                        return Err(JsonError {
                            offset: *pos,
                            message: format!("invalid escape `\\{}`", other as char),
                        })
                    }
                }
            }
            0x00..=0x1F => {
                return Err(JsonError {
                    offset: *pos,
                    message: "unescaped control character in string".to_string(),
                })
            }
            _ => {
                // UTF-8 の非 ASCII 先頭バイトも含め、バイト列としてそのまま読み進め、
                // 文字境界で String へ復元する。
                let start = *pos;
                let width = utf8_len(c);
                let end = start + width;
                if end > bytes.len() {
                    return Err(JsonError {
                        offset: start,
                        message: "invalid UTF-8 sequence".to_string(),
                    });
                }
                let s = std::str::from_utf8(&bytes[start..end]).map_err(|_| JsonError {
                    offset: start,
                    message: "invalid UTF-8 sequence".to_string(),
                })?;
                out.push_str(s);
                *pos = end;
            }
        }
    }
}

fn utf8_len(first_byte: u8) -> usize {
    if first_byte & 0x80 == 0 {
        1
    } else if first_byte & 0xE0 == 0xC0 {
        2
    } else if first_byte & 0xF0 == 0xE0 {
        3
    } else {
        4
    }
}

fn parse_hex4(bytes: &[u8], pos: &mut usize) -> Result<u16, JsonError> {
    if *pos + 4 > bytes.len() {
        return Err(JsonError {
            offset: *pos,
            message: "truncated \\u escape".to_string(),
        });
    }
    let s = std::str::from_utf8(&bytes[*pos..*pos + 4]).map_err(|_| JsonError {
        offset: *pos,
        message: "invalid \\u escape".to_string(),
    })?;
    let value = u16::from_str_radix(s, 16).map_err(|_| JsonError {
        offset: *pos,
        message: "invalid \\u escape hex digits".to_string(),
    })?;
    *pos += 4;
    Ok(value)
}

fn parse_number(bytes: &[u8], pos: &mut usize) -> Result<Json, JsonError> {
    let start = *pos;
    if peek(bytes, *pos)? == b'-' {
        *pos += 1;
    }
    let int_start = *pos;
    while *pos < bytes.len() && bytes[*pos].is_ascii_digit() {
        *pos += 1;
    }
    if *pos == int_start {
        return Err(JsonError {
            offset: *pos,
            message: "invalid number: missing digits".to_string(),
        });
    }
    if *pos < bytes.len() && bytes[*pos] == b'.' {
        *pos += 1;
        let frac_start = *pos;
        while *pos < bytes.len() && bytes[*pos].is_ascii_digit() {
            *pos += 1;
        }
        if *pos == frac_start {
            return Err(JsonError {
                offset: *pos,
                message: "invalid number: missing fraction digits".to_string(),
            });
        }
    }
    if *pos < bytes.len() && matches!(bytes[*pos], b'e' | b'E') {
        *pos += 1;
        if *pos < bytes.len() && matches!(bytes[*pos], b'+' | b'-') {
            *pos += 1;
        }
        let exp_start = *pos;
        while *pos < bytes.len() && bytes[*pos].is_ascii_digit() {
            *pos += 1;
        }
        if *pos == exp_start {
            return Err(JsonError {
                offset: *pos,
                message: "invalid number: missing exponent digits".to_string(),
            });
        }
    }
    let s = std::str::from_utf8(&bytes[start..*pos]).map_err(|_| JsonError {
        offset: start,
        message: "invalid number encoding".to_string(),
    })?;
    let n: f64 = s.parse().map_err(|_| JsonError {
        offset: start,
        message: "invalid number literal".to_string(),
    })?;
    Ok(Json::Number(n))
}

/// テスト専用: 深いネストの入力を生成する（`MAX_DEPTH` 超過の検証用）。
#[cfg(test)]
fn nested_arrays(depth: usize) -> String {
    let mut s = "[".repeat(depth);
    s.push_str(&"]".repeat(depth));
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_primitives() {
        assert_eq!(parse("null").unwrap(), Json::Null);
        assert_eq!(parse("true").unwrap(), Json::Bool(true));
        assert_eq!(parse("false").unwrap(), Json::Bool(false));
        assert_eq!(parse("123").unwrap(), Json::Number(123.0));
        assert_eq!(parse("-1.5e2").unwrap(), Json::Number(-150.0));
        assert_eq!(
            parse("\"hello\"").unwrap(),
            Json::String("hello".to_string())
        );
    }

    #[test]
    fn parses_escapes_and_unicode() {
        assert_eq!(
            parse(r#""a\"b\\c\/d\n\t""#).unwrap(),
            Json::String("a\"b\\c/d\n\t".to_string())
        );
        // 日本語（マルチバイト UTF-8）がそのまま復元されること
        assert_eq!(parse("\"依存\"").unwrap(), Json::String("依存".to_string()));
        // \uXXXX エスケープ
        assert_eq!(parse(r#""é""#).unwrap(), Json::String("é".to_string()));
        // サロゲートペア（絵文字等の非 BMP 文字）
        assert_eq!(parse(r#""😀""#).unwrap(), Json::String("😀".to_string()));
    }

    #[test]
    fn parses_array_and_object() {
        let v = parse(r#"{"a": [1, 2, 3], "b": {"c": null}}"#).unwrap();
        assert_eq!(
            v.get("a").unwrap().as_array().unwrap(),
            &[Json::Number(1.0), Json::Number(2.0), Json::Number(3.0)]
        );
        assert_eq!(v.get("b").unwrap().get("c").unwrap(), &Json::Null);
    }

    #[test]
    fn rejects_truncated_input() {
        assert!(parse(r#"{"a": "#).is_err());
        assert!(parse(r#""unterminated"#).is_err());
        assert!(parse("[1, 2").is_err());
    }

    #[test]
    fn rejects_invalid_escape() {
        assert!(parse(r#""\x41""#).is_err());
        assert!(parse(r#""\u12""#).is_err());
    }

    #[test]
    fn rejects_too_deep_nesting() {
        let input = nested_arrays(MAX_DEPTH + 10);
        assert!(parse(&input).is_err());
    }

    #[test]
    fn rejects_trailing_data() {
        assert!(parse("null null").is_err());
    }
}
