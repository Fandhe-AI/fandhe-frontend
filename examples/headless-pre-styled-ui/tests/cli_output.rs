//! `examples/headless-pre-styled-ui` の CLI ブラックボックス統合テスト
//! （イシュー #552）。
//!
//! `src/main.rs` 内の `#[cfg(test)]` ユニットテストが anatomy・`data-state`・
//! ARIA 属性・既定エスケープをライブラリレベルで固定するのに対し、本ファイル
//! は `examples/ssg-blog/tests/ssg_output.rs` と同じ方針でビルド済みバイナリ
//! をサブプロセス起動し、実際に `cargo run` した利用者が目にする `dist/`
//! 出力（ファイル配置・エスケープ済み HTML・静的 CSS 同梱）を検証する。

use std::path::PathBuf;
use std::process::Command;

/// テスト専用の一時ディレクトリ。`Drop` でベストエフォート削除する
/// （`examples/ssg-blog/tests/ssg_output.rs::TempDir` と同じ方針。`tempfile`
/// 等の外部クレートを追加しない、REQ-3）。
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        // cargo が `CARGO_TARGET_TMPDIR` を設定するのはテストバイナリの
        // コンパイル時のみ（Cargo Book）であり、実行時 `std::env::var` 参照は
        // 常に失敗する。既定はコンパイル時に確定する
        // `env!("CARGO_TARGET_TMPDIR")`（`<target>/tmp` 配下。本サンプルは
        // root workspace から意図的に切り離された独立 `[workspace]` のため、
        // ここでの `<target>` は `examples/headless-pre-styled-ui/target`）を
        // 使い、`/tmp` へは一切フォールバックしない（イシュー #637/#658）。
        // 実行時 env による明示上書きは引き続き許容する。
        let root = std::env::var("CARGO_TARGET_TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
        let _ = std::fs::create_dir_all(&root);
        let path = root.join(format!(
            "fandhe-frontend-example-headless-pre-styled-ui-test-{tag}-{}-{unique}",
            std::process::id()
        ));
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// `src/main.rs` のバイナリを一意な一時ディレクトリを `current_dir` として
/// 起動し、生成された `dist/` を含むディレクトリを返す。
fn run_cli_in_scratch_dir(tag: &str) -> TempDir {
    let scratch = TempDir::new(tag);
    std::fs::create_dir_all(&scratch.0).expect("failed to create scratch dir");

    let output = Command::new(env!(
        "CARGO_BIN_EXE_fandhe-frontend-example-headless-pre-styled-ui"
    ))
    .current_dir(&scratch.0)
    .output()
    .expect("binary should spawn and run to completion");
    assert!(
        output.status.success(),
        "CLI should exit 0: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    scratch
}

/// `cargo run` で `dist/index.html`・`dist/assets/ui.css` が生成されること
/// を固定する。
#[test]
fn cli_generates_expected_dist_files() {
    let scratch = run_cli_in_scratch_dir("dist-files");
    let dist = scratch.0.join("dist");

    assert!(dist.join("index.html").is_file());
    assert!(dist.join("assets").join("ui.css").is_file());
}

/// 全コンポーネントの `data-scope` セレクタが出力へ含まれることを固定する
/// （受け入れ条件(a): anatomy の実演。pre-styled-ui 統合後は headless 系・
/// pre-styled 系両層のコンポーネントを対象とする）。
#[test]
fn cli_output_includes_all_component_scopes() {
    let scratch = run_cli_in_scratch_dir("component-scopes");
    let body = std::fs::read_to_string(scratch.0.join("dist/index.html"))
        .expect("index.html should be readable");

    for scope in [
        // pre-styled-ui の headless ラッパー経由（マークアップは headless 層）
        "data-scope=\"tabs\"",
        "data-scope=\"accordion\"",
        "data-scope=\"dialog\"",
        "data-scope=\"menu\"",
        "data-scope=\"select\"",
        "data-scope=\"popover\"",
        "data-scope=\"tooltip\"",
        // pre-styled-ui の単純 styled 部品
        "data-scope=\"button\"",
        "data-scope=\"badge\"",
        "data-scope=\"card\"",
        "data-scope=\"alert\"",
        "data-scope=\"spinner\"",
        // pre-styled-ui の styled root（variant 付与、#684・PR #719）
        "data-scope=\"switch\"",
        "data-scope=\"radio-group\"",
        "data-scope=\"avatar\"",
    ] {
        assert!(body.contains(scope), "missing {scope} in dist/index.html");
    }
}

/// Menu / Select / Popover / Tooltip（ラッパー第 1 弾 #551・第 2 弾 #664）の
/// ARIA/`data-state` が `dist/index.html` へ現れることを固定する（受け入れ
/// 条件(a) の拡張、closed のまま実演する既定方針の回帰確認）。
#[test]
fn cli_output_includes_menu_select_popover_tooltip_aria_attrs() {
    let scratch = run_cli_in_scratch_dir("overlay-aria");
    let body = std::fs::read_to_string(scratch.0.join("dist/index.html"))
        .expect("index.html should be readable");

    assert!(body.contains(r#"aria-haspopup="menu""#));
    assert!(body.contains(r#"role="listbox""#));
    assert!(body.contains(r#"role="tooltip""#));
    assert!(body.contains(r#"aria-selected="true""#));
}

/// pre-styled-ui の recipe 生成クラスが HTML 側（variant クラス）へ現れる
/// ことを固定する（統合後の styled 部品実演の回帰）。
#[test]
fn cli_output_includes_pre_styled_variant_classes() {
    let scratch = run_cli_in_scratch_dir("variant-classes");
    let body = std::fs::read_to_string(scratch.0.join("dist/index.html"))
        .expect("index.html should be readable");

    assert!(body.contains("fd-button--variant-solid"));
    assert!(body.contains("fd-button--variant-outline"));
    assert!(body.contains("fd-badge--variant-solid"));
    assert!(body.contains("fd-alert--status-warning"));
    // Avatar の styled root（size/shape variant、#684/#689）の回帰確認。
    assert!(body.contains("fd-avatar--size-md"));
    assert!(body.contains("fd-avatar--shape-circle"));
    // Switch / RadioGroup の styled root（size/palette variant、pre-styled-ui
    // 0.5.0・PR #719 で追加。イシュー #728 で 0.5.0 へ追随）の回帰確認。
    assert!(body.contains("fd-switch--size-md"));
    assert!(body.contains("fd-switch--color-palette-accent"));
    assert!(body.contains("fd-radio-group--size-md"));
    assert!(body.contains("fd-radio-group--color-palette-accent"));
}

/// `dist/assets/ui.css` がテーマトークン（`Theme::default`）・pre-styled
/// recipe CSS・ページ骨格のみの手書き CSS の 3 系統すべてを含み、`<` を含まない
/// （StyleSheet の fail-closed 検証を通過した）ことを固定する。
#[test]
fn cli_output_css_aggregates_theme_recipes_and_manual_css() {
    let scratch = run_cli_in_scratch_dir("css-aggregation");
    let css = std::fs::read_to_string(scratch.0.join("dist/assets/ui.css"))
        .expect("ui.css should be readable");

    // 1. テーマトークン
    assert!(css.contains("--fandhe-color-accent"));
    // 2. pre-styled recipe（headless ラッパー分 + styled root 分 + 単純
    //    styled 部品分。ラッパーは `fd-*` variant クラスを持たない slot
    //    recipe のため、variant クラス検証の新部品分拡張として data-scope
    //    セレクタで確認する）
    assert!(css.contains(r#"[data-scope="tabs"][data-part="trigger"]"#));
    assert!(css.contains(r#"[data-scope="select"][data-part="trigger"]"#));
    assert!(css.contains(r#"[data-scope="menu"][data-part="item"]"#));
    assert!(css.contains(r#"[data-scope="popover"][data-part="content"]"#));
    assert!(css.contains(r#"[data-scope="tooltip"][data-part="content"]"#));
    assert!(css.contains(r#"[data-scope="switch"][data-part="control"]"#));
    assert!(css.contains(r#"[data-scope="radio-group"][data-part="item-control"]"#));
    assert!(css.contains(r#"[data-scope="avatar"][data-part="root"]"#));
    assert!(css.contains(".fd-button--variant-solid"));
    // 3. ページ骨格のみの手書き CSS（コンポーネント CSS は v0.4.0 で全部品
    //    recipe 提供となったため撤去済み、#689）
    assert!(css.contains(".showcase-row"));
    // StyleSheet の不変条件（`<style>` 文脈でも安全）
    assert!(!css.contains('<'));
}

/// 全ページに `@view-transition { navigation: auto; }` が含まれることを
/// 固定する（`layout()` が出力する契約）。
#[test]
fn cli_output_includes_view_transition_style() {
    let scratch = run_cli_in_scratch_dir("view-transition");
    let body = std::fs::read_to_string(scratch.0.join("dist/index.html"))
        .expect("index.html should be readable");

    assert!(body.contains("<style>@view-transition { navigation: auto; }</style>"));
    assert!(body.starts_with("<!DOCTYPE html>"));
}

/// 既定エスケープ回帰（REQ-1・OWASP A03）: `<script>` ペイロードを含む
/// トリガーラベルが実体参照化されて出力され、生の `<script>` タグとしては
/// 現れないことを CLI 経由で固定する。
#[test]
fn cli_escapes_xss_payload() {
    let scratch = run_cli_in_scratch_dir("xss-payload");
    let body = std::fs::read_to_string(scratch.0.join("dist/index.html"))
        .expect("index.html should be readable");

    assert!(!body.contains("<script>alert"));
    assert!(body.contains("&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"));
}
