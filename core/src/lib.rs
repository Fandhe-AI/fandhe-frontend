//! `rws-core`: 描画コア（外部依存ゼロ）。
//!
//! フロントエンドフレームワークの中核クレート。ノード木 API（`Node` / `el` /
//! `text` / `raw_html`）と、それを HTML 文字列へ変換するモード非依存レンダラ
//! （`render`）を提供する。`rws-server`（SSR/SSG）・`rws-wasm-client` /
//! `rws-wasm-full`（CSR）など上位クレートが本クレートの `render()` を共通で
//! 呼び出す前提であり、**その出力は既定エスケープ済みであることを呼び出し側
//! フレームワーク各層が前提とする**（REQ-1）。
//!
//! # 本クレートの不変条件（REQ-1・REQ-2、TASK-1.2 の XSS 回帰テスト・
//! TASK-5.1/6.x が依存する契約）
//!
//! 1. `Node::Text` の内容・`Element` の属性値は `render()` 内で必ず
//!    [`escape_html`] / [`escape_html_into`]（`escape` モジュール）を経由して
//!    出力する。
//! 2. エスケープを迂回できる経路は `Node::RawHtml`（コンストラクタ
//!    [`raw_html`]）のみとする。新たな迂回経路を追加しない。
//! 3. `format!("<div>{}</div>", user_input)` のような HTML 文字列の直接組み立て
//!    を内部にも作らない。タグの書き出しは `render_into`（内部実装）の構造化
//!    した手順のみで行う。
//! 4. 属性名はフレームワーク利用者コード由来の動的文字列になり得るため、出力前
//!    にホワイトリスト検証を行う。不正な属性名（空白・`=`・`"` 等の注入形）は
//!    panic させず出力からスキップする（ライブラリコードでの panic 回避規約）。
//! 5. タグ名は `&'static str` に限定し、動的文字列を受け付けない（型レベルで
//!    のタグ名注入抑止）。ただし `&'static str` は有効期間を保証するのみで
//!    文字内容までは保証しないため、出力前にホワイトリスト検証
//!    （`is_valid_tag_name`）も行う多層防御とする。不正なタグ名の要素は
//!    panic させず出力全体をスキップする。
//! 6. **`unsafe` コード禁止**: `#![forbid(unsafe_code)]` によりクレート全体で
//!    機械的に禁止する。`unsafe` は WASM バインディング層・FFI 境界に限定され、
//!    本クレートには含まれない。
//! 7. **外部依存ゼロ**: `Cargo.toml` の `[dependencies]` は常に空を維持する。
//!
//! 本クレートは外部依存ゼロ（`core/Cargo.toml` 参照）。PoC-2 で判明した
//! 「マクロ DSL が依存グラフを押し上げる」という知見を踏まえ、`view!`/`html!`
//! のような独自マクロは使わず、素の Rust の enum・関数のみでノード木を組み立てる。
//!
//! ## タグショートカット（TASK-5.1b）
//!
//! `docs/component-api.md`（TASK-5.1a 設計確定書）に従い、`div`/`p`/`ul`/`li`/
//! `a`/`h1`/`main_tag` を提供する。いずれも [`el`] への薄い委譲のみであり、
//! 独自の出力経路・独自のエスケープ処理を持たない（不変条件 1・2 がそのまま
//! 適用される）。網羅的なタグヘルパー群・インデント規約は既存 backlog
//! （Issue #164）のスコープとし、本クレートでは追加しない。
//!
//! ## ハイドレーション支援（TASK-6.2b）
//!
//! [`find_attr_values`] / [`find_nav_targets`] は、`rws-wasm-client`
//! （TASK-6.2 系）が既存 DOM 上でハイドレーション対象を特定するために
//! 呼び出す **DOM 非依存の純粋関数**。引数に取るのは本クレート自身の
//! ノード木（[`Node`]）であり、実 DOM 型（`web-sys::Node` 等）には一切
//! 依存しない。そのため `core` の外部依存ゼロ契約（不変条件 7）を侵さず、
//! wasm ビルドを介さないネイティブ環境でもテスト可能（`docs/hydration-api.md`
//! 第 2〜3 節・判断 3 の設計どおり）。
//!
//! ## スコープ外
//!
//! void 要素の自己終了処理は本クレートでは扱わない。`docs/component-api.md`
//! 第 3 節に記載のとおり、v1 では常に終了タグを出力する現行仕様を意図した
//! 挙動として凍結する。

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::fmt::Write as _;

