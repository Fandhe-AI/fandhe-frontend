//! イシュー #1064: Primitives（`fandhe-frontend-headless-ui`、63 部品）と
//! Themes（`fandhe-frontend-pre-styled-ui`、108 部品）の**層をまたぐラップ状態**
//! を機械可視化する契約テスト。
//!
//! # 背景・既存テストとの分担
//!
//! `tests/primitives_catalog.rs` は headless-ui ソース ↔ 台帳のドリフトを
//! レイヤー内で検知するのみで、「Themes 108 部品のどれが headless をラップし、
//! どれが独自実装か」という層をまたぐ対応関係は検証しない
//! （`primitives_titles_match_themes_page_titles_where_both_exist` は同名
//! ページが両方に存在する場合の title 一致のみを見る）。本ファイルはその
//! 空白を埋め、`docs/policy/intentional-non-adoption.md` §3.25（UI 部品の
//! 責務境界）に反する「本来 headless をラップすべき部品の独自実装」がレビュー
//! をすり抜けるのを防ぐ。判別規約は
//! `docs/design/docs-site-primitives-themes-split.md` §6a を参照。
//!
//! # 4 バケット分割（Themes 108 部品）
//!
//! - [`WRAPPED_SAME_NAME`]（61）: 同名の Primitives 部品が存在し、かつ同名
//!   headless モジュールへコード委譲している
//! - [`WRAPPED_CROSS_NAME`]（5）: 同名 Primitives 部品は無いが、別名の
//!   headless 部品へコード委譲している
//! - [`DOC_REFERENCE_ONLY`]（3）: headless 部品への参照が rustdoc のみ
//!   （コード委譲なし）。**このバケットはドリフトしやすい**: rustdoc に
//!   1 文足すだけで C（rustdoc 言及）と D（無関係）の境界が動く。テストが
//!   落ちたら「実装が変わった」のではなく「台帳更新イベント」であることが
//!   多いので、定数を実態に合わせて更新し PR 本文に理由を書く運用とする
//! - [`PRE_STYLED_ONLY`]（39）: headless 部品への参照がコード・rustdoc
//!   いずれにも無い
//!
//! rustdoc（`//!` / `///`）の言及は「ラップ済み」の根拠にしない
//! （コード実体を伴わない主張は壊れやすい契約になるため）。
//!
//! # なぜ `crates/pre-styled-ui` へ直接依存せずソース走査するか
//!
//! `tests/primitives_catalog.rs`（イシュー #1020/#693）と同じ理由。
//! `crates/docs-site` の lib 本体は headless-ui/pre-styled-ui の実装詳細へ
//! 実行時依存しない方針であり、層をまたぐ突合はテスト専用の走査で行う。

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use fandhe_frontend_docs_site::nav::parse_nav;
use fandhe_frontend_docs_site::primitives_catalog::{
    self, PRIMITIVES, PRIMITIVES_WITHOUT_THEMES_PAGE,
};

/// `CARGO_MANIFEST_DIR`（`crates/docs-site`）から repo_root を解決する
/// （`tests/primitives_catalog.rs` と同一規則）。
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo_root should resolve from CARGO_MANIFEST_DIR")
}

/// 統合テストのスクラッチ基点。`CARGO_TARGET_TMPDIR` は cargo が統合テスト
/// バイナリの**コンパイル時のみ**設定する（イシュー #637 の事実誤認の
/// 再発防止）ため `env!` で確定し、`/tmp` へは一切フォールバックしない
/// （`tests/primitives_catalog.rs` と同一パターン）。
fn scratch_root() -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    let _ = std::fs::create_dir_all(&root);
    root
}

fn pre_styled_ui_src_dir() -> PathBuf {
    repo_root().join("crates/pre-styled-ui/src")
}

// ---------------------------------------------------------------------
// スキャナ
// ---------------------------------------------------------------------

/// `crates/pre-styled-ui/src/` 直下で正当に許容するサブディレクトリ。
/// これ以外のサブディレクトリの出現は fail-closed に panic する
/// （§3.7: `primitives_catalog.rs` のフラット規約に対する意図的な差異。
/// pre-styled-ui は `charts/` の入れ子を正当に持つため同一規則を適用できない）。
const NESTED_MODULE_DIRS: &[&str] = &["charts"];

/// 1 ファイルの走査結果。
#[derive(Debug, Default)]
struct FileScan {
    /// 非コメント行（コード行）に現れる `fandhe_frontend_headless_ui::<ident>`
    /// の `<ident>` 集合。
    code_refs: BTreeSet<String>,
    /// コメント行（`//`/`///`/`//!` のいずれか）に現れる同上の `<ident>` 集合。
    comment_refs: BTreeSet<String>,
}

/// `crates/pre-styled-ui/src/` の走査結果。トップレベルと `charts/` を
/// 分離して保持する（`tooltip` のようなステム衝突を解決するため、
/// §3.5 の resolve_page がどちらを優先するかを明示的に扱う必要がある）。
#[derive(Debug, Default)]
struct PreStyledScan {
    /// `src/*.rs` の stem（拡張子抜きファイル名）→ 走査結果。
    top_level: BTreeMap<String, FileScan>,
    /// `src/charts/*.rs` の stem → 走査結果（`mod` を含む）。
    charts: BTreeMap<String, FileScan>,
}

