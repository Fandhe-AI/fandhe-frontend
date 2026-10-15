//! ページの外枠（`<!DOCTYPE html>` + `html`/`head`/`body`）を組み立てる [`Document`] builder。
//!
//! `fandhe-frontend-app` はモード非依存の純関数層であり、本モジュールも HTTP ヘッダ・
//! キャッシュ方針を持たない（それらは SSR ハンドラ側の責務）。`page_shell` が固定している
//! lang・CSS・script を、fandhe-backend の認証画面のように hydrate.js が不要・CSS の置き場所が
//! 異なる・厳格な CSP を掛ける呼び出し側が指定できるようにするために存在する。
//!
//! # セキュリティ不変条件
//!
//! - title / lang / href / src / head ノードのテキストはすべて `fandhe_frontend_core` の
//!   `el` / `text` 経由で既定エスケープされる（REQ-1）。独自エスケープ・独自 raw 経路は持たない。
//! - stylesheet の href と script の src は core の属性出力と同じ `is_safe_url`
//!   （許可リスト方式）で判定し、不合格なら **その要素ごと出力しない**（fail-closed）。
//! - `<script>` 本文（inline script）を渡す経路はなく、`src` 参照のみ。既定で inline `<style>` も
//!   出さないため厳格 CSP と両立する。
//! - CSP meta の値も `el` の属性出力を通り、`"` `<` `>` `'` `&` は必ずエスケープされる。
//! - 文字列でヘッドを差し込む API はなく、`Node` のみ受け付ける。

use fandhe_frontend_core::{el, is_safe_url, render, text, Node};

/// `@view-transition { navigation: auto; }` の固定リテラル（`page_shell` と同値）。
///
/// [`ViewTransitions::InlineStyle`] が `<style>` へ出す内容であり、
/// [`ViewTransitions::Stylesheet`] を使う呼び出し側が自前の CSS へ書き写すための定数でもある。
pub const VIEW_TRANSITION_CSS: &str = "@view-transition { navigation: auto; }";

/// View Transitions の at-rule（[`VIEW_TRANSITION_CSS`]）を出す方法。
///
/// 既定は [`ViewTransitions::Disabled`]（何も出さない）。`style-src 'self'` のような厳格な CSP と
/// [`ViewTransitions::InlineStyle`] は両立しないため、その場合は [`ViewTransitions::Stylesheet`] を使い、
/// 配信する CSS に [`VIEW_TRANSITION_CSS`] を書く。矛盾の機械検査（強制）は行わない。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum ViewTransitions {
    /// 何も出さない（既定）。inline `<style>` を出さない。
    #[default]
    Disabled,
    /// `<style>` に [`VIEW_TRANSITION_CSS`] を出す（`page_shell` 互換）。inline style を伴う。
    InlineStyle,
    /// 外部 CSS への `link rel="stylesheet"` を出す。URL が安全でなければ要素ごと省略（fail-closed）。
    Stylesheet(String),
}

/// script の読み込み種別。bool 引数による取り違えを避けるため非公開 enum で保持する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScriptKind {
    /// `type="module"`
    Module,
    /// 通常の classic script
    Classic,
}

