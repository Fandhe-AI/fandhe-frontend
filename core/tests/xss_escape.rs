//! TASK-1.2 の 3 経路（SSR・SSG・CSR）XSS 回帰テスト（親タスク #8、REQ-1 の受け入れ基準）。
//!
//! 本ファイルは以下を提供する。
//!
//! - `mod payloads`: CSR 経路のテスト（`mod csr`）が使う共有ペイロード集合
//! - `mod csr`: TASK-1.2b（#10）が担当する CSR 経路のテスト
//! - トップレベルの `XSS_PAYLOADS` 定数・各 `#[test]` 関数:
//!   TASK-1.2a（#9）が担当する SSR・SSG 経路のテスト
//!
//! SSR・SSG 側は PoC-3（`docs/spec/03-poc/rendering-web-standards/`）の実測
//! （SSR は curl 実測・SSG は生成ファイル grep 実測）を先行実装し独自のペイロード
//! 定数を持つ。CSR 側は別途 OWASP 準拠の共有ペイロード集合（`mod payloads`）を
//! 持つ。両者は現時点で別の定数として存在するが、いずれも
//! `rws-core::render()` が経路非依存にエスケープを貫通させることを検証する
//! 目的は共通である。
//!
//! `rws-core::render()` は SSR・SSG・CSR のいずれのモードからも共通で
//! 呼ばれる（`core/src/lib.rs` 冒頭の不変条件を参照）。CSR 経路は
//! `rws-wasm-client` の `mount_csr()` 相当（未実装。TASK-1.3 で
//! `wasm-client/tests/xss_escape_wasm.rs` が WASM 実機経由の検証を担当する）
//! の呼び出し文脈——**部分ノードの断片レンダリングを `innerHTML` へ設定し、
//! 状態更新のたびに再レンダリングする**——を、`rws-core` が保証する契約
//! （`render()` の出力は常にエスケープ済み）のレベルでネイティブに検証する。
//!
//! 現時点の workspace には `rws-server` が存在せず、SSR（HTTP レスポンス
//! ボディ）・SSG（ファイル書き出し）はいずれもモード非依存の
//! `rws_core::render()` を共通で呼ぶ契約（`core/src/lib.rs` 冒頭の不変条件）
//! である。そのため本ファイルの SSR/SSG 側では:
//!
//! - **SSR 経路** = `render()` の戻り値をそのまま HTTP レスポンスボディ相当として検証する
//! - **SSG 経路** = 同じ `render()` 出力を実ファイルへ書き出し、読み戻した内容を検証する
//!
//! 将来 `server/` 実装後は実サーバー経由の E2E テストへ拡張する余地を残すが、
//! 本タスクのスコープ外とする（計画書参照）。
//!
//! WASM 経由の一貫性検証は TASK-1.3、escape 文字仕様の単体テストは #7
//! （TASK-1.1c）が担当する。
//!
//! テストは「エスケープ済み表現を含む」ことと「生ペイロードを含まない」こと
//! の両方を assert する。前者だけでは、たとえば出力が空文字列になる
//! 偽陰性（何もレンダリングされずに PASS してしまう不具合）を見逃すため。
//!
//! # 削除・弱体化の禁止
//!
//! 本ファイルの XSS 回帰テストは `.claude/rules/coding-rust.md` の規約により、
//! 以後の削除・弱体化・`#[ignore]` 化を禁止する。

use rws_core::{bind_attr_token, bind_text, el, escape_html, raw_html, render, text, Node};

