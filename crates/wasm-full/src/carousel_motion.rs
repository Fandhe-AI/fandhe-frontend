//! carousel の Motion+ Carousel 相当拡張（ドラッグ + spring スナップ）配線層
//! （イシュー #2541、親 #2530/#2476、feature `carousel-motion`）。
//!
//! # 責務境界（3 層構成、`docs/design/motion-reference-adoption-policy.md` §6）
//!
//! ポインタ進行度 → 着地 index の計算・spring アニメーションの rAF 実行・
//! DOM 書き込みは [`fandhe_frontend_animation::carousel::CarouselTrack`] の
//! 責務。本モジュールは以下のみを担う:
//!
//! 1. opt-in 属性（[`CAROUSEL_DRAG_ATTR`]）を持つ carousel root への
//!    pointerdown/pointermove/pointerup/pointercancel の委譲登録
//!    （`root` 1 箇所へ 4 リスナー、`gesture.rs` と同じ「登録回数を定数個に
//!    抑える」方針）。
//! 2. `item-group`/先頭 `item` の解決・`getBoundingClientRect()` による
//!    1 スライド分の px 計測。
//! 3. settle 完了時、`on_action`（[`crate::lib::Runtime::wire`] の閉包）へ
//!    `"goto"` を dispatch する。
//!
//! # `data-*` 属性の命名（他クレートとの契約）
//!
//! [`CAROUSEL_DRAG_ATTR`]/[`CAROUSEL_DRAGGING_STATE_ATTR`] のリテラル値は
//! `fandhe_frontend_pre_styled_ui::carousel_motion` の同名定数と一致する
//! 必要がある（`crate::hold_to_confirm`/`crate::magnetic` と同じ「リテラル
//! の写し」契約。wasm-full は pre-styled-ui に依存しないため型共有は
//! できない）。ドリフトは
//! `crates/pre-styled-ui/tests/motion_carousel_css.rs` が本ファイルを
//! 読んで fail-closed に検知する。
//!
//! # `CarouselTrack` の保持責任
//!
//! [`fandhe_frontend_animation::carousel::CarouselTrack`] は settle 中の
//! spring アニメーション（`AnimationLoop`）を自身のフィールドに保持する。
//! settle 完了前に `CarouselTrack` 自体を drop すると `AnimationLoop` も
//! drop され、アニメーションが途中で停止してしまう
//! （`hold_to_confirm.rs::finish_confirmation` doc の「never drop here」と
//! 同じ制約）。本モジュールは carousel root ごとに 1 個の
//! `Rc<RefCell<Option<CarouselTrack>>>`（[`TrackSlot`]）を
//! [`TrackRegistry`] で保持し、次の pointerdown が**同じ** root 上で
//! 上書きするまでは settle 完了後も保持し続ける（`magnetic.rs::
//! ActiveMagnetic` と同型のパターン）。settle 完了コールバック自身は
//! この slot に触れない（tick 実行中に自分自身を drop しないため、
//! `hold_to_confirm.rs` と同じ「never drop from within the tick」規律）。
//!
//! # 複数 carousel 間の状態分離（[`TrackRegistry`]、codex-review/Cursor
//! Bugbot 指摘 是正）
//!
//! `wire_carousel_motion_events` は `Runtime` の mount root 1 個へ
//! 委譲登録する（[`crate::lib::Runtime::wire_carousel_motion`]、1
//! インスタンスにつき 1 回のみ呼ばれる）ため、素朴に単一の
//! `TrackSlot`/`DragMetaSlot` を共有すると、ある carousel の settle
//! アニメーション進行中に**別の** carousel root をドラッグし始めた
//! 時点で前者の状態が上書きされ、`AnimationLoop` が中断される（結果を
//! 待つコールバックが呼ばれないまま `goto` dispatch が失われる）。
//! 是正として [`TrackRegistry`]（carousel root ごとの
//! `(TrackSlot, DragMetaSlot)` を保持する小さな線形テーブル）を導入した。
//! `handle_pointerdown` は `target.closest(CAROUSEL_ROOT_SELECTOR)` で
//! 解決した carousel root をキーに [`slot_for`] を引いてから操作する
//! （pointerdown 自体がまだアクティブな [`DragMeta`] を持たないため
//! pointer_id 検索は使えない）。`handle_pointermove`/
//! `handle_pointer_release` は `event.target()` の部分木解決には頼らず
//! `pointer_id` をキーに [`find_active_slot`] で直接引く（`closest()` に
//! 頼らない理由は [`find_active_slot`] doc 参照）。`Element` は
//! `Hash`/`Eq` を実装しないため両者とも `Node::is_same_node` による線形
//! 探索で同一性判定する（ponytail: 1 ページ内の carousel 数は通常一桁〜
//! 十数個程度に収まる想定の簡略化。件数が実運用で問題になった場合は
//! `js_sys::WeakMap` 等キー付きコレクションへの切替を検討する）。
//!
//! # settle 完了を待たず release 時に即 dispatch する（codex-review/Cursor
//! Bugbot 指摘 是正、イシュー #2541 第 3 ラウンド）
//!
//! 従来は着地 index の `"goto"` dispatch を spring 収束完了時（非同期の
//! `on_settle` コールバック）まで遅延していた。これは 2 つの不具合を招く:
//! (1) 収束中に `next-trigger`/`prev-trigger`/`indicator` を操作して別の
//! index へ `"goto"` した場合、後から収束を終える古い spring が最後に古い
//! index を dispatch し新しい選択を上書きする。(2) 収束中に carousel の
//! DOM 部分木が置換されて root が切断されると、[`slot_for`] の遅延回収が
//! `TrackSlot` ごと `CarouselTrack` を drop し、進行中の spring が二度と
//! 完了しないため意図した着地 index が状態へ一切反映されないまま失われる。
//! [`fandhe_frontend_animation::carousel::CarouselTrack::on_release`] は
//! 着地 index を spring 開始前に同期的に確定して返すため、`"goto"`
//! dispatch は release 時に即実行し、spring は純粋に見た目の追従
//! （`--fandhe-carousel-index` の連続値書き込み）のみを担う設計へ改めた
//! （状態更新は操作確定時に即時、アニメーションは装飾という Motion+ 系の
//! 標準的な責務分離）。これにより (1)(2) とも dispatch 自体は既に完了済み
//! となり、残る懸念は spring が引き続き旧 target へ向けて
//! `--fandhe-carousel-index` を上書きし続ける見た目の競合のみとなるため、
//! [`wire_carousel_motion_events`] の click（capture）ハンドラで
//! `next-trigger`/`prev-trigger`/`indicator` クリックを検知した時点で
//! 該当 carousel root の [`TrackSlot`] を `None` にし進行中の spring を
//! 打ち切る（`CarouselTrack` の `Drop` が `AnimationLoop` を止める）。
//!
//! # ドラッグ確定前は pointer capture しない（codex-review 指摘 是正）
//!
//! `pointerdown` 直後に `item_group.set_pointer_capture()` すると、以降の
//! `pointerup`（ひいては派生する `click`）の実際の発火対象がすべて
//! capture 元の `item_group` へ retarget され、item 内のリンク/ボタンを
//! 「移動なしでタップ」しても `click` がそのリンク/ボタンへ届かなくなる
//! （pointer capture の既知の副作用。ブラウザは compatibility mouse event
//! も capture target へ retarget する）。是正として capture は
//! [`CLICK_GUARD_PX`] を超える実移動を検知した最初の `pointermove` まで
//! 遅延する（`DragMeta::captured` で 1 回のみ実行）。閾値未満のタップは
//! 一切 capture されないため、内部要素の `click` は通常どおり発火する。
//!
//! # capture 喪失時のドラッグ終了回収（codex-review 指摘 是正）
//!
//! `pointerup`/`pointercancel` リスナーは `root` へ委譲登録しているが、
//! pointer capture が（ブラウザ・OS 側の事情や複合ジェスチャー競合で）
//! 暗黙に失われた場合、以降の実イベントは実際のヒットテスト対象（carousel
//! の部分木外を指しうる）へ配信されるため `root` へ届かず、
//! [`DragMeta`]/`data-fandhe-carousel-dragging` 属性が残留し続け、次の
//! `pointerdown` が「進行中のドラッグがある」判定で拒否され続ける
//! （[`handle_pointerdown`] の `drag.borrow().is_some()` ガード参照）。
//! Pointer Events 仕様上 `lostpointercapture` は capture 喪失の理由を
//! 問わず必ず発火し、かつバブルするため、`pointerup`/`pointercancel` と
//! 同じ委譲リスナーへこのイベントも加えて終了処理の取りこぼしを塞ぐ
//! （[`handle_pointer_release`] は `pointer_id` 一致確認後にしか状態を
//! 取り出さないため、正常系の `pointerup` に続いて非同期に発火する
//! `lostpointercapture` が二重に届いても [`find_active_slot`] が `None`
//! を返すだけで安全に no-op になる）。
//!
//! # 複数 carousel の識別（codex-review 指摘 是正、イシュー #2541 第 3
//! ラウンド）
//!
//! `next-trigger`/`prev-trigger`/`indicator` のクリックは
//! `crate::events::wire_events` の `data-action` 属性契約（クリックした
//! 要素自身の属性値をそのままアクション名として使う）で配線されるため、
//! 著者はボタンごとに一意なアクション名（例:
//! `data-action="carousel-hero:goto"`）を付与でき、1 つの `Component`
//! 配下に複数 carousel があっても `decode_action` 側でどの carousel への
//! 操作か判別できる。一方ドラッグ由来の `"goto"` dispatch
//! （[`handle_pointer_release`]）はこの `data-action` 契約を経由せず、
//! 固定文字列 `"goto"` をハードコードしていたため、同じ判別ができなかった
//! （複数 carousel 配下でどの操作か区別不能）。是正として
//! [`CAROUSEL_GOTO_ACTION_ATTR`]（`data-action-carousel-goto`、
//! [`ACTION_INPUT_ATTR`]/[`ACTION_CHANGE_ATTR`] と同型の「著者が明示指定
//! するアクション名属性」契約）を carousel root へ付与できるようにし、
//! 指定があればそれを、無ければ後方互換のため `"goto"` を使う。
//!
//! # 入れ子 carousel の root/item-group 誤結合（codex-review 指摘 是正
//! P1、イシュー #2541 第 5 ラウンド）
//!
//! `handle_pointerdown` は `carousel_root`（[`wiring::CAROUSEL_ROOT_SELECTOR`]、
//! opt-in 属性を持つ最も近い祖先）と `item_group_el`（[`wiring::
//! ITEM_GROUP_SELECTOR`]、最も近い `item-group` 祖先）を独立した 2 回の
//! `closest()` で解決する。opt-in していない carousel が opt-in している
//! 別の carousel の `item` 内へ丸ごと入れ子で存在する構成では、内側
//! carousel の要素を操作した際にこの 2 つの `closest()` が異なる
//! carousel（外側の opt-in root と内側の item-group）を指してしまい、
//! 内側の操作が外側の `TrackSlot`/`slide_count` を借用して誤った
//! `"goto"` を dispatch する。是正として `item_group_el` の anatomy 上の
//! 最も近い carousel root（opt-in の有無を問わない
//! `[data-scope="carousel"][data-part="root"]`）を別途解決し、
//! `carousel_root` と一致しない場合は no-op にする。

