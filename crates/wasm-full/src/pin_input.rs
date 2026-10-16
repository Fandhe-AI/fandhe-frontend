//! PinInput（`fandhe-frontend-headless-ui` `pin_input` モジュール）の
//! `input` / `focusin` / `keydown` / `paste` / `compositionend` → dispatch 配線
//! （イシュー #3834・#3835、親 #3831）。
//!
//! # 背景
//!
//! `crates/headless-ui/src/pin_input.rs` は桁ごとの `input` パーツ
//! （`data-index`・`maxlength="1"`）と `"input"`/`"backspace"`/`"delete"`/`"prev"`/
//! `"next"`/`"focus"`/`"paste"` 等の dispatch 契約（`PinInput::decode_action`）を提供する
//! 一方、実 DOM のイベントを dispatch へつなぐ配線を「wasm 層の後続責務」として残していた。
//! 本モジュールは #3834 で 1 文字入力と次桁へのフォーカス移動を、#3835 で貼り付け・
//! Backspace/Delete/矢印キー・autofill・モバイルの削除・IME 確定入力を実装する。
//! 実ブラウザでの網羅的な検証は #3836 の担当。
//!
//! # 不変条件（本モジュールの設計の核）
//!
//! **入力の受理・拒否と `focused` の位置は状態機械（`PinInput`）だけが決める。**
//! DOM 層は状態機械の判断を推測せず、判断した後の状態（各桁の値と `focused`）を
//! 読み戻して DOM へ反映するだけにする。したがって本モジュールは次を行わない。
//!
//! - kind（`data-hydrate-kind`）の解決・文字種の事前判定・大文字そろえ
//!   （状態機械の `normalize_char` / `is_valid_char` が唯一の判定者）。
//! - ライブ `value`・`value` content attribute・文字比較からの受理の推定。
//! - 「受理されたら次桁（`index + 1`）」といった `focused` の推測。
//!
//! 流れは `input`（capture: 入力された桁の場所と値の分類の記録のみ）→ 状態機械へ dispatch
//! （bubble）→ 状態の読み戻し → DOM 反映（各桁の値とフォーカス）の 1 本道で、判断は
//! 状態機械の中にしかない。受理・拒否・文字種判定・大文字そろえ・同じ文字の打ち直し・
//! `focused` の前進は、すべて状態機械の結果として値とフォーカスに現れる。
//!
//! # 読み戻しの契約（`BindingSource` の予約 field）
//!
//! 正準ビューは `focused` を持たず、`Runtime<C>` の `C` はアプリ固有のホスト型のため、
//! 状態の読み戻しは `Runtime` が既に `C` へ要求している
//! [`fandhe_frontend_wasm_client::BindingSource`] の予約 field で行う
//! （新しい trait 境界を `Runtime` に足さない）。ホストは `bound_value` から
//! [`parse_field`] で field 名を解釈し [`bound_value_for`] へ委譲する。
//!
//! | field（[`field_name`]） | 値 |
//! |---|---|
//! | `pin-input.<n>.focused` | `Text(focused の桁番号)`。未設定は `Text("")` |
//! | `pin-input.<n>.digit.<i>` | `Text(桁 i の値)`。未入力は `Text("")` |
//!
//! `<n>` は配線 root 配下の PinInput Root の文書順（0 起点。配線 root 自身が Root ならそれが 0）。
//! 1 つでも読めない（`None`・形式不正・桁数超過）場合は **fail-closed**: 値もフォーカスも DOM を
//! 書き換えず、状態機械への dispatch だけを行う（見た目はアプリの再描画に委ねる。
//! `focusin` のみ、ユーザーが選んだ桁へフォーカスを戻す）。
//!
//! **予約 field の実装は事実上必須**。未実装だと、拒否された文字が DOM の桁に残り、表示と
//! 状態がずれる（fail-closed は「DOM を書かない」であって「見た目を保証する」ではない）。
//! したがって次を守ること。
//!
//! - 桁 input の `value` を JS から直接読まない（拒否された文字を含みうる）。
//! - フォーム送信値は `hidden_input`（状態機械の値）を使う。
//! - `pin-input.` は予約接頭辞のため、アプリ独自の `BindingSource` field 名に使わない。
//!   `pin-input.<n>.digit.<i>` を表示用の束縛（`data-bind-*`）に使わない（OTP が DOM へ露出する）。
//!
//! 再描画で要素が差し替わっても追えるよう、要素参照は保持せず「何番目の Root か」と
//! `data-index` から引き直す。
//!
//! # dispatch とアクションの対応
//!
//! `index` は対象桁の `data-index`。どの経路も先頭に `"focus"(index)` を送る（アプリ状態の
//! focused が古くても、DOM で操作された桁へ作用させ、拒否時のフォーカス状態を実際の DOM と
//! そろえるため）。
//!
//! | DOM イベント | 条件 | dispatch 列 | DOM の値の書き込み範囲 |
//! |---|---|---|---|
//! | `input`（桁 input） | 値がちょうど 1 文字 | `focus` → `input`（その 1 文字、無加工） | その桁のみ |
//! | `input` | 2 文字以上（autofill・`insertFromPaste`、上限以内） | `focus` → `paste`（ライブ値、無加工） | 全桁 |
//! | `input` | 0 文字・`inputType` が後方削除系 | `focus` → `backspace` | その桁のみ |
//! | `input` | 0 文字・上記以外または不明 | `focus` → `delete` | その桁のみ |
//! | `keydown` Backspace / Delete | 修飾キーなし・変換中でない | `focus` → `backspace` / `delete` | その桁のみ |
//! | `keydown` ArrowLeft / ArrowRight | 同上 | `focus` → `prev` / `next` | 書かない（フォーカスのみ） |
//! | `paste` | `clipboardData` あり・`text/plain` が上限以内 | `focus` → `paste`（無加工） | 全桁 |
//! | `compositionend` | 桁の値が 1 文字 / 2 文字以上 | `focus` → `input` / `paste` | その桁のみ / 全桁 |
//! | `focusin`（桁 input） | `data-index` が桁数未満 | `focus` | 書かない |
//!
//! 後方削除系の `inputType` は `deleteContentBackward`・`deleteWordBackward`・
//! `deleteSoftLineBackward`・`deleteHardLineBackward`。それ以外（`deleteContentForward`・
//! `deleteByCut`・`historyUndo` 等）と不明（`InputEvent` でない合成 `Event`）は `delete` に
//! そろえる（消えた桁でキャレットが留まる DOM の実態に一致させる）。
//!
//! 状態機械の `Paste` は発生した桁に関わらず **先頭の桁から** 充填する。DOM 層は対象桁から
//! 詰めるような補正をしない（`"focus"` を前置するのは拒否時のフォーカスをそろえるためだけ）。
//! 区切り文字の除去・大文字そろえ・桁数や文字種の適否は状態機械の入力補助で、DOM 層は
//! クリップボードのテキストを無加工で渡す。
//!
//! 書き込み範囲を「その桁のみ」に絞るのは、ネイティブで削除済みの別の桁を状態から復活させない
//! ため。貼り付け・autofill は状態機械が全桁を置き換えるので「全桁」（状態と異なる桁だけ書く）。
//!
//! # `preventDefault` の方針
//!
//! - keydown の 4 キーは処理したときに `preventDefault` する。ネイティブの削除・キャレット移動が
//!   起きず、後続の 0 文字 `input` で二重に dispatch されない。修飾キー付き・変換中
//!   （`isComposing` / keyCode 229）は dispatch も `preventDefault` もしない（ネイティブ動作に任せる。
//!   例: Ctrl+Backspace は `deleteWordBackward` の 0 文字 `input` として `backspace` に反映される）。
//! - `paste` は `clipboardData` を読めたときだけ `preventDefault` して dispatch する（生のクリップ
//!   ボード文字列は DOM の桁に入らず、反映は状態機械の値だけになる）。`clipboardData` が無い・読めない
//!   ときは何もせず、ネイティブ挿入で起きる `input` の経路に任せる。
//! - 2 文字以上・0 文字の `input` と `compositionend` はイベント後のため `preventDefault` しない。
//! - `stopPropagation` は呼ばない（document/window の部品の配線を壊さない）。
//!
//! # 汎用 keydown 配線（`events::wire_keydown`）との排他
//!
//! 本配線は `Runtime::mount`/`hydrate` で `wire_keydown` より前に登録され、`wire_keydown` は
//! 冒頭で `default_prevented()` なら何もしない。処理した 4 キーへの `preventDefault` がそのまま
//! 排他になる。`keydown_claim`（#3768 方式 2b）には **登録しない**。あの登録簿は root の bubble より後に
//! 動く document/window の部品（overlay・sidebar・command）向けで、root の bubble で先に動く本配線は
//! `preventDefault` で足りるため。
//!
//! # IME
//!
//! - keydown は `isComposing` または keyCode 229 なら無視する。
//! - 変換中（`isComposing`）の `input` は無視する。確定は `compositionend` で受け、桁の確定値を
//!   1 文字なら `input`、2 文字以上なら `paste` として dispatch する（0 文字は変換の取り消しで何もしない）。
//! - Safari 系は確定後に非 composing の `input`（`inputType` が `insertFromComposition` 等）を追加で
//!   発火することがある。`compositionend` を処理した桁を 1 回だけ記録し、同じ桁の直後の
//!   composition 系 `inputType` の `input` は dispatch しない（二重適用の防止）。記録は次の `input`・
//!   `keydown` で必ず破棄する。
//! - `compositionstart` は購読しない（`input` 側の `isComposing` 判定と上記ガードで足りる）。
//!
//! # `data-action-input` を持つ桁（アプリ所有の経路）
//!
//! 桁 input（または配線 root 内の祖先）に `data-action-input` がある場合、`input` の dispatch は
//! `crate::events::wire_events` とアプリの action が担う。この経路では本モジュールは
//! **何も dispatch しない**（`input` も汎用 `"focus"` も。汎用 `"focus"` は payload に
//! インスタンス情報を持たず、複数インスタンスのアプリで別状態を書き換えうるため）。
//! その代わり、アプリの dispatch が終わった後に（`Runtime::mount`/`hydrate` は
//! `wire_events` より後にこの配線を登録するため、同一 `input` イベントの bubble 段で
//! `wire_events` の後に実行される）、他の経路と同じ読み戻しで値とフォーカスだけを反映する。
//! アプリ側の契約は次のとおり。
//!
//! - action 名（`data-action-input` の値）で桁を識別すること（桁ごとに別名を付け、
//!   ホストの `update` で `Focus(i)` → `Input(c)` を順に適用する等）。汎用イベントは `data-index` を送らない。
//! - `focusin` では本モジュールは何も dispatch しないため、桁の選択（クリック・Tab）の同期もアプリの責務。
//! - 読み戻し field を実装しなければ、フォーカス移動と値の反映は行われない。
//! - `keydown`・`paste`・`compositionend` でも所有者のいる桁は dispatch も `preventDefault` もしない
//!   （`paste` を止めるとアプリの `input` 経路が発火しなくなるため）。`input`（2 文字以上・0 文字を含む）
//!   だけがアプリの dispatch 後の読み戻し反映を行う（値の書き込みはその桁のみ）。キー操作をアプリで
//!   扱いたいときは `data-action-keydown`（汎用 keydown 配線）を使う。
//!
//! # 配線の回数・入れ子・dispatch 回数
//!
//! - 配線は 1 root につき 1 回。入れ子の配線 root では capture の記録（`PENDING`）が 1 スロットのため、
//!   内側の配線だけが反映する。
//! - 1 イベントで `"focus"` と本体アクションの 2 回 dispatch するため、`on_update` も 2 回走る。
//!
//! # 複数インスタンス
//!
//! 汎用経路（`data-action-input` なし）の `"focus"`/`"input"` は payload にインスタンス ID を
//! 持たないため、**1 つの配線 root につき PinInput は 1 つ**（1 root : 1 状態機械）が前提。
//! 同一 root に複数の PinInput を置く場合は、各桁へ `data-action-input`（インスタンスと桁を
//! 識別する action 名）を付け、ホストの `bound_value` を [`parse_field`] の `<n>`（Root の文書順）
//! に応じて各インスタンスの状態から答えること。
//!
//! # `MAPPING_TABLE` 境界行との役割分担
//!
//! `crate::headless::MAPPING_TABLE` の `(pin-input, input)` 行は `data-value` を持たない
//! 現行マークアップでは常に解決されない「境界行」であり、pin-input を他のクリック可能な
//! 部品の内側に置いたとき祖先探索が外側の部品へ抜けるのを防ぐためだけに存在する。
//! 桁番号の同期は本モジュールの `focusin` リスナーが担う。
//!
//! # 設計（`number_input.rs` と同型の 2 層構成）
//!
//! - 純粋ロジック層（本ファイル直下の `pub` 関数）は web-sys に依存せず、
//!   native の `cargo test` で検証できる。
//! - 配線層（`wiring`）のみ `#[cfg(target_arch = "wasm32")]` でゲートする。
//!
//! 本モジュールは `lib.rs` 側で `#[cfg(feature = "pin-input")]` によりモジュール
//! ごとゲートされる（feature off のビルドへ配線コードを含めない、イシュー #3834
//! の受け入れ条件。`number_input` がモジュールをゲートしないのとは意図的に異なる）。
//!
//! # `docs/policy/intentional-non-adoption.md` §3.25 との整理
//!
//! 本モジュールは文字種の判定も値の検証も正規化も行わない。区切り除去と大文字そろえは
//! 状態機械の入力補助、値の正しさ（kind への適合）は状態機械とアプリ側の責務のままである。
//!
//! # セキュリティ不変条件
//!
//! - `input.value` は分類に必要な範囲（先頭 2 文字）と、`MAX_PASTE_CHARS` 以下の貼り付け候補の
//!   取り出しにしか使わない。`MAX_PASTE_CHARS` は資源上限であって値の検証ではない（上限を超えた
//!   テキストは dispatch せず、貼り付けは `preventDefault` 済みで DOM は変わらない）。
//!   `data-index` は `usize` へパースし桁数未満を検査する（fail-closed）。DOM へ書く値は状態機械が
//!   返した桁の値（1 文字以下）に限り、`HtmlInputElement::set_value` のみを使う。HTML 文字列は
//!   組み立てない（REQ-1）。
//! - リスナーは配線 1 回につき 6 個（`Closure::forget` 6 回: `input` capture・`input` bubble・`focusin`・
//!   `keydown`・`paste`・`compositionend`）で固定する。
//! - PIN/OTP とクリップボードのテキストは秘密値のため、値・payload を console やエラーに出さない。
//!   `clipboardData` が無い・`getData` が失敗したときは何もしない（fail-closed）。

