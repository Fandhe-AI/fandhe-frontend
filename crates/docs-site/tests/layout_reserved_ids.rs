//! `crate::layout::RESERVED_LAYOUT_IDS`（イシュー #950 のセキュリティ監査
//! Low 指摘是正、本ファイルが対応する回帰テスト）の双方向乖離検知。
//!
//! # 背景
//!
//! `layout::with_heading_anchors` は本文見出しから自動生成する slug が
//! レイアウト自身の固定 `id`（`label[for]`/`aria-controls` の関連付け先）と
//! 衝突しないよう、走査前にそれらを予約する。しかし `TOC_HEADING_ID`
//! （右目次見出し、イシュー #950）のみを予約する実装だったため、
//! `docs-search-input`/`docs-search-results`/`docs-sidebar-toggle` の 3 件は
//! 予約対象外のまま放置されており、`site/**.md` に「Docs search input」の
//! ような見出しがあると `id` 重複（HTML 仕様違反、関連付け破壊）が
//! 発生し得た。`layout::RESERVED_LAYOUT_IDS` は今後この 4 件（および将来
//! 追加される固定 `id`）を単一の情報源として管理する契約であり、本ファイルは
//! `crates/docs-site/tests/site_css_contract.rs` の層 1（class 名の双方向
//! fail-closed 契約）と同型のやり方で `id` 版の双方向契約を固定する。
//!
//! # SkipNav の `id` も予約対象に含む
//!
//! `docs_page`/`docs_page_with_assets` は `layout.rs` が定義する固定 `id` 群とは
//! 別に `fandhe_frontend_pre_styled_ui::skip_nav::DEFAULT_ID`
//! （`"fandhe-skip-nav"`）も常時出力する。値の定義は
//! `crates/headless-ui/src/skip_nav.rs` にあり別クレートの所有だが、
//! **衝突回避の観点では値の所有者が誰かは無関係**（本ページが実際に出力する
//! 固定 `id` である以上、本文見出しの slug と衝突し得る）ため
//! [`RESERVED_LAYOUT_IDS`] へ含めている。予約しない場合、本文見出しが
//! `"fandhe-skip-nav"` へ slug 化されると SkipNav リンクの `href="#..."` が
//! 本文冒頭ではなくその見出しへ飛ぶ回帰が起こる。
//!
//! (b) 方向の検証では `SKIP_NAV_ID` を許容集合へ個別追加しない。追加すると
//! [`RESERVED_LAYOUT_IDS`] から SkipNav の `id` が外れて予約の穴が再発しても
//! テストが通ってしまい、fail-closed が崩れるため。代わりに
//! [`reserved_layout_ids_contains_skip_nav_id`] が包含関係を直接固定する。

use std::collections::HashSet;

use fandhe_frontend_core::{h2, p, render, text};
use fandhe_frontend_docs_site::layout::{
    docs_page, with_heading_anchors, RESERVED_LAYOUT_IDS, SEARCH_INPUT_ID,
};
use fandhe_frontend_pre_styled_ui::skip_nav::DEFAULT_ID as SKIP_NAV_ID;

fn sample_sidebar() -> fandhe_frontend_core::Node {
    fandhe_frontend_core::ul(
        vec![],
        vec![fandhe_frontend_core::li(vec![], vec![text("はじめに")])],
    )
}

/// html 文字列中の全 `id="..."` 属性値を収集する。属性の直前には必ず
/// 空白が入る（`fandhe_frontend_core::render` の属性シリアライズ規則。
/// `layout_render.rs`/`site_css_contract.rs` の class 抽出ヘルパと同型の
/// 前提）ため、単なる `"id=\""` ではなく先頭に空白を含む `" id=\""` を
/// マーカーに使う。空白を付けない場合 `aria-invalid="..."` のような
/// 属性名の末尾（`...alid="`）が偶然 `id="` を部分文字列として含み誤検出
/// する（`invalid` は末尾 2 文字が `i`/`d` のため）。
fn extract_id_attrs(html: &str) -> HashSet<String> {
    let mut ids = HashSet::new();
    let marker = " id=\"";
    let mut rest = html;
    while let Some(start) = rest.find(marker) {
        let after = &rest[start + marker.len()..];
        let Some(end) = after.find('"') else { break };
        ids.insert(after[..end].to_string());
        rest = &after[end + 1..];
    }
    ids
}

