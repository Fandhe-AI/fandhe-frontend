//! [`KeyedListApplyResult`] の DOM 非依存な定義（イシュー #1381）。
//!
//! [`crate::keyed_dom::apply_keyed_list`]/
//! [`crate::keyed_dom::apply_keyed_list_with_previous`]（wasm32 配線層、
//! `web-sys` 呼び出しを伴う実 DOM 適用）の戻り値型だが、型自体のフィールド
//! （`fandhe_frontend_core::Node`・`HashSet<String>`・`bool`）は
//! `web-sys` に一切依存しないため、`keyed_dom` モジュール本体
//! （`#[cfg(target_arch = "wasm32")]`）とは独立にゲートせず常時コンパイル
//! する。これにより、`fandhe-frontend-wasm-full` 側の判定ロジック
//! （モジュールトップレベルの自由関数
//! `commit_keyed_list_result_with_resync`。`Runtime<C>` から独立させて
//! あり、DOM 操作はクロージャ注入で外側へ追い出した純粋な分岐処理）が
//! native `cargo test` から本型を直接構築してテストできる
//! （`keyed_apply`/`keyed_diff` と同じ「DOM 非依存の純粋ロジック層 +
//! `#[cfg(target_arch = "wasm32")]` 配線層」の 2 層構成方針）。

use fandhe_frontend_core::Node;

/// [`crate::keyed_dom::apply_keyed_list_with_previous`] の適用結果（イシュー #1324）。
///
/// 呼び出し元（`fandhe-frontend-wasm-full` の `Runtime`）が「直前に DOM へ
/// 反映した内容」のキャッシュを次回呼び出しの `previous_list_node` として
/// 使い続けるための状態遷移を表す（設計書 §4.2/§4.2a）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyedListApplyResult {
    /// ライブ DOM が実際に表している「達成 Node」（設計書 §4.2）。
    Achieved {
        /// `Update` が全件成功していれば `new_list_node` そのものと等価
        /// だが、子ノード構築に失敗して据え置かれたアイテムがあれば当該
        /// アイテムのみ旧内容のまま含む。呼び出し元はこの `Node` を次回
        /// 呼び出しの `previous_list_node` として保持し続けることで、
        /// 以降の diff 基準を実際の DOM 内容と一致させ続ける（キャッシュ
        /// の再同期）。
        node: Node,
        /// 独立敵対レビュー指摘 A（イシュー #1340）対応: 本 field を丸ごと
        /// 新規構築した部分木（`Insert`・タグ変更を伴う `Update`・親タグ
        /// 変更・内容変更の `Update`）の子孫に現れた**別の** keyed list
        /// field 名の集合（`crate::keyed_apply::ApplyOutcome::
        /// invalidated_nested_fields` doc 参照）。
        ///
        /// これらの field はライブ DOM 上では既に新しい状態になっている
        /// が、`fandhe-frontend-wasm-full` の `Runtime::keyed_list_cache`
        /// は field ごとに独立したエントリのため、この副作用を知らない
        /// まま古い内容を指し続ける（ネストした keyed list の field 間
        /// キャッシュ無効化欠落）。呼び出し元はこの集合に含まれる field
        /// を `keyed_list_cache` から remove し、次回はライブ DOM 読み
        /// 出し基準の cache-miss フォールバックへ委ねて自己修復させる
        /// こと。
        invalidated_nested_fields: std::collections::HashSet<String>,
    },
    /// 「要再同期」（設計書 §4.2a）。
    ///
    /// `Update` の子ノード構築失敗（`Node::RawHtml` 混入等）は当該アイテムが
    /// 旧内容のまま DOM 上に残り続けるだけなので `stale_update_keys` 経由で
    /// 「達成 Node」へ正しく表現でき、本 variant の対象にはならない
    /// （`Achieved` が返る）。一方、`Insert` の構築失敗・`Move`/`Update` の
    /// 対象キーがライブ DOM 上に見つからない等「op が計画どおりに適用され
    /// なかった」ケース（[`crate::keyed_apply::ApplyOutcome::resync_required`]
    /// doc 参照、イシュー #1340 codex-review P1 対応）では本 variant が返る:
    /// `diff_keyed_items` が計画した `index` は「全 op が成功した前提の
    /// 最終並び」上の位置であり、一部が未達成のまま「達成 Node」を確定させ
    /// キャッシュしてしまうと、次回呼び出しの diff 基準がライブ DOM の実際
    /// の内容と乖離したまま固定され、以降いくら同じ view を再適用しても
    /// 乖離が解消されない（本 variant 導入前の実際の不具合、PR #1340
    /// codex-review 指摘）。呼び出し元はこの `field` の保持 Node を破棄し、
    /// 次回は [`crate::keyed_dom::apply_keyed_list`] のフォールバック経路
    /// （ライブ DOM を直接読み出す構造変化のみの適用、`Update` を発行しない
    /// ため diff 基準が常に実際の DOM と一致する）へ委ねること。
    ResyncRequired {
        /// 独立敵対レビュー指摘（イシュー #1340、最終確認レビュー指摘 1）
        /// 対応: `resync_required` が立つ**前**に成功した op（例:
        /// 保持キーの一部が丸ごと新規構築された `Update` 直後に、別の
        /// `Update` の対象キーがライブ DOM 上に見つからず本 variant が
        /// 返る場合）は、既にライブ DOM を変更済みであり、その部分木に
        /// 含まれるネストした別 field のキャッシュも同様に無効化する
        /// 必要がある（`Achieved::invalidated_nested_fields` doc・
        /// `crate::keyed_apply::ApplyOutcome::invalidated_nested_fields`
        /// doc 参照）。呼び出し元（`Runtime::commit_keyed_list_result`）は
        /// 自 field のキャッシュ remove に加えてこの集合も remove する
        /// こと。DOM に一切触れていない・ロールバックで DOM 未変更相当に
        /// 戻した早期 `ResyncRequired`（契約検証失敗・ detached 親タグ
        /// 変更・`replace_root_node`/`insert_before_batch`/`move_before`/
        /// `remove_child` の実 DOM 操作失敗）は空集合を返す。
        invalidated_nested_fields: std::collections::HashSet<String>,
        /// この適用試行でライブ DOM への書き込み操作を**最初に試行した
        /// 時点**で `true` になる実測フラグ（成功不問。イシュー #1381
        /// 設計 §6.1「`dom_mutated` 判定」、
        /// `crate::keyed_apply::ApplyOutcome::dom_mutated` doc 参照）。
        /// 呼び出し元（`fandhe-frontend-wasm-full` の
        /// `commit_keyed_list_result`）は、この `ResyncRequired` を受けた
        /// 同一更新サイクル内で即時再同期
        /// （[`crate::keyed_dom::apply_keyed_list`]）を実行し、
        /// それも失敗した場合、最初の適用試行の本フィールドと即時再同期
        /// 試行自身の `dom_mutated`（同様に実測）の論理和が `true` なら
        /// [`crate::keyed_apply::KeyedListDom::clear_children`] による
        /// 一括クリアでリストを「空」という確定状態へ倒す（OR が `false`
        /// ならクリアせず旧 view を温存する）。DOM に一切触れていない・
        /// ロールバックで DOM 未変更相当に戻した早期 `ResyncRequired`は
        /// `false` を返す。
        dom_mutated: bool,
    },
}