/// OWASP XSS Prevention Cheat Sheet Rule #1 が挙げる脅威パターンを核とした
/// 共有ペイロード集合。CSR（本ファイル `mod csr`）が使用する。
pub mod payloads {
    /// タグ注入（PoC-3 実測ペイロード）。
    pub const SCRIPT_TAG: &str = "<script>alert('xss')</script>";
    /// イベントハンドラ属性つきタグ注入。
    pub const IMG_ONERROR: &str = "<img src=x onerror=alert(1)>";
    /// 二重引用符属性値からの breakout。
    pub const DOUBLE_QUOTE_BREAKOUT: &str = "\"><script>alert(1)</script>";
    /// 単一引用符属性値からの breakout（イベントハンドラ注入込み）。
    pub const SINGLE_QUOTE_BREAKOUT: &str = "' onmouseover='alert(1)";
    /// エンティティ偽装（`&` 先頭処理の確認。二重エスケープ・エスケープ漏れの双方を検知）。
    pub const ENTITY_SPOOF_TAG: &str = "&lt;script&gt;";
    /// エンティティ偽装（`&amp;` の再エスケープ挙動確認）。
    pub const ENTITY_SPOOF_AMP: &str = "&amp;lt;";
    /// コンテキスト脱出系（`</title>` 等、閉じタグによる親コンテキスト離脱）。
    pub const CONTEXT_BREAKOUT: &str = "</title><script>alert(1)</script>";
    /// 非 ASCII 混在文字列（マルチバイト透過の確認。エスケープ処理が
    /// バイト単位で文字境界を破壊しないことの回帰）。
    pub const NON_ASCII_MIXED: &str = "こんにちは<script>alert(1)</script>世界";

    /// 全ペイロードをまとめて返す（網羅的にループ検証する用途）。
    pub fn all() -> Vec<&'static str> {
        vec![
            SCRIPT_TAG,
            IMG_ONERROR,
            DOUBLE_QUOTE_BREAKOUT,
            SINGLE_QUOTE_BREAKOUT,
            ENTITY_SPOOF_TAG,
            ENTITY_SPOOF_AMP,
            CONTEXT_BREAKOUT,
            NON_ASCII_MIXED,
        ]
    }
}

/// CSR（Client-Side Rendering）経路の XSS 回帰テスト（TASK-1.2b、#10）。
///
/// `rws-wasm-client` の `mount_csr()` 相当——`render()` の出力文字列を
/// `innerHTML` に設定し、状態変化のたびに断片を再レンダリングする——呼び出し
/// パターンをネイティブ側で模した検証。`rws-core::render()` はモード非依存
/// レンダラであるため、CSR 固有のコード経路は本クレートには存在しないが、
/// 「CSR が実際に行う呼び出し方」でエスケープ保証が崩れないことを直接証明する。
mod csr {
    use super::*;

    /// 断片ノード（ページ全体でなく部分木）のレンダリングで、全ペイロードが
    /// テキストノード・属性値の両コンテキストでエスケープされ、ペイロード中の
    /// `<` `>` `&` `"` `'` を含む生の構文（`<script>` タグ・`<img ...>` タグの
    /// 実タグとしての出現）が出力に現れないことを確認する。
    ///
    /// 「エスケープ後も `onerror=` という語自体は文字列として残る」ことは
    /// 安全（実際の属性としては機能しないため）なので、判定はペイロード
    /// そのものが生の部分文字列として出力に含まれるか、および `<script>` /
    /// `<img` の実タグ開始が出力に含まれるかで行う（語の有無ではなく
    /// 構文としての危険性の有無を見る）。
    ///
    /// `innerHTML` に直接設定される断片であることを想定し、`<div id="app">`
    /// のようなマウントポイント直下の子要素としてレンダリングする。
    #[test]
    fn csr_fragment_render_escapes_all_payloads() {
        for payload in payloads::all() {
            // テキストコンテキスト: <p>{payload}</p> をマウントポイント直下に置く。
            let fragment = el(
                "div",
                vec![("id", "app")],
                vec![el("p", vec![], vec![text(payload)])],
            );
            let html = render(&fragment);
            assert_fragment_is_safe(payload, &html, "テキストコンテキスト");

            // 属性値コンテキスト: title 属性に payload を渡す。
            let attr_fragment = el(
                "div",
                vec![("id", "app")],
                vec![el("span", vec![("title", payload)], vec![])],
            );
            let attr_html = render(&attr_fragment);
            assert_fragment_is_safe(payload, &attr_html, "属性値コンテキスト");
        }
    }

