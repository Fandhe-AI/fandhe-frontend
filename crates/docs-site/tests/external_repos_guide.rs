//! 外部リポジトリ向け利用ガイド（`docs/guides/docs-site-external-repos.md`、
//! イシュー #3726）のドリフト検知テスト。
//!
//! ガイドの nav 登録（AC1）・最小構成例が現行の `parse_nav` に通ること・帰属表記と
//! ライセンスの明記（AC2）・setup-github-pages への導線を固定する。実サイトの
//! ビルドは使わず、`CARGO_MANIFEST_DIR` 起点の固定パスだけを読む軽量テスト。

use std::fs;
use std::path::PathBuf;

use fandhe_frontend_docs_site::nav::parse_nav;

const GUIDE_SOURCE: &str = "docs/guides/docs-site-external-repos.md";
const GUIDE_PATH: &str = "/guides/docs-site-external-repos/";
const SKILL_URL: &str =
    "https://github.com/Fandhe-AI/agent-util-skills/tree/main/skills/setup-github-pages";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn guide() -> String {
    fs::read_to_string(repo_root().join(GUIDE_SOURCE)).expect("guide must be readable")
}

/// ガイドが Guides セクションからナビ登録されていること（受け入れ条件 1）。
#[test]
fn guide_is_registered_under_guides() {
    let raw = fs::read_to_string(repo_root().join("site/nav.toml")).expect("nav.toml readable");
    let nav = parse_nav(&raw).expect("nav.toml must parse");
    let guides = nav
        .sections
        .iter()
        .find(|s| s.title == "Guides")
        .expect("Guides section must exist");
    assert!(
        guides
            .all_pages()
            .any(|p| p.source == GUIDE_SOURCE && p.path == GUIDE_PATH),
        "guide is not registered under Guides"
    );
}

/// 見出し `## 最小構成の例` 直後の `toml` フェンスを `parse_nav` に通す。
/// この見出し名は本テストとの契約なので、ガイド側で変えない。
#[test]
fn sample_nav_parses_and_sets_every_optional_site_key() {
    let text = guide();
    let after = text
        .split("### 最小構成の例")
        .nth(1)
        .expect("heading `### 最小構成の例` must exist");
    let fence = after
        .split("```toml\n")
        .nth(1)
        .expect("toml fence must follow the heading")
        .split("```")
        .next()
        .expect("toml fence must be closed");
    let nav = parse_nav(fence).expect("sample nav must parse");
    let s = &nav.site;
    assert!(s.brand.is_some());
    assert!(s.repository_url.is_some());
    assert!(s.tagline.is_some());
    assert!(s.copyright.is_some());
    assert!(s.version_badge.is_some());
    assert!(s.lang.is_some());
    assert!(s.brand_mark.is_some());
    assert!(s.brand_color.is_some());
    assert!(s.is_brand_customized());
}

/// 帰属表記とライセンスの扱いが明記されていること（受け入れ条件 2）。
#[test]
fn guide_states_attribution_and_license() {
    let text = guide();
    for needle in [
        "Built with fandhe-frontend docs-site",
        "MIT OR Apache-2.0",
        "LICENSE-MIT",
        "LICENSE-APACHE",
    ] {
        assert!(text.contains(needle), "guide must mention `{needle}`");
    }
}

/// 必須フラグと setup-github-pages への導線が残っていること。
#[test]
fn guide_mentions_flag_and_skill() {
    let text = guide();
    assert!(text.contains("--no-page-sections"));
    assert!(text.contains(SKILL_URL));
}
