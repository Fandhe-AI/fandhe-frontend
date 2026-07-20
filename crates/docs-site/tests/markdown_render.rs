//! `fandhe_frontend_docs_site::markdown::render_markdown` の統合テスト（イシュー #466）。
//!
//! ここで検証する不変条件は 2 つ:
//! 1. 各ブロック構文（見出し / 段落 / リスト / フェンスコード / 引用 / テーブル）が
//!    仕様どおりのノード木（`fandhe_frontend_core::render` の HTML 文字列出力で
//!    厳密比較）へレンダリングされること
//! 2. 全経路が既定エスケープ（REQ-1）を維持し、`<script>` 等の生 HTML がテキストと
//!    してエスケープされること（XSS 回帰）
//!
//! アサーションは決定性・機械検証可能性のため文字列の厳密一致で行う
//! （`.claude/rules/coding-rust.md` の一般規約に沿う）。

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::markdown::render_markdown;

/// `render_markdown` の出力を結合して 1 つの HTML 文字列にする補助関数。
fn render_all(input: &str) -> String {
    render_markdown(input)
        .iter()
        .map(render)
        .collect::<Vec<_>>()
        .join("")
}

// ---------------------------------------------------------------------
// 見出し
// ---------------------------------------------------------------------

#[test]
fn heading_levels_h1_to_h6() {
    let input = "# H1\n\n## H2\n\n### H3\n\n#### H4\n\n##### H5\n\n###### H6\n";
    assert_eq!(
        render_all(input),
        "<h1>H1</h1><h2>H2</h2><h3>H3</h3><h4>H4</h4><h5>H5</h5><h6>H6</h6>"
    );
}

#[test]
fn seven_hashes_falls_back_to_paragraph() {
    assert_eq!(
        render_all("####### not a heading"),
        "<p>####### not a heading</p>"
    );
}

#[test]
fn hash_without_space_falls_back_to_paragraph() {
    // "#tag" のように # 直後に空白がない場合は見出しと認識しない。
    assert_eq!(render_all("#tag"), "<p>#tag</p>");
}

// ---------------------------------------------------------------------
// 段落
// ---------------------------------------------------------------------

#[test]
fn multiline_paragraph_joins_with_space() {
    assert_eq!(
        render_all("line one\nline two\nline three"),
        "<p>line one line two line three</p>"
    );
}

#[test]
fn blank_lines_separate_paragraphs() {
    assert_eq!(
        render_all("first paragraph\n\nsecond paragraph"),
        "<p>first paragraph</p><p>second paragraph</p>"
    );
}

#[test]
fn empty_input_produces_no_nodes() {
    assert_eq!(render_markdown("").len(), 0);
    assert_eq!(render_markdown("\n\n\n").len(), 0);
}

// ---------------------------------------------------------------------
// リスト
// ---------------------------------------------------------------------

#[test]
fn bullet_list_basic() {
    assert_eq!(
        render_all("- one\n- two\n- three"),
        "<ul><li>one</li><li>two</li><li>three</li></ul>"
    );
}

#[test]
fn ordered_list_basic() {
    assert_eq!(
        render_all("1. one\n2. two\n3. three"),
        "<ol><li>one</li><li>two</li><li>three</li></ol>"
    );
}

#[test]
fn bullet_list_star_marker() {
    assert_eq!(
        render_all("* alpha\n* beta"),
        "<ul><li>alpha</li><li>beta</li></ul>"
    );
}

#[test]
fn list_item_continuation_line_merges_into_previous_item() {
    // マーカーなしの継続行（インデント付き）は直前の li のテキストへ結合する。
    assert_eq!(
        render_all("- one\n  continued text\n- two"),
        "<ul><li>one continued text</li><li>two</li></ul>"
    );
}

#[test]
fn nested_list_one_level() {
    assert_eq!(
        render_all("- outer one\n  - inner a\n  - inner b\n- outer two"),
        "<ul><li>outer one<ul><li>inner a</li><li>inner b</li></ul></li><li>outer two</li></ul>"
    );
}

#[test]
fn list_nesting_beyond_max_depth_falls_back_to_continuation_text() {
    // MAX_DEPTH（16）を超えるネストはスタックオーバーフロー防止のため打ち切られ、
    // それ以降のマーカー行は子リストにならず直前アイテムへの継続テキストとして
    // 結合される（境界値のリグレッション回帰、非ブロッキング指摘への対応）。
    //
    // 18 段のマーカー行（インデント 2 段刻み）を用意すると、0〜16 段目
    // （`parse_list` が `depth < MAX_DEPTH` を満たす限り再帰する範囲）までは
    // 実際に `<ul>` としてネストされ（計 17 個）、17 段目のマーカー行は
    // `depth(16) < MAX_DEPTH(16)` が偽になるため子リスト化されず、
    // 直前の `<li>` テキストへ生のマーカー文字列（`- deepest`）ごと結合される。
    let mut lines: Vec<String> = Vec::new();
    for level in 0..18 {
        lines.push(format!("{}- L{level}", "  ".repeat(level)));
    }
    let input = lines.join("\n");

    let output = render_all(&input);

    assert_eq!(
        output.matches("<ul>").count(),
        17,
        "MAX_DEPTH（16）に対応する <ul> ネスト段数は 17 段（0〜16）であるべき: {output}"
    );
    assert!(
        output.contains("L16 - L17"),
        "17 段目のマーカー行は子リスト化されず、生の \"- L17\" ごと直前 li の継続テキストへ \
         結合されるべき: {output}"
    );
    assert!(
        !output.contains("<ul><li>L17"),
        "17 段目が独立した <ul><li> を形成してはならない（MAX_DEPTH 打ち切りの回帰）: {output}"
    );
}

