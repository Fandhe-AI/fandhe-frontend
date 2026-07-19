//! 三モード（SSR/SSG/CSR）表示整合の実ブラウザ統合テスト
//! （TASK-CSR-loader・#349、親 #337、`wasm-pack test --headless --chrome`）。
//!
//! # 責務境界（重複実装しない領域）
//!
//! `respond("/")` ≡ `page_shell("記事一覧", assemble_list_page(&DemoItemsLoader, &()))`
//! というバイトレベルの等価性は `server/src/ssr.rs` の unit テスト・
//! `server/tests/three_mode_integration.rs`（#348）が既に native で固定
//! 済みである。本ファイルはその契約の連鎖を使い、rws-server を dev-dependency
//! に加えず（server → wasm 方向の依存逆流を避ける、`structure.toml` の
//! `allowed_dependents` 宣言にも反するため）、**SSR 相当の出力を
//! `rws_app::page_shell` + `assemble_list_page`/`assemble_detail_page` の
//! 直接合成で再現**し、CSR 側（[`rws_wasm_full::csr::resolve_list_node`]/
//! [`resolve_detail_node`]）との実 DOM 上の表示整合を検証する
//! （実装計画 §3 設計判断 3）。
//!
//! # 検証内容
//!
//! 1. 一覧・詳細（XSS ペイロード item 含む）の両方で、SSR 相当ボディと
//!    CSR 出力を実 DOM へ展開したときの該当領域（`data-rws="root"` の
//!    `div#app-root`）シリアライズ結果が一致すること。detail ルートでは
//!    XSS ペイロードが実 DOM 上でテキストとして保持され（要素化されない）
//!    ことも合わせて固定する。
//! 2. CSR 固定エラービュー: `Error` に機微情報風文字列を含む loader から
//!    [`rws_wasm_full::csr::resolve_list_node`] を経由して実 DOM へ paint
//!    したのち、固定文言のみが表示され機微文字列が DOM に存在しないこと。
//!
//! フィクスチャの HTML はすべて `rws_core::render` 経由で生成し、`format!`
//! 等による HTML 文字列直接組み立て・`raw_html()` は使用しない
//! （`.claude/rules/coding-rust.md`）。

#![cfg(target_arch = "wasm32")]

use rws_app::{
    assemble_detail_page, assemble_list_page, demo_items, page_shell, DemoItemDetailLoader,
    DemoItemsLoader, Item, Loader,
};
use rws_core::render;
use rws_wasm_full::csr::{resolve_detail_node, resolve_list_node};
use wasm_bindgen_test::*;
use web_sys::{Document, Element};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のプレースホルダ要素を document body へ 1 個生成する。id を
/// 一意にすることで、同一テストバイナリ内の複数テストケース・複数
/// プレースホルダ（SSR 用・CSR 用の 2 つを 1 テストで使う場合を含む）が
/// 要素を奪い合わないようにする（`wasm-full/tests/runtime_browser.rs`
/// `create_placeholder` と同じ意図）。
fn create_placeholder(document: &Document, id: &str) -> Element {
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

/// テスト末尾でプレースホルダを document から確実に除去する RAII ガード
/// （`wasm-full/tests/runtime_browser.rs::RemoveOnDrop` と同じ再発防止策）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `page_shell` が返す完全文書文字列から `<body>` 内側（`div#app-root`
/// 全体）だけを取り出して比較する。
///
/// `page_shell` 出力（`<!DOCTYPE html>` を含む完全文書）を div へ
/// `set_inner_html` するとブラウザが `html`/`head`/`body` を剥がすため、
/// 文書全体ではなく該当領域（`div[data-rws="root"]`）の実 DOM シリアライズ
/// 同士で比較する（双方を同一のパース・シリアライズ経路に通すことで比較の
/// 対称性を確保する、実装計画 §9 のリスク対策）。SSR 相当・CSR いずれも
/// 同じ本ヘルパーを通す。
fn paint_and_extract_app_root(placeholder: &Element, document: &Document, html: &str) -> String {
    placeholder.set_inner_html(html);
    let root = placeholder
        .query_selector("[data-rws='root']")
        .expect("query_selector must not fail")
        .unwrap_or_else(|| {
            // CSR 固定エラービュー（loader_error_view）は data-rws="root" を
            // 持たないため、この場合はプレースホルダ自身の内容を比較対象とする。
            placeholder.clone()
        });
    let _ = document;
    root.outer_html()
}

/// 検証 1（一覧）: SSR 相当ボディ（`page_shell` + `assemble_list_page`）と
/// CSR（`resolve_list_node`）を、それぞれ独立したプレースホルダへ実 DOM
/// 展開し、`div[data-rws="root"]` のシリアライズ結果が一致することを確認する。
#[wasm_bindgen_test]
fn ssr_equivalent_and_csr_render_identical_dom_for_list_page() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let ssr_placeholder = create_placeholder(&document, "three-mode-list-ssr-root");
    let _cleanup_ssr = RemoveOnDrop(ssr_placeholder.clone());
    let csr_placeholder = create_placeholder(&document, "three-mode-list-csr-root");
    let _cleanup_csr = RemoveOnDrop(csr_placeholder.clone());

    // SSR 相当: assemble_list_page(&DemoItemsLoader, &()) の Ok を
    // page_shell へ渡す。respond("/") とのバイトレベル等価性は #348 の
    // native テスト（server 側）が既に固定済みであるため、ここでは
    // rws_app レベルの直接合成のみを行う（server への dev-dependency 追加なし）。
    let ssr_body = assemble_list_page(&DemoItemsLoader, &()).expect("infallible loader");
    let ssr_html = page_shell("記事一覧", ssr_body);
    let ssr_serialized = paint_and_extract_app_root(&ssr_placeholder, &document, &ssr_html);

    let csr_node = resolve_list_node(&DemoItemsLoader);
    let csr_html = render(&csr_node);
    let csr_serialized = paint_and_extract_app_root(&csr_placeholder, &document, &csr_html);

    assert_eq!(
        ssr_serialized, csr_serialized,
        "SSR 相当出力と CSR 出力の実 DOM シリアライズが一致すること（三モード整合）"
    );
}

