//! オーバーレイ共通の閉鎖制御（Escape キー・外側インタラクション、イシュー #585、親 #584）。
//!
//! `fandhe-frontend-headless-ui` の Dialog（#562）/ Popover（#563）/ Tooltip（#564）/
//! Menu（#566）は SSR マークアップと開閉状態機械（[`crate::events`] と対になる
//! `data-scope`/`data-part`/`data-state` 出力、`Component::decode_action` の
//! `"close"` アクション）までを提供し、「Escape キー閉鎖・外側クリック閉鎖は
//! JS ランタイム側（本クレート）の責務」として明示的にスコープ外としていた
//! （各コンポーネントモジュール冒頭 doc の「スコープ外」節参照）。本モジュールは
//! その欠落を埋め、オーバーレイ横断の閉鎖判定（登録・解除の対称性、複数
//! オーバーレイの入れ子）を提供する。
//!
//! [`events`] と同じ 2 層構成を踏襲する: web-sys に依存しない純粋ロジック層
//! （[`OverlayKind`]・opt-out 判定・スタック閉鎖判定、native の `cargo test`
//! で検証可能）と、`#[cfg(target_arch = "wasm32")]` でゲートした配線層
//! （[`wiring::OverlayCloseController`]）に分離する。
//!
//! # 他モジュール・他クレートとの契約
//!
//! - [`OverlayKind::from_scope`] は `fandhe-frontend-headless-ui` の各 anatomy
//!   （`crates/headless-ui/src/anatomy.rs` の `anatomy(scope)`）が出力する
//!   `data-scope` 属性値（`"dialog"`/`"popover"`/`"menu"`/`"tooltip"`/
//!   `"navigation-menu"`/`"menubar"`）と一致させる。未知の scope 値（改ざん・
//!   将来追加の別コンポーネント）は `None` とし、呼び出し側は当該要素を
//!   閉鎖制御の対象外として無視する（fail-closed、panic しない）。
//! - opt-out 属性（`data-close-on-escape`/`data-close-on-interact-outside`）は
//!   `fandhe-frontend-headless-ui` 側に専用 API を追加せず、呼び出し側が各
//!   anatomy パーツの `attrs` 引数（例: `dialog::content` の `attrs: Vec<(&str, &str)>`）
//!   経由でオプトインする前提とする（本イシューでは headless-ui クレートを
//!   変更しない）。
//! - 本モジュールは実際の `"close"` dispatch・再描画・DOM 更新を一切行わない。
//!   [`wiring::OverlayCloseController`] は閉鎖要求をコールバック
//!   （[`OverlayCloseRequest`]）へ通知するのみであり、`dispatch`
//!   （`fandhe_frontend_interactive::dispatch`）の呼び出しは呼び出し側
//!   （イシュー #580 の DOM イベント配線統合層）の責務とする
//!   （[`events::wire_events`] と同じ責務分離方針）。
//! - フォーカストラップ・トリガーへのフォーカス復帰はイシュー #586、
//!   Tooltip の `openDelay`/`closeDelay`/interactive 継続はイシュー #587の
//!   スコープであり、いずれも本モジュールでは扱わない
//!   （`.claude/rules/out-of-scope-tracking.md` 対応済み、兄弟イシューで追跡中）。
//!
//! ## `NavigationMenu`/`Menubar` の閉鎖要求と呼び出し側の dispatch（イシュー #1173）
//!
//! `OverlayCloseRequest` を受け取った呼び出し側（#580 統合層）が実際に
//! `dispatch` すべきアクション名は種別ごとに異なる（本モジュールは通知のみで
//! 完結し、`dispatch` 自体は行わない前提を上記契約節のとおり維持する）:
//!
//! - [`OverlayKind::NavigationMenu`][]: `crates/headless-ui/src/navigation_menu.rs`
//!   の `SingleSelect::decode_action` が受理する `"deselect"`（payload 不使用）。
//!   既に開いている項目を未選択へ戻す冪等操作であり、二重 dispatch されても
//!   no-op のまま安全に収束する。
//! - [`OverlayKind::Menubar`][]: `crates/headless-ui/src/menubar.rs` の
//!   `MenubarAction::Close`（`"close"`、payload 不使用）。全 Menu を閉じる
//!   冪等操作。
//!
//! ## keynav との二重処理の収束（イシュー #1173）
//!
//! `crate::keynav` は NavigationMenu の Escape を「open 中の trigger/content
//! 上でのみ `trigger.click()` を合成して close を委譲」する既存挙動を持つ
//! （`crate::keynav` モジュール doc §NavigationMenu 参照）。本モジュールの
//! Escape 処理と keynav の Escape 処理は document への keydown リスナー登録順
//! に依存して両方発火しうるが、いずれの順序でも同一の closed 状態へ収束する:
//!
//! - keynav 先行: `trigger.click()` 合成 → `toggle` dispatch で closed に
//!   なる。続く本モジュールの `"deselect"` dispatch は既に未選択のため
//!   冪等 no-op。
//! - 本モジュール先行: `"deselect"` dispatch → closed・再描画。続く keynav は
//!   DOM の `data-state` を再確認するため、closed になったトリガー上の
//!   Escape は claim せず no-op（`crate::keynav` の「closed の trigger 上の
//!   Escape は no-op」既定と同じ fail-closed 経路）。
//!
//! Menubar 側の keynav Escape は元々「highlight の後始末のみ、閉鎖は overlay
//! の責務」と明記済みであり（`crate::keynav` モジュール doc §Menubar「既知の
//! ギャップ」節、本イシューで解消済みへ更新）、本モジュールの閉鎖と競合しない。

use crate::events::AttrSource;