    /// [`csr_fragment_render_escapes_all_payloads`] の共通アサーション。
    ///
    /// (1) ペイロードのエスケープ済み表現（[`escape_html`] が返す正解値）が
    ///     出力中に実際に存在する（肯定的アサーション。これが無いと、
    ///     `render()` が壊れてテキスト・属性の中身ごと出力しなくなる
    ///     （例: `<p></p>`）リグレッションが、他の否定条件をすべて素通り
    ///     させて偽陰性 PASS してしまう）、
    /// (2) ペイロードの生文字列が出力中に部分文字列として現れない
    ///     （現れれば `<` `>` `&` 等がエスケープされずに透過した証拠）、
    /// (3) `<script>` / `<img` の実タグ開始が出力に現れない、の 3 点を見る。
    fn assert_fragment_is_safe(payload: &str, html: &str, context_label: &str) {
        let expected_escaped = escape_html(payload);
        assert!(
            html.contains(&expected_escaped),
            "CSR 断片レンダリングの{context_label}で期待されるエスケープ済み表現が出力に見当たらない \
             （render() が内容自体を出力しなくなる偽陰性リグレッションの疑い）: \
             payload={payload:?}, expected_escaped={expected_escaped:?}, html={html}"
        );
        assert!(
            !html.contains(payload),
            "CSR 断片レンダリングの{context_label}でペイロードが生のまま出力に含まれた（エスケープ漏れの疑い）: payload={payload:?}, html={html}"
        );
        assert!(
            !html.contains("<script>"),
            "CSR 断片レンダリングの{context_label}で生の <script> タグが出力に現れた: payload={payload:?}, html={html}"
        );
        assert!(
            !html.contains("<img"),
            "CSR 断片レンダリングの{context_label}で生の <img タグが出力に現れた: payload={payload:?}, html={html}"
        );
    }

    /// 状態更新を模して同一構造のノード木をペイロード入りで再構築・再
    /// `render()` しても、エスケープが欠落せず、かつ二重エスケープにも
    /// ならないことを確認する（1 回目と 2 回目の出力が完全一致）。
    ///
    /// CSR では状態変化のたびに新しいノード木を組み立てて再レンダリングする
    /// （仮想 DOM 差分ではなく `rws-core` は文字列レンダラであるため、
    /// 再マウント相当の全体再構築を模す）。同一入力から常に同一出力になる
    /// （エスケープ回数が呼び出し回数に依存しない）ことが再レンダリング安全性
    /// の核心。
    #[test]
    fn csr_rerender_after_state_update_keeps_escaping() {
        for payload in payloads::all() {
            let build = |value: &str| -> Node {
                el(
                    "div",
                    vec![("id", "app")],
                    vec![el("span", vec![("data-value", value)], vec![text(value)])],
                )
            };

            let first_render = render(&build(payload));
            // 状態更新: 同じペイロードで再構築（値そのものは変わらないシナリオ）。
            let second_render = render(&build(payload));

            assert_eq!(
                first_render, second_render,
                "同一状態からの再レンダリングで出力が変化した（エスケープが呼び出し回数に依存している可能性）: payload={payload:?}"
            );
            assert!(
                !first_render.contains("<script>"),
                "再レンダリング結果に生スクリプトタグが含まれる: payload={payload:?}, html={first_render}"
            );
            // 二重エスケープの検知: render() をもう一度、今度は 1 回目の
            // 出力結果そのものをテキストとして与えて呼び出す（誤って
            // 「既にエスケープ済みの文字列」を再エスケープしてしまう実装
            // 誤りを想定した回帰）。ここでは 1 回目の出力を新規ノードの
            // テキスト内容として与えるのではなく、`escape_html` が
            // 呼び出し回数に応じて多重適用されていないことを、同一入力
            // からの複数回 render() 結果が常にバイト単位で一致すること
            // （上記 assert_eq!）で担保する。
            // build() は payload を data-value 属性値・テキスト内容の 2 箇所に
            // 埋め込むため、エスケープ対象文字 1 個につき出力中の '&' は 2 個
            // 生成される（属性値・テキストそれぞれに 1 回ずつエスケープが
            // 適用されるため）。この関係が崩れていれば、二重エスケープ
            // （さらに '&' が増える）かエスケープ漏れ（'&' が減る）の疑いがある。
            let escapable_count = payload.matches(['&', '<', '>', '"', '\'']).count();
            assert_eq!(
                first_render.matches('&').count(),
                escapable_count * 2,
                "エスケープ対象文字の出現回数から見て、二重エスケープまたは \
                 エスケープ漏れが疑われる: payload={payload:?}, html={first_render}"
            );
        }
    }

