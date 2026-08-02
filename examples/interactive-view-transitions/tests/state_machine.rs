//! `examples/interactive-view-transitions` の integration test。
//!
//! `fandhe_frontend_interactive` の状態機械 API（`dispatch` / `decode_action` /
//! `render_for_hydration`）の契約を、このサンプルが実演する範囲で固定する。
//! `src/main.rs` はバイナリクレートのため本ファイルからは `use` できず、
//! `fandhe_frontend_interactive::AppState`（クレート公開の参照コンポーネント）
//! を直接使う（`examples/ssr-routing/tests/routing.rs` と同じ方針）。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::menubar::{self, Menubar};
use fandhe_frontend_headless_ui::navigation_menu::{self, NavigationMenu};
use fandhe_frontend_interactive::{dispatch, render_for_hydration, AppState};
use std::path::Path;

/// `dispatch("increment")` で counter が増え、戻り値が `true`
/// （decode_action が既知アクションを復号できた）になることを固定する。
#[test]
fn dispatch_increment_increases_counter_and_returns_true() {
    let mut state = AppState::new();
    let before = state.counter;

    let applied = dispatch(&mut state, "increment", "");

    assert!(applied);
    assert_eq!(state.counter, before + 1);
}

/// 未知アクション名は `decode_action` の復号失敗として no-op になり、
/// `dispatch` は `false` を返し状態を変更しない（不変条件 4、安全側
/// フォールバック）。
#[test]
fn dispatch_unknown_action_is_no_op_and_returns_false() {
    let mut state = AppState::new();
    let before = state.clone();

    let applied = dispatch(&mut state, "no-such-action", "");

    assert!(!applied);
    assert_eq!(state, before);
}

