//! `rws-interactive`: 状態管理コア（外部依存は `rws-core` のみ）。
//!
//! DOM/`wasm-bindgen` 非依存の状態機械とハイドレーション契約を提供する。
//! `rws-wasm-full`（TASK-11.2、イベント→dispatch→再描画）と `rws-server`
//! （SSR、ハイドレーション属性付き HTML 出力）が本クレートの型・トレイトを
//! 共有して呼び出す前提であり、PoC-5
//! （`docs/spec/03-poc/wasm-runtime-split/interactive/src/lib.rs`）の
//! カウンター・フォーム・動的リスト固有の具象実装（`AppState`/`dispatch`/
//! `hydration_attrs` 等）を、アプリ非依存の汎用 API（[`Component`]/[`Hydrate`]
//! トレイト）へ一般化したものである。
//!
//! 本ファイルは TASK-11.1a（#70、`docs/interactive-api.md`）が確定した
//! `Component`/`Hydrate` トレイト・`dispatch`/`codec`/`render_for_hydration`
//! の凍結シグネチャに従い、TASK-11.1b（#71）で関数本体を実装し、
//! TASK-11.1c（#72）でテストスイート（ラウンドトリップ・XSS 回帰・
//! `forbid` 検証・状態遷移網羅）を整備したものである。
//!
//! # 本クレートの不変条件（REQ-1・REQ-2・REQ-11、`docs/interactive-api.md` 第 6 節）
//!
//! 1. [`Component::view`] の出力は `rws_core::Node` のみであり、
//!    `rws_core::render()` の既定エスケープを必ず経由する。**本クレート内では
//!    `raw_html()` を使用しない**（新たなエスケープ迂回経路を作らない）。
//! 2. ハイドレーション属性値はレンダリング時に `rws-core` の属性エスケープで
//!    保護され、復元時（[`Hydrate::from_hydration_attrs`]）は「データ」として
//!    のみ扱い HTML として解釈しない。
//! 3. [`Hydrate::from_hydration_attrs`] は改ざんされうるクライアント入力を
//!    前提に、panic せず [`Result`] で失敗（[`HydrateError`]）を返す。
//! 4. 未知アクション名の dispatch は no-op とし、状態を変更しない（安全側
//!    フォールバック）。カウンターの `increment`/`decrement` は
//!    `saturating_add`/`saturating_sub` を用い、ハイドレーション経由で
//!    復元された `i64::MAX`/`i64::MIN` に対しても overflow panic しない
//!    （クライアント制御下の入力に対する DoS 耐性、不変条件 4 の一部）。
//! 5. codec（`docs/interactive-api.md` 第 3 節）は区切り文字・エスケープ
//!    文字を含む入力でもラウンドトリップが成立する方式（Unit Separator +
//!    バックスラッシュエスケープ）を採用する。
//! 6. **`unsafe` コード禁止**: `#![forbid(unsafe_code)]` によりクレート全体で
//!    機械的に禁止する。
//! 7. **外部依存は `rws-core`（path）のみ**: `interactive/Cargo.toml` の
//!    `[dependencies]` にサードパーティクレートを追加しない。

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use rws_core::{el, li, text, ul, Node};

/// アプリ状態と描画・遷移を結ぶ中核トレイト。
///
/// PoC-5 の `AppState`/`dispatch`/`render`（`docs/spec/03-poc/wasm-runtime-split/
/// interactive/src/lib.rs`）を一般化したもの。`view()` は `rws-core` の
/// ノード木 API のみを使う純粋関数として実装すること
/// （`docs/component-api.md` REQ-5 の「コンポーネントは通常の Rust 関数」
/// 規約と、本トレイトが導入する状態機械の抽象は独立した関心事であり、
/// 矛盾しない。`docs/interactive-api.md` 第 3 節・第 4 節・判断 1 参照）。
///
/// `rws-wasm-full`（TASK-11.2）が呼ぶ最小面はここで確定する
/// （`decode_action`/`update`/`view`）。
pub trait Component {
    /// 型付きアクション。WASM 境界の文字列 dispatch（`name`/`payload`）とは
    /// [`Component::decode_action`] で接続する。
    type Action;

    /// アクションを適用して状態を遷移させる（純粋な状態遷移。panic しない）。
    fn update(&mut self, action: Self::Action);

    /// 状態から `rws_core::Node` 木を構築する純粋関数。
    ///
    /// `rws_core::render()` の既定エスケープを必ず経由させるため、
    /// 戻り値は必ず `rws_core::Node` のみとし、HTML 文字列や DOM 型を
    /// 直接返さない（本クレートの不変条件 1）。
    fn view(&self) -> rws_core::Node;

