//! `fandhe_frontend_animation::flip` を実ブラウザ上で駆動し、Invert 直後の
//! 補正 transform・収束後の transform 解除を確認する統合テスト
//! （イシュー #2518）。
//!
//! `crates/frontend-animation/src/flip.rs` の native テストは `invert`/
//! `to_transform_css`/`Interpolate` の純粋計算のみを検証済みである。本
//! テストはその先、実 DOM の `getBoundingClientRect()`（[`measure`]）と
//! 実ブラウザの `requestAnimationFrame`（[`play`]）を使っても FLIP の
//! 契約（Invert 直後は補正値を持ち、`settle_duration()` 経過後は恒等状態
//! へ戻る）が成り立つことを固定する。`spring_via_raf_dom_browser.rs` と
//! 同型の待機戦略（実時間で `settle_duration` の倍以上待つ）を用いる。

#![cfg(target_arch = "wasm32")]

use fandhe_animation::spring::{Spring, SpringConfig};
use fandhe_frontend_animation::flip::{self, Rect};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// `spring_via_raf_dom_browser.rs::sleep_ms` と同じ意図: 実タイマーを
/// `Promise` 化して `await` する決定的な待機。
async fn sleep_ms(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window must exist");
        let callback = Closure::once_into_js(move || {
            let _ = resolve.call0(&JsValue::NULL);
        });
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(callback.unchecked_ref(), ms)
            .expect("setTimeout must not fail in test environment");
    });
    JsFuture::from(promise)
        .await
        .expect("setTimeout promise must not reject");
}

#[wasm_bindgen_test]
async fn play_applies_invert_then_settles_to_identity_transform() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    // 位置・サイズを固定するため absolute + 明示 px（レイアウト依存を避ける）。
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    // 変化前（first）の矩形を実測してから、レイアウト変化後の位置
    // （last）へ実際に移動させる。Invert は first/last の差分から
    // 補正値を計算するため、Rect を手で構成せず measure() を 2 回
    // 呼んで実測する（`layout_flip.rs`（wasm-full 側配線）が行う
    // Before/After 計測と同型の手順）。
    let first: Rect = flip::measure(&div);

    div.style()
        .set_property("left", "100px")
        .expect("set_property must not fail");
    div.style()
        .set_property("top", "40px")
        .expect("set_property must not fail");
    let last: Rect = flip::measure(&div);

    let delta = fandhe_frontend_animation::flip::invert(first, last)
        .expect("同一要素の連続計測は常に有効な Rect のため Some");

    let spring_config = SpringConfig::default();
    let settle_ms = Spring::new(spring_config, 0.0, 1.0, 0.0)
        .expect("既定 SpringConfig は有効な spring パラメータのため None にならない")
        .settle_duration()
        * 1000.0;

    let original = flip::OriginalStyle::capture(&div);
    let anim_loop = flip::play(div.clone(), delta, spring_config, original);

    // Invert 直後（Play 開始直後）は補正 transform が書き込まれている
    // （`play()` が同期的に `transform`/`transform-origin` を設定する
    // ため、次の rAF を待たずに確認できる）。
    let transform_after_invert = div
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert!(
        !transform_after_invert.is_empty(),
        "Invert 直後は transform が設定されているはず"
    );

    // settle_duration の倍 + マージンで実時間待機し、収束後は transform
    // が解除されていることを確認する（`spring_via_raf_dom_browser.rs` と
    // 同じ待機戦略）。
    sleep_ms((settle_ms * 2.0).ceil() as i32 + 200).await;
    anim_loop.stop();

    let transform_after_settle = div
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert!(
        transform_after_settle.is_empty(),
        "収束後は transform が解除されているはず: {transform_after_settle}"
    );

    div.remove();
}

#[wasm_bindgen_test]
fn measure_reads_actual_bounding_client_rect() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("left", "10px")
        .expect("set_property must not fail");
    div.style()
        .set_property("top", "20px")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "30px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "40px")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    let rect = flip::measure(&div);
    assert!((rect.x - 10.0).abs() < 1.0, "x={}", rect.x);
    assert!((rect.y - 20.0).abs() < 1.0, "y={}", rect.y);
    assert!((rect.width - 30.0).abs() < 1.0, "width={}", rect.width);
    assert!((rect.height - 40.0).abs() < 1.0, "height={}", rect.height);

    div.remove();
}

/// 受け入れ条件（レビュー指摘対応、イシュー #2518 フォローアップ）: 要素が
/// 元々持っていた inline `transform` は、FLIP 補正の適用中は合成され
/// （消えず）、収束後は FLIP 補正のみが解除されて元の値へ正確に復元
/// される（空文字列へは解除されない）。
#[wasm_bindgen_test]
async fn play_preserves_and_restores_pre_existing_transform() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    // 利用者が FLIP とは無関係に指定していた transform（レビュー指摘対応の
    // 検証対象）。
    div.style()
        .set_property("transform", "rotate(10deg)")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    let first: Rect = flip::measure(&div);
    div.style()
        .set_property("left", "100px")
        .expect("set_property must not fail");
    div.style()
        .set_property("top", "40px")
        .expect("set_property must not fail");
    let last: Rect = flip::measure(&div);
    let delta = fandhe_frontend_animation::flip::invert(first, last)
        .expect("同一要素の連続計測は常に有効な Rect のため Some");

    // `play_after`（wasm-full 側配線）が行う「クリアしてから元の値を捕捉」
    // ではなく、ここでは `play()` に直接渡す前の生の値を捕捉する（本
    // テストは `flip::play` 単体の契約を検証する）。
    let original = flip::OriginalStyle::capture(&div);

    // レビュー指摘対応（イシュー #2518 さらなるフォローアップ）:
    // `OriginalStyle::compose_transform` は CSS クラス由来の `transform`
    // も合成できるよう `getComputedStyle` の解決済み `matrix(...)` を
    // 合成元に使う（`resolve_computed_transform` doc 参照）。このため
    // 合成後の文字列は inline で指定した `"rotate(10deg)"` という
    // リテラルではなく、ブラウザが解決した `matrix(...)` 表現を含む。
    // 期待値もその実際の computed 値から取得する（テストの弱体化ではなく
    // 実装済みの契約に合わせた修正: 視覚的な効果は同一）。
    let expected_computed_transform = web_sys::window()
        .expect("window must exist in browser test environment")
        .get_computed_style(&div)
        .expect("get_computed_style must not fail")
        .expect("computed style must exist for an attached element")
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_ne!(
        expected_computed_transform, "none",
        "rotate(10deg) は恒等変換ではないはず"
    );

    let spring_config = SpringConfig::default();
    let settle_ms = Spring::new(spring_config, 0.0, 1.0, 0.0)
        .expect("既定 SpringConfig は有効な spring パラメータのため None にならない")
        .settle_duration()
        * 1000.0;

    let anim_loop = flip::play(div.clone(), delta, spring_config, original);

    let transform_during_play = div
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert!(
        transform_during_play.contains(&expected_computed_transform),
        "Play 中も元の transform（computed 値: {expected_computed_transform}）が \
         合成されているはず: {transform_during_play}"
    );

    sleep_ms((settle_ms * 2.0).ceil() as i32 + 200).await;
    anim_loop.stop();

    let transform_after_settle = div
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_eq!(
        transform_after_settle, "rotate(10deg)",
        "収束後は FLIP 補正のみが解除され、元の transform へ正確に復元されるはず"
    );

    div.remove();
}

