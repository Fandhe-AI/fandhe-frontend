//! Message Scroller（`fandhe-frontend-headless-ui` `message_scroller` モジュール）の
//! 最下部追従（stick-to-bottom）・新着検知・履歴読み込み時のスクロール
//! 位置維持を配線する（イシュー #2122、親 #2120、祖父 #2001）。
//!
//! `crates/headless-ui/src/message_scroller.rs` は anatomy（`root`/
//! `viewport`/`content`/`anchor`/`jump-to-latest`/`load-more`）と表示状態
//! `data-*`（`data-stuck`/`data-has-new`/[jump-to-latest]の`data-visible`/
//! [load-more]の`data-loading`）のみを提供し、実行時のスクロール計測・
//! 自動追従・新着検知・履歴読み込み時の位置補正は本クレートへ申し送られ
//! ている（同モジュール冒頭 rustdoc「呼び出し文脈」節、
//! `.claude/rules/coding-rust.md` §3.25 規則 2）。本モジュールがその配線を
//! 実装する。
//!
//! # 責務境界（§3.25 規則 1）
//!
//! ストリーミングの購読・履歴取得（`load-more` 押下後の実データ取得）は
//! アプリケーション責務であり、本モジュールは持たない。`load-more`
//! クリックは [`ACTION_LOAD_MORE`] としてアプリへ通知するのみで、
//! `data-loading`/`data-disabled` の付け外し・要素の挿入は一切行わない。
//!
//! # 2 層構成（`questionnaire.rs`/`sidebar.rs`/`content_height.rs` と同型）
//!
//! - 純粋層（web-sys 非依存）は native の `cargo test` で検証できる:
//!   [`is_at_bottom`]・[`stuck_from_attr`]・[`jump_visible`]・
//!   [`classify_change`]・[`corrected_scroll_top`]・[`plan_after_change`]・
//!   [`encode_notification_payload`]。
//! - 配線層（[`wiring::wire_message_scroller_events`]）のみ
//!   `#[cfg(target_arch = "wasm32")]` でゲートする。
//!
//! # 最下部判定（しきい値付き `scrollTop` 算術）
//!
//! [`IntersectionObserver`] ベースの `anchor` 監視は web-sys feature
//! 未追加であり、しきい値付きの `scrollTop`/`scrollHeight`/`clientHeight`
//! 算術（[`is_at_bottom`]）は追加のレイアウト読み取りを要しないため、
//! 本イテレーションでは `anchor` パーツを観測しない（DOM 上の `anchor`
//! はそのまま残し、除去も要求もしない）。`IntersectionObserver` ベースの
//! 検知への移行は将来のスコープ外事項として扱う
//! （`docs/design/wasm-full-architecture.md` §29.4 参照）。
//!
//! # 可視判定の確定（`jump-to-latest`）
//!
//! [`jump_visible`] は `stuck == Free` のときのみ可視とする（`data-has-new`
//! は独立したスタイルフックであり可視条件に含めない）。新着が無くても
//! 上へスクロールした利用者が最下部へ戻る手段を持つべきという判断で
//! あり、shadcn/ui の ScrollToBottom ボタンが「最下部にいない」だけで
//! 現れる挙動に揃える。
//!
//! # プログラム的スクロールは常に即時（`ScrollBehavior::Instant`）
//!
//! smooth スクロールだと中間 `scroll` イベントで `free` へ誤遷移し
//! `data-has-new` の誤検知を招くため、本モジュールが行うプログラム的
//! スクロールは常に即時とする。smooth なジャンプ演出はスコープ外。
//!
//! # 構造フォールバック（`rerender_subtree`）で viewport が丸ごと
//! 差し替えられた場合の方針
//!
//! 旧 viewport の `scrollTop` は復元しない。新 viewport は `scrollTop = 0`
//! で現れ、アプリの `view()` が出力した `data-stuck`（既定 `Bottom`）に
//! 従って初期同期される。サポートされる経路は「`content` 配下を keyed
//! list（`data-keyed-list`）で差分更新し、ストリーミング本文を
//! `bind_text` 束縛点で更新する構成」である。
//!
//! # 既知の限界
//!
//! - 同一 `MutationObserver` コールバック内で上方向挿入と下方向追記が
//!   同時に起きた場合、`scrollHeight` 差分での補正は追記分だけ過補正に
//!   なる。
//! - 画像ロード等、DOM 変異を伴わない高さ変化（`ResizeObserver` 相当）は
//!   検知しない。
//! - 配線後の再描画で初めて出現する message-scroller への遅延配線は
//!   行わない（[`wiring::wire_message_scroller_events`] の搭載判定ゲート、
//!   `sidebar`/`splitter` と同じトレードオフ）。
//!
//! # セキュリティ不変条件（REQ-1・`security.md` A03）
//!
//! HTML 文字列の組み立て・`set_inner_html` は一切行わない。書き込む
//! 属性名（`data-stuck`/`data-has-new`/`data-visible`/`hidden`）と値は
//! すべて `&'static str` リテラル。CSS へ流すのは `overflow-anchor: none`
//! の固定リテラルのみ。`querySelector` セレクタは [`SCOPE`]/`PART_*`
//! リテラルのみから組み立て、DOM 由来の値をセレクタへ補間しない。通知
//! payload（root の `id`）は文字列のまま [`ACTION_LOAD_MORE`] とともに
//! `on_action` へ渡すだけで、HTML/セレクタとして解釈しない。

