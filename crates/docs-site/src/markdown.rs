//! Markdown ブロック構文を `fandhe_frontend_core::Node` 木へレンダリングするパーサ。
//!
//! `docs/` 配下の Markdown ファイルを本フレームワーク自身のノード木 API で
//! HTML 化する docs サイトジェネレータ（イシュー #458 Phase 2-3/2-4）の中核部品。
//! 生成した `Vec<Node>` は後続（Phase 3 の `layout.rs`）が `article` 等の
//! コンテナへ包んでページ全体を組み立てる想定であり、本モジュールはブロック
//! レベルのノード列を返すところまでを責務とする。
//!
//! # 安全性契約（REQ-1: 既定エスケープ）
//!
//! - 出力するテキストはすべて [`fandhe_frontend_core::text`]（`Node::Text`、
//!   `render()` 時に必ず `escape_html_into` を経由）を通す。`raw_html()` は
//!   本モジュールでは一切使用しない
//! - HTML 文字列を `format!` 等で直接組み立てることもしない。ノード木 API
//!   （`crates/core/src/tags.rs` のショートカット関数）のみを用いる
//! - Markdown 中に生の HTML タグ（例: `<div>` `<script>`）が現れても構文として
//!   解釈せず、テキストとしてそのままエスケープ経路へ渡す（HTML ブロック非対応）
//! - 外部入力（信頼できない Markdown）を将来受け取る可能性を見越し、本モジュール
//!   の入力は常に「信頼できない入力」として扱う（`docs/` 配下はリポジトリ管理下だが
//!   脅威モデル上の扱いは緩めない）
//! - インライン構文の閉じマーカー探索（[`find_closing_run`] / [`find_char`]）
//!   は 1 回の探索でスキャンする文字数を [`MAX_INLINE_SCAN_WINDOW`] に
//!   制限する。無制限に末尾まで走査すると、閉じマーカーが見つからない
//!   `*` / `` ` `` / `[` の連続に対し開始位置ごとの走査が最悪 O(n^2) の
//!   アルゴリズム的計算量 DoS（OWASP A04）を招くため、上限超過分は
//!   「閉じマーカーなし」と同じ fail-safe フォールバック（リテラル文字
//!   として扱う）で打ち切る
//! - インライン構文（イシュー #467）のリンク URL は `is_safe_link_url`
//!   （本モジュール内、第 1 層: http/https/相対のみ許可）→ core の
//!   [`fandhe_frontend_core::is_safe_url`]（第 2 層、`render_into` が属性出力時に
//!   適用）の多層で検証する。不合格の URL は `<a>` を生成せずリンクテキストのみを
//!   出力する（fail-closed）。属性値自体も core が出力時にエスケープするため
//!   `"` によるリンクテキスト/URL からの属性 breakout は core 側でも遮断される
//!
//! パニックしない全域関数として実装する（ライブラリコードでの `unwrap()` /
//! `panic!` 回避規約、`.claude/rules/coding-rust.md`）。未知の行・不正な構文は
//! 段落として扱うフォールバックにより、任意の `&str` を受理する。

use fandhe_frontend_core::{
    a, blockquote, code, em, h1, h2, h3, h4, h5, h6, is_safe_url, li, ol, p, pre, strong, table,
    tbody, td, text, th, thead, tr, ul, Node,
};

/// 引用・ネストリストの再帰的解釈における最大深さ。
///
/// 悪意ある/破損した入力（例: `>` を数千行連続させた入力）によるスタック
/// オーバーフローを防ぐための DoS 対策（OWASP A04 安全でない設計）。超過分は
/// 通常の段落テキストとして扱い、取りこぼしをしない fail-safe とする。
const MAX_DEPTH: usize = 16;

/// インライン構文の閉じマーカー探索（[`find_closing_run`] / [`find_char`]）
/// が開始位置から前方走査する文字数の上限（レビュー指摘イシュー #467、
/// OWASP A04 安全でない設計 — アルゴリズム的計算量 DoS 対策）。
///
/// 閉じマーカーが見つからない `*` / `` ` `` / `[` の連続に対し、この上限
/// なしでは各開始位置ごとに残り入力の末尾まで走査するため最悪計算量が
/// O(n^2) になる（実測: 全て `*` の入力で release ビルド n=40,000 のとき
/// 約 156ms、n=262,144 で約 6.5s、debug ビルドはさらに数倍遅い）。本定数
/// により 1 回の探索コストを O(`MAX_INLINE_SCAN_WINDOW`) に固定し、
/// `parse_inline` 全体の計算量を入力長 n に対し O(n) に抑える。
///
/// 値（2,000 文字）は GFM の強調・リンク・インラインコードスパンの実用的な
/// 長さ（通常は数十文字、長文の強調でも数百文字程度）を大きく超えており、
/// 正当な Markdown コンテンツの解釈結果には影響しない。上限に達した場合は
/// 「閉じマーカーなし」と同じフォールバック（リテラル文字として扱う）に
/// より取りこぼしなく処理を継続する（fail-safe）。
const MAX_INLINE_SCAN_WINDOW: usize = 2000;