/// ダブルクォート文字列リテラルのスパンを除去した行を返す。
/// ブロックコメント検出（`/*` の残留チェック）が文字列リテラル内の `/*`
/// （現状 `file_upload.rs` の `"image/*"` 等）を誤検知しないための前処理。
/// エスケープは `\"` のみを考慮する簡易実装（本クレートに複雑な文字列
/// リテラルは無い前提。誤って解析が壊れた場合は `/*` 残留 panic 側に
/// 倒れるため安全側）。
fn strip_string_literals(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_string = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if in_string {
            if c == '\\' {
                chars.next();
                continue;
            }
            if c == '"' {
                in_string = false;
            }
            continue;
        }
        if c == '"' {
            in_string = true;
            continue;
        }
        out.push(c);
    }
    out
}

/// 1 行から `fandhe_frontend_headless_ui::<ident>` の `<ident>` をすべて
/// 抽出する（正規表現クレートを追加しない、REQ-3 依存上限）。
fn extract_headless_refs(line: &str) -> Vec<String> {
    const MARKER: &str = "fandhe_frontend_headless_ui::";
    let mut refs = Vec::new();
    let mut search_from = 0usize;
    while let Some(pos) = line[search_from..].find(MARKER) {
        let start = search_from + pos + MARKER.len();
        let rest = &line[start..];
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        let ident = &rest[..end];
        if !ident.is_empty() {
            refs.push(ident.to_string());
        }
        search_from = start + end.max(1);
    }
    refs
}

/// 1 ファイルを走査する。ブロックコメント（`/*`）の混入は fail-closed に
/// panic する（§3.2: 現状ブロックコメントは存在せず、規則を賢くするより
/// 判別規約の改訂を先に検討させる）。
///
/// `#[cfg(test)]` に続くモジュールブロック内の行は `code_refs` の走査対象
/// から除外する（イシュー #1064 の Bugbot 指摘、PR #1096）。素朴な
/// 「非コメント行はすべて証跡」判定だと、テストコードにしか
/// `fandhe_frontend_headless_ui::<ident>` を持たない部品（`radio_card.rs` が
/// 該当）を「本番コードが委譲している」と誤分類してしまう
/// （`wrap_state_partition_matches_the_declared_ledger` が参照する 4 バケット
/// 台帳の正当性を損なう）。除外範囲は波括弧の深さを追跡して境界を決定し、
/// 「`mod tests` という文字列以降を全部無視」のような文字列一致に頼る脆い
/// 実装は避ける（同名の別モジュール混入・複数 `mod` 併存に耐えるため）。
fn scan_file(path: &Path, rel_path: &Path) -> FileScan {
    let body = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));

    let mut scan = FileScan::default();
    // ファイル先頭からの累積波括弧深さ。コメント行の `{`/`}` は数えない
    // （rustdoc の Examples に波括弧が含まれても深さがずれないようにする）。
    let mut depth: i64 = 0;
    // `#[cfg(test)]` 検出後、直後に開くブロックの「開始前の深さ」を保持する。
    // `Some(d)` の間は該当ブロック内（skip 対象）であり、`depth` が `d` まで
    // 戻った時点で通常走査へ復帰する。
    let mut skip_from_depth: Option<i64> = None;
    // `#[cfg(test)]` を検出したがまだ対応する `mod ... {` の開き括弧行に
    // 到達していない状態を示す。
    let mut cfg_test_pending = false;

    for (idx, line) in body.lines().enumerate() {
        let stripped = strip_string_literals(line);
        if stripped.contains("/*") {
            panic!(
                "block comment marker `/*` detected at {}:{} — 本テストの \
                 スキャナは §3.2 の判別規則上ブロックコメントを想定していません。\
                 判別規約（docs/design/docs-site-primitives-themes-split.md §6a）\
                 の改訂を先に検討してください。",
                rel_path.display(),
                idx + 1
            );
        }

        let is_comment_line = line.trim_start().starts_with("//");

        if !is_comment_line {
            let trimmed = stripped.trim();
            if skip_from_depth.is_none() && trimmed == "#[cfg(test)]" {
                cfg_test_pending = true;
            }

            let opens = stripped.matches('{').count() as i64;
            let closes = stripped.matches('}').count() as i64;

            // `#[cfg(test)]` 検出後、最初に `mod` を伴って `{` が現れる行を
            // ブロック開始とみなす（`mod tests;`（ブロックなし外部ファイル
            // 参照）のような形は対象外とし、無関係な後続ブロックを誤って
            // skip 対象にしない）。
            if cfg_test_pending && opens > 0 && trimmed.contains("mod ") {
                skip_from_depth = Some(depth);
                cfg_test_pending = false;
            }

            depth += opens - closes;

            if let Some(resume_depth) = skip_from_depth {
                if depth <= resume_depth {
                    skip_from_depth = None;
                }
            }
        }

        if skip_from_depth.is_some() {
            continue;
        }

        for r in extract_headless_refs(line) {
            if is_comment_line {
                scan.comment_refs.insert(r);
            } else {
                scan.code_refs.insert(r);
            }
        }
    }
    scan
}

