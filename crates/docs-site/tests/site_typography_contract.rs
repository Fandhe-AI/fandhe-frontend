//! 本文タイポグラフィ（`.docs-content` 配下、`crate::site_theme::typography_css`
//! が組み立てる）と、そのミラー元である `fandhe_frontend_pre_styled_ui` の
//! 各部品 recipe（`css()`/`stylesheet()` が返す静的 CSS）との乖離を検知する
//! 回帰テスト（イシュー #911）。
//!
//! # 検証方針
//!
//! `crates/docs-site/src/site_theme.rs` の `typography_css` はミラー元部品を
//! **呼び出さず**、`fandhe_frontend_pre_styled_ui::css::{decl, serialize_rule}`
//! で `.docs-content <tag>` セレクタの規則を独自に組み立てる（`docs/design/
//! docs-site-three-column-redesign.md` §3.6: `markdown.rs` の出力・class
//! 付与ロジックを変更しないための設計）。このため「ミラー宣言の値がコピー
//! ミスで部品側の recipe と食い違う」「部品側の recipe が変わったのに docs
//! 側の追随が漏れる」事故はコンパイラでは検知できない。本テストは各ミラー
//! 宣言（`(property, value)` の組）が対応する部品の `css()`/`stylesheet()`
//! 出力の**部分文字列として存在する**ことを機械的に検証し、ドリフトを
//! fail-closed で検知する（手動同期に頼らない）。
//!
//! 宣言の出力書式は [`fandhe_frontend_pre_styled_ui::css::serialize_rule`]
//! の凍結書式（`  <property>: <value>;\n`）に揃えているため、
//! `"  {property}: {value};"` を部分文字列として照合すれば足りる。

use fandhe_frontend_docs_site::site_theme;

/// サイト骨格 CSS 全量（typography_css を含む）を取得する。
fn site_css() -> String {
    site_theme::stylesheet()
        .expect("site theme stylesheet should assemble")
        .as_css()
        .to_string()
}

/// `"  {property}: {value};"` の形で `haystack` に含まれるかを検証する
/// （[`fandhe_frontend_pre_styled_ui::css::serialize_rule`] の凍結書式）。
fn assert_declaration_mirrored(haystack: &str, property: &str, value: &str, context: &str) {
    let needle = format!("  {property}: {value};");
    assert!(
        haystack.contains(&needle),
        "{context}: docs-content 側に mirror 宣言が見つからない: {needle}"
    );
}

#[test]
fn heading_size_variants_mirror_pre_styled_ui_heading_recipe() {
    let docs_css = site_css();
    let heading_css = fandhe_frontend_pre_styled_ui::heading::css();

    // base 宣言（全サイズ variant 共通）。
    for (property, value) in [
        ("font-weight", "var(--fandhe-font-font-weight-semibold)"),
        ("letter-spacing", "-0.01em"),
    ] {
        assert_declaration_mirrored(&heading_css, property, value, "heading base (component)");
        assert_declaration_mirrored(&docs_css, property, value, "heading base (docs mirror)");
    }

    // タグ → サイズ variant 対応（§3.2）。
    for (docs_size_token, component_size_token, line_height) in [
        ("3xl", "3xl", "1.2"),
        ("2xl", "2xl", "1.25"),
        ("xl", "xl", "1.3"),
        ("lg", "lg", "1.3"),
        ("md", "md", "1.3"),
        ("sm", "sm", "1.25"),
    ] {
        let font_size_value = format!("var(--fandhe-font-font-size-{component_size_token})");
        assert_declaration_mirrored(
            &heading_css,
            "font-size",
            &font_size_value,
            "heading size variant (component)",
        );
        assert_declaration_mirrored(
            &docs_css,
            "font-size",
            &font_size_value,
            &format!("heading size {docs_size_token} (docs mirror)"),
        );
        // line-height の値は複数サイズ間で重複する（h3/h4/h5 の 1.3、h2/h6 の
        // 1.25 等）ため、部分文字列一致では「特定サイズの line-height だけが
        // ドリフトした」事故までは検知できない（component/docs 双方に存在
        // することの確認に留まる）。font-size トークン名は各サイズで一意の
        // ため、サイズ変数と組の識別はそちらが担保する。
        assert_declaration_mirrored(
            &heading_css,
            "line-height",
            line_height,
            "heading size variant (component)",
        );
        assert_declaration_mirrored(
            &docs_css,
            "line-height",
            line_height,
            &format!("heading size {docs_size_token} (docs mirror)"),
        );
    }
}

