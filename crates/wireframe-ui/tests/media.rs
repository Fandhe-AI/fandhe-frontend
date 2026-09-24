//! `media` 部品の契約テスト（イシュー #2661）。
//!
//! `crates/wireframe-ui/tests/avatar.rs` と同型の観点（非対話制約・XSS
//! 回帰・CSS 配線）を単体固定する。

use fandhe_frontend_core::{el_owned, render, text};
use fandhe_frontend_wireframe_ui::{icon, media, wireframe_css, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = media(None, size);
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-media {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn none_content_falls_back_to_play_glyph() {
    let html = render(&media(None, Size::Md));
    assert_eq!(html.matches(r#"data-icon="play""#).count(), 1);
    assert!(html.contains("<svg"));
}

#[test]
fn some_content_replaces_default_glyph() {
    let with_image = render(&media(Some(icon::image(Size::Md)), Size::Md));
    assert!(with_image.contains(r#"data-icon="image""#));
    assert!(!with_image.contains(r#"data-icon="play""#));

    let with_text = render(&media(Some(text("REC")), Size::Md));
    assert!(!with_text.contains("<svg"));
    assert!(with_text.contains("REC"));
}

#[test]
fn disc_part_class_appears_exactly_once() {
    let html = render(&media(None, Size::Md));
    assert_eq!(html.matches("fw-wire-media-disc").count(), 1);
}

#[test]
fn xss_regression_slot_text_is_escaped() {
    let payload_a = "<script>alert(1)</script>";
    let payload_b = "\"><img src=x onerror=alert(1)>";
    let html_a = render(&media(Some(text(payload_a)), Size::Md));
    let html_b = render(&media(Some(text(payload_b)), Size::Md));

    assert!(!html_a.contains(payload_a));
    assert!(html_a.contains("&lt;script&gt;"));
    assert!(!html_b.contains(payload_b));
    assert!(html_b.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_style_or_media_elements() {
    let html = render(&media(Some(text("REC")), Size::Md));
    for forbidden in [
        " role=\"",
        " aria-expanded",
        " tabindex=\"",
        " style=\"",
        "href=",
        "src=",
        "poster=",
        "controls",
        "<video",
        "<iframe",
        "<source",
        "<img",
        "<a ",
        "<button",
        " onclick=\"",
        " onerror=\"",
        "javascript:",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }

    // 既定（None）フォールバックの `icon::play` は装飾用途の
    // `aria-hidden` を持つ（`crate::icon` の出力契約）。対話的 ARIA では
    // ないため許容し、上の判定対象からは区別する。
    let default_html = render(&media(None, Size::Md));
    assert!(default_html.contains(r#"aria-hidden="true""#));
    assert!(!default_html.contains(" aria-expanded"));
}

/// `content` スロットは §11.4 の `Node` スロット規約どおり任意の `Node`
/// を受け取り、本部品はその中身をサニタイズしない（`avatar`/`link`/`tag`
/// 等の他スロット部品と同じ、rustdoc に明記した既知の限界）。ただし
/// `on*` イベントハンドラ属性だけは content の出所によらず core の
/// `render()` が構造的に出力を拒否する（不変条件 9）ことを、部品側の
/// 追加対策なしに固定する回帰テスト（codex-review PR #2718 指摘対応）。
#[test]
fn content_slot_event_handler_attributes_are_stripped_by_core_structurally() {
    let malicious = el_owned(
        "div",
        vec![
            ("onclick".to_string(), "alert(1)".to_string()),
            ("onerror".to_string(), "alert(1)".to_string()),
            ("data-safe".to_string(), "kept".to_string()),
        ],
        vec![text("caller-supplied")],
    );
    let html = render(&media(Some(malicious), Size::Md));

    assert!(!html.contains("onclick"));
    assert!(!html.contains("onerror"));
    // イベントハンドラ以外の通常属性は影響を受けず出力される
    // （on* だけを対象にした判定であることの対照確認）。
    assert!(html.contains(r#"data-safe="kept""#));
    assert!(html.contains("caller-supplied"));
}

/// `content` スロットへ呼び出し側が対話的なタグ（`<button>` 等）を
/// 直接構築して渡した場合、core はタグ名の語彙を制限しないため、その
/// タグ名はそのまま出力される。これは型では防げない既知の限界であり、
/// `avatar`/`link`/`tag` 等の他スロット部品と共通（rustdoc「content
/// スロットに渡した Node の内容は呼び出し側の責務」節参照）。本テストは
/// その契約を裏付ける記録であり、将来の意図しない挙動変化を検知する。
#[test]
fn content_slot_interactive_tag_names_are_the_caller_responsibility_by_design() {
    let caller_supplied_button = el_owned("button", vec![], vec![text("play")]);
    let html = render(&media(Some(caller_supplied_button), Size::Md));
    assert!(html.contains("<button>play</button>"));
}

#[test]
fn media_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::media::MEDIA_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::media::MEDIA_CSS));
}

#[test]
fn media_css_selectors_use_fw_wire_prefix_and_reference_tokens_not_literals() {
    let css = fandhe_frontend_wireframe_ui::media::MEDIA_CSS;
    for line in css.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('.') {
            assert!(
                trimmed.starts_with(".fw-wire-"),
                "selector line should start with .fw-wire-: {line:?}"
            );
        }
    }

    assert!(css.contains("var(--fw-wire-control-size"));
    assert!(css.contains("aspect-ratio"));
    assert!(!css.contains("--fandhe-"));
    assert!(!css.contains("@keyframes"));
    assert!(!css.contains("animation"));

    // Size の段階値（例: 2rem 以外の rem リテラル）を直書きしていない
    // ことの弱い保証として、`size::SCALE` の control_size 値を文字列
    // として含まないことを確認する。
    for control_size in fandhe_frontend_wireframe_ui::size::css()
        .lines()
        .filter_map(|line| {
            let marker = "--fw-wire-control-size: ";
            let start = line.find(marker)? + marker.len();
            let end = line[start..].find(';')? + start;
            Some(line[start..end].to_string())
        })
    {
        // 唯一の許容参照は `var(--fw-wire-control-size, 2rem)` の
        // フォールバック値であり、これは Md 段階の control_size と
        // 一致する仕様上の例外（CSS カスタムプロパティ未定義時の
        // フォールバック値）。他段階の値が直書きされていないことを
        // 確認する。
        if control_size != "2rem" {
            assert!(
                !css.contains(&control_size),
                "unexpected literal control_size {control_size:?} in MEDIA_CSS"
            );
        }
    }
}

/// codex-review PR #2718 P1 指摘対応の回帰テスト:
/// `.fw-wire-media .fw-wire-icon-glyph`（および disc パート）は
/// `--fw-wire-control-size` を直接参照せず、ルート `.fw-wire-media` 上で
/// 一度だけ確定させた `--fw-wire-media-control-size` のみを参照する。
///
/// `content` スロットに渡した `Node`（`icon::image` 等）は自身の
/// `fw-wire-size-<段階>` class により、そのノード自身の要素上で
/// `--fw-wire-control-size` を再定義する。もしグリフ側が
/// `--fw-wire-control-size` を直接読んでいると、CSS カスタムプロパティの
/// 解決規則（同一要素上の宣言が祖先からの継承値より優先される）により、
/// `content` 側の `Size` がルートの `size` 引数を上書きしてしまう
/// （rustdoc「size は中央のディスク・グリフの大きさに効く」契約違反）。
/// 本テストは文字列レベルでその配線を固定する（wireframe-ui にはブラウザ
/// 計算スタイルを検証するハーネスがないため、CSS ルールブロックの
/// テキスト内容を直接検査する）。
#[test]
fn glyph_and_disc_size_are_independent_of_content_slots_own_size_class() {
    let css = fandhe_frontend_wireframe_ui::media::MEDIA_CSS;

    // ルートブロックが専用変数を一度だけ確定させる。
    let root_block_start = css.find(".fw-wire-media {").expect("root block");
    let root_block_end = css[root_block_start..]
        .find('}')
        .map(|i| root_block_start + i)
        .expect("root block end");
    let root_block = &css[root_block_start..root_block_end];
    assert!(
        root_block.contains("--fw-wire-media-control-size: var(--fw-wire-control-size, 2rem)"),
        "root block should declare --fw-wire-media-control-size from --fw-wire-control-size: {root_block:?}"
    );

    // ディスク・グリフの両パートは専用変数のみを参照し、
    // `--fw-wire-control-size` を直接参照しない
    // （直接参照が残っていると content スロットの Size 漏れが再発する）。
    for part_selector in [".fw-wire-media-disc", ".fw-wire-icon-glyph"] {
        let block_header = format!(".fw-wire-media {part_selector} {{");
        let block_start = css
            .find(&block_header)
            .unwrap_or_else(|| panic!("expected block for {part_selector} in {css:?}"));
        let block_end = css[block_start..]
            .find('}')
            .map(|i| block_start + i)
            .unwrap_or_else(|| panic!("expected closing brace for {part_selector}"));
        let block = &css[block_start..block_end];

        assert!(
            block.contains("var(--fw-wire-media-control-size)"),
            "{part_selector} block should reference --fw-wire-media-control-size: {block:?}"
        );
        assert!(
            !block.contains("--fw-wire-control-size"),
            "{part_selector} block should not reference --fw-wire-control-size directly \
             (it would let a content slot's own Size class override the root size): {block:?}"
        );
    }
}

/// `media(Some(icon::image(Size::Xs)), Size::Xl)` のようにルートと
/// content スロットで異なる Size を渡しても、マークアップ上は両者の
/// class がそれぞれの要素にそのまま出力される（ルートは size 引数、
/// content は呼び出し側が渡した Node のまま）ことを固定する。CSS 側の
/// 実際の計算値の独立性は
/// `glyph_and_disc_size_are_independent_of_content_slots_own_size_class`
/// が CSS 配線として固定する。
#[test]
fn root_and_content_slot_size_classes_are_independent_in_markup() {
    let html = render(&media(Some(icon::image(Size::Xs)), Size::Xl));
    assert!(html.contains(r#"class="fw-wire-media fw-wire-size-xl""#));
    assert!(html.contains(r#"class="fw-wire-icon-glyph fw-wire-size-xs""#));
}
