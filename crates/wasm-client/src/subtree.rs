//! view 外パラメータ付き部分描画（サブツリー再マウント）のための safe ヘルパ
//! （イシュー #1121）。
//!
//! `fandhe_frontend_interactive::Component::view` は状態からの純関数という契約
//! （`docs/api/interactive-api.md` 第 2 節）だが、PII（個人情報）を状態機械へ
//! 持ち込まず DOM のみに置く構成（イシュー #1121 報告者のユースケース）では、
//! `view()` の外側にしか存在しない値（DOM の現在値・ブラウザ API の戻り値等）
//! を使ってサブツリーだけを再構築したい場面が生じる。本モジュールは
//! 「通常の Rust 関数で [`fandhe_frontend_core::Node`] を組み立て →
//! [`replace_subtree`] で差し替え」という最小限の公式パターンを提供する。
//!
//! `wasm-client` の既存不変条件（`lib.rs` クレート冒頭コメント）を継承する:
//! DOM への挿入は [`crate::build_dom_node`]（`fandhe_frontend_core::render` と
//! 同じノード木 API 経由）のみを通し、`set_inner_html`/`insert_adjacent_html`
//! は呼ばない。`Node::RawHtml` を含む部分木は [`crate::build_dom_node`] が
//! `None` を返す（fail-closed）ため、本関数も DOM を一切変更せず `Err` で
//! 返す。

use crate::build_dom_node;
use fandhe_frontend_core::Node;
use wasm_bindgen::JsValue;
use web_sys::Element;

/// `slot`（差し替え対象の既存要素）を `node` から構築した新しい DOM ノードで
/// 置き換える。
///
/// # 呼び出し文脈
///
/// view 外パラメータ（DOM の現在値・イベントの戻り値等、`Component::view`
/// が受け取れない値）を使ってサブツリーだけを再マウントしたい呼び出し元
/// （利用者コードの Closure・`fandhe-frontend-wasm-full` の遷移後再配線と同様の
/// 立ち位置）から呼ばれることを想定する。
///
/// # 再配線責務
///
/// 置換によって `slot` 配下の旧サブツリーとそこに付いていたイベント
/// リスナーは DOM から失われる（`Element::replace_child` の標準挙動）。
/// 新しいサブツリーへイベントリスナーを付け直す必要がある場合は、呼び出し
/// 元が [`crate::wire_hydrate_targets`]（`data-hydrate` 属性ベース）や
/// [`crate::registry`] と同型の Closure 管理を用いて再配線すること
/// （本関数はハイドレーション配線自体は行わない、置換のみに責務を限定
/// する）。
///
/// # Errors
///
/// - `node` が `Node::RawHtml` を含む、または子要素の `create_element` が
///   失敗する等で [`crate::build_dom_node`] が `None` を返した場合、DOM を
///   一切変更せず `Err` を返す（fail-closed。`RawHtml` 混入時に部分的な
///   DOM 破壊を起こさない）。
/// - `slot` に親ノードが存在しない（DOM に未接続）場合も `Err` を返す。
/// - `owner_document()` が取得できない場合（`slot` が detached）も `Err`
///   を返す。
///
/// いずれのエラー経路でも `slot` および周辺 DOM は変更されない。
///
/// # Returns
///
/// 成功時は置換後に DOM へ挿入された新しいノードを返す（呼び出し元が
/// 続けて再配線・スクロール位置調整等を行うためのハンドル）。
pub fn replace_subtree(slot: &Element, node: &Node) -> Result<web_sys::Node, JsValue> {
    let document = slot
        .owner_document()
        .ok_or_else(|| JsValue::from_str("slot has no owner document"))?;

    // `build_dom_node` は RawHtml 混入・要素生成失敗を fail-closed に None
    // で示す。ここで DOM へは一切触れていないため、この時点で Err 化して
    // 返しても slot 側の状態は無傷のまま。
    let new_node = build_dom_node(&document, node)
        .ok_or_else(|| JsValue::from_str("failed to build replacement subtree"))?;

    let parent = slot
        .parent_node()
        .ok_or_else(|| JsValue::from_str("slot is not attached to a parent node"))?;

    parent
        .replace_child(&new_node, slot)
        .map_err(|_| JsValue::from_str("replace_child failed"))?;

    Ok(new_node)
}