use crate::events::ActionRef;
use fandhe_frontend_headless_ui::pin_input::PinInput;
use fandhe_frontend_wasm_client::BoundValue;

/// PinInput の `data-scope` 属性値。
pub const PIN_INPUT_SCOPE: &str = "pin-input";
/// 桁 input パーツの `data-part` 属性値。
pub const INPUT_PART: &str = "input";
/// Root パーツの `data-part` 属性値。
pub const ROOT_PART: &str = "root";
/// 1 文字入力を状態機械へ渡すアクション名（`PinInput::decode_action`）。
pub const ACTION_INPUT: &str = "input";
/// 桁へのフォーカス同期アクション名（`PinInput::decode_action`）。
pub const ACTION_FOCUS: &str = "focus";
/// Backspace（現在桁を消去し前の桁へ）アクション名。
pub const ACTION_BACKSPACE: &str = "backspace";
/// Delete（現在桁を消去、フォーカスは動かさない）アクション名。
pub const ACTION_DELETE: &str = "delete";
/// 前の桁へフォーカスを移すアクション名。
pub const ACTION_PREV: &str = "prev";
/// 次の桁へフォーカスを移すアクション名。
pub const ACTION_NEXT: &str = "next";
/// 貼り付け・autofill のテキストを渡すアクション名。
pub const ACTION_PASTE: &str = "paste";
/// 貼り付け候補テキストの文字数上限（資源上限。値の検証ではない、モジュール doc 参照）。
pub const MAX_PASTE_CHARS: usize = 256;

