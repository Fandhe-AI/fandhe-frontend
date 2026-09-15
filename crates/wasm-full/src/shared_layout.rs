//! 共有レイアウト遷移（motion.dev `layoutId` 相当）の `data-*` 配線
//! （イシュー #2536、`feature = "layout-animation"`。`crates/wasm-full/
//! src/layout_flip.rs` の「同一要素」対象範囲を補完する）。
//!
//! # 責務境界
//!
//! 突合（id ごとの旧要素・新要素の対応付け）・計測・Invert・Play は
//! すべて [`fandhe_frontend_animation::shared_layout`] の責務であり、本
//! モジュールはそれを `Runtime::apply_update_for_dirty`/`rerender`/
//! `view_transition_swap` という「いつ呼ぶか」にのみ関与する
//! （`layout_flip.rs` モジュール doc「責務境界」節と同じ設計）。DOM
//! 計測・transform 計算のロジックは一切持たない。
//!
//! # 対象要素の明示的オプトイン
//!
//! [`LAYOUT_ID_ATTR`] を持つ要素にのみ適用する（`layout_flip::
//! FLIP_AUTO_ATTR` と同じ opt-in 設計）。値は id として使う（`HashMap`
//! キーのみに使用し、CSS/セレクタへ混入しない、`layout_flip.rs`
//! モジュール doc「セキュリティ不変条件」参照）。
//!
//! # 呼び出しタイミング
//!
//! [`capture_before`] は構造変化が DOM へ適用される**前**、[`play_after`]
//! は適用された**後**に呼ぶ（`layout_flip::capture_before`/`play_after`
//! と同じ契約）。`Runtime::apply_update_for_dirty`/`rerender`/
//! `view_transition_swap` の該当箇所から呼ばれる。
//!
//! # ハンドル保持
//!
//! 再生中の [`fandhe_frontend_animation::flip::FlipAnimation`] は
//! [`ACTIVE`]（id → ハンドル）で保持する。[`capture_before`] の呼び出し
//! ごとに収束済み（`is_done()`）のエントリを prune し、id 数で有界に保つ
//! （`layout_flip::FLIP_LOOPS` のような `setTimeout` 自己クリーンアップは
//! 持たない——`layoutId` の総数は通常アプリの静的な部品数に収まり、次回
//! 更新で必ず prune 機会が来るため。`ponytail:` id 数が数千件規模で
//! 増え続ける構成では prune 頻度が不足し得る。上限が必要になれば
//! `layout_flip::FLIP_LOOPS` と同型の `setTimeout` クリーンアップへ拡張
//! する）。`insert` による置換で旧ハンドルが drop されれば、未収束時は
//! [`fandhe_frontend_animation::flip::FlipAnimation`] の [`Drop`] が
//! 元のスタイルへ復元する（`layout_flip.rs` と同じ契約）。

/// 要素に付け、共有レイアウト遷移の対象であることを明示するオプトイン
/// 属性名。値は id として [`ACTIVE`]/突合の `HashMap` キーに使う
/// （`layout_flip::FLIP_AUTO_ATTR` と異なり値そのものを使う点に注意。
/// CSS/セレクタへは混入しない、モジュール doc「対象要素の明示的
/// オプトイン」参照）。
pub const LAYOUT_ID_ATTR: &str = "data-fandhe-layout-id";

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::LAYOUT_ID_ATTR;
    use fandhe_frontend_animation::fandhe_animation::spring::SpringConfig;
    use fandhe_frontend_animation::flip::FlipAnimation;
    use fandhe_frontend_animation::shared_layout::{self, SharedSnapshot};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use wasm_bindgen::JsCast;
    use web_sys::{Element, HtmlElement};

    thread_local! {
        /// id ごとの進行中の共有レイアウト遷移（モジュール doc
        /// 「ハンドル保持」参照）。
        static ACTIVE: RefCell<HashMap<String, FlipAnimation>> = RefCell::new(HashMap::new());
    }

    /// `root` 配下（`root` 自身も含む）で [`LAYOUT_ID_ATTR`] を持つ要素を
    /// すべて収集し、`(id, HtmlElement)` の一覧を返す（[`capture_before`]/
    /// [`play_after`] 共通の走査。`root` 自身を含めるのは、共有レイアウト
    /// 遷移の対象が `root` 直下ではなく `root` 自身になる構成
    /// 〔単一要素をまるごと差し替える `rerender`〕にも対応するため）。
    fn collect(root: &Element) -> Vec<(String, HtmlElement)> {
        let mut result = Vec::new();
        if let Some(id) = root.get_attribute(LAYOUT_ID_ATTR) {
            if let Ok(html) = root.clone().dyn_into::<HtmlElement>() {
                result.push((id, html));
            }
        }
        let Ok(node_list) = root.query_selector_all(&format!("[{LAYOUT_ID_ATTR}]")) else {
            return result;
        };
        for i in 0..node_list.length() {
            let Some(node) = node_list.get(i) else {
                continue;
            };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            let Some(id) = element.get_attribute(LAYOUT_ID_ATTR) else {
                continue;
            };
            if let Ok(html) = element.dyn_into::<HtmlElement>() {
                result.push((id, html));
            }
        }
        result
    }

    /// `root` 配下の [`LAYOUT_ID_ATTR`] 付き要素の現在の視覚矩形を捕捉する
    /// （構造変化を DOM へ適用する**前**に呼ぶ想定、モジュール doc
    /// 「呼び出しタイミング」参照）。0 件なら `query_selector_all` の
    /// reflow のみで測定コストは発生しない（[`shared_layout::snapshot`]
    /// は空の iterator に対して `measure` を 0 回呼ぶ）。
    #[must_use]
    pub fn capture_before(root: &Element) -> Option<SharedSnapshot> {
        // 収束済みエントリの prune（モジュール doc「ハンドル保持」参照）。
        ACTIVE.with(|cell| cell.borrow_mut().retain(|_, anim| !anim.is_done()));
        let entries = collect(root);
        if entries.is_empty() {
            return None;
        }
        Some(shared_layout::snapshot(entries))
    }

    /// `snapshot`（[`capture_before`] の結果）と `root` 配下の構造変化
    /// 反映後の現在の要素を突き合わせ、共有レイアウト遷移を再生する
    /// （構造変化を DOM へ適用した**後**に呼ぶ想定）。`snapshot` が `None`
    /// （[`capture_before`] が対象なしと判定した）場合は no-op。
    pub fn play_after(root: &Element, snapshot: Option<SharedSnapshot>) {
        let Some(snapshot) = snapshot else {
            return;
        };
        let after = collect(root);
        if after.is_empty() {
            return;
        }
        let played = shared_layout::play_shared(&snapshot, &after, SpringConfig::default());
        ACTIVE.with(|cell| {
            let mut active = cell.borrow_mut();
            for (id, animation) in played {
                active.insert(id, animation);
            }
        });
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{capture_before, play_after};
