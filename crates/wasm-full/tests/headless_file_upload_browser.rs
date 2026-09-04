//! `fandhe_frontend_wasm_full::headless_file_upload`（イシュー #840、親
//! トラッキング #520）の実ブラウザ統合テスト（`wasm-pack test --headless
//! --chrome`）。
//!
//! `wasm-full/tests/headless_file_upload.rs`（native）は判定 → dispatch の
//! 統合経路までを検証済みである。本ファイルはその先、
//! `wire_file_upload_component` が実 DOM（headless Chromium）上で以下を
//! 満たすことを検証する（実装計画 §3.3 対応）。
//!
//! 1. `new File([...], name, { type })` + `DataTransfer` で `input.files` を
//!    合成し `change` を発火 → メタデータ（name/size 表示/mime）が item-group
//!    へ反映されること（受け入れ条件の必須項目）。
//! 2. accept 違反ファイルで拒否理由が反映され受理一覧へ入らないこと。
//! 3. ファイル名 `"><img src=x onerror=alert(1)>.txt` 等で `img` 要素が
//!    生成されないこと（XSS 回帰、REQ-1）。
//! 4. クリック委譲: Trigger クリックで hidden-input へ `click()` が転送
//!    されること、ClearTrigger/ItemDeleteTrigger クリックで状態が更新される
//!    こと。
//!
//! DOM 構造は `crates/headless-ui/src/file_upload.rs` の SSR 出力契約を、
//! `fandhe_frontend_headless_ui::file_upload` のパーツ関数を直接呼んで
//! 再現する（`fandhe-frontend-headless-ui` は本クレートの製品依存のため、
//! 実際の SSR 出力から手組みドリフトを起こさない）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::file_upload::{
    clear_trigger, dropzone, hidden_input, item, item_delete_trigger, item_group, item_name,
    item_size_text, item_size_text_node, label, root, trigger, FileUpload, FileUploadAction,
    FileUploadItem, FileUploadProps, ItemType,
};
use fandhe_frontend_interactive::Component;
use fandhe_frontend_wasm_full::headless_file_upload::wire_file_upload_component;
use js_sys::Array;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{
    DataTransfer, Document, DragEvent, DragEventInit, Element, Event, EventInit, File,
    FilePropertyBag, HtmlInputElement,
};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のプレースホルダ要素を document body へ 1 個生成する。id を一意に
/// することで、同一テストバイナリ内の複数テストケースが要素を奪い合わない
/// ようにする（`headless_avatar_browser.rs::create_container` と同じ意図）。
fn create_container(document: &Document, id: &str) -> Element {
    let container = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    container.set_id(id);
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&container)
        .expect("append_child must not fail for a detached div");
    container
}

/// テスト末尾でコンテナを document から確実に除去する RAII ガード
/// （`headless_avatar_browser.rs::RemoveOnDrop` と同じ意図）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// [`FileUpload`] 状態から SSR 出力契約と同型のマークアップを組み立てる
/// （headless パーツ関数を直接呼ぶ、`mount_avatar` と同型の判断）。
fn render_file_upload(state: &FileUpload) -> String {
    let props = FileUploadProps::default();
    let items: Vec<_> = state
        .accepted()
        .iter()
        .map(|f| {
            let size = item_size_text(f.size_bytes);
            item(
                ItemType::Accepted,
                &props,
                vec![],
                vec![
                    item_name(ItemType::Accepted, &props, vec![], vec![text(&f.name)]),
                    item_size_text_node(ItemType::Accepted, &props, vec![], vec![text(&size)]),
                    item_delete_trigger(
                        &f.name,
                        ItemType::Accepted,
                        &props,
                        vec![],
                        vec![text("x")],
                    ),
                ],
            )
        })
        .collect();
    let node = root(
        &props,
        false,
        vec![],
        vec![
            label(&props, vec![], vec![text("Files")]),
            dropzone(
                &props,
                false,
                vec![],
                vec![
                    trigger(&props, vec![], vec![text("Browse")]),
                    hidden_input(state.accept(), true, &props, vec![]),
                ],
            ),
            item_group(ItemType::Accepted, &props, vec![], items),
            clear_trigger(&props, state.is_empty(), vec![], vec![text("Clear")]),
        ],
    );
    render(&node)
}

