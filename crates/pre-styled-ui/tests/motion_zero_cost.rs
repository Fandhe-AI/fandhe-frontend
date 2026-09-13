//! イシュー #2416「motion 拡張の Cargo feature 化と無効時ゼロコスト契約
//! テストを実装する」の fail-closed 契約テスト。
//!
//! `docs/design/motion-reference-adoption-policy.md` §7「ゼロコスト方針」が
//! 挙げる 4 指標のうち、本ファイルは以下を機械固定する:
//!
//! 1. crate サイズ（依存グラフ非出現）: [`motion_off_excludes_fandhe_animation_from_dependency_graph`]
//! 2. ビルド時間: 上記の帰結として扱う（依存グラフに現れない依存はビルド
//!    対象にならない）。
//! 3. [`fandhe_frontend_pre_styled_ui::theme::Theme::to_css`] の処理量:
//!    [`to_css_body_has_no_feature_cfg_or_motion_branch`]（走査ループへの
//!    分岐追加不在をソース走査で固定）。
//! 4. CSS 出力: [`default_theme_css_matches_pre_motion_golden`]（既定
//!    テーマの `to_css()` 全文バイト一致。`--features motion` 実行でも
//!    同じ golden で走らせることで「feature on でも opt-in 未指定なら
//!    出力不変」も同時に固定する）。
//!
//! [`motion_on_includes_fandhe_animation_in_dependency_graph`] は 1 の
//! 陽性対照（feature 名・`dep:` 配線の破損を検知）。
//!
//! `crates/xtask/tests/wasm_full_animation_optional_dep.rs`（イシュー
//! #2417）と同型の `cargo tree` 文字列走査方式を踏襲する（外部
//! JSON/TOML パーサは使わない、REQ-3）。golden・ソース走査の期待値更新
//! 手順は `docs/internal/pre-styled-ui-golden-test-update-guide.md` を
//! 参照（本ファイルは §3.2 のグルーピング対象外の単独ファイルとして
//! 追記済み）。

use fandhe_frontend_pre_styled_ui::theme::Theme;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// workspace ルート（`crates/pre-styled-ui/` から 2 段上）の絶対パス。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect(
            "crates/pre-styled-ui/ から 2 段上でワークスペースルートに到達する（イシュー #436）",
        )
        .to_path_buf()
}

