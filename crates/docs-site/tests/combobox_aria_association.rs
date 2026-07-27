//! combobox / listbox の ARIA 関連付け契約（イシュー #1067）を、実出荷
//! マークアップ（Primitives 全 63 ページ + Themes 全 107 ページの生成
//! HTML）へ適用する回帰テスト。
//!
//! `crates/headless-ui/src/combobox.rs` の `input()`/`trigger()` は
//! `aria-controls`/`aria-activedescendant` を `Option` の opt-in として
//! 受け、`None` を渡しても型では検知できない。`crates/headless-ui/src/listbox.rs`
//! の `content()` の `labelledby` も同様に opt-in である。本ファイルは
//! `crates/headless-ui/tests/combobox.rs::verify_combobox_aria_association`
//! / `crates/headless-ui/tests/listbox.rs::verify_listbox_has_accessible_name`
//! と**同一規則の意図的な重複実装**を、docs-site が実際に生成する全ページ
//! （`crate::component_page::generated_content`、Primitives/Themes 両層を
//! 自動判定して合成する、`crates/docs-site/tests/component_pages.rs` と
//! 同じ公開エントリポイント）へ適用する。クレート境界を跨ぐ test helper
//! 共有は Rust の統合テストでは不可能なため、検証ロジックは 3 箇所
//! （`crates/headless-ui/tests/combobox.rs`・`crates/headless-ui/tests/listbox.rs`・
//! 本ファイル）で重複する。**規則を変更するときは 3 箇所すべてを更新
//! すること。**
//!
//! イシュー #1067 計画時点の実測（リポジトリ内 combobox/listbox 呼び出し
//! 全数調査）では、Primitives/Themes の全呼び出しが本契約に準拠している
//! （`docs/internal/headless-ui-implementation-notes.md` 参照）。本テストは
//! 既存欠陥の摘発ではなく**回帰防止**が目的であり、mutation テスト（一時的に
//! `Some`→`None` へ書き換えて赤くなることを確認する手順、イシュー #1067
//! 計画 §6）で検知力を確認済み（コミットには含めない一時検証）。

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::{component_page, primitives_catalog, showcase};

