//! フェンスコードブロック本文のビルド時トークナイズ（イシュー #1078、
//! 親 #1059 / ルート #1056）。
//!
//! `crate::markdown` の `parse_fence` から呼ばれ、Rust / TOML / HTML の
//! コード例へ `<span class="token-*">` を挿入して色分け表示できるようにする。
//! 外部依存ゼロ・JS ハイドレーションなし（親 #1059 の制約）という前提の下、
//! 「ビルド時に Rust の簡易トークナイザで `span` を挿入し CSS で色を当てる」
//! 以外の実現手段を採らない。
//!
//! # 全域性不変条件（本モジュールの安全性の起点）
//!
//! [`tokenize`] が `Some` を返す場合、返した `Vec<Token>` について
//! `tokens.iter().map(|t| t.text).collect::<String>() == src` が常に成り立つ
//! （入力の全バイトが順序どおりちょうど 1 回だけ現れる。欠落も重複も改変も
//! ない）。この不変条件から以下が導かれる。
//!
//! - **REQ-1（既定エスケープ）が構造的に不変**: すべてのトークンは
//!   [`fandhe_frontend_core::text`] として出力される。span で包むトークンは
//!   `span(vec![("class", "token-…")], vec![text(t.text)])`、包まないトークン
//!   （空白・記号・識別子）は `text(t.text)` をそのまま子として並べる。
//!   `raw_html()` は一切使わず、HTML 文字列の `format!` 組み立ても行わない
//!   （`crate::markdown` モジュール doc のセキュリティ不変条件をそのまま継承する）。
//! - **`<pre>` の表示忠実性**: 空白・改行が 1 バイトも欠落しないため、色分けの
//!   有無でコードの見え方が変わらない。
//! - **fail-safe が 1 機構で完結**: 「未対応言語」「全域性検証の失敗」
//!   「上限超過」のすべてを同一のフォールバック（[`highlight_children`] が
//!   `None` を返し、呼び出し元が従来どおり `text(content)` を使う）へ収束させる。
//!
//! `class` 属性値は [`TokenKind::class`] が返す `&'static str` 定数のみで、
//! 入力由来の文字列がタグ名・属性名・属性値へ流れる経路は存在しない
//! （HTML タグ名・属性名の抽出結果もすべて [`text`] を通るため、コード
//! ブロック内に HTML が書かれていても構文として解釈されることはない）。
//!
//! 入力の信頼レベルは `crate::markdown` の方針を継承する: `docs/` 配下が
//! リポジトリ管理下であっても、本モジュールの入力は常に「信頼できない
//! Markdown」として扱う。

use fandhe_frontend_core::{span, text, Node};

/// トークン化対象のソースバイト数上限（OWASP A04: アルゴリズム的計算量 DoS 対策）。
///
/// 実測コーパス（`docs/**` の Rust フェンス）は数百〜数千バイトが大半であり、
/// 64 KiB は正当なコード例の解釈結果に影響しない。超過した場合は
/// [`tokenize`] が `None` を返し、[`highlight_children`] 経由でプレーン表示へ
/// フォールバックする。
const MAX_SOURCE_BYTES: usize = 64 * 1024;

/// 生成トークン数の上限（OWASP A04 対策）。
///
/// `"a""a""a"…` のような病的な入力に対し、1 文字ごとに `span` トークンが
/// 生成され続けると HTML 出力サイズが入力サイズに対して線形以上に膨張し得る。
/// 4096 は実コーパスの最大トークン数を大きく超えており、通常の使用では
/// 到達しない。
const MAX_TOKENS: usize = 4096;

/// トークン種別。CSS クラス名（`token-*`）と 1:1 対応する。
///
/// `crate::site_theme::highlight_css` が [`TokenKind::ALL`] を走査して CSS
/// セレクタ網羅性を機械検証する（`tests/highlight.rs` 参照）ため、新しい
/// 種別を追加する場合は必ず [`TokenKind::ALL`] にも追加すること。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// span で包まない（空白・記号・未分類識別子）。既存の言語未指定
    /// フェンスと同じ「無色のテキスト」として出力される。
    Plain,
    /// 予約語（`fn` `let` `if` 等、TOML の `true`/`false` を含む）。
    Keyword,
    /// 文字列・文字リテラル。
    String,
    /// 行コメント・複数行コメント。
    Comment,
    /// 数値リテラル。
    Number,
    /// HTML タグ名 / TOML テーブル見出し（`[table]`）。
    Tag,
    /// HTML 属性名 / TOML キー名（`key =` の左辺）。
    Attr,
}

