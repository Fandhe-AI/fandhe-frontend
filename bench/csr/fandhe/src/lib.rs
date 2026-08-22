//! フレームワーク横断 CSR ベンチマーク（`bench/PROTOCOL.md` §2.2）向けの
//! fandhe-frontend wasm アプリ。
//!
//! playwright ハーネス（`bench/csr/run_csr.mjs`）が `bench/csr/dist/fandhe/`
//! を配信し、`window.__bench.{create,update,clear}` を呼び出して perf 計測
//! する。本クレートはその 3 関数を `#[wasm_bindgen]` でエクスポートし、
//! `bench/csr/fandhe/bootstrap.js`（起動コード。`build.sh` が minify して
//! dist へ配置する）が `window.__bench` へ束縛する（束縛自体は JS グルー側
//! の責務、本クレートは Rust 関数を提供するのみ）。
//!
//! # ワークロード定義
//!
//! `bench/PROTOCOL.md` §2.2 を正とする。
//!
//! - create: 1,000 行を生成する。行 i（0 始まり）= `{id: i, label: "Row {i}
//!   & \"quoted\" 'single' <script>alert(1)</script>"}`
//! - update: `i % 10 == 0` の行の label 末尾へ ` !!!` を追記して再描画する
//!   （100 行更新）
//! - clear: 全行削除する
//!
//! label は既定エスケープ対象の 5 文字（`& < > " '`）を含み、
//! [`fandhe_frontend_core::text`]（テキストノード、`render()`/DOM 適用時に
//! 必ず既定エスケープを経由する）でのみ出力する。`raw_html()` は使わない
//! （`.claude/rules/coding-rust.md` REQ-1）。
//!
//! # DOM 適用経路の選定理由
//!
//! `fandhe-frontend-wasm-full` の `Runtime<C>`（`crates/wasm-full/src/lib.rs`）は
//! DOM の click/input イベント（`events::wire_events` が委譲登録するリスナー）
//! を契機にした `Component::update` → 再描画という一方向フローのみを公開して
//! おり、DOM イベントを介さず外部（JS）から直接 `update()` を起動する公開 API
//! を持たない。本ベンチは `window.__bench.create()` 等 JS 側からの直接呼び
//! 出しで `performance.now()` 境界を取る必要があるため、ボタンクリックの
//! 合成イベントで `Runtime` を駆動する遠回りをせず、`Runtime` が dirty な
//! keyed list field に対して内部で使っているのと**同じ** DOM 適用プリミティブ
//! （[`fandhe_frontend_wasm_client::apply_keyed_list`] /
//! [`fandhe_frontend_wasm_client::apply_keyed_list_with_previous`]。
//! `crates/wasm-client/src/keyed_dom.rs` の doc コメントで
//! 「[`apply_keyed_list_with_previous`] は通常運用の主経路」と明記されている）
//! を直接呼ぶ。`Runtime::apply_update_for_dirty`/`commit_keyed_list_result`
//! （`crates/wasm-full/src/lib.rs`）と同じ二経路構成
//! （直前の適用結果キャッシュが無ければ [`apply_keyed_list`]（DOM 読み出し
//! ベースのフォールバック）、あれば [`apply_keyed_list_with_previous`]
//! （内容比較付き差分適用）、`ResyncRequired` はキャッシュを破棄して次回
//! フォールバック経路へ自己修復）を単一 keyed list field（`rows`）分だけ
//! 縮小して踏襲するため、計測されるコストは `Runtime` 経由の通常運用時と
//! 同一の DOM 適用アルゴリズムである。
//!
//! [`apply_keyed_list`]: fandhe_frontend_wasm_client::apply_keyed_list
//! [`apply_keyed_list_with_previous`]: fandhe_frontend_wasm_client::apply_keyed_list_with_previous
//!
//! 本クレートの自作コードは safe Rust のみとし、`unsafe` は `wasm-bindgen` /
//! `web-sys` の FFI 境界（依存クレート内部・自動生成コード）に限定する
//! （`docs/policy/unsafe-boundary.md` 第 2 節）。自作コードでの新規 `unsafe`
//! 追加をビルド時に検出するため `#![deny(unsafe_code)]` を採用する
//! （`#[wasm_bindgen]` 展開コードが内部で `unsafe` を含むため `forbid` は
//! 不採用。`wasm-full`/`wasm-client` と同方針）。

