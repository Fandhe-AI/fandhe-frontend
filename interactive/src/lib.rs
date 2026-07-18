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

    /// ネスト可能なハイドレーション値ツリー（イシュー #163）。
    ///
    /// `docs/hydration-state-format.md` が凍結した「数値・文字列・文字列配列
    /// のみ」制約（TASK-11.4a）は `AppState`/[`super::Hydrate`] の既存契約
    /// として変更しない。本型は、ネスト構造・オブジェクト等の複雑な状態を
    /// 扱いたいアプリの [`super::Hydrate`] 実装が**オプトインで**使う追加の
    /// 属性値表現であり、`docs/hydration-nested-state.md`（設計確定書）が
    /// 正の規範文書。JSON 等の外部シリアライズクレートには依存しない
    /// （REQ-11・本クレートの外部依存ゼロ制約を維持）。
    #[derive(Debug, Clone, PartialEq)]
    pub enum Value {
        /// 文字列値。
        Str(String),
        /// 整数値（`i64`）。
        Int(i64),
        /// 真偽値。
        Bool(bool),
        /// 順序付きリスト（要素は任意の [`Value`]、ネスト可）。
        List(Vec<Value>),
        /// 順序を保持したキー・値の一覧（`HashMap` は使わず、エンコードを
        /// 決定的に保つ。キーの重複チェックは行わない = アプリ側の責務）。
        Map(Vec<(String, Value)>),
    }

    /// [`decode_value`] が許容する最大ネスト深さ。
    ///
    /// `data-hydrate-*` 属性値は改ざんされうるクライアント入力であり、上限を
    /// 設けない場合は深くネストした `List`/`Map` の再帰デコードでスタックを
    /// 枯渇させられる（A05 相当の DoS、`docs/hydration-nested-state.md` 参照）。
    /// 32 段は通常のアプリ状態（ネストしたフォーム・設定オブジェクト等）を
    /// 十分許容しつつ、無制限の深さを弾く値として選定した。
    pub const MAX_VALUE_DEPTH: u32 = 32;

    /// [`decode_value`] の失敗を表す。
    ///
    /// `data-hydrate-*` 属性値（改ざんされうるクライアント入力）のデコードに
    /// 特有の失敗を表現する。[`into_hydrate_error`](ValueDecodeError::into_hydrate_error)
    /// で呼び出し側（アプリの [`super::Hydrate::from_hydration_attrs`] 実装）
    /// が扱う [`super::HydrateError`] へ変換できる。
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ValueDecodeError {
        /// 入力が空文字列だった（有効な [`Value`] エンコードは常に非空）。
        Empty,
        /// 先頭の型タグ文字が未知だった。
        UnknownTag(char),
        /// `Str`/`Map` キーの長さプレフィックスが非負整数としてパースできない、
        /// または宣言された長さが残り入力より大きい。
        InvalidLength,
        /// 長さプレフィックスが指し示すバイト範囲が UTF-8 として不正
        /// （マルチバイト文字の境界を跨いだ改ざん等）。
        InvalidUtf8,
        /// `Int` タグのペイロードが `i64` としてパースできなかった。
        InvalidInt,
        /// `Bool` タグのペイロードが `"0"`/`"1"` のいずれでもなかった。
        InvalidBool,
        /// `Map` のキー位置に文字列以外の値が来た（内部実装は常にキーを
        /// [`Value::Str`] としてエンコードするため、改ざん入力でのみ発生する）。
        InvalidMapKey,
        /// 入力が途中で終わっていた（終端記号 `e` や宣言された長さ分の
        /// バイト列に到達する前に入力が尽きた）。
        UnexpectedEnd,
        /// [`MAX_VALUE_DEPTH`] を超えるネストが検出された。
        DepthExceeded,
        /// トップレベルの値をデコードした後に余分なバイト列が残っていた。
        TrailingData,
    }

    impl core::fmt::Display for ValueDecodeError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                ValueDecodeError::Empty => write!(f, "empty value encoding"),
                ValueDecodeError::UnknownTag(tag) => write!(f, "unknown value tag: {tag:?}"),
                ValueDecodeError::InvalidLength => write!(f, "invalid length prefix"),
                ValueDecodeError::InvalidUtf8 => {
                    write!(f, "length-prefixed bytes are not valid utf-8")
                }
                ValueDecodeError::InvalidInt => write!(f, "invalid integer payload"),
                ValueDecodeError::InvalidBool => write!(f, "invalid boolean payload"),
                ValueDecodeError::InvalidMapKey => write!(f, "map key must be a string"),
                ValueDecodeError::UnexpectedEnd => write!(f, "unexpected end of input"),
                ValueDecodeError::DepthExceeded => {
                    write!(f, "nested value exceeds max depth ({MAX_VALUE_DEPTH})")
                }
                ValueDecodeError::TrailingData => write!(f, "trailing data after decoded value"),
            }
        }
    }

    impl std::error::Error for ValueDecodeError {}

    impl ValueDecodeError {
        /// アプリの [`super::Hydrate::from_hydration_attrs`] 実装が `?` で
        /// 使えるよう、属性名を添えて [`super::HydrateError::InvalidValue`]
        /// へ変換するヘルパ。エラーメッセージは英語（機微情報は含めない、
        /// `.claude/rules/japanese-style.md`・A09 相当の不変条件）。
        pub fn into_hydrate_error(self, attr: &str) -> super::HydrateError {
            super::HydrateError::InvalidValue {
                attr: attr.to_string(),
                reason: self.to_string(),
            }
        }
    }

    /// [`Value`] を 1 属性値へエンコードする（サーバー側責務）。
    ///
    /// 長さ明示型（netstring/Bencode 系）の再帰下降エンコードを採用する。
    ///
    /// | バリアント | 形式 |
    /// |-----------|------|
    /// | `Str(s)` | `s{バイト長}:{s}` |
    /// | `Int(i)` | `i{10進文字列}e` |
    /// | `Bool(b)` | `b0`/`b1` |
    /// | `List(items)` | `l{各要素のエンコード結果を連結}e` |
    /// | `Map(entries)` | `m{(キーの Str エンコード + 値のエンコード) を連結}e` |
    ///
    /// 子要素・キーの境界は「事前に宣言したバイト長」または明示的な終端
    /// 記号（`e`）で決定されるため、`escape_item`（`ITEM_SEP`/`ESCAPE_CHAR`
    /// を対象とする既存 codec のエスケープ）を一切使わない。これにより、
    /// エスケープ処理をネストの各段で繰り返し適用した結果バックスラッシュの
    /// 数が段数に対して指数的に増える問題（旧設計で判明した不具合。ネスト
    /// 済みの既エスケープ済み文字列を、さらに親レベルでエスケープすると
    /// バックスラッシュの出現数が毎段倍増し、深さ D で O(2^D) のサイズへ
    /// 発散する）を構造的に回避する。エンコード・デコードとも入力サイズに
    /// 対して線形時間・線形サイズで完結する。既存の [`encode_list`]/
    /// [`decode_list`]・`escape_item`/`unescape_item`（`AppState` 等が使用）
    /// は一切変更しない。
    pub fn encode_value(value: &Value) -> String {
        let mut out = String::new();
        encode_value_into(value, &mut out);
        out
    }

    /// [`encode_value`] の内部実装。`out` へ追記する形にすることで、ネスト
    /// した `List`/`Map` の子要素ごとに中間 `String` を確保しない
    /// （エンコードが常に線形時間・線形メモリで完結することの実装上の裏付け）。
    fn encode_value_into(value: &Value, out: &mut String) {
        match value {
            Value::Str(s) => {
                out.push('s');
                out.push_str(&s.len().to_string());
                out.push(':');
                out.push_str(s);
            }
            Value::Int(i) => {
                out.push('i');
                out.push_str(&i.to_string());
                out.push('e');
            }
            Value::Bool(b) => {
                out.push('b');
                out.push(if *b { '1' } else { '0' });
            }
            Value::List(items) => {
                out.push('l');
                for item in items {
                    encode_value_into(item, out);
                }
                out.push('e');
            }
            Value::Map(entries) => {
                out.push('m');
                for (key, val) in entries {
                    // キーは Value::Str と同じ長さプレフィックス形式でエンコード
                    // する（`decode_value` 側は「Str をデコードして文字列として
                    // 取り出す」共通経路をキー・値の双方に用いるため、専用の
                    // キー用フォーマットを別途持たない）。
                    encode_value_into(&Value::Str(key.clone()), out);
                    encode_value_into(val, out);
                }
                out.push('e');
            }
        }
    }

    /// [`encode_value`] の逆変換（クライアント側責務）。
    ///
    /// `data-hydrate-*` 属性値は改ざんされうるクライアント入力として扱い、
    /// 未知の型タグ・パース失敗・[`MAX_VALUE_DEPTH`] 超過・不正な UTF-8
    /// 境界のいずれでも panic せず [`ValueDecodeError`] を返す
    /// （`unwrap()`/`expect()`/`panic!` 不使用、`.claude/rules/coding-rust.md`）。
    /// 長さプレフィックスにより境界が事前に確定するため、デコードは
    /// 入力バイト長に対して線形時間で完結する。
    ///
    /// # Errors
    ///
    /// 入力が空・型タグが未知・長さプレフィックスや UTF-8 境界が不正・
    /// ペイロードの形式が不正・ネスト深さが [`MAX_VALUE_DEPTH`] を超える・
    /// 末尾に余分なバイト列が残る場合に [`ValueDecodeError`] を返す。
    pub fn decode_value(input: &str) -> Result<Value, ValueDecodeError> {
        if input.is_empty() {
            return Err(ValueDecodeError::Empty);
        }
        let mut cursor = ValueCursor {
            bytes: input.as_bytes(),
            pos: 0,
        };
        let value = cursor.decode(0)?;
        if cursor.pos != cursor.bytes.len() {
            return Err(ValueDecodeError::TrailingData);
        }
        Ok(value)
    }

    /// [`decode_value`] の再帰下降パーサ本体。バイト列上の読み取り位置
    /// （`pos`）を保持し、`Str`/`Map` キーの長さプレフィックスが指す
    /// バイト範囲を直接スライスすることで、文字列の再構築コピーを
    /// 最小限に抑える（各バイトは高々 1 回読み取られる）。
    struct ValueCursor<'a> {
        bytes: &'a [u8],
        pos: usize,
    }

    impl<'a> ValueCursor<'a> {
        fn peek(&self) -> Option<u8> {
            self.bytes.get(self.pos).copied()
        }

        fn advance(&mut self) -> Option<u8> {
            let b = self.peek()?;
            self.pos += 1;
            Some(b)
        }

        /// `terminator` に到達するまでの ASCII バイト列を読み取り、
        /// `terminator` 自身は消費して返り値には含めない。`terminator` に
        /// 到達せず入力が尽きた場合は `UnexpectedEnd`。
        fn read_until(&mut self, terminator: u8) -> Result<&'a [u8], ValueDecodeError> {
            let start = self.pos;
            loop {
                match self.peek() {
                    Some(b) if b == terminator => {
                        let slice = &self.bytes[start..self.pos];
                        self.pos += 1;
                        return Ok(slice);
                    }
                    Some(_) => self.pos += 1,
                    None => return Err(ValueDecodeError::UnexpectedEnd),
                }
            }
        }

        /// `Str`/`Map` キーの長さプレフィックス（先頭の型タグ `s` は既に
        /// 消費済みの前提）を読み、宣言されたバイト数ぶんの文字列を取り出す。
        /// 宣言された長さが残り入力より大きい場合・UTF-8 境界が不正な場合は
        /// panic せず `Err` を返す（改ざんされた長さプレフィックスによる
        /// パニック・範囲外読み取りを防ぐ）。
        fn read_length_prefixed_str(&mut self) -> Result<String, ValueDecodeError> {
            let len_bytes = self.read_until(b':')?;
            let len_str =
                core::str::from_utf8(len_bytes).map_err(|_| ValueDecodeError::InvalidLength)?;
            let len: usize = len_str
                .parse()
                .map_err(|_| ValueDecodeError::InvalidLength)?;
            let remaining = self.bytes.len().saturating_sub(self.pos);
            if len > remaining {
                return Err(ValueDecodeError::UnexpectedEnd);
            }
            let start = self.pos;
            let end = start + len;
            let slice = &self.bytes[start..end];
            let s = core::str::from_utf8(slice)
                .map_err(|_| ValueDecodeError::InvalidUtf8)?
                .to_string();
            self.pos = end;
            Ok(s)
        }

        /// `depth` は現在の再帰深さ（トップレベルは 0）。`List`/`Map` の
        /// 子要素を復元する前に [`MAX_VALUE_DEPTH`] を超えていないか確認
        /// してから再帰する（超過した入力でのスタックオーバーフローを
        /// `Err` で遮断する）。
        fn decode(&mut self, depth: u32) -> Result<Value, ValueDecodeError> {
            let tag = self.advance().ok_or(ValueDecodeError::UnexpectedEnd)?;
            match tag {
                b's' => Ok(Value::Str(self.read_length_prefixed_str()?)),
                b'i' => {
                    let digits = self.read_until(b'e')?;
                    let s =
                        core::str::from_utf8(digits).map_err(|_| ValueDecodeError::InvalidInt)?;
                    s.parse::<i64>()
                        .map(Value::Int)
                        .map_err(|_| ValueDecodeError::InvalidInt)
                }
                b'b' => match self.advance() {
                    Some(b'1') => Ok(Value::Bool(true)),
                    Some(b'0') => Ok(Value::Bool(false)),
                    Some(_) => Err(ValueDecodeError::InvalidBool),
                    None => Err(ValueDecodeError::UnexpectedEnd),
                },
                b'l' => {
                    let next_depth = depth
                        .checked_add(1)
                        .filter(|d| *d <= MAX_VALUE_DEPTH)
                        .ok_or(ValueDecodeError::DepthExceeded)?;
                    let mut items = Vec::new();
                    loop {
                        match self.peek() {
                            Some(b'e') => {
                                self.pos += 1;
                                break;
                            }
                            Some(_) => items.push(self.decode(next_depth)?),
                            None => return Err(ValueDecodeError::UnexpectedEnd),
                        }
                    }
                    Ok(Value::List(items))
                }
                b'm' => {
                    let next_depth = depth
                        .checked_add(1)
                        .filter(|d| *d <= MAX_VALUE_DEPTH)
                        .ok_or(ValueDecodeError::DepthExceeded)?;
                    let mut entries = Vec::new();
                    loop {
                        match self.peek() {
                            Some(b'e') => {
                                self.pos += 1;
                                break;
                            }
                            Some(_) => {
                                let key = match self.decode(next_depth)? {
                                    Value::Str(k) => k,
                                    _ => return Err(ValueDecodeError::InvalidMapKey),
                                };
                                let val = self.decode(next_depth)?;
                                entries.push((key, val));
                            }
                            None => return Err(ValueDecodeError::UnexpectedEnd),
                        }
                    }
                    Ok(Value::Map(entries))
                }
                other => Err(ValueDecodeError::UnknownTag(other as char)),
            }
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

        /// イシュー #163: `Value` codec の追加が既存 `encode_list` の出力・
        /// 挙動を一切変えないことの回帰確認（後方互換）。`Value` codec は
        /// 長さプレフィックス方式であり `encode_list`/`escape_item` を一切
        /// 呼ばないため、U+001E/U+001F を含む項目も既存のエスケープ表
        /// （バックスラッシュ・U+001F のみ）でそのまま扱われる。
        #[test]
        fn value_codec_addition_does_not_change_encode_list_output() {
            let items = vec![
                "plain".to_string(),
                "with-\u{1e}-record-sep".to_string(),
                "with-\u{1f}-unit-sep".to_string(),
                "with-\\-backslash".to_string(),
            ];
            let before = encode_list(&items);
            // `Value` codec 呼び出し後も `encode_list` の出力は不変であること
            let _ = encode_value(&Value::Str("unrelated".to_string()));
            let after = encode_list(&items);
            assert_eq!(before, after);
            assert_eq!(decode_list(&after), items);
        }

        #[test]
        fn value_roundtrip_str_int_bool() {
            for value in [
                Value::Str(String::new()),
                Value::Str("hello".to_string()),
                Value::Int(0),
                Value::Int(i64::MAX),
                Value::Int(i64::MIN),
                Value::Bool(true),
                Value::Bool(false),
            ] {
                let encoded = encode_value(&value);
                assert_eq!(decode_value(&encoded).unwrap(), value);
            }
        }

        #[test]
        fn value_roundtrip_nested_list_and_map() {
            let value = Value::Map(vec![
                (
                    "user".to_string(),
                    Value::Map(vec![
                        ("name".to_string(), Value::Str("Alice".to_string())),
                        (
                            "tags".to_string(),
                            Value::List(vec![
                                Value::Str("admin".to_string()),
                                Value::Str("beta".to_string()),
                            ]),
                        ),
                        ("active".to_string(), Value::Bool(true)),
                    ]),
                ),
                (
                    "counters".to_string(),
                    Value::List(vec![Value::Int(1), Value::Int(-2), Value::Int(3)]),
                ),
            ]);
            let encoded = encode_value(&value);
            assert_eq!(decode_value(&encoded).unwrap(), value);
        }

        #[test]
        fn value_roundtrip_empty_list_and_map() {
            assert_eq!(
                decode_value(&encode_value(&Value::List(vec![]))).unwrap(),
                Value::List(vec![])
            );
            assert_eq!(
                decode_value(&encode_value(&Value::Map(vec![]))).unwrap(),
                Value::Map(vec![])
            );
        }

        /// 項目文字列に区切り文字（U+001F）・エスケープ文字（`\`）・
        /// コロン・日本語・絵文字が混在してもラウンドトリップが成立する
        /// こと（長さプレフィックス方式では文字列内容に対する特別扱いが
        /// 一切不要であることの回帰確認）。
        #[test]
        fn value_roundtrip_survives_adversarial_strings() {
            let value = Value::List(vec![
                Value::Str("separator:\u{1f}here".to_string()),
                Value::Str("backslash:\\here".to_string()),
                Value::Str("both:\\\u{1f}mixed".to_string()),
                Value::Str("colon:in:string".to_string()),
                Value::Str("日本語と絵文字🎉".to_string()),
                Value::Map(vec![(
                    "key:\u{1f}with-sep".to_string(),
                    Value::Str("val\\ue".to_string()),
                )]),
            ]);
            let encoded = encode_value(&value);
            assert_eq!(decode_value(&encoded).unwrap(), value);
        }

        #[test]
        fn decode_value_rejects_empty_input() {
            assert_eq!(decode_value(""), Err(ValueDecodeError::Empty));
        }

        #[test]
        fn decode_value_rejects_unknown_tag() {
            assert_eq!(
                decode_value("xhello"),
                Err(ValueDecodeError::UnknownTag('x'))
            );
        }

        #[test]
        fn decode_value_rejects_invalid_int_payload() {
            assert_eq!(
                decode_value("inot-a-numbere"),
                Err(ValueDecodeError::InvalidInt)
            );
        }

        #[test]
        fn decode_value_rejects_invalid_bool_payload() {
            assert_eq!(decode_value("b2"), Err(ValueDecodeError::InvalidBool));
        }

        #[test]
        fn decode_value_rejects_map_with_non_string_key() {
            // Map のキー位置に整数（本来は文字列のみ許容）を仕込んだ改ざん入力。
            let broken = format!(
                "m{}{}e",
                encode_value(&Value::Int(1)),
                encode_value(&Value::Int(2))
            );
            assert_eq!(decode_value(&broken), Err(ValueDecodeError::InvalidMapKey));
        }

        #[test]
        fn decode_value_rejects_length_prefix_exceeding_remaining_input() {
            // 宣言された長さ（100 バイト）が実際に残っている入力より大きい。
            assert_eq!(
                decode_value("s100:short"),
                Err(ValueDecodeError::UnexpectedEnd)
            );
        }

        #[test]
        fn decode_value_rejects_missing_terminator() {
            assert_eq!(decode_value("i42"), Err(ValueDecodeError::UnexpectedEnd));
            assert_eq!(decode_value("l"), Err(ValueDecodeError::UnexpectedEnd));
        }

        #[test]
        fn decode_value_rejects_trailing_data() {
            assert_eq!(
                decode_value("i1egarbage"),
                Err(ValueDecodeError::TrailingData)
            );
        }

        #[test]
        fn decode_value_rejects_excessive_nesting_without_panicking() {
            // MAX_VALUE_DEPTH を超える深さのネストしたリストを構築し、
            // panic（スタックオーバーフロー含む）せず Err を返すことを確認する。
            // 長さプレフィックス方式は再帰段ごとにエスケープを繰り返さない
            // ため、エンコード結果のサイズは深さに対して線形にとどまる。
            let mut value = Value::Int(0);
            for _ in 0..(MAX_VALUE_DEPTH + 5) {
                value = Value::List(vec![value]);
            }
            let encoded = encode_value(&value);
            assert_eq!(decode_value(&encoded), Err(ValueDecodeError::DepthExceeded));
        }

        #[test]
        fn decode_value_accepts_nesting_at_the_depth_limit() {
            let mut value = Value::Int(42);
            for _ in 0..MAX_VALUE_DEPTH {
                value = Value::List(vec![value]);
            }
            let encoded = encode_value(&value);
            assert_eq!(decode_value(&encoded).unwrap(), value);
        }

        /// エンコード結果のサイズがネスト深さに対して線形であることの回帰
        /// 確認（旧設計の指数的サイズ増大バグの再発防止）。深さ 32 段の
        /// リストのエンコード結果が数百バイト程度に収まることを確認する
        /// （指数的増大であれば `2^32` バイト超になり得る）。
        #[test]
        fn encoded_size_grows_linearly_with_nesting_depth() {
            let mut value = Value::Int(0);
            for _ in 0..MAX_VALUE_DEPTH {
                value = Value::List(vec![value]);
            }
            let encoded = encode_value(&value);
            assert!(
                encoded.len() < 500,
                "ネスト深さ {MAX_VALUE_DEPTH} のエンコード結果が想定より大きい (指数的増大の疑い): len={}",
                encoded.len()
            );
        }

        #[test]
        fn decode_value_does_not_panic_on_multibyte_leading_char() {
            // 攻撃者が非 ASCII のマルチバイト文字を先頭（本来は型タグ位置）に
            // 置いた場合でも、バイト境界パニックせず Err を返すこと。
            // マルチバイト文字の最初のバイトは有効な型タグ（ASCII）と一致
            // しないため、UnknownTag として扱われる。
            let first_byte = "日本語".as_bytes()[0];
            assert_eq!(
                decode_value("日本語"),
                Err(ValueDecodeError::UnknownTag(first_byte as char))
            );
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
        render_with_root_attrs(self)
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

/// [`AppState::view`] の木構築本体。
///
/// ハイドレーション属性はここでは付与しない。SSR 側の付与は
/// [`render_for_hydration`] が `Component::view` の戻り値（本関数の結果）を
/// 受け取った後、ルート要素へ後付けで行う責務分離になっている
/// （Bugbot 指摘: 旧実装は本関数に未使用の `extra_root_attrs` 引数を持ち、
/// ハイドレーション属性共有経路であるかのような doc コメントを付けていたが、
/// 唯一の呼び出し元（[`AppState::view`]）は常に空 vec を渡しており実質死んで
/// いた。引数を削除し、責務分離の実態に合わせて doc コメントを修正する）。
/// テキスト・属性値はすべて `rws_core::text`/`el` の attrs 経由で出力する
/// ため、`rws_core::render` が既定エスケープを必ず適用する（不変条件 1）。
fn render_with_root_attrs(state: &AppState) -> Node {
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

    let root_attrs = vec![
        ("id", "interactive-root"),
        ("data-testid", "interactive-root"),
    ];
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