/// (a) 方向: [`RESERVED_LAYOUT_IDS`] の各 id が、見出しを含むページの
/// レンダリング結果に `id="<値>"` として出現する（定数に幽霊エントリが
/// 残らないことの固定）。右目次見出し（`TOC_HEADING_ID`）は見出しが
/// 1 件も無いページでは `toc_nav` が `None` を返し出力されないため、
/// 本テストは見出し入りのフィクスチャを使う。
#[test]
fn all_reserved_layout_ids_appear_in_rendered_html_with_headings() {
    let body = fandhe_frontend_core::div(vec![], vec![h2(vec![], vec![text("導入")])]);
    let html = render(&docs_page("タイトル", "", sample_sidebar(), body));

    for id in RESERVED_LAYOUT_IDS {
        assert!(
            html.contains(&format!(r#"id="{id}""#)),
            "RESERVED_LAYOUT_IDS の {id} がレンダリング結果に出現しない（幽霊エントリの疑い）"
        );
    }
}

/// (b) 方向: 見出しを一切含まない本文でレンダリングすると、出現する
/// `id="..."` はすべてレイアウト固定 id（[`RESERVED_LAYOUT_IDS`] +
/// スコープ外の SkipNav id）の部分集合になる。本文に見出しが無いため、
/// 本文走査由来の自動生成 slug は 1 件も混入しない（モジュール doc の
/// 前提どおり、出現するすべての id を「レイアウトが出す固定 id」と
/// みなせる）。`layout.rs`/`nav.rs` が新しい固定 id を無断で追加し
/// [`RESERVED_LAYOUT_IDS`] への追記を忘れた場合、本テストが検知する。
///
/// 部分集合検証にとどめる（完全一致にしない）理由: 見出しが無いページでは
/// `toc_nav` が `None` を返すため [`RESERVED_LAYOUT_IDS`] のうち
/// `TOC_HEADING_ID` は実際には出力されない（[`toc_nav`] の既存契約、
/// `layout_render.rs::no_headings_means_no_toc_nav_and_no_toc_section_in_document`
/// 参照）。「全件出現」の確認は見出しありフィクスチャを使う
/// [`all_reserved_layout_ids_appear_in_rendered_html_with_headings`] の役割。
#[test]
fn rendered_html_without_headings_has_no_id_outside_reserved_layout_ids() {
    let body = p(vec![], vec![text("見出しの無い本文です。")]);
    let html = render(&docs_page("タイトル", "", sample_sidebar(), body));

    let allowed: HashSet<String> = RESERVED_LAYOUT_IDS.iter().map(|s| s.to_string()).collect();

    let actual = extract_id_attrs(&html);
    let unexpected: Vec<&String> = actual.iter().filter(|id| !allowed.contains(*id)).collect();
    assert!(
        unexpected.is_empty(),
        "見出し無しページに RESERVED_LAYOUT_IDS に無い id が出現した: {unexpected:?}"
    );
}

/// SkipNav の `id` が [`RESERVED_LAYOUT_IDS`] に含まれ続けることを固定する。
///
/// 値の定義元は別クレート（`fandhe-frontend-headless-ui`）だが、本ページが
/// 実際に出力する固定 `id` である以上、本文見出しの自動生成 slug との衝突
/// 回避のために予約が必要（モジュール doc 参照）。
/// [`rendered_html_without_headings_has_no_id_outside_reserved_layout_ids`] は
/// 許容集合を [`RESERVED_LAYOUT_IDS`] のみから作るため、本アサーションが
/// 落ちるときは同時にそちらも落ちる（二重の fail-closed）。
#[test]
fn reserved_layout_ids_contains_skip_nav_id() {
    assert!(
        RESERVED_LAYOUT_IDS.contains(&SKIP_NAV_ID),
        "SkipNav の id（{SKIP_NAV_ID}）が RESERVED_LAYOUT_IDS から外れている。\
         本文見出しが同じ slug を生成すると SkipNav の href が見出しへ飛ぶ回帰が起こる"
    );
}

/// 回帰テスト: 本文見出しが `docs-search-input` へ slug 化される
/// テキスト（"Docs search input"）を含む場合でも、レイアウトの検索
/// 入力（本物の `id="docs-search-input"`）は 1 個のみ出力され、見出し側は
/// `with_heading_anchors` の既存衝突回避分岐で `docs-search-input-2` へ
/// 退避される（[`RESERVED_LAYOUT_IDS`] 全件予約の直接的な効果）。
#[test]
fn heading_colliding_with_search_input_id_is_reassigned_and_id_stays_unique() {
    let body = fandhe_frontend_core::div(vec![], vec![h2(vec![], vec![text("Docs search input")])]);

    let (_, entries) = with_heading_anchors(body.clone());
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].id, format!("{SEARCH_INPUT_ID}-2"));
    assert_ne!(entries[0].id, SEARCH_INPUT_ID);

    let html = render(&docs_page("タイトル", "", sample_sidebar(), body));
    let expected_marker = format!(r#"id="{SEARCH_INPUT_ID}""#);
    assert_eq!(
        html.matches(&expected_marker).count(),
        1,
        "id=\"{SEARCH_INPUT_ID}\" はレイアウトの検索入力のみが持つべきで、\
         見出し由来の重複が発生してはならない"
    );
    let collided_marker = format!(r#"id="{SEARCH_INPUT_ID}-2""#);
    assert_eq!(
        html.matches(&collided_marker).count(),
        1,
        "衝突した見出し側は id=\"{SEARCH_INPUT_ID}-2\" へ一意化される必要がある"
    );
}

/// [`extract_id_attrs`] の自己テスト: `aria-invalid="..."` のような
/// 「末尾が `id=\"`」に見える属性を `id` 属性と誤検出しないことを固定する
/// （モジュール doc の抽出マーカー選定理由の裏付け）。
#[test]
fn extract_id_attrs_does_not_misdetect_attributes_ending_in_id() {
    let html = r#"<input aria-invalid="true" id="real-id">"#;
    let ids = extract_id_attrs(html);
    assert_eq!(ids, HashSet::from(["real-id".to_string()]));
}