/// 受け入れ条件（レビュー指摘対応、イシュー #2518 フォローアップ）: 要素が
/// 元々 inline `transform: none` を明示指定していた場合、FLIP 補正へ
/// 他の `transform` 値と同列に文字列連結すると `"... none"` という無効な
/// CSS 値になり `set_property` が反映されない（Play が視覚的に無効化
/// される）。`none` は恒等変換として扱い、補正 CSS のみを設定すること。
#[wasm_bindgen_test]
async fn play_treats_explicit_none_transform_as_identity_and_restores_it_exactly() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    // 利用者が明示的に `transform: none` を指定していたケース（レビュー
    // 指摘対応の検証対象）。
    div.style()
        .set_property("transform", "none")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    let first: Rect = flip::measure(&div);
    div.style()
        .set_property("left", "100px")
        .expect("set_property must not fail");
    div.style()
        .set_property("top", "40px")
        .expect("set_property must not fail");
    let last: Rect = flip::measure(&div);
    let delta = fandhe_frontend_animation::flip::invert(first, last)
        .expect("同一要素の連続計測は常に有効な Rect のため Some");

    let original = flip::OriginalStyle::capture(&div);

    let spring_config = SpringConfig::default();
    let settle_ms = Spring::new(spring_config, 0.0, 1.0, 0.0)
        .expect("既定 SpringConfig は有効な spring パラメータのため None にならない")
        .settle_duration()
        * 1000.0;

    let anim_loop = flip::play(div.clone(), delta, spring_config, original);

    let transform_during_play = div
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert!(
        !transform_during_play.is_empty() && !transform_during_play.ends_with("none"),
        "`none` を他の値と同列に連結した無効な CSS 値になっていないはず: \
         {transform_during_play}"
    );
    assert!(
        transform_during_play.contains("translate") || transform_during_play.contains("scale"),
        "Play 中は FLIP 補正の transform 関数が設定されているはず: {transform_during_play}"
    );

    sleep_ms((settle_ms * 2.0).ceil() as i32 + 200).await;
    anim_loop.stop();

    let transform_after_settle = div
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_eq!(
        transform_after_settle, "none",
        "収束後は元の `transform: none` へ正確に復元されるはず"
    );

    div.remove();
}

/// 受け入れ条件（codex-review 第 4 ラウンド是正・再設計、イシュー
/// #2518）: 旧実装（round 2〜3）は Play 中の CSS `transform-origin`
/// プロパティ自体を元の変形の実際の基準点（既定は要素中心）へ切り替えて
/// いたが、新設計では合成全体を常に `transform-origin: 0 0` の下で行い、
/// 元の変形の基準点は [`flip::OriginalStyle::compose_transform`] が
/// `translate(ox, oy) <O> translate(-ox, -oy)` として明示的に埋め込む
/// （`flip::flip` クレート doc「計測・合成の再設計」節参照）。本テストは
/// `transform-origin` が常に `"0 0"` のまま変わらないことと、それでも
/// 元の変形（`rotate(10deg)`、既定原点 = 中心）の視覚的な効果自体は
/// 正しく再現される（Play 中も収束後も要素の中心位置が変わらない）ことを
/// 固定する。
#[wasm_bindgen_test]
async fn play_keeps_transform_origin_at_top_left_while_preserving_rotation_center() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("left", "0px")
        .expect("set_property must not fail");
    div.style()
        .set_property("top", "0px")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    // 明示 `transform-origin` を指定せず、既定（中心 `50% 50%` ≒
    // `25px 25px`）のまま `rotate` を使う。
    div.style()
        .set_property("transform", "rotate(10deg)")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    // 変化前の中心座標（サイズ・回転が変わらないので中心は要素の
    // `left + width/2`・`top + height/2` から直接計算できる）。
    let expected_center_x = 0.0 + 50.0 / 2.0;
    let expected_center_y = 0.0 + 50.0 / 2.0;

    let first: Rect = flip::measure(&div);
    // 位置のみ変化させる（サイズ・回転は不変のため anchor 補正は
    // 無効化される、`flip::anchor_correct` の `(1-S)` 係数参照）。
    div.style()
        .set_property("left", "100px")
        .expect("set_property must not fail");
    div.style()
        .set_property("top", "40px")
        .expect("set_property must not fail");
    let last: Rect = flip::measure(&div);
    let delta = fandhe_frontend_animation::flip::invert(first, last)
        .expect("同一要素の連続計測は常に有効な Rect のため Some");

    let original = flip::OriginalStyle::capture(&div);
    let spring_config = SpringConfig::default();
    let anim_loop = flip::play(div.clone(), delta, spring_config, original);

    // ブラウザは inline `transform-origin: 0 0` の値をそのままではなく
    // 正規化した `"0px 0px"` として `getPropertyValue` に反映する場合が
    // ある（実測、Chrome）。リテラル一致ではなく「原点が左上（0,0）に
    // 固定されている」ことを検証する（第 4 ラウンドの契約: 合成全体の
    // transform-origin は常に左上に固定する）。
    let origin_during_play = div
        .style()
        .get_property_value("transform-origin")
        .expect("get_property_value must not fail");
    assert!(
        origin_during_play == "0 0" || origin_during_play == "0px 0px",
        "新設計では合成全体の transform-origin は常に左上（0 0）のはず: \
         {origin_during_play}"
    );

    // それでも Invert 直後の実際の中心位置は変化前（中心 25,25）と
    // 一致するはず（`left`/`top` 変化のみで回転・サイズは不変なため）。
    let rect_during_play = div.get_bounding_client_rect();
    let center_x_during_play = rect_during_play.x() + rect_during_play.width() / 2.0;
    let center_y_during_play = rect_during_play.y() + rect_during_play.height() / 2.0;
    assert!(
        (center_x_during_play - expected_center_x).abs() < 1.0,
        "Invert 直後の中心 x は変化前の中心と一致するはずが: {center_x_during_play}"
    );
    assert!(
        (center_y_during_play - expected_center_y).abs() < 1.0,
        "Invert 直後の中心 y は変化前の中心と一致するはずが: {center_y_during_play}"
    );

    anim_loop.stop();
    div.remove();
}

/// [`FIXED_SCALE_CLASS`] の定義を `document.head` へ 1 度だけ挿入する
/// （`crates/wasm-full/tests/position_browser.rs::
/// ensure_fixed_floating_size_stylesheet` と同型: `get_element_by_id` で
/// 冪等性を確保する）。
const FIXED_SCALE_CLASS: &str = "flip-browser-fixed-scale-2";

fn ensure_fixed_scale_stylesheet(document: &web_sys::Document) {
    const STYLE_ELEMENT_ID: &str = "flip-browser-fixed-scale-2-style";
    if document.get_element_by_id(STYLE_ELEMENT_ID).is_some() {
        return;
    }
    let style = document
        .create_element("style")
        .expect("create_element must not fail for a style element");
    style.set_id(STYLE_ELEMENT_ID);
    style.set_text_content(Some(&format!(
        ".{FIXED_SCALE_CLASS} {{ transform: scale(2); transform-origin: 0 0; }}"
    )));
    document
        .head()
        .expect("document head must exist in browser test environment")
        .append_child(&style)
        .expect("append_child must not fail for a style element");
}