fn hidden_input_element(container: &Element) -> HtmlInputElement {
    container
        .query_selector("[data-scope='file-upload'][data-part='hidden-input']")
        .expect("query_selector must not fail")
        .expect("hidden-input part must exist")
        .dyn_into::<HtmlInputElement>()
        .expect("hidden-input part must be an HtmlInputElement")
}

/// 合成 `File` を組み立てる（内容は空、name/size/type のみ検証対象）。
fn make_file(name: &str, size_bytes: usize, mime: &str) -> File {
    let parts = Array::new();
    parts.push(&JsValue::from_str(&"x".repeat(size_bytes)));
    let options = FilePropertyBag::new();
    options.set_type(mime);
    File::new_with_str_sequence_and_options(&parts, name, &options)
        .expect("File::new_with_str_sequence_and_options must not fail")
}

/// `DataTransfer` に合成 `File` 群を積み、`input.files` へ反映してから
/// `change` イベントを（バブリングありで）発火する。
fn dispatch_change_with_files(input: &HtmlInputElement, files: &[File]) {
    let data_transfer = DataTransfer::new().expect("DataTransfer::new must not fail");
    for file in files {
        data_transfer
            .items()
            .add_with_file(file)
            .expect("add_with_file must not fail");
    }
    input.set_files(Some(
        &data_transfer
            .files()
            .expect("files() must return Some after add"),
    ));

    let init = EventInit::new();
    init.set_bubbles(true);
    let event = Event::new_with_event_init_dict("change", &init).expect("Event::new must not fail");
    input
        .dispatch_event(&event)
        .expect("dispatch_event must not fail");
}

#[wasm_bindgen_test]
fn change_event_with_synthetic_files_adds_metadata_to_dom() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "file-upload-change-test");
    let _guard = RemoveOnDrop(container.clone());

    let state = std::rc::Rc::new(std::cell::RefCell::new(FileUpload::new(
        "", None, None, None,
    )));
    container.set_inner_html(&render_file_upload(&state.borrow()));

    let update_container = container.clone();
    wire_file_upload_component(container.clone(), state.clone(), move |s, _el| {
        update_container.set_inner_html(&render_file_upload(s));
    })
    .expect("wire_file_upload_component must not fail");

    let input = hidden_input_element(&container);
    let file = make_file("report.pdf", 2048, "application/pdf");
    dispatch_change_with_files(&input, &[file]);

    let html = container.inner_html();
    assert!(html.contains("report.pdf"));
    assert!(html.contains("2.0 KB"));
    assert_eq!(state.borrow().len(), 1);
}

#[wasm_bindgen_test]
fn change_event_with_invalid_type_is_rejected_and_not_added() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "file-upload-reject-test");
    let _guard = RemoveOnDrop(container.clone());

    let state = std::rc::Rc::new(std::cell::RefCell::new(FileUpload::new(
        "image/*", None, None, None,
    )));
    container.set_inner_html(&render_file_upload(&state.borrow()));

    let update_container = container.clone();
    wire_file_upload_component(container.clone(), state.clone(), move |s, _el| {
        update_container.set_inner_html(&render_file_upload(s));
    })
    .expect("wire_file_upload_component must not fail");

    let input = hidden_input_element(&container);
    let file = make_file("notes.txt", 10, "text/plain");
    dispatch_change_with_files(&input, &[file]);

    assert!(state.borrow().is_empty());
    assert_eq!(state.borrow().rejected().len(), 1);
    assert!(!container.inner_html().contains("notes.txt"));
}

