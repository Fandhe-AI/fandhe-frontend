//! 共有レイアウト遷移（motion.dev `layoutId` 相当）の `data-*` 配線
//! （イシュー #2536、`feature = "layout-animation"`。`crates/wasm-full/
//! src/layout_flip.rs` の「同一要素」対象範囲を補完する）。
//!
//! # 責務境界
//!
//! 突合（id ごとの旧要素・新要素の対応付け）・計測・Invert・Play は
//! すべて [`fandhe_frontend_animation::shared_layout`] の責務であり、本
//! モジュールはそれを `Runtime::apply_update_for_dirty`/`rerender`/
//! `view_transition_swap` という「いつ呼ぶか」にのみ関与する
//! （`layout_flip.rs` モジュール doc「責務境界」節と同じ設計）。DOM
//! 計測・transform 計算のロジックは一切持たない。
//!
//! # 対象要素の明示的オプトイン
//!
//! [`LAYOUT_ID_ATTR`] を持つ要素にのみ適用する（`layout_flip::
//! FLIP_AUTO_ATTR` と同じ opt-in 設計）。値は id として使う（`HashMap`
//! キーのみに使用し、CSS/セレクタへ混入しない、`layout_flip.rs`
//! モジュール doc「セキュリティ不変条件」参照）。
//!
//! # 呼び出しタイミング
//!
//! [`capture_before`] は構造変化が DOM へ適用される**前**、[`play_after`]
//! は適用された**後**に呼ぶ（`layout_flip::capture_before`/`play_after`
//! と同じ契約）。`Runtime::apply_update_for_dirty`/`rerender`/
//! `view_transition_swap` の該当箇所から呼ばれる。
//!
//! # ハンドル保持
//!
//! 再生中の [`fandhe_frontend_animation::flip::FlipAnimation`] は
//! [`ACTIVE`]（`(root_id, id)` → ハンドル）で保持する。`insert` による
//! 置換で旧ハンドルが drop されれば、未収束時は [`fandhe_frontend_
//! animation::flip::FlipAnimation`] の [`Drop`] が元のスタイルへ復元する
//! （`layout_flip.rs` と同じ契約）。
//!
//! ## 収束後の自己回収（Cursor Bugbot 指摘是正、イシュー #2578）
//!
//! [`ACTIVE`] の prune は [`capture_before`] からしか走らないが、
//! `Runtime::apply_with_view_transition` の UA 委譲経路は
//! `capture_before` を呼ばない。共有レイアウト遷移の再生後に VT 委譲
//! 経路だけが続く構成では、収束済みハンドルが保持する切断済み要素への
//! 強参照が回収されずに残る。そのため [`play_after_excluding`] は
//! `insert` のたびに spring の `settle_duration`（+ 余裕）経過後に
//! 同じ `token` のエントリだけを除去する `setTimeout` を仕掛ける
//! （`layout_flip::schedule_cleanup` と同じ設計。復元の要否は
//! `FlipAnimation::drop` が `is_done()` で判断し、ここでは drop する
//! だけに留める）。
//!
//! ## DOM 更新前の停止契約（codex-review P1 是正、イシュー #2578）
//!
//! [`capture_before`] は、このルート（`root_id`）の視覚矩形を捕捉した
//! **直後**（構造変化を DOM へ適用する**前**）に、このルートの [`ACTIVE`]
//! エントリを収束・未収束を問わず**すべて**停止・復元する（他ルートの
//! エントリは収束済み〔`is_done()`〕のもののみ prune し、id 数で有界に
//! 保つ）。
//!
//! 同じ id・同じ DOM 要素が旧要素・新要素の両方として残る更新（構造変化
//! を伴わない更新、あるいは [`fandhe_frontend_animation::shared_layout::
//! pair_by_id`] が `same_node` として除外する「id を維持したまま残る
//! 要素」）では、[`play_after`]/[`play_after_excluding`] は当該 id の
//! ハンドルを新規に `insert` しない。この場合、停止しなかった旧ハンドル
//! は DOM 更新後も rAF ループで書き込みを続けてしまい、その間に構造変化
//! コミット（`apply_dirty` 等）が同じ要素へ新しい `transform`/
//! `transform-origin`/`transition` を設定すると、旧アニメーションの
//! 次フレーム書き込みがそれを上書きし、収束時には `OriginalStyle::
//! restore` が更新**前**の値を復元して更新後の状態を破壊してしまう。
//! `capture_before` が DOM 更新前に必ず全ハンドルを停止・復元することで、
//! 後続の DOM 更新は常にクリーンな（進行中の FLIP 補正を持たない）
//! ベース状態へ適用される。
//!
//! `root_id`（`u64`、[`wiring::root_scope_id`]）は `capture_before`/
//! `play_after` に渡される `root`（`Runtime` のマウント先要素）の DOM
//! 参照同一性から `js_sys::WeakMap` で発行する。`ACTIVE` はモジュール
//! 単位のプロセス内唯一のマップであり、複数の `Runtime` ルートが同時に
//! マウントされる構成では、id をキーに含めないと別ルートが同じ
//! `layoutId` 文字列（アプリ側が付与する任意値、ルートをまたいで一意な
//! 保証はない）を使った場合に後発ルートの `play_after` が先発ルートの
//! ハンドルを上書き `Drop` させ、無関係な要素のアニメーションを途中終了
//! させてしまう（codex-review 指摘、イシュー #2578。`layout_flip::
//! list_instance_id` と同じ設計。`WeakMap` のキーは弱参照のため、
//! ルート要素が DOM から除去され GC されれば発行済み `root_id` も自然に
//! 回収対象になる）。

