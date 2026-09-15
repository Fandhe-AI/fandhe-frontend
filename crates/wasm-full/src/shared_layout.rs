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
//! [`ACTIVE`]（`(root_id, id)` → ハンドル）で保持する。[`capture_before`]
//! の呼び出しごとに収束済み（`is_done()`）のエントリを prune し、id 数で
//! 有界に保つ（`layout_flip::FLIP_LOOPS` のような `setTimeout` 自己
//! クリーンアップは持たない——`layoutId` の総数は通常アプリの静的な部品数
//! に収まり、次回更新で必ず prune 機会が来るため。`ponytail:` id 数が
//! 数千件規模で増え続ける構成では prune 頻度が不足し得る。上限が必要に
//! なれば `layout_flip::FLIP_LOOPS` と同型の `setTimeout` クリーンアップへ
//! 拡張する）。`insert` による置換で旧ハンドルが drop されれば、未収束時
//! は [`fandhe_frontend_animation::flip::FlipAnimation`] の [`Drop`] が
//! 元のスタイルへ復元する（`layout_flip.rs` と同じ契約）。
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
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, HtmlElement};

    thread_local! {
        /// `(root_id, layout_id)` ごとの進行中の共有レイアウト遷移
        /// （モジュール doc「ハンドル保持」参照）。
        static ACTIVE: RefCell<HashMap<(u64, String), FlipAnimation>> =
            RefCell::new(HashMap::new());
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
        // 収束済みエントリの prune（モジュール doc「ハンドル保持」参照）。
        // 他ルートのエントリ（別 root_id）も同じ prune 走査に混ざるが、
        // 判定は `is_done()`（自ハンドル固有の状態）のみで、ルート横断の
        // 書き込み・削除は行わないため問題ない。
        ACTIVE.with(|cell| cell.borrow_mut().retain(|_, anim| !anim.is_done()));
        let entries = collect(root);
        if entries.is_empty() {
            return None;
        }
        Some(shared_layout::snapshot(entries))
    }

    /// `snapshot`（[`capture_before`] の結果）と `root` 配下の構造変化
    /// 反映後の現在の要素を突き合わせ、共有レイアウト遷移を再生する
    /// （構造変化を DOM へ適用した**後**に呼ぶ想定）。`snapshot` が `None`
    /// （[`capture_before`] が対象なしと判定した）場合は no-op。
    pub fn play_after(root: &Element, snapshot: Option<SharedSnapshot>) {
        let Some(snapshot) = snapshot else {
            return;
        };
        let after = collect(root);
        if after.is_empty() {
            return;
        }
        let played = shared_layout::play_shared(&snapshot, &after, SpringConfig::default());
        let root_id = root_scope_id(root);
        ACTIVE.with(|cell| {
            let mut active = cell.borrow_mut();
            for (id, animation) in played {
                active.insert((root_id, id), animation);
            }
        });
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

        fn make_root() -> Element {
            let document = web_sys::window().unwrap().document().unwrap();
            document.create_element("div").unwrap()
        }

        fn make_layout_element(root: &Element, id: &str) -> Element {
            let document = web_sys::window().unwrap().document().unwrap();
            let element = document.create_element("span").unwrap();
            element.set_attribute(LAYOUT_ID_ATTR, id).unwrap();
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
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg(any(feature = "view-transitions", feature = "view-transition-preset"))]
pub(crate) use wiring::assign_transition_names;
#[cfg(target_arch = "wasm32")]
pub use wiring::{capture_before, play_after};