/// headless-ui の anatomy scope（`data-scope="message-scroller"`）。
pub const SCOPE: &str = "message-scroller";

/// anatomy パート名（`data-part` 値）。
pub const PART_ROOT: &str = "root";
pub const PART_VIEWPORT: &str = "viewport";
pub const PART_CONTENT: &str = "content";
pub const PART_ANCHOR: &str = "anchor";
pub const PART_JUMP_TO_LATEST: &str = "jump-to-latest";
pub const PART_LOAD_MORE: &str = "load-more";

/// `load-more` クリックをアプリへ通知するアクション名（`"questionnaire:*"`/
/// `"timer:*"` と同じ名前空間付き語彙）。
pub const ACTION_LOAD_MORE: &str = "message-scroller:load-more";

/// 最下部判定のしきい値（px）。高 DPI 環境での `scrollTop` 端数誤差
/// （1〜2px）を吸収する余裕を持たせた値。
pub const STICK_THRESHOLD_PX: i32 = 8;

/// `data-stuck` の値（`fandhe_frontend_headless_ui::message_scroller::
/// MessageScrollerStuck` と同じ語彙を配線層向けに再定義する。headless-ui
/// 側の enum をそのまま再利用してもよいが、本モジュールは wiring 内で
/// DOM の文字列属性から都度導出するため、独立した薄い型として持つ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageScrollerStuck {
    /// 利用者が最下部に張り付いている（新着到着時は自動追従する）。
    Bottom,
    /// 利用者が手動スクロールで最下部から離脱した。
    Free,
}

impl MessageScrollerStuck {
    /// `data-stuck` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bottom => "bottom",
            Self::Free => "free",
        }
    }
}

/// `viewport` の `scrollTop`/`scrollHeight`/`clientHeight` から「最下部に
/// 居るか」を判定する。`scroll_height - client_height - scroll_top` が
/// `threshold` 以下なら最下部と見なす。負値は 0 として扱う（飽和演算、
/// `panic` しない）。
#[must_use]
pub fn is_at_bottom(
    scroll_top: i32,
    scroll_height: i32,
    client_height: i32,
    threshold: i32,
) -> bool {
    let max_scroll = scroll_height.saturating_sub(client_height).max(0);
    let remaining = max_scroll.saturating_sub(scroll_top).max(0);
    remaining <= threshold.max(0)
}

/// `root` の `data-stuck` 属性値から [`MessageScrollerStuck`] を導出する。
///
/// `"bottom"`/`"free"` 以外（欠落・改ざん）は [`MessageScrollerStuck::Free`]
/// を返す（不明な状態で利用者のスクロールを勝手に奪わない fail-closed
/// 方針）。
#[must_use]
pub fn stuck_from_attr(value: Option<&str>) -> MessageScrollerStuck {
    match value {
        Some("bottom") => MessageScrollerStuck::Bottom,
        _ => MessageScrollerStuck::Free,
    }
}

/// `jump-to-latest` の可視判定（モジュール doc「可視判定の確定」参照）。
/// `stuck == Free` のときのみ可視。
#[must_use]
pub fn jump_visible(stuck: MessageScrollerStuck) -> bool {
    stuck == MessageScrollerStuck::Free
}

/// `content` に起きた DOM 変異の分類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentChange {
    /// 可視領域より上（先頭側）への要素挿入（履歴読み込み相当）。
    Prepend,
    /// それ以外の高さ増加（末尾への追記・ストリーミング更新）。
    Grow,
    /// 高さが不変または減少（無視してよい変異）。
    None,
}

/// `content` の高さ変化と、最初に追加された要素の位置から
/// [`ContentChange`] を判定する。
///
/// - `new_scroll_height <= prev_scroll_height` は [`ContentChange::None`]。
/// - `first_added_top`（今回のバッチで最初に追加された要素ノードの
///   `getBoundingClientRect().top`）が viewport 上端
///   （`viewport_top + STICK_THRESHOLD_PX` 以下、`content` の padding
///   により僅かに下にずれても `Grow` へ誤分類しないための許容差込み）
///   なら [`ContentChange::Prepend`]。
/// - `first_added_top` が `None`（`characterData` レコードのみで要素
///   ノードの追加が無い場合を含む）を含め、それ以外は
///   [`ContentChange::Grow`]。
#[must_use]
pub fn classify_change(
    first_added_top: Option<f64>,
    viewport_top: f64,
    prev_scroll_height: i32,
    new_scroll_height: i32,
) -> ContentChange {
    if new_scroll_height <= prev_scroll_height {
        return ContentChange::None;
    }
    match first_added_top {
        Some(top) if top <= viewport_top + f64::from(STICK_THRESHOLD_PX) => ContentChange::Prepend,
        _ => ContentChange::Grow,
    }
}