/// `cargo tree -p fandhe-frontend-pre-styled-ui -e normal --prefix none
/// --locked`（+ 追加引数）を実行し、stdout を返す。非 0 終了は panic
/// させる（fail-closed。cargo 自体の異常は「未検証」として PASS 扱いに
/// しない）。
fn run_cargo_tree(extra_args: &[&str]) -> String {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(workspace_root()).args([
        "tree",
        "-p",
        "fandhe-frontend-pre-styled-ui",
        "-e",
        "normal",
        "--prefix",
        "none",
        "--locked",
    ]);
    cmd.args(extra_args);
    let output = cmd
        .output()
        .expect("cargo tree の起動に失敗した（cargo バイナリが PATH にない可能性がある）");
    assert!(
        output.status.success(),
        "cargo tree が非 0 終了した: status={:?} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("cargo tree の出力が UTF-8 として不正")
}

#[test]
fn motion_off_excludes_fandhe_animation_from_dependency_graph() {
    let tree = run_cargo_tree(&[]);
    assert!(
        !tree.contains("fandhe-animation"),
        "既定 feature（motion off）の依存グラフに fandhe-animation が現れて\
         いる。optional 依存が既定 on 化された可能性がある:\n{tree}"
    );
}

#[test]
fn motion_on_includes_fandhe_animation_in_dependency_graph() {
    let tree = run_cargo_tree(&["--features", "motion"]);
    assert!(
        tree.contains("fandhe-animation"),
        "--features motion でも fandhe-animation が依存グラフに現れない。\
         feature 名の変更・`dep:` 配線の破損の可能性がある（陽性対照の\
         破損）:\n{tree}"
    );
}

/// `Theme::to_css` の走査ループ（`fn to_css` 開始 〜 `write_reduced_motion_block`
/// 終了）に motion feature 由来の実行時分岐を追加していないことをソース
/// 走査で固定する。`to_css`/`write_reduced_motion_block` の実装は
/// `crates/pre-styled-ui/src/theme.rs` にあり、両者は無条件に呼ばれる
/// 内部ヘルパのため `cfg!`/`#[cfg(` を含めてはならない（含めると feature
/// off でも実行時分岐コストが生じ、ゼロコスト方針 §7 の指標 3 に反する）。
#[test]
fn to_css_body_has_no_feature_cfg_or_motion_branch() {
    let theme_rs = workspace_root().join("crates/pre-styled-ui/src/theme.rs");
    let src = fs::read_to_string(&theme_rs)
        .unwrap_or_else(|e| panic!("{} の読み込みに失敗した: {e}", theme_rs.display()));

    let start = src
        .find("pub fn to_css(&self) -> String {")
        .expect("`pub fn to_css` が theme.rs に見つからない（シグネチャ変更の可能性）");
    // `write_reduced_motion_block` の関数本体末尾（次の `fn ` 定義の直前）まで
    // を走査範囲とする（`to_css` 本体 + 直後のヘルパを一括カバー）。
    let after_reduced_motion = src[start..]
        .find("fn write_reduced_motion_block(&self, out: &mut String) {")
        .map(|rel| start + rel)
        .expect("`fn write_reduced_motion_block` が見つからない（関数名変更の可能性）");
    let block_end_rel = src[after_reduced_motion..]
        .find("\n    fn write_dark_declarations")
        .expect("`write_reduced_motion_block` の終端（次関数の開始）が見つからない");
    let scanned = &src[start..after_reduced_motion + block_end_rel];

    for needle in ["cfg!(feature", "#[cfg(feature", "motion_enabled"] {
        assert!(
            !scanned.contains(needle),
            "to_css/write_reduced_motion_block の走査範囲に `{needle}` が\
             見つかった。motion feature 由来の実行時分岐は禁止（イシュー\
             #2416 のゼロコスト方針 §7 指標 3 違反）:\n{scanned}"
        );
    }
}

/// `motion` feature 導入前（イシュー #2416 着手直前の main）で採取した
/// `Theme::default().to_css()` 全文。feature 追加のみで CSS 出力は
/// 一切変わらないことをバイト一致で固定する（ゼロコスト方針 §7 指標 4）。
/// 本テストは `--features motion`（opt-in API 未呼び出し）でも同じ golden
/// で実行され、feature を有効化しただけでは出力が変わらないことも
/// あわせて固定する（Cargo.toml 側で `--features motion` 用の別テスト
/// バイナリは作らない）。
const EXPECTED_DEFAULT_THEME_CSS: &str = r#":root {
  color-scheme: light dark;
  --fandhe-color-bg: #ffffff;
  --fandhe-color-bg-subtle: #f7f7f7;
  --fandhe-color-bg-muted: #eeeeee;
  --fandhe-color-bg-emphasized: #e2e2e2;
  --fandhe-color-bg-overlay: rgba(0, 0, 0, 0.4);
  --fandhe-color-fg: #111111;
  --fandhe-color-fg-muted: #4a4a4a;
  --fandhe-color-fg-subtle: #767676;
  --fandhe-color-border: #d9d9d9;
  --fandhe-color-border-muted: #e6e6e6;
  --fandhe-color-border-subtle: #f0f0f0;
  --fandhe-color-border-emphasized: #b3b3b3;
  --fandhe-color-accent: #3182ce;
  --fandhe-color-accent-emphasized: #2b6cb0;
  --fandhe-color-accent-fg: #ffffff;
  --fandhe-color-accent-subtle: #ebf8ff;
  --fandhe-color-accent-muted: #bee3f8;
  --fandhe-color-accent-fg-subtle: #1a4971;
  --fandhe-color-info: #3182ce;
  --fandhe-color-info-emphasized: #2b6cb0;
  --fandhe-color-info-fg: #ffffff;
  --fandhe-color-info-subtle: #ebf8ff;
  --fandhe-color-info-muted: #bee3f8;
  --fandhe-color-info-fg-subtle: #1a4971;
  --fandhe-color-success: #2f855a;
  --fandhe-color-success-emphasized: #276749;
  --fandhe-color-success-fg: #ffffff;
  --fandhe-color-success-subtle: #f0fff4;
  --fandhe-color-success-muted: #c6f6d5;
  --fandhe-color-success-fg-subtle: #1c4a32;
  --fandhe-color-warning: #b7791f;
  --fandhe-color-warning-emphasized: #975a16;
  --fandhe-color-warning-fg: #ffffff;
  --fandhe-color-warning-subtle: #fffaf0;
  --fandhe-color-warning-muted: #feebc8;
  --fandhe-color-warning-fg-subtle: #5a3c0a;
  --fandhe-color-danger: #c53030;
  --fandhe-color-danger-emphasized: #9b2c2c;
  --fandhe-color-danger-fg: #ffffff;
  --fandhe-color-danger-subtle: #fff5f5;
  --fandhe-color-danger-muted: #fed7d7;
  --fandhe-color-danger-fg-subtle: #6b1414;
  --fandhe-color-neutral: #718096;
  --fandhe-color-neutral-emphasized: #4a5568;
  --fandhe-color-neutral-fg: #ffffff;
  --fandhe-color-neutral-subtle: #f7f7f7;
  --fandhe-color-neutral-muted: #e2e8f0;
  --fandhe-color-neutral-fg-subtle: #333333;
  --fandhe-color-focus-ring: #3182ce;
  --fandhe-color-chart-1: #3182ce;
  --fandhe-color-chart-2: #dd6b20;
  --fandhe-color-chart-3: #2f855a;
  --fandhe-color-chart-4: #805ad5;
  --fandhe-color-chart-5: #d53f8c;
  --fandhe-color-chart-6: #00a3c4;
  --fandhe-color-sidebar-bg: #f7f7f7;
  --fandhe-color-sidebar-fg: #111111;
  --fandhe-color-sidebar-accent: #3182ce;
  --fandhe-color-sidebar-accent-fg: #ffffff;
  --fandhe-color-sidebar-muted: #eeeeee;
  --fandhe-color-sidebar-border: #d9d9d9;
  --fandhe-color-sidebar-focus-ring: #3182ce;
  --fandhe-space-0-5: 0.125rem;
  --fandhe-space-1: 0.25rem;
  --fandhe-space-1-5: 0.375rem;
  --fandhe-space-2: 0.5rem;
  --fandhe-space-2-5: 0.625rem;
  --fandhe-space-3: 0.75rem;
  --fandhe-space-4: 1rem;
  --fandhe-space-5: 1.25rem;
  --fandhe-space-6: 1.5rem;
  --fandhe-space-8: 2rem;
  --fandhe-space-10: 2.5rem;
  --fandhe-space-12: 3rem;
  --fandhe-space-16: 4rem;
  --fandhe-space-20: 5rem;
  --fandhe-space-24: 6rem;
  --fandhe-font-font-body: system-ui, -apple-system, sans-serif;
  --fandhe-font-font-mono: ui-monospace, monospace;
  --fandhe-font-font-size-xs: 0.75rem;
  --fandhe-font-font-size-sm: 0.875rem;
  --fandhe-font-font-size-md: 1rem;
  --fandhe-font-font-size-lg: 1.125rem;
  --fandhe-font-font-size-xl: 1.25rem;
  --fandhe-font-font-size-2xl: 1.5rem;
  --fandhe-font-font-size-3xl: 1.875rem;
  --fandhe-font-font-size-4xl: 2.25rem;
  --fandhe-font-font-weight-normal: 400;
  --fandhe-font-font-weight-medium: 500;
  --fandhe-font-font-weight-semibold: 600;
  --fandhe-font-font-weight-bold: 700;
  --fandhe-font-line-height-tight: 1.25;
  --fandhe-font-line-height-normal: 1.5;
  --fandhe-font-line-height-relaxed: 1.75;
  --fandhe-radius-none: 0;
  --fandhe-radius-xs: 0.125rem;
  --fandhe-radius-sm: 0.25rem;
  --fandhe-radius-md: 0.375rem;
  --fandhe-radius-lg: 0.5rem;
  --fandhe-radius-xl: 0.75rem;
  --fandhe-radius-2xl: 1rem;
  --fandhe-radius-full: 9999px;
  --fandhe-shadow-xs: 0 1px 2px rgba(0, 0, 0, 0.06);
  --fandhe-shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.12);
  --fandhe-shadow-md: 0 4px 6px rgba(0, 0, 0, 0.1);
  --fandhe-shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.16);
  --fandhe-shadow-xl: 0 20px 25px rgba(0, 0, 0, 0.2);
  --fandhe-shadow-2xl: 0 25px 50px rgba(0, 0, 0, 0.25);
  --fandhe-z-index-hide: -1;
  --fandhe-z-index-base: 0;
  --fandhe-z-index-docked: 10;
  --fandhe-z-index-dropdown: 1000;
  --fandhe-z-index-sticky: 1100;
  --fandhe-z-index-popover: 1200;
  --fandhe-z-index-overlay: 1300;
  --fandhe-z-index-modal: 1400;
  --fandhe-z-index-skip-nav: 1500;
  --fandhe-z-index-toast: 1600;
  --fandhe-z-index-tooltip: 1700;
  --fandhe-z-index-max: 2147483647;
  --fandhe-focus-ring-width: 2px;
  --fandhe-focus-ring-offset: 2px;
  --fandhe-size-control-height-xs: 2rem;
  --fandhe-size-control-height-sm: 2.25rem;
  --fandhe-size-control-height-md: 2.5rem;
  --fandhe-size-control-height-lg: 2.75rem;
  --fandhe-size-control-height-xl: 3rem;
  --fandhe-size-control-padding-x-xs: 0.625rem;
  --fandhe-size-control-padding-x-sm: 0.75rem;
  --fandhe-size-control-padding-x-md: 1rem;
  --fandhe-size-control-padding-x-lg: 1.25rem;
  --fandhe-size-control-padding-x-xl: 1.5rem;
  --fandhe-size-control-font-size-xs: var(--fandhe-font-font-size-xs);
  --fandhe-size-control-font-size-sm: var(--fandhe-font-font-size-sm);
  --fandhe-size-control-font-size-md: var(--fandhe-font-font-size-md);
  --fandhe-size-control-font-size-lg: var(--fandhe-font-font-size-lg);
  --fandhe-size-control-font-size-xl: var(--fandhe-font-font-size-xl);
  --fandhe-motion-duration-fast: 150ms;
  --fandhe-motion-duration-normal: 200ms;
  --fandhe-motion-duration-slow: 300ms;
  --fandhe-motion-easing-standard: cubic-bezier(0.4, 0, 0.2, 1);
  --fandhe-motion-easing-emphasized: cubic-bezier(0.2, 0, 0, 1);
  --fandhe-motion-duration-faster: 100ms;
  --fandhe-motion-duration-slower: 400ms;
  --fandhe-motion-easing-in: cubic-bezier(0.42, 0, 1, 1);
  --fandhe-motion-easing-out: cubic-bezier(0, 0, 0.58, 1);
  --fandhe-motion-easing-in-out: cubic-bezier(0.42, 0, 0.58, 1);
  --fandhe-motion-easing-emphasized-decelerate: cubic-bezier(0.05, 0.7, 0.1, 1);
  --fandhe-motion-easing-emphasized-accelerate: cubic-bezier(0.3, 0, 0.8, 0.15);
  --fandhe-breakpoint-sm: 640px;
  --fandhe-breakpoint-md: 768px;
  --fandhe-breakpoint-lg: 1024px;
  --fandhe-breakpoint-xl: 1280px;
}
:root[data-theme="light"] { color-scheme: light; }
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    color-scheme: dark;
    --fandhe-color-bg: #111111;
    --fandhe-color-bg-subtle: #1a1a1a;
    --fandhe-color-bg-muted: #242424;
    --fandhe-color-bg-emphasized: #2e2e2e;
    --fandhe-color-bg-overlay: rgba(0, 0, 0, 0.6);
    --fandhe-color-fg: #f7f7f7;
    --fandhe-color-fg-muted: #cccccc;
    --fandhe-color-fg-subtle: #a3a3a3;
    --fandhe-color-border: #3a3a3a;
    --fandhe-color-border-muted: #2a2a2a;
    --fandhe-color-border-subtle: #202020;
    --fandhe-color-border-emphasized: #525252;
    --fandhe-color-accent: #4299e1;
    --fandhe-color-accent-emphasized: #63b3ed;
    --fandhe-color-accent-fg: #0b1720;
    --fandhe-color-accent-subtle: #1a2b3d;
    --fandhe-color-accent-muted: #2c4a66;
    --fandhe-color-accent-fg-subtle: #90cdf4;
    --fandhe-color-info: #63b3ed;
    --fandhe-color-info-emphasized: #90cdf4;
    --fandhe-color-info-fg: #0b1720;
    --fandhe-color-info-subtle: #1a2b3d;
    --fandhe-color-info-muted: #2c4a66;
    --fandhe-color-info-fg-subtle: #90cdf4;
    --fandhe-color-success: #68d391;
    --fandhe-color-success-emphasized: #9ae6b4;
    --fandhe-color-success-fg: #0b1a12;
    --fandhe-color-success-subtle: #122a1c;
    --fandhe-color-success-muted: #1c4a32;
    --fandhe-color-success-fg-subtle: #9ae6b4;
    --fandhe-color-warning: #f6ad55;
    --fandhe-color-warning-emphasized: #fbd38d;
    --fandhe-color-warning-fg: #1a1203;
    --fandhe-color-warning-subtle: #2e2410;
    --fandhe-color-warning-muted: #4a3510;
    --fandhe-color-warning-fg-subtle: #fbd38d;
    --fandhe-color-danger: #fc8181;
    --fandhe-color-danger-emphasized: #feb2b2;
    --fandhe-color-danger-fg: #1a0b0b;
    --fandhe-color-danger-subtle: #2e1616;
    --fandhe-color-danger-muted: #4a1f1f;
    --fandhe-color-danger-fg-subtle: #feb2b2;
    --fandhe-color-neutral: #a0aec0;
    --fandhe-color-neutral-emphasized: #cbd5e0;
    --fandhe-color-neutral-fg: #0b1720;
    --fandhe-color-neutral-subtle: #1a1a1a;
    --fandhe-color-neutral-muted: #2d3748;
    --fandhe-color-neutral-fg-subtle: #d4d4d4;
    --fandhe-color-focus-ring: #63b3ed;
    --fandhe-color-chart-1: #63b3ed;
    --fandhe-color-chart-2: #f6ad55;
    --fandhe-color-chart-3: #68d391;
    --fandhe-color-chart-4: #b794f4;
    --fandhe-color-chart-5: #f687b3;
    --fandhe-color-chart-6: #76e4f7;
    --fandhe-color-sidebar-bg: #1a1a1a;
    --fandhe-color-sidebar-fg: #f7f7f7;
    --fandhe-color-sidebar-accent: #4299e1;
    --fandhe-color-sidebar-accent-fg: #0b1720;
    --fandhe-color-sidebar-muted: #242424;
    --fandhe-color-sidebar-border: #3a3a3a;
    --fandhe-color-sidebar-focus-ring: #63b3ed;
    --fandhe-shadow-xs: 0 1px 2px rgba(0, 0, 0, 0.24);
    --fandhe-shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.32);
    --fandhe-shadow-md: 0 4px 6px rgba(0, 0, 0, 0.3);
    --fandhe-shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.4);
    --fandhe-shadow-xl: 0 20px 25px rgba(0, 0, 0, 0.5);
    --fandhe-shadow-2xl: 0 25px 50px rgba(0, 0, 0, 0.55);
  }
}
:root[data-theme="dark"] {
  color-scheme: dark;
  --fandhe-color-bg: #111111;
  --fandhe-color-bg-subtle: #1a1a1a;
  --fandhe-color-bg-muted: #242424;
  --fandhe-color-bg-emphasized: #2e2e2e;
  --fandhe-color-bg-overlay: rgba(0, 0, 0, 0.6);
  --fandhe-color-fg: #f7f7f7;
  --fandhe-color-fg-muted: #cccccc;
  --fandhe-color-fg-subtle: #a3a3a3;
  --fandhe-color-border: #3a3a3a;
  --fandhe-color-border-muted: #2a2a2a;
  --fandhe-color-border-subtle: #202020;
  --fandhe-color-border-emphasized: #525252;
  --fandhe-color-accent: #4299e1;
  --fandhe-color-accent-emphasized: #63b3ed;
  --fandhe-color-accent-fg: #0b1720;
  --fandhe-color-accent-subtle: #1a2b3d;
  --fandhe-color-accent-muted: #2c4a66;
  --fandhe-color-accent-fg-subtle: #90cdf4;
  --fandhe-color-info: #63b3ed;
  --fandhe-color-info-emphasized: #90cdf4;
  --fandhe-color-info-fg: #0b1720;
  --fandhe-color-info-subtle: #1a2b3d;
  --fandhe-color-info-muted: #2c4a66;
  --fandhe-color-info-fg-subtle: #90cdf4;
  --fandhe-color-success: #68d391;
  --fandhe-color-success-emphasized: #9ae6b4;
  --fandhe-color-success-fg: #0b1a12;
  --fandhe-color-success-subtle: #122a1c;
  --fandhe-color-success-muted: #1c4a32;
  --fandhe-color-success-fg-subtle: #9ae6b4;
  --fandhe-color-warning: #f6ad55;
  --fandhe-color-warning-emphasized: #fbd38d;
  --fandhe-color-warning-fg: #1a1203;
  --fandhe-color-warning-subtle: #2e2410;
  --fandhe-color-warning-muted: #4a3510;
  --fandhe-color-warning-fg-subtle: #fbd38d;
  --fandhe-color-danger: #fc8181;
  --fandhe-color-danger-emphasized: #feb2b2;
  --fandhe-color-danger-fg: #1a0b0b;
  --fandhe-color-danger-subtle: #2e1616;
  --fandhe-color-danger-muted: #4a1f1f;
  --fandhe-color-danger-fg-subtle: #feb2b2;
  --fandhe-color-neutral: #a0aec0;
  --fandhe-color-neutral-emphasized: #cbd5e0;
  --fandhe-color-neutral-fg: #0b1720;
  --fandhe-color-neutral-subtle: #1a1a1a;
  --fandhe-color-neutral-muted: #2d3748;
  --fandhe-color-neutral-fg-subtle: #d4d4d4;
  --fandhe-color-focus-ring: #63b3ed;
  --fandhe-color-chart-1: #63b3ed;
  --fandhe-color-chart-2: #f6ad55;
  --fandhe-color-chart-3: #68d391;
  --fandhe-color-chart-4: #b794f4;
  --fandhe-color-chart-5: #f687b3;
  --fandhe-color-chart-6: #76e4f7;
  --fandhe-color-sidebar-bg: #1a1a1a;
  --fandhe-color-sidebar-fg: #f7f7f7;
  --fandhe-color-sidebar-accent: #4299e1;
  --fandhe-color-sidebar-accent-fg: #0b1720;
  --fandhe-color-sidebar-muted: #242424;
  --fandhe-color-sidebar-border: #3a3a3a;
  --fandhe-color-sidebar-focus-ring: #63b3ed;
  --fandhe-shadow-xs: 0 1px 2px rgba(0, 0, 0, 0.24);
  --fandhe-shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.32);
  --fandhe-shadow-md: 0 4px 6px rgba(0, 0, 0, 0.3);
  --fandhe-shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.4);
  --fandhe-shadow-xl: 0 20px 25px rgba(0, 0, 0, 0.5);
  --fandhe-shadow-2xl: 0 25px 50px rgba(0, 0, 0, 0.55);
}
@media (prefers-reduced-motion: reduce) {
  :root {
    --fandhe-motion-duration-fast: 0ms;
    --fandhe-motion-duration-normal: 0ms;
    --fandhe-motion-duration-slow: 0ms;
    --fandhe-motion-duration-faster: 0ms;
    --fandhe-motion-duration-slower: 0ms;
  }
}
"#;

#[test]
fn default_theme_css_matches_pre_motion_golden() {
    assert_eq!(Theme::default().to_css(), EXPECTED_DEFAULT_THEME_CSS);
}
