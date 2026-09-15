# examples/interactive-view-transitions

## 概要

`fandhe-frontend` フレームワークの状態管理（REQ-8）+ View Transitions の
正本サンプルです（イシュー #503）。`examples/ssr-routing`（イシュー #499、
examples 規約の初例）と同じ構成規約に従い、crates.io へ公開済みの
`fandhe-frontend-core` / `fandhe-frontend-app` / `fandhe-frontend-interactive`
（いずれも v0.4.3/v0.2.6/v0.2.7、イシュー #2525 で追随）をバージョン依存として
実際に使う「正本」です（`wasm/` の `fandhe-frontend-wasm-full` は独自系列の
v0.33.0）。

`fandhe-frontend-interactive` の状態機械 API（`Component` / `dispatch` /
`decode_action` / `render_for_hydration`）と、`page_shell` 同梱の
`@view-transition` at-rule + `fandhe-frontend-wasm-full` の `start_router`
（SPA 内遷移の View Transitions が JS 0 行で自動有効）を実演します。

イシュー #1199 で `fandhe-frontend-headless-ui`（v0.69.2、イシュー #2525 で
追随）の navigation-menu / menubar を追加し、`fandhe-frontend-wasm-full`
のオーバーレイ配線（`headless::MAPPING_TABLE`・
`overlay::OverlayCloseController`・`keynav`・`position::PositionedKind`
の scope enum 追加）を実演します。

イシュー #2525 で `fandhe-frontend-wasm-full` 0.33.0 系（Phase 4、親 #2508）
の配線群 feature へ追随し、in-view / gesture / scroll-driver / layout FLIP /
stagger の実演（`motion-demo-root` セクション・`item-list` の opt-in 属性）を
追加しました。

## 学べること

- `fandhe_frontend_interactive::Component` trait（`update` / `view` /
  `decode_action`）を実装した参照コンポーネント `AppState`（カウンター・
  フォーム入力・動的リスト）に対する `dispatch(component, name, payload)`
  境界関数の使い方
- 未知アクション名の `dispatch` が no-op（`false` を返し状態不変）になる
  安全側フォールバック契約
- `render_for_hydration` によるハイドレーション属性付き `Node` の組み立てと、
  `fandhe_frontend_core::render()` の既定エスケープ（REQ-1）
- `page_shell` 同梱の `@view-transition { navigation: auto; }` と、
  `fandhe-frontend-wasm-full::entry::start_router` によるクロスドキュメント /
  SPA 内ページ遷移時の View Transitions 自動有効化（JS 0 行）
- `hydrate`（`AppState` 系、`id="interactive-root"`）と `start_router`
  （`layout()` が組む `<div id="app-root">` 系）は**別系統・別 DOM**である
  契約（`fandhe-frontend-wasm-full` entry.rs の doc 参照）
- headless-ui `NavigationMenu`（`id="nav-menu-root"`）/ `Menubar`
  （`id="menubar-root"`）の状態機械（`SingleSelect`/`MenubarAction`）と、
  `wasm-full` のオーバーレイ配線 3 点セット:
  - `headless::wire_headless_component`（`data-scope`/`data-part` →
    文字列アクションの静的マッピング、trigger クリック → `"toggle"`）
  - `keynav::wire_keynav`（Arrow/Home/End/Escape のキーボード操作）
  - `overlay::OverlayCloseController`（Escape・外側クリックでの閉鎖要求。
    呼び出し側が `"deselect"`（NavigationMenu）/`"close"`（Menubar）を
    dispatch する契約）
  - `position::PositionController`（menubar の `positioner` パーツ、
    scroll/resize 契機の座標再計算。navigation-menu は `positioner` を
    持たないため対象外）
  `Runtime<C>`（`DirtyTracked + BindingSource` 要求）に載らない headless
  コンポーネントに対し、アプリ側が `wasm-full::entry` と同型の薄い
  ラッパー（`wasm/src/lib.rs::nav_overlays`）を自作する参照実装です
- `wasm/src/lib.rs`（`nav_overlays`）が使う `overlay`/`position` は
  `fandhe-frontend-wasm-full` の gating 対象外 API（`Runtime` を経由せず
  直接呼び出す公開 API）である点。`wasm/Cargo.toml` は
  `default-features = false` + 本サンプルが実際に使う配線のみを
  `features` で実指定しています（[wasm-full feature 選択ガイド](../../docs/guides/wasm-full-features.md)参照）
- **Phase 4 動きの実演（イシュー #2525）**: `<section id="motion-demo-root">`
  （`wasm/src/lib.rs::hydrate_motion_demo`）が `in_view::wire_in_view`/
  `gesture::wire_gesture`/`scroll_driver::wire_scroll_driver`（いずれも
  `Runtime` を経由しない汎用 opt-in `data-*` 属性配線）を直接呼びます。
  スクロールで `data-in-view` カードの opacity が変わり、hover/press で
  `data-fandhe-hover`/`data-fandhe-press` が付け外しされ、
  `--fandhe-motion-scroll-progress` がスクロール進捗バーへ書き込まれます
  （ネイティブ `animation-timeline: view()` 対応ブラウザでは JS 側の
  書き込みは行われません、`scroll_driver.rs` の doc 参照）。`item-list`
  （`hydrate` 系統の `<ul data-bind-list="items">`）には
  `data-fandhe-flip-auto`/`data-fandhe-stagger-auto-first` を
  `static/embed.html` 側で付与しており（`AppState::view()` 自体は変更
  していません）、「追加」→「削除」で残行が layout FLIP 移動し新規行が
  stagger 起点でフェードインします
