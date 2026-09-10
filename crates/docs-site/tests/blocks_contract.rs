//! Blocks（`/blocks/`）ページの契約テスト（イシュー #2088）。
//!
//! `build_site` で実サイトをビルドし、生成物に対して以下を固定する。
//! いずれも設計文書 `docs/design/docs-site-blocks-section.md` と実装計画
//! §2.3/§2.6/§2.7/§6 が定める不変条件であり、後続イシュー #2089〜#2095 が
//! block を追加する際もそのまま継承される。
//!
//! - `/blocks/login-01/` に `class="blocks-demo"` と `pre-styled-ui.css` /
//!   `blocks.css` の `<link>` があり、`/blocks/` 索引ページには無い
//! - block ページ全体に `<form` が無い（無 JS 制約、`crate::layout` 参照）
//! - `src="data:` が無い（A05、`showcase::image_demo_svg` と同じ判断軸）
//! - 節順序が H1 → Demo → 使用部品 → Rust コード
//! - 使用部品リンクが各 `Part.path` を指す
//! - 合成関数の出力を直接 `render()` した結果に未エスケープ `<script` が
//!   無い（XSS 回帰）
//! - `blocks::stylesheet()` が `.blocks-demo` の `overflow-x` 宣言を含む
//! - `login-01` の `data-blocks-login-01-card`/`-field`/`-submit` 属性が
//!   生成 HTML に実際に出力され、`blocks::stylesheet()` にも対応する
//!   `[data-blocks-login-01-*]` セレクタが存在する（イシュー #2088 PR #2277
//!   codex-review P1 / Cursor Bugbot 指摘の是正: `card::root`/`field::root`/
//!   `button::button` は `drop_class_attr` で呼び出し側 `class` を除去する
//!   ため、これら 3 パーツの CSS フックは `class` ではなく `data-*` 属性で
//!   渡す契約に変更した）

use std::path::{Path, PathBuf};

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::blocks;
use fandhe_frontend_docs_site::build::build_site;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo_root should resolve from CARGO_MANIFEST_DIR")
}

/// `tests/site_build.rs::scratch_root`/`TempDir` と同じ規約
/// （`CARGO_TARGET_TMPDIR` 固定配置、`/tmp` へフォールバックしない）。
fn scratch_root() -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    let _ = std::fs::create_dir_all(&root);
    root
}

struct TempDir(PathBuf);