// ---------------------------------------------------------------------
// フェンスコードブロック
// ---------------------------------------------------------------------

#[test]
fn fence_with_lang_info_string() {
    assert_eq!(
        render_all("```rust\nfn main() {}\n```"),
        "<pre><code class=\"language-rust\">fn main() {}</code></pre>"
    );
}

#[test]
fn fence_with_comma_info_string_uses_first_token() {
    assert_eq!(
        render_all("```rust,ignore\nlet x = 1;\n```"),
        "<pre><code class=\"language-rust\">let x = 1;</code></pre>"
    );
}

#[test]
fn fence_without_info_string_has_no_class() {
    assert_eq!(
        render_all("```\nplain text\n```"),
        "<pre><code>plain text</code></pre>"
    );
}

#[test]
fn fence_with_invalid_info_string_has_no_class() {
    // ホワイトリスト外の文字（"）を含む info string は class を付与しない。
    assert_eq!(
        render_all("```\"onerror=alert(1)\nbody\n```"),
        "<pre><code>body</code></pre>"
    );
}

#[test]
fn unclosed_fence_consumes_rest_of_input() {
    assert_eq!(
        render_all("```rust\nfn main() {\n    unclosed"),
        "<pre><code class=\"language-rust\">fn main() {\n    unclosed</code></pre>"
    );
}

#[test]
fn fence_content_is_not_reinterpreted_as_markdown() {
    assert_eq!(
        render_all("```\n# not a heading\n- not a list\n```"),
        "<pre><code># not a heading\n- not a list</code></pre>"
    );
}

// ---------------------------------------------------------------------
// 引用
// ---------------------------------------------------------------------

#[test]
fn blockquote_single_line() {
    assert_eq!(
        render_all("> hello"),
        "<blockquote><p>hello</p></blockquote>"
    );
}

#[test]
fn blockquote_multiline_joins_as_paragraph() {
    assert_eq!(
        render_all("> line one\n> line two"),
        "<blockquote><p>line one line two</p></blockquote>"
    );
}

#[test]
fn blockquote_with_nested_list() {
    assert_eq!(
        render_all("> - item one\n> - item two"),
        "<blockquote><ul><li>item one</li><li>item two</li></ul></blockquote>"
    );
}

#[test]
fn blockquote_nesting_beyond_max_depth_falls_back_to_paragraph() {
    // MAX_DEPTH（16、引用とリストで共有）を超える引用ネストはスタックオーバーフロー
    // 防止のため打ち切られ、それ以降の `>` は再帰的に剥がされず、残った `>` ごと
    // 単一の段落テキストとして扱われる（境界値のリグレッション回帰、
    // 非ブロッキング指摘への対応）。
    //
    // `>` を 20 個連続させると、`parse_quote` は depth 0〜16（計 17 回）呼び出され
    // その都度先頭の `>` を 1 個ずつ剥がして `blockquote` を生成するが、
    // 17 回目（depth=16）の呼び出しは `depth(16) >= MAX_DEPTH(16)` が真になるため
    // それ以上再帰せず、残り 3 個の `>` を含む本文をそのまま段落へ格納する
    // （エスケープにより `&gt;&gt;&gt;x` として出力される）。
    let input = format!("{}x", ">".repeat(20));

    let output = render_all(&input);

    assert_eq!(
        output.matches("<blockquote>").count(),
        17,
        "MAX_DEPTH（16）に対応する <blockquote> ネスト段数は 17 段（0〜16）であるべき: {output}"
    );
    let expected = format!(
        "{}<p>&gt;&gt;&gt;x</p>{}",
        "<blockquote>".repeat(17),
        "</blockquote>".repeat(17)
    );
    assert_eq!(output, expected);
}

// ---------------------------------------------------------------------
// テーブル
// ---------------------------------------------------------------------

#[test]
fn table_basic() {
    let input = "| a | b |\n|---|---|\n| 1 | 2 |\n| 3 | 4 |";
    assert_eq!(
        render_all(input),
        "<table><thead><tr><th>a</th><th>b</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr><tr><td>3</td><td>4</td></tr></tbody></table>"
    );
}

#[test]
fn table_with_alignment_markers_is_ignored() {
    let input = "| a | b |\n|:---|---:|\n| 1 | 2 |";
    assert_eq!(
        render_all(input),
        "<table><thead><tr><th>a</th><th>b</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></table>"
    );
}