/// `Prepend` 判定時に補正すべき新しい `scrollTop` を返す
/// （`prev_scroll_top + (new - prev)`、負値は 0 にクランプ）。
#[must_use]
pub fn corrected_scroll_top(
    prev_scroll_top: i32,
    prev_scroll_height: i32,
    new_scroll_height: i32,
) -> i32 {
    let delta = new_scroll_height.saturating_sub(prev_scroll_height);
    prev_scroll_top.saturating_add(delta).max(0)
}

/// [`classify_change`] の結果から実行すべき処理を決定表として返す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChangePlan {
    /// `Prepend` の `scrollTop` 補正を行う。
    pub correct_prepend: bool,
    /// 最下部へスクロールする（補正後、または `Grow` かつ `Bottom`）。
    pub scroll_to_bottom: bool,
    /// `data-has-new` を付与する（`Grow` かつ `Free`）。
    pub mark_has_new: bool,
}

/// `stuck`（変異直前の状態）と `change`（[`classify_change`] の結果）から
/// [`ChangePlan`] を決定する（モジュール冒頭の決定表を実装する）。
#[must_use]
pub fn plan_after_change(stuck: MessageScrollerStuck, change: ContentChange) -> ChangePlan {
    match (stuck, change) {
        (MessageScrollerStuck::Bottom, ContentChange::Prepend) => ChangePlan {
            correct_prepend: true,
            scroll_to_bottom: true,
            mark_has_new: false,
        },
        (MessageScrollerStuck::Free, ContentChange::Prepend) => ChangePlan {
            correct_prepend: true,
            scroll_to_bottom: false,
            mark_has_new: false,
        },
        (MessageScrollerStuck::Bottom, ContentChange::Grow) => ChangePlan {
            correct_prepend: false,
            scroll_to_bottom: true,
            mark_has_new: false,
        },
        (MessageScrollerStuck::Free, ContentChange::Grow) => ChangePlan {
            correct_prepend: false,
            scroll_to_bottom: false,
            mark_has_new: true,
        },
        (_, ContentChange::None) => ChangePlan::default(),
    }
}

/// `load-more` 通知の payload を組み立てる（インスタンス root の `id`
/// 属性値。未設定時は空文字列。questionnaire の `"{step}|{id}"` と異なり
/// step を持たないため `id` のみで区切り文字は使わない）。
#[must_use]
pub fn encode_notification_payload(instance_id: &str) -> String {
    instance_id.to_string()
}

/// [`encode_notification_payload`] の逆変換（テスト・呼び出し側の往復
/// 確認用）。
#[must_use]
pub fn decode_notification_payload(payload: &str) -> &str {
    payload
}