/// `render_for_hydration` はルート要素へ `data-hydrate-*` 属性を付与する
/// （`AppState::hydration_attrs` の契約、`HYDRATE_ATTR_PREFIX` 参照）。
#[test]
fn render_for_hydration_adds_hydrate_attrs_to_root_element() {
    let state = AppState::new();

    let node = render_for_hydration(&state);
    let html = render(&node);

    assert!(html.contains("data-hydrate-counter="), "html was: {html}");
    assert!(
        html.contains(r#"id="interactive-root""#),
        "html was: {html}"
    );
}

/// 既定エスケープ回帰（REQ-1）: `<script>` を含む draft を `set_draft` で
/// 反映したのち `render_for_hydration` の出力に、生の `<script>` タグとして
/// 現れないことを固定する（ハイドレーション属性値のエスケープも含む）。
#[test]
fn render_for_hydration_escapes_script_payload_in_draft_and_items() {
    let mut state = AppState::new();
    let payload = "<script>alert(1)</script>";

    dispatch(&mut state, "set_draft", payload);
    dispatch(&mut state, "add_item", "");

    let node = render_for_hydration(&state);
    let html = render(&node);

    assert!(
        !html.contains("<script>alert"),
        "raw <script> tag leaked into rendered HTML: {html}"
    );
    assert!(
        html.contains("&lt;script&gt;"),
        "expected escaped script payload in html: {html}"
    );
}

/// `static/embed.html` の回帰テスト（PR #510 Bugbot 指摘、review comment
/// 3621300109 "Hydrate mount id collides"）。
///
/// `#interactive-root` を空のまま `hydrate("interactive-root")` を呼ぶと、
/// 状態復元（`hydration::restore_state`）が `data-hydrate-*` 属性なしで
/// 失敗し、CSR フォールバック（`dom::mount_initial`）が `AppState::view()`
/// （自身も `id="interactive-root"` を持つ）をこの `<div>` の中へ丸ごと
/// 差し込んでしまい、同一 id が入れ子で重複する。これを防ぐには
/// `#interactive-root` があらかじめ `data-hydrate-*` 属性付きの SSR
/// 済みマークアップを保持し、`hydrate()` の状態復元が成功する経路のみを
/// 通ることが必須（`dom::mount_initial` を一切呼ばせない）。本テストは
/// その前提となる属性の存在をファイル内容の静的検査で固定する
/// （wasm 実行を伴わない native テストのため、ブラウザでの実際の
/// `hydrate()` 呼び出し結果までは検証できない点に注意）。
#[test]
fn embed_html_interactive_root_has_hydrate_attrs_to_avoid_csr_fallback_id_collision() {
    let embed_html_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("static/embed.html");
    let html = std::fs::read_to_string(&embed_html_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", embed_html_path.display()));

    // 実タグのみを対象にする。`<head>`/`<body>` 双方のコメント内に
    // `<div id="interactive-root">`（属性なしの例示文字列）が出現するため、
    // 実タグにのみ付く `data-testid="interactive-root"` まで含めた接頭辞で
    // 開始位置を一意に特定する（`AppState::view()` / `render_for_hydration`
    // が常に id の直後にこの属性を出力する契約、interactive/src/lib.rs
    // 参照）。開始タグを閉じる最初の `>` までを属性検査対象とする
    // （マルチバイト文字境界を跨がないよう、バイト固定長ではなく `>` の
    // 位置で区切る）。
    let root_start = html
        .find(r#"<div id="interactive-root" data-testid="interactive-root""#)
        .expect(
            "static/embed.html must contain the actual \
             <div id=\"interactive-root\" data-testid=\"interactive-root\" ...> mount tag \
             (not just a mention in a comment)",
        );
    let tag_end = html[root_start..]
        .find('>')
        .map(|offset| root_start + offset)
        .expect("static/embed.html #interactive-root start tag must be closed with '>'");
    let tag_slice = &html[root_start..tag_end];

    for attr in [
        "data-hydrate-counter=",
        "data-hydrate-draft=",
        "data-hydrate-items=",
        "data-hydrate-item-ids=",
    ] {
        assert!(
            tag_slice.contains(attr),
            "static/embed.html の #interactive-root に {attr} がありません。\
             空のまま hydrate() を呼ぶと CSR フォールバックが AppState::view() を \
             二重に差し込み id 衝突が再発します（PR #510 Bugbot 指摘の回帰）。\
             tag_slice was: {tag_slice}"
        );
    }
}

/// `static/embed.html` の回帰テスト（PR #510 Bugbot 指摘「Hydrate list attrs
/// miss separator」、review thread 未解決分）。
///
/// `data-hydrate-items` / `data-hydrate-item-ids` は `codec::decode_list`
/// が要求する先頭の Unit Separator（`\u{1f}`、`codec::encode_list` 契約）を
/// 欠くと、`decode_list` が空文字列を空リストと誤認して空ベクタを返し、
/// `hydrate()` は復元「成功」のまま `mount_initial` を呼ばずスキップして
/// しまう（DOM は「最初の項目」を表示したまま `AppState.items` が空になる
/// サイレントな状態不整合）。本テストは `fandhe_frontend_interactive::codec`
/// の実装（正）を基準に、embed.html の属性値が実際に
/// `["最初の項目"]` / `["0"]` へ decode できることを固定する
/// （静的検査に留めず、正本の codec 経由でラウンドトリップ検証する）。
#[test]
fn embed_html_interactive_root_hydrate_list_attrs_roundtrip_via_codec() {
    use fandhe_frontend_interactive::codec;

    let embed_html_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("static/embed.html");
    let html = std::fs::read_to_string(&embed_html_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", embed_html_path.display()));

    let root_start = html
        .find(r#"<div id="interactive-root" data-testid="interactive-root""#)
        .expect("static/embed.html must contain the #interactive-root mount tag");
    let tag_end = html[root_start..]
        .find('>')
        .map(|offset| root_start + offset)
        .expect("static/embed.html #interactive-root start tag must be closed with '>'");
    let tag_slice = &html[root_start..tag_end];

    let items_value = extract_attr_value(tag_slice, "data-hydrate-items")
        .expect("data-hydrate-items attribute must be present");
    let item_ids_value = extract_attr_value(tag_slice, "data-hydrate-item-ids")
        .expect("data-hydrate-item-ids attribute must be present");

    assert_eq!(
        codec::decode_list(&items_value),
        vec!["最初の項目".to_string()],
        "data-hydrate-items must decode to the non-empty initial item list \
         (missing leading U+001F Unit Separator makes decode_list return an \
         empty Vec, PR #510 Bugbot 指摘の回帰). raw value was: {items_value:?}"
    );
    assert_eq!(
        codec::decode_list(&item_ids_value),
        vec!["0".to_string()],
        "data-hydrate-item-ids must decode to the matching id list. \
         raw value was: {item_ids_value:?}"
    );
}

// --- navigation-menu / menubar 実演（イシュー #1199）の dispatch 契約 ---

/// `("navigation-menu", "trigger")` → `"toggle"`（`crates/wasm-full/src/headless.rs`
/// の `MAPPING_TABLE`）が payload に項目値を渡す契約を固定する。開いている
/// 項目の再クリックは disclosure nav として閉じる（`SingleSelect::update`
/// の `Toggle` 挙動）。
#[test]
fn navigation_menu_toggle_opens_then_closes_same_item() {
    let mut state = NavigationMenu::default();
    assert_eq!(state.open_value(), None);

    let applied = dispatch(&mut state, "toggle", "products");
    assert!(applied);
    assert_eq!(state.open_value(), Some("products"));

    let applied = dispatch(&mut state, "toggle", "products");
    assert!(applied);
    assert_eq!(state.open_value(), None);
}

/// `overlay::OverlayCloseController` の閉鎖要求（Escape・外側クリック）を
/// 受けた呼び出し側が dispatch すべき `"deselect"`（payload なし）は、
/// 既に閉じている状態へ再度送っても冪等 no-op（`applied` は decode 成功
/// なので `true` を保ちつつ状態は不変）であることを固定する（`overlay.rs`
/// モジュール doc §イシュー #1173「二重 dispatch されても no-op のまま
/// 安全に収束する」の回帰）。
#[test]
fn navigation_menu_deselect_is_idempotent_no_op_when_already_closed() {
    let mut state = NavigationMenu::default();
    let before = state.clone();

    let applied = dispatch(&mut state, "deselect", "");

    assert!(applied);
    assert_eq!(state, before);
}

/// 未知アクション名は `decode_action`（`SingleSelect::decode_action` への
/// 全委譲）の復号失敗として no-op になる不変条件 4 の回帰。
#[test]
fn navigation_menu_unknown_action_is_no_op() {
    let mut state = NavigationMenu::default();
    dispatch(&mut state, "toggle", "products");
    let before = state.clone();

    let applied = dispatch(&mut state, "no-such-action", "");

    assert!(!applied);
    assert_eq!(state, before);
}

/// `("menubar", "trigger")` → `"toggle"`（payload は Menu の index）が
/// `MenubarAction::Toggle` へ復号され、対象 Menu を開くことを固定する。
#[test]
fn menubar_toggle_opens_target_menu() {
    let mut state = Menubar::new(0, 2, None, false, Orientation::Horizontal);
    assert_eq!(state.open(), None);

    let applied = dispatch(&mut state, "toggle", "1");

    assert!(applied);
    assert_eq!(state.open(), Some(1));
    assert_eq!(state.focused(), 1);
}

/// `overlay::OverlayCloseController` の閉鎖要求を受けた呼び出し側が
/// dispatch すべき `"close"`（payload なし、`MenubarAction::Close`）は
/// 全 Menu を閉じる冪等操作であることを固定する（`overlay.rs` モジュール
/// doc §イシュー #1173 参照）。
#[test]
fn menubar_close_closes_open_menu_and_is_idempotent() {
    let mut state = Menubar::new(0, 2, Some(0), false, Orientation::Horizontal);

    let applied = dispatch(&mut state, "close", "");
    assert!(applied);
    assert_eq!(state.open(), None);

    let before = state;
    let applied = dispatch(&mut state, "close", "");
    assert!(applied);
    assert_eq!(state, before);
}

/// パース不能な payload（`trigger_count` を超える index・数値でない文字列）
/// を伴う `"toggle"`/`"focus"`/`"open"` は no-op になる fail-closed 契約の
/// 回帰（`Menubar::decode_action`/`normalize_focus`/`normalize_open`）。
#[test]
fn menubar_toggle_with_unparseable_payload_is_no_op() {
    let mut state = Menubar::new(0, 2, None, false, Orientation::Horizontal);
    let before = state;

    let applied = dispatch(&mut state, "toggle", "not-a-number");

    assert!(!applied);
    assert_eq!(state, before);
}

/// 既定エスケープ回帰（REQ-1）: navigation-menu の trigger/link/content へ
/// 攻撃者制御ラベル（`<script>` を含む）を渡した場合でも、`render()` の
/// 出力に生の `<script>` タグとして現れないことを固定する。
#[test]
fn navigation_menu_escapes_script_payload_in_label_and_link() {
    let payload = "<script>alert(1)</script>";
    let state = NavigationMenu::default();

    let node = state.trigger(
        "products",
        false,
        None,
        None,
        vec![],
        vec![fandhe_frontend_core::text(payload)],
    );
    let html = render(&node);

    assert!(
        !html.contains("<script>alert"),
        "raw <script> tag leaked into navigation-menu trigger html: {html}"
    );
    assert!(html.contains("&lt;script&gt;"), "html was: {html}");

    let link_node = navigation_menu::link(payload, false, vec![], vec![]);
    let link_html = render(&link_node);
    assert!(
        !link_html.contains(r#"href="<script>"#),
        "unescaped href payload leaked into navigation-menu link html: {link_html}"
    );
}

/// 既定エスケープ回帰（REQ-1）: menubar の item value/label へ攻撃者制御
/// ラベルを渡した場合でも、`render()` の出力に生の `<script>` タグとして
/// 現れないことを固定する（`data-value` 属性・子テキストの双方）。
#[test]
fn menubar_escapes_script_payload_in_item_value_and_label() {
    let payload = "<script>alert(1)</script>";

    let node = menubar::item(
        payload,
        false,
        false,
        vec![],
        vec![fandhe_frontend_core::text(payload)],
    );
    let html = render(&node);

    assert!(
        !html.contains("<script>alert"),
        "raw <script> tag leaked into menubar item html: {html}"
    );
    assert!(html.contains("&lt;script&gt;"), "html was: {html}");
}

/// `static/embed.html` の回帰テスト（イシュー #1199）。navigation-menu /
/// menubar のマウント要素があらかじめ `data-hydrate-*` 付き SSR 済み
/// マークアップを保持すること（`embed_html_interactive_root_has_hydrate_attrs_to_avoid_csr_fallback_id_collision`
/// と同型の理由: 空のまま `hydrate_navigation_menu`/`hydrate_menubar` を
/// 呼ぶと `data-hydrate-*` 属性が存在せず状態復元が失敗し、CSR フォール
/// バックによる二重差し込みが起きる）。
#[test]
fn embed_html_nav_menu_and_menubar_roots_have_hydrate_attrs() {
    let embed_html_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("static/embed.html");
    let html = std::fs::read_to_string(&embed_html_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", embed_html_path.display()));

    // `id="..."` 単独ではモジュール冒頭コメント（<title> 前・本文コメント）の
    // 説明文にも同じ文字列が出現するため、実タグでのみ隣接する
    // `data-testid="..."` まで含めて開始位置を一意に特定する
    // （`embed_html_interactive_root_has_hydrate_attrs_to_avoid_csr_fallback_id_collision`
    // と同じ手法）。
    for (needle, attrs) in [
        (
            r#"id="nav-menu-root" data-testid="nav-menu-root""#,
            vec!["data-hydrate-selected="],
        ),
        (
            r#"id="menubar-root" data-testid="menubar-root""#,
            vec![
                "data-hydrate-focused=",
                "data-hydrate-trigger-count=",
                "data-hydrate-open=",
                "data-hydrate-loop=",
                "data-hydrate-orientation=",
            ],
        ),
    ] {
        let root_start = html
            .find(needle)
            .unwrap_or_else(|| panic!("static/embed.html must contain the {needle} mount tag"));
        let tag_end = html[root_start..]
            .find('>')
            .map(|offset| root_start + offset)
            .unwrap_or_else(|| panic!("{needle} start tag must be closed with '>'"));
        let tag_slice = &html[root_start..tag_end];

        for attr in attrs {
            assert!(
                tag_slice.contains(attr),
                "static/embed.html の {needle} に {attr} がありません。\
                 空のまま hydrate_navigation_menu()/hydrate_menubar() を呼ぶと \
                 CSR フォールバックが二重に差し込まれ id 衝突が発生します。\
                 tag_slice was: {tag_slice}"
            );
        }
    }
}

/// `tag_slice`（開始タグ内部の文字列）から `attr="..."` 形式の属性値を
/// 抽出するテスト専用ヘルパー。embed.html の属性値は既定エスケープ済み
/// SSR 出力の転記であり `"` 自体は含まれない前提（`&quot;` にエスケープ
/// される、`fandhe_frontend_core::escape` 契約）ため、単純な `"..."` の
/// 対応で十分。
fn extract_attr_value(tag_slice: &str, attr_name: &str) -> Option<String> {
    let needle = format!(r#"{attr_name}=""#);
    let start = tag_slice.find(&needle)? + needle.len();
    let rest = &tag_slice[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}