mod escape;

pub use escape::{escape_html, escape_html_into};

/// HTML ノード木。マクロ DSL に依存しない素の Rust 値として組み立てる。
///
/// 各腕のレンダリング時の扱いはセキュリティ不変条件そのものであるため、
/// 腕を追加・変更する際は本モジュールの rustdoc（不変条件 1・2）を必ず更新する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// 要素ノード。タグ名は `&'static str` に固定し、動的文字列によるタグ名
    /// 注入を型で防ぐ。属性値・子ノードは `render_into`（内部実装）で再帰的に処理する。
    Element {
        /// タグ名。`&'static str` に固定し動的文字列を受け付けない（不変条件 5）。
        /// 出力前にホワイトリスト検証（`is_valid_tag_name`、内部実装）も通す多層防御。
        tag: &'static str,
        /// `(属性名, 属性値)` のペア列。属性値は出力時に必ずエスケープされ、
        /// 属性名はホワイトリスト検証（`is_valid_attr_name`、内部実装）を通過したもの
        /// だけが書き出される（不変条件 4）。
        attrs: Vec<(String, String)>,
        /// 子ノード列。`render_into` が出現順に再帰的に処理する。
        children: Vec<Node>,
    },
    /// テキストノード。`render()` 時に必ず [`escape_html_into`] を経由する
    /// （既定安全 = REQ-1 の中核）。
    Text(String),
    /// 生 HTML ノード。`render()` 時にエスケープされない、唯一の明示的
    /// オプトイン経路（React の `dangerouslySetInnerHTML` 相当）。
    /// 信頼できない入力をそのまま渡してはならない。
    RawHtml(String),
}

/// 要素ノードを組み立てる素の Rust 関数（マクロではない）。
///
/// `attrs` は `(属性名, 属性値)` のペア列。属性値は `render_into`（内部実装）が
/// [`escape_html_into`] を経由して出力する。属性名は出力時にホワイトリスト検証を
/// 通過したものだけが書き出される（不変条件 4）。
///
/// # Examples
///
/// ```
/// use rws_core::{el, text, render};
///
/// let node = el("p", vec![("class", "greeting")], vec![text("hello")]);
/// assert_eq!(render(&node), r#"<p class="greeting">hello</p>"#);
/// ```
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
///
/// # Examples
///
/// ```
/// use rws_core::{el, text, render};
///
/// let node = el("p", vec![], vec![text("<script>alert(1)</script>")]);
/// assert_eq!(render(&node), "<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>");
/// ```
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
///
/// # Examples
///
/// ```
/// use rws_core::{el, raw_html, render};
///
/// // 信頼できる固定文字列のみを渡す（ユーザー入力を直接渡さない）。
/// let node = el("div", vec![], vec![raw_html("<b>bold</b>")]);
/// assert_eq!(render(&node), "<div><b>bold</b></div>");
/// ```
pub fn raw_html(s: impl Into<String>) -> Node {
    Node::RawHtml(s.into())
}

/// `<div>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲であり、
/// エスケープ・タグ名検証は [`el`]/[`render`] の既存経路をそのまま利用する
/// （`docs/component-api.md` 第 4 節・定義規則）。
///
/// # Examples
///
/// ```
/// use rws_core::{div, text, render};
///
/// let node = div(vec![("class", "card")], vec![text("hello")]);
/// assert_eq!(render(&node), r#"<div class="card">hello</div>"#);
/// ```
pub fn div(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("div", attrs, children)
}

/// `<p>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
///
/// # Examples
///
/// ```
/// use rws_core::{p, text, render};
///
/// let node = p(vec![], vec![text("hello")]);
/// assert_eq!(render(&node), "<p>hello</p>");
/// ```
pub fn p(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("p", attrs, children)
}

/// `<ul>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
///
/// # Examples
///
/// ```
/// use rws_core::{ul, li, text, render};
///
/// let node = ul(vec![], vec![li(vec![], vec![text("item")])]);
/// assert_eq!(render(&node), "<ul><li>item</li></ul>");
/// ```
pub fn ul(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("ul", attrs, children)
}

