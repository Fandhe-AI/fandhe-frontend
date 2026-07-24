//! FileUpload（`fandhe-frontend-headless-ui` `file_upload` モジュール）の
//! `input[type=file]`/ドロップゾーンからの実ファイル選択検知グルー
//! （イシュー #840、親トラッキング #520）。
//!
//! `crates/headless-ui/src/file_upload.rs` は `File` オブジェクト自体を一切
//! 保持しない設計（同モジュール冒頭 rustdoc「保留解除」節参照）であるため、
//! `input[type=file]` の `change` イベント・ドロップゾーンの `drop` イベント
//! から実 `File` オブジェクトへ接触してメタデータ（name/size/mime type）へ
//! 変換する処理は、本クレート（wasm 層）に隔離する。これが
//! `docs/policy/intentional-non-adoption.md` §7 の再評価トリガー
//! 「`File` API 依存部分を wasm-full 側の限定配線に閉じ込める」を実現する
//! 部分である。
//!
//! # 2 層構成（`events.rs`/`keynav.rs`/`headless_avatar.rs` と同じ方針）
//!
//! DOM 非依存の純粋ロジック層（クリック/ドラッグイベントの種別判定、
//! ファイルメタデータへの変換）と `#[cfg(target_arch = "wasm32")]` 配線層
//! （実 DOM イベント登録・`File`/`FileList`/`DataTransfer` API 接触）に分離
//! する。純粋ロジック層は native `cargo test` で検証可能。
//!
//! # `Runtime<C>::mount`/`Runtime::hydrate` へ自動配線しない理由（意図的な
//! スコープ限定、`out-of-scope-tracking.md` に従い Issue 化を提案する）
//!
//! [`headless_avatar::wire_avatar_events`] 等の既存配線は「文字列 dispatch
//! （`Component::decode_action` が受理する `&str` アクション名）のみで完結する」
//! ため、`Runtime<C>` が `C: Component` という総称境界のまま自動配線できた。
//! 一方 FileUpload の [`fandhe_frontend_headless_ui::file_upload::FileUploadAction::AddFiles`]
//! は型付き API 限定（`crates/headless-ui/src/file_upload.rs` 冒頭 rustdoc
//! 「dispatch 契約」節）であり、`Component` トレイトの総称境界だけでは
//! 「`C` が `FileUpload` を含む」ことを表現できない。したがって本モジュールの
//! [`wire_file_upload_component`] は具象型 `FileUpload` に特化した API として
//! 提供し、`Runtime<C>::mount`/`Runtime::hydrate` への総称的な自動配線は行わない
//! （`headless_avatar::wire_avatar_events` が #711 で汎化される前の #591 時点と
//! 同型の段階的スコープ）。FileUpload を使うアプリは
//! [`wire_file_upload_component`] を `Runtime::mount`/`Runtime::hydrate` 呼び出し
//! 後に明示的に呼び出す。`Component` トレイトを拡張して型付きアクションを
//! 一般化する設計（総称自動配線の実現）は本イシューのスコープ外として
//! Issue 化を提案する。
//!
//! # 他クレート・他モジュールとの契約
//!
//! - [`click_action_for_target`]/[`dropzone_dragging_state_for_event`] が
//!   判定するアクション名（`"remove"`/`"clear"`）は
//!   `fandhe_frontend_headless_ui::file_upload::FileUpload::decode_action`
//!   の対応する分岐と一致する。
//! - [`files_from_metadata`] は
//!   `fandhe_frontend_headless_ui::file_upload::FileUploadItem` を組み立てる
//!   薄い変換であり、`File` オブジェクトの内容は一切読まない
//!   （`FileReader` 不使用、DoS 面のメモリ膨張経路がない）。
//! - 状態更新・DOM 反映のいずれも HTML 文字列を組み立てない（REQ-1）。DOM
//!   反映は `set_attribute`/`remove_attribute`/`HtmlElement::click`/
//!   `HtmlInputElement::set_value` のみで、属性名・属性値はすべて
//!   `&'static str` リテラル（不変条件、`.claude/rules/coding-rust.md`）。

