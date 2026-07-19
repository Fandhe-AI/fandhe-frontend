//! `rws-wasm-client`: 最小ハイドレーション方式のクライアントランタイム（TASK-6.2b）。
//!
//! REQ-6（`docs/spec/04-requirements.md`）の受け入れ基準「ハイドレーション
//! （`hydrate()`）が、サーバー出力済み DOM を再構築せず、既存 DOM へ
//! イベントリスナーを後付けする最小ハイドレーション方式で成立すること」を
//! 満たす実装クレート。設計は `docs/api/hydration-api.md`（TASK-6.2a、凍結済み）
//! に従う。実装と同書に乖離が生じた場合は同書を正とする。
//!
//! # 2 層構成（`rws-wasm-full` の先例を踏襲）
//!
//! 1. **純粋ロジック層**（本ファイル直下、ネイティブテスト可能）:
//!    `rws_app::Loader`（#347・イシュー #346 設計確定書
//!    `docs/design/loader-trait-design.md`）経由でデータを解決し
//!    `rws_core::render` を呼ぶ CSR 用 HTML 生成（[`render_list_page_html`] /
//!    [`render_detail_page_html`])、および `rws_core::find_attr_values` /
//!    `rws_core::find_nav_targets` を用いたハイドレーション対象の特定
//!    （[`find_hydrate_target_kinds`] / [`find_list_nav_targets`]）。実 DOM 型
//!    （`web-sys` 各型）には一切依存しないため、wasm ビルドを介さず
//!    `cargo test -p rws-wasm-client` で検証できる
//!    （`wasm-client/tests/hydration_targets.rs`）。イシュー #375 で
//!    `rws_app::demo_items()` の直接呼び出しを排し、`rws-server`（#348）・
//!    `rws-wasm-full`（#349）と同一の Loader 契約へ統一した
//!    （「データ取得の実装は 1 箇所・モード別分岐なし」REQ-6）。
//! 2. **wasm32 配線層**（[`mod wiring`]、`#[cfg(target_arch = "wasm32")]`）:
//!    `#[wasm_bindgen]` エクスポート `hydrate` / `mount_csr`。実 DOM
//!    （`web-sys`）を操作するのはこの層のみに限定する。
//!
//! # セキュリティ不変条件（`docs/api/hydration-api.md` 第 6 節を引き継ぐ）
//!
//! 1. DOM への HTML 挿入は [`rws_core::render`] の出力（既定エスケープ済み）
//!    **のみ**を経由する。`format!` による HTML 断片組み立て・ユーザー入力
//!    の直接 `set_inner_html` 渡しを行わない。
//! 2. `hydrate()` は対象 DOM に対し `set_inner_html` 等の再構築系 API を
//!    **一切呼ばない**。イベントリスナーの後付け（`add_event_listener_with_callback`）
//!    のみを行う。
//! 3. ハンドラ内 DOM 更新・束縛点ベースの最小更新（[`binding`]/[`binding_dom`]、
//!    イシュー #343）は `set_text_content` / `set_attribute` / `class_list`
//!    （`DomTokenList`）のテキスト・属性・class API に限定する。`data-bind-*`
//!    束縛点（`rws_core::bind`、#342）と `DirtyTracked::dirty_fields()`
//!    （`rws_interactive`、#341）から駆動する汎用経路（[`binding_dom::BindingTable`]）
//!    もこの限定に従い、DOM 再構築なし・イベントリスナー保持を維持する
//!    （`docs/design/dom-binding-update-design.md` §4.1・§9 不変条件 1〜4）。
//! 4. `rws_core::raw_html()` は本クレートから呼ばない。
//! 5. `#![deny(unsafe_code)]` を採用する（`#[wasm_bindgen]` 展開コードが
//!    内部で `unsafe` を含むため `forbid` は不採用。自作コードでの新規
//!    `unsafe` 追加はビルド時に検出される）。
//! 6. `JsValue` エラー・`web_sys::console` ログは英語・固定文言とし、内部
//!    パス・状態値・属性値の内容を含めない。