/// フェンスコードブロックの `class` 属性へ許可する info string の文字集合。
///
/// 英数字・`_`・`+`・`.`・`-` のみを許可するホワイトリスト方式。属性値自体は
/// `crates/core` 側の [`fandhe_frontend_core::render`] が出力時にエスケープ
/// するため XSS 上の必須要件ではないが、`"` 等を含む info string を無条件に
/// `class` へ流し込まない多層防御として本関数を経由させる。
fn is_valid_lang_token(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '+' | '.' | '-'))
}

/// ブロック内テキストをインライン Node 列へ変換する継ぎ目関数（イシュー #467）。
///
/// インラインコード（`` `code` ``）/ 強調（`**strong**` / `*em*`）/ リンク
/// （`[text](url)`）を解釈し、それ以外の文字は [`text`]（既定エスケープ経由）
/// として出力する。全呼び出し元（見出し・段落・リスト項目・テーブルセル・
/// 引用、`inline_nodes` 継ぎ目契約）は本関数経由でテキストを取得するため、
/// ここだけ差し替えれば全ブロック文脈にインライン構文が波及する
/// （`raw_html()` は使わない、既定エスケープの迂回経路を増やさない、REQ-1）。
///
/// # スコープ外
///
/// 画像構文 `![alt](url)`（`!` はリテラル、続く `[...](...)` は通常リンク
/// として解釈される）・`_`/`__` によるアンダースコア強調（既存 docs で未使用
/// かつ `raw_html_lint_e2e` 等の識別子を誤解釈するリスクがあるため意図的に
/// 非対応）・自動リンク・参照形式リンクは非対応（イシュー #467 計画のスコープ外）。
fn inline_nodes(s: &str) -> Vec<Node> {
    parse_inline(s, 0, false)
}

/// [`inline_nodes`] の内部実装。強調・リンクテキストの再帰的解釈のために
/// 深さカウンタ（`depth`）とリンクネスト禁止フラグ（`in_link`）を保持する。
///
/// `depth` は [`MAX_DEPTH`]（引用・リストと共通の定数値。カウンタ自体は
/// ブロック側と独立した別軸）に達すると強調・リンクの開始マーカーを
/// リテラル文字として扱いそれ以上再帰しない（OWASP A04、スタック
/// オーバーフロー対策の設計をインラインにも適用）。
///
/// `in_link` は `true` の間 `[` をリンク開始として解釈しない（リンクの
/// ネストを禁止し、内側の `[` はリテラル扱いにする設計どおり）。
fn parse_inline(s: &str, depth: usize, in_link: bool) -> Vec<Node> {
    let chars: Vec<char> = s.chars().collect();
    let mut nodes: Vec<Node> = Vec::new();
    let mut literal = String::new();
    let mut i = 0usize;

    while i < chars.len() {
        let c = chars[i];

        if c == '`' {
            if let Some((mut parsed, next)) = try_inline_code(&chars, i) {
                flush_literal(&mut literal, &mut nodes);
                nodes.append(&mut parsed);
                i = next;
                continue;
            }
        } else if c == '*' {
            if let Some((mut parsed, next)) = try_emphasis(&chars, i, depth, in_link) {
                flush_literal(&mut literal, &mut nodes);
                nodes.append(&mut parsed);
                i = next;
                continue;
            }
        } else if c == '[' && !in_link {
            if let Some((mut parsed, next)) = try_link(&chars, i, depth) {
                flush_literal(&mut literal, &mut nodes);
                nodes.append(&mut parsed);
                i = next;
                continue;
            }
        }

        // どの構文にも一致しなかった（または閉じマーカーが見つからなかった）
        // 場合は開始文字をリテラルとして 1 文字だけ進める（fail-safe、
        // バックトラック蓄積による O(n²) を避ける設計どおり）。
        literal.push(c);
        i += 1;
    }

    flush_literal(&mut literal, &mut nodes);
    nodes
}