/// `BindingSource` 予約 field の接頭辞（モジュール doc「読み戻しの契約」参照）。
const FIELD_PREFIX: &str = "pin-input.";

/// `input` イベント後の `input.value` の分類（先頭 2 文字だけを読んだ結果）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputValue {
    /// 空文字（削除系。`backspace`/`delete` へ振り分ける）。
    Empty,
    /// ちょうど 1 文字。
    Single(char),
    /// 2 文字以上（autofill・貼り付け。`paste` へ振り分ける）。
    Multi,
}

/// `data-index` の生文字列を桁数未満の `usize` へパースする（それ以外は `None`）。
#[must_use]
pub fn parse_digit_index(raw: Option<&str>, count: usize) -> Option<usize> {
    let index = raw?.parse::<usize>().ok()?;
    (index < count).then_some(index)
}

/// `input.value` を先頭 2 文字だけ読んで分類する（巨大な値でも確保量は増えない）。
///
/// 分類は「どのアクションへ送るか（1 文字入力・貼り付け・削除）」の振り分けだけで、
/// 受理・拒否の判定ではない（文字種の判定は状態機械が行う）。
#[must_use]
pub fn classify_input_value(value: &str) -> InputValue {
    let mut chars = value.chars();
    match (chars.next(), chars.next()) {
        (None, _) => InputValue::Empty,
        (Some(c), None) => InputValue::Single(c),
        (Some(_), Some(_)) => InputValue::Multi,
    }
}

/// 1 文字入力で dispatch する列（`focus` → `input` の順）。`c` は無加工で渡す
/// （正規化・文字種判定は状態機械の責務）。
#[must_use]
pub fn input_dispatches(index: usize, c: char) -> [ActionRef; 2] {
    [
        ActionRef {
            action: ACTION_FOCUS.to_string(),
            payload: index.to_string(),
        },
        ActionRef {
            action: ACTION_INPUT.to_string(),
            payload: c.to_string(),
        },
    ]
}

fn action(name: &str, payload: String) -> ActionRef {
    ActionRef {
        action: name.to_string(),
        payload,
    }
}

/// キー操作の意図（keydown のうち本配線が処理する 4 キー）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyIntent {
    /// Backspace。
    Backspace,
    /// Delete。
    Delete,
    /// ArrowLeft。
    Prev,
    /// ArrowRight。
    Next,
}

/// keydown を [`KeyIntent`] へ分類する（処理しないときは `None`）。
///
/// 修飾キー付き・変換中（`is_composing` または `key_code == 229`）はネイティブ動作に任せるため
/// `None`。`key` は大文字小文字を区別して完全一致で比較する。
#[must_use]
pub fn key_intent(
    key: &str,
    ctrl: bool,
    alt: bool,
    shift: bool,
    meta: bool,
    is_composing: bool,
    key_code: u32,
) -> Option<KeyIntent> {
    if ctrl || alt || shift || meta || is_composing || key_code == 229 {
        return None;
    }
    match key {
        "Backspace" => Some(KeyIntent::Backspace),
        "Delete" => Some(KeyIntent::Delete),
        "ArrowLeft" => Some(KeyIntent::Prev),
        "ArrowRight" => Some(KeyIntent::Next),
        _ => None,
    }
}

