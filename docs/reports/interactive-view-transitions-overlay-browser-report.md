# interactive-view-transitions オーバーレイ実演の実ブラウザ検証レポート（#1203）

## 1. 目的・トレーサビリティ

PR #1200（イシュー #1199）で `examples/interactive-view-transitions/wasm`
（`interactive-vt-wasm`）へ追加した navigation-menu / menubar オーバーレイ実演
（`hydrate_navigation_menu`/`hydrate_menubar`、`wasm/src/lib.rs` の
`nav_overlays` モジュール）は、実装環境の制約により Escape / 外側クリック /
矢印キー等の実ブラウザインタラクション確認が未実施のままマージされた。本
イシュー（親 #1201）は、`wasm-pack test --headless --chrome` による実測でこの
検証ギャップを埋め、結果を実測レポートとして記録することを目的とする。

マークアップ・配線順序自体（`OverlayCloseController`・`wire_keynav`・
`wire_headless_component`）は `crates/wasm-full/tests/overlay_close_browser.rs`
（イシュー #585 系）・`crates/wasm-full/tests/keynav_browser.rs`（イシュー
#581 系）が CI で pass 済みだが、これらは `fandhe-frontend-wasm-full`
本体のフィクスチャであり、example 側が自前実装するアプリ固有の統合コード
（`nav_overlays::render_and_sync_nav_menu`/`render_and_sync_menubar`/
`sync_shared_overlays`/`ensure_shared_overlay` 等、`SHARED_OVERLAY` を
navigation-menu・menubar 双方で共有する example 固有の設計、#1200 Bugbot
指摘対応）はカバーしない。本レポートはその統合コードを実 DOM で検証した
一次記録である。

## 2. 総括

実測の結果、**click によるトグル開閉・キーボードによるトリガー間フォーカス
移動は正しく動作する一方、Escape / 外側クリックによるオーバーレイ閉鎖は
navigation-menu・menubar のいずれでも機能しない実バグを発見した**。原因は
`examples/interactive-view-transitions/wasm/src/lib.rs`（example 正本、本
イシューでは変更しない）の `sync_shared_overlays()` が、呼び出し元の
`RefCell<Component>` 借用が生存したまま再入して `try_borrow()` するため
常に `Err` となり、click で新しく開いた項目が `SHARED_OVERLAY` スタックへ
一度も push されない、という再入（reentrancy）バグである。詳細は §5 参照。

このバグは example 固有の統合コードに限定され、`fandhe-frontend-wasm-full`
本体（`crates/wasm-full/src/overlay.rs`/`headless.rs`）には存在しない
（§5.3 参照）。本イシューのスコープは実測・記録であり、example 正本の修正は
行わない（§7 対象外参照）。

## 3. 実測環境

| 項目 | 値 |
|------|-----|
| 対象コミット | `b59b2d1fde589fa3fe183cc1f68c29603ee60f89`（本 PR のベース、main） |
| OS | Linux 7.0.0-28-generic |
| rustc | 1.96.0 (ac68faa20 2026-05-25) |
| Chromium | 150.0.7871.128（snap） |
| chromedriver | 150.0.7871.128（システム導入済み `/usr/bin/chromedriver`。Chromium と同一メジャーバージョン） |
| wasm-pack | 0.15.0 |
| wasm-bindgen（解決バージョン） | 0.2.126 |
| 導入経路 | いずれもローカル環境に既導入済みのバイナリを使用（新規ダウンロードなし）。`.github/workflows/ci.yml` の `browser-test` ジョブが導入する Chrome for Testing 151.0.7922.34 とは異なるバージョン系だが、メジャーバージョンが Chromium 150.x で chromedriver と一致しており、同一挙動が期待できる範囲内と判断した |

## 4. ハーネス構成（scratch・非コミット）

`.claude/rules/ci.md`・`docs/policy/intentional-non-adoption.md` の判断軸に
基づき、常設 CI 化は見送り（§6 参照）、使い捨てハーネスによる一度きりの実測を
採用した。

- 配置: `target/tmp/ivt-overlay-browser-1203/wasm/`（イシュー #637 の配置
  規約に整合。`/tmp` 直下は使わない。実測後に削除済み、リポジトリへは含まれない）
- 生成方法: `examples/interactive-view-transitions/wasm/`（**example 正本、
  無編集のコピー元**）を丸ごとコピーし、コピー側にのみ以下の変更を加えた
  （**example 正本 `examples/interactive-view-transitions/wasm/` はイシュー
  #1203 のこの PR で一切変更していない**）。