/// FileUpload の `data-scope` 属性値（`fandhe_frontend_headless_ui::file_upload`
/// の `ANATOMY` と一致、`crates/headless-ui/src/file_upload.rs` 参照）。
const FILE_UPLOAD_SCOPE: &str = "file-upload";
/// Dropzone パーツの `data-part` 属性値。
const DROPZONE_PART: &str = "dropzone";
/// Trigger パーツの `data-part` 属性値。
const TRIGGER_PART: &str = "trigger";
/// Item パーツの `data-part` 属性値。
const ITEM_PART: &str = "item";
/// ItemDeleteTrigger パーツの `data-part` 属性値。
const ITEM_DELETE_TRIGGER_PART: &str = "item-delete-trigger";
/// ClearTrigger パーツの `data-part` 属性値。
const CLEAR_TRIGGER_PART: &str = "clear-trigger";
/// HiddenInput パーツの `data-part` 属性値。
const HIDDEN_INPUT_PART: &str = "hidden-input";

/// dispatch アクション名 "remove"（`FileUpload::decode_action` と一致）。
const ACTION_REMOVE: &str = "remove";
/// dispatch アクション名 "clear"（`FileUpload::decode_action` と一致）。
const ACTION_CLEAR: &str = "clear";

/// クリックイベントのターゲット属性・（削除操作の場合の）item インデックスから
/// 文字列 dispatch すべきアクションを判定する（DOM 非依存の純粋関数、native
/// `cargo test` で検証可能）。
///
/// `scope` が一致しない場合は常に `None`（fail-closed、改ざんされた
/// `data-*` を持つ無関係要素上のイベントを dispatch へ流さない）。
/// `ClearTrigger` は `("clear", "")` を、`ItemDeleteTrigger` は
/// `item_index` が `Some` の場合のみ `("remove", "<index>")` を返す
/// （`item_index` が `None` の場合はインデックスを特定できなかったことを
/// 意味し、誤ったインデックスで削除しないよう no-op とする）。
#[must_use]
pub fn click_action_for_target(
    scope: Option<&str>,
    part: Option<&str>,
    item_index: Option<usize>,
) -> Option<crate::events::ActionRef> {
    if scope != Some(FILE_UPLOAD_SCOPE) {
        return None;
    }
    match part {
        Some(CLEAR_TRIGGER_PART) => Some(crate::events::ActionRef {
            action: ACTION_CLEAR.to_string(),
            payload: String::new(),
        }),
        Some(ITEM_DELETE_TRIGGER_PART) => item_index.map(|idx| crate::events::ActionRef {
            action: ACTION_REMOVE.to_string(),
            payload: idx.to_string(),
        }),
        _ => None,
    }
}

/// クリックターゲットが Trigger パーツかどうかを判定する（DOM 非依存の
/// 純粋関数）。Trigger クリックは dispatch を経由せず、配線層が
/// [`hidden_input`]（`HIDDEN_INPUT_PART`）へ `click()` を転送するピッカー
/// 起動専用の合図であるため、[`click_action_for_target`] とは別関数にする。
#[must_use]
pub fn is_trigger_click(scope: Option<&str>, part: Option<&str>) -> bool {
    scope == Some(FILE_UPLOAD_SCOPE) && part == Some(TRIGGER_PART)
}

/// ドラッグ&ドロップイベント種別・ターゲット属性から dropzone の
/// `data-dragging` 反映値を判定する（DOM 非依存の純粋関数）。
///
/// `scope`/`part` が Dropzone と一致しない場合は常に `None`。
/// `"dragenter"`/`"dragover"` → `Some(true)`、`"dragleave"`/`"drop"` →
/// `Some(false)`、それ以外のイベント種別は `None`（判定対象外）。
#[must_use]
pub fn dropzone_dragging_state_for_event(
    event_type: &str,
    scope: Option<&str>,
    part: Option<&str>,
) -> Option<bool> {
    if scope != Some(FILE_UPLOAD_SCOPE) || part != Some(DROPZONE_PART) {
        return None;
    }
    match event_type {
        "dragenter" | "dragover" => Some(true),
        "dragleave" | "drop" => Some(false),
        _ => None,
    }
}

/// クリックターゲットが Item パーツと一致するかを判定するための `data-part`
/// リテラル参照（配線層が `closest` セレクタ組み立てに使う）。
#[must_use]
pub const fn item_part() -> &'static str {
    ITEM_PART
}

/// `data-scope`/`data-part` リテラル参照（配線層がセレクタ組み立てに使う）。
#[must_use]
pub const fn scope_and_hidden_input_part() -> (&'static str, &'static str) {
    (FILE_UPLOAD_SCOPE, HIDDEN_INPUT_PART)
}