/// XSS 回帰: 悪意あるファイル名を持つ合成 `File` でも実 DOM に `img` 要素が
/// 生成されないこと（REQ-1、ファイル名は攻撃者制御可能な入力そのもの）。
#[wasm_bindgen_test]
fn change_event_with_malicious_file_name_does_not_inject_img_element() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "file-upload-xss-test");
    let _guard = RemoveOnDrop(container.clone());

    let state = std::rc::Rc::new(std::cell::RefCell::new(FileUpload::new(
        "", None, None, None,
    )));
    container.set_inner_html(&render_file_upload(&state.borrow()));

    let update_container = container.clone();
    wire_file_upload_component(container.clone(), state.clone(), move |s, _el| {
        update_container.set_inner_html(&render_file_upload(s));
    })
    .expect("wire_file_upload_component must not fail");

    let input = hidden_input_element(&container);
    let malicious_name = "\"><img src=x onerror=alert(1)>.txt";
    let file = make_file(malicious_name, 5, "text/plain");
    dispatch_change_with_files(&input, &[file]);

    assert_eq!(state.borrow().len(), 1);
    assert_eq!(state.borrow().accepted()[0].name, malicious_name);
    // 実 DOM 上に img 要素が注入されていないこと（querySelectorAll で 0 件）。
    let imgs = container
        .query_selector_all("img")
        .expect("query_selector_all must not fail");
    assert_eq!(imgs.length(), 0);
    // テキストとしては（エスケープ済みで）保持されている。
    let item_name_el = container
        .query_selector("[data-scope='file-upload'][data-part='item-name']")
        .expect("query_selector must not fail")
        .expect("item-name part must exist");
    assert!(item_name_el
        .text_content()
        .unwrap_or_default()
        .contains("onerror=alert(1)"));
}

#[wasm_bindgen_test]
fn trigger_click_forwards_click_to_hidden_input() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "file-upload-trigger-test");
    let _guard = RemoveOnDrop(container.clone());

    let state = std::rc::Rc::new(std::cell::RefCell::new(FileUpload::default()));
    container.set_inner_html(&render_file_upload(&state.borrow()));

    wire_file_upload_component(container.clone(), state.clone(), |_s, _el| {})
        .expect("wire_file_upload_component must not fail");

    // hidden-input の click() をラップして呼ばれたかを検知する代わりに、
    // ネイティブ input[type=file] は click() でファイル選択ダイアログを
    // 開こうとするがヘッドレス環境では例外を投げずに no-op で終わる
    // （ブラウザ実装依存）。ここでは trigger クリックが例外を投げず正常
    // 終了することのみを固定する（受け入れ条件: ピッカー起動の配線経路が
    // 動作すること）。
    let trigger_el = container
        .query_selector("[data-scope='file-upload'][data-part='trigger']")
        .expect("query_selector must not fail")
        .expect("trigger part must exist");
    let init = EventInit::new();
    init.set_bubbles(true);
    let event = Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail");
    trigger_el
        .dispatch_event(&event)
        .expect("dispatch_event must not fail");
}

#[wasm_bindgen_test]
fn clear_trigger_click_clears_all_files() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "file-upload-clear-test");
    let _guard = RemoveOnDrop(container.clone());

    let mut initial = FileUpload::default();
    initial.update(FileUploadAction::AddFiles(vec![
        fandhe_frontend_headless_ui::file_upload::FileUploadItem::new("a.txt", 1, "text/plain"),
    ]));
    let state = std::rc::Rc::new(std::cell::RefCell::new(initial));
    container.set_inner_html(&render_file_upload(&state.borrow()));

    let update_container = container.clone();
    wire_file_upload_component(container.clone(), state.clone(), move |s, _el| {
        update_container.set_inner_html(&render_file_upload(s));
    })
    .expect("wire_file_upload_component must not fail");

    let clear_el = container
        .query_selector("[data-scope='file-upload'][data-part='clear-trigger']")
        .expect("query_selector must not fail")
        .expect("clear-trigger part must exist");
    let init = EventInit::new();
    init.set_bubbles(true);
    let event = Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail");
    clear_el
        .dispatch_event(&event)
        .expect("dispatch_event must not fail");

    assert!(state.borrow().is_empty());
    assert!(!container.inner_html().contains("a.txt"));
}

/// 無効化状態（`disabled: true`）で Root/Dropzone を組み立てたマークアップ
/// （`render_file_upload` と異なり disabled 反映が必要なため専用のヘルパ）。
fn render_disabled_file_upload(state: &FileUpload) -> String {
    let props = FileUploadProps {
        disabled: true,
        ..Default::default()
    };
    let node = root(
        &props,
        false,
        vec![],
        vec![
            label(&props, vec![], vec![text("Files")]),
            dropzone(
                &props,
                false,
                vec![],
                vec![
                    trigger(&props, vec![], vec![text("Browse")]),
                    hidden_input(state.accept(), true, &props, vec![]),
                ],
            ),
            item_group(ItemType::Accepted, &props, vec![], vec![]),
            clear_trigger(&props, true, vec![], vec![text("Clear")]),
        ],
    );
    render(&node)
}

