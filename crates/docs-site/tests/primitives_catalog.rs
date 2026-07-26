//! イシュー #1020: Primitives 台帳（`fandhe_frontend_docs_site::primitives_catalog`）
//! と `crates/headless-ui/src/` の実態のドリフトを fail-closed に検知する。
//!
//! 判別規約は `docs/design/docs-site-primitives-themes-split.md` §6。
//! headless-ui ソースの走査は本ファイル（`tests/`）にのみ置き、
//! `crates/docs-site` の lib 本体（`src/primitives_catalog.rs`）は
//! `std::fs` を持たない純データ + 純関数に留める（docs-site は headless-ui
//! へ直接依存しないため、イシュー #693）。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use fandhe_frontend_docs_site::nav::parse_nav;
use fandhe_frontend_docs_site::primitives_catalog::{
    self, CatalogAudit, PrimitiveCategory, CRATE_ROOT_MODULE, FOUNDATION_MODULES, PRIMITIVES,
    PRIMITIVES_WITHOUT_THEMES_PAGE,
};

/// `CARGO_MANIFEST_DIR`（`crates/docs-site`）から repo_root を解決する
/// （`tests/site_nav.rs` / `tests/api_component_cross_links.rs` と同一規則）。
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo_root should resolve from CARGO_MANIFEST_DIR")
}

/// 統合テストのスクラッチ基点。`CARGO_TARGET_TMPDIR` は cargo が統合テスト
/// バイナリの**コンパイル時のみ**設定する（Cargo Book）ため `env!` で確定し、
/// 実行時 env による明示上書きのみ許容する。`/tmp` へは一切フォールバック
/// しない（イシュー #637 の事実誤認の再発防止、#658、`tests/nav_group_schema.rs`
/// と同一パターン）。
fn scratch_root() -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    let _ = std::fs::create_dir_all(&root);
    root
}

/// `crates/headless-ui/src/*.rs` の走査結果。
struct ScanResult {
    /// `lib` を除く mod 名の集合。
    module_names: BTreeSet<String>,
    /// `src/*.rs` の総ファイル数（`lib.rs` を含む）。
    total_rs_files: usize,
    /// 本文に部分文字列 `anatomy(` を含む mod 名の集合（`anatomy` 自身の
    /// 除外は呼び出し側の責務。設計 §6 の
    /// `grep -l 'anatomy(' … | grep -v '/anatomy.rs'` と逐語一致させる
    /// ための分離）。
    anatomy_callers: BTreeSet<String>,
}

/// `dir` 直下の `*.rs` を走査する。設計 §6「glob の穴を塞ぐ規定」に従い、
/// ディレクトリの出現・`.rs` 以外の拡張子は fail-closed に panic する
/// （将来 `src/<name>/mod.rs` のような入れ子モジュールが glob をすり抜けて
/// 台帳から静かに漏れるのを防ぐ）。
fn scan_headless_ui_src(dir: &Path) -> ScanResult {
    let mut module_names = BTreeSet::new();
    let mut anatomy_callers = BTreeSet::new();
    let mut total_rs_files = 0usize;

    let read_dir =
        std::fs::read_dir(dir).unwrap_or_else(|e| panic!("failed to read {}: {e}", dir.display()));
    for entry in read_dir {
        let entry = entry.unwrap_or_else(|e| panic!("failed to read dir entry: {e}"));
        let path = entry.path();
        let file_type = entry
            .file_type()
            .unwrap_or_else(|e| panic!("failed to stat {}: {e}", path.display()));

        if file_type.is_dir() {
            panic!(
                "unexpected directory `{}` under {} — 設計 §6「glob の穴を塞ぐ規定」により \
                 crates/headless-ui/src/ 直下はフラットな *.rs のみを許容する。\
                 入れ子モジュールを追加する場合は primitives_catalog.rs の台帳・\
                 基盤リストへの反映と本テストの更新方針を設計文書 §6 で先に検討すること",
                path.display(),
                dir.display()
            );
        }

        let extension = path.extension().and_then(|e| e.to_str());
        if extension != Some("rs") {
            panic!(
                "unexpected non-`.rs` entry `{}` under {} — fail-closed: \
                 未知の混入を握り潰さない設計方針（設計 §6）",
                path.display(),
                dir.display()
            );
        }

        total_rs_files += 1;
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("mod file stem should be valid UTF-8")
            .to_string();

        let body = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        if body.contains("anatomy(") {
            anatomy_callers.insert(stem.clone());
        }

        if stem != CRATE_ROOT_MODULE {
            module_names.insert(stem);
        }
    }

    ScanResult {
        module_names,
        total_rs_files,
        anatomy_callers,
    }
}