/// 受け入れ条件（レビュー指摘対応、イシュー #2518 さらなるフォローアップ、
/// codex-review P1「計測時にスタイルシート由来の transform も除外する」）:
/// `remove_property` は inline 宣言しか除去しないため、CSS クラス由来の
/// `transform`（本テストでは `scale(2)`）は [`flip::measure`] にそのまま
/// 反映されるが、[`flip::measure_clearing_transform`] は由来を問わず
/// 実際に適用される変形を計測時のみ確実に無効化し、真の layout サイズ
/// （変形前のサイズ）を返さなければならない。
#[wasm_bindgen_test]
async fn measure_clearing_transform_excludes_stylesheet_transform() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    ensure_fixed_scale_stylesheet(&document);

    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.set_class_name(FIXED_SCALE_CLASS);
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("left", "0px")
        .expect("set_property must not fail");
    div.style()
        .set_property("top", "0px")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "100px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    // セットアップ確認: CSS クラスの `scale(2)` が実際に効いているはず
    // （効いていなければ本テストは前提が崩れ意味をなさない）。
    let with_transform = flip::measure(&div);
    assert!(
        (with_transform.width - 200.0).abs() < 1.0,
        "class の scale(2) が適用され見かけの幅は 200px のはず: width={}",
        with_transform.width
    );

    // 検証対象: `measure_clearing_transform` はスタイルシート由来の
    // `transform` も無効化し、真の layout 幅（100px）を返すはず。
    let cleared = flip::measure_clearing_transform(&div);
    assert!(
        (cleared.width - 100.0).abs() < 1.0,
        "スタイルシート由来の transform を除外した真の layout 幅は 100px \
         のはず: width={}",
        cleared.width
    );
    assert!(
        (cleared.x - 0.0).abs() < 1.0,
        "transform 除外後の左端は 0px のはず: x={}",
        cleared.x
    );

    // 測定後、クラスの transform（stylesheet 側）はそのまま残るため、
    // 通常の `measure` は依然 scale(2) 適用後の値を返すはず（一時的な
    // クリアが inline へだけ影響し、測定後は元の見た目に戻ることの確認）。
    let after = flip::measure(&div);
    assert!(
        (after.width - 200.0).abs() < 1.0,
        "計測後も見た目（class の scale(2)）は変化していないはず: width={}",
        after.width
    );

    div.remove();
}

/// 受け入れ条件（codex-review 第 4 ラウンド是正・再設計、イシュー
/// #2518）: 要素が元々持っていた inline `translateX(20px)` は、幅変化に
/// 伴う FLIP のスケール補正へ巻き込まれてはならない。左端固定
/// （`left: 100px` は幅変化と無関係）で幅だけ 100px→50px に変化する
/// 場合、Invert 直後（Play 開始直後）の実際の左端は「変更前の並進のみを
/// 反映した位置」（100+20=120px）と一致するはず。新設計の計測・合成
/// パイプライン（First/Last 視覚矩形 + Last layout 矩形 +
/// [`flip::anchor_correct`]）をそのまま経由して確認する。
#[wasm_bindgen_test]
async fn play_does_not_scale_pre_existing_translate_when_width_changes() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("left", "100px")
        .expect("set_property must not fail");
    div.style()
        .set_property("top", "0px")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "100px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    // 利用者が FLIP とは無関係に指定していた並進（検証対象）。
    div.style()
        .set_property("transform", "translateX(20px)")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    // First は現在の視覚矩形（要素自身の変形込み）を直接測る（新設計、
    // モジュール doc「計測・合成の再設計」参照。旧実装の
    // `measure_clearing_transform` はここでは使わない）。
    let first_visual: Rect = flip::measure(&div);
    // 幅のみを変化させる（左端 `left` は不変）。
    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    let last_visual: Rect = flip::measure(&div);
    let last_layout: Rect = flip::measure_clearing_transform(&div);
    let delta = fandhe_frontend_animation::flip::invert(first_visual, last_visual)
        .expect("同一要素の連続計測は常に有効な Rect のため Some");
    let corrected = flip::anchor_correct(delta, last_visual, last_layout);

    let original = flip::OriginalStyle::capture(&div);
    let spring_config = SpringConfig::default();
    let anim_loop = flip::play(div.clone(), corrected, spring_config, original);

    let rect_during_play = div.get_bounding_client_rect();
    assert!(
        (rect_during_play.x() - 120.0).abs() < 1.0,
        "変更前の並進のみを反映した左端 120px（=100+20）のはずが: x={}",
        rect_during_play.x()
    );

    anim_loop.stop();
    div.remove();
}

/// 受け入れ条件（レビュー指摘対応、イシュー #2518 さらなるフォローアップ、
/// codex-review P1「公開 play API の途中停止でも元のスタイルを復元
/// する」）: 収束前に [`FlipAnimation::stop`] を呼んだ場合でも、元の
/// `transform` へ正確に復元されなければならない（`AnimationLoop::stop`
/// のみでは rAF が止まるだけで DOM の `transform` は残ってしまう旧実装
/// の回帰防止）。
#[wasm_bindgen_test]
async fn play_stop_before_convergence_restores_original_style() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("transform", "rotate(10deg)")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    let first: Rect = flip::measure(&div);
    div.style()
        .set_property("left", "100px")
        .expect("set_property must not fail");
    let last: Rect = flip::measure(&div);
    let delta = fandhe_frontend_animation::flip::invert(first, last)
        .expect("同一要素の連続計測は常に有効な Rect のため Some");

    let original = flip::OriginalStyle::capture(&div);
    let spring_config = SpringConfig::default();
    let anim_loop = flip::play(div.clone(), delta, spring_config, original);

    // 収束前（Play 開始直後）に明示停止する。
    let transform_during_play = div
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_ne!(
        transform_during_play, "rotate(10deg)",
        "Play 開始直後は FLIP 補正が合成されているはず"
    );
    anim_loop.stop();

    let transform_after_stop = div
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_eq!(
        transform_after_stop, "rotate(10deg)",
        "収束前に stop しても元の transform へ正確に復元されるはず"
    );

    div.remove();
}

/// [`play_stop_before_convergence_restores_original_style`] の drop 版
/// （`stop()` を呼ばず `AnimationLoop` の戻り値を単に drop するだけでも
/// 同じ復元契約が成り立つことを固定する。`FlipAnimation::stop` は
/// self を消費するだけの空実装であり、[`Drop`] のみが実際の復元処理を
/// 担うため、両経路が等価であることの確認でもある）。
#[wasm_bindgen_test]
async fn play_drop_before_convergence_restores_original_style() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    let first: Rect = flip::measure(&div);
    div.style()
        .set_property("left", "100px")
        .expect("set_property must not fail");
    let last: Rect = flip::measure(&div);
    let delta = fandhe_frontend_animation::flip::invert(first, last)
        .expect("同一要素の連続計測は常に有効な Rect のため Some");

    let original = flip::OriginalStyle::capture(&div);
    let spring_config = SpringConfig::default();
    let anim_loop = flip::play(div.clone(), delta, spring_config, original);
    drop(anim_loop);

    let transform_after_drop = div
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_eq!(
        transform_after_drop, "",
        "元々 transform 未設定だった場合、drop 後は remove_property と \
         等価な状態（空文字列）へ復元されるはず"
    );

    div.remove();
}

/// [`ancestor_div_with_transform`] が挿入する親要素の transform を
/// 一意にするための prefix（複数テストの並行実行があっても id が衝突
/// しないよう呼び出し元がテスト名を渡す）。
fn ancestor_div_with_transform(
    document: &web_sys::Document,
    id_prefix: &str,
    transform: &str,
) -> web_sys::Element {
    let ancestor = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    ancestor.set_id(&format!("{id_prefix}-ancestor"));
    ancestor
        .dyn_ref::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement")
        .style()
        .set_property("transform", transform)
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&ancestor)
        .expect("append_child must not fail for a detached div");
    ancestor
}

/// 受け入れ条件（レビュー指摘対応、イシュー #2518 さらなるフォローアップ、
/// codex-review P1「祖先の回転をサイズ補正にも反映する」）: 祖先が
/// 非軸整列変換（`rotate(90deg)`）を持つ場合、`sx`/`sy` を正しく
/// ローカル座標へ変換できないため [`flip::play`] は再生を省略し、
/// 即座に元のスタイルへ復元した収束済みハンドルを返す。
#[wasm_bindgen_test]
async fn play_skips_animation_under_rotated_ancestor() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let ancestor =
        ancestor_div_with_transform(&document, "play-skips-rotated-ancestor", "rotate(90deg)");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "20px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "20px")
        .expect("set_property must not fail");
    ancestor
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    let delta = flip::FlipDelta {
        tx: 10.0,
        ty: 5.0,
        sx: 2.0,
        sy: 0.5,
    };
    let original = flip::OriginalStyle::capture(&div);
    let spring_config = SpringConfig::default();
    let anim_loop = flip::play(div.clone(), delta, spring_config, original);

    assert!(
        anim_loop.is_done(),
        "祖先が rotate(90deg) の場合、sx/sy をローカル座標へ正しく変換 \
         できないため再生を省略し即座に収束済みとなるはず"
    );
    let transform_after_skip = div
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_eq!(
        transform_after_skip, "",
        "再生を省略した場合、元々 transform 未設定だった要素の transform \
         は書き込まれないままのはず: {transform_after_skip}"
    );

    div.remove();
    ancestor.remove();
}