/// 要素に付け、共有レイアウト遷移の対象であることを明示するオプトイン
/// 属性名。値は id として [`ACTIVE`]/突合の `HashMap` キーに使う
/// （`layout_flip::FLIP_AUTO_ATTR` と異なり値そのものを使う点に注意。
/// CSS/セレクタへは混入しない、モジュール doc「対象要素の明示的
/// オプトイン」参照）。
pub const LAYOUT_ID_ATTR: &str = "data-fandhe-layout-id";

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::LAYOUT_ID_ATTR;
    use fandhe_frontend_animation::fandhe_animation::spring::SpringConfig;
    use fandhe_frontend_animation::flip::FlipAnimation;
    use fandhe_frontend_animation::shared_layout::{self, SharedSnapshot};
    use js_sys::WeakMap;
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, HtmlElement};

    /// [`ACTIVE`] の値。`token` は [`schedule_cleanup`] が「自分が仕掛けた
    /// エントリがまだ置き換えられていない」ことを識別するための発行番号
    /// （`layout_flip::FlipEntry` と同じ設計）。
    struct ActiveEntry {
        token: u64,
        animation: FlipAnimation,
    }

    thread_local! {
        /// `(root_id, layout_id)` ごとの進行中の共有レイアウト遷移
        /// （モジュール doc「ハンドル保持」参照）。
        static ACTIVE: RefCell<HashMap<(u64, String), ActiveEntry>> =
            RefCell::new(HashMap::new());
        /// [`ActiveEntry::token`] の発行元カウンタ。
        static NEXT_TOKEN: Cell<u64> = const { Cell::new(0) };
        /// `root`（`Runtime` のマウント先要素、DOM 参照同一性）→ `u64`
        /// 識別子（モジュール doc「ハンドル保持」参照、
        /// `layout_flip::LIST_IDS` と同じ設計）。
        static ROOT_IDS: WeakMap = WeakMap::new();
        /// [`ROOT_IDS`] の発行元カウンタ。
        static NEXT_ROOT_ID: Cell<u64> = const { Cell::new(0) };
        /// `(root_id, layout_id)` ごとに前回 `view-transition-name` を書き込んだ
        /// 要素（[`assign_transition_names`] 専用）。id が別要素へ移った・id
        /// 自体が対象外になった場合、次回呼び出し冒頭で旧要素の inline style
        /// を明示的に消去する（`assign_transition_names` doc「旧要素の名前
        /// 復元」参照。旧要素が `LAYOUT_ID_ATTR` を外れたまま DOM に残ると、
        /// 次に別要素が同じ id を名乗った際に名前重複で UA 側 View Transition
        /// が失敗する、codex-review 指摘・イシュー #2578）。
        static PREV_TRANSITION_NAMES: RefCell<HashMap<(u64, String), HtmlElement>> =
            RefCell::new(HashMap::new());
    }

    /// `root` に対応する [`ACTIVE`] キー空間分離用の識別子を取得・未発行
    /// なら新規発行する（モジュール doc「ハンドル保持」参照）。同じ DOM
    /// 要素（JS 参照同一性）へは常に同じ識別子を返す。
    fn root_scope_id(root: &Element) -> u64 {
        ROOT_IDS.with(|map| {
            let key: js_sys::Object = root.clone().unchecked_into();
            if let Some(existing) = map.get(&key).as_f64() {
                return existing as u64;
            }
            let id = NEXT_ROOT_ID.with(|cell| {
                let next = cell.get().wrapping_add(1);
                cell.set(next);
                next
            });
            map.set(&key, &JsValue::from_f64(id as f64));
            id
        })
    }

    /// `root` 配下（`root` 自身も含む）で [`LAYOUT_ID_ATTR`] を持つ要素を
    /// すべて収集し、`(id, HtmlElement)` の一覧を返す（[`capture_before`]/
    /// [`play_after`] 共通の走査。`root` 自身を含めるのは、共有レイアウト
    /// 遷移の対象が `root` 直下ではなく `root` 自身になる構成
    /// 〔単一要素をまるごと差し替える `rerender`〕にも対応するため）。
    fn collect(root: &Element) -> Vec<(String, HtmlElement)> {
        let mut result = Vec::new();
        if let Some(id) = root.get_attribute(LAYOUT_ID_ATTR) {
            if let Ok(html) = root.clone().dyn_into::<HtmlElement>() {
                result.push((id, html));
            }
        }
        let Ok(node_list) = root.query_selector_all(&format!("[{LAYOUT_ID_ATTR}]")) else {
            return result;
        };
        for i in 0..node_list.length() {
            let Some(node) = node_list.get(i) else {
                continue;
            };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            let Some(id) = element.get_attribute(LAYOUT_ID_ATTR) else {
                continue;
            };
            if let Ok(html) = element.dyn_into::<HtmlElement>() {
                result.push((id, html));
            }
        }
        result
    }

    /// `root` 配下の [`LAYOUT_ID_ATTR`] 付き各要素へ、その id を CSS
    /// `view-transition-name` として書き込む（View Transitions API への
    /// 委譲時専用、モジュール doc「呼び出しタイミング」参照）。
    ///
    /// `Runtime::apply_with_view_transition` が View Transitions 対応
    /// ブラウザでは `shared_flip = false`（本モジュールの FLIP 計測・
    /// 再生を起動せず UA 側の同名要素 morph に委譲）とするが、
    /// `data-fandhe-layout-id` を `view-transition-name` へ対応付ける配線
    /// がなければ UA は「同名要素」を認識できず、対応ブラウザでも共有
    /// レイアウト遷移が起きずページ全体遷移になってしまう（codex-review
    /// 指摘、イシュー #2578）。呼び出し元は `document.startViewTransition`
    /// 呼び出し**前**（旧要素側）と、更新コールバック内の DOM 差し替え
    /// **後**（新要素側）の 2 回本関数を呼ぶ（UA は
    /// `startViewTransition()` 呼び出し時点の DOM から旧スナップショットを
    /// 撮るため、旧要素側の名前付けは呼び出し前に同期的に完了している
    /// 必要がある。`crate::view_transition::with_view_transition` の
    /// `preset`（`view-transition-preset` 属性）と同じ制約）。
    ///
    /// [`crate::view_transition_name::is_valid_view_transition_name`] の
    /// 許可リストを満たさない id は書き込まない（fail-closed。書き込まれ
    /// なかった id は従来どおり UA からは無名要素として扱われ、ページ
    /// 全体遷移にフォールバックする、fail-safe）。この検証関数は feature
    /// `"view-transition-name"` の有無に関わらず常時コンパイルされるため
    /// （同モジュール doc参照）、本関数はその feature に依存しない
    /// （`layout-animation` 単体構成でも動作する）。
    ///
    /// # 旧要素の名前復元
    ///
    /// 書き込んだ `view-transition-name` インライン style は
    /// [`PREV_TRANSITION_NAMES`] で `(root_id, id)` → 要素を記録し、次回
    /// 呼び出し冒頭で「id が別要素へ移った」「id 自体が今回の対象から
    /// 外れた」旧要素があれば `remove_property` で明示的に消去する。旧
    /// 要素が `LAYOUT_ID_ATTR` だけ外れて DOM に残り続ける構成でも、別の
    /// 要素が同じ id を新たに名乗った時点で名前重複によるブラウザ側
    /// View Transition 失敗を防げる（codex-review 指摘、イシュー #2578）。
    ///
    /// 呼び出し元（`Runtime::apply_with_view_transition`/
    /// `apply_with_view_transition_named`）と同じ feature でゲートする
    /// （`view_transition_swap` の `#[cfg(any(...))]` と同型。いずれの
    /// feature も off の構成では呼び出し元自体が存在せず未使用になる
    /// ため、CI の wasm-full feature matrix `-wiring` ジョブ〔イシュー
    /// #2328、`layout-animation` 単体構成〕で dead-code として検知される、
    /// イシュー #2578）。
    ///
    /// 書き込む名前は [`root_scope_id`] で発行したルート固有の識別子を
    /// 前置した `fandhe-shared-<root_id>-<id>` とする（[`ACTIVE`] と同じ
    /// `root_id` 名前空間）。`id` はアプリ側が付与する任意値で、複数の
    /// `Runtime` ルートをまたいで一意な保証はない——単一 `document` 内で
    /// `view-transition-name` が重複すると UA は遷移全体をスキップする
    /// ため、素の `id` をそのまま書き込むと別ルートが同じ `id` を使った
    /// 場合に破綻する（Cursor Bugbot 指摘、イシュー #2578）。同一ルート内
    /// で `id` が重複する場合も 2 件目以降は書き込まない（`shared_layout::
    /// pair_by_id`〔`fandhe-frontend-animation` 側〕と同じ fail-safe:
    /// 曖昧な状況では何もしない側へ倒す）。
    #[cfg(any(feature = "view-transitions", feature = "view-transition-preset"))]
    pub fn assign_transition_names(root: &Element) {
        let root_id = root_scope_id(root);
        let mut seen = std::collections::HashSet::new();
        let mut current: HashMap<String, HtmlElement> = HashMap::new();
        for (id, element) in collect(root) {
            if !seen.insert(id.clone()) {
                continue;
            }
            if !crate::view_transition_name::is_valid_view_transition_name(&id) {
                continue;
            }
            current.insert(id, element);
        }

        PREV_TRANSITION_NAMES.with(|cell| {
            let mut prev = cell.borrow_mut();
            // codex-review P1 是正（イシュー #2578「View Transitions 経路
            // でも破棄済みルートの参照を回収する」）: 全ルート横断で DOM
            // から切断済み（`is_connected() == false`）の要素を掃除する
            // （`capture_before` が行う prune と同じ設計）。
            //
            // `Runtime::apply_with_view_transition`/
            // `apply_with_view_transition_named` は、View Transitions が
            // 実際に使える（UA へ委譲する）経路では `capture_before` を
            // 一切呼ばない（`view_transition_swap` 参照）。そのため、
            // ある `Runtime` ルートが常にこの VT 委譲経路だけを使って
            // マウント・破棄を繰り返す構成では、下の「このルートの前回
            // エントリ」の retain（`*pid != root_id` で他ルートを無条件に
            // 素通しする）だけでは、破棄されて二度と `assign_transition_
            // names` が呼ばれなくなったルートの `HtmlElement` 強参照が
            // 回収されないまま残り続ける（`capture_before` からしか
            // 全ルート横断 prune が走らなかった旧実装の穴）。ここで
            // 呼び出しのたびに全ルート横断で prune することで、
            // `capture_before` を経由しない経路でも回収される。
            prev.retain(|_, element| element.is_connected());
            // このルートの前回エントリのうち、今回別要素が同じ id を名乗った、
            // または id 自体が対象外になったものは、旧要素の inline style を
            // 明示的に消去してから外す（モジュール doc「旧要素の名前復元」
            // 参照。曖昧な同一性判定を避けるため DOM 参照同一性
            // 〔`is_same_node`〕で比較する）。
            prev.retain(|(pid, id), element| {
                if *pid != root_id {
                    return true;
                }
                let still_same = current
                    .get(id)
                    .is_some_and(|new_el| element.is_same_node(Some(new_el.unchecked_ref())));
                if !still_same {
                    let _ = element.style().remove_property("view-transition-name");
                }
                still_same
            });

            for (id, element) in &current {
                let name = format!("fandhe-shared-{root_id}-{id}");
                let _ = element.style().set_property("view-transition-name", &name);
                prev.insert((root_id, id.clone()), element.clone());
            }
        });
    }

    /// `root` 配下の [`LAYOUT_ID_ATTR`] 付き要素の現在の視覚矩形を捕捉する
    /// （構造変化を DOM へ適用する**前**に呼ぶ想定、モジュール doc
    /// 「呼び出しタイミング」参照）。0 件なら `query_selector_all` の
    /// reflow のみで測定コストは発生しない（[`shared_layout::snapshot`]
    /// は空の iterator に対して `measure` を 0 回呼ぶ）。
    #[must_use]
    pub fn capture_before(root: &Element) -> Option<SharedSnapshot> {
        let root_id = root_scope_id(root);
        // `PREV_TRANSITION_NAMES` の prune（codex-review 指摘、イシュー
        // #2578）。`assign_transition_names` の retain は同じ root_id が
        // 再度呼ばれた時にのみ走るため、ルート自体が DOM から除去され
        // 破棄されて二度と呼ばれなくなると、それが保持する `HtmlElement`
        // 強参照が回収されず残り続ける（メモリリーク）。`capture_before`
        // は `Runtime::apply_update_for_dirty`/`rerender` から更新のたび
        // 呼ばれるため、ここで DOM から切断済み（`is_connected() == false`）
        // の要素を全ルート横断で掃除する（`ACTIVE` の prune と同じ設計：
        // 判定は要素固有の状態のみで、他ルートへの書き込み・削除は行わない）。
        PREV_TRANSITION_NAMES.with(|cell| cell.borrow_mut().retain(|_, el| el.is_connected()));
        let entries = collect(root);
        let snapshot = if entries.is_empty() {
            None
        } else {
            Some(shared_layout::snapshot(entries))
        };
        // codex-review P1 是正（イシュー #2578「DOM 更新前に進行中の
        // 共有 FLIP を停止する」）: 現在の視覚矩形を捕捉した**後**、この
        // ルートの進行中ハンドルを DOM 更新前に停止・復元する
        // （`HashMap::retain` が偽を返したエントリを drop し、未収束
        // なら `FlipAnimation::drop` が元のスタイルへ復元する）。
        //
        // 同じ要素が旧要素・新要素の両方として突合される場合（構造変化を
        // 伴わない更新、あるいは `pair_by_id` が `same_node` として除外
        // する「id を維持したまま残る要素」）、`play_after`/
        // `play_after_excluding` は当該 id のハンドルを新規に `insert`
        // せず、ここで停止しなかった旧ハンドルが DOM 更新後もそのまま
        // rAF ループで書き込みを続ける。この間に構造変化コミット
        // （`apply_dirty` 等）が同じ要素へ新しい `transform`/
        // `transform-origin`/`transition` を設定すると、旧アニメーション
        // の次フレーム書き込みがそれを上書きし、収束時には
        // `OriginalStyle::restore` が更新**前**の値を復元して更新後の
        // 状態を破壊する（codex-review 指摘）。DOM 更新の前にここで
        // 全ハンドルを停止・復元しておけば、後続の DOM 更新は常に
        // クリーンな（進行中の FLIP 補正を持たない）ベース状態へ適用
        // される。他ルート（別 `root_id`）の収束済みエントリも同じ
        // 走査で prune する（モジュール doc「ハンドル保持」参照。判定は
        // 要素固有の状態のみで、他ルートへの書き込み・削除は行わない）。
        ACTIVE.with(|cell| {
            cell.borrow_mut()
                .retain(|(pid, _), entry| *pid != root_id && !entry.animation.is_done());
        });
        snapshot
    }

    /// `snapshot`（[`capture_before`] の結果）と `root` 配下の構造変化
    /// 反映後の現在の要素を突き合わせ、共有レイアウト遷移を再生する
    /// （構造変化を DOM へ適用した**後**に呼ぶ想定）。`snapshot` が `None`
    /// （[`capture_before`] が対象なしと判定した）場合は no-op。
    pub fn play_after(root: &Element, snapshot: Option<SharedSnapshot>) {
        play_after_excluding(root, snapshot, &[]);
    }

    /// [`play_after`] に、対象から除外する要素（とその子孫）を追加した版
    /// （codex-review 指摘、イシュー #2578）。
    ///
    /// `excluded` に含まれる要素配下の [`LAYOUT_ID_ATTR`] 付き要素は、
    /// この呼び出しでは共有レイアウト遷移の候補（`after` 集合）に含めない。
    ///
    /// `Runtime::apply_update_for_dirty` は本関数を、同一更新内で先に
    /// 走らせた [`crate::layout_flip::play_after`] が transform を適用
    /// 済みの行要素を `excluded` に渡して呼ぶ。`layout_flip::play_after`
    /// は Invert 変形を要素へ**同期的に**書き込むため、後段の本関数が
    /// 素朴に現在の視覚矩形を測ると、その transform 込みの（Last では
    /// なく First 寄りの）位置を Last として誤って捕捉してしまい、
    /// [`flip::OriginalStyle::capture`] が保持する「復元先」も
    /// `layout_flip` の transform を含んだ値になる。結果として、
    /// (1) `flip::invert` の delta 計算が狂う、(2) 本モジュールの
    /// アニメーション完了・破棄時の復元が `layout_flip` の transform を
    /// 巻き戻さないまま固定してしまう、の二重の破綻が起きる（`data-
    /// fandhe-flip-auto` 付きリストで `data-fandhe-layout-id` 付きの行を
    /// 同じキーのままタグ変更する置換ケースで顕在化。同一ノード判定
    /// （`shared_layout::pair_by_id` の `same_node` 除外）では新規ノード
    /// のため検知できない）。対象範囲は `layout_flip::play_after` が実際に
    /// transform を適用した行要素（とその子孫）のみに限定する（Bugbot
    /// 指摘是正、イシュー #2578「FLIP lists skip nested shared layout」）。
    /// `list_element` サブツリー全体を渡すと、今回の更新で動かなかった
    /// （`delta == flip::IDENTITY` で Play を起動しなかった）行に含まれる
    /// `data-fandhe-layout-id` 要素——新規挿入された行、別行へ移動した
    /// 行のうち FLIP 対象にならなかったもの——まで丸ごと除外してしまい、
    /// 共有レイアウト遷移を一度も受け取れなくなる。
    ///
    /// なお除外の粒度自体は要素単位（`excluded` に渡された行要素とその
    /// 子孫）のままとし、行の中の個々の `data-fandhe-layout-id` 要素が
    /// transform の影響を受けたかまでは追跡しない（fail-safe: 曖昧な
    /// 状況では何もしない側へ倒す、モジュール doc「突合ルール」節と同じ
    /// 方針）。
    pub fn play_after_excluding(
        root: &Element,
        snapshot: Option<SharedSnapshot>,
        excluded: &[Element],
    ) {
        let Some(snapshot) = snapshot else {
            return;
        };
        let after: Vec<(String, HtmlElement)> = collect(root)
            .into_iter()
            .filter(|(_, html)| !excluded.iter().any(|ex| ex.contains(Some(html))))
            .collect();
        if after.is_empty() {
            return;
        }
        let config = SpringConfig::default();
        let played = shared_layout::play_shared(&snapshot, &after, config);
        let root_id = root_scope_id(root);
        // `Spring::new` が理論上 `None` を返す構成（`SpringConfig::default()`
        // では起きない）でも探索上限（10 秒）を超えないため、失敗時はその
        // 上限で代替する（`layout_flip::play_row` と同じ fail-safe）。
        let settle_ms =
            fandhe_frontend_animation::fandhe_animation::spring::Spring::new(config, 0.0, 1.0, 0.0)
                .map(|spring| spring.settle_duration() * 1000.0)
                .unwrap_or(10_000.0);
        for (id, animation) in played {
            let token = NEXT_TOKEN.with(|cell| {
                let next = cell.get().wrapping_add(1);
                cell.set(next);
                next
            });
            ACTIVE.with(|cell| {
                cell.borrow_mut()
                    .insert((root_id, id.clone()), ActiveEntry { token, animation });
            });
            schedule_cleanup(root_id, id, token, settle_ms);
        }
    }

    /// `settle_ms`（+ 余裕マージン）経過後に、[`ACTIVE`] の `(root_id, id)`
    /// エントリがまだ同じ `token` を持つ場合に限り除去する（モジュール
    /// doc「収束後の自己回収」参照）。`token` 不一致（既に新しい遷移へ
    /// 置き換え済み、または `capture_before` が停止済み）なら何もしない。
    /// `window`/`setTimeout` の取得に失敗した場合はスケジュールが成立せず、
    /// 次回の `capture_before` prune か同じ id の再 `insert` まで残る
    /// （fail-safe、`layout_flip::schedule_cleanup` と同じ扱い）。
    fn schedule_cleanup(root_id: u64, id: String, token: u64, settle_ms: f64) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let delay_ms = (settle_ms * 1.2 + 100.0).min(i32::MAX as f64) as i32;
        let map_key = (root_id, id);
        let callback = Closure::once_into_js(move || {
            ACTIVE.with(|cell| {
                let mut active = cell.borrow_mut();
                if matches!(active.get(&map_key), Some(entry) if entry.token == token) {
                    active.remove(&map_key);
                }
            });
        });
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.unchecked_ref(),
            delay_ms,
        );
    }

    /// [`assign_transition_names`] の残留名クリア回帰テスト（codex-review
    /// 指摘、イシュー #2578。「旧要素の名前復元」doc 参照）。
    #[cfg(all(
        test,
        target_arch = "wasm32",
        any(feature = "view-transitions", feature = "view-transition-preset")
    ))]
    mod tests {
        use super::*;
        use wasm_bindgen_test::wasm_bindgen_test;

        wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

        /// テスト用ルート要素を作り、`document.body` へ接続して返す
        /// （codex-review P1 是正、イシュー #2578「View Transitions
        /// 経路でも破棄済みルートの参照を回収する」で
        /// `assign_transition_names` へ追加した全ルート横断の
        /// `is_connected()` prune を正しく検証するため、`document.
        /// createElement` だけで作った未接続要素ではなく実際に接続
        /// された要素を使う）。呼び出し元は戻り値をテスト末尾で
        /// `Element::remove` するか、次アサーション前に接続状態を
        /// 明示的に切断すること。
        fn make_root() -> Element {
            let document = web_sys::window().unwrap().document().unwrap();
            let root = document.create_element("div").unwrap();
            document.body().unwrap().append_child(&root).unwrap();
            root
        }

        fn make_layout_element(root: &Element, id: &str) -> Element {
            let document = web_sys::window().unwrap().document().unwrap();
            let element = document.create_element("span").unwrap();
            // `fw gate` `url_validation_check`（U1、イシュー #401）の
            // sink/guard 共起契約に合わせ、生の `set_attribute` ではなく
            // 共通ガード付きラッパーを使う（`dom.rs` doc コメント参照）。
            crate::dom::set_dom_attribute_result(&element, LAYOUT_ID_ATTR, id).unwrap();
            root.append_child(&element).unwrap();
            element
        }

        fn transition_name(element: &Element) -> String {
            element
                .clone()
                .dyn_into::<HtmlElement>()
                .unwrap()
                .style()
                .get_property_value("view-transition-name")
                .unwrap()
        }

        #[wasm_bindgen_test]
        fn stale_name_is_cleared_when_id_moves_to_a_different_element() {
            let root = make_root();
            let a = make_layout_element(&root, "shared-x");
            assign_transition_names(&root);
            assert!(
                !transition_name(&a).is_empty(),
                "A への最初の名前付けがされているはず"
            );

            // A から `LAYOUT_ID_ATTR` を外して DOM には残留させたまま、
            // 別要素 B が同じ id を新たに名乗る（codex-review 指摘の再現
            // 手順）。
            a.remove_attribute(LAYOUT_ID_ATTR).unwrap();
            let b = make_layout_element(&root, "shared-x");

            assign_transition_names(&root);

            assert!(
                transition_name(&a).is_empty(),
                "旧要素 A の残留名は消去されているはず（名前重複の防止）"
            );
            let name_b = transition_name(&b);
            assert!(!name_b.is_empty(), "新要素 B へ名前が付与されているはず");

            root.remove();
        }

        /// codex-review P1 是正回帰テスト（イシュー #2578「DOM 更新前に
        /// 進行中の共有 FLIP を停止する」）: 別要素への引き継ぎ FLIP が
        /// 進行中のまま次の `capture_before` が呼ばれると、DOM 更新前に
        /// 停止・復元されるべきである。
        #[wasm_bindgen_test]
        fn capture_before_stops_in_progress_handle_before_dom_update() {
            let root = make_root();
            let document = web_sys::window().unwrap().document().unwrap();

            // 1 回目のサイクル: 位置 A の旧要素 X1 → 位置 B の新規要素 X2
            // への「別要素への引き継ぎ」FLIP を起動する。
            // `fw gate` `url_validation_check`（U1、イシュー #401）の
            // sink/guard 共起契約に合わせ、生の `set_attribute` ではなく
            // 共通ガード付きラッパーを使う（`dom.rs` doc コメント参照。
            // `make_layout_element` と同じ方式、`76ddcaed` の先例）。
            let x1 = document.create_element("span").unwrap();
            crate::dom::set_dom_attribute_result(&x1, LAYOUT_ID_ATTR, "target").unwrap();
            crate::dom::set_dom_attribute_result(
                &x1,
                "style",
                "position:absolute;left:0px;top:0px;width:10px;height:10px",
            )
            .unwrap();
            root.append_child(&x1).unwrap();

            let snapshot1 = capture_before(&root);
            assert!(snapshot1.is_some(), "初回 capture は対象要素を捕捉するはず");

            x1.remove();
            let x2 = document.create_element("span").unwrap();
            crate::dom::set_dom_attribute_result(&x2, LAYOUT_ID_ATTR, "target").unwrap();
            crate::dom::set_dom_attribute_result(
                &x2,
                "style",
                "position:absolute;left:150px;top:80px;width:80px;height:80px",
            )
            .unwrap();
            root.append_child(&x2).unwrap();

            play_after(&root, snapshot1);

            let x2_html = x2.clone().dyn_into::<HtmlElement>().unwrap();
            assert!(
                !x2_html
                    .style()
                    .get_property_value("transform")
                    .unwrap()
                    .is_empty(),
                "play_after 直後は補正 transform が書き込まれ、アニメーション \
                 が進行中のはず"
            );

            // 2 回目の `capture_before`（DOM 更新自体はまだ起きていない）:
            // 本テストの検証対象。旧実装は `is_done()` の収束済みエントリ
            // しか prune せず、未収束の進行中ハンドルはここで停止されない
            // ため transform は書き込まれたまま残った。
            let _snapshot2 = capture_before(&root);

            assert!(
                x2_html
                    .style()
                    .get_property_value("transform")
                    .unwrap()
                    .is_empty(),
                "DOM 更新前に進行中のハンドルを停止・復元しているはず \
                 （codex-review P1「DOM 更新前に進行中の共有 FLIP を \
                 停止する」是正の検証）"
            );

            root.remove();
        }

        /// codex-review P1 是正回帰テスト（イシュー #2578「View
        /// Transitions 経路でも破棄済みルートの参照を回収する」）:
        /// あるルートが `assign_transition_names` を呼んだ後に二度と
        /// 呼ばれず破棄されても、**別ルート**が `assign_transition_names`
        /// を呼んだ時点で `PREV_TRANSITION_NAMES` から回収される
        /// （`capture_before` を経由しない VT 委譲専用の経路を模す）。
        #[wasm_bindgen_test]
        fn assign_transition_names_reclaims_disconnected_other_root_entries() {
            let root1 = make_root();
            let _a = make_layout_element(&root1, "root1-item");
            assign_transition_names(&root1);
            // root1 は以降 `capture_before`/`assign_transition_names` を
            // 一切呼ばれずに破棄される想定（VT 委譲経路専用のマウント・
            // 破棄の繰り返し）。
            root1.remove();

            let root2 = make_root();
            let _b = make_layout_element(&root2, "root2-item");
            assign_transition_names(&root2);

            let all_connected = PREV_TRANSITION_NAMES
                .with(|cell| cell.borrow().values().all(|element| element.is_connected()));
            assert!(
                all_connected,
                "assign_transition_names 呼び出し後、PREV_TRANSITION_NAMES は \
                 DOM から切断された要素の参照を保持していないはず \
                 （root1 破棄後、capture_before を経由しない root2 の \
                 呼び出しだけで回収されている必要がある）"
            );

            root2.remove();
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg(any(feature = "view-transitions", feature = "view-transition-preset"))]
pub(crate) use wiring::assign_transition_names;
#[cfg(target_arch = "wasm32")]
pub use wiring::{capture_before, play_after, play_after_excluding};