/// keydown で dispatch する列（`focus` → 対応アクション）。
#[must_use]
pub fn key_dispatches(index: usize, intent: KeyIntent) -> [ActionRef; 2] {
    let name = match intent {
        KeyIntent::Backspace => ACTION_BACKSPACE,
        KeyIntent::Delete => ACTION_DELETE,
        KeyIntent::Prev => ACTION_PREV,
        KeyIntent::Next => ACTION_NEXT,
    };
    [
        action(ACTION_FOCUS, index.to_string()),
        action(name, String::new()),
    ]
}

/// 貼り付け・autofill で dispatch する列（`focus` → `paste`）。`text` は無加工で渡す
/// （区切り除去・大文字そろえ・適否判定は状態機械の責務）。
#[must_use]
pub fn paste_dispatches(index: usize, text: &str) -> [ActionRef; 2] {
    [
        action(ACTION_FOCUS, index.to_string()),
        action(ACTION_PASTE, text.to_string()),
    ]
}

/// 0 文字 `input`（削除系）で dispatch する列。後方削除系の `inputType` は `backspace`、
/// それ以外と不明（`None`）は `delete`（モジュール doc の対応表）。
#[must_use]
pub fn deletion_dispatches(index: usize, input_type: Option<&str>) -> [ActionRef; 2] {
    let name = match input_type {
        Some(
            "deleteContentBackward"
            | "deleteWordBackward"
            | "deleteSoftLineBackward"
            | "deleteHardLineBackward",
        ) => ACTION_BACKSPACE,
        _ => ACTION_DELETE,
    };
    [
        action(ACTION_FOCUS, index.to_string()),
        action(name, String::new()),
    ]
}

/// 貼り付け候補テキストの上限判定の結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteText<'a> {
    /// [`MAX_PASTE_CHARS`] 以内（無加工のまま渡す）。
    Text(&'a str),
    /// 上限超過（dispatch しない）。
    Oversized,
}

/// テキストが [`MAX_PASTE_CHARS`] 以内かだけを判定する（先頭 `MAX + 1` 文字で打ち切る。
/// 加工も検証もしない）。
#[must_use]
pub fn classify_paste_text(text: &str) -> PasteText<'_> {
    if text.chars().nth(MAX_PASTE_CHARS).is_some() {
        PasteText::Oversized
    } else {
        PasteText::Text(text)
    }
}

/// `BindingSource` の予約 field が指す状態の種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinInputField {
    /// 状態機械の `focused`（桁番号。未設定は空文字）。
    Focused,
    /// 桁 `i` の値（未入力は空文字）。
    Digit(usize),
}

/// 予約 field 名を組み立てる（`pin-input.<n>.focused` / `pin-input.<n>.digit.<i>`）。
#[must_use]
pub fn field_name(ordinal: usize, field: PinInputField) -> String {
    match field {
        PinInputField::Focused => format!("{FIELD_PREFIX}{ordinal}.focused"),
        PinInputField::Digit(i) => format!("{FIELD_PREFIX}{ordinal}.digit.{i}"),
    }
}

/// 予約 field 名を `(Root の文書順, 種別)` へ解釈する。
///
/// [`field_name`] の出力と完全一致する正準形だけを受理する（`+0` や先頭 0 など
/// 別表記は `None`。ホストの `bound_value` が未知 field へ `None` を返す契約と整合）。
#[must_use]
pub fn parse_field(name: &str) -> Option<(usize, PinInputField)> {
    let rest = name.strip_prefix(FIELD_PREFIX)?;
    let (ordinal_raw, kind_raw) = rest.split_once('.')?;
    let ordinal = ordinal_raw.parse::<usize>().ok()?;
    let field = if kind_raw == "focused" {
        PinInputField::Focused
    } else {
        let index = kind_raw.strip_prefix("digit.")?.parse::<usize>().ok()?;
        PinInputField::Digit(index)
    };
    (field_name(ordinal, field) == name).then_some((ordinal, field))
}

/// ホストの `BindingSource::bound_value` が予約 field を [`PinInput`] の状態から答えるための
/// 委譲先（`parse_field` で解釈した種別を渡す）。範囲外の桁は空文字（`PinInput::digit` と同じ）。
#[must_use]
pub fn bound_value_for(pin: &PinInput, field: PinInputField) -> BoundValue {
    match field {
        PinInputField::Focused => BoundValue::Text(
            pin.focused_index()
                .map(|i| i.to_string())
                .unwrap_or_default(),
        ),
        PinInputField::Digit(i) => BoundValue::Text(pin.digit(i).to_string()),
    }
}

/// 状態機械が dispatch 後に持つ状態の読み戻し結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinInputSnapshot {
    /// 状態機械の `focused`。
    pub focused: Option<usize>,
    /// 各桁の値（長さは桁数、空文字は未入力）。
    pub digits: Vec<String>,
}

/// 予約 field をすべて読んで [`PinInputSnapshot`] を作る。
///
/// 1 つでも読めない・形式が不正（`Text` 以外、1 文字超の桁、桁数以上の `focused`）なら
/// `None`（fail-closed。モジュール doc「読み戻しの契約」）。
pub fn snapshot_from_bound(
    ordinal: usize,
    count: usize,
    read: impl Fn(&str) -> Option<BoundValue>,
) -> Option<PinInputSnapshot> {
    let focused = match read(&field_name(ordinal, PinInputField::Focused))? {
        BoundValue::Text(t) if t.is_empty() => None,
        BoundValue::Text(t) => Some(t.parse::<usize>().ok().filter(|i| *i < count)?),
        BoundValue::Flag(_) => return None,
    };
    let mut digits = Vec::new();
    for i in 0..count {
        match read(&field_name(ordinal, PinInputField::Digit(i)))? {
            BoundValue::Text(t) if t.chars().count() <= 1 => digits.push(t),
            _ => return None,
        }
    }
    Some(PinInputSnapshot { focused, digits })
}

/// 読み戻した状態を DOM へ反映する計画。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReflectPlan {
    /// ライブ `value` を書き換える桁と値（状態機械の値とライブ値が異なる桁のみ）。
    pub value_writes: Vec<(usize, String)>,
    /// フォーカスを置く桁（`None` なら動かさない）。
    pub focus: Option<usize>,
}

/// DOM の値を状態へそろえる桁の範囲。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueScope {
    /// 値は書かない（フォーカスのみ。`focusin`・矢印キー）。
    None,
    /// 指定の 1 桁だけ書く（ネイティブ削除済みの別の桁を復活させない）。
    One(usize),
    /// 全桁（貼り付け・autofill は状態機械が全桁を置き換える）。
    All,
}