/// `crates/pre-styled-ui/src/` を走査する。`NESTED_MODULE_DIRS` 以外の
/// サブディレクトリ・深さ 2 段目以降のサブディレクトリ・`.rs` 以外の
/// エントリ・symlink エントリは fail-closed に panic する（§3.7）。
fn scan_pre_styled_src(dir: &Path) -> PreStyledScan {
    let mut scan = PreStyledScan::default();

    let read_dir =
        std::fs::read_dir(dir).unwrap_or_else(|e| panic!("failed to read {}: {e}", dir.display()));
    for entry in read_dir {
        let entry = entry.unwrap_or_else(|e| panic!("failed to read dir entry: {e}"));
        let path = entry.path();
        let file_type = entry
            .file_type()
            .unwrap_or_else(|e| panic!("failed to stat {}: {e}", path.display()));

        if file_type.is_symlink() {
            panic!(
                "unexpected symlink `{}` under {} — fail-closed: symlink は \
                 read_to_string が意図せず追跡してしまう経路のため許容しない",
                path.display(),
                dir.display()
            );
        }

        if file_type.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !NESTED_MODULE_DIRS.contains(&name.as_str()) {
                panic!(
                    "unexpected subdirectory `{}` under {} — 許容サブ \
                     ディレクトリは NESTED_MODULE_DIRS ({NESTED_MODULE_DIRS:?}) \
                     のみです。新規サブディレクトリを追加する場合は本テストの \
                     走査規約（§3.7）の更新を先に検討してください。",
                    path.display(),
                    dir.display()
                );
            }

            let nested_read_dir = std::fs::read_dir(&path)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
            for nested_entry in nested_read_dir {
                let nested_entry =
                    nested_entry.unwrap_or_else(|e| panic!("failed to read dir entry: {e}"));
                let nested_path = nested_entry.path();
                let nested_file_type = nested_entry
                    .file_type()
                    .unwrap_or_else(|e| panic!("failed to stat {}: {e}", nested_path.display()));

                if nested_file_type.is_symlink() {
                    panic!(
                        "unexpected symlink `{}` under {} — fail-closed",
                        nested_path.display(),
                        path.display()
                    );
                }
                if nested_file_type.is_dir() {
                    panic!(
                        "unexpected nested directory `{}` under {} — 深さ 1 段 \
                         （`src/{name}/`）のみを許容します（§3.7）",
                        nested_path.display(),
                        path.display()
                    );
                }

                let ext = nested_path.extension().and_then(|e| e.to_str());
                if ext != Some("rs") {
                    panic!(
                        "unexpected non-`.rs` entry `{}` under {} — fail-closed",
                        nested_path.display(),
                        path.display()
                    );
                }

                let stem = nested_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .expect("mod file stem should be valid UTF-8")
                    .to_string();
                let rel = nested_path
                    .strip_prefix(dir)
                    .expect("nested_path should be under dir")
                    .to_path_buf();
                scan.charts.insert(stem, scan_file(&nested_path, &rel));
            }
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("rs") {
            panic!(
                "unexpected non-`.rs` entry `{}` under {} — fail-closed",
                path.display(),
                dir.display()
            );
        }

        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("mod file stem should be valid UTF-8")
            .to_string();
        let rel = path
            .strip_prefix(dir)
            .expect("path should be under dir")
            .to_path_buf();
        scan.top_level.insert(stem, scan_file(&path, &rel));
    }

    scan
}

// ---------------------------------------------------------------------
// nav.toml 読み取り
// ---------------------------------------------------------------------

/// `site/nav.toml` から Themes 部品ページ（`source == "site/themes/<kebab>.md"`）
/// の kebab 集合を取り出す。`path` ではなく `source` をキーにする理由は
/// `tests/primitives_catalog.rs` と同じ（URL 移転に耐えるため、#1017）。
fn themes_page_kebabs() -> BTreeSet<String> {
    let nav_path = repo_root().join("site/nav.toml");
    let input = std::fs::read_to_string(&nav_path).expect("site/nav.toml should be readable");
    let nav = parse_nav(&input).expect("site/nav.toml should parse");

    nav.all_pages()
        .filter_map(|p| {
            p.source
                .strip_prefix("site/themes/")
                .and_then(|rest| rest.strip_suffix(".md"))
                .map(|kebab| kebab.to_string())
        })
        .collect()
}

fn kebab_to_snake(kebab: &str) -> String {
    kebab.replace('-', "_")
}

// ---------------------------------------------------------------------
// §3.5: ページ → モジュール解決
// ---------------------------------------------------------------------

/// `charts` ページ（Themes 索引ページ）が特別に解決する先。
const CHARTS_INDEX_PAGE: &str = "charts";

/// トップレベルと `charts/` の両方に同名ステムが存在する既知の衝突。
/// トップレベルを優先解決する（§3.5）。ここに無い衝突が新たに出現したら
/// fail-closed に panic する（`top_level_and_charts_module_name_collisions_are_declared`）。
const TOP_LEVEL_WINS_COLLISIONS: &[&str] = &["tooltip"];

/// Themes ページ（kebab）を `PreStyledScan` 上の唯一のモジュールへ解決する。
/// 解決順序: (1) `charts` 特例 → `charts/mod.rs`、(2) トップレベル同名ステム、
/// (3) `charts/` 同名ステム。いずれにも無ければ fail-closed に panic する。
fn resolve_page<'a>(scan: &'a PreStyledScan, page_kebab: &str) -> &'a FileScan {
    let mod_name = kebab_to_snake(page_kebab);

    if mod_name == CHARTS_INDEX_PAGE {
        return scan.charts.get("mod").unwrap_or_else(|| {
            panic!(
                "Themes ページ `{page_kebab}` は charts 索引特例として \
                 crates/pre-styled-ui/src/charts/mod.rs に解決するはずですが \
                 見つかりません"
            )
        });
    }
    if let Some(f) = scan.top_level.get(&mod_name) {
        return f;
    }
    if let Some(f) = scan.charts.get(&mod_name) {
        return f;
    }
    panic!(
        "Themes ページ `{page_kebab}`（モジュール名 `{mod_name}`）が \
         crates/pre-styled-ui/src/ 直下にも charts/ 配下にも見つかりません。\
         site/nav.toml の source パスとファイル名の対応を確認してください。"
    );
}

// ---------------------------------------------------------------------
// §3.3: 4 バケットの期待値台帳
// ---------------------------------------------------------------------