### 4.1 `Cargo.toml` 差分（コピー側のみ）

```diff
 [dependencies.web-sys]
 version = "0.3"
-features = ["Window", "Document", "Element"]
+features = [
+    "Window",
+    "Document",
+    "Element",
+    "Event",
+    "EventInit",
+    "KeyboardEvent",
+    "KeyboardEventInit",
+    "HtmlElement",
+    "DomTokenList",
+]
+
+# scratch ハーネス専用の変更（イシュー #1203。example 正本は変更しない）:
+# tests/ 配下の integration test からこのクレートの公開関数
+# （hydrate_navigation_menu/hydrate_menubar）を呼び出すには rlib が必要。
+[dev-dependencies]
+wasm-bindgen-test = "0.3"

 [lib]
-crate-type = ["cdylib"]
+crate-type = ["cdylib", "rlib"]
```

（`Cargo.lock` はコピー側で削除し、`wasm-pack test` 実行時に再生成させた。
example 正本の `Cargo.lock` には触れていない。)

### 4.2 追加した integration test（`tests/*.rs`、コピー側のみ）

`nav_overlays::SHARED_OVERLAY`/`NAV_MENU_ROOT`/`MENUBAR_ROOT` は
`thread_local!`（example 正本 `wasm/src/lib.rs`）であり、同一 `.wasm`
インスタンス内で複数テストを実行すると前のテストの状態が残留する
（`wasm-pack test` は `tests/*.rs` の**ファイル単位**で別々の `.wasm`
バイナリを生成するため、ファイルを分ければテストごとに独立した
thread_local を得られる）。このため 1 ファイル 1 テストへ分割し、共通の
フィクスチャ生成・イベント合成のみ `tests/support.rs` へ切り出した。

`tests/support.rs`（共通ヘルパー、全文）:

