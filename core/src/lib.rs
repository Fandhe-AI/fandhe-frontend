//! rws-core: 描画コアクレート（TASK-1.1「rws-core 既定エスケープの製品化」）。
//!
//! 責務境界: ノード木 API（`Node` / `el` / `text` / `raw_html`）と、それを HTML
//! 文字列へ変換するモード非依存レンダラ（`render`）を提供する。rws-server の
//! SSR パス・SSG 出力・rws-wasm-client の CSR がいずれも本クレートの `render()`
//! を共通で呼び出す前提であり、**その出力は既定エスケープ済みであることを
//! 呼び出し側フレームワーク各層が前提とする**（REQ-1）。
//!
//! セキュリティ不変条件（TASK-1.2 の XSS 回帰テスト・TASK-5.1/6.x が依存する契約）:
//! 1. `Node::Text` の内容・`Element` の属性値は `render()` 内で必ず [`escape_html`]
//!    を経由して出力する。
//! 2. エスケープを迂回できる経路は `Node::RawHtml`（コンストラクタ [`raw_html`]）
//!    のみとする。新たな迂回経路を追加しない。
//! 3. `format!("<div>{}</div>", user_input)` のような HTML 文字列の直接組み立て
//!    を内部にも作らない。タグの書き出しは [`render_into`] の構造化した手順のみ
//!    で行う。
//! 4. 属性名はフレームワーク利用者コード由来の動的文字列になり得るため、出力前に
//!    ホワイトリスト検証を行う。不正な属性名（空白・`=`・`"` 等の注入形）は
//!    panic させず出力からスキップする（ライブラリコードでの panic 回避規約）。
//! 5. タグ名は `&'static str` に限定し、動的文字列を受け付けない（型レベルでの
//!    タグ名注入抑止）。
//!
//! 本クレートは外部依存ゼロ（`core/Cargo.toml` 参照）。PoC-2 で判明した
//! 「マクロ DSL が依存グラフを押し上げる」という知見を踏まえ、`view!`/`html!`
//! のような独自マクロは使わず、素の Rust の enum・関数のみでノード木を組み立てる。
//!
//! ## スコープ外（TASK-1.1b 時点）
//!
//! タグショートカット（`div`/`p` 等）・ハイドレーション支援
//! （`find_attr_values`/`find_nav_targets`）・void 要素の自己終了処理は本クレート
//! では扱わない。前者 2 つは TASK-5.1・TASK-6.2 系で追加予定。

#![forbid(unsafe_code)]

use std::fmt::Write as _;

/// HTML ノード木。マクロ DSL に依存しない素の Rust 値として組み立てる。
///
/// 各腕のレンダリング時の扱いはセキュリティ不変条件そのものであるため、
/// 腕を追加・変更する際は本モジュールの rustdoc（不変条件 1・2）を必ず更新する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// 要素ノード。タグ名は `&'static str` に固定し、動的文字列によるタグ名
    /// 注入を型で防ぐ。属性値・子ノードは [`render_into`] で再帰的に処理する。
    Element {
        tag: &'static str,
        attrs: Vec<(String, String)>,
        children: Vec<Node>,
    },
    /// テキストノード。`render()` 時に必ず [`escape_html`] を経由する
    /// （既定安全 = REQ-1 の中核）。
    Text(String),
    /// 生 HTML ノード。`render()` 時にエスケープされない、唯一の明示的
    /// オプトイン経路（React の `dangerouslySetInnerHTML` 相当）。
    /// 信頼できない入力をそのまま渡してはならない。
    RawHtml(String),
}

/// 要素ノードを組み立てる素の Rust 関数（マクロではない）。
///
/// `attrs` は `(属性名, 属性値)` のペア列。属性値は [`render_into`] が
/// [`escape_html`] を経由して出力する。属性名は出力時にホワイトリスト検証を
/// 通過したものだけが書き出される（不変条件 4）。
pub fn el(tag: &'static str, attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    Node::Element {
        tag,
        attrs: attrs
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        children,
    }
}

/// テキストノードを組み立てる。レンダリング時に既定でエスケープされる
/// （REQ-1 が要求する「テキスト補間は既定エスケープ」の入口 API）。
pub fn text(s: impl Into<String>) -> Node {
    Node::Text(s.into())
}

/// 生 HTML ノードを組み立てる。**唯一の明示的オプトイン API**であり、
/// ここに渡した文字列はエスケープされずにそのまま出力へ書き出される。
///
/// # Security
///
/// 信頼できない外部入力（ユーザー入力・外部 API のレスポンス等）を
/// このまま渡すと XSS を招く。信頼できる固定文字列・別途サニタイズ済みの
/// 文字列のみを渡すこと。
pub fn raw_html(s: impl Into<String>) -> Node {
    Node::RawHtml(s.into())
}