/// [`FIXED_SCALE_IMPORTANT_CLASS`] の定義を `document.head` へ 1 度だけ
/// 挿入する（[`ensure_fixed_scale_stylesheet`] と同型）。
const FIXED_SCALE_IMPORTANT_CLASS: &str = "flip-browser-fixed-scale-2-important";

fn ensure_fixed_scale_important_stylesheet(document: &web_sys::Document) {
    const STYLE_ELEMENT_ID: &str = "flip-browser-fixed-scale-2-important-style";
    if document.get_element_by_id(STYLE_ELEMENT_ID).is_some() {
        return;
    }
    let style = document
        .create_element("style")
        .expect("create_element must not fail for a style element");
    style.set_id(STYLE_ELEMENT_ID);
    style.set_text_content(Some(&format!(
        ".{FIXED_SCALE_IMPORTANT_CLASS} {{ transform: scale(2) !important; \
         transform-origin: 0 0 !important; }}"
    )));
    document
        .head()
        .expect("document head must exist in browser test environment")
        .append_child(&style)
        .expect("append_child must not fail for a style element");
}

/// 受け入れ条件（レビュー指摘対応、イシュー #2518 さらなるフォローアップ、
/// codex-review P1「再生中もスタイルシートの important 宣言を上書きする」）:
/// スタイルシートに `transform: scale(2) !important` がある要素でも、
/// Play 中の inline 書き込みは `!important` 付きで行われ、カスケードで
/// スタイルシート側に負けない（優先度が `"important"` になっている）。
#[wasm_bindgen_test]
async fn play_writes_transform_with_important_priority_to_beat_stylesheet() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    ensure_fixed_scale_important_stylesheet(&document);

    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.set_class_name(FIXED_SCALE_IMPORTANT_CLASS);
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "20px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "20px")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    let first: Rect = flip::measure_clearing_transform(&div);
    div.style()
        .set_property("left", "50px")
        .expect("set_property must not fail");
    let last: Rect = flip::measure_clearing_transform(&div);
    let delta = fandhe_frontend_animation::flip::invert(first, last)
        .expect("同一要素の連続計測は常に有効な Rect のため Some");

    let original = flip::OriginalStyle::capture(&div);
    let spring_config = SpringConfig::default();
    let anim_loop = flip::play(div.clone(), delta, spring_config, original);

    let style = div.style();
    assert_eq!(
        style.get_property_priority("transform"),
        "important",
        "スタイルシートの `!important` に勝つため、Play 中の inline \
         transform 書き込みも `!important` で行われるはず"
    );
    assert_eq!(
        style.get_property_priority("transform-origin"),
        "important",
        "transform-origin も同様に `!important` で書き込まれるはず"
    );

    anim_loop.stop();
    div.remove();
}

/// 受け入れ条件（レビュー指摘対応、イシュー #2518 さらなるフォローアップ、
/// codex-review P1「反転した祖先の下では拡縮に伴う原点のずれも補正
/// する」）: 祖先が `scaleX(-1)`（対角成分が負の反転）を持つ場合、並進
/// のみの逆行列変換ではサイズ変更に伴う原点のずれを補正できないため、
/// [`flip::play`] は再生を省略し即座に元のスタイルへ復元した収束済み
/// ハンドルを返す（`is_axis_aligned_linear` doc「対応範囲」参照）。
#[wasm_bindgen_test]
async fn play_skips_animation_under_flipped_ancestor() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let ancestor =
        ancestor_div_with_transform(&document, "play-skips-flipped-ancestor", "scaleX(-1)");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("left", "0px")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "100px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "20px")
        .expect("set_property must not fail");
    ancestor
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    let first: Rect = flip::measure_clearing_transform(&div);
    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    let last: Rect = flip::measure_clearing_transform(&div);
    let delta = fandhe_frontend_animation::flip::invert(first, last)
        .expect("同一要素の連続計測は常に有効な Rect のため Some");

    let original = flip::OriginalStyle::capture(&div);
    let spring_config = SpringConfig::default();
    let anim_loop = flip::play(div.clone(), delta, spring_config, original);

    assert!(
        anim_loop.is_done(),
        "祖先が scaleX(-1)（反転）の場合、並進のみの逆行列変換ではサイズ \
         変更に伴う原点のずれを補正できないため再生を省略し即座に収束済み \
         となるはず"
    );
    let transform_after_skip = div
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_eq!(
        transform_after_skip, "",
        "再生を省略した場合、元々 transform 未設定だった要素の transform \
         は書き込まれないままのはず: {transform_after_skip}"
    );

    div.remove();
    ancestor.remove();
}

/// 受け入れ条件（codex-review 第 4 ラウンド是正・再設計、イシュー
/// #2518。テスト (c)）: 要素自身が `rotate(90deg)`（`transform-origin:
/// 0 0`）を持ち、幅が変化する場合、Invert 直後（Play 開始直後、rAF 未
/// 実行の時点）の実際の視覚矩形は「変更前の実際の視覚矩形」（回転で
/// 幅・高さが入れ替わった 50×100px）と一致するはず。100×50px→50×50px
/// の変化に対し、新設計の計測・合成パイプライン（First/Last **視覚**
/// 矩形 + Last **layout** 矩形 + [`flip::anchor_correct`]）を経由して
/// 確認する（round 3 は本シナリオを "<original> <flip>" の文字列順
/// 検証で固定していたが、新設計は合成の内部構造が変わったため、視覚
/// 矩形の一致を assert する形へ書き換える）。
#[wasm_bindgen_test]
async fn play_reproduces_first_rendered_rect_under_own_rotation_with_non_uniform_scale() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("left", "0px")
        .expect("set_property must not fail");
    div.style()
        .set_property("top", "0px")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "100px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("transform", "rotate(90deg)")
        .expect("set_property must not fail");
    div.style()
        .set_property("transform-origin", "0 0")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    // First は現在の視覚矩形を直接測る（新設計、モジュール doc
    // 「計測・合成の再設計」参照）。
    let first_visual: Rect = flip::measure(&div);
    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    let last_visual: Rect = flip::measure(&div);
    let last_layout: Rect = flip::measure_clearing_transform(&div);
    let delta = fandhe_frontend_animation::flip::invert(first_visual, last_visual)
        .expect("同一要素の連続計測は常に有効な Rect のため Some");
    let corrected = flip::anchor_correct(delta, last_visual, last_layout);

    let original = flip::OriginalStyle::capture(&div);
    let spring_config = SpringConfig::default();
    let anim_loop = flip::play(div.clone(), corrected, spring_config, original);

    // 変更前（First、幅 100px）に `rotate(90deg)` を適用した実際の表示
    // 矩形は 50×100px（幅・高さが入れ替わる）。旧実装の誤った合成順は
    // 100×50px を生む（回転後の軸を拡縮してしまうため）。
    let rect_during_play = div.get_bounding_client_rect();
    assert!(
        (rect_during_play.width() - 50.0).abs() < 1.0,
        "変更前の実際の表示幅は 50px のはずが: width={}",
        rect_during_play.width()
    );
    assert!(
        (rect_during_play.height() - 100.0).abs() < 1.0,
        "変更前の実際の表示高さは 100px のはずが: height={}",
        rect_during_play.height()
    );

    anim_loop.stop();
    div.remove();
}