/// 蓄積中のリテラル文字列を 1 つの [`text`] ノードとして `nodes` へ確定する。
fn flush_literal(literal: &mut String, nodes: &mut Vec<Node>) {
    if !literal.is_empty() {
        nodes.push(text(literal.as_str()));
        literal.clear();
    }
}

/// `chars[start..]` から、`ch` が過不足なく `run_len` 個連続する箇所を探す。
///
/// インラインコードの閉じバッククォート（`run_len` = 開始と同じ本数）・
/// 強調の閉じマーカー（`*` を `run_len` = 1 または 2）の探索を共通化する。
/// 目的の本数と異なる連続（例: 強調の閉じを探索中に遭遇した `**`）はまるごと
/// 読み飛ばして続行するため、部分一致による誤閉じを避けられる。見つかった
/// 場合は連続の開始インデックスを返す。
///
/// 走査は `start` から高々 [`MAX_INLINE_SCAN_WINDOW`] 文字までに限定する
/// （超過分は「閉じマーカーなし」と同じ `None` を返す）。上限なしで
/// `chars.len()` まで無条件に走査すると、呼び出し元（[`try_emphasis`] /
/// [`try_inline_code`]）が開始位置ごとに本関数を呼ぶ構造と組み合わさって
/// 最悪計算量が O(n^2) になる（アルゴリズム的計算量 DoS、OWASP A04、
/// レビュー指摘イシュー #467）。
fn find_closing_run(chars: &[char], start: usize, ch: char, run_len: usize) -> Option<usize> {
    let limit = chars.len().min(start + MAX_INLINE_SCAN_WINDOW);
    let mut k = start;
    while k < limit {
        if chars[k] == ch {
            let run_start = k;
            let mut m = 0usize;
            while k < limit && chars[k] == ch {
                k += 1;
                m += 1;
            }
            if k == limit && k < chars.len() && chars[k] == ch {
                // 走査上限に達した時点でもまだ同じ文字の連続が続いており、
                // 真の本数が確定できない。誤って一致と判定しないよう、ここで
                // 探索を打ち切る（fail-safe、「見つからない」と同じ扱い）。
                return None;
            }
            if m == run_len {
                return Some(run_start);
            }
            // 本数が異なる連続は内容の一部として読み飛ばす。
            continue;
        }
        k += 1;
    }
    None
}

/// `i` が指すバッククォート連続を開始とみなし、インラインコードを試みる。
///
/// 開始と同じ本数の連続で閉じる（CommonMark 簡略版、`` ``a`b`` `` のように
/// コード中にバッククォートを含められる）。中身はリテラル（リンク・強調を
/// 解釈しない）とし、`raw_html()` を使わず [`text`] 経由でエスケープする。
/// 閉じが見つからない場合は `None` を返し、呼び出し元がバッククォートを
/// リテラル文字へフォールバックする。
fn try_inline_code(chars: &[char], i: usize) -> Option<(Vec<Node>, usize)> {
    let open_len = {
        let mut n = 0usize;
        while i + n < chars.len() && chars[i + n] == '`' {
            n += 1;
        }
        n
    };
    let content_start = i + open_len;
    let close_start = find_closing_run(chars, content_start, '`', open_len)?;
    let content: String = chars[content_start..close_start].iter().collect();
    let node = code(vec![], vec![text(content.as_str())]);
    Some((vec![node], close_start + open_len))
}

/// `i` が指す `*` を開始とみなし、強調（`**strong**` を先に判定、次いで
/// `*em*`）を試みる。`_`/`__` による強調は意図的に非対応（既存 docs で
/// 不使用かつ識別子中の `_` を誤解釈するため、モジュール rustdoc 参照）。
///
/// 中身は再帰的にインライン解釈する（強調のネスト・コード内包可）。
/// `depth` が [`MAX_DEPTH`] に達している場合は再帰せず `None` を返し、
/// 呼び出し元が `*` をリテラル文字へフォールバックする。対応する閉じ
/// マーカーが見つからない場合も同様にフォールバックする。
fn try_emphasis(
    chars: &[char],
    i: usize,
    depth: usize,
    in_link: bool,
) -> Option<(Vec<Node>, usize)> {
    if depth >= MAX_DEPTH {
        return None;
    }
    let is_strong = i + 1 < chars.len() && chars[i + 1] == '*';
    let marker_len = if is_strong { 2 } else { 1 };
    let content_start = i + marker_len;
    let close_start = find_closing_run(chars, content_start, '*', marker_len)?;
    let inner: String = chars[content_start..close_start].iter().collect();
    let children = parse_inline(&inner, depth + 1, in_link);
    let node = if is_strong {
        strong(vec![], children)
    } else {
        em(vec![], children)
    };
    Some((vec![node], close_start + marker_len))
}

