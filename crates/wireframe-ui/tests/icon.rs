//! `fandhe-frontend-wireframe-ui` SVG アイコン基盤の契約テスト（イシュー #2606）。
//!
//! 出力契約（§2.3）・線画契約（fill なし・stroke のみ・非インタラクティブ）・
//! `Node` スロット規約の実動・CSS 登録を固定する。

use std::collections::HashSet;

use fandhe_frontend_core::{el, render, text};
use fandhe_frontend_wireframe_ui::{icon, wireframe_css, Size, PARTS};

#[test]
fn all_registry_is_nonempty_unique_and_kebab() {
    assert!(icon::ALL.len() >= 12, "12 種以上のアイコンが必要");
    let mut seen = HashSet::new();
    for (name, _) in icon::ALL {
        assert!(seen.insert(*name), "アイコン名が重複している: {name}");
        assert!(
            !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "アイコン名は kebab-case でなければならない: {name}"
        );
    }
}

#[test]
fn every_icon_every_size_has_svg_root_contract() {
    for (name, ctor) in icon::ALL {
        for size in Size::ALL {
            let html = render(&ctor(size));
            assert!(
                html.starts_with("<svg"),
                "{name}/{size:?}: <svg 始まりでない"
            );
            assert!(
                html.ends_with("</svg>"),
                "{name}/{size:?}: </svg> 終わりでない"
            );
            let expected_class = format!(
                r#"class="fw-wire-icon-glyph fw-wire-size-{}""#,
                size.as_str()
            );
            assert!(
                html.contains(&expected_class),
                "{name}/{size:?}: class 不一致: {html}"
            );
            assert!(
                html.contains(&format!(r#"data-icon="{name}""#)),
                "{name}/{size:?}: data-icon 不一致"
            );
            assert!(
                html.contains(r#"viewBox="0 0 24 24""#),
                "{name}/{size:?}: viewBox 不一致"
            );
            assert!(
                html.contains(r#"width="1em""#),
                "{name}/{size:?}: width 不一致"
            );
            assert!(
                html.contains(r#"height="1em""#),
                "{name}/{size:?}: height 不一致"
            );
            assert!(
                html.contains(r#"fill="none""#),
                "{name}/{size:?}: fill 不一致"
            );
            assert!(
                html.contains(r#"stroke="currentColor""#),
                "{name}/{size:?}: stroke 不一致"
            );
            assert!(
                html.contains(r#"stroke-width="1.5""#),
                "{name}/{size:?}: stroke-width 不一致"
            );
            assert!(
                html.contains(r#"stroke-linecap="round""#),
                "{name}/{size:?}: stroke-linecap 不一致"
            );
            assert!(
                html.contains(r#"stroke-linejoin="round""#),
                "{name}/{size:?}: stroke-linejoin 不一致"
            );
            assert!(
                html.contains(r#"aria-hidden="true""#),
                "{name}/{size:?}: aria-hidden 不一致"
            );
            assert!(
                html.contains(r#"focusable="false""#),
                "{name}/{size:?}: focusable 不一致"
            );
        }
    }
}

#[test]
fn every_icon_is_stroke_only() {
    for (name, ctor) in icon::ALL {
        let html = render(&ctor(Size::Md));
        let fill_count = html.matches("fill=").count();
        assert_eq!(fill_count, 1, "{name}: fill= が 1 回のみでない: {html}");
        assert!(
            html.contains(r#"fill="none""#),
            "{name}: fill=\"none\" でない"
        );
        assert!(!html.contains("href"), "{name}: href を含む");
        assert!(!html.contains("<script"), "{name}: <script を含む");
        assert!(!html.contains("style="), "{name}: style= を含む");
        assert!(
            !html.contains(" on"),
            "{name}: イベントハンドラ属性らしき文字列を含む"
        );
        assert!(
            !html.contains("role="),
            "{name}: role= を含む（非対話契約違反）"
        );
        assert!(
            !html.contains("aria-label"),
            "{name}: aria-label を含む（非対話契約違反）"
        );
        assert!(
            !html.contains("tabindex"),
            "{name}: tabindex を含む（非対話契約違反）"
        );
    }
}

#[test]
fn every_icon_has_at_least_one_shape_child_and_no_foreign_tags() {
    const ALLOWED: [&str; 6] = [
        "<path",
        "<circle",
        "<line",
        "<polyline",
        "<polygon",
        "<rect",
    ];
    const FORBIDDEN: [&str; 4] = ["<use", "<image", "<a ", "<foreignObject"];
    for (name, ctor) in icon::ALL {
        let html = render(&ctor(Size::Md));
        let has_shape = ALLOWED.iter().any(|tag| html.contains(tag));
        assert!(has_shape, "{name}: 図形子要素が 1 つもない: {html}");
        for tag in FORBIDDEN {
            assert!(
                !html.contains(tag),
                "{name}: 許可されないタグ {tag} を含む: {html}"
            );
        }
    }
}

#[test]
fn render_is_deterministic() {
    for (name, ctor) in icon::ALL {
        let a = render(&ctor(Size::Lg));
        let b = render(&ctor(Size::Lg));
        assert_eq!(a, b, "{name}: 描画結果が非決定的");
    }
}

#[test]
fn size_default_is_md_class() {
    let html = render(&icon::plus(Size::default()));
    assert!(html.contains("fw-wire-size-md"));
}

#[test]
fn caret_directions_differ() {
    let up = render(&icon::caret_up(Size::Md));
    let down = render(&icon::caret_down(Size::Md));
    let left = render(&icon::caret_left(Size::Md));
    let right = render(&icon::caret_right(Size::Md));
    let all = [&up, &down, &left, &right];
    for i in 0..all.len() {
        for j in (i + 1)..all.len() {
            assert_ne!(all[i], all[j], "caret 方向 {i} と {j} の出力が同一");
        }
    }
}

#[test]
fn icon_glyph_css_is_registered_once_and_uses_font_size_var() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == icon::ICON_GLYPH_CSS)
        .count();
    assert_eq!(
        occurrences, 1,
        "ICON_GLYPH_CSS が PARTS にちょうど 1 回登録されていない"
    );

    let css = wireframe_css();
    assert!(
        css.contains(".fw-wire-icon-glyph {"),
        "wireframe_css() にセレクタが含まれない"
    );
    assert!(
        css.contains("var(--fw-wire-font-size"),
        "wireframe_css() が --fw-wire-font-size を参照していない"
    );
    assert!(
        css.contains("width: 1em"),
        "wireframe_css() に width: 1em が含まれない"
    );

    // 色リテラル不在の検査は ICON_GLYPH_CSS 単体に対して行う
    // （wireframe_css() 全体は :root トークンに #ffffff 等を含むため）。
    assert!(
        !icon::ICON_GLYPH_CSS.contains('#'),
        "ICON_GLYPH_CSS に # 色リテラルを含む"
    );
    assert!(
        !icon::ICON_GLYPH_CSS.contains("rgb("),
        "ICON_GLYPH_CSS に rgb( 色リテラルを含む"
    );
}

#[test]
fn node_slot_host_text_is_escaped_next_to_icon() {
    let node = el(
        "span",
        vec![],
        vec![
            icon::search(Size::Md),
            text("<b>x</b><script>alert(1)</script>"),
        ],
    );
    let html = render(&node);
    assert!(
        html.contains("&lt;script&gt;"),
        "テキストがエスケープされていない: {html}"
    );
    assert!(
        !html.contains("<script>"),
        "生の <script> が出力されている: {html}"
    );
    assert!(html.contains("<svg"), "アイコン SVG が失われている: {html}");
    assert!(
        html.contains(r#"data-icon="search""#),
        "アイコンが無傷でない: {html}"
    );
}

#[test]
fn slot_none_omits_icon() {
    fn labeled_row(leading: Option<fandhe_frontend_core::Node>, label: &str) -> String {
        let mut children: Vec<fandhe_frontend_core::Node> = Vec::new();
        if let Some(icon) = leading {
            children.push(icon);
        }
        children.push(text(label));
        render(&el("div", vec![("class", "fw-wire-row")], children))
    }

    let without_icon = labeled_row(None, "plain");
    assert!(
        !without_icon.contains("<svg"),
        "None のとき svg が出力されている: {without_icon}"
    );

    let with_icon = labeled_row(Some(icon::plus(Size::Sm)), "plain");
    assert!(
        with_icon.contains("<svg"),
        "Some のとき svg が出力されていない: {with_icon}"
    );
}
