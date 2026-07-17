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
//! # 本ファイルのスコープ（TASK-11.1a）
//!
//! 本ファイルは [`Component`]・[`Hydrate`] トレイトと [`HYDRATE_ATTR_PREFIX`]・
//! [`HydrateError`] の**定義のみ**を含む骨格である。`dispatch`・`codec`
//! モジュールの関数本体・`render_for_hydration` の実装は TASK-11.1b（#71）の
//! スコープであり、本タスクでは意図的に含めない
//! （設計詳細は `docs/interactive-api.md` 第 3〜4 節を参照）。
//! テストスイート（ラウンドトリップ・XSS 回帰・`forbid` 検証）は
//! TASK-11.1c（#72）のスコープ。
#![forbid(unsafe_code)]
#![warn(missing_docs)]

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
    /// 不変条件 4）。呼び出し元（`dispatch`、TASK-11.1b）はこの結果を
    /// 使って状態変更の要否を判断する。
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

/// ハイドレーション属性値のエンコード/デコード（外部依存ゼロの codec）。
///
/// Unit Separator（`\u{1f}`）区切り＋バックスラッシュエスケープにより、
/// JSON 等の追加クレートなしで複数値を 1 属性値へエンコードする
/// （PoC-5 実証方式、REQ-11 受け入れ基準「追加の JSON 等の依存なしに
/// 成立すること」）。関数本体は TASK-11.1b（#71）で実装する
/// （`docs/interactive-api.md` 第 3 節の凍結シグネチャに従うこと）。
pub mod codec {}