/// opt-in（著者が SSR 出力に静的に付与）: root へ付与するとドラッグ +
/// spring スナップを有効化するマーカー属性。値は `""`（非 loop）または
/// `"loop"`（末尾からの折り返し）のみを意味を持ち、それ以外は `""` 扱い
/// （[`parse_loop_opt_in`] 参照）。
pub const CAROUSEL_DRAG_ATTR: &str = "data-fandhe-carousel-drag";

/// 状態属性: ドラッグ中〜spring 収束完了まで carousel root へ付与される。
pub const CAROUSEL_DRAGGING_STATE_ATTR: &str = "data-fandhe-carousel-dragging";

/// opt-in（著者が SSR 出力に静的に付与、任意）: ドラッグ確定時に dispatch
/// する `"goto"` の代わりに使うアクション名（モジュール doc「複数
/// carousel の識別」節参照）。未指定・空文字列は既定の `"goto"` を使う
/// （後方互換）。`crate::events::ACTION_INPUT_ATTR`/`ACTION_CHANGE_ATTR`
/// と同じ「著者がアクション名を明示指定する属性」契約であり、値の
/// 妥当性検証は行わない（`fandhe_frontend_interactive::Component::
/// decode_action` 側の責務、本クレートの不変条件 4「未知のアクション名は
/// no-op」を前提とする）。
pub const CAROUSEL_GOTO_ACTION_ATTR: &str = "data-action-carousel-goto";

/// [`CAROUSEL_DRAG_ATTR`] の属性値から loop モードを決める。属性が存在
/// しない（`None`）場合のみ opt-in 自体が無効（呼び出し側は属性の有無を
/// 別途 `closest()` で判定する）。値が `"loop"` なら `true`、それ以外
/// （`""` を含む）はすべて `false` 扱い（不正な opt-in 値は `""` 扱い、
/// 実装計画 §2.3 のセキュリティ方針）。
#[must_use]
pub fn parse_loop_opt_in(attr_value: Option<&str>) -> Option<bool> {
    attr_value.map(|value| value == "loop")
}

