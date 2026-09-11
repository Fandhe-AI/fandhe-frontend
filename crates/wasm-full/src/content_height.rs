//! collapsible / accordion の content 高さを実測し CSS 変数
//! `--fandhe-content-height` へ書き込む、**部品非依存**の共通配線
//! ヘルパー（イシュー #2191、親トラッキング #2189）。
//!
//! # 背景・責務境界
//!
//! `fandhe-frontend-headless-ui` の disclosure 系（collapsible / accordion）
//! は closed のとき content 要素へ `hidden` 存在属性を付与する契約
//! （`crates/headless-ui/src/collapsible.rs::content` /
//! `crates/headless-ui/src/accordion.rs::item_content` 参照）であり、
//! `hidden` は `display: none` を強制するため `height: auto` への CSS
//! トランジションが成立しない。設計評価
//! `docs/design/collapsible-height-animation.md`（#2190）で確定した
//! **案 C**（headless-ui の `hidden`/`aria-expanded`/`data-state` 契約は
//! 一切変えず、pre-styled-ui 側の離散遷移〔#2192〕と組み合わせて
//! wasm-full が実測高さを CSS 変数へ供給する）のうち、本モジュールは
//! wasm-full 側の実測・書き込みを担う。
//!
//! レイアウト計測は `.claude/rules/coding-rust.md`
//! （`docs/policy/intentional-non-adoption.md` §3.25 規則 2）が
//! headless-ui へ持ち込まず wasm-full/pre-styled-ui の責務とする対象の
//! ため、headless-ui 側の変更は一切伴わない（`crates/headless-ui/` は
//! 本イシューで差分ゼロ）。
//!
//! # 2 層構成（`chart.rs`/`sidebar.rs` と同型）
//!
//! - 純粋層（[`format_content_height`]/[`is_target`]/[`target_selector`]）
//!   は web-sys に依存せず、native の `cargo test` で検証できる。
//! - 配線層（`wiring::sync_content_height`）のみ
//!   `#[cfg(target_arch = "wasm32")]` でゲートする。
//!
//! # 対象パーツの宣言（[`TARGETS`]）
//!
//! 対象は `(data-scope, data-part)` の静的表 [`TARGETS`] のみが決める。
//! 部品名で分岐するコードはここにも `wiring` にも書かない。bubble の
//! `collapse-content`（#2282）はこの表への 1 行追加のみで乗せた。
//!
//! # 書き込み手段の決定（Issue 記載パターンとの意図的な差分）
//!
//! イシュー本文は `position.rs` のような `set_attribute("style", ...)`
//! 直書きパターンを例示するが、本モジュールは最初から CSSOM
//! （[`web_sys::CssStyleDeclaration::set_property`]/`remove_property`、
//! `crate::chart::wiring::set_tooltip_position` に同一クレート内の
//! 先例あり）を採る。理由:
//!
//! 1. `set_attribute("style", ...)` は content 要素に利用者が付けた
//!    他のインライン宣言（例: 開発者が直接 `style` を指定した場合）を
//!    丸ごと破壊する。CSSOM 経由なら当該プロパティのみを更新できる。
//! 2. CSP `style-src` に `unsafe-inline` が無い配布環境でも、
//!    CSSOM 経由のプロパティ設定は拒否されない（`setAttribute` による
//!    `style` 属性の書き換えは環境によりブロックされ得る）。
//! 3. DOM 上の観測結果としては `style` 属性へ
//!    `--fandhe-content-height: 240px` がシリアライズされるため、
//!    「content 要素自身の `style` に値が現れる」という #2192 側との
//!    取り決めは CSSOM 経由でも変わらず満たされる。
//!
//! # `hidden`・0px の扱い（表示状態を偽装しない不変条件）
//!
//! - `hidden` 属性を持つ要素は測定・書き込みの対象から**スキップ**する
//!   （`display: none` 下の `scroll_height()` は常に 0 であり、既存の
//!   変数値を壊さないため）。
//! - 実測が 0 の場合は `0px` を書き込まず、変数を
//!   **除去**する（`wiring::sync_content_height` 参照）。closed な
//!   accordion item にネストした open な collapsible は祖先の
//!   `display: none` により実測 0 を返すため、`0px` を焼き込むと祖先が
//!   開いた直後にネスト先が不可視のまま固定されてしまう。除去すれば
//!   消費側（#2192）のフォールバック `auto` が効く。
//! - 測定前に既存の変数値を除去しない。`overflow: hidden` 下では
//!   `scrollHeight = max(clientHeight, コンテンツ高)` のため、in-place
//!   開閉で content が縮んだ場合に前回の大きい値が残り得る（既知の
//!   限界。再描画で要素が作り直されれば解消する）。
//!
//! # 遷移成立条件についての注記（#2192 との協調点）
//!
//! [`web_sys::Element::scroll_height`] の呼び出しは同期的にスタイル
//! 再計算・レイアウトを強制する。`hidden` 解除（または要素挿入）直後の
//! **最初のスタイル計算**時点で変数が未設定（消費側は `auto` へ
//! フォールバック）だと、`@starting-style` 方式の `0 → auto` 遷移は
//! 補間不能であり、後から変数を設定しても `auto → Npx` は補間不能で
//! 即時スナップになる。したがって本ヘルパー（ステートレス）でオープン
//! 方向の遷移が成立するのは、同一要素に前回の測定値が残っている
//! in-place 開閉の 2 回目以降に限られる。初回オープン、および
//! `set_inner_html` による丸ごと再描画モデル（content 要素が開閉ごとに
//! 新しい要素になる構成）では、遷移なしの即時表示へ自然劣化する
//! （既知の限界。前回値プライミング等の追補は #2191 のスコープ外）。
//!
//! # 禁止事項（ステートレス同期関数のみに留める）
//!
//! `transitionend`/`animationend` を待たない。`MutationObserver`/
//! `ResizeObserver`/`requestAnimationFrame` を新設しない。追加の
//! イベントリスナー・`Closure::forget` を持たない。
//!
//! # セキュリティ不変条件（REQ-1・`security.md` A03）
//!
//! CSS へ流れる文字列は [`format_content_height`] の出力（10 進整数 +
//! `px`）のみであり、`data-value`/`id` 等の利用者・攻撃者制御文字列を
//! `format!` やセレクタへ混ぜない。[`target_selector`] は `&'static str`
//! リテラルのみから組み立てる。
//!
//! # `crate::headless::wire_headless_component` との統合
//!
//! `wiring::sync_content_height` は `wire_headless_component` から
//! (1) 配線時（初期表示の SSR 状態に対する先行同期）、(2)
//! `on_update` 直後（呼び出し側の再描画で content 要素が作り直された
//! 後の要素への同期）の 2 箇所で呼ばれる（`crate::headless` 参照）。
//! `wire_headless_events`/`wire_headless_events_scoped`（アクション通知
//! のみの低レベル API）には統合しない（DOM 反映を伴わないため）。
//!
//! # スコープ外（#2191 §8、Issue 化提案）
//!
//! - `Runtime::apply_dirty_if_any` 経路（`data-action` 駆動アプリ）への
//!   統合。
//! - 前回測定値の `id` キー・プライミング、または #2192 側の keyframe
//!   `animation` 方式採用。
//! - `overflow: hidden` 下で content が縮んだ場合に前回値が残る限界の
//!   解消。
//! - bubble（#2282 で [`TARGETS`] へ適用済み）を含め、`crate::headless`
//!   の `MAPPING_TABLE` に配線が無い部品では `wire_headless_component`
//!   経由のクリックで本モジュールが呼ばれない（bubble は
//!   `crates/headless-ui/src/bubble.rs` rustdoc「wasm-full 未配線」節
//!   参照。呼び出し側が独自に `hidden` を切り替え
//!   `sync_content_height` 相当を呼ぶ経路を用意する必要がある）。
//!   ただし本モジュールの同期はあくまで「実測値を書き込む」役割に
//!   留まる: `crates/pre-styled-ui` の共通 preset
//!   （`SlotRecipe::content_height_transition`）は `calc-size()`
//!   対応ブラウザでは `hidden` の切り替えのみで高さトランジションを
//!   成立させ本モジュールの同期を必須としない一方、`calc-size()`
//!   未対応ブラウザでは `@supports not (...)` が
//!   `transition: none` 相当を適用するため本モジュールが同期して
//!   いても遷移しない（`crates/pre-styled-ui/src/bubble.rs` モジュール
//!   doc・`crates/pre-styled-ui/src/recipe.rs` の
//!   `content_height_open_declarations` rustdoc 参照）。トリガー配線の
//!   有無と CSS の遷移条件は独立した別々の前提である。

