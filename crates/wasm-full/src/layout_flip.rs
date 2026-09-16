//! keyed list の構造変化（`Insert`/`Move` 等）前後で layout FLIP アニメー
//! ションを起動する薄い配線（イシュー #2518、`feature = "layout-animation"`）。
//!
//! # 責務境界
//!
//! 座標計測（`getBoundingClientRect()`）・Invert 計算・Play（spring +
//! rAF）はすべて `fandhe_frontend_animation::flip` の責務であり、本モジュール
//! はそれを keyed list の構造変化コミット前後という「いつ呼ぶか」にのみ
//! 関与する（`docs/design/animation-core-architecture.md` の層責務、
//! `docs/guides/wasm-full-features.md` §11.2）。DOM 計測・transform 計算の
//! ロジックは一切持たない。
//!
//! # 対象リストの明示的オプトイン
//!
//! [`FLIP_AUTO_ATTR`] を持つ keyed list にのみ適用する
//! （`stagger_index::STAGGER_AUTO_FIRST_ATTR` と同じ設計）。この属性を
//! 持たないリストは、keyed list 構造変化のたびに無条件で計測・アニメー
//! ションされることがない。
//!
//! # 入れ子 FLIP リストの所有権契約（codex-review 追加ラウンド 2 巡目
//! 是正、イシュー #2518。P1 2 件〔field 単位の `continue` による走査
//! 漏れ・入れ子リストの二重補正〕+ Bugbot Medium 1 件〔内側リストの構造
//! 変化時に外側 FLIP を停止・再捕捉しない〕を 1 ルールで解決する）
//!
//! [`FLIP_AUTO_ATTR`] リストが同属性の**祖先リストを持つ場合**、その
//! 内側リストは**自分では capture/play しない**。**最外側の FLIP リスト
//! がサブツリー全体のアニメーションを所有する**（内側リストの子孫は、
//! 外側リストの Invert 変形にまとめて追従する。内側リスト自身の並べ替え
//! 差分は、外側リストの行（内側リストを含む）の Before/Last 矩形の変化
//! として自然に取り込まれるため、内側リスト単独の FLIP は不要）。
//!
//! 具体的な処理契約（`Runtime::apply_update_for_dirty` 側の実装）:
//!
//! 1. 内側リスト（またはその配下の束縛要素）が dirty のときは、
//!    [`flip_lists_containing`] で祖先方向の [`FLIP_AUTO_ATTR`] 付き
//!    リストを**全件**列挙し、そのすべてを [`capture_before`] で停止・
//!    捕捉する（重複排除のみで、field の列挙順には依存しない）。
//! 2. 列挙した祖先チェーンの**最後の要素**（root に最も近い、真の最外側）
//!    だけを「実際に play する対象」として記録する。内側リストは停止・
//!    捕捉こそされるが、二度と play されない（play 対象から除外される
//!    ことで「自分では再生しない」契約を満たす）。
//! 3. dirty field ごとの走査は「field 自身の keyed list」「field に
//!    束縛された要素」の両方を**常に**行う（旧実装は前者が解決した field
//!    を以後の走査から `continue` で除外しており、同じ field 名が
//!    「リスト自体」と「別リストの行の束縛」の両方に使われる構成で
//!    後者の走査が抜けていた）。
//! 4. 実際の Last 計測・Invert・Play は、全 dirty field の構造変化
//!    コミットが完了した後に、2 で決定した最外側リストの集合へ対して
//!    のみ行う（`flip_target_lists` の実装は `Runtime::
//!    apply_update_for_dirty` 側）。
//!
//! 本契約は「入れ子リストそれぞれが独立にアニメーションする」ことを
//! 目指すものではない（それは本 PR のスコープ外——最外側リストが
//! サブツリー全体の Invert 変形を所有する設計上、内側リスト単独の
//! 動きを外側から分離して表現するには別途アーキテクチャが必要であり、
//! 将来の課題として扱う）。
//!
//! この 4 点により、`dirty` の列挙順（例: 内側→外側 か 外側→内側 か）に
//! 関わらず「実際に play されるリストの集合」は同じに収束する。入れ子
//! リストを内側から処理すると Invert 変形が二重適用され、子の視覚位置が
//! 跳ぶ（外側の Move が内側 DOM を保持するため実際の更新経路でも発生
//! し得る）問題は、そもそも内側リストを play 対象に含めないことで構造的
//! に起こらない。
//!
//! # 走査方法についての注記
//!
//! `stagger_index.rs` と同じ理由（`HTMLCollection` のランダムアクセスは
//! 使わない）で `first_element_child`/`next_element_sibling` による
//! 1 パス（本モジュールは 4 パス、§「4 パス走査」参照）sibling 走査を
//! 用いる。
//!
//! # 計測・合成の再設計（codex-review 第 4 ラウンド是正、イシュー #2518）
//!
//! 第 2〜3 ラウンドは要素自身の変形 `O` を解析して FLIP 補正へ補正項を
//! 積み増す方式だったが、要素自身のサイズが変化する場合に収束しない
//! 構造的な欠陥があった。本ラウンドは計測・合成そのものを再設計する
//! （詳細な導出・数値例は `fandhe_frontend_animation::flip` クレート doc
//! 「計測・合成の再設計」節・[`flip::anchor_correct`] の rustdoc を正と
//! する）。本モジュールが計測するタイミング・矩形の種類が変わった:
//!
//! - **First**（[`capture_before`]）: 進行中の FLIP 補正込みの「現在の
//!   視覚矩形」（[`flip::measure`]）を測る。旧実装（`flip::
//!   measure_clearing_transform` による layout 矩形計測 + `current_
//!   viewport_delta`/`apply_delta_to_rect` による引き継ぎ計算）は廃止した
//!   （現在の視覚矩形を直接測るだけで連続性・スクロール反映の両方を
//!   自動的に満たすため、引き継ぎ計算そのものが不要になった）。
//! - **Last**（[`play_after`]）: 視覚矩形（`last_visual`、要素自身の
//!   変形を含む）と layout 矩形（`last_layout`、`flip::
//!   measure_clearing_transform` で要素自身の変形を一時的に無効化して
//!   測る）の**両方**を測る。後者は要素自身の変形の基準点のずれを補正
//!   する [`flip::anchor_correct`] の入力として必要（詳細は同 rustdoc）。
//!
//! # 4 パス走査（reflow 削減 + 旧アニメーションの決定的停止）
//!
//! `play_after` は対象行を 4 回走査する。
//!
//! **捕捉順の契約（codex-review 第 7 ラウンド是正、イシュー #2518。
//! P1「計測で確定した変形と FLIP の収束先を一致させる」）**: [`flip::
//! OriginalStyle::capture`] と Last **視覚**矩形（[`flip::measure`]）は
//! 必ず [`flip::measure_layout_batch`]（Last **layout** 矩形計測、
//! パス 2）の**後**に取得する。旧実装（第 4〜6 ラウンド）はパス 0 で
//! `OriginalStyle::capture` を行っていたため、対象行の要素自身の
//! `transform` に CSS transition が進行中の場合に不整合が生じていた:
//! `measure_layout_batch` は要素自身の `transform` を一時的に無効化・
//! 復元する過程で進行中の transition を目標値へ確定（settle）させる
//! 副作用を持つ（[`flip::measure_layout_batch`]/[`flip::
//! measure_clearing_transform`] doc 参照）ため、それより前に捕捉した
//! `OriginalStyle::computed_transform`（Play の収束先）は settle 前の
//! 中間値のままとなり、実際に `measure_layout_batch` が確定させた
//! 目標値と食い違う（Play が中間 matrix を合成し続け、収束・復元時に
//! 目標値へ跳ぶ）。本ラウンドでパス順を以下へ変更した:
//!
//! 1. **パス 0**（reflow なし）: 対象行を 1 パス走査し、`(key,
//!    HtmlElement)` を収集しつつ、進行中の旧 `AnimationLoop`（あれば。
//!    通常は [`capture_before`] が構造変化コミット前に既に停止済みの
//!    ため、ここでの `remove` は防御的な二重停止に留まる）を停止する。
//!    **`OriginalStyle::capture` はまだ行わない**（パス 2 の後へ移動、
//!    上記契約参照）。旧アニメーションの停止だけは引き続きこの時点で
//!    行う: 停止（drop）は未収束なら旧 `OriginalStyle` へ即座に復元する
//!    ため、これより後に測る Last 視覚矩形・layout 矩形が旧 FLIP 補正
//!    込みの値になってしまわないよう、他の計測より先に行う必要がある。
//! 2. **パス 1**（reflow を伴う Last layout 矩形の一括計測）: パス 0 で
//!    収集した対象行の `HtmlElement` をまとめて [`flip::
//!    measure_layout_batch`] へ渡す（codex-review 第 5 ラウンド是正、
//!    イシュー #2518。全対象行の `transform`/`transform-origin`/
//!    `transition` を**先にすべてクリア**→**まとめて計測**→**まとめて
//!    復元**の 3 フェーズに分けることで、対象行数に関わらず reflow を
//!    実質 1 回に抑える。旧実装〔`flip::measure_clearing_transform` を
//!    行ごとにループ呼び出し〕は各呼び出しが内部でクリア→計測→復元を
//!    個別に行うため、実質 reflow が行数分発生していた。本モジュール
//!    自身は行ごとのクリア書き込み処理を持たない——一括計測の実装は
//!    `flip::measure_layout_batch` の責務であり、本モジュールはそれを
//!    呼ぶだけである）。この呼び出しが進行中の transform transition を
//!    settle させる（上記契約参照）。
//! 3. **パス 2**（reflow を伴う `OriginalStyle::capture` + Last 視覚
//!    矩形の計測）: パス 1 の settle 後に、対象行ごとに「復元すべき
//!    元の `transform`/`transform-origin`」（[`flip::OriginalStyle::
//!    capture`]）を確定し、現在の視覚矩形（要素自身の変形はそのまま）
//!    を `flip::measure` する。元の値は現在（構造変化・属性更新
//!    コミット後、かつパス 1 の settle 後）の inline `transform`/
//!    `transform-origin` を常にフレッシュ捕捉する（構造変化と同じ
//!    コミットで行われた属性更新〔`Update` が `style`/`transform` を
//!    書き換えていた場合〕を正しく反映するため）。
//! 4. **パス 3**: `invert(first, last_visual)` → [`flip::anchor_correct`]
//!    （`last_visual`/`last_layout` で基準点のずれを補正）→ Play。
//!    `invert` の結果が `flip::IDENTITY`（並べ替えなしの内容のみ
//!    `Update` 等、行の視覚矩形が変化していない）または `None` の場合は
//!    Play を起動せず、パス 2 で確定した [`flip::OriginalStyle`] へ直接
//!    復元する（旧ループはパス 0 で既に停止済みのため、Play を省略して
//!    も古い補正が書き戻されることはない）。
//!
//! [`flip::play`] を本モジュールを経由せず直接呼ぶ利用者（本クレート
//! 単体 API 経路）も、同じ捕捉順の契約を守ること（[`flip::OriginalStyle::
//! capture`] の rustdoc「呼び出し順」節参照）。
//!
//! # リーク対策（`AnimationLoop` の外部保持契約）
//!
//! [`fandhe_frontend_animation::raf_driver::AnimationLoop`] は `stop()`
//! （または `Drop`）が呼ばれるまで rAF コールバックの `Closure` を保持し
//! 続ける（`raf_driver.rs` doc 参照）。本モジュールは [`FLIP_LOOPS`]
//! （`(リスト識別子, data-key)` → [`FlipEntry`]）に所有者を持たせ、
//! `play_after` が同じキーへ新しいアニメーションを開始する（あるいは
//! 再生を省略する）たびに、パス 0 で旧エントリを `remove` して即座に
//! drop する（`Drop::stop()`）。加えて、収束予定時刻（`settle_duration`
//! 経過後）に `window.setTimeout` でエントリ自体を除去する自己クリーン
//! アップをスケジュールし、二度と並べ替えられない `data-key` が
//! [`FLIP_LOOPS`] へ無制限に蓄積しないようにする。除去はトークン一致を
//! 条件とする（タイムアウト発火前に同じキーへ新しいアニメーションが
//! 始まっていた場合、古いタイムアウトが新しいエントリを誤って消さない
//! ための版数チェック）。**この強制停止（タイムアウト発火）時にも、
//! [`FlipEntry`] が保持する要素・[`flip::OriginalStyle`] を使って
//! 明示的に元のスタイルへ復元する**（`AnimationLoop::drop` は rAF
//! キャンセルのみで DOM の `transform` を消さないため、タブ非表示や
//! メインスレッド長時間占有で rAF が spring 収束前に進まなくなった場合、
//! 復元しないと補正 transform が永続的に残ってしまう）。
//!
//! # リスト識別子によるキー空間分離
//!
//! [`FLIP_LOOPS`] のキーを `data-key` の値のみにすると、[`FLIP_AUTO_ATTR`]
//! を持つ 2 つの独立したリストが同じ `data-key` 値集合（例:
//! "1"/"2"/"3"）を使った場合、片方のリストの `start_flip` がもう片方の
//! 進行中 `AnimationLoop` を誤って `insert` 置き換え・drop してしまう
//! （`AnimationLoop::stop`/`Drop` は rAF キャンセルのみで DOM の
//! `transform` を消さないため、settle 前に drop された要素は補正
//! transform のまま固定表示され続ける）。これを避けるため、`list_element`
//! ごとに一意な `u64` 識別子（[`list_instance_id`]、`js_sys::WeakMap` で
//! DOM 要素の同一性から解決）を発行し、[`FLIP_LOOPS`] のキーを
//! `(list_instance_id, data_key)` の組にしてリストをまたいだキー衝突を
//! 構造的に防ぐ。
//!
//! # セキュリティ不変条件（REQ-1・security.md A03）
//!
//! [`FLIP_AUTO_ATTR`] は存在のみを見る opt-in 属性であり値を使わない
//! （`STAGGER_AUTO_FIRST_ATTR` と同型）。`data-key` の値は
//! [`FLIP_LOOPS`] の `HashMap` キーとしてのみ使用し、CSS/セレクタへ
//! 混入しない。

