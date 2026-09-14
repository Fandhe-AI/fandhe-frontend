//! keyed list の挿入・並べ替え後、各行要素へ stagger index を CSSOM で
//! 書き込む配線（イシュー #2397）。
//!
//! # 背景・責務境界
//!
//! `fandhe-frontend-pre-styled-ui::recipe::STAGGER_INDEX_VAR`（`motion`
//! feature 配下、#2384）と同一のカスタムプロパティ名を書く。SSR/初期描画
//! 時の書き出しは呼び出し側アプリ/pre-styled-ui の責務
//! （`recipe::stagger_index_style`）であり、本モジュールは**動的更新**
//! （`Runtime::apply_update_for_dirty` の keyed list 構造反映後）にのみ
//! 関与する。`content_height.rs` と同型の 2 層構成: 純粋層
//! （[`stagger_index_value`]）は native `cargo test` で検証可能、配線層
//! （`wiring::sync_stagger_index`）のみ `#[cfg(target_arch = "wasm32")]`。
//!
//! index は「起点からの距離」の `First` 相当（0 始まりの DOM 順位置）の
//! みを書く。`Center`/`Last` 起点や `fandhe_animation::timeline::Stagger`
//! の遅延計算そのものはアプリ/pre-styled-ui 側の責務
//! （`recipe.rs` rustdoc「`fandhe-animation` の `Stagger` との対応」節
//! 参照）であり、本モジュールは書き換えない。
//!
//! # 走査方法についての注記（性能上の既知の落とし穴を踏襲回避）
//!
//! `Element::children()`（`HtmlCollection`）+ `item(index)` によるランダム
//! アクセス走査は使わない。`HTMLCollection` は live collection であり
//! `item(index)` の計算量は仕様上保証されないため、繰り返し呼ぶと退行し
//! 得る（`crates/wasm-client/Cargo.toml` イシュー #1319 の教訓、
//! `keyed_dom.rs` の `WebSysKeyedDom` が同じ理由で `first_element_child`/
//! `next_element_sibling` の 1 パス走査へ切り替えた前例）。本モジュールも
//! 同じ sibling 走査（1 パス O(n)）を採用する。
//!
//! # セキュリティ不変条件（REQ-1・security.md A03）
//!
//! CSS へ流れる文字列は [`stagger_index_value`] の出力（`usize` の 10 進
//! 表記のみ）であり、利用者・攻撃者制御の文字列は混ざらない。

/// `--fandhe-motion-stagger-index` の custom property 名。
///
/// `fandhe-frontend-pre-styled-ui::recipe::STAGGER_INDEX_VAR` と同一
/// リテラルを保つ契約（ドリフト検知は
/// `crates/pre-styled-ui/tests/stagger_index_var_drift.rs`）。
pub const STAGGER_INDEX_VAR: &str = "--fandhe-motion-stagger-index";

/// `index` から CSS へ書き込む値文字列（10 進数のみ）を組み立てる純粋関数。
#[must_use]
pub fn stagger_index_value(index: usize) -> String {
    index.to_string()
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{stagger_index_value, STAGGER_INDEX_VAR};
    use wasm_bindgen::JsCast;
    use web_sys::{Element, HtmlElement};

    /// `list_element` の直接の要素子（keyed list の各行）を DOM 順に
    /// 1 パス走査し、0 始まりの位置を [`STAGGER_INDEX_VAR`] へ書き込む。
    ///
    /// `Runtime::apply_update_for_dirty` の keyed list 構造反映直後に
    /// 呼ばれる（`Insert`/`Move` を含むあらゆる構造変化コミット後に呼ぶ
    /// ため、「今回変化した行だけ」ではなく全行を再計算する。冪等かつ
    /// `content_height::sync_content_height` と同型の「毎回再同期」方針）。
    pub fn sync_stagger_index(list_element: &Element) {
        let mut current = list_element.first_element_child();
        let mut index: usize = 0;
        while let Some(el) = current {
            if let Some(html) = el.dyn_ref::<HtmlElement>() {
                let _ = html
                    .style()
                    .set_property(STAGGER_INDEX_VAR, &stagger_index_value(index));
            }
            current = el.next_element_sibling();
            index += 1;
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::sync_stagger_index;

#[cfg(test)]
mod tests {
    use super::stagger_index_value;

    #[test]
    fn stagger_index_value_is_plain_decimal() {
        assert_eq!(stagger_index_value(0), "0");
        assert_eq!(stagger_index_value(1), "1");
        assert_eq!(stagger_index_value(41), "41");
    }
}