impl TokenKind {
    /// 全種別（`ALL` を回すテストが enum 駆動でドリフトを検知する唯一の
    /// 情報源）。
    pub const ALL: &'static [TokenKind] = &[
        TokenKind::Plain,
        TokenKind::Keyword,
        TokenKind::String,
        TokenKind::Comment,
        TokenKind::Number,
        TokenKind::Tag,
        TokenKind::Attr,
    ];

    /// `Plain` は `None`（span を付与しない）。それ以外は対応する
    /// `token-*` クラス名を返す。
    pub fn class(self) -> Option<&'static str> {
        match self {
            TokenKind::Plain => None,
            TokenKind::Keyword => Some("token-keyword"),
            TokenKind::String => Some("token-string"),
            TokenKind::Comment => Some("token-comment"),
            TokenKind::Number => Some("token-number"),
            TokenKind::Tag => Some("token-tag"),
            TokenKind::Attr => Some("token-attr"),
        }
    }
}

/// 対応言語。フェンス info string の第 1 トークンから解決する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Toml,
    Html,
}

impl Language {
    /// info string の言語トークンを ASCII 小文字化して完全一致で照合する
    /// allow-list。未知の言語は `None`（= プレーン表示へフォールバック）。
    ///
    /// エイリアス（`rs` / `htm`）は意図的に非対応（コーパス実測で不使用、
    /// イシュー #1078 スコープ外として PR 本文に記録する）。
    pub fn from_token(token: &str) -> Option<Language> {
        // ASCII 小文字化のみ行う（既存フェンス言語トークンは ASCII の
        // 実用的な範囲のみ想定。`is_valid_lang_token` の許容文字集合
        // （英数字・`_`・`+`・`.`・`-`）を前提にしており、非 ASCII を
        // 誤って変換する余地はない）。
        match token.to_ascii_lowercase().as_str() {
            "rust" => Some(Language::Rust),
            "toml" => Some(Language::Toml),
            "html" => Some(Language::Html),
            _ => None,
        }
    }
}

/// トークン化結果 1 件。`text` は `src` の部分スライス（借用のみ、コピーしない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub text: &'a str,
}

/// `src` を `lang` の規則でトークン化する。
///
/// 全域性不変条件（モジュール doc 参照）を満たさない場合・上限
/// （[`MAX_SOURCE_BYTES`] / [`MAX_TOKENS`]）を超える場合は `None` を返し、
/// 呼び出し元はプレーン表示へ倒す（設計上、`None` はエラーではなく
/// 「色分け適用対象外」の意図的な結果）。
pub fn tokenize(src: &str, lang: Language) -> Option<Vec<Token<'_>>> {
    if src.len() > MAX_SOURCE_BYTES {
        return None;
    }
    let tokens = match lang {
        Language::Rust => tokenize_rust(src)?,
        Language::Toml => tokenize_toml(src),
        Language::Html => tokenize_html(src),
    };
    if tokens.len() > MAX_TOKENS {
        return None;
    }
    // 全域性の最終検証: concat した結果が src と完全一致しない実装バグは
    // 表示崩れ・情報欠落に直結するため、ここで機械的に遮断してプレーンへ
    // 倒す（テストでも別途検証するが、本番の入力に対しても同じ保証を
    // 実行時に効かせる）。
    let rebuilt: usize = tokens.iter().map(|t| t.text.len()).sum();
    if rebuilt != src.len() {
        return None;
    }
    let mut offset = 0usize;
    for t in &tokens {
        let piece = &src[offset..offset + t.text.len()];
        if piece != t.text {
            return None;
        }
        offset += t.text.len();
    }
    Some(tokens)
}

/// `crate::markdown::parse_fence` 向けの唯一の入口。
///
/// `lang_token` が [`Language::from_token`] で解決できない、または
/// [`tokenize`] が `None` を返す場合は `None` を返す（呼び出し元の分岐を
/// 1 本にする。「未対応言語」と「トークナイズ失敗」を区別する必要が
/// 呼び出し元にはない）。
pub fn highlight_children(src: &str, lang_token: &str) -> Option<Vec<Node>> {
    let lang = Language::from_token(lang_token)?;
    let tokens = tokenize(src, lang)?;
    Some(
        tokens
            .into_iter()
            .map(|t| match t.kind.class() {
                Some(class) => span(vec![("class", class)], vec![text(t.text)]),
                None => text(t.text),
            })
            .collect(),
    )
}

/// カーソル位置から識別子（`[A-Za-z_][A-Za-z0-9_]*`）を読み取り、消費した
/// バイト長を返す。識別子でなければ `0`。
fn ident_len(bytes: &[u8]) -> usize {
    if bytes.is_empty() {
        return 0;
    }
    let first = bytes[0];
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return 0;
    }
    let mut n = 1;
    while n < bytes.len() && (bytes[n].is_ascii_alphanumeric() || bytes[n] == b'_') {
        n += 1;
    }
    n
}