    /// 「SSR 相当の呼び出し」（サーバー側でページ全体を一括レンダリングして
    /// レスポンス送出する想定。ペイロードを含む断片を `<html><body>` 相当の
    /// 親構造でラップして呼ぶ）と「CSR 相当の呼び出し」（マウント時に断片
    /// 単体を `innerHTML` へ設定する想定。同じ内容をそのまま呼ぶ）とで、
    /// 実際に**異なる呼び出し経路**（親構造の有無・ネスト深さが異なる
    /// `render()` 呼び出し）を比較する。
    ///
    /// PoC-3 成功基準 1（エスケープ保証はレンダリングモードに依存しない）の
    /// 製品版回帰。`rws-core::render()` はモード引数を取らない単一実装で
    /// あるため、周辺構造が異なっていても断片自体のエスケープ結果
    /// （ペイロードのエスケープ済み表現）は一致し、かつ双方から生タグが
    /// 漏れないことを確認する。これにより「SSR 用と CSR 用で別のエスケープ
    /// 経路を新設していない」ことを、同一引数の重複呼び出しではなく異なる
    /// 呼び出し経路の比較で検証する（旧版は `render(&node)` を同一引数で
    /// 2 回呼ぶだけで常に自明に一致してしまい、モード分岐の新設を検知
    /// できなかった）。
    #[test]
    fn csr_output_is_mode_independent_from_ssr() {
        for payload in payloads::all() {
            let fragment = el(
                "div",
                vec![("id", "app"), ("data-role", payload)],
                vec![el("p", vec![], vec![text(payload)])],
            );

            // CSR 相当: クライアントがマウント時に断片単体を render()。
            let csr_like_output = render(&fragment);

            // SSR 相当: サーバーが同じ断片をページ全体構造でラップして
            // 一括レンダリング。呼び出し経路（親要素・ネスト深さ）が
            // CSR 経路とは異なる点が本比較の核心。
            let ssr_page = el(
                "html",
                vec![],
                vec![el("body", vec![], vec![fragment.clone()])],
            );
            let ssr_like_output = render(&ssr_page);

            // 肯定的アサーション: 期待されるエスケープ済み表現が両出力に
            // 実際に存在することを確認する。これが無いと、`render()` が
            // 内容ごと出力しなくなる（空文字列化）リグレッションでも
            // 以下の否定条件・部分一致条件のみでは検知できず偽陰性 PASS
            // してしまう。
            let expected_escaped = escape_html(payload);
            assert!(
                csr_like_output.contains(&expected_escaped),
                "CSR 相当出力に期待されるエスケープ済み表現が見当たらない（空出力リグレッションの疑い）: \
                 payload={payload:?}, expected_escaped={expected_escaped:?}, csr={csr_like_output}"
            );
            assert!(
                ssr_like_output.contains(&expected_escaped),
                "SSR 相当出力に期待されるエスケープ済み表現が見当たらない（空出力リグレッションの疑い）: \
                 payload={payload:?}, expected_escaped={expected_escaped:?}, ssr={ssr_like_output}"
            );

            assert!(
                ssr_like_output.contains(&csr_like_output),
                "SSR 相当のページ全体レンダリングに CSR 相当の断片レンダリング結果がそのまま含まれない \
                 （周辺構造の違いにより断片自体のエスケープ結果が変化した疑い）: payload={payload:?}"
            );
            assert!(
                !csr_like_output.contains("<script>") && !ssr_like_output.contains("<script>"),
                "モード非依存の render() 出力に生スクリプトタグが含まれる: payload={payload:?}, csr={csr_like_output}, ssr={ssr_like_output}"
            );
        }
    }