/// 閉鎖制御の対象となるオーバーレイ種別。
///
/// `fandhe-frontend-headless-ui` の `data-scope` 属性値と 1 対 1 対応する
/// （`crates/headless-ui/src/{dialog,popover,menu,tooltip,navigation_menu,
/// menubar}.rs` の `const ANATOMY: Anatomy = anatomy("...")` 参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayKind {
    /// `data-scope="dialog"`。
    Dialog,
    /// `data-scope="popover"`。
    Popover,
    /// `data-scope="menu"`。
    Menu,
    /// `data-scope="tooltip"`。
    Tooltip,
    /// `data-scope="navigation-menu"`（イシュー #1173。
    /// `crates/headless-ui/src/navigation_menu.rs`）。
    NavigationMenu,
    /// `data-scope="menubar"`（イシュー #1173。
    /// `crates/headless-ui/src/menubar.rs`）。
    Menubar,
}

impl OverlayKind {
    /// `data-scope` 属性値からのパース。未知の scope 値は `None`
    /// （改ざん・非対応コンポーネントに対する fail-closed。呼び出し側は
    /// `None` の要素を閉鎖制御の対象から除外する）。
    #[must_use]
    pub fn from_scope(scope: &str) -> Option<Self> {
        match scope {
            "dialog" => Some(Self::Dialog),
            "popover" => Some(Self::Popover),
            "menu" => Some(Self::Menu),
            "tooltip" => Some(Self::Tooltip),
            "navigation-menu" => Some(Self::NavigationMenu),
            "menubar" => Some(Self::Menubar),
            _ => None,
        }
    }

    /// Escape キーでの閉鎖既定値。全種別で `true`
    /// （opt-out 属性で個別に無効化可能、[`close_on_escape_for`] 参照）。
    #[must_use]
    pub const fn close_on_escape(self) -> bool {
        true
    }

    /// 外側インタラクションでの閉鎖既定値（**kind 単位**の既定であり、
    /// `role="alertdialog"` 等コンテンツ属性による上書きは含まない。実効値は
    /// [`close_on_interact_outside_for`] を使う）。
    ///
    /// Dialog/Popover/Menu/NavigationMenu/Menubar は `true`。Tooltip のみ
    /// `false` とする — Tooltip の非表示はポインタ離脱・遅延タイマー
    /// （イシュー #587）が主経路であり、外側クリックによる即時閉鎖を既定に
    /// すると #587 の `closeDelay`/interactive 継続の設計判断と競合し得る
    /// ため、既定では外側インタラクション閉鎖の対象外とする（opt-in は将来
    /// #587 側の設計次第。現時点で Tooltip 用の opt-in 属性は未定義）。
    /// NavigationMenu/Menubar は Tooltip のような遅延タイマー競合の事情が
    /// なく、外側クリック即時閉鎖が参照軸（Radix/ark-ui）の標準挙動である
    /// ため Menu と同じ既定とする（イシュー #1173）。
    #[must_use]
    pub const fn close_on_interact_outside(self) -> bool {
        !matches!(self, Self::Tooltip)
    }

    /// 「外側インタラクションで閉鎖しない」ことが、下層オーバーレイへの
    /// 伝播（[`outside_close_indices`]）を遮断する「意図的な永続化」を
    /// 意味するか。
    ///
    /// Tooltip のみ `false` とする —Tooltip が閉じない既定値は
    /// [`close_on_interact_outside`] の doc の通り「オーバーレイスタックへの
    /// 非参加」であり、永続化を選んだわけではない。そのため Tooltip が
    /// 開いている間も、その下にある Dialog/Popover/Menu は外側クリックで
    /// 閉じられるべきであり、Tooltip の存在で伝播を止めてはならない。
    ///
    /// Dialog（`role="alertdialog"` を含む）/Popover/Menu/NavigationMenu/
    /// Menubar は `true` — これらが外側クリックで閉じない場合
    /// （`role="alertdialog"` の既定、または
    /// `data-close-on-interact-outside="false"` の明示 opt-out）は
    /// 呼び出し側が意図的に永続化を選んだものであり、子を孤児化させてまで
    /// 親オーバーレイを閉じない安全側の判断を維持する。NavigationMenu/
    /// Menubar の明示 opt-out も Menu と同型の意図的な永続化として扱う
    /// （イシュー #1173）。
    #[must_use]
    pub const fn outside_dismiss_blocks_propagation_by_default(self) -> bool {
        !matches!(self, Self::Tooltip)
    }
}

/// `data-close-on-escape` opt-out 属性を読み、[`OverlayKind::close_on_escape`]
/// の既定値へ反映する。
///
/// `content` 要素上の値が `"false"` のときのみ無効化として解釈する。属性が
/// 欠落・`"true"`・その他の不正値の場合はいずれも種別既定値へフォールバック
/// する（クライアントで改ざんされうる `data-*` 入力に対する fail-closed。
/// opt-out できる対象は閉鎖挙動のみであり、エスケープ保証・状態機械の
/// 不変条件には影響しない）。
#[must_use]
pub fn close_on_escape_for<T: AttrSource>(kind: OverlayKind, content: &T) -> bool {
    match content.attr("data-close-on-escape").as_deref() {
        Some("false") => false,
        _ => kind.close_on_escape(),
    }
}

/// `data-close-on-interact-outside` opt-out 属性を読み、
/// [`OverlayKind::close_on_interact_outside`] の既定値へ反映する。
///
/// [`close_on_escape_for`] と同じ fail-closed 方針（`"false"` のみ無効化、
/// 他は既定値へフォールバック）。加えて、`kind` が [`OverlayKind::Dialog`]
/// かつ `content` の `role` 属性が `"alertdialog"` の場合、属性が明示的に
/// 与えられていない限り既定値を `false` に上書きする — ark-ui / WAI-ARIA の
/// alertdialog パターン（外側クリックで閉じない。エラー・破壊的操作の確認
/// など、ユーザーに明示的な選択を強制する用途のため）に合わせる。
/// `role="dialog"`（通常ダイアログ）はこの上書きの対象外。
#[must_use]
pub fn close_on_interact_outside_for<T: AttrSource>(kind: OverlayKind, content: &T) -> bool {
    match content.attr("data-close-on-interact-outside").as_deref() {
        Some("false") => false,
        _ => {
            if matches!(kind, OverlayKind::Dialog) && is_alertdialog(content) {
                false
            } else {
                kind.close_on_interact_outside()
            }
        }
    }
}

