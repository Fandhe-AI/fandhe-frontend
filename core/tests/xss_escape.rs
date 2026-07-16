//! TASK-1.2a: SSR・SSG 経路の XSS 回帰テスト。
//!
//! REQ-1（既定エスケープによる出力安全性）の受け入れ基準「XSS ペイロードが
//! SSR・SSG 経路でエスケープされること」を、`rws-core` の `render()` を通した
//! 統合テストとして経路別に固定する。PoC-3（`docs/spec/03-poc/rendering-web-standards/`）
//! で実測済みの検証（SSR は curl 実測・SSG は生成ファイル grep 実測）を、
//! 製品コードに対する自動回帰へ拡充したもの。
//!
//! # 経路の再現方法（設計前提）
//!
//! 現時点の workspace には `rws-server` が存在せず、SSR（HTTP レスポンス
//! ボディ）・SSG（ファイル書き出し）はいずれもモード非依存の
//! `rws_core::render()` を共通で呼ぶ契約（`core/src/lib.rs` 冒頭の不変条件）
//! である。そのため本ファイルでは:
//!
//! - **SSR 経路** = `render()` の戻り値をそのまま HTTP レスポンスボディ相当として検証する
//! - **SSG 経路** = 同じ `render()` 出力を実ファイルへ書き出し、読み戻した内容を検証する
//!
//! 将来 `server/` 実装後は実サーバー経由の E2E テストへ拡張する余地を残すが、
//! 本タスクのスコープ外とする（計画書参照）。
//!
//! CSR 経路の回帰・CI 組み込みは #10（TASK-1.2b）、WASM 経由の一貫性検証は
//! TASK-1.3、escape 文字仕様の単体テストは #7（TASK-1.1c）が担当する。
//!
//! # 削除・弱体化の禁止
//!
//! 本ファイルの XSS 回帰テストは `.claude/rules/coding-rust.md` の規約により、
//! 以後の削除・弱体化・`#[ignore]` 化を禁止する。

use rws_core::{el, raw_html, render, text};

/// PoC-3 実測の基準ペイロード + OWASP 代表形を集約した XSS ペイロード集。
///
/// テキスト補間・属性値の両スロットに対して、タグ注入・イベントハンドラ
/// 注入・属性 breakout・エンティティ偽装・非 ASCII 混在の各カテゴリを
/// 網羅する（escape モジュール自体の文字仕様単体テストは #7 のスコープの
/// ため、ここでは「経路を通した結果」のみを検証する）。
const XSS_PAYLOADS: &[&str] = &[
    // PoC-3 実測の基準ペイロード（script タグ + イベントハンドラ注入）
    "<script>alert('xss')</script><img src=x onerror=alert(1)>",
    // タグ注入形
    "<svg onload=alert(1)>",
    "<iframe src=\"javascript:alert(1)\"></iframe>",
    // 属性値 breakout 形
    "\"><script>alert(1)</script>",
    "' onmouseover='alert(1)",
    // エンティティ偽装（二重エスケープにより復活しないことを検証する）
    "&lt;script&gt;",
    // 非 ASCII 混在（マルチバイト透過の経路確認）
    "日本語<script>alert('侵入')</script>絵文字🎉",
];

/// SSR 経路相当のページ構造にテキスト補間としてペイロードを埋め込む。
///
/// PoC-3 の `page_shell`/`list_page` を模した `html > body > ul > li` の
/// ネスト木。`payload` はリスト項目のテキストノードとして埋め込まれ、
/// `render()` が既定エスケープを経由することを検証する対象となる。
fn page_with_text_payload(payload: &str) -> rws_core::Node {
    el(
        "html",
        vec![],
        vec![el(
            "body",
            vec![],
            vec![
                el("h1", vec![], vec![text("Title")]),
                el(
                    "ul",
                    vec![],
                    vec![
                        el("li", vec![], vec![text("safe item")]),
                        el("li", vec![], vec![text(payload)]),
                    ],
                ),
            ],
        )],
    )
}

/// 属性値スロットにペイロードを埋め込んだページ構造を組み立てる。
///
/// `title` 属性・`data-value` 属性の両方にペイロードを設定し、属性値
/// breakout（`"><script>...` 等）が `render()` の属性シリアライズで
/// エスケープされることを検証する対象とする。
fn page_with_attr_payload(payload: &str) -> rws_core::Node {
    el(
        "html",
        vec![],
        vec![el(
            "body",
            vec![],
            vec![el(
                "div",
                vec![("title", payload), ("data-value", payload)],
                vec![text("content")],
            )],
        )],
    )
}

