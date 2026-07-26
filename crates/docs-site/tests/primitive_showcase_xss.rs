//! `crate::primitive_showcase`（イシュー #1022）の XSS 回帰テスト（受け入れ
//! 条件 4）。
//!
//! headless-ui のパート関数（`label`/`value`/`id` 等の動的引数）へ XSS
//! ペイロードを流し込み、以下 2 経路の両方でエスケープが保たれることを
//! 固定する。
//!
//! 1. デモ木自体の描画（`render()` の既定エスケープ、REQ-1）
//! 2. `component_page::api_reference_section` が機械導出する
//!    `Data Attributes` 表のセル（`text()` 経由の新しいシンク。属性値 →
//!    表セルという経路は `collect_data_attrs_from_tree` が本イシューで
//!    実際に使われるようになって初めて到達可能になった）
//!
//! 実際の `crate::primitive_showcase` の本番デモには危険な文字列を
//! 混入させない（`security.md` 秘密情報混入防止と同じ精神：テスト専用の
//! 合成フィクスチャでのみペイロードを扱う）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_docs_site::component_page::{render_component_page, ComponentPageSpec, Layer};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui as hui;

const SCRIPT_PAYLOAD: &str = "<script>alert(1)</script>";
const ATTR_BREAKOUT_PAYLOAD: &str = "\"><img src=x onerror=alert(1)>";

/// ペイロードを含む合成 checkbox-group デモ。`value`（`data-value` 属性、
/// 動的値）と children テキストの両方へペイロードを注入する。
fn synthetic_checkbox_group_demo(payload: &str) -> fandhe_frontend_core::Node {
    use hui::checkbox_group;
    checkbox_group::root(
        false,
        None,
        None,
        vec![],
        vec![checkbox_group::item(
            true,
            false,
            payload,
            vec![],
            vec![
                checkbox_group::item_control(
                    true,
                    false,
                    vec![],
                    vec![checkbox_group::item_indicator(true, false, vec![], vec![])],
                ),
                checkbox_group::item_text(true, false, vec![], vec![text(payload)]),
            ],
        )],
    )
}

/// `payload` 自体が実際にマークアップ境界を破ったかどうかを判定する。
/// エスケープ済み表現（`&lt;`/`&gt;`/`&quot;`）は `<`/`>`/`"` へ戻さない
/// 限り HTML タグ・属性を構成できないため、"onerror=alert" のような
/// **無害化された地の文としての残存**は許容し、実際に新しいタグ・属性が
/// 開始される形（生の `<script>`/`<img`）のみを不合格にする
/// （素朴な部分文字列一致だと誤検知するため、`<` を伴う形のみを見る）。
fn assert_no_raw_payload(html: &str, payload: &str, context: &str) {
    assert!(
        !html.contains(payload),
        "{context}: 生のペイロード全体がエスケープされずに出現した: {html}"
    );
    assert!(
        !html.contains("<script>"),
        "{context}: 生の <script> 開始タグが出現した: {html}"
    );
    assert!(
        !html.contains("<img "),
        "{context}: 生の <img 開始タグが出現した（属性値からの脱出）: {html}"
    );
}

#[test]
fn demo_tree_escapes_script_payload_in_label_and_children() {
    let demo = synthetic_checkbox_group_demo(SCRIPT_PAYLOAD);
    let page = render_component_page(
        "/primitives/checkbox-group/",
        demo,
        &ComponentPageSpec::EMPTY,
        Layer::Primitives,
    );
    let html = render(&page);

    assert_no_raw_payload(
        &html,
        SCRIPT_PAYLOAD,
        "checkbox-group demo (script payload)",
    );
    assert!(
        html.contains("&lt;script&gt;"),
        "エスケープ済みペイロードが出現しない（既定エスケープ自体が働いていない可能性）: {html}"
    );
}

#[test]
fn demo_tree_escapes_attribute_breakout_payload_in_data_value() {
    let demo = synthetic_checkbox_group_demo(ATTR_BREAKOUT_PAYLOAD);
    let page = render_component_page(
        "/primitives/checkbox-group/",
        demo,
        &ComponentPageSpec::EMPTY,
        Layer::Primitives,
    );
    let html = render(&page);

    assert_no_raw_payload(
        &html,
        ATTR_BREAKOUT_PAYLOAD,
        "checkbox-group demo (attribute breakout payload)",
    );
    // 属性値中の `"` は `&quot;` へエスケープされ、`data-value` 属性の
    // 境界を抜け出せない。
    assert!(
        !html.contains(r#"data-value="">"#),
        "属性値の `\"` エスケープが破られ、data-value 属性から脱出できている: {html}"
    );
}

/// 受け入れ条件 4 の中核: `Data Attributes` 表（`api_reference_section` が
/// `collect_data_attrs_from_tree` から機械導出する新しいシンク）のセル内でも
/// ペイロードがエスケープされることを固定する。
#[test]
fn data_attributes_table_cell_escapes_payload_from_observed_data_value() {
    let demo = synthetic_checkbox_group_demo(ATTR_BREAKOUT_PAYLOAD);
    let page = render_component_page(
        "/primitives/checkbox-group/",
        demo,
        &ComponentPageSpec::EMPTY,
        Layer::Primitives,
    );
    let html = render(&page);

    // `Data Attributes` 表が実際に生成されていることを前提として確認する
    // （空虚な成功を避ける。表が無ければこの回帰テストは何も検証していない
    // ことになる）。
    assert!(
        html.contains("Data Attributes"),
        "Data Attributes 表が生成されていない（前提が崩れている）: {html}"
    );
    assert_no_raw_payload(&html, ATTR_BREAKOUT_PAYLOAD, "Data Attributes table cell");
}
