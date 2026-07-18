//! TASK-5.2b（#35、親 #33）: 生成 HTML の「素直さ」回帰テスト。
//!
//! REQ-5（独自 DSL に依存しないプレーン Rust コンポーネント記述、
//! `docs/spec/04-requirements.md`）の受け入れ基準のうち、「生成される HTML が、
//! 観測用属性（`data-*`）以外にフレームワーク固有のカスタム要素・不透明な
//! マーカーを含まないこと」を自動回帰として固定する。PoC-3
//! （`docs/spec/03-poc/rendering-web-standards/`）で定性的に確認済みの
//! 「素直さ」を、以後の `rws_core::render()` 実装変更に対する回帰検知として
//! 機械的に担保する。
//!
//! テスト観点（T1〜T7）は TASK-5.2a（#34）の設計書
//! `docs/plain-html-output-test-design.md`（PR #182）の観点表に従う。
//! 設計書と本ファイルの実装が乖離した場合は設計書を正とする。
//!
//! # 本ファイルの位置づけ（`xss_escape.rs` との関係）
//!
//! 本ファイルは既定エスケープ検証（`core/tests/xss_escape.rs`）の**代替では
//! なく補完**である。エスケープが「危険な入力を安全な表現に変換すること」を
//! 保証するのに対し、本ファイルは「安全な入力に対して余計なものを注入しない
//! こと（素直さ）」を保証する。両者は独立した不変条件であり、一方が他方を
//! 包含しない。
//!
//! # 削除・弱体化の禁止
//!
//! 本ファイルの回帰テストは `.claude/rules/coding-rust.md` の規約により、
//! 以後の削除・弱体化・`#[ignore]` 化を禁止する（`xss_escape.rs` と同格の
//! 保護対象とする）。
//!
//! # T8 の扱い（スコープ外）
//!
//! 設計書第 4・5 節に従い、タグショートカット（`div()`/`p()` 等）と `el()` の
//! 等価性を検証する T8 は本ファイルでは実装しない。TASK-5.1b（#31、タグ
//! ショートカット実装）が未完了のため対象 API が存在しないことが理由であり、
//! #31 完了後のフォローアップとして親タスク #33 に引き継ぐ（出力対象外
//! 追跡、`.claude/rules/out-of-scope-tracking.md`）。

use rws_core::{el, raw_html, render, text};

/// フレームワーク固有の不透明マーカーとして出力に現れてはならない代表的な
/// パターン集合。「フレームワーク固有のカスタム要素・不透明なマーカー」
/// （REQ-5 受け入れ基準）の否定 assert に横断的に使う。
///
/// - `<rws-`: フレームワーク独自のカスタム要素タグ名の接頭辞
/// - `<!--`: フレームワークがハイドレーション境界等に挿入しうるコメントマーカー
/// - `data-rws-`: フレームワーク内部専用の観測属性接頭辞
///   （利用者が明示的に指定した `data-*` 属性はこの接頭辞を含まない前提）
///
/// `xss_escape.rs` の `XSS_PAYLOADS` と同格の保護対象とし、削除・弱体化を
/// 禁止する。
const FORBIDDEN_MARKERS: &[&str] = &["<rws-", "<!--", "data-rws-"];

/// 出力 HTML に [`FORBIDDEN_MARKERS`] のいずれも含まれないことを確認する
/// 共有ヘルパー。各テストが完全一致 assert に加えて横断的に呼び出す
/// （設計書第 3 節: 完全一致を第一の検証手段とし、否定 assert を補助として
/// 併用する優先順位に対応）。
fn assert_no_forbidden_markers(html: &str) {
    for marker in FORBIDDEN_MARKERS {
        assert!(
            !html.contains(marker),
            "禁止パターン `{marker}` が出力に含まれていた: {html}"
        );
    }
}

/// T1: 単一要素の出力が入力から素直に導出された文字列と完全一致することを
/// 確認する（属性・子ノードの追加注入がないことの最小ケース）。
#[test]
fn single_element_renders_exactly_as_derived_from_input() {
    let node = el("p", vec![("class", "greeting")], vec![text("hello")]);
    let html = render(&node);
    assert_eq!(html, r#"<p class="greeting">hello</p>"#);
    assert_no_forbidden_markers(&html);
}

/// T2: 3 階層以上の入れ子ノード木でも、出力が入力からの素直な導出結果と
/// 完全一致し、要素・属性・コメントが注入されないことを確認する。
#[test]
fn nested_tree_has_no_injected_elements_or_comments() {
    let tree = el(
        "section",
        vec![("id", "app")],
        vec![el(
            "ul",
            vec![("class", "list")],
            vec![
                el("li", vec![], vec![text("item1")]),
                el("li", vec![], vec![text("item2")]),
            ],
        )],
    );
    let html = render(&tree);
    assert_eq!(
        html,
        r#"<section id="app"><ul class="list"><li>item1</li><li>item2</li></ul></section>"#
    );
    assert_no_forbidden_markers(&html);
}

/// T3: 代表的なノード木群（単純要素・入れ子・属性つき・`raw_html` 併用）の
/// 出力いずれにも [`FORBIDDEN_MARKERS`] が一切含まれないことを、否定 assert
/// として横断的に確認する。
#[test]
fn output_never_contains_framework_markers() {
    #[expect(
        clippy::disallowed_methods,
        reason = "ESCAPE-REVIEWED: FORBIDDEN_MARKERS 非混入の横断確認に raw_html ノードも含める。固定の信頼済み文字列のみで外部入力を含まない"
    )]
    let trees = vec![
        el("div", vec![], vec![text("plain")]),
        el(
            "article",
            vec![("data-testid", "card")],
            vec![el("h2", vec![], vec![text("title")])],
        ),
        el(
            "div",
            vec![],
            vec![raw_html("<em>trusted</em>"), text("after")],
        ),
        el("input", vec![("type", "text"), ("value", "")], vec![]),
    ];

    for tree in &trees {
        let html = render(tree);
        assert_no_forbidden_markers(&html);
    }
}

