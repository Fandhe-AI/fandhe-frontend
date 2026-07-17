//! `rws-wasm-thin`: REQ-11（`docs/spec/04-requirements.md`）が定める「薄い JS
//! グルー方式」のオプトイン実装クレート（TASK-11.3、親イシュー #78）。
//!
//! WASM 完全方式（`rws-wasm-full`、既定実装）はイベント配線・DOM 更新まで
//! safe Rust の範囲に閉じ込めるのに対し、本クレートはそれらを JS グルー側へ
//! **意図的に**委ね、WASM 側は「文字列 in・文字列 out の純粋計算」
//! （状態遷移＋既定エスケープ済み HTML 描画）に限定する。この責務限定こそが
//! 本方式の制約であり利点でもある（軽量・既存 DOM ヘルパーとの段階的併用が
//! 可能）。制約の詳細な警告文書（JS グルー実装ガイド）は本タスク（11.3a）の
//! スコープ外であり、TASK-11.3b（#80）の `docs/opt-in-thin-js-glue.md` が担当する。
//!
//! # 2 層構造
//!
//! - **汎用層** [`ThinRuntime`]: `wasm-bindgen` 非依存・web-sys 非依存の
//!   純粋 Rust。`rws_interactive::Component`/`Hydrate` の任意の実装を束縛して
//!   使える汎用ランタイム。native の `cargo test` で検証できる
//!   （`wasm-full` の `Runtime<C>` 設計、`docs/wasm-full-architecture.md` と対をなす）。
//! - **境界層** [`demo`]: `#[wasm_bindgen]` エクスポートの参照実装。
//!   `wasm_bindgen` はジェネリクスをエクスポートできないため、
//!   `rws_interactive::AppState` に束縛した具象エクスポートを同梱する。
//!   自クレートを持つアプリケーションは、自コンポーネントに対して同型の
//!   エクスポート関数を自身のクレートに書く前提とする（本モジュールはその
//!   実装例・スモークテストの対象）。
//!
//! # 本クレートの不変条件（REQ-1・REQ-11）
//!
//! 1. WASM 境界を越えて JS グルーへ渡す HTML 文字列はすべて
//!    [`rws_interactive::Component::view`] → `rws_core::render()` の既定
//!    エスケープを経由したものに限る。**本クレート内で `raw_html()` を
//!    使用しない・HTML 文字列を `format!` 等で直接組み立てない**
//!    （`.claude/rules/coding-rust.md`）。
//! 2. JS グルーとの契約: グルーは戻り値の HTML 文字列を**そのまま**
//!    `innerHTML` へ設定するのみとし、連結・加工をしない。この保証が
//!    グルー側（JS）に委ねられること自体が薄いグルー方式固有の制約であり、
//!    詳細な注意点は TASK-11.3b（#80）の `docs/opt-in-thin-js-glue.md` に譲る。
//! 3. `action`/`payload`/ハイドレーション属性値はクライアント側で改ざん
//!    されうる信頼できない入力として扱う。未知アクションは
//!    `rws_interactive::dispatch` の仕様どおり no-op とし、ハイドレーション
//!    失敗は panic せず `Err`（[`rws_interactive::HydrateError`]）を返す。
//! 4. 自作コードに `unsafe` ブロックを書かない（`unsafe` は `wasm-bindgen` の
//!    FFI 依存クレート内部・自動生成コードに限定して許容する。
//!    `docs/unsafe-boundary.md` 参照）。

#![warn(missing_docs)]

use rws_interactive::{Component, Hydrate, HydrateError};

/// 薄い JS グルー方式向けの汎用ランタイム。
///
/// [`rws_interactive::Component`] を実装した任意のコンポーネント `C` を
/// 束縛し、WASM 境界（[`demo`] モジュール、またはアプリ側が用意する
/// 同型のエクスポート）から呼び出される「文字列 in・文字列 out」の
/// 純粋計算層を提供する。`wasm-bindgen`/web-sys のいずれにも依存しない
/// ため、native の `cargo test` でそのまま検証できる。
pub struct ThinRuntime<C: Component> {
    component: C,
}

impl<C: Component> ThinRuntime<C> {
    /// 与えられた初期状態でランタイムを構築する。
    pub fn new(component: C) -> Self {
        Self { component }
    }

    /// 現在の状態を描画した既定エスケープ済み HTML を返す。
    ///
    /// `rws_interactive::Component::view` が返す `rws_core::Node` を
    /// `rws_core::render()` に通す（不変条件 1）。JS グルーはこの戻り値を
    /// そのまま `innerHTML` へ設定する契約（不変条件 2）。
    pub fn html(&self) -> String {
        rws_core::render(&self.component.view())
    }

