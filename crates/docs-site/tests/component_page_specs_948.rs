//! イシュー #948（親 #928 Phase 4）が供給する
//! `crate::component_page_specs_948::SPECS` レジストリの契約テスト。
//!
//! `crates/docs-site/tests/component_pages.rs` は 4 並列 PR（#945〜#948）が
//! いずれも触り得る共有ファイルのため変更しない方針（実装計画 §6 参照）。
//! 本ファイルはイシュー番号ごとに 1 個ずつ新設される想定であり、各自の
//! `SPECS` レジストリのみを検証する（他イシューの `component_page_specs_*`
//! モジュールへは依存しない）。

use std::collections::BTreeSet;

use fandhe_frontend_docs_site::component_page_specs_948::SPECS;
use fandhe_frontend_docs_site::showcase;

/// `SPECS` 内でパスが重複していないこと。`component_page::spec_for` は
/// `.find(...)` で先勝ちに解決するため、重複登録があっても機械的には
/// エラーにならず後発エントリが黙って無視される（fail-closed でない
/// 事故）。本テストはその事故を検知する。
#[test]
fn specs_has_no_duplicate_paths() {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut duplicates: Vec<&str> = Vec::new();
    for (path, _) in SPECS {
        if !seen.insert(path) {
            duplicates.push(path);
        }
    }
    assert!(
        duplicates.is_empty(),
        "component_page_specs_948::SPECS has duplicate path(s): {duplicates:?}"
    );
}

/// `SPECS` の全登録パスが `showcase::component_page_paths()`（Rust 側デモを
/// 持つモード A ページ）に含まれること。モード B（`angle-slider` /
/// `clipboard` / `image-cropper` / `signature-pad` / `skip-nav`）は
/// `showcase::COMPONENT_PAGES` に未登録で `component_page::generated_content`
/// が `None` を返すため、誤って spec を登録するとデッドコード化する
/// （実装計画 §2.2/§7 が明示する事故を fail-closed で防ぐ）。
#[test]
fn specs_paths_are_registered_component_pages() {
    let registered: BTreeSet<&str> = showcase::component_page_paths().collect();
    for (path, _) in SPECS {
        assert!(
            registered.contains(path),
            "component_page_specs_948::SPECS registers {path}, but it is not in \
             showcase::component_page_paths() (mode B pages must not carry a spec)"
        );
    }
}

/// #948 の担当 28 ページがちょうど登録されていること（過不足の検知）。
/// 分割算術は実装計画 §2.1 参照。
#[test]
fn specs_registers_exactly_the_expected_28_paths() {
    const EXPECTED: &[&str] = &[
        "/components/blockquote/",
        "/components/code/",
        "/components/em/",
        "/components/heading/",
        "/components/highlight/",
        "/components/kbd/",
        "/components/list/",
        "/components/mark/",
        "/components/text/",
        "/components/visually-hidden/",
        "/components/charts/",
        "/components/area-chart/",
        "/components/bar-chart/",
        "/components/bar-list/",
        "/components/bar-segment/",
        "/components/donut-chart/",
        "/components/line-chart/",
        "/components/pie-chart/",
        "/components/radar-chart/",
        "/components/scatter-chart/",
        "/components/sparkline/",
        "/components/download-trigger/",
        "/components/qr-code/",
        "/components/timer/",
        "/components/color-picker/",
        "/components/calendar/",
        "/components/date-picker/",
        "/components/date-input/",
    ];
    let expected: BTreeSet<&str> = EXPECTED.iter().copied().collect();
    let actual: BTreeSet<&str> = SPECS.iter().map(|(path, _)| *path).collect();
    assert_eq!(
        expected.len(),
        28,
        "EXPECTED fixture itself must list 28 unique paths"
    );
    assert_eq!(actual, expected);
}

/// 登録済みページが `generated_content` を経由して実際に Features/API
/// Reference/Examples のいずれか（`ComponentPageSpec::EMPTY` ではない）を
/// 持つこと。空のまま登録するとデッドコードになる（この PR の目的は
/// 「未充填告知の除去」であり、告知だけ消して spec が空のままの回帰を防ぐ）。
#[test]
fn every_registered_spec_is_non_empty() {
    for (path, spec) in SPECS {
        let has_content = !spec.features.is_empty()
            || !spec.arguments.is_empty()
            || !spec.examples.is_empty()
            || !spec.keyboard.is_empty()
            || !spec.aria.is_empty();
        assert!(
            has_content,
            "component_page_specs_948::SPECS[{path}] is empty"
        );
    }
}
