//! Clipboard（値コピー・コピー済み表示）headless コンポーネント
//! （イシュー #773、親トラッキング #520）。
//!
//! ark-ui の Clipboard
//!（`.claude/skills/ark-ui/references/components/display/clipboard.md`）/
//! chakra-ui の Clipboard を参考に、Root / Label / Control / Input / Trigger /
//! Indicator / ValueText の 7 anatomy パーツと、コピー済みかどうかの
//! 2 値状態機械 [`Clipboard`] を提供する。
//!
//! # `data-copied` 存在属性（[`crate::state`] を使わない理由）
//!
//! Clipboard の状態は「コピー済みかどうか」の 2 値だが、
//! [`crate::state::Disclosure`]（`"open"`/`"closed"`）にも
//! [`crate::state::Checkable`]（`"checked"`/`"unchecked"`）にも意味的に
//! 写像しない（コピー済み表示は値語彙ではなく存在属性 `data-copied` で
//! 表現するのが ark-ui/chakra-ui の慣習）。[`crate::avatar::Avatar`]/
//! [`crate::switch::Switch`] と同様、本モジュールは
//! [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装し、Phase 1 が確立した
//! dispatch 契約（未知アクション no-op）・fail-closed hydration という
//! **統合様式**にのみ準拠する。
//!
//! # `value`（コピー対象値）は状態機械に持たせない
//!
//! [`avatar::image`](crate::avatar::image) の `src`/`alt` と同様、コピー
//! 対象の `value` は呼び出し側が各パーツ関数へ都度渡す描画パラメータであり、
//! [`Clipboard`] 状態機械のフィールドには含めない（実際にクリップボードへ
//! 書き込む値は [`root`] が出力する `data-value` 属性からクライアント側
//! （`fandhe-frontend-wasm-full`）が読み取る契約、モジュール冒頭
//! 「呼び出し文脈」節参照）。状態機械が保持するのはコピー成功/リセットの
//! 2 値のみである。
//!
//! # コピー完了後の自動リセット（タイムアウト）は本モジュールの責務外
//!
//! ark-ui の Clipboard は `timeout`（既定 3000ms）経過後に自動で
//! copied 表示を解除する。時間経過という副作用は SSR/hydration の純粋な
//! 状態機械が持つべき責務ではないため、本モジュールは
//! `"clipboard:copy"`/`"clipboard:reset"` の 2 アクションのみを提供し、
//! タイマー予約・解除は `fandhe-frontend-wasm-full` の
//! `headless_clipboard`（イシュー #773、PR #816 で実装済み。既定 3000ms の
//! `schedule_reset` がタイマーを予約・解除する）が担う
//! クライアント配線層の責務とする（[`crate::tooltip::Tooltip`] の
//! 開閉遅延と同型の責務分離）。
//!
//! # アクション名を `"clipboard:"` 名前空間で修飾する理由（イシュー #773
//! PR #816 Bugbot 指摘）
//!
//! `fandhe-frontend-wasm-full` の `Runtime<C>` は、マウントされたページの
//! ルート状態機械 `C` の型に関わらず Avatar/Clipboard 双方のイベント配線を
//! 無条件に行う（`crate::lib::Runtime::mount`/`Runtime::hydrate` 参照）。
//! そのため `C` が `Clipboard` 自身ではなく、たとえば独自の `AppState`
//! （カウンタの `"reset"` アクションを持つ想定）や [`crate::avatar::Avatar`]
//! （`"reset"` で `Loading` へ戻る）であっても、同一ページに Clipboard の
//! trigger が存在すればタイムアウト経過後の自動リセットが裸の `"reset"`
//! を `C::decode_action` へ dispatch してしまい、無関係な `C` の `"reset"`
//! アクションと衝突する（コピー操作が後からカウンタをゼロにしたり Avatar を
//! 強制的に loading 状態へ戻す）。`"clipboard:"` 接頭辞は他コンポーネントの
//! 裸のアクション名（`"copy"`/`"reset"`）と構造的に衝突しない一意な名前空間
//! を確保する。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`label`]/[`control`]/[`input`]/
//! [`trigger`]/[`indicator`]/[`value_text`]、純粋関数で完結）を直接呼んで
//! 組み立てる。CSR/hydration は [`Clipboard`] を経由し、dispatch
//! （`"clipboard:copy"`/`"clipboard:reset"`）で状態遷移する。
//! `fandhe-frontend-pre-styled-ui`が
//! 本モジュールを呼んでスタイル済み Clipboard を組み立てる想定である。
//!
//! # ARIA について（イシュー #1631 で是正）
//!
//! Clipboard は WAI-ARIA の専用パターンを持たない表示系コンポーネントだが、
//! 参照実装（ark-ui / Zag.js `clipboard.connect.ts`）は関連付け・
//! アクセシブルネームを補う最小限の属性を付与しており、fandhe も同様に
//! 追随する: [`label`] は `for`（`input_id` 引数、`input` の `id` を指す）
//! で明示的に紐付け、[`trigger`] は既定 `aria-label`
//! （[`TRIGGER_ARIA_LABEL_IDLE`]/[`TRIGGER_ARIA_LABEL_COPIED`]、`copied` に
//! 応じて反転する。クライアント側の反転配線は
//! `fandhe-frontend-wasm-full::headless_clipboard` の責務）を持つ。
//! いずれも呼び出し側 `attrs` が同名属性を指定していれば既定値を出力しない
//! （[`crate::number_input`] の `has_caller_attr` と同型の dedup、
//! fail-closed）。[`input`] はコピー元テキストの表示専用（`readonly` +
//! `data-readonly`）であり、フォーム送信を目的としない。ark-ui にある
//! `input` フォーカス時の全選択・ネイティブコピー（Ctrl+C/Cmd+C）検知は
//! 本イシューのスコープ外（モジュール末尾「スコープ外」節参照）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`type`/`readonly`）はすべて `&'static str` リテラル
//!   または固定スロットであり、動的値が属性名スロットへ混入する経路は
//!   ない（[`crate::anatomy`]/[`crate::data_attrs`] の既存不変条件をそのまま
//!   継承する）。[`trigger`] の既定 `aria-label` 値
//!   （[`TRIGGER_ARIA_LABEL_IDLE`]/[`TRIGGER_ARIA_LABEL_COPIED`]）も
//!   `&'static str` リテラル固定であり、動的値は混入しない。
//! - 動的値（`value`/呼び出し側 `attrs`/`children` テキスト）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - hydration 属性（`data-hydrate-copied`）はクライアント側で改ざんされ
//!   うる入力として扱う。[`Clipboard`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は panic せず
//!   `HydrateError` を返す。
//! - コピー対象値そのもの（`value`）を本モジュールがログ・エラーメッセージへ
//!   出力する経路は存在しない（本モジュールは純粋な描画/状態機械であり
//!   I/O を一切行わない）。クライアント側の実 `navigator.clipboard`
//!   配線（`fandhe-frontend-wasm-full` の `headless_clipboard`、
//!   イシュー #773 / PR #816 で実装済み）でも同じ不変条件を維持する契約
//!   （モジュール冒頭「コピー完了後の自動リセット」節参照）。
//!
//! # スコープ外
//!
//! - `navigator.clipboard.writeText` の実配線・タイムアウトによる自動
//!   リセットは `fandhe-frontend-wasm-full` の `headless_clipboard`
//!   （イシュー #773 / PR #816）で実装済み。`onStatusChange` コールバック
//!   相当（呼び出し側フックの提供）は未実装のまま据え置く。
//! - `asChild`・`ids` オプション（ark-ui 固有機能）は非採用。
//! - [`input`] へのフォーカス時の全選択（`select()`）、および
//!   ネイティブコピー（Ctrl+C/Cmd+C）検知による `"clipboard:copy"` 発火
//!   （ark-ui/Zag.js の `onFocus`/`onCopy` 相当）はイシュー #1631 でも
//!   未対応のまま据え置く（wasm-full 配線が必要でスコープが大きいため）。
//!   ブラウザ既定のネイティブコピー自体（選択してのコピー）は input が
//!   通常の `<input readonly>` である以上引き続き可能だが、copied
//!   フィードバック（`data-copied`/`aria-label` 反転）は [`trigger`]
//!   経由の操作のみで発生する。
//! - `translations.triggerLabel` 相当の i18n 差し替え API は非採用
//!   （呼び出し側 `attrs` に独自 `aria-label` を渡すことで代替可能）。