/// `content` の `role` 属性が `"alertdialog"` かどうかを判定する
/// （[`close_on_interact_outside_for`] の alertdialog 既定上書き専用の
/// 内部ヘルパー）。
fn is_alertdialog<T: AttrSource>(content: &T) -> bool {
    content.attr("role").as_deref() == Some("alertdialog")
}

/// 「外側インタラクションで閉鎖しない」ことが下層への伝播を遮断する
/// 「意図的な永続化」を意味するかを判定する
/// （[`OverlayKind::outside_dismiss_blocks_propagation_by_default`] 参照）。
///
/// `data-close-on-interact-outside="false"` の明示 opt-out は、kind を問わず
/// 常に「意図的な永続化」として扱い、伝播を遮断する（属性を明示した以上、
/// 呼び出し側の意図は明確なため）。属性が欠落・その他の不正値の場合は
/// kind 既定値へフォールバックする（Tooltip の既定非参加は遮断しない。
/// `role="alertdialog"` の既定非閉鎖は kind 既定値 `true` により遮断する
/// ——Dialog kind である以上、role によらず遮断側の既定を維持する）。
#[must_use]
pub fn outside_dismiss_blocks_propagation_for<T: AttrSource>(
    kind: OverlayKind,
    content: &T,
) -> bool {
    match content.attr("data-close-on-interact-outside").as_deref() {
        Some("false") => true,
        _ => kind.outside_dismiss_blocks_propagation_by_default(),
    }
}

/// オーバーレイスタック上の 1 エントリ（[`wiring::OverlayCloseController`] が
/// push/remove で管理する）。
///
/// `close_on_escape`/`close_on_interact_outside` は各エントリの push 時点で
/// [`close_on_escape_for`]/[`close_on_interact_outside_for`] により確定した
/// 値を保持する（DOM 属性を毎回読み直さない。開閉のたびに opt-out 属性が
/// 変わる想定はないため、push 時点でのスナップショットで十分）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverlayEntry {
    /// このエントリのオーバーレイ種別。
    pub kind: OverlayKind,
    /// Escape キーでの閉鎖を許可するか。
    pub close_on_escape: bool,
    /// 外側インタラクションでの閉鎖を許可するか。
    pub close_on_interact_outside: bool,
    /// `close_on_interact_outside == false` のとき、それが
    /// [`outside_close_indices`] の下層への伝播を遮断する「意図的な永続化」
    /// を意味するか。`close_on_interact_outside == true` のときは無関係
    /// （伝播判定に使われない）。
    ///
    /// 「オーバーレイスタックへの非参加」（Tooltip の既定）と「意図的な
    /// 永続化オプトアウト」（明示 opt-out・`role="alertdialog"` の既定）は
    /// 別概念であり、前者は `false`（下層を閉じさせる）、後者は `true`
    /// （下層を巻き添えで閉じない）とする（
    /// [`OverlayKind::outside_dismiss_blocks_propagation_by_default`] /
    /// [`outside_dismiss_blocks_propagation_for`] 参照）。
    pub outside_dismiss_blocks_propagation: bool,
}

/// 開いているオーバーレイのスタック（下から古い順、末尾が最上位/topmost）
/// のうち、Escape キーで閉鎖すべきエントリの index を判定する。
///
/// 最上位（末尾）のみが Escape の対象となる（レイヤー方式: 入れ子オーバー
/// レイの Escape キーは最前面の 1 枚だけを閉じ、下層へ透過させない）。
/// 最上位が opt-out（`close_on_escape == false`）の場合は `None` を返し、
/// 下層エントリへは透過させない（永続化を選んだ最上位の意図を尊重する）。
/// 空スタックも `None`。
#[must_use]
pub fn escape_close_index(stack: &[OverlayEntry]) -> Option<usize> {
    let last_index = stack.len().checked_sub(1)?;
    let topmost = &stack[last_index];
    topmost.close_on_escape.then_some(last_index)
}

/// 外側インタラクション（pointerdown 等）で閉鎖すべきエントリの index 一覧
/// （最上位から順）を判定する。
///
/// `contains_target[i]` は「スタック `i` 番目のエントリがターゲットを
/// 含む（content がターゲットを含む、またはそのオーバーレイの trigger が
/// ターゲットを含む）」ことを表す合成値。trigger 上の pointerdown を outside
/// 扱いにすると、閉鎖判定の直後に同じ pointerdown 由来の click で
/// トグルが再度開いてしまう競合が起きるため、trigger 含有は「内側」として
/// 扱う（呼び出し側でこの合成を行う。本関数は結果の `bool` のみを受け取る）。
///
/// 最上位から下層へ順に走査し、各エントリについて:
/// - ターゲットを含む（`contains_target[i] == true`）: そこで走査を打ち切る
///   （このエントリより下は閉鎖対象に含めない。ターゲットが属する
///   オーバーレイより外側にある祖先オーバーレイまで巻き添えで閉じない）。
/// - 外側インタラクションで閉じない（`close_on_interact_outside == false`）
///   場合、`outside_dismiss_blocks_propagation` で 2 通りに分岐する:
///   - `true`（意図的な永続化オプトアウト。明示 opt-out・
///     `role="alertdialog"` の既定）: そこで走査を打ち切る（永続化を選んだ
///     エントリの下にある親オーバーレイを、子を孤児化させてまで閉じない
///     安全側の判断）。
///   - `false`（スタック非参加。Tooltip の既定）: このエントリは閉鎖対象に
///     含めず、かつ走査も打ち切らずに次（1 つ下層）へ進む（Tooltip が
///     開いている間も、その下の Dialog/Popover/Menu の外側クリック閉鎖を
///     妨げない）。
/// - 上記いずれでもない: 閉鎖対象へ積み、次（1 つ下層）へ進む。
///
/// `stack`/`contains_target` の長さが一致しない場合は空を返す（呼び出し側の
/// 契約違反を panic させず安全側 no-op にする）。
#[must_use]
pub fn outside_close_indices(stack: &[OverlayEntry], contains_target: &[bool]) -> Vec<usize> {
    if stack.len() != contains_target.len() {
        return Vec::new();
    }
    let mut closing = Vec::new();
    for i in (0..stack.len()).rev() {
        if contains_target[i] {
            break;
        }
        let entry = &stack[i];
        if !entry.close_on_interact_outside {
            if entry.outside_dismiss_blocks_propagation {
                break;
            }
            continue;
        }
        closing.push(i);
    }
    closing
}