/// `i` が指す `[` を開始とみなし、`[text](url)` 形式のリンクを試みる。
///
/// `url` は [`is_safe_link_url`] を通過した場合のみ `<a href="...">` を
/// 生成する。不合格時は `<a>` を生成せず、リンクテキストのみを
/// インライン解釈して返す（fail-closed かつ内容の取りこぼしをしない）。
/// リンクテキスト内は `in_link = true` で再帰するため、内側の `[` は
/// リテラル扱いとなりリンクのネストは発生しない（設計どおり）。
///
/// 閉じ括弧 `]` 不在・直後の `(` 不在・`)` 不在のいずれかの場合は `None` を
/// 返し、呼び出し元が `[` をリテラル文字へフォールバックする。
fn try_link(chars: &[char], i: usize, depth: usize) -> Option<(Vec<Node>, usize)> {
    if depth >= MAX_DEPTH {
        return None;
    }
    let close_bracket = find_char(chars, i + 1, ']')?;
    if chars.get(close_bracket + 1) != Some(&'(') {
        return None;
    }
    let url_start = close_bracket + 2;
    let close_paren = find_char(chars, url_start, ')')?;

    let link_text: String = chars[i + 1..close_bracket].iter().collect();
    let url: String = chars[url_start..close_paren].iter().collect();
    let next = close_paren + 1;

    let children = parse_inline(&link_text, depth + 1, true);
    if is_safe_link_url(&url) {
        Some((vec![a(vec![("href", url.as_str())], children)], next))
    } else {
        Some((children, next))
    }
}

/// `chars[start..]` から最初に一致する `target` のインデックスを返す。
///
/// [`find_closing_run`] と同様、走査は `start` から高々
/// [`MAX_INLINE_SCAN_WINDOW`] 文字までに限定する（[`try_link`] が `[` の
/// 出現ごとに本関数を呼ぶため、無制限走査は同種の O(n^2) DoS を招く）。
fn find_char(chars: &[char], start: usize, target: char) -> Option<usize> {
    let limit = chars.len().min(start + MAX_INLINE_SCAN_WINDOW);
    chars[start..limit]
        .iter()
        .position(|&c| c == target)
        .map(|offset| start + offset)
}

/// リンク URL の第 1 層スキーム検証（受け入れ条件: http / https / 相対のみ許可）。
///
/// core の [`is_safe_url`]（`fandhe_frontend_core::is_safe_url`、`render_into`
/// が `href` 等の属性出力時に適用する第 2 層）をまず通し、`javascript:` /
/// `data:` 等の危険スキームをタブ・改行除去/先頭制御文字トリムを含む共通
/// 正規化ロジックで遮断する（`crates/core/src/url.rs`、イシュー #373）。
///
/// その上で本関数は docs-site 独自の追加制限として、core と同じ正規化
/// （`\t`/`\n`/`\r` の全除去 → 先頭の制御文字・空白トリム）を適用した値が
/// `mailto:` / `tel:`（ASCII 大文字小文字非依存）で始まる場合を拒否する
/// （core は許可するが、本関数は受け入れ条件どおりそれより厳しい集合のみ
/// 許可する）。正規化を経ない値で判定すると `mai\tlto:` のような偽装が
/// すり抜けるため、必ず正規化後の値で前方一致判定する。
///
/// 結果として許可されるのは相対 URL（protocol-relative `//host` 含む）と
/// `http:` / `https:` のみ。拒否時の挙動は呼び出し元 [`try_link`] を参照。
fn is_safe_link_url(url: &str) -> bool {
    if !is_safe_url(url) {
        return false;
    }
    let normalized: String = url
        .chars()
        .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
        .collect();
    let trimmed = normalized.trim_start_matches(|c: char| c.is_control() || c.is_whitespace());
    !(starts_with_ignore_case_ascii(trimmed, "mailto:")
        || starts_with_ignore_case_ascii(trimmed, "tel:"))
}