/// 受け入れ条件（codex-review 第 4 ラウンド是正、イシュー #2518。テスト
/// (a)）: 要素自身 `scale(2)`・`transform-origin: 0 0` を持ち、サイズは
/// 変化せず `left` のみ変化する場合、Invert 直後の実際の左端は First
/// （変化前）の左端と一致する（advisor 検証済みの thread 1 相当。
/// `transform-origin: 0 0` は原点が box の局所原点と一致するため
/// [`flip::anchor_correct`] の補正項が構造的に 0 になるケース、
/// `flip::anchor_correct` doc「数値例」参照）。
#[wasm_bindgen_test]
async fn play_reproduces_first_left_edge_under_own_scale_with_origin_at_top_left() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("left", "0px")
        .expect("set_property must not fail");
    div.style()
        .set_property("top", "0px")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("transform", "scale(2)")
        .expect("set_property must not fail");
    div.style()
        .set_property("transform-origin", "0 0")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    let first_visual: Rect = flip::measure(&div);
    assert!(
        (first_visual.x - 0.0).abs() < 1.0,
        "scale(2)・origin 0 0 の下では left=0 の視覚上の左端も 0 のはず: \
         x={}",
        first_visual.x
    );

    // サイズは変えず、layout 位置のみ変化させる。
    div.style()
        .set_property("left", "50px")
        .expect("set_property must not fail");
    let last_visual: Rect = flip::measure(&div);
    let last_layout: Rect = flip::measure_clearing_transform(&div);
    let delta = fandhe_frontend_animation::flip::invert(first_visual, last_visual)
        .expect("同一要素の連続計測は常に有効な Rect のため Some");
    let corrected = flip::anchor_correct(delta, last_visual, last_layout);

    let original = flip::OriginalStyle::capture(&div);
    let spring_config = SpringConfig::default();
    let anim_loop = flip::play(div.clone(), corrected, spring_config, original);

    let rect_during_play = div.get_bounding_client_rect();
    assert!(
        (rect_during_play.x() - first_visual.x).abs() < 1.0,
        "Invert 直後の左端は First の左端 {} と一致するはずが: {}",
        first_visual.x,
        rect_during_play.x()
    );

    anim_loop.stop();
    div.remove();
}

/// 受け入れ条件（codex-review 第 4 ラウンド是正、イシュー #2518。テスト
/// (b)）: 要素自身 `scale(2)`・既定原点（中心）を持ち、幅が
/// 100px→50px に変化する場合、Invert 直後の実際の左端は First（変化前）
/// の左端 -50px と一致する（`flip::anchor_correct` doc の数値例と同一
/// シナリオ。第 2〜3 ラウンドはこのケースで収束しなかった、イシュー
/// #2518 の codex-review 第 4 ラウンド指摘の再現ケース）。
#[wasm_bindgen_test]
async fn play_reproduces_first_left_edge_under_own_scale_with_center_origin_on_resize() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("left", "0px")
        .expect("set_property must not fail");
    div.style()
        .set_property("top", "0px")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "100px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    // 明示 `transform-origin` を指定せず既定（中心 `50% 50%`）のまま
    // `scale(2)` を使う（検証対象）。
    div.style()
        .set_property("transform", "scale(2)")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    let first_visual: Rect = flip::measure(&div);
    assert!(
        (first_visual.x - -50.0).abs() < 1.0,
        "中心原点の scale(2)・幅 100px の場合、変化前の視覚上の左端は \
         -50px のはずが: x={}",
        first_visual.x
    );

    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    let last_visual: Rect = flip::measure(&div);
    let last_layout: Rect = flip::measure_clearing_transform(&div);
    let delta = fandhe_frontend_animation::flip::invert(first_visual, last_visual)
        .expect("同一要素の連続計測は常に有効な Rect のため Some");
    let corrected = flip::anchor_correct(delta, last_visual, last_layout);

    let original = flip::OriginalStyle::capture(&div);
    let spring_config = SpringConfig::default();
    let anim_loop = flip::play(div.clone(), corrected, spring_config, original);

    let rect_during_play = div.get_bounding_client_rect();
    assert!(
        (rect_during_play.x() - first_visual.x).abs() < 1.0,
        "Invert 直後の左端は First の左端 {} と一致するはずが: {}\
         （旧実装〔第 2〜3 ラウンド〕はここで収束しなかった）",
        first_visual.x,
        rect_during_play.x()
    );

    anim_loop.stop();
    div.remove();
}

/// [`FIXED_SCALE_RESIZE_CLASS`] の定義を `document.head` へ 1 度だけ
/// 挿入する（[`ensure_fixed_scale_stylesheet`] と同型）。
const FIXED_SCALE_RESIZE_CLASS: &str = "flip-browser-fixed-scale-2-resize";

fn ensure_fixed_scale_resize_stylesheet(document: &web_sys::Document) {
    const STYLE_ELEMENT_ID: &str = "flip-browser-fixed-scale-2-resize-style";
    if document.get_element_by_id(STYLE_ELEMENT_ID).is_some() {
        return;
    }
    let style = document
        .create_element("style")
        .expect("create_element must not fail for a style element");
    style.set_id(STYLE_ELEMENT_ID);
    style.set_text_content(Some(&format!(
        ".{FIXED_SCALE_RESIZE_CLASS} {{ transform: scale(2); }}"
    )));
    document
        .head()
        .expect("document head must exist in browser test environment")
        .append_child(&style)
        .expect("append_child must not fail for a style element");
}

/// 受け入れ条件（codex-review 第 4 ラウンド是正、イシュー #2518。テスト
/// (d)）: 要素自身の変形がスタイルシート（CSS クラス）由来
/// （`transform: scale(2)`、既定中心原点）でも inline 由来と同じ契約
/// （幅変化時の Invert 直後の左端一致）が成り立つ
/// （[`flip::OriginalStyle::capture`] の `resolve_computed_transform`/
/// `resolve_origin_px` がスタイルシート由来の値も解決する契約、
/// `measure_clearing_transform_excludes_stylesheet_transform` と同型）。
#[wasm_bindgen_test]
async fn play_reproduces_first_left_edge_under_stylesheet_scale_on_resize() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    ensure_fixed_scale_resize_stylesheet(&document);

    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.set_class_name(FIXED_SCALE_RESIZE_CLASS);
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("left", "0px")
        .expect("set_property must not fail");
    div.style()
        .set_property("top", "0px")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "100px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    let first_visual: Rect = flip::measure(&div);
    assert!(
        (first_visual.x - -50.0).abs() < 1.0,
        "class 由来 scale(2)・中心原点・幅 100px の場合、変化前の視覚上の \
         左端は -50px のはずが: x={}",
        first_visual.x
    );

    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    let last_visual: Rect = flip::measure(&div);
    let last_layout: Rect = flip::measure_clearing_transform(&div);
    let delta = fandhe_frontend_animation::flip::invert(first_visual, last_visual)
        .expect("同一要素の連続計測は常に有効な Rect のため Some");
    let corrected = flip::anchor_correct(delta, last_visual, last_layout);

    // `OriginalStyle::capture` はスタイルシート由来の transform も
    // `computed_transform` として解決するため、class を外さずそのまま
    // 合成できる（`compose_transform` doc 参照）。
    let original = flip::OriginalStyle::capture(&div);
    let spring_config = SpringConfig::default();
    let anim_loop = flip::play(div.clone(), corrected, spring_config, original);

    let rect_during_play = div.get_bounding_client_rect();
    assert!(
        (rect_during_play.x() - first_visual.x).abs() < 1.0,
        "Invert 直後の左端は First の左端 {} と一致するはずが: {}",
        first_visual.x,
        rect_during_play.x()
    );

    anim_loop.stop();
    div.remove();
}