/// 状態の読み戻し結果から DOM 反映の計画を作る（純粋関数）。
///
/// - `snapshot` があれば、値は `scope` の範囲だけを状態機械の値へそろえる（状態と異なる桁のみ）。
///   フォーカスは状態機械の `focused` へ置く（受理・拒否・正規化・次桁への前進はすべて状態機械の結果）。
/// - `snapshot` が無ければ（読み戻せない）値は触らず、フォーカスは `fallback_focus`
///   （`focusin` でユーザーが選んだ桁の復元）にだけ使う。
#[must_use]
pub fn reflect_plan_scoped(
    snapshot: Option<&PinInputSnapshot>,
    live: &[String],
    scope: ValueScope,
    fallback_focus: Option<usize>,
) -> ReflectPlan {
    let Some(snapshot) = snapshot else {
        return ReflectPlan {
            value_writes: Vec::new(),
            focus: fallback_focus,
        };
    };
    let differs = |i: usize| -> Option<(usize, String)> {
        let digit = snapshot.digits.get(i)?;
        live.get(i)
            .is_some_and(|l| l != digit)
            .then(|| (i, digit.clone()))
    };
    let value_writes = match scope {
        ValueScope::None => Vec::new(),
        ValueScope::One(i) => differs(i).into_iter().collect(),
        ValueScope::All => (0..snapshot.digits.len()).filter_map(differs).collect(),
    };
    ReflectPlan {
        value_writes,
        focus: snapshot.focused,
    }
}