use crate::anatomy::{anatomy, Anatomy};
use crate::data_attrs::{data_copied, data_readonly, data_state};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// Clipboard の anatomy（`data-scope="clipboard"`）。
const ANATOMY: Anatomy = anatomy("clipboard");

/// [`trigger`] の既定 `aria-label`（未コピー時）。参照実装
/// （ark-ui/Zag.js `clipboard.connect.ts` の既定 `translations.triggerLabel`）
/// に合わせた固定英語リテラル（`.claude/rules/japanese-style.md`
/// のユーザー向け文字列は英語規約）。呼び出し側 `attrs` に独自の
/// `aria-label` があれば出力しない（本モジュール内の非公開ヘルパ
/// `has_caller_attr` 参照）。
pub const TRIGGER_ARIA_LABEL_IDLE: &str = "Copy to clipboard";

/// [`trigger`] の既定 `aria-label`（コピー済み時）。
/// `fandhe-frontend-wasm-full::headless_clipboard` はコピー成功/リセット時に
/// 現在値がこの 2 リテラルのいずれかと一致する場合のみ反転させる
/// （利用者の独自 `aria-label` を壊さない fail-closed 契約、
/// モジュール冒頭「ARIA について」節参照）。
pub const TRIGGER_ARIA_LABEL_COPIED: &str = "Copied to clipboard";

