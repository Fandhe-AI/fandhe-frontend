//! docs サイトビルドエントリの E2E テスト（イシュー #470 受け入れ条件）。
//!
//! `tests/fixtures/site-ok/` / `tests/fixtures/site-broken-link/` の
//! ミニリポジトリ（`site/nav.toml` + Markdown + `site/assets/`）に対して
//! [`fandhe_frontend_docs_site::build::build_site`] を直接呼ぶテストと、
//! `env!("CARGO_BIN_EXE_docs-site")` でバイナリ本体を起動して終了コード・
//! stderr を検証するテストの 2 系統からなる。
//!
//! フィクスチャは cargo プロジェクトではない単なるディレクトリのため、
//! 共有 `CARGO_TARGET_DIR`（`ci.md`）のキャッシュ誤命中問題は生じない
//! （バイナリ実行のみで `cargo build` を再度呼ばない）。
//!
//! 受け入れ条件 3 の実サイトビルド検証（`env!("CARGO_MANIFEST_DIR")/../.."`
//! をルートに実際の `site/nav.toml` でビルド）もここに含める。以後の
//! docs 編集によるリンク切れを `cargo test` が継続的に検出する
//! （ドッグフーディング保証）。

use std::path::{Path, PathBuf};
use std::process::Command;

use fandhe_frontend_docs_site::build::{build_site, BuildError};

/// 統合テストのスクラッチ基点。`CARGO_TARGET_TMPDIR` は cargo が統合テスト
/// バイナリの**コンパイル時のみ**設定する（Cargo Book）ため `env!` で確定し、
/// 実行時 env による明示上書きのみ許容する。`/tmp` へは一切フォールバック
/// しない（イシュー #637 の事実誤認の再発防止、#658、`cli/tests/support/mod.rs`
/// と同一パターン）。
fn scratch_root() -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    let _ = std::fs::create_dir_all(&root);
    root
}