#![deny(unsafe_code)]

use fandhe_frontend_core::{keyed::keyed_list, td, text, tr, Node};
use fandhe_frontend_wasm_client::{
    apply_keyed_list, apply_keyed_list_with_previous, KeyedListApplyResult,
};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

/// ベンチ対象の tbody 要素の DOM id（`index.html` と対応、build.sh は
/// この値を書き換えない前提の固定契約）。
const TBODY_ID: &str = "bench-body";

/// ベンチ状態: 現在の論理行データと、直前に DOM へ反映済みの keyed list
/// ノード（[`apply_keyed_list_with_previous`] の内容比較差分の基準）。
///
/// WASM はシングルスレッド実行のため `thread_local!` + `RefCell` で保持する
/// （`crates/wasm-full/src/entry.rs` の `RUNTIME` と同じ設計方針）。
struct BenchState {
    /// (id, label) の順序付きリスト。`id` の文字列表現を keyed list の
    /// キーとして使う（`bench/PROTOCOL.md` §2.2「キー付きリスト描画を持つ
    /// フレームワークは id をキーに使う」）。
    rows: Vec<(u32, String)>,
    /// 直前に DOM へ反映済みの tbody ノード（[`Runtime::keyed_list_cache`]
    /// 相当、単一 field 分）。`None` は「まだ 1 度も適用していない」
    /// （初期状態の空 tbody と一致する）ことを表す。
    previous: Option<Node>,
}

thread_local! {
    static STATE: std::cell::RefCell<BenchState> = const {
        std::cell::RefCell::new(BenchState {
            rows: Vec::new(),
            previous: None,
        })
    };
}

/// `bench/PROTOCOL.md` §2.2 のラベル生成規則（SSR ベンチ・xtask
/// `bench_ssr` と同一文言）。既定エスケープ対象の 5 文字
/// （`& < > " '`）と `<script>` タグを意図的に含み、既定エスケープ経路
/// （テキストノード）でのみ出力されることを検証可能にする。
fn build_label(id: u32) -> String {
    format!("Row {id} & \"quoted\" 'single' <script>alert(1)</script>")
}

/// `rows` から tbody 全体の keyed list ノードを構築する。
///
/// 各行は `tr` = `td(id)` + `td(label)`。`label` は必ず [`text`]（既定
/// エスケープ経由のテキストノード）で包み、HTML 文字列の直接組み立ては
/// 行わない（`.claude/rules/coding-rust.md`）。
///
/// # Panics
///
/// `rows` のキー（`id` の文字列表現）が空文字列・重複することはなく
/// （`id` は `u32` の一意な連番）、各アイテムは必ず `Node::Element`
/// （[`tr`]）であるため、[`keyed_list`] は本関数の呼び出し文脈では常に
/// `Ok` を返す。`.expect` はこの不変条件の表明であり、キー生成ロジックを
/// 変更する場合はこの前提を保つこと。
fn build_tbody_node(rows: &[(u32, String)]) -> Node {
    let items: Vec<(String, Node)> = rows
        .iter()
        .map(|(id, label)| {
            let key = id.to_string();
            let row = tr(
                vec![],
                vec![
                    td(vec![], vec![text(id.to_string())]),
                    td(vec![], vec![text(label.clone())]),
                ],
            );
            (key, row)
        })
        .collect();
    // id 属性はノード木側にも持たせる: apply_keyed_list はリスト要素を
    // ノード木由来の新要素で置換するため、ここに id が無いと初回適用後に
    // `#bench-body` の解決が失敗する（実挙動で確認済み）。
    keyed_list("tbody", vec![("id", TBODY_ID)], "rows", items)
        .expect("bench の keyed list キーは id 由来の非空一意文字列であり常に Ok")
}