/// バケット A: 同名 Primitives 部品が存在し、同名 headless モジュールへ
/// コード委譲している Themes ページ（kebab、ソート済み、61 件）。
const WRAPPED_SAME_NAME: &[&str] = &[
    "accordion",
    "action-bar",
    "angle-slider",
    "avatar",
    "breadcrumb",
    "calendar",
    "carousel",
    "checkbox",
    "checkbox-group",
    "clipboard",
    "collapsible",
    "color-picker",
    "combobox",
    "date-input",
    "date-picker",
    "dialog",
    "download-trigger",
    "drawer",
    "editable",
    "file-upload",
    "floating-panel",
    "hover-card",
    "image-cropper",
    "json-tree-view",
    "link",
    "link-overlay",
    "listbox",
    "menu",
    "menubar",
    "nav-list",
    "navigation-menu",
    "number-input",
    "pagination",
    "password-input",
    "pin-input",
    "popover",
    "progress",
    "qr-code",
    "radio-group",
    "rating-group",
    "scroll-area",
    "segment-group",
    "select",
    "signature-pad",
    "skip-nav",
    "slider",
    "splitter",
    "steps",
    "switch",
    "tabs",
    "tags-input",
    "timer",
    "toast",
    "toggle",
    "toggle-group",
    "toggle-tip",
    "toolbar",
    "tooltip",
    "tour",
    "tree-view",
    "visually-hidden",
];

/// バケット B: 同名 Primitives 部品は無いが、別名の headless 部品へ
/// コード委譲している Themes ページ（`(page_kebab, headless_module)`、4 件）。
///
/// `radio-card` は当初ここに分類されていたが、`fandhe_frontend_headless_ui::
/// radio_group` への参照がコード上は `#[cfg(test)] mod tests` 内にしか
/// 存在せず（本番コードは委譲していない）、`scan_file` が
/// `#[cfg(test)]` ブロックを走査対象から除外するよう是正されたことに伴い
/// [`DOC_REFERENCE_ONLY`] へ移動した（イシュー #1064 の Bugbot 指摘、
/// PR #1096）。
const WRAPPED_CROSS_NAME: &[(&str, &str)] = &[
    ("checkbox-card", "checkbox"),
    ("input", "field"),
    ("native-select", "field"),
    ("textarea", "field"),
];

/// バケット C: headless 部品への参照が rustdoc のみ（コード委譲なし）の
/// Themes ページ（`(page_kebab, headless_module)`、4 件）。**このバケットは
/// ドリフトしやすい**（モジュール冒頭コメントを参照）。
const DOC_REFERENCE_ONLY: &[(&str, &str)] = &[
    ("button", "number_input"),
    ("image", "avatar"),
    ("radio-card", "radio_group"),
    ("tab-nav", "tabs"),
];

/// バケット D: headless 部品への参照がコード・rustdoc いずれにも無い
/// Themes ページ（kebab、ソート済み、39 件。イシュー #1064 本文の受け入れ
/// 条件 2 が求める一覧）。
const PRE_STYLED_ONLY: &[&str] = &[
    "alert",
    "area-chart",
    "badge",
    "bar-chart",
    "bar-list",
    "bar-segment",
    "blockquote",
    "callout",
    "card",
    "charts",
    "code",
    "color-swatch",
    "data-list",
    "donut-chart",
    "em",
    "empty-state",
    "heading",
    "highlight",
    "icon",
    "kbd",
    "line-chart",
    "list",
    "mark",
    "marquee",
    "pie-chart",
    "quote",
    "radar-chart",
    "scatter-chart",
    "separator",
    "skeleton",
    "sparkline",
    "spinner",
    "stat",
    "status",
    "strong",
    "table",
    "tag",
    "text",
    "timeline",
];

/// §3.4（受け入れ条件 3）: pre-styled-ui のどこからもコード委譲されていない
/// headless 部品（module 名、1 件）。
const HEADLESS_UNWRAPPED: &[&str] = &["fieldset"];

/// `field`（Themes ページを持たない headless 部品）を別名でラップしている
/// pre-styled モジュール名（4 件）。イシュー #1684 で `field.rs`
/// （headless `field::root` へコード委譲する同名モジュール）を追加した
/// （`field` 自身も headless `field` へのコード委譲元であるため本台帳に
/// 含める。#1685 で `/themes/field/` ページを新設した際は本台帳の扱いを
/// 見直す）。
const FIELD_CROSS_WRAPPERS: &[&str] = &["field", "input", "native_select", "textarea"];

/// §3.6: トップレベルのうち Themes ページに対応しないモジュール（7 件。
/// イシュー #1684 で `field`（pre-styled-ui クレート内で完結する recipe
/// のみ実装、`/themes/field/` ページ未登録）を追加。ページ登録は #1685 の
/// スコープ）。
const NON_PAGE_TOP_LEVEL: &[&str] = &[
    "class_attr",
    "css",
    "field",
    "lib",
    "recipe",
    "stylesheet",
    "theme",
];

/// §3.6: `charts/` のうち Themes ページに対応しないモジュール（8 件。
/// `mod` は charts 索引ページとして別枠で扱うため含まない。`tooltip` は
/// トップレベルの同名ページに解決が奪われるためここに含む）。
const NON_PAGE_CHARTS: &[&str] = &[
    "axis", "data", "grid", "legend", "pie", "scale", "svg", "tooltip",
];

fn primitive_module_names() -> BTreeSet<&'static str> {
    PRIMITIVES.iter().map(|e| e.module).collect()
}

// ---------------------------------------------------------------------
// テスト本体
// ---------------------------------------------------------------------