/// プロセス内で `TempDir::new` が呼ばれるたびに単調増加する値。`Drop` が
/// 実際にディレクトリを削除するようになった（旧 `std::mem::forget` リーク
/// 運用の是正、イシュー #2088 PR #2277 codex-review P2 指摘）ことで、
/// 同一テストバイナリ内の複数スレッドが `build_real_site()`（同一 tag
/// `"real-site"`）をほぼ同時刻に呼ぶと、ナノ秒精度の時刻だけでは衝突し得る
/// （実測: `cargo test`（既定並列）で 4 テスト中 1 件が
/// `NotFound: blocks/login-01/index.html` で偶発 FAIL、`--test-threads=1`
/// では常に成功。2 スレッドが同じ `(pid, nanos)` でディレクトリ名を得ると
/// 両者が同じパスへ書き込み・先に終わった側の `Drop` がもう片方の生成物を
/// 削除してしまうため）。pid・時刻に加えプロセス内カウンタを混ぜ、
/// 同一プロセス内での衝突を構造的に無くす。
static TEMP_DIR_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let seq = TEMP_DIR_SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let path = scratch_root().join(format!(
            "fandhe-frontend-docs-site-blocks-contract-{tag}-{}-{unique}-{seq}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("create temp dir for blocks_contract.rs test");
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// `TempDir` を `&Path` として透過的に扱えるようにする（`out.join(...)` 等の
/// 呼び出し元を変えずに所有権だけをテスト関数の戻り値へ持ち出すため）。
impl std::ops::Deref for TempDir {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

/// 生成物のディレクトリ「所有者」（`TempDir`）をそのまま返す。呼び出し元が
/// 戻り値を保持している間だけ生成物が生存し、テスト関数終了時に `Drop` で
/// 確実に削除される（以前の `std::mem::forget` によるリーク運用を是正）。
fn build_real_site() -> TempDir {
    let out = TempDir::new("real-site");
    build_site(&repo_root(), &out.0).expect("real site/nav.toml should build cleanly");
    out
}

#[test]
fn login_01_page_wires_demo_class_and_both_stylesheets_index_page_does_not() {
    let out = build_real_site();

    let login_html = std::fs::read_to_string(out.join("blocks/login-01/index.html"))
        .expect("blocks/login-01/index.html should be generated");
    assert!(
        login_html.contains("class=\"blocks-demo blocks-login-01\""),
        "login-01 page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        login_html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "login-01 page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        login_html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "login-01 page should link the Blocks-specific stylesheet"
    );
    // codex-review P1 是正（イシュー #2088 PR #2277 指摘）: card::root/
    // field::root/button::button は drop_class_attr で呼び出し側 class を
    // 除去するため、blocks.css のレイアウト規則は class ではなく data-*
    // 属性へ張り替えた（login_01.rs の実装コメント参照）。生成 HTML に
    // その属性が実際に出力され、blocks.css 側にも対応するセレクタが
    // 存在することの両方を固定し、「CSS フックが黙って効かない」再発を防ぐ。
    for hook in [
        "data-blocks-login-01-card=\"\"",
        "data-blocks-login-01-field=\"\"",
        "data-blocks-login-01-submit=\"\"",
    ] {
        assert!(
            login_html.contains(hook),
            "login-01 page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        "[data-blocks-login-01-card]",
        "[data-blocks-login-01-field]",
        "[data-blocks-login-01-submit]",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }

    let index_html = std::fs::read_to_string(out.join("blocks/index.html"))
        .expect("blocks/index.html should be generated");
    assert!(
        !index_html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "Blocks index page should not link pre-styled-ui.css (no generated content)"
    );
    assert!(
        !index_html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "Blocks index page should not link blocks.css (no Demo section)"
    );
}

#[test]
fn block_pages_never_contain_a_form_element_or_data_uri() {
    let out = build_real_site();
    let mut relatives: Vec<String> = vec!["blocks/index.html".to_string()];
    relatives.extend(blocks::BLOCKS.iter().map(|block| {
        let kebab = block
            .path
            .trim_start_matches("/blocks/")
            .trim_end_matches('/');
        format!("blocks/{kebab}/index.html")
    }));
    for relative in relatives {
        let html = std::fs::read_to_string(out.join(&relative))
            .unwrap_or_else(|e| panic!("{relative}: {e}"));
        assert!(
            !html.contains("<form"),
            "{relative} should never contain a <form> element (no-JS implicit submit avoidance)"
        );
        assert!(
            !html.contains("src=\"data:"),
            "{relative} should never contain a data: URI src"
        );
    }
}

/// dashboard-01 の Demo 固有 CSS フック（sidebar/inset/stats 等）が実際に
/// 生成 HTML へ出力され、`blocks::stylesheet()` にも対応するセレクタが
/// 存在することを固定する（login-01 のイシュー #2088 codex-review 是正と
/// 同型: `sidebar` の全パーツ・`card::root`・`stat::root` 等は
/// `drop_class_attr` で呼び出し側 `class` を除去するため、CSS フックは
/// `class` ではなく `data-*` 属性で渡す契約になっている、
/// `crates/docs-site/src/blocks/dashboard_01.rs` モジュール doc参照）。
#[test]
fn dashboard_01_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/dashboard-01/index.html"))
        .expect("blocks/dashboard-01/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-dashboard-01\""),
        "dashboard-01 page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "dashboard-01 page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "dashboard-01 page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-dashboard-01-card=\"\"",
        "data-blocks-dashboard-01-header=\"\"",
        "data-blocks-dashboard-01-chart=\"\"",
        "data-blocks-dashboard-01-table=\"\"",
        "data-blocks-dashboard-01-header-link=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "dashboard-01 page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        "[data-blocks-dashboard-01-card]",
        "[data-blocks-dashboard-01-header]",
        "[data-blocks-dashboard-01-chart]",
        "[data-blocks-dashboard-01-table]",
        "[data-blocks-dashboard-01-header-link]",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }
}

/// dashboard-01 の合成部品（sidebar/stat/gradient area-chart/toggle-group/
/// tabs/table）が anatomy の `data-*` として実際に出力されていることを
/// 固定する。
#[test]
fn dashboard_01_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/dashboard-01/index.html"))
        .expect("blocks/dashboard-01/index.html should be generated");
    for needle in [
        "data-scope=\"sidebar\"",
        "data-variant=\"inset\"",
        "data-scope=\"stat\"",
        "<linearGradient",
        "data-range=\"90d\"",
        "aria-label=\"Select date range\"",
        "aria-sort=\"ascending\"",
        "data-selected",
        "data-scope=\"tabs\"",
    ] {
        assert!(
            html.contains(needle),
            "dashboard-01 page should contain {needle}"
        );
    }
    assert!(
        !html.contains("<form"),
        "dashboard-01 should never contain a <form>"
    );
    assert!(
        !html.contains("href=\"#\""),
        "dashboard-01 should never contain a dead href=\"#\" link"
    );
    assert!(
        !html.contains("aria-labelledby=\"blocks-dashboard-01-range-label\""),
        "dashboard-01 should not reference a range-toggle label id that has no matching element \
         (regression: codex-review P1 / Cursor Bugbot Low on PR #2280)"
    );
}

#[test]
fn login_01_page_orders_h1_then_demo_then_used_parts_then_rust_code() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/login-01/index.html"))
        .expect("blocks/login-01/index.html should be generated");

    // イシュー #1080: `main.docs-main` の折りたたみ目次
    // （`nav.docs-toc-inline`）は本文（`article.docs-content`）より前に
    // 置かれ、見出しテキスト（"Demo"/"使用部品"/"Rust コード"）を先出し
    // する（`crate::layout::docs_page_with_assets` 参照）。節順序の検証は
    // この重複箇所を含めず、`article.docs-content` の開始位置より後ろだけを
    // 対象にする。
    let content_start = html
        .find(r#"class="docs-content""#)
        .expect("page should have an article.docs-content");
    let content = &html[content_start..];

    // 見出し要素そのもの（`<h2>テキスト</h2>`）でしか一致しない部分文字列で
    // 探す。素の Markdown 本文（例: 導入段落中の「Rust コードで書いて
    // ください」）に見出しと同じ語が偶然出現しても誤検知しないため
    // （実測で発生した false positive、`site/blocks/login-01.md` 参照）。
    let h1_pos = content.find("<h1").expect("page should have an h1");
    let demo_pos = content
        .find(">Demo</h2>")
        .expect("page should have a Demo heading");
    let parts_pos = content
        .find(">使用部品</h2>")
        .expect("page should have a 使用部品 heading");
    let rust_pos = content
        .find(">Rust コード</h2>")
        .expect("page should have a Rust コード heading");

    assert!(
        h1_pos < demo_pos && demo_pos < parts_pos && parts_pos < rust_pos,
        "section order should be H1 -> Demo -> 使用部品 -> Rust コード, got positions {h1_pos}/{demo_pos}/{parts_pos}/{rust_pos}"
    );
}

#[test]
fn login_01_used_parts_links_point_at_each_declared_part_path() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/login-01/index.html"))
        .expect("blocks/login-01/index.html should be generated");

    let block = blocks::BLOCKS
        .iter()
        .find(|b| b.path == "/blocks/login-01/")
        .expect("login-01 should be registered");
    assert!(!block.parts.is_empty(), "login-01 should declare parts");
    for part in block.parts {
        let expected_href = format!("href=\"/fandhe-frontend{}\"", part.path);
        assert!(
            html.contains(&expected_href),
            "login-01 page should link to used part {} via {expected_href:?}",
            part.path
        );
    }
}