#![deny(unsafe_code)]
#![warn(missing_docs)]

use rws_app::{
    assemble_detail_page, assemble_list_page, DemoItemDetailLoader, DemoItemsLoader, Item, Loader,
};
use rws_core::Node;

mod binding;
pub use binding::*;

#[cfg(target_arch = "wasm32")]
mod binding_dom;
#[cfg(target_arch = "wasm32")]
pub use binding_dom::BindingTable;

/// keyed list（`rws_core::keyed`、#344）の DOM 適用: 純粋 diff 層（イシュー
/// #345）。`binding`/`binding_dom` と同じ 2 層構成方針を踏襲する。
pub mod keyed_diff;

#[cfg(target_arch = "wasm32")]
mod keyed_dom;
#[cfg(target_arch = "wasm32")]
pub use keyed_dom::{apply_keyed_list, find_keyed_list_node, find_list_element};

#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen-exports"))]
mod registry;

/// ハイドレーション対象を示す属性名（`rws_app::detail_page` が「いいね」
/// ボタンに付与する `data-hydrate` 属性、`docs/api/app-api.md` 第 3 節・
/// `docs/api/hydration-api.md` 第 3.1 節の契約）。
///
/// 純粋ロジック層（[`find_hydrate_target_kinds`]）・wasm32 配線層
/// （[`wiring::hydrate`]）の双方が同じ属性名を参照することで、
/// 「どの属性を見て対象を判定するか」の契約を一箇所に固定する。
pub const HYDRATE_ATTR: &str = "data-hydrate";

/// [`HYDRATE_ATTR`] の値のうち、「いいね」ボタンに割り当てられる値
/// （`rws_app::LIKE_BUTTON_ID` と対になる契約）。
pub const LIKE_HYDRATE_VALUE: &str = "like";

/// loader 解決失敗時の fail-closed ビュー（`server/src/ssr.rs::loader_error_response`
/// と同型の構造的保証、イシュー #375）。`rws-wasm-full` は本関数を独自実装せず
/// `wasm-full/src/csr.rs` から再エクスポートして共有する（Bugbot 指摘対応、
/// 重複コピーの解消）。
///
/// **呼び出し元はこの関数へ `Loader::Error` の値を渡さない**（意図的に
/// シグネチャへ含めない）。`Display`/`Debug` を一切経由しないため、loader
/// 実装が `Error` に機微情報を含めていても出力へ混入する経路が型レベルで
/// 存在しない（`security.md`「機微情報の露出」）。本文はノード木 API
/// （[`rws_core`]）のみで組み立て、`format!` によるタグ文字列の直接組み立て
/// は行わない（REQ-1・不変条件 1）。英語固定文言とする
/// （`.claude/rules/japanese-style.md`「エラーメッセージ・ログ等のユーザー
/// 向け文字列は英語」）。
///
/// 現行の参照 loader（[`DemoItemsLoader`] / [`DemoItemDetailLoader`]、
/// `Error = Infallible`）では実行時に到達しないが、将来 loader を差し替えた
/// 場合でも機微情報が非露出であることを構造的に保証するために用意する。
pub fn loader_error_view() -> Node {
    rws_core::div(
        vec![("data-rws", "csr-error")],
        vec![rws_core::p(
            vec![],
            vec![rws_core::text("Something went wrong. Please try again.")],
        )],
    )
}

/// 一覧画面向け CSR loader 解決（イシュー #375）。`rws-wasm-full` は
/// `wasm-full/src/csr.rs` から本関数を再エクスポートして共有する
/// （Bugbot 指摘対応、重複コピーの解消）。
///
/// `assemble_list_page(loader, &())` の `Ok` はそのまま返し、`Err(_)` は
/// 値に一切触れず [`loader_error_view`] へ変換する（fail-closed、未解決
/// データで描画を続行しない）。`L::Output` が `Vec<Item>` でない loader を
/// 渡すとコンパイルエラーになる（`where` 束縛による型接続）。
pub fn resolve_list_node<L>(loader: &L) -> Node
where
    L: Loader<Input = (), Output = Vec<Item>>,
{
    match assemble_list_page(loader, &()) {
        Ok(node) => node,
        Err(_) => loader_error_view(),
    }
}

