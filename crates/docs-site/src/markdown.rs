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
//!
//! パニックしない全域関数として実装する（ライブラリコードでの `unwrap()` /
//! `panic!` 回避規約、`.claude/rules/coding-rust.md`）。未知の行・不正な構文は
//! 段落として扱うフォールバックにより、任意の `&str` を受理する。

use fandhe_frontend_core::{
    blockquote, code, h1, h2, h3, h4, h5, h6, li, ol, p, pre, table, tbody, td, text, th, thead,
    tr, ul, Node,
};

/// 引用・ネストリストの再帰的解釈における最大深さ。
///
/// 悪意ある/破損した入力（例: `>` を数千行連続させた入力）によるスタック
/// オーバーフローを防ぐための DoS 対策（OWASP A04 安全でない設計）。超過分は
/// 通常の段落テキストとして扱い、取りこぼしをしない fail-safe とする。
const MAX_DEPTH: usize = 16;

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

/// ブロック内テキストをインライン Node 列へ変換する継ぎ目関数。
///
/// 本イシュー（#466）時点ではインライン構文（リンク・強調・インラインコード）
/// を未実装のため、入力全体を単一の [`text`]（既定エスケープ経由）として
/// 返すのみ。後続イシュー #467 がこの関数の内部実装のみを差し替える契約であり、
/// 差し替え後も全呼び出し元（見出し・段落・リスト項目・テーブルセル・引用）は
/// 本関数経由でテキストを取得するため、既定エスケープの迂回経路を増やさずに
/// インライン構文を追加できる（`raw_html()` を導入しない限り契約は保たれる）。
fn inline_nodes(s: &str) -> Vec<Node> {
    vec![text(s)]
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
    let open_line = lines[start].trim_start();
    let info = open_line.trim_start_matches(open.0);
    let lang_token = info.split([' ', ',', '\t']).next().unwrap_or("");

    let mut body_lines: Vec<&str> = Vec::new();
    let mut i = start + 1;
    let mut closed = false;
    while i < lines.len() {
        let candidate = lines[i].trim_start();
        let indent_ok = lines[i].len() - candidate.len() <= 3;
        let backticks = candidate.chars().take_while(|&c| c == '`').count();
        // 閉じフェンスは「同じフェンス文字が開始フェンス以上の長さ連続し、
        // それ以外の文字を含まない行」（CommonMark のフェンス閉じ規則の簡略版）。
        if indent_ok && backticks >= open.1 && candidate.chars().all(|c| c == open.0) {
            closed = true;
            i += 1;
            break;
        }
        body_lines.push(lines[i]);
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
        !c.is_empty() && c.chars().all(|ch| matches!(ch, '-' | ':'))
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