/// 受け入れ条件（codex-review 第 5 ラウンド是正、イシュー #2518）: 対象
/// 要素に `transition: transform 300ms` があっても、計測・再生中は
/// `transition` を `none` へ強制し、収束後は元の `transition` 宣言へ
/// 正確に復元する（[`OriginalStyle::restore`] doc 参照）。
#[wasm_bindgen_test]
async fn play_suppresses_existing_transition_during_play_and_restores_it() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    // 利用者が FLIP とは無関係に指定していた transition（検証対象）。
    div.style()
        .set_property("transition", "transform 300ms")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    let first: Rect = flip::measure(&div);
    div.style()
        .set_property("left", "100px")
        .expect("set_property must not fail");
    let last: Rect = flip::measure(&div);
    let delta = fandhe_frontend_animation::flip::invert(first, last)
        .expect("同一要素の連続計測は常に有効な Rect のため Some");

    let original = flip::OriginalStyle::capture(&div);
    let spring_config = SpringConfig::default();
    let settle_ms = Spring::new(spring_config, 0.0, 1.0, 0.0)
        .expect("既定 SpringConfig は有効な spring パラメータのため None にならない")
        .settle_duration()
        * 1000.0;
    let anim_loop = flip::play(div.clone(), delta, spring_config, original);

    // codex-review 第 7 ラウンド是正（イシュー #2518。P1「抑止を
    // transform / transform-origin に限定する」）: 抑止は
    // `transition-property` の末尾へ `transform, transform-origin` を
    // **追加**し、対応する `duration`/`delay` を `0s, 0s` へ指定する
    // 方式へ変更した（既存の項目は変更しない、`compute_transition_
    // suppression` doc 参照）。CSS の「同じプロパティが複数回現れたら
    // 最後が勝つ」規則により、末尾に追加した `transform` エントリの
    // `0s` が実効の duration/delay になる。リストの正確な項目数・区切り
    // 表記はブラウザの CSSOM シリアライズ実装依存であるため、末尾に
    // `transform`/`transform-origin` が追加され、その対応 index の
    // duration/delay が `0s` であることのみを確認する（実装詳細への
    // 過度な結合を避ける）。
    let duration_during_play = div
        .style()
        .get_property_value("transition-duration")
        .expect("get_property_value must not fail");
    let delay_during_play = div
        .style()
        .get_property_value("transition-delay")
        .expect("get_property_value must not fail");
    let property_during_play = div
        .style()
        .get_property_value("transition-property")
        .expect("get_property_value must not fail");
    let property_items: Vec<&str> = property_during_play.split(',').map(str::trim).collect();
    let duration_items: Vec<&str> = duration_during_play.split(',').map(str::trim).collect();
    let delay_items: Vec<&str> = delay_during_play.split(',').map(str::trim).collect();
    assert_eq!(
        &property_items[property_items.len() - 2..],
        ["transform", "transform-origin"],
        "transition-property の末尾に transform, transform-origin が \
         追加されているはず: {property_during_play}"
    );
    assert_eq!(
        &duration_items[duration_items.len() - 2..],
        ["0s", "0s"],
        "追加された transform, transform-origin の duration が 0s へ \
         強制されているはず: {duration_during_play}"
    );
    assert_eq!(
        &delay_items[delay_items.len() - 2..],
        ["0s", "0s"],
        "追加された transform, transform-origin の delay が 0s へ \
         強制されているはず: {delay_during_play}"
    );

    sleep_ms((settle_ms * 2.0).ceil() as i32 + 200).await;
    anim_loop.stop();

    let duration_after_settle = div
        .style()
        .get_property_value("transition-duration")
        .expect("get_property_value must not fail");
    assert_eq!(
        duration_after_settle, "300ms",
        "収束後は元の transition-duration（300ms）へ正確に復元される \
         はず: {duration_after_settle}"
    );

    div.remove();
}

/// 受け入れ条件（codex-review 第 6 ラウンド是正、イシュー #2518。P1
/// 「FLIP と無関係な進行中の遷移をキャンセルしない」）: 対象行の
/// layout 計測（`measure_clearing_transform`）が走っても、同時に
/// `opacity` で進行中の別の CSS transition はキャンセルされず、
/// フェード完了まで継続する。
#[wasm_bindgen_test]
async fn measure_clearing_transform_does_not_cancel_unrelated_opacity_transition() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("opacity", "1")
        .expect("set_property must not fail");
    div.style()
        .set_property("transition", "opacity 2000ms linear")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    // 現在の opacity（1）を確定させてから遷移を開始する（同期スタイル
    // 計算の強制、`flush_style` と同じ手法）。
    let _ = div.offset_width();
    div.style()
        .set_property("opacity", "0")
        .expect("set_property must not fail");

    // 遷移が実際に開始してから（十分に短い時間だけ待つ）、layout 計測を
    // 走らせる。
    sleep_ms(100).await;
    let opacity_before_measure: f64 = web_sys::window()
        .expect("window must exist in browser test environment")
        .get_computed_style(&div)
        .expect("get_computed_style must not fail")
        .expect("computed style must exist for an attached element")
        .get_property_value("opacity")
        .expect("get_property_value must not fail")
        .parse()
        .expect("opacity computed value must be a valid float");
    assert!(
        opacity_before_measure < 0.99 && opacity_before_measure > 0.0,
        "計測前、opacity の遷移は進行中（1 と 0 の間）のはずが: {opacity_before_measure}"
    );

    let _ = flip::measure_clearing_transform(&div);

    let opacity_after_measure: f64 = web_sys::window()
        .expect("window must exist in browser test environment")
        .get_computed_style(&div)
        .expect("get_computed_style must not fail")
        .expect("computed style must exist for an attached element")
        .get_property_value("opacity")
        .expect("get_property_value must not fail")
        .parse()
        .expect("opacity computed value must be a valid float");
    assert!(
        (opacity_after_measure - opacity_before_measure).abs() < 0.05,
        "layout 計測が opacity の進行中の遷移をキャンセルして終端値へ \
         跳ばしてはならない: before={opacity_before_measure} \
         after={opacity_after_measure}"
    );

    div.remove();
}

/// 受け入れ条件（codex-review 第 6 ラウンド是正、イシュー #2518。P1
/// 「変形の復元を確定してから transition を戻す」）: 要素自身が
/// `transform: scale(2); transition: transform 300ms` を持つ場合、
/// `measure_clearing_transform` で layout 矩形を計測した直後、computed
/// transform は即座に `scale(2)`（`matrix(2, 0, 0, 2, 0, 0)`）へ戻って
/// おり、元の 300ms transition に乗って途中の値のまま留まらない。
#[wasm_bindgen_test]
async fn measure_clearing_transform_restores_own_scale_instantly_despite_transition() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("transform", "scale(2)")
        .expect("set_property must not fail");
    div.style()
        .set_property("transition", "transform 300ms")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    let _ = flip::measure_clearing_transform(&div);

    // `measure_clearing_transform` から戻った直後（次の rAF/タイマーを
    // 待たない同期的なタイミング）で computed transform を確認する。
    // `flush_style` によって、復元後の scale(2) が 300ms の遷移の途中の
    // 値ではなく即座に確定しているはず。
    let computed_transform = web_sys::window()
        .expect("window must exist in browser test environment")
        .get_computed_style(&div)
        .expect("get_computed_style must not fail")
        .expect("computed style must exist for an attached element")
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_eq!(
        computed_transform, "matrix(2, 0, 0, 2, 0, 0)",
        "測定後、要素自身の scale(2) は 300ms の遷移を経ずに即座に \
         確定しているはずが: {computed_transform}"
    );

    div.remove();
}