    /// CSR 経路でも非エスケープ出力点が `raw_html()` オプトインのみである
    /// ことを確認する（`raw_html` は透過し、隣接する `text()` はエスケープ
    /// される混在ケース）。
    ///
    /// CSR の断片更新では「信頼済みの固定 HTML 片（`raw_html`）」と
    /// 「ユーザー入力由来のテキスト（`text`）」が同じ断片内に混在し得る。
    /// 両者が隣接しても互いの扱いに影響しない（`raw_html` がテキストの
    /// エスケープを弱めたり、逆にテキストが `raw_html` をエスケープしたり
    /// しない）ことを検証する。
    #[test]
    fn csr_raw_html_optin_boundary_is_unchanged() {
        let trusted_fragment = "<b>trusted</b>";
        let untrusted_payload = payloads::SCRIPT_TAG;

        #[expect(
            clippy::disallowed_methods,
            reason = "ESCAPE-REVIEWED: raw_html オプトイン境界の回帰テスト。固定の信頼済み文字列のみを渡し、外部入力を含まない"
        )]
        let node = el(
            "div",
            vec![("id", "app")],
            vec![
                raw_html(trusted_fragment),
                el("p", vec![], vec![text(untrusted_payload)]),
            ],
        );
        let html = render(&node);

        assert!(
            html.contains("<b>trusted</b>"),
            "raw_html() オプトインの固定 HTML がそのまま透過していない: html={html}"
        );
        assert!(
            !html.contains("<script>alert('xss')</script>"),
            "raw_html() に隣接する text() の内容がエスケープされずに出力された（オプトイン境界が漏れた疑い）: html={html}"
        );
        assert!(
            html.contains("&lt;script&gt;"),
            "text() 側のペイロードが期待通りエスケープされていない: html={html}"
        );
    }
}

/// PoC-3 実測の基準ペイロード + OWASP 代表形を集約した XSS ペイロード集
/// （SSR・SSG 経路のテスト、TASK-1.2a・#9 が使用）。
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
        html.contains(&escape_html(payload)),
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
        // 陰性チェック（原文が属性値としてそのまま出現しない）だけでは、
        // `render()` が属性自体を丸ごと欠落させる・誤った形でエスケープする
        // といった回帰を素通りさせてしまう（vacuous pass）。属性シリアライズ
        // （`core/src/lib.rs` の `render_into`: ` {属性名}="{escape_html(値)}"`）が
        // 期待どおりの完全エスケープ済み文字列を生成していることを陽性側でも固定する。
        let expected_attr = format!("=\"{}\"", escape_html(payload));
        assert!(
            html.contains(&expected_attr),
            "escape_html の期待出力が title/data-value 属性値として render() 結果に含まれていない（payload: {payload:?}）: {html}"
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
        // SSR 側の陽性アサートと同様、陰性チェックのみでは属性の欠落・誤エスケープを
        // 検知できない（vacuous pass）。SSG ファイル読み戻し内容にも escape_html の
        // 期待出力がそのまま含まれることを陽性側で固定する。
        let expected_attr = format!("=\"{}\"", escape_html(payload));
        assert!(
            attr_from_file.contains(&expected_attr),
            "escape_html の期待出力が SSG ファイル内の属性値として含まれていない（payload: {payload:?}）: {attr_from_file}"
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
    #[expect(
        clippy::disallowed_methods,
        reason = "ESCAPE-REVIEWED: raw_html が唯一の迂回経路であることを対比固定するテスト。固定・無害な文字列のみを使用"
    )]
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

/// 束縛点マーキング API（イシュー #342、`core/src/bind.rs`）の XSS 回帰。
///
/// `bind_text`/`bind_attr_token` は既存 `el`/`render` への薄い委譲のみで
/// 新しいエスケープ処理を持たない（設計書 §3.3・不変条件 1・2 の継承）ため、
/// 本モジュールの既存ペイロード集合（`XSS_PAYLOADS`）に対しても同一の
/// エスケープ貫通が成立することを固定する。
mod bind_points {
    use super::*;

    /// `bind_text` の value に既存ペイロード集合を渡し、出力に生タグが
    /// 現れないこと（既存テキストエスケープ経路への全面委譲の固定）。
    #[test]
    fn bind_text_value_escapes_xss_payloads() {
        for payload in XSS_PAYLOADS {
            let node = bind_text("span", vec![("class", "count")], "counter", *payload);
            let html = render(&node);
            assert!(
                !html.contains("<script>") && !html.contains("<svg") && !html.contains("<iframe"),
                "bind_text の value 経由で生タグが出力された（payload={payload}）: {html}"
            );
            // マーカー属性自体は常に出力され、束縛点対応表構築（#343）が
            // 欠落しないことも併せて固定する。
            assert!(html.contains("data-bind-text=\"counter\""));
        }
    }