```rust
//! イシュー #1203 scratch ハーネス共通ヘルパー（非コミット）。
//!
//! `nav_overlays::SHARED_OVERLAY`/`NAV_MENU_ROOT`/`MENUBAR_ROOT` は
//! `thread_local!`（`examples/interactive-view-transitions/wasm/src/lib.rs`
//! 参照）であり、同一 `.wasm` インスタンス内で複数テストを実行すると
//! 前のテストの状態が残留する（`wasm-pack test` は `tests/*.rs` の
//! ファイル単位で別々の `.wasm` バイナリを生成するため、ファイルを
//! 分ければテストごとに独立した thread_local を得られる）。このため各
//! シナリオは個別の `tests/nav_overlay_browser_*.rs` ファイルへ分離し、
//! 本モジュールは共通のフィクスチャ生成・イベント合成のみを提供する。

#![cfg(target_arch = "wasm32")]
#![allow(dead_code)]

use wasm_bindgen::JsCast;
use web_sys::{Document, Element, Event, EventInit, HtmlElement, KeyboardEvent, KeyboardEventInit};

/// テスト用のプレースホルダ要素を document body へ 1 個生成する
/// （`crates/wasm-full/tests/overlay_close_browser.rs::create_placeholder`
/// と同型）。
pub fn create_placeholder(document: &Document, id: &str) -> Element {
    let container = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    container.set_id(id);
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&container)
        .expect("append_child must not fail for a detached div");
    container
}

/// `hydrate_navigation_menu(root_id)` の対象要素を、実 SSR
/// （`examples/interactive-view-transitions/src/main.rs::nav_menu_view` が
/// `navigation_menu::root(...)` で出力する `<nav data-scope="navigation-menu"
/// data-part="root">`）と同じ anatomy 属性を持つ状態で用意する。
///
/// `crates/wasm-full/src/keynav.rs::handle_navigation_menu_trigger_keydown`
/// は `closest(trigger, NAVIGATION_MENU_ROOT_SELECTOR)`
/// （`[data-scope="navigation-menu"][data-part="root"]`）でトリガー間移動の
/// 対象範囲を確定するため、この属性を欠くプレースホルダ（素の `div`）では
/// キーボード操作の配線が一切発火しない（イシュー #1203 実測で判明。
/// §5.2「フィクスチャ属性の要否」節参照）。
pub fn create_navigation_menu_root(document: &Document, id: &str) -> Element {
    let root = create_placeholder(document, id);
    let _ = root.set_attribute("data-scope", "navigation-menu");
    let _ = root.set_attribute("data-part", "root");
    let _ = root.set_attribute("aria-label", "Main");
    root
}

/// `hydrate_menubar(root_id)` の対象要素を、実 SSR
/// （`examples/interactive-view-transitions/src/main.rs::menubar_view` が
/// `menubar::root(...)` で出力する `<div data-scope="menubar"
/// data-part="root" role="menubar" data-orientation="horizontal">`）と
/// 同じ anatomy 属性を持つ状態で用意する（[`create_navigation_menu_root`]
/// と同じ理由。menubar の keydown 配線も `MENUBAR_ROOT_SELECTOR` で
/// `data-scope="menubar"][data-part="root"]` を要求する）。
pub fn create_menubar_root(document: &Document, id: &str) -> Element {
    let root = create_placeholder(document, id);
    let _ = root.set_attribute("data-scope", "menubar");
    let _ = root.set_attribute("data-part", "root");
    let _ = root.set_attribute("role", "menubar");
    let _ = root.set_attribute("data-orientation", "horizontal");
    root
}

/// テスト末尾でプレースホルダを document から確実に除去する RAII ガード。
pub struct RemoveOnDrop(pub Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

pub fn keydown_event(key: &str) -> Event {
    let init = KeyboardEventInit::new();
    init.set_key(key);
    init.set_bubbles(true);
    KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .expect("KeyboardEvent::new must not fail")
        .unchecked_into::<Event>()
}

pub fn pointerdown_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("pointerdown", &init).expect("Event::new must not fail")
}

pub fn document() -> Document {
    web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist in browser test environment")
}

pub fn by_id(document: &Document, id: &str) -> Element {
    document
        .get_element_by_id(id)
        .unwrap_or_else(|| panic!("element #{id} must exist after hydrate"))
}

pub fn click(element: &Element) {
    element
        .dyn_ref::<HtmlElement>()
        .expect("target must be an HtmlElement")
        .click();
}

pub fn is_hidden(element: &Element) -> bool {
    element.has_attribute("hidden")
}

pub fn active_element_id(document: &Document) -> Option<String> {
    document.active_element().map(|el| el.id())
}
```

各シナリオファイル（`tests/nav_overlay_browser_0{1..9}_*.rs`）は上記
`support.rs` を `#[path = "support.rs"] mod support;` で取り込み、`id` のみを
持つ root 要素（`data-hydrate-*` を持たせない）を経由して
`hydrate_navigation_menu`/`hydrate_menubar` を呼ぶ。復元失敗（`data-hydrate-*`
欠落）は `NavigationMenu::default()`/`Menubar::new(0, 2, None, false,
Orientation::Horizontal)` への安全側フォールバックで example 自身の
レンダラが全マークアップを再描画するため、以降の操作対象はすべて example
の実出力である。個々のテスト本体（`click()`/`dispatch_event()`/`assert_eq!`
呼び出し）は §5.1 の結果表の各行に対応し、全文は本 PR のブランチ履歴では
なく本節の記述と `crates/wasm-full/tests/overlay_close_browser.rs`/
`keynav_browser.rs` の既存パターン（`create_placeholder`/`keydown_event`/
`pointerdown_event`/`RemoveOnDrop` の設計を踏襲）から再現可能である。

### 4.3 実行コマンド

```bash
cd target/tmp/ivt-overlay-browser-1203/wasm
export CHROMEDRIVER=/usr/bin/chromedriver
export CARGO_TARGET_DIR=<repo>/target/tmp/ivt-overlay-browser-1203/target
wasm-pack test --headless --chrome . --test <test-file-name>
```

## 5. 結果

### 5.1 結果表

| # | ファイル | テスト名 | 検証内容 | 結果 |
|---|---------|---------|---------|------|
| 1 | `nav_overlay_browser_01_click_toggle.rs` | `nav_menu_trigger_click_opens_and_second_click_closes` | navigation-menu trigger クリックでの開閉（`data-state`/`aria-expanded`/`hidden`） | **PASS** |
| 2 | `nav_overlay_browser_02_escape.rs` | `nav_menu_escape_closes_open_item` | navigation-menu 開状態での document Escape 押下による閉鎖 | **FAIL**（§5.2 実バグ） |
| 3 | `nav_overlay_browser_03_outside_pointerdown.rs` | `nav_menu_outside_pointerdown_closes_open_item` | navigation-menu 開状態での外側 pointerdown による閉鎖 | **FAIL**（§5.2 実バグ、同一原因） |
| 4 | `nav_overlay_browser_04_inside_pointerdown.rs` | `nav_menu_inside_content_pointerdown_does_not_close` | content 内側 pointerdown では閉鎖しないことの回帰 | **PASS** |
| 5 | `nav_overlay_browser_05_keynav.rs` | `nav_menu_arrow_right_moves_focus_between_triggers` | `wire_keynav` によるトリガー間フォーカス移動（ArrowRight） | **PASS**（フィクスチャに `data-scope="navigation-menu" data-part="root"` を追加後） |
| 6 | `nav_overlay_browser_06_menubar_click.rs` | `menubar_trigger_click_opens_and_second_click_closes` | menubar trigger クリックでの開閉 | **PASS** |
| 7 | `nav_overlay_browser_07_menubar_escape_outside.rs` | `menubar_escape_and_outside_pointerdown_close` | menubar 開状態での Escape・外側 pointerdown による閉鎖 | **FAIL**（§5.2 実バグ、menubar 側でも再現） |
| 8 | `nav_overlay_browser_08_menubar_keynav.rs` | `menubar_arrow_right_moves_trigger_focus` | `wire_keynav` による menubar トリガー間フォーカス移動 | **PASS**（フィクスチャに `data-scope="menubar" data-part="root"` 等を追加後） |
| 9 | `nav_overlay_browser_09_shared_overlay.rs` | `shared_overlay_escape_closes_only_topmost` | 共有 `OverlayCloseController` の統合検証（navigation-menu・menubar 同時開状態での Escape が最上位のみ閉じる） | **FAIL**（§5.2 実バグの複合影響） |

**9 件中 5 件 PASS・4 件 FAIL**。所要時間はいずれも 1 テストあたり
0.01〜0.04 秒（`wasm-pack test` 実行時間はブラウザ起動込みで 1 ファイル
あたり数秒〜十数秒）。

### 5.2 発見した実バグ: click 由来の状態更新がオーバーレイスタックへ反映されない（再入バグ）

**症状**: navigation-menu / menubar のトリガーをクリックして開いた項目は、
DOM 上は正しく開状態（`data-state="open"`/`aria-expanded="true"`/`hidden`
属性解除）になるが、その後の Escape 押下・外側クリックのいずれでも閉じない
（テスト #2/#3/#7/#9 が再現）。

**原因**: `examples/interactive-view-transitions/wasm/src/lib.rs`
（`nav_overlays` モジュール）の以下の呼び出し連鎖に、`RefCell` の同時借用
違反による無言の no-op（fail-closed 設計ゆえに panic はしないが、意図した
push も起きない）が存在する。

1. トリガークリック → `fandhe_frontend_wasm_full::headless::wire_headless_component`
   （`crates/wasm-full/src/headless.rs:575-596`）の登録済みコールバックが
   発火する。
2. コールバック内で `let Ok(mut state) = component.try_borrow_mut() else { return; };`
   により `Rc<RefCell<C>>` を**ミュータブルに借用したまま**、
   `fandhe_frontend_interactive::dispatch(&mut *state, ...)` を実行し、
   続けて **同じ借用を保持したまま** `(on_update.borrow_mut())(&state, &wired_root);`
   を呼ぶ（`headless.rs:583-596`。`state` という `RefMut` はこの行の実行中
   もドロップされていない）。
3. `on_update` として渡されているのが example 側の
   `render_and_sync_nav_menu`/`render_and_sync_menubar`（`wasm/src/lib.rs`）
   であり、その内部で `sync_shared_overlays()` を呼ぶ。
4. `sync_shared_overlays()` は `NAV_MENU_STATE`/`MENUBAR_STATE`
   （`thread_local!`、`hydrate_navigation_menu`/`hydrate_menubar` が保持する
   のと**同一の** `Rc<RefCell<C>>`）から `state.try_borrow()`
   （**新規の共有借用**）を試みるが、手順 2 のミュータブル借用がまだ
   生存しているため必ず `Err` となり、`if let Ok(current) = state.try_borrow()`
   の分岐に入らず**現在開いている項目を `SHARED_OVERLAY` へ push する処理が
   スキップされる**。

結果として、click 由来で新しく開いた項目は一度も `OverlayCloseController`
のスタックに登録されず、Escape・外側クリックの閉鎖判定
（`overlay::escape_close_index`/`outside_close_indices`）の対象外のまま
残る。SSR 初期状態（マウント前から開いている項目）だけは、この借用が
存在しない `hydrate_*` 関数末尾の `sync_shared_overlays()` 呼び出し
（借用の外側）で正しく push されるため、その経路に限り閉鎖が機能する。

**フィクスチャ属性の要否（副次的な発見）**: 上記とは独立に、テスト #5/#8
（キーボードによるトリガー間移動）は、当初 `id` のみを持つ素の `div` を
root 要素に使った状態では常に no-op だった。原因はテストフィクスチャ側の
不備であり、実装のバグではない: `crates/wasm-full/src/keynav.rs` の
`handle_navigation_menu_trigger_keydown`/menubar 側の対応処理は
`closest(trigger, NAVIGATION_MENU_ROOT_SELECTOR)`
（`[data-scope="navigation-menu"][data-part="root"]`）でトリガー集合の
探索範囲を確定するが、`nav_overlays::hydrate_navigation_menu`/
`hydrate_menubar` は root 要素**自身**へこの `data-scope`/`data-part`
属性を付与しない（`nav_menu_content`/`menubar_content` は root 要素を含まない
子ノードのみを返す設計、`apply_root_hydrate_attrs` が付与するのは
`data-hydrate-*` のみ）。実運用（`examples/interactive-view-transitions/
src/main.rs::nav_menu_view`/`menubar_view`）では `navigation_menu::root(...)`/
`menubar::root(...)` の出力する要素自体に `id="nav-menu-root"`/
`id="menubar-root"` を付けて `hydrate_*` へ渡しているため、SSR 出力には
この属性が最初から乗っており実運用では問題にならない。§4.2 の
`create_navigation_menu_root`/`create_menubar_root` へ修正後、テスト
#5/#8 はいずれも PASS した。

### 5.3 `fandhe-frontend-wasm-full` 本体への影響有無

§5.2 のバグは `examples/interactive-view-transitions/wasm/src/lib.rs`
（`nav_overlays` モジュール、example 固有のアプリ側配線コード）に閉じており、
`crates/wasm-full/src/headless.rs::wire_headless_component` 自体の契約
（`on_update: impl FnMut(&C, &web_sys::Element)` の呼び出し中に `C` の
`RefCell` がミュータブル借用されたままであること）は明示された仕様であり
（同関数 rustdoc「配線は状態更新・再描画に結合しない」方針、モジュール doc
参照）、`on_update` 実装側が同じ `Rc<RefCell<C>>` への**別経路の借用**を
`on_update` の呼び出し中に試みない限り問題は起きない。`crates/wasm-full/
tests/overlay_close_browser.rs` は `OverlayCloseController` を都度
`mount_dialog`/`recording_controller` で独立に構築し、`on_update` 内で
同じ `RefCell` への再借用を行わないフィクスチャのため、このバグを検出
できない設計だった（本イシューの動機どおり、example 固有の統合コードの
検証ギャップが実際に実バグを覆っていた事例）。

## 6. 常設 CI 化を見送る判断・根拠

使い捨てハーネス（scratch ワークスペース）による一度きりの実測に留め、
常設 CI 化（`examples/interactive-view-transitions/wasm` へのテスト同梱・
`.github/workflows/ci.yml` の `browser-test` ジョブへのステップ追加）は
見送る。

1. **論理の CI 担保は既存**: オーバーレイ閉鎖制御の中核ロジック（Escape /
   外側クリック / opt-out / 未知 scope no-op / XSS 経路）は
   `crates/wasm-full/tests/overlay_close_browser.rs` が CI で常設実行済み。
   §5.3 のとおり、今回発見したバグは wasm-full 本体ではなく example 側の
   統合コードに限定される。
2. **example smoke の先例**: 既存 examples e2e
   （`crates/cli/tests/new_gate_e2e.rs`）は `cargo build`/`cargo run`
   起動確認・`fw gate` までであり（`.claude/rules/ci.md`）、example
   固有の browser テスト常設はこの先例を超える。
3. **同期・バンプ連鎖の回避**: example 正本へテストファイル・
   `wasm-bindgen-test` dev 依存を追加すると `crates/cli/embedded-examples/`
   のバイト一致同期（`example_publish_copy_drift` テスト）と
   `fandhe-frontend-cli` の semver バンプ（イシュー #638 規約）が連鎖し、
   しかも CI で実行しない限りそのテストは headless-ui / wasm-full バンプの
   たびにサイレントに腐る。CI で実行するなら `browser-test` ジョブへ
   example ワークスペース分の crates.io 依存フルビルドが加わり CI 時間増が
   大きい。
4. **再現可能性はレポートで担保**: 本レポート §4 にハーネスの全差分・全文・
   実行コマンド・環境情報を埋め込み、誰でも再実測できる状態にした。

**再評価トリガー**: example のオーバーレイ実演が今後 `Runtime<C>` 非対応の
独自配線をさらに増やす場合、または wasm-full 側の overlay/keynav API に
破壊的変更が入り example 追随の失敗が実際に発生した場合は、常設 CI 化を
再検討する。その際は `.claude/rules/ci.md` のツール前提明示に従う。

## 7. 対象外（out-of-scope）・フォローアップ提案

- **§5.2 の再入バグ自体の修正**: `examples/interactive-view-transitions/wasm/
  src/lib.rs`（example 正本）の変更を要するため、本イシュー（実測・記録が
  スコープ）では行わない。`.claude/rules/out-of-scope-tracking.md` に従い、
  別イシューとしての起票を提案する（ユーザー承認後に起票、想定タイトル例:
  「`nav_overlays` の click 経路で `SHARED_OVERLAY` への push が
  `RefCell` 再入により常にスキップされる不具合を修正する」。親 #1201 配下）。
  修正方針の一案としては、`on_update` 内で `sync_shared_overlays()` を直接
  呼ばず、`RefCell` の借用が解放された後（`wire_headless_component` の
  コールバック終了後）に非同期でスタック同期を行う、または
  `sync_shared_overlays()` 内で `NAV_MENU_STATE`/`MENUBAR_STATE` の
  `try_borrow()` 失敗時に渡された `state: &C` 引数（既に借用済みの参照）を
  再利用する形へ設計変更する、等が考えられる（いずれも詳細検討は別イシュー
  側で行う）。**イシュー #1209 で修正済み**（後者の方針を採用。再実測結果は
  §10 参照）。
- **wasm-full 本体・キー配線ロジックの修正**: §5.3 のとおり原因は example
  側にあり、`crates/wasm-full/` の変更は不要と判断した。
- **常設 CI 化**: §6 のとおり現時点では見送り。
- **他 examples への同種実演追加・検証**: イシュー #1204 の担当領域。
- **`wasm/src/lib.rs` と `src/main.rs` の同名ビュー関数ドリフト検知**:
  イシュー #1202 の担当領域。

## 8. セキュリティ考慮事項（OWASP Top 10 観点）

- **A01 アクセス制御**: scratch ハーネスの書き込み先は repo 配下
  `target/tmp/ivt-overlay-browser-1203/` に限定し、実測後に削除した
  （リポジトリへ残置していない）。
- **A03 インジェクション / XSS**: テストの DOM 構築は `create_element`/
  `set_id`/`set_attribute` 等の API のみで行い、HTML 文字列組み立て・
  `set_inner_html` の直接使用は行っていない（REQ-1・`docs/guides/
  browser-testing.md` §8 の不変条件に整合）。既存の XSS 回帰
  （`overlay_close_browser.rs` の `data-value` XSS ケース等）は削除・
  弱体化していない（無編集のまま）。
- **A05 セキュリティ設定ミス**: CI ワークフロー・`deny.toml`・
  `structure.toml` はいずれも変更していない。
- **A08 サプライチェーン**: ブラウザ・ドライバはシステムに導入済みの
  バイナリをそのまま使用し、その場の最新版を `cargo install`/自動
  ダウンロードしていない。恒久的な新規依存クレート追加はゼロ
  （`wasm-bindgen-test` は非コミットの scratch コピー内のみに追加）。
- **A09 ログ・監視**: 本レポートへユーザー名を含む絶対パス・トークン等は
  含めていない（パスは repo 相対または `<repo>/target/tmp/...` の記号で
  記載）。

## 9. 変更ファイル

本 PR で変更したのは本レポートと `docs/guides/browser-testing.md`
（§10 追記）のみである。`examples/interactive-view-transitions/wasm/`
（example 正本）・`crates/cli/embedded-examples/`・
`.github/workflows/` はいずれも無編集のため、`fandhe-frontend-cli` を
含む公開済みクレートの semver バンプは不要（`.claude/rules/coding-rust.md`
「公開済みクレートの実体変更時は semver バンプ必須」の対象外。実体変更
なし）。

## 10. 修正後の再実測結果（イシュー #1209）

§7 で対象外としていた §5.2 の再入バグ自体を、イシュー #1209
（`examples/interactive-view-transitions/wasm/src/lib.rs` の
`sync_shared_overlays()` を `sync_shared_overlays_with(nav_menu_snapshot,
menubar_snapshot)` へ拡張し、`wire_headless_component` の `on_update`
経路〔`render_and_sync_nav_menu`/`render_and_sync_menubar`〕からは呼び出し元が
既に保持している `&NavigationMenu`/`&Menubar` をスナップショットとして渡す
ことで、同一 `RefCell` への再入 `try_borrow()` を回避する設計）で修正した。
本節は §4 と同一のハーネス構成・同一の 9 シナリオを、修正後コミットに対して
再実測した結果を記録する。

### 10.1 実測環境

| 項目 | 値 |
|------|-----|
| 対象コミット | イシュー #1209 修正コミット（本 PR、base: main） |
| OS | Linux 7.0.0-28-generic |
| rustc | 1.96.0 相当（§3 と同一環境） |
| Chromium | 150.0.7871.128（snap） |
| chromedriver | 150.0.7871.128（システム導入済み `/usr/bin/chromedriver`） |
| wasm-pack | 0.15.0 |
| ハーネス構成 | §4 と同一（`target/tmp/ivt-overlay-browser-1209/wasm/` へ example 正本を丸ごとコピーし、§4.1/§4.2 と同一の `Cargo.toml` 差分・`tests/support.rs`・9 シナリオファイルを配置。実測後に削除済み、リポジトリへは含まれない） |

### 10.2 結果表

| # | ファイル | テスト名 | 修正前（§5.1） | 修正後 |
|---|---------|---------|---------------|--------|
| 1 | `nav_overlay_browser_01_click_toggle.rs` | `nav_menu_trigger_click_opens_and_second_click_closes` | PASS | **PASS** |
| 2 | `nav_overlay_browser_02_escape.rs` | `nav_menu_escape_closes_open_item` | FAIL | **PASS** |
| 3 | `nav_overlay_browser_03_outside_pointerdown.rs` | `nav_menu_outside_pointerdown_closes_open_item` | FAIL | **PASS** |
| 4 | `nav_overlay_browser_04_inside_pointerdown.rs` | `nav_menu_inside_content_pointerdown_does_not_close` | PASS | **PASS** |
| 5 | `nav_overlay_browser_05_keynav.rs` | `nav_menu_arrow_right_moves_focus_between_triggers` | PASS | **PASS** |
| 6 | `nav_overlay_browser_06_menubar_click.rs` | `menubar_trigger_click_opens_and_second_click_closes` | PASS | **PASS** |
| 7 | `nav_overlay_browser_07_menubar_escape_outside.rs` | `menubar_escape_and_outside_pointerdown_close` | FAIL | **PASS** |
| 8 | `nav_overlay_browser_08_menubar_keynav.rs` | `menubar_arrow_right_moves_trigger_focus` | PASS | **PASS** |
| 9 | `nav_overlay_browser_09_shared_overlay.rs` | `shared_overlay_escape_closes_only_topmost` | FAIL | **PASS** |

**9 件中 9 件 PASS**（従来 5 PASS / 4 FAIL から全 FAIL 解消）。§5.2 で
FAIL していた 4 件（#2/#3/#7/#9）はいずれも click 由来で開いた項目が
Escape・外側 pointerdown で正しく閉じるようになったことを確認した。

### 10.3 補足

- ハーネス差分（`Cargo.toml`・`tests/support.rs`・9 シナリオファイルの内容）は
  §4 からの変更なし。差分は「テスト対象のコミット（修正後）」のみである。
- §5.2 で発見した「フィクスチャ属性の要否」（`data-scope`/`data-part` を
  root 要素へ付与する必要）も同じ制約のまま再現しており、テスト #5/#8 は
  §4.2 の `create_navigation_menu_root`/`create_menubar_root` を経由して
  PASS した。
- §6 の「常設 CI 化を見送る判断」は本修正後も変更しない（判断根拠・
  再評価トリガーは §6 のまま）。