/// 閉鎖制御の配線層（[`wiring::OverlayCloseController`]）が発する、閉鎖を
/// 要求されたオーバーレイの通知。
///
/// `dispatch`（`"close"` アクション）の実呼び出し・DOM 再描画・
/// スタックからの `remove_overlay` は呼び出し側（イシュー #580 の統合層）の
/// 責務であり、本モジュールはこの通知を渡すのみで完結する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverlayCloseRequest {
    /// 閉鎖対象のオーバーレイ種別。
    pub kind: OverlayKind,
    /// スタック上の index（呼び出し側が対応する content 要素・状態を
    /// 特定するために使う。[`wiring::OverlayCloseController::push_overlay`]
    /// が返す index と対応する）。
    ///
    /// # 不変条件: index は非最上位の remove でシフトする
    ///
    /// この index は [`wiring::OverlayCloseController`] が内部で保持する
    /// `Vec` 上の**現在位置**であり、要素へのモノトニックなハンドルでは
    /// ない。最上位でないエントリを
    /// [`wiring::OverlayCloseController::remove_overlay`] で取り除くと、
    /// それより上位（大きい index）の全エントリの index が 1 ずつ
    /// シフトする（`Vec::remove` の仕様）。
    ///
    /// そのため、呼び出し側（イシュー #580 統合層）が「内部 close（既存
    /// close button 等、本コントローラの keydown/pointerdown 経路を通らず
    /// 独自に `remove_overlay` を呼ぶ経路）」で非最上位のオーバーレイを
    /// 閉じる場合、呼び出し側が保持する「index → オーバーレイ状態」の
    /// 対応表は `remove_overlay` 呼び出し直後にその上位分だけ古くなる。
    /// 具体例: `push_overlay` で Dialog(A) が index 0、Popover(B) が
    /// index 1 として開いている状態で、A を内部 close ボタンで閉じ
    /// `remove_overlay(0)` を呼ぶと、B は内部的に index 0 へシフトする。
    /// この後 B で Escape を押すと発行される `OverlayCloseRequest` の
    /// `index` は 0（コントローラ内では正しい最上位）だが、呼び出し側が
    /// 「B は index 1」のまま対応表を更新していないと取り違いが起きる。
    ///
    /// 呼び出し側は次のいずれかで対処する必要がある:
    /// - 内部 close で `remove_overlay` を呼ぶたびに、対応表上のそれより
    ///   大きい index を全て 1 減算して同期する
    /// - index を対応表のキーに使わず、push 時に呼び出し側が発行する
    ///   シフトに強いモノトニックなハンドル（オーバーレイ ID 等）を別途
    ///   管理し、[`wiring::OverlayCloseController::push_overlay`] が返す
    ///   index はコントローラへの操作（`remove_overlay` の引数）専用の
    ///   一時的な値として扱う
    pub index: usize,
}