/// [`reflect_plan_scoped`] の「1 桁」「なし」だけを受ける互換形（`value_index` が `None` なら値は書かない）。
#[must_use]
pub fn reflect_plan(
    snapshot: Option<&PinInputSnapshot>,
    live: &[String],
    value_index: Option<usize>,
    fallback_focus: Option<usize>,
) -> ReflectPlan {
    let scope = value_index.map_or(ValueScope::None, ValueScope::One);
    reflect_plan_scoped(snapshot, live, scope, fallback_focus)
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        classify_input_value, classify_paste_text, deletion_dispatches, input_dispatches,
        key_dispatches, key_intent, parse_digit_index, paste_dispatches, reflect_plan_scoped,
        snapshot_from_bound, InputValue, KeyIntent, PasteText, ValueScope, ACTION_FOCUS,
        INPUT_PART, PIN_INPUT_SCOPE, ROOT_PART,
    };
    use crate::dom::has_disabled_ancestor;
    use crate::events::{ActionRef, ACTION_INPUT_ATTR};
    use fandhe_frontend_wasm_client::BoundValue;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{ClipboardEvent, Element, Event, HtmlInputElement, InputEvent, KeyboardEvent};

    const INPUT_SELECTOR: &str = r#"[data-scope="pin-input"][data-part="input"]"#;
    const ROOT_SELECTOR: &str = r#"[data-scope="pin-input"][data-part="root"]"#;

    /// capture 段で記録する入力の分類（受理・拒否の判断は含まない）。
    enum PendingKind {
        /// ちょうど 1 文字。
        Single(char),
        /// 2 文字以上。`None` は `MAX_PASTE_CHARS` 超過（dispatch しない）。
        Multi(Option<String>),
        /// 0 文字（削除系）。`inputType`（不明なら `None`）で `backspace`/`delete` を選ぶ。
        Empty { input_type: Option<String> },
    }

    /// capture 段で記録する「どの桁に何が入力されたか」（判断は含まない）。
    struct Pending {
        /// 記録元のイベント（bubble 段で同一イベントかを `Object.is` で照合する）。
        event: Event,
        ordinal: usize,
        index: usize,
        count: usize,
        kind: PendingKind,
        /// `data-action-input` の所有者がいる（dispatch はアプリと `wire_events` が担う）。
        app_owned: bool,
    }

    thread_local! {
        /// フォーカス反映中の `focusin` 再入を抑止するフラグ（wasm は単一スレッド）。
        static REFOCUSING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
        /// capture 段の記録。`input` イベントは同期的に capture → bubble と進むため 1 件で足りる。
        static PENDING: std::cell::RefCell<Option<Pending>> = const { std::cell::RefCell::new(None) };
        /// 直近に `compositionend` を処理した桁 input（Safari 系の確定後の追加 `input` を 1 回だけ
        /// 捨てるためのガード。次の `input`・`keydown` で必ず破棄する）。
        static COMPOSED: std::cell::RefCell<Option<Element>> = const { std::cell::RefCell::new(None) };
    }

    type SharedOnAction<F> = std::rc::Rc<std::cell::RefCell<F>>;
    /// 状態機械の予約 field の読み出し（読めなければ `None`）。
    type ReadBound = std::rc::Rc<dyn Fn(&str) -> Option<BoundValue>>;

    fn is_part(element: &Element, part: &str) -> bool {
        element.get_attribute("data-scope").as_deref() == Some(PIN_INPUT_SCOPE)
            && element.get_attribute("data-part").as_deref() == Some(part)
    }

    /// 最寄りの PinInput Root を探す。
    fn find_instance_root(start: &Element) -> Option<Element> {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if is_part(&element, ROOT_PART) {
                return Some(element);
            }
            current = element.parent_element();
        }
        None
    }

    /// `data-readonly` は最寄りの PinInput Root で打ち切る（インスタンス内に限る、PR #1982）。
    fn has_readonly_ancestor(start: &Element) -> bool {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if element.has_attribute("data-readonly") {
                return true;
            }
            if is_part(&element, ROOT_PART) {
                break;
            }
            current = element.parent_element();
        }
        false
    }

    fn instance_inputs(instance_root: &Element) -> Vec<Element> {
        let Ok(list) = instance_root.query_selector_all(INPUT_SELECTOR) else {
            return Vec::new();
        };
        (0..list.length())
            .filter_map(|i| list.item(i)?.dyn_into::<Element>().ok())
            .collect()
    }

    /// 配線 root 配下（root 自身を含む）の PinInput Root を文書順に列挙する。
    ///
    /// `query_selector_all` は検索元自身を含まないため、配線 root 自身が PinInput Root の
    /// 場合は先頭へ加える（`Runtime` が Root 要素そのものを渡す構成への対応）。
    fn instance_roots(root: &Element) -> Vec<Element> {
        let mut roots = Vec::new();
        if is_part(root, ROOT_PART) {
            roots.push(root.clone());
        }
        if let Ok(list) = root.query_selector_all(ROOT_SELECTOR) {
            roots.extend(
                (0..list.length()).filter_map(|i| list.item(i)?.dyn_into::<Element>().ok()),
            );
        }
        roots
    }

    /// 配線 root 配下で `instance_root` が何番目の PinInput Root か。
    fn root_ordinal(root: &Element, instance_root: &Element) -> Option<usize> {
        instance_roots(root).iter().position(|e| e == instance_root)
    }

    /// 桁 input 以外・操作不能な要素なら `None`。戻り値は (桁 input, 最寄りの Root)。
    fn target_digit(root: &Element, event: &Event) -> Option<(Element, Element)> {
        let target = event.target()?.dyn_into::<Element>().ok()?;
        if !is_part(&target, INPUT_PART) {
            return None;
        }
        if has_disabled_ancestor(root, &target) || has_readonly_ancestor(&target) {
            return None;
        }
        let instance_root = find_instance_root(&target)?;
        Some((target, instance_root))
    }

    /// `data-action-input` を持つ最寄りの祖先（自身を含む）が配線 root 内にあるか。
    ///
    /// `events::attribute_input_action` と同じ `closest` 探索・root 境界で判定する
    /// （祖先の Root やラッパーに属性があっても `wire_events` が dispatch するため）。
    fn has_input_action_owner(root: &Element, target: &Element) -> bool {
        let selector = format!("[{ACTION_INPUT_ATTR}]");
        target
            .closest(&selector)
            .ok()
            .flatten()
            .is_some_and(|matched| root.contains(Some(&matched)))
    }

    /// `ordinal` 番目の Root の桁 input を `data-index` 順（桁数 `count`）に引き直す。
    fn digit_elements(
        root: &Element,
        ordinal: usize,
        count: usize,
    ) -> Vec<Option<HtmlInputElement>> {
        let mut digits: Vec<Option<HtmlInputElement>> = vec![None; count];
        let Some(instance_root) = instance_roots(root).into_iter().nth(ordinal) else {
            return digits;
        };
        for element in instance_inputs(&instance_root) {
            let Some(index) =
                parse_digit_index(element.get_attribute("data-index").as_deref(), count)
            else {
                continue;
            };
            if let Ok(input) = element.dyn_into::<HtmlInputElement>() {
                digits[index] = Some(input);
            }
        }
        digits
    }

    /// 状態機械の状態を読み戻して DOM へ反映する（値は `scope` の範囲だけ、フォーカスは状態の `focused`）。
    ///
    /// 判断は [`reflect_plan_scoped`]（純粋関数）に閉じており、ここは計画を DOM へ適用するだけ。
    /// `fallback_focus` は読み戻せないときの `focusin` 用の復元先（モジュール doc 参照）。
    fn reflect(
        root: &Element,
        ordinal: usize,
        count: usize,
        fallback_focus: Option<usize>,
        scope: ValueScope,
        read_bound: &ReadBound,
    ) {
        let digits = digit_elements(root, ordinal, count);
        let live: Vec<String> = digits
            .iter()
            .map(|d| d.as_ref().map(HtmlInputElement::value).unwrap_or_default())
            .collect();
        let snapshot = snapshot_from_bound(ordinal, count, |field| read_bound(field));
        let plan = reflect_plan_scoped(snapshot.as_ref(), &live, scope, fallback_focus);
        for (index, value) in &plan.value_writes {
            if let Some(Some(input)) = digits.get(*index) {
                input.set_value(value);
            }
        }
        let Some(Some(target)) = plan.focus.and_then(|f| digits.get(f)) else {
            return;
        };
        let active = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.active_element());
        if active.as_ref() != Some(target.unchecked_ref::<Element>()) {
            REFOCUSING.with(|f| f.set(true));
            let _ = target.focus();
            REFOCUSING.with(|f| f.set(false));
        }
    }

    /// `input`（capture / bubble）・`focusin`・`keydown`・`paste`・`compositionend` を `root` へ
    /// 登録する（`Closure::forget` は計 6 回）。
    ///
    /// - `input`（capture）: 入力された桁の場所と値の分類（1 文字・2 文字以上・0 文字）を記録するだけ。
    ///   この時点で要素はまだ差し替わっていない。
    /// - `input`（bubble）: `Runtime` では `wire_events` より後に登録されるため、アプリ所有の
    ///   経路ではアプリの dispatch が済んでから実行される。所有者がいなければここで分類に応じた
    ///   アクションを dispatch し、いずれも状態の読み戻しで DOM を反映する。
    /// - `focusin`: 所有者がいない桁だけ `"focus"` を dispatch して反映する。
    /// - `keydown` / `paste` / `compositionend`: モジュール doc の対応表のとおり。
    ///
    /// `on_action` は dispatch できたとき `true` を返す。`read_bound` は予約 field の読み出し。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback` の失敗を伝播する。
    pub fn wire_pin_input_events(
        root: Element,
        on_action: impl FnMut(ActionRef) -> bool + 'static,
        read_bound: impl Fn(&str) -> Option<BoundValue> + 'static,
    ) -> Result<(), JsValue> {
        let on_action = std::rc::Rc::new(std::cell::RefCell::new(on_action));
        let read_bound: ReadBound = std::rc::Rc::new(read_bound);

        let capture_root = root.clone();
        let capture_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            record_input(&capture_root, &event);
        });
        root.add_event_listener_with_callback_and_bool(
            "input",
            capture_closure.as_ref().unchecked_ref(),
            true,
        )?;
        capture_closure.forget();

        let bubble_root = root.clone();
        let bubble_on_action = on_action.clone();
        let bubble_read = read_bound.clone();
        let bubble_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_input(&bubble_root, &event, &bubble_on_action, &bubble_read);
        });
        root.add_event_listener_with_callback("input", bubble_closure.as_ref().unchecked_ref())?;
        bubble_closure.forget();

        let focus_root = root.clone();
        let focus_on_action = on_action.clone();
        let focus_read = read_bound.clone();
        let focusin_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_focusin(&focus_root, &event, &focus_on_action, &focus_read);
        });
        root.add_event_listener_with_callback("focusin", focusin_closure.as_ref().unchecked_ref())?;
        focusin_closure.forget();

        let key_root = root.clone();
        let key_on_action = on_action.clone();
        let key_read = read_bound.clone();
        let keydown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_keydown(&key_root, &event, &key_on_action, &key_read);
        });
        root.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())?;
        keydown_closure.forget();

        let paste_root = root.clone();
        let paste_on_action = on_action.clone();
        let paste_read = read_bound.clone();
        let paste_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_paste(&paste_root, &event, &paste_on_action, &paste_read);
        });
        root.add_event_listener_with_callback("paste", paste_closure.as_ref().unchecked_ref())?;
        paste_closure.forget();

        let comp_root = root.clone();
        let compositionend_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_compositionend(&comp_root, &event, &on_action, &read_bound);
        });
        root.add_event_listener_with_callback(
            "compositionend",
            compositionend_closure.as_ref().unchecked_ref(),
        )?;
        compositionend_closure.forget();
        Ok(())
    }

    fn call<F: FnMut(ActionRef) -> bool>(on_action: &SharedOnAction<F>, action: ActionRef) -> bool {
        on_action
            .try_borrow_mut()
            .map(|mut f| (f)(action))
            .unwrap_or(false)
    }

    /// 桁 input の値の分類。`Empty` は呼び出し側が `inputType` を添えて [`PendingKind::Empty`] にする。
    enum Classified {
        Empty,
        Kind(PendingKind),
    }

    /// 桁 input の値を分類する。2 文字以上は `MAX_PASTE_CHARS` 以内のときだけテキストを持つ。
    fn classify_live(input: &HtmlInputElement) -> Classified {
        let value = input.value();
        match classify_input_value(&value) {
            InputValue::Empty => Classified::Empty,
            InputValue::Single(c) => Classified::Kind(PendingKind::Single(c)),
            InputValue::Multi => Classified::Kind(PendingKind::Multi(
                matches!(classify_paste_text(&value), PasteText::Text(_)).then_some(value),
            )),
        }
    }

    /// capture 段: 入力された桁の場所と値の分類を記録する（受理・拒否の判断はしない）。
    fn record_input(root: &Element, event: &Event) {
        PENDING.with(|p| *p.borrow_mut() = None);
        let composed = COMPOSED.with(|c| c.borrow_mut().take());
        let input_event = event.dyn_ref::<InputEvent>();
        // IME 変換中は確定前の文字列を扱わない（fail-closed）。
        if input_event.is_some_and(InputEvent::is_composing) {
            return;
        }
        let input_type = input_event.map(InputEvent::input_type);
        let Some((target, instance_root)) = target_digit(root, event) else {
            return;
        };
        // `compositionend` で処理済みの桁への、確定後の追加 `input`（Safari 系）は二重適用しない。
        if composed.as_ref() == Some(&target)
            && matches!(
                input_type.as_deref(),
                Some("insertFromComposition" | "insertCompositionText")
            )
        {
            return;
        }
        let Some(input) = target.dyn_ref::<HtmlInputElement>() else {
            return;
        };
        let kind = match classify_live(input) {
            Classified::Kind(kind) => kind,
            Classified::Empty => PendingKind::Empty { input_type },
        };
        let count = instance_inputs(&instance_root).len();
        let Some(index) = parse_digit_index(target.get_attribute("data-index").as_deref(), count)
        else {
            return;
        };
        let Some(ordinal) = root_ordinal(root, &instance_root) else {
            return;
        };
        let app_owned = has_input_action_owner(root, &target);
        PENDING.with(|p| {
            *p.borrow_mut() = Some(Pending {
                event: event.clone(),
                ordinal,
                index,
                count,
                kind,
                app_owned,
            });
        });
    }

    /// bubble 段: 所有者がいなければ状態機械へ dispatch し、状態を読み戻して DOM へ反映する。
    fn handle_input<F: FnMut(ActionRef) -> bool>(
        root: &Element,
        event: &Event,
        on_action: &SharedOnAction<F>,
        read_bound: &ReadBound,
    ) {
        let Some(pending) = PENDING.with(|p| p.borrow_mut().take()) else {
            return;
        };
        // 別イベントの記録が残っていた場合は処理しない（fail-closed）。
        if !js_sys::Object::is(pending.event.as_ref(), event.as_ref()) {
            return;
        }
        let one = ValueScope::One(pending.index);
        let (actions, scope) = match &pending.kind {
            PendingKind::Single(c) => (Some(input_dispatches(pending.index, *c)), one),
            PendingKind::Multi(Some(text)) => {
                (Some(paste_dispatches(pending.index, text)), ValueScope::All)
            }
            PendingKind::Multi(None) => (None, one),
            PendingKind::Empty { input_type } => (
                Some(deletion_dispatches(pending.index, input_type.as_deref())),
                one,
            ),
        };
        if pending.app_owned {
            // アプリ所有の桁は dispatch せず、値の書き込みもその桁だけにとどめる。
            reflect(root, pending.ordinal, pending.count, None, one, read_bound);
            return;
        }
        for action in actions.into_iter().flatten() {
            call(on_action, action);
        }
        reflect(
            root,
            pending.ordinal,
            pending.count,
            None,
            scope,
            read_bound,
        );
    }

    /// 対象桁の解決結果（keydown・paste で共有）。
    struct Digit {
        ordinal: usize,
        index: usize,
        count: usize,
    }

    /// 操作可能で所有者のいない桁だけ返す（アプリ所有の桁は汎用 dispatch を送らない、モジュール doc 参照）。
    fn resolve_generic_digit(root: &Element, event: &Event) -> Option<Digit> {
        let (target, instance_root) = target_digit(root, event)?;
        if has_input_action_owner(root, &target) {
            return None;
        }
        let count = instance_inputs(&instance_root).len();
        let index = parse_digit_index(target.get_attribute("data-index").as_deref(), count)?;
        let ordinal = root_ordinal(root, &instance_root)?;
        Some(Digit {
            ordinal,
            index,
            count,
        })
    }

    /// keydown: Backspace/Delete/ArrowLeft/ArrowRight を処理したときだけ `preventDefault` する。
    fn handle_keydown<F: FnMut(ActionRef) -> bool>(
        root: &Element,
        event: &Event,
        on_action: &SharedOnAction<F>,
        read_bound: &ReadBound,
    ) {
        // 次の `input` へ Safari 系の確定後ガードを持ち越さない。
        COMPOSED.with(|c| *c.borrow_mut() = None);
        let Some(key) = event.dyn_ref::<KeyboardEvent>() else {
            return;
        };
        let Some(intent) = key_intent(
            &key.key(),
            key.ctrl_key(),
            key.alt_key(),
            key.shift_key(),
            key.meta_key(),
            key.is_composing(),
            key.key_code(),
        ) else {
            return;
        };
        let Some(digit) = resolve_generic_digit(root, event) else {
            return;
        };
        // ネイティブの削除・キャレット移動を止める。これが汎用 keydown 配線との排他にもなる。
        event.prevent_default();
        for action in key_dispatches(digit.index, intent) {
            call(on_action, action);
        }
        let scope = match intent {
            KeyIntent::Backspace | KeyIntent::Delete => ValueScope::One(digit.index),
            KeyIntent::Prev | KeyIntent::Next => ValueScope::None,
        };
        reflect(root, digit.ordinal, digit.count, None, scope, read_bound);
    }

    /// paste: `clipboardData` の `text/plain` を無加工で `"paste"` へ渡す。読めなければ何もしない。
    fn handle_paste<F: FnMut(ActionRef) -> bool>(
        root: &Element,
        event: &Event,
        on_action: &SharedOnAction<F>,
        read_bound: &ReadBound,
    ) {
        let Some(clipboard) = event.dyn_ref::<ClipboardEvent>() else {
            return;
        };
        let Some(data) = clipboard.clipboard_data() else {
            return;
        };
        let Some(digit) = resolve_generic_digit(root, event) else {
            return;
        };
        let Ok(text) = data.get_data("text/plain") else {
            return;
        };
        // 生のクリップボード文字列を DOM の桁へ入れない（反映は状態機械の値だけ）。
        event.prevent_default();
        let PasteText::Text(text) = classify_paste_text(&text) else {
            return;
        };
        for action in paste_dispatches(digit.index, text) {
            call(on_action, action);
        }
        reflect(
            root,
            digit.ordinal,
            digit.count,
            None,
            ValueScope::All,
            read_bound,
        );
    }

    /// compositionend: 桁の確定値を 1 文字なら `input`、2 文字以上なら `paste` として渡す。
    fn handle_compositionend<F: FnMut(ActionRef) -> bool>(
        root: &Element,
        event: &Event,
        on_action: &SharedOnAction<F>,
        read_bound: &ReadBound,
    ) {
        let Some((target, instance_root)) = target_digit(root, event) else {
            return;
        };
        let Some(input) = target.dyn_ref::<HtmlInputElement>() else {
            return;
        };
        let count = instance_inputs(&instance_root).len();
        let (Some(index), Some(ordinal)) = (
            parse_digit_index(target.get_attribute("data-index").as_deref(), count),
            root_ordinal(root, &instance_root),
        ) else {
            return;
        };
        let (actions, scope) = match classify_live(input) {
            // 変換の取り消し（確定値なし）は何もしない。
            Classified::Empty => return,
            Classified::Kind(PendingKind::Single(c)) => {
                (Some(input_dispatches(index, c)), ValueScope::One(index))
            }
            Classified::Kind(PendingKind::Multi(Some(text))) => {
                (Some(paste_dispatches(index, &text)), ValueScope::All)
            }
            Classified::Kind(_) => (None, ValueScope::One(index)),
        };
        COMPOSED.with(|c| *c.borrow_mut() = Some(target.clone()));
        if has_input_action_owner(root, &target) {
            // アプリ所有の桁は dispatch せず、値の書き込みもその桁だけにとどめる。
            reflect(
                root,
                ordinal,
                count,
                None,
                ValueScope::One(index),
                read_bound,
            );
            return;
        }
        for action in actions.into_iter().flatten() {
            call(on_action, action);
        }
        reflect(root, ordinal, count, None, scope, read_bound);
    }

    fn handle_focusin<F: FnMut(ActionRef) -> bool>(
        root: &Element,
        event: &Event,
        on_action: &SharedOnAction<F>,
        read_bound: &ReadBound,
    ) {
        if REFOCUSING.with(std::cell::Cell::get) {
            return;
        }
        let Some((target, instance_root)) = target_digit(root, event) else {
            return;
        };
        // 所有者（アプリの action）がいる桁へはインスタンス情報のない汎用 "focus" を送らない
        // （モジュール doc「アプリ所有の経路」）。
        if has_input_action_owner(root, &target) {
            return;
        }
        let count = instance_inputs(&instance_root).len();
        let Some(index) = parse_digit_index(target.get_attribute("data-index").as_deref(), count)
        else {
            return;
        };
        let Some(ordinal) = root_ordinal(root, &instance_root) else {
            return;
        };
        let dispatched = call(
            on_action,
            ActionRef {
                action: ACTION_FOCUS.to_string(),
                payload: index.to_string(),
            },
        );
        if !dispatched {
            return;
        }
        // 再描画（構造フォールバック）で桁 input が差し替わるとフォーカスを失うため、
        // 状態機械の focused（読めなければ選ばれた桁）へ引き直した要素でフォーカスを戻す。
        reflect(
            root,
            ordinal,
            count,
            Some(index),
            ValueScope::None,
            read_bound,
        );
    }

    /// [`wire_pin_input_events`] に `Component` の dispatch と状態の読み戻しを接続する。
    ///
    /// dispatch できたら `on_update` で再描画し `true` を返す。読み戻しは `C` の
    /// `BindingSource::bound_value`（予約 field、モジュール doc 参照）。`Runtime::mount`/`hydrate`
    /// から `Runtime::wire_pin_input` 経由で呼ばれる。
    ///
    /// # Errors
    ///
    /// [`wire_pin_input_events`] の失敗を伝播する。
    pub fn wire_pin_input_component<C>(
        root: Element,
        component: std::rc::Rc<std::cell::RefCell<C>>,
        on_update: impl FnMut(&C, &Element) + 'static,
    ) -> Result<(), JsValue>
    where
        C: fandhe_frontend_interactive::Component
            + fandhe_frontend_wasm_client::BindingSource
            + 'static,
    {
        let on_update = std::rc::Rc::new(std::cell::RefCell::new(on_update));
        let wired_root = root.clone();
        let view_component = component.clone();
        let read_bound = move |field: &str| {
            view_component
                .try_borrow()
                .ok()
                .and_then(|state| state.bound_value(field))
        };
        wire_pin_input_events(
            root,
            move |action_ref: ActionRef| {
                let Ok(mut state) = component.try_borrow_mut() else {
                    return false;
                };
                let dispatched = fandhe_frontend_interactive::dispatch(
                    &mut *state,
                    &action_ref.action,
                    &action_ref.payload,
                );
                if !dispatched {
                    return false;
                }
                if let Ok(mut cb) = on_update.try_borrow_mut() {
                    (cb)(&state, &wired_root);
                }
                true
            },
            read_bound,
        )
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{wire_pin_input_component, wire_pin_input_events};