/// 呼び出し側 `attrs` に指定の属性キーが既に含まれるかを判定する
/// （[`crate::number_input`] の同名ヘルパと同型の dedup 判断、
/// fail-closed。重複属性による無効な HTML 出力・後勝ちの非決定的な描画を
/// 防ぐ）。
fn has_caller_attr(attrs: &[(&str, &str)], key: &str) -> bool {
    attrs.iter().any(|(k, _)| k.eq_ignore_ascii_case(key))
}

/// [`indicator`] の `data-state` 属性値 "visible"。
const DATA_STATE_VISIBLE: &str = "visible";
/// [`indicator`] の `data-state` 属性値 "hidden"。
const DATA_STATE_HIDDEN: &str = "hidden";

/// `visible`/`hidden` から `data-state` 属性値文字列へ変換する内部ヘルパ
/// （[`crate::avatar`] の同名ヘルパと同型）。
const fn visibility_str(visible: bool) -> &'static str {
    if visible {
        DATA_STATE_VISIBLE
    } else {
        DATA_STATE_HIDDEN
    }
}

/// Root パーツ（`div`）。
///
/// `value`（コピー対象値）を `data-value` としてそのまま出力する
/// （[`crate::select::item`] の `data-value` と同型のパターン、
/// `render()` の既定エスケープを必ず経由する）。クライアント側
/// （`fandhe-frontend-wasm-full`）はこの `data-value` を読み取って
/// `navigator.clipboard.writeText` へ渡す契約（モジュール冒頭「呼び出し
/// 文脈」節参照）。`copied` が `true` のときのみ [`data_copied`] により
/// `data-copied` を付与する。
#[must_use]
pub fn root<'a>(
    value: &'a str,
    copied: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("data-value", value)];
    merged.extend(data_copied(copied));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`label`）。
///
/// `input_id`（[`input`] の `id`）を渡すと `for` で明示的に紐付ける
/// （参照実装 ark-ui/Zag.js の `htmlFor` 相当、イシュー #1631 是正）。
/// `copied` に応じ [`data_copied`] により `data-copied` を付与する。
#[must_use]
pub fn label<'a>(
    copied: bool,
    input_id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = input_id {
        if !has_caller_attr(&attrs, "for") {
            merged.push(("for", id));
        }
    }
    merged.extend(data_copied(copied));
    merged.extend(attrs);
    ANATOMY.part("label", "label", merged, children)
}

/// Control パーツ（`div`）。[`input`]/[`trigger`] を内包するラッパー。
#[must_use]
pub fn control<'a>(copied: bool, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_copied(copied));
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// Input パーツ（`input type="text" readonly`）。
///
/// コピー元テキストの表示専用であり、フォーム送信を目的としない
/// （`name` 属性を持たない）。`value` は既定エスケープを経由した属性値
/// として出力する。
#[must_use]
pub fn input<'a>(value: &'a str, copied: bool, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![("type", "text"), ("readonly", ""), ("value", value)];
    merged.extend(data_readonly(true));
    merged.extend(data_copied(copied));
    merged.extend(attrs);
    ANATOMY.part("input", "input", merged, Vec::new())
}