/// §3.5: nav 登録済み Themes ページ 108 件すべてが `resolve_page` で panic
/// せず解決できること。
#[test]
fn every_themes_page_resolves_to_exactly_one_pre_styled_module() {
    let scan = scan_pre_styled_src(&pre_styled_ui_src_dir());
    let pages = themes_page_kebabs();
    assert_eq!(
        pages.len(),
        108,
        "site/nav.toml の Themes ページ数が想定と異なります"
    );

    for page in &pages {
        resolve_page(&scan, page);
    }
}

/// §3.5: トップレベルと `charts/` のステム衝突が `TOP_LEVEL_WINS_COLLISIONS`
/// と完全一致すること（新たな衝突が黙って解決されるのを防ぐ）。
#[test]
fn top_level_and_charts_module_name_collisions_are_declared() {
    let scan = scan_pre_styled_src(&pre_styled_ui_src_dir());

    let top_stems: BTreeSet<&String> = scan.top_level.keys().collect();
    let charts_stems: BTreeSet<&String> =
        scan.charts.keys().filter(|s| s.as_str() != "mod").collect();

    let collisions: BTreeSet<&str> = top_stems
        .intersection(&charts_stems)
        .map(|s| s.as_str())
        .collect();
    let expected: BTreeSet<&str> = TOP_LEVEL_WINS_COLLISIONS.iter().copied().collect();

    assert_eq!(
        collisions, expected,
        "src/ 直下と src/charts/ のステム衝突が TOP_LEVEL_WINS_COLLISIONS \
         と一致しません。新規衝突が発生した場合は resolve_page の解決順序が \
         妥当か確認したうえで定数を更新してください。"
    );
}

/// §3.3: Themes 108 部品を 4 バケットへ分割し、宣言済み台帳と双方向に
/// 一致すること。合計が nav 由来のページ数と一致することも検証する
/// （108 をハードコードしない）。
#[test]
fn wrap_state_partition_matches_the_declared_ledger() {
    let scan = scan_pre_styled_src(&pre_styled_ui_src_dir());
    let pages = themes_page_kebabs();
    let primitive_modules = primitive_module_names();
    let same_name_pages: BTreeSet<&str> = WRAPPED_SAME_NAME.iter().copied().collect();

    let mut actual_same_name = BTreeSet::new();
    let mut actual_cross_name: BTreeMap<String, String> = BTreeMap::new();
    let mut actual_doc_only: BTreeMap<String, String> = BTreeMap::new();
    let mut actual_pre_styled_only = BTreeSet::new();

    for page in &pages {
        let file = resolve_page(&scan, page);
        let mod_name = kebab_to_snake(page);

        if same_name_pages.contains(page.as_str()) {
            assert!(
                file.code_refs.contains(&mod_name),
                "`{page}` は WRAPPED_SAME_NAME 台帳に載っていますが、同名 \
                 headless モジュール `{mod_name}` へのコード委譲が見つかりません"
            );
            actual_same_name.insert(page.clone());
            continue;
        }

        let code_hits: Vec<&String> = file
            .code_refs
            .iter()
            .filter(|r| primitive_modules.contains(r.as_str()))
            .collect();
        if !code_hits.is_empty() {
            assert_eq!(
                code_hits.len(),
                1,
                "`{page}` のコード行が複数の Primitives モジュールへ委譲して \
                 いるように見えます（{code_hits:?}）。判別規約（§3.2）が \
                 単一委譲を前提としているため、想定外の状態です"
            );
            actual_cross_name.insert(page.clone(), code_hits[0].clone());
            continue;
        }

        let doc_hits: Vec<&String> = file
            .comment_refs
            .iter()
            .filter(|r| primitive_modules.contains(r.as_str()))
            .collect();
        if !doc_hits.is_empty() {
            assert_eq!(
                doc_hits.len(),
                1,
                "`{page}` の rustdoc が複数の Primitives モジュールを言及して \
                 いるように見えます（{doc_hits:?}）。判別規約（§3.2）が \
                 単一言及を前提としているため、想定外の状態です"
            );
            actual_doc_only.insert(page.clone(), doc_hits[0].clone());
            continue;
        }

        actual_pre_styled_only.insert(page.clone());
    }

    assert_eq!(
        actual_same_name,
        same_name_pages
            .into_iter()
            .map(String::from)
            .collect::<BTreeSet<_>>(),
        "WRAPPED_SAME_NAME バケットが宣言と一致しません"
    );

    let expected_cross_name: BTreeMap<String, String> = WRAPPED_CROSS_NAME
        .iter()
        .map(|(p, m)| (p.to_string(), m.to_string()))
        .collect();
    assert_eq!(
        actual_cross_name, expected_cross_name,
        "WRAPPED_CROSS_NAME バケットが宣言と一致しません"
    );

    let expected_doc_only: BTreeMap<String, String> = DOC_REFERENCE_ONLY
        .iter()
        .map(|(p, m)| (p.to_string(), m.to_string()))
        .collect();
    assert_eq!(
        actual_doc_only, expected_doc_only,
        "DOC_REFERENCE_ONLY バケットが宣言と一致しません。C ↔ D の移動は \
         多くの場合「欠陥」ではなく「台帳更新イベント」です（ファイル冒頭の \
         rustdoc コメントを参照）。実態を確認したうえで定数を更新し、PR 本文に \
         理由を書いてください。"
    );

    let expected_pre_styled_only: BTreeSet<String> =
        PRE_STYLED_ONLY.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        actual_pre_styled_only, expected_pre_styled_only,
        "PRE_STYLED_ONLY バケットが宣言と一致しません"
    );

    let total = actual_same_name.len()
        + actual_cross_name.len()
        + actual_doc_only.len()
        + actual_pre_styled_only.len();
    assert_eq!(
        total,
        pages.len(),
        "4 バケットの合計が site/nav.toml の Themes ページ数と一致しません"
    );
}

