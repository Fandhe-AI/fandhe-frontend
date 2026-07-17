//! `rws-interactive`: 状態管理コア（外部依存は `rws-core` のみ）。
//!
//! DOM/`wasm-bindgen` 非依存の状態機械とハイドレーション契約を提供する。
//! `rws-wasm-full`（TASK-11.2、イベント→dispatch→再描画）と `rws-server`
//! （SSR、ハイドレーション属性付き HTML 出力）が本クレートの型・トレイトを
//! 共有して呼び出す前提であり、PoC-5
//! （`docs/spec/03-poc/wasm-runtime-split/interactive/src/lib.rs`）の
//! カウンター・フォーム・動的リスト固有の具象実装（`AppState`/`dispatch`/
//! `hydration_attrs` 等）を、アプリ非依存の汎用 API へ一般化したものである。
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
//!    フォールバック）。
//! 5. codec（`docs/interactive-api.md` 第 3 節、TASK-11.1b で実装）は区切り
//!    文字・エスケープ文字を含む入力でもラウンドトリップが成立する方式
//!    （Unit Separator + バックスラッシュエスケープ）を採用する。
//! 6. **`unsafe` コード禁止**: `#![forbid(unsafe_code)]` によりクレート全体で
//!    機械的に禁止する。
//! 7. **外部依存は `rws-core`（path）のみ**: `interactive/Cargo.toml` の
//!    `[dependencies]` にサードパーティクレートを追加しない。
//!
//! # 本ファイルのスコープ（TASK-11.1b）
//!
//! 本ファイルは TASK-11.1a（#70、`docs/interactive-api.md`）が確定した
//! [`Component`]・[`Hydrate`] トレイトと [`HYDRATE_ATTR_PREFIX`]・
//! [`HydrateError`] の骨格に対し、`dispatch`・`codec` モジュールの関数本体・
//! `render_for_hydration` の実装を追加し、PoC-5 のカウンター・フォーム・
//! 動的リストコンポーネント（[`AppState`]）を [`Component`]/[`Hydrate`] の
//! 具象実装として提供する（設計詳細は `docs/interactive-api.md` 第 3〜4 節）。
//! テストスイートの本格網羅は TASK-11.1c（#72）のスコープ。本クレートに
//! 同梱するテストはスモーク〜回帰確認水準にとどめる。

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
    /// 不変条件 4）。呼び出し元（[`dispatch`]）はこの結果を使って状態変更の
    /// 要否を判断する。
    fn decode_action(name: &str, payload: &str) -> Option<Self::Action>;
}

/// WASM 境界の `(name, payload)` 文字列 dispatch ヘルパ。
///
/// `component.decode_action` で型付きアクションへの復号を試み、成功時のみ
/// `component.update` を呼んで状態を変更する。復号失敗（未知のアクション名）
/// では状態を変更せず `false` を返す（本クレートの不変条件 4、安全側
/// フォールバック）。`rws-wasm-full`/`rws-wasm-thin`（TASK-11.2/11.3）が
/// ブラウザの `data-action`/`data-payload` 属性から受け取った文字列を
/// そのまま渡す想定の境界関数。
pub fn dispatch<C: Component>(component: &mut C, name: &str, payload: &str) -> bool {
    match C::decode_action(name, payload) {
        Some(action) => {
            component.update(action);
            true
        }
        None => false,
    }
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

/// ハイドレーション属性値のエンコード/デコード（外部依存ゼロの codec）。
///
/// Unit Separator（`\u{1f}`）区切り＋バックスラッシュエスケープにより、
/// JSON 等の追加クレートなしで複数値を 1 属性値へエンコードする
/// （PoC-5 実証方式、REQ-11 受け入れ基準「追加の JSON 等の依存なしに
/// 成立すること」）。
pub mod codec {
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
    ///
    /// 各項目の**前**に [`ITEM_SEP`] を 1 つ付与する方式を採る（項目間の区切りではなく
    /// 項目ごとの前置区切りとすることで、区切り文字の出現数が常に項目数と一致する）。
    /// これにより空リスト（出力 `""`）と「空文字列 1 件のみを含むリスト」（出力 `"\u{1f}"`）
    /// が異なるエンコードになり、[`decode_list`] との往復で区別できる
    /// （Bugbot 指摘: 旧実装は `join` 方式のため両者が `""` に衝突していた）。
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
        fn distinguishes_empty_list_from_single_empty_item() {
            let empty: Vec<String> = Vec::new();
            let single_empty: Vec<String> = vec!["".to_string()];

            assert_ne!(encode_list(&empty), encode_list(&single_empty));
            assert_eq!(decode_list(&encode_list(&empty)), empty);
            assert_eq!(decode_list(&encode_list(&single_empty)), single_empty);
        }

        #[test]
        fn roundtrip_survives_separator_and_backslash_in_item_text() {
            let items = vec![
                "separator:\u{1f}here".to_string(),
                "backslash:\\here".to_string(),
                "both:\\\u{1f}mixed".to_string(),
                "plain item".to_string(),
            ];
            assert_eq!(decode_list(&encode_list(&items)), items);
        }
    }
}

