//! `data-scope` 名が接頭辞包含関係にある scope ペア（例:
//! `image`/`image-cropper`）について、短い方の scope の部品ページに長い方
//! の scope の CSS 変数が誤って列挙されない（クロスページ漏れしない）こと
//! を全 scope 横断で契約テスト化する（イシュー #1061、PR #1088 への
//! Bugbot Medium 指摘）。
//!
//! # 背景
//!
//! `crates/docs-site/src/component_page.rs::collect_css_vars_for_scope` は
//! `showcase::stylesheet()` が返す全部品集約 CSS を `var(--fandhe-{scope}-*)`
//! で走査して `/themes/<kebab>/` の「CSS Variables」表を機械導出する。
//! かつての実装は単純な前方一致（`name.starts_with("--fandhe-{scope}-")`）
//! だったため、`image` と `image-cropper` のように scope 名が接頭辞包含
//! 関係にあるペアでは、短い方の scope（`image`）のページに長い方の scope
//! （`image-cropper`）の変数が誤って列挙されていた
//! （`crates/docs-site/tests/css_var_scope_prefix.rs` の契約はルール内の
//! scope 整合〔ある変数がそのルールの `data-scope` 由来のものであるか〕
//! しか見ておらず、この「ページ単位でどの scope 名の表に出るか」という
//! クロスページ漏れは検知できない）。
//!
//! 本テストは特定 2 部品のペアをハードコードするのではなく、集約 CSS 中に
//! 実在する全 `data-scope` を機械収集し、接頭辞包含関係にあるペアを
//! 総当たりで検出したうえで、短い方の scope ページの生成 HTML に長い方の
//! scope 専有の CSS 変数名が出現しないことを確認する（一般化した回帰
//! 検知、計画 §4「他に同様の接頭辞包含関係にある scope ペアが存在しないか
//! 洗い出し」）。

use std::collections::BTreeSet;

use fandhe_frontend_docs_site::{component_page, showcase};

/// 集約 CSS 中の `data-scope="..."` をすべて収集する
/// （`css_var_scope_prefix.rs::collect_scopes` と同じ素の文字列走査方針）。
fn collect_scopes(css: &str) -> BTreeSet<String> {
    let mut scopes = BTreeSet::new();
    let marker = "data-scope=\"";
    let mut search_from = 0usize;
    while let Some(rel) = css[search_from..].find(marker) {
        let start = search_from + rel + marker.len();
        let Some(end_rel) = css[start..].find('"') else {
            break;
        };
        scopes.insert(css[start..start + end_rel].to_string());
        search_from = start + end_rel + 1;
    }
    scopes
}

/// 集約 CSS 中で `--fandhe-{scope}-` に前方一致する `var(...)` の変数名を
/// すべて収集する（`--fandhe-{scope}` 完全一致は対象外。CSS 変数表の
/// エントリ名そのものであり、`scope` 専有の名前空間を表す）。
fn collect_var_names_with_prefix(css: &str, scope: &str) -> BTreeSet<String> {
    let prefix = format!("--fandhe-{scope}-");
    let mut names = BTreeSet::new();
    let marker = "var(";
    let mut search_from = 0usize;
    while let Some(rel) = css[search_from..].find(marker) {
        let var_start = search_from + rel + marker.len();
        let rest = &css[var_start..];
        let name_end = rest.find([',', ')']).unwrap_or(rest.len());
        let name = rest[..name_end].trim();
        if name.starts_with(&prefix) {
            names.insert(name.to_string());
        }
        search_from = var_start + name_end.max(1);
    }
    names
}

/// 本テストの入力 CSS（全部品集約、`showcase::stylesheet()`）を組み立てる。
fn aggregated_css() -> String {
    showcase::stylesheet()
        .expect("showcase::stylesheet() should build without StylesheetError")
        .as_css()
        .to_string()
}

/// 接頭辞包含関係にある scope ペア（`short` は `long` の前方一致接頭辞
/// `"{short}-"`）を、集約 CSS 中に実在する全 scope から総当たりで検出する。
fn prefix_included_scope_pairs(scopes: &BTreeSet<String>) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for short in scopes {
        let prefix = format!("{short}-");
        for long in scopes {
            if long != short && long.starts_with(&prefix) {
                pairs.push((short.clone(), long.clone()));
            }
        }
    }
    pairs
}

/// 契約本体: 接頭辞包含関係にある scope ペアのうち、長い方の scope が
/// 自身専有の CSS 変数を実際に持つ場合、短い方の scope ページの生成 HTML
/// にその変数名が一切出現しないこと。
///
/// 実装時点（イシュー #1061 是正後）で以下のペアが該当する:
/// `checkbox`/`checkbox-card`、`checkbox`/`checkbox-group`、
/// `image`/`image-cropper`、`toggle`/`toggle-group`。
/// 修正前の実装（単純前方一致）ではこれらすべてで短い方のページに長い方の
/// 変数が漏れて出現し、本テストは失敗する。
#[test]
fn short_scope_page_does_not_leak_longer_scope_css_vars() {
    let css = aggregated_css();
    let scopes = collect_scopes(&css);
    let pairs = prefix_included_scope_pairs(&scopes);
    assert!(
        !pairs.is_empty(),
        "接頭辞包含関係にある scope ペアが 1 件も検出できていない（検出ロジックの回帰を疑う）"
    );

    let mut checked_pairs_with_leakage_risk = 0usize;

    for (short, long) in &pairs {
        let long_only_vars = collect_var_names_with_prefix(&css, long);
        if long_only_vars.is_empty() {
            // long scope 自身が CSS 変数を持たない場合、そもそも漏れようが
            // ないためスキップ（例: link-overlay が現時点で専有変数を
            // 持たない場合）。
            continue;
        }
        checked_pairs_with_leakage_risk += 1;

        let short_path = format!("/themes/{short}/");
        let page = component_page::generated_content(&short_path).unwrap_or_else(|| {
            panic!(
                "`{short_path}` が登録済み部品ページとして見つからない \
                 （scope 名と /themes/<kebab>/ パスの対応が崩れている疑い）"
            )
        });
        let html = fandhe_frontend_core::render(&page);

        for var_name in &long_only_vars {
            assert!(
                !html.contains(var_name.as_str()),
                "scope `{short}` のページ（{short_path}）に scope `{long}` \
                 専有の CSS 変数 `{var_name}` が漏れて出現している \
                 （collect_css_vars_for_scope の前方一致境界のバグ、イシュー #1061）"
            );
        }
    }

    assert!(
        checked_pairs_with_leakage_risk > 0,
        "漏洩リスクのある scope ペア（長い方が専有 CSS 変数を持つペア）が \
         1 件も検証されていない（`image`/`image-cropper` 等の既知ペアが \
         見つかるはずであり、検出ロジックまたはフィクスチャの回帰を疑う）"
    );
}
