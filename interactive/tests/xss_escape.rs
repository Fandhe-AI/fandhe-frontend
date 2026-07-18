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

use rws_core::{el, text, Node};
use rws_interactive::codec::{self, Value};
use rws_interactive::{render_for_hydration, AppState, Component, Hydrate, HydrateError};

/// `AppState::view()`（[`Component`] トレイト経由）を `rws_core::render()` に
/// 通した CSR 相当の HTML 文字列を返すテスト用ヘルパ。
///
/// TASK-11.1a/TASK-11.1b で API が [`Component`]/[`Hydrate`] トレイトへ
/// 一般化される前は `rws_interactive::render_html` という自由関数が
/// 存在したが、現行 API では `view()` の呼び出しと `rws_core::render()` の
/// 呼び出しを利用側が明示的に組み合わせる契約になっている（lib.rs 参照）。
fn render_html(state: &AppState) -> String {
    rws_core::render(&state.view())
}

/// [`render_for_hydration`] の結果を `rws_core::render()` に通した SSR 相当の
/// HTML 文字列を返すテスト用ヘルパ（上記 `render_html` の SSR 版）。
fn render_html_for_hydration(state: &AppState) -> String {
    rws_core::render(&render_for_hydration(state))
}

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
/// 現れていないこと、かつペイロードのエスケープ済み表現が実際に出力へ
/// 含まれていることを確認する共通アサーション。
///
/// `core/tests/xss_escape.rs`（`assert_fragment_is_safe`）の方針と揃え、
/// 以下 3 点を見る。
///
/// 1. **肯定的アサーション（偽陰性防止）**: [`rws_core::escape_html`] が
///    返す正解のエスケープ済み表現が出力中に実際に存在すること。これが
///    無いと、ペイロードが出力へ丸ごと現れなくなる（例: 空文字列化・
///    フィールドの取りこぼし）リグレッションを、後続の否定条件がすべて
///    素通りさせて偽陰性 PASS してしまう。
/// 2. 生のペイロード部分文字列が出力に含まれない（`<` `>` `&` `"` `'` が
///    エスケープされずに透過していないこと）。
/// 3. `<script>`/`<img` の実タグ開始が出力に含まれない（語の有無ではなく
///    構文としての危険性の有無を見る）。
fn assert_html_is_safe(payload: &str, html: &str, context_label: &str) {
    let expected_escaped = rws_core::escape_html(payload);
    assert!(
        html.contains(&expected_escaped),
        "{context_label}: 期待されるエスケープ済み表現が出力に見当たらない \
         （render() がペイロードごと出力しなくなる偽陰性リグレッションの疑い）: \
         payload={payload:?}, expected_escaped={expected_escaped:?}, html={html}"
    );
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
        s.draft = payload.to_string();
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
        s.draft = payload.to_string();
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
    s.draft = payloads::DOUBLE_QUOTE_BREAKOUT.to_string();

    let csr_html = render_html(&s);
    let ssr_html = render_html_for_hydration(&s);

    assert_html_is_safe(payloads::SCRIPT_TAG, &csr_html, "CSR 同一性検証");
    assert_html_is_safe(payloads::SCRIPT_TAG, &ssr_html, "SSR 同一性検証");

    // rws_core::el の attrs 出力順序（挿入順）に合わせて、ルート要素に
    // 追加された 3 つのハイドレーション属性をこの順で連結した断片を
    // 組み立てる。属性値自体は render_html 側と同じくエスケープ済みの
    // 生 HTML 文字列として得られるため、単純な文字列除去で比較できる。
    let attrs = s.hydration_attrs();
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
    s.draft = payloads::SINGLE_QUOTE_BREAKOUT.to_string();
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

// --- イシュー #163: ネスト構造（codec::Value）経由の XSS 回帰 ---------------

/// ネストした `codec::Value`（`Map`/`List`）で状態を表現する最小コンポーネント。
///
/// `docs/hydration-nested-state.md` が確定した設計どおり、`Value` codec は
/// 区切り文字・エスケープ文字のみを対象とした構造的エスケープ（データ注入
/// 防止）を担い、HTML エスケープは一切行わない。HTML としての安全性は
/// 常に `render_for_hydration` → `rws_core::render()` の既定エスケープ経路
/// のみが担保する契約であることを、ネスト値経由でも固定する回帰テスト。
struct NestedComponent {
    profile_name: String,
}

impl Component for NestedComponent {
    type Action = ();
    fn update(&mut self, _action: ()) {}
    fn view(&self) -> Node {
        el(
            "div",
            vec![("id", "nested-root")],
            vec![text(self.profile_name.clone())],
        )
    }
    fn decode_action(_name: &str, _payload: &str) -> Option<()> {
        None
    }
}

impl Hydrate for NestedComponent {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let value = Value::Map(vec![(
            "profile".to_string(),
            Value::Map(vec![(
                "name".to_string(),
                Value::Str(self.profile_name.clone()),
            )]),
        )]);
        vec![(
            "data-hydrate-state".to_string(),
            codec::encode_value(&value),
        )]
    }

    fn from_hydration_attrs(_attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        // 本テストは SSR 出力の安全性のみを検証するため、復元経路は不要。
        Ok(NestedComponent {
            profile_name: String::new(),
        })
    }
}

/// ネスト値（`Map` の中の `Map` の中の `Str`）に XSS ペイロードを埋め込んでも、
/// `render_for_hydration` の SSR 出力が属性境界・タグ境界を破らないこと
/// （既定エスケープが `Value` codec のネスト段数に関わらず貫通することの
/// 回帰確認、イシュー #163）。
#[test]
fn render_for_hydration_escapes_payloads_nested_inside_value_codec() {
    for payload in payloads::all() {
        let component = NestedComponent {
            profile_name: payload.to_string(),
        };
        let html = rws_core::render(&render_for_hydration(&component));
        assert_html_is_safe(payload, &html, "ネスト Value（Map in Map）属性コンテキスト");
    }
}

/// `Value` codec 自体は区切り文字・バックスラッシュのみをエスケープし、HTML
/// メタ文字（`<` `>` `"` `'` `&`）はそのまま素通しする契約であることを直接
/// 確認する（HTML エスケープはあくまで render 層の責務であり、codec 層に
/// 二重実装しないという設計判断の固定）。
#[test]
fn value_codec_encode_does_not_html_escape_payloads() {
    for payload in payloads::all() {
        let encoded = codec::encode_value(&Value::Str(payload.to_string()));
        let decoded = codec::decode_value(&encoded).expect("well-formed encoding must decode");
        assert_eq!(
            decoded,
            Value::Str(payload.to_string()),
            "codec 層で HTML エスケープ相当の変換が混入している（render 層との責務分離違反）"
        );
    }
}
