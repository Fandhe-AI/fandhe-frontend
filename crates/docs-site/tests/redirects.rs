//! 旧 URL 互換のリダイレクトページ生成機構の統合テスト（イシュー #1016）。
//!
//! `crates/docs-site/src/redirect.rs` の unit test がパーサ・パス検証・
//! レンダラの純粋なロジックを固定するのに対し、本ファイルは
//! [`build_site`] を実際に呼び「一時フィクスチャの `site/redirects.toml` が
//! ビルドパイプライン全体（`nav.toml` との突合検証・`out_dir` への書き出し・
//! fail-closed な打ち切り）へ正しく配線されていること」を固定する。
//!
//! 併せて `site/redirects.toml`（実マニフェスト）の宣言件数を fail-closed に
//! 固定し、#1017/#1018 が実宣言を追加する際にこの期待値の更新を要求する
//! （`tests/site_build.rs` のページ数固定と同じ規律）。

use std::path::{Path, PathBuf};

use fandhe_frontend_docs_site::build::{build_site, BuildError};
use fandhe_frontend_docs_site::redirect::{self, RedirectError};

/// 統合テストのスクラッチ基点。`CARGO_TARGET_TMPDIR` は cargo が統合テスト
/// バイナリの**コンパイル時のみ**設定する（Cargo Book）ため `env!` で確定し、
/// 実行時 env による明示上書きのみ許容する。`/tmp` へは一切フォールバック
/// しない（イシュー #637 の事実誤認の再発防止、`tests/site_build.rs` と
/// 同一パターン）。
fn scratch_root() -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    let _ = std::fs::create_dir_all(&root);
    root
}