    /// WASM 境界の `(name, payload)` 文字列でアクションを適用し、
    /// 再描画後の HTML を返す。
    ///
    /// 未知のアクション名は `rws_interactive::dispatch` の仕様に従い
    /// no-op（状態を変更しない）。この場合も現在の状態を再描画した HTML を
    /// 返す（呼び出し側から見て「アクション適用後の最新表示」を返す契約を
    /// 一貫させるため）。
    pub fn apply(&mut self, name: &str, payload: &str) -> String {
        rws_interactive::dispatch(&mut self.component, name, payload);
        self.html()
    }

    /// 内部の component への参照を返す（テスト・デバッグ用途）。
    pub fn component(&self) -> &C {
        &self.component
    }
}

impl<C: Component + Hydrate> ThinRuntime<C> {
    /// SSR が出力したハイドレーション属性列から状態を復元する。
    ///
    /// 属性はクライアント側で改ざんされうる入力のため、`rws_interactive::
    /// Hydrate::from_hydration_attrs` を経由し panic しない。復元に失敗した
    /// 場合は本ランタイムの状態を変更せず `Err` を返す（呼び出し側は初期状態の
    /// まま CSR を継続するフォールバックを取れる。`docs/wasm-full-architecture.md`
    /// 判断 5 と同じ安全側戦略。不変条件 3）。
    ///
    /// # Errors
    ///
    /// 属性の欠落・値の形式不正時に [`HydrateError`] を返す。
    pub fn hydrate_from_attrs(&mut self, attrs: &[(String, String)]) -> Result<(), HydrateError> {
        let restored = C::from_hydration_attrs(attrs)?;
        self.component = restored;
        Ok(())
    }
}

/// 境界層（`#[wasm_bindgen]` エクスポート）の参照実装。
///
/// `wasm_bindgen` はジェネリクスをエクスポートできないため、[`ThinRuntime`]
/// を `rws_interactive::AppState` に束縛した具象エクスポートをここに用意する。
/// JS グルーはこのモジュールの 3 関数のみを呼び、イベント配線・DOM 更新
/// （`innerHTML` への反映）は自身の責務として行う（クレートドキュメント
/// 不変条件 2）。自コンポーネントを持つアプリケーションは、本モジュールと
/// 同型のエクスポート関数を自身のクレートに実装する前提。
pub mod demo {
    use super::ThinRuntime;
    use rws_interactive::AppState;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::wasm_bindgen;

    thread_local! {
        // WASM は基本的にシングルスレッド実行のため `thread_local!` +
        // `RefCell` で状態を保持する（PoC-5 実証方式。`wasm-full` の
        // `Runtime` 同様、複数スレッドからの同時アクセスは想定しない）。
        static RUNTIME: RefCell<ThinRuntime<AppState>> =
            RefCell::new(ThinRuntime::new(AppState::new()));
    }

    /// 初期状態を描画した HTML を返す（マウント直後、JS グルーが
    /// `innerHTML` へ設定する初回描画用）。
    #[wasm_bindgen]
    pub fn initial_html() -> String {
        RUNTIME.with(|r| r.borrow().html())
    }

    /// `action`/`payload` を適用し、再描画後の HTML を返す。
    ///
    /// JS グルーが `data-action`/`data-payload` 属性からそのまま読み取った
    /// 文字列を渡す想定（`rws_interactive::dispatch` の契約に一致）。
    #[wasm_bindgen]
    pub fn apply(action: &str, payload: &str) -> String {
        RUNTIME.with(|r| r.borrow_mut().apply(action, payload))
    }

    /// SSR が出力したハイドレーション属性から状態を復元する。
    ///
    /// `names`/`values` は同じ添字が対応する 2 本の配列として渡す
    /// （`wasm_bindgen` がタプルの `Vec` を直接エクスポートできないための
    /// 表現。JS 側は `Object.entries(dataset)` 等から 2 本の配列を組み立てて
    /// 渡す想定）。長さが一致しない場合、または復元に失敗した場合は状態を
    /// 変更せず `false` を返す（初期状態のまま CSR を継続する安全側
    /// フォールバック。不変条件 3）。
    #[wasm_bindgen]
    pub fn hydrate_from_attrs(names: Vec<String>, values: Vec<String>) -> bool {
        if names.len() != values.len() {
            return false;
        }
        let attrs: Vec<(String, String)> = names.into_iter().zip(values).collect();
        RUNTIME.with(|r| r.borrow_mut().hydrate_from_attrs(&attrs).is_ok())
    }
}
