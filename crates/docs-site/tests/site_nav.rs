//! `site/nav.toml`（実マニフェスト）と既存 docs 資産のドリフト検知テスト。
//!
//! イシュー #473 の受け入れ条件（登録ページの網羅・サブセット外構文の
//! 残存によるレンダリング崩れ無し・内容不変更）のうち、機械的に検証できる
//! 部分（パース成功・ページ登録数と path・source 実在・ブロックレベルの
//! レンダリング健全性）を本テストで担保する。ジェネレータ本体（`main.rs`、
//! イシュー #470）は本テスト実行時点で fail-closed スタブのままのため、
//! `dist/` を実際に生成しての目視確認は #476（受入検証）へ引き継ぐ。
//!
//! # ページ数について（イシュー本文との数値差異）
//!
//! イシュー #473 本文は受け入れ条件を「全 11 ページ（トップ + quickstart +
//! guides 4 + api 6）」と記載するが、`1 + 1 + 4 + 6 = 12` であり列挙内容と
//! 総数表記が矛盾している（イシュー本文側の算術上の誤記と判断）。本テストは
//! 列挙されたページ集合（トップ 1 + quickstart 1 + guides 4 + api 6 =
//! 実質 12 ページ）を正としてすべて登録する。
//!
//! イシュー #504（examples 導線）で Examples セクション（概説ページ 1 +
//! サンプル README 4 = 5 ページ）が追加され、登録ページ総数は 17 ページ・
//! セクション数は 4（Getting Started / Guides / Examples / API Reference）
//! となった。イシュー #548 で API Reference セクションへ
//! `docs/api/pre-styled-recipe-api.md` が追加され、登録ページ総数は
//! 18 ページとなった。イシュー #552 で Examples セクションへ
//! `examples/headless-pre-styled-ui/README.md` が、API Reference セクションへ
//! `docs/api/headless-ui-api.md` / `docs/api/pre-styled-ui-api.md` が追加され、
//! 登録ページ総数は 21 ページとなった。
//!
//! `site/index.md`（イシュー #472）は本テスト実行時点で未マージのことが
//! あるため、存在すれば厳格検証（他ページと同様に `render_markdown` の
//! 健全性を確認）し、存在しなければ「他 11 ページの検証」のみに縮退する。
//! #472 マージ後は自動的に全ページの厳格検証になる。

use std::path::{Path, PathBuf};

use fandhe_frontend_core::Node;
use fandhe_frontend_docs_site::markdown::render_markdown;
use fandhe_frontend_docs_site::nav::{parse_nav, validate_sources, Nav};

/// `CARGO_MANIFEST_DIR`（`crates/docs-site`）から repo_root を解決する。
/// テストフィクスチャがクレート内に閉じず repo_root 配下の実ファイルを
/// 参照するため、`crates/docs-site` の 2 階層上を repo_root とみなす
/// （`Cargo.toml` の `members = ["crates/*"]` 構成に対応）。
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo_root should resolve from CARGO_MANIFEST_DIR")
}

fn load_nav() -> Nav {
    let path = repo_root().join("site/nav.toml");
    let input = std::fs::read_to_string(&path).expect("site/nav.toml should be readable");
    parse_nav(&input).expect("site/nav.toml should conform to the fail-closed TOML subset")
}

#[test]
fn site_nav_parses_successfully() {
    let nav = load_nav();
    assert_eq!(nav.site.title, "fandhe-frontend");
    assert_eq!(nav.site.base_path, "/fandhe-frontend");
}

/// イシュー #1021: Primitives（`fandhe-frontend-headless-ui`）セクションが
/// Examples の後・Themes の前へ加わり、セクション数は 5 → 6 になった
/// （ヘッダー上の並びは設計 §2「Primitives → Themes」）。
#[test]
fn site_nav_registers_seven_sections_with_expected_titles() {
    let nav = load_nav();
    let titles: Vec<&str> = nav.sections.iter().map(|s| s.title.as_str()).collect();
    // イシュー #2088: Blocks セクション（shadcn/ui Blocks 相当の既存部品
    // 合成例）が Themes の直後・API Reference の直前へ加わり、
    // セクション数は 6 → 7 になった。
    assert_eq!(
        titles,
        vec![
            "Getting Started",
            "Guides",
            "Examples",
            "Primitives",
            "Themes",
            "Blocks",
            "API Reference"
        ]
    );
}