/// Trigger パーツ（`button type="button"`）。クリックでコピーを実行する
/// 唯一の操作パーツ（クライアント配線層 `fandhe-frontend-wasm-full` の
/// `headless_clipboard`〔イシュー #773 / PR #816〕はこのパーツへの
/// クリックのみを監視する契約）。
#[must_use]
pub fn trigger<'a>(copied: bool, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    if !has_caller_attr(&attrs, "aria-label") {
        let label = if copied {
            TRIGGER_ARIA_LABEL_COPIED
        } else {
            TRIGGER_ARIA_LABEL_IDLE
        };
        merged.push(("aria-label", label));
    }
    merged.extend(data_copied(copied));
    merged.extend(attrs);
    ANATOMY.part("trigger", "button", merged, children)
}

/// Indicator パーツ（`span`）。
///
/// copied 用/idle 用の 2 変種を SSR で両方描画し、現在状態
/// （`copied`）と不一致の側へ `hidden` を付与する（[`crate::avatar::image`]/
/// [`crate::avatar::fallback`] の可視性切り替えと同型のパターン。
/// `fandhe-frontend-pre-styled-ui` の [`crate::anatomy::Anatomy`] ベースの
/// recipe は同一要素上の属性のみをセレクタにできるため、子孫セレクタを
/// 使わずに済むこの表現を採る）。`is_copied_variant` に `true` を渡すと
/// 「コピー済み」表示用の変種（例: チェックマーク）を、`false` を渡すと
/// 「未コピー」表示用の変種（例: コピーアイコン）を組み立てる。
#[must_use]
pub fn indicator<'a>(
    is_copied_variant: bool,
    copied: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let visible = is_copied_variant == copied;
    // `data-variant`（`"copied"`/`"idle"`、固定リテラル）: 2 変種の indicator
    // が同じ `data-scope`/`data-part` を共有するため、`data-state`
    // （現在の可視性）だけではクライアント側（`fandhe-frontend-wasm-full`）が
    // 「どちらの変種か」を区別できない。コピー成功/リセット時に可視性を
    // 再計算するために必要な変種識別子として付与する（ユーザー入力ではなく
    // 呼び出し側が渡す固定引数由来の値のため、動的値のエスケープ対象では
    // ない）。
    let variant = if is_copied_variant { "copied" } else { "idle" };
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        data_state(visibility_str(visible)),
        ("data-variant", variant),
    ];
    if !visible {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("indicator", "span", merged, children)
}

/// ValueText パーツ（`span`）。コピー対象値をテキストとして表示する
/// 装飾用パーツ（`children` は呼び出し側が組み立てる）。
#[must_use]
pub fn value_text<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("value-text", "span", attrs, children)
}

/// Clipboard のアクション（WASM 境界の文字列 dispatch と
/// [`Clipboard::decode_action`] で接続する）。payload は使用しない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardAction {
    /// コピー成功へ遷移する（`navigator.clipboard.writeText` 成功後にのみ
    /// クライアント側から発火する想定、モジュール冒頭「セキュリティ不変
    /// 条件」節参照）。
    Copy,
    /// 未コピー状態へ戻す（タイムアウト経過後にクライアント側から発火する
    /// 想定）。
    Reset,
}

/// Clipboard のコピー済み状態機械。
///
/// `data-copied` と実際のコピー済み状態の整合を型レベルで保証する入口
/// として、各パーツ関数（[`root`]/[`control`]/[`input`]/[`trigger`]/
/// [`indicator`]）へ `self.copied` を注入する利便メソッドを提供する。SSR
/// での自由関数直接利用（本型を経由しない構成）も引き続き可能。`Default`
/// は未コピー（SSR の状態なし初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Clipboard {
    copied: bool,
}

impl Clipboard {
    /// `data-hydrate-copied` 属性名のフィールド部分
    /// （`docs/api/hydration-state-format.md` の `<field>` 命名規約に従う）。
    pub const FIELD_COPIED: &'static str = "copied";

    /// `data-hydrate-copied` 属性値 "copied"。
    const HYDRATE_VALUE_COPIED: &'static str = "copied";
    /// `data-hydrate-copied` 属性値 "not-copied"。
    const HYDRATE_VALUE_NOT_COPIED: &'static str = "not-copied";

