//! keyed list の削除行を退場アニメーションさせる薄い配線（イシュー
//! #2544、`feature = "presence"`）。
//!
//! # 責務境界
//!
//! 座標計測・ゴースト配置・除去タイマーは
//! `fandhe_frontend_animation::presence` の責務であり、本モジュールは
//! それを keyed list の構造変化コミット前後という「いつ呼ぶか」にのみ
//! 関与する（`layout_flip.rs`・`docs/design/animation-core-architecture.md`
//! と同じ層分離）。DOM 計測・アニメーション時間計算のロジックは一切
//! 持たない。
//!
//! # 対象リストの明示的オプトイン
//!
//! [`PRESENCE_AUTO_ATTR`] を持つ keyed list にのみ適用する
//! （`layout_flip::FLIP_AUTO_ATTR`/`stagger_index::STAGGER_AUTO_FIRST_ATTR`
//! と同じ設計）。この属性を持たないリストは、keyed list 構造変化のたびに
//! 無条件で計測・ゴースト化されることがない。
//!
//! # ゴーストがフレームワーク配線から見えないようにする理由
//!
//! [`play_exit_after`] は挿入したゴーストから `data-key`（keyed list の
//! 差分アルゴリズムが読む識別子）・`data-bind-*`（束縛点）・`data-action`
//! （イベント委譲）を [`strip_selector`] で剥がす。これにより:
//!
//! - `Runtime::apply_update_for_dirty` の次回コミットで `dom_item_keys`/
//!   `ensure_children_cache` がゴーストを「現在の keyed 行」として誤認
//!   しない（`data-key` 無し要素は無視される既存契約）。
//! - `layout_flip::flip_lists_containing`/`capture_before` がゴーストを
//!   FLIP 対象として拾わない（同じく `data-key` 無し要素を無視する既存
//!   契約）。
//! - `BindingTable::scan`・イベント委譲がゴースト（またはその子孫）を
//!   束縛点・アクション起点として再スキャンしない。
//!
//! ゴーストは list 末尾へ挿入するため、`sync_stagger_index` の DOM 順
//! 位置計算（先頭からの index）にも影響しない。

// native（非 wasm32）ビルドでは wasm32 専用の `capture_before`/
// `play_exit_after`/`strip_framework_attrs` 自体がコンパイルされないため
// （下記 `#[cfg(target_arch = "wasm32")]`）、その内部でのみ使う型は
// 同じ cfg でガードする（未使用 import 警告を避ける、`confetti.rs` と
// 同じ方針）。
#[cfg(target_arch = "wasm32")]
use fandhe_frontend_animation::presence::RowSnapshot;
#[cfg(target_arch = "wasm32")]
use web_sys::Element;

/// keyed list の親要素に付け、[`capture_before`]/[`play_exit_after`] に
/// よる削除行の退場アニメーション対象であることを明示するオプトイン
/// 属性名。値は不問（存在のみを見る）。
pub const PRESENCE_AUTO_ATTR: &str = "data-fandhe-presence-auto";

/// ゴーストから剥がすフレームワーク属性名の一覧（決定的、native
/// テストで検証可能）。
pub const STRIPPED_ATTRS: &[&str] = &[
    fandhe_frontend_core::keyed::KEY_ATTR,
    fandhe_frontend_core::BIND_TEXT_ATTR,
    fandhe_frontend_core::BIND_ATTR_ATTR,
    fandhe_frontend_core::BIND_CLASS_ATTR,
    fandhe_frontend_core::keyed::BIND_LIST_ATTR,
    "data-action",
];

/// [`STRIPPED_ATTRS`] のいずれかを持つ要素（ゴースト自身 + 子孫）を選ぶ
/// `querySelectorAll` セレクタを組み立てる（決定的、リテラルのみ）。
#[must_use]
pub fn strip_selector() -> String {
    STRIPPED_ATTRS
        .iter()
        .map(|attr| format!("[{attr}]"))
        .collect::<Vec<_>>()
        .join(",")
}

/// [`PRESENCE_AUTO_ATTR`] を持つ `list_element` に限り、削除前の全行
/// スナップショットを撮る。属性を持たないリストは `None`
/// （モジュール doc「対象リストの明示的オプトイン」参照）。
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn capture_before(list_element: &Element) -> Option<Vec<RowSnapshot>> {
    if !list_element.has_attribute(PRESENCE_AUTO_ATTR) {
        return None;
    }
    Some(fandhe_frontend_animation::presence::snapshot_rows(
        list_element,
        fandhe_frontend_core::keyed::KEY_ATTR,
    ))
}

/// keyed list の構造変化コミット後、`before`（[`capture_before`] が返した
/// スナップショット）のうち DOM から切り離された行（= `Remove` された
/// 元要素）をゴーストとして `list` の末尾へ再挿入し、フレームワーク属性を
/// 剥がしてから実測アニメーション時間経過後の除去をスケジュールする。
///
/// `list` の [`PRESENCE_AUTO_ATTR`] を再確認する（`capture_before` 呼び
/// 出し後に同一更新内で属性が除去された場合、新規ゴーストは生成しない。
/// `layout_flip::play_after` の "オプトイン状態を再生前に確認する" 契約
/// と同型）。
#[cfg(target_arch = "wasm32")]
pub fn play_exit_after(list: &Element, before: Vec<RowSnapshot>) {
    if !list.has_attribute(PRESENCE_AUTO_ATTR) {
        return;
    }
    let Some(list_html) = wasm_bindgen::JsCast::dyn_ref::<web_sys::HtmlElement>(list) else {
        return;
    };
    fandhe_frontend_animation::presence::ensure_positioned(list_html);

    let selector = strip_selector();
    for row in &before {
        let Some(ghost) = fandhe_frontend_animation::presence::insert_exit_ghost(list, row) else {
            continue;
        };
        strip_framework_attrs(&ghost, &selector);
        fandhe_frontend_animation::presence::remove_when_settled(ghost);
    }
}

/// `ghost`（自身 + [`strip_selector`] に一致する子孫）から
/// [`STRIPPED_ATTRS`] を全件剥がす。
#[cfg(target_arch = "wasm32")]
fn strip_framework_attrs(ghost: &web_sys::HtmlElement, selector: &str) {
    // `ghost` 自身（`HtmlElement` は `Element` へ `Deref` するため
    // `remove_attribute` を直接呼べる）。
    for attr in STRIPPED_ATTRS {
        let _ = ghost.remove_attribute(attr);
    }
    if selector.is_empty() {
        return;
    }
    let Ok(nodes) = ghost.query_selector_all(selector) else {
        return;
    };
    for i in 0..nodes.length() {
        let Some(node) = nodes.item(i) else {
            continue;
        };
        let Ok(el) = wasm_bindgen::JsCast::dyn_into::<Element>(node) else {
            continue;
        };
        for attr in STRIPPED_ATTRS {
            let _ = el.remove_attribute(attr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_selector_wraps_each_attr_in_brackets() {
        let selector = strip_selector();
        for attr in STRIPPED_ATTRS {
            assert!(selector.contains(&format!("[{attr}]")));
        }
        assert_eq!(selector.matches(',').count(), STRIPPED_ATTRS.len() - 1);
    }

    #[test]
    fn strip_selector_is_deterministic() {
        assert_eq!(strip_selector(), strip_selector());
    }
}
