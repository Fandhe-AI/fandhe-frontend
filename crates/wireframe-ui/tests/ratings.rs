//! `ratings` 部品の契約テスト（イシュー #2631）。
//!
//! `crates/wireframe-ui/tests/slider.rs`・`question.rs` と同型の観点
//! （root class・非対話制約・CSS 配線）に加え、`ratings` 固有の塗り個数の
//! クランプ（0〜255 → 0〜[`STAR_COUNT`]）と星の anatomy を固定する。
//!
//! `ratings` は `&str` 引数を持たない（塗り数は `u8`、サイズは `Size`）
//! ため、text 引数の XSS 回帰テストは構造的に充足される
//! （`crates/wireframe-ui/src/ratings.rs` モジュール doc・
//! `site/wireframes/ratings.md` の「原案差分メモ」節も参照）。label 引数を
//! 発明してテストを追加することはしない。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{ratings, wireframe_css, Size, PARTS, STAR_COUNT};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let html = render(&ratings(3, size));
        let expected_class = format!(r#"class="fw-wire-ratings {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn always_renders_star_count_stars() {
    for rating in [0, 1, 3, 5, 9, 255] {
        let html = render(&ratings(rating, Size::Md));
        assert_eq!(
            html.matches(r#"class="fw-wire-ratings-star""#).count(),
            STAR_COUNT as usize,
            "rating {rating}: expected {STAR_COUNT} star parts in {html:?}"
        );
        assert_eq!(
            html.matches(r#"data-icon="star""#).count(),
            STAR_COUNT as usize,
            "rating {rating}: expected {STAR_COUNT} star svgs in {html:?}"
        );
    }
}

#[test]
fn rating_fills_leading_stars_and_clamps() {
    let cases: [(u8, usize); 6] = [(0, 0), (1, 1), (4, 4), (5, 5), (6, 5), (255, 5)];
    for (input, expected_active) in cases {
        let html = render(&ratings(input, Size::Md));
        let active_count = html.matches(r#"data-active="""#).count();
        assert_eq!(
            active_count, expected_active,
            "input {input}: expected {expected_active} active stars in {html:?}"
        );

        // 塗られた星は先頭側に並ぶ: 最初の非 active 星より後ろに active 星がない。
        if let Some(first_star_span) = html.find(r#"class="fw-wire-ratings-star""#) {
            let stars_region = &html[first_star_span..];
            let mut seen_inactive = false;
            for star_html in stars_region
                .split(r#"class="fw-wire-ratings-star""#)
                .skip(1)
            {
                let is_active = star_html
                    .split("</span>")
                    .next()
                    .is_some_and(|segment| segment.contains(r#"data-active="""#));
                if !is_active {
                    seen_inactive = true;
                } else {
                    assert!(
                        !seen_inactive,
                        "input {input}: active star found after an inactive one in {html:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn full_render_matches_for_representative_value() {
    let html = render(&ratings(4, Size::Md));
    let star_svg_start = html
        .find("<svg")
        .expect("at least one star svg should be present");
    let star_svg_end = html[star_svg_start..]
        .find("</svg>")
        .map(|offset| star_svg_start + offset + "</svg>".len())
        .expect("star svg should be closed");
    let star_svg = &html[star_svg_start..star_svg_end];

    assert_eq!(
        html,
        format!(
            concat!(
                r#"<div class="fw-wire-ratings fw-wire-size-md">"#,
                r#"<span class="fw-wire-ratings-star" data-active="">{star}</span>"#,
                r#"<span class="fw-wire-ratings-star" data-active="">{star}</span>"#,
                r#"<span class="fw-wire-ratings-star" data-active="">{star}</span>"#,
                r#"<span class="fw-wire-ratings-star" data-active="">{star}</span>"#,
                r#"<span class="fw-wire-ratings-star">{star}</span>"#,
                "</div>",
            ),
            star = star_svg,
        )
    );
}

#[test]
fn output_has_no_interactive_semantics_or_value_data_attributes() {
    let html = render(&ratings(5, Size::Md));
    for forbidden in [
        "<input",
        "<button",
        "<a ",
        "href=",
        "javascript:",
        " tabindex=\"",
        " style=\"",
        " onclick=\"",
        " onload=\"",
        "data-rating",
        "data-value",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }

    // ルートの開始タグは `class` 以外の属性を持たない。
    let root_open_tag_end = html.find('>').expect("root tag should have a close");
    let root_open_tag = &html[..root_open_tag_end];
    assert!(root_open_tag.starts_with(r#"<div class="fw-wire-ratings"#));
    assert_eq!(root_open_tag.matches('=').count(), 1);

    // `aria-` は svg の `aria-hidden="true"` 以外に出現しない。
    for occurrence in html.match_indices("aria-") {
        let start = occurrence.0;
        assert!(
            html[start..].starts_with(r#"aria-hidden="true""#),
            "unexpected aria-* attribute in {html:?}"
        );
    }
}

// `ratings` は `&str` 引数を持たない（塗り数は `u8`、サイズは `Size`）ため、
// テキスト引数を通じた既定エスケープ（REQ-1）回帰テストは構造的に対象外と
// なる。動的文字列が HTML へ流れ込む経路が存在しないため、label 等の引数を
// 発明してテストを追加することはしない
// （`crates/wireframe-ui/src/ratings.rs` モジュール doc参照）。

#[test]
fn ratings_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::ratings::RATINGS_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::ratings::RATINGS_CSS));
}

#[test]
fn ratings_css_declares_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::ratings::RATINGS_CSS;
    for selector in [
        ".fw-wire-ratings {",
        ".fw-wire-ratings-star {",
        ".fw-wire-ratings-star[data-active] {",
        ".fw-wire-ratings-star[data-active] .fw-wire-icon-glyph {",
    ] {
        assert!(css.contains(selector), "missing selector {selector:?}");
    }

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
    assert!(!css.contains(" fd-"));
}
