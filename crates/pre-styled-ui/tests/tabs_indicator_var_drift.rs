//! `fandhe_frontend_pre_styled_ui::tabs` の `indicator` slot CSS が参照する
//! `--left`/`--top`/`--width`/`--height` と、`fandhe-frontend-wasm-full`
//! （`crates/wasm-full/src/tabs_indicator.rs`）が実測値の書き込み先として
//! 定義する `INDICATOR_LEFT_VAR`/`INDICATOR_TOP_VAR`/`INDICATOR_WIDTH_VAR`/
//! `INDICATOR_HEIGHT_VAR` のドリフト検知（イシュー #2211）。
//!
//! `fandhe-frontend-pre-styled-ui` は `fandhe-frontend-wasm-full` に依存
//! していない（`Cargo.toml` に dep 行なし。wasm-full は逆に pre-styled-ui
//! へ依存する関係のため、コンパイル時の型共有はできない）ため、
//! `crates/pre-styled-ui/tests/content_height_var_drift.rs` と同じ
//! ソース走査型の fail-closed 契約テスト手法（`fs::read_to_string` +
//! リテラル一致確認）を用いる。
//!
//! `fandhe-frontend-headless-ui`（`crates/headless-ui/src/tabs.rs`）の
//! `INDICATOR_STYLE_INITIAL`（SSR 初期値、`pub` ではない内部定数）とも
//! 3 者で同じ 4 変数名を共有する契約であるため、headless 側の SSR 出力
//! （`tabs::tabs()` の実 HTML）もあわせて検証する。
//!
//! `include_str!` ではなく実行時 `fs::read_to_string` を使う理由:
//! `cargo package` 後の packaged コピー単体ではワークスペース内の他
//! クレートパスは存在しない。本テストは `cargo package` の検証ビルド
//! （テストを実行しない）には現れず、通常の `cargo test` 実行時のみ走る
//! ため、ワークスペースルートからの相対パス解決で問題ない。

use std::fs;
use std::path::PathBuf;

/// `crates/wasm-full/src/tabs_indicator.rs` の絶対パスを、本クレートの
/// `CARGO_MANIFEST_DIR`（`crates/pre-styled-ui`）から兄弟クレートへの
/// 相対パスとして組み立てる。
fn wasm_full_tabs_indicator_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("wasm-full")
        .join("src")
        .join("tabs_indicator.rs")
}

/// `crates/headless-ui/src/tabs.rs` の絶対パス（同上の兄弟クレート解決）。
fn headless_ui_tabs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("headless-ui")
        .join("src")
        .join("tabs.rs")
}

fn read_fail_closed(path: &PathBuf) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| {
        panic!(
            "{} を読めること（fail-closed。CSS 変数名のドリフト検知が \
             本テストの目的であり、読み取り不能を PASS 扱いにしない）: {err}",
            path.display()
        )
    })
}

/// [`fandhe_frontend_pre_styled_ui::tabs::stylesheet`] の `indicator` slot
/// が参照する 4 変数（`crate::tabs::recipe` `.base("indicator", ...)`）が、
/// `wasm-full` 側の `INDICATOR_*_VAR` 定数のリテラル値（`--left` 等）と
/// 一致することを固定する。
#[test]
fn pre_styled_ui_indicator_css_vars_match_wasm_full_consts() {
    let path = wasm_full_tabs_indicator_path();
    let content = read_fail_closed(&path);

    let expected_pairs = [
        ("INDICATOR_LEFT_VAR", "--left", "var(--left, 0px)"),
        ("INDICATOR_TOP_VAR", "--top", "var(--top, 0px)"),
        ("INDICATOR_WIDTH_VAR", "--width", "var(--width, 0px)"),
        ("INDICATOR_HEIGHT_VAR", "--height", "var(--height, 0px)"),
    ];

    let css = fandhe_frontend_pre_styled_ui::tabs::stylesheet();

    for (const_name, var_literal, css_reference) in expected_pairs {
        let expected_decl = format!("pub const {const_name}: &str = \"{var_literal}\";");
        assert!(
            content.contains(&expected_decl),
            "{} の {const_name} 定義が {var_literal:?} と一致しません。\
             期待した宣言行: {expected_decl:?}",
            path.display()
        );
        assert!(
            css.contains(css_reference),
            "fandhe_frontend_pre_styled_ui::tabs::stylesheet() の indicator \
             slot が {css_reference:?} を参照していません（wasm-full の \
             {const_name} とのドリフト）"
        );
    }
}

/// headless-ui の SSR 初期値（`INDICATOR_STYLE_INITIAL`）が同じ 4 変数名を
/// 使うことを、実際の SSR 出力（`tabs::tabs()`）経由で固定する（headless
/// 側は private 定数のためソース走査ではなくレンダリング結果で検証する）。
#[test]
fn headless_ui_ssr_output_uses_same_indicator_var_names() {
    use fandhe_frontend_core::render;
    use fandhe_frontend_headless_ui::data_attrs::Orientation;
    use fandhe_frontend_headless_ui::tabs::{tabs, ActivationMode, TabItem, TabsProps};

    let props = TabsProps {
        id: "t",
        selected: "a",
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: true,
    };
    let items = vec![TabItem {
        value: "a",
        trigger: vec![],
        content: vec![],
        disabled: false,
    }];
    let html = render(&tabs(&props, items));

    for var_literal in ["--left", "--top", "--width", "--height"] {
        assert!(
            html.contains(var_literal),
            "headless-ui tabs() の SSR 出力に {var_literal:?} が含まれません \
             （crates/headless-ui/src/tabs.rs の INDICATOR_STYLE_INITIAL \
             とのドリフト）"
        );
    }

    // headless_ui_tabs_path() は本テストが「ソース側の内部定数も存在する」
    // ことを人が確認できるよう、パス解決自体は健全であることを併せて
    // 固定する（read 自体は上記 SSR 検証で代替するため中身の走査はしない）。
    let path = headless_ui_tabs_path();
    assert!(
        path.is_file(),
        "{} が存在すること（兄弟クレートパス解決の健全性）",
        path.display()
    );
}
