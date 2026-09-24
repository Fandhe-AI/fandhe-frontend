//! `list` 部品の契約テスト（イシュー #2657）。
//!
//! `crates/wireframe-ui/tests/avatar.rs`・`crates/wireframe-ui/tests/breadcrumbs.rs`
//! と同型の観点（非対話制約・XSS 回帰・CSS 配線）に加え、番号付きモード
//! のカウンタがそのモード専用セレクタへ正しくスコープされていることを
//! 固定する。
use fandhe_frontend_core::{render, text};
use fandhe_frontend_wireframe_ui::{list, wireframe_css, PARTS};

#[test]
fn unordered_renders_root_class_without_ordered_modifier() {
    let node = list(vec![text("A"), text("B")], false);
    let html = render(&node);
    assert!(html.contains(r#"class="fw-wire-list""#));
    assert!(!html.contains("fw-wire-list-ordered"));
    assert!(html.starts_with("<div"));
    assert!(html.trim_end().ends_with("</div>"));
}

#[test]
fn ordered_appends_ordered_modifier() {
    let node = list(vec![text("A"), text("B")], true);
    let html = render(&node);
    assert!(html.contains(r#"class="fw-wire-list fw-wire-list-ordered""#));
}

#[test]
fn renders_one_item_wrapper_per_item_in_order() {
    let node = list(vec![text("A"), text("B"), text("C")], false);
    let html = render(&node);
    assert_eq!(html.matches(r#"class="fw-wire-list-item""#).count(), 3);

    let idx_a = html.find('A').unwrap();
    let idx_b = html.find('B').unwrap();
    let idx_c = html.find('C').unwrap();
    assert!(idx_a < idx_b);
    assert!(idx_b < idx_c);
}

#[test]
fn empty_items_renders_root_only() {
    let unordered = render(&list(vec![], false));
    assert!(!unordered.contains("fw-wire-list-item"));
    assert!(unordered.contains(r#"class="fw-wire-list""#));

    let ordered = render(&list(vec![], true));
    assert!(!ordered.contains("fw-wire-list-item"));
    assert!(ordered.contains(r#"class="fw-wire-list fw-wire-list-ordered""#));
}

#[test]
fn xss_regression_item_text_is_escaped() {
    let payload_a = "<script>alert(1)</script>";
    let payload_b = "\"><img src=x onerror=alert(1)>";
    let html_a = render(&list(vec![text(payload_a)], false));
    let html_b = render(&list(vec![text(payload_b)], false));

    assert!(!html_a.contains(payload_a));
    assert!(html_a.contains("&lt;script&gt;"));
    assert!(!html_b.contains(payload_b));
    assert!(html_b.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics() {
    let html = render(&list(vec![text("A"), text("B")], true));
    for forbidden in [
        " role=\"",
        " aria-",
        " tabindex=\"",
        " style=\"",
        "<button",
        "<a ",
        "href=",
        "<input",
        "<select",
        "<ul",
        "<ol",
        "<li",
        "javascript:",
        " onclick=\"",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn no_data_attributes() {
    let html = render(&list(vec![text("A")], true));
    assert!(!html.contains("data-"));
}

#[test]
fn list_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::list::LIST_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::list::LIST_CSS));
}

#[test]
fn list_css_selectors_use_fw_wire_prefix_and_no_pre_styled_ui_prefix() {
    let css = fandhe_frontend_wireframe_ui::list::LIST_CSS;
    for line in css.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('.') {
            assert!(
                trimmed.starts_with(".fw-wire-"),
                "selector line should start with .fw-wire-: {line:?}"
            );
        }
    }
    assert!(!css.contains("--fandhe-"));
}

#[test]
fn nested_list_anywhere_inside_an_item_is_indented() {
    let css = fandhe_frontend_wireframe_ui::list::LIST_CSS;

    // 項目内に現れた入れ子 `.fw-wire-list`（直接の子・孫のいずれも）は
    // 子孫セレクタで左マージンの字下げを受ける（イシュー #2657 Review
    // 指摘の是正）。子結合子 `>` ではなく子孫結合子（空白）を使うのは、
    // 正しい使い方（`div(vec![], vec![text(..), list(..)])` で本文と
    // 入れ子を 1 項目へ合成する）だと入れ子リストが項目ラッパーの孫に
    // なるため。
    assert!(
        css.contains(".fw-wire-list > .fw-wire-list-item .fw-wire-list {"),
        "missing nested list indentation rule: {css:?}"
    );
    let nested_rule_start = css
        .find(".fw-wire-list > .fw-wire-list-item .fw-wire-list {")
        .unwrap();
    let nested_rule_end = css[nested_rule_start..].find('}').unwrap() + nested_rule_start;
    let nested_rule_body = &css[nested_rule_start..nested_rule_end];
    assert!(nested_rule_body.contains("margin-left:"));

    // 直接の子限定（子結合子 `>`）の規則は残っていない（誤用パターン
    // 〔入れ子 list() を items の別要素として並べる〕を暗黙に想定した
    // セレクタを持ち込まない）。
    assert!(!css.contains(".fw-wire-list > .fw-wire-list-item > .fw-wire-list {"));
    assert!(!css.contains(":has("));
}

#[test]
fn nested_list_composed_via_div_stays_within_a_single_item_and_ordinal() {
    // 正しい入れ子の作り方: 「本文 + 入れ子 list()」を 1 つの Node へ
    // 合成してから items の 1 要素として渡す（list::list rustdoc
    // 「入れ子リスト」節参照）。入れ子 list() を items の別要素として
    // 並べると、ネストではなく単なる隣接項目になりカウンタも余分に
    // 1 つ進んでしまう（イシュー #2657 Review 指摘、当初の docs デモは
    // この誤用パターンだった）。
    let nested = list(vec![text("子項目 A-1"), text("子項目 A-2")], false);
    let parent_item = fandhe_frontend_core::div(vec![], vec![text("親項目 A"), nested]);
    let node = list(vec![parent_item, text("親項目 B")], true);
    let html = render(&node);

    // 項目ラッパーは外側 2 件 + 入れ子 2 件の計 4 件（入れ子 list() 自身の
    // 項目も同じ ITEM_CLASS でラップされるため）。外側の並び順そのものは
    // 「親項目 A（本文 + 入れ子）」「親項目 B」の 2 件のままで、入れ子側が
    // 外側 items の別要素として増えるわけではない点が誤用パターンとの差。
    assert_eq!(html.matches(r#"class="fw-wire-list-item""#).count(), 4);
    // 外側は ordered（`fw-wire-list-ordered` 付き）、内側は unordered
    // （付かない）で 1 件ずつ。カウンタ（CSS カウンタ、DOM には現れない）
    // は `.fw-wire-list.fw-wire-list-ordered > .fw-wire-list-item` に限り
    // 子結合子で適用されるため、内側 unordered 側の項目は外側の番号
    // カウンタを消費しない（`counter_rules_are_scoped_to_ordered_modifier`
    // が CSS 側のスコープを別途固定する）。
    assert_eq!(
        html.matches(r#"class="fw-wire-list fw-wire-list-ordered""#)
            .count(),
        1
    );
    assert_eq!(html.matches(r#"class="fw-wire-list""#).count(), 1);
    assert!(html.contains("子項目 A-1"));
    assert!(html.contains("子項目 A-2"));
    // 入れ子リストにも対話セマンティクスが混入しない。
    assert!(!html.contains("<ul"));
    assert!(!html.contains("<ol"));
    assert!(!html.contains("<li"));
}

#[test]
fn unordered_marker_aligns_to_first_line_not_whole_item_center() {
    // Cursor Bugbot 指摘（イシュー #2717 レビュー）: `align-self: center` は
    // 項目全体（入れ子ブロックや複数行 `Node` を含む高さ）の縦中央に
    // マーカーを置いてしまい、複数行項目では行頭からズレる。マーカーを
    // 先頭行に揃えるため `align-self: flex-start` + 先頭行の中央へ寄せる
    // `margin-top` の組み合わせへ変更したことを固定する。
    let css = fandhe_frontend_wireframe_ui::list::LIST_CSS;
    let selector = ".fw-wire-list > .fw-wire-list-item::before {";
    let start = css.find(selector).expect("unordered marker rule missing");
    let end = css[start..].find('}').map(|e| start + e).unwrap();
    let body = &css[start..end];

    assert!(
        body.contains("align-self: flex-start;"),
        "unordered marker should align to the first line, not the whole item center: {body:?}"
    );
    assert!(
        !body.contains("align-self: center;"),
        "unordered marker must not center on the whole (possibly multi-line) item: {body:?}"
    );
    assert!(
        body.contains("margin-top:"),
        "unordered marker needs a margin-top to center within the first line's height: {body:?}"
    );
}

#[test]
fn ordered_marker_resets_margin_top_and_does_not_inherit_unordered_offset() {
    // Cursor Bugbot 指摘（イシュー #2717 レビュー、Medium）: 箇条書き用
    // `.fw-wire-list > .fw-wire-list-item::before { margin-top: ... }` は
    // `.fw-wire-list-ordered` 修飾がついた要素にもカスケードで適用され
    // （番号付き側のセレクタが `margin-top` を宣言しなければ上書きされず
    // 残ってしまう）、番号（`align-self: baseline` でテキストの先頭行
    // ベースラインへ自然に揃うはずの数字）に余分な余白が付く。番号付き
    // 側で明示的に `margin-top: 0` を宣言し、この意図しない継承を防ぐ
    // ことを固定する。
    let css = fandhe_frontend_wireframe_ui::list::LIST_CSS;
    let selector = ".fw-wire-list.fw-wire-list-ordered > .fw-wire-list-item::before {";
    let start = css.find(selector).expect("ordered marker rule missing");
    let end = css[start..].find('}').map(|e| start + e).unwrap();
    let body = &css[start..end];

    assert!(
        body.contains("margin-top: 0;") || body.contains("margin-top: 0em;"),
        "ordered marker must reset margin-top to 0 to avoid inheriting the unordered marker's first-line offset: {body:?}"
    );
}

#[test]
fn counter_rules_are_scoped_to_ordered_modifier() {
    let css = fandhe_frontend_wireframe_ui::list::LIST_CSS;

    // `.fw-wire-list { ... }` 形式のルールをセレクタ単位に分割し、
    // `counter(`/`counter-increment` を含むルールのセレクタが必ず
    // `fw-wire-list-ordered` を含むことを確認する。
    let mut remaining = css;
    while let Some(open) = remaining.find('{') {
        let selector = remaining[..open].trim();
        let close = remaining[open..]
            .find('}')
            .map(|end| open + end)
            .expect("rule must be closed");
        let body = &remaining[open + 1..close];

        if body.contains("counter(") || body.contains("counter-increment") {
            assert!(
                selector.contains("fw-wire-list-ordered"),
                "counter rule must be scoped to fw-wire-list-ordered: {selector:?}"
            );
        }

        remaining = &remaining[close + 1..];
    }
}
