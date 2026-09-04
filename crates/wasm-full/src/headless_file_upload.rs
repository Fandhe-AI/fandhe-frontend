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

    /// Item 系パーツの `data-type` 属性値のうち「受理済み」を表す固定リテラル
    /// （`fandhe_frontend_headless_ui::file_upload::ItemType::Accepted::as_str()`
    /// と同値。`FileUploadAction::Remove` が `accepted` 一覧のみを対象とする
    /// ことに対応し、[`compute_item_index`] の走査を `data-type="accepted"` の
    /// item 要素に限定するために使う、イシュー #1609 Cursor Bugbot 指摘の是正）。
    const ACCEPTED_ITEM_TYPE: &str = "accepted";

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

    /// `root` 配下で `item_el` と
    /// `[data-scope="file-upload"][data-part="item"][data-type="accepted"]`
    /// が一致する要素の出現順インデックスを求める。
    /// [`fandhe_frontend_headless_ui::file_upload::FileUploadAction::Remove`]
    /// は `accepted` 一覧（`data-type="accepted"`）のみをインデックス対象と
    /// するため、選択条件も `data-type="accepted"` に限定する
    /// （イシュー #1609 Cursor Bugbot 指摘の是正: `data-type` を区別せず
    /// 数えると、accepted/rejected を同一 root に描画するデモ構成で
    /// rejected item の削除操作が誤った accepted ファイルを削除しうる、
    /// または no-op になる）。`query_selector_all` の失敗時・`item_el` が
    /// 見つからない場合は `None`（fail-closed、誤ったインデックスで
    /// 削除しない）。
    fn compute_item_index(root: &Element, item_el: &Element) -> Option<usize> {
        let selector = format!(
            "[data-scope=\"{FILE_UPLOAD_SCOPE}\"][data-part=\"{}\"][data-type=\"{ACCEPTED_ITEM_TYPE}\"]",
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
        // イシュー #1609 Cursor Bugbot 指摘の是正: `hidden_input` が
        // `props.required` のときネイティブ `required` を出力するが、
        // `wire_change` は同一ファイルの再選択を可能にするため
        // `change` のたびに `input.set_value("")` でネイティブ値を
        // 空にする（`required` 属性自体は残ったまま）。ネイティブの
        // required 制約検証は「value が空か」だけを見るため、
        // `FileUpload` 状態にファイルが入っていてもフォーム送信が
        // ブロックされうる。配線時点（初回マウント/ハイドレーション後、
        // 状態変更が一切起きていない時点）の hidden-input 要素が
        // `required` 属性を持つかどうかを一度だけ記録し（呼び出し側の
        // 意図＝`props.required` の値をここから逆算する）、以降は
        // 状態更新のたびに [`sync_hidden_input_required`] が
        // `accepted` が非空の間だけネイティブ `required` を除去する
        // （空に戻れば再付与し、未入力のままの送信は引き続き阻止する）。
        let required_intent = hidden_input_required_intent(&root);
        let on_update = std::rc::Rc::new(std::cell::RefCell::new(
            move |state: &FileUpload, el: &Element| {
                on_update(state, el);
                sync_hidden_input_required(el, state, required_intent);
            },
        ));

        wire_click(&root, component.clone(), on_update.clone())?;
        wire_change(&root, component.clone(), on_update.clone())?;
        wire_drag_and_drop(&root, component.clone(), on_update)?;

        // イシュー #1609 codex-review/Bugbot 指摘の是正: 上記の
        // `sync_hidden_input_required` 呼び出しはいずれも状態更新
        // コールバック経由（＝ユーザー操作でイベントが発火した後）にしか
        // 実行されない。SSR hydration や `component.accepted()` が最初から
        // 非空の状態でマウントされた場合、状態変更が一度も起きないまま
        // hidden input には `required` 属性が残り続け、ファイルは受理済み
        // なのにネイティブ constraint validation がフォーム送信を阻止して
        // しまう。配線直後に現在の `component` 状態で一度だけ同期し、
        // 初期 DOM と状態を一致させる。
        if let Ok(state) = component.try_borrow() {
            sync_hidden_input_required(&root, &state, required_intent);
        }

        Ok(())
    }

    /// [`wire_file_upload_component`] 配線時点での hidden-input パーツの
    /// ネイティブ `required` 属性の有無を読み取る（呼び出し側が
    /// `FileUploadProps.required` を渡したかどうかの唯一の観測手段。
    /// 見つからない場合は `false` に倒す＝fail-closed で誤って
    /// required を強制しない）。
    fn hidden_input_required_intent(root: &Element) -> bool {
        let selector =
            format!("[data-scope=\"{FILE_UPLOAD_SCOPE}\"][data-part=\"{HIDDEN_INPUT_PART}\"]");
        root.query_selector(&selector)
            .ok()
            .flatten()
            .is_some_and(|el| el.has_attribute("required"))
    }

    /// 状態更新のたびに hidden-input パーツのネイティブ `required` 属性を
    /// `state.accepted()` の非空判定に同期させる（`required_intent` が
    /// `false`＝呼び出し側がそもそも required を意図していない場合は
    /// 何もしない）。`root` 探索は `el`（`on_update` コールバックへ渡る
    /// マウントルート）配下に限定する。
    fn sync_hidden_input_required(root: &Element, state: &FileUpload, required_intent: bool) {
        if !required_intent {
            return;
        }
        let selector =
            format!("[data-scope=\"{FILE_UPLOAD_SCOPE}\"][data-part=\"{HIDDEN_INPUT_PART}\"]");
        let Ok(Some(el)) = root.query_selector(&selector) else {
            return;
        };
        if state.accepted().is_empty() {
            let _ = set_dom_attribute(&el, "required", "");
        } else {
            let _ = el.remove_attribute("required");
        }
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
                // `dropzone` がいずれも `FileUploadProps.disabled` から反映
                // する `data-disabled`、`crates/headless-ui/src/file_upload.rs`
                // 参照）を明示チェックし、無効化時はドラッグ&ドロップ操作を
                // すべて無視する（PR #868 Cursor Bugbot 指摘: 無効化した
                // dropzone でもドロップでファイルが追加できてしまう不具合の
                // 修正）。Root 自体に `data-disabled` が付与されている場合、
                // または Dropzone パーツ（イベントターゲット自身を含む祖先
                // 方向）に `data-disabled` が付与されている場合のいずれかで
                // 無効化とみなす（`root`/`dropzone` のどちらから disabled が
                // 伝播していても取りこぼさないための fail-closed 判定）。
                //
                // イシュー #1609（参照突合）: `FileUploadProps.readonly` も
                // 同様に `data-readonly` として root/dropzone へ反映される
                // ようになった。zag の `readOnly` は新規ファイルの追加操作を
                // 抑止する（disabled と同じ「追加できない」意味論だが、
                // 既存ファイルの削除ボタン等は disabled にしない）ため、
                // ドラッグ&ドロップによる追加も同じ判定に含める
                // （headless 側の `dropzone` は disabled と readonly を
                // 区別せず同じ `tabindex="-1"`/`aria-disabled` を出す設計と
                // 対応、モジュール doc「参照突合」節参照）。
                if drag_root.has_attribute("data-disabled")
                    || drag_root.has_attribute("data-readonly")
                {
                    return;
                }
                let disabled_dropzone_selector = format!(
                    "[data-scope=\"{FILE_UPLOAD_SCOPE}\"][data-part=\"{DROPZONE_PART}\"][data-disabled], \
                     [data-scope=\"{FILE_UPLOAD_SCOPE}\"][data-part=\"{DROPZONE_PART}\"][data-readonly]"
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
        use fandhe_frontend_headless_ui::file_upload::{item_name, FileUploadProps, ItemType};

        let items = files_from_metadata(
            vec!["<script>alert(1)</script>".to_string()],
            vec![10],
            vec!["text/plain".to_string()],
        );
        let mut f = FileUpload::default();
        f.update(FileUploadAction::AddFiles(items));
        let props = FileUploadProps::default();
        let html = render(&item_name(
            ItemType::Accepted,
            &props,
            vec![],
            vec![text(&f.accepted()[0].name)],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    // --- readonly 時のドロップ無視（イシュー #1609） ---
    //
    // ブラウザ実インタラクション（DragEvent 発火）は
    // `tests/headless_file_upload_browser.rs` の
    // `#[cfg(target_arch = "wasm32")]` テストが担う（disabled 版の先例と
    // 同型）。本モジュールは native 側のロジック（`wire_drag_and_drop` の
    // ガード条件が `data-readonly` を含むこと）をコンパイルレベルで
    // 固定するのみであり、実際の DOM 操作は browser テストが検証する。
}