/// ページ外枠の builder。`render()` で DOCTYPE 付きの完全な HTML 文字列を返す。
///
/// head の出力順は `meta charset` → `meta viewport` → CSP meta（指定時）→ View Transitions
/// スロット（[`ViewTransitions`]）→ `title` → stylesheet（指定順）→ [`Document::head_node`]（指定順）。
/// script は body 末尾（利用者 body の後）に指定順で出す。CSP meta は meta 以降の要素にしか効かない
/// ため、title・style・link・script より前に固定配置する。
/// `lang` の既定は `"ja"`（`page_shell` と同値）。値は既定エスケープのみで BCP 47 検証はしない。
///
/// HTTP ヘッダ（Cache-Control 等）は本型の責務外で、ハンドラ側で設定する。CSP は
/// [`Document::content_security_policy`] で meta として出せるが、meta では `frame-ancestors`・
/// `report-uri`・`report-to`・`sandbox` が無効で clickjacking 対策にならないため、可能なら
/// HTTP ヘッダでの送出を推奨する。CSP は二次防御であり、一次防御は既定エスケープである。
///
/// # CSP 厳格構成のレシピ（inline の style / script を一切出さない）
///
/// `.content_security_policy(..)` + `.view_transitions(ViewTransitions::Stylesheet(..))` +
/// `.stylesheet(..)` + `.script_module(..)` の組み合わせで、inline の `<style>`・`<script>` 本文を
/// 一切出さない。ただし [`Document::head_node`] や body に inline style / script を入れないことは
/// 利用者の責任であり、本型は渡されたノードを走査・除外しない（部分的なフィルタは誤った安心感を
/// 与え、受け付けたノードを黙って落とす挙動は決定性を下げるため）。
///
/// ```
/// use fandhe_frontend_app::{Document, ViewTransitions};
/// use fandhe_frontend_core::{el, text};
///
/// let html = Document::new("Sign in", el("p", vec![], vec![text("hi")]))
///     .content_security_policy("default-src 'none'; style-src 'self'; script-src 'self'")
///     .view_transitions(ViewTransitions::Stylesheet("/assets/vt.css".into()))
///     .stylesheet("/assets/app.css")
///     .script_module("/assets/app.js")
///     .render();
/// assert!(html.contains("http-equiv=\"Content-Security-Policy\""));
/// assert!(!html.contains("<style"));
/// assert!(html.contains("<script type=\"module\" src=\"/assets/app.js\"></script>"));
/// ```
///
/// # Examples
///
/// ```
/// use fandhe_frontend_app::Document;
/// use fandhe_frontend_core::{el, text};
///
/// let form = el("form", vec![("method", "post")], vec![text("Sign in")]);
/// let html = Document::new("Sign in", form)
///     .lang("en")
///     .stylesheet("/assets/auth.css")
///     .script_module("/assets/auth.js")
///     .render();
/// assert!(html.starts_with("<!DOCTYPE html>\n"));
/// assert!(html.contains("<title>Sign in</title>"));
/// assert!(html.contains("/assets/auth.css"));
/// assert!(html.contains("type=\"module\""));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    title: String,
    lang: String,
    body: Node,
    stylesheets: Vec<String>,
    scripts: Vec<(ScriptKind, String)>,
    head_nodes: Vec<Node>,
    csp: Option<String>,
    view_transitions: ViewTransitions,
}

impl Document {
    /// タイトルと body 要素の内容から builder を作る（lang は `"ja"`）。
    pub fn new(title: impl Into<String>, body: Node) -> Self {
        Self {
            title: title.into(),
            lang: "ja".to_string(),
            body,
            stylesheets: Vec::new(),
            scripts: Vec::new(),
            head_nodes: Vec::new(),
            csp: None,
            view_transitions: ViewTransitions::Disabled,
        }
    }

    /// CSP を `meta http-equiv="Content-Security-Policy"` として出す（後から呼んだ値で上書き）。
    ///
    /// 値は任意文字列で、`el` の属性出力により既定エスケープされる。型で組まないのは、
    /// (1) 型にしても REQ-1 上の得がない、(2) ディレクティブ集合が進化し続け追随コストが高い、
    /// (3) 弱いポリシーの指定は型でも防げない、(4) docs-site の CSP 運用と一貫する、ためである。
    ///
    /// meta CSP では `frame-ancestors`・`report-uri`・`report-to`・`sandbox` が無効なので、
    /// clickjacking 対策を含め HTTP ヘッダでの送出を推奨する。meta は charset・viewport の直後
    /// （title・style・link・script より前）に置く。
    pub fn content_security_policy(mut self, policy: impl Into<String>) -> Self {
        self.csp = Some(policy.into());
        self
    }

    /// View Transitions の at-rule の出し方を選ぶ（既定は [`ViewTransitions::Disabled`]）。
    ///
    /// 出力位置は viewport（と CSP meta）の直後・title の前で、`page_shell` と同位置。
    pub fn view_transitions(mut self, vt: ViewTransitions) -> Self {
        self.view_transitions = vt;
        self
    }

