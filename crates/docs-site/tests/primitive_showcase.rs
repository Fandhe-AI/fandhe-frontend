//! `crate::primitive_showcase`（イシュー #1022）の台帳突合・Anatomy 網羅・
//! scope 一致を検証する統合テスト。
//!
//! 台帳との集合完全一致（[`primitive_pages_match_the_catalog_exactly`]）・
//! 全ページの Demo/Anatomy 描画（[`all_primitive_pages_render_demo_and_anatomy`]）・
//! scope の取り違え検知（[`resolved_scope_matches_the_page_kebab_for_every_entry`]）・
//! headless-ui ソース全走査によるパート網羅の双方向 fail-closed 固定
//! （[`anatomy_coverage_matches_known_uncovered_exactly`]）を担う。

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::{component_page, primitive_showcase, primitives_catalog};

/// `CARGO_MANIFEST_DIR`（`crates/docs-site`）から repo_root を解決する
/// （`tests/primitives_catalog.rs` 等と同じ規約）。
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo_root should resolve from CARGO_MANIFEST_DIR")
}

/// [`scan_part_names`] のソース走査（`.part("literal", ...)`）では拾えない
/// 「`data-part` 値を計算した変数経由で `ANATOMY.part(part, ...)` へ渡す」
/// モジュールの追加パート名（実測: `grep -rn 'ANATOMY\.part([a-z_]'
/// crates/headless-ui/src/*.rs` で該当 2 モジュールのみと確認済み）。
/// `color_picker::Channel::parts()` の 4 チャンネル×3 パーツ、
/// `pagination::trigger_part`/`tag_part_name` の prev/next トリガーと
/// button/link 共通の `item` パート。
const DYNAMIC_PART_NAMES: &[(&str, &[&str])] = &[
    (
        "color_picker",
        &[
            "hue-slider",
            "hue-slider-track",
            "hue-slider-thumb",
            "saturation-slider",
            "saturation-slider-track",
            "saturation-slider-thumb",
            "value-slider",
            "value-slider-track",
            "value-slider-thumb",
            "alpha-slider",
            "alpha-slider-track",
            "alpha-slider-thumb",
        ],
    ),
    ("pagination", &["prev-trigger", "next-trigger", "item"]),
];

/// headless-ui ソース `crates/headless-ui/src/<module>.rs` 中の
/// `ANATOMY.part("<name>", ...)` 呼び出しから `<name>` を抽出する
/// （自由関数・struct メソッドいずれの呼び出し形でも同一ファイル内である
/// ため検出できる。`primitives_catalog.rs::scan_headless_ui_src` と同じ
/// 「ソース全走査でコードとの乖離を機械検知する」方針）。[`DYNAMIC_PART_NAMES`]
/// で計算済み `data-part` 値のモジュールを補完する。
fn scan_part_names(module: &str) -> BTreeSet<String> {
    let path = repo_root()
        .join("crates/headless-ui/src")
        .join(format!("{module}.rs"));
    let src = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("headless-ui source should be readable: {path:?}: {e}"));
    let mut names = BTreeSet::new();
    let marker = ".part(\"";
    let mut rest = src.as_str();
    while let Some(idx) = rest.find(marker) {
        let after = &rest[idx + marker.len()..];
        if let Some(end) = after.find('"') {
            names.insert(after[..end].to_string());
            rest = &after[end..];
        } else {
            break;
        }
    }
    if let Some((_, extra)) = DYNAMIC_PART_NAMES.iter().find(|(m, _)| *m == module) {
        names.extend(extra.iter().map(|s| s.to_string()));
    }
    names
}