/// カーソル位置から連続する空白（改行含む）を読み取り、消費したバイト長を
/// 返す。空白でなければ `0`。
fn whitespace_len(bytes: &[u8]) -> usize {
    let mut n = 0;
    while n < bytes.len() && (bytes[n] as char).is_whitespace() {
        n += 1;
    }
    n
}

const RUST_KEYWORDS: &[&str] = &[
    "fn", "let", "mut", "pub", "struct", "enum", "impl", "trait", "use", "mod", "const", "static",
    "if", "else", "match", "for", "while", "loop", "return", "self", "Self", "crate", "super",
    "as", "where", "type", "dyn", "move", "ref", "unsafe", "async", "await", "in", "break",
    "continue", "true", "false",
];

/// `src[start]` が `'` であるとき、そこから始まる文字リテラルの終端
/// （排他的、`src` のバイトオフセット）を返す。妥当な文字リテラルの形
/// （非バックスラッシュの 1 文字、または既知のエスケープシーケンス 1 個の
/// 直後に必ず閉じクォート `'` が続く）を満たさない場合は `None`（＝ライフ
/// タイムとして扱う）。
///
/// 「次の `'` まで貪欲に探す」方式は `&'a str` の直後にもう 1 つのライフ
/// タイム（`&'a`）や文字リテラルが現れる入力（例: `fn f<'a>(s: &'a str)`）
/// で誤って 2 つのライフタイムをまたいだ 1 個の「文字列」として飲み込んで
/// しまう。本関数は「妥当な文字リテラルの形」のみを認めることでこれを
/// 避ける（[`tokenize_rust`] の `'` 分岐から呼ばれる）。
///
/// 対応エスケープ: `\n` `\r` `\t` `\0` `\\` `\'` `\"` `\xNN`（2 桁 16 進）
/// `\u{…}`（最大 6 桁 16 進）。未知のエスケープ（例: `\p`）は文字リテラルと
/// 認めず `None` を返す（安全側 = ライフタイム扱い）。走査は `\u{…}` の桁数
/// 上限で打ち切り、閉じ `}` の無い入力でも O(1) で確定する（OWASP A04 対策）。
fn rust_char_literal_end(src: &str, start: usize) -> Option<usize> {
    let rest = &src[start + 1..];
    let mut chars = rest.char_indices();
    let (_, c0) = chars.next()?;

    if c0 == '\'' || c0 == '\n' {
        // 空リテラル `''` や改行を跨ぐものは妥当な文字リテラルではない。
        return None;
    }

    // content_end: `rest` 内で「文字リテラルの中身」が終わる直後のオフセット。
    let content_end = if c0 == '\\' {
        let (i1, c1) = chars.next()?;
        let after_c1 = i1 + c1.len_utf8();
        match c1 {
            'n' | 'r' | 't' | '0' | '\\' | '\'' | '"' => after_c1,
            'x' => {
                let (_, h1) = chars.next()?;
                let (i2, h2) = chars.next()?;
                if !h1.is_ascii_hexdigit() || !h2.is_ascii_hexdigit() {
                    return None;
                }
                i2 + h2.len_utf8()
            }
            'u' => {
                let (_, brace) = chars.next()?;
                if brace != '{' {
                    return None;
                }
                // Unicode コードポイントは最大 6 桁の 16 進数。上限を超えて
                // 桁が続く、または `}` で閉じない場合は打ち切って `None`。
                const MAX_HEX_DIGITS: usize = 6;
                let mut digits = 0usize;
                loop {
                    let (i, c) = chars.next()?;
                    if c == '}' {
                        break i + c.len_utf8();
                    }
                    if !c.is_ascii_hexdigit() || digits >= MAX_HEX_DIGITS {
                        return None;
                    }
                    digits += 1;
                }
            }
            _ => return None,
        }
    } else {
        c0.len_utf8()
    };

    // content_end の直後（rest 内オフセット）に閉じクォートがあるかを確認する。
    let (close_idx, close_char) = rest[content_end..].char_indices().next()?;
    if close_char != '\'' || close_idx != 0 {
        return None;
    }
    Some(start + 1 + content_end + close_char.len_utf8())
}

