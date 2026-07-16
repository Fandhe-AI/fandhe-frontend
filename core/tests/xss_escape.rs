//! TASK-1.2 の 3 経路（SSR・SSG・CSR）XSS 回帰テスト（親タスク #8、REQ-1 の受け入れ基準）。
//!
//! 本ファイルは `mod payloads`（共有ペイロード集合）と `mod csr`
//! （TASK-1.2b、#10 が担当する CSR 経路のテスト）を提供する。
//! SSR・SSG 経路のテスト（`mod ssr` / `mod ssg`、TASK-1.2a・#9）は
//! 本ファイルに追記される想定であり、`payloads` モジュールはそちらからも
//! 再利用できるよう独立させてある。
//!
//! `rws-core::render()` は SSR・SSG・CSR のいずれのモードからも共通で
//! 呼ばれる（`core/src/lib.rs` 冒頭の不変条件を参照）。CSR 経路は
//! `rws-wasm-client` の `mount_csr()` 相当（未実装。TASK-1.3 で
//! `wasm-client/tests/xss_escape_wasm.rs` が WASM 実機経由の検証を担当する）
//! の呼び出し文脈——**部分ノードの断片レンダリングを `innerHTML` へ設定し、
//! 状態更新のたびに再レンダリングする**——を、`rws-core` が保証する契約
//! （`render()` の出力は常にエスケープ済み）のレベルでネイティブに検証する。
//!
//! テストは「エスケープ済み表現を含む」ことと「生ペイロードを含まない」こと
//! の両方を assert する。前者だけでは、たとえば出力が空文字列になる
//! 偽陰性（何もレンダリングされずに PASS してしまう不具合）を見逃すため。

use rws_core::{el, escape_html, raw_html, render, text, Node};

/// OWASP XSS Prevention Cheat Sheet Rule #1 が挙げる脅威パターンを核とした
/// 共有ペイロード集合。SSR/SSG（#9）・CSR（本ファイル `mod csr`）の双方が
/// 同一の脅威網羅性を持つことを保証するため、モジュールとして共有する。
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
