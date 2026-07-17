//! `rws-interactive`: DOM / `wasm-bindgen` 非依存の状態管理コア。
//!
//! `rws-core`（path 依存のみ）に依存し、アプリ状態の保持・アクション
//! ディスパッチ・ハイドレーション属性コーデック・状態から `Node` への
//! 純粋レンダリングを提供する。TASK-11.2（`rws-wasm-full`）・TASK-11.3
//! （`rws-wasm-thin`）・TASK-11.4（ハイドレーション）が共通で利用する
//! プラットフォーム非依存コアであり、DOM API・`web_sys`・`wasm-bindgen`
//! を一切参照しない（ネイティブ環境でもそのままテスト・計測できる）。
//!
//! `docs/spec/03-poc/wasm-runtime-split/interactive/src/lib.rs`（PoC-5）の
//! 状態構造・API 形状を製品版として引き継ぐ。TASK-11.1a（状態管理 API
//! 設計確定書）がマージされた場合はそちらを正とし、本実装との乖離が
//! あれば追従 PR で調整する。
//!
//! # 本クレートの不変条件（REQ-1・REQ-2・REQ-11）
//!
//! 1. HTML 出力はすべて `rws_core::el` / `text` のノード木 API を経由する。
//!    `format!` 等による HTML 文字列の直接組み立て・独自エスケープ・
//!    `rws_core::raw_html` 以外の迂回経路を新設しない（REQ-1 を弱めない）。
//! 2. **`unsafe` コード禁止**: `#![forbid(unsafe_code)]` によりクレート全体で
//!    機械的に禁止する（REQ-2）。`unsafe` は WASM バインディング層・FFI 境界
//!    に限定され、本クレートには含まれない。
//! 3. **外部依存ゼロ**: `Cargo.toml` の `[dependencies]` は `rws-core`（path
//!    依存）のみを維持する。`rws-interactive -> rws-core` の第一者依存は
//!    `xtask` の `check-core-deps` で外部依存に計上されない前提のクレート
//!    構成（`xtask/src/check_deps.rs` の `ZERO_DEP_CRATES` 参照）。
//! 4. ライブラリコードで `unwrap()` / `expect()` / `panic!` を使わない。
//!    クライアント制御下になり得る入力（ハイドレーション属性値）のパース
//!    失敗は既定値へフォールバックし、処理を止めない（DoS 耐性）。
//!
//! ## スコープ外
//!
//! 状態管理 API の設計確定（TASK-11.1a）・テストの本格網羅（TASK-11.1c）・
//! `wasm-full`/`wasm-thin` からの実利用とイベント配線（TASK-11.2/11.3）・
//! `wasm-full` 側のハイドレーション製品化（TASK-11.4）は本クレートでは
//! 扱わない。本クレートに同梱するテストはスモーク水準にとどめる。

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use rws_core::{el, li, text, ul, Node};

/// アプリ状態: カウンター・フォーム入力（下書き）・動的リスト。
///
/// PoC-5 の最小インタラクティブコンポーネント（カウンター＋フォーム入力＋
/// 動的リスト更新）をそのまま製品状態として引き継ぐ。TASK-11.1a の設計
/// 確定書がマージされ次第、フィールド構成はそちらに従って調整する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    /// カウンター値。`increment`/`decrement`/`reset_counter` からのみ変更する。
    pub counter: i64,
    /// フォーム入力欄の下書き文字列。`add_item` 実行時に `items` へ確定し
    /// クリアされる。
    pub draft: String,
    /// 動的リストの項目群。ハイドレーション時は [`encode_items`] でリスト
    /// 全体を 1 属性値へエンコードする。
    pub items: Vec<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            counter: 0,
            draft: String::new(),
            items: vec!["最初の項目".to_string()],
        }
    }
}

impl AppState {
    /// 既定状態（カウンター 0・下書き空・初期項目 1 件）を生成する。
    pub fn new() -> Self {
        Self::default()
    }

    /// カウンターを 1 増やす。
    pub fn increment(&mut self) {
        self.counter += 1;
    }

    /// カウンターを 1 減らす。
    pub fn decrement(&mut self) {
        self.counter -= 1;
    }

    /// カウンターを 0 へ戻す。
    pub fn reset_counter(&mut self) {
        self.counter = 0;
    }

    /// フォーム入力欄の下書きを置き換える。
    pub fn set_draft(&mut self, value: &str) {
        self.draft = value.to_string();
    }

    /// 下書きの前後空白を除去し、空でなければリストへ追加して下書きをクリアする。
    pub fn add_item(&mut self) {
        let trimmed = self.draft.trim();
        if !trimmed.is_empty() {
            self.items.push(trimmed.to_string());
            self.draft.clear();
        }
    }

