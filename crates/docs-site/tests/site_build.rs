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
    // 加わり、125 → 127 になった。
    assert_eq!(
        report.written.len(),
        127,
        "実サイトの生成ページ数が期待値と異なる: {:?}",
        report.written
    );

    // 上の 127 は「その時点の実測値」であり、Phase 6/7/8 でページが増減したら
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

    // /components/ 配下は部品ページ 105 件（イシュー #991 で Toolbar が
    // 加わり 99 → 100、イシュー #992 で Menubar が加わり 100 → 101、
    // イシュー #993 で Navigation Menu が加わり 101 → 102、イシュー #994 で
    // Callout が加わり 102 → 103、イシュー #995 で Quote / Strong が加わり
    // 103 → 105） + 索引ページ /components/pre-styled-ui/ 1 件の計 106 件
    // （イシュー #943）。
    // Phase 4 以降で部品が増減したら本値の更新が必要になる
    // （fail-closed。黙って減っても気付けるようにする意図）。
    let components_dir = out.0.join("components");
    let component_pages = report
        .written
        .iter()
        .filter(|p| p.starts_with(&components_dir))
        .count();
    assert_eq!(
        component_pages, 106,
        "/components/ 配下の生成ページ数（部品 105 + 索引 1）"
    );

    // アセットは site.css / admonition.css / skip-nav.css / site.js /
    // pre-styled-ui.css / search-index.json の 6 件（部品ページが showcase
    // CSS を要求するため実サイトでは fixture（5 件）より 1 件多い）。
    assert_eq!(report.assets.len(), 6, "{:?}", report.assets);
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
        // イシュー #943: /components/pre-styled-ui/ は索引ページへ改組済みで
        // Rust 生成コンテンツ（pre-styled-ui.css 配線）を持たない。ショーケース
        // CSS 配線の代表は部品ページ（dialog）側で確認する。
        ("components/pre-styled-ui/index.html", false),
        ("components/dialog/index.html", true),
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