    /// 指定した初期状態で Clipboard を生成する。
    #[must_use]
    pub fn new(copied: bool) -> Self {
        Self { copied }
    }

    /// 現在コピー済みかどうか。
    #[must_use]
    pub fn is_copied(&self) -> bool {
        self.copied
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        value: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(value, self.copied, attrs, children)
    }

    /// [`label`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn label<'a>(
        &self,
        input_id: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        label(self.copied, input_id, attrs, children)
    }

    /// [`control`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn control<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        control(self.copied, attrs, children)
    }

    /// [`input`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn input<'a>(&self, value: &'a str, attrs: Vec<(&'a str, &'a str)>) -> Node {
        input(value, self.copied, attrs)
    }

    /// [`trigger`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn trigger<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        trigger(self.copied, attrs, children)
    }

    /// [`indicator`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn indicator<'a>(
        &self,
        is_copied_variant: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        indicator(is_copied_variant, self.copied, attrs, children)
    }
}

impl Component for Clipboard {
    type Action = ClipboardAction;

    fn update(&mut self, action: ClipboardAction) {
        self.copied = match action {
            ClipboardAction::Copy => true,
            ClipboardAction::Reset => false,
        };
    }

    /// 共通契約（`data-copied` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > trigger、`value` は空文字列・children 空・呼び出し側
    /// attrs なし）。[`crate::switch::Switch::view`]/
    /// [`crate::avatar::Avatar::view`] と同じ位置付けであり、公開 UI
    /// としての利用は想定しない（実際の UI 構築は §パーツ関数群を呼び出し側
    /// が組み合わせる）。
    fn view(&self) -> Node {
        self.root("", Vec::new(), vec![self.trigger(Vec::new(), Vec::new())])
    }

    fn decode_action(name: &str, _payload: &str) -> Option<ClipboardAction> {
        // アクション名は "clipboard:" 名前空間で修飾する（モジュール冒頭
        // 「アクション名を "clipboard:" 名前空間で修飾する理由」節参照。
        // 裸の "copy"/"reset" は Avatar/独自 AppState の既存アクション名と
        // 衝突しうるため使わない）。
        match name {
            "clipboard:copy" => Some(ClipboardAction::Copy),
            "clipboard:reset" => Some(ClipboardAction::Reset),
            _ => None,
        }
    }
}