/// 受け入れ条件（codex-review 第 7 ラウンド是正、イシュー #2518。P1
/// 「抑止を transform / transform-origin に限定する」）: 要素が
/// `transition: opacity 300ms, transform 300ms` を持つ場合、FLIP の
/// 計測・再生中（[`play`]）に `opacity` を変えても、その遷移は
/// `transform`/`transform-origin` に限定された抑止の影響を受けず引き続き
/// 300ms で進行する（`transition-property` に既に `transform` が含まれて
/// いても、末尾に追加したエントリだけが実効を持ち既存の `opacity`
/// エントリは変更されない、[`fandhe_frontend_animation::flip::wiring::
/// compute_transition_suppression`] doc 参照。`wiring` は非公開なため
/// このリンクは説明目的のコメント）。一方 `transform` 自身は FLIP 補正
/// により即時反映される。
#[wasm_bindgen_test]
async fn play_does_not_affect_unrelated_opacity_transition_while_suppressing_own_transform() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("opacity", "1")
        .expect("set_property must not fail");
    // FLIP と無関係な opacity の遷移・FLIP が抑止対象にする transform の
    // 遷移を両方持つ要素（コーディネータ指定の受け入れ条件）。
    div.style()
        .set_property("transition", "opacity 300ms linear, transform 300ms linear")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    let first: Rect = flip::measure(&div);
    div.style()
        .set_property("left", "100px")
        .expect("set_property must not fail");
    let last: Rect = flip::measure(&div);
    let delta = fandhe_frontend_animation::flip::invert(first, last)
        .expect("同一要素の連続計測は常に有効な Rect のため Some");

    let original = flip::OriginalStyle::capture(&div);
    let spring_config = SpringConfig::default();
    let anim_loop = flip::play(div.clone(), delta, spring_config, original);

    // 現在の opacity（1）を確定させてから遷移を開始する（`flush_style` と
    // 同じ手法）。FLIP 再生中に無関係な opacity 遷移を開始する。
    let _ = div.offset_width();
    div.style()
        .set_property("opacity", "0")
        .expect("set_property must not fail");

    sleep_ms(100).await;
    let opacity_at_100ms: f64 = web_sys::window()
        .expect("window must exist in browser test environment")
        .get_computed_style(&div)
        .expect("get_computed_style must not fail")
        .expect("computed style must exist for an attached element")
        .get_property_value("opacity")
        .expect("get_property_value must not fail")
        .parse()
        .expect("opacity computed value must be a valid float");
    assert!(
        opacity_at_100ms < 0.99 && opacity_at_100ms > 0.0,
        "FLIP 再生中でも opacity の 300ms 遷移は継続しているはず \
         （1 と 0 の間の中間値であるはず）: {opacity_at_100ms}"
    );

    sleep_ms(400).await;
    let opacity_after_settle: f64 = web_sys::window()
        .expect("window must exist in browser test environment")
        .get_computed_style(&div)
        .expect("get_computed_style must not fail")
        .expect("computed style must exist for an attached element")
        .get_property_value("opacity")
        .expect("get_property_value must not fail")
        .parse()
        .expect("opacity computed value must be a valid float");
    assert!(
        opacity_after_settle < 0.01,
        "十分な時間経過後、opacity の遷移は終端値（0）へ到達している \
         はず: {opacity_after_settle}"
    );

    anim_loop.stop();
    div.remove();
}

/// 受け入れ条件（codex-review 第 5 ラウンド是正、イシュー #2518。
/// 契約絞り込み）: 要素自身が 3D 変形（`matrix3d(...)` として算出
/// される `rotateY`）を持つ場合、`play` は再生自体を省略し即座に元の
/// スタイルへ復元した収束済みハンドルを返す（`OriginalStyle::
/// unsupported`・`is_matrix3d` の rustdoc 参照。2D 専用の計測・合成
/// 契約の対応範囲外であることを明示する契約絞り込み）。
#[wasm_bindgen_test]
async fn play_skips_animation_for_own_3d_transform() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    // `rotateY` は 3D 変形関数であり、computed transform は `matrix(...)`
    // ではなく `matrix3d(...)`（16 要素）として算出される（検証対象）。
    div.style()
        .set_property("transform", "rotateY(30deg)")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    // セットアップ確認: computed transform が実際に matrix3d であるはず
    // （前提が崩れていれば本テストは意味をなさない）。
    let computed_transform = web_sys::window()
        .expect("window must exist in browser test environment")
        .get_computed_style(&div)
        .expect("get_computed_style must not fail")
        .expect("computed style must exist for an attached element")
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert!(
        computed_transform.starts_with("matrix3d("),
        "rotateY(30deg) の computed transform は matrix3d(...) のはずが: \
         {computed_transform}"
    );

    let first: Rect = flip::measure(&div);
    div.style()
        .set_property("left", "100px")
        .expect("set_property must not fail");
    let last: Rect = flip::measure(&div);
    let delta = fandhe_frontend_animation::flip::invert(first, last)
        .expect("同一要素の連続計測は常に有効な Rect のため Some");

    let original = flip::OriginalStyle::capture(&div);
    let spring_config = SpringConfig::default();
    let anim_loop = flip::play(div.clone(), delta, spring_config, original);

    assert!(
        anim_loop.is_done(),
        "3D 変形（matrix3d）を持つ要素は契約絞り込みにより再生を省略し \
         即座に収束済みとなるはず"
    );
    let transform_after_skip = div
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_eq!(
        transform_after_skip, "rotateY(30deg)",
        "再生を省略した場合、元の transform（rotateY(30deg)）へそのまま \
         復元されているはず: {transform_after_skip}"
    );

    div.remove();
}

/// 受け入れ条件（codex-review 第 5 ラウンド是正、イシュー #2518。
/// 契約絞り込み）: 要素自身が独立した CSS 変形プロパティ（`scale`）を
/// 持つ場合、`play` は再生自体を省略する（`has_independent_transform_
/// property` の rustdoc 参照。`transform` プロパティだけを読む
/// `resolve_computed_transform`/合成では `scale` の効果を捕捉できない
/// ため）。
#[wasm_bindgen_test]
async fn play_skips_animation_for_own_independent_scale_property() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    // `transform` とは独立した CSS Transforms Level 2 プロパティ
    // （検証対象）。
    div.style()
        .set_property("scale", "2")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    // セットアップ確認: `scale: 2` が実際に効いているはず（効いていな
    // ければ本テストは前提が崩れ意味をなさない）。
    let with_scale = flip::measure(&div);
    assert!(
        (with_scale.width - 100.0).abs() < 1.0,
        "独立プロパティ scale: 2 が適用され見かけの幅は 100px のはず: \
         width={}",
        with_scale.width
    );

    let first: Rect = flip::measure(&div);
    div.style()
        .set_property("left", "100px")
        .expect("set_property must not fail");
    let last: Rect = flip::measure(&div);
    let delta = fandhe_frontend_animation::flip::invert(first, last)
        .expect("同一要素の連続計測は常に有効な Rect のため Some");

    let original = flip::OriginalStyle::capture(&div);
    let spring_config = SpringConfig::default();
    let anim_loop = flip::play(div.clone(), delta, spring_config, original);

    assert!(
        anim_loop.is_done(),
        "独立した scale プロパティを持つ要素は契約絞り込みにより再生を \
         省略し即座に収束済みとなるはず"
    );

    div.remove();
}

/// 受け入れ条件（codex-review 第 5 ラウンド是正、イシュー #2518。
/// 契約絞り込み）: 祖先が独立した CSS 変形プロパティ（`scale`）を
/// 持つ場合、`play` は再生自体を省略する（`ancestor_linear_matrix` が
/// `None` を返す経路、`has_independent_transform_property` の rustdoc
/// 参照）。
#[wasm_bindgen_test]
async fn play_skips_animation_under_ancestor_independent_scale_property() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let ancestor = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    ancestor.set_id("ancestor-independent-scale-property");
    ancestor
        .dyn_ref::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement")
        .style()
        .set_property("scale", "2")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&ancestor)
        .expect("append_child must not fail for a detached div");

    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "20px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "20px")
        .expect("set_property must not fail");
    ancestor
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    let delta = flip::FlipDelta {
        tx: 10.0,
        ty: 5.0,
        sx: 1.0,
        sy: 1.0,
    };
    let original = flip::OriginalStyle::capture(&div);
    let spring_config = SpringConfig::default();
    let anim_loop = flip::play(div.clone(), delta, spring_config, original);

    assert!(
        anim_loop.is_done(),
        "祖先が独立した scale プロパティを持つ場合、transform プロパティ \
         だけを読む累積行列ではその効果を捕捉できないため再生を省略し \
         即座に収束済みとなるはず"
    );
    let transform_after_skip = div
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_eq!(
        transform_after_skip, "",
        "再生を省略した場合、元々 transform 未設定だった要素の transform \
         は書き込まれないままのはず: {transform_after_skip}"
    );

    div.remove();
    ancestor.remove();
}