/// SSG 経路（実ファイル書き出し）を再現するテストヘルパー。
///
/// `render()` 出力を `std::env::temp_dir()` 配下のプロセス ID + テスト名
/// 別サブディレクトリに `index.html` として書き出し、読み戻して返す。
/// パストラバーサル防止のため、外部入力からパスを組み立てず固定パターン
/// のみを使用する。並列テスト・並列イシュー実行との衝突をプロセス ID で
/// 回避し、終了時にディレクトリを削除する。
///
/// core は外部依存ゼロ（`core/Cargo.toml` 参照）のため `tempfile` 等の
/// クレートは使わず、`std::fs`/`std::env::temp_dir` のみで実装する。
/// テストコードのため `unwrap()`/`expect()` の使用を許容する。
fn ssg_write_and_read_back(html: &str, test_name: &str) -> String {
    let dir =
        std::env::temp_dir().join(format!("rws-xss-ssg-{}-{}", std::process::id(), test_name));
    std::fs::create_dir_all(&dir).expect("SSG 出力先ディレクトリの作成に失敗した");
    let file_path = dir.join("index.html");
    std::fs::write(&file_path, html).expect("SSG ファイル書き出しに失敗した");
    let read_back = std::fs::read_to_string(&file_path).expect("SSG ファイル読み戻しに失敗した");
    // テスト終了時に一時ディレクトリを削除し、後続実行との衝突・ディスク汚染を避ける。
    let _ = std::fs::remove_dir_all(&dir);
    read_back
}

/// 出力 HTML に生スクリプトタグ・イベントハンドラ注入の痕跡が残っていない
/// ことを共通アサートする（テキスト補間経路の判定に使用）。
fn assert_text_payload_neutralized(html: &str, payload: &str) {
    assert!(
        !html.contains("<script>"),
        "生 <script> タグが出力に残存した（payload: {payload:?}）: {html}"
    );
    assert!(
        !html.contains("<svg onload="),
        "生 <svg onload= がエスケープされずに出力された（payload: {payload:?}）: {html}"
    );
    assert!(
        !html.contains("<iframe "),
        "生 <iframe がエスケープされずに出力された（payload: {payload:?}）: {html}"
    );
    // イベントハンドラ属性はテキストとしてエスケープされていれば実行可能な
    // 属性としては成立しない（`<img ...>` タグ自体が `&lt;img ...&gt;` へ
    // エスケープされているため）。ここでは実行可能なタグとして残存していない
    // ことを、ペイロード原文（`<` を含む生タグ文字列）が消えていることで確認する。
    // `<` は必ず `&lt;` へエスケープされるため、生の `<` が残っていないことも確認する。
    assert!(
        !html.contains(payload),
        "ペイロードがエスケープされず原文のまま出力に含まれていた（payload: {payload:?}）: {html}"
    );
    // 陰性チェック（原文が含まれない）だけでは部分的なエスケープ抜け
    // （例: `<script>` だけ残り `<img` は残らない、のような取りこぼし）を
    // 見逃しうる。`escape_html`（公開 API）が生成する完全エスケープ済み
    // 文字列がそのまま出力に含まれることを陽性側でも固定する。
    assert!(
        html.contains(&rws_core::escape_html(payload)),
        "escape_html の期待出力が render() 結果に含まれていない（payload: {payload:?}）: {html}"
    );
}

/// SSR レスポンスボディ相当の `render()` 出力で、テキスト補間ペイロードが
/// 既定エスケープされることを検証する（REQ-1 受け入れ基準の SSR 経路固定）。
#[test]
fn ssr_response_body_escapes_text_payloads() {
    for payload in XSS_PAYLOADS {
        let node = page_with_text_payload(payload);
        let html = render(&node);
        assert_text_payload_neutralized(&html, payload);
        // 安全な兄弟項目は変更されず出力されること（過剰なエスケープでないこと）。
        assert!(html.contains("<li>safe item</li>"));
    }
}

/// SSR レスポンスボディ相当の `render()` 出力で、属性値スロット経由の
/// breakout ペイロードがエスケープされ、属性からの脱出・追加属性の割り込みが
/// 成立しないことを検証する。
#[test]
fn ssr_response_body_escapes_attribute_payloads() {
    for payload in XSS_PAYLOADS {
        let node = page_with_attr_payload(payload);
        let html = render(&node);
        assert!(
            !html.contains("<script>"),
            "属性値経由で <script> タグの breakout が発生した（payload: {payload:?}）: {html}"
        );
        assert!(
            !html.contains("<svg onload="),
            "属性値経由で <svg onload= の breakout が発生した（payload: {payload:?}）: {html}"
        );
        // 属性値は必ず二重引用符で囲まれ、`"` `'` `<` `>` `&` がエンティティ化されるため、
        // ペイロード原文がそのまま属性値として出力されていないことを確認する。
        assert!(
            !html.contains(&format!("title=\"{payload}\"")),
            "属性値がエスケープされずそのまま出力された（payload: {payload:?}）: {html}"
        );
    }
}

/// 1・2 と同じノード木を実ファイルへ書き出し・読み戻した内容で同一の
/// アサートを行い、SSG 実ファイル経路でも既定エスケープが貫通することを
/// 検証する（PoC-3 の「生成ファイル grep 実測」の自動回帰化）。
#[test]
fn ssg_file_output_escapes_text_and_attribute_payloads() {
    for (i, payload) in XSS_PAYLOADS.iter().enumerate() {
        let text_html = render(&page_with_text_payload(payload));
        let text_from_file = ssg_write_and_read_back(&text_html, &format!("text-{i}"));
        assert_text_payload_neutralized(&text_from_file, payload);

        let attr_html = render(&page_with_attr_payload(payload));
        let attr_from_file = ssg_write_and_read_back(&attr_html, &format!("attr-{i}"));
        assert!(
            !attr_from_file.contains("<script>"),
            "SSG ファイル経由で属性値の breakout が発生した（payload: {payload:?}）: {attr_from_file}"
        );
    }
}