/// keyed list の親要素に付け、Before/After 計測 + FLIP 再生の対象である
/// ことを明示するオプトイン属性名。値は不問(存在のみを見る)。
pub const FLIP_AUTO_ATTR: &str = "data-fandhe-flip-auto";

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::FLIP_AUTO_ATTR;
    use fandhe_frontend_animation::fandhe_animation::spring::SpringConfig;
    use fandhe_frontend_animation::flip;
    use js_sys::WeakMap;
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, HtmlElement};

    /// [`FLIP_LOOPS`] の 1 エントリ。
    ///
    /// `anim_loop`（[`flip::FlipAnimation`]）は未収束時に限り [`Drop`] で
    /// 元のスタイルへ復元する（`flip::FlipAnimation` doc 参照）ため、
    /// 要素・元のスタイルを本構造体側で別途保持する必要はない（drop
    /// するだけで正しい復元判断が行われる）。
    ///
    /// codex-review 第 4 ラウンド是正（イシュー #2518）: First の計測が
    /// 「現在の視覚矩形を直接測る」方式へ変わったため、旧実装が保持して
    /// いた `last_rect`（Play 開始時点の Last 矩形）は不要になった（
    /// モジュール doc「計測・合成の再設計」参照）。
    struct FlipEntry {
        token: u64,
        // `flip::FlipAnimation` の `Drop` 実装（未収束なら元のスタイルへ
        // 復元する）を発火させるためだけに保持するフィールド。フィールド
        // 自体を読むことはない（`schedule_cleanup`/`play_after` パス 0 は
        // `remove` して drop するだけで復元判断を `Drop` に委ねる、モジュール
        // doc「リーク対策」参照）。
        #[allow(dead_code)]
        anim_loop: fandhe_frontend_animation::flip::FlipAnimation,
    }

    thread_local! {
        /// `(list_instance_id, data-key)` ごとの進行中 FLIP アニメーション
        /// （モジュール doc「リーク対策」「リスト識別子によるキー空間分離」
        /// 参照）。プロセス内（wasm はシングルスレッドのため実質アプリ生存
        /// 期間）唯一のマップであり、`position::wiring::GLOBAL_CONTROLLER`
        /// と同型の設計（イシュー #2209）。
        static FLIP_LOOPS: RefCell<HashMap<(u64, String), FlipEntry>> =
            RefCell::new(HashMap::new());
        /// [`FLIP_LOOPS`] エントリの版数トークン発行元（キー単位の一意性は
        /// 不要、全体で単調増加すれば十分）。
        static NEXT_TOKEN: Cell<u64> = const { Cell::new(0) };
        /// `list_element`（DOM 要素の同一性）→ `u64` 識別子。`WeakMap` の
        /// キーは弱参照であり、要素が DOM から除去され GC されればエントリ
        /// も自然に消える（[`list_instance_id`] 参照）。
        static LIST_IDS: WeakMap = WeakMap::new();
        /// [`LIST_IDS`] の発行元カウンタ。
        static NEXT_LIST_ID: Cell<u64> = const { Cell::new(0) };
    }

    /// `list_element` に対応する [`FLIP_LOOPS`] キー空間分離用の識別子を
    /// 取得・未発行なら新規発行する（モジュール doc「リスト識別子による
    /// キー空間分離」参照）。同じ DOM 要素（JS 参照同一性）へは常に同じ
    /// 識別子を返す。
    fn list_instance_id(list_element: &Element) -> u64 {
        LIST_IDS.with(|map| {
            let key: js_sys::Object = list_element.clone().unchecked_into();
            if let Some(existing) = map.get(&key).as_f64() {
                return existing as u64;
            }
            let id = NEXT_LIST_ID.with(|cell| {
                let next = cell.get().wrapping_add(1);
                cell.set(next);
                next
            });
            map.set(&key, &JsValue::from_f64(id as f64));
            id
        })
    }

    /// `(list_id, key)` 行の進行中 FLIP を無条件で停止する唯一の関数
    /// （codex-review 追加ラウンド 3 巡目是正、イシュー #2518。「停止が
    /// `capture_before`/`play_after` パス 0 に個別実装され、経路追加の
    /// たびに片方だけ更新されて停止漏れを繰り返した」構造的原因への
    /// 対処）。[`FLIP_LOOPS`] から該当エントリを `remove` して戻り値ごと
    /// `drop` する。未収束であれば [`flip::FlipAnimation`] の [`Drop`]
    /// が元のスタイルへ即座に復元する（クレート doc「リーク対策」参照。
    /// この関数自体は復元の要否を判断しない——判断は常に `Drop` に委ねる）。
    ///
    /// [`capture_before`]・[`play_after`] パス 0 は共にこの関数だけを
    /// 呼ぶ（`FLIP_LOOPS` への直接 `remove` はこの関数の内部に限定する）。
    /// [`schedule_cleanup`] の条件付き `remove`（`token` 一致時のみ）は
    /// 「割り込みによる停止」ではなく「自然収束後の GC」であり意味論が
    /// 異なるため対象外とする（既に自然収束済みのエントリを掃除するのみ
    /// で、`Drop` によるスタイル復元は起こらない）。
    fn stop_flip(list_id: u64, key: &str) {
        let existing =
            FLIP_LOOPS.with(|cell| cell.borrow_mut().remove(&(list_id, key.to_string())));
        drop(existing);
    }

    /// `element` 自身、またはその祖先のうち [`FLIP_AUTO_ATTR`] を持つ
    /// 要素を**すべて**（root 方向へ辿った出現順で）返す（イシュー #2518
    /// codex-review 追加ラウンド。P1「入れ子リストの行自身を動かす外側の
    /// FLIP も停止する」）。
    ///
    /// `Runtime::apply_update_for_dirty` が、keyed list 自体ではなく行内の
    /// 子要素へ束縛された field（例: `row_style` で行の `style` 属性を
    /// 更新する構成）が dirty のときに、その束縛先要素が
    /// [`FLIP_AUTO_ATTR`] 付きリストの配下（行そのもの、または行の子孫）
    /// にあるかを判定するための前提 API。見つかったリストそれぞれへ
    /// 呼び出し元が [`capture_before`] を適用し、進行中の旧 FLIP を
    /// [`fandhe_frontend_wasm_client::BindingTable::apply_dirty`] の
    /// 書き込みより前に停止する（構造変化を伴わない更新でも、旧
    /// [`flip::FlipAnimation`] の `Drop` による復元が新しい束縛値を
    /// 巻き戻してしまうのを防ぐ契約、`Runtime::apply_update_for_dirty`
    /// doc 参照）。
    ///
    /// # 「最寄り 1 件」ではなく全件を返す理由
    ///
    /// 旧実装（`nearest_flip_list`、`Element::closest` で最寄り 1 件のみ
    /// 返す方式）は、入れ子 keyed list（外側リストの直接の行要素が同時に
    /// 内側リストの `FLIP_AUTO_ATTR` を持つ構成）で、束縛要素自身を
    /// **行として管理する外側リスト**の FLIP を取りこぼした（codex-review
    /// 追加ラウンド 2 巡目 P1 指摘）。`closest` は「マッチする最も近い
    /// 祖先（自身含む）」1 件しか返さないため、束縛要素自身が内側
    /// リストの `FLIP_AUTO_ATTR` を持つ場合、そこで探索が止まり外側
    /// リストへ到達しない。本関数は `parent_element()` で root 方向へ
    /// 1 段ずつ辿り、経路上の [`FLIP_AUTO_ATTR`] 付き要素をすべて収集
    /// することで、この取りこぼしを構造的に防ぐ。
    ///
    /// `element` 自身も判定対象に含む（束縛先がリスト要素自身の場合も
    /// 拾う。`Element::closest` と同じ挙動）。
    #[must_use]
    pub fn flip_lists_containing(element: &Element) -> Vec<Element> {
        let mut result = Vec::new();
        let mut current: Option<Element> = Some(element.clone());
        while let Some(candidate) = current {
            if candidate.has_attribute(FLIP_AUTO_ATTR) {
                result.push(candidate.clone());
            }
            current = candidate.parent_element();
        }
        result
    }

    /// `root` 配下で `field` を束縛する要素をすべて返す（イシュー #2518
    /// codex-review 追加ラウンド。P1「行の束縛だけを更新する場合も進行中
    /// の FLIP を停止する」）。
    ///
    /// `fandhe_frontend_wasm_client::BindingTable`（`Runtime` が保持する
    /// キャッシュ）は束縛先要素を外部へ公開する API を持たない（0.6.1
    /// 時点、`entries` は非公開フィールド。`elements_for_field` のような
    /// 公開 API を新設して束縛先要素を取得する案は、`wasm-client` の
    /// semver バンプが `templates/app/wasm/Cargo.toml`（`fandhe-frontend-
    /// wasm-client = "0.6.1"`）の crates.io 未公開バージョンへの追随を
    /// 要求し `template_vendor_drift` テストを構造的に落とすため不採用と
    /// した。詳細は `docs/ci/version-bump-publish-order-gap.md`）ため、
    /// 本関数は `root` を独立に 1 回走査して束縛点を再構築する。
    ///
    /// # wasm-client との追随契約
    ///
    /// 走査規則（セレクタ）は `fandhe_frontend_wasm_client::binding_dom::
    /// BindingTable::scan` 内部の非公開ヘルパ `binding_selector`（`[data-
    /// bind-text],[data-bind-attr],[data-bind-class]`）と同じ 3 属性を
    /// 対象にする。属性名は両クレート共通の公開定数
    /// （`fandhe_frontend_core::{BIND_TEXT_ATTR, BIND_ATTR_ATTR,
    /// BIND_CLASS_ATTR}`）を直接参照するため、属性名自体のドリフトは
    /// 起こらない。spec 抽出（`"<name>:<field>"` トークン列のパース・
    /// 複数トークンの分割規則）は独自実装せず、`fandhe_frontend_
    /// wasm_client::element_binding_specs`（公開関数、wasm-client 0.6.1）
    /// をそのまま呼ぶため、パース規則のドリフトも構造的に起こらない。
    /// 本関数が追随契約として負うのは「3 属性を対象にした
    /// `query_selector_all` を 1 回行う」というセレクタ組み立てのみ
    /// （`BindingTable::scan` が同じ 3 属性で `query_selector_all` する
    /// ことを前提とする）であり、この前提が崩れる場合（wasm-client が
    /// 束縛点マーカー属性を追加・変更する場合）は本関数も追随更新が
    /// 必要になる。
    #[must_use]
    pub fn elements_bound_to_field(root: &Element, field: &str) -> Vec<Element> {
        let selector = format!(
            "[{}],[{}],[{}]",
            fandhe_frontend_core::BIND_TEXT_ATTR,
            fandhe_frontend_core::BIND_ATTR_ATTR,
            fandhe_frontend_core::BIND_CLASS_ATTR
        );
        let Ok(node_list) = root.query_selector_all(&selector) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for i in 0..node_list.length() {
            let Some(node) = node_list.get(i) else {
                continue;
            };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            let bind_text = element.get_attribute(fandhe_frontend_core::BIND_TEXT_ATTR);
            let bind_attr = element.get_attribute(fandhe_frontend_core::BIND_ATTR_ATTR);
            let bind_class = element.get_attribute(fandhe_frontend_core::BIND_CLASS_ATTR);
            let specs = fandhe_frontend_wasm_client::element_binding_specs(
                bind_text.as_deref(),
                bind_attr.as_deref(),
                bind_class.as_deref(),
            );
            if specs.iter().any(|spec| spec.field == field) {
                out.push(element);
            }
        }
        out
    }

    /// `list_element` が [`FLIP_AUTO_ATTR`] を持つ場合に限り、直接の子
    /// （keyed list の各行）を 1 パス走査し、`data-key` ごとの**現在の
    /// 視覚矩形**（[`flip::measure`]）を First として返す（codex-review
    /// 第 4 ラウンド是正・再設計、イシュー #2518。モジュール doc
    /// 「計測・合成の再設計」参照）。
    ///
    /// `Runtime::apply_update_for_dirty` が keyed list の構造変化を DOM へ
    /// 適用する**前**に呼ぶ想定（呼び出し元 doc 参照）。
    ///
    /// アニメーション進行中に呼ばれた場合、進行中の FLIP 補正込みの
    /// 「現在実際に表示されている矩形」がそのまま First になる（連続性・
    /// スクロール反映の両方をこの計測だけで満たす。旧実装が行っていた
    /// `flip::measure_clearing_transform` による layout 矩形計測 +
    /// `current_viewport_delta`/`apply_delta_to_rect` による引き継ぎ計算
    /// は不要になったため削除した）。**測定は必ず旧アニメーションの
    /// 停止より前に行う**: 停止（`FLIP_LOOPS` から `remove` して drop）は
    /// 未収束なら即座に元のスタイルへ復元する（`flip::FlipAnimation` の
    /// [`Drop`] 参照）ため、先に停止すると測定対象の視覚的な見た目が
    /// 変わってしまい、進行中の補正込みの表示位置を取得できない。
    #[must_use]
    pub fn capture_before(list_element: &Element) -> Option<HashMap<String, flip::Rect>> {
        if !list_element.has_attribute(FLIP_AUTO_ATTR) {
            return None;
        }
        let list_id = list_instance_id(list_element);
        let mut result = HashMap::new();
        let mut current = list_element.first_element_child();
        while let Some(el) = current {
            if let (Some(key), Some(html)) = (
                el.get_attribute(fandhe_frontend_core::keyed::KEY_ATTR),
                el.dyn_ref::<HtmlElement>(),
            ) {
                // 現在の視覚的な表示位置（進行中の FLIP 補正込み）を
                // First として測る（旧アニメーションの停止より前に行う、
                // 本関数 doc 参照）。
                let visual = flip::measure(html);
                // 停止は [`stop_flip`]（唯一の停止関数）に一元化する。
                // 未収束なら drop 時点で `flip::FlipAnimation::drop` が
                // 元のスタイルへ復元するが、この復元は上記で既に測定済み
                // の `visual` に影響しない。
                stop_flip(list_id, &key);
                result.insert(key, visual);
            }
            current = el.next_element_sibling();
        }
        Some(result)
    }

    /// `before`（[`capture_before`] の結果）と `list_element` の構造変化
    /// 反映後の現在の子を突き合わせ、同じ `data-key` が残っている行に
    /// ついて Invert・Play を行う（Last 計測、モジュール doc「4 パス
    /// 走査」参照）。
    ///
    /// `Runtime::apply_update_for_dirty` が keyed list の構造変化を DOM へ
    /// 適用した**後**に呼ぶ想定。`before` が空（[`capture_before`] が
    /// `None` を返した、あるいは対象行が 1 つもなかった）場合は no-op。
    ///
    /// 戻り値は、実際に `start_flip`（transform 適用）を起動した行要素
    /// のみ（`delta == flip::IDENTITY` で Play を起動せず元のスタイルへ
    /// 直接復元した行は含まない）。呼び出し元（`Runtime::
    /// apply_update_for_dirty`）はこれを `shared_layout::
    /// play_after_excluding` の除外対象として使う（Bugbot 指摘、イシュー
    /// #2578「FLIP lists skip nested shared layout」: 除外範囲を
    /// `list_element` サブツリー全体ではなく実際に transform を適用した
    /// 行のみへ絞ることで、同一 keyed list 内の新規挿入・移動行に含まれる
    /// `data-fandhe-layout-id` 要素が共有レイアウト遷移から取りこぼされ
    /// なくなる）。
    ///
    /// 公開 API [`play_after`]（戻り値 `()`）はこの関数の薄いラッパーで
    /// あり、既存利用者との戻り値互換性を維持する（codex-review P1 指摘、
    /// イシュー #2578: 0.40.1 → 0.40.2 の patch 更新で公開再エクスポート
    /// 済み関数の戻り値を変えると破壊的変更になる）。
    #[must_use]
    pub fn play_after_reporting(
        list_element: &Element,
        before: HashMap<String, flip::Rect>,
    ) -> Vec<HtmlElement> {
        if before.is_empty() {
            return Vec::new();
        }

        let list_id = list_instance_id(list_element);

        // パス 0（reflow なし）: 対象行の `(key, HtmlElement)` を順序付き
        // で収集し、進行中の旧アニメーションがあれば無条件に停止する
        // （モジュール doc「4 パス走査」「リーク対策」参照。新アニメー
        // ションを起動するか否かに停止を依存させない）。
        //
        // `OriginalStyle::capture` は**ここでは行わない**（パス 2 へ
        // 移動、モジュール doc「捕捉順の契約」節参照。パス 1
        // `measure_layout_batch` が settle させた後に確定させる必要が
        // あるため）。旧アニメーションの停止だけは他の計測より先に行う
        // 必要がある: `capture_before` が構造変化コミット**前**の時点で
        // 進行中の旧アニメーションを既に `FLIP_LOOPS` から除去済み
        // （未収束なら `flip::FlipAnimation::drop` が真の元の値へ復元
        // 済み）のため、ここでの `remove` は `capture_before` を経由し
        // なかった経路（対象外だった呼び出し順序の変化等）に対する
        // 防御的な二重停止であり、通常経路では既に空である（drop する
        // だけで十分、`flip::FlipAnimation::drop` が復元判断を担う）。
        let mut targets: Vec<(String, HtmlElement)> = Vec::new();
        {
            let mut current = list_element.first_element_child();
            while let Some(el) = current {
                if let (Some(key), Some(html)) = (
                    el.get_attribute(fandhe_frontend_core::keyed::KEY_ATTR),
                    el.dyn_ref::<HtmlElement>(),
                ) {
                    if before.contains_key(&key) {
                        // 停止は [`stop_flip`]（唯一の停止関数）に一元化
                        // する（`capture_before` と同じ関数を経由する）。
                        stop_flip(list_id, &key);
                        targets.push((key, html.clone()));
                    }
                }
                current = el.next_element_sibling();
            }
        }

        // パス 1（reflow を伴う Last layout 矩形の一括計測）: 対象行の
        // `HtmlElement` をまとめて [`flip::measure_layout_batch`] へ渡す
        // （codex-review 第 5 ラウンド是正、イシュー #2518。`flip::
        // measure_layout_batch` が全要素の書き込み→全要素の読み取り→
        // 全要素の復元の 3 フェーズに分けるため、対象行数に関わらず
        // reflow は実質 1 回に抑えられる。旧実装は `flip::
        // measure_clearing_transform` を行ごとにループ呼び出ししており、
        // 内部で書き込み→読み取り→復元を個別に行うため実質 reflow が
        // 行数分発生していた）。この呼び出しは要素自身の `transform` に
        // 進行中の CSS transition があれば目標値へ settle させる副作用を
        // 持つ（モジュール doc「捕捉順の契約」節参照）。
        let target_elements: Vec<HtmlElement> =
            targets.iter().map(|(_, html)| html.clone()).collect();
        let last_layout_rects = flip::measure_layout_batch(&target_elements);
        let last_layouts: HashMap<String, flip::Rect> = targets
            .iter()
            .zip(last_layout_rects)
            .map(|((key, _), rect)| (key.clone(), rect))
            .collect();

        // パス 2（reflow を伴う `OriginalStyle::capture` + Last 視覚矩形
        // の計測）: パス 1 の settle 後に確定させる（モジュール doc
        // 「捕捉順の契約」節参照）。`OriginalStyle::computed_transform`
        // （Play の収束先）と Last 視覚矩形は、この時点でどちらも
        // settle 済みの状態を反映するため整合する。
        let mut originals: HashMap<String, flip::OriginalStyle> = HashMap::new();
        let mut last_visuals: HashMap<String, flip::Rect> = HashMap::new();
        for (key, html) in &targets {
            originals.insert(key.clone(), flip::OriginalStyle::capture(html));
            last_visuals.insert(key.clone(), flip::measure(html));
        }

        // パス 3: Invert → anchor_correct → Play。`delta == flip::IDENTITY`
        // （並べ替えなしの内容のみ `Update` 等、行の視覚矩形が変化して
        // いない）または `invert` が `None` の場合は Play を起動せず、
        // パス 2 で確定した元のスタイルへ直接復元する（旧ループはパス 0
        // で既に停止済みのため、Play を省略しても古い補正が書き戻され
        // ることはない）。実際に Play を起動した行要素は `played` へ集め
        // 戻り値として返す（本関数 doc「戻り値」参照）。
        let mut played = Vec::new();
        for (key, html) in targets {
            let Some(&first) = before.get(&key) else {
                continue;
            };
            let original = originals.remove(&key).unwrap_or_default();
            let last_visual = last_visuals.get(&key).copied();
            let last_layout = last_layouts.get(&key).copied();
            let delta = match (last_visual, last_layout) {
                (Some(last_visual), Some(last_layout)) => flip::invert(first, last_visual)
                    .map(|d| flip::anchor_correct(d, last_visual, last_layout)),
                _ => None,
            };
            match delta {
                Some(delta) if delta != flip::IDENTITY => {
                    start_flip(list_id, key, html.clone(), delta, original);
                    played.push(html);
                }
                _ => {
                    original.restore(&html);
                }
            }
        }
        played
    }

    /// `(list_id, key)` 行の FLIP 再生を開始し、[`FLIP_LOOPS`] へ登録する
    /// （呼び出し元の `play_after` パス 0 が旧エントリを既に `remove`
    /// 済みであるため、ここでの `insert` が旧エントリを上書きすることは
    /// ない）。収束予定時刻後に自己クリーンアップの `setTimeout` を 1 回
    /// スケジュールする（モジュール doc「リーク対策」参照）。
    fn start_flip(
        list_id: u64,
        key: String,
        element: HtmlElement,
        delta: flip::FlipDelta,
        original: flip::OriginalStyle,
    ) {
        let config = SpringConfig::default();
        let anim_loop = flip::play(element, delta, config, original);

        let token = NEXT_TOKEN.with(|cell| {
            let next = cell.get().wrapping_add(1);
            cell.set(next);
            next
        });
        FLIP_LOOPS.with(|cell| {
            cell.borrow_mut()
                .insert((list_id, key.clone()), FlipEntry { token, anim_loop });
        });

        // `Spring::new` が理論上 `None` を返す構成（`SpringConfig::default()`
        // では起きない）でも `settle_duration` は `fandhe_animation::spring`
        // 側の探索上限（`MAX_SETTLE_DURATION`、10 秒）を超えないため、失敗時は
        // その上限で代替する（fail-safe、`window`/`setTimeout` 呼び出し失敗と
        // 同様にクリーンアップが遅れるだけで実害はない）。
        let settle_ms =
            fandhe_frontend_animation::fandhe_animation::spring::Spring::new(config, 0.0, 1.0, 0.0)
                .map(|spring| spring.settle_duration() * 1000.0)
                .unwrap_or(10_000.0);
        schedule_cleanup(list_id, key, token, settle_ms);
    }

    /// [`play_after_reporting`] の戻り値互換ラッパー（戻り値 `()`）。
    /// 従来どおり `list_element` の Last 計測・Invert・Play を行い、実際に
    /// transform を適用した行の一覧は破棄する。外部利用者向けの公開
    /// シグネチャはこちらを維持し、除外対象の行一覧が必要な
    /// `Runtime::apply_update_for_dirty` だけが [`play_after_reporting`]
    /// を呼ぶ（codex-review P1 指摘、イシュー #2578）。
    pub fn play_after(list_element: &Element, before: HashMap<String, flip::Rect>) {
        let _ = play_after_reporting(list_element, before);
    }

    /// `settle_ms`（+ 余裕マージン）経過後に、[`FLIP_LOOPS`] の
    /// `(list_id, key)` エントリがまだ同じ `token` を持つ場合に限り除去
    /// する。`token` が不一致（既に新しいアニメーションへ置き換え済み）
    /// の場合は何もしない。
    ///
    /// 除去した [`FlipEntry`] を drop するだけに留め、**明示的な
    /// `restore` 呼び出しは行わない**。復元の要否は [`flip::
    /// FlipAnimation`] の [`Drop`] が [`flip::FlipAnimation::is_done`] で
    /// 判断する: spring が既に自然収束していた場合（`is_done() == true`、
    /// `flip::play` が `state.done` の時点で既に元のスタイルへ復元済み）
    /// は何もしない。タブ非表示・メインスレッド長時間占有等で rAF が
    /// spring 収束前に進まなくなった場合（`is_done() == false`）は、
    /// `Drop` がここで唯一のスタイル復元経路として機能する（モジュール
    /// doc「リーク対策」参照）。`window`/`setTimeout` の取得に失敗した
    /// 場合はスケジュール自体が成立せず、エントリは [`FLIP_LOOPS`] に
    /// 残り続けるが、次回同じキーでアニメーションが始まった際に
    /// `play_after` パス 0 の `remove` が置き換える（fail-safe、蓄積は
    /// 「同一キーが再度アニメーションされるまで」に留まる）。
    fn schedule_cleanup(list_id: u64, key: String, token: u64, settle_ms: f64) {
        let Some(window) = web_sys::window() else {
            return;
        };
        // 実測とタイマー精度のずれを吸収する余裕（`spring_via_raf_dom_browser.rs`
        // と同じ 2 倍 + 固定マージンの考え方、実装計画 §3.4）。
        let delay_ms = (settle_ms * 1.2 + 100.0).min(i32::MAX as f64) as i32;
        let map_key = (list_id, key);
        let callback = Closure::once_into_js(move || {
            FLIP_LOOPS.with(|cell| {
                let mut loops = cell.borrow_mut();
                if matches!(loops.get(&map_key), Some(entry) if entry.token == token) {
                    loops.remove(&map_key);
                }
            });
        });
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.unchecked_ref(),
            delay_ms,
        );
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{
    capture_before, elements_bound_to_field, flip_lists_containing, play_after,
    play_after_reporting,
};