    /// WASM 境界の `(name, payload)` 文字列を型付きアクションへ復号する。
    ///
    /// 未知のアクション名は `None` を返す（安全側 no-op、本クレートの
    /// 不変条件 4）。呼び出し元（[`dispatch`]）はこの結果を使って状態変更
    /// の要否を判断する。
    fn decode_action(name: &str, payload: &str) -> Option<Self::Action>;
}

/// SSR ↔ WASM のハイドレーション契約。
///
/// `hydration_attrs` はサーバー側（SSR）の責務、`from_hydration_attrs` は
/// クライアント側（WASM）の責務を表す。属性値はクライアント側で改ざん
/// されうる信頼できない入力として扱い、`from_hydration_attrs` は panic
/// せず [`Result`] で失敗を返す（本クレートの不変条件 3）。
///
/// [`Component`] とは独立したトレイトであり、SSR ハイドレーションを必要
/// としないコンポーネントは本トレイトを実装しなくてよい
/// （`docs/interactive-api.md` 第 3.1 節）。
pub trait Hydrate: Sized {
    /// 状態を `data-hydrate-*` 属性列へエンコードする（サーバー側責務）。
    ///
    /// 属性名は [`HYDRATE_ATTR_PREFIX`] を接頭辞とする規約に従うこと。
    fn hydration_attrs(&self) -> Vec<(String, String)>;

    /// 属性列から状態を復元する（クライアント側責務）。
    ///
    /// # Errors
    ///
    /// 属性が欠落している場合、または値の形式が不正な場合に
    /// [`HydrateError`] を返す。panic しない。
    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError>;
}

/// ハイドレーション属性名の接頭辞規約。
///
/// PoC-5 実績（`data-hydrate-counter`/`data-hydrate-draft`/
/// `data-hydrate-items`）を標準化したもの。個々の属性名はコンポーネント
/// ごとに異なるため、共有すべき接頭辞のみを定数として公開する
/// （`docs/interactive-api.md` 第 4 節・判断 3）。
pub const HYDRATE_ATTR_PREFIX: &str = "data-hydrate-";

/// [`Hydrate::from_hydration_attrs`] の失敗を表す。
///
/// クライアント側で改ざんされうる入力（DOM 属性値）を扱うための型であり、
/// 呼び出し側は panic ではなくこの型を介してエラーハンドリングする
/// （`.claude/rules/coding-rust.md` のエラーハンドリング規約）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HydrateError {
    /// 復元に必要な属性が見つからなかった。値は欠落した属性名。
    MissingAttr(String),
    /// 属性は存在するが値の形式が不正だった（例: 数値パース失敗）。
    InvalidValue {
        /// 不正な値を持っていた属性名。
        attr: String,
        /// 不正と判定した理由（内部実装詳細・機微情報は含めない）。
        reason: String,
    },
}

impl core::fmt::Display for HydrateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HydrateError::MissingAttr(attr) => {
                write!(f, "missing hydration attribute: {attr}")
            }
            HydrateError::InvalidValue { attr, reason } => {
                write!(f, "invalid hydration attribute value for {attr}: {reason}")
            }
        }
    }
}

impl std::error::Error for HydrateError {}

/// アクション名と引数文字列から [`Component`] を更新する共通ディスパッチャ。
///
/// `wasm-full`（Rust 側イベントハンドラ）・`wasm-thin`（JS グルーが呼ぶ wasm
/// 関数）・ネイティブ計測（`bench`）のいずれからも同一関数を呼び出す想定の
/// 境界設計であり、追加のシリアライズ依存を必要としない（PoC-5 実証済み）。
/// [`Component::decode_action`] が `None` を返す場合（未知のアクション名・
/// 復号失敗）は状態を変更せず `false` を返す（本クレートの不変条件 4）。
pub fn dispatch<C: Component>(component: &mut C, name: &str, payload: &str) -> bool {
    match C::decode_action(name, payload) {
        Some(action) => {
            component.update(action);
            true
        }
        None => false,
    }
}

/// `Component::view()` のルート要素へ [`Hydrate::hydration_attrs`] を追加した
/// `Node` を返す SSR 用ヘルパ。
///
/// クライアントはこの属性を読み取って WASM 内部状態を復元し、DOM を
/// 作り直さずにイベント配線のみ行う想定（「最小ハイドレーション」方針）。
/// `view()` の戻り値が `Node::Element` でない場合（`Text`/`RawHtml` を直接
/// 返す実装）は属性を付与できないため、`view()` の戻り値をそのまま返す
/// （`docs/interactive-api.md` 第 4 節・判断 7。panic ではなく安全側フォールバック）。
pub fn render_for_hydration<C: Component + Hydrate>(component: &C) -> Node {
    let hydrate_attrs = component.hydration_attrs();
    match component.view() {
        Node::Element {
            tag,
            mut attrs,
            children,
        } => {
            attrs.extend(hydrate_attrs);
            Node::Element {
                tag,
                attrs,
                children,
            }
        }
        other => other,
    }
}