/// テスト専用の一時ディレクトリ。`tests/site_build.rs::TempDir` と
/// 同方針（外部クレート `tempfile` を追加しない、REQ-3）。
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = scratch_root().join(format!(
            "fandhe-frontend-docs-site-redirects-{tag}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("create temp dir for redirects.rs test");
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// `nav.toml` 2 ページ（`/` と `/next/`）を持つ最小フィクスチャを書く。
/// `redirects.toml` は呼び出し側が個別に追加する。
fn write_fixture_site(root: &Path) {
    std::fs::create_dir_all(root.join("site")).unwrap();
    std::fs::write(
        root.join("site/nav.toml"),
        r#"
[site]
title = "Docs"
base_path = "/fixture-base"

[[section]]
title = "Guide"
index_path = "/"

[[section.page]]
title = "Intro"
source = "site/intro.md"
path = "/"

[[section.page]]
title = "Next"
source = "site/next.md"
path = "/next/"
"#,
    )
    .unwrap();
    std::fs::write(root.join("site/intro.md"), "# Intro\n\nHello.\n").unwrap();
    std::fs::write(root.join("site/next.md"), "# Next\n\nBack to intro.\n").unwrap();
}

fn write_redirects_toml(root: &Path, contents: &str) {
    std::fs::write(root.join("site/redirects.toml"), contents).unwrap();
}

// ---- fail-closed: `to` 不在 ----

#[test]
fn build_fails_closed_when_redirect_target_does_not_exist() {
    let temp = TempDir::new("unknown-target");
    write_fixture_site(&temp.0);
    write_redirects_toml(
        &temp.0,
        "[[redirect]]\nfrom = \"/old/\"\nto = \"/does-not-exist/\"\n",
    );
    let out_dir = temp.0.join("dist");

    let err = build_site(&temp.0, &out_dir)
        .expect_err("redirect targeting a non-existent page should fail the build");
    match err {
        BuildError::Redirect(RedirectError::UnknownTarget { from, to }) => {
            assert_eq!(from, "/old/");
            assert_eq!(to, "/does-not-exist/");
        }
        other => panic!("expected Redirect(UnknownTarget), got {other:?}"),
    }
    assert!(
        !out_dir.exists(),
        "out_dir must not exist when a redirect target is unknown"
    );
}

// ---- fail-closed: `from` が本体ページ path と衝突 ----

#[test]
fn build_fails_closed_when_redirect_from_collides_with_an_existing_page() {
    let temp = TempDir::new("collides");
    write_fixture_site(&temp.0);
    write_redirects_toml(&temp.0, "[[redirect]]\nfrom = \"/next/\"\nto = \"/\"\n");
    let out_dir = temp.0.join("dist");

    let err = build_site(&temp.0, &out_dir)
        .expect_err("redirect colliding with an existing page should fail the build");
    match err {
        BuildError::Redirect(RedirectError::CollidesWithPage(path)) => {
            assert_eq!(path, "/next/");
        }
        other => panic!("expected Redirect(CollidesWithPage), got {other:?}"),
    }
    assert!(
        !out_dir.exists(),
        "out_dir must not exist when a redirect collides with an existing page"
    );
}

// ---- fail-closed: `from` 重複 ----

#[test]
fn build_fails_closed_on_duplicate_from() {
    let temp = TempDir::new("duplicate-from");
    write_fixture_site(&temp.0);
    write_redirects_toml(
        &temp.0,
        r#"
[[redirect]]
from = "/old/"
to = "/"

[[redirect]]
from = "/old/"
to = "/next/"
"#,
    );
    let out_dir = temp.0.join("dist");

    let err =
        build_site(&temp.0, &out_dir).expect_err("duplicate redirect `from` should fail the build");
    assert!(matches!(
        err,
        BuildError::Redirect(RedirectError::DuplicateFrom(ref f)) if f == "/old/"
    ));
    assert!(!out_dir.exists());
}

// ---- 正常系 ----

#[test]
fn build_writes_a_redirect_page_with_all_four_required_elements() {
    let temp = TempDir::new("ok");
    write_fixture_site(&temp.0);
    write_redirects_toml(&temp.0, "[[redirect]]\nfrom = \"/old/\"\nto = \"/next/\"\n");
    let out_dir = temp.0.join("dist");

    let report = build_site(&temp.0, &out_dir).expect("valid redirect manifest should build");
    assert_eq!(report.redirects.len(), 1);
    assert_eq!(report.written.len(), 2, "本体ページ数は変わらない");

    let redirect_html_path = out_dir.join("old/index.html");
    assert!(redirect_html_path.exists());
    assert!(report.redirects.contains(&redirect_html_path));

    let html = std::fs::read_to_string(&redirect_html_path).unwrap();
    assert!(html.contains(r#"<meta http-equiv="refresh" content="0; url=/fixture-base/next/">"#));
    assert!(html.contains(r#"<link rel="canonical" href="/fixture-base/next/">"#));
    assert!(html.contains(r#"<meta name="robots" content="noindex">"#));
    assert!(html.contains(r#"<a href="/fixture-base/next/">/fixture-base/next/</a>"#));
}

// ---- base_path 反映 ----

#[test]
fn build_reflects_base_path_in_redirect_href_not_the_bare_target() {
    let temp = TempDir::new("base-path");
    write_fixture_site(&temp.0);
    write_redirects_toml(&temp.0, "[[redirect]]\nfrom = \"/old/\"\nto = \"/\"\n");
    let out_dir = temp.0.join("dist");

    build_site(&temp.0, &out_dir).expect("valid redirect manifest should build");
    let html = std::fs::read_to_string(out_dir.join("old/index.html")).unwrap();
    assert!(html.contains("/fixture-base/"));
    // 素の `to`（`base_path` 抜き）がどこにも現れないことの回帰防止。
    // `redirect_page` は `content` 属性値を非クォートの
    // `format!("0; url={to_href}")` で埋め込む（`redirect.rs` 参照）ため、
    // クォート付き `url="/` 形は実装上決して現れず、それを探す旧 assert は
    // 常に true で回帰を検知できなかった（Cursor Bugbot 指摘）。
    // `base_path` 抜きの裸の `to`（`url=/"`）がちょうど閉じクォートへ
    // 直結する形が現れないことを直接固定する
    // （`redirect.rs::redirect_page_reflects_base_path_in_href` と同一パターン）。
    assert!(!html.contains(r#"url=/""#));
    assert!(!html.contains(r#"href="/">"#));
}

// ---- redirects.toml 不在 ----

#[test]
fn build_succeeds_with_zero_redirects_when_manifest_is_absent() {
    let temp = TempDir::new("no-manifest");
    write_fixture_site(&temp.0);
    let out_dir = temp.0.join("dist");

    let report =
        build_site(&temp.0, &out_dir).expect("missing site/redirects.toml should still build");
    assert!(report.redirects.is_empty());
    assert_eq!(report.written.len(), 2);
}

// ---- 実マニフェスト ----

/// イシュー #1016 時点で `site/redirects.toml` は 1 件のみを宣言していた
/// （§2.4「未提供 URL への予防的移転案内」）。イシュー #1017 で既存 107
/// 部品ページの `/components/<kebab>/` → `/themes/<kebab>/` 移行分を追記し
/// 108 件（予防的移転案内 1 + 部品ページ移行 107）。イシュー #1018 で
/// `/components/pre-styled-ui/`（索引ページの旧 URL）→ `/themes/` の
/// リダイレクトを追加し、109 件になった。イシュー #1683 で Collapsible の
/// Themes ページ新設に伴い `/components/collapsible/` を追加し、現在は
/// 110 件。本値の更新が要る変更は fail-closed に検知する（黙って増減しても
/// 気付けるようにする意図。`tests/site_build.rs` のページ数固定と同型）。
#[test]
fn real_redirects_manifest_parses_and_validates_against_the_real_nav() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve repository root");

    let manifest_path = repo_root.join(redirect::MANIFEST_REL_PATH);
    let input = std::fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", manifest_path.display()));
    let redirects =
        redirect::parse_redirects(&input).expect("site/redirects.toml should parse cleanly");
    assert_eq!(
        redirects.entries.len(),
        110,
        "site/redirects.toml の宣言件数が期待値と異なる: {:?}",
        redirects.entries
    );

    let nav_input = std::fs::read_to_string(repo_root.join("site/nav.toml"))
        .expect("site/nav.toml should be readable");
    let nav =
        fandhe_frontend_docs_site::nav::parse_nav(&nav_input).expect("site/nav.toml should parse");
    redirect::validate_against_nav(&redirects, &nav)
        .expect("real site/redirects.toml should validate against the real site/nav.toml");
}

/// イシュー #1017 受け入れ条件（実装計画 §4.7）: 107 件の機械生成された
/// リダイレクト宣言は「対応関係の取り違え（ずれ 1 件）」という現実的な
/// 失敗モードを持つ。件数一致（上記テスト）と `validate_against_nav`
/// （`to` の実在確認のみ）だけでは `/components/button/` →
/// `/themes/card/` のような取り違えを検出できないため、nav 側の
/// `site/themes/` 部品ページと `redirects.toml` の宣言を `source`
/// （`api_component_cross_links.rs` / `primitives_catalog.rs` の先例に
/// 合わせ、URL 移転に耐えるキーとして `path` ではなく `source` を使う）
/// で双方向に突合する。
#[test]
fn every_themes_page_has_exactly_one_matching_components_redirect() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve repository root");

    let manifest_path = repo_root.join(redirect::MANIFEST_REL_PATH);
    let input = std::fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", manifest_path.display()));
    let redirects =
        redirect::parse_redirects(&input).expect("site/redirects.toml should parse cleanly");

    let nav_input = std::fs::read_to_string(repo_root.join("site/nav.toml"))
        .expect("site/nav.toml should be readable");
    let nav =
        fandhe_frontend_docs_site::nav::parse_nav(&nav_input).expect("site/nav.toml should parse");

    // nav 側: `site/themes/<kebab>.md` を source に持つ全ページの
    // (kebab, path) 集合（108 件、イシュー #1683 で 107 → 108）。
    let themes_pages: Vec<(String, String)> = nav
        .all_pages()
        .filter_map(|p| {
            p.source
                .strip_prefix("site/themes/")
                .and_then(|rest| rest.strip_suffix(".md"))
                .map(|kebab| (kebab.to_string(), p.path.clone()))
        })
        .collect();
    assert_eq!(
        themes_pages.len(),
        108,
        "expected 108 site/themes/ pages, got {}: {themes_pages:?}",
        themes_pages.len()
    );

    // 順方向: 各 themes ページに対し `from == "/components/<kebab>/"` かつ
    // `to == page.path` の宣言がちょうど 1 件存在する。
    for (kebab, path) in &themes_pages {
        let expected_from = format!("/components/{kebab}/");
        let matches: Vec<_> = redirects
            .entries
            .iter()
            .filter(|e| e.from == expected_from)
            .collect();
        assert_eq!(
            matches.len(),
            1,
            "expected exactly one redirect declaration with from={expected_from:?}, got {}: {matches:?}",
            matches.len()
        );
        assert_eq!(
            &matches[0].to, path,
            "redirect from={expected_from:?} should point to {path:?} (nav page.path), got {:?}",
            matches[0].to
        );
    }

    // 逆方向: `/components/<kebab>/` 形式の宣言のうち、上記 themes 集合に
    // 属さないものが存在しない（部品ページではなく索引扱いの 2 件のみが
    // 例外として許容される。形状ベースで緩めず、名前付きの明示的
    // allowlist として固定する。イシュー #1018 で `/components/pre-styled-ui/`
    // を追加）。
    const NON_PART_COMPONENT_FROMS: [&str; 2] = ["/components/", "/components/pre-styled-ui/"];
    let expected_froms: std::collections::BTreeSet<String> = themes_pages
        .iter()
        .map(|(kebab, _)| format!("/components/{kebab}/"))
        .collect();
    for entry in &redirects.entries {
        if NON_PART_COMPONENT_FROMS.contains(&entry.from.as_str()) {
            continue;
        }
        let is_component_kebab_shape = entry.from.starts_with("/components/")
            && entry.from != "/components/"
            && entry.from.ends_with('/');
        if !is_component_kebab_shape {
            continue;
        }
        assert!(
            expected_froms.contains(&entry.from),
            "redirect from={:?} does not correspond to any site/themes/ page \
             (取り違え、または nav 側の kebab とのずれの可能性)",
            entry.from
        );
    }

    // 肯定形: NON_PART_COMPONENT_FROMS の各 from がちょうど 1 件存在し、
    // いずれも `/themes/`（Themes 索引）へ向いていることを固定する
    // （形状ベースの一律スキップに緩めないための対となる検証、
    // イシュー #1018）。
    for from in NON_PART_COMPONENT_FROMS {
        let matches: Vec<_> = redirects
            .entries
            .iter()
            .filter(|e| e.from == from)
            .collect();
        assert_eq!(
            matches.len(),
            1,
            "expected exactly one redirect declaration with from={from:?}, got {}: {matches:?}",
            matches.len()
        );
        assert_eq!(
            matches[0].to, "/themes/",
            "redirect from={from:?} should point to /themes/, got {:?}",
            matches[0].to
        );
    }
}