    /// `html` 要素の `lang` を上書きする。
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.lang = lang.into();
        self
    }

    /// `link rel="stylesheet"` を追加する（複数可・指定順）。
    /// 安全でない URL（`javascript:` 等）は要素ごと省略される。
    pub fn stylesheet(mut self, href: impl Into<String>) -> Self {
        self.stylesheets.push(href.into());
        self
    }

    /// `script type="module"` を body 末尾へ追加する（複数可・指定順）。
    /// 安全でない URL は要素ごと省略される。
    pub fn script_module(mut self, src: impl Into<String>) -> Self {
        self.scripts.push((ScriptKind::Module, src.into()));
        self
    }

    /// classic な `script` を body 末尾へ追加する（複数可・指定順）。
    /// 安全でない URL は要素ごと省略される。
    pub fn script(mut self, src: impl Into<String>) -> Self {
        self.scripts.push((ScriptKind::Classic, src.into()));
        self
    }

    /// head へ任意のノード（`meta` / `style` 等）を追加する（複数可・指定順）。
    ///
    /// 信頼できない入力を `raw_html` で包んで渡してはならない（利用者責任）。
    pub fn head_node(mut self, node: Node) -> Self {
        self.head_nodes.push(node);
        self
    }

    /// DOCTYPE 付きの完全な HTML 文字列を返す。同一入力に対し常に同一出力。
    ///
    /// 前置する `<!DOCTYPE html>\n` は利用者入力を含まない固定リテラルで、
    /// 新たなエスケープ迂回経路ではない。
    pub fn render(&self) -> String {
        let mut head = vec![
            el("meta", vec![("charset", "utf-8")], vec![]),
            el(
                "meta",
                vec![
                    ("name", "viewport"),
                    ("content", "width=device-width, initial-scale=1"),
                ],
                vec![],
            ),
        ];
        if let Some(policy) = &self.csp {
            head.push(el(
                "meta",
                vec![
                    ("http-equiv", "Content-Security-Policy"),
                    ("content", policy.as_str()),
                ],
                vec![],
            ));
        }
        match &self.view_transitions {
            ViewTransitions::Disabled => {}
            // 内容は利用者入力を含まない固定リテラルで、text() 経由（既定エスケープ）で出す。
            ViewTransitions::InlineStyle => {
                head.push(el("style", vec![], vec![text(VIEW_TRANSITION_CSS)]));
            }
            ViewTransitions::Stylesheet(href) => {
                if is_safe_url(href) {
                    head.push(el(
                        "link",
                        vec![("rel", "stylesheet"), ("href", href.as_str())],
                        vec![],
                    ));
                }
            }
        }
        head.push(el("title", vec![], vec![text(&self.title)]));
        for href in self.stylesheets.iter().filter(|h| is_safe_url(h)) {
            head.push(el(
                "link",
                vec![("rel", "stylesheet"), ("href", href)],
                vec![],
            ));
        }
        head.extend(self.head_nodes.iter().cloned());

        let mut body_children = vec![self.body.clone()];
        for (kind, src) in self.scripts.iter().filter(|(_, s)| is_safe_url(s)) {
            let attrs = match kind {
                ScriptKind::Module => vec![("type", "module"), ("src", src.as_str())],
                ScriptKind::Classic => vec![("src", src.as_str())],
            };
            body_children.push(el("script", attrs, vec![]));
        }

        let html = el(
            "html",
            vec![("lang", self.lang.as_str())],
            vec![el("head", vec![], head), el("body", vec![], body_children)],
        );
        let mut out = String::from("<!DOCTYPE html>\n");
        out.push_str(&render(&html));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body() -> Node {
        el("p", vec![], vec![text("hi")])
    }

    #[test]
    fn minimal() {
        let s = Document::new("T", body()).render();
        assert_eq!(
            s,
            "<!DOCTYPE html>\n<html lang=\"ja\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>T</title></head><body><p>hi</p></body></html>"
        );
    }

    #[test]
    fn full_order_and_lang() {
        let s = Document::new("T", body())
            .lang("en")
            .stylesheet("/a.css")
            .stylesheet("/b.css")
            .head_node(el("style", vec![], vec![text("p{}")]))
            .script_module("/m.js")
            .script("/c.js")
            .render();
        assert!(s.contains("<html lang=\"en\">"));
        let a = s.find("/a.css").unwrap();
        let b = s.find("/b.css").unwrap();
        let st = s.find("<style>").unwrap();
        let t = s.find("<title>").unwrap();
        assert!(t < a && a < b && b < st);
        assert!(s.contains("<p>hi</p><script type=\"module\" src=\"/m.js\"></script><script src=\"/c.js\"></script></body>"));
    }

    #[test]
    fn deterministic() {
        let d = Document::new("T", body()).stylesheet("/a.css");
        assert_eq!(d.render(), d.render());
    }

    #[test]
    fn unsafe_urls_dropped_whole_element() {
        let s = Document::new("T", body())
            .stylesheet("javascript:alert(1)")
            .stylesheet("/ok.css")
            .script_module("data:text/javascript,alert(1)")
            .script("/ok.js")
            .render();
        assert!(!s.contains("javascript:"));
        assert!(!s.contains("data:"));
        assert_eq!(s.matches("<link").count(), 1);
        assert_eq!(s.matches("<script").count(), 1);
        assert!(s.contains("/ok.css") && s.contains("/ok.js"));
    }

    #[test]
    fn page_shell_output_unchanged() {
        let s = crate::page_shell("T", body());
        assert_eq!(
            s,
            "<!DOCTYPE html>\n<html lang=\"ja\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><style>@view-transition { navigation: auto; }</style><title>T</title><link rel=\"stylesheet\" href=\"/static/style.css\"></head><body><p>hi</p><script type=\"module\" src=\"/static/hydrate.js\"></script></body></html>"
        );
    }

    #[test]
    fn csp_meta_position_and_escape() {
        let policy = "default-src 'none'; style-src 'self'";
        let s = Document::new("T", body())
            .content_security_policy("x")
            .content_security_policy(policy)
            .render();
        let expected = format!(
            "<meta http-equiv=\"Content-Security-Policy\" content=\"{}\">",
            fandhe_frontend_core::escape_html(policy)
        );
        assert!(s.contains(&expected), "{s}");
        assert_eq!(s.matches("http-equiv").count(), 1);
        let vp = s.find("name=\"viewport\"").unwrap();
        let csp = s.find("http-equiv").unwrap();
        let t = s.find("<title>").unwrap();
        assert!(vp < csp && csp < t);
    }

    #[test]
    fn view_transitions_inline_slot_order() {
        let s = Document::new("T", body())
            .content_security_policy("default-src 'self'")
            .view_transitions(ViewTransitions::InlineStyle)
            .stylesheet("/a.css")
            .head_node(el("meta", vec![("name", "x")], vec![]))
            .render();
        let vp = s.find("name=\"viewport\"").unwrap();
        let csp = s.find("http-equiv").unwrap();
        let st = s
            .find("<style>@view-transition { navigation: auto; }</style>")
            .unwrap();
        let t = s.find("<title>").unwrap();
        let a = s.find("/a.css").unwrap();
        let h = s.find("name=\"x\"").unwrap();
        assert!(vp < csp && csp < st && st < t && t < a && a < h);
    }

    #[test]
    fn view_transitions_stylesheet_external() {
        let s = Document::new("T", body())
            .view_transitions(ViewTransitions::Stylesheet("/vt.css".into()))
            .render();
        assert!(!s.contains("<style"));
        assert_eq!(s.matches("/vt.css").count(), 1);
        assert!(s.find("/vt.css").unwrap() < s.find("<title>").unwrap());
        let bad = Document::new("T", body())
            .view_transitions(ViewTransitions::Stylesheet("javascript:alert(1)".into()))
            .render();
        assert!(!bad.contains("<link") && !bad.contains("javascript:"));
    }

    #[test]
    fn inline_free_configuration() {
        let s = Document::new("T", body())
            .content_security_policy("default-src 'none'; style-src 'self'")
            .view_transitions(ViewTransitions::Stylesheet("/vt.css".into()))
            .stylesheet("/a.css")
            .script_module("/m.js")
            .script("/c.js")
            .render();
        assert!(!s.contains("<style"));
        let mut n = 0;
        for (i, _) in s.match_indices("<script") {
            let rest = &s[i..];
            let gt = rest.find('>').unwrap();
            assert!(rest[gt + 1..].starts_with("</script>"), "{s}");
            n += 1;
        }
        assert_eq!(n, 2);
    }

    #[test]
    fn default_has_no_inline_or_csp() {
        let s = Document::new("T", body()).render();
        assert!(!s.contains("<style") && !s.contains("http-equiv"));
    }

    #[test]
    fn page_shell_delegates_to_document() {
        let d = Document::new("T", body())
            .view_transitions(ViewTransitions::InlineStyle)
            .stylesheet("/static/style.css")
            .script_module("/static/hydrate.js")
            .render();
        assert_eq!(crate::page_shell("T", body()), d);
    }
}