/// `s` が `prefix`（ASCII のみを想定）で始まるかを大文字小文字非依存で判定する。
///
/// バイト境界パニックを避けるため文字単位で比較する（`s` の先頭が非 ASCII
/// マルチバイト文字の場合に `s[..prefix.len()]` のようなバイト添字スライスは
/// 文字境界を跨いでパニックし得るため使わない）。
fn starts_with_ignore_case_ascii(s: &str, prefix: &str) -> bool {
    let mut s_chars = s.chars();
    for p in prefix.chars() {
        match s_chars.next() {
            Some(c) if c.is_ascii() && c.eq_ignore_ascii_case(&p) => continue,
            _ => return false,
        }
    }
    true
}

/// Markdown 文字列をブロック単位で解釈し、Node のブロック列へレンダリングする。
///
/// 見出し（ATX）/ 段落 / 箇条書き・番号リスト（1 段ネスト対応）/ フェンス
/// コードブロック / 引用（複数行・入れ子ブロック対応）/ テーブルに対応する。
/// 全テキストは [`inline_nodes`] 経由で [`text`] を通り既定エスケープされる
/// （`raw_html()` は使わない、REQ-1）。パニックしない全域関数。
pub fn render_markdown(input: &str) -> Vec<Node> {
    render_markdown_at_depth(input, 0)
}

/// [`render_markdown`] の内部実装。引用・ネストリストの再帰呼び出しに
/// 深さカウンタを渡すためのエントリ（`MAX_DEPTH` 打ち切りのため分離）。
fn render_markdown_at_depth(input: &str, depth: usize) -> Vec<Node> {
    let lines: Vec<&str> = input.lines().collect();
    let mut out = Vec::new();
    let mut i = 0usize;

    while i < lines.len() {
        let line = lines[i];

        if line.trim().is_empty() {
            i += 1;
            continue;
        }

        if let Some(fence_marker) = fence_open(line) {
            let (node, next_i) = parse_fence(&lines, i, &fence_marker);
            out.push(node);
            i = next_i;
            continue;
        }

        if let Some((level, content)) = parse_atx_heading(line) {
            out.push(heading_node(level, content));
            i += 1;
            continue;
        }

        if is_quote_line(line) {
            let (node, next_i) = parse_quote(&lines, i, depth);
            out.push(node);
            i = next_i;
            continue;
        }

        if let Some(kind) = list_item_kind(line) {
            let (node, next_i) = parse_list(&lines, i, kind, depth);
            out.push(node);
            i = next_i;
            continue;
        }

        if is_table_start(&lines, i) {
            let (node, next_i) = parse_table(&lines, i);
            out.push(node);
            i = next_i;
            continue;
        }

        let (node, next_i) = parse_paragraph(&lines, i);
        out.push(node);
        i = next_i;
    }

    out
}

/// 見出しレベルに応じたタグショートカットへ振り分ける。
///
/// `parse_atx_heading` が返すレベルは `1..=6` に丸め済みのため、それ以外の
/// 値は到達しない防御的な `_ => h6` としている（`unwrap`/`panic!` 回避）。
fn heading_node(level: u8, content: &str) -> Node {
    let children = inline_nodes(content);
    match level {
        1 => h1(vec![], children),
        2 => h2(vec![], children),
        3 => h3(vec![], children),
        4 => h4(vec![], children),
        5 => h5(vec![], children),
        6 => h6(vec![], children),
        _ => h6(vec![], children),
    }
}