/// `<li>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
///
/// # Examples
///
/// ```
/// use rws_core::{li, text, render};
///
/// let node = li(vec![], vec![text("item")]);
/// assert_eq!(render(&node), "<li>item</li>");
/// ```
pub fn li(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("li", attrs, children)
}

/// `<a>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
///
/// # Examples
///
/// ```
/// use rws_core::{a, text, render};
///
/// let node = a(vec![("href", "/about")], vec![text("about")]);
/// assert_eq!(render(&node), r#"<a href="/about">about</a>"#);
/// ```
pub fn a(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("a", attrs, children)
}

/// `<h1>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
///
/// # Examples
///
/// ```
/// use rws_core::{h1, text, render};
///
/// let node = h1(vec![], vec![text("title")]);
/// assert_eq!(render(&node), "<h1>title</h1>");
/// ```
pub fn h1(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("h1", attrs, children)
}

/// `<main>` 要素を組み立てる標準タグショートカット。`main` は Rust の予約語では
/// ないが、可読性のため PoC-3 の命名（`main_tag`）をそのまま踏襲する
/// （`docs/component-api.md` 第 4 節・定義規則 4）。[`el`] への薄い委譲。
///
/// # Examples
///
/// ```
/// use rws_core::{main_tag, text, render};
///
/// let node = main_tag(vec![], vec![text("content")]);
/// assert_eq!(render(&node), "<main>content</main>");
/// ```
pub fn main_tag(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("main", attrs, children)
}

/// 属性名として安全に出力してよいかを判定する。
///
/// 属性名はフレームワーク利用者コードが動的に組み立てる可能性があり
/// （例: コンポーネントのプロパティ経由）、[`escape_html_into`] は属性値にしか
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

/// タグ名として安全に出力してよいかを判定する。
///
/// `tag` は `&'static str` に限定される（不変条件 5）が、`&'static str` は
/// 値の**有効期間**を保証するのみで文字内容までは保証しない。
/// `Box::leak` 等で動的に生成した文字列を `'static` に昇格させれば、
/// 空白・`=`・`<`・`>`・`/` のようなタグ名スロットからの breakout に使える
/// 文字を含む値が型検査をすり抜けて `render_into` に届き得る。属性名の
/// [`is_valid_attr_name`] と対になる防御として、タグ名にも出力前の
/// ホワイトリスト検証を課す（型レベルの制約への多層防御であり、通常の
/// リテラルタグ運用では常に true になる）。
///
/// 先頭 ASCII 英字 + 以降 ASCII 英数字・`-` のみを許可する
/// （標準 HTML タグ名・カスタム要素名の両方を満たす最小限の文字集合）。
fn is_valid_tag_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// 指定属性名を持つ子孫要素（自身を含む）の属性値を出現順に列挙する。
///
/// `rws-wasm-client`（TASK-6.2 系）が `hydrate()` 実行時にハイドレーション
/// 対象を特定するために呼ぶ契約の関数。本関数は `node` が表す木構造のみを
/// 辿る DOM 非依存の純粋関数であり、実 DOM（`web-sys::Node` 等）にはまだ
/// 反映されていない/対応しない値を返しうる。呼び出し側（`wasm-client`）は、
/// SSR/CSR いずれかで実際に描画した木と同値の [`Node`] に対して本関数を呼び、
/// 得られた属性値をキーに `web-sys` 経由で実 DOM 要素を検索してイベント
/// リスナーを後付けする（`docs/hydration-api.md` 第 3〜4 節・判断 3）。
///
/// `Node::Text` / `Node::RawHtml` は属性を持たないため無視する。同一要素に
/// 同名属性が重複して渡された場合（[`el`] の `attrs` は生成時に重複除去
/// しないため理論上あり得る）は、出現順にすべて列挙する（呼び出し側での
/// 重複判定・除去は本関数の責務外）。
///
/// # Examples
///
/// ```
/// use rws_core::{div, el, text, find_attr_values};
///
/// let tree = div(
///     vec![],
///     vec![el("button", vec![("data-hydrate", "like")], vec![text("いいね")])],
/// );
/// assert_eq!(find_attr_values(&tree, "data-hydrate"), vec!["like".to_string()]);
/// ```
pub fn find_attr_values(node: &Node, attr_name: &str) -> Vec<String> {
    let mut out = Vec::new();
    collect_attr_values(node, attr_name, &mut out);
    out
}