fn headless_ui_src_dir() -> PathBuf {
    repo_root().join("crates/headless-ui/src")
}

/// 受け入れ条件 2・3 の本番側: headless-ui の全モジュールが、台帳か基盤
/// リストのどちらかちょうど一方に属すること。
#[test]
fn every_headless_ui_module_belongs_to_exactly_one_of_catalog_or_foundation() {
    let scan = scan_headless_ui_src(&headless_ui_src_dir());
    let result: CatalogAudit = primitives_catalog::audit(&scan.module_names);

    assert!(
        result.is_clean(),
        "Primitives 台帳（crates/docs-site/src/primitives_catalog.rs）が \
         crates/headless-ui/src/ とドリフトしています。\n\
         - unregistered（台帳にも基盤にも無い新規モジュール、PRIMITIVES か \
           FOUNDATION_MODULES へ追加が必要）: {:?}\n\
         - missing_from_source（台帳に載っているが消滅したモジュール、PRIMITIVES \
           から削除が必要）: {:?}\n\
         - missing_foundation（基盤リストに載っているが消滅したモジュール、\
           FOUNDATION_MODULES から削除が必要）: {:?}\n\
         - duplicated（台帳と基盤の両方に載っているモジュール、片方から削除が \
           必要）: {:?}\n\
         詳細は docs/design/docs-site-primitives-themes-split.md §6 を参照。",
        result.unregistered,
        result.missing_from_source,
        result.missing_foundation,
        result.duplicated,
    );
}

/// 設計 §6 の判別規則（`anatomy(` を呼ぶ `.rs`、`anatomy.rs` 自身を除く）と
/// 台帳の module 集合が完全一致すること（片側包含ではなく集合一致）。
#[test]
fn catalog_matches_the_anatomy_call_rule() {
    let scan = scan_headless_ui_src(&headless_ui_src_dir());

    let anatomy_rule_modules: BTreeSet<String> = scan
        .anatomy_callers
        .iter()
        .filter(|m| m.as_str() != "anatomy")
        .cloned()
        .collect();

    let catalog_modules: BTreeSet<String> =
        PRIMITIVES.iter().map(|e| e.module.to_string()).collect();

    assert_eq!(
        anatomy_rule_modules, catalog_modules,
        "設計 §6 の判別規則（anatomy( 呼び出し、anatomy.rs 自身を除く）と \
         PRIMITIVES の module 集合が一致しません"
    );
}

/// 基盤モジュールのうち `anatomy` 以外は `anatomy(` を 1 件も含まないこと
/// （`anatomy` 自身は定義元のため除外する）。
#[test]
fn foundation_modules_do_not_call_anatomy() {
    let scan = scan_headless_ui_src(&headless_ui_src_dir());

    for module in FOUNDATION_MODULES {
        if *module == "anatomy" {
            // anatomy() の定義元自身。呼び出し判定の対象外。
            continue;
        }
        assert!(
            !scan.anatomy_callers.contains(*module),
            "基盤モジュール `{module}` が anatomy( を含んでいます。設計 §6 の \
             判別規則が破綻している可能性があります（規則を賢くするのではなく \
             設計文書の改訂を検討すること）"
        );
    }
}

/// `PRIMITIVES.len() + FOUNDATION_MODULES.len() + 1（lib.rs）` が実測の
/// `.rs` 総数と一致すること。73 をハードコードせず走査結果から導出する
/// （手動同期点を作らないため）。
#[test]
fn module_counts_are_consistent_with_the_source_tree() {
    let scan = scan_headless_ui_src(&headless_ui_src_dir());

    assert_eq!(PRIMITIVES.len(), 63);
    assert_eq!(FOUNDATION_MODULES.len(), 9);
    assert_eq!(
        PRIMITIVES.len() + FOUNDATION_MODULES.len() + 1,
        scan.total_rs_files,
        "PRIMITIVES(63) + FOUNDATION_MODULES(9) + lib.rs(1) が \
         crates/headless-ui/src/*.rs の実測総数({})と一致しません",
        scan.total_rs_files
    );
}