/// 詳細画面向け CSR loader 解決（イシュー #375）。`rws-wasm-full` は
/// `wasm-full/src/csr.rs` から本関数を再エクスポートして共有する
/// （Bugbot 指摘対応、重複コピーの解消）。
///
/// `assemble_detail_page(loader, id)` の `Ok` はそのまま返す。`Output`
/// （`Option<Item>`）が `None`（未知の id、404 相当）の場合は
/// `detail_page(None)` の既存契約どおり描画する（見つからない、を
/// `Error` として扱わない）。`Err(_)` のみ値に触れず [`loader_error_view`]
/// へ変換する。
pub fn resolve_detail_node<D>(loader: &D, id: &str) -> Node
where
    D: Loader<Input = String, Output = Option<Item>>,
{
    match assemble_detail_page(loader, &id.to_string()) {
        Ok(node) => node,
        Err(_) => loader_error_view(),
    }
}

/// CSR 用の一覧ページ HTML を生成する純粋関数（`mount_csr` の中核）。
///
/// [`DemoItemsLoader`]（参照 loader、`Error = Infallible`）を
/// [`resolve_list_node`] 経由で解決し `rws_core::render` を呼ぶ。
/// SSR/SSG（`rws-server`、TASK-6.1c）が同一の `rws_app::Loader` 契約
/// （`assemble_list_page` → `list_page`）を同一入力で解決した場合と文字列
/// 完全一致することを、CSR が SSR/SSG と同一関数を呼ぶという REQ-6 の
/// 受け入れ基準として保証する（`docs/api/hydration-api.md` 第 3.1 節・
/// `docs/design/loader-trait-design.md`）。
///
/// # Examples
///
/// ```
/// use rws_wasm_client::render_list_page_html;
/// use rws_app::{list_page, demo_items};
/// use rws_core::render;
///
/// assert_eq!(
///     render_list_page_html(),
///     render(&list_page(&demo_items()))
/// );
/// ```
pub fn render_list_page_html() -> String {
    rws_core::render(&resolve_list_node(&DemoItemsLoader))
}

/// CSR 用の詳細ページ HTML を生成する純粋関数。[`render_list_page_html`] と
/// 同様、[`DemoItemDetailLoader`] を [`resolve_detail_node`] 経由で解決し
/// `rws_core::render` を呼ぶ。該当 `id` が存在しない場合は
/// `rws_app::detail_page` の 404 相当ノードを返す（呼び出し元での
/// `panic!` を避ける、`.claude/rules/coding-rust.md`）。
///
/// # Examples
///
/// ```
/// use rws_wasm_client::render_detail_page_html;
///
/// let html = render_detail_page_html("1");
/// assert!(html.contains("Rust 製フロントエンド基盤の構想"));
/// ```
pub fn render_detail_page_html(id: &str) -> String {
    rws_core::render(&resolve_detail_node(&DemoItemDetailLoader, id))
}

/// 指定 `id` の詳細ページのノード木から、[`HYDRATE_ATTR`] を持つ要素の値を
/// [`rws_core::find_attr_values`] で列挙する。
///
/// DOM 非依存の純粋関数のため wasm ビルドを介さずネイティブテスト可能
/// （`wasm-client/tests/hydration_targets.rs`）。実 DOM 上でのハイドレーション
/// 配線（[`wiring::hydrate`]、wasm32 配線層）は、本関数と同じ属性名契約
/// （[`HYDRATE_ATTR`]）を使って `web_sys::Element::query_selector_all` で
/// 実要素を検索する。両者は同じ属性名定数を共有することで、対象特定ロジックの
/// 契約が単一箇所に保たれる。
///
/// # Examples
///
/// ```
/// use rws_wasm_client::{find_hydrate_target_kinds, LIKE_HYDRATE_VALUE};
///
/// assert_eq!(find_hydrate_target_kinds("1"), vec![LIKE_HYDRATE_VALUE.to_string()]);
/// assert!(find_hydrate_target_kinds("missing-id").is_empty());
/// ```
pub fn find_hydrate_target_kinds(id: &str) -> Vec<String> {
    let tree = resolve_detail_node(&DemoItemDetailLoader, id);
    rws_core::find_attr_values(&tree, HYDRATE_ATTR)
}