    /// `bind_attr_token` を値に持つ属性へペイロードを併置した要素で、
    /// 属性値エスケープ（5 文字: `"` `'` `<` `>` `&`）が貫通すること。
    #[test]
    fn attribute_alongside_bind_attr_marker_is_escaped() {
        for payload in XSS_PAYLOADS {
            let node = el(
                "button",
                vec![
                    ("aria-pressed", "false"),
                    ("title", payload),
                    (
                        rws_core::BIND_ATTR_ATTR,
                        &bind_attr_token("aria-pressed", "liked"),
                    ),
                ],
                vec![text("いいね")],
            );
            let html = render(&node);
            assert!(
                !html.contains("<script>") && !html.contains("<svg") && !html.contains("<iframe"),
                "bind_attr_token 併置要素の title 属性経由で生タグが出力された（payload={payload}）: {html}"
            );
            assert!(html.contains("data-bind-attr=\"aria-pressed:liked\""));
        }
    }

    /// `Box::leak` で `'static` に昇格させた悪性フィールド名でも、属性値
    /// コンテキストのエスケープにより breakout しないことを固定する
    /// （既存の悪性タグ名テスト `invalid_tag_name_is_skipped_without_panic`
    /// と対になる多層防御。フィールド名は本来 `&'static str` 定数のみを
    /// 受理する設計だが、型制約をすり抜けた場合の防御を確認する）。
    #[test]
    fn malicious_leaked_field_name_does_not_break_out_of_attribute_context() {
        let malicious_field: &'static str =
            Box::leak(String::from("counter\" onmouseover=\"alert(1)").into_boxed_str());
        let node = bind_text("span", vec![], malicious_field, "0");
        let html = render(&node);
        assert!(
            !html.contains("onmouseover=\"alert(1)\""),
            "悪性フィールド名経由で属性値 breakout が発生した: {html}"
        );
        assert!(html.contains("&quot;"));
    }

    /// 束縛点を使わない既存ノード構築の出力が不変であることの回帰
    /// （受け入れ条件 1: `bind.rs` 追加後も既存 `render()` 経路にコード変更が
    /// ないことのピン留め）。
    #[test]
    fn existing_node_construction_output_is_unaffected() {
        let payload = "<script>alert('xss')</script>";
        let tree = el("div", vec![], vec![text(payload)]);
        let html = render(&tree);
        assert_eq!(
            html,
            "<div>&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;</div>"
        );
    }
}

/// `keyed_list`（イシュー #344）の XSS 回帰テスト。
///
/// キー・項目テキスト・親属性値の 3 箇所に XSS ペイロードを注入し、
/// `data-key`/`data-bind-list` を含む出力が SSR/SSG いずれの経路でも
/// 既定エスケープを経由することを固定する。`keyed_list` は新しい
/// レンダリング経路・新しいエスケープ処理を追加しない（`core/src/keyed.rs`
/// 冒頭の不変条件参照）ため、本モジュールのテストは既存の
/// `page_with_text_payload`/`ssg_write_and_read_back` と同じ検証形式を踏襲する。
mod keyed_list_xss {
    use super::*;
    use rws_core::keyed::keyed_list;

    /// キーはアプリ側の識別子（データベース ID 等）由来であり、外部入力に
    /// 汚染され得る前提で扱う。XSS ペイロードをキーとして渡しても
    /// `data-key` 属性値としてエスケープされることを検証する。
    #[test]
    fn key_attribute_value_is_escaped_on_both_paths() {
        for payload in XSS_PAYLOADS {
            let list = keyed_list(
                "ul",
                vec![],
                "items",
                vec![((*payload).to_string(), el("li", vec![], vec![text("item")]))],
            )
            .expect("payload はキー欠落・重複に該当しないため構築できる");
            let ssr_html = render(&list);

            assert!(
                !ssr_html.contains("<script>"),
                "data-key 経由で <script> の breakout が発生した（payload: {payload:?}）: {ssr_html}"
            );
            let expected_key_attr = format!("data-key=\"{}\"", escape_html(payload));
            assert!(
                ssr_html.contains(&expected_key_attr),
                "data-key 属性値が期待どおりエスケープされていない（payload: {payload:?}）: {ssr_html}"
            );

            let ssg_html = ssg_write_and_read_back(&ssr_html, "keyed-key");
            assert_eq!(
                ssr_html, ssg_html,
                "SSR/SSG 出力が一致しない（keyed_list キー、payload: {payload:?}）"
            );
        }
    }