/// 受け入れ条件 4: 6 グループ 11/11/10/10/11/10 = 63、カテゴリ出現順・
/// グループ内順序が設計 §7 逐語であること。
#[test]
fn category_counts_and_order_follow_the_design_spec() {
    // 設計 §7 の表を逐語で再掲する（グループ内順序も含む）。並びを
    // アルファベット順へ正規化しないこと（#1021 が「1020 の台帳順」で
    // nav へ登録するため）。
    let spec: [(PrimitiveCategory, &[&str]); 6] = [
        (
            PrimitiveCategory::FormsA,
            &[
                "angle_slider",
                "checkbox",
                "checkbox_group",
                "color_picker",
                "combobox",
                "editable",
                "field",
                "fieldset",
                "file_upload",
                "image_cropper",
                "listbox",
            ],
        ),
        (
            PrimitiveCategory::FormsB,
            &[
                "number_input",
                "password_input",
                "pin_input",
                "radio_group",
                "rating_group",
                "segment_group",
                "select",
                "signature_pad",
                "slider",
                "switch",
                "tags_input",
            ],
        ),
        (
            PrimitiveCategory::FormsCDateStatus,
            &[
                "calendar",
                "date_input",
                "date_picker",
                "download_trigger",
                "toggle",
                "toggle_group",
                "clipboard",
                "timer",
                "progress",
                "qr_code",
            ],
        ),
        (
            PrimitiveCategory::OverlayDisclosure,
            &[
                "accordion",
                "collapsible",
                "dialog",
                "drawer",
                "floating_panel",
                "hover_card",
                "popover",
                "toast",
                "toggle_tip",
                "tooltip",
            ],
        ),
        (
            PrimitiveCategory::Navigation,
            &[
                "action_bar",
                "breadcrumb",
                "link",
                "link_overlay",
                "menu",
                "menubar",
                "nav_list",
                "navigation_menu",
                "pagination",
                "tabs",
                "toolbar",
            ],
        ),
        (
            PrimitiveCategory::DataDisplayUtilities,
            &[
                "avatar",
                "carousel",
                "json_tree_view",
                "scroll_area",
                "skip_nav",
                "splitter",
                "steps",
                "tour",
                "tree_view",
                "visually_hidden",
            ],
        ),
    ];

    let expected_total: usize = spec.iter().map(|(_, modules)| modules.len()).sum();
    assert_eq!(expected_total, 63);

    let actual_modules_in_order: Vec<&str> = PRIMITIVES.iter().map(|e| e.module).collect();
    let expected_modules_in_order: Vec<&str> = spec
        .iter()
        .flat_map(|(_, modules)| modules.iter().copied())
        .collect();
    assert_eq!(
        actual_modules_in_order, expected_modules_in_order,
        "PRIMITIVES の並び（カテゴリ順・グループ内順）が設計 §7 の逐語記載と \
         一致しません"
    );

    let actual_categories_in_order: Vec<PrimitiveCategory> =
        PRIMITIVES.iter().map(|e| e.category).collect();
    let expected_categories_in_order: Vec<PrimitiveCategory> = spec
        .iter()
        .flat_map(|(category, modules)| std::iter::repeat_n(*category, modules.len()))
        .collect();
    assert_eq!(actual_categories_in_order, expected_categories_in_order);
}

/// 台帳の全ページ path を `[[section.page]]` として組み立てた最小
/// `nav.toml` が実バリデータ（`nav::parse_nav` 内部の `validate_page_path`
/// 相当）を通ること。パス規約の二重実装を避け、#1021 が nav 登録した
/// 時点で初めて弾かれる事故を前倒しで防ぐ。
#[test]
fn page_paths_pass_the_nav_path_allowlist() {
    // イシュー #1010 で `[[section]]` に `index_path` が必須化されたため、
    // 台帳の先頭ページの path をそのままセクション代表ページとして流用する
    // （`index_path ⊆ 生成ページの path 集合` の不変条件を満たす必要がある）。
    let first_path = PRIMITIVES
        .first()
        .expect("PRIMITIVES should not be empty")
        .path;
    let mut toml = format!(
        "[site]\ntitle = \"t\"\nbase_path = \"\"\n\n[[section]]\ntitle = \"Primitives\"\nindex_path = \"{first_path}\"\n\n",
    );
    for (i, entry) in PRIMITIVES.iter().enumerate() {
        toml.push_str(&format!(
            "[[section.page]]\ntitle = \"{}\"\nsource = \"site/primitives/fixture-{i}.md\"\npath = \"{}\"\n\n",
            entry.title, entry.path
        ));
    }

    parse_nav(&toml)
        .unwrap_or_else(|e| panic!("台帳の path が nav.toml パス allowlist を通りません: {e}"));
}