/// イシュー #1010: `[[section]].index_path` が全セクションに宣言され、
/// 各値が期待どおりであることを固定する（`nav.rs` のパース時点で
/// `index_path` は当該セクション配下ページの `path` と完全一致すること
/// 自体は既に保証されているため、本テストは「値のドリフト」検知に限定する）。
///
/// イシュー #1018 でセクション名を「Components」から「Themes」へ改称し、
/// `index_path` を `/components/pre-styled-ui/` から `/themes/` へ移行した
/// （Primitives = `fandhe-frontend-headless-ui` との対称構成、
/// docs/design/docs-site-primitives-themes-split.md §3 参照）。旧 URL は
/// `site/redirects.toml` で互換維持する。イシュー #1021 で Primitives
/// セクション自体（`/primitives/`）が新設され、6 セクションになった。
#[test]
fn site_nav_declares_index_path_for_every_section() {
    let nav = load_nav();
    let actual: Vec<(&str, &str)> = nav
        .sections
        .iter()
        .map(|s| (s.title.as_str(), s.index_path.as_str()))
        .collect();
    assert_eq!(
        actual,
        vec![
            ("Getting Started", "/"),
            ("Guides", "/guides/"),
            ("Examples", "/examples/"),
            ("Primitives", "/primitives/"),
            ("Themes", "/themes/"),
            ("Blocks", "/blocks/"),
            ("API Reference", "/api/"),
        ]
    );
}