/// `TBODY_ID` の要素を解決する。要素不在は環境エラーとして `Err` を返す
/// （panic しない、`.claude/rules/coding-rust.md`）。
fn tbody_element() -> Result<web_sys::Element, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window is unavailable"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("document is unavailable"))?;
    document
        .get_element_by_id(TBODY_ID)
        .ok_or_else(|| JsValue::from_str("#bench-body element not found"))
}

/// `new_node` を tbody へ適用し、`STATE.previous` を達成済み状態へ更新する。
///
/// [`Runtime::commit_keyed_list_result`]（`crates/wasm-full/src/lib.rs`）と
/// 同じ契約: `Achieved` のみをキャッシュへ確定させ、`ResyncRequired` は
/// キャッシュを破棄して次回 [`apply_keyed_list`]（cache-miss フォール
/// バック、ライブ DOM 読み出し基準で自己修復する）へ委ねる。未達成状態を
/// キャッシュしないことで、diff 基準がライブ DOM の実際の内容と乖離した
/// まま固定化されるのを防ぐ。
fn apply_and_commit(new_node: Node) -> Result<(), JsValue> {
    let element = tbody_element()?;
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window is unavailable"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("document is unavailable"))?;

    STATE.with(|cell| {
        let previous = cell.borrow().previous.clone();
        let result = match previous {
            Some(previous_node) => {
                apply_keyed_list_with_previous(&document, &element, &previous_node, &new_node)
            }
            None => apply_keyed_list(&document, &element, &new_node),
        };
        match result {
            KeyedListApplyResult::Achieved { node, .. } => {
                cell.borrow_mut().previous = Some(node);
            }
            KeyedListApplyResult::ResyncRequired { .. } => {
                cell.borrow_mut().previous = None;
            }
        }
    });
    Ok(())
}

/// create: 1,000 行を新規生成して描画する（`bench/PROTOCOL.md` §2.2）。
///
/// harness 側は「create は毎回 clear 後」に呼ぶ運用（同ファイル §2.2）だが、
/// 本関数自体は現在の `rows` を無条件で 1,000 行へ置き換える（既存行が
/// 残っていても安全に上書きする、防御的実装）。
///
/// # Errors
///
/// `#bench-body` 要素が見つからない等、DOM 解決に失敗した場合に `Err` を
/// 返す。
#[wasm_bindgen]
pub fn bench_create() -> Result<(), JsValue> {
    let rows: Vec<(u32, String)> = (0..1000u32).map(|id| (id, build_label(id))).collect();
    let new_node = build_tbody_node(&rows);
    STATE.with(|cell| cell.borrow_mut().rows = rows);
    apply_and_commit(new_node)
}

/// update: `i % 10 == 0` の行の label 末尾へ ` !!!` を追記して再描画する
/// （`bench/PROTOCOL.md` §2.2、100 行更新）。
///
/// # Errors
///
/// [`bench_create`] と同じ DOM 解決失敗条件で `Err` を返す。
#[wasm_bindgen]
pub fn bench_update() -> Result<(), JsValue> {
    let rows = STATE.with(|cell| {
        let mut rows = cell.borrow().rows.clone();
        for (id, label) in rows.iter_mut() {
            if *id % 10 == 0 {
                label.push_str(" !!!");
            }
        }
        cell.borrow_mut().rows = rows.clone();
        rows
    });
    let new_node = build_tbody_node(&rows);
    apply_and_commit(new_node)
}

/// clear: 全行削除する（`bench/PROTOCOL.md` §2.2）。
///
/// # Errors
///
/// [`bench_create`] と同じ DOM 解決失敗条件で `Err` を返す。
#[wasm_bindgen]
pub fn bench_clear() -> Result<(), JsValue> {
    STATE.with(|cell| cell.borrow_mut().rows = Vec::new());
    let new_node = build_tbody_node(&[]);
    apply_and_commit(new_node)
}
