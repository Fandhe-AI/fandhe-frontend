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
//! # 対象リストの明示的オプトイン（イシュー #2397 codex-review P1 是正）
//!
//! 上記の責務境界を DOM 上でも守るため、[`wiring::sync_stagger_index`] は
//! [`STAGGER_AUTO_FIRST_ATTR`] を持つ keyed list にのみ適用する。この属性
//! を持たないリスト（`Center`/`Last` 起点をアプリ側が計算して
//! `stagger_index_style` を直接書いているリスト）は、キー無関係の
//! keyed list 更新のたびに DOM 順位置で無条件上書きされることがない。
//! `First` 起点の自動追随を使いたい呼び出し側は、`keyed_list` の親属性
//! （`attrs` 引数）へ `(STAGGER_AUTO_FIRST_ATTR, "")` を明示的に加える。
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

/// keyed list の親要素に付け、[`wiring::sync_stagger_index`] による
/// `First` 起点（DOM 順位置）自動同期の対象であることを明示する
/// オプトイン属性名（イシュー #2397 codex-review P1 是正）。
///
/// `Center`/`Last` 起点で `index` をアプリ側が計算・管理するリストは
/// この属性を付けない（付けなければ本モジュールは一切書き込まない）。
/// 値は不問（存在のみを見る）。
pub const STAGGER_AUTO_FIRST_ATTR: &str = "data-fandhe-stagger-auto-first";

/// [`crate::list_presence::PRESENCE_AUTO_ATTR`]（イシュー #2544）と同一
/// リテラルの属性名。「既存行の delay 更新方針」（下記
/// [`wiring::sync_stagger_index`] doc）の巻き戻り防止凍結を、退場ゴースト
/// 演出（`pre-styled-ui::list_motion` の `animation-fill-mode: both` 前提）
/// が実際に存在するリストへ限定するために使う（codex-review P1 是正、
/// イシュー #2544）。
///
/// `list_presence` モジュールを直接参照しない（モジュール参照ではなく
/// リテラル複製にする）理由: `list_presence` は feature `"presence"`
/// 配下（既定 on だが無効化可能）であり、本モジュールを利用する
/// feature `"stagger"` は独立した別枠 feature のため、`"presence"` を
/// 無効化した構成でも本モジュールが単独でコンパイルできる必要がある
/// （`pre-styled-ui::list_motion::PRESENCE_AUTO_ATTR` が同じ理由で
/// リテラル複製している契約と同型、ドリフト検知は
/// `presence_auto_attr_sync_matches_list_presence_literal`）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const PRESENCE_AUTO_ATTR: &str = "data-fandhe-presence-auto";