/// 受け入れ条件 1: 既存の利用者向けドキュメント（トップ + quickstart +
/// guides 4 本 + api 6 本 = 12 ページ）と、イシュー #504 で追加された
/// Examples セクション（概説ページ 1 + サンプル README 4 = 5 ページ）、
/// イシュー #548 で追加された `docs/api/pre-styled-recipe-api.md`（1 ページ）、
/// イシュー #552 で追加された `examples/headless-pre-styled-ui/README.md`・
/// `docs/api/headless-ui-api.md`・`docs/api/pre-styled-ui-api.md`（3 ページ）、
/// pre-styled-ui ショーケース統合で追加された
/// `site/components-pre-styled-ui.md`（1 ページ、索引ページ）が
/// サイト生成対象として登録されている（イシュー本文の「全 11 ページ」表記と
/// 列挙内容の数値差異はモジュール冒頭のコメント参照）。
///
/// イシュー #943 で `site/components/*.md` 99 部品ページが 6 カテゴリの
/// `[[section.group]]` 配下へ登録され、登録ページ総数は 22 + 99 = 121 と
/// なった（台帳との三方突合・充足率計測は #944 の責務、本テストは既存
/// 22 件の宣言内容の不変を厳格検証しつつ、部品ページ側は件数・代表エントリの
/// spot-check に留める）。イシュー #991 で Toolbar（`site/components/toolbar.md`）
/// が加わり、部品ページは 99 → 100、登録ページ総数は 121 → 122 になった。
/// イシュー #992 で Menubar（`site/components/menubar.md`）が加わり、
/// 部品ページは 100 → 101、登録ページ総数は 122 → 123 になった。
/// イシュー #993 で Navigation Menu（`site/components/navigation-menu.md`）
/// が加わり、部品ページは 101 → 102、登録ページ総数は 123 → 124 になった。
/// イシュー #994 で Callout（`site/components/callout.md`）が加わり、
/// 部品ページは 102 → 103、登録ページ総数は 124 → 125 になった。イシュー
/// #995 で Quote / Strong の 2 ページが加わり、部品ページは 103 → 105、
/// 登録ページ総数は 125 → 127 になった。イシュー #996 で Tab Nav
/// （`site/components/tab-nav.md`）が加わり、部品ページは 105 → 106、
/// 登録ページ総数は 127 → 128 になった。イシュー #997 で Checkbox Group
/// （`site/components/checkbox-group.md`）が加わり、部品ページは
/// 106 → 107、登録ページ総数は 128 → 129 になった。イシュー #1009 で
/// Guides / API Reference のセクショントップページ（`site/guides.md` /
/// `site/api.md`）2 ページが加わり、登録ページ総数は 129 → 131 になった
/// （部品ページ 107 は不変）。イシュー #1017 で既存 107 部品ページの
/// `source`/`path` を `site/components/<kebab>.md` / `/components/<kebab>/`
/// から `site/themes/<kebab>.md` / `/themes/<kebab>/` へ移行した
/// （登録ページ総数 131 は不変。`/components/` 配下は索引ページ 1 件のみ
/// 残る）。イシュー #1018 で索引ページ自体を
/// `site/components-pre-styled-ui.md` / `/components/pre-styled-ui/` から
/// `site/themes.md` / `/themes/` へ移設した（登録ページ総数 131 は不変。
/// `/components/` 配下の本体ページは 0 件になり、`/themes/` 配下は
/// 部品 107 + 索引 1 = 108 件になる）。イシュー #1683 で Collapsible
/// （`site/themes/collapsible.md`）が加わり、部品ページは 107 → 108、
/// 登録ページ総数は 197 → 198 になった。イシュー #1685 で Field
/// （`site/themes/field.md`）が加わり、部品ページは 108 → 109、
/// 登録ページ総数は 198 → 199 になった。イシュー #1687 で Fieldset
/// （`site/themes/fieldset.md`）が加わり、部品ページは 109 → 110、
/// 登録ページ総数は 199 → 200 になった。イシュー #2059 で Button Group・
/// イシュー #2062 で Input Group（いずれも Primitives）がそれぞれ加わり、
/// 登録ページ総数は 200 → 202 になった。
#[test]
fn site_nav_registers_all_pages_with_expected_paths() {
    let nav = load_nav();
    // `nav.all_pages()`（イシュー #939 の唯一の正規走査経路）で数える。
    let pages: Vec<(&str, &str)> = nav
        .all_pages()
        .map(|p| (p.source.as_str(), p.path.as_str()))
        .collect();

    // イシュー #995 で Quote / Strong の 2 ページが加わり 124 → 126 になった。
    // イシュー #996 で Tab Nav が加わり 127 → 128、イシュー #997 で
    // Checkbox Group が加わり 128 → 129 になった。イシュー #1009 で
    // Guides / API Reference のセクショントップページ 2 ページが加わり
    // 129 → 131 になった。イシュー #1021 で Primitives セクション（索引 1 +
    // 部品 63 = 64 ページ）が新設され、131 → 195 になった。イシュー #1118 で
    // Guides セクションへ「JS ゼロ SSG での利用ガイド」が加わり 195 → 196 に
    // なった。イシュー #1156 で API Reference セクションへ
    // `docs/api/server-api.md`（`generate_assets` 等 SSG API リファレンス）
    // が加わり 196 → 197 になった。イシュー #1683 で Collapsible が加わり
    // 197 → 198 になった。イシュー #1685 で Field が加わり 198 → 199 に
    // なった。イシュー #1687 で Fieldset が加わり 199 → 200 になった。
    // イシュー #2059 で Button Group（Primitives）・イシュー #2062 で
    // Input Group（Primitives）がそれぞれ加わり 200 → 202 になった。
    // イシュー #2063 で Input Group の Themes ページが加わり 202 → 203
    // になった。イシュー #2065 で Item（Primitives）が加わり 203 → 204
    // になった。イシュー #2066 で Item の Themes ページが加わり 204 → 205
    // になった。イシュー #2068 で Command（Primitives）が加わり 205 → 206
    // になった。イシュー #2060 で Button Group の Themes ページが加わり
    // 206 → 207、イシュー #2070 で Command の Themes ページが加わり
    // 207 → 208、イシュー #2072 で Sidebar（Primitives）が加わり
    // 208 → 209、イシュー #2105 で Message（Primitives）が加わり
    // 209 → 210、イシュー #2080 で Radial Chart の Themes ページが加わり
    // 210 → 211、イシュー #2106 で Message の Themes ページが加わり
    // 211 → 212、イシュー #2108 で Bubble（Primitives）が加わり
    // 212 → 213、イシュー #2075 で Sidebar の Themes ページが加わり
    // 213 → 214、イシュー #2109 で Bubble の Themes ページが加わり
    // 214 → 215、イシュー #2111 で Attachment（Primitives）が加わり
    // 215 → 216、イシュー #2112 で Attachment の Themes ページが加わり
    // 216 → 217、イシュー #2114 で Marker（Primitives）が加わり
    // 217 → 218、イシュー #2115 で Marker の Themes ページが加わり
    // 218 → 219 になった。イシュー #2088 で Blocks セクション（索引 1 +
    // login-01 1 = 2 ページ）が新設され、219 → 221 になった。イシュー
    // #2089 で dashboard-01 が加わり、221 → 222 になった。イシュー #2117
    // で Questionnaire（Primitives）が加わり、222 → 223 になった。イシュー
    // #2090 で sidebar-07 が加わり、223 → 224 になった。イシュー #2091 で
    // sidebar-03 が加わり、224 → 225 になった。イシュー #2093 で login-04
    // が加わり、225 → 226 になった。イシュー #2119 で Questionnaire の
    // Themes ページが加わり、226 → 227 になった。イシュー #2094 で
    // signup-01 が加わり、227 → 228 になった。イシュー #2095 で signup-05
    // が加わり、228 → 229 になった。イシュー #2121 で Message Scroller
    // （Primitives）が加わり、229 → 230 になった。イシュー #2125 で
    // Data Table（Primitives）が加わり、230 → 231 になった。イシュー
    // #2123 で Message Scroller の Themes ページが加わり、231 → 232
    // になった。
    assert_eq!(pages.len(), 232, "expected 232 pages, got {pages:?}");

    // イシュー #2088: `/blocks/` 配下は索引ページ（`/blocks/` 自身）1 件 +
    // login-01 1 件の 2 件。イシュー #2089 で dashboard-01 が加わり 3 件。
    // イシュー #2090 で sidebar-07 が加わり 4 件。イシュー #2091 で
    // sidebar-03 が加わり 5 件。イシュー #2093 で login-04 が加わり 6 件。
    // イシュー #2094 で signup-01 が加わり 7 件。イシュー #2095 で
    // signup-05 が加わり 8 件。
    let blocks_pages: Vec<&(&str, &str)> = pages
        .iter()
        .filter(|(_, path)| path.starts_with("/blocks/"))
        .collect();
    assert_eq!(
        blocks_pages.len(),
        8,
        "expected 8 /blocks/ pages (index + login-01 + login-04 + dashboard-01 + sidebar-07 + sidebar-03 + signup-01 + signup-05), got {blocks_pages:?}"
    );
    assert!(
        pages.contains(&("site/blocks.md", "/blocks/")),
        "nav.toml is missing the Blocks index page"
    );
    assert!(
        pages.contains(&("site/blocks/login-01.md", "/blocks/login-01/")),
        "nav.toml is missing the login-01 block page"
    );
    assert!(
        pages.contains(&("site/blocks/login-04.md", "/blocks/login-04/")),
        "nav.toml is missing the login-04 block page"
    );
    assert!(
        pages.contains(&("site/blocks/dashboard-01.md", "/blocks/dashboard-01/")),
        "nav.toml is missing the dashboard-01 block page"
    );
    assert!(
        pages.contains(&("site/blocks/sidebar-07.md", "/blocks/sidebar-07/")),
        "nav.toml is missing the sidebar-07 block page"
    );
    assert!(
        pages.contains(&("site/blocks/sidebar-03.md", "/blocks/sidebar-03/")),
        "nav.toml is missing the sidebar-03 block page"
    );
    assert!(
        pages.contains(&("site/blocks/signup-01.md", "/blocks/signup-01/")),
        "nav.toml is missing the signup-01 block page"
    );
    assert!(
        pages.contains(&("site/blocks/signup-05.md", "/blocks/signup-05/")),
        "nav.toml is missing the signup-05 block page"
    );

    // イシュー #1021: `/primitives/` 配下は部品ページ 63 件 + 索引ページ
    // （`/primitives/` 自身）1 件の 64 件。イシュー #2059 で Button Group・
    // イシュー #2062 で Input Group がそれぞれ加わり 64 → 66 件になった。
    // イシュー #2065 で Item が加わり 66 → 67 件になった。イシュー #2068
    // で Command が加わり 67 → 68 件になった。イシュー #2072 で Sidebar
    // が加わり 68 → 69 件になった。イシュー #2105 で Message が加わり
    // 69 → 70 件になった。イシュー #2108 で Bubble が加わり 70 → 71 件に
    // なった。イシュー #2111 で Attachment が加わり 71 → 72 件になった。
    // イシュー #2114 で Marker が加わり 72 → 73 件になった。イシュー
    // #2117 で Questionnaire が加わり 73 → 74 件になった。イシュー #2121
    // で Message Scroller が加わり 74 → 75 件になった。イシュー #2125 で
    // Data Table が加わり 75 → 76 件になった。
    let primitives_pages: Vec<&(&str, &str)> = pages
        .iter()
        .filter(|(_, path)| path.starts_with("/primitives/"))
        .collect();
    assert_eq!(
        primitives_pages.len(),
        76,
        "expected 76 /primitives/ pages (75 部品 + 1 索引), got {primitives_pages:?}"
    );
    let source_based_primitive_pages = pages
        .iter()
        .filter(|(source, _)| source.starts_with("site/primitives/"))
        .count();
    assert_eq!(
        source_based_primitive_pages, 75,
        "expected 75 pages sourced from site/primitives/"
    );
    assert!(
        pages.contains(&("site/primitives.md", "/primitives/")),
        "nav.toml is missing the Primitives index page"
    );
    // 代表 2 件（Themes 対応ページを持たない部品を含む）で (source, path) の
    // 一致を spot-check する。
    for expected_pair in [
        ("site/primitives/accordion.md", "/primitives/accordion/"),
        ("site/primitives/field.md", "/primitives/field/"),
    ] {
        assert!(
            pages.contains(&expected_pair),
            "nav.toml is missing expected primitive page {expected_pair:?}"
        );
    }

    // イシュー #1018 で索引ページ自体も `/themes/` へ移設したため、
    // `/components/` 配下に残る本体ページは 0 件（旧 URL はすべて
    // site/redirects.toml でリダイレクト）。
    let component_index_pages: Vec<&(&str, &str)> = pages
        .iter()
        .filter(|(_, path)| path.starts_with("/components/"))
        .collect();
    assert_eq!(
        component_index_pages.len(),
        0,
        "expected 0 /components/ pages (all migrated to /themes/), got {component_index_pages:?}"
    );

    // `/themes/` 配下は部品ページ 118 件 + 索引ページ（`/themes/` 自身）1 件
    // の 119 件（イシュー #1018。イシュー #1683 で部品ページが 107 → 108、
    // イシュー #1685 で 108 → 109、イシュー #1687 で 109 → 110、
    // イシュー #2063 で 110 → 111、イシュー #2066 で 111 → 112、
    // イシュー #2060 で 112 → 113、イシュー #2070 で 113 → 114、
    // イシュー #2080 で 114 → 115、イシュー #2106 で 115 → 116、
    // イシュー #2075 で Sidebar が加わり 116 → 117、イシュー #2109 で
    // Bubble が加わり 117 → 118、イシュー #2112 で Attachment が加わり
    // 118 → 119、イシュー #2115 で Marker が加わり 119 → 120、イシュー
    // #2119 で Questionnaire が加わり 120 → 121、イシュー #2123 で
    // Message Scroller が加わり 121 → 122）。
    let themes_pages: Vec<&(&str, &str)> = pages
        .iter()
        .filter(|(_, path)| path.starts_with("/themes/"))
        .collect();
    assert_eq!(
        themes_pages.len(),
        123,
        "expected 123 /themes/ pages (122 部品 + 1 索引), got {themes_pages:?}"
    );

    let source_based_component_pages = pages
        .iter()
        .filter(|(source, _)| source.starts_with("site/themes/"))
        .count();
    assert_eq!(
        source_based_component_pages, 122,
        "expected 122 pages sourced from site/themes/"
    );

    // 代表 3 件で (source, path) の一致を spot-check する（台帳・レジストリ
    // との三方突合は #944 の責務）。
    for expected_pair in [
        ("site/themes/button.md", "/themes/button/"),
        // Demo なしスタブ（showcase レジストリ未登録の 11 件の 1 つ）。
        ("site/themes/toggle.md", "/themes/toggle/"),
        // charts mod に内包される Charts カテゴリの代表。
        ("site/themes/bar-chart.md", "/themes/bar-chart/"),
    ] {
        assert!(
            pages.contains(&expected_pair),
            "nav.toml is missing expected component page {expected_pair:?}"
        );
    }

    let expected = vec![
        ("site/index.md", "/"),
        ("docs/guides/quickstart.md", "/getting-started/quickstart/"),
        ("site/guides.md", "/guides/"),
        (
            "docs/guides/component-authoring.md",
            "/guides/component-authoring/",
        ),
        ("docs/guides/embedding-guide.md", "/guides/embedding-guide/"),
        (
            "docs/guides/view-transitions.md",
            "/guides/view-transitions/",
        ),
        ("docs/guides/npm-asset-build.md", "/guides/npm-asset-build/"),
        ("docs/guides/examples.md", "/examples/"),
        ("examples/ssr-routing/README.md", "/examples/ssr-routing/"),
        ("examples/ssg-blog/README.md", "/examples/ssg-blog/"),
        (
            "examples/dist-server-docker/README.md",
            "/examples/dist-server-docker/",
        ),
        (
            "examples/interactive-view-transitions/README.md",
            "/examples/interactive-view-transitions/",
        ),
        (
            "examples/headless-pre-styled-ui/README.md",
            "/examples/headless-pre-styled-ui/",
        ),
        ("site/themes.md", "/themes/"),
        ("site/api.md", "/api/"),
        ("docs/api/component-api.md", "/api/component-api/"),
        ("docs/api/app-api.md", "/api/app-api/"),
        ("docs/api/interactive-api.md", "/api/interactive-api/"),
        ("docs/api/hydration-api.md", "/api/hydration-api/"),
        (
            "docs/api/hydration-state-format.md",
            "/api/hydration-state-format/",
        ),
        (
            "docs/api/router-path-matching.md",
            "/api/router-path-matching/",
        ),
        ("docs/api/server-api.md", "/api/server-api/"),
        ("docs/api/headless-ui-api.md", "/api/headless-ui-api/"),
        ("docs/api/pre-styled-ui-api.md", "/api/pre-styled-ui-api/"),
        (
            "docs/api/pre-styled-recipe-api.md",
            "/api/pre-styled-recipe-api/",
        ),
    ];
    // "site/index.md" は #472（未マージのことがある）依存のため、期待値の
    // 先頭は上のリストに含めつつ後段の実在チェックでは条件付きにする。
    // ここでは path/source の宣言内容そのもの（22 件）を厳格検証する
    // （イシュー #943 で部品ページ 99 件が加わり `pages.len()` は 121 に
    // なったため、`expected` は既存 22 件のみを列挙する部分集合として
    // 個別に `contains` で照合する。総数チェックは冒頭の 121/100/99 の
    // 各アサーションが担う）。
    for expected_pair in &expected {
        assert!(
            pages.contains(expected_pair),
            "nav.toml is missing expected page {expected_pair:?}"
        );
    }
}

