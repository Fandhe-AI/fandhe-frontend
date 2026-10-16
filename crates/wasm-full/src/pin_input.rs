//! PinInput（`fandhe-frontend-headless-ui` `pin_input` モジュール）の
//! `input` / `focusin` → dispatch 配線（イシュー #3834、親 #3831）。
//!
//! # 背景
//!
//! `crates/headless-ui/src/pin_input.rs` は桁ごとの `input` パーツ
//! （`data-index`・`maxlength="1"`）と `"input"`/`"focus"` 等の dispatch 契約
//! （`PinInput::decode_action`）を提供する一方、実 DOM のイベントを dispatch へ
//! つなぐ配線を「wasm 層の後続責務」として残していた。本モジュールは
//! そのうち **1 文字入力と次桁へのフォーカス移動** だけを実装する。
//! 貼り付け・Backspace/Delete/矢印キー・実ブラウザ検証は後続の #3835/#3836 の担当で、
//! 本モジュールは `paste`/`keydown` に一切触れない。
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
//! 流れは `input`（capture: 入力された桁の場所と 1 文字の記録のみ）→ 状態機械へ dispatch
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
//! | DOM イベント | 条件 | アクション | payload |
//! |---|---|---|---|
//! | `input`（桁 input） | 値がちょうど 1 文字（Root 内で操作可能） | `"focus"` → `"input"` | `data-index` → その 1 文字（無加工） |
//! | `focusin`（桁 input） | `data-index` が桁数未満 | `"focus"` | `data-index` |
//!
//! `"focus"` を `"input"` の直前に送るのは、アプリ状態の focused が古くても DOM で入力された桁へ
//! 書き込ませるため。0 文字（削除系）と 2 文字以上（autofill・`insertFromPaste`）は何もしない
//! （DOM の値も消さない）。これらは #3835 が `"paste"` 等へ接続する。
//!
//! # 本モジュールが状態にも DOM の値にも触れない入力（#3835 の担当）
//!
//! - 0 文字（ネイティブの Backspace/Delete）・2 文字以上（貼り付け・autofill）: dispatch せず、
//!   DOM の値も書き換えない。状態機械は変わらないため、DOM と状態が食い違ったままになりうる。
//! - **IME 確定入力は未対応**: `isComposing` 中の `input` は無視する。Chrome・Firefox では
//!   `isComposing=true` の最後の `input` の後に `compositionend` が来て、その後に `input` は来ないため、
//!   確定した文字は状態機械へ渡らず DOM に残る。`compositionstart`/`compositionend` の扱いは #3835
//!   で実装する（`events::wire_events` の `compositionend` 補完〔#2069〕は `data-action-input` 経路用で、
//!   汎用経路の桁には効かない）。
//!
//! DOM の値の書き込みは「`input` のあった桁 1 つ」に限る（ネイティブ削除済みの別の桁の値を
//! 状態から復活させないため）。
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
//!
//! # 配線の回数・入れ子・dispatch 回数
//!
//! - 配線は 1 root につき 1 回。入れ子の配線 root では capture の記録（`PENDING`）が 1 スロットのため、
//!   内側の配線だけが反映する。
//! - 1 打鍵で `"focus"` と `"input"` の 2 回 dispatch するため、`on_update` も 2 回走る。
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
//! 本モジュールは文字種の判定も値の検証も行わない。値の正しさ（kind への適合）は
//! 状態機械とアプリ側の責務のままである。
//!
//! # セキュリティ不変条件
//!
//! - `input.value` は先頭 2 文字までしか読まない。`data-index` は `usize` へパースし桁数未満を
//!   検査する（fail-closed）。DOM へ書く値は状態機械が返した桁の値（1 文字以下）に限り、
//!   `HtmlInputElement::set_value` のみを使う。HTML 文字列は組み立てない（REQ-1）。
//! - リスナーは配線 1 回につき 3 個（`Closure::forget` 3 回: `input` capture・`input` bubble・`focusin`）で固定する。
//! - PIN/OTP は秘密値のため、値・payload を console やエラーに出さない。

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

/// `BindingSource` 予約 field の接頭辞（モジュール doc「読み戻しの契約」参照）。
const FIELD_PREFIX: &str = "pin-input.";

/// `input` イベント後の `input.value` の分類（先頭 2 文字だけを読んだ結果）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputValue {
    /// 空文字（削除系。#3835 の担当のため何もしない）。
    Empty,
    /// ちょうど 1 文字。
    Single(char),
    /// 2 文字以上（autofill・貼り付け。#3835 の担当のため何もしない）。
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
/// 分類は「どのアクションへ送るか（1 文字入力か、#3835 の担当か）」の振り分けだけで、
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