/// Rust フェンス本文をトークン化する。
///
/// 単一パス・全分岐でカーソルを 1 バイト以上前進させる（`debug_assert!` で
/// 無限ループを禁止する。OWASP A04 対策、`crate::markdown` の
/// `MAX_INLINE_SCAN_WINDOW` と同じ設計思想）。
///
/// - raw string（`r"…"` / `r#"…"#`）は非対応。`r#"` 開始を検出したら
///   ブロック全体をプレーンへ倒す保守的分岐を取る（`#` を含む raw string の
///   終端規則を正確に実装しないことによる着色崩れを避ける、イシュー #1078
///   スコープ外）
/// - `'` はライフタイム（`'a` `'static`）か文字リテラル（`'x'` `'\n'`）かを
///   [`rust_char_literal_end`] が判別する（`&'a str` を誤って文字列開始と
///   みなすと後続が壊れるため）
/// - `/* … */` はネスト非対応（最初の `*/` で閉じる）
/// - 文字列・複数行コメントが未終端のまま EOF に達した場合は、残余をその
///   種別のトークンとして確定して終了する（取りこぼしを起こさない fail-safe）
fn tokenize_rust(src: &str) -> Option<Vec<Token<'_>>> {
    let bytes = src.as_bytes();
    let len = bytes.len();
    let mut tokens: Vec<Token<'_>> = Vec::new();
    let mut i = 0usize;

    while i < len {
        let start = i;
        let rest = &bytes[i..];

        // 行コメント: `//`（`///` `//!` も同種別として一括で行末まで）。
        if rest.starts_with(b"//") {
            let mut j = i + 2;
            while j < len && bytes[j] != b'\n' {
                j += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Comment,
                text: &src[start..j],
            });
            i = j;
            debug_assert!(i > start);
            continue;
        }

        // 複数行コメント: `/* … */`（ネスト非対応、最初の `*/` で閉じる）。
        if rest.starts_with(b"/*") {
            let close = src[i + 2..].find("*/");
            let end = match close {
                Some(off) => i + 2 + off + 2,
                None => len, // 未終端 EOF: 残余全体をコメントとして確定する。
            };
            tokens.push(Token {
                kind: TokenKind::Comment,
                text: &src[start..end],
            });
            i = end;
            debug_assert!(i > start);
            continue;
        }

        // raw string（`r"..."` / `r#"..."#`）: 保守的にブロック全体を
        // プレーンへ倒す（このフェンス本文の色分けを諦め、上位の
        // `tokenize` から `None` として扱わせる）。
        if rest.starts_with(b"r\"") || rest.starts_with(b"r#") {
            // `r#` の直後が `"`（raw string）である場合のみ倒す。
            // `r#ident#`（存在しない構文だが安全側）誤爆を避けるため、
            // `r"` 直後 か `r#...#"` パターンの先頭一致のみで判定する。
            if rest.starts_with(b"r\"") {
                return None;
            }
            // `r#` の後、`#`* の後に `"` が続くかを確認する。
            let mut k = i + 1;
            while k < len && bytes[k] == b'#' {
                k += 1;
            }
            if k < len && bytes[k] == b'"' {
                return None;
            }
            // `r#` だが raw string ではない場合（raw identifier `r#ident`、
            // 例: `r#fn`）は `r#` + 識別子をここで 1 個の Plain トークンと
            // して確定させる。下の識別子分岐へフォールスルーさせると `r#`
            // の `r` だけが識別子として切り出され、続く `#` が個別の記号
            // トークンとして消費されたのち `fn` 等のキーワード同名部分が
            // 独立した識別子として再走査され `Keyword` に誤分類される
            // （レビュー指摘: `let r#fn = 1;` が `r`/`#`/`fn(Keyword)` に
            // 分割される色分け崩れ）。raw identifier は Rust の構文上
            // キーワードと同名でも通常の識別子として扱われるため、常に
            // `Plain` として一体で消費することでこれを避ける。
            let after_hash_len = ident_len(&bytes[i + 2..]);
            if after_hash_len > 0 {
                let end = i + 2 + after_hash_len;
                tokens.push(Token {
                    kind: TokenKind::Plain,
                    text: &src[start..end],
                });
                i = end;
                debug_assert!(i > start);
                continue;
            }
            // `r#` の直後に識別子が続かない異常入力（存在しない構文だが
            // 安全側）: 下の識別子分岐へフォールスルーし、`r` のみを
            // 通常の識別子として処理する。
        }

        // 文字列リテラル: `"…"`（`\` エスケープ考慮）。
        if bytes[start] == b'"' {
            let mut j = start + 1;
            while j < len {
                if bytes[j] == b'\\' && j + 1 < len {
                    j += 2;
                    continue;
                }
                if bytes[j] == b'"' {
                    j += 1;
                    break;
                }
                j += 1;
            }
            tokens.push(Token {
                kind: TokenKind::String,
                text: &src[start..j],
            });
            i = j;
            debug_assert!(i > start);
            continue;
        }

        // `'`: ライフタイムか文字リテラルかを判別する。
        //
        // 「次の `'` まで貪欲に探す」方式は `&'a str` の直後にもう 1 つの
        // ライフタイム（`&'a`）や文字リテラルが現れる入力（例:
        // `fn f<'a>(s: &'a str)`）で誤って 2 つのライフタイムをまたいだ
        // 1 個の「文字列」として飲み込んでしまう（レビュー指摘で発覚した
        // 不具合）。[`rust_char_literal_end`] は「妥当な文字リテラルの形
        // （1 文字 or 1 エスケープシーケンスの直後に必ず閉じクォートが続く）」
        // のみを文字リテラルと認め、それ以外はライフタイム（`Plain`、`'` 1 文字
        // のみ消費）として扱う。
        if bytes[start] == b'\'' {
            if let Some(end) = rust_char_literal_end(src, start) {
                tokens.push(Token {
                    kind: TokenKind::String,
                    text: &src[start..end],
                });
                i = end;
            } else {
                tokens.push(Token {
                    kind: TokenKind::Plain,
                    text: &src[start..start + 1],
                });
                i = start + 1;
            }
            debug_assert!(i > start);
            continue;
        }

        // 数値リテラル: `[0-9]` で始まる `[0-9A-Za-z_.]` の連続。
        if bytes[start].is_ascii_digit() {
            let mut j = start;
            while j < len
                && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'.')
            {
                j += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Number,
                text: &src[start..j],
            });
            i = j;
            debug_assert!(i > start);
            continue;
        }

        // 識別子: キーワード完全一致なら Keyword、それ以外は Plain。
        let id_len = ident_len(rest);
        if id_len > 0 {
            let word = &src[start..start + id_len];
            let kind = if RUST_KEYWORDS.contains(&word) {
                TokenKind::Keyword
            } else {
                TokenKind::Plain
            };
            tokens.push(Token { kind, text: word });
            i = start + id_len;
            debug_assert!(i > start);
            continue;
        }

        // 空白: 連続する空白をまとめて Plain として消費する。
        let ws_len = whitespace_len(rest);
        if ws_len > 0 {
            tokens.push(Token {
                kind: TokenKind::Plain,
                text: &src[start..start + ws_len],
            });
            i = start + ws_len;
            debug_assert!(i > start);
            continue;
        }

        // その他 1 バイト（記号等）は Plain として個別に消費する
        // （UTF-8 文字境界違反を避けるため `char` 単位で進める）。
        let ch_len = src[start..].chars().next().map(char::len_utf8).unwrap_or(1);
        tokens.push(Token {
            kind: TokenKind::Plain,
            text: &src[start..start + ch_len],
        });
        i = start + ch_len;
        debug_assert!(i > start);
    }

    Some(tokens)
}