/// [`render_for_hydration`] を経由して HTML 文字列を得るショートカット
/// （`rws_core::render` との合成、各呼び出し元での重複を避ける）。
pub fn render_html_for_hydration<C: Component + Hydrate>(component: &C) -> String {
    rws_core::render(&render_for_hydration(component))
}

/// [`Component::view`] を経由して HTML 文字列を得るショートカット。
pub fn render_html<C: Component>(component: &C) -> String {
    rws_core::render(&component.view())
}

/// ハイドレーション属性値のエンコード/デコード（外部依存ゼロの codec）。
///
/// Unit Separator（`\u{1f}`）区切り＋バックスラッシュエスケープにより、
/// JSON 等の追加クレートなしで複数値を 1 属性値へエンコードする
/// （PoC-5 実証方式、REQ-11 受け入れ基準「追加の JSON 等の依存なしに
/// 成立すること」、`docs/interactive-api.md` 第 3 節の凍結シグネチャ）。
pub mod codec {
    /// リスト項目を 1 属性値へ結合する際の区切り文字。
    ///
    /// 通常のテキスト入力に混入しない ASCII 制御文字（Unit Separator）を使う
    /// ことで、JSON 等の追加依存なしに複数項目をハイドレーション属性 1 個へ
    /// エンコードできる。
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
    ///
    /// 各項目の**前**に [`ITEM_SEP`] を 1 つ付与する方式を採る（項目間の区切りではなく
    /// 項目ごとの前置区切りとすることで、区切り文字の出現数が常に項目数と一致する）。
    /// これにより空リスト（出力 `""`）と「空文字列 1 件のみを含むリスト」（出力 `"\u{1f}"`）
    /// が異なるエンコードになり、[`decode_list`] との往復で区別できる。
    pub fn encode_list(items: &[String]) -> String {
        items
            .iter()
            .map(|s| format!("{ITEM_SEP}{}", escape_item(s)))
            .collect()
    }

    /// [`encode_list`] の逆変換（クライアント側責務）。
    ///
    /// 空文字列のみを空リストとして扱い、それ以外は先頭の区切り文字で区切って
    /// 各項目を復元する（`split` の最初の要素は先頭区切りより前の空文字列となるため読み捨てる）。
    pub fn decode_list(joined: &str) -> Vec<String> {
        if joined.is_empty() {
            Vec::new()
        } else {
            joined.split(ITEM_SEP).skip(1).map(unescape_item).collect()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn roundtrip_empty_and_single_empty_string_are_distinct() {
            assert_eq!(encode_list(&[]), "");
            assert_eq!(decode_list(""), Vec::<String>::new());

            let single_empty = vec![String::new()];
            let encoded = encode_list(&single_empty);
            assert_ne!(encoded, "");
            assert_eq!(decode_list(&encoded), single_empty);
        }

        #[test]
        fn roundtrip_with_separator_and_escape_char_in_items() {
            let items = vec!["a\u{1f}b".to_string(), "c\\d".to_string()];
            let encoded = encode_list(&items);
            assert_eq!(decode_list(&encoded), items);
        }
    }
}

/// アプリ状態: カウンター・フォーム入力（下書き）・動的リスト。
///
/// PoC-5 の最小インタラクティブコンポーネント（カウンター＋フォーム入力＋
/// 動的リスト更新）を [`Component`]/[`Hydrate`] トレイトの参照実装として
/// 製品状態に引き継いだもの。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    /// カウンター値。`increment`/`decrement`/`reset_counter` からのみ変更する。
    pub counter: i64,
    /// フォーム入力欄の下書き文字列。`add_item` 実行時に `items` へ確定し
    /// クリアされる。
    pub draft: String,
    /// 動的リストの項目群。ハイドレーション時は [`codec::encode_list`] で
    /// リスト全体を 1 属性値へエンコードする。
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

/// [`AppState`] の型付きアクション（[`Component::Action`]）。
///
/// WASM 境界の文字列 dispatch（`name`/`payload`）とは
/// [`AppState::decode_action`] で接続する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppAction {
    /// カウンターを 1 増やす。
    Increment,
    /// カウンターを 1 減らす。
    Decrement,
    /// カウンターを 0 へ戻す。
    Reset,
    /// フォーム入力欄の下書きを置き換える。
    SetDraft(String),
    /// 下書きを確定してリストへ追加する。
    AddItem,
    /// 指定インデックスの項目を削除する。
    RemoveItem(usize),
}

impl AppState {
    /// 既定状態（カウンター 0・下書き空・初期項目 1 件）を生成する。
    pub fn new() -> Self {
        Self::default()
    }

