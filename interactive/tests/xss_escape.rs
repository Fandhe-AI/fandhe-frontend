//! TASK-11.1c（#72、REQ-11・REQ-1）: interactive 経路（状態 → render）の
//! XSS 回帰テスト。
//!
//! `core/tests/xss_escape.rs`（`mod payloads`）が定義する OWASP 準拠の
//! 共有ペイロード集合と整合する観点で、`rws-interactive` が状態（`AppState`
//! の `draft`/`items`）を `render`/`render_for_hydration` 経由でノード木へ
//! 展開する際、`rws_core::render()` の既定エスケープが最後まで貫通する
//! ことを固定する。`rws-interactive` は `raw_html` を一切使用しない
//! （lib.rs 冒頭の不変条件 1 参照）ため、エスケープ迂回経路は存在しない
//! 前提だが、本ファイルはその前提を統合テストとして直接検証する。
//!
//! # 削除・弱体化の禁止
//!
//! `.claude/rules/coding-rust.md` の規約により、本ファイルの XSS 回帰
//! テストは以後の削除・弱体化・`#[ignore]` 化を禁止する。

use rws_interactive::{hydration_attrs, render_html, render_html_for_hydration, AppState};

/// OWASP XSS Prevention Cheat Sheet Rule #1 系のペイロード集合。
/// `core/tests/xss_escape.rs` の `mod payloads` と観点を揃える
/// （interactive クレートは core に依存するが、テストコード自体は
/// クレート境界をまたいで共有せず、ここで明示的に再定義する）。
mod payloads {
    pub const SCRIPT_TAG: &str = "<script>alert('xss')</script>";
    pub const IMG_ONERROR: &str = "<img src=x onerror=alert(1)>";
    pub const DOUBLE_QUOTE_BREAKOUT: &str = "\"><script>alert(1)</script>";
    pub const SINGLE_QUOTE_BREAKOUT: &str = "' onmouseover='alert(1)";
    pub const CONTEXT_BREAKOUT: &str = "</title><script>alert(1)</script>";
    pub const NON_ASCII_MIXED: &str = "こんにちは<script>alert(1)</script>世界";

    pub fn all() -> Vec<&'static str> {
        vec![
            SCRIPT_TAG,
            IMG_ONERROR,
            DOUBLE_QUOTE_BREAKOUT,
            SINGLE_QUOTE_BREAKOUT,
            CONTEXT_BREAKOUT,
            NON_ASCII_MIXED,
        ]
    }
}

/// ペイロードが実タグ・実イベントハンドラとして機能する構文で出力に
/// 現れていないことを確認する共通アサーション。
///
/// `core/tests/xss_escape.rs` の方針と同様、「生のペイロード部分文字列が
/// 含まれない」ことに加え、`<script>`/`<img` の実タグ開始が現れないこと
/// も確認する（語の有無ではなく構文としての危険性の有無を見る）。
fn assert_html_is_safe(payload: &str, html: &str, context_label: &str) {
    assert!(
        !html.contains(payload),
        "{context_label}: 生のペイロードがエスケープされずに出力へ透過している: \
         payload={payload:?}, html={html}"
    );
    assert!(
        !html.contains("<script>") && !html.contains("<img"),
        "{context_label}: 実タグ開始が出力に含まれる（エスケープ漏れの疑い）: \
         payload={payload:?}, html={html}"
    );
}

// --- items（テキストコンテキスト + data-hydrate-items 属性コンテキスト） ---

#[test]
fn render_html_escapes_all_payloads_injected_via_items() {
    for payload in payloads::all() {
        let mut s = AppState::new();
        s.items = vec![payload.to_string()];
        let html = render_html(&s);
        assert_html_is_safe(payload, &html, "CSR items テキストコンテキスト");
    }
}

#[test]
fn render_html_for_hydration_escapes_all_payloads_injected_via_items() {
    // items は data-hydrate-items 属性にもエンコードされて埋め込まれる
    // （lib.rs の hydration_attrs 参照）。属性値コンテキスト・テキスト
    // コンテキストの双方で安全であることを確認する。
    for payload in payloads::all() {
        let mut s = AppState::new();
        s.items = vec![payload.to_string()];
        let html = render_html_for_hydration(&s);
        assert_html_is_safe(payload, &html, "SSR items（テキスト + 属性コンテキスト）");
    }
}

// --- draft（value 属性コンテキスト） ----------------------------------------

#[test]
fn render_html_escapes_all_payloads_injected_via_draft_value_attribute() {
    // draft は <input value="..."> 属性へ入る（属性値経路）。二重引用符
    // breakout ペイロードが属性から脱出できないことが本テストの核心。
    for payload in payloads::all() {
        let mut s = AppState::new();
        s.set_draft(payload);
        let html = render_html(&s);
        assert_html_is_safe(payload, &html, "CSR draft value 属性コンテキスト");

        // 属性脱出の直接確認: エスケープ済み `"` (&quot;) が
        // input 要素の value 属性内に留まり、次の属性トークンとして
        // 独立したタグ・属性が生成されていないこと。
        if payload.contains('"') {
            assert!(
                !html.contains("\"><script>"),
                "draft の二重引用符 breakout が属性脱出を許している: html={html}"
            );
        }
    }
}