#[test]
fn pipe_line_without_delimiter_row_falls_back_to_paragraph() {
    // 直後が区切り行でない `|` 開始行はテーブルと認識せず段落として扱う（誤爆防止）。
    assert_eq!(render_all("| not a table |"), "<p>| not a table |</p>");
}

#[test]
fn table_missing_cells_are_padded_with_empty_td() {
    let input = "| a | b | c |\n|---|---|---|\n| 1 |";
    assert_eq!(
        render_all(input),
        "<table><thead><tr><th>a</th><th>b</th><th>c</th></tr></thead><tbody><tr><td>1</td><td></td><td></td></tr></tbody></table>"
    );
}

#[test]
fn table_extra_cells_are_not_dropped() {
    let input = "| a | b |\n|---|---|\n| 1 | 2 | 3 |";
    assert_eq!(
        render_all(input),
        "<table><thead><tr><th>a</th><th>b</th></tr></thead><tbody><tr><td>1</td><td>2</td><td>3</td></tr></tbody></table>"
    );
}

// ---------------------------------------------------------------------
// 複合フィクスチャ（既存 docs の構文実態に近い入力）
// ---------------------------------------------------------------------

#[test]
fn fixture_mixed_document_renders_expected_html() {
    let input = include_str!("fixtures/mixed.md");
    let expected = include_str!("fixtures/mixed.expected.html").trim_end();
    assert_eq!(render_all(input), expected);
}

// ---------------------------------------------------------------------
// XSS 回帰（ブロック経路すべてで既定エスケープが保たれること、REQ-1）
// ---------------------------------------------------------------------

const SCRIPT_PAYLOAD: &str = "<script>alert(1)</script>";
const IMG_PAYLOAD: &str = "<img src=x onerror=alert(1)>";

#[test]
fn xss_payload_in_heading_is_escaped() {
    let out = render_all("# <script>alert(1)</script>");
    assert!(!out.contains("<script"));
    assert!(out.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert_eq!(out, "<h1>&lt;script&gt;alert(1)&lt;/script&gt;</h1>");
}

#[test]
fn xss_payload_in_paragraph_is_escaped() {
    let out = render_all(SCRIPT_PAYLOAD);
    assert!(!out.contains("<script"));
    assert_eq!(out, "<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>");
}

#[test]
fn xss_payload_in_list_item_is_escaped() {
    // "onerror=" という文字列自体はエスケープ後もテキストとして残り得るが、
    // 危険なのは実際の属性として解釈されることなので、タグ開始マーカー
    // （`<img`）が出力されないこと・全体が仕様どおりエスケープされていること
    // を厳密一致で確認する。
    let out = render_all(&format!("- {IMG_PAYLOAD}"));
    assert!(!out.contains("<img"));
    assert_eq!(out, "<ul><li>&lt;img src=x onerror=alert(1)&gt;</li></ul>");
}

#[test]
fn xss_payload_in_table_cell_is_escaped() {
    let input = format!("| a |\n|---|\n| {SCRIPT_PAYLOAD} |");
    let out = render_all(&input);
    assert!(!out.contains("<script"));
    assert!(out.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
}

#[test]
fn xss_payload_in_blockquote_is_escaped() {
    let out = render_all(&format!("> {SCRIPT_PAYLOAD}"));
    assert!(!out.contains("<script"));
    assert_eq!(
        out,
        "<blockquote><p>&lt;script&gt;alert(1)&lt;/script&gt;</p></blockquote>"
    );
}

#[test]
fn xss_payload_in_fence_content_is_escaped() {
    // フェンス内容も text() 経由で出力されるため、コードブロック脱出（`</code></pre><script>`）
    // を狙った入力もエスケープされ、構造は崩れない。
    let input = "```\n</code></pre><script>alert(1)</script>\n```";
    let out = render_all(input);
    assert!(!out.contains("<script"));
    assert_eq!(
        out,
        "<pre><code>&lt;/code&gt;&lt;/pre&gt;&lt;script&gt;alert(1)&lt;/script&gt;</code></pre>"
    );
}

#[test]
fn xss_payload_in_fence_info_string_does_not_inject_attribute() {
    // info string に " を含めて class 属性からの脱出を試みても、ホワイトリスト
    // 判定（is_valid_lang_token）で class 自体が付与されない。
    let input = "```\"><script>alert(1)</script>\nbody\n```";
    let out = render_all(input);
    assert!(!out.contains("<script"));
    assert!(!out.contains("class="));
    assert_eq!(out, "<pre><code>body</code></pre>");
}

#[test]
fn raw_html_tag_in_paragraph_is_treated_as_text() {
    // HTML ブロックは構文として解釈しない。<div> はテキストとしてエスケープされる。
    let out = render_all("<div class=\"x\">hello</div>");
    assert!(!out.contains("<div"));
    assert_eq!(
        out,
        "<p>&lt;div class=&quot;x&quot;&gt;hello&lt;/div&gt;</p>"
    );
}
