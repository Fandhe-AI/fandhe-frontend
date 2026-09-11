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
//! # プログラム的スクロールは常に即時（インライン `scroll-behavior: auto`）
//!
//! smooth スクロールだと中間 `scroll` イベントで `free` へ誤遷移し
//! `data-has-new` の誤検知を招くため、本モジュールが行うプログラム的
//! スクロールは常に即時とする。配線時（[`wiring::initial_sync_instance`]）
//! に `viewport` へインライン `scroll-behavior: auto` を固定設定し
//! （`overflow-anchor: none` と同じ箇所）、アプリ側 CSS が
//! `scroll-behavior: smooth` を指定していてもインラインスタイル
//! （最高詳細度）で上書きすることで、[`wiring::scroll_viewport_to_top`]
//! の `Element::set_scroll_top` を常に即時にする（`ScrollToOptions`/
//! `ScrollBehavior` 型を呼び出しごとに経由する構成から、配線時 1 回の
//! CSS 固定へ変更。バンドルサイズ抑制、レビュー指摘 #2122: REQ-11 gzip
//! 上限超過の是正）。smooth なジャンプ演出はスコープ外。
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

/// `content` の高さ変化と、最初に追加されたノードの構造的な挿入位置から
/// [`ContentChange`] を判定する。
///
/// - `new_scroll_height <= prev_scroll_height` は [`ContentChange::None`]。
/// - `first_added_is_prepend`（今回のバッチで最初に見つかった要素追加を
///   伴う `MutationRecord` について、追加ノード群がその親の先頭かつ
///   既存ノードの前（`previousSibling` が無く `nextSibling` がある
///   位置）へ挿入されたか）が `true` なら [`ContentChange::Prepend`]。
/// - それ以外（`characterData` レコードのみで要素ノードの追加が無い
///   場合、および空リスト・空の `data-bind-list` への初回追加
///   〔`previousSibling`/`nextSibling` がともに無い〕を含む）は
///   [`ContentChange::Grow`]。
///
/// 判定は `MutationRecord.previousSibling`/`nextSibling` という DOM
/// 構造情報のみで行い、viewport のジオメトリ
/// （`getBoundingClientRect`）には依存しない（レビュー指摘 #2122:
/// `content` に上部 padding があると Free 状態で scrollTop=0 でも
/// 先頭挿入した要素が viewport 上端より下に位置し `Grow` へ誤分類され、
/// scrollTop 未補正・新着誤通知が起きる旧実装の問題を回避する）。
/// `nextSibling` の要求は、空リストへの初回追加を先頭挿入と誤判定して
/// Free 状態で位置を動かし `data-has-new` を付与しない不具合を防ぐ
/// （codex-review 指摘 #2122 line 789）。
#[must_use]
pub fn classify_change(
    first_added_is_prepend: bool,
    prev_scroll_height: i32,
    new_scroll_height: i32,
) -> ContentChange {
    if new_scroll_height <= prev_scroll_height {
        return ContentChange::None;
    }
    if first_added_is_prepend {
        ContentChange::Prepend
    } else {
        ContentChange::Grow
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
    use super::ContentChange;
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
    use web_sys::{Element, Event, MutationObserver, MutationObserverInit, MutationRecord};

    /// `[data-part="root"]` セレクタの固定リテラル（`sidebar.rs` の
    /// `ROOT_SELECTOR` と同じ方針）。呼び出し先パートは本モジュール内
    /// すべて `&'static str` リテラルで決まるため、`format!` によるセレクタ
    /// 組み立て（実行時コスト・コードサイズ増）を避け、パートごとに固定
    /// リテラル定数を用意する（バンドルサイズ抑制の一部、レビュー指摘
    /// #2122: REQ-11 gzip 上限超過対応の一環。[`scoped_parts`] も本方針に
    /// 合わせ、呼び出し元が渡す `part` 文字列ではなく事前組み立て済みの
    /// `*_SELECTOR` 定数を受け取る形へ変更した。この変更単独では上限
    /// 超過を解消しない — 残るギャップの扱いはユーザー判断待ち、PR 本文
    /// 参照）。
    const ROOT_SELECTOR: &str = "[data-scope=\"message-scroller\"][data-part=\"root\"]";
    /// `[data-part="viewport"]` セレクタの固定リテラル。
    const VIEWPORT_SELECTOR: &str = "[data-scope=\"message-scroller\"][data-part=\"viewport\"]";
    /// `[data-part="content"]` セレクタの固定リテラル。
    const CONTENT_SELECTOR: &str = "[data-scope=\"message-scroller\"][data-part=\"content\"]";
    /// `[data-part="jump-to-latest"]` セレクタの固定リテラル。
    const JUMP_TO_LATEST_SELECTOR: &str =
        "[data-scope=\"message-scroller\"][data-part=\"jump-to-latest\"]";

    /// `crate::dom::set_dom_attribute` を本モジュールの語彙で再エクスポート
    /// する（イシュー #2122 レビュー指摘: REQ-11 gzip バンドルサイズ抑制の
    /// ため、`sidebar`/`focus_visible`/`focus_trap`/`position` 等と重複
    /// していた実装を `crate::dom` へ共通化した）。本モジュールが書き込む
    /// 属性（`data-stuck`/`data-has-new`/`data-visible`/`hidden`）はいずれも
    /// `&'static str` リテラルで固定された非 URL・非イベントハンドラ属性
    /// であり実害はないが、共通化後もガードは経由し続ける。
    use crate::dom::set_dom_attribute;

    /// `crate::dom::closest_matching` を `scope = SCOPE` 固定で呼ぶ薄い
    /// ラッパー（呼び出し元は本モジュール内すべて `SCOPE` 固定のため、
    /// 3 引数の従来シグネチャを維持したまま `crate::dom` の共通実装へ
    /// 委譲する。REQ-11 gzip バンドルサイズ抑制、イシュー #2122）。
    fn closest_matching(root: &Element, start: &Element, part: &str) -> Option<Element> {
        crate::dom::closest_matching(root, start, SCOPE, part)
    }

    /// `start` から `boundary`（含む）まで祖先方向を辿り、`predicate` を
    /// 満たす要素が 1 つでもあれば `true`（バンドルサイズ抑制のため
    /// [`has_disabled_ancestor`]/[`has_explicit_action`] の祖先歩行部分を
    /// 共通化した版。`questionnaire::wiring` の同種祖先歩行と同じ意図）。
    fn any_ancestor(
        boundary: &Element,
        start: &Element,
        predicate: impl Fn(&Element) -> bool,
    ) -> bool {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if predicate(&element) {
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
    /// （`include_loading` が `true` なら `data-loading` も判定に含める
    /// `load-more` 用、`false` なら `jump-to-latest` 用に `data-loading` を
    /// 無視する）。
    fn has_disabled_ancestor(boundary: &Element, start: &Element, include_loading: bool) -> bool {
        any_ancestor(boundary, start, |element| {
            element.has_attribute("data-disabled")
                || element.has_attribute("disabled")
                || (include_loading && element.has_attribute("data-loading"))
        })
    }

    /// `start` から `boundary`（含む）まで祖先方向を辿り、`data-action`
    /// を持つ要素が 1 つでもあれば `true`（アプリが手動配線を明示的に
    /// 選んだものとして自動通知を抑止する、`questionnaire::wiring::
    /// resolve_trigger` の `has_explicit_action` 累積判定と同じ意図）。
    fn has_explicit_action(boundary: &Element, start: &Element) -> bool {
        any_ancestor(boundary, start, |element| {
            element.has_attribute("data-action")
        })
    }

    /// `element` の `data-stuck` を `stuck` へ書き込み、連動する
    /// `data-has-new`（`Bottom` なら除去）・`jump-to-latest` の
    /// `data-visible`/`hidden`（[`jump_visible`] に同期）を反映する。
    fn write_stuck(instance_root: &Element, stuck: super::MessageScrollerStuck) {
        set_dom_attribute(instance_root, "data-stuck", stuck.as_str());
        if stuck == super::MessageScrollerStuck::Bottom {
            let _ = instance_root.remove_attribute("data-has-new");
        }
        sync_jump_to_latest_visibility(instance_root, stuck);
    }

    /// `instance_root` 配下から `selector`（`*_SELECTOR` 定数のいずれか）
    /// に一致し、かつ最も近い `data-part="root"` 祖先が `instance_root`
    /// 自身である要素のみを集めて返す（ネストしたインスタンスの同名
    /// パートを誤って対象に含めない、バンドルサイズ抑制のため
    /// [`sync_jump_to_latest_visibility`]/[`handle_click`] の
    /// NodeList 走査を共通化した）。呼び出し元は `&'static str` の
    /// 事前組み立て済みセレクタ定数のみを渡す契約（`part_selector`/
    /// `format!` を廃止したため、実行時の部分文字列は受け付けない）。
    fn scoped_parts(instance_root: &Element, selector: &str) -> Vec<Element> {
        let Ok(nodes) = instance_root.query_selector_all(selector) else {
            return Vec::new();
        };
        let mut result = Vec::new();
        for i in 0..nodes.length() {
            let Some(node) = nodes.get(i) else { continue };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            if closest_matching(instance_root, &element, PART_ROOT).as_ref() == Some(instance_root)
            {
                result.push(element);
            }
        }
        result
    }

    /// `instance_root` 配下の `jump-to-latest` 要素の可視状態
    /// （`data-visible`/`hidden` の排他契約、headless-ui
    /// `jump_to_latest` の排他契約と同型）を [`jump_visible`] に同期する。
    fn sync_jump_to_latest_visibility(instance_root: &Element, stuck: super::MessageScrollerStuck) {
        let visible = jump_visible(stuck);
        for element in scoped_parts(instance_root, JUMP_TO_LATEST_SELECTOR) {
            if visible {
                set_dom_attribute(&element, "data-visible", "");
                let _ = element.remove_attribute("hidden");
            } else {
                set_dom_attribute(&element, "hidden", "");
                let _ = element.remove_attribute("data-visible");
            }
        }
    }

    /// `viewport` を即時で最下部へスクロールする（モジュール doc
    /// 「プログラム的スクロールは常に即時」参照）。
    fn scroll_viewport_to_bottom(viewport: &Element) {
        scroll_viewport_to_top(viewport, viewport.scroll_height());
    }

    /// `viewport` を即時で `top` へスクロールする。
    ///
    /// `Element::set_scroll_top` は CSSOM View 仕様上、対象要素の
    /// `scroll-behavior` が `smooth` だとアニメーション付きスクロールに
    /// なり得る（アニメーション中の割り込みで補正量が欠落し得る）ため、
    /// [`initial_sync_instance`] が配線時に `viewport` へインライン
    /// `scroll-behavior: auto` を固定設定し（[`overflow-anchor`]
    /// 設定と同じ箇所・同じ理由）、アプリ側 CSS が `scroll-behavior:
    /// smooth` を指定していてもインラインスタイル（最高詳細度）で
    /// 上書きして常に即時にする。`ScrollToOptions`/`ScrollBehavior`
    /// 型を経由する構成（呼び出しごとに behavior を明示する設計）から、
    /// 配線時 1 回の CSS 固定へ変更した（バンドルサイズ抑制、レビュー
    /// 指摘 #2122: REQ-11 gzip 上限超過の是正）。
    fn scroll_viewport_to_top(viewport: &Element, top: i32) {
        viewport.set_scroll_top(top);
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

    /// `viewport` の計測スナップショットを最新値へ置換する（既存があれば
    /// 上書き、無ければ追加。バンドルサイズ抑制のため、各呼び出し箇所に
    /// 重複していた「探索 → 組み立て → 上書き/追加」の定型を共通化した）。
    fn upsert_snapshot(snapshots: &SnapshotList, viewport: &Element) {
        let entry = InstanceSnapshot {
            viewport: viewport.clone(),
            last_scroll_top: viewport.scroll_top(),
            last_scroll_height: viewport.scroll_height(),
        };
        let index = find_snapshot_index(snapshots, viewport);
        let mut list = snapshots.borrow_mut();
        match index {
            Some(i) => list[i] = entry,
            None => list.push(entry),
        }
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
            let style = html.style();
            let _ = style.set_property("overflow-anchor", "none");
            // `scroll_viewport_to_top` doc 参照: アプリ側 CSS の
            // `scroll-behavior: smooth` をインラインスタイルで固定上書き
            // し、`set_scroll_top` を常に即時にする。
            let _ = style.set_property("scroll-behavior", "auto");
        }

        let stuck = stuck_from_attr(instance_root.get_attribute("data-stuck").as_deref());
        if stuck == super::MessageScrollerStuck::Bottom {
            scroll_viewport_to_bottom(viewport);
        }
        sync_jump_to_latest_visibility(instance_root, stuck);
        upsert_snapshot(snapshots, viewport);
    }

    /// `root` 配下の message-scroller インスタンスへ初期同期を適用する。
    /// `only_new == false`（マウント直後の一括初期化専用）なら既存
    /// インスタンス分も無条件で `initial_sync_instance` を呼び直し、
    /// スナップショットを「今の」計測値で上書きする。`only_new == true`
    /// （`handle_mutations` の冒頭専用）ならスナップショット未登録の
    /// viewport（構造フォールバック再描画で新規に現れたインスタンス）
    /// のみへ適用し、既存インスタンスの `prev_scroll_top`/
    /// `prev_scroll_height` を後続の変化判定用に温存する（Review 指摘
    /// #2122: 無条件で呼ぶと既存インスタンス分の prev 値がミューテーション
    /// 適用後の現在値で上書きされ、`classify_change` が常に
    /// `ContentChange::None` を返してしまい、新着検知・履歴読み込み時の
    /// 位置維持が機能しなくなる退行があった）。
    fn initial_sync(root: &Element, snapshots: &SnapshotList, only_new: bool) {
        let selector = VIEWPORT_SELECTOR;
        let Ok(nodes) = root.query_selector_all(selector) else {
            return;
        };
        for i in 0..nodes.length() {
            let Some(node) = nodes.get(i) else { continue };
            let Ok(viewport) = node.dyn_into::<Element>() else {
                continue;
            };
            if only_new && find_snapshot_index(snapshots, &viewport).is_some() {
                continue;
            }
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
        upsert_snapshot(snapshots, viewport);
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
            let Some(instance_root) = closest_matching(root, &jump, PART_ROOT) else {
                return;
            };
            // codex-review 指摘（PR #2312）: 境界に `jump` 自身を渡すと、
            // インスタンス root や root と jump の間にあるラッパーへ
            // `data-disabled` が付いていても無効化を見逃す（load-more と
            // 挙動が不一致になる）。load-more 側（下記）と同じく、まず
            // instance root を解決してからそれを境界にクリック対象からの
            // 無効化祖先を判定する。
            if has_disabled_ancestor(&instance_root, &target_element, false) {
                return;
            }
            for viewport in scoped_parts(&instance_root, VIEWPORT_SELECTOR) {
                scroll_viewport_to_bottom(&viewport);
                upsert_snapshot(snapshots, &viewport);
            }
            write_stuck(&instance_root, super::MessageScrollerStuck::Bottom);
            event.stop_propagation();
            return;
        }

        if let Some(load_more) = closest_matching(root, &target_element, PART_LOAD_MORE) {
            let Some(instance_root) = closest_matching(root, &load_more, PART_ROOT) else {
                return;
            };
            // §31.6 契約: クリック対象からインスタンス root までの祖先を
            // 確認する（`load_more` 止まりでは `root` と `load_more` の
            // 間にある `data-disabled`/`data-loading` 祖先を見逃す）。
            if has_disabled_ancestor(&instance_root, &target_element, true) {
                return;
            }
            if has_explicit_action(&load_more, &target_element) {
                return;
            }
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
        let viewport_selector = VIEWPORT_SELECTOR;
        let viewport = instance_root
            .query_selector(viewport_selector)
            .ok()
            .flatten()?;
        Some((instance_root, viewport))
    }

    /// 会話リスト本体を解決する: `content` 自身が `data-bind-list` を
    /// 持てば `content` 自身（フラットな keyed list 構成）、そうでなければ
    /// `content` 配下で最も浅い `data-bind-list`（幅優先探索で求める。
    /// メッセージ内部にネストした `data-bind-list`〔添付・リアクション等〕
    /// より必ず浅い階層で見つかる）。`content` 配下のどこにも
    /// `data-bind-list` が無ければ `content` 自身を会話リストとみなす
    /// （keyed list を使わない素の追記構成、`build_message_scroller` の
    /// テストフィクスチャ等）。
    ///
    /// [`classify_target`] が「会話リスト本体そのものへの変更」と
    /// 「会話リストより深いネストした二次的な bind-list への変更」を
    /// 区別するための基準点として使う（コーディネータ指摘、PR #2312:
    /// 旧実装は 2 段階の祖先歩行ヒューリスティックで代用しており、
    /// フラット構成〔`content` 自身が `data-bind-list`〕で `content` 自身の
    /// 属性を見ずに探索を打ち切っていたため、メッセージ内の `attachments`
    /// を会話リスト本体と誤認していた）。
    fn conversation_list(content: &Element) -> Element {
        if content.has_attribute(fandhe_frontend_core::keyed::BIND_LIST_ATTR) {
            return content.clone();
        }
        let mut queue: std::collections::VecDeque<Element> = std::collections::VecDeque::new();
        let mut sibling = content.first_element_child();
        while let Some(element) = sibling {
            sibling = element.next_element_sibling();
            queue.push_back(element);
        }
        while let Some(element) = queue.pop_front() {
            if element.has_attribute(fandhe_frontend_core::keyed::BIND_LIST_ATTR) {
                return element;
            }
            let mut child = element.first_element_child();
            while let Some(c) = child {
                child = c.next_element_sibling();
                queue.push_back(c);
            }
        }
        content.clone()
    }

    /// [`first_relevant_change`] が `Prepend`/`Grow` 判定の対象にしてよい
    /// 変更かどうかを分類した結果。
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TargetRelevance {
        /// `target_element` が会話リスト本体（[`conversation_list`]）
        /// そのもの。先頭挿入かどうかで `Prepend`/`Grow` を判定できる。
        ConversationList,
        /// 会話リスト本体の内側（またはリストを使わない `content` 直下）
        /// だが、リスト本体自身ではない要素への変更。`Prepend` 候補には
        /// せず `Grow` の根拠としてのみ扱う。
        Other,
    }

    /// `record.target()`（`childList`）または `Text` ノードの親要素
    /// （`characterData`）から `content` まで祖先歩行し（**両端を含む**:
    /// `target_element` 自身・`content` 自身の属性も確認する）、この変更が
    /// scroller レベルの `Prepend`/`Grow` 判定対象かを分類する。
    ///
    /// 判定は 1 回の祖先歩行で完結する: `target_element` から `content`
    /// （含む）まで歩きながら、(a) ネストした message-scroller インスタンス
    /// の `root`（`data-scope="message-scroller"`/`data-part="root"`）を
    /// 跨げば `None`（別インスタンスの変異、`scoped_parts` と同じネスト
    /// 分離パターン）、(b) [`conversation_list`] **以外**の `data-bind-list`
    /// を跨げば `None`（会話リストより深いネストした二次的なリスト
    /// 〔添付・リアクション・ツールステップ等〕への変更、閲覧中の会話とは
    /// 無関係な二次的コンテンツのため `scrollTop` 補正・`data-has-new` の
    /// 根拠にしない）。どちらにも該当しなければ対象とし、`target_element`
    /// が会話リスト本体そのものなら
    /// [`TargetRelevance::ConversationList`]、そうでなければ
    /// [`TargetRelevance::Other`] を返す。
    ///
    /// **`childList`（target がリスト要素自身、または `data-bind-list` を
    /// 持たない任意の子孫要素）と `characterData`（target が `Text` ノード
    /// の親要素）の両方から、record の種類・target の種類を問わず
    /// この 1 つの関数を呼ぶ**（コーディネータ指摘、PR #2312 再指摘 3 件の
    /// 根本原因: 経路ごとに別のヒューリスティックを持っていたため、
    /// (1) childList で `data-bind-list` を持たない target〔ネストした
    /// `attachments` 内の `span` を `set_text_content` で更新する等〕は
    /// ネスト除外を経由せず Grow 扱いになり、同じ要素を `characterData`
    /// 〔`appendData`〕で更新した場合と挙動が食い違っていた。(2)(3)
    /// フラット構成〔`content` 自身が `data-bind-list`〕で、祖先歩行が
    /// `content` に到達した時点で `content` 自身の属性を確認せず打ち切る
    /// 2 段階ヒューリスティックが、メッセージ内の `attachments` を会話
    /// リスト本体と誤認していた。単一の祖先歩行で `content` 自身の属性も
    /// 含めて判定する本関数はこの 3 件すべてを構造的に解消する）。
    fn classify_target(content: &Element, target_element: &Element) -> Option<TargetRelevance> {
        if target_element != content && !content.contains(Some(target_element)) {
            return None;
        }
        let conversation = conversation_list(content);
        let mut current = Some(target_element.clone());
        loop {
            let Some(element) = current else {
                break;
            };
            // (a) ネストした別インスタンスの root を跨ぐ場合は除外。
            if element.get_attribute("data-scope").as_deref() == Some(SCOPE)
                && element.get_attribute("data-part").as_deref() == Some(PART_ROOT)
            {
                return None;
            }
            // (b) 会話リスト本体以外の data-bind-list を跨ぐ場合は除外
            // （会話リスト本体自身は跨いでもよい・むしろ跨ぐことが
            // ConversationList 判定の前提）。
            if element.has_attribute(fandhe_frontend_core::keyed::BIND_LIST_ATTR)
                && element != conversation
            {
                return None;
            }
            if element == *content {
                break;
            }
            current = element.parent_element();
        }
        if *target_element == conversation {
            Some(TargetRelevance::ConversationList)
        } else {
            Some(TargetRelevance::Other)
        }
    }

    /// 今回のバッチが scroller レベルの `Prepend`/`Grow` 分類（`scrollTop`
    /// 補正・`data-has-new` 付与）の対象になるかを判定する
    /// （[`classify_change`] の `first_added_is_prepend` 引数用）。
    ///
    /// 戻り値は 3 通り:
    /// - `Some(true)`: 会話リスト本体（[`conversation_list`]）へ、
    ///   追加ノード群がその親の先頭（`previousSibling` が無い位置）へ
    ///   **既存ノードの前に**挿入された（先頭挿入 = Prepend）。viewport の
    ///   ジオメトリには依存しない（`content` の上部 padding の影響を
    ///   受けない、レビュー指摘 #2122 line 196）。
    /// - `Some(false)`: 会話リスト本体への追加だが先頭挿入ではない
    ///   （末尾追記等）、または [`TargetRelevance::Other`]（会話リストの
    ///   内側の子孫・`bind_text` によるストリーミング本文のテキスト置換・
    ///   `characterData` によるテキスト更新等）で、高さ増分は正当な成長
    ///   （`Grow`）として扱ってよい。
    /// - `None`: 今回のバッチに、scroller レベルの分類対象となる変更が
    ///   一切無い（ネストした message-scroller インスタンス・会話リストより
    ///   深いネストした `data-bind-list` のみへの変更だった等、
    ///   [`classify_target`] がすべての record で `None` を返した場合）。
    ///   呼び出し側は `scrollTop` 補正・`data-has-new` 付与のいずれも
    ///   行わないこと（`ContentChange::None` 相当）。
    ///
    /// `previousSibling` が無いことに加えて `nextSibling` が存在する
    /// ことも要求する: 空リスト（または空の `data-bind-list`）への
    /// 初回追加は追加ノード群の前後どちらにも既存ノードが無いため
    /// `previousSibling`/`nextSibling` がともに `None` になり、この
    /// 場合は先頭挿入ではなく初期成長（Grow）として扱う。Free 状態の
    /// 空リストへ viewport より高い新着メッセージを追加した場合に
    /// scrollTop 補正で位置を動かしてしまい `data-has-new` も
    /// 付与されない（Free 状態は位置を維持して新着を通知する契約に
    /// 反する）不具合を修正する（codex-review 指摘 #2122 line 789）。
    ///
    /// `characterData` レコード（既存 `Text` ノードの `nodeValue`/
    /// `appendData` によるストリーミング本文更新）の `target` は `Text`
    /// ノードであり `Element` へダウンキャストできないため、
    /// [`resolve_instance_from_record`] と同じパターンで `parent_element()`
    /// により要素へ解決してから [`classify_target`] へ渡す。`characterData`
    /// レコードは `added_nodes()` が常に空（`childList` レコードのみが
    /// 要素追加を持つ）であるため、`added.length() == 0` による除外より
    /// 前に判定を完結させ、[`classify_target`] が対象と判定した場合は
    /// 常に成長根拠（`has_growth_evidence`）へ合流させる。
    fn first_relevant_change(records: &[MutationRecord], content: &Element) -> Option<bool> {
        // 会話リスト本体以外（例: ストリーミング本文のテキスト置換）で、
        // 高さ増分を正当な成長として扱ってよい変更を確認したかどうか。
        let mut has_growth_evidence = false;
        for record in records {
            let Some(target) = record.target() else {
                continue;
            };
            // `characterData` レコードの `target` は `Text` ノードのため
            // `Element` へダウンキャストできない。`resolve_instance_from_record`
            // と同じパターンで `parent_element()` により要素へ解決する。
            let target_element: Element = if let Some(element) = target.dyn_ref::<Element>() {
                element.clone()
            } else {
                let Some(node) = target.dyn_ref::<web_sys::Node>() else {
                    continue;
                };
                let Some(parent) = node.parent_element() else {
                    continue;
                };
                parent
            };
            let Some(relevance) = classify_target(content, &target_element) else {
                continue;
            };
            if record.type_() == "characterData" {
                has_growth_evidence = true;
                continue;
            }
            let added = record.added_nodes();
            if added.length() == 0 {
                continue;
            }
            if relevance == TargetRelevance::ConversationList {
                // 前後どちらにも既存ノードが無い（`nextSibling` も無い）
                // 場合は空リストへの初回追加であり、先頭挿入ではない。
                return Some(
                    record.previous_sibling().is_none() && record.next_sibling().is_some(),
                );
            }
            // `bind_text`（`fandhe-frontend-wasm-client::binding_dom::
            // apply_one` の `set_text_content`、ストリーミング本文の
            // 更新）は本文を表示する、`data-bind-list` を持たない任意の
            // 子孫要素（メッセージ 1 件の内部にあるテキスト表示要素）へ
            // `childList` レコードを生む（既存の子ノードを丸ごと入れ替え
            // るため、置き換え後の唯一の子ノードは `previousSibling` を
            // 持たない）。本文内で子要素だけを追加するケース（テキスト
            // ではなく Element を追記する段階的レンダリング）の target も
            // 同様に `data-bind-list` を持たない子孫要素である。これらは
            // 会話リスト本体そのものへの挿入ではないが、会話の一部として
            // 表示される本文の成長であるため、Prepend には分類せず
            // （先頭挿入と誤判定しない）、Grow の根拠としては許容する。
            has_growth_evidence = true;
        }
        if has_growth_evidence {
            Some(false)
        } else {
            None
        }
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
        // 先立って現在の viewport 集合へ未登録分のみを初期同期しておく。
        // 既存インスタンス分は触らない: `only_new = false`（無条件で
        // 全 viewport を再同期し `data-stuck == Bottom` なら毎回
        // 強制スクロールする）をここで使うと、直後の per-record ループが
        // 読む `prev_scroll_top`/`prev_scroll_height` がミューテーション
        // 適用後の現在値で上書きされてしまい、`classify_change` が
        // 常に `ContentChange::None` を返す（`new <= prev` 恒真）ため
        // 新着検知・履歴読み込み時の位置維持が機能しなくなる
        // （Review 指摘 #2122）。
        initial_sync(root, snapshots, true);

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
            let content_selector = CONTENT_SELECTOR;
            let Some(content) = instance_root
                .query_selector(content_selector)
                .ok()
                .flatten()
            else {
                continue;
            };
            // `None`（今回のバッチに scroller レベルの分類対象となる変更が
            // 無い、ネストした keyed list への変更のみだった等）の場合は
            // `scrollTop` 補正・`data-has-new` のいずれも行わない
            // （`ContentChange::None` 相当。Bugbot 指摘 PR #2312、
            // `first_relevant_change` doc 参照）。
            let change = match first_relevant_change(&record_list, &content) {
                Some(is_prepend) => {
                    classify_change(is_prepend, prev_scroll_height, new_scroll_height)
                }
                None => ContentChange::None,
            };

            let stuck = stuck_from_attr(instance_root.get_attribute("data-stuck").as_deref());
            let plan = plan_after_change(stuck, change);

            if plan.correct_prepend {
                let new_top =
                    corrected_scroll_top(prev_scroll_top, prev_scroll_height, new_scroll_height);
                scroll_viewport_to_top(&viewport, new_top);
            }
            if plan.scroll_to_bottom {
                scroll_viewport_to_bottom(&viewport);
            }
            if plan.mark_has_new {
                set_dom_attribute(&instance_root, "data-has-new", "");
                sync_jump_to_latest_visibility(&instance_root, stuck);
            }

            upsert_snapshot(snapshots, &viewport);
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
        let root_selector = ROOT_SELECTOR;
        let has_instance = root.get_attribute("data-scope").as_deref() == Some(SCOPE)
            && root.get_attribute("data-part").as_deref() == Some(PART_ROOT)
            || root.query_selector(root_selector).ok().flatten().is_some();
        if !has_instance {
            return Ok(());
        }

        let on_action = Rc::new(RefCell::new(on_action));
        let snapshots: SnapshotList = Rc::new(RefCell::new(Vec::new()));

        initial_sync(&root, &snapshots, false);

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
        assert_eq!(classify_change(false, 200, 200), ContentChange::None);
        assert_eq!(classify_change(true, 200, 150), ContentChange::None);
    }

    #[test]
    fn classify_change_prepend_when_first_added_is_prepend() {
        assert_eq!(classify_change(true, 200, 280), ContentChange::Prepend);
    }

    #[test]
    fn classify_change_grow_when_not_prepend() {
        assert_eq!(classify_change(false, 200, 280), ContentChange::Grow);
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