- **View Transitions の汎用化・`animate`/`animation-driver`（イシュー
  #2525、feature 指定のみ）**: `start_router` は feature 非依存で既に
  `@view-transition` at-rule 経由の View Transitions を使うため、本
  サンプルに追加の呼び出しコードはありません。`Runtime::
  apply_with_view_transition`（`view-transitions` feature）は自前の
  `Runtime` を持つアプリ向けの API であり、`entry::hydrate`/`start_router`
  を使う本サンプルからは呼び出す経路がありません。`animate`
  （`fandhe_frontend_animation` の re-export のみ・宣言的配線なし）とあわせ、
  `wasm/Cargo.toml` の feature 指定例として掲載するに留めています。詳細は
  [`docs/guides/animation.md`](../../docs/guides/animation.md) /
  [`docs/guides/animation-core.md`](../../docs/guides/animation-core.md) を
  参照してください

## 前提

- Rust ツールチェーン（`cargo`）
- crates.io（`https://index.crates.io` / `https://static.crates.io`）への到達性
  （依存解決に使用します）
- `fw gate --project examples/interactive-view-transitions` を実行する場合は
  clippy component / cargo-deny が必要です（`tools/ci/ensure-gate-tools.sh`
  で導入できます）
- ブラウザでの実動作確認（wasm ビルド）には `rustup target add
  wasm32-unknown-unknown` と `wasm/Cargo.lock` が解決したバージョンと一致する
  wasm-bindgen-cli が必要です（`tools/wasm/build.sh` 参照）

## 動かし方

```bash
# native デモ: 状態機械の dispatch 実演 + dist/index.html への SSR HTML 書き出し
cargo run

# テスト（既定エスケープ回帰・状態機械の不変条件を含む）
cargo test

# fw gate（リポジトリルートから実行）
tools/ci/ensure-gate-tools.sh
cargo run -p fandhe-frontend-cli -- gate --project examples/interactive-view-transitions

# ブラウザでの実動作確認（wasm ビルド。事前に rustup target add
# wasm32-unknown-unknown と wasm-bindgen-cli の導入が必要）
tools/wasm/build.sh
python3 -m http.server --directory static 8000
# ブラウザで http://localhost:8000/embed.html を開く
# （history API を使う data-nav 遷移の確認には file:// ではなく HTTP 配信が必須）
```

## 主要ファイル

| ファイル | 説明 |
|---------|------|
| `Cargo.toml` | crates.io バージョン依存 4 件（`fandhe-frontend-core` / `-app` / `-interactive` / `-headless-ui`、イシュー #1199 で `-headless-ui` を追加、イシュー #2525 で 0.4.3/0.2.6/0.2.7/0.69.2 へ追随）。root workspace から独立した `[workspace] members = ["."]` |
| `structure.toml` | `fw gate` が唯一の情報源として読む構造マニフェスト |
| `clippy.toml` | `raw_html()` 迂回検出ポリシー（`templates/default/` と内容同一） |
| `deny.toml` | 依存ポリシー（`templates/default/` と内容同一） |
| `src/main.rs` | native デモ（`AppState`/`NavigationMenu`/`Menubar` の `dispatch` 実演）+ `dist/index.html` への SSR HTML 書き出し。`motion_demo_view`（イシュー #2525）が motion デモのマークアップを追加 |
| `tests/state_machine.rs` | `dispatch` の状態遷移・未知アクション no-op・`render_for_hydration`・既定エスケープ回帰・`static/embed.html` のハイドレーション属性 + motion/FLIP/stagger opt-in 属性回帰テスト |
| `static/embed.html` | ブラウザマウント骨格。`tools/wasm/build.sh` 実行後に動作（`hydrate("interactive-root")` / `start_router("app-root")` / `hydrate_navigation_menu("nav-menu-root")` / `hydrate_menubar("menubar-root")` / `hydrate_motion_demo("motion-demo-root")`）。5 つのマウント要素はいずれも `cargo run` が書き出す `dist/index.html` の同要素を事前に埋め込み済みで、各 `hydrate*()` の状態復元が成功する（空のまま呼ぶと CSR フォールバックが二重に差し込まれ id 衝突するため）。`item-list` には layout FLIP / stagger の opt-in 属性を手動付与している |
| `tools/wasm/build.sh` | `wasm/`（独立ワークスペースの glue クレート）を wasm32 へビルドする手順 |
| `wasm/` | `fandhe-frontend-wasm-full` の `hydrate` / `mount` / `start_router` を再エクスポートし、`nav_overlays` モジュール（イシュー #1199）で `hydrate_navigation_menu` / `hydrate_menubar` を、`motion_demo` モジュール（イシュー #2525）で `hydrate_motion_demo` を自前実装する glue クレート（root の依存グラフから隔離） |

## 関連ガイド

- [`docs/guides/quickstart.md`](../../docs/guides/quickstart.md)
- [`docs/guides/wasm-full-features.md`](../../docs/guides/wasm-full-features.md)
- [`docs/guides/animation.md`](../../docs/guides/animation.md)
- [`docs/guides/animation-core.md`](../../docs/guides/animation-core.md)
- [`docs/api/interactive-api.md`](../../docs/api/interactive-api.md)
- [`docs/api/hydration-api.md`](../../docs/api/hydration-api.md)
- [`docs/design/wasm-full-architecture.md`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/design/wasm-full-architecture.md)