#[test]
fn paragraph_size_mirrors_pre_styled_ui_text_recipe() {
    let docs_css = site_css();
    let text_css = fandhe_frontend_pre_styled_ui::text::css();

    for (property, value) in [
        ("font-size", "var(--fandhe-font-font-size-md)"),
        ("line-height", "1.5"),
    ] {
        assert_declaration_mirrored(&text_css, property, value, "text TextSize::Md (component)");
        assert_declaration_mirrored(&docs_css, property, value, "paragraph (docs mirror)");
    }
}

#[test]
fn list_declarations_mirror_pre_styled_ui_list_recipe() {
    let docs_css = site_css();
    let list_css = fandhe_frontend_pre_styled_ui::list::css();

    // root（ul/ol）: ListVariant::Marker.
    for (property, value) in [
        ("list-style", "revert"),
        ("padding-inline-start", "var(--fandhe-space-6)"),
    ] {
        assert_declaration_mirrored(
            &list_css,
            property,
            value,
            "list Marker variant (component)",
        );
        assert_declaration_mirrored(&docs_css, property, value, "ul/ol (docs mirror)");
    }

    // item（li）base.
    for (property, value) in [
        ("margin-block", "var(--fandhe-space-1)"),
        ("line-height", "1.5"),
    ] {
        assert_declaration_mirrored(&list_css, property, value, "list item base (component)");
        assert_declaration_mirrored(&docs_css, property, value, "li (docs mirror)");
    }

    // item の ::marker（イシュー #1438、fg.muted 化）.
    let (property, value) = ("color", "var(--fandhe-color-fg-muted)");
    assert_declaration_mirrored(&list_css, property, value, "list item ::marker (component)");
    assert_declaration_mirrored(&docs_css, property, value, "li::marker (docs mirror)");
}

#[test]
fn link_declarations_mirror_pre_styled_ui_link_recipe() {
    let docs_css = site_css();
    let link_css = fandhe_frontend_pre_styled_ui::link::stylesheet();

    // text-decoration/cursor は component 側・docs 側で同一の宣言値。
    for (property, value) in [
        (
            "text-decoration",
            "var(--fandhe-link-text-decoration, none)",
        ),
        ("cursor", "pointer"),
    ] {
        assert_declaration_mirrored(&link_css, property, value, "link base (component)");
        assert_declaration_mirrored(&docs_css, property, value, "a (docs mirror)");
    }

    // イシュー #1437: base `color` は component 側で
    // `var(--fandhe-palette, var(--fandhe-color-accent))` へ変更されたが、
    // docs 文脈では palette 軸を公開しないため常に accent 固定で解決する
    // （blockquote #1431 の先例と同じ単純化）。
    assert_declaration_mirrored(
        &link_css,
        "color",
        "var(--fandhe-palette, var(--fandhe-color-accent))",
        "link base (component)",
    );
    assert_declaration_mirrored(
        &docs_css,
        "color",
        "var(--fandhe-color-accent, var(--fandhe-color-fg))",
        "a (docs mirror, palette resolved)",
    );

    // hover 時の文字色強調（component 側は palette-emphasized、docs 側は
    // accent-emphasized へ固定解決）。
    assert_declaration_mirrored(
        &link_css,
        "color",
        "var(--fandhe-palette-emphasized, var(--fandhe-color-accent-emphasized))",
        "link hover (component)",
    );
    assert_declaration_mirrored(
        &docs_css,
        "color",
        "var(--fandhe-color-accent-emphasized, var(--fandhe-color-fg))",
        "a:hover (docs mirror, palette resolved)",
    );
}

#[test]
fn code_declarations_mirror_pre_styled_ui_code_recipe() {
    let docs_css = site_css();
    let code_css = fandhe_frontend_pre_styled_ui::code::css();

    // base（variant/size/palette 非依存）宣言のミラー確認。
    for (property, value) in [
        (
            "font-family",
            "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
        ),
        ("border-radius", "var(--fandhe-radius-sm)"),
    ] {
        assert_declaration_mirrored(&code_css, property, value, "code base (component)");
        assert_declaration_mirrored(&docs_css, property, value, "inline code (docs mirror)");
    }

    // 既定 variant/size（Subtle・Md）の宣言はコンポーネント側では
    // `.fd-code--variant-subtle`/`.fd-code--size-md` クラスへ載るが、docs
    // ミラーはコンポーネントを介さないため具体値のみを比較する
    // （`.docs-content code` 側は palette 非配線で `--fandhe-color-neutral-*`
    // へ直接接続する、[`crate`] 冒頭 rustdoc 参照）。
    assert!(
        code_css.contains("padding: 0.125rem 0.5rem;"),
        "code component missing Md padding: {code_css}"
    );
    assert_declaration_mirrored(
        &docs_css,
        "padding",
        "0.125rem 0.5rem",
        "inline code (docs mirror)",
    );
    assert!(
        code_css.contains("font-size: var(--fandhe-font-font-size-sm);"),
        "code component missing Md font-size: {code_css}"
    );
    assert_declaration_mirrored(
        &docs_css,
        "font-size",
        "var(--fandhe-font-font-size-sm)",
        "inline code (docs mirror)",
    );
    assert_declaration_mirrored(
        &docs_css,
        "background",
        "var(--fandhe-color-neutral-subtle)",
        "inline code (docs mirror)",
    );
    assert_declaration_mirrored(
        &docs_css,
        "color",
        "var(--fandhe-color-neutral-fg-subtle)",
        "inline code (docs mirror)",
    );
}