    /// カウンターを 1 増やす。
    ///
    /// ハイドレーション経由でクライアント制御下の `i64::MAX` まで復元
    /// され得るため、`saturating_add` により overflow panic を避ける
    /// （本クレートの不変条件 4、DoS 耐性）。
    pub fn increment(&mut self) {
        self.counter = self.counter.saturating_add(1);
    }

    /// カウンターを 1 減らす。
    ///
    /// [`AppState::increment`] と同様に `saturating_sub` を用いる。
    pub fn decrement(&mut self) {
        self.counter = self.counter.saturating_sub(1);
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

impl Component for AppState {
    type Action = AppAction;

    fn update(&mut self, action: AppAction) {
        match action {
            AppAction::Increment => self.increment(),
            AppAction::Decrement => self.decrement(),
            AppAction::Reset => self.reset_counter(),
            AppAction::SetDraft(value) => self.set_draft(&value),
            AppAction::AddItem => self.add_item(),
            AppAction::RemoveItem(index) => self.remove_item(index),
        }
    }

    fn view(&self) -> Node {
        render_with_root_attrs(self, vec![])
    }

    fn decode_action(name: &str, payload: &str) -> Option<AppAction> {
        match name {
            "increment" => Some(AppAction::Increment),
            "decrement" => Some(AppAction::Decrement),
            "reset" => Some(AppAction::Reset),
            "set_draft" => Some(AppAction::SetDraft(payload.to_string())),
            "add_item" => Some(AppAction::AddItem),
            "remove_item" => payload.parse::<usize>().ok().map(AppAction::RemoveItem),
            _ => None,
        }
    }
}

impl Hydrate for AppState {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}counter"),
                self.counter.to_string(),
            ),
            (format!("{HYDRATE_ATTR_PREFIX}draft"), self.draft.clone()),
            (
                format!("{HYDRATE_ATTR_PREFIX}items"),
                codec::encode_list(&self.items),
            ),
        ]
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |key: &str| -> Option<&str> {
            attrs
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.as_str())
        };

        let counter_key = format!("{HYDRATE_ATTR_PREFIX}counter");
        let draft_key = format!("{HYDRATE_ATTR_PREFIX}draft");
        let items_key = format!("{HYDRATE_ATTR_PREFIX}items");

        let counter_raw =
            find(&counter_key).ok_or_else(|| HydrateError::MissingAttr(counter_key.clone()))?;
        let counter = counter_raw
            .parse::<i64>()
            .map_err(|_| HydrateError::InvalidValue {
                attr: counter_key.clone(),
                reason: "value is not a valid i64".to_string(),
            })?;

        let draft = find(&draft_key)
            .ok_or_else(|| HydrateError::MissingAttr(draft_key.clone()))?
            .to_string();

        let items_raw =
            find(&items_key).ok_or_else(|| HydrateError::MissingAttr(items_key.clone()))?;
        let items = codec::decode_list(items_raw);

        Ok(AppState {
            counter,
            draft,
            items,
        })
    }
}

/// [`Component::view`]/[`render_for_hydration`] 共通の木構築本体。
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
    fn increment_saturates_at_i64_max_without_panicking() {
        let mut s = AppState {
            counter: i64::MAX,
            ..AppState::new()
        };
        s.increment();
        assert_eq!(s.counter, i64::MAX);
    }

    #[test]
    fn decrement_saturates_at_i64_min_without_panicking() {
        let mut s = AppState {
            counter: i64::MIN,
            ..AppState::new()
        };
        s.decrement();
        assert_eq!(s.counter, i64::MIN);
    }

    #[test]
    fn dispatch_unknown_action_is_noop() {
        let mut s = AppState::new();
        let before = s.clone();
        assert!(!dispatch(&mut s, "no_such_action", ""));
        assert_eq!(s, before);
    }

    #[test]
    fn hydrate_roundtrip_via_traits() {
        let s = AppState {
            counter: 42,
            draft: "draft".to_string(),
            items: vec!["a".to_string(), "b".to_string()],
        };
        let attrs = s.hydration_attrs();
        let restored = AppState::from_hydration_attrs(&attrs).expect("roundtrip should succeed");
        assert_eq!(s, restored);
    }

    #[test]
    fn from_hydration_attrs_reports_missing_attr() {
        let err = AppState::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr(format!("{HYDRATE_ATTR_PREFIX}counter"))
        );
    }

    #[test]
    fn from_hydration_attrs_reports_invalid_counter() {
        let attrs = vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}counter"),
                "not-a-number".to_string(),
            ),
            (format!("{HYDRATE_ATTR_PREFIX}draft"), String::new()),
            (format!("{HYDRATE_ATTR_PREFIX}items"), String::new()),
        ];
        let err = AppState::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}
