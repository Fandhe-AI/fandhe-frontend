//! イシュー #2216（slider の vertical 方向の marker デモを showcase へ
//! 追加する）専用の契約テスト。
//!
//! `crates/docs-site/tests/component_pages.rs` / `tests/site_showcase.rs` /
//! `tests/primitive_showcase.rs` は並列実行される他イシューも触り得る
//! 共有ファイルのため変更しない方針（`component_specs_1691.rs` と同じ
//! per-issue テストファイル方式）。本ファイルは `/themes/slider/` と
//! `/primitives/slider/` の 2 ページを検証する: (1) vertical marker が
//! Demo・Examples の両方に載ること、(2) vertical marker が vertical
//! control の内側にあること、(3) hidden-input の name が重複しないこと、
//! (4) レンダリングが決定的であること、(5) 生の `<script` が出力に
//! 含まれないこと（XSS 回帰の軽量確認）。

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::component_page::generated_content;

/// `/themes/slider/` の生成 HTML（1 回分）。
fn themes_slider_html() -> String {
    let node = generated_content("/themes/slider/")
        .expect("generated_content(\"/themes/slider/\") should be Some");
    render(&node)
}

/// `/primitives/slider/` の生成 HTML（1 回分）。
fn primitives_slider_html() -> String {
    let node = generated_content("/primitives/slider/")
        .expect("generated_content(\"/primitives/slider/\") should be Some");
    render(&node)
}

/// `<h2>{name}</h2>` から次の `<h2>` 直前まで（末尾なら文字列末尾まで）を
/// 切り出す。Demo / Examples 節ごとに vertical marker の有無を分けて
/// 検証するための小道具。
fn section_body<'a>(html: &'a str, name: &str) -> &'a str {
    let open = format!("<h2>{name}</h2>");
    let start = html
        .find(&open)
        .unwrap_or_else(|| panic!("page should have an <h2>{name}</h2> section, got: {html}"))
        + open.len();
    let rest = &html[start..];
    let end = rest.find("<h2>").unwrap_or(rest.len());
    &rest[..end]
}

/// `data-part="marker"` を持つ開始タグ（`<div ... >`）を、属性順に依存
/// せず全属性まとめて抽出する。
fn marker_tags(html: &str) -> Vec<&str> {
    let mut tags = Vec::new();
    let mut rest = html;
    while let Some(idx) = rest.find("<div ") {
        let after = &rest[idx..];
        let end = after
            .find('>')
            .unwrap_or_else(|| panic!("unterminated <div ...> tag in: {after}"));
        let tag = &after[..=end];
        if tag.contains(r#"data-part="marker""#) && !tag.contains(r#"data-part="marker-group""#) {
            tags.push(tag);
        }
        rest = &after[end + 1..];
    }
    tags
}