/// content 高さを供給する CSS カスタムプロパティ名。
///
/// `crates/pre-styled-ui`（#2192）が消費側として同じリテラルを参照する
/// 契約であり、本クレート内では `wiring::sync_content_height` のみが
/// 書き込む（唯一の書き込み経路）。
pub const CONTENT_HEIGHT_VAR: &str = "--fandhe-content-height";

/// 実測対象の `(data-scope, data-part)` 静的表。
///
/// 対象の追加・削除はこの表への行の増減のみで行う。部品名で分岐する
/// コードを `wiring` 側に書かない（モジュール doc 参照）。bubble の
/// `collapse-content`（イシュー #2282）を追加済み。
pub const TARGETS: &[(&str, &str)] = &[
    ("collapsible", "content"),
    ("accordion", "item-content"),
    ("bubble", "collapse-content"),
];

/// `scope`/`part` が [`TARGETS`] のいずれかに一致するかを返す。
#[must_use]
pub fn is_target(scope: &str, part: &str) -> bool {
    TARGETS.iter().any(|&(s, p)| s == scope && p == part)
}

/// [`TARGETS`] から `querySelectorAll` に渡す静的 CSS セレクタ文字列を
/// 組み立てる。
///
/// 入力はすべて `&'static str` リテラル（[`TARGETS`]）のみであり、
/// 利用者・攻撃者制御の文字列は混ぜない（REQ-1・`security.md` A03）。
#[must_use]
pub fn target_selector() -> String {
    TARGETS
        .iter()
        .map(|(scope, part)| format!(r#"[data-scope="{scope}"][data-part="{part}"]"#))
        .collect::<Vec<_>>()
        .join(",")
}

/// 実測高さ（`scrollHeight`、負値は取り得ないが `i32` 由来のため負値も
/// 受け取れる signature）から CSS へ書き込む文字列を組み立てる。
///
/// - 負値: `None`（呼び出し側は変数へ触れない）。
/// - `0`: `Some("0px".to_string())`。**呼び出し側（`wiring`）はこの
///   場合に変数を書き込まず除去する**契約であり、本関数自体は
///   `"0px"` を返すのみで除去判断は行わない（判断の分離）。
/// - 正値: `Some(format!("{n}px"))`。
///
/// CSS へ流れる文字列はこの関数の出力（10 進数字 + `px`）のみであり、
/// `format!` に利用者制御文字列を混ぜない不変条件を単体で固定する。
#[must_use]
pub fn format_content_height(scroll_height: i32) -> Option<String> {
    let n = u32::try_from(scroll_height).ok()?;
    Some(format!("{n}px"))
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{is_target, target_selector, CONTENT_HEIGHT_VAR};
    use wasm_bindgen::JsCast;
    use web_sys::{Element, HtmlElement};

    /// `root` 配下（`root` 自身を含む）の対象 content 要素の実測高さを
    /// [`CONTENT_HEIGHT_VAR`] へ同期する。
    ///
    /// `crate::headless::wire_headless_component` から (1) 配線時の
    /// 先行同期、(2) `on_update` 直後の再描画後同期、の 2 箇所で呼ばれる
    /// （モジュール doc「`wire_headless_component` との統合」節）。
    ///
    /// # 処理順序（モジュール doc の不変条件と対応）
    ///
    /// 1. `hidden` 属性を持つ要素はスキップする（既存値を壊さない）。
    /// 2. `HtmlElement` へダウンキャストできない要素はスキップする。
    /// 3. `scroll_height()` を実測する（既存の変数値は測定前に除去
    ///    しない）。
    /// 4. 実測 0 → 変数を `remove_property`。正値 →
    ///    `format_content_height` の出力を `set_property`。
    ///
    /// エラー（`JsValue`）は呼び出し側が無視できるよう `Result` で返す
    /// （`chart.rs` は `let _ =` で無視する先例に倣う）。
    pub fn sync_content_height(root: &Element) -> Result<(), wasm_bindgen::JsValue> {
        let selector = target_selector();

        // root 自身が対象パーツである場合（`query_selector_all` は
        // 子孫のみを列挙し root 自身を含まないため）も同期対象に含める。
        if let (Some(scope), Some(part)) = (
            root.get_attribute("data-scope"),
            root.get_attribute("data-part"),
        ) {
            if is_target(&scope, &part) {
                sync_one(root);
            }
        }

        let nodes = root.query_selector_all(&selector)?;
        let len = nodes.length();
        for i in 0..len {
            let Some(node) = nodes.get(i) else {
                continue;
            };
            let Some(element) = node.dyn_ref::<Element>() else {
                continue;
            };
            sync_one(element);
        }
        Ok(())
    }

    /// 単一要素に対する実測・書き込み（[`sync_content_height`] の本体）。
    fn sync_one(element: &Element) {
        if element.has_attribute("hidden") {
            return;
        }
        let Some(html) = element.dyn_ref::<HtmlElement>() else {
            return;
        };
        let scroll_height = element.scroll_height();
        let style = html.style();
        match super::format_content_height(scroll_height) {
            Some(px) if scroll_height > 0 => {
                let _ = style.set_property(CONTENT_HEIGHT_VAR, &px);
            }
            _ => {
                // 実測 0（またはあり得ないはずの負値）は変数を除去する。
                // `0px` を焼き込まない理由はモジュール doc
                // 「`hidden`・0px の扱い」節を参照。
                let _ = style.remove_property(CONTENT_HEIGHT_VAR);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::sync_content_height;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_content_height_rejects_negative() {
        assert_eq!(format_content_height(-1), None);
        assert_eq!(format_content_height(i32::MIN), None);
    }

    #[test]
    fn format_content_height_zero_is_0px() {
        assert_eq!(format_content_height(0), Some("0px".to_string()));
    }

    #[test]
    fn format_content_height_positive() {
        assert_eq!(format_content_height(240), Some("240px".to_string()));
        assert_eq!(
            format_content_height(i32::MAX),
            Some(format!("{}px", i32::MAX as u32))
        );
    }

    #[test]
    fn is_target_matches_targets_table() {
        assert!(is_target("collapsible", "content"));
        assert!(is_target("accordion", "item-content"));
        assert!(is_target("bubble", "collapse-content"));
        assert!(!is_target("collapsible", "trigger"));
        assert!(!is_target("dialog", "content"));
        assert!(!is_target("bubble", "collapse-trigger"));
    }

    #[test]
    fn target_selector_joins_targets_table() {
        assert_eq!(
            target_selector(),
            r#"[data-scope="collapsible"][data-part="content"],[data-scope="accordion"][data-part="item-content"],[data-scope="bubble"][data-part="collapse-content"]"#
        );
    }
}