#[test]
fn site_nav_has_no_duplicate_paths_or_sources() {
    let nav = load_nav();
    let mut seen_paths = std::collections::BTreeSet::new();
    let mut seen_sources = std::collections::BTreeSet::new();
    // `nav.all_pages()`（イシュー #939 の唯一の正規走査経路）で走査する。
    // `section.pages` を直接走査すると `[[section.group]]` 配下のページが
    // 対象から漏れ、#943 でグループが登録された際に重複検知が沈黙する
    // （Bugbot 指摘、PR #968）。
    for page in nav.all_pages() {
        assert!(
            seen_paths.insert(page.path.clone()),
            "duplicate page.path: {}",
            page.path
        );
        assert!(
            seen_sources.insert(page.source.clone()),
            "duplicate page.source: {}",
            page.source
        );
    }
}

/// `site/index.md` を除く 11 source は #465〜#469 マージ済みの時点で必ず
/// repo_root 配下に実在する（#472 に依存しない既存資産）。
#[test]
fn site_nav_sources_other_than_site_index_exist() {
    let root = repo_root();
    let nav = load_nav();
    // `nav.all_pages()`（イシュー #939 の唯一の正規走査経路）で走査する。
    // `section.pages` を直接走査すると `[[section.group]]` 配下のページが
    // 対象から漏れ、#943 でグループが登録された際に実在確認が沈黙する
    // （Bugbot 指摘、PR #968）。
    for page in nav.all_pages() {
        if page.source == "site/index.md" {
            continue;
        }
        let full_path = root.join(&page.source);
        assert!(
            full_path.is_file(),
            "expected source file to exist: {}",
            page.source
        );
    }
}