/// 状態の読み戻し結果から DOM 反映の計画を作る（純粋関数）。
///
/// - `snapshot` があれば、値は `value_index`（`input` のあった桁）だけを状態機械の値へそろえる
///   （他の桁は触らない。ネイティブ削除などで状態と食い違う桁を復活させないため）。
///   フォーカスは状態機械の `focused` へ置く（受理・拒否・正規化・次桁への前進はすべて状態機械の結果）。
///   `value_index` が `None`（`focusin`）なら値は書かない。
/// - `snapshot` が無ければ（読み戻せない）値は触らず、フォーカスは `fallback_focus`
///   （`focusin` でユーザーが選んだ桁の復元）にだけ使う。
#[must_use]
pub fn reflect_plan(
    snapshot: Option<&PinInputSnapshot>,
    live: &[String],
    value_index: Option<usize>,
    fallback_focus: Option<usize>,
) -> ReflectPlan {
    let Some(snapshot) = snapshot else {
        return ReflectPlan {
            value_writes: Vec::new(),
            focus: fallback_focus,
        };
    };
    let value_writes = value_index
        .and_then(|i| {
            let digit = snapshot.digits.get(i)?;
            live.get(i)
                .is_some_and(|l| l != digit)
                .then(|| (i, digit.clone()))
        })
        .into_iter()
        .collect();
    ReflectPlan {
        value_writes,
        focus: snapshot.focused,
    }
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        classify_input_value, input_dispatches, parse_digit_index, reflect_plan,
        snapshot_from_bound, InputValue, ACTION_FOCUS, INPUT_PART, PIN_INPUT_SCOPE, ROOT_PART,
    };
    use crate::dom::has_disabled_ancestor;
    use crate::events::{ActionRef, ACTION_INPUT_ATTR};
    use fandhe_frontend_wasm_client::BoundValue;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, HtmlInputElement, InputEvent};

    const INPUT_SELECTOR: &str = r#"[data-scope="pin-input"][data-part="input"]"#;
    const ROOT_SELECTOR: &str = r#"[data-scope="pin-input"][data-part="root"]"#;

    /// capture 段で記録する「どの桁に何が入力されたか」（判断は含まない）。
    struct Pending {
        /// 記録元のイベント（bubble 段で同一イベントかを `Object.is` で照合する）。
        event: Event,
        ordinal: usize,
        index: usize,
        count: usize,
        typed: char,
        /// `data-action-input` の所有者がいる（dispatch はアプリと `wire_events` が担う）。
        app_owned: bool,
    }

    thread_local! {
        /// フォーカス反映中の `focusin` 再入を抑止するフラグ（wasm は単一スレッド）。
        static REFOCUSING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
        /// capture 段の記録。`input` イベントは同期的に capture → bubble と進むため 1 件で足りる。
        static PENDING: std::cell::RefCell<Option<Pending>> = const { std::cell::RefCell::new(None) };
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

    /// 状態機械の状態を読み戻して DOM へ反映する（値は `value_index` の桁だけ、フォーカスは状態の `focused`）。
    ///
    /// 判断は [`reflect_plan`]（純粋関数）に閉じており、ここは計画を DOM へ適用するだけ。
    /// `fallback_focus` は読み戻せないときの `focusin` 用の復元先（モジュール doc 参照）。
    fn reflect(
        root: &Element,
        ordinal: usize,
        count: usize,
        fallback_focus: Option<usize>,
        value_index: Option<usize>,
        read_bound: &ReadBound,
    ) {
        let digits = digit_elements(root, ordinal, count);
        let live: Vec<String> = digits
            .iter()
            .map(|d| d.as_ref().map(HtmlInputElement::value).unwrap_or_default())
            .collect();
        let snapshot = snapshot_from_bound(ordinal, count, |field| read_bound(field));
        let plan = reflect_plan(snapshot.as_ref(), &live, value_index, fallback_focus);
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

    /// `input`（capture / bubble）と `focusin` を `root` へ登録する（`Closure::forget` は計 3 回）。
    ///
    /// - `input`（capture）: 入力された桁の場所と 1 文字を記録するだけ。この時点で要素は
    ///   まだ差し替わっていない。
    /// - `input`（bubble）: `Runtime` では `wire_events` より後に登録されるため、アプリ所有の
    ///   経路ではアプリの dispatch が済んでから実行される。所有者がいなければここで
    ///   `"focus"` → `"input"` を dispatch し、いずれも状態の読み戻しで DOM を反映する。
    /// - `focusin`: 所有者がいない桁だけ `"focus"` を dispatch して反映する。
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
        let focusin_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_focusin(&focus_root, &event, &on_action, &read_bound);
        });
        root.add_event_listener_with_callback("focusin", focusin_closure.as_ref().unchecked_ref())?;
        focusin_closure.forget();
        Ok(())
    }

    fn call<F: FnMut(ActionRef) -> bool>(on_action: &SharedOnAction<F>, action: ActionRef) -> bool {
        on_action
            .try_borrow_mut()
            .map(|mut f| (f)(action))
            .unwrap_or(false)
    }

    /// capture 段: 1 文字入力された桁の場所と文字を記録する（受理・拒否の判断はしない）。
    fn record_input(root: &Element, event: &Event) {
        PENDING.with(|p| *p.borrow_mut() = None);
        // IME 変換中は確定前の文字列を扱わない（fail-closed）。
        if event
            .dyn_ref::<InputEvent>()
            .is_some_and(InputEvent::is_composing)
        {
            return;
        }
        let Some((target, instance_root)) = target_digit(root, event) else {
            return;
        };
        let Some(input) = target.dyn_ref::<HtmlInputElement>() else {
            return;
        };
        let InputValue::Single(typed) = classify_input_value(&input.value()) else {
            return;
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
                typed,
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
        if !pending.app_owned {
            let [focus, value] = input_dispatches(pending.index, pending.typed);
            call(on_action, focus);
            call(on_action, value);
        }
        reflect(
            root,
            pending.ordinal,
            pending.count,
            None,
            Some(pending.index),
            read_bound,
        );
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
        reflect(root, ordinal, count, Some(index), None, read_bound);
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