/// 一覧ページのノード木から `data-nav` 属性値を [`rws_core::find_nav_targets`]
/// で列挙する。クライアント側ルーティング配線（将来の TASK-7.2 系のスコープ、
/// 本クレートでは配線自体は実装しない）が対象を特定する際に使う契約の関数。
///
/// # Examples
///
/// ```
/// use rws_wasm_client::find_list_nav_targets;
///
/// let targets = find_list_nav_targets();
/// assert!(targets.contains(&"/items/1".to_string()));
/// ```
pub fn find_list_nav_targets() -> Vec<String> {
    let tree = resolve_list_node(&DemoItemsLoader);
    rws_core::find_nav_targets(&tree)
}

// ---------------------------------------------------------------------
// wasm32 配線層: 実 DOM（web-sys）を操作するのはこのモジュールに限定する。
// native の `cargo test -p rws-wasm-client` にはコンパイル対象外
// （wasm-full/src/events.rs の wiring モジュールと同じ切り分け方針）。
// ---------------------------------------------------------------------
// イシュー #345: `rws-wasm-full` が本クレートを rlib として依存し
// `BindingTable`/`keyed_diff`/`keyed_dom`（DOM 依存だが `#[wasm_bindgen]`
// ではない通常の Rust API）のみを消費する。`wiring::hydrate`/`mount_csr`
// は REQ-6（本クレート独自の最小ハイドレーション CSR デモ、#48）向けの
// `#[wasm_bindgen]` エクスポートであり、`wasm-full` 側にも同名の
// `#[wasm_bindgen] pub fn hydrate`/`mount`（`entry.rs`）が存在するため、
// 両クレートを 1 つの wasm バイナリへ静的リンクすると
// `__wbindgen_describe_hydrate`/`hydrate` のシンボルが重複しリンクエラーに
// なる（`wasm-bindgen` の "describe" シンボルはクレートで名前空間分離され
// ない）。`wasm-bindgen-exports` feature（既定 on）でこの 2 エクスポートを
// 切り離せるようにし、`wasm-full/Cargo.toml` は
// `default-features = false` で依存することでこの衝突を避ける
// （本クレートを単体で使う既存の利用者には影響しない、既定 on のまま）。
#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen-exports"))]
mod wiring {
    use super::{HYDRATE_ATTR, LIKE_HYDRATE_VALUE};
    use crate::registry;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use web_sys::{Document, Element, Event};

    /// `window().document()` を解決する。いずれかが取得できない環境
    /// （テストランナーの合成 DOM 以外の非ブラウザ実行等）では `Err` を返す。
    /// エラー文字列は固定の英語文言とし、内部状態を含めない（不変条件 6）。
    fn document() -> Result<Document, JsValue> {
        web_sys::window()
            .ok_or_else(|| JsValue::from_str("window is unavailable"))?
            .document()
            .ok_or_else(|| JsValue::from_str("document is unavailable"))
    }

    /// 指定 `root_id` のルート要素を取得する。存在しない場合は `Err`
    /// （呼び出し元へ内部情報を含まない固定文言で伝える、不変条件 6）。
    fn get_root(root_id: &str) -> Result<Element, JsValue> {
        document()?
            .get_element_by_id(root_id)
            .ok_or_else(|| JsValue::from_str("root element not found"))
    }