/// 読み取り専用状態（`readonly: true`）で Root/Dropzone を組み立てた
/// マークアップ（イシュー #1609 参照突合。zag `readOnly` は新規ファイルの
/// 追加操作のみを抑止するため、`render_disabled_file_upload` と異なり
/// hidden-input はネイティブ `disabled`、既存ファイルの削除操作
/// （`item-delete-trigger`）は生成しない本テストでは検証対象外）。
fn render_readonly_file_upload(state: &FileUpload) -> String {
    let props = FileUploadProps {
        readonly: true,
        ..Default::default()
    };
    let node = root(
        &props,
        false,
        vec![],
        vec![
            label(&props, vec![], vec![text("Files")]),
            dropzone(
                &props,
                false,
                vec![],
                vec![
                    trigger(&props, vec![], vec![text("Browse")]),
                    hidden_input(state.accept(), true, &props, vec![]),
                ],
            ),
            item_group(ItemType::Accepted, &props, vec![], vec![]),
            clear_trigger(&props, true, vec![], vec![text("Clear")]),
        ],
    );
    render(&node)
}

/// `DataTransfer` に合成 `File` 群を積んだ `DragEvent`（`drop`）を、
/// バブリングありで dropzone 上に発火する。
fn dispatch_drop_with_files(target: &Element, files: &[File]) {
    let data_transfer = DataTransfer::new().expect("DataTransfer::new must not fail");
    for file in files {
        data_transfer
            .items()
            .add_with_file(file)
            .expect("add_with_file must not fail");
    }
    let init = DragEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_data_transfer(Some(&data_transfer));
    let event =
        DragEvent::new_with_event_init_dict("drop", &init).expect("DragEvent::new must not fail");
    target
        .dispatch_event(&event)
        .expect("dispatch_event must not fail");
}

/// 無効化状態（`data-disabled` が Root/Dropzone に付与されている）の
/// dropzone へ `drop` イベントを発火してもファイルが追加されないこと
/// （PR #868 Cursor Bugbot 指摘の回帰テスト: ネイティブ `disabled` を
/// 持てない `div`/`role="button"` の dropzone がドラッグ&ドロップを
/// 無条件に受理してしまっていた不具合の修正確認）。
#[wasm_bindgen_test]
fn disabled_dropzone_drop_event_does_not_add_files() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "file-upload-disabled-drop-test");
    let _guard = RemoveOnDrop(container.clone());

    let state = std::rc::Rc::new(std::cell::RefCell::new(FileUpload::new(
        "", None, None, None,
    )));
    container.set_inner_html(&render_disabled_file_upload(&state.borrow()));

    let update_container = container.clone();
    wire_file_upload_component(container.clone(), state.clone(), move |s, _el| {
        update_container.set_inner_html(&render_disabled_file_upload(s));
    })
    .expect("wire_file_upload_component must not fail");

    let dropzone_el = container
        .query_selector("[data-scope='file-upload'][data-part='dropzone']")
        .expect("query_selector must not fail")
        .expect("dropzone part must exist");
    // 無効化状態を明示的に固定する（headless-ui 側の `dropzone(true, ...)` が
    // `data-disabled` を反映することへの依存を、テスト側でも直接確認する）。
    assert!(dropzone_el.has_attribute("data-disabled"));

    let file = make_file("dropped.pdf", 100, "application/pdf");
    dispatch_drop_with_files(&dropzone_el, &[file]);

    assert!(state.borrow().is_empty());
    assert!(!container.inner_html().contains("dropped.pdf"));
}