/// `index` から CSS へ書き込む値文字列（10 進数のみ）を組み立てる純粋関数。
#[must_use]
pub fn stagger_index_value(index: usize) -> String {
    index.to_string()
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        stagger_index_value, PRESENCE_AUTO_ATTR, STAGGER_AUTO_FIRST_ATTR, STAGGER_INDEX_VAR,
    };
    use wasm_bindgen::JsCast;
    use web_sys::{Element, HtmlElement};

    /// [`STAGGER_AUTO_FIRST_ATTR`] を持つ `list_element` に限り、直接の
    /// 要素子（keyed list の各行）を DOM 順に 1 パス走査し、0 始まりの
    /// 位置を [`STAGGER_INDEX_VAR`] へ書き込む。属性を持たないリスト
    /// （`Center`/`Last` 起点をアプリが管理するリスト）は no-op で
    /// 抜ける（モジュール冒頭 doc「対象リストの明示的オプトイン」参照）。
    ///
    /// `Runtime::apply_update_for_dirty` の keyed list 構造反映直後に
    /// 呼ばれる（`Insert`/`Move` を含むあらゆる構造変化コミット後に呼ぶ）。
    /// 全行を DOM 順に走査するが、書き込むのは
    /// [`STAGGER_INDEX_VAR`] を**まだ持たない行のみ**（下記「既存行の
    /// delay を凍結する理由」参照）。`content_height::sync_content_height`
    /// と異なり「毎回無条件で全行上書き」ではない点に注意。
    ///
    /// `data-key` を持たない要素（`list_presence::play_exit_after` が
    /// 挿入した退場ゴースト。`data-key` は `strip_selector` で必ず剥がされる）
    /// は走査で無視し index を消費しない（codex-review/Bugbot 指摘是正）。
    /// keyed DOM の末尾挿入ロジック（`keyed_dom.rs`）も同じく `data-key`
    /// 無し要素を「現在の keyed 行」として扱わない既存契約であり、両者を
    /// 揃えないと presence + stagger 併用時にゴースト存続中の末尾追加行の
    /// index がずれ、ゴースト除去後も誤った値が残留する。
    ///
    /// # 既存行の delay 更新方針（codex-review 指摘是正、イシュー #2544）
    ///
    /// [`PRESENCE_AUTO_ATTR`] を併せ持つリスト（`list_presence` の退場
    /// ゴースト演出対象。`pre-styled-ui::list_motion::enter_css` の
    /// `animation-fill-mode: both` が実際に効く構成）に限り、既に
    /// [`STAGGER_INDEX_VAR`] を持つ行（前回までの構造変化で書き込み
    /// 済み、または SSR が `stagger_index_style` で書き出した初期値）は、
    /// 新たに算出した DOM 順位置が**現在値以下**の場合にのみ上書きする。
    /// 現在値より大きい（＝先頭側への `Insert` でこの行が後方へ押し
    /// 出された）場合は上書きしない。
    ///
    /// 理由: CSS Animations は `animation-delay` を伸ばすと現在の経過
    /// 時間が再び delay 前フェーズへ戻り得るため（`animation-fill-mode:
    /// both` と組み合わさると、既に enter アニメーション再生済みの行が
    /// `opacity: 0` へ巻き戻る）。一方 delay を縮める・変えない更新は
    /// この巻き戻りを起こさない（経過時間は既に新しい delay + duration
    /// を超えたまま）。この非対称性を利用し、「巻き戻りを起こし得る
    /// 更新のみ凍結し、それ以外（純粋な `Move` による前方移動・新規行）
    /// は DOM 順へ追随させる」判定にする。
    ///
    /// [`PRESENCE_AUTO_ATTR`] を持たないリスト（stagger 単独利用。
    /// `enter_css` の `fill-mode: both` 巻き戻り対象になり得ない）は、
    /// 凍結せず常に DOM 順位置へ無条件上書きする（codex-review P1
    /// 是正、イシュー #2544 fix ラウンド）: 以前は全リストへ一律で凍結
    /// を適用しており、既存行を逆順に並べ替えると一部の行が凍結された
    /// まま index が更新されなくなり、`STAGGER_AUTO_FIRST_ATTR` の
    /// 「DOM 順位置へ自動同期する」契約が崩れていた。
    pub fn sync_stagger_index(list_element: &Element) {
        if !list_element.has_attribute(STAGGER_AUTO_FIRST_ATTR) {
            return;
        }
        let freeze_on_increase = list_element.has_attribute(PRESENCE_AUTO_ATTR);
        let mut current = list_element.first_element_child();
        let mut index: usize = 0;
        while let Some(el) = current {
            if let Some(html) = el.dyn_ref::<HtmlElement>() {
                if html.has_attribute(fandhe_frontend_core::keyed::KEY_ATTR) {
                    let style = html.style();
                    let should_write = if freeze_on_increase {
                        let existing = style
                            .get_property_value(STAGGER_INDEX_VAR)
                            .unwrap_or_default();
                        let existing_index = existing.trim().parse::<usize>().ok();
                        existing_index.is_none_or(|value| index <= value)
                    } else {
                        true
                    };
                    if should_write {
                        let _ = style.set_property(STAGGER_INDEX_VAR, &stagger_index_value(index));
                    }
                    index += 1;
                }
            }
            current = el.next_element_sibling();
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::sync_stagger_index;

#[cfg(test)]
mod tests {
    use super::stagger_index_value;
    #[cfg(feature = "presence")]
    use super::PRESENCE_AUTO_ATTR;

    /// `feature = "presence"` が有効な構成でのみ、`list_presence` 側の
    /// 定義とのドリフトを直接比較できる（無効構成では
    /// [`PRESENCE_AUTO_ATTR`] のリテラル複製理由〔モジュール doc
    /// 参照〕により本テスト自体が対象外になる）。
    #[cfg(feature = "presence")]
    #[test]
    fn presence_auto_attr_matches_list_presence_literal() {
        assert_eq!(PRESENCE_AUTO_ATTR, crate::list_presence::PRESENCE_AUTO_ATTR);
    }

    #[test]
    fn stagger_index_value_is_plain_decimal() {
        assert_eq!(stagger_index_value(0), "0");
        assert_eq!(stagger_index_value(1), "1");
        assert_eq!(stagger_index_value(41), "41");
    }
}
