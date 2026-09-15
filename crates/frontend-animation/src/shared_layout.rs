//! 共有レイアウト遷移（motion.dev `layoutId` 相当）の突合・計測・再生
//! （イシュー #2536）。
//!
//! [`crate::flip`]（イシュー #2518）は**同一要素**の layout 変化
//! （keyed list の `Move` による並べ替え等）のみを対象とし、「別要素が
//! 同じ役割を引き継ぐ」ケース（旧要素がアンマウントされ、新要素が
//! マウントされる。tabs の下線インジケータ移動・カード→詳細展開の基盤）
//! を明示的にスコープ外としている（`flip.rs` クレート doc 冒頭参照）。
//! 本モジュールはその差分を埋める: `data-fandhe-layout-id` の値で旧要素・
//! 新要素を対応付け、旧要素のアンマウント前の視覚矩形（First）から
//! 新要素の現在位置・サイズ（Last）へ FLIP 補正する。
//!
//! # 責務境界
//!
//! 本モジュールは [`crate::flip`] が提供する計測（[`crate::flip::measure`]/
//! [`crate::flip::measure_layout_batch`]）・Invert（[`crate::flip::invert`]/
//! [`crate::flip::anchor_correct`]）・Play（[`crate::flip::play`]）を
//! **そのまま再利用**し、新規の幾何計算は一切持たない。本モジュールが
//! 独自に持つのは「どの旧要素とどの新要素を同一の共有レイアウトとして
//! 対応付けるか」（[`pair_by_id`]、DOM 非依存の純粋関数）の突合ルール
//! のみである。`data-fandhe-layout-id` 属性から本モジュールを呼ぶ
//! タイミング判定・View Transitions への委譲判定は `fandhe-frontend-
//! wasm-full` 側（`crate::shared_layout`）の責務であり、本モジュールは
//! 持たない（[`crate::flip`] クレート doc「責務境界」節と同じ設計）。
//!
//! # 突合ルール（[`pair_by_id`]）
//!
//! after（新要素）側の各 id について、before（旧要素）側の**同 id 先頭
//! エントリ**と組にする。以下のいずれかに該当する場合は対象外とする
//! （fail-safe: 曖昧な状況では何もしない側へ倒す）。
//!
//! - **同一ノード**（`same_node` が `true`）: [`crate::flip`] の対象領域
//!   （同一要素の layout 変化）と重複するため、二重補正を避ける
//! - **旧要素がまだ DOM に接続中**（`before_connected` が `true`）:
//!   旧要素・新要素が同時に存在する状態は「引き継ぎ」ではなく通常の
//!   複数要素表示であり曖昧なため対象外とする
//! - **after 側で id が重複**: 2 件目以降は無視する（先頭 1 件のみ
//!   再生。Motion の lead 要素切替の最小近似）
//!
//! # 捕捉順の契約
//!
//! [`crate::flip::measure_layout_batch`] による Last **layout** 矩形計測は
//! [`crate::flip::OriginalStyle::capture`] + Last **視覚**矩形の計測より
//! **先に**行う（[`crate::flip::OriginalStyle::capture`] doc「呼び出し順」
//! 節と同じ理由: `measure_layout_batch` は要素自身の `transform` に進行中
//! の CSS transition があれば目標値へ確定〔settle〕させる副作用を持つ
//! ため、それより後に `capture`/Last 視覚矩形を取得しないと、Play が
//! 収束させる先と実際に確定する目標値が食い違う）。`crate::wiring::
//! play_shared`（wasm32 限定層）がこの順序で呼ぶ。
//!
//! # VT 委譲判定は本モジュールの責務外
//!
//! View Transitions が使える場合に本モジュールを起動しないという判定
//! （`document.startViewTransition` の機能検出）は `fandhe-frontend-
//! wasm-full` 側の責務であり、本モジュールは常に呼ばれれば計測・再生を
//! 行う（判定を持たない）。