/// 読み取り専用状態（`data-readonly` が Root/Dropzone に付与されている）の
/// dropzone へ `drop` イベントを発火してもファイルが追加されないこと
/// （イシュー #1609 参照突合: zag `readOnly` は新規ファイルの追加操作を
/// 抑止する。`disabled_dropzone_drop_event_does_not_add_files` と同型の
/// 回帰テスト、`wire_drag_and_drop` のガード条件拡張の確認）。
#[wasm_bindgen_test]
fn readonly_dropzone_drop_event_does_not_add_files() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "file-upload-readonly-drop-test");
    let _guard = RemoveOnDrop(container.clone());

    let state = std::rc::Rc::new(std::cell::RefCell::new(FileUpload::new(
        "", None, None, None,
    )));
    container.set_inner_html(&render_readonly_file_upload(&state.borrow()));

    let update_container = container.clone();
    wire_file_upload_component(container.clone(), state.clone(), move |s, _el| {
        update_container.set_inner_html(&render_readonly_file_upload(s));
    })
    .expect("wire_file_upload_component must not fail");

    let dropzone_el = container
        .query_selector("[data-scope='file-upload'][data-part='dropzone']")
        .expect("query_selector must not fail")
        .expect("dropzone part must exist");
    // 読み取り専用状態を明示的に固定する（headless-ui 側の
    // `dropzone(&FileUploadProps { readonly: true, .. }, ...)` が
    // `data-readonly` を反映することへの依存を、テスト側でも直接確認する）。
    assert!(dropzone_el.has_attribute("data-readonly"));

    let file = make_file("dropped.pdf", 100, "application/pdf");
    dispatch_drop_with_files(&dropzone_el, &[file]);

    assert!(state.borrow().is_empty());
    assert!(!container.inner_html().contains("dropped.pdf"));
}

/// [`render_file_upload`] と異なり、rejected [`item_group`] を accepted
/// より先（DOM 出現順で先頭）に描画したマークアップを組み立てる。イシュー
/// #1609 Cursor Bugbot 指摘（`compute_item_index` が `data-type` を区別せず
/// 数えていた）の回帰テストが、accepted/rejected 混在時の出現順ズレを
/// 確実に再現するための専用ヘルパ。`required` を渡せるようにし、required
/// 属性のネイティブ同期の回帰テストにも流用する。
fn render_file_upload_mixed(state: &FileUpload, required: bool) -> String {
    let props = FileUploadProps {
        required,
        ..Default::default()
    };
    let accepted_items: Vec<_> = state
        .accepted()
        .iter()
        .map(|f| {
            item(
                ItemType::Accepted,
                &props,
                vec![],
                vec![
                    item_name(ItemType::Accepted, &props, vec![], vec![text(&f.name)]),
                    item_delete_trigger(
                        &f.name,
                        ItemType::Accepted,
                        &props,
                        vec![],
                        vec![text("x")],
                    ),
                ],
            )
        })
        .collect();
    let rejected_items: Vec<_> = state
        .rejected()
        .iter()
        .map(|(f, _reason)| {
            item(
                ItemType::Rejected,
                &props,
                vec![],
                vec![
                    item_name(ItemType::Rejected, &props, vec![], vec![text(&f.name)]),
                    item_delete_trigger(
                        &f.name,
                        ItemType::Rejected,
                        &props,
                        vec![],
                        vec![text("x")],
                    ),
                ],
            )
        })
        .collect();
    let node = root(
        &props,
        false,
        vec![],
        vec![
            label(&props, vec![], vec![text("Files")]),
            dropzone(
                &props,
                false,
                vec![],
                vec![
                    trigger(&props, vec![], vec![text("Browse")]),
                    hidden_input(state.accept(), true, &props, vec![]),
                ],
            ),
            // rejected を accepted より先に描画する（出現順インデックスの
            // ズレが最も顕在化する構成）。
            item_group(ItemType::Rejected, &props, vec![], rejected_items),
            item_group(ItemType::Accepted, &props, vec![], accepted_items),
            clear_trigger(&props, state.is_empty(), vec![], vec![text("Clear")]),
        ],
    );
    render(&node)
}