// ---------------------------------------------------------------------
// 配線層: web-sys/js-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`questionnaire.rs`/`sidebar.rs`/`content_height.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        classify_change, corrected_scroll_top, encode_notification_payload, is_at_bottom,
        jump_visible, plan_after_change, stuck_from_attr, ACTION_LOAD_MORE, PART_CONTENT,
        PART_JUMP_TO_LATEST, PART_LOAD_MORE, PART_ROOT, PART_VIEWPORT, SCOPE, STICK_THRESHOLD_PX,
    };
    use crate::events::ActionRef;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{
        Element, Event, MutationObserver, MutationObserverInit, MutationRecord, ScrollBehavior,
        ScrollToOptions,
    };

    /// `[data-scope="message-scroller"][data-part="<part>"]` セレクタを
    /// 組み立てる（[`SCOPE`]/引数の `part` はいずれも `&'static str`
    /// リテラルのみから呼ばれる契約、モジュール doc「セキュリティ不変
    /// 条件」参照）。
    fn part_selector(part: &str) -> String {
        format!(r#"[data-scope="{SCOPE}"][data-part="{part}"]"#)
    }

    /// `start` から `root`（含む）まで祖先方向へ辿り、`data-scope`/
    /// `data-part` が指定値と一致する最初の要素を返す
    /// （`questionnaire::wiring::closest_matching` と同型）。
    fn closest_matching(root: &Element, start: &Element, part: &str) -> Option<Element> {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if !root.contains(Some(&element)) {
                break;
            }
            if element.get_attribute("data-scope").as_deref() == Some(SCOPE)
                && element.get_attribute("data-part").as_deref() == Some(part)
            {
                return Some(element);
            }
            if element == *root {
                break;
            }
            current = element.parent_element();
        }
        None
    }

    /// `start` から `boundary`（含む）まで祖先方向を辿り、`data-disabled`
    /// またはネイティブ `disabled`/`data-loading` を持つ要素が 1 つでも
    /// あれば `true`（`questionnaire::wiring::has_disabled_ancestor` と
    /// 同型に `data-loading` を追加した版）。
    fn has_disabled_or_loading_ancestor(boundary: &Element, start: &Element) -> bool {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if element.has_attribute("data-disabled")
                || element.has_attribute("disabled")
                || element.has_attribute("data-loading")
            {
                return true;
            }
            if !boundary.contains(Some(&element)) || element == *boundary {
                break;
            }
            current = element.parent_element();
        }
        false
    }

    /// `start` から `boundary`（含む）まで祖先方向を辿り、`data-disabled`
    /// またはネイティブ `disabled` を持つ要素が 1 つでもあれば `true`
    /// （`jump-to-latest` 用。`data-loading` は無関係のため含めない）。
    fn has_disabled_ancestor(boundary: &Element, start: &Element) -> bool {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if element.has_attribute("data-disabled") || element.has_attribute("disabled") {
                return true;
            }
            if !boundary.contains(Some(&element)) || element == *boundary {
                break;
            }
            current = element.parent_element();
        }
        false
    }

    /// `start` から `boundary`（含む）まで祖先方向を辿り、`data-action`
    /// を持つ要素が 1 つでもあれば `true`（アプリが手動配線を明示的に
    /// 選んだものとして自動通知を抑止する、`questionnaire::wiring::
    /// resolve_trigger` の `has_explicit_action` 累積判定と同じ意図）。
    fn has_explicit_action(boundary: &Element, start: &Element) -> bool {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if element.has_attribute("data-action") {
                return true;
            }
            if !boundary.contains(Some(&element)) || element == *boundary {
                break;
            }
            current = element.parent_element();
        }
        false
    }

    /// `element` の `data-stuck` を `stuck` へ書き込み、連動する
    /// `data-has-new`（`Bottom` なら除去）・`jump-to-latest` の
    /// `data-visible`/`hidden`（[`jump_visible`] に同期）を反映する。
    fn write_stuck(instance_root: &Element, stuck: super::MessageScrollerStuck) {
        let _ = instance_root.set_attribute("data-stuck", stuck.as_str());
        if stuck == super::MessageScrollerStuck::Bottom {
            let _ = instance_root.remove_attribute("data-has-new");
        }
        sync_jump_to_latest_visibility(instance_root, stuck);
    }

    /// `instance_root` 配下の `jump-to-latest` 要素の可視状態
    /// （`data-visible`/`hidden` の排他契約、headless-ui
    /// `jump_to_latest` の排他契約と同型）を [`jump_visible`] に同期する。
    fn sync_jump_to_latest_visibility(instance_root: &Element, stuck: super::MessageScrollerStuck) {
        let visible = jump_visible(stuck);
        let selector = part_selector(PART_JUMP_TO_LATEST);
        let Ok(nodes) = instance_root.query_selector_all(&selector) else {
            return;
        };
        for i in 0..nodes.length() {
            let Some(node) = nodes.get(i) else { continue };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            if closest_matching(instance_root, &element, PART_ROOT).as_ref() != Some(instance_root)
            {
                continue;
            }
            if visible {
                let _ = element.set_attribute("data-visible", "");
                let _ = element.remove_attribute("hidden");
            } else {
                let _ = element.set_attribute("hidden", "");
                let _ = element.remove_attribute("data-visible");
            }
        }
    }

    /// `viewport` を即時（`ScrollBehavior::Instant`）で最下部へスクロール
    /// する（モジュール doc「プログラム的スクロールは常に即時」参照）。
    fn scroll_viewport_to_bottom(viewport: &Element) {
        let options = ScrollToOptions::new();
        options.set_top(f64::from(viewport.scroll_height()));
        options.set_behavior(ScrollBehavior::Instant);
        viewport.scroll_to_with_scroll_to_options(&options);
    }

    /// インスタンスごとのスクロール計測スナップショット
    /// （`last_scroll_top`/`last_scroll_height`）。DOM から読めない
    /// 「前回の計測値」のみを保持し、`data-*` 語彙は DOM を唯一の真とする
    /// （`sidebar`/`questionnaire` と同じ原則）。
    struct InstanceSnapshot {
        viewport: Element,
        last_scroll_top: i32,
        last_scroll_height: i32,
    }

    type SnapshotList = Rc<RefCell<Vec<InstanceSnapshot>>>;

    /// `viewport` に一致するスナップショットのインデックスを線形探索する
    /// （JS 同一性 `==` で比較。ページ内のスクローラー数は高々数個のため
    /// 線形探索で十分）。
    fn find_snapshot_index(snapshots: &SnapshotList, viewport: &Element) -> Option<usize> {
        snapshots
            .borrow()
            .iter()
            .position(|snapshot| &snapshot.viewport == viewport)
    }

    /// `root` 配下から外れた viewport のスナップショットを prune する
    /// （メモリ有界化、`sidebar` の同種パターンに倣う）。
    fn prune_snapshots(snapshots: &SnapshotList, root: &Element) {
        snapshots
            .borrow_mut()
            .retain(|snapshot| root.contains(Some(&snapshot.viewport)));
    }

    /// `instance_root`（`data-scope="message-scroller"][data-part="root"]`）
    /// 配下の viewport 1 件に対する初期同期（配線時、
    /// `sidebar::apply_mobile_state` の初回適用と同型）:
    ///
    /// 1. viewport に CSSOM 経由で `overflow-anchor: none` を設定する
    ///    （ネイティブ scroll anchoring による二重補正を防ぐ）。
    /// 2. `data-stuck` が `Bottom` なら即時で最下部へスクロールする
    ///    （`Free`/不明なら触らない）。
    /// 3. `jump-to-latest` の可視状態を同期する。
    /// 4. スナップショットへ登録する。
    fn initial_sync_instance(
        instance_root: &Element,
        viewport: &Element,
        snapshots: &SnapshotList,
    ) {
        if let Ok(html) = viewport.clone().dyn_into::<web_sys::HtmlElement>() {
            let _ = html.style().set_property("overflow-anchor", "none");
        }

        let stuck = stuck_from_attr(instance_root.get_attribute("data-stuck").as_deref());
        if stuck == super::MessageScrollerStuck::Bottom {
            scroll_viewport_to_bottom(viewport);
        }
        sync_jump_to_latest_visibility(instance_root, stuck);

        let scroll_top = viewport.scroll_top();
        let scroll_height = viewport.scroll_height();
        let index = find_snapshot_index(snapshots, viewport);
        let entry = InstanceSnapshot {
            viewport: viewport.clone(),
            last_scroll_top: scroll_top,
            last_scroll_height: scroll_height,
        };
        let mut list = snapshots.borrow_mut();
        match index {
            Some(i) => list[i] = entry,
            None => list.push(entry),
        }
    }

    /// `root` 配下の全 message-scroller インスタンスへ初期同期を適用する。
    fn initial_sync_all(root: &Element, snapshots: &SnapshotList) {
        let selector = part_selector(PART_VIEWPORT);
        let Ok(nodes) = root.query_selector_all(&selector) else {
            return;
        };
        for i in 0..nodes.length() {
            let Some(node) = nodes.get(i) else { continue };
            let Ok(viewport) = node.dyn_into::<Element>() else {
                continue;
            };
            let Some(instance_root) = closest_matching(root, &viewport, PART_ROOT) else {
                continue;
            };
            initial_sync_instance(&instance_root, &viewport, snapshots);
        }
        prune_snapshots(snapshots, root);
    }

    /// `scroll` イベント 1 件を処理する（capture フェーズで委譲登録、
    /// モジュール doc・`handle_scroll` 呼び出し元 doc 参照）。
    fn handle_scroll(root: &Element, event: &Event, snapshots: &SnapshotList) {
        let Some(target) = event.target() else {
            return;
        };
        let Some(viewport) = target.dyn_ref::<Element>() else {
            return;
        };
        if !root.contains(Some(viewport)) {
            return;
        }
        if viewport.get_attribute("data-part").as_deref() != Some(PART_VIEWPORT)
            || viewport.get_attribute("data-scope").as_deref() != Some(SCOPE)
        {
            return;
        }
        let Some(instance_root) = closest_matching(root, viewport, PART_ROOT) else {
            return;
        };

        let scroll_top = viewport.scroll_top();
        let scroll_height = viewport.scroll_height();
        let client_height = viewport.client_height();
        let at_bottom = is_at_bottom(scroll_top, scroll_height, client_height, STICK_THRESHOLD_PX);
        let stuck = if at_bottom {
            super::MessageScrollerStuck::Bottom
        } else {
            super::MessageScrollerStuck::Free
        };
        write_stuck(&instance_root, stuck);

        let index = find_snapshot_index(snapshots, viewport);
        let entry = InstanceSnapshot {
            viewport: viewport.clone(),
            last_scroll_top: scroll_top,
            last_scroll_height: scroll_height,
        };
        let mut list = snapshots.borrow_mut();
        match index {
            Some(i) => list[i] = entry,
            None => list.push(entry),
        }
    }

    /// `click` イベント 1 件を処理する（`jump-to-latest`/`load-more` の
    /// バブル委譲、`root` へ 1 個登録）。
    fn handle_click(
        root: &Element,
        event: &Event,
        on_action: &Rc<RefCell<impl FnMut(ActionRef) + 'static>>,
        snapshots: &SnapshotList,
    ) {
        let Some(target) = event.target() else {
            return;
        };
        let target_element: Element = match target.dyn_ref::<Element>() {
            Some(element) => element.clone(),
            None => {
                let Some(node) = target.dyn_ref::<web_sys::Node>() else {
                    return;
                };
                let Some(parent) = node.parent_element() else {
                    return;
                };
                parent
            }
        };

        if let Some(jump) = closest_matching(root, &target_element, PART_JUMP_TO_LATEST) {
            if has_disabled_ancestor(&jump, &target_element) {
                return;
            }
            let Some(instance_root) = closest_matching(root, &jump, PART_ROOT) else {
                return;
            };
            let selector = part_selector(PART_VIEWPORT);
            let Ok(nodes) = instance_root.query_selector_all(&selector) else {
                return;
            };
            for i in 0..nodes.length() {
                let Some(node) = nodes.get(i) else { continue };
                let Ok(viewport) = node.dyn_into::<Element>() else {
                    continue;
                };
                if closest_matching(&instance_root, &viewport, PART_ROOT).as_ref()
                    != Some(&instance_root)
                {
                    continue;
                }
                scroll_viewport_to_bottom(&viewport);
                let index = find_snapshot_index(snapshots, &viewport);
                let entry = InstanceSnapshot {
                    viewport: viewport.clone(),
                    last_scroll_top: viewport.scroll_top(),
                    last_scroll_height: viewport.scroll_height(),
                };
                let mut list = snapshots.borrow_mut();
                match index {
                    Some(i) => list[i] = entry,
                    None => list.push(entry),
                }
            }
            write_stuck(&instance_root, super::MessageScrollerStuck::Bottom);
            event.stop_propagation();
            return;
        }

        if let Some(load_more) = closest_matching(root, &target_element, PART_LOAD_MORE) {
            if has_disabled_or_loading_ancestor(&load_more, &target_element) {
                return;
            }
            if has_explicit_action(&load_more, &target_element) {
                return;
            }
            let Some(instance_root) = closest_matching(root, &load_more, PART_ROOT) else {
                return;
            };
            let instance_id = instance_root.get_attribute("id").unwrap_or_default();
            let payload = encode_notification_payload(&instance_id);
            if let Ok(mut cb) = on_action.try_borrow_mut() {
                (cb)(ActionRef {
                    action: ACTION_LOAD_MORE.to_string(),
                    payload,
                });
            }
            event.stop_propagation();
        }
    }

    /// `MutationRecord::target()` から、その変異の影響を受けた
    /// message-scroller の `content`/`root`/`viewport` を解決する。
    /// `characterData` レコードの `target` は `Text` ノードであり
    /// `Element` へダウンキャストできないため `parent_element()` で
    /// 要素へ解決してから `closest` する（`questionnaire::wiring` の
    /// click target 解決と同じパターン）。
    fn resolve_instance_from_record(
        root: &Element,
        record: &MutationRecord,
    ) -> Option<(Element, Element)> {
        let target = record.target()?;
        let start_element = if let Some(element) = target.dyn_ref::<Element>() {
            element.clone()
        } else {
            let node = target.dyn_ref::<web_sys::Node>()?;
            node.parent_element()?
        };
        let content = closest_matching(root, &start_element, PART_CONTENT)?;
        let instance_root = closest_matching(root, &content, PART_ROOT)?;
        let viewport_selector = part_selector(PART_VIEWPORT);
        let viewport = instance_root
            .query_selector(&viewport_selector)
            .ok()
            .flatten()?;
        Some((instance_root, viewport))
    }

    /// 今回のバッチで最初に追加された要素ノード（`record.added_nodes()`
    /// の先頭が `Element` であるもの）の `getBoundingClientRect().top` を
    /// 返す。複数レコードにまたがる場合は最初に見つかったものを使う。
    fn first_added_element_top(records: &[MutationRecord], content: &Element) -> Option<f64> {
        for record in records {
            let Some(target) = record.target() else {
                continue;
            };
            let Some(target_element) = target.dyn_ref::<Element>() else {
                continue;
            };
            if target_element != content {
                continue;
            }
            let added = record.added_nodes();
            for i in 0..added.length() {
                let Some(node) = added.get(i) else { continue };
                if let Some(element) = node.dyn_ref::<Element>() {
                    return Some(element.get_bounding_client_rect().top());
                }
            }
        }
        None
    }

    /// `MutationObserver` コールバック本体。レコードをインスタンスごとに
    /// グループ化し、viewport ごとに 1 回だけ計測・補正・状態書き戻しを
    /// 行う（モジュール doc「既知の限界」参照）。
    fn handle_mutations(root: &Element, records: &js_sys::Array, snapshots: &SnapshotList) {
        let mut record_list: Vec<MutationRecord> = Vec::with_capacity(records.length() as usize);
        for record in records.iter() {
            if let Ok(record) = record.dyn_into::<MutationRecord>() {
                record_list.push(record);
            }
        }

        // 新規インスタンス発見（構造フォールバック再描画で viewport が
        // 作り直された場合を含む）にも対応するため、レコード解決に
        // 先立って現在の viewport 集合へ未登録分を初期同期しておく
        // （二重初期化は `initial_sync_instance` が index 更新で吸収する）。
        initial_sync_all(root, snapshots);

        let mut seen_viewports: Vec<Element> = Vec::new();
        for record in &record_list {
            let Some((instance_root, viewport)) = resolve_instance_from_record(root, record) else {
                continue;
            };
            if seen_viewports.contains(&viewport) {
                continue;
            }
            seen_viewports.push(viewport.clone());

            let Some(index) = find_snapshot_index(snapshots, &viewport) else {
                continue;
            };
            let (prev_scroll_top, prev_scroll_height) = {
                let list = snapshots.borrow();
                (list[index].last_scroll_top, list[index].last_scroll_height)
            };
            let new_scroll_height = viewport.scroll_height();
            let content_selector = part_selector(PART_CONTENT);
            let Some(content) = instance_root
                .query_selector(&content_selector)
                .ok()
                .flatten()
            else {
                continue;
            };
            let first_top = first_added_element_top(&record_list, &content);
            let viewport_top = viewport.get_bounding_client_rect().top();
            let change = classify_change(
                first_top,
                viewport_top,
                prev_scroll_height,
                new_scroll_height,
            );

            let stuck = stuck_from_attr(instance_root.get_attribute("data-stuck").as_deref());
            let plan = plan_after_change(stuck, change);

            if plan.correct_prepend {
                let new_top =
                    corrected_scroll_top(prev_scroll_top, prev_scroll_height, new_scroll_height);
                viewport.set_scroll_top(new_top);
            }
            if plan.scroll_to_bottom {
                scroll_viewport_to_bottom(&viewport);
            }
            if plan.mark_has_new {
                let _ = instance_root.set_attribute("data-has-new", "");
                sync_jump_to_latest_visibility(&instance_root, stuck);
            }

            let mut list = snapshots.borrow_mut();
            list[index] = InstanceSnapshot {
                viewport: viewport.clone(),
                last_scroll_top: viewport.scroll_top(),
                last_scroll_height: viewport.scroll_height(),
            };
        }

        prune_snapshots(snapshots, root);
    }

    /// `root` 配下の Message Scroller へ最下部追従・新着検知・履歴読み込み
    /// 時のスクロール位置維持を配線する。
    ///
    /// `root` 配下（`root` 自身を含む）に
    /// `[data-scope="message-scroller"][data-part="root"]` が 1 件も
    /// 無ければリスナーを 1 つも登録せず `Ok(())` を返す（非搭載アプリへの
    /// 副作用なし契約、`sidebar`/`splitter` と同型。マウント後に動的挿入
    /// された message-scroller は配線対象外というトレードオフも同じ）。
    ///
    /// `Closure::forget` は定数 3 個（`scroll` capture リスナー 1・
    /// `click` リスナー 1・`MutationObserver` コールバック 1）に限定する。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback_and_bool`/
    /// `MutationObserver::observe_with_options` の失敗を伝播する。
    pub fn wire_message_scroller_events(
        root: Element,
        on_action: impl FnMut(ActionRef) + 'static,
    ) -> Result<(), JsValue> {
        let root_selector = part_selector(PART_ROOT);
        let has_instance = root.get_attribute("data-scope").as_deref() == Some(SCOPE)
            && root.get_attribute("data-part").as_deref() == Some(PART_ROOT)
            || root.query_selector(&root_selector).ok().flatten().is_some();
        if !has_instance {
            return Ok(());
        }

        let on_action = Rc::new(RefCell::new(on_action));
        let snapshots: SnapshotList = Rc::new(RefCell::new(Vec::new()));

        initial_sync_all(&root, &snapshots);

        // `scroll` はバブルしないため capture フェーズで `root` へ委譲
        // 登録する（viewport がその後の再描画で差し替わっても生存する）。
        let scroll_root = root.clone();
        let scroll_snapshots = snapshots.clone();
        let scroll_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_scroll(&scroll_root, &event, &scroll_snapshots);
        });
        root.add_event_listener_with_callback_and_bool(
            "scroll",
            scroll_closure.as_ref().unchecked_ref(),
            true,
        )?;
        scroll_closure.forget();

        let click_root = root.clone();
        let click_snapshots = snapshots.clone();
        let click_on_action = on_action.clone();
        let click_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_click(&click_root, &event, &click_on_action, &click_snapshots);
        });
        root.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())?;
        click_closure.forget();

        let observer_root = root.clone();
        let observer_snapshots = snapshots;
        let observer_callback = Closure::<dyn FnMut(js_sys::Array, MutationObserver)>::new(
            move |records: js_sys::Array, _observer: MutationObserver| {
                handle_mutations(&observer_root, &records, &observer_snapshots);
            },
        );
        let observer = MutationObserver::new(observer_callback.as_ref().unchecked_ref())?;
        let init = MutationObserverInit::new();
        init.set_child_list(true);
        init.set_subtree(true);
        init.set_character_data(true);
        observer.observe_with_options(&root, &init)?;
        observer_callback.forget();

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_message_scroller_events;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_at_bottom_exact_match() {
        assert!(is_at_bottom(100, 200, 100, 8));
    }

    #[test]
    fn is_at_bottom_within_threshold() {
        assert!(is_at_bottom(94, 200, 100, 8));
        assert!(is_at_bottom(92, 200, 100, 8));
    }

    #[test]
    fn is_at_bottom_outside_threshold() {
        assert!(!is_at_bottom(50, 200, 100, 8));
    }

    #[test]
    fn is_at_bottom_negative_values_saturate() {
        assert!(is_at_bottom(0, 0, 0, 8));
        assert!(is_at_bottom(0, 50, 100, 8));
        assert!(!is_at_bottom(-100, 500, 100, 0));
    }

    #[test]
    fn is_at_bottom_zero_threshold_boundary() {
        assert!(is_at_bottom(100, 200, 100, 0));
        assert!(!is_at_bottom(99, 200, 100, 0));
    }

    #[test]
    fn stuck_from_attr_recognizes_bottom_and_free() {
        assert_eq!(
            stuck_from_attr(Some("bottom")),
            MessageScrollerStuck::Bottom
        );
        assert_eq!(stuck_from_attr(Some("free")), MessageScrollerStuck::Free);
    }

    #[test]
    fn stuck_from_attr_unknown_or_missing_is_free() {
        assert_eq!(stuck_from_attr(None), MessageScrollerStuck::Free);
        assert_eq!(stuck_from_attr(Some("evil")), MessageScrollerStuck::Free);
        assert_eq!(stuck_from_attr(Some("")), MessageScrollerStuck::Free);
    }

    #[test]
    fn jump_visible_matches_free_only() {
        assert!(jump_visible(MessageScrollerStuck::Free));
        assert!(!jump_visible(MessageScrollerStuck::Bottom));
    }

    #[test]
    fn classify_change_none_when_height_not_increased() {
        assert_eq!(classify_change(None, 0.0, 200, 200), ContentChange::None);
        assert_eq!(classify_change(None, 0.0, 200, 150), ContentChange::None);
    }

    #[test]
    fn classify_change_prepend_when_top_at_or_above_viewport() {
        assert_eq!(
            classify_change(Some(0.0), 0.0, 200, 280),
            ContentChange::Prepend
        );
        // 許容差込み: viewport_top + STICK_THRESHOLD_PX 以下も Prepend。
        assert_eq!(
            classify_change(Some(6.0), 0.0, 200, 280),
            ContentChange::Prepend
        );
    }

    #[test]
    fn classify_change_grow_when_top_below_viewport_or_missing() {
        assert_eq!(
            classify_change(Some(50.0), 0.0, 200, 280),
            ContentChange::Grow
        );
        assert_eq!(classify_change(None, 0.0, 200, 280), ContentChange::Grow);
    }

    #[test]
    fn corrected_scroll_top_adds_delta() {
        assert_eq!(corrected_scroll_top(50, 200, 280), 130);
    }

    #[test]
    fn corrected_scroll_top_clamps_to_zero() {
        assert_eq!(corrected_scroll_top(0, 200, 150), 0);
    }

    #[test]
    fn plan_after_change_decision_table() {
        assert_eq!(
            plan_after_change(MessageScrollerStuck::Bottom, ContentChange::Prepend),
            ChangePlan {
                correct_prepend: true,
                scroll_to_bottom: true,
                mark_has_new: false,
            }
        );
        assert_eq!(
            plan_after_change(MessageScrollerStuck::Free, ContentChange::Prepend),
            ChangePlan {
                correct_prepend: true,
                scroll_to_bottom: false,
                mark_has_new: false,
            }
        );
        assert_eq!(
            plan_after_change(MessageScrollerStuck::Bottom, ContentChange::Grow),
            ChangePlan {
                correct_prepend: false,
                scroll_to_bottom: true,
                mark_has_new: false,
            }
        );
        assert_eq!(
            plan_after_change(MessageScrollerStuck::Free, ContentChange::Grow),
            ChangePlan {
                correct_prepend: false,
                scroll_to_bottom: false,
                mark_has_new: true,
            }
        );
        assert_eq!(
            plan_after_change(MessageScrollerStuck::Bottom, ContentChange::None),
            ChangePlan::default()
        );
        assert_eq!(
            plan_after_change(MessageScrollerStuck::Free, ContentChange::None),
            ChangePlan::default()
        );
    }

    #[test]
    fn notification_payload_round_trips() {
        assert_eq!(
            decode_notification_payload(&encode_notification_payload("chat-1")),
            "chat-1"
        );
        assert_eq!(
            decode_notification_payload(&encode_notification_payload("")),
            ""
        );
    }
}