/// after（新要素）の id 一覧を before（旧要素）の id 一覧と突き合わせ、
/// 共有レイアウト遷移として再生すべき `(after_idx, before_idx)` の組を
/// 返す（DOM 非依存の純粋関数、native `cargo test` で検証できる）。
///
/// - `before_ids`/`before_connected`/`after_ids` は同じ添字で対応する
///   スライス（`before_connected[i]` は `before_ids[i]` に対応する旧要素が
///   まだ DOM に接続中かどうか）
/// - `same_node(after_idx, before_idx)` は当該の組が実際には同一 DOM
///   ノードかどうかを判定するコールバック（wasm32 層が `Node::
///   is_same_node` で実装する。native テストでは任意の判定関数を渡せる）
///
/// 戻り値は after の出現順を保つ。突合ルールの詳細はモジュール doc
/// 「突合ルール」節を参照。
#[must_use]
pub fn pair_by_id(
    before_ids: &[&str],
    before_connected: &[bool],
    after_ids: &[&str],
    same_node: impl Fn(usize, usize) -> bool,
) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for (after_idx, after_id) in after_ids.iter().enumerate() {
        // after 側の重複 id は先頭 1 件のみ再生する（モジュール doc参照）。
        if !seen.insert(after_id) {
            continue;
        }
        let Some(before_idx) = before_ids.iter().position(|id| id == after_id) else {
            continue;
        };
        if before_connected.get(before_idx).copied().unwrap_or(false) {
            // 旧要素がまだ DOM に接続中: 同時表示は曖昧なため対象外。
            continue;
        }
        if same_node(after_idx, before_idx) {
            // 同一ノード: `crate::flip` の対象領域と重複するため対象外。
            continue;
        }
        result.push((after_idx, before_idx));
    }
    result
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::pair_by_id;
    use crate::flip;
    use fandhe_animation::spring::SpringConfig;
    use web_sys::HtmlElement;

    /// [`snapshot`] が捕捉した、共有レイアウト遷移の対象になり得る旧要素
    /// の一覧（id・要素・捕捉時点の視覚矩形）。
    pub struct SharedSnapshot {
        entries: Vec<(String, HtmlElement, flip::Rect)>,
    }

    /// `entries`（`data-fandhe-layout-id` の値と対応する要素の組）の現在の
    /// 視覚矩形（[`flip::measure`]、進行中の FLIP 補正込み）を一括で測り、
    /// [`SharedSnapshot`] として保持する。
    ///
    /// 呼び出し元（`fandhe-frontend-wasm-full` 側）が要素をアンマウント
    /// する**前**に呼ぶ想定（`crate::flip::layout_flip::capture_before`
    /// doc と同じ「旧アニメーションの停止より前に測る」制約は、本関数は
    /// 旧アニメーションの停止自体を行わないため課さない——停止・捕捉の
    /// 責務分離は呼び出し元〔wasm-full 側〕が担う）。
    #[must_use]
    pub fn snapshot(entries: impl IntoIterator<Item = (String, HtmlElement)>) -> SharedSnapshot {
        let entries = entries
            .into_iter()
            .map(|(id, element)| {
                let rect = flip::measure(&element);
                (id, element, rect)
            })
            .collect();
        SharedSnapshot { entries }
    }

    /// [`snapshot`] の内容と `after`（新要素の id・要素の組）を
    /// [`pair_by_id`] で突き合わせ、対応する各組に対して FLIP を再生する。
    ///
    /// 戻り値は実際に再生を開始した `(id, FlipAnimation)` の一覧
    /// （呼び出し元がハンドルを保持し続けることで再生を継続させる。
    /// `crate::flip::layout_flip`〔wasm-full 側〕の `FLIP_LOOPS` と同じ
    /// 契約）。再生を省略した組（変化なし・計測不能）は含まれない。
    ///
    /// # 捕捉順（モジュール doc「捕捉順の契約」節）
    ///
    /// 対象となる after 要素をまず [`flip::measure_layout_batch`] で
    /// 一括計測（settle させる副作用込み）してから、[`flip::
    /// OriginalStyle::capture`] と Last 視覚矩形（[`flip::measure`]）を
    /// 取得する。
    #[must_use]
    pub fn play_shared(
        snapshot: &SharedSnapshot,
        after: &[(String, HtmlElement)],
        config: SpringConfig,
    ) -> Vec<(String, flip::FlipAnimation)> {
        let before_ids: Vec<&str> = snapshot
            .entries
            .iter()
            .map(|(id, _, _)| id.as_str())
            .collect();
        let before_connected: Vec<bool> = snapshot
            .entries
            .iter()
            .map(|(_, element, _)| element.is_connected())
            .collect();
        let after_ids: Vec<&str> = after.iter().map(|(id, _)| id.as_str()).collect();
        let same_node = |after_idx: usize, before_idx: usize| {
            let (_, before_element, _) = &snapshot.entries[before_idx];
            let (_, after_element) = &after[after_idx];
            before_element.is_same_node(Some(after_element))
        };
        let pairs = pair_by_id(&before_ids, &before_connected, &after_ids, same_node);
        if pairs.is_empty() {
            return Vec::new();
        }

        // Last layout 矩形の一括計測（settle させる副作用込み、モジュール
        // doc「捕捉順の契約」節）。
        let target_elements: Vec<HtmlElement> = pairs
            .iter()
            .map(|&(after_idx, _)| after[after_idx].1.clone())
            .collect();
        let last_layout_rects = flip::measure_layout_batch(&target_elements);

        let mut result = Vec::new();
        for (&(after_idx, before_idx), last_layout) in pairs.iter().zip(last_layout_rects) {
            let (id, element) = &after[after_idx];
            let first = snapshot.entries[before_idx].2;
            let original = flip::OriginalStyle::capture(element);
            let last_visual = flip::measure(element);
            let delta = flip::invert(first, last_visual)
                .map(|delta| flip::anchor_correct(delta, last_visual, last_layout));
            match delta {
                Some(delta) if delta != flip::IDENTITY => {
                    let animation = flip::play(element.clone(), delta, config, original);
                    result.push((id.clone(), animation));
                }
                _ => {
                    original.restore(element);
                }
            }
        }
        result
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{play_shared, snapshot, SharedSnapshot};

#[cfg(test)]
mod tests {
    use super::pair_by_id;

    #[test]
    fn pairs_matching_ids_in_after_order() {
        let before_ids = ["hero", "other"];
        let before_connected = [false, false];
        let after_ids = ["other", "hero"];
        let pairs = pair_by_id(&before_ids, &before_connected, &after_ids, |_, _| false);
        assert_eq!(pairs, vec![(0, 1), (1, 0)]);
    }

    #[test]
    fn excludes_same_node_pair() {
        let before_ids = ["hero"];
        let before_connected = [false];
        let after_ids = ["hero"];
        // 同一ノード（`crate::flip` の対象領域）は除外する。
        let pairs = pair_by_id(&before_ids, &before_connected, &after_ids, |_, _| true);
        assert!(pairs.is_empty());
    }

    #[test]
    fn excludes_pair_when_old_element_still_connected() {
        let before_ids = ["hero"];
        let before_connected = [true];
        let after_ids = ["hero"];
        let pairs = pair_by_id(&before_ids, &before_connected, &after_ids, |_, _| false);
        assert!(pairs.is_empty());
    }

    #[test]
    fn ignores_after_id_absent_from_before() {
        let before_ids = ["other"];
        let before_connected = [false];
        let after_ids = ["hero"];
        let pairs = pair_by_id(&before_ids, &before_connected, &after_ids, |_, _| false);
        assert!(pairs.is_empty());
    }

    #[test]
    fn keeps_only_first_occurrence_of_duplicate_after_id() {
        let before_ids = ["hero"];
        let before_connected = [false];
        let after_ids = ["hero", "hero"];
        let pairs = pair_by_id(&before_ids, &before_connected, &after_ids, |_, _| false);
        assert_eq!(pairs, vec![(0, 0)]);
    }
}