    /// 指定インデックスの項目を削除する。範囲外の場合は何もしない（安全側フォールバック）。
    pub fn remove_item(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
        }
    }
}

/// アクション名と引数文字列から状態を更新する共通ディスパッチャ。
///
/// `wasm-full`（Rust 側イベントハンドラ）・`wasm-thin`（JS グルーが呼ぶ wasm
/// 関数）・ネイティブ計測（`bench`）のいずれからも同一関数を呼び出す想定の
/// 境界設計であり、追加のシリアライズ依存を必要としない（PoC-5 実証済み）。
/// 未知のアクション名は無視する（panic させない）。
pub fn dispatch(state: &mut AppState, action: &str, payload: &str) {
    match action {
        "increment" => state.increment(),
        "decrement" => state.decrement(),
        "reset" => state.reset_counter(),
        "set_draft" => state.set_draft(payload),
        "add_item" => state.add_item(),
        "remove_item" => {
            if let Ok(idx) = payload.parse::<usize>() {
                state.remove_item(idx);
            }
        }
        _ => {}
    }
}

/// リスト項目を 1 属性値へ結合する際の区切り文字。
///
/// 通常のテキスト入力に混入しない ASCII 制御文字（Unit Separator）を使う
/// ことで、JSON 等の追加依存なしに複数項目をハイドレーション属性 1 個へ
/// エンコードできる（REQ-11 受け入れ基準・`rws-core` の「外部依存ゼロ」
/// 方針を本クレートでも踏襲する）。
const ITEM_SEP: char = '\u{1f}';
/// [`ITEM_SEP`] / [`ESCAPE_CHAR`] 自体が項目文字列に混入した場合のエスケープ文字。
const ESCAPE_CHAR: char = '\\';

/// アイテム文字列中の [`ESCAPE_CHAR`] と [`ITEM_SEP`] をエスケープする。
///
/// 項目文字列自体に区切り文字・エスケープ文字が含まれていても、区切り文字
/// による項目境界の偽装（データ注入）が起きないようにするための内部処理。
fn escape_item(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            ESCAPE_CHAR => {
                out.push(ESCAPE_CHAR);
                out.push(ESCAPE_CHAR);
            }
            ITEM_SEP => {
                out.push(ESCAPE_CHAR);
                out.push('u');
            }
            other => out.push(other),
        }
    }
    out
}