/// ATX 見出し行（`#`〜`######` + 空白 + 本文）を判定する。
///
/// 7 個以上の `#` や、`#` 直後に空白を伴わない行（例: `#tag`）は見出しと
/// 認識せず段落へフォールバックする（設計どおり）。
fn parse_atx_heading(line: &str) -> Option<(u8, &str)> {
    let hashes = line.chars().take_while(|&c| c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &line[hashes..];
    if rest.is_empty() {
        // "#" のみの行（本文なし）は空見出しとして許容する。
        return Some((hashes as u8, ""));
    }
    let mut chars = rest.chars();
    match chars.next() {
        Some(c) if c.is_whitespace() => Some((hashes as u8, rest.trim())),
        _ => None,
    }
}

/// フェンス開始行を判定し、フェンス種別（`` ` `` 3 連続以上）を返す。
///
/// 戻り値は実際に使われたフェンス文字（バッククォート固定、`~` 系は
/// 既存 docs で不使用のためスコープ外）と長さの組。
fn fence_open(line: &str) -> Option<(char, usize)> {
    let trimmed = line.trim_start();
    let indent_ok = line.len() - trimmed.len() <= 3;
    if !indent_ok {
        return None;
    }
    let backticks = trimmed.chars().take_while(|&c| c == '`').count();
    if backticks >= 3 {
        Some(('`', backticks))
    } else {
        None
    }
}

/// フェンスコードブロックを解析する。`open` は開始行の `(フェンス文字, 長さ)`。
///
/// 閉じフェンスが見つからず EOF に達した場合は、残り全行をコード内容として
/// 扱う（取りこぼしをテキスト化しない fail-safe、設計どおり）。
fn parse_fence(lines: &[&str], start: usize, open: &(char, usize)) -> (Node, usize) {
    let open_line = lines[start];
    let open_trimmed = open_line.trim_start();
    // 開始フェンスの字下げ幅（`fence_open` が許容する最大 3 スペース）。
    // インデント付きフェンス内の本文行から、開始フェンスと同じ幅だけ字下げを
    // 取り除くために保持する（CommonMark 準拠、字下げは内容の意味を変えるため
    // 除去しないと `pre`/`code` 出力に余分な先頭空白が残ってしまう）。
    let open_indent = open_line.len() - open_trimmed.len();
    let info = open_trimmed.trim_start_matches(open.0);
    // 言語トークンは info string 全体を trim してから分割する。先に空白分割
    // すると「フェンス直後にスペースを挟んだ info string」（例: ``` ` ``` rust`）
    // の第 1 トークンが空文字列になり、有効な言語指定が誤って棄却される。
    let lang_token = info.trim().split([' ', ',', '\t']).next().unwrap_or("");

    let mut body_lines: Vec<&str> = Vec::new();
    let mut i = start + 1;
    let mut closed = false;
    while i < lines.len() {
        let candidate = lines[i].trim_start();
        let indent_ok = lines[i].len() - candidate.len() <= 3;
        // 閉じフェンス行は行末の空白・タブを許容する（CommonMark 準拠）。
        // 行頭側は `indent_ok` の判定に使うため別途保持し、末尾のみ trim する。
        let candidate_end_trimmed = candidate.trim_end();
        let backticks = candidate_end_trimmed
            .chars()
            .take_while(|&c| c == '`')
            .count();
        // 閉じフェンスは「同じフェンス文字が開始フェンス以上の長さ連続し、
        // それ以外の文字を含まない行」（CommonMark のフェンス閉じ規則の簡略版）。
        if indent_ok && backticks >= open.1 && candidate_end_trimmed.chars().all(|c| c == open.0) {
            closed = true;
            i += 1;
            break;
        }
        // 開始フェンスと同じ幅（最大 3 スペース）だけ本文行の先頭字下げを
        // 除去する。本文行の字下げが開始フェンスより浅い場合は全除去に留める
        // （取りこぼし防止、CommonMark の「共通字下げの除去」規則の簡略版）。
        let line = lines[i];
        let strip_len = line.len() - line.trim_start_matches(' ').len();
        let strip_len = strip_len.min(open_indent);
        body_lines.push(&line[strip_len..]);
        i += 1;
    }
    // EOF まで閉じフェンスがなかった場合も body_lines は蓄積済みであり、
    // 取りこぼしなくコード内容として扱われる（`closed` は次回拡張用に保持）。
    let _ = closed;

    // `code()` は `Vec<(&str, &str)>` を取るため、"language-<lang>" の所有
    // 文字列を本関数のスコープ内に保持してから借用する（`Box::leak` 等での
    // `'static` 昇格はしない。使い捨ての一時値なのでリークは不要かつ不適切）。
    let class_value = if is_valid_lang_token(lang_token) {
        Some(format!("language-{lang_token}"))
    } else {
        None
    };
    let code_attrs: Vec<(&str, &str)> = match &class_value {
        Some(v) => vec![("class", v.as_str())],
        None => vec![],
    };
    let content = body_lines.join("\n");
    let node = pre(vec![], vec![code(code_attrs, vec![text(content)])]);
    (node, i)
}

/// `>` で始まる行（引用）かどうかを判定する。
fn is_quote_line(line: &str) -> bool {
    line.trim_start().starts_with('>')
}

/// 引用ブロック（連続する `>` 行）を解析し、`>` と直後の 1 空白を剥がした
/// 本文を再帰的に [`render_markdown_at_depth`] へ渡して `blockquote` に格納する。
///
/// `depth` が [`MAX_DEPTH`] に達した場合は再帰せず、剥がした本文を単一の
/// 段落として扱う（スタックオーバーフロー防止、設計どおり）。
fn parse_quote(lines: &[&str], start: usize, depth: usize) -> (Node, usize) {
    let mut inner_lines: Vec<String> = Vec::new();
    let mut i = start;
    while i < lines.len() && is_quote_line(lines[i]) {
        let trimmed = lines[i].trim_start();
        let without_marker = &trimmed[1..];
        let without_space = without_marker.strip_prefix(' ').unwrap_or(without_marker);
        inner_lines.push(without_space.to_string());
        i += 1;
    }
    let inner_text = inner_lines.join("\n");

    let children = if depth >= MAX_DEPTH {
        vec![p(vec![], inline_nodes(&inner_text))]
    } else {
        render_markdown_at_depth(&inner_text, depth + 1)
    };
    (blockquote(vec![], children), i)
}

/// リスト種別（箇条書き `-`/`*` か番号 `N.`）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum ListKind {
    Bullet,
    Ordered,
}