/// [`Component::view`] のルート要素へ [`Hydrate::hydration_attrs`] を付与した
/// `Node` を返す SSR 用ヘルパ。
///
/// ルート要素が `Node::Element` でない場合（`Text`/`RawHtml` を直接返す
/// コンポーネント）は属性を付与できないため、`view()` の戻り値をそのまま
/// 返す（属性欠落を panic で扱わない、`docs/interactive-api.md` 第 4 節・
/// 判断 7）。
pub fn render_for_hydration<C: Component + Hydrate>(component: &C) -> Node {
    let view = component.view();
    let attrs = component.hydration_attrs();
    match view {
        Node::Element {
            tag,
            attrs: mut existing,
            children,
        } => {
            existing.extend(attrs);
            Node::Element {
                tag,
                attrs: existing,
                children,
            }
        }
        other => other,
    }
}

/// アプリ状態: カウンター・フォーム入力（下書き）・動的リスト。
///
/// PoC-5 の最小インタラクティブコンポーネント（カウンター＋フォーム入力＋
/// 動的リスト更新）をそのまま製品状態として引き継ぐ。[`Component`]/
/// [`Hydrate`] を実装し、`docs/interactive-api.md` が確定した API 表面の
/// 具体例として機能する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    /// カウンター値。[`Action::Increment`]/[`Action::Decrement`]/
    /// [`Action::Reset`] からのみ変更する。
    pub counter: i64,
    /// フォーム入力欄の下書き文字列。[`Action::AddItem`] 実行時に `items`
    /// へ確定しクリアされる。
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

impl AppState {
    /// 既定状態（カウンター 0・下書き空・初期項目 1 件）を生成する。
    pub fn new() -> Self {
        Self::default()
    }
}

/// [`AppState`] に対する型付きアクション。
///
/// WASM 境界の文字列 dispatch（`name`/`payload`）とは
/// [`AppState::decode_action`] で接続する（[`Component::decode_action`] 実装）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// カウンターを 1 増やす。
    Increment,
    /// カウンターを 1 減らす。
    Decrement,
    /// カウンターを 0 へ戻す。
    Reset,
    /// フォーム入力欄の下書きを置き換える。
    SetDraft(String),
    /// 下書きをリストへ追加する。
    AddItem,
    /// 指定インデックスの項目を削除する。
    RemoveItem(usize),
}

impl Component for AppState {
    type Action = Action;

    fn update(&mut self, action: Action) {
        match action {
            // `counter` はハイドレーション属性経由でクライアント制御下の
            // 極端な値（i64::MAX/MIN）から復元されうる（`from_hydration_attrs`）。
            // 素朴な `+`/`-` は debug ビルドで overflow panic するため、
            // `saturating_add`/`saturating_sub` により不変条件 4 相当の
            // 安全側フォールバック（panic しない）を維持する
            // （interactive/tests/hydration_codec.rs・state_management.rs の
            // 極端値回帰テスト参照）。
            Action::Increment => self.counter = self.counter.saturating_add(1),
            Action::Decrement => self.counter = self.counter.saturating_sub(1),
            Action::Reset => self.counter = 0,
            Action::SetDraft(value) => self.draft = value,
            Action::AddItem => {
                let trimmed = self.draft.trim();
                if !trimmed.is_empty() {
                    self.items.push(trimmed.to_string());
                    self.draft.clear();
                }
            }
            // 範囲外インデックスは何もしない（安全側フォールバック）。
            Action::RemoveItem(index) => {
                if index < self.items.len() {
                    self.items.remove(index);
                }
            }
        }
    }

    fn view(&self) -> Node {
        render_with_root_attrs(self, vec![])
    }