/// [`escape_item`] の逆変換。
fn unescape_item(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == ESCAPE_CHAR {
            match chars.next() {
                Some(ESCAPE_CHAR) => out.push(ESCAPE_CHAR),
                Some('u') => out.push(ITEM_SEP),
                // 未知のエスケープシーケンスはそのまま残す（安全側フォールバック。panic しない）。
                Some(other) => {
                    out.push(ESCAPE_CHAR);
                    out.push(other);
                }
                None => out.push(ESCAPE_CHAR),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// 項目一覧を [`ITEM_SEP`] 区切りの 1 文字列へエンコードする（サーバー側責務）。
fn encode_items(items: &[String]) -> String {
    items
        .iter()
        .map(|s| escape_item(s))
        .collect::<Vec<_>>()
        .join(&ITEM_SEP.to_string())
}

/// [`encode_items`] の逆変換（クライアント側責務）。
fn decode_items(items_joined: &str) -> Vec<String> {
    if items_joined.is_empty() {
        Vec::new()
    } else {
        items_joined.split(ITEM_SEP).map(unescape_item).collect()
    }
}

/// サーバー（SSR）側の責務: 状態をハイドレーション用の DOM 属性へエンコードする。
///
/// [`render_for_hydration`] がこの戻り値をルート要素の属性として埋め込む。
/// 出力値は `el()` の attrs としてのみ利用される想定であり、呼び出し側が
/// これを HTML 文字列へ直接埋め込むことは想定しない（不変条件 1 参照）。
pub fn hydration_attrs(state: &AppState) -> Vec<(String, String)> {
    vec![
        (
            "data-hydrate-counter".to_string(),
            state.counter.to_string(),
        ),
        ("data-hydrate-draft".to_string(), state.draft.clone()),
        ("data-hydrate-items".to_string(), encode_items(&state.items)),
    ]
}

/// クライアント（WASM）側の責務: ルート要素から読み取った属性値を状態へ復元する。
///
/// `wasm-full`/`wasm-thin` の `hydrate()`（TASK-11.2/11.3 のスコープ）から
/// 呼ばれる想定。不正な `counter` 値（パース失敗）は `0` へフォールバック
/// し、panic しない（クライアント制御下の入力に対する DoS 耐性）。
pub fn state_from_hydration_attrs(counter: &str, draft: &str, items_joined: &str) -> AppState {
    let counter = counter.parse::<i64>().unwrap_or(0);
    let items = decode_items(items_joined);
    AppState {
        counter,
        draft: draft.to_string(),
        items,
    }
}

/// 状態から `Node` 木を構築する（純粋関数、DOM 非依存）。
///
/// `rws_app` と同じ「モード非依存の共通レンダリング関数」という設計方針を
/// インタラクティブコンポーネントにも適用したもの。SSR（[`render_html`]）
/// と CSR 再描画の双方から同一関数を呼び、出力の同一性を保証する。
pub fn render(state: &AppState) -> Node {
    render_with_root_attrs(state, vec![])
}

/// SSR 用: [`render`] と同じ木に、ハイドレーション用の状態属性
/// （[`hydration_attrs`]）をルート要素へ追加する。
///
/// クライアントはこの属性を読み取って WASM 内部状態を復元し、DOM を
/// 作り直さずにイベント配線のみ行う想定（「最小ハイドレーション」方針）。
pub fn render_for_hydration(state: &AppState) -> Node {
    let attrs = hydration_attrs(state);
    let extra: Vec<(&str, &str)> = attrs
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    render_with_root_attrs(state, extra)
}

/// [`render_for_hydration`] を経由して HTML 文字列を得るショートカット。
pub fn render_html_for_hydration(state: &AppState) -> String {
    rws_core::render(&render_for_hydration(state))
}

/// [`render`] / [`render_for_hydration`] 共通の木構築本体。
///
/// `extra_root_attrs` はルート要素へ追加する属性（ハイドレーション属性）。
/// テキスト・属性値はすべて `rws_core::text`/`el` の attrs 経由で出力する
/// ため、`rws_core::render` が既定エスケープを必ず適用する（不変条件 1）。
fn render_with_root_attrs(state: &AppState, extra_root_attrs: Vec<(&str, &str)>) -> Node {
    let counter_section = el(
        "div",
        vec![("data-testid", "counter")],
        vec![
            text(format!("カウント: {}", state.counter)),
            el(
                "button",
                vec![("data-action", "increment"), ("data-testid", "inc-btn")],
                vec![text("+1")],
            ),
            el(
                "button",
                vec![("data-action", "decrement"), ("data-testid", "dec-btn")],
                vec![text("-1")],
            ),
            el(
                "button",
                vec![("data-action", "reset"), ("data-testid", "reset-btn")],
                vec![text("リセット")],
            ),
        ],
    );

    let form_section = el(
        "div",
        vec![("data-testid", "form")],
        vec![
            el(
                "input",
                vec![
                    ("id", "draft-input"),
                    ("data-testid", "draft-input"),
                    ("value", &state.draft),
                ],
                vec![],
            ),
            el(
                "button",
                vec![("data-action", "add_item"), ("data-testid", "add-btn")],
                vec![text("追加")],
            ),
        ],
    );

    let items: Vec<Node> = state
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            li(
                vec![("data-idx", &i.to_string())],
                vec![
                    text(item.clone()),
                    el(
                        "button",
                        vec![
                            ("data-action", "remove_item"),
                            ("data-idx", &i.to_string()),
                            ("data-testid", "remove-btn"),
                        ],
                        vec![text("削除")],
                    ),
                ],
            )
        })
        .collect();
    let list_section = ul(vec![("data-testid", "item-list")], items);

    let mut root_attrs = vec![
        ("id", "interactive-root"),
        ("data-testid", "interactive-root"),
    ];
    root_attrs.extend(extra_root_attrs);
    el(
        "div",
        root_attrs,
        vec![counter_section, form_section, list_section],
    )
}

/// [`render`] を経由して HTML 文字列を得るショートカット
/// （`rws_core::render` との合成、各クレートでの重複を避ける）。
pub fn render_html(state: &AppState) -> String {
    rws_core::render(&render(state))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increment_and_decrement() {
        let mut s = AppState::new();
        s.increment();
        s.increment();
        s.decrement();
        assert_eq!(s.counter, 1);
    }

    #[test]
    fn reset_sets_counter_to_zero() {
        let mut s = AppState::new();
        s.increment();
        s.increment();
        s.reset_counter();
        assert_eq!(s.counter, 0);
    }

    #[test]
    fn add_item_trims_and_clears_draft() {
        let mut s = AppState::new();
        s.set_draft("  new item  ");
        s.add_item();
        assert_eq!(s.items.last().unwrap(), "new item");
        assert_eq!(s.draft, "");
    }

    #[test]
    fn add_item_ignores_empty_draft() {
        let mut s = AppState::new();
        let before = s.items.len();
        s.set_draft("   ");
        s.add_item();
        assert_eq!(s.items.len(), before);
    }

    #[test]
    fn remove_item_by_index() {
        let mut s = AppState::new();
        s.items.push("second".into());
        s.remove_item(0);
        assert_eq!(s.items, vec!["second".to_string()]);
    }

    #[test]
    fn remove_item_out_of_range_is_noop() {
        let mut s = AppState::new();
        let before = s.items.clone();
        s.remove_item(99);
        assert_eq!(s.items, before);
    }

    #[test]
    fn dispatch_routes_actions() {
        let mut s = AppState::new();
        dispatch(&mut s, "increment", "");
        dispatch(&mut s, "set_draft", "hello");
        dispatch(&mut s, "add_item", "");
        assert_eq!(s.counter, 1);
        assert_eq!(s.items.last().unwrap(), "hello");
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut s = AppState::new();
        let before = s.clone();
        dispatch(&mut s, "no_such_action", "payload");
        assert_eq!(s, before);
    }

    #[test]
    fn render_reflects_state() {
        let mut s = AppState::new();
        s.increment();
        let html = render_html(&s);
        assert!(html.contains("カウント: 1"));
        assert!(html.contains("最初の項目"));
    }

    #[test]
    fn hydration_roundtrip_preserves_state() {
        let mut s = AppState::new();
        s.increment();
        s.increment();
        s.items.push("second".to_string());
        s.set_draft("draft text");

        let attrs = hydration_attrs(&s);
        let map: std::collections::HashMap<_, _> = attrs.into_iter().collect();
        let restored = state_from_hydration_attrs(
            &map["data-hydrate-counter"],
            &map["data-hydrate-draft"],
            &map["data-hydrate-items"],
        );
        assert_eq!(restored, s);
    }

    /// ハイドレーションのエンコード/デコードは、アイテム文字列に区切り文字
    /// （`\u{1f}`）やエスケープ文字（`\`）そのものが含まれていてもラウンド
    /// トリップの正しさを保つこと（項目境界の偽装＝データ注入ができないことの回帰確認）。
    #[test]
    fn hydration_roundtrip_survives_separator_and_backslash_in_item_text() {
        let mut s = AppState::new();
        s.items = vec![
            "separator:\u{1f}here".to_string(),
            "backslash:\\here".to_string(),
            "both:\\\u{1f}mixed".to_string(),
            "plain item".to_string(),
        ];
        s.set_draft("draft");

        let attrs = hydration_attrs(&s);
        let map: std::collections::HashMap<_, _> = attrs.into_iter().collect();
        let restored = state_from_hydration_attrs(
            &map["data-hydrate-counter"],
            &map["data-hydrate-draft"],
            &map["data-hydrate-items"],
        );
        assert_eq!(restored, s);
    }

    #[test]
    fn state_from_hydration_attrs_falls_back_on_invalid_counter() {
        // クライアント制御下になり得る属性値のパース失敗は panic せず
        // 既定値へフォールバックする（不変条件 4）。
        let restored = state_from_hydration_attrs("not-a-number", "", "");
        assert_eq!(restored.counter, 0);
    }

    #[test]
    fn render_escapes_item_text() {
        // REQ-1 の既定エスケープが本クレートでも維持されることを確認する。
        let mut s = AppState::new();
        s.set_draft("<script>alert(1)</script>");
        s.add_item();
        let html = render_html(&s);
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;alert"));
    }

    #[test]
    fn render_for_hydration_embeds_state_attrs_and_matches_render_dom() {
        let mut s = AppState::new();
        s.increment();
        let ssr_html = render_html_for_hydration(&s);
        assert!(ssr_html.contains(r#"data-hydrate-counter="1""#));
        assert!(ssr_html.contains(r#"data-hydrate-items="最初の項目""#));

        // ハイドレーション属性を除けば、CSR（render_html）と同一の DOM 構造を持つ
        // （サーバーが出す本文とクライアントが後で描画する本文が一致することの保証）。
        let csr_html = render_html(&s);
        assert!(ssr_html.contains("カウント: 1"));
        assert_eq!(
            ssr_html.replace(
                r#" data-hydrate-counter="1" data-hydrate-draft="" data-hydrate-items="最初の項目""#,
                ""
            ),
            csr_html
        );
    }
}