/// 受け入れ条件の中核: 「同名 Themes ページがあるのに同名 headless へ \
/// 委譲していない」部品が 0 件であることを恒久固定する
/// （`docs/policy/intentional-non-adoption.md` §3.25 が最も警戒する \
/// 独自実装の兆候）。
#[test]
fn no_same_name_themes_page_reimplements_its_primitive() {
    let scan = scan_pre_styled_src(&pre_styled_ui_src_dir());
    let pages = themes_page_kebabs();
    let primitive_kebabs: BTreeSet<String> = PRIMITIVES
        .iter()
        .map(|e| primitives_catalog::kebab_of(e.module))
        .collect();

    let mut reimplemented = Vec::new();
    for page in &pages {
        if !primitive_kebabs.contains(page) {
            continue;
        }
        let mod_name = kebab_to_snake(page);
        let file = resolve_page(&scan, page);
        if !file.code_refs.contains(&mod_name) {
            reimplemented.push(page.clone());
        }
    }

    assert!(
        reimplemented.is_empty(),
        "同名 Primitives 部品が存在するのに headless モジュールへコード \
         委譲していない Themes ページが見つかりました: {reimplemented:?}。\
         docs/policy/intentional-non-adoption.md §3.25 に反する独自実装の \
         可能性があります。"
    );
}

/// バケット B（WRAPPED_CROSS_NAME）のマッピングが宣言どおりであること
/// （`wrap_state_partition_matches_the_declared_ledger` の一部を単独でも
/// 検証できるよう分離）。
#[test]
fn cross_name_wrapper_targets_match_the_declared_mapping() {
    let scan = scan_pre_styled_src(&pre_styled_ui_src_dir());
    for (page, target_module) in WRAPPED_CROSS_NAME {
        let file = resolve_page(&scan, page);
        assert!(
            file.code_refs.contains(*target_module),
            "`{page}` は WRAPPED_CROSS_NAME で `{target_module}` への委譲を \
             宣言していますが、コード行に見つかりません"
        );
        let mod_name = kebab_to_snake(page);
        assert!(
            !file.code_refs.contains(&mod_name) || mod_name == *target_module,
            "`{page}` が同名モジュール `{mod_name}` へも委譲しています。\
             WRAPPED_SAME_NAME への移動を検討してください"
        );
    }
}

/// バケット C（DOC_REFERENCE_ONLY）が「コード委譲なし・rustdoc 言及あり」の
/// 両方向を満たすこと。
#[test]
fn doc_reference_only_pages_have_no_code_level_delegation() {
    let scan = scan_pre_styled_src(&pre_styled_ui_src_dir());
    for (page, target_module) in DOC_REFERENCE_ONLY {
        let file = resolve_page(&scan, page);
        assert!(
            !file.code_refs.contains(*target_module),
            "`{page}` は DOC_REFERENCE_ONLY 宣言ですが、`{target_module}` への \
             コード委譲が見つかりました。WRAPPED_CROSS_NAME への移動を \
             検討してください"
        );
        assert!(
            file.comment_refs.contains(*target_module),
            "`{page}` は DOC_REFERENCE_ONLY 宣言ですが、`{target_module}` への \
             rustdoc 言及が見つかりません。PRE_STYLED_ONLY への移動を \
             検討してください"
        );
    }
}

/// §3.4（受け入れ条件 3）: pre-styled-ui のどこからもコード委譲されていない
/// headless 部品を求め、`HEADLESS_UNWRAPPED` と一致することを検証する。
#[test]
fn headless_primitives_unwrapped_by_pre_styled_match_the_ledger() {
    let scan = scan_pre_styled_src(&pre_styled_ui_src_dir());

    let mut referenced: BTreeSet<&str> = BTreeSet::new();
    for file in scan.top_level.values().chain(scan.charts.values()) {
        for r in &file.code_refs {
            referenced.insert(r.as_str());
        }
    }

    let primitive_modules = primitive_module_names();
    let unwrapped: BTreeSet<&str> = primitive_modules
        .iter()
        .filter(|m| !referenced.contains(*m))
        .copied()
        .collect();

    let expected: BTreeSet<&str> = HEADLESS_UNWRAPPED.iter().copied().collect();
    assert_eq!(
        unwrapped, expected,
        "pre-styled-ui からコード委譲されていない headless 部品が \
         HEADLESS_UNWRAPPED（{HEADLESS_UNWRAPPED:?}）と一致しません"
    );
}