/// 開始タグ内容 `tag` から属性 `name` の値を取り出す
/// （`crates/headless-ui/tests/combobox.rs` の `attr` と同一実装。属性値
/// 中に生の `"` が現れないという既定エスケープの不変条件に依拠する）。
fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!(" {name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

/// HTML 断片から開始タグの内容を「どの combobox インスタンスの部分木に
/// 属するか」（`scope`）付きで登場順に切り出す
/// （`crates/headless-ui/tests/combobox.rs` の `scoped_open_tags` と同一
/// 実装）。1 ページに複数の combobox インスタンスが共存する構成（Demo +
/// Examples 原稿断片が同一ページへ合成される docs-site の実出荷ページ、
/// 実測: `/primitives/combobox/` に開いた本体 combobox（ハイライト item
/// あり）と閉じた Examples 用 combobox（ハイライト item なし）の 2
/// インスタンスが共存する）で、一方のハイライト item がもう一方の R3
/// 判定へ誤って波及しないよう、[`combobox::root`] が出力する
/// `data-scope="combobox" data-part="root"` マーカーで部分木ごとに区切る。
fn scoped_open_tags(html: &str) -> Vec<(&str, Option<usize>)> {
    let mut tags = Vec::new();
    let mut stack: Vec<Option<usize>> = Vec::new();
    let mut next_scope = 0usize;
    let mut rest = html;
    while let Some(lt) = rest.find('<') {
        let after_lt = &rest[lt + 1..];
        if after_lt.starts_with('/') {
            match after_lt.find('>') {
                Some(gt) => {
                    stack.pop();
                    rest = &after_lt[gt + 1..];
                    continue;
                }
                None => break,
            }
        }
        match after_lt.find('>') {
            Some(gt) => {
                let tag = &after_lt[..gt];
                let parent_scope = stack.last().copied().flatten();
                let own_scope = if attr(tag, "data-scope") == Some("combobox")
                    && attr(tag, "data-part") == Some("root")
                {
                    next_scope += 1;
                    Some(next_scope)
                } else {
                    parent_scope
                };
                tags.push((tag, own_scope));
                stack.push(own_scope);
                rest = &after_lt[gt + 1..];
            }
            None => break,
        }
    }
    tags
}

/// combobox の ARIA 関連付け規則 R1〜R4（`crates/headless-ui/tests/combobox.rs`
/// の `verify_combobox_aria_association` と同一規則。R3 のハイライト item
/// 集合は combobox インスタンス〔[`scoped_open_tags`]〕単位で構築し、R2/R4
/// の `id` 参照実在確認はページ全体〔`id` の一意性が前提〕で行う）。
fn verify_combobox_aria_association(html: &str) -> Result<(), String> {
    let scoped_tags = scoped_open_tags(html);

    let mut id_role: std::collections::HashMap<&str, Option<&str>> =
        std::collections::HashMap::new();
    for (tag, _) in &scoped_tags {
        if let Some(id) = attr(tag, "id") {
            id_role.insert(id, attr(tag, "role"));
        }
    }

    let mut highlighted_ids_by_scope: std::collections::HashMap<Option<usize>, Vec<&str>> =
        std::collections::HashMap::new();
    let mut highlighted_without_id_scopes: std::collections::HashSet<Option<usize>> =
        std::collections::HashSet::new();
    for (tag, scope) in &scoped_tags {
        if attr(tag, "role") == Some("option") && attr(tag, "data-highlighted").is_some() {
            match attr(tag, "id") {
                Some(id) => highlighted_ids_by_scope.entry(*scope).or_default().push(id),
                None => {
                    highlighted_without_id_scopes.insert(*scope);
                }
            }
        }
    }

    for (tag, scope) in &scoped_tags {
        if attr(tag, "role") != Some("combobox") {
            continue;
        }
        let expanded = attr(tag, "aria-expanded") == Some("true");
        let controls = attr(tag, "aria-controls");
        let activedescendant = attr(tag, "aria-activedescendant");

        if expanded && controls.is_none() {
            return Err(format!(
                "R1 violation: role=\"combobox\" element with aria-expanded=\"true\" lacks aria-controls: <{tag}>"
            ));
        }

        if let Some(target) = controls {
            match id_role.get(target) {
                Some(Some("listbox")) => {}
                Some(_) => {
                    return Err(format!(
                        "R2 violation: aria-controls=\"{target}\" target lacks role=\"listbox\""
                    ));
                }
                None => {
                    return Err(format!(
                        "R2 violation: aria-controls=\"{target}\" target id does not exist (dangling IDREF)"
                    ));
                }
            }
        }

        if highlighted_without_id_scopes.contains(scope) {
            return Err(
                "R3 violation: highlighted role=\"option\" element lacks id (cannot be referenced by aria-activedescendant)"
                    .to_string(),
            );
        }
        if let Some(highlighted_ids) = highlighted_ids_by_scope.get(scope) {
            if !highlighted_ids.is_empty() {
                match activedescendant {
                    None => {
                        return Err(
                            "R3 violation: highlighted option exists but combobox lacks aria-activedescendant"
                                .to_string(),
                        );
                    }
                    Some(target) => {
                        if !highlighted_ids.contains(&target) {
                            return Err(format!(
                                "R3 violation: aria-activedescendant=\"{target}\" does not match any highlighted option id {highlighted_ids:?}"
                            ));
                        }
                    }
                }
            }
        }

        if let Some(target) = activedescendant {
            if !id_role.contains_key(target) {
                return Err(format!(
                    "R4 violation: aria-activedescendant=\"{target}\" target id does not exist (dangling IDREF)"
                ));
            }
        }
    }

    Ok(())
}

/// listbox のアクセシブルネーム経路の契約（`crates/headless-ui/tests/listbox.rs`
/// の `verify_listbox_has_accessible_name` と同一規則）。
fn verify_listbox_has_accessible_name(html: &str) -> Result<(), String> {
    let tags = scoped_open_tags(html);

    let mut ids: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for (tag, _) in &tags {
        if let Some(id) = attr(tag, "id") {
            ids.insert(id);
        }
    }

    for (tag, _) in &tags {
        if attr(tag, "role") != Some("listbox") {
            continue;
        }
        let labelledby = attr(tag, "aria-labelledby");
        let label = attr(tag, "aria-label");

        match (labelledby, label) {
            (Some(target), _) if ids.contains(target) => {}
            (_, Some(_)) => {}
            (Some(target), None) => {
                return Err(format!(
                    "naming violation: aria-labelledby=\"{target}\" target id does not exist (dangling IDREF) and no aria-label fallback: <{tag}>"
                ));
            }
            (None, None) => {
                return Err(format!(
                    "naming violation: role=\"listbox\" element has neither aria-labelledby nor aria-label: <{tag}>"
                ));
            }
        }
    }

    Ok(())
}

/// 対象ページ path（Themes + Primitives、`nav.toml` 登録の正規レジストリ
/// 経由）を全走査する共通イテレータ。
fn all_page_paths() -> impl Iterator<Item = &'static str> {
    showcase::component_page_paths().chain(primitives_catalog::page_paths())
}

/// Primitives 全 63 ページ + Themes 全 107 ページの生成 HTML に R1〜R4
/// （combobox の ARIA 関連付け）を適用する。将来ページが増えても
/// `all_page_paths()` を通じて自動的に対象へ含まれる（combobox ページ
/// 限定ではなく全ページ走査、イシュー #1067 計画 §4）。
#[test]
fn all_pages_satisfy_combobox_aria_association_contract() {
    for path in all_page_paths() {
        let content = component_page::generated_content(path)
            .unwrap_or_else(|| panic!("{path} must have generated content"));
        let html = render(&content);
        if let Err(reason) = verify_combobox_aria_association(&html) {
            panic!("{path}: combobox ARIA association contract violated: {reason}");
        }
    }
}

/// Primitives 全 63 ページ + Themes 全 107 ページの生成 HTML に listbox
/// のアクセシブルネーム契約を適用する。
#[test]
fn all_pages_satisfy_listbox_accessible_name_contract() {
    for path in all_page_paths() {
        let content = component_page::generated_content(path)
            .unwrap_or_else(|| panic!("{path} must have generated content"));
        let html = render(&content);
        if let Err(reason) = verify_listbox_has_accessible_name(&html) {
            panic!("{path}: listbox accessible name contract violated: {reason}");
        }
    }
}