/// TOML フェンス本文をトークン化する。
///
/// - `#` 行コメント
/// - `"…"` / `'…'` 文字列
/// - 行頭 `[table]` / `[[array]]` を `Tag`
/// - `key =` の左辺（行頭からの識別子・記号列）を `Attr`
/// - `true` / `false` を `Keyword`
/// - 数値・日付（`[0-9]` 始まりの `[0-9A-Za-z_.:+-]` 連続）を `Number`
fn tokenize_toml(src: &str) -> Vec<Token<'_>> {
    let bytes = src.as_bytes();
    let len = bytes.len();
    let mut tokens: Vec<Token<'_>> = Vec::new();
    let mut i = 0usize;
    let mut at_line_start = true;

    while i < len {
        let start = i;
        let rest = &bytes[i..];

        if bytes[start] == b'\n' {
            tokens.push(Token {
                kind: TokenKind::Plain,
                text: &src[start..start + 1],
            });
            i = start + 1;
            at_line_start = true;
            continue;
        }

        // 行頭の空白（インデント）は at_line_start を維持したまま消費する。
        if at_line_start && (bytes[start] == b' ' || bytes[start] == b'\t') {
            let ws_len = whitespace_len(rest);
            // 改行はここでは含めない（whitespace_len は改行も空白扱いする
            // ため、改行に当たる直前までに制限する）。
            let mut j = start;
            while j < len && bytes[j] != b'\n' && (bytes[j] as char).is_whitespace() {
                j += 1;
            }
            let _ = ws_len;
            tokens.push(Token {
                kind: TokenKind::Plain,
                text: &src[start..j],
            });
            i = j;
            continue;
        }

        if bytes[start] == b'#' {
            let mut j = start;
            while j < len && bytes[j] != b'\n' {
                j += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Comment,
                text: &src[start..j],
            });
            i = j;
            at_line_start = false;
            continue;
        }

        // 行頭のテーブル見出し `[table]` / `[[array]]`。
        if at_line_start && bytes[start] == b'[' {
            let mut j = start;
            while j < len && bytes[j] != b'\n' {
                j += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Tag,
                text: &src[start..j],
            });
            i = j;
            at_line_start = false;
            continue;
        }

        if bytes[start] == b'"' || bytes[start] == b'\'' {
            let quote = bytes[start];
            let mut j = start + 1;
            while j < len {
                if quote == b'"' && bytes[j] == b'\\' && j + 1 < len {
                    j += 2;
                    continue;
                }
                if bytes[j] == quote {
                    j += 1;
                    break;
                }
                if bytes[j] == b'\n' {
                    break;
                }
                j += 1;
            }
            tokens.push(Token {
                kind: TokenKind::String,
                text: &src[start..j],
            });
            i = j;
            at_line_start = false;
            continue;
        }

        // 行頭の識別子は `key =` の左辺（Attr）とみなす。
        let id_len = ident_len(rest);
        if id_len > 0 {
            let word = &src[start..start + id_len];
            if at_line_start {
                tokens.push(Token {
                    kind: TokenKind::Attr,
                    text: word,
                });
            } else if word == "true" || word == "false" {
                tokens.push(Token {
                    kind: TokenKind::Keyword,
                    text: word,
                });
            } else {
                tokens.push(Token {
                    kind: TokenKind::Plain,
                    text: word,
                });
            }
            i = start + id_len;
            at_line_start = false;
            continue;
        }

        if bytes[start].is_ascii_digit() {
            let mut j = start;
            while j < len
                && (bytes[j].is_ascii_alphanumeric()
                    || matches!(bytes[j], b'_' | b'.' | b':' | b'+' | b'-'))
            {
                j += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Number,
                text: &src[start..j],
            });
            i = j;
            at_line_start = false;
            continue;
        }

        let ws_len = whitespace_len(rest);
        if ws_len > 0 && bytes[start] != b'\n' {
            let mut j = start;
            while j < len && bytes[j] != b'\n' && (bytes[j] as char).is_whitespace() {
                j += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Plain,
                text: &src[start..j],
            });
            i = j;
            at_line_start = false;
            continue;
        }

        let ch_len = src[start..].chars().next().map(char::len_utf8).unwrap_or(1);
        tokens.push(Token {
            kind: TokenKind::Plain,
            text: &src[start..start + ch_len],
        });
        i = start + ch_len;
        at_line_start = false;
    }

    tokens
}