/// テスト専用の一時出力ディレクトリ。`crates/docs-site/src/nav.rs` の
/// `TempDir` と同方針（外部クレート `tempfile` を追加しない、REQ-3）。
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = scratch_root().join(format!(
            "fandhe-frontend-docs-site-e2e-{tag}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("create temp dir for site_build.rs test");
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn fixture_root(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

// ---- lib 経由（build_site を直接呼ぶ） ----

#[test]
fn build_site_generates_all_pages_and_assets_for_ok_fixture() {
    let out = TempDir::new("ok");
    let report =
        build_site(&fixture_root("site-ok"), &out.0).expect("site-ok fixture should build");

    assert_eq!(report.written.len(), 2);
    // site.css（`site_theme` のビルド時生成、イシュー #905） + admonition.css
    // （site-ok の index.md が admonition マーカーを 1 件使うため、
    // `crate::admonition` 専用 CSS も条件付きで書き出される）
    // + skip-nav.css（イシュー #776、全ビルドで無条件に書き出す）
    // + site.js（イシュー #951、同じく全ビルドで無条件に書き出す）
    // + search-index.json（イシュー #957、同じく全ビルドで無条件に書き出す）。
    assert_eq!(report.assets.len(), 5);
    assert!(out.0.join("index.html").exists());
    assert!(out.0.join("guide/quickstart/index.html").exists());
    assert!(out.0.join("assets/site.css").exists());
    assert!(out.0.join("assets/admonition.css").exists());
    assert!(out.0.join("assets/skip-nav.css").exists());
    assert!(out.0.join("assets/site.js").exists());
    assert!(out.0.join("assets/search-index.json").exists());
}

/// イシュー #715: admonition 専用 CSS（`assets/admonition.css`）への
/// `<link>` は admonition を実際に使うページにのみ差し込まれ、使わない
/// ページ（quickstart）には現れないことを固定する（showcase と同型の
/// 「使われているページだけ」配線、`build.rs` モジュール doc 参照）。
#[test]
fn build_site_wires_admonition_css_only_to_pages_using_it() {
    let out = TempDir::new("admonition-wiring");
    build_site(&fixture_root("site-ok"), &out.0).expect("site-ok fixture should build");

    let index_html = std::fs::read_to_string(out.0.join("index.html")).unwrap();
    assert!(index_html.contains(r#"href="/fixture-base/assets/admonition.css""#));
    assert!(index_html.contains(r#"data-scope="alert""#));

    let quickstart_html =
        std::fs::read_to_string(out.0.join("guide/quickstart/index.html")).unwrap();
    assert!(!quickstart_html.contains("admonition.css"));
    assert!(!quickstart_html.contains(r#"data-scope="alert""#));

    let admonition_css = std::fs::read_to_string(out.0.join("assets/admonition.css")).unwrap();
    assert!(admonition_css.contains(".fd-alert--status-info"));
    // イシュー #732: 実サイトビルドが書き出す admonition.css にダーク
    // モード配色条件が含まれること。
    assert!(admonition_css.contains("prefers-color-scheme: dark"));
}

#[test]
fn build_site_rewrites_md_links_to_site_paths_for_ok_fixture() {
    let out = TempDir::new("md-rewrite");
    build_site(&fixture_root("site-ok"), &out.0).expect("site-ok fixture should build");

    let index_html = std::fs::read_to_string(out.0.join("index.html")).unwrap();
    assert!(index_html.contains(r#"href="/fixture-base/guide/quickstart/""#));
    assert!(!index_html.contains(".md"));

    let quickstart_html =
        std::fs::read_to_string(out.0.join("guide/quickstart/index.html")).unwrap();
    assert!(quickstart_html.contains(r#"href="/fixture-base/""#));
    assert!(!quickstart_html.contains(".md"));
}

#[test]
fn build_site_fails_closed_and_writes_nothing_for_broken_link_fixture() {
    let temp = TempDir::new("broken");
    // `TempDir::new` 自体が一時ディレクトリを作成するため、`out_dir` には
    // その配下の未作成サブディレクトリを渡す（fail-closed で一切書き出さない
    // ことを「サブディレクトリが作成されないこと」で検証するため）。
    let out_dir = temp.0.join("dist");
    let err = build_site(&fixture_root("site-broken-link"), &out_dir)
        .expect_err("site-broken-link fixture should fail the build");

    match err {
        BuildError::LinkCheck(broken) => {
            assert_eq!(broken.len(), 1);
            assert!(broken[0].href.contains("missing.md"));
        }
        other => panic!("expected LinkCheck, got {other:?}"),
    }
    assert!(
        !out_dir.exists(),
        "out_dir must not exist on link-check failure"
    );
}

/// 受け入れ条件 3: `cargo run -p fandhe-frontend-docs-site -- --out dist/` が
/// リポジトリ自身の `site/nav.toml` で成功し続けることをドッグフーディング
/// 保証として固定する。以後の docs 編集によるリンク切れも本テストが検出する。
#[test]
fn build_site_succeeds_for_the_real_repository_site() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve repository root");
    let out = TempDir::new("real-site");

    let report = build_site(&repo_root, &out.0).expect("real site/nav.toml should build cleanly");
    assert!(!report.written.is_empty());
    assert!(!report.assets.is_empty());
    assert!(out.0.join("index.html").exists());

    // イシュー #944: #943 で部品ページ 99 件が加わり、実サイトの生成ページ数は
    // 121（既存 22 + 部品 99）になった。site_nav.rs は nav 登録側の件数を、
    // 本テストは build_site が実際に書き出したページ側の件数を固定する
    // （nav 登録 = 生成ページの恒等契約。片側だけ壊れる退行を検知する）。
    // 部品ページの台帳は docs/design/docs-site-component-pages.md §3。
    // イシュー #991 で Toolbar（`site/components/toolbar.md`）が加わり、
    // 121 → 122 になった。イシュー #992 で Menubar
    // （`site/components/menubar.md`）が加わり、122 → 123 になった。
    // イシュー #993 で Navigation Menu
    // （`site/components/navigation-menu.md`）が加わり、123 → 124 になった。
    // イシュー #994 で Callout（`site/components/callout.md`）が加わり、
    // 124 → 125 になった。イシュー #995 で Quote / Strong の 2 ページが
    // 加わり、125 → 127 になった。イシュー #996 で Tab Nav
    // （`site/components/tab-nav.md`）が加わり、127 → 128 になった。
    // イシュー #997 で Checkbox Group（`site/components/checkbox-group.md`）
    // が加わり、128 → 129 になった。イシュー #1009 で Guides / API Reference
    // のセクショントップページ（`site/guides.md` / `site/api.md`）2 ページが
    // 加わり、129 → 131 になった（部品ページ 107 件は不変）。イシュー #1021 で
    // Primitives セクション（索引 1 + 部品 63 = 64 ページ）が新設され、
    // 131 → 195 になった。イシュー #1118 で Guides セクションへ「JS ゼロ
    // SSG での利用ガイド」（`docs/guides/no-js-ssg.md`）が加わり、
    // 195 → 196 になった。イシュー #1156 で API Reference セクションへ
    // `docs/api/server-api.md`（`generate_assets` 等 SSG API リファレンス）
    // が加わり、196 → 197 になった。
    assert_eq!(
        report.written.len(),
        197,
        "実サイトの生成ページ数が期待値と異なる: {:?}",
        report.written
    );

    // 上の 131 は「その時点の実測値」であり、Phase 6/7/8 でページが増減したら
    // 更新が要る。恒等契約（nav 登録数 = 生成ページ数）そのものは値に依存しない
    // 形でも固定し、片方だけ更新して片方が形骸化する事故を防ぐ。
    let nav_toml = std::fs::read_to_string(repo_root.join("site/nav.toml"))
        .expect("site/nav.toml should be readable");
    let nav =
        fandhe_frontend_docs_site::nav::parse_nav(&nav_toml).expect("site/nav.toml should parse");
    assert_eq!(
        report.written.len(),
        nav.all_pages().count(),
        "nav 登録ページ数と生成ページ数が一致しない"
    );

    // イシュー #1017 で既存 107 部品ページを `/components/<kebab>/` から
    // `/themes/<kebab>/` へ移行し、イシュー #1018 で索引ページ自体も
    // `/components/pre-styled-ui/` から `/themes/` へ移設した。
    // `/components/` 配下の本体ページ（`report.written`。リダイレクトページは
    // `report.redirects` に別計上されるため対象外）は 0 件になり、移行先
    // `/themes/` 配下に部品 107 件 + 索引 1 件 = 108 件が生成される。
    // Phase 4 以降で部品が増減したら両方の値の更新が必要になる
    // （fail-closed。黙って減っても気付けるようにする意図）。
    let components_dir = out.0.join("components");
    let component_index_pages = report
        .written
        .iter()
        .filter(|p| p.starts_with(&components_dir))
        .count();
    assert_eq!(
        component_index_pages, 0,
        "/components/ 配下の生成ページ数（本体ページは全件 /themes/ へ移行済み）"
    );

    let themes_dir = out.0.join("themes");
    let theme_pages = report
        .written
        .iter()
        .filter(|p| p.starts_with(&themes_dir))
        .count();
    assert_eq!(
        theme_pages, 108,
        "/themes/ 配下の生成ページ数（部品 107 件 + 索引 1 件）"
    );

    // イシュー #1021: `/primitives/` 配下は部品 63 件 + 索引 1 件 = 64 件。
    let primitives_dir = out.0.join("primitives");
    let primitive_pages = report
        .written
        .iter()
        .filter(|p| p.starts_with(&primitives_dir))
        .count();
    assert_eq!(
        primitive_pages, 64,
        "/primitives/ 配下の生成ページ数（部品 63 件 + 索引 1 件）"
    );

    // アセットは site.css / admonition.css / skip-nav.css / site.js /
    // pre-styled-ui.css / primitives-showcase.css / search-index.json の
    // 7 件（部品ページが showcase CSS を要求するため実サイトでは
    // fixture（5 件）より多い。イシュー #1022 で primitives-showcase.css が
    // 加わり 6 → 7 になった）。
    assert_eq!(report.assets.len(), 7, "{:?}", report.assets);

    // イシュー #1016: リダイレクトページは `written`（本体ページ）にも
    // `assets` にも含めない独立フィールド（`BuildReport::redirects`）。
    // `site/redirects.toml` の宣言件数（1 件、`tests/redirects.rs` が
    // fail-closed に固定）と生成ページ数が一致することを固定する
    // （nav 登録数 = 本体ページ数の恒等契約と同型）。
    let redirects_toml = std::fs::read_to_string(
        repo_root.join(fandhe_frontend_docs_site::redirect::MANIFEST_REL_PATH),
    )
    .expect("site/redirects.toml should be readable");
    let redirects = fandhe_frontend_docs_site::redirect::parse_redirects(&redirects_toml)
        .expect("site/redirects.toml should parse");
    assert_eq!(
        report.redirects.len(),
        redirects.entries.len(),
        "リダイレクト宣言数と生成リダイレクトページ数が一致しない"
    );
    assert!(
        out.0.join("components/index.html").exists(),
        "redirect declared from /components/ should produce components/index.html"
    );
    // イシュー #1018: 索引ページ移設に伴う旧 URL 互換リダイレクト
    // （`/components/pre-styled-ui/` → `/themes/`）の生成物を固定する。
    assert!(
        out.0.join("components/pre-styled-ui/index.html").exists(),
        "redirect declared from /components/pre-styled-ui/ should produce components/pre-styled-ui/index.html"
    );
    for rel in [
        "assets/site.css",
        "assets/admonition.css",
        "assets/skip-nav.css",
        "assets/site.js",
        "assets/pre-styled-ui.css",
        "assets/search-index.json",
    ] {
        assert!(out.0.join(rel).exists(), "{rel} should be written");
    }

    // イシュー #908: 実サイトの nav.toml が持つセクション別ドロップダウン
    // ヘッダーナビが実際に配線されていることを確認する（`nav::header_nav`
    // が全ページ共通で `layout::docs_page_with_assets` へ渡される契約）。
    let index_html =
        std::fs::read_to_string(out.0.join("index.html")).expect("read generated index.html");
    assert!(index_html.contains(r#"class="docs-header-nav""#));
    assert!(index_html.contains("Getting Started"));
    assert!(out.0.join("assets/site.css").exists());
    let site_css = std::fs::read_to_string(out.0.join("assets/site.css"))
        .expect("read generated assets/site.css");
    assert!(site_css.contains(".docs-header-dropdown"));

    // イシュー #1012: ヘッダートリガーがセクショントップページへの遷移
    // リンク（`a[href]`）へ切り替わり、`<button>` が使われなくなったこと・
    // ドロップダウン（`ul.docs-header-dropdown`）が引き続き出力されること
    // を実サイトの生成物で固定する。ヘッダーナビブロックだけを切り出して
    // 判定する（`.docs-theme-toggle` 等は引き続き `<button>` のため、
    // ページ全体での `<button` 不在は主張できない）。
    let header_nav_start = index_html
        .find(r#"class="docs-header-nav""#)
        .expect("header nav should be present");
    let header_nav_end = index_html[header_nav_start..]
        .find("</nav>")
        .map(|rel| header_nav_start + rel)
        .expect("header nav should close with </nav>");
    let header_nav_block = &index_html[header_nav_start..header_nav_end];
    assert!(!header_nav_block.contains("<button"));
    assert!(header_nav_block.contains(r#"class="docs-header-trigger""#));
    assert!(header_nav_block.contains(r#"class="docs-header-dropdown""#));
    // Getting Started セクションのトリガー href（index_path = "/"）。
    assert!(header_nav_block.contains(r#"href="/fandhe-frontend/""#));
}

/// イシュー #912（受け入れ条件 1 の機械検証可能な半分）: 実サイトの
/// 4 ページ種別（index / ガイド / ショーケース / API リファレンス）すべてで
/// 3 カラム骨格・SkipNav・View Transitions opt-in・両 CSS リンクが揃うことを
/// 固定する。`build_site_succeeds_for_the_real_repository_site` は index の
/// みを確認するため、他ページ種別への横展開として追加する（既存テストは
/// 変更しない）。
#[test]
fn real_site_build_covers_all_page_kinds_with_shared_layout_contract() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve repository root");
    let out = TempDir::new("all-page-kinds");

    build_site(&repo_root, &out.0).expect("real site/nav.toml should build cleanly");

    // (相対パス, ショーケースページか否か)。ショーケースのみ
    // `assets/pre-styled-ui.css` への追加 `<link>` を持つ
    // （`crate::build::build_site` の「使われているページだけ」配線）。
    let pages: &[(&str, bool)] = &[
        ("index.html", false),
        ("guides/view-transitions/index.html", false),
        // イシュー #943: /themes/ は索引ページへ改組済みで Rust 生成コンテンツ
        // （pre-styled-ui.css 配線）を持たない。ショーケース CSS 配線の代表は
        // 部品ページ（dialog）側で確認する。イシュー #1018 で索引ページ URL が
        // `/components/pre-styled-ui/` から `/themes/` へ移設したため対象パスを
        // 追随させる（`components/pre-styled-ui/index.html` は現在クロームを
        // 持たないリダイレクトページであり、本テストの対象には使えない）。
        ("themes/index.html", false),
        // イシュー #1017 で /components/dialog/ から /themes/dialog/ へ移行。
        ("themes/dialog/index.html", true),
        ("api/component-api/index.html", false),
    ];

    for (relative, is_showcase) in pages {
        let html = std::fs::read_to_string(out.0.join(relative))
            .unwrap_or_else(|e| panic!("read generated {relative}: {e}"));

        for needle in [
            r#"class="docs-container"#,
            r#"class="docs-sidebar""#,
            r#"class="docs-main""#,
            r#"class="docs-content""#,
            r#"class="docs-toc-aside""#,
            r#"data-scope="skip-nav""#,
            r#"data-part="link""#,
            r#"data-part="content""#,
            "@view-transition { navigation: auto; }",
            r#"href="/fandhe-frontend/assets/site.css""#,
            r#"href="/fandhe-frontend/assets/skip-nav.css""#,
            r#"src="/fandhe-frontend/assets/site.js" defer="""#,
            r#"class="docs-header-actions""#,
            r#"class="docs-github-link""#,
            r#"class="docs-theme-toggle""#,
        ] {
            assert!(
                html.contains(needle),
                "{relative} should contain {needle:?} (3 カラム骨格・SkipNav・View Transitions・CSS/JS 配線の共通契約)"
            );
        }

        let has_showcase_css = html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#);
        assert_eq!(
            has_showcase_css, *is_showcase,
            "{relative}: pre-styled-ui.css link presence should match is_showcase={is_showcase}"
        );
    }

    // イシュー #951: 実サイトビルドで `dist/assets/site.js` が生成され、
    // 内容が `script::site_js()` とバイト一致することを固定する
    // （`build.rs` の `fs::write` 書き出しが常に同一内容を書くことの回帰
    // テスト。生成物が実装から乖離した場合に検知する）。
    let dist_site_js = std::fs::read_to_string(out.0.join("assets/site.js"))
        .expect("dist/assets/site.js should be generated");
    assert_eq!(dist_site_js, fandhe_frontend_docs_site::script::site_js());
}

/// イシュー #1013: 実サイト生成物でサイドバーが現在セクションへスコープ
/// されていることを回帰固定する。`header_nav`（全セクションのトリガー +
/// 直下ページのドロップダウン）はスコープ対象外で全セクション分を出し
/// 続けるため、断定は必ず `docs-sidebar` ブロック（`</aside>` までの窓）へ
/// 限定する。ファイル全体への否定的 grep は `header_nav` の出力に必ず
/// 当たって落ちるため、意図的にテストを緩める方向の「修正」を誘発しない
/// ようにするための窓限定である（`nav.rs::sidebar` rustdoc「セクション
/// スコープ」節参照）。
#[test]
fn real_site_sidebar_is_scoped_to_the_current_section() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve repository root");
    let out = TempDir::new("sidebar-scope");

    build_site(&repo_root, &out.0).expect("real site/nav.toml should build cleanly");

    // `docs-sidebar` ブロックのみを取り出すヘルパ。
    fn sidebar_window(html: &str) -> &str {
        let start = html
            .find(r#"class="docs-sidebar""#)
            .expect("docs-sidebar block should be present");
        let end = html[start..]
            .find("</aside>")
            .map(|rel| start + rel)
            .expect("docs-sidebar block should close with </aside>");
        &html[start..end]
    }

    // Guides セクション内のページ（Components 配下は 1 件も出ない）。
    // イシュー #1017 で部品ページの URL が `/themes/` へ移行したため、
    // ここでの否定確認対象も `/themes/` へ追随する。
    let guides_html = std::fs::read_to_string(out.0.join("guides/view-transitions/index.html"))
        .expect("read generated guides/view-transitions/index.html");
    let guides_window = sidebar_window(&guides_html);
    assert!(guides_window.contains(r#"href="/fandhe-frontend/guides/""#));
    assert!(!guides_window.contains("/themes/"));
    assert!(!guides_window.contains("docs-nav-group"));
    assert_eq!(guides_window.matches("<h2").count(), 1);
    assert_eq!(guides_window.matches(r#"aria-current="page""#).count(), 1);

    // Components セクション内のページ（Guides 配下は 1 件も出ない、現在
    // グループのみ open）。イシュー #1017 で `/components/button/` から
    // `/themes/button/` へ移行。イシュー #1021: Primitives セクション新設に
    // 伴い、`/primitives/` が 1 件も混入しないことも合わせて固定する
    // （逆方向の混入も同時に fail-closed にする、計画 §6-1b）。
    let themes_html = std::fs::read_to_string(out.0.join("themes/button/index.html"))
        .expect("read generated themes/button/index.html");
    let themes_window = sidebar_window(&themes_html);
    assert!(themes_window.contains("docs-nav-group"));
    assert_eq!(
        themes_window
            .matches(r#"<details class="docs-nav-group" open="">"#)
            .count(),
        1
    );
    assert!(!themes_window.contains("/guides/"));
    assert!(!themes_window.contains("/primitives/"));
    assert_eq!(themes_window.matches("<h2").count(), 1);
    assert_eq!(themes_window.matches(r#"aria-current="page""#).count(), 1);

    // イシュー #1021 受け入れ条件 5: Primitives セクション内のページのサイド
    // バーが Themes/Guides を一切含まず、Primitives 自身のグループ・
    // リンク集合に限定されていることを固定する（目視確認に委ねない、
    // 計画 §6-1b）。否定形だけでは空窓でも通ってしまうため、肯定形
    // （現在グループが開いている・部品 63 + 索引 1 = 64 件のリンクが
    // すべて `/primitives/` 配下）も合わせて確認する。
    let primitives_html = std::fs::read_to_string(out.0.join("primitives/accordion/index.html"))
        .expect("read generated primitives/accordion/index.html");
    let primitives_window = sidebar_window(&primitives_html);
    assert!(!primitives_window.contains("/themes/"));
    assert!(!primitives_window.contains("/guides/"));
    assert!(primitives_window.contains(r#"href="/fandhe-frontend/primitives/""#));
    assert!(primitives_window.contains("docs-nav-group"));
    assert_eq!(
        primitives_window
            .matches(r#"<details class="docs-nav-group" open="">"#)
            .count(),
        1
    );
    assert_eq!(primitives_window.matches("<h2").count(), 1);
    assert_eq!(
        primitives_window.matches(r#"aria-current="page""#).count(),
        1
    );
    let primitives_link_count = primitives_window
        .matches("/fandhe-frontend/primitives/")
        .count();
    assert_eq!(
        primitives_link_count, 64,
        "Primitives サイドバーのリンク数が索引 1 + 部品 63 = 64 件と一致しない: {primitives_window}"
    );
}

// ---- バイナリ経由（終了コード・stderr の契約） ----

fn docs_site_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_docs-site"))
}

#[test]
fn binary_exits_zero_and_reports_written_counts_for_ok_fixture() {
    let out = TempDir::new("bin-ok");
    let output = Command::new(docs_site_bin())
        .arg("--root")
        .arg(fixture_root("site-ok"))
        .arg("--out")
        .arg(&out.0)
        .output()
        .expect("spawn docs-site binary");

    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out.0.join("index.html").exists());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("wrote 2 page(s)"));
}

#[test]
fn binary_exits_nonzero_with_link_check_report_for_broken_fixture() {
    let temp = TempDir::new("bin-broken");
    let out_dir = temp.0.join("dist");
    let output = Command::new(docs_site_bin())
        .arg("--root")
        .arg(fixture_root("site-broken-link"))
        .arg("--out")
        .arg(&out_dir)
        .output()
        .expect("spawn docs-site binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("link check failed"));
    assert!(stderr.contains("missing.md"));
    assert!(!out_dir.exists());
}

#[test]
fn binary_exits_nonzero_when_out_argument_is_missing() {
    let output = Command::new(docs_site_bin())
        .arg("--root")
        .arg(fixture_root("site-ok"))
        .output()
        .expect("spawn docs-site binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--out"));
}

#[test]
fn binary_exits_nonzero_for_unknown_argument() {
    let out = TempDir::new("bin-unknown-arg");
    let output = Command::new(docs_site_bin())
        .arg("--out")
        .arg(&out.0)
        .arg("--bogus")
        .output()
        .expect("spawn docs-site binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown argument"));
}