/// 検証 1（詳細、XSS ペイロード item）: `demo_items()[1]`（id="2"）の
/// XSS ペイロードを含む詳細ページで、SSR 相当・CSR の実 DOM シリアライズが
/// 一致し、かつペイロードが実 DOM 上でテキストとして保持される（`script`
/// 要素として生成されない）ことを確認する。
#[wasm_bindgen_test]
fn ssr_equivalent_and_csr_render_identical_dom_for_detail_page_with_xss_payload() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let ssr_placeholder = create_placeholder(&document, "three-mode-detail-ssr-root");
    let _cleanup_ssr = RemoveOnDrop(ssr_placeholder.clone());
    let csr_placeholder = create_placeholder(&document, "three-mode-detail-csr-root");
    let _cleanup_csr = RemoveOnDrop(csr_placeholder.clone());

    let xss_item_id = demo_items()
        .into_iter()
        .find(|it| it.title.contains("<script>"))
        .map(|it| it.id)
        .expect("demo_items() must contain the XSS payload fixture item");

    let ssr_body =
        assemble_detail_page(&DemoItemDetailLoader, &xss_item_id).expect("infallible loader");
    let ssr_html = page_shell("記事詳細", ssr_body);
    let ssr_serialized = paint_and_extract_app_root(&ssr_placeholder, &document, &ssr_html);

    let csr_node = resolve_detail_node(&DemoItemDetailLoader, &xss_item_id);
    let csr_html = render(&csr_node);
    let csr_serialized = paint_and_extract_app_root(&csr_placeholder, &document, &csr_html);

    assert_eq!(
        ssr_serialized, csr_serialized,
        "XSS ペイロード item の詳細ページでも SSR 相当出力と CSR 出力の \
         実 DOM シリアライズが一致すること"
    );

    // 実 DOM 上でペイロードが要素化されていないこと（REQ-1、既定エスケープ
    // が実 DOM 反映後も保持されていることの固定）。
    assert!(
        csr_placeholder.query_selector("script").unwrap().is_none(),
        "XSS ペイロードが実 DOM 上で <script> 要素として生成されてはならない"
    );
    assert!(
        csr_placeholder
            .inner_html()
            .contains("&lt;script&gt;alert('xss')&lt;/script&gt;"),
        "XSS ペイロードはエスケープ済みテキストとして DOM に保持されること: {}",
        csr_placeholder.inner_html()
    );
}

/// 検証 2: CSR 固定エラービュー。`Error` に機微情報風文字列を含む loader
/// から [`resolve_list_node`] を経由して実 DOM へ paint したのち、固定文言
/// のみが表示され機微文字列が DOM に一切存在しないこと
/// （`server/src/ssr.rs::loader_error_response` と同型の fail-closed 契約を
/// 実 DOM まで固定する）。
#[wasm_bindgen_test]
fn csr_resolve_list_node_paints_fixed_error_view_without_leaking_error_value_in_real_dom() {
    struct FailingLoader;

    impl Loader for FailingLoader {
        type Input = ();
        type Output = Vec<Item>;
        type Error = String;

        fn load(&self, _input: &()) -> Result<Vec<Item>, String> {
            Err("secret://db-password@internal-host".to_string())
        }
    }

    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "three-mode-csr-error-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let node = resolve_list_node(&FailingLoader);
    let html = render(&node);
    placeholder.set_inner_html(&html);

    let dom_text = placeholder.inner_html();
    assert!(
        !dom_text.contains("secret://db-password@internal-host"),
        "loader の Error 値（機微情報風文字列）が実 DOM へ混入してはならない: {dom_text}"
    );
    assert!(
        dom_text.contains("Something went wrong"),
        "実 DOM に固定文言のエラービューが反映されていること: {dom_text}"
    );
}