/// 生成 HTML の `<h2>Anatomy</h2>` 直後の `<pre><code>...</code></pre>`
/// からパート名の集合を抽出する（`component_page::anatomy_section` の
/// インデント表現をそのまま剥がす。各行は `"  ".repeat(depth) + name`）。
fn extract_anatomy_parts(html: &str) -> BTreeSet<String> {
    let Some(after_heading) = html.split("<h2>Anatomy</h2>").nth(1) else {
        return BTreeSet::new();
    };
    let Some(code_start) = after_heading.find("<code>") else {
        return BTreeSet::new();
    };
    let after_code = &after_heading[code_start + "<code>".len()..];
    let Some(code_end) = after_code.find("</code>") else {
        return BTreeSet::new();
    };
    after_code[..code_end]
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

/// パート網羅の意図的な例外（module, part, 理由）。
///
/// 双方向 fail-closed（[`anatomy_coverage_matches_known_uncovered_exactly`]）:
/// - ここに載っていない未網羅パートがあれば FAIL（登録漏れの検知）
/// - ここに載っているのに実は網羅済みのパートがあれば FAIL（形骸化した
///   免除の検知。デモを充実させたら対応するエントリを削除する契約）
///
/// 除外フラグ・ワイルドカード除外は作らない（A05、`security.md` 参照）。
const KNOWN_UNCOVERED: &[(&str, &str, &str)] = &[
    (
        "menu",
        "trigger-item",
        "サブメニューを持たないデモ構成のため trigger_item を使用しない（他パートで menu scope の網羅要件は満たす）",
    ),
    (
        "menu",
        "checkbox-item",
        "CheckboxItem 変種は同一 scope 内の item と役割が重複するため未使用",
    ),
    (
        "menu",
        "radio-item-group",
        "RadioItemGroup 変種は同一 scope 内の item_group と役割が重複するため未使用",
    ),
    (
        "menu",
        "radio-item",
        "RadioItem 変種は同一 scope 内の item と役割が重複するため未使用",
    ),
    (
        "menubar",
        "sub-trigger",
        "サブメニューを持たないデモ構成のため sub_trigger/sub_content を使用しない",
    ),
    (
        "menubar",
        "sub-content",
        "サブメニューを持たないデモ構成のため sub_trigger/sub_content を使用しない",
    ),
    (
        "menu",
        "context-trigger",
        "右クリックコンテキストメニュー変種は本デモでは未使用（trigger で開閉契約を示す）",
    ),
    (
        "breadcrumb",
        "ellipsis",
        "3 項目のみのデモ構成のため折り畳み表現（ellipsis）を使用しない",
    ),
    (
        "progress",
        "circle",
        "circular variant（circle/circle-track/circle-range）は linear variant と役割が重複するため未使用",
    ),
    (
        "progress",
        "circle-track",
        "circular variant（circle/circle-track/circle-range）は linear variant と役割が重複するため未使用",
    ),
    (
        "progress",
        "circle-range",
        "circular variant（circle/circle-track/circle-range）は linear variant と役割が重複するため未使用",
    ),
    (
        "color_picker",
        "saturation-slider",
        "4 チャンネル（Hue/Saturation/Value/Alpha）のうち Hue のみをデモし、残り 3 チャンネルは同型の繰り返しのため省略",
    ),
    (
        "color_picker",
        "saturation-slider-track",
        "同上（Saturation チャンネル省略）",
    ),
    (
        "color_picker",
        "saturation-slider-thumb",
        "同上（Saturation チャンネル省略）",
    ),
    (
        "color_picker",
        "value-slider",
        "同上（Value チャンネル省略）",
    ),
    (
        "color_picker",
        "value-slider-track",
        "同上（Value チャンネル省略）",
    ),
    (
        "color_picker",
        "value-slider-thumb",
        "同上（Value チャンネル省略）",
    ),
    (
        "color_picker",
        "alpha-slider",
        "同上（Alpha チャンネル省略）",
    ),
    (
        "color_picker",
        "alpha-slider-track",
        "同上（Alpha チャンネル省略）",
    ),
    (
        "color_picker",
        "alpha-slider-thumb",
        "同上（Alpha チャンネル省略）",
    ),
];

/// 台帳（`primitives_catalog::PRIMITIVES`）と Demo レジストリ
/// （`primitive_showcase::page_paths`）の path 集合が完全一致することを
/// 固定する（過不足どちらも fail-closed）。
#[test]
fn primitive_pages_match_the_catalog_exactly() {
    let catalog: BTreeSet<&str> = primitives_catalog::page_paths().collect();
    let demo: BTreeSet<&str> = primitive_showcase::page_paths().collect();
    assert_eq!(
        catalog, demo,
        "primitives_catalog と primitive_showcase の path 集合が一致しない"
    );
}

/// 63 ページすべてが Demo（`<h2>Demo</h2>`）・Anatomy（1 パート以上）を
/// 持つことを固定する（受け入れ条件 1・2）。CSS Variables 節は Primitives
/// 層で恒常的に省略されるため出現しないことも併せて確認する。
#[test]
fn all_primitive_pages_render_demo_and_anatomy() {
    for path in primitives_catalog::page_paths() {
        let content = component_page::generated_content(path)
            .unwrap_or_else(|| panic!("{path} must have generated content"));
        let html = render(&content);
        assert!(
            html.contains("<h2>Demo</h2>"),
            "{path}: Demo 節が無い: {html}"
        );
        let parts = extract_anatomy_parts(&html);
        assert!(
            !parts.is_empty(),
            "{path}: Anatomy 節が 1 パート以上を持たない: {html}"
        );
        assert!(
            !html.contains("CSS Variables"),
            "{path}: Primitives 層で CSS Variables 節が恒常的に省略される契約に反する"
        );
        assert!(
            html.contains(r#"class="primitives-showcase""#),
            "{path}: primitives-showcase ラッパ class が無い"
        );
    }
}

/// scope 取り違えの fail-closed 固定（デモ執筆規約 1）。
///
/// `component_page::resolve_anatomy_scope` は第一候補（path 末尾の kebab）が
/// デモ内に出現すればそれを採用するため、各ページの Anatomy 節に出現する
/// パート名集合が「対象モジュール自身が定義するパート名の集合
/// （[`scan_part_names`]）の部分集合」であることを確認する。無関係な別
/// scope の部分木が最も外側に来てフォールバックされた場合、パート名集合は
/// 一般に対象モジュール自身の定義と一致しない（`root`/`trigger`/`content`
/// 等の頻出パート名の単純衝突を除き、実質的に scope 取り違えを検知できる）。
#[test]
fn resolved_scope_matches_the_page_kebab_for_every_entry() {
    for entry in primitives_catalog::entries() {
        let content = component_page::generated_content(entry.path)
            .unwrap_or_else(|| panic!("{} must have generated content", entry.path));
        let html = render(&content);
        let observed = extract_anatomy_parts(&html);
        let expected_pool = scan_part_names(entry.module);
        let unexpected: Vec<&String> = observed.difference(&expected_pool).collect();
        assert!(
            unexpected.is_empty(),
            "{}: Anatomy に {}（module `{}`）が定義しないパートが出現した（scope 取り違えの疑い）: {:?}",
            entry.path,
            entry.module,
            entry.module,
            unexpected
        );
    }
}

/// headless-ui ソース全走査によるパート網羅を双方向 fail-closed に固定する
/// （デモ執筆規約 2、`KNOWN_UNCOVERED` の運用規約はモジュール doc参照）。
#[test]
fn anatomy_coverage_matches_known_uncovered_exactly() {
    let known_uncovered: BTreeSet<(&str, &str)> = KNOWN_UNCOVERED
        .iter()
        .map(|(module, part, _reason)| (*module, *part))
        .collect();

    let mut actual_uncovered: BTreeSet<(String, String)> = BTreeSet::new();
    let mut covered_but_listed: Vec<(String, String)> = Vec::new();

    for entry in primitives_catalog::entries() {
        let content = component_page::generated_content(entry.path)
            .unwrap_or_else(|| panic!("{} must have generated content", entry.path));
        let html = render(&content);
        let observed = extract_anatomy_parts(&html);
        let expected_pool = scan_part_names(entry.module);

        for part in expected_pool.difference(&observed) {
            let key = (entry.module.to_string(), part.clone());
            if known_uncovered.contains(&(entry.module, part.as_str())) {
                // 想定通りの未網羅（KNOWN_UNCOVERED に登録済み）。
            } else {
                actual_uncovered.insert(key);
            }
        }
    }

    for (module, part) in &known_uncovered {
        // 台帳に載っている module に対応するページで実際に走査し、既に
        // 網羅済み（KNOWN_UNCOVERED が形骸化）になっていないか確認する。
        let entry = primitives_catalog::find(module)
            .unwrap_or_else(|| panic!("KNOWN_UNCOVERED module `{module}` not found in catalog"));
        let content = component_page::generated_content(entry.path)
            .unwrap_or_else(|| panic!("{} must have generated content", entry.path));
        let html = render(&content);
        let observed = extract_anatomy_parts(&html);
        if observed.contains(*part) {
            covered_but_listed.push((module.to_string(), part.to_string()));
        }
    }

    assert!(
        actual_uncovered.is_empty(),
        "KNOWN_UNCOVERED に未登録の未網羅パートがある（登録漏れ）: {actual_uncovered:?}"
    );
    assert!(
        covered_but_listed.is_empty(),
        "KNOWN_UNCOVERED に登録済みだが実は網羅済みのパートがある（形骸化した免除、エントリを削除すること）: {covered_but_listed:?}"
    );
}

/// [`KNOWN_UNCOVERED`] のモジュール名がすべて台帳に実在することを固定する
/// （タイポ・リネーム追随漏れの検知）。
#[test]
fn known_uncovered_modules_all_exist_in_the_catalog() {
    let modules: BTreeMap<&str, ()> = primitives_catalog::entries()
        .map(|entry| (entry.module, ()))
        .collect();
    for (module, part, _reason) in KNOWN_UNCOVERED {
        assert!(
            modules.contains_key(module),
            "KNOWN_UNCOVERED の module `{module}`（part `{part}`）が台帳に存在しない"
        );
    }
}