/// `site/index.md`（#472）が既にマージされていれば、`validate_sources` は
/// 全ページを対象に成功しなければならない。未マージであれば本テストは
/// 早期リターンし、他 11 ページ分の検証は上記テストに委ねる。#472 マージ後は
/// 本テストが自動的に全件厳格検証へ切り替わる。
#[test]
fn site_nav_validate_sources_covers_all_pages_once_site_index_exists() {
    let root = repo_root();
    if !root.join("site/index.md").is_file() {
        // #472（site/index.md 新設）が未マージ。フォールバックとして
        // 他 11 ページの実在確認は site_nav_sources_other_than_site_index_exist
        // が担保する。
        return;
    }
    let nav = load_nav();
    validate_sources(&nav, &root)
        .expect("all page.source entries should exist once site/index.md is present");
}

/// 受け入れ条件 2: サブセット外構文の残存によるレンダリング崩れがない。
///
/// `render_markdown` はブロックレベル解釈のみを担う（インライン構文は
/// #467 が別途差し替える契約、`markdown.rs` の `inline_nodes` 参照）ため、
/// 本テストはブロック構造の健全性（先頭が見出しであること・フェンス
/// コードの閉じ忘れがテキストノードに漏れ出していないこと）のみを確認する。
#[test]
fn every_existing_source_renders_without_fence_leakage_and_starts_with_a_heading() {
    let root = repo_root();
    let nav = load_nav();
    // `nav.all_pages()`（イシュー #939 の唯一の正規走査経路）で走査する。
    // `section.pages` を直接走査すると `[[section.group]]` 配下のページ
    // （イシュー #943 で追加された 99 部品ページ）が対象から漏れ、
    // レンダリング健全性検証が沈黙する（他 3 テストと同型の Bugbot 指摘、
    // PR #968 参照）。
    for page in nav.all_pages() {
        let full_path = root.join(&page.source);
        if !full_path.is_file() {
            // site/index.md が未マージの場合はスキップ（#472 依存）。
            continue;
        }
        let input = std::fs::read_to_string(&full_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", page.source));
        let blocks = render_markdown(&input);
        assert!(
            !blocks.is_empty(),
            "{} rendered to an empty block list",
            page.source
        );
        assert!(
            is_heading(&blocks[0]),
            "{} does not start with a heading (H1 expected as the page title)",
            page.source
        );
        for block in &blocks {
            assert!(
                !contains_unclosed_fence_marker(block),
                "{} contains a stray ``` marker in rendered text, likely an unclosed fence",
                page.source
            );
        }
    }
}

fn is_heading(node: &Node) -> bool {
    matches!(
        node,
        Node::Element { tag, .. } if matches!(*tag, "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
    )
}

/// テキストノードに ``` （フェンス開始/終了マーカー）がそのまま残っていないか
/// 再帰的に確認する。閉じ忘れたフェンスはブロックパーサが段落テキストとして
/// フォールバックするため、マーカー文字列がテキストノードに漏れ出る形で検知できる。
fn contains_unclosed_fence_marker(node: &Node) -> bool {
    match node {
        Node::Text(s) => s.contains("```"),
        Node::RawHtml(_) => false,
        Node::Element { children, .. } => children.iter().any(contains_unclosed_fence_marker),
    }
}

/// イシュー #955（設計 `docs/design/docs-site-api-reference-split.md` §3-3）:
/// `docs/internal/` は内部設計メモ置き場であり、サイト非出力を「nav 未登録」
/// だけで構造的に担保する契約（除外リスト・後付けフィルタは設けない、
/// 同文書 §4 A05）。本テストは `docs/internal/*.md` を実ディレクトリ走査で
/// 列挙し、どのファイルも `nav.all_pages()` の `source` と一致しないこと、
/// および逆方向（どの登録済み `source` も `docs/internal/` 配下を指さない
/// こと）を双方向に固定する。将来 `docs/internal/` へファイルが追加されても
/// 本テストは走査ベースのため自動的に対象へ含まれ、誤って nav へ登録された
/// 瞬間にリポジトリが public である以上サイト経由で公開されてしまう回帰を
/// fail-closed に検知する。
#[test]
fn docs_internal_notes_are_never_registered_in_nav() {
    let root = repo_root();
    let internal_dir = root.join("docs/internal");
    assert!(
        internal_dir.is_dir(),
        "docs/internal/ should exist as the internal design-notes directory (design doc §3-3)"
    );

    let mut internal_sources = std::collections::BTreeSet::new();
    for entry in std::fs::read_dir(&internal_dir)
        .unwrap_or_else(|e| panic!("failed to read docs/internal/: {e}"))
    {
        let entry = entry.unwrap_or_else(|e| panic!("failed to read docs/internal/ entry: {e}"));
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let relative = path
            .strip_prefix(&root)
            .expect("docs/internal/ entries should be under repo_root")
            .to_str()
            .expect("docs/internal/ entry path should be valid UTF-8")
            .replace('\\', "/");
        internal_sources.insert(relative);
    }
    assert!(
        !internal_sources.is_empty(),
        "docs/internal/ should contain at least the known implementation-notes files \
         (headless-ui-implementation-notes.md / pre-styled-ui-implementation-notes.md / \
         pre-styled-recipe-implementation-notes.md); an empty directory likely signals a \
         layout change this test was not updated for"
    );

    let nav = load_nav();
    let registered_sources: std::collections::BTreeSet<&str> =
        nav.all_pages().map(|p| p.source.as_str()).collect();

    for internal_source in &internal_sources {
        assert!(
            !registered_sources.contains(internal_source.as_str()),
            "docs/internal/ note {internal_source} must never be registered in site/nav.toml \
             (design doc §3-3: sites non-publication is guaranteed solely by nav non-registration; \
             the repository is public, so registering this would publish it via the docs site)"
        );
    }

    for source in &registered_sources {
        assert!(
            !source.starts_with("docs/internal/"),
            "site/nav.toml registers {source} under docs/internal/, which must stay unregistered \
             (design doc §3-3 / §4 A05)"
        );
    }
}