    /// 項目テキストに XSS ペイロードを埋め込んでも、通常の `text()` 経路と
    /// 同様にエスケープされることを確認する（`keyed_list` が子ノードの
    /// テキストエスケープに影響しないことの回帰）。
    #[test]
    fn item_text_content_is_escaped_on_both_paths() {
        for payload in XSS_PAYLOADS {
            let list = keyed_list(
                "ul",
                vec![],
                "items",
                vec![("k1".to_string(), el("li", vec![], vec![text(*payload)]))],
            )
            .unwrap();
            let ssr_html = render(&list);
            assert_text_payload_neutralized(&ssr_html, payload);

            let ssg_html = ssg_write_and_read_back(&ssr_html, "keyed-text");
            assert_text_payload_neutralized(&ssg_html, payload);
        }
    }

    /// 親要素の呼び出し側属性（`data-bind-list` を除く）に XSS ペイロードを
    /// 埋め込んでも属性値としてエスケープされることを確認する。
    #[test]
    fn parent_attribute_value_is_escaped_on_both_paths() {
        for payload in XSS_PAYLOADS {
            let list = keyed_list(
                "ul",
                vec![("data-testid", payload)],
                "items",
                vec![("k1".to_string(), el("li", vec![], vec![text("item")]))],
            )
            .unwrap();
            let ssr_html = render(&list);

            assert!(
                !ssr_html.contains("<script>"),
                "親属性経由で <script> の breakout が発生した（payload: {payload:?}）: {ssr_html}"
            );
            let expected_attr = format!("data-testid=\"{}\"", escape_html(payload));
            assert!(
                ssr_html.contains(&expected_attr),
                "親属性値が期待どおりエスケープされていない（payload: {payload:?}）: {ssr_html}"
            );
            // data-bind-list マーカー属性自体は呼び出し側入力に依存しない固定値
            // （field は &'static str）であり、常にそのまま出力される。
            assert!(ssr_html.contains("data-bind-list=\"items\""));

            let ssg_html = ssg_write_and_read_back(&ssr_html, "keyed-parent-attr");
            assert_eq!(
                ssr_html, ssg_html,
                "SSR/SSG 出力が一致しない（keyed_list 親属性、payload: {payload:?}）"
            );
        }
    }
}

/// URL スキーム検証・イベントハンドラ属性ブロックの XSS 回帰テスト
/// （イシュー #373）。`escape_html` は属性値コンテキストからの breakout
/// （`"` 等）を防ぐが、脱出を伴わない `javascript:` 等の URL スキーム
/// 経由の XSS は別の防御（`rws_core::is_safe_url` の許可リスト判定・
/// `render_into` での属性スキップ）が必要になる。本モジュールは
/// SSR（`render()` 戻り値）・SSG（ファイル書き出し読み戻し）・CSR
/// （部分ノードの断片レンダリング）の 3 経路すべてで、危険スキームの
/// URL 属性値・`on*` 属性が出力に一切現れないこと、および安全な URL
/// （相対 URL・許可スキーム）は従来どおり透過することを固定する
/// （削除・弱体化禁止は本ファイル冒頭の規約に従う）。
mod url_scheme_xss {
    use super::*;

    /// 拒否されるべき URL（危険スキーム・偽装形）。
    const DANGEROUS_URLS: &[&str] = &[
        "javascript:alert(1)",
        "JaVaScRiPt:alert(1)",
        "java\tscript:alert(1)",
        "java\nscript:alert(1)",
        "\u{0}javascript:alert(1)",
        " javascript:alert(1)",
        "data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==",
        "vbscript:msgbox(1)",
        "foo:bar",
    ];

    /// 透過されるべき安全な URL（相対 URL・許可スキーム）。
    const SAFE_URLS: &[&str] = &[
        "/items/1",
        "./rel",
        "?q=1",
        "#frag",
        "//example.com/x",
        "https://example.com",
        "http://example.com",
        "mailto:a@example.com",
        "tel:+819012345678",
        "HTTPS://EXAMPLE.COM",
        "/path/a:b",
    ];

    fn page_with_href(url: &str) -> Node {
        el(
            "a",
            vec![("href", url), ("data-nav", "safe")],
            vec![text("link")],
        )
    }