    /// REQ-6 受け入れ基準の中核 API。`root_id` 配下の**サーバー出力済み DOM
    /// を再構築せず**（`set_inner_html` 等を一切呼ばない、不変条件 2）、
    /// 既存要素へイベントリスナーを後付けする。
    ///
    /// ハイドレーション対象は [`HYDRATE_ATTR`] を持つ子孫要素を
    /// `query_selector_all` で検索して特定する（純粋ロジック層
    /// [`find_hydrate_target_kinds`] と同一の属性名契約を共有）。値が
    /// [`LIKE_HYDRATE_VALUE`] の要素にのみ `click` リスナーを登録し、
    /// ハンドラ内では `class_list`（`DomTokenList::toggle`）のみを操作する
    /// （`set_text_content`/`class_list` に限定、不変条件 3）。
    ///
    /// クロージャは [`registry::replace_handles`] が root_id 単位で保持する
    /// （`closure.forget()` は使わない、`docs/api/hydration-api.md` 判断 4）。
    ///
    /// # Errors
    ///
    /// `root_id` に対応する要素が存在しない場合、または `query_selector_all`
    /// / イベントリスナー登録が失敗した場合に `Err` を返す。
    #[wasm_bindgen]
    pub fn hydrate(root_id: &str) -> Result<(), JsValue> {
        let root = get_root(root_id)?;

        let selector = format!("[{HYDRATE_ATTR}]");
        let targets = root
            .query_selector_all(&selector)
            .map_err(|_| JsValue::from_str("query_selector_all failed"))?;

        let mut handles = Vec::new();
        for i in 0..targets.length() {
            let Some(node) = targets.get(i) else {
                continue;
            };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            let kind = element.get_attribute(HYDRATE_ATTR).unwrap_or_default();
            if kind != LIKE_HYDRATE_VALUE {
                // v1 最小スコープでは「いいね」ボタン以外の data-hydrate 値は
                // 未対応（`docs/api/hydration-api.md` 第 3.1 節）。
                continue;
            }

            let target_for_closure = element.clone();
            let closure = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
                // ハンドラ内 DOM 更新は class_list（DomTokenList）に限定する
                // （set_inner_html 等の再構築系 API を呼ばない不変条件 2・3）。
                let _ = target_for_closure.class_list().toggle("liked");
            });
            match element
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            {
                Ok(()) => {
                    handles.push(registry::Handle::new(element, "click", closure));
                }
                Err(_) => {
                    // 部分失敗時の孤立防止: この呼び出し内で既に登録済みの
                    // DOM リスナーをここで解除してから Err を返す
                    // （ローカル handles の Drop だけに任せると、Closure は
                    // 破棄されるのに DOM 側リスナーだけ残ってしまう）。
                    // 既存レジストリ（前回までの hydrate() 分）には触れない。
                    registry::rollback_partial_handles(handles);
                    return Err(JsValue::from_str("add_event_listener_with_callback failed"));
                }
            }
        }

        registry::replace_handles(root_id, handles);
        Ok(())
    }

    /// CSR エントリポイント。[`super::render_list_page_html`]（純粋ロジック層、
    /// SSR/SSG と同一関数の呼び出し結果）を `root_id` 要素へ `set_inner_html`
    /// で反映する。REQ-6 の「CSR が SSR/SSG と同一関数を呼び `innerHTML` へ
    /// 反映すること」という受け入れ基準に対応する CSR 経路そのものであり、
    /// `hydrate()`（不変条件 2）とは異なり本関数は意図的に DOM を構築する
    /// （初回マウント時のみ呼ばれる想定で、`hydrate()` の対象にはならない）。
    ///
    /// # Errors
    ///
    /// `root_id` に対応する要素が存在しない場合に `Err` を返す。
    #[wasm_bindgen]
    pub fn mount_csr(root_id: &str) -> Result<(), JsValue> {
        let root = get_root(root_id)?;
        root.set_inner_html(&super::render_list_page_html());
        Ok(())
    }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen-exports"))]
pub use wiring::{hydrate, mount_csr};