/// HTML フェンス本文をトークン化する。
///
/// - `<!-- … -->` を `Comment`
/// - `<tag` `</tag` の後続タグ名を `Tag`
/// - タグ内の属性名を `Attr`
/// - `"…"` / `'…'` 属性値を `String`
///
/// タグ名・属性名は Rust 側では単なる文字列スライスであり、出力は必ず
/// [`fandhe_frontend_core::text`] を経由するため HTML として解釈されることは
/// ない（コードブロック脱出の経路が増えないことをここに明記する）。
fn tokenize_html(src: &str) -> Vec<Token<'_>> {
    let bytes = src.as_bytes();
    let len = bytes.len();
    let mut tokens: Vec<Token<'_>> = Vec::new();
    let mut i = 0usize;
    let mut in_tag = false;
    let mut expect_tag_name = false;

    while i < len {
        let start = i;
        let rest = &bytes[i..];

        if rest.starts_with(b"<!--") {
            let close = src[i + 4..].find("-->");
            let end = match close {
                Some(off) => i + 4 + off + 3,
                None => len,
            };
            tokens.push(Token {
                kind: TokenKind::Comment,
                text: &src[start..end],
            });
            i = end;
            continue;
        }

        if bytes[start] == b'<' {
            in_tag = true;
            expect_tag_name = true;
            let mut j = start + 1;
            if j < len && bytes[j] == b'/' {
                j += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Plain,
                text: &src[start..j],
            });
            i = j;
            continue;
        }

        if in_tag && (bytes[start] == b'>' || rest.starts_with(b"/>")) {
            let end = if rest.starts_with(b"/>") {
                start + 2
            } else {
                start + 1
            };
            tokens.push(Token {
                kind: TokenKind::Plain,
                text: &src[start..end],
            });
            i = end;
            in_tag = false;
            expect_tag_name = false;
            continue;
        }

        if in_tag && (bytes[start] == b'"' || bytes[start] == b'\'') {
            let quote = bytes[start];
            let mut j = start + 1;
            while j < len && bytes[j] != quote {
                j += 1;
            }
            if j < len {
                j += 1; // 閉じクォートを含める。
            }
            tokens.push(Token {
                kind: TokenKind::String,
                text: &src[start..j],
            });
            i = j;
            continue;
        }

        if in_tag {
            let id_len = ident_len_html(rest);
            if id_len > 0 {
                let word = &src[start..start + id_len];
                let kind = if expect_tag_name {
                    TokenKind::Tag
                } else {
                    TokenKind::Attr
                };
                tokens.push(Token { kind, text: word });
                i = start + id_len;
                expect_tag_name = false;
                continue;
            }
        }

        let ws_len = whitespace_len(rest);
        if ws_len > 0 {
            tokens.push(Token {
                kind: TokenKind::Plain,
                text: &src[start..start + ws_len],
            });
            i = start + ws_len;
            continue;
        }

        let ch_len = src[start..].chars().next().map(char::len_utf8).unwrap_or(1);
        tokens.push(Token {
            kind: TokenKind::Plain,
            text: &src[start..start + ch_len],
        });
        i = start + ch_len;
    }

    tokens
}