#[test]
fn em_declarations_mirror_pre_styled_ui_em_recipe() {
    let docs_css = site_css();
    let em_css = fandhe_frontend_pre_styled_ui::em::css();

    // イシュー #1433: font-weight の上書きは参考サイト（chakra-ui /
    // Radix Themes）のいずれにも存在しないため廃止済み。italic のみを
    // ミラー対象とする。
    let (property, value) = ("font-style", "italic");
    assert_declaration_mirrored(&em_css, property, value, "em base (component)");
    assert_declaration_mirrored(&docs_css, property, value, "em (docs mirror)");
    assert!(
        !em_css.contains("font-weight"),
        "em base (component) が font-weight を宣言してはならない"
    );
    // `.docs-content em` の規則ブロック自体（`serialize_rule` の凍結書式）
    // が italic のみであることをブロック単位で固定する（他部品ミラーの
    // font-weight-medium 宣言との誤検知を避けるため、部分文字列一致では
    // なく規則全文で照合する）。
    assert!(
        docs_css.contains(
            ".docs-content em {
  font-style: italic;
}
"
        ),
        "em (docs mirror) の規則ブロックが italic のみで構成されていない"
    );
}

#[test]
fn blockquote_declarations_mirror_pre_styled_ui_blockquote_subtle_variant() {
    let docs_css = site_css();
    let blockquote_css = fandhe_frontend_pre_styled_ui::blockquote::css();

    // root base（margin は docs 固有に上書きするため対象外）。
    for (property, value) in [
        ("padding-inline-start", "1rem"),
        ("padding-block", "0.5rem"),
    ] {
        assert_declaration_mirrored(
            &blockquote_css,
            property,
            value,
            "blockquote root base (component)",
        );
        assert_declaration_mirrored(&docs_css, property, value, "blockquote (docs mirror)");
    }

    // Subtle variant（イシュー #1431 で背景・角丸を廃し、
    // `--fandhe-palette-muted` の左罫線のみへ変更した。`--fandhe-palette-muted`
    // は docs 側で常に accent-muted へ解決するため、component 側の宣言
    // そのものではなく解決後の値を照合する）。
    assert_declaration_mirrored(
        &blockquote_css,
        "border-inline-start",
        "4px solid var(--fandhe-palette-muted)",
        "blockquote Subtle variant (component)",
    );
    assert_declaration_mirrored(
        &docs_css,
        "border-inline-start",
        "4px solid var(--fandhe-color-accent-muted)",
        "blockquote (docs mirror, palette resolved)",
    );
    let blockquote_rule_start = docs_css
        .find(".docs-content blockquote {")
        .expect(".docs-content blockquote 規則が生成 CSS に存在する");
    let blockquote_rule = docs_css[blockquote_rule_start..]
        .split('}')
        .next()
        .expect(".docs-content blockquote 規則が閉じている");
    assert!(
        !blockquote_rule.contains("background"),
        "blockquote (docs mirror) は #1431 是正後、背景の面を持たない"
    );
    assert!(
        !blockquote_rule.contains("border-radius"),
        "blockquote (docs mirror) は #1431 是正後、角丸を持たない"
    );
}

#[test]
fn typography_selectors_never_escape_default_encoding() {
    // 受け入れ条件 2（既定エスケープ経路の迂回を追加していない）の CSS 側の
    // 裏付け: 生成 CSS は StyleSheet::push_css の `<` 拒否検証を経由済み
    // （`site_theme::stylesheet` が Result で失敗しうる設計自体がこの契約を
    // 表す）。ここでは生成物に `<` が含まれないことを typography 由来の
    // セレクタ・値についても直接固定する。
    let docs_css = site_css();
    assert!(!docs_css.contains('<'));
}