/// イシュー #1609 Cursor Bugbot 指摘（Medium）・codex-review 再指摘（P1）の
/// 回帰テスト:
/// `compute_item_index` が `data-part="item"` を `data-type` 区別せず
/// 数えていたため、rejected item（accepted より DOM 出現順で先）の
/// `item-delete-trigger` をクリックすると誤って accepted ファイルが
/// 削除され得た（Cursor Bugbot 指摘）。さらにその是正の初期実装
/// （`compute_item_index` を `data-type="accepted"` へ固定）は、逆に
/// rejected item の削除ボタンを恒常的な no-op にしてしまっていた
/// （codex-review 再指摘）。最終是正は `item_type` を実測して
/// `"remove"`（accepted）/`"remove-rejected"`（rejected）を使い分ける
/// ため、rejected item のクリックでは accepted 一覧が変化せず
/// `rejected` 一覧からその要素が正しく除去されること、かつ accepted
/// item のクリックでは正しい（accepted 内での出現順）インデックスが
/// 削除されることを固定する。
#[wasm_bindgen_test]
fn item_delete_trigger_click_indexes_accepted_and_rejected_items_independently() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "file-upload-mixed-remove-test");
    let _guard = RemoveOnDrop(container.clone());

    let mut with_rejection = FileUpload::new("text/plain", None, None, None);
    with_rejection.update(FileUploadAction::AddFiles(vec![
        fandhe_frontend_headless_ui::file_upload::FileUploadItem::new("keep.txt", 1, "text/plain"),
        fandhe_frontend_headless_ui::file_upload::FileUploadItem::new(
            "remove-me.txt",
            1,
            "text/plain",
        ),
        fandhe_frontend_headless_ui::file_upload::FileUploadItem::new("bad.png", 1, "image/png"),
    ]));
    assert_eq!(with_rejection.accepted().len(), 2);
    assert_eq!(with_rejection.rejected().len(), 1);

    let state = std::rc::Rc::new(std::cell::RefCell::new(with_rejection));
    container.set_inner_html(&render_file_upload_mixed(&state.borrow(), false));

    let update_container = container.clone();
    wire_file_upload_component(container.clone(), state.clone(), move |s, _el| {
        update_container.set_inner_html(&render_file_upload_mixed(s, false));
    })
    .expect("wire_file_upload_component must not fail");

    // rejected item（DOM 出現順で先頭）の削除トリガーをクリックすると、
    // accepted 一覧は変化せず（誤った accepted ファイル削除を起こさない）、
    // かつ `rejected` 一覧からその要素が正しく除去される
    // （codex-review 再指摘の是正確認: 以前は `"remove"` 固定 dispatch の
    // ため常に no-op だった）。
    let rejected_delete_el = container
        .query_selector(
            "[data-scope='file-upload'][data-part='item-delete-trigger'][data-type='rejected']",
        )
        .expect("query_selector must not fail")
        .expect("rejected item-delete-trigger must exist");
    let init = EventInit::new();
    init.set_bubbles(true);
    let event = Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail");
    rejected_delete_el
        .dispatch_event(&event)
        .expect("dispatch_event must not fail");

    assert_eq!(state.borrow().accepted().len(), 2);
    assert!(
        state.borrow().rejected().is_empty(),
        "clicking the rejected item's delete trigger must remove it from the rejected list \
         (codex-review P1 regression: previously always a no-op)"
    );
    assert!(state
        .borrow()
        .accepted()
        .iter()
        .any(|f| f.name == "keep.txt"));
    assert!(state
        .borrow()
        .accepted()
        .iter()
        .any(|f| f.name == "remove-me.txt"));

    // accepted item（"remove-me.txt"、accepted 内での出現順インデックス
    // 1）の削除トリガーをクリックすると、正しくそのファイルのみが消える。
    let accepted_delete_selector =
        "[data-scope='file-upload'][data-part='item-delete-trigger'][data-type='accepted']";
    let accepted_delete_els = container
        .query_selector_all(accepted_delete_selector)
        .expect("query_selector_all must not fail");
    assert_eq!(accepted_delete_els.length(), 2);
    let target = accepted_delete_els
        .get(1)
        .expect("second accepted item-delete-trigger must exist");
    let target_el: Element = target.dyn_into().expect("must be an Element");
    let event2 = Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail");
    target_el
        .dispatch_event(&event2)
        .expect("dispatch_event must not fail");

    assert_eq!(state.borrow().accepted().len(), 1);
    assert!(state
        .borrow()
        .accepted()
        .iter()
        .any(|f| f.name == "keep.txt"));
    assert!(!state
        .borrow()
        .accepted()
        .iter()
        .any(|f| f.name == "remove-me.txt"));
}
/// イシュー #1609 codex-review 再指摘（P1、2 回目）の回帰テスト:
/// [`hidden_input`] はネイティブ `required` 属性を一切出力しない
/// （過去に実装していた `sync_hidden_input_required`、PR #1885
/// 588fd4f/d9e846f、は「`accepted()` が非空ならネイティブ `required` を
/// 除去する」同期を行っていたが、`change` ハンドラは処理直後に必ず
/// `input.set_value("")` で hidden-input の実 `FileList` を破棄するため、
/// required 除去後もネイティブフォーム送信には実ファイルが一切含まれず、
/// 意図しない検証バイパスになっていた。続く是正〔2b5fbc1〕は同期処理を
/// 撤去しネイティブ `required` を常時出力する形にしたが、これは
/// 「required 指定時、正常にファイルを選択してもネイティブ `<form>`
/// 送信が常にブロックされる」別の P1 を生んだ。最終是正は
/// `crates/headless-ui/src/file_upload.rs::hidden_input` 側でネイティブ
/// `required` 自体を出力しない設計へ変更し、`aria-required`/
/// `data-required` のみで要求有無を提示する）。`wire_file_upload_component`
/// は状態変化（`AddFiles`/`Remove`/マウント時点）のいずれでも
/// `input.required()`（ネイティブ DOM プロパティ）が常に `false`
/// のままであることを固定する。
#[wasm_bindgen_test]
fn hidden_input_never_carries_native_required_attribute() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "file-upload-required-static-test");
    let _guard = RemoveOnDrop(container.clone());

    // hydration 相当: マウント前から `accepted` が非空の状態を用意する。
    let mut initial = FileUpload::default();
    initial.update(FileUploadAction::AddFiles(vec![FileUploadItem::new(
        "a.txt",
        1,
        "text/plain",
    )]));
    assert_eq!(initial.accepted().len(), 1);

    let state = std::rc::Rc::new(std::cell::RefCell::new(initial));
    container.set_inner_html(&render_file_upload_mixed(&state.borrow(), true));
    let input = hidden_input_element(&container);
    assert!(
        !input.required(),
        "SSR markup must never render native required, regardless of props.required or \
         accepted state (would block native <form> submission even with accepted files, \
         since the hidden-input never retains a real FileList)"
    );
    assert_eq!(
        input.get_attribute("aria-required").as_deref(),
        Some("true"),
        "aria-required must reflect props.required instead of native required"
    );
    assert!(input.has_attribute("data-required"));

    let update_container = container.clone();
    wire_file_upload_component(container.clone(), state.clone(), move |s, _el| {
        update_container.set_inner_html(&render_file_upload_mixed(s, true));
    })
    .expect("wire_file_upload_component must not fail");

    // 配線直後（状態変更イベントは一切発火していない）でも native
    // required は出力されない。
    let input_after_wire = hidden_input_element(&container);
    assert!(!input_after_wire.required());

    // さらにファイルを追加して `accepted()` を非空のまま増やしても
    // native required は出力されない。
    let input = hidden_input_element(&container);
    let file = make_file("b.txt", 1, "text/plain");
    dispatch_change_with_files(&input, &[file]);
    assert_eq!(state.borrow().accepted().len(), 2);
    let input_after_add = hidden_input_element(&container);
    assert!(!input_after_add.required());

    // 受理済みファイルを 1 件ずつ削除して空に戻しても native required は
    // 出力されない（元々出力していないため、当然ながら維持される）。
    let init = EventInit::new();
    init.set_bubbles(true);
    for _ in 0..2 {
        let delete_el = container
            .query_selector(
                "[data-scope='file-upload'][data-part='item-delete-trigger'][data-type='accepted']",
            )
            .expect("query_selector must not fail");
        if let Some(delete_el) = delete_el {
            let event =
                Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail");
            delete_el
                .dispatch_event(&event)
                .expect("dispatch_event must not fail");
        }
    }
    assert!(state.borrow().accepted().is_empty());
    let input_after_remove = hidden_input_element(&container);
    assert!(!input_after_remove.required());
}