#[test]
fn demo_output_never_leaks_an_unescaped_script_tag() {
    for block in blocks::BLOCKS {
        let html = render(&(block.demo)());
        assert!(
            !html.contains("<script"),
            "block {} demo output should never contain an unescaped <script tag",
            block.path
        );
    }
}

#[test]
fn blocks_stylesheet_declares_demo_frame_overflow() {
    let sheet = blocks::stylesheet().expect("blocks::stylesheet should build");
    assert!(sheet.as_css().contains(".blocks-demo"));
    assert!(sheet.as_css().contains("overflow-x: auto"));
}

/// 受け入れ条件 f の実効化: 現行 Demo（`login-01`）は `<script` を含む
/// 入力を一切持たないため `demo_output_never_leaks_an_unescaped_script_tag`
/// は空虚な検証にとどまる。`crates/docs-site/src/blocks/` 配下のソース
/// そのものを走査し、`raw_html()`（REQ-1 迂回）・HTML 文字列直接組み立て
/// （`format!("<...`）を使っていないことを固定する
/// （`tests/primitive_specs_1026.rs::primitive_specs_source_does_not_use_raw_html`
/// と同型のガード。後続イシュー #2089〜#2095 が block を追加してもこの
/// 不変条件が自動的に効く）。
#[test]
fn blocks_source_does_not_use_raw_html_or_build_html_strings() {
    fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
    fn code_lines_without_comments(path: &Path) -> String {
        let src = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("{} should be readable: {e}", path.display()));
        src.lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    let dir = repo_root().join("crates/docs-site/src/blocks");
    let mut files = Vec::new();
    collect_rs_files(&dir, &mut files);
    assert!(
        !files.is_empty(),
        "crates/docs-site/src/blocks/ should contain at least one .rs file to guard"
    );
    for path in &files {
        let code_only = code_lines_without_comments(path);
        assert!(
            !code_only.contains("raw_html"),
            "{} must not use raw_html() (REQ-1 escape bypass) in code (non-comment) lines",
            path.display()
        );
        assert!(
            !code_only.contains("format!(\"<"),
            "{} must not build HTML strings directly (use the node-tree API)",
            path.display()
        );
    }
}