/// 実 `File` から読み取った `(name, size, mime_type)` 列を
/// [`fandhe_frontend_headless_ui::file_upload::FileUploadItem`] 列へ変換する
/// 薄い純粋関数（`File` オブジェクトの内容は読まない前提を保つため、
/// 呼び出し側の配線層は名前・サイズ・MIME タイプのみを渡す）。
#[must_use]
pub fn files_from_metadata(
    names: Vec<String>,
    sizes: Vec<u64>,
    mimes: Vec<String>,
) -> Vec<fandhe_frontend_headless_ui::file_upload::FileUploadItem> {
    names
        .into_iter()
        .zip(sizes)
        .zip(mimes)
        .map(|((name, size), mime)| {
            fandhe_frontend_headless_ui::file_upload::FileUploadItem::new(name, size, mime)
        })
        .collect()
}

// ---------------------------------------------------------------------
// 配線層: web-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`events.rs`/`keynav.rs`/`headless_avatar.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        click_action_for_target, dropzone_dragging_state_for_event, files_from_metadata,
        is_trigger_click, item_part, scope_and_hidden_input_part, DROPZONE_PART, FILE_UPLOAD_SCOPE,
        HIDDEN_INPUT_PART, ITEM_DELETE_TRIGGER_PART,
    };
    use fandhe_frontend_headless_ui::file_upload::{FileUpload, FileUploadAction, FileUploadItem};
    use fandhe_frontend_interactive::{dispatch, Component};
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{
        DataTransfer, DragEvent, Element, Event, FileList, HtmlElement, HtmlInputElement,
    };

    /// `FileList`（`input.files()`/`DataTransfer::files()` の戻り値）の
    /// 各 `File` から name/size/type のみを読み取り、内容は一切読まない
    /// （`FileReader` 不使用、モジュール冒頭 rustdoc「他クレート・他モジュール
    /// との契約」節参照）。
    fn extract_file_list_items(list: &FileList) -> Vec<FileUploadItem> {
        let len = list.length();
        let mut names = Vec::with_capacity(len as usize);
        let mut sizes = Vec::with_capacity(len as usize);
        let mut mimes = Vec::with_capacity(len as usize);
        for i in 0..len {
            if let Some(file) = list.get(i) {
                names.push(file.name());
                // `File::size()` は `f64`（Web IDL `double`）。バイト数として
                // 非負整数域に丸める（負値・NaN は 0 として扱い fail-closed）。
                let size = file.size();
                sizes.push(if size.is_finite() && size > 0.0 {
                    size as u64
                } else {
                    0
                });
                mimes.push(file.type_());
            }
        }
        files_from_metadata(names, sizes, mimes)
    }

    /// `root` 配下で `item_el` と `[data-scope="file-upload"][data-part="item"]`
    /// が一致する要素の出現順インデックスを求める。`query_selector_all` の
    /// 失敗時・`item_el` が見つからない場合は `None`（fail-closed、誤った
    /// インデックスで削除しない）。
    fn compute_item_index(root: &Element, item_el: &Element) -> Option<usize> {
        let selector = format!(
            "[data-scope=\"{FILE_UPLOAD_SCOPE}\"][data-part=\"{}\"]",
            item_part()
        );
        let Ok(list) = root.query_selector_all(&selector) else {
            return None;
        };
        let len = list.length();
        for i in 0..len {
            if let Some(node) = list.get(i) {
                if node.is_same_node(Some(item_el.unchecked_ref())) {
                    return Some(i as usize);
                }
            }
        }
        None
    }

    /// `element.set_attribute(name, value)` の薄いガード付きラッパー
    /// （イシュー #401 の `fw gate` `url_validation_check` 契約に準拠、
    /// `.claude/rules/security.md`。`focus_visible.rs::set_dom_attribute` と
    /// 同じ方針）。本モジュールが書き込む属性（`data-dragging`）は
    /// `&'static str` リテラルで固定された非 URL・非イベントハンドラ属性で
    /// あり値も常に空文字列だが、`fandhe_frontend_core` のガード関数群
    /// （`is_event_handler_attr`/`is_url_attr`/`is_safe_url`/
    /// `is_safe_srcset`）を経由することで、将来 `name`/`value` が動的な
    /// 入力から組み立てられるよう変更された場合の防御としても機能する。
    fn set_dom_attribute(element: &Element, name: &str, value: &str) -> Result<(), JsValue> {
        if fandhe_frontend_core::is_event_handler_attr(name) {
            return Ok(());
        }
        if fandhe_frontend_core::is_url_attr(name) && !fandhe_frontend_core::is_safe_url(value) {
            return Ok(());
        }
        if name.eq_ignore_ascii_case("srcset") && !fandhe_frontend_core::is_safe_srcset(value) {
            return Ok(());
        }
        element.set_attribute(name, value)
    }

    /// `data-dragging` 存在属性を反映する（`fandhe_frontend_core::render` を
    /// 経由した再描画は行わず、既存 DOM の属性のみを書き換える。
    /// `headless_avatar::wiring::apply_avatar_visibility` と同じ方針）。
    fn set_dragging_attr(element: &Element, dragging: bool) -> Result<(), JsValue> {
        if dragging {
            set_dom_attribute(element, "data-dragging", "")
        } else {
            element.remove_attribute("data-dragging")
        }
    }

    /// `root` 配下の FileUpload パーツへイベント委譲を配線し、
    /// [`FileUpload`] 状態機械（具象型、モジュール冒頭 rustdoc
    /// 「`Runtime<C>` へ自動配線しない理由」節参照）を更新する。
    ///
    /// 配線するイベント（すべて `root` へのバブリング委譲。`click`/
    /// `change`/`drag*`/`drop` はいずれもバブリングするため
    /// `events.rs::wire_events` と同じ非 capture 委譲で足りる）:
    ///
    /// - `click`: Trigger → hidden-input への `click()` 転送（ピッカー起動、
    ///   状態は変えない）。ClearTrigger → `"clear"` 文字列 dispatch。
    ///   ItemDeleteTrigger → 出現順インデックスを [`compute_item_index`] で
    ///   求めて `"remove"` 文字列 dispatch。
    /// - `change`: HiddenInput → `HtmlInputElement::files()` から
    ///   [`FileUploadAction::AddFiles`]（型付き dispatch）。処理後
    ///   `input.set_value("")` で同一ファイルの再選択を可能にする。
    /// - `dragenter`/`dragover`/`dragleave`/`drop`: Dropzone →
    ///   `data-dragging` のトグル（[`dropzone_dragging_state_for_event`]）。
    ///   `dragenter`/`dragover`/`drop` は既定動作（ブラウザのファイルオープン
    ///   ナビゲーション）を `prevent_default()` で抑止する。`drop` は
    ///   `DataTransfer::files()` から [`FileUploadAction::AddFiles`]
    ///   （型付き dispatch）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback` の失敗を伝播する。
    pub fn wire_file_upload_component(
        root: Element,
        component: std::rc::Rc<std::cell::RefCell<FileUpload>>,
        mut on_update: impl FnMut(&FileUpload, &Element) + 'static,
    ) -> Result<(), JsValue> {
        let on_update = std::rc::Rc::new(std::cell::RefCell::new(
            move |state: &FileUpload, el: &Element| {
                on_update(state, el);
            },
        ));

        wire_click(&root, component.clone(), on_update.clone())?;
        wire_change(&root, component.clone(), on_update.clone())?;
        wire_drag_and_drop(&root, component, on_update)?;

        Ok(())
    }

    fn dispatch_and_update(
        root: &Element,
        component: &std::rc::Rc<std::cell::RefCell<FileUpload>>,
        on_update: &std::rc::Rc<std::cell::RefCell<impl FnMut(&FileUpload, &Element) + 'static>>,
        action: &str,
        payload: &str,
    ) {
        let Ok(mut state) = component.try_borrow_mut() else {
            return;
        };
        if !dispatch(&mut *state, action, payload) {
            return;
        }
        if let Ok(mut cb) = on_update.try_borrow_mut() {
            (cb)(&state, root);
        }
    }

    fn add_files_and_update(
        root: &Element,
        component: &std::rc::Rc<std::cell::RefCell<FileUpload>>,
        on_update: &std::rc::Rc<std::cell::RefCell<impl FnMut(&FileUpload, &Element) + 'static>>,
        items: Vec<FileUploadItem>,
    ) {
        if items.is_empty() {
            return;
        }
        let Ok(mut state) = component.try_borrow_mut() else {
            return;
        };
        state.update(FileUploadAction::AddFiles(items));
        if let Ok(mut cb) = on_update.try_borrow_mut() {
            (cb)(&state, root);
        }
    }

    fn wire_click(
        root: &Element,
        component: std::rc::Rc<std::cell::RefCell<FileUpload>>,
        on_update: std::rc::Rc<std::cell::RefCell<impl FnMut(&FileUpload, &Element) + 'static>>,
    ) -> Result<(), JsValue> {
        let click_root = root.clone();
        let closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event.target() else {
                return;
            };
            let Some(element) = target.dyn_ref::<Element>() else {
                return;
            };
            if !click_root.contains(Some(element)) {
                return;
            }
            let scope = element.get_attribute("data-scope");
            let part = element.get_attribute("data-part");

            if is_trigger_click(scope.as_deref(), part.as_deref()) {
                let (fu_scope, hidden_part) = scope_and_hidden_input_part();
                let selector = format!("[data-scope=\"{fu_scope}\"][data-part=\"{hidden_part}\"]");
                if let Ok(Some(hidden)) = click_root.query_selector(&selector) {
                    if let Ok(html_el) = hidden.dyn_into::<HtmlElement>() {
                        html_el.click();
                    }
                }
                return;
            }

            let item_index = if part.as_deref() == Some(ITEM_DELETE_TRIGGER_PART) {
                let selector = format!(
                    "[data-scope=\"{FILE_UPLOAD_SCOPE}\"][data-part=\"{}\"]",
                    item_part()
                );
                element
                    .closest(&selector)
                    .ok()
                    .flatten()
                    .and_then(|item_el| compute_item_index(&click_root, &item_el))
            } else {
                None
            };

            let Some(action_ref) =
                click_action_for_target(scope.as_deref(), part.as_deref(), item_index)
            else {
                return;
            };
            dispatch_and_update(
                &click_root,
                &component,
                &on_update,
                &action_ref.action,
                &action_ref.payload,
            );
        });
        root.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        // `Closure::forget`: マウント時 1 回・定数個リーク契約
        // （`events.rs::wire_events`/`headless_avatar.rs` と同じ方針）。
        closure.forget();
        Ok(())
    }

    fn wire_change(
        root: &Element,
        component: std::rc::Rc<std::cell::RefCell<FileUpload>>,
        on_update: std::rc::Rc<std::cell::RefCell<impl FnMut(&FileUpload, &Element) + 'static>>,
    ) -> Result<(), JsValue> {
        let change_root = root.clone();
        let closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event.target() else {
                return;
            };
            let Some(element) = target.dyn_ref::<Element>() else {
                return;
            };
            if !change_root.contains(Some(element)) {
                return;
            }
            let scope = element.get_attribute("data-scope");
            let part = element.get_attribute("data-part");
            if scope.as_deref() != Some(FILE_UPLOAD_SCOPE)
                || part.as_deref() != Some(HIDDEN_INPUT_PART)
            {
                return;
            }
            let Ok(input) = element.clone().dyn_into::<HtmlInputElement>() else {
                return;
            };
            let items = input
                .files()
                .map(|list| extract_file_list_items(&list))
                .unwrap_or_default();
            // 同一ファイルの再選択を可能にするため、処理直後に値をクリアする
            // （ネイティブ `<input type="file">` の慣習的挙動、`change` は
            // 同一パスの再選択では再発火しないブラウザ実装が多いため）。
            input.set_value("");
            add_files_and_update(&change_root, &component, &on_update, items);
        });
        root.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;
        closure.forget();
        Ok(())
    }

    fn wire_drag_and_drop(
        root: &Element,
        component: std::rc::Rc<std::cell::RefCell<FileUpload>>,
        on_update: std::rc::Rc<std::cell::RefCell<impl FnMut(&FileUpload, &Element) + 'static>>,
    ) -> Result<(), JsValue> {
        for event_type in ["dragenter", "dragover", "dragleave", "drop"] {
            let drag_root = root.clone();
            let component = component.clone();
            let on_update = on_update.clone();
            let closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
                let Some(target) = event.target() else {
                    return;
                };
                let Some(element) = target.dyn_ref::<Element>() else {
                    return;
                };
                if !drag_root.contains(Some(element)) {
                    return;
                }

                // Root・Dropzone いずれも `div`（`role="button"`）であり
                // ネイティブ `disabled` 属性を持てないため、native
                // `<button>`/`<input>` と異なり `disabled` を付与しても
                // ブラウザは drag/drop を自動抑止しない。無効化状態
                // （`fandhe_frontend_headless_ui::file_upload::root`/
                // `dropzone` がいずれも `disabled` から反映する
                // `data-disabled`、`crates/headless-ui/src/file_upload.rs`
                // 参照）を明示チェックし、無効化時はドラッグ&ドロップ操作を
                // すべて無視する（PR #868 Cursor Bugbot 指摘: 無効化した
                // dropzone でもドロップでファイルが追加できてしまう不具合の
                // 修正）。Root 自体に `data-disabled` が付与されている場合、
                // または Dropzone パーツ（イベントターゲット自身を含む祖先
                // 方向）に `data-disabled` が付与されている場合のいずれかで
                // 無効化とみなす（`root`/`dropzone` のどちらから disabled が
                // 伝播していても取りこぼさないための fail-closed 判定）。
                if drag_root.has_attribute("data-disabled") {
                    return;
                }
                let disabled_dropzone_selector = format!(
                    "[data-scope=\"{FILE_UPLOAD_SCOPE}\"][data-part=\"{DROPZONE_PART}\"][data-disabled]"
                );
                if element
                    .closest(&disabled_dropzone_selector)
                    .ok()
                    .flatten()
                    .is_some()
                {
                    return;
                }

                let scope = element.get_attribute("data-scope");
                let part = element.get_attribute("data-part");
                let Some(dragging) = dropzone_dragging_state_for_event(
                    &event.type_(),
                    scope.as_deref(),
                    part.as_deref(),
                ) else {
                    return;
                };

                // dragenter/dragover/drop の既定動作（ブラウザのファイル
                // オープンナビゲーション）を抑止する。dragleave は既定動作を
                // 抑止する必要がない。
                if event.type_() != "dragleave" {
                    event.prevent_default();
                }

                let _ = set_dragging_attr(element, dragging);

                if event.type_() == "drop" {
                    if let Some(drag_event) = event.dyn_ref::<DragEvent>() {
                        if let Some(data_transfer) = drag_event.data_transfer() {
                            let items = extract_data_transfer_items(&data_transfer);
                            add_files_and_update(&drag_root, &component, &on_update, items);
                        }
                    }
                }
            });
            root.add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        Ok(())
    }

    /// `DataTransfer::files()` から [`FileUploadItem`] 列を組み立てる
    /// （[`extract_file_list_items`] と同じくメタデータのみ読む）。
    fn extract_data_transfer_items(data_transfer: &DataTransfer) -> Vec<FileUploadItem> {
        data_transfer
            .files()
            .map(|list| extract_file_list_items(&list))
            .unwrap_or_default()
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_file_upload_component;

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_headless_ui::file_upload::{FileUpload, FileUploadAction, FileUploadItem};
    use fandhe_frontend_interactive::{dispatch, Component};

    // --- click_action_for_target ---

    #[test]
    fn clear_trigger_click_dispatches_clear() {
        let action_ref =
            click_action_for_target(Some("file-upload"), Some("clear-trigger"), None).unwrap();
        assert_eq!(action_ref.action, "clear");
        assert_eq!(action_ref.payload, "");
    }

    #[test]
    fn item_delete_trigger_click_dispatches_remove_with_index() {
        let action_ref =
            click_action_for_target(Some("file-upload"), Some("item-delete-trigger"), Some(2))
                .unwrap();
        assert_eq!(action_ref.action, "remove");
        assert_eq!(action_ref.payload, "2");
    }

    #[test]
    fn item_delete_trigger_click_without_index_is_ignored() {
        assert_eq!(
            click_action_for_target(Some("file-upload"), Some("item-delete-trigger"), None),
            None
        );
    }

    #[test]
    fn mismatched_scope_is_ignored() {
        assert_eq!(
            click_action_for_target(Some("attacker"), Some("clear-trigger"), None),
            None
        );
    }

    #[test]
    fn unrelated_part_is_ignored() {
        assert_eq!(
            click_action_for_target(Some("file-upload"), Some("root"), None),
            None
        );
    }

    // --- is_trigger_click ---

    #[test]
    fn trigger_click_is_detected() {
        assert!(is_trigger_click(Some("file-upload"), Some("trigger")));
    }

    #[test]
    fn non_trigger_click_is_not_detected() {
        assert!(!is_trigger_click(Some("file-upload"), Some("dropzone")));
        assert!(!is_trigger_click(Some("attacker"), Some("trigger")));
    }

    // --- dropzone_dragging_state_for_event ---

    #[test]
    fn dragenter_and_dragover_on_dropzone_yield_dragging_true() {
        assert_eq!(
            dropzone_dragging_state_for_event("dragenter", Some("file-upload"), Some("dropzone")),
            Some(true)
        );
        assert_eq!(
            dropzone_dragging_state_for_event("dragover", Some("file-upload"), Some("dropzone")),
            Some(true)
        );
    }

    #[test]
    fn dragleave_and_drop_on_dropzone_yield_dragging_false() {
        assert_eq!(
            dropzone_dragging_state_for_event("dragleave", Some("file-upload"), Some("dropzone")),
            Some(false)
        );
        assert_eq!(
            dropzone_dragging_state_for_event("drop", Some("file-upload"), Some("dropzone")),
            Some(false)
        );
    }

    #[test]
    fn unrelated_event_type_is_ignored() {
        assert_eq!(
            dropzone_dragging_state_for_event("click", Some("file-upload"), Some("dropzone")),
            None
        );
    }

    #[test]
    fn mismatched_scope_or_part_is_ignored_for_dragging() {
        assert_eq!(
            dropzone_dragging_state_for_event("dragenter", Some("attacker"), Some("dropzone")),
            None
        );
        assert_eq!(
            dropzone_dragging_state_for_event("dragenter", Some("file-upload"), Some("trigger")),
            None
        );
    }

    // --- files_from_metadata ---

    #[test]
    fn files_from_metadata_zips_name_size_mime() {
        let items = files_from_metadata(
            vec!["a.png".to_string(), "b.pdf".to_string()],
            vec![100, 200],
            vec!["image/png".to_string(), "application/pdf".to_string()],
        );
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], FileUploadItem::new("a.png", 100, "image/png"));
        assert_eq!(
            items[1],
            FileUploadItem::new("b.pdf", 200, "application/pdf")
        );
    }

    // --- ドリフト検知: headless-ui の decode_action がここで判定するアクション
    // 名を受理すること。

    #[test]
    fn decode_action_accepts_remove_and_clear() {
        assert!(<FileUpload as Component>::decode_action("remove", "0").is_some());
        assert!(<FileUpload as Component>::decode_action("clear", "").is_some());
        assert!(<FileUpload as Component>::decode_action("add-files", "x").is_none());
    }

    // --- roundtrip: click_action_for_target → dispatch → 状態確認 ---

    #[test]
    fn clear_click_roundtrip_empties_state() {
        let mut f = FileUpload::default();
        f.update(FileUploadAction::AddFiles(vec![FileUploadItem::new(
            "a", 1, "",
        )]));
        let action_ref =
            click_action_for_target(Some("file-upload"), Some("clear-trigger"), None).unwrap();
        assert!(dispatch(&mut f, &action_ref.action, &action_ref.payload));
        assert!(f.is_empty());
    }

    #[test]
    fn remove_click_roundtrip_removes_indexed_file() {
        let mut f = FileUpload::default();
        f.update(FileUploadAction::AddFiles(vec![
            FileUploadItem::new("a", 1, ""),
            FileUploadItem::new("b", 1, ""),
        ]));
        let action_ref =
            click_action_for_target(Some("file-upload"), Some("item-delete-trigger"), Some(0))
                .unwrap();
        assert!(dispatch(&mut f, &action_ref.action, &action_ref.payload));
        assert_eq!(f.accepted()[0].name, "b");
    }

    // --- XSS 回帰: 実ファイル名にスクリプト断片があっても、AddFiles →
    // headless-ui の render() 経路で必ずエスケープされる（webview 統合の
    // browser テスト側が実 File で検証、native 側は AddFiles → render の
    // 契約を固定する）。

    #[test]
    fn add_files_with_script_payload_name_then_render_escapes() {
        use fandhe_frontend_core::{render, text};
        use fandhe_frontend_headless_ui::file_upload::item_name;

        let items = files_from_metadata(
            vec!["<script>alert(1)</script>".to_string()],
            vec![10],
            vec!["text/plain".to_string()],
        );
        let mut f = FileUpload::default();
        f.update(FileUploadAction::AddFiles(items));
        let html = render(&item_name(vec![], vec![text(&f.accepted()[0].name)]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