/// T4: 利用者が明示的に指定した `data-*` 属性はそのまま出力され
/// （完全一致）、利用者が指定していない `data-*` 属性が出力に一切
/// 出現しないことを確認する（観測用属性の素直さ）。
#[test]
fn user_specified_data_attrs_pass_through_and_no_others_appear() {
    let node = el(
        "button",
        vec![("data-testid", "submit-button"), ("type", "submit")],
        vec![text("Submit")],
    );
    let html = render(&node);
    assert_eq!(
        html,
        r#"<button data-testid="submit-button" type="submit">Submit</button>"#
    );
    // 利用者が指定した `data-testid` 以外の `data-*` は出現しないため、
    // `data-` の出現回数は指定した 1 件と一致するはずである。
    assert_eq!(html.matches("data-").count(), 1);
    assert_no_forbidden_markers(&html);
}

/// T5: PoC-3 実測ページ（`docs/spec/03-poc/rendering-web-standards/`）相当の
/// list/detail 風複合ノード木の出力を、期待文字列との完全一致でスナップ
/// ショット固定する。設計書の「完全一致を第一とする」優先順位に従い、緩い
/// 部分一致検証へは逃げない。将来 `render()` の出力仕様変更（void 要素の
/// 自己終了等）で本テストの期待値更新が必要になり得る点は許容する。
#[test]
fn poc3_like_composite_page_matches_snapshot() {
    let page = el(
        "main",
        vec![("data-testid", "item-list")],
        vec![
            el(
                "nav",
                vec![("data-nav", "primary")],
                vec![el(
                    "a",
                    vec![("href", "/items/1"), ("data-nav", "item-1")],
                    vec![text("Item 1")],
                )],
            ),
            el(
                "section",
                vec![("data-testid", "item-detail")],
                vec![
                    el("h1", vec![], vec![text("Item 1")]),
                    el("p", vec![], vec![text("description")]),
                ],
            ),
        ],
    );
    let html = render(&page);
    assert_eq!(
        html,
        concat!(
            r#"<main data-testid="item-list">"#,
            r#"<nav data-nav="primary">"#,
            r#"<a href="/items/1" data-nav="item-1">Item 1</a>"#,
            "</nav>",
            r#"<section data-testid="item-detail">"#,
            "<h1>Item 1</h1>",
            "<p>description</p>",
            "</section>",
            "</main>",
        )
    );
    assert_no_forbidden_markers(&html);
}

/// T6: `raw_html` の前後に core がラッパー・マーカーを注入しないことを
/// 確認する（完全一致）。エスケープされないこと自体の再検証（`xss_escape.rs`
/// が既に担保）は目的としない。信頼できる固定文字列のみを渡す
/// （`raw_html` の乱用を促す例は書かない）。
#[test]
fn raw_html_content_is_emitted_without_wrapper_markers() {
    #[expect(
        clippy::disallowed_methods,
        reason = "ESCAPE-REVIEWED: raw_html の前後にラッパー・マーカーが注入されないことの検証。固定の信頼済み文字列のみ"
    )]
    let node = el(
        "div",
        vec![],
        vec![text("before-"), raw_html("<b>bold</b>"), text("-after")],
    );
    let html = render(&node);
    assert_eq!(html, "<div>before-<b>bold</b>-after</div>");
    assert_no_forbidden_markers(&html);
}

/// T7: 不正なタグ名・不正な属性名の入力に対し、core が代替マーカーや
/// プレースホルダを出力せず、当該要素・当該属性のみを欠落させた素直な結果を
/// 完全一致で返すことを確認する（不正入力時にも不透明な出力を注入しない）。
#[test]
fn invalid_input_skip_emits_no_placeholder() {
    // 不正なタグ名: 型（&'static str）は文字内容までは保証しないため、
    // `Box::leak` で `'static` に昇格させたタグ名注入を模した入力を使う
    // （`core/src/lib.rs` の `invalid_tag_name_is_skipped_without_panic` と
    // 同種の入力）。出力は代替マーカーなしの空文字列と完全一致する。
    let malicious_tag: &'static str =
        Box::leak(String::from("div><script>alert(1)</script").into_boxed_str());
    let invalid_tag_node = el(malicious_tag, vec![], vec![text("safe")]);
    let html = render(&invalid_tag_node);
    assert_eq!(html, "");

    // 不正な属性名: 当該属性のみが欠落し、要素自体・他の正常な属性は
    // 素直に出力される（代替マーカー・プレースホルダを挟まない）。
    let invalid_attr_node = el(
        "div",
        vec![("onmouseover=alert(1) x", "y"), ("class", "safe")],
        vec![text("ok")],
    );
    let html = render(&invalid_attr_node);
    assert_eq!(html, r#"<div class="safe">ok</div>"#);
    assert_no_forbidden_markers(&html);
}
