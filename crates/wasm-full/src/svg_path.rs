//! SVG path drawing アニメーションの DOM 配線層（イシュー #2519、親 #2508）。
//!
//! # 責務境界
//!
//! `getTotalLength()` による全長計算・`stroke-dasharray`/`stroke-dashoffset`
//! の初期値計算・WAAPI 呼び出しはいずれも
//! `fandhe-frontend-animation::svg_path::draw_path` の責務であり、本モジュール
//! は以下のみを担う（3 層構成、`docs/design/motion-reference-adoption-policy.md`
//! §4・`docs/design/animation-core-architecture.md` §6.2）:
//!
//! 1. `root` 配下の `[data-fandhe-svg-path-draw]` 要素を出現順に走査
//!    （`in_view.rs::collect_in_view_candidates` と同型の
//!    `query_selector_all` 1 回走査、`docs/design` の設計に従う）
//! 2. 各要素へ `fandhe_frontend_animation::svg_path::draw_path` を呼ぶだけ
//!
//! マウント時に 1 回だけ描画する性質のため、動的挿入要素の追随
//! （`MutationObserver`）は本イシューのスコープ外（YAGNI、必要になれば
//! 別 issue）。`data-*` 属性経由の duration/easing カスタマイズも扱わない
//! （`fandhe_frontend_animation::svg_path::default_options()` の固定値の
//! みを使う、`confetti.rs` と同じ判断）。
//!
//! # Runtime への統合
//!
//! `crate::lib::Runtime::mount`/`Runtime::hydrate` の双方が
//! `Self::wire_confetti` の直後で `Self::wire_svg_path` を呼ぶ（feature
//! `svg-path`、既定 on）。`dispatch` チャネルを持たない属性専用配線のため
//! （`Self::wire_sidebar`/`Self::wire_confetti` と同型）、
//! `Component`/`binding_table`/`keyed_list_cache` は必要としない。
//!
//! # ロケータ契約（security.md A03）
//!
//! [`SVG_PATH_DRAW_ATTR`] は値なし存在マーカーであり、属性値を
//! セレクタ・DOM API へ補間する経路を持たない。対象要素が
//! `SVGGeometryElement`/`SVGElement` のいずれかにキャストできない場合、
//! `fandhe_frontend_animation::svg_path::draw_path` が `Ok(None)` を返し
//! 何も起きない（著者マークアップの誤りとして静かに無視する fail-closed
//! 方針、`confetti.rs` と同じ）。

/// opt-in（著者が SSR 出力に静的に付与）: 描画アニメーション対象の SVG 要素
/// （`<path>`/`<circle>` 等の `SVGGeometryElement`）であることを示す
/// マーカー（値なし存在属性）。
pub const SVG_PATH_DRAW_ATTR: &str = "data-fandhe-svg-path-draw";
/// 対象要素の走査セレクタ。
pub const SVG_PATH_DRAW_SELECTOR: &str = "[data-fandhe-svg-path-draw]";

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::SVG_PATH_DRAW_SELECTOR;
    use fandhe_frontend_animation::svg_path::draw_path;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::Element;

    /// `root` 配下の `[data-fandhe-svg-path-draw]` 要素（複数可）を出現順に
    /// 集める。`query_selector_all` の失敗は空 `Vec` として扱う
    /// （fail-closed、panic しない。`in_view.rs::collect_in_view_candidates`
    /// と同じ方針）。
    fn collect_svg_path_candidates(root: &Element) -> Vec<Element> {
        let Ok(node_list) = root.query_selector_all(SVG_PATH_DRAW_SELECTOR) else {
            return Vec::new();
        };
        let len = node_list.length();
        let mut out = Vec::with_capacity(len as usize);
        for i in 0..len {
            if let Some(node) = node_list.get(i) {
                if let Ok(el) = node.dyn_into::<Element>() {
                    out.push(el);
                }
            }
        }
        out
    }

    /// `root` 配下の `[data-fandhe-svg-path-draw]` 要素へ描画アニメーション
    /// をマウント時に 1 回だけ起動する（[`crate::lib::Runtime::wire_svg_path`]
    /// から呼ばれる）。個々の要素の `draw_path` 呼び出し結果（`Err`/`Ok(None)`
    /// を含む）は無視し、他の要素・他配線へ波及させない（著者マークアップの
    /// 誤りは黙って無視する、`confetti.rs::handle_click` と同じ方針）。
    pub fn wire_svg_path(root: &Element) -> Result<(), JsValue> {
        for element in collect_svg_path_candidates(root) {
            let _ = draw_path(&element);
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_svg_path;