    fn decode_action(name: &str, payload: &str) -> Option<Action> {
        match name {
            "increment" => Some(Action::Increment),
            "decrement" => Some(Action::Decrement),
            "reset" => Some(Action::Reset),
            "set_draft" => Some(Action::SetDraft(payload.to_string())),
            "add_item" => Some(Action::AddItem),
            // 未知のインデックス表現（パース失敗）は復号失敗として扱い、
            // 呼び出し元（dispatch）で no-op になる（不変条件 4）。
            "remove_item" => payload.parse::<usize>().ok().map(Action::RemoveItem),
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
        let find = |name: &str| -> Result<&str, HydrateError> {
            attrs
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.as_str())
                .ok_or_else(|| HydrateError::MissingAttr(name.to_string()))
        };

        let counter_attr = format!("{HYDRATE_ATTR_PREFIX}counter");
        let draft_attr = format!("{HYDRATE_ATTR_PREFIX}draft");
        let items_attr = format!("{HYDRATE_ATTR_PREFIX}items");

        let counter_raw = find(&counter_attr)?;
        let counter = counter_raw
            .parse::<i64>()
            .map_err(|_| HydrateError::InvalidValue {
                attr: counter_attr.clone(),
                reason: "not a valid integer".to_string(),
            })?;
        let draft = find(&draft_attr)?.to_string();
        let items = codec::decode_list(find(&items_attr)?);

        Ok(AppState {
            counter,
            draft,
            items,
        })
    }
}