/// 行がリスト項目の開始行かどうかを判定し、種別を返す。
///
/// インデント幅は判定しない（トップレベルかネスト行かは呼び出し元
/// `parse_list` がインデント比較で振り分ける。本関数はマーカー種別のみ判定
/// する純粋関数）。
fn list_item_kind(line: &str) -> Option<ListKind> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
        return Some(ListKind::Bullet);
    }
    // 番号リスト: 数字列 + "." + 空白。
    let digits_end = trimmed.find(|c: char| !c.is_ascii_digit()).unwrap_or(0);
    if digits_end > 0 && trimmed[digits_end..].starts_with(". ") {
        return Some(ListKind::Ordered);
    }
    None
}

/// リスト項目行からマーカーを剥がした本文と、その行のインデント幅を返す。
fn strip_list_marker(line: &str) -> (usize, &str) {
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();
    if let Some(rest) = trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))
    {
        return (indent, rest);
    }
    let digits_end = trimmed.find(|c: char| !c.is_ascii_digit()).unwrap_or(0);
    let after_digits = &trimmed[digits_end..];
    if let Some(after_dot) = after_digits.strip_prefix(". ") {
        return (indent, after_dot);
    }
    (indent, trimmed)
}

/// リストブロックを解析する。同一項目の継続行（マーカーなしインデント行）は
/// 前 `li` のテキストへ結合し、2 スペース以上インデントされたマーカー行は
/// 子リストとして再帰処理する。ネストは `depth < MAX_DEPTH`（`MAX_DEPTH` は
/// 引用と共通）を条件に何段でも許容し、上限に達した段のみ継続行として
/// 直前アイテムのテキストへ結合される。
fn parse_list(lines: &[&str], start: usize, kind: ListKind, depth: usize) -> (Node, usize) {
    let base_indent = {
        let trimmed = lines[start].trim_start();
        lines[start].len() - trimmed.len()
    };
    let mut items: Vec<Vec<Node>> = Vec::new();
    // 各アイテムの直接テキスト行（継続行込み）。ネスト子リストはこれとは
    // 別に items へ追記する（テキストの後ろに配置）。
    let mut item_texts: Vec<Vec<String>> = Vec::new();

    let mut i = start;
    while i < lines.len() {
        let line = lines[i];
        if line.trim().is_empty() {
            break;
        }
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        if indent == base_indent {
            if let Some(k) = list_item_kind(line) {
                if k == kind {
                    let (_, body) = strip_list_marker(line);
                    item_texts.push(vec![body.to_string()]);
                    items.push(Vec::new());
                    i += 1;
                    continue;
                }
            }
            // 同じインデントだがマーカー種別が異なる/マーカーなし → リスト終了。
            break;
        }

        if indent > base_indent && !item_texts.is_empty() {
            // ネストしたマーカー行（1 段のみ）かどうかを判定する。
            if list_item_kind(line).is_some() && depth < MAX_DEPTH {
                let (nested_node, next_i) = parse_list(
                    lines,
                    i,
                    list_item_kind(line).unwrap_or(ListKind::Bullet),
                    depth + 1,
                );
                if let Some(last) = items.last_mut() {
                    last.push(nested_node);
                }
                i = next_i;
                continue;
            }
            // マーカーなしの継続行 → 直前アイテムのテキストへ結合。
            if let Some(last_text) = item_texts.last_mut() {
                last_text.push(trimmed.to_string());
            }
            i += 1;
            continue;
        }

        break;
    }

    let li_nodes: Vec<Node> = item_texts
        .into_iter()
        .zip(items)
        .map(|(texts, nested)| {
            let joined = texts.join(" ");
            let mut children = inline_nodes(&joined);
            children.extend(nested);
            li(vec![], children)
        })
        .collect();

    let node = match kind {
        ListKind::Bullet => ul(vec![], li_nodes),
        ListKind::Ordered => ol(vec![], li_nodes),
    };
    (node, i)
}