/// HTML エンティティエスケープ（テキストノード・属性値の共通経路）。
///
/// 対象文字は `& < > " '` の 5 文字。[`render_into`] からのみ呼ばれる想定で、
/// 呼び出し側（rws-server 等）がこの関数を経由せず独自にエスケープ処理を
/// 行うことは想定しない（既定エスケープの一本化）。
pub fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    out
}

/// 属性名として安全に出力してよいかを判定する。
///
/// 属性名はフレームワーク利用者コードが動的に組み立てる可能性があり
/// （例: コンポーネントのプロパティ経由）、`escape_html` は属性値にしか
/// 適用されないため、属性名スロット経由の注入（`onerror=alert(1) x=` の
/// ような追加属性の割り込み）を別途遮断する必要がある。
///
/// 英数字・`-`・`_`・`:` のみを許可するホワイトリスト方式とし、空文字列は
/// 拒否する。判定に失敗した属性名は [`render_into`] が出力をスキップする
/// （panic させない。ライブラリコードでの panic 回避規約に従う防御的実装）。
fn is_valid_attr_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':')
}

/// ノード木を HTML 文字列へレンダリングする。
///
/// SSR（サーバーからのレスポンス送出）・SSG（ファイル書き出し）・CSR
/// （ブラウザで `innerHTML` に設定）のいずれも**この関数を共通で使う**モード
/// 非依存レンダラ。出力は既定エスケープ済みであることを呼び出し側の各層が
/// 前提とする（クレート冒頭の契約を参照）。
pub fn render(node: &Node) -> String {
    let mut out = String::new();
    render_into(node, &mut out);
    out
}

/// [`render`] の内部実装。エスケープを経由しない `push_str` は
/// `Node::RawHtml` の腕（唯一の非エスケープ出力点）と、フレームワーク制御下
/// のタグ名・山括弧・属性名（ホワイトリスト検証済み）に限定する。
/// `format!` によるトップレベル HTML 文字列組み立ては行わない
/// （不変条件 3）。
fn render_into(node: &Node, out: &mut String) {
    match node {
        Node::Text(s) => out.push_str(&escape_html(s)),
        // ここが唯一の非エスケープ出力点（raw_html オプトイン境界）。
        Node::RawHtml(s) => out.push_str(s),
        Node::Element {
            tag,
            attrs,
            children,
        } => {
            let _ = write!(out, "<{}", tag);
            for (k, v) in attrs {
                if !is_valid_attr_name(k) {
                    // 不正な属性名は panic させず出力からスキップする（不変条件 4）。
                    continue;
                }
                let _ = write!(out, " {}=\"{}\"", k, escape_html(v));
            }
            out.push('>');
            // void 要素の自己終了処理は本クレートのスコープ外（常に終了タグを出す）。
            for child in children {
                render_into(child, out);
            }
            let _ = write!(out, "</{}>", tag);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_node_is_escaped_by_default() {
        let payload = "<script>alert('xss')</script><img src=x onerror=alert(1)>";
        let node = el("p", vec![], vec![text(payload)]);
        let html = render(&node);
        assert!(
            !html.contains("<script>"),
            "生スクリプトタグがエスケープされずに出力された: {html}"
        );
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&#x27;"));
    }

    #[test]
    fn raw_html_node_is_not_escaped_when_opted_in() {
        let payload = "<b>bold</b>";
        let node = el("p", vec![], vec![raw_html(payload)]);
        let html = render(&node);
        assert!(html.contains("<b>bold</b>"));
    }

    #[test]
    fn attribute_values_are_escaped() {
        let node = el(
            "div",
            vec![("title", "\"><script>alert(1)</script>")],
            vec![],
        );
        let html = render(&node);
        assert!(!html.contains("<script>"));
        assert!(html.contains("&quot;&gt;&lt;script&gt;"));
    }

    #[test]
    fn invalid_attribute_name_is_skipped_without_panic() {
        // 属性名スロット経由でのイベントハンドラ注入（追加属性の割り込み）を模した入力。
        let node = el(
            "div",
            vec![("onmouseover=alert(1) x", "y"), ("class", "safe")],
            vec![],
        );
        let html = render(&node);
        assert!(!html.contains("onmouseover"));
        assert!(html.contains("class=\"safe\""));
    }

    #[test]
    fn nested_element_tree_renders_expected_structure() {
        let tree = el(
            "ul",
            vec![],
            vec![
                el("li", vec![], vec![text("item1")]),
                el("li", vec![], vec![text("item2")]),
            ],
        );
        let html = render(&tree);
        assert_eq!(html, "<ul><li>item1</li><li>item2</li></ul>");
    }
}