// ---------------------------------------------------------------------
// 配線層: web-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`events.rs`/`hydration.rs`/`dom.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        close_on_escape_for, close_on_interact_outside_for, escape_close_index,
        outside_close_indices, outside_dismiss_blocks_propagation_for, OverlayCloseRequest,
        OverlayEntry, OverlayKind,
    };
    use crate::events::AttrSource;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Document, Element, Event, KeyboardEvent, Node};

    /// `web_sys::Element` を [`AttrSource`] へ橋渡しする薄いラッパー
    /// （`events.rs::wiring::ElementAttrSource` と同じ意図の配線層専用アダプタ）。
    struct ElementAttrSource<'a>(&'a Element);

    impl AttrSource for ElementAttrSource<'_> {
        fn attr(&self, name: &str) -> Option<String> {
            self.0.get_attribute(name)
        }
    }

    /// [`OverlayCloseController`] が管理する 1 スタックエントリの実体
    /// （純粋ロジック層の [`OverlayEntry`] に加え、containment 判定に使う
    /// 実 DOM 要素を保持する）。
    struct MountedOverlay {
        entry: OverlayEntry,
        /// このオーバーレイの content（またはそれに準ずるルート）要素。
        content: Element,
        /// トリガー要素（`None` の場合もある。例: 明示的な trigger を
        /// 持たない構成）。trigger 上の pointerdown を outside 扱いにしない
        /// ための containment 判定に使う（[`super::outside_close_indices`]
        /// doc 参照）。
        trigger: Option<Element>,
    }

    /// document へ keydown（Escape 判定）/ pointerdown（外側インタラクション
    /// 判定）のリスナーを **各 1 回だけ** 登録し、開いているオーバーレイの
    /// スタックを管理する配線層の中核型。
    ///
    /// [`crate::events::wire_events`] は `Closure::forget` によりリスナーを
    /// 意図的にリークさせる（マウントがアプリ生存期間に 1 度だけという前提）
    /// が、本コントローラはオーバーレイという**アプリ生存期間より短い
    /// ライフサイクル**を持つ要素を扱うため、[`Drop`] でリスナーを対称的に
    /// 解除する。これにより、コントローラを繰り返し生成・破棄しても
    /// document 上のリスナー数が無制限に増加しない（A04 安全でない設計
    /// （リスナーリーク／DoS）対策）。
    pub struct OverlayCloseController {
        document: Document,
        stack: std::rc::Rc<std::cell::RefCell<Vec<MountedOverlay>>>,
        keydown_closure: Closure<dyn FnMut(Event)>,
        pointerdown_closure: Closure<dyn FnMut(Event)>,
    }

    impl OverlayCloseController {
        /// `document` へ keydown/pointerdown リスナーを登録し、閉鎖要求発生時に
        /// `on_close_request` を呼ぶコントローラを組み立てる。
        ///
        /// `on_close_request` は状態変更・DOM 更新を一切行わず、呼び出し側
        /// （#580 統合層）へ「どのオーバーレイが閉じるべきか」を通知する
        /// だけの役割に限定する（本モジュール冒頭 doc の責務分離）。
        ///
        /// # Errors
        ///
        /// `add_event_listener_with_callback` が失敗した場合に `Err` を返す。
        pub fn new(
            document: &Document,
            on_close_request: impl FnMut(OverlayCloseRequest) + 'static,
        ) -> Result<Self, JsValue> {
            let stack: std::rc::Rc<std::cell::RefCell<Vec<MountedOverlay>>> =
                std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
            let on_close_request = std::rc::Rc::new(std::cell::RefCell::new(on_close_request));

            let keydown_stack = stack.clone();
            let keydown_callback = on_close_request.clone();
            let keydown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
                let Ok(keyboard_event) = event.clone().dyn_into::<KeyboardEvent>() else {
                    return;
                };
                if keyboard_event.key() != "Escape" {
                    return;
                }
                let stack_ref = keydown_stack.borrow();
                let entries: Vec<OverlayEntry> =
                    stack_ref.iter().map(|mounted| mounted.entry).collect();
                let Some(index) = escape_close_index(&entries) else {
                    return;
                };
                let kind = stack_ref[index].entry.kind;
                drop(stack_ref);
                (keydown_callback.borrow_mut())(OverlayCloseRequest { kind, index });
            });
            document.add_event_listener_with_callback(
                "keydown",
                keydown_closure.as_ref().unchecked_ref(),
            )?;

            let pointerdown_stack = stack.clone();
            let pointerdown_callback = on_close_request;
            let pointerdown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
                let Some(target) = event.target() else {
                    return;
                };
                // `event.target()` はテキストノードを指すことがある
                // （`events.rs::wiring::wire_events` と同じ事情）。要素まで
                // 遡ってから containment 判定を行う。
                let target_node: Node = match target.dyn_ref::<Node>() {
                    Some(node) => node.clone(),
                    None => return,
                };

                let stack_ref = pointerdown_stack.borrow();
                let entries: Vec<OverlayEntry> =
                    stack_ref.iter().map(|mounted| mounted.entry).collect();
                let contains_target: Vec<bool> = stack_ref
                    .iter()
                    .map(|mounted| {
                        let in_content = mounted.content.contains(Some(&target_node));
                        let in_trigger = mounted
                            .trigger
                            .as_ref()
                            .is_some_and(|trigger| trigger.contains(Some(&target_node)));
                        in_content || in_trigger
                    })
                    .collect();
                let closing_indices = outside_close_indices(&entries, &contains_target);
                if closing_indices.is_empty() {
                    return;
                }
                let requests: Vec<OverlayCloseRequest> = closing_indices
                    .into_iter()
                    .map(|index| OverlayCloseRequest {
                        kind: stack_ref[index].entry.kind,
                        index,
                    })
                    .collect();
                drop(stack_ref);
                let mut callback = pointerdown_callback.borrow_mut();
                for request in requests {
                    callback(request);
                }
            });
            if let Err(err) = document.add_event_listener_with_callback(
                "pointerdown",
                pointerdown_closure.as_ref().unchecked_ref(),
            ) {
                // keydown の登録は既に成功しているが、この時点では
                // `Self` を構築できておらず `Drop` が走らない
                // （対称的な登録・解除を保証する本型の doc 冒頭の前提が
                // 崩れる）。ここで明示的に keydown を解除し、リスナー
                // リークを防ぐ（戻り値は無視: 解除自体の失敗は復旧不能な
                // 異常系であり、呼び出し元へは元の登録エラーを伝える）。
                let _ = document.remove_event_listener_with_callback(
                    "keydown",
                    keydown_closure.as_ref().unchecked_ref(),
                );
                return Err(err);
            }

            Ok(Self {
                document: document.clone(),
                stack,
                keydown_closure,
                pointerdown_closure,
            })
        }

        /// オーバーレイの開時にスタックへ push する。`content` の
        /// `data-scope`（[`OverlayKind::from_scope`]）が未知の場合は登録
        /// せず `None` を返す（fail-closed。呼び出し側は戻り値 `None` の
        /// 場合、後続の [`Self::remove_overlay`] を呼ぶ必要がない）。
        ///
        /// opt-out 判定（[`close_on_escape_for`]/[`close_on_interact_outside_for`]）
        /// は `content` 要素の属性を push 時点で 1 回読み取り、
        /// [`OverlayEntry`] へスナップショットする。
        ///
        /// 戻り値の index は [`OverlayCloseRequest::index`] と対応し、呼び出し側が
        /// [`Self::remove_overlay`] を呼ぶ際に使う。この index は
        /// **push 時点のスタック末尾位置**であり、以降の非最上位 remove で
        /// シフトしうる（[`OverlayCloseRequest::index`] の「不変条件」節参照）。
        #[must_use]
        pub fn push_overlay(&self, content: &Element, trigger: Option<&Element>) -> Option<usize> {
            let scope = content.get_attribute("data-scope")?;
            let kind = OverlayKind::from_scope(&scope)?;
            let source = ElementAttrSource(content);
            let entry = OverlayEntry {
                kind,
                close_on_escape: close_on_escape_for(kind, &source),
                close_on_interact_outside: close_on_interact_outside_for(kind, &source),
                outside_dismiss_blocks_propagation: outside_dismiss_blocks_propagation_for(
                    kind, &source,
                ),
            };
            let mut stack = self.stack.borrow_mut();
            stack.push(MountedOverlay {
                entry,
                content: content.clone(),
                trigger: trigger.cloned(),
            });
            Some(stack.len() - 1)
        }

        /// オーバーレイの閉時にスタックから取り除く（[`Self::push_overlay`] と
        /// 対称の呼び出しを呼び出し側の契約とする）。`index` が範囲外の場合は
        /// panic せず no-op とする（呼び出し側の二重 remove・契約違反に対する
        /// 安全側フォールバック）。
        ///
        /// # 不変条件: 非最上位の remove は上位 index を全てシフトさせる
        ///
        /// 内部的には `Vec::remove(index)` を使うため、`index` が最上位
        /// （`stack_len() - 1`）でない場合、それより上位にある全エントリの
        /// index が 1 ずつ詰められる（シフトする）。呼び出し側が
        /// 「index → オーバーレイ状態」の対応表を保持している場合、この
        /// シフトにより対応表が実体と乖離しうる。詳細な失敗シナリオと
        /// 呼び出し側が取るべき対処は [`OverlayCloseRequest::index`] の
        /// doc を参照。
        pub fn remove_overlay(&self, index: usize) {
            let mut stack = self.stack.borrow_mut();
            if index < stack.len() {
                stack.remove(index);
            }
        }

        /// 現在スタックに登録されているオーバーレイの件数（テスト・
        /// デバッグ用途）。
        #[must_use]
        pub fn stack_len(&self) -> usize {
            self.stack.borrow().len()
        }
    }

    impl Drop for OverlayCloseController {
        /// keydown/pointerdown リスナーを対称的に解除する（登録は [`Self::new`]
        /// の 2 回のみ、解除もここでの 2 回のみで完結し、`Closure::forget` を
        /// 使わない。本型の doc 冒頭参照）。
        fn drop(&mut self) {
            let _ = self.document.remove_event_listener_with_callback(
                "keydown",
                self.keydown_closure.as_ref().unchecked_ref(),
            );
            let _ = self.document.remove_event_listener_with_callback(
                "pointerdown",
                self.pointerdown_closure.as_ref().unchecked_ref(),
            );
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::OverlayCloseController;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// native `cargo test` 用のテストダブル（`events.rs::tests::FakeElement`
    /// と同じ意図）。
    struct FakeElement {
        attrs: HashMap<&'static str, &'static str>,
    }

    impl AttrSource for FakeElement {
        fn attr(&self, name: &str) -> Option<String> {
            self.attrs.get(name).map(|v| v.to_string())
        }
    }

    fn element(attrs: &[(&'static str, &'static str)]) -> FakeElement {
        FakeElement {
            attrs: attrs.iter().copied().collect(),
        }
    }

    fn entry(
        kind: OverlayKind,
        close_on_escape: bool,
        close_on_interact_outside: bool,
    ) -> OverlayEntry {
        // `close_on_interact_outside == true` の呼び出し元では
        // `outside_dismiss_blocks_propagation` は判定に使われないため、
        // kind の既定値をそのまま使う（Tooltip 固有の非遮断挙動を検証する
        // テストは `entry_with_propagation` を使う）。
        OverlayEntry {
            kind,
            close_on_escape,
            close_on_interact_outside,
            outside_dismiss_blocks_propagation: kind
                .outside_dismiss_blocks_propagation_by_default(),
        }
    }

    /// `outside_dismiss_blocks_propagation` を明示指定する版（Tooltip の
    /// スタック非参加 vs 意図的な永続化オプトアウトの区別を検証するテスト
    /// 専用）。
    fn entry_with_propagation(
        kind: OverlayKind,
        close_on_escape: bool,
        close_on_interact_outside: bool,
        outside_dismiss_blocks_propagation: bool,
    ) -> OverlayEntry {
        OverlayEntry {
            kind,
            close_on_escape,
            close_on_interact_outside,
            outside_dismiss_blocks_propagation,
        }
    }

    // --- OverlayKind::from_scope ---

    #[test]
    fn from_scope_recognizes_known_scopes() {
        assert_eq!(OverlayKind::from_scope("dialog"), Some(OverlayKind::Dialog));
        assert_eq!(
            OverlayKind::from_scope("popover"),
            Some(OverlayKind::Popover)
        );
        assert_eq!(OverlayKind::from_scope("menu"), Some(OverlayKind::Menu));
        assert_eq!(
            OverlayKind::from_scope("tooltip"),
            Some(OverlayKind::Tooltip)
        );
        assert_eq!(
            OverlayKind::from_scope("navigation-menu"),
            Some(OverlayKind::NavigationMenu)
        );
        assert_eq!(
            OverlayKind::from_scope("menubar"),
            Some(OverlayKind::Menubar)
        );
    }

    #[test]
    fn from_scope_rejects_unknown_scopes() {
        for bogus in ["drawer", "", "DIALOG", "dialog "] {
            assert_eq!(OverlayKind::from_scope(bogus), None, "scope={bogus:?}");
        }
    }

    // --- 種別既定値 ---

    #[test]
    fn all_kinds_default_close_on_escape_true() {
        for kind in [
            OverlayKind::Dialog,
            OverlayKind::Popover,
            OverlayKind::Menu,
            OverlayKind::Tooltip,
            OverlayKind::NavigationMenu,
            OverlayKind::Menubar,
        ] {
            assert!(kind.close_on_escape());
        }
    }

    #[test]
    fn tooltip_defaults_interact_outside_to_false_others_true() {
        assert!(OverlayKind::Dialog.close_on_interact_outside());
        assert!(OverlayKind::Popover.close_on_interact_outside());
        assert!(OverlayKind::Menu.close_on_interact_outside());
        assert!(!OverlayKind::Tooltip.close_on_interact_outside());
        assert!(OverlayKind::NavigationMenu.close_on_interact_outside());
        assert!(OverlayKind::Menubar.close_on_interact_outside());
    }

    #[test]
    fn navigation_menu_and_menubar_default_outside_dismiss_blocks_propagation_true() {
        // Menu と同じ既定（Tooltip のみ false）であることを直接固定する
        // （イシュー #1173）。
        for kind in [OverlayKind::NavigationMenu, OverlayKind::Menubar] {
            assert!(
                kind.outside_dismiss_blocks_propagation_by_default(),
                "kind={kind:?}"
            );
        }
    }

    // --- opt-out 判定（fail-closed: "false" のみ無効化、他は既定値） ---

    #[test]
    fn close_on_escape_for_defaults_when_attr_absent() {
        let content = element(&[]);
        assert!(close_on_escape_for(OverlayKind::Dialog, &content));
    }

    #[test]
    fn close_on_escape_for_false_disables() {
        let content = element(&[("data-close-on-escape", "false")]);
        assert!(!close_on_escape_for(OverlayKind::Dialog, &content));
    }

    #[test]
    fn close_on_escape_for_invalid_value_falls_back_to_default() {
        for bogus in ["true", "0", "FALSE", ""] {
            let content = element(&[("data-close-on-escape", bogus)]);
            assert!(
                close_on_escape_for(OverlayKind::Dialog, &content),
                "value={bogus:?}"
            );
        }
    }

    #[test]
    fn close_on_interact_outside_for_defaults_when_attr_absent() {
        let content = element(&[]);
        assert!(close_on_interact_outside_for(
            OverlayKind::Popover,
            &content
        ));
        assert!(!close_on_interact_outside_for(
            OverlayKind::Tooltip,
            &content
        ));
    }

    #[test]
    fn close_on_interact_outside_for_false_disables() {
        let content = element(&[("data-close-on-interact-outside", "false")]);
        assert!(!close_on_interact_outside_for(
            OverlayKind::Popover,
            &content
        ));
    }

    #[test]
    fn close_on_interact_outside_for_invalid_value_falls_back_to_default() {
        for bogus in ["true", "0", "FALSE", ""] {
            let content = element(&[("data-close-on-interact-outside", bogus)]);
            assert!(
                close_on_interact_outside_for(OverlayKind::Popover, &content),
                "value={bogus:?}"
            );
        }
    }

    // --- alertdialog 既定上書き（指摘 2: role="alertdialog" は既定で
    // 外側インタラクション閉鎖の対象外とする） ---

    #[test]
    fn close_on_interact_outside_for_alertdialog_defaults_to_false() {
        let content = element(&[("role", "alertdialog")]);
        assert!(
            !close_on_interact_outside_for(OverlayKind::Dialog, &content),
            "role=\"alertdialog\" は外側クリックで閉じない既定であるべき"
        );
    }

    #[test]
    fn close_on_interact_outside_for_plain_dialog_role_keeps_default_true() {
        let content = element(&[("role", "dialog")]);
        assert!(
            close_on_interact_outside_for(OverlayKind::Dialog, &content),
            "role=\"dialog\"（通常ダイアログ）は既定通り外側クリックで閉じる"
        );
    }

    #[test]
    fn close_on_interact_outside_for_alertdialog_explicit_false_still_disables() {
        // 明示 opt-out ("false") は alertdialog 既定と結果が同じだが、
        // 経路として重複していても panic・矛盾しないことを確認する。
        let content = element(&[
            ("role", "alertdialog"),
            ("data-close-on-interact-outside", "false"),
        ]);
        assert!(!close_on_interact_outside_for(
            OverlayKind::Dialog,
            &content
        ));
    }

    #[test]
    fn close_on_interact_outside_for_non_dialog_kind_ignores_alertdialog_role() {
        // role="alertdialog" は Dialog kind 専用の上書きであり、他 kind
        // （改ざん・非対応の組み合わせ）には影響しない。
        let content = element(&[("role", "alertdialog")]);
        assert!(close_on_interact_outside_for(
            OverlayKind::Popover,
            &content
        ));
    }

    // --- outside_dismiss_blocks_propagation_for（指摘 1: スタック非参加
    // と意図的な永続化オプトアウトの区別） ---

    #[test]
    fn outside_dismiss_blocks_propagation_for_tooltip_default_is_false() {
        let content = element(&[]);
        assert!(
            !outside_dismiss_blocks_propagation_for(OverlayKind::Tooltip, &content),
            "Tooltip の既定非参加は下層への伝播を遮断しない"
        );
    }

    #[test]
    fn outside_dismiss_blocks_propagation_for_dialog_default_is_true() {
        let content = element(&[]);
        assert!(outside_dismiss_blocks_propagation_for(
            OverlayKind::Dialog,
            &content
        ));
    }

    #[test]
    fn outside_dismiss_blocks_propagation_for_explicit_opt_out_always_true() {
        let content = element(&[("data-close-on-interact-outside", "false")]);
        for kind in [
            OverlayKind::Dialog,
            OverlayKind::Popover,
            OverlayKind::Menu,
            OverlayKind::Tooltip,
            OverlayKind::NavigationMenu,
            OverlayKind::Menubar,
        ] {
            assert!(
                outside_dismiss_blocks_propagation_for(kind, &content),
                "kind={kind:?}: 明示 opt-out は kind を問わず遮断する"
            );
        }
    }

    // --- NavigationMenu/Menubar の opt-out fail-closed（イシュー #1173）---

    #[test]
    fn close_on_escape_for_navigation_menu_and_menubar_false_disables() {
        let content = element(&[("data-close-on-escape", "false")]);
        assert!(!close_on_escape_for(OverlayKind::NavigationMenu, &content));
        assert!(!close_on_escape_for(OverlayKind::Menubar, &content));
    }

    #[test]
    fn close_on_escape_for_navigation_menu_and_menubar_invalid_value_falls_back_to_default() {
        for bogus in ["true", "0", "FALSE", ""] {
            let content = element(&[("data-close-on-escape", bogus)]);
            assert!(
                close_on_escape_for(OverlayKind::NavigationMenu, &content),
                "value={bogus:?}"
            );
            assert!(
                close_on_escape_for(OverlayKind::Menubar, &content),
                "value={bogus:?}"
            );
        }
    }

    #[test]
    fn close_on_interact_outside_for_navigation_menu_and_menubar_false_disables() {
        let content = element(&[("data-close-on-interact-outside", "false")]);
        assert!(!close_on_interact_outside_for(
            OverlayKind::NavigationMenu,
            &content
        ));
        assert!(!close_on_interact_outside_for(
            OverlayKind::Menubar,
            &content
        ));
    }

    #[test]
    fn close_on_interact_outside_for_navigation_menu_and_menubar_defaults_when_attr_absent() {
        let content = element(&[]);
        assert!(close_on_interact_outside_for(
            OverlayKind::NavigationMenu,
            &content
        ));
        assert!(close_on_interact_outside_for(
            OverlayKind::Menubar,
            &content
        ));
    }

    // --- スタック判定への統合（Dialog の上に NavigationMenu/Menubar が
    // 乗った入れ子、イシュー #1173）---

    #[test]
    fn escape_close_index_targets_topmost_navigation_menu_over_dialog() {
        let stack = [
            entry(OverlayKind::Dialog, true, true),
            entry(OverlayKind::NavigationMenu, true, true),
        ];
        assert_eq!(escape_close_index(&stack), Some(1));
    }

    #[test]
    fn outside_close_indices_dialog_and_menubar_both_close_when_outside() {
        let stack = [
            entry(OverlayKind::Dialog, true, true),
            entry(OverlayKind::Menubar, true, true),
        ];
        let contains_target = [false, false];
        assert_eq!(outside_close_indices(&stack, &contains_target), vec![1, 0]);
    }

    // --- escape_close_index ---

    #[test]
    fn escape_close_index_empty_stack_is_none() {
        assert_eq!(escape_close_index(&[]), None);
    }

    #[test]
    fn escape_close_index_single_entry_is_topmost() {
        let stack = [entry(OverlayKind::Dialog, true, true)];
        assert_eq!(escape_close_index(&stack), Some(0));
    }

    #[test]
    fn escape_close_index_nested_stack_targets_topmost_only() {
        let stack = [
            entry(OverlayKind::Dialog, true, true),
            entry(OverlayKind::Popover, true, true),
        ];
        assert_eq!(escape_close_index(&stack), Some(1));
    }

    #[test]
    fn escape_close_index_topmost_opt_out_does_not_fall_through() {
        let stack = [
            entry(OverlayKind::Dialog, true, true),
            entry(OverlayKind::Popover, false, true),
        ];
        assert_eq!(
            escape_close_index(&stack),
            None,
            "最上位が opt-out の場合は下層へ透過させない"
        );
    }

    // --- outside_close_indices ---

    #[test]
    fn outside_close_indices_all_outside_closes_all_from_top() {
        let stack = [
            entry(OverlayKind::Dialog, true, true),
            entry(OverlayKind::Popover, true, true),
        ];
        let contains_target = [false, false];
        assert_eq!(outside_close_indices(&stack, &contains_target), vec![1, 0]);
    }

    #[test]
    fn outside_close_indices_stops_at_entry_containing_target() {
        let stack = [
            entry(OverlayKind::Dialog, true, true),
            entry(OverlayKind::Popover, true, true),
        ];
        // 最上位（index 1）がターゲットを含む（クリックがその内側）ため、
        // index 0（親）は閉鎖対象に含めない。
        let contains_target = [false, true];
        assert_eq!(
            outside_close_indices(&stack, &contains_target),
            Vec::<usize>::new()
        );
    }

    #[test]
    fn outside_close_indices_stops_at_opt_out_entry() {
        let stack = [
            entry(OverlayKind::Dialog, true, false),
            entry(OverlayKind::Popover, true, true),
        ];
        let contains_target = [false, false];
        // 最上位（index 1）は閉じるが、index 0 は opt-out のため打ち切り、
        // 対象に含めない。
        assert_eq!(outside_close_indices(&stack, &contains_target), vec![1]);
    }

    #[test]
    fn outside_close_indices_mismatched_lengths_returns_empty() {
        let stack = [entry(OverlayKind::Dialog, true, true)];
        assert_eq!(outside_close_indices(&stack, &[]), Vec::<usize>::new());
    }

    #[test]
    fn outside_close_indices_empty_stack_returns_empty() {
        assert_eq!(outside_close_indices(&[], &[]), Vec::<usize>::new());
    }

    // --- 指摘 1 回帰: Tooltip のスタック非参加は下層の閉鎖を妨げない ---

    #[test]
    fn outside_close_indices_tooltip_on_top_does_not_block_dialog_below() {
        let stack = [
            // index 0: 親 Dialog（外側クリックで閉じる）。
            entry_with_propagation(OverlayKind::Dialog, true, true, true),
            // index 1: 最上位 Tooltip。既定 close_on_interact_outside=false、
            // outside_dismiss_blocks_propagation=false（スタック非参加）。
            entry_with_propagation(OverlayKind::Tooltip, true, false, false),
        ];
        let contains_target = [false, false];
        assert_eq!(
            outside_close_indices(&stack, &contains_target),
            vec![0],
            "Tooltip 自身は閉鎖対象に含めないが、下の Dialog は外側クリックで閉じる"
        );
    }

    #[test]
    fn outside_close_indices_persistent_opt_out_still_blocks_below() {
        // 指摘 1 の対比: 明示的な永続化オプトアウト（
        // outside_dismiss_blocks_propagation == true）は Tooltip と異なり、
        // 従来通り下層への伝播を遮断する。
        let stack = [
            entry_with_propagation(OverlayKind::Dialog, true, true, true),
            entry_with_propagation(OverlayKind::Popover, true, false, true),
        ];
        let contains_target = [false, false];
        assert_eq!(
            outside_close_indices(&stack, &contains_target),
            Vec::<usize>::new(),
            "意図的な永続化オプトアウトは下の Dialog も巻き添えで閉じさせない"
        );
    }

    #[test]
    fn outside_close_indices_multiple_non_participating_tooltips_skip_through() {
        // 複数の Tooltip が入れ子でスタックされていても、いずれも走査を
        // 打ち切らず、最下層の Dialog まで到達して閉じる。
        let stack = [
            entry_with_propagation(OverlayKind::Dialog, true, true, true),
            entry_with_propagation(OverlayKind::Tooltip, true, false, false),
            entry_with_propagation(OverlayKind::Tooltip, true, false, false),
        ];
        let contains_target = [false, false, false];
        assert_eq!(outside_close_indices(&stack, &contains_target), vec![0]);
    }
}