/// `|` 区切り行と直後の区切り行（`|---|` 形式）の組をテーブル開始として判定する。
///
/// 区切り行に一致しない場合はテーブルと認識せず、呼び出し元で段落へ
/// フォールバックさせる（誤爆防止、設計どおり）。
fn is_table_start(lines: &[&str], i: usize) -> bool {
    if !lines[i].trim_start().starts_with('|') {
        return false;
    }
    match lines.get(i + 1) {
        Some(next) => is_table_delimiter_row(next),
        None => false,
    }
}

/// テーブル区切り行（例: `|---|:---:|---|`）かどうかを判定する。
///
/// アライメント記号 `:` は許容するが解釈しない（設計どおりアライメント無視）。
fn is_table_delimiter_row(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') {
        return false;
    }
    let inner = trimmed.trim_matches('|');
    if inner.is_empty() {
        return false;
    }
    inner.split('|').all(|cell| {
        let c = cell.trim();
        // GFM 仕様上、区切りセルは `-`/`:` のみで構成されることに加え、最低
        // 1 つの `-` を含む必要がある。`::` のようなコロンのみのセルを許容
        // すると `|::|::|` のような通常テキスト行が誤ってテーブル区切り行と
        // 判定され、後続の `|` 始まりの行が意図せず tbody に取り込まれる。
        !c.is_empty() && c.chars().all(|ch| matches!(ch, '-' | ':')) && c.contains('-')
    })
}

/// `|` で区切られた 1 行をセル文字列列へ分解する（行頭・行末の `|` を除去）。
fn split_table_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    inner.split('|').map(|c| c.trim().to_string()).collect()
}

/// テーブルブロックを解析する。1 行目をヘッダ（`thead > tr > th*`）、
/// 2 行目（区切り行）を読み飛ばし、以降の `|` 開始行を本体
/// （`tbody > tr > td*`）として扱う。セル数はヘッダ列数に合わせ、不足は
/// 空セルで埋め、超過セルは独立した `td` として出力する（切り捨てない、設計どおり）。
fn parse_table(lines: &[&str], start: usize) -> (Node, usize) {
    let header_cells = split_table_row(lines[start]);
    let col_count = header_cells.len();

    let header_row = tr(
        vec![],
        header_cells
            .iter()
            .map(|c| th(vec![], inline_nodes(c)))
            .collect(),
    );

    let mut body_rows: Vec<Node> = Vec::new();
    let mut i = start + 2; // ヘッダ行 + 区切り行をスキップ。
    while i < lines.len() && lines[i].trim_start().starts_with('|') {
        let mut cells = split_table_row(lines[i]);
        while cells.len() < col_count {
            cells.push(String::new());
        }
        let td_nodes: Vec<Node> = cells.iter().map(|c| td(vec![], inline_nodes(c))).collect();
        body_rows.push(tr(vec![], td_nodes));
        i += 1;
    }

    let node = table(
        vec![],
        vec![thead(vec![], vec![header_row]), tbody(vec![], body_rows)],
    );
    (node, i)
}

/// 段落を解析する。非空行が連続する限り取り込み、改行を半角スペースで
/// 結合して 1 段落の `p` を返す。他ブロック構文の開始行に達したら終了する。
fn parse_paragraph(lines: &[&str], start: usize) -> (Node, usize) {
    let mut collected: Vec<&str> = Vec::new();
    let mut i = start;
    while i < lines.len() {
        let line = lines[i];
        if line.trim().is_empty() {
            break;
        }
        if fence_open(line).is_some()
            || parse_atx_heading(line).is_some()
            || is_quote_line(line)
            || list_item_kind(line).is_some()
            || is_table_start(lines, i)
        {
            break;
        }
        collected.push(line.trim());
        i += 1;
    }
    if collected.is_empty() {
        // 呼び出し元は非空行でのみ本関数を呼ぶ契約だが、防御的に 1 行だけ
        // 消費して無限ループを避ける（フォールバック、パニックしない）。
        let joined = lines.get(start).copied().unwrap_or("");
        return (p(vec![], inline_nodes(joined)), start + 1);
    }
    let joined = collected.join(" ");
    (p(vec![], inline_nodes(&joined)), i)
}