/// 受け入れ条件（codex-review 第 8 ラウンド是正、イシュー #2518。
/// 契約絞り込み）: 祖先が CSS `zoom`（恒等以外の値）を持つ場合、`play`
/// は再生自体を省略する（`ancestor_linear_matrix` が `None` を返す
/// 経路、`is_non_identity_zoom` の rustdoc 参照）。並べ替え相当の delta
/// を渡しても補正 transform は一切書かれず、要素は即座に Last 状態
/// （元々 transform 未設定なら未設定のまま）となる。
#[wasm_bindgen_test]
async fn play_skips_animation_under_ancestor_zoom() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let ancestor = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    ancestor.set_id("ancestor-zoom");
    ancestor
        .dyn_ref::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement")
        .style()
        .set_property("zoom", "2")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&ancestor)
        .expect("append_child must not fail for a detached div");

    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "20px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "20px")
        .expect("set_property must not fail");
    ancestor
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    // 並べ替えで行が CSS 座標上 100px 移動した状況を模した delta
    // （祖先 `zoom: 2` 下で誤って単位行列のまま扱うと、画面上 200px の
    // 補正 `translate(-200px)` が書かれてしまう不具合の再現条件）。
    let delta = flip::FlipDelta {
        tx: 100.0,
        ty: 0.0,
        sx: 1.0,
        sy: 1.0,
    };
    let original = flip::OriginalStyle::capture(&div);
    let spring_config = SpringConfig::default();
    let anim_loop = flip::play(div.clone(), delta, spring_config, original);

    assert!(
        anim_loop.is_done(),
        "祖先が zoom を持つ場合、transform プロパティだけを読む累積行列 \
         ではその拡縮を捕捉できないため再生を省略し即座に収束済みと \
         なるはず"
    );
    let transform_after_skip = div
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_eq!(
        transform_after_skip, "",
        "再生を省略した場合、元々 transform 未設定だった要素の transform \
         は書き込まれないまま（過大な補正 translate が書かれない）はず: \
         {transform_after_skip}"
    );

    div.remove();
    ancestor.remove();
}

/// 受け入れ条件（codex-review 第 9 ラウンド是正、イシュー #2518。P1
/// 「transform-box の参照原点を補正に反映する」）: 要素が
/// `transform-box: content-box; padding: 20px`（既定の `border-box`
/// ではない基準ボックス）を持つ場合でも、`play` が書き込む
/// `transform-origin: 0 0` は border-box 左上を指すよう
/// `transform-box: border-box !important` を強制するため、幅の変化を
/// 伴う並べ替え（FLIP のスケール成分が非 1）でも Invert 直後の視覚矩形
/// は First と一致する（`transform-box` を強制しない場合、スケールの
/// 基準点が content-box 左上へずれ、padding 分だけ表示位置がずれる）。
#[wasm_bindgen_test]
async fn play_reproduces_first_left_edge_under_content_box_transform_box_on_resize() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("left", "0px")
        .expect("set_property must not fail");
    div.style()
        .set_property("top", "0px")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "100px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("padding", "20px")
        .expect("set_property must not fail");
    div.style()
        .set_property("transform-box", "content-box")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    let first_visual: Rect = flip::measure(&div);

    // 位置は変えず、幅のみ変化させる（FLIP のスケール成分を非 1 にし、
    // transform-origin の基準ボックスのずれを顕在化させる）。
    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    let last_visual: Rect = flip::measure(&div);
    let last_layout: Rect = flip::measure_clearing_transform(&div);
    let delta = fandhe_frontend_animation::flip::invert(first_visual, last_visual)
        .expect("同一要素の連続計測は常に有効な Rect のため Some");
    let corrected = flip::anchor_correct(delta, last_visual, last_layout);

    let original = flip::OriginalStyle::capture(&div);
    let spring_config = SpringConfig::default();
    let anim_loop = flip::play(div.clone(), corrected, spring_config, original);

    let rect_during_play = div.get_bounding_client_rect();
    assert!(
        (rect_during_play.x() - first_visual.x).abs() < 1.0,
        "Invert 直後の左端は First の左端 {} と一致するはずが（transform-box \
         の参照原点が border-box に強制されていない場合、padding 分だけ \
         ずれる）: {}",
        first_visual.x,
        rect_during_play.x()
    );
    assert!(
        (rect_during_play.width() - first_visual.width).abs() < 1.0,
        "Invert 直後の幅は First の幅 {} と一致するはずが: {}",
        first_visual.width,
        rect_during_play.width()
    );

    anim_loop.stop();
    div.remove();
}

/// 受け入れ条件（Bugbot 指摘対応、codex-review 第 10 ラウンド、イシュー
/// #2518。「own-transform origin ignores forced box」）: 要素自身が
/// `transform-box: content-box; transform: scale(2); transform-origin:
/// 0 0`（border-box とは異なる基準ボックス）を持つ場合でも、
/// `OriginalStyle::capture` が `resolved_origin_px` を `transform-box:
/// border-box` 強制**後**に読むため、Invert 直後の視覚矩形は First と
/// 一致する（強制前に読むと、要素自身の変形の見かけの原点が
/// padding/border 分だけずれる）。
#[wasm_bindgen_test]
async fn play_reproduces_first_rect_under_own_scale_with_content_box_transform_box() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    div.style()
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    div.style()
        .set_property("left", "0px")
        .expect("set_property must not fail");
    div.style()
        .set_property("top", "0px")
        .expect("set_property must not fail");
    div.style()
        .set_property("width", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("height", "50px")
        .expect("set_property must not fail");
    div.style()
        .set_property("padding", "20px")
        .expect("set_property must not fail");
    div.style()
        .set_property("transform-box", "content-box")
        .expect("set_property must not fail");
    div.style()
        .set_property("transform", "scale(2)")
        .expect("set_property must not fail");
    div.style()
        .set_property("transform-origin", "0 0")
        .expect("set_property must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");

    let first_visual: Rect = flip::measure(&div);

    // 位置のみ変化させる（要素自身の変形はそのまま）。
    div.style()
        .set_property("left", "50px")
        .expect("set_property must not fail");
    let last_visual: Rect = flip::measure(&div);
    let delta = fandhe_frontend_animation::flip::invert(first_visual, last_visual)
        .expect("同一要素の連続計測は常に有効な Rect のため Some");

    let original = flip::OriginalStyle::capture(&div);
    let spring_config = SpringConfig::default();
    let anim_loop = flip::play(div.clone(), delta, spring_config, original);

    let rect_during_play = div.get_bounding_client_rect();
    assert!(
        (rect_during_play.x() - first_visual.x).abs() < 1.0,
        "Invert 直後の左端は First の左端 {} と一致するはずが（要素自身の \
         transform-box が border-box 強制後に解決されていない場合、\
         padding 分だけ見かけの原点がずれる）: {}",
        first_visual.x,
        rect_during_play.x()
    );
    assert!(
        (rect_during_play.y() - first_visual.y).abs() < 1.0,
        "Invert 直後の上端は First の上端 {} と一致するはずが: {}",
        first_visual.y,
        rect_during_play.y()
    );

    anim_loop.stop();
    div.remove();
}