/// §3.4: `HEADLESS_UNWRAPPED` ⊆ `PRIMITIVES_WITHOUT_THEMES_PAGE`、かつ \
/// `PRIMITIVES_WITHOUT_THEMES_PAGE ∖ HEADLESS_UNWRAPPED == {"field"}`、かつ \
/// `field` の参照元が `FIELD_CROSS_WRAPPERS` と一致することを検証する。\
/// 2 つの台帳（ページレベルの `PRIMITIVES_WITHOUT_THEMES_PAGE` と、本ファイルの \
/// コードレベル `HEADLESS_UNWRAPPED`）が独立にドリフトして片方が黙って嘘を \
/// つく事故を防ぐ。
#[test]
fn unwrapped_ledger_is_consistent_with_primitives_without_themes_page() {
    let unwrapped: BTreeSet<&str> = HEADLESS_UNWRAPPED.iter().copied().collect();
    let without_page: BTreeSet<&str> = PRIMITIVES_WITHOUT_THEMES_PAGE.iter().copied().collect();

    assert!(
        unwrapped.is_subset(&without_page),
        "HEADLESS_UNWRAPPED（{unwrapped:?}）は \
         PRIMITIVES_WITHOUT_THEMES_PAGE（{without_page:?}）の部分集合である \
         はずです"
    );

    let diff: BTreeSet<&str> = without_page.difference(&unwrapped).copied().collect();
    let expected_diff: BTreeSet<&str> = ["field"].into_iter().collect();
    assert_eq!(
        diff, expected_diff,
        "PRIMITIVES_WITHOUT_THEMES_PAGE ∖ HEADLESS_UNWRAPPED は \
         {{\"field\"}} のみであるはずです（`field` は別名ラップ済みのため \
         HEADLESS_UNWRAPPED には含めない）"
    );

    let scan = scan_pre_styled_src(&pre_styled_ui_src_dir());
    let mut field_referrers: BTreeSet<&str> = BTreeSet::new();
    for (stem, file) in scan.top_level.iter().chain(scan.charts.iter()) {
        if file.code_refs.contains("field") {
            field_referrers.insert(stem.as_str());
        }
    }

    let expected_referrers: BTreeSet<&str> = FIELD_CROSS_WRAPPERS.iter().copied().collect();
    assert_eq!(
        field_referrers, expected_referrers,
        "`field` へコード委譲しているモジュールが FIELD_CROSS_WRAPPERS と \
         一致しません"
    );
}

/// §3.6: `crates/pre-styled-ui/src/` の全モジュールが「Themes ページを持つ \
/// 部品モジュール」か「非ページモジュール（`NON_PAGE_TOP_LEVEL` / \
/// `NON_PAGE_CHARTS`）」のちょうど一方に属すること。
#[test]
fn every_pre_styled_module_is_either_a_page_or_declared_non_page() {
    let scan = scan_pre_styled_src(&pre_styled_ui_src_dir());
    let pages = themes_page_kebabs();
    let themes_modnames: BTreeSet<String> = pages.iter().map(|p| kebab_to_snake(p)).collect();

    let non_page_top_actual: BTreeSet<String> = scan
        .top_level
        .keys()
        .filter(|s| !themes_modnames.contains(*s))
        .cloned()
        .collect();
    let expected_non_page_top: BTreeSet<String> =
        NON_PAGE_TOP_LEVEL.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        non_page_top_actual, expected_non_page_top,
        "src/ 直下の非ページモジュールが NON_PAGE_TOP_LEVEL と一致しません \
         （新規モジュール追加時は Themes ページ新設漏れの可能性があります）"
    );

    let non_page_charts_actual: BTreeSet<String> = scan
        .charts
        .keys()
        .filter(|s| s.as_str() != "mod")
        .filter(|s| {
            !themes_modnames.contains(*s) || TOP_LEVEL_WINS_COLLISIONS.contains(&s.as_str())
        })
        .cloned()
        .collect();
    let expected_non_page_charts: BTreeSet<String> =
        NON_PAGE_CHARTS.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        non_page_charts_actual, expected_non_page_charts,
        "src/charts/ の非ページモジュールが NON_PAGE_CHARTS と一致しません"
    );

    assert_eq!(
        scan.top_level.len(),
        109,
        "src/*.rs の総数が想定と異なります（イシュー #1684 で field.rs \
         を新設し 108 → 109。field は Themes ページを持たない \
         NON_PAGE_TOP_LEVEL 扱いの暫定台帳、#1685 でページ登録次第 \
         WRAPPED_CROSS_NAME 等の該当バケットへ移す）"
    );
    assert_eq!(
        scan.charts.len(),
        14,
        "src/charts/*.rs の総数が想定と異なります"
    );
}

// ---------------------------------------------------------------------
// スキャナ自己検証（TempDir フィクスチャ、`tests/primitives_catalog.rs`
// と同方針。外部クレート `tempfile` は追加しない、REQ-3）
// ---------------------------------------------------------------------

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = scratch_root().join(format!(
            "fandhe-frontend-docs-site-wrap-state-{tag}-{}-{unique}",
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

#[test]
fn scanner_rejects_undeclared_subdirectory() {
    let dir = TempDir::new("undeclared-subdir");
    std::fs::write(dir.0.join("foo.rs"), "// fixture\n").expect("failed to write foo.rs");
    std::fs::create_dir_all(dir.0.join("widgets")).expect("failed to create widgets/");

    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| scan_pre_styled_src(&dir.0)));
    assert!(
        result.is_err(),
        "未宣言サブディレクトリを持つフィクスチャに対して panic しませんでした"
    );
}

#[test]
fn scanner_rejects_deeper_nesting() {
    let dir = TempDir::new("deeper-nesting");
    std::fs::create_dir_all(dir.0.join("charts/sub")).expect("failed to create charts/sub/");
    std::fs::write(dir.0.join("charts/mod.rs"), "// fixture\n").expect("failed to write mod.rs");

    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| scan_pre_styled_src(&dir.0)));
    assert!(
        result.is_err(),
        "charts/ 配下の 2 段目ネストを持つフィクスチャに対して panic しませんでした"
    );
}

#[test]
fn scanner_rejects_non_rs_entry() {
    let dir = TempDir::new("non-rs-entry");
    std::fs::write(dir.0.join("README.md"), "not rust\n").expect("failed to write README.md");

    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| scan_pre_styled_src(&dir.0)));
    assert!(
        result.is_err(),
        "非 .rs エントリを持つフィクスチャに対して panic しませんでした"
    );
}

