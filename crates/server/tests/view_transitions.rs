//! TASK-8.1（イシュー #59）回帰テスト: SSR ページ遷移向け
//! `@view-transition` at-rule の既定同梱を製品仕様として固定する。
//!
//! REQ-8（View Transitions API のネイティブ活用）のうちクロスドキュメント
//! ナビゲーション側の受け入れ基準「宣言 1 行・JS 0 行で全ルートに有効化
//! されること」を、`fandhe_frontend_app::page_shell()` を経由する SSR（[`respond`]）・
//! SSG（[`generate`]）全ルートについて固定する。
//!
//! `page_shell()` 自体の単体テスト（`app/src/lib.rs` の
//! `page_shell_includes_view_transition_at_rule_and_matches_across_ssr_and_ssg`）
//! は「関数の戻り値」を検証済みだが、本ファイルは一段上の観点として
//! 「fandhe-frontend-server が実際に応答する全ルート（`respond("/")` ・
//! `demo_items()` 全 id の詳細ページ・未知 id の 404 ページ）」と
//! 「SSG が書き出す全ファイル」の双方で既定同梱が崩れていないことを、
//! `server/tests/ssr_ssg_parity.rs` と同じ流儀（`support/temp_dir.rs` の
//! `include!` 再利用、外部 HTML パーサ非追加）で回帰固定する。
//!
//! # セキュリティ不変条件
//!
//! 追加する at-rule はユーザー入力を一切含まない固定リテラルであり、
//! `page_shell()` は `el`/`text`（既定エスケープ経路）経由でこれを出力する
//! （REQ-1 非弱体化）。本ファイルは「廃止済みの
//! `<meta name="view-transition" content="same-origin">` 構文が再導入
//! されていないこと」も併せて固定し、仕様書（`docs/spec/`）記載の旧構文への
//! 先祖返りを検知する。

use fandhe_frontend_app::demo_items;
use fandhe_frontend_server::ssg::generate;
use fandhe_frontend_server::ssr::respond;
use std::fs;

include!("support/temp_dir.rs");

/// View Transitions Level 2 で標準化された at-rule。`app/src/lib.rs` の
/// `page_shell()` が出力するリテラルと完全一致することを前提にした定数
/// （this と `page_shell()` 側の実装が乖離した場合、本ファイルの各テストが
/// 失敗して検知する）。
const VIEW_TRANSITION_AT_RULE: &str = "<style>@view-transition { navigation: auto; }</style>";

/// 廃止済みの旧実験構文。View Transitions Level 2 の標準化過程で
/// `<meta name="view-transition" content="same-origin">` は廃止されており、
/// `page_shell()` は at-rule へ置換済み（`docs/guides/view-transitions.md` 参照）。
const DEPRECATED_VIEW_TRANSITION_META: &str = r#"<meta name="view-transition""#;

/// テスト観点 1: SSR のトップページ（`respond("/")`）に at-rule が
/// 既定同梱され、廃止済み meta タグを含まないこと。
#[test]
fn ssr_index_includes_view_transition_at_rule() {
    let body = respond("/").expect("\"/\" should match").body;
    assert!(
        body.contains(VIEW_TRANSITION_AT_RULE),
        "SSR トップページに @view-transition at-rule が既定同梱されていない"
    );
    assert!(
        !body.contains(DEPRECATED_VIEW_TRANSITION_META),
        "廃止済みの <meta name=\"view-transition\"> が SSR トップページに含まれている"
    );
}

/// テスト観点 2: `demo_items()` 全件の SSR 詳細ページ（`/items/{id}`）に
/// at-rule が既定同梱されること。ルートが増減しても `demo_items()` を
/// 起点に走査するため追従する。
#[test]
fn ssr_all_item_detail_pages_include_view_transition_at_rule() {
    for item in demo_items() {
        let body = respond(&format!("/items/{}", item.id))
            .unwrap_or_else(|| panic!("item detail route should match for id={}", item.id))
            .body;
        assert!(
            body.contains(VIEW_TRANSITION_AT_RULE),
            "id={} の SSR 詳細ページに @view-transition at-rule が見つからない",
            item.id
        );
        assert!(
            !body.contains(DEPRECATED_VIEW_TRANSITION_META),
            "id={} の SSR 詳細ページに廃止済み meta タグが含まれている",
            item.id
        );
    }
}

/// テスト観点 3: 未知 id の 404 応答にも at-rule が既定同梱されること
/// （`page_shell()` はエラーページ生成でも分岐なく呼ばれる契約）。
#[test]
fn ssr_404_page_includes_view_transition_at_rule() {
    let response = respond("/items/does-not-exist").expect("pattern should still match");
    assert_eq!(response.status, 404);
    assert!(
        response.body.contains(VIEW_TRANSITION_AT_RULE),
        "404 ページに @view-transition at-rule が既定同梱されていない"
    );
    assert!(!response.body.contains(DEPRECATED_VIEW_TRANSITION_META));
}

/// テスト観点 4: SSG が書き出す全ファイル（`generate()` の戻り値）にも
/// 同一の at-rule が含まれ、SSR とモード間で既定同梱の契約が食い違わないこと。
#[test]
fn ssg_generated_files_include_view_transition_at_rule() {
    let dir = TempDir::new("view-transitions-ssg");
    let written = generate(&dir.0).expect("generate should succeed");
    assert!(
        !written.is_empty(),
        "少なくとも 1 ファイルは書き出されるはず"
    );

    for file_path in &written {
        let contents = fs::read_to_string(file_path)
            .unwrap_or_else(|_| panic!("{file_path:?} should be readable as UTF-8"));
        assert!(
            contents.contains(VIEW_TRANSITION_AT_RULE),
            "{file_path:?} に @view-transition at-rule が見つからない"
        );
        assert!(
            !contents.contains(DEPRECATED_VIEW_TRANSITION_META),
            "{file_path:?} に廃止済み meta タグが含まれている"
        );
    }
}

/// テスト観点 5: SSR と SSG のトップページが at-rule を含めてバイト一致し、
/// クロスドキュメント遷移の既定同梱が両モード間で分岐していないこと
/// （`ssr_ssg_parity.rs` の一般的なバイト一致固定に対し、本テストは
/// at-rule の存在それ自体をピンポイントで再確認する意図）。
#[test]
fn ssr_and_ssg_index_bodies_match_including_view_transition_at_rule() {
    let dir = TempDir::new("view-transitions-parity");
    generate(&dir.0).expect("generate should succeed");

    let ssr_body = respond("/").expect("\"/\" should match").body;
    let ssg_body = fs::read_to_string(dir.0.join("index.html")).expect("index.html should exist");

    assert_eq!(ssr_body, ssg_body);
    assert!(ssr_body.contains(VIEW_TRANSITION_AT_RULE));
}