/// `/themes/slider/` の Demo・Examples 両節に `data-orientation="vertical"`
/// の marker が存在する（既定エスケープ経由の属性順に依存しない判定）。
#[test]
fn themes_slider_demo_and_examples_contain_vertical_marker() {
    let html = themes_slider_html();

    let demo = section_body(&html, "Demo");
    let demo_vertical_markers: Vec<&str> = marker_tags(demo)
        .into_iter()
        .filter(|tag| tag.contains(r#"data-orientation="vertical""#))
        .collect();
    assert!(
        !demo_vertical_markers.is_empty(),
        "Demo section should contain at least one vertical marker, got demo body: {demo}"
    );

    let examples = section_body(&html, "Examples");
    let examples_vertical_markers: Vec<&str> = marker_tags(examples)
        .into_iter()
        .filter(|tag| tag.contains(r#"data-orientation="vertical""#))
        .collect();
    assert!(
        !examples_vertical_markers.is_empty(),
        "Examples section should contain at least one vertical marker, got examples body: {examples}"
    );

    // `--fandhe-slider-marker-percent` は styled marker の唯一の動的値
    // 出力点（`crates/pre-styled-ui/src/slider.rs` doc 参照）。
    for tag in demo_vertical_markers
        .iter()
        .chain(examples_vertical_markers.iter())
    {
        assert!(
            tag.contains("--fandhe-slider-marker-percent"),
            "vertical marker should carry --fandhe-slider-marker-percent, got: {tag}"
        );
    }
}

/// `data-part="control"` かつ `data-orientation="vertical"` が Demo /
/// Examples それぞれに少なくとも 1 件存在する（marker が vertical
/// スライダー内に配置されていることの間接確認）。
#[test]
fn themes_slider_vertical_marker_group_sits_inside_vertical_control() {
    let html = themes_slider_html();

    for section in ["Demo", "Examples"] {
        let body = section_body(&html, section);
        assert!(
            body.contains(r#"data-part="control" data-orientation="vertical""#),
            "{section} section should contain a vertical control, got: {body}"
        );
    }
}

/// slider の `hidden_input` はフォーム送信専用パーツであり、`name` が
/// ページ内で重複すると実運用のフォーム送信で値が衝突する。vertical
/// インスタンス追加（`volume-vertical` / `brightness-vertical`）後も
/// 一意性が保たれることを固定する。
#[test]
fn themes_slider_hidden_input_names_are_unique() {
    let html = themes_slider_html();
    let mut rest = html.as_str();
    let mut names = Vec::new();
    while let Some(idx) = rest.find(r#"data-part="hidden-input""#) {
        let after = &rest[idx..];
        let end = after
            .find('>')
            .unwrap_or_else(|| panic!("unterminated hidden-input tag in: {after}"));
        let tag = &after[..end];
        let name_marker = "name=\"";
        let name_start = tag
            .find(name_marker)
            .unwrap_or_else(|| panic!("hidden-input tag should have a name attribute: {tag}"))
            + name_marker.len();
        let name_end = tag[name_start..]
            .find('"')
            .unwrap_or_else(|| panic!("unterminated name attribute in: {tag}"));
        names.push(tag[name_start..name_start + name_end].to_string());
        rest = &after[end + 1..];
    }
    assert!(
        names.len() >= 7,
        "expected at least 7 hidden-input parts on /themes/slider/, got {names:?}"
    );
    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        names.len(),
        "hidden-input name should be unique across the slider page, got: {names:?}"
    );
}

/// `/primitives/slider/` の Examples 節に、`data-disabled` を伴う vertical
/// marker が存在する（`ex_slider_vertical_disabled` への marker 追加、
/// イシュー #2216）。
#[test]
fn primitives_slider_examples_contain_vertical_marker() {
    let html = primitives_slider_html();
    let examples = section_body(&html, "Examples");
    let vertical_markers: Vec<&str> = marker_tags(examples)
        .into_iter()
        .filter(|tag| tag.contains(r#"data-orientation="vertical""#))
        .collect();
    assert!(
        vertical_markers.len() >= 3,
        "primitives slider Examples should contain at least 3 vertical markers, got: {examples}"
    );
    for tag in &vertical_markers {
        assert!(
            tag.contains(r#"data-disabled="""#),
            "primitives vertical marker example is disabled, marker should carry data-disabled, got: {tag}"
        );
    }
}

/// 両ページとも 2 回生成して完全一致すること（SSG の決定性契約）。
#[test]
fn pages_render_deterministically() {
    assert_eq!(themes_slider_html(), themes_slider_html());
    assert_eq!(primitives_slider_html(), primitives_slider_html());
}

/// 既定エスケープ経路のみで組み立てているため、生の `<script` が出力へ
/// 混入しないことを軽量に確認する（`primitive_showcase_xss.rs` と同趣旨）。
#[test]
fn pages_do_not_contain_raw_script_tag() {
    assert!(!themes_slider_html().contains("<script"));
    assert!(!primitives_slider_html().contains("<script"));
}