/// HTML タグ名・属性名向けの識別子読み取り。`-` を許容する点が
/// [`ident_len`]（Rust 識別子規則）と異なる（`data-*` 属性名・カスタム要素名
/// のため）。
fn ident_len_html(bytes: &[u8]) -> usize {
    if bytes.is_empty() {
        return 0;
    }
    let first = bytes[0];
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return 0;
    }
    let mut n = 1;
    while n < bytes.len()
        && (bytes[n].is_ascii_alphanumeric() || matches!(bytes[n], b'_' | b'-' | b':'))
    {
        n += 1;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`tokenize`] の結果を連結すると常に入力へ戻ることを検証する
    /// （全域性不変条件、モジュール doc 参照）共通ヘルパ。
    fn assert_totality(src: &str, lang: Language) -> Vec<Token<'_>> {
        let tokens = tokenize(src, lang).expect("tokenize should not fail for this fixture");
        let rebuilt: String = tokens.iter().map(|t| t.text).collect();
        assert_eq!(rebuilt, src, "totality violated for lang={lang:?}");
        tokens
    }

    #[test]
    fn rust_totality_basic() {
        assert_totality(
            "fn main() {\n    let x = 1;\n    println!(\"{}\", x);\n}\n",
            Language::Rust,
        );
    }

    #[test]
    fn rust_totality_japanese_comment_and_string() {
        assert_totality("// 日本語コメント\nlet s = \"日本語\";\n", Language::Rust);
    }

    #[test]
    fn rust_keyword_detection() {
        let tokens = assert_totality("fn f() {}", Language::Rust);
        assert!(tokens
            .iter()
            .any(|t| t.text == "fn" && t.kind == TokenKind::Keyword));
    }

    #[test]
    fn rust_fnord_is_not_a_keyword() {
        let tokens = assert_totality("let fnord = 1;", Language::Rust);
        assert!(tokens
            .iter()
            .any(|t| t.text == "fnord" && t.kind != TokenKind::Keyword));
    }

    #[test]
    fn rust_lifetime_vs_char_literal() {
        let tokens = assert_totality("fn f<'a>(s: &'a str) -> char { 'x' }", Language::Rust);
        // ライフタイム `'a` は文字列扱いされない(Plain のまま)。
        assert!(tokens
            .iter()
            .any(|t| t.text == "'" && t.kind == TokenKind::Plain));
        // 文字リテラル `'x'` は String として一括で出る。
        assert!(tokens
            .iter()
            .any(|t| t.text == "'x'" && t.kind == TokenKind::String));
    }

    #[test]
    fn rust_char_literal_escapes() {
        let tokens = assert_totality("let a = '\\n'; let b = '\\u{1F600}';", Language::Rust);
        assert!(tokens
            .iter()
            .any(|t| t.text == "'\\n'" && t.kind == TokenKind::String));
        assert!(tokens
            .iter()
            .any(|t| t.text == "'\\u{1F600}'" && t.kind == TokenKind::String));
    }

    #[test]
    fn rust_multiline_comment() {
        let tokens = assert_totality("/* a\nb */\nlet x = 1;", Language::Rust);
        assert!(tokens
            .iter()
            .any(|t| t.text == "/* a\nb */" && t.kind == TokenKind::Comment));
    }

    #[test]
    fn rust_unterminated_string_reaches_eof() {
        let tokens = assert_totality("let s = \"abc", Language::Rust);
        assert!(tokens
            .iter()
            .any(|t| t.text == "\"abc" && t.kind == TokenKind::String));
    }

    #[test]
    fn rust_unterminated_block_comment_reaches_eof() {
        let tokens = assert_totality("/* never closes", Language::Rust);
        assert!(tokens
            .iter()
            .any(|t| t.text == "/* never closes" && t.kind == TokenKind::Comment));
    }

    #[test]
    fn rust_raw_string_falls_back_to_none() {
        assert!(tokenize("let s = r#\"raw\"#;", Language::Rust).is_none());
        assert!(tokenize("let s = r\"raw\";", Language::Rust).is_none());
    }

    #[test]
    fn rust_raw_identifier_is_single_plain_token() {
        // raw identifier（`r#fn` のようなキーワード同名の識別子）は `r#` +
        // 識別子全体を 1 個の Plain トークンとして扱い、`fn` 部分だけが
        // 独立した Keyword トークンとして誤分類されないことを確認する
        // （レビュー指摘の回帰テスト、イシュー #1078）。
        let tokens = assert_totality("let r#fn = 1;", Language::Rust);
        assert!(tokens
            .iter()
            .any(|t| t.text == "r#fn" && t.kind == TokenKind::Plain));
        assert!(!tokens
            .iter()
            .any(|t| t.text == "fn" && t.kind == TokenKind::Keyword));

        // `r#` の直後に識別子が続かない異常入力（存在しない構文だが安全側）
        // は `r` のみを通常の識別子として処理するフォールスルー分岐を通る。
        // 全域性（`assert_totality`）が崩れないことを確認する。
        assert_totality("let r#", Language::Rust);
        assert_totality("let r#1 = 1;", Language::Rust);
    }

    #[test]
    fn rust_number_literals() {
        let tokens = assert_totality("let x = 0xFF_u8 + 1.5f64;", Language::Rust);
        assert!(tokens
            .iter()
            .any(|t| t.text == "0xFF_u8" && t.kind == TokenKind::Number));
        assert!(tokens
            .iter()
            .any(|t| t.text == "1.5f64" && t.kind == TokenKind::Number));
    }

    #[test]
    fn toml_totality_basic() {
        assert_totality(
            "# comment\n[table]\nkey = \"value\"\nflag = true\nnum = 42\n",
            Language::Toml,
        );
    }

    #[test]
    fn toml_key_and_table_kinds() {
        let tokens = assert_totality("[table]\nkey = \"v\"\n", Language::Toml);
        assert!(tokens
            .iter()
            .any(|t| t.text == "[table]" && t.kind == TokenKind::Tag));
        assert!(tokens
            .iter()
            .any(|t| t.text == "key" && t.kind == TokenKind::Attr));
    }

    #[test]
    fn html_totality_basic() {
        assert_totality(
            "<!-- c -->\n<div class=\"a\" data-x='y'>text</div>\n",
            Language::Html,
        );
    }

    #[test]
    fn html_tag_and_attr_kinds() {
        let tokens = assert_totality("<div class=\"a\">", Language::Html);
        assert!(tokens
            .iter()
            .any(|t| t.text == "div" && t.kind == TokenKind::Tag));
        assert!(tokens
            .iter()
            .any(|t| t.text == "class" && t.kind == TokenKind::Attr));
        assert!(tokens
            .iter()
            .any(|t| t.text == "\"a\"" && t.kind == TokenKind::String));
    }

    #[test]
    fn highlight_children_none_for_unsupported_language() {
        assert!(highlight_children("echo hi", "bash").is_none());
        assert!(highlight_children("echo hi", "").is_none());
        assert!(highlight_children("echo hi", "json").is_none());
    }

    #[test]
    fn highlight_children_some_for_rust() {
        assert!(highlight_children("fn main() {}", "rust").is_some());
    }

    #[test]
    fn tokenize_over_size_limit_returns_none() {
        let huge = "a".repeat(MAX_SOURCE_BYTES + 1);
        assert!(tokenize(&huge, Language::Rust).is_none());
    }

    #[test]
    fn token_kind_all_covers_every_variant() {
        // enum に新しい variant を追加したら ALL への追加漏れをここで検知する
        // （網羅性は match の exhaustiveness にも間接的に依存する）。
        for kind in TokenKind::ALL {
            match kind {
                TokenKind::Plain
                | TokenKind::Keyword
                | TokenKind::String
                | TokenKind::Comment
                | TokenKind::Number
                | TokenKind::Tag
                | TokenKind::Attr => {}
            }
        }
        assert_eq!(TokenKind::ALL.len(), 7);
    }
}