#[test]
fn scanner_rejects_block_comment() {
    let dir = TempDir::new("block-comment");
    std::fs::write(
        dir.0.join("widget.rs"),
        "/* block comment */\nfn root() {}\n",
    )
    .expect("failed to write widget.rs");

    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| scan_pre_styled_src(&dir.0)));
    assert!(
        result.is_err(),
        "ブロックコメントを含むフィクスチャに対して panic しませんでした"
    );
}

/// 上記の対照: フラット + 宣言済みネストのみのフィクスチャは正常に走査でき、
/// 上記のテストが常時 panic する実装になっていないことを保証する。
#[test]
fn scanner_accepts_flat_and_declared_nested_fixture() {
    let dir = TempDir::new("flat-and-nested");
    std::fs::write(
        dir.0.join("badge.rs"),
        "//! badge は headless を持たない\npub fn root() {}\n",
    )
    .expect("failed to write badge.rs");
    std::fs::write(
        dir.0.join("input.rs"),
        "//! [`fandhe_frontend_headless_ui::field`] を包む\npub use fandhe_frontend_headless_ui::field::input;\n",
    )
    .expect("failed to write input.rs");
    std::fs::create_dir_all(dir.0.join("charts")).expect("failed to create charts/");
    std::fs::write(
        dir.0.join("charts/mod.rs"),
        "// no headless reference\npub fn root() {}\n",
    )
    .expect("failed to write charts/mod.rs");
    std::fs::write(
        dir.0.join("charts/bar_chart.rs"),
        "// bar chart\npub fn root() {}\n",
    )
    .expect("failed to write charts/bar_chart.rs");

    let scan = scan_pre_styled_src(&dir.0);

    assert_eq!(scan.top_level.len(), 2);
    assert_eq!(scan.charts.len(), 2);
    assert!(scan.top_level["badge"].code_refs.is_empty());
    assert!(scan.top_level["badge"].comment_refs.is_empty());
    assert!(scan.top_level["input"].code_refs.contains("field"));
    assert!(scan.top_level["input"].comment_refs.contains("field"));
}

/// イシュー #1064 の Bugbot 指摘（PR #1096）に対する回帰テスト:
/// `#[cfg(test)] mod tests { ... }` 内にしか `fandhe_frontend_headless_ui::
/// <ident>` の参照を持たないファイルは、`code_refs` に当該 ident を含んで
/// はならない（本番コードが委譲しているとの誤判定を防ぐ）。`radio_card.rs`
/// が該当した実例のうち、テスト側の import 行のみを模したフィクスチャ。
/// 素朴な「非コメント行はすべて証跡」判定では本テストは失敗する
/// （このコメント自体が rustdoc 言及として `comment_refs` に載ることも
/// あわせて確認し、DOC_REFERENCE_ONLY 相当への分類が保たれることを示す）。
#[test]
fn scanner_excludes_cfg_test_module_body_from_code_refs() {
    let dir = TempDir::new("cfg-test-only-reference");
    std::fs::write(
        dir.0.join("radio_card.rs"),
        "//! [`fandhe_frontend_headless_ui::radio_group::RadioGroup`] を \
         rustdoc でのみ言及する（本番コードは独自実装）。\n\
         pub fn root() {}\n\
         \n\
         #[cfg(test)]\n\
         mod tests {\n\
         \x20   use fandhe_frontend_headless_ui::radio_group::RadioGroup;\n\
         \n\
         \x20   #[test]\n\
         \x20   fn uses_it(r: RadioGroup) {\n\
         \x20       let _ = r;\n\
         \x20   }\n\
         }\n",
    )
    .expect("failed to write radio_card.rs");

    let scan = scan_pre_styled_src(&dir.0);

    assert!(
        !scan.top_level["radio_card"]
            .code_refs
            .contains("radio_group"),
        "#[cfg(test)] モジュール内の参照が code_refs へ漏れています \
         （WRAPPED_CROSS_NAME への誤分類を招く欠陥が再発しています）"
    );
    assert!(
        scan.top_level["radio_card"]
            .comment_refs
            .contains("radio_group"),
        "rustdoc 言及（comment_refs）は引き続き検出されるべきです \
         （DOC_REFERENCE_ONLY 分類の根拠）"
    );
}

/// `#[cfg(test)]` ブロック除外が兄弟の通常コードへ波及しない（除外範囲が
/// 波括弧の深さで正しく閉じる）ことを確認する。`mod tests` の後ろに
/// 通常のコード委譲を続けて配置し、後続の委譲が code_refs へ検出される
/// ことを検証する（深さ追跡の境界がずれて過剰に除外していないか）。
#[test]
fn scanner_resumes_code_ref_detection_after_cfg_test_block_closes() {
    let dir = TempDir::new("cfg-test-then-real-delegation");
    std::fs::write(
        dir.0.join("widget.rs"),
        "pub fn root() {}\n\
         \n\
         #[cfg(test)]\n\
         mod tests {\n\
         \x20   use fandhe_frontend_headless_ui::radio_group::RadioGroup;\n\
         \n\
         \x20   fn nested() {\n\
         \x20       let _x = 1;\n\
         \x20   }\n\
         }\n\
         \n\
         pub use fandhe_frontend_headless_ui::field::input;\n",
    )
    .expect("failed to write widget.rs");

    let scan = scan_pre_styled_src(&dir.0);

    assert!(
        !scan.top_level["widget"].code_refs.contains("radio_group"),
        "cfg(test) ブロック内の参照が漏れています"
    );
    assert!(
        scan.top_level["widget"].code_refs.contains("field"),
        "cfg(test) ブロック終了後の通常コードの委譲が検出されていません \
         （除外範囲の境界判定が過剰に広がっています）"
    );
}