/// [`AppState::view`] / [`render_for_hydration`] 共通の木構築本体。
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
                            // `dispatch`/`decode_action` は WASM 境界の
                            // `data-action`/`data-payload` 属性契約（本ファイル冒頭
                            // doc コメント参照）に従って payload を読み取る。
                            // `data-idx` のみでは payload が空文字列のまま渡され
                            // `remove_item` の index パースが常に失敗する
                            // （Bugbot 指摘: Medium Severity）ため、同じ値を
                            // `data-payload` としても公開する。
                            ("data-payload", &i.to_string()),
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
        s.update(Action::Increment);
        s.update(Action::Increment);
        s.update(Action::Decrement);
        assert_eq!(s.counter, 1);
    }

    #[test]
    fn reset_sets_counter_to_zero() {
        let mut s = AppState::new();
        s.update(Action::Increment);
        s.update(Action::Increment);
        s.update(Action::Reset);
        assert_eq!(s.counter, 0);
    }

    #[test]
    fn add_item_trims_and_clears_draft() {
        let mut s = AppState::new();
        s.update(Action::SetDraft("  new item  ".to_string()));
        s.update(Action::AddItem);
        assert_eq!(s.items.last().unwrap(), "new item");
        assert_eq!(s.draft, "");
    }

    #[test]
    fn add_item_ignores_empty_draft() {
        let mut s = AppState::new();
        let before = s.items.len();
        s.update(Action::SetDraft("   ".to_string()));
        s.update(Action::AddItem);
        assert_eq!(s.items.len(), before);
    }

    #[test]
    fn remove_item_by_index() {
        let mut s = AppState::new();
        s.items.push("second".into());
        s.update(Action::RemoveItem(0));
        assert_eq!(s.items, vec!["second".to_string()]);
    }

    #[test]
    fn remove_item_out_of_range_is_noop() {
        let mut s = AppState::new();
        let before = s.items.clone();
        s.update(Action::RemoveItem(99));
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
        let dispatched = dispatch(&mut s, "no_such_action", "payload");
        assert!(!dispatched);
        assert_eq!(s, before);
    }

    #[test]
    fn render_reflects_state() {
        let mut s = AppState::new();
        s.update(Action::Increment);
        let html = rws_core::render(&s.view());
        assert!(html.contains("カウント: 1"));
        assert!(html.contains("最初の項目"));
    }

    #[test]
    fn hydration_roundtrip_preserves_state() {
        let mut s = AppState::new();
        s.update(Action::Increment);
        s.update(Action::Increment);
        s.items.push("second".to_string());
        s.update(Action::SetDraft("draft text".to_string()));

        let attrs = s.hydration_attrs();
        let restored = AppState::from_hydration_attrs(&attrs).expect("valid attrs");
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
        s.update(Action::SetDraft("draft".to_string()));

        let attrs = s.hydration_attrs();
        let restored = AppState::from_hydration_attrs(&attrs).expect("valid attrs");
        assert_eq!(restored, s);
    }

    /// Bugbot 指摘の回帰テスト: 空リストと「空文字列 1 件のみを含むリスト」は
    /// 旧実装では同一のエンコード（`""`）に衝突していた。両者が区別できることを確認する。
    #[test]
    fn hydration_roundtrip_distinguishes_empty_list_from_single_empty_item() {
        let mut empty_list_state = AppState::new();
        empty_list_state.items = Vec::new();
        let mut single_empty_state = AppState::new();
        single_empty_state.items = vec!["".to_string()];

        assert_ne!(
            empty_list_state.hydration_attrs(),
            single_empty_state.hydration_attrs()
        );
        assert_eq!(
            AppState::from_hydration_attrs(&empty_list_state.hydration_attrs()).unwrap(),
            empty_list_state
        );
        assert_eq!(
            AppState::from_hydration_attrs(&single_empty_state.hydration_attrs()).unwrap(),
            single_empty_state
        );
    }

    #[test]
    fn from_hydration_attrs_fails_on_invalid_counter() {
        // クライアント制御下になり得る属性値のパース失敗は panic せず
        // HydrateError を返す（不変条件 3）。
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

    #[test]
    fn from_hydration_attrs_fails_on_missing_attr() {
        let attrs: Vec<(String, String)> = Vec::new();
        let err = AppState::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::MissingAttr(_)));
    }

    /// Bugbot 指摘の回帰テスト（Medium Severity）: 削除ボタンは `dispatch`/
    /// `decode_action` の WASM 境界契約（`data-action`/`data-payload`）に従い
    /// `data-payload` 属性で index を公開しなければならない。`data-idx` のみ
    /// では payload が空になり `remove_item` が常に no-op になっていた。
    #[test]
    fn render_remove_button_exposes_index_via_data_payload() {
        let mut s = AppState::new();
        s.items.push("second".to_string());
        let html = rws_core::render(&s.view());
        assert!(html.contains(r#"data-payload="0""#));
        assert!(html.contains(r#"data-payload="1""#));
    }

    #[test]
    fn render_escapes_item_text() {
        // REQ-1 の既定エスケープが本クレートでも維持されることを確認する。
        let mut s = AppState::new();
        s.update(Action::SetDraft("<script>alert(1)</script>".to_string()));
        s.update(Action::AddItem);
        let html = rws_core::render(&s.view());
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;alert"));
    }

    #[test]
    fn render_for_hydration_embeds_state_attrs_and_matches_view_dom() {
        let mut s = AppState::new();
        s.update(Action::Increment);
        let ssr_html = rws_core::render(&render_for_hydration(&s));
        assert!(ssr_html.contains(r#"data-hydrate-counter="1""#));
        // encode_list は各項目の前に ITEM_SEP（\u{1f}）を付与するため、
        // 属性値は先頭に区切り文字を含む（「空リスト」との衝突回避、Bugbot 指摘対応）。
        assert!(ssr_html.contains("data-hydrate-items=\"\u{1f}最初の項目\""));

        // ハイドレーション属性を除けば、CSR（view）と同一の DOM 構造を持つ
        // （サーバーが出す本文とクライアントが後で描画する本文が一致することの保証）。
        let csr_html = rws_core::render(&s.view());
        assert!(ssr_html.contains("カウント: 1"));
        assert_eq!(
            ssr_html.replace(
                " data-hydrate-counter=\"1\" data-hydrate-draft=\"\" data-hydrate-items=\"\u{1f}最初の項目\"",
                ""
            ),
            csr_html
        );
    }

    #[test]
    fn render_for_hydration_returns_view_unchanged_for_non_element_root() {
        // Component::view のルートが Node::Element でない場合、
        // render_for_hydration は属性を付与できず view() をそのまま返す
        // （panic しない、docs/interactive-api.md 第 4 節・判断 7）。
        struct TextOnly;

        impl Component for TextOnly {
            type Action = ();
            fn update(&mut self, _action: ()) {}
            fn view(&self) -> Node {
                text("plain text root")
            }
            fn decode_action(_name: &str, _payload: &str) -> Option<()> {
                None
            }
        }

        impl Hydrate for TextOnly {
            fn hydration_attrs(&self) -> Vec<(String, String)> {
                vec![("data-hydrate-x".to_string(), "1".to_string())]
            }
            fn from_hydration_attrs(_attrs: &[(String, String)]) -> Result<Self, HydrateError> {
                Ok(TextOnly)
            }
        }

        let node = render_for_hydration(&TextOnly);
        assert_eq!(rws_core::render(&node), "plain text root");
    }
}