/// `item-group` の `data-orientation` 属性値から、ドラッグ軸が縦方向
/// （`client_y` を使う）かどうかを決める。`"vertical"` のみ縦方向、それ
/// 以外（未設定含む）は横方向。
#[must_use]
pub fn is_vertical_orientation(orientation: Option<&str>) -> bool {
    orientation == Some("vertical")
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use std::cell::RefCell;
    use std::rc::Rc;

    use fandhe_frontend_animation::carousel::CarouselTrack;
    use web_sys::{HtmlElement, PointerEvent};

    use super::{
        is_vertical_orientation, parse_loop_opt_in, CAROUSEL_DRAGGING_STATE_ATTR,
        CAROUSEL_DRAG_ATTR, CAROUSEL_GOTO_ACTION_ATTR,
    };
    use crate::dom::set_dom_attribute_result as set_dom_attribute;
    use crate::events::ActionRef;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event};

    /// `[data-fandhe-carousel-drag]` を `closest()` で辿るためのセレクタ。
    const CAROUSEL_ROOT_SELECTOR: &str = "[data-fandhe-carousel-drag]";
    /// carousel の anatomy root（opt-in 属性の有無を問わない、
    /// `crates/headless-ui/src/carousel.rs::root` が必ず出力する）を
    /// `closest()` で辿るためのセレクタ。[`handle_pointerdown`] が
    /// [`CAROUSEL_ROOT_SELECTOR`] と組み合わせて「入れ子 carousel の
    /// root/item-group 誤結合」（codex-review 指摘 是正 P1、イシュー
    /// #2541 第 5 ラウンド）を検知するために使う。
    const ANATOMY_ROOT_SELECTOR: &str = "[data-scope=\"carousel\"][data-part=\"root\"]";
    /// `item-group` を `closest()` で辿るためのセレクタ
    /// （`crates/headless-ui/src/carousel.rs` の ANATOMY `data-scope`/
    /// `data-part` と一致）。
    const ITEM_GROUP_SELECTOR: &str = "[data-scope=\"carousel\"][data-part=\"item-group\"]";
    /// `item-group` 配下の `item` 一覧を数える・先頭要素を計測するための
    /// セレクタ。`:scope >` で直接の子のみに限定する（Cursor Bugbot 指摘
    /// 是正「ネスト carousel で slide_count 誤検知」、イシュー #2541 第 4
    /// ラウンド）。anatomy 上 `item` は `item-group` の直接の子である
    /// （`crates/headless-ui/src/carousel.rs` 参照）ため、スコープなしの
    /// `query_selector_all` は「carousel の `item` の中に、別の carousel
    /// が丸ごと入れ子で存在する」構成で内側 carousel の `item` まで
    /// 巻き込んで数えてしまい、`slide_count` の誤検知・計測対象
    /// （`first_item`）の誤接続を招いていた。
    const ITEM_SELECTOR: &str = ":scope > [data-scope=\"carousel\"][data-part=\"item\"]";
    /// `next-trigger`/`prev-trigger`/`indicator` を `closest()` で判定する
    /// ためのセレクタ（モジュール doc「settle 完了を待たず release 時に
    /// 即 dispatch する」節参照）。クリックでこれら操作 UI が押された
    /// 場合、進行中の spring（旧 target への視覚的な上書き）を打ち切る。
    const NAV_TRIGGER_SELECTOR: &str = concat!(
        "[data-scope=\"carousel\"][data-part=\"next-trigger\"],",
        "[data-scope=\"carousel\"][data-part=\"prev-trigger\"],",
        "[data-scope=\"carousel\"][data-part=\"indicator\"]",
    );

    /// [`CarouselTrack`] の保持スロット（モジュール doc「`CarouselTrack`
    /// の保持責任」節参照）。
    type TrackSlot = Rc<RefCell<Option<CarouselTrack>>>;

    /// `carousel_root` が `root`（mount root）配下の [`CAROUSEL_ROOT_SELECTOR`]
    /// 一致要素の中で何番目か（文書順）を返す（[`replacement_item_group`]
    /// と対で「再描画後も同じ位置の carousel」を突き合わせるための位置
    /// ヒューリスティック。モジュール doc「release 時の再描画後も表示中の
    /// DOM で spring を継続する」節参照）。要素の同一性ではなく位置で
    /// 突き合わせるのは、`Runtime::apply_subtree_swap` が `root` 配下を
    /// 丸ごと新規ノードへ差し替えるため旧要素と新要素の間に DOM
    /// レベルの同一性が一切残らないため（`drag_gesture.rs::
    /// resync_drag_gesture_attachments` が新規要素を「常に新規」として
    /// 状態を諦めて再 attach するのと同じ制約下にある）。
    fn position_of_carousel_root(root: &Element, carousel_root: &Element) -> Option<usize> {
        let node_list = root.query_selector_all(CAROUSEL_ROOT_SELECTOR).ok()?;
        (0..node_list.length())
            .find(|&i| {
                node_list
                    .get(i)
                    .and_then(|node| node.dyn_into::<Element>().ok())
                    .is_some_and(|el| el.is_same_node(Some(carousel_root.as_ref())))
            })
            .map(|i| i as usize)
    }

    /// [`position_of_carousel_root`] で求めた位置 `index` に対応する、
    /// 再描画後の carousel root/`item-group` の組を返す（同じ位置に
    /// carousel が無くなっていれば `None`、fail-safe）。
    fn replacement_item_group(root: &Element, index: usize) -> Option<(Element, HtmlElement)> {
        let node_list = root.query_selector_all(CAROUSEL_ROOT_SELECTOR).ok()?;
        let new_root = node_list
            .get(index.try_into().ok()?)
            .and_then(|node| node.dyn_into::<Element>().ok())?;
        let new_item_group = new_root
            .query_selector(ITEM_GROUP_SELECTOR)
            .ok()
            .flatten()
            .and_then(|el| el.dyn_into::<HtmlElement>().ok())?;
        Some((new_root, new_item_group))
    }

    /// `old_carousel_root` の再描画後の置き換え先（carousel root/
    /// `item-group` の組）を解決する。`old_carousel_root` が `root`
    /// （`wire_carousel_motion_events` の mount root）自身と同じ要素の
    /// 場合は特別扱いする: `Runtime::apply_subtree_swap` は `root` 自身は
    /// 保持したまま子ノードのみを丸ごと差し替えるため、`root` 自身は
    /// `query_selector_all(CAROUSEL_ROOT_SELECTOR)`（`root` 自身を含まず
    /// 子孫のみを走査する）では見つからず、かつ常に `is_connected()` の
    /// ままである（モジュール doc「release 時の再描画後も表示中の DOM で
    /// spring を継続する」節参照）。この場合は位置ヒューリスティックを
    /// 使わず、`root` 自身を新 carousel root として `item-group` のみを
    /// 再解決する。
    /// `item_group` 配下（[`ITEM_SELECTOR`]、直接の子のみ）の枚数を返す。
    fn slide_count_of(item_group: &Element) -> usize {
        item_group
            .query_selector_all(ITEM_SELECTOR)
            .map(|list| list.length() as usize)
            .unwrap_or(0)
    }

    /// `old_carousel_root` の再描画後の置き換え先を解決する。
    ///
    /// # 文書順の位置ヒューリスティックが指す先を枚数で検証する
    /// （codex-review 指摘 是正 P1、イシュー #2541 第 6 ラウンド）
    ///
    /// [`position_of_carousel_root`]/[`replacement_item_group`] は要素の
    /// 同一性ではなく**文書順の位置**のみで再描画後の carousel を
    /// 突き合わせる（`root` 自身が old root の特別扱いを除く）。dispatch
    /// した `"goto"` が carousel 自体の追加・削除・並べ替えを伴う構成
    /// （例: 一覧の carousel をフィルタで絞り込む）では、同じ位置に
    /// **別の carousel** が現れてしまい、無関係な carousel へ進行中の
    /// spring を retarget してしまう。`old_carousel_root` に紐づいていた
    /// `expected_slide_count`（pointerdown 時点で計測した枚数）と解決先
    /// `item-group` の実際の枚数が食い違えば「別 carousel」と判定し
    /// `None` を返す（既存の「見つからなければ spring を打ち切ってリーク
    /// を防ぐ」フォールバックへ合流させる、完全な識別ではなく明白な
    /// 食い違いだけを検知する fail-safe）。
    fn resolve_replacement(
        root: &Element,
        old_carousel_root: &Element,
        position: Option<usize>,
        expected_slide_count: usize,
    ) -> Option<(Element, HtmlElement)> {
        let (new_root, new_item_group) = if old_carousel_root.is_same_node(Some(root.as_ref())) {
            let new_item_group = root
                .query_selector(ITEM_GROUP_SELECTOR)
                .ok()
                .flatten()
                .and_then(|el| el.dyn_into::<HtmlElement>().ok())?;
            (root.clone(), new_item_group)
        } else {
            replacement_item_group(root, position?)?
        };
        if slide_count_of(&new_item_group) != expected_slide_count {
            return None;
        }
        Some((new_root, new_item_group))
    }

    /// アクティブなドラッグの付随情報（[`CarouselTrack`] 本体とは別に
    /// pointerdown〜release の間だけ保持する）。
    struct DragMeta {
        pointer_id: i32,
        carousel_root: Element,
        item_group: HtmlElement,
        slide_px: f64,
        vertical: bool,
        // 再描画後の carousel 識別（[`resolve_replacement`] doc「文書順の
        // 位置ヒューリスティックが指す先を枚数で検証する」節参照、
        // イシュー #2541 第 6 ラウンド）。
        slide_count: usize,
        origin_coord: f64,
        moved: bool,
        /// `item_group.set_pointer_capture()` を実行済みかどうか（モジュール
        /// doc「ドラッグ確定前は pointer capture しない」節参照）。
        captured: bool,
    }

    type DragMetaSlot = Rc<RefCell<Option<DragMeta>>>;

    /// carousel root ごとの [`TrackSlot`]/[`DragMetaSlot`] を保持する
    /// レジストリ（モジュール doc「複数 carousel 間の状態分離」節参照）。
    type TrackRegistry = Rc<RefCell<Vec<(Element, TrackSlot, DragMetaSlot)>>>;

    /// `registry` から `carousel_root` に対応する `(TrackSlot,
    /// DragMetaSlot)` を引く。未登録なら新規に確保して登録する
    /// （carousel root の同一性は `Node::is_same_node` で判定する、
    /// モジュール doc「複数 carousel 間の状態分離」節参照）。
    ///
    /// # 切り離された root の遅延回収（codex-review 指摘 是正、イシュー
    /// #2541 第 2 ラウンド）
    ///
    /// `Runtime::apply_subtree_swap` 等で carousel の DOM が置換される
    /// と、旧 root は文書から切断される（`Element::is_connected()` が
    /// 偽になる）が、`TrackRegistry` は強参照で保持し続けるため、置換の
    /// たびにエントリが積み上がり `TrackSlot` の `CarouselTrack`
    /// （settle 中の `AnimationLoop` を含む）ごと解放されずに残ってしまう
    /// （メモリリーク・線形探索の劣化）。是正として、毎回の解決前に
    /// 切断済みエントリを間引く（`crates/wasm-full/src/drag_gesture.rs::
    /// controller_for_tracking_reuse` と同型の遅延掃除、明示的な
    /// unmount フックを持たない設計を踏襲）。間引かれたエントリの
    /// `TrackSlot`（他に参照を持つ呼び出し元がいなければ）が drop
    /// されると `CarouselTrack` も drop され、進行中の `AnimationLoop`
    /// も `Drop` で停止する（`carousel.rs` モジュール doc「`Carousel
    /// Track` を drop すると...」節参照）。
    fn slot_for(registry: &TrackRegistry, carousel_root: &Element) -> (TrackSlot, DragMetaSlot) {
        let mut entries = registry.borrow_mut();
        entries.retain(|(root, _, _)| root.is_connected());
        if let Some((_, track, drag)) = entries
            .iter()
            .find(|(root, _, _)| root.is_same_node(Some(carousel_root.as_ref())))
        {
            return (track.clone(), drag.clone());
        }
        let track: TrackSlot = Rc::new(RefCell::new(None));
        let drag: DragMetaSlot = Rc::new(RefCell::new(None));
        entries.push((carousel_root.clone(), track.clone(), drag.clone()));
        (track, drag)
    }

    /// `registry` 内で `old_root` をキーに持つエントリを `new_root` へ
    /// 差し替える（codex-review 指摘 是正「再描画後の carousel root へ
    /// レジストリの保持先も移す」、イシュー #2541）。
    ///
    /// [`CarouselTrack::retarget`] は書き込み先の DOM 要素を差し替える
    /// だけで、`TrackRegistry` のキー（旧 `carousel_root`）はそのまま
    /// 切断済み要素を指し続ける。次に別の carousel が [`slot_for`] を
    /// 呼ぶと、旧キーの `is_connected() == false` により当該エントリが
    /// `retain` で間引かれ、進行中の spring を保持する [`TrackSlot`]
    /// （この呼び出し内のローカル変数以外に参照を持たない）が丸ごと
    /// drop されて `AnimationLoop` が中断してしまう。retarget と対で
    /// 必ずキーも新 root へ移すことで、以降の `slot_for`/`find_active_slot`
    /// が新 root 上の操作から同じ `TrackSlot` を引けるようにする。
    fn rekey_carousel_root(registry: &TrackRegistry, old_root: &Element, new_root: &Element) {
        let mut entries = registry.borrow_mut();
        if let Some(entry) = entries
            .iter_mut()
            .find(|(root, _, _)| root.is_same_node(Some(old_root.as_ref())))
        {
            entry.0 = new_root.clone();
        }
    }

    /// `next-trigger`/`prev-trigger`/`indicator`（[`NAV_TRIGGER_SELECTOR`]）
    /// のクリックを検知したら、その carousel root の進行中の spring
    /// （settle 未完了の [`CarouselTrack`]）を打ち切る（モジュール doc
    /// 「settle 完了を待たず release 時に即 dispatch する」節参照）。
    /// `"goto"` dispatch は release 時に既に完了済みのため、ここでの
    /// 打ち切りは純粋に見た目の競合（旧 target への上書き）を止める
    /// だけであり、状態には一切影響しない。opt-in root が見つからない・
    /// nav trigger 以外のクリックはいずれも no-op。
    fn invalidate_settle_on_nav_trigger_click(event: &Event, registry: &TrackRegistry) {
        let Some(target) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
            return;
        };
        let Ok(Some(carousel_root)) = target.closest(CAROUSEL_ROOT_SELECTOR) else {
            return;
        };
        if target
            .closest(NAV_TRIGGER_SELECTOR)
            .ok()
            .flatten()
            .is_none()
        {
            return;
        }
        let (track, _) = slot_for(registry, &carousel_root);
        let mut track_guard = track.borrow_mut();
        if let Some(mut t) = track_guard.take() {
            // 打ち切る前に確定 index を DOM へ即座に書き戻す（codex-review
            // 指摘 是正、イシュー #2541 第 6 ラウンド「同一 index への
            // no-op 更新だと途中の小数 progress が復元されない」）。
            // 続く action dispatch が状態を変えない no-op でも、この
            // 書き込みだけで表示は確定 index と一致する。
            t.cancel_and_snap();
            let _ = carousel_root.remove_attribute(CAROUSEL_DRAGGING_STATE_ATTR);
        }
    }

    /// ドラッグ後の合成 click を抑止するためのフラグ（5px 超の移動が
    /// あった場合のみ次の 1 回の click を capture フェーズで止める）。
    type SuppressClickFlag = Rc<std::cell::Cell<bool>>;

    /// 移動量がこれを超えたら「ドラッグ」とみなし、release 直後の click
    /// を抑止する（トリガー/インジケータの誤発火防止）。
    const CLICK_GUARD_PX: f64 = 5.0;

    /// `root` へポインタイベント 4 種・click（capture）を委譲登録する。
    ///
    /// `on_action` は settle 完了時の `"goto"` dispatch 依頼を呼び出し側
    /// （`crate::lib::Runtime::wire_carousel_motion`）へ渡す
    /// （`angle_slider::wire_angle_slider_events` と同型）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback[_and_bool]` の失敗を伝播する。
    pub fn wire_carousel_motion_events(
        root: Element,
        on_action: impl FnMut(ActionRef) + 'static,
    ) -> Result<(), JsValue> {
        let on_action = Rc::new(RefCell::new(on_action));
        let registry: TrackRegistry = Rc::new(RefCell::new(Vec::new()));
        let suppress_click: SuppressClickFlag = Rc::new(std::cell::Cell::new(false));

        let pointerdown_registry = registry.clone();
        let pointerdown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointerdown(&event, &pointerdown_registry);
        });
        root.add_event_listener_with_callback(
            "pointerdown",
            pointerdown_closure.as_ref().unchecked_ref(),
        )?;
        pointerdown_closure.forget();

        // `pointermove` は `root` ではなく `window` にのみ登録する
        // （codex-review 指摘 是正「同一イベントの二重処理」、イシュー
        // #2541 第 4 ラウンド）。以前は `root`/`window` 双方に同じ closure
        // を登録していたが、`root` 内で発生した `pointermove` はバブル
        // フェーズで `root`（1 回目）→ `window`（2 回目）の順に同一
        // closure を 2 回呼び出す。`CarouselTrack::on_pointer_move` は
        // 呼ばれるたびに `previous_sample`/`latest_sample` を更新する
        // ため、2 回目の呼び出しでは両サンプルが同一座標・同一時刻になり
        // `estimate_velocity` が常に 0 を返し、通常のフリック操作で速度に
        // 応じた着地判定が機能しなくなっていた。`window` は文書内の任意の
        // 要素で発生したイベントも常にバブルで受け取る（`root` が
        // `window` の子孫であるため `root` 単独の登録は冗長かつ有害）ため、
        // pointer capture 確定前に `root` の外へ出たフリックの取りこぼし
        // （モジュール doc「ドラッグ確定前は pointer capture しない」節）
        // も `window` 単独の登録で引き続き防げる。
        let pointermove_registry = registry.clone();
        let pointermove_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointermove(&event, &pointermove_registry);
        });
        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback(
                "pointermove",
                pointermove_closure.as_ref().unchecked_ref(),
            );
        }
        pointermove_closure.forget();

        // `pointerup`/`pointercancel`/`lostpointercapture` は `root` に加え
        // `window` にも登録する（`drag_gesture.rs::wire_drag_gesture` doc
        // 「`pointerup`/`pointercancel` は `window` にも登録する」節と同じ
        // 是正、モジュール doc「capture 喪失時のドラッグ終了回収」節参照）。
        // pointer capture は移動閾値を超えるまで確定しないため（モジュール
        // doc「ドラッグ確定前は pointer capture しない」節）、確定前に
        // carousel root の外へポインタが出て release されると、`root`
        // 単独の委譲登録では実イベントの配送先（実際のヒットテスト対象）が
        // `root` の祖先でなくなり取りこぼす。`handle_pointer_release` は
        // `pointer_id` の一致確認後にしか状態を取り出さないため、`root`/
        // `window` 双方から同じイベントが届いても（`root` が `window` の
        // 子孫であるため両方から呼ばれる場合がある）2 回目は
        // `find_active_slot` が `None` を返すだけで安全に no-op になる
        // （`release_drag` の冪等性契約と同型）。`window()` が取得できない
        // 非ブラウザ環境（native テスト等）では `root` 側のみで動作し続ける
        // fail-safe（`let _ =` で結果を握り潰す）。
        for event_name in ["pointerup", "pointercancel", "lostpointercapture"] {
            let release_root = root.clone();
            let release_registry = registry.clone();
            let release_on_action = on_action.clone();
            let release_suppress = suppress_click.clone();
            let release_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
                handle_pointer_release(
                    &event,
                    &release_root,
                    &release_registry,
                    &release_on_action,
                    &release_suppress,
                );
            });
            root.add_event_listener_with_callback(
                event_name,
                release_closure.as_ref().unchecked_ref(),
            )?;
            if let Some(window) = web_sys::window() {
                let _ = window.add_event_listener_with_callback(
                    event_name,
                    release_closure.as_ref().unchecked_ref(),
                );
            }
            release_closure.forget();
        }

        let click_suppress = suppress_click;
        let click_registry = registry;
        let click_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            if click_suppress.replace(false) {
                event.stop_propagation();
                event.prevent_default();
                // 抑止した click は「ドラッグ release が偶然 nav trigger の
                // 上で起きただけ」であり、著者が意図した nav trigger 操作
                // ではない（Cursor Bugbot 指摘 是正「Suppressed click still
                // cancels spring」）。この click 由来で
                // invalidate_settle_on_nav_trigger_click を呼ぶと、
                // ドラッグ確定によって release 時に既に dispatch 済みの
                // `"goto"` の spring（見た目の追従のみ）を無関係に打ち切って
                // しまう。抑止した click はここで終端し、以降の判定へは
                // 渡さない。
                return;
            }
            invalidate_settle_on_nav_trigger_click(&event, &click_registry);
        });
        root.add_event_listener_with_callback_and_bool(
            "click",
            click_closure.as_ref().unchecked_ref(),
            true,
        )?;
        click_closure.forget();

        Ok(())
    }

    /// carousel root（[`CAROUSEL_ROOT_SELECTOR`]）配下・`item-group`
    /// （[`ITEM_GROUP_SELECTOR`]）を解決し、`item` 数・1 スライド分の px を
    /// 計測してドラッグセッションを開始する。opt-in root/`item-group` が
    /// 見つからない・`data-disabled` 祖先を持つ・`item` が 0 件・計測結果
    /// が非正/非有限のいずれかは no-op（fail-safe）。
    fn handle_pointerdown(event: &Event, registry: &TrackRegistry) {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        // 主ボタン（`button() == 0`。タッチ/ペンの接触も 0）かつ最初の
        // 接触点（`is_primary()`）のみドラッグを開始する（codex-review 指摘
        // 是正「右クリック・中クリックでドラッグを開始しない」、
        // `drag_gesture.rs::handle_pointerdown` Bugbot「Non-primary buttons
        // start drags」と同型。右クリックのコンテキストメニュー・中クリック
        // のページ操作と競合しつつ `suppress_click` が無関係な click まで
        // 抑止してしまう不具合の是正）。
        if pointer_event.button() != 0 || !pointer_event.is_primary() {
            return;
        }
        let Some(target) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
            return;
        };
        let Ok(Some(carousel_root)) = target.closest(CAROUSEL_ROOT_SELECTOR) else {
            return;
        };
        if carousel_root
            .closest("[data-disabled]")
            .ok()
            .flatten()
            .is_some()
        {
            return;
        }
        let Ok(Some(item_group_el)) = target.closest(ITEM_GROUP_SELECTOR) else {
            return;
        };
        // `carousel_root`（[`CAROUSEL_ROOT_SELECTOR`]、opt-in 属性を持つ
        // 祖先）と `item_group_el`（[`ITEM_GROUP_SELECTOR`]、最も近い
        // item-group 祖先）は独立した 2 回の `closest()` で解決するため、
        // opt-in していない carousel が opt-in している carousel の
        // `item` 内へ入れ子で存在する構成では、`target` が内側 carousel
        // 上にある場合に `carousel_root` は外側（内側に opt-in 属性が
        // 無いため `closest()` が外側まで辿り着く）、`item_group_el` は
        // 内側（`item-group` は内側の方が近い）という食い違った組が
        // 得られてしまう（codex-review 指摘 是正 P1「入れ子 carousel の
        // root/item-group 誤結合」、イシュー #2541 第 5 ラウンド）。
        // `item_group_el` の**anatomy 上の**最も近い carousel root
        // （opt-in の有無を問わない [`ANATOMY_ROOT_SELECTOR`]）を
        // `carousel_root` と突き合わせ、一致しない場合は
        // `item_group_el` が別の（opt-in していない）carousel に属する
        // と判定して no-op にする。
        let Ok(Some(owning_root)) = item_group_el.closest(ANATOMY_ROOT_SELECTOR) else {
            return;
        };
        if !owning_root.is_same_node(Some(carousel_root.as_ref())) {
            return;
        }
        let Ok(item_group) = item_group_el.clone().dyn_into::<HtmlElement>() else {
            return;
        };
        let loop_ = parse_loop_opt_in(carousel_root.get_attribute(CAROUSEL_DRAG_ATTR).as_deref())
            .unwrap_or(false);
        let vertical =
            is_vertical_orientation(item_group.get_attribute("data-orientation").as_deref());

        let Ok(items) = item_group_el.query_selector_all(ITEM_SELECTOR) else {
            return;
        };
        let slide_count = items.length() as usize;
        if slide_count == 0 {
            return;
        }
        let Some(first_item) = items
            .get(0)
            .and_then(|node| node.dyn_into::<HtmlElement>().ok())
        else {
            return;
        };
        // coverflow は item へ rotateY/translateZ 等の transform を適用する
        // （`crates/pre-styled-ui/src/carousel_motion.rs` 参照）ため、
        // `getBoundingClientRect()` は変形後の見かけ上の寸法を返し、現在
        // スライドによって計測値がぶれる（codex-review 指摘 是正）。
        // `offsetWidth`/`offsetHeight` は CSS `transform` の影響を受けない
        // レイアウト寸法（border 込み・margin 抜き）であるため、こちらを
        // 1 スライド分の基準として使う。
        let slide_px = f64::from(if vertical {
            first_item.offset_height()
        } else {
            first_item.offset_width()
        });
        if !slide_px.is_finite() || slide_px <= 0.0 {
            return;
        }

        let (track, drag) = slot_for(registry, &carousel_root);
        // 進行中のドラッグ（別ポインタの pointerdown）がある間は新規
        // pointerdown を無視する（codex-review 指摘 是正）。既存の
        // pointer_id を確認せず無条件に上書きすると、複数指の同時
        // pointerdown で操作中の指から別の指へ黙って乗っ取られてしまう。
        if drag.borrow().is_some() {
            return;
        }

        // pointer capture はまだ行わない（モジュール doc「ドラッグ確定前は
        // pointer capture しない」節参照）。移動閾値を超えた最初の
        // `pointermove` で `captured` を立てて実行する。
        let coord = if vertical {
            f64::from(pointer_event.client_y())
        } else {
            f64::from(pointer_event.client_x())
        };
        // 既存 `CarouselTrack` があれば破棄せず再利用する（codex-review
        // 指摘 是正「収束中タップで確定 index が失われる」、イシュー
        // #2541 第 4 ラウンド）。毎回 `CarouselTrack::attach` で新規
        // 生成すると `last_target`（直前に確定した着地 index）が失われ、
        // settle 未完了の状態へ移動なしのタップが来た際
        // （`handle_pointer_release` の `on_tap_release`）確定済みの
        // 着地先を復元できなくなる。
        let mut track_guard = track.borrow_mut();
        match track_guard.as_mut() {
            Some(existing) => {
                existing.resume_pointer_down(
                    item_group.clone(),
                    slide_count,
                    loop_,
                    coord,
                    event.time_stamp(),
                );
            }
            None => {
                let mut new_track = CarouselTrack::attach(item_group.clone(), slide_count, loop_);
                new_track.on_pointer_down(coord, event.time_stamp());
                *track_guard = Some(new_track);
            }
        }
        drop(track_guard);

        let _ = set_dom_attribute(&carousel_root, CAROUSEL_DRAGGING_STATE_ATTR, "");
        *drag.borrow_mut() = Some(DragMeta {
            pointer_id: pointer_event.pointer_id(),
            carousel_root,
            item_group,
            slide_px,
            vertical,
            slide_count,
            origin_coord: coord,
            moved: false,
            captured: false,
        });
    }

    /// `registry` 内の全 carousel root を横断し、`pointer_id` に一致する
    /// アクティブな [`DragMeta`] を持つ 1 件を探す
    /// （[`TrackSlot`]/[`DragMetaSlot`] の組を返す）。
    ///
    /// # `event.target()` からの `closest()` 解決をやめた理由（Cursor
    /// Bugbot 指摘 是正）
    ///
    /// 当初は `event.target()` を `CAROUSEL_ROOT_SELECTOR` で `closest()`
    /// して carousel root を解決していたが、`set_pointer_capture` が
    /// 失敗した場合やキャプチャが失われた場合（ブラウザ実装・OS 側の
    /// 事情でキャプチャが外れることがある）、`pointerup`/`pointercancel`
    /// は実際にポインタ直下にある要素へ配信されるため、carousel の DOM
    /// 部分木の外（`<body>` 等）で release されると `closest()` が
    /// `None` を返し、settle も `data-fandhe-carousel-dragging` 属性の
    /// 除去も一切実行されないまま状態が残留してしまっていた。
    /// アクティブな [`DragMeta`] は `registry` が pointer_id ごとに保持
    /// しているため、event.target() に頼らず pointer_id で直接引けば
    /// この問題は起きない。
    fn find_active_slot(
        registry: &TrackRegistry,
        pointer_id: i32,
    ) -> Option<(TrackSlot, DragMetaSlot)> {
        let entries = registry.borrow();
        entries.iter().find_map(|(_, track, drag)| {
            let is_match = drag
                .borrow()
                .as_ref()
                .is_some_and(|meta| meta.pointer_id == pointer_id);
            is_match.then(|| (track.clone(), drag.clone()))
        })
    }

    fn handle_pointermove(event: &Event, registry: &TrackRegistry) {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        let Some((track, drag)) = find_active_slot(registry, pointer_event.pointer_id()) else {
            return;
        };
        let mut drag_guard = drag.borrow_mut();
        let Some(meta) = drag_guard.as_mut() else {
            return;
        };
        let coord = if meta.vertical {
            f64::from(pointer_event.client_y())
        } else {
            f64::from(pointer_event.client_x())
        };
        if (coord - meta.origin_coord).abs() > CLICK_GUARD_PX {
            meta.moved = true;
            // 移動閾値を超えた最初の pointermove でのみ capture する
            // （モジュール doc「ドラッグ確定前は pointer capture しない」
            // 節参照）。閾値未満のタップは capture されないため内部要素の
            // click が通常どおり発火する。
            if !meta.captured {
                let _ = meta
                    .item_group
                    .set_pointer_capture(pointer_event.pointer_id());
                meta.captured = true;
            }
        }
        let slide_px = meta.slide_px;
        if let Some(t) = track.borrow_mut().as_mut() {
            t.on_pointer_move(coord, slide_px, event.time_stamp());
        };
    }

    /// pointerup/pointercancel 共通ハンドラ。`drag` の `pointer_id` が
    /// このイベントと一致することを確認してから取り出し settle を
    /// 開始する（settle 完了時に `"goto"` を dispatch する）。
    ///
    /// # pointer_id を確認してから `take()` する理由（codex-review/Cursor
    /// Bugbot 指摘 是正）
    ///
    /// 当初は `drag_guard.take()` を pointer_id 確認より先に実行していた
    /// ため、マルチタッチ環境で無関係な別指の `pointerup`/`pointercancel`
    /// が先に配信されると、アクティブなドラッグの [`DragMeta`] が
    /// pointer_id 不一致にもかかわらず失われてしまい、本来のドラッグ
    /// 指の `pointerup` が届いても settle が開始されない不具合があった。
    /// 是正として `as_ref()` で照合してから一致した場合のみ `take()`
    /// する。
    ///
    /// # pointercancel では合成 click を抑止しない理由（codex-review/
    /// Cursor Bugbot 指摘 是正）
    ///
    /// `pointercancel` はブラウザ仕様上、続く合成 `click` イベントを
    /// 発火**しない**（`pointerup` のみが click の起点になる）。当初は
    /// event 種別を区別せず `moved` に応じて `suppress_click` を常に
    /// 立てていたため、cancel 後は誰も消費しない抑止フラグが残留し、
    /// 全く無関係な次の `click`（別の trigger/indicator クリック等）まで
    /// 誤って止めてしまっていた。是正として `pointerup` のときのみ
    /// `suppress_click` を設定する。
    fn handle_pointer_release(
        event: &Event,
        root: &Element,
        registry: &TrackRegistry,
        on_action: &Rc<RefCell<impl FnMut(ActionRef) + 'static>>,
        suppress_click: &SuppressClickFlag,
    ) {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        let Some((track, drag)) = find_active_slot(registry, pointer_event.pointer_id()) else {
            return;
        };
        let mut drag_guard = drag.borrow_mut();
        let meta = drag_guard
            .take()
            .expect("find_active_slot が Some を確認済み");
        drop(drag_guard);

        if meta.captured {
            let _ = meta
                .item_group
                .release_pointer_capture(pointer_event.pointer_id());
        }
        if event.type_() == "pointerup" {
            suppress_click.set(meta.moved);
        }

        let mut track_guard = track.borrow_mut();
        let Some(t) = track_guard.as_mut() else {
            let _ = meta
                .carousel_root
                .remove_attribute(CAROUSEL_DRAGGING_STATE_ATTR);
            return;
        };
        // 着地 index は spring 開始前に同期的に確定するため、`"goto"`
        // dispatch は settle 完了（非同期の `on_settle`）を待たず release
        // 時に即実行する（モジュール doc「settle 完了を待たず release 時に
        // 即 dispatch する」節参照）。spring 自体は純粋に見た目の追従用
        // （`--fandhe-carousel-index` の連続値書き込み）として引き続き
        // 走らせ、収束完了時には dragging 属性の除去のみを行う。
        //
        // dispatch するアクション名は [`super::CAROUSEL_GOTO_ACTION_ATTR`]
        // が carousel root へ指定されていればそれを使う（モジュール doc
        // 「複数 carousel の識別」節参照）。空文字列は未指定と同義に扱う。
        let action_name = meta
            .carousel_root
            .get_attribute(CAROUSEL_GOTO_ACTION_ATTR)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "goto".to_string());
        let carousel_root = meta.carousel_root.clone();
        // `on_action` dispatch（同期）が `Runtime::apply_subtree_swap` を
        // 経由すると `carousel_root`/`item_group` が文書から切断され得る
        // ため、切断前の位置を控えておく（モジュール doc「release 時の
        // 再描画後も表示中の DOM で spring を継続する」節参照）。
        let carousel_position = position_of_carousel_root(root, &meta.carousel_root);
        // `CLICK_GUARD_PX` 未満の移動しかないタップは `"goto"` を
        // dispatch しない（codex-review 指摘 是正「ドラッグしていない
        // タップでは goto を dispatch しない」、イシュー #2541）だけでなく、
        // 着地先の**計算方法自体**も分ける（codex-review 指摘 是正
        // 「収束中タップで確定 index が失われる」、イシュー #2541 第 4
        // ラウンド）。`pointerdown`（[`resume_pointer_down`]）が settle
        // 未完了の spring を打ち切った直後に無移動タップされると、通常の
        // `t.on_release` は中断時点の途中経過進行度から新たに最寄り index
        // を計算してしまい、既に確定済み（`"goto"` dispatch 済み）の
        // 状態と表示が食い違う。[`CarouselTrack::on_tap_release`] は
        // 直前に確定していた着地 index へ収束し直すため、状態と表示が
        // 一致し続ける。
        let target = if meta.moved {
            t.on_release(event.time_stamp(), meta.slide_px, move |_index| {
                let _ = carousel_root.remove_attribute(CAROUSEL_DRAGGING_STATE_ATTR);
            })
        } else {
            t.on_tap_release(move |_index| {
                let _ = carousel_root.remove_attribute(CAROUSEL_DRAGGING_STATE_ATTR);
            })
        };
        drop(track_guard);
        if meta.moved {
            (on_action.borrow_mut())(ActionRef {
                action: action_name,
                payload: target.to_string(),
            });
        }

        // dispatch が同期的に DOM 部分木を差し替えた場合、settle 中の
        // spring が書き込んでいた要素は既に切断されている。同じ位置の
        // 新しい carousel root/`item-group` を再解決し、進行中の spring を
        // その要素へ retarget する（見つからなければ、これ以上表示に
        // 影響しない spring を打ち切ってリークを防ぐ）。`carousel_root`
        // 自身が `Runtime` の mount root と一致するケース（差し替え対象の
        // 子ノードのみが再生成され root 自身は保持される、
        // `apply_subtree_swap` doc 参照）では `carousel_root.is_connected()`
        // が常に真のまま残るため、`item_group`（実際に書き込み先となる
        // 要素）の切断も合わせて判定する。
        // ノードが切断されず（同一ノードのまま属性のみ再描画される
        // キー付き diff 構成で）残っている場合、SSR が確定済み baseline
        // 値でインライン style を上書きし得るため、次の rAF tick を待たず
        // 現在の進行度を即座に書き戻す（codex-review 指摘 是正 P2「同一
        // ノードへの更新後にも途中進行値を復元すべき」、イシュー #2541
        // 第 5 ラウンド）。切断された場合は下の retarget 経路が同じ防止策
        // （`CarouselTrack::retarget` 内の同種フラッシュ）を担う。
        if meta.carousel_root.is_connected() && meta.item_group.is_connected() {
            if let Some(t) = track.borrow().as_ref() {
                t.flush_current_progress();
            }
        }

        if !meta.carousel_root.is_connected() || !meta.item_group.is_connected() {
            let mut track_guard = track.borrow_mut();
            if let Some(t) = track_guard.as_mut() {
                match resolve_replacement(
                    root,
                    &meta.carousel_root,
                    carousel_position,
                    meta.slide_count,
                ) {
                    Some((new_root, new_item_group)) => {
                        let attr_root = new_root.clone();
                        t.retarget(new_item_group, move |_index| {
                            let _ = attr_root.remove_attribute(CAROUSEL_DRAGGING_STATE_ATTR);
                        });
                        // `retarget` が実際に spring を再起動した（settle
                        // 未完了だった）場合のみ、新 root にも
                        // `CAROUSEL_DRAGGING_STATE_ATTR` を引き継ぐ
                        // （codex-review 指摘 是正「再生成された root に
                        // dragging 属性を引き継ぐ」、イシュー #2541）。旧
                        // root にしか付与していないと、旧 DOM 前提の
                        // `carousel_motion.rs`（pre-styled-ui）側 CSS の
                        // `transition: none` セレクタが新 root へ効かず、
                        // CSS transition と毎フレームの spring 書き込みが
                        // 競合する。
                        if t.is_settling() {
                            let _ = set_dom_attribute(&new_root, CAROUSEL_DRAGGING_STATE_ATTR, "");
                        }
                        // レジストリのキーも新 root へ移す（`rekey_carousel_root`
                        // doc 参照。移さないと次の `slot_for` の遅延掃除で
                        // このエントリが切断済みキーのまま間引かれ、進行中の
                        // spring を保持する `TrackSlot` ごと drop される）。
                        rekey_carousel_root(registry, &meta.carousel_root, &new_root);
                    }
                    None => {
                        *track_guard = None;
                    }
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_carousel_motion_events;

#[cfg(test)]
mod tests {
    use super::{is_vertical_orientation, parse_loop_opt_in};

    #[test]
    fn parse_loop_opt_in_reads_loop_literal_only() {
        assert_eq!(parse_loop_opt_in(Some("loop")), Some(true));
        assert_eq!(parse_loop_opt_in(Some("")), Some(false));
        assert_eq!(parse_loop_opt_in(Some("bogus")), Some(false));
        assert_eq!(parse_loop_opt_in(None), None);
    }

    #[test]
    fn is_vertical_orientation_matches_literal_only() {
        assert!(is_vertical_orientation(Some("vertical")));
        assert!(!is_vertical_orientation(Some("horizontal")));
        assert!(!is_vertical_orientation(None));
    }

    #[test]
    fn attr_constants_are_stable_strings() {
        assert_eq!(super::CAROUSEL_DRAG_ATTR, "data-fandhe-carousel-drag");
        assert_eq!(
            super::CAROUSEL_DRAGGING_STATE_ATTR,
            "data-fandhe-carousel-dragging"
        );
        assert_eq!(
            super::CAROUSEL_GOTO_ACTION_ATTR,
            "data-action-carousel-goto"
        );
    }
}