/// [`find_attr_values`] の内部再帰実装。
fn collect_attr_values(node: &Node, attr_name: &str, out: &mut Vec<String>) {
    if let Node::Element {
        attrs, children, ..
    } = node
    {
        for (k, v) in attrs {
            if k == attr_name {
                out.push(v.clone());
            }
        }
        for child in children {
            collect_attr_values(child, attr_name, out);
        }
    }
}

/// `data-nav` 属性値を列挙する [`find_attr_values`] のショートカット。
///
/// `rws_app::list_page` が各項目リンクへ付与する `data-nav` 属性
///（`docs/app-api.md` 第 3 節）をクライアント側ルーティング配線の対象として
/// 特定するために `wasm-client` が呼ぶ契約の関数（`docs/hydration-api.md`
/// 第 3 節・公開 API 凍結表）。
///
/// # Examples
///
/// ```
/// use rws_core::{a, text, find_nav_targets};
///
/// let tree = a(vec![("href", "/items/1"), ("data-nav", "/items/1")], vec![text("記事1")]);
/// assert_eq!(find_nav_targets(&tree), vec!["/items/1".to_string()]);
/// ```
pub fn find_nav_targets(node: &Node) -> Vec<String> {
    find_attr_values(node, "data-nav")
}

/// ノード木を HTML 文字列へレンダリングする。
///
/// SSR（サーバーからのレスポンス送出）・SSG（ファイル書き出し）・CSR
/// （ブラウザで `innerHTML` に設定）のいずれも**この関数を共通で使う**モード
/// 非依存レンダラ。出力は既定エスケープ済みであることを呼び出し側の各層が
/// 前提とする（クレート冒頭の契約を参照）。
///
/// # Examples
///
/// ```
/// use rws_core::{el, text, render, Node};
///
/// let tree = el("ul", vec![], vec![el("li", vec![], vec![text("item")])]);
/// assert_eq!(render(&tree), "<ul><li>item</li></ul>");
///
/// // Node::Text 単体を要素で包まず直接レンダリングすることもできる。
/// assert_eq!(render(&Node::Text("<b>".to_string())), "&lt;b&gt;");
/// ```
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
        Node::Text(s) => escape_html_into(s, out),
        // ここが唯一の非エスケープ出力点（raw_html オプトイン境界）。
        Node::RawHtml(s) => out.push_str(s),
        Node::Element {
            tag,
            attrs,
            children,
        } => {
            if !is_valid_tag_name(tag) {
                // 不正なタグ名は panic させず要素全体の出力をスキップする
                // （タグ名スロット経由の breakout を防ぐ多層防御。ライブラリ
                // コードでの panic 回避規約に従い、属性名検証と同様に
                // 「不正なら出力しない」で安全側に倒す）。
                return;
            }
            let _ = write!(out, "<{}", tag);
            for (k, v) in attrs {
                if !is_valid_attr_name(k) {
                    // 不正な属性名は panic させず出力からスキップする（不変条件 4）。
                    continue;
                }
                let _ = write!(out, " {}=\"", k);
                escape_html_into(v, out);
                out.push('"');
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
        #[expect(
            clippy::disallowed_methods,
            reason = "ESCAPE-REVIEWED: raw_html オプトイン時の非エスケープ透過を検証するユニットテスト。固定の信頼済み文字列のみ"
        )]
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
    fn invalid_tag_name_is_skipped_without_panic() {
        // 型（&'static str）は文字内容までは保証しないため、
        // Box::leak で `'static` に昇格させたタグ名注入を模した入力を使う
        // （PR #166 Bugbot 指摘の再現テスト）。
        let malicious_tag: &'static str =
            Box::leak(String::from("div><script>alert(1)</script").into_boxed_str());
        let node = el(malicious_tag, vec![], vec![text("safe")]);
        let html = render(&node);
        assert!(
            !html.contains("<script>"),
            "不正なタグ名経由で breakout が発生した: {html}"
        );
        assert!(
            html.is_empty(),
            "不正なタグ名の要素は出力全体をスキップするべき: {html}"
        );
    }

    #[test]
    fn valid_custom_element_tag_name_renders_normally() {
        let node = el("my-widget", vec![], vec![text("ok")]);
        let html = render(&node);
        assert_eq!(html, "<my-widget>ok</my-widget>");
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

    /// 属性値コンテキストで 5 文字すべてがエスケープされることを一括で固定する
    /// （テキストコンテキストと同一規則を適用する不変条件 1 の属性側網羅）。
    #[test]
    fn attribute_value_escapes_all_five_target_characters() {
        let node = el("div", vec![("data-payload", "\"'<>&")], vec![]);
        let html = render(&node);
        assert_eq!(
            html,
            "<div data-payload=\"&quot;&#x27;&lt;&gt;&amp;\"></div>"
        );
    }

    /// 属性名ホワイトリストの許可側境界: 英数字に加えて `-` `_` `:` を含む
    /// 実運用上の代表的な属性名（`data-*`・スネークケース・名前空間付き）が
    /// スキップされず出力されることを確認する。
    #[test]
    fn valid_attr_names_with_hyphen_underscore_colon_are_rendered() {
        let node = el(
            "input",
            vec![("data-id", "1"), ("foo_bar", "2"), ("xml:lang", "ja")],
            vec![],
        );
        let html = render(&node);
        assert!(html.contains("data-id=\"1\""));
        assert!(html.contains("foo_bar=\"2\""));
        assert!(html.contains("xml:lang=\"ja\""));
    }

    /// 属性名ホワイトリストの拒否側境界: 空文字列の属性名は
    /// `is_valid_attr_name` が `!name.is_empty()` で拒否し、panic せず
    /// 出力からスキップされることを確認する。
    #[test]
    fn empty_attr_name_is_skipped_without_panic() {
        let node = el("div", vec![("", "value"), ("class", "safe")], vec![]);
        let html = render(&node);
        assert_eq!(html, "<div class=\"safe\"></div>");
    }

    /// タグ名検証の境界: 先頭が ASCII 英字以外（数字・`-`）のタグ名は
    /// `is_valid_tag_name` の `chars.next()` 判定で拒否され、要素全体が
    /// 出力からスキップされることを確認する。空文字列タグ名も同様に拒否する。
    #[test]
    fn tag_name_validation_boundaries() {
        assert_eq!(render(&el("1tag", vec![], vec![text("x")])), "");
        assert_eq!(render(&el("-tag", vec![], vec![text("x")])), "");
        assert_eq!(render(&el("", vec![], vec![text("x")])), "");
    }

    /// 大文字を含むタグ名は `is_valid_tag_name` が `is_ascii_alphabetic` /
    /// `is_ascii_alphanumeric` で判定するため許可される（HTML の小文字化は
    /// 本クレートの責務外であることを現状の実装どおり固定する）。
    #[test]
    fn uppercase_tag_name_is_rendered_as_is() {
        let node = el("DIV", vec![], vec![text("x")]);
        assert_eq!(render(&node), "<DIV>x</DIV>");
    }

    /// `raw_html` の非エスケープが兄弟の `text` ノードへ波及しないことを
    /// 確認する（`raw_html` は唯一の非エスケープ出力点だが、その効果は
    /// 当該ノードに閉じており、木全体を非エスケープ化しない）。
    #[test]
    fn text_and_raw_html_siblings_render_independently() {
        #[expect(
            clippy::disallowed_methods,
            reason = "ESCAPE-REVIEWED: raw_html の非エスケープが兄弟 text ノードへ波及しないことの検証。固定の信頼済み文字列のみ"
        )]
        let node = el(
            "div",
            vec![],
            vec![text("<script>"), raw_html("<b>ok</b>"), text("<script>")],
        );
        let html = render(&node);
        assert_eq!(html, "<div>&lt;script&gt;<b>ok</b>&lt;script&gt;</div>");
    }

    /// 要素で包まない `Node::Text` 単体を直接 `render` に渡すケース。
    /// `render`/`render_into` が `Node::Element` を経由しなくても
    /// エスケープ経路を通ることを確認する。
    #[test]
    fn render_text_node_directly() {
        assert_eq!(render(&Node::Text("<b>".to_string())), "&lt;b&gt;");
    }

    /// 空 children の要素は開始・終了タグのみをレンダリングすることを確認する。
    #[test]
    fn empty_element_renders_open_and_close_tags() {
        let node = el("div", vec![], vec![]);
        assert_eq!(render(&node), "<div></div>");
    }

    /// 深いネスト（10 段）でもスタックオーバーフローや panic なく再帰的に
    /// レンダリングできることを確認する（`render_into` の再帰呼び出しの
    /// 健全性を回帰的に担保する）。
    #[test]
    fn deeply_nested_tree_renders_without_panic() {
        let mut node = text("leaf");
        for _ in 0..10 {
            node = el("div", vec![], vec![node]);
        }
        let html = render(&node);
        assert_eq!(html.matches("<div>").count(), 10);
        assert_eq!(html.matches("</div>").count(), 10);
        assert!(html.contains("leaf"));
    }

    /// TASK-5.1b で追加したタグショートカット（`docs/component-api.md` 第 4 節）
    /// が、それぞれ期待どおりのタグ名で出力されることを一括で固定する。
    /// `main_tag` のみ委譲先タグ名 `"main"` と関数名が異なる点に注意。
    #[test]
    fn tag_shortcuts_render_expected_tag_names() {
        assert_eq!(render(&div(vec![], vec![text("x")])), "<div>x</div>");
        assert_eq!(render(&p(vec![], vec![text("x")])), "<p>x</p>");
        assert_eq!(render(&ul(vec![], vec![text("x")])), "<ul>x</ul>");
        assert_eq!(render(&li(vec![], vec![text("x")])), "<li>x</li>");
        assert_eq!(render(&a(vec![], vec![text("x")])), "<a>x</a>");
        assert_eq!(render(&h1(vec![], vec![text("x")])), "<h1>x</h1>");
        assert_eq!(render(&main_tag(vec![], vec![text("x")])), "<main>x</main>");
    }

    /// タグショートカットは `el()` への薄い委譲であるため、`el()` を直接
    /// 使った場合と出力が完全に一致することを確認する（`docs/component-api.md`
    /// 第 4 節・定義規則 1〜3 が求める「独自の出力経路を持たない」ことの回帰）。
    #[test]
    fn tag_shortcut_output_matches_direct_el_call() {
        let via_shortcut = div(vec![("class", "card")], vec![p(vec![], vec![])]);
        let via_el = el(
            "div",
            vec![("class", "card")],
            vec![el("p", vec![], vec![])],
        );
        assert_eq!(render(&via_shortcut), render(&via_el));
    }

    /// ショートカット経由でもテキスト・属性値の既定エスケープ（REQ-1）が
    /// 迂回されないことを XSS ペイロードで確認する。
    #[test]
    fn tag_shortcuts_escape_text_and_attrs_by_default() {
        let payload = "<script>alert(1)</script>";
        let node = div(
            vec![("title", "\"><script>alert(1)</script>")],
            vec![p(vec![], vec![text(payload)])],
        );
        let html = render(&node);
        assert!(
            !html.contains("<script>"),
            "ショートカット経由でエスケープが迂回された: {html}"
        );
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&quot;&gt;&lt;script&gt;"));
    }

    /// ショートカットと `raw_html` を組み合わせても、非エスケープ出力点が
    /// `raw_html` のみであるという境界（不変条件 2）が変わらないことを確認する。
    #[test]
    fn tag_shortcut_combined_with_raw_html_keeps_opt_in_boundary() {
        #[expect(
            clippy::disallowed_methods,
            reason = "ESCAPE-REVIEWED: タグショートカット併用時もオプトイン境界が変わらないことの検証。固定の信頼済み文字列のみ"
        )]
        let node = div(
            vec![],
            vec![text("<script>"), raw_html("<b>ok</b>"), text("<script>")],
        );
        let html = render(&node);
        assert_eq!(html, "<div>&lt;script&gt;<b>ok</b>&lt;script&gt;</div>");
    }
}