/// nav 登録済み Themes 部品ページ（`source == "site/themes/<kebab>.md"`）
/// の title と、同名 module の Primitives title が一致すること。
/// URL 移転に耐えるため `path` ではなく `source` をキーにする
/// （`api_component_cross_links.rs` の先例に合わせる。#1017 で
/// `/components/` → `/themes/` へ移行済み）。
#[test]
fn primitives_titles_match_themes_page_titles_where_both_exist() {
    let nav_path = repo_root().join("site/nav.toml");
    let input = std::fs::read_to_string(&nav_path).expect("site/nav.toml should be readable");
    let nav = parse_nav(&input).expect("site/nav.toml should parse");

    let themes_titles: std::collections::BTreeMap<String, String> = nav
        .all_pages()
        .filter_map(|p| {
            p.source
                .strip_prefix("site/themes/")
                .and_then(|rest| rest.strip_suffix(".md"))
                .map(|kebab| (kebab.replace('-', "_"), p.title.clone()))
        })
        .collect();

    for entry in PRIMITIVES {
        if PRIMITIVES_WITHOUT_THEMES_PAGE.contains(&entry.module) {
            continue;
        }
        match themes_titles.get(entry.module) {
            Some(themes_title) => assert_eq!(
                entry.title, themes_title,
                "module `{}` の Primitives title と Themes title が一致しません \
                 （#1018/#1021 で source ディレクトリごと動かした場合は本テストの \
                 更新が必要）",
                entry.module
            ),
            None => panic!(
                "module `{}` に対応する Themes ページ（site/themes/{}.md）が \
                 見つかりません。PRIMITIVES_WITHOUT_THEMES_PAGE への追加漏れ、\
                 または site/nav.toml 側の変化の可能性があります",
                entry.module,
                primitives_catalog::kebab_of(entry.module)
            ),
        }
    }
}

/// テスト専用の一時ディレクトリ（`tests/nav_group_schema.rs` の `TempDir`
/// と同方針。外部クレート `tempfile` を追加しない、REQ-3）。
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = scratch_root().join(format!(
            "fandhe-frontend-docs-site-primitives-catalog-{tag}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path)
            .unwrap_or_else(|e| panic!("failed to create {}: {e}", path.display()));
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// 設計 §6 の「glob の穴を塞ぐ規定」: ディレクトリが混入したフィクスチャで
/// `scan_headless_ui_src` が panic すること。
#[test]
fn scanner_rejects_nested_module_directories() {
    let dir = TempDir::new("nested-dir");
    std::fs::write(dir.0.join("foo.rs"), "// fixture\n").expect("failed to write foo.rs");
    std::fs::create_dir_all(dir.0.join("bar")).expect("failed to create nested dir bar/");

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        scan_headless_ui_src(&dir.0)
    }));

    assert!(
        result.is_err(),
        "ディレクトリ混入フィクスチャに対して scan_headless_ui_src が panic しませんでした"
    );
}

/// 上記の対照: フラットな `*.rs` のみのフィクスチャは正常に走査でき、
/// `scanner_rejects_nested_module_directories` が常時 panic する実装に
/// なっていないことを保証する。
#[test]
fn scanner_accepts_a_flat_rs_only_fixture() {
    let dir = TempDir::new("flat-fixture");
    std::fs::write(dir.0.join("lib.rs"), "// fixture crate root\n")
        .expect("failed to write lib.rs");
    std::fs::write(dir.0.join("widget.rs"), "fn root() { anatomy(); }\n")
        .expect("failed to write widget.rs");
    std::fs::write(dir.0.join("helper.rs"), "// no anatomy call here\n")
        .expect("failed to write helper.rs");

    let scan = scan_headless_ui_src(&dir.0);

    assert_eq!(scan.total_rs_files, 3);
    assert_eq!(
        scan.module_names,
        BTreeSet::from(["widget".to_string(), "helper".to_string()])
    );
    assert_eq!(scan.anatomy_callers, BTreeSet::from(["widget".to_string()]));
}