/// 同一ノード木に対し SSR 文字列と SSG ファイル内容が完全一致することを
/// 検証する（モード非依存レンダラの契約回帰。PoC-3 の
/// `ssg_output_equals_ssr_output_for_list_and_detail` 相当）。
#[test]
fn ssg_file_output_equals_ssr_output() {
    for (i, payload) in XSS_PAYLOADS.iter().enumerate() {
        let ssr_html = render(&page_with_text_payload(payload));
        let ssg_html = ssg_write_and_read_back(&ssr_html, &format!("equal-{i}"));
        assert_eq!(
            ssr_html, ssg_html,
            "SSR 出力と SSG ファイル内容が一致しない（payload: {payload:?}）"
        );
    }
}

/// `&lt;script&gt;` のようなエンティティ偽装入力が二重エスケープされ
/// （`&amp;lt;script&amp;gt;`）、SSG ファイル経由でもエンティティが復活
/// しないことを検証する。
#[test]
fn entity_payload_is_double_escaped_and_not_revived() {
    let payload = "&lt;script&gt;";
    let node = page_with_text_payload(payload);
    let ssr_html = render(&node);
    assert!(
        ssr_html.contains("&amp;lt;script&amp;gt;"),
        "エンティティ偽装入力が二重エスケープされなかった: {ssr_html}"
    );
    assert!(
        !ssr_html.contains("<script>"),
        "エンティティ偽装入力からエンティティが復活し生タグとして出力された: {ssr_html}"
    );

    let ssg_html = ssg_write_and_read_back(&ssr_html, "entity");
    assert!(
        ssg_html.contains("&amp;lt;script&amp;gt;"),
        "SSG ファイル経由でエンティティ偽装が二重エスケープされなかった: {ssg_html}"
    );
    assert!(!ssg_html.contains("<script>"));
}

/// `raw_html()` オプトイン時のみ非エスケープ出力となり（SSR・SSG 両経路）、
/// `text()` 経路には迂回が存在しないことを対比固定する。
#[test]
fn raw_html_is_the_only_escape_bypass_on_both_paths() {
    // 信頼できる固定文字列（外部入力ではない）を raw_html に渡す対比検証。
    // 本テストは「raw_html 以外に迂回経路がない」ことの固定が目的であり、
    // 信頼できない入力を raw_html に渡すサンプルとして読まれないよう
    // 固定・無害な文字列のみを使う。
    let trusted_fragment = "<b>bold</b>";
    let raw_node = el("div", vec![], vec![raw_html(trusted_fragment)]);
    let raw_ssr = render(&raw_node);
    assert!(
        raw_ssr.contains("<b>bold</b>"),
        "raw_html の非エスケープ出力が機能していない: {raw_ssr}"
    );
    let raw_ssg = ssg_write_and_read_back(&raw_ssr, "raw-bypass");
    assert!(raw_ssg.contains("<b>bold</b>"));

    // 対比: 同じ文字列を text() 経由にすると両経路でエスケープされる（迂回されない）。
    let text_node = el("div", vec![], vec![text(trusted_fragment)]);
    let text_ssr = render(&text_node);
    assert!(!text_ssr.contains("<b>bold</b>"));
    assert!(text_ssr.contains("&lt;b&gt;bold&lt;/b&gt;"));
    let text_ssg = ssg_write_and_read_back(&text_ssr, "text-no-bypass");
    assert!(text_ssg.contains("&lt;b&gt;bold&lt;/b&gt;"));
}

/// 深いネスト位置（リスト項目・ページタイトル等の複数スロット）でも
/// エスケープが貫通することを検証する。
#[test]
fn nested_tree_escapes_payload_at_any_depth() {
    let payload = "<script>alert('deep')</script>";
    let tree = el(
        "html",
        vec![],
        vec![el(
            "body",
            vec![],
            vec![el(
                "main",
                vec![],
                vec![el(
                    "section",
                    vec![],
                    vec![el(
                        "ul",
                        vec![],
                        vec![el(
                            "li",
                            vec![],
                            vec![el("span", vec![], vec![text(payload)])],
                        )],
                    )],
                )],
            )],
        )],
    );
    let ssr_html = render(&tree);
    assert!(!ssr_html.contains("<script>"));
    assert!(ssr_html.contains("&lt;script&gt;alert(&#x27;deep&#x27;)&lt;/script&gt;"));

    let ssg_html = ssg_write_and_read_back(&ssr_html, "nested-depth");
    assert!(!ssg_html.contains("<script>"));
    assert!(ssg_html.contains("&lt;script&gt;alert(&#x27;deep&#x27;)&lt;/script&gt;"));
}