    /// SSR 経路: 危険スキームの `href` は属性ごと出力から消え、`javascript:`
    /// という文字列自体が render() 出力に一切現れないことを確認する。
    #[test]
    fn ssr_response_body_blocks_dangerous_url_schemes() {
        for url in DANGEROUS_URLS {
            let html = render(&page_with_href(url));
            assert!(
                !html.contains("href="),
                "危険スキームの href が出力された（url: {url:?}）: {html}"
            );
            assert!(
                !html.to_lowercase().contains("javascript:")
                    && !html.to_lowercase().contains("vbscript:")
                    && !html.to_lowercase().contains("data:text/html"),
                "危険スキーム文字列が出力に残存した（url: {url:?}）: {html}"
            );
            // 兄弟属性（data-nav）は影響を受けず出力される（過剰ブロックでないこと）。
            assert!(html.contains("data-nav=\"safe\""));
        }
    }

    /// SSR 経路: 安全な URL は従来どおり透過し、`href` 属性がエスケープ済みの
    /// 値で出力されることを確認する。
    #[test]
    fn ssr_response_body_allows_safe_urls() {
        for url in SAFE_URLS {
            let html = render(&page_with_href(url));
            let expected = format!("href=\"{}\"", escape_html(url));
            assert!(
                html.contains(&expected),
                "安全な URL が透過されなかった（url: {url:?}）: {html}"
            );
        }
    }

    /// SSG 経路: ファイル書き出し・読み戻し後も同一の遮断・透過が成立する
    /// ことを確認する。
    #[test]
    fn ssg_file_output_blocks_dangerous_url_schemes() {
        for (i, url) in DANGEROUS_URLS.iter().enumerate() {
            let html = render(&page_with_href(url));
            let from_file = ssg_write_and_read_back(&html, &format!("url-danger-{i}"));
            assert!(
                !from_file.contains("href="),
                "SSG 出力に危険スキームの href が残存した（url: {url:?}）: {from_file}"
            );
        }
    }

    /// CSR 経路: `render()` の断片出力を `innerHTML` 相当として扱う呼び出し
    /// パターンでも同一の保証が成立することを確認する（`mod csr` と同型の
    /// ネイティブ検証）。
    #[test]
    fn csr_fragment_render_blocks_dangerous_url_schemes() {
        for url in DANGEROUS_URLS {
            let fragment = render(&page_with_href(url));
            assert!(
                !fragment.contains("href="),
                "CSR 断片レンダリングで危険スキームの href が出力された（url: {url:?}）: {fragment}"
            );
        }
    }

    /// `srcset` はカンマ区切りの複数候補を持つ特殊構文。1 候補でも危険
    /// スキームなら属性全体をスキップし、全候補安全なら透過することを
    /// 確認する（部分書き換えをしない決定的挙動の固定）。
    #[test]
    fn srcset_attribute_all_or_nothing_validation() {
        let node = el(
            "img",
            vec![
                ("srcset", "/a.png 1x, javascript:alert(1) 2x"),
                ("alt", "safe"),
            ],
            vec![],
        );
        let html = render(&node);
        assert!(
            !html.contains("srcset="),
            "危険スキーム候補を含む srcset が出力された: {html}"
        );
        assert!(html.contains("alt=\"safe\""));

        let safe_node = el(
            "img",
            vec![("srcset", "/a.png 1x, /b.png 2x"), ("alt", "safe")],
            vec![],
        );
        let safe_html = render(&safe_node);
        assert!(
            safe_html.contains("srcset=\"/a.png 1x, /b.png 2x\""),
            "全候補安全な srcset が透過されなかった: {safe_html}"
        );
    }

    /// イベントハンドラ属性（`on*`）は値によらず一律出力されないことを
    /// SSR/CSR 双方の呼び出しパターンで確認する。
    #[test]
    fn event_handler_attributes_are_never_rendered() {
        for attr in ["onclick", "ONERROR", "OnMouseOver"] {
            let node = el(
                "div",
                vec![(attr, "alert(1)"), ("class", "safe")],
                vec![text("x")],
            );
            let html = render(&node);
            assert!(
                !html.to_lowercase().contains(&attr.to_lowercase()),
                "イベントハンドラ属性 {attr} が出力された: {html}"
            );
            assert!(html.contains("class=\"safe\""));
        }
    }
}
