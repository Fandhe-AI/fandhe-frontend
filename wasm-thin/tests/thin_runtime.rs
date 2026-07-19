//! `fandhe-frontend-wasm-thin` の汎用層・境界層（demo）の回帰テスト（TASK-11.3a、#79）。
//!
//! ここでの検証対象は「WASM 境界を越える HTML 文字列が既定エスケープ済みで
//! あること」「未知アクション・改ざんハイドレーション属性に対して panic
//! しないこと」の 2 点であり、実ブラウザでの JS グルー結合検証は
//! TASK-11.5/11.6 系列のスコープ（`docs/spec/05-tasks.md`）。

use fandhe_frontend_interactive::{AppState, Hydrate, HydrateError};
use fandhe_frontend_wasm_thin::demo;
use fandhe_frontend_wasm_thin::ThinRuntime;

/// XSS 回帰（薄いグルー経路、REQ-1）: `apply` に script タグ相当の payload を
/// 渡しても、戻り値 HTML に生タグが現れず既定エスケープされていること。
#[test]
fn apply_escapes_script_payload() {
    let mut runtime = ThinRuntime::new(AppState::new());

    let html = runtime.apply("set_draft", "<script>alert(1)</script>");
    // set_draft 単体では draft はまだ画面に反映されるが追加はされない。
    // 実際に描画へ現れるのは add_item 後。
    let html_after_add = runtime.apply("add_item", "");

    assert!(!html.contains("<script>alert"));
    assert!(!html_after_add.contains("<script>alert"));
    assert!(html_after_add.contains("&lt;script&gt;alert"));
}

/// 属性値でのインジェクション回帰（REQ-1）: `"` や `<` を含む payload でも
/// 属性値が既定エスケープされ、DOM 構造を壊す生の `"` が混入しないこと。
#[test]
fn apply_escapes_attribute_breaking_payload() {
    let mut runtime = ThinRuntime::new(AppState::new());
    runtime.apply("set_draft", "\"><img src=x onerror=alert(1)>");
    let html = runtime.apply("add_item", "");

    assert!(!html.contains("<img src=x onerror"));
    assert!(html.contains("&lt;img"));
}

/// 未知アクション no-op（安全側フォールバック）: 未知の `name` で `apply` して
/// も状態が変化せず、HTML も変わらないこと。
#[test]
fn apply_ignores_unknown_action() {
    let mut runtime = ThinRuntime::new(AppState::new());
    let before = runtime.html();
    let after = runtime.apply("no_such_action", "payload");
    assert_eq!(before, after);
}

/// ハイドレーションのラウンドトリップ: 状態 → `hydration_attrs()` →
/// `hydrate_from_attrs()` → `html()` が SSR 側 `render_for_hydration` の
/// 描画（ハイドレーション属性を除く）と一致すること。区切り文字 `\u{1f}`・
/// バックスラッシュを含む項目でも成立することを確認する。
#[test]
fn hydrate_from_attrs_roundtrip_matches_ssr_view() {
    use fandhe_frontend_interactive::Component;

    let mut source = AppState::new();
    source.update(fandhe_frontend_interactive::Action::Increment);
    source.items.push("separator:\u{1f}here".to_string());
    source.items.push("backslash:\\here".to_string());
    // `items` への直接代入は `item_ids`（keyed list の安定キー、イシュー
    // #345）を追随させない。`hydrate_from_attrs` 側は復元時に
    // `item_ids = 0..items.len()` を決定的に再構築するため（`interactive`
    // クレートの `AppState::item_ids` 型ドキュメント参照）、比較対象の
    // `source.view()` 側もここで揃えておく。
    source.item_ids = (0..source.items.len() as u64).collect();
    source.update(fandhe_frontend_interactive::Action::SetDraft(
        "draft text".to_string(),
    ));

    let attrs = source.hydration_attrs();

    let mut runtime = ThinRuntime::new(AppState::new());
    runtime
        .hydrate_from_attrs(&attrs)
        .expect("valid attrs should hydrate");

    let expected = fandhe_frontend_core::render(&source.view());
    assert_eq!(runtime.html(), expected);
}

/// 改ざん耐性: 欠落属性・不正値（数値パース失敗）で `hydrate_from_attrs` が
/// panic せず `Err` を返し、状態を変更しないこと（境界層は初期状態のまま
/// CSR を継続できる）。
#[test]
fn hydrate_from_attrs_rejects_invalid_input_without_panicking() {
    let mut runtime = ThinRuntime::new(AppState::new());
    let before = runtime.html();

    let missing: Vec<(String, String)> = Vec::new();
    let err = runtime
        .hydrate_from_attrs(&missing)
        .expect_err("missing attrs must fail");
    assert!(matches!(err, HydrateError::MissingAttr(_)));
    assert_eq!(runtime.html(), before);

    let invalid = vec![
        (
            format!(
                "{}counter",
                fandhe_frontend_interactive::HYDRATE_ATTR_PREFIX
            ),
            "not-a-number".to_string(),
        ),
        (
            format!("{}draft", fandhe_frontend_interactive::HYDRATE_ATTR_PREFIX),
            String::new(),
        ),
        (
            format!("{}items", fandhe_frontend_interactive::HYDRATE_ATTR_PREFIX),
            String::new(),
        ),
    ];
    let err = runtime
        .hydrate_from_attrs(&invalid)
        .expect_err("invalid counter must fail");
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
    assert_eq!(runtime.html(), before);
}

/// demo 境界層のスモークテスト: `initial_html`/`apply`/`hydrate_from_attrs`
/// の連続呼び出しで状態が一貫すること（`#[wasm_bindgen]` エクスポートは
/// native 上でも通常の Rust 関数として呼び出せる）。
#[test]
fn demo_boundary_layer_smoke() {
    // カウンター値はイシュー #345 で束縛点（`<span data-bind-text="counter">`）
    // に分離出力されるため「カウント: N」は連続部分文字列にならない
    // （`interactive/src/lib.rs` の `render_with_root_attrs` 参照）。
    let initial = demo::initial_html();
    assert!(initial.contains(r#"data-bind-text="counter">0</span>"#));

    let after_increment = demo::apply("increment", "");
    assert!(after_increment.contains(r#"data-bind-text="counter">1</span>"#));

    // 不正な attrs（names/values の長さ不一致）は false を返し panic しない。
    let ok = demo::hydrate_from_attrs(vec!["only-one".to_string()], Vec::new());
    assert!(!ok);

    // XSS 回帰: demo 経路でも script タグはエスケープされる。
    demo::apply("set_draft", "<script>alert(1)</script>");
    let after_add = demo::apply("add_item", "");
    assert!(!after_add.contains("<script>alert"));
    assert!(after_add.contains("&lt;script&gt;alert"));
}