impl Hydrate for Clipboard {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let value = if self.copied {
            Self::HYDRATE_VALUE_COPIED
        } else {
            Self::HYDRATE_VALUE_NOT_COPIED
        };
        vec![(
            format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_COPIED),
            value.to_string(),
        )]
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let attr_name = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_COPIED);
        let raw = attrs
            .iter()
            .find(|(k, _)| *k == attr_name)
            .map(|(_, v)| v.as_str())
            .ok_or_else(|| HydrateError::MissingAttr(attr_name.clone()))?;
        let copied = match raw {
            Self::HYDRATE_VALUE_COPIED => true,
            Self::HYDRATE_VALUE_NOT_COPIED => false,
            _ => {
                return Err(HydrateError::InvalidValue {
                    attr: attr_name.clone(),
                    reason: "expected \"copied\" or \"not-copied\"".to_string(),
                })
            }
        };
        Ok(Self { copied })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part/data-copied/data-value 出力 ---

    #[test]
    fn root_outputs_scope_part_and_data_value() {
        let html = render(&root("https://example.com", false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="clipboard""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-value="https://example.com""#));
        assert!(html.contains("<div"));
        assert!(!html.contains("data-copied"));
    }

    #[test]
    fn root_copied_true_adds_data_copied() {
        let html = render(&root("v", true, vec![], vec![]));
        assert!(html.contains(r#"data-copied="""#));
    }

    #[test]
    fn label_outputs_scope_and_part() {
        let html = render(&label(false, None, vec![], vec![text("Label")]));
        assert!(html.contains(r#"data-scope="clipboard""#));
        assert!(html.contains(r#"data-part="label""#));
        assert!(html.contains("<label"));
        assert!(html.contains("Label"));
        assert!(!html.contains(" for="));
        assert!(!html.contains("data-copied"));
    }

    #[test]
    fn label_with_input_id_adds_for() {
        let html = render(&label(false, Some("clipboard-input"), vec![], vec![]));
        assert!(html.contains(r#"for="clipboard-input""#));
    }

    #[test]
    fn label_copied_true_adds_data_copied() {
        let html = render(&label(true, None, vec![], vec![]));
        assert!(html.contains(r#"data-copied="""#));
    }

    #[test]
    fn label_caller_for_overrides_input_id_without_duplication() {
        // 呼び出し側が独自の "for" を attrs に渡した場合、input_id 由来の
        // 既定値と重複させない（イシュー #1631 Review 指摘、trigger の
        // aria-label dedup と同型）。
        let html = render(&label(
            false,
            Some("clipboard-input"),
            vec![("for", "custom-input")],
            vec![],
        ));
        assert_eq!(html.matches(" for=").count(), 1);
        assert!(html.contains(r#"for="custom-input""#));
        assert!(!html.contains(r#"for="clipboard-input""#));
    }

    #[test]
    fn control_outputs_scope_part_and_data_copied_when_true() {
        let html = render(&control(true, vec![], vec![]));
        assert!(html.contains(r#"data-scope="clipboard""#));
        assert!(html.contains(r#"data-part="control""#));
        assert!(html.contains(r#"data-copied="""#));
    }

    #[test]
    fn control_not_copied_omits_data_copied() {
        let html = render(&control(false, vec![], vec![]));
        assert!(!html.contains("data-copied"));
    }

    #[test]
    fn input_has_type_text_readonly_and_value() {
        let html = render(&input("secret-value", false, vec![]));
        assert!(html.contains(r#"data-scope="clipboard""#));
        assert!(html.contains(r#"data-part="input""#));
        assert!(html.contains(r#"type="text""#));
        assert!(html.contains(r#"readonly="""#));
        assert!(html.contains(r#"data-readonly="""#));
        assert!(html.contains(r#"value="secret-value""#));
        assert!(!html.contains("data-copied"));
    }

    #[test]
    fn input_copied_true_adds_data_copied() {
        let html = render(&input("v", true, vec![]));
        assert!(html.contains(r#"data-copied="""#));
    }

    #[test]
    fn trigger_has_type_button_and_data_copied_when_true() {
        let html = render(&trigger(true, vec![], vec![text("Copy")]));
        assert!(html.contains(r#"data-scope="clipboard""#));
        assert!(html.contains(r#"data-part="trigger""#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"data-copied="""#));
        assert!(html.contains("Copy"));
    }

    #[test]
    fn trigger_not_copied_omits_data_copied() {
        let html = render(&trigger(false, vec![], vec![]));
        assert!(!html.contains("data-copied"));
    }

    #[test]
    fn trigger_default_aria_label_reflects_copied_state() {
        let idle_html = render(&trigger(false, vec![], vec![]));
        assert!(idle_html.contains(r#"aria-label="Copy to clipboard""#));

        let copied_html = render(&trigger(true, vec![], vec![]));
        assert!(copied_html.contains(r#"aria-label="Copied to clipboard""#));
    }

    #[test]
    fn trigger_caller_aria_label_overrides_default_without_duplication() {
        let html = render(&trigger(false, vec![("aria-label", "Copy URL")], vec![]));
        assert!(html.contains(r#"aria-label="Copy URL""#));
        assert!(!html.contains("Copy to clipboard"));
        // 属性は 1 個だけであること（既定値との重複出力がないこと）を
        // 出現回数で確認する。
        assert_eq!(html.matches("aria-label=").count(), 1);
    }

    #[test]
    fn value_text_outputs_scope_and_part() {
        let html = render(&value_text(vec![], vec![text("https://example.com")]));
        assert!(html.contains(r#"data-scope="clipboard""#));
        assert!(html.contains(r#"data-part="value-text""#));
        assert!(html.contains("https://example.com"));
    }

    // --- Indicator の可視性切り替え ---

    #[test]
    fn indicator_copied_variant_visible_when_copied() {
        let html = render(&indicator(true, true, vec![], vec![]));
        assert!(html.contains(r#"data-state="visible""#));
        assert!(html.contains(r#"data-variant="copied""#));
        assert!(!html.contains("hidden"));
    }

    #[test]
    fn indicator_copied_variant_hidden_when_not_copied() {
        let html = render(&indicator(true, false, vec![], vec![]));
        assert!(html.contains(r#"data-state="hidden""#));
        assert!(html.contains(r#"data-variant="copied""#));
        assert!(html.contains(r#"hidden="""#));
    }

    #[test]
    fn indicator_idle_variant_visible_when_not_copied() {
        let html = render(&indicator(false, false, vec![], vec![]));
        assert!(html.contains(r#"data-state="visible""#));
        assert!(html.contains(r#"data-variant="idle""#));
        assert!(!html.contains("hidden"));
    }

    #[test]
    fn indicator_idle_variant_hidden_when_copied() {
        let html = render(&indicator(false, true, vec![], vec![]));
        assert!(html.contains(r#"data-state="hidden""#));
        assert!(html.contains(r#"data-variant="idle""#));
        assert!(html.contains(r#"hidden="""#));
    }

    // --- XSS 回帰: data-value/input value のエスケープ ---

    #[test]
    fn root_data_value_attribute_breakout_payload_is_escaped() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&root(PAYLOAD, false, vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn input_value_attribute_breakout_payload_is_escaped() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&input(PAYLOAD, false, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn value_text_children_are_escaped_on_render() {
        let html = render(&value_text(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    // --- 状態機械: dispatch・decode_action ---

    #[test]
    fn default_clipboard_is_not_copied() {
        let c = Clipboard::default();
        assert!(!c.is_copied());
    }

    #[test]
    fn decode_action_accepts_copy_and_reset_and_rejects_unknown() {
        assert!(<Clipboard as Component>::decode_action("clipboard:copy", "").is_some());
        assert!(<Clipboard as Component>::decode_action("clipboard:reset", "").is_some());
        assert!(<Clipboard as Component>::decode_action("no_such_action", "").is_none());
    }

    /// 裸の `"copy"`/`"reset"` は Avatar/独自 AppState の既存アクション名と
    /// 衝突しうるため受理しない回帰テスト（イシュー #773 PR #816 Bugbot
    /// 指摘、モジュール冒頭「アクション名を "clipboard:" 名前空間で修飾する
    /// 理由」節参照）。
    #[test]
    fn decode_action_rejects_unnamespaced_copy_and_reset() {
        assert!(<Clipboard as Component>::decode_action("copy", "").is_none());
        assert!(<Clipboard as Component>::decode_action("reset", "").is_none());
    }

    #[test]
    fn copy_action_sets_copied_true_and_reset_sets_false() {
        let mut c = Clipboard::default();
        assert!(dispatch(&mut c, "clipboard:copy", ""));
        assert!(c.is_copied());
        assert!(dispatch(&mut c, "clipboard:reset", ""));
        assert!(!c.is_copied());
    }

    #[test]
    fn unknown_dispatch_action_is_no_op() {
        let mut c = Clipboard::default();
        assert!(!dispatch(&mut c, "no_such_action", ""));
        assert!(!c.is_copied());
    }

    // --- SSR view: hydrate 属性を出力しない ---

    #[test]
    fn ssr_default_view_has_no_hydrate_attrs() {
        let html = render(&Clipboard::default().view());
        assert!(!html.contains("data-hydrate"));
    }

    // --- Hydrate: 正常・異常系 ---

    #[test]
    fn ssr_and_hydration_round_trip() {
        let mut c = Clipboard::default();
        assert!(dispatch(&mut c, "clipboard:copy", ""));

        let hydrate_html = render(&render_for_hydration(&c));
        assert!(hydrate_html.contains(r#"data-hydrate-copied="copied""#));

        let restored = Clipboard::from_hydration_attrs(&c.hydration_attrs()).unwrap();
        assert_eq!(restored, c);
    }

    #[test]
    fn hydration_missing_attr_is_error() {
        let result = Clipboard::from_hydration_attrs(&[]);
        assert!(matches!(result, Err(HydrateError::MissingAttr(_))));
    }

    #[test]
    fn hydration_invalid_value_is_error_not_panic() {
        let attrs = vec![("data-hydrate-copied".to_string(), "maybe".to_string())];
        let result = Clipboard::from_hydration_attrs(&attrs);
        assert!(matches!(result, Err(HydrateError::InvalidValue { .. })));
    }

    // --- fail-closed: 呼び出し側の data-scope/data-part 偽装は無視される ---

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            "v",
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="clipboard""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }
}