#[test]
fn render_html_for_hydration_escapes_draft_in_both_value_and_hydrate_attrs() {
    // draft は value 属性と data-hydrate-draft 属性の 2 箇所に現れる
    // （lib.rs render_with_root_attrs / hydration_attrs 参照）。
    // いずれの属性値コンテキストでもエスケープが貫通することを確認する。
    for payload in payloads::all() {
        let mut s = AppState::new();
        s.set_draft(payload);
        let html = render_html_for_hydration(&s);
        assert_html_is_safe(payload, &html, "SSR draft（value + data-hydrate-draft）");
    }
}

// --- render と render_for_hydration の DOM 同一性（悪性入力下でも維持） ----

#[test]
fn render_and_render_for_hydration_share_same_dom_shape_under_malicious_input() {
    // ハイドレーション属性差分を除けば CSR/SSR の出力 DOM は同一である
    // （lib.rs render_for_hydration の rustdoc 契約）。悪性入力を含む
    // 状態でもこの同一性が崩れないことを、実際に SSR 側からハイドレー
    // ション属性の断片を取り除いた文字列が CSR 側と一致するところまで
    // 確認する（lib.rs 既存スモークテストと同じ手法）。属性値をここで
    // 再度ハードコードすると壊れやすいため、`hydration_attrs` の戻り値
    // から動的に「取り除くべき断片」を組み立てる。
    let mut s = AppState::new();
    s.items = vec![
        payloads::SCRIPT_TAG.to_string(),
        payloads::IMG_ONERROR.to_string(),
    ];
    s.set_draft(payloads::DOUBLE_QUOTE_BREAKOUT);

    let csr_html = render_html(&s);
    let ssr_html = render_html_for_hydration(&s);

    assert_html_is_safe(payloads::SCRIPT_TAG, &csr_html, "CSR 同一性検証");
    assert_html_is_safe(payloads::SCRIPT_TAG, &ssr_html, "SSR 同一性検証");

    // rws_core::el の attrs 出力順序（挿入順）に合わせて、ルート要素に
    // 追加された 3 つのハイドレーション属性をこの順で連結した断片を
    // 組み立てる。属性値自体は render_html 側と同じくエスケープ済みの
    // 生 HTML 文字列として得られるため、単純な文字列除去で比較できる。
    let attrs = hydration_attrs(&s);
    let hydrate_fragment: String = attrs
        .iter()
        .map(|(k, v)| format!(" {k}=\"{}\"", rws_core::escape_html(v)))
        .collect();
    assert!(
        ssr_html.contains(&hydrate_fragment),
        "SSR 出力にハイドレーション属性断片が想定順序で見当たらない: \
         fragment={hydrate_fragment:?}, ssr_html={ssr_html}"
    );
    assert_eq!(
        ssr_html.replacen(&hydrate_fragment, "", 1),
        csr_html,
        "ハイドレーション属性を取り除いた SSR 出力が CSR 出力と一致しない \
         （悪性入力下で DOM 形状の同一性が崩れている）"
    );
}

// --- 複合状態（counter + draft + items すべてに悪性入力） -------------------

#[test]
fn render_html_stays_safe_with_malicious_input_across_all_fields() {
    let mut s = AppState::new();
    s.set_draft(payloads::SINGLE_QUOTE_BREAKOUT);
    s.items = payloads::all().iter().map(|p| p.to_string()).collect();

    let csr_html = render_html(&s);
    let ssr_html = render_html_for_hydration(&s);

    for payload in payloads::all() {
        assert_html_is_safe(payload, &csr_html, "複合状態 CSR");
        assert_html_is_safe(payload, &ssr_html, "複合状態 SSR");
    }
}

// --- 偽陰性防止: エスケープ済み表現が実際に出力へ含まれることの肯定的確認 ---

#[test]
fn render_html_actually_contains_escaped_representation_of_item_payload() {
    // 否定条件（生ペイロードを含まない）だけでは、render() が内容ごと
    // 出力しなくなる偽陰性リグレッション（例: 空文字列化）を見逃す。
    // 肯定的に「エスケープ済み表現が存在する」ことも確認する
    // （core/tests/xss_escape.rs と同じ設計判断）。
    let mut s = AppState::new();
    s.items = vec![payloads::SCRIPT_TAG.to_string()];
    let html = render_html(&s);
    assert!(
        html.contains("&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;")
            || html.contains("&lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;")
            || html.contains("&lt;script&gt;"),
        "エスケープ済み script 表現が出力に見当たらない（偽陰性リグレッションの疑い）: html={html}"
    );
}
