//! NumberInput（`fandhe-frontend-headless-ui` `number_input` モジュール）の
//! keydown / click → dispatch 配線（イシュー #1613、PR #1881 codex-review
//! P1 是正／イシュー #1962: IncrementTrigger/DecrementTrigger の click 配線を追加）。
//!
//! # 背景
//!
//! `crates/headless-ui/src/number_input.rs` は Root/Label/Control/Input/
//! IncrementTrigger/DecrementTrigger/ValueText の 7 anatomy パーツと
//! `"increment"`/`"decrement"`/`"set"`/`"clear"`/`"home"`/`"end"` dispatch
//! 契約を提供する一方、実際に `input`（`role="spinbutton"`）上のキー入力を
//! dispatch へ接続する DOM 配線は同モジュール冒頭 rustdoc が「クライアント
//! ランタイム側の後続責務」と申し送っていた。本モジュールがその配線を
//! 実装する。
//!
//! この申し送り文言（「本イシューでも新設しない」）は、同時に参考サイト
//! 突合された [`crate::angle_slider`]（イシュー #1601、Arrow キーのみ配線
//! 済み）・`crate::keynav`（Combobox、イシュー #1071）が実際には自身の
//! イシュー内で keydown 配線を新設した実績と矛盾しており、PR #1881
//! codex-review（P1）が「`crates/docs-site/src/primitive_specs/forms_b.rs`
//! の keyboard 一覧が対応済みとして表示する ArrowUp/ArrowDown/Home/End/
//! Enter がいずれも実配線を持たず操作不能」と指摘した。本モジュールは
//! その是正として keydown 配線を新設する（headless-ui 側の rustdoc も
//! 追随して更新済み）。
//!
//! # 設計（`angle_slider.rs`/`headless_signature_pad.rs` と同型の 2 層構成）
//!
//! - 純粋ロジック層（[`action_for_key`]）は web-sys に依存せず、native の
//!   `cargo test` で決定的に検証できる。
//! - 配線層（[`wire_number_input_events`]/[`wire_number_input_component`]）
//!   のみ `#[cfg(target_arch = "wasm32")]` でゲートする。
//!
//! IncrementTrigger/DecrementTrigger の click（イシュー #1962）も、
//! `crate::headless::MAPPING_TABLE`（クリック委譲の静的マッピング表）へは
//! 乗せない。`MAPPING_TABLE` 方式は解決したアクションの payload に
//! `data-value` 属性値を使う契約だが、NumberInput は複数インスタンスの
//! 識別を Input パーツの `name` 属性値・`data-action-input` 上書き名
//! （下記「複数インスタンスの識別」節）で行っており、`MAPPING_TABLE` へ
//! 乗せるには headless-ui 側に `data-value` 相当の新規属性出力が要る
//! （親イシュー #1961 の方針）。keydown・click いずれも
//! [`crate::angle_slider`]/[`crate::splitter`] と同じく独立配線モジュールと
//! して切り出し、両イベントとも本モジュールの純粋関数
//! [`resolve_dispatches`] を共有する。
//!
//! # dispatch とアクションの対応
//!
//! | キー | アクション | payload |
//! |---|---|---|
//! | `ArrowUp` | （同期）`"set"`（または後述の上書き名） → `"increment"` | `input.value` → Input の `name` 属性値 |
//! | `ArrowDown` | （同期）`"set"`（または後述の上書き名） → `"decrement"` | `input.value` → Input の `name` 属性値 |
//! | `Home` | `"home"` | Input の `name` 属性値 |
//! | `End` | `"end"` | Input の `name` 属性値 |
//! | `Enter`（`input.value` が空でない） | `"set"`（または後述の上書き名） | `input` 要素の現在の `value`（未確定のタイプ中文字列） |
//! | `Enter`（`input.value` が trim 後空文字） | `"clear"` | Input の `name` 属性値 |
//! | IncrementTrigger click | （同期）`"set"`（または後述の上書き名） → `"increment"` | `input.value` → Input の `name` 属性値 |
//! | DecrementTrigger click | （同期）`"set"`（または後述の上書き名） → `"decrement"` | `input.value` → Input の `name` 属性値 |
//!
//! # 複数インスタンスの識別（PR #1881 codex-review P1 是正）
//!
//! `Runtime::mount`/`Runtime::hydrate`（`crate::lib::Runtime`）はアプリ全体の
//! root へ 1 回だけ本モジュールの keydown リスナーを登録するため、同一
//! root 配下に複数の NumberInput（例: 数量と価格）がある場合、アプリの
//! 単一 `Component::decode_action` は dispatch された `(action, payload)`
//! だけで更新先を区別できなければならない。当初の実装は
//! `"set"`/`"increment"` 等の固定アクション名のみを dispatch しており、
//! どちらの Input で ArrowUp を押しても同一の `(action, payload)` になり
//! 区別不能だった（codex-review P1 指摘）。本モジュールは以下の 2 つの
//! **既存契約**を再利用してこれを解消する（新しい payload エンコーディング
//! は発明しない）:
//!
//! - **Set**（Enter 確定・Arrow 前の同期 `"set"`）: Input パーツに
//!   `data-action-input` 属性（[`crate::events::ACTION_INPUT_ATTR`]、
//!   `crate::events` の input イベント配線と同じ属性契約）があれば、その
//!   値をアプリ定義のアクション名としてそのまま使う（例:
//!   `data-action-input="price_set"`）。無ければ従来どおり固定名
//!   [`ACTION_SET`] のまま（単一インスタンスモード、後方互換）。payload は
//!   いずれの場合も `input.value` そのもの。
//! - **Increment/Decrement/Home/End/Clear**: アクション名は固定のまま
//!   変えず、payload へ Input パーツの `name` 属性値を載せる
//!   （[`crate::splitter`] が trigger index を payload に載せて複数
//!   トリガーを識別する設計と同型）。`name` は
//!   [`fandhe_frontend_headless_ui::number_input::input`] の必須引数で
//!   あり常に出力されるため、追加の属性契約を新設する必要がない。
//!
//! アプリ側は例えば以下のように `decode_action` を書いて 2 インスタンスを
//! 区別できる:
//!
//! ```text
//! fn decode_action(name: &str, payload: &str) -> Option<Action> {
//!     match name {
//!         "price_set" => payload.parse().ok().map(Action::SetPrice),
//!         "qty_set" => payload.parse().ok().map(Action::SetQty),
//!         "increment" if payload == "qty" => Some(Action::IncrementQty),
//!         "increment" if payload == "price" => Some(Action::IncrementPrice),
//!         // ...
//!         _ => None,
//!     }
//! }
//! ```
//!
//! `data-action-input` を付けない・`name` を区別しない単一インスタンス
//! アプリ（[`fandhe_frontend_headless_ui::number_input::NumberInput`] 自身を
//! `Component` として使う経路。`crates/wasm-full/tests/number_input_browser.rs`
//! 参照）では、[`fandhe_frontend_headless_ui::number_input::NumberInput::decode_action`]
//! がこれらの payload をすべて無視するため挙動は変わらない（後方互換）。
//!
//! `ArrowUp`/`ArrowDown` は、キャレット確定前にタイプ中の `input.value` が
//! 状態値と食い違っているケース（例: 状態値 5 のまま入力欄を 8 に書き換えて
//! ArrowUp）で編集前の状態値を基準に増減すると実利用者の目に見える表示値と
//! 矛盾する（PR #1881 codex-review P1 是正その 1）。これを避けるため、
//! 増減アクションの **直前** に `input.value` を `"set"` として同期
//! dispatch してから増減する（1 回のキー操作で 2 アクションを dispatch
//! する）。`input.value` が数値としてパース不能・非有限な場合、`"set"` は
//! [`fandhe_frontend_headless_ui::number_input::NumberInput::decode_action`]
//! が no-op（`None`）として fail-closed に無視するため、増減は編集前の
//! 状態値のまま行われる（「不正な入力は破棄し状態値基準で増減する」契約）。
//! `Home`/`End` は同期を行わない（`min`/`max` への絶対設定であり、タイプ中
//! の値に依存しないため元々矛盾が生じない）。
//!
//! `Enter` は、`input.value` を trim した結果が空文字列であれば未入力状態
//! （`NumberInputAction::Clear`）へ、それ以外は従来どおり `"set"` へ分岐
//! する（PR #1881 codex-review P1 是正その 2。空欄確定時に旧値が残留する
//! 不具合の是正）。
//!
//! `"set"` の payload はキャレット確定前のテキストそのものであり、
//! [`fandhe_frontend_headless_ui::number_input::NumberInput::decode_action`]
//! が改めて `str::parse::<f64>()` + 有限性検証で fail-closed に扱う
//! （不正な文字列は no-op、多層防御）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - **修飾キー（Shift/Alt/Ctrl+Arrow）による step 倍率**:
//!   `fandhe_frontend_headless_ui::number_input::NumberInput` の状態機械
//!   自体が倍率 API を持たない（`crates/headless-ui/src/number_input.rs`
//!   モジュール doc「非追随」節）ため、本モジュールも対応しない。
//! - **IncrementTrigger/DecrementTrigger ボタンのクリック配線**:
//!   イシュー #1962（親 #1961）で回収済み（[`wiring::wire_number_input_events`]
//!   が keydown と同一 root へ click リスナーも登録する）。
//! - **クリック後の Input へのフォーカス復帰**: 参考実装（ark-ui）はトリガー
//!   の `pointerdown` を `preventDefault` して Input のフォーカスを維持する
//!   挙動を持つが、本モジュールは対応しない（別事象として追跡）。
//!
//! # セキュリティ不変条件
//!
//! - dispatch payload（`"set"` の文字列）は
//!   [`fandhe_frontend_headless_ui::number_input::NumberInput::decode_action`]
//!   が改めて厳密パース・有限性検証する（本モジュールは payload 文字列を
//!   組み立てるのみ、多層防御）。
//! - `data-disabled` **または** `data-readonly` を持つ input/祖先パーツ上の
//!   keydown は no-op（[`has_noninteractive_ancestor`]、`crate::angle_slider`
//!   の `has_noninteractive_ancestor` と同型の fail-closed 判定）。IncrementTrigger/
//!   DecrementTrigger の click も同じ判定を再利用する（[`wiring::handle_click`]）。
//!   ネイティブ `disabled` を持つ `<button>` はブラウザ自体が click を発火
//!   させないため、この判定は多層防御の位置づけである。
//! - click 経路は Trigger 要素・Input パーツがいずれも解決できた場合のみ
//!   dispatch する（[`wiring::handle_click`]）。トリガー要素が見つからない・
//!   Input パーツが見つからない・`data-part` が未知の値であるケースは
//!   すべて早期 return（fail-closed、no-op）とし、独自の境界（min/max）
//!   計算は一切行わない（clamp はヘッドレス側の状態機械・トリガー
//!   `disabled` 出力に委ねる）。
//! - DOM 反映は `set_attribute`/`get_attribute`/`value` プロパティ読み取りの
//!   みで行い、HTML 文字列を一切組み立てない（REQ-1）。属性名・イベント名は
//!   すべて `&'static str` リテラル。
//! - 新規 `unsafe` コードは追加しない（`web-sys`/`js-sys` の safe API のみ
//!   使用）。
//! - IME 変換中（`KeyboardEvent::is_composing()` が `true`、または互換
//!   シグナル `key_code() == 229`）の keydown は
//!   [`wiring::handle_keydown`] が早期 return で除外し、`prevent_default()`
//!   も `on_action` dispatch も一切行わない（PR #1881 codex-review P1
//!   是正その 3。変換中の候補選択キーで数値が意図せず上書きされることを
//!   防ぐ）。

use crate::events::{ActionRef, AttrSource, ACTION_INPUT_ATTR};
use crate::keynav::Modifiers;

/// dispatch アクション名 `"increment"`。
pub const ACTION_INCREMENT: &str = "increment";
/// dispatch アクション名 `"decrement"`。
pub const ACTION_DECREMENT: &str = "decrement";
/// dispatch アクション名 `"home"`。
pub const ACTION_HOME: &str = "home";
/// dispatch アクション名 `"end"`。
pub const ACTION_END: &str = "end";
/// dispatch アクション名 `"set"`。
pub const ACTION_SET: &str = "set";
/// dispatch アクション名 `"clear"`（Enter 確定時に `input.value` が
/// trim 後空文字の場合に使う、PR #1881 codex-review P1 是正）。
pub const ACTION_CLEAR: &str = "clear";

/// keydown から決定される操作種別（純粋層、web-sys 非依存）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// `step` 分の増加（[`ACTION_INCREMENT`]、payload なし）。
    Increment,
    /// `step` 分の減少（[`ACTION_DECREMENT`]、payload なし）。
    Decrement,
    /// `min` へ設定（[`ACTION_HOME`]、payload なし）。
    Home,
    /// `max` へ設定（[`ACTION_END`]、payload なし）。
    End,
    /// タイプ中の値を確定する（[`ACTION_SET`]、payload は `input.value`）。
    Set,
}

impl KeyAction {
    /// dispatch アクション名（`&'static str`）。
    #[must_use]
    pub fn action_name(self) -> &'static str {
        match self {
            KeyAction::Increment => ACTION_INCREMENT,
            KeyAction::Decrement => ACTION_DECREMENT,
            KeyAction::Home => ACTION_HOME,
            KeyAction::End => ACTION_END,
            KeyAction::Set => ACTION_SET,
        }
    }
}

/// キー名 + 修飾キーから [`KeyAction`] を決定する純粋関数（DOM 非依存、
/// native `cargo test` で検証可能）。
///
/// 修飾キー（Ctrl/Alt/Meta）付きは常に `None`（[`Modifiers::any`]、
/// `crate::keynav`/`crate::angle_slider` と同じ安全側判断。Shift+Arrow の
/// step 倍率は状態機械側に API がないため対応しない、モジュール冒頭 doc
/// 「スコープ外」節参照）。それ以外の未知キーも `None`（no-op）。
#[must_use]
pub fn action_for_key(key: &str, modifiers: Modifiers) -> Option<KeyAction> {
    if modifiers.any() {
        return None;
    }
    match key {
        "ArrowUp" => Some(KeyAction::Increment),
        "ArrowDown" => Some(KeyAction::Decrement),
        "Home" => Some(KeyAction::Home),
        "End" => Some(KeyAction::End),
        "Enter" => Some(KeyAction::Set),
        _ => None,
    }
}

/// IncrementTrigger パーツの `data-part` 属性値
/// （`fandhe_frontend_headless_ui::number_input::increment_trigger`
/// が出力する固定値と一致、イシュー #1962）。
pub const INCREMENT_TRIGGER_PART: &str = "increment-trigger";
/// DecrementTrigger パーツの `data-part` 属性値
/// （`fandhe_frontend_headless_ui::number_input::decrement_trigger`
/// が出力する固定値と一致、イシュー #1962）。
pub const DECREMENT_TRIGGER_PART: &str = "decrement-trigger";

/// クリックされたトリガーパーツの `data-part` 値から [`KeyAction`] を
/// 決定する純粋関数（DOM 非依存、native `cargo test` で検証可能）。
///
/// keydown 用 [`action_for_key`] と同型の役割で、click 配線層
/// （[`wiring::handle_click`]）から呼ばれる。[`INCREMENT_TRIGGER_PART`]/
/// [`DECREMENT_TRIGGER_PART`] 以外（`"input"`/`"root"`/`"control"`/未知の
/// 文字列）は `None`（no-op）。決定した [`KeyAction`] は keydown と同じ
/// [`resolve_dispatches`] へそのまま渡され、Increment/Decrement は
/// 「タイプ中の値を `"set"` として同期してから増減する」契約を共有する
/// （モジュール冒頭 doc「dispatch とアクションの対応」節参照）。
#[must_use]
pub fn action_for_trigger_part(part: &str) -> Option<KeyAction> {
    match part {
        p if p == INCREMENT_TRIGGER_PART => Some(KeyAction::Increment),
        p if p == DECREMENT_TRIGGER_PART => Some(KeyAction::Decrement),
        _ => None,
    }
}

/// Input パーツの `name` 属性値を dispatch payload として読む
/// （空文字許容・欠落時は空文字列、モジュール冒頭 doc「複数インスタンスの
/// 識別」節参照）。`name` は
/// [`fandhe_frontend_headless_ui::number_input::input`] の必須引数であり
/// 常に出力される。
fn instance_payload(input: &impl AttrSource) -> String {
    input.attr("name").unwrap_or_default()
}

/// `"set"`（Enter 確定・Arrow 前の同期）の dispatch アクション名を決定する。
///
/// Input パーツに [`ACTION_INPUT_ATTR`]（`data-action-input`）があれば
/// そのアプリ定義アクション名を使う（`crate::events` の input イベント
/// 配線と同じ属性契約の再利用、モジュール冒頭 doc 参照）。無ければ
/// 固定名 [`ACTION_SET`]（単一インスタンスモード、後方互換）。
fn set_action_name(input: &impl AttrSource) -> String {
    input
        .attr(ACTION_INPUT_ATTR)
        .unwrap_or_else(|| ACTION_SET.to_string())
}

/// [`action_for_key`] が決定した [`KeyAction`] から、実際に dispatch すべき
/// `ActionRef` 列を組み立てる純粋関数（web-sys 非依存、native
/// `cargo test` で検証可能。`crate::events::AttrSource` を介して Input
/// パーツの属性を読むだけで DOM には触れない）。
///
/// `input` は Input パーツの属性読み取り抽象（[`AttrSource`]）、
/// `raw_value` はキャレット確定前の `input.value`
/// （Increment/Decrement/Set が参照する、モジュール冒頭 doc「dispatch と
/// アクションの対応」節参照）。返り値は 1〜2 件（Increment/Decrement の
/// みキャレット確定前の値を同期する `"set"` を先に含む 2 件、それ以外は
/// 1 件）。
///
/// `raw_value` の前後空白は本関数内で trim してから空欄判定・`"set"`
/// payload の双方に使う（PR #1881 codex P1 / Bugbot Medium 是正）。
/// `NumberInput::decode_action` の `parse::<f64>()` は前後空白を含む
/// 文字列を受理しないため、配線側で trim してから同期しないと
/// 「空白付きの値を貼り付けて増減キーを押しても反映されない」不具合が
/// 生じる。
///
/// 複数インスタンスの識別方式はモジュール冒頭 doc「複数インスタンスの
/// 識別」節参照。
#[must_use]
pub fn resolve_dispatches(
    key_action: KeyAction,
    input: &impl AttrSource,
    raw_value: &str,
) -> Vec<ActionRef> {
    // PR #1881 codex P1 / Bugbot Medium 是正: `decode_action` の
    // `parse::<f64>()` は前後空白を含む文字列を拒否する（Rust の
    // `f64::from_str` 仕様）ため、`"set"` へ渡す payload は必ずここで
    // trim 済みの値を使う。空欄判定（下記 `KeyAction::Set` 分岐）と
    // 同じ trim 済み値を共有することで、判定に使った値と実際に
    // dispatch する値が食い違わないようにする（前後空白を含む値を
    // 貼り付けて ArrowUp/ArrowDown/Enter を押しても "set" の同期が
    // no-op にならず正しく反映される）。
    let trimmed_value = raw_value.trim();
    match key_action {
        // PR #1881 codex-review P1 是正その 1: 増減の直前にタイプ中の
        // `input.value` を `"set"`（または上書き名）として同期 dispatch
        // する。値がパース不能・非有限な場合は `decode_action` が no-op
        // として無視するため、増減は編集前の状態値のまま安全に行われる
        // （fail-closed、モジュール冒頭 doc 参照）。
        KeyAction::Increment => vec![
            ActionRef {
                action: set_action_name(input),
                payload: trimmed_value.to_string(),
            },
            ActionRef {
                action: ACTION_INCREMENT.to_string(),
                payload: instance_payload(input),
            },
        ],
        KeyAction::Decrement => vec![
            ActionRef {
                action: set_action_name(input),
                payload: trimmed_value.to_string(),
            },
            ActionRef {
                action: ACTION_DECREMENT.to_string(),
                payload: instance_payload(input),
            },
        ],
        KeyAction::Home => vec![ActionRef {
            action: ACTION_HOME.to_string(),
            payload: instance_payload(input),
        }],
        KeyAction::End => vec![ActionRef {
            action: ACTION_END.to_string(),
            payload: instance_payload(input),
        }],
        // PR #1881 codex-review P1 是正その 2: trim 後空文字は `"set"`
        // （`decode_action` が空文字列パース失敗で no-op にし旧値が残留
        // する）ではなく `"clear"` へ分岐し、未入力状態へ正しく同期する。
        KeyAction::Set => {
            if trimmed_value.is_empty() {
                vec![ActionRef {
                    action: ACTION_CLEAR.to_string(),
                    payload: instance_payload(input),
                }]
            } else {
                vec![ActionRef {
                    action: set_action_name(input),
                    payload: trimmed_value.to_string(),
                }]
            }
        }
    }
}

// ---------------------------------------------------------------------
// 配線層: web-sys/js-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`angle_slider.rs`/`headless_signature_pad.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        action_for_key, action_for_trigger_part, resolve_dispatches, DECREMENT_TRIGGER_PART,
        INCREMENT_TRIGGER_PART,
    };
    use crate::events::{ActionRef, AttrSource};
    use crate::keynav::Modifiers;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, HtmlInputElement, KeyboardEvent, Node};

    /// NumberInput の `data-scope` 属性値（`fandhe_frontend_headless_ui::number_input`
    /// の `ANATOMY` と一致、`crates/headless-ui/src/number_input.rs` 参照）。
    const NUMBER_INPUT_SCOPE: &str = "number-input";
    /// NumberInput Input パーツの `data-part` 属性値。
    const INPUT_PART: &str = "input";
    /// NumberInput Control パーツの `data-part` 属性値（click 配線が
    /// トリガーから Input を探す起点、[`find_input_within_control`] 参照）。
    const CONTROL_PART: &str = "control";
    /// NumberInput Root パーツの `data-part` 属性値（Control が見つからない
    /// 場合の Input 探索フォールバック起点）。
    const ROOT_PART: &str = "root";

    /// `web_sys::Element` を [`AttrSource`] に橋渡しする薄いラッパー
    /// （`events.rs::wiring::ElementAttrSource`/`overlay.rs` と同じ意図の
    /// 配線層専用アダプタ）。[`super::resolve_dispatches`] を web-sys の
    /// 具象型から独立させたまま呼び出すために使う。
    struct ElementAttrSource<'a>(&'a Element);

    impl AttrSource for ElementAttrSource<'_> {
        fn attr(&self, name: &str) -> Option<String> {
            self.0.get_attribute(name)
        }
    }

    /// `event` から [`Modifiers`] を抽出する（`crate::keynav::modifiers_of`
    /// と同型の判断だが `pub(crate)` ではないためここで個別定義する）。
    fn modifiers_of(event: &KeyboardEvent) -> Modifiers {
        Modifiers {
            ctrl: event.ctrl_key(),
            alt: event.alt_key(),
            meta: event.meta_key(),
        }
    }

    /// `start` から `root` まで祖先方向を辿り、`data-disabled` **または**
    /// `data-readonly` を持つ要素が 1 つでもあれば `true` を返す
    /// （`crate::angle_slider::wiring::has_noninteractive_ancestor` と同型の
    /// fail-closed 判定。`root`/`control`/`input` はいずれも disabled/
    /// readonly 時に該当 data-* 属性を持つ、`crates/headless-ui/src/
    /// number_input.rs` の `root`/`control`/`input` 参照）。
    fn has_noninteractive_ancestor(root: &Element, start: &Element) -> bool {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if element.has_attribute("data-disabled") || element.has_attribute("data-readonly") {
                return true;
            }
            if !root.contains(Some(&element)) || element == *root {
                break;
            }
            current = element.parent_element();
        }
        false
    }

    /// `root` 配下の NumberInput Input パーツへ keydown 配線を、
    /// IncrementTrigger/DecrementTrigger パーツへ click 配線を、それぞれ
    /// 1 回だけ登録する（マウント時 1 回契約、`angle_slider.rs`/
    /// `splitter.rs` と同型。イシュー #1962 で click リスナーを追加）。
    ///
    /// `on_action` は `"increment"`/`"decrement"`/`"home"`/`"end"`/`"set"` の
    /// dispatch 依頼を呼び出し側へ渡すのみで、状態更新・DOM 反映は行わない
    /// （`headless_clipboard::wire_clipboard_events` と同じ責務分離）。
    /// keydown・click 双方のリスナーが同じ `on_action` を共有する
    /// （`Rc<RefCell<_>>` で clone、`Closure::forget` は本関数呼び出し
    /// につき keydown 1 回・click 1 回の計 2 回のみに限定する）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback`（keydown・click いずれか）の
    /// 失敗を伝播する。
    pub fn wire_number_input_events(
        root: Element,
        on_action: impl FnMut(ActionRef) + 'static,
    ) -> Result<(), JsValue> {
        let on_action = std::rc::Rc::new(std::cell::RefCell::new(on_action));

        let keydown_root = root.clone();
        let keydown_on_action = on_action.clone();
        let keydown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_keydown(&keydown_root, &event, &keydown_on_action);
        });
        root.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())?;
        keydown_closure.forget();

        let click_root = root.clone();
        let click_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_click(&click_root, &event, &on_action);
        });
        root.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())?;
        click_closure.forget();

        Ok(())
    }

    /// keydown: Input（`data-scope="number-input"` `data-part="input"`）上
    /// でのみ反応する（[`action_for_key`]、モジュール冒頭 doc 「dispatch と
    /// アクションの対応」節参照）。
    fn handle_keydown(
        root: &Element,
        event: &Event,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        let Some(keyboard_event) = event.dyn_ref::<KeyboardEvent>() else {
            return;
        };
        let Some(target) = event.target() else {
            return;
        };
        let Some(target_element) = target.dyn_ref::<Element>() else {
            return;
        };
        if !root.contains(Some(target_element)) {
            return;
        }

        // PR #1881 codex-review P1 是正その 3: IME 変換中（`isComposing`）の
        // keydown は候補選択の ArrowUp/ArrowDown・変換確定の Enter を意味し、
        // NumberInput の増減/確定操作ではない。ここで除外しないと、変換中
        // 文字列が `decode_action` のパースに失敗しても increment/decrement
        // 自体は実行され、変換中に利用者が意図しない数値へ上書きされる
        // （`on_action` を一切呼ばず、`prevent_default()` も行わない。
        // ブラウザの IME 標準動作をそのまま素通しする）。
        // `key_code() == 229` は IME 変換中に一部ブラウザが送る互換シグナル
        // （`is_composing()` が未実装/false でも 229 は立つ実装がある）で
        // あり、両方を fail-closed に見て除外する。
        if keyboard_event.is_composing() || keyboard_event.key_code() == 229 {
            return;
        }

        let scope = target_element.get_attribute("data-scope");
        let part = target_element.get_attribute("data-part");
        if scope.as_deref() != Some(NUMBER_INPUT_SCOPE) || part.as_deref() != Some(INPUT_PART) {
            return;
        }
        if has_noninteractive_ancestor(root, target_element) {
            return;
        }

        let modifiers = modifiers_of(keyboard_event);
        let Some(key_action) = action_for_key(&keyboard_event.key(), modifiers) else {
            return;
        };

        // `input.value` はキャレット確定前のタイプ中文字列であり、
        // Increment/Decrement/Set のいずれも参照し得るため一度だけ読む
        // （モジュール冒頭 doc「dispatch とアクションの対応」節参照）。
        let raw_value = target_element
            .clone()
            .dyn_into::<HtmlInputElement>()
            .map(|input| input.value())
            .unwrap_or_default();

        keyboard_event.prevent_default();

        // dispatch すべき `ActionRef` 列の決定は純粋関数
        // `super::resolve_dispatches` に委ねる（PR #1881 codex-review P1
        // 是正: 複数インスタンス識別のためのアクション名上書き
        // （`data-action-input`）・payload への `name` 属性値埋め込みは
        // モジュール冒頭 doc「複数インスタンスの識別」節参照）。ここでは
        // Element を `AttrSource` へ橋渡ししてから呼び出すだけに留める。
        let source = ElementAttrSource(target_element);
        for action_ref in resolve_dispatches(key_action, &source, &raw_value) {
            if let Ok(mut cb) = on_action.try_borrow_mut() {
                (cb)(action_ref);
            }
        }
    }

    /// `start` から `root` まで祖先方向を辿り、`data-scope="number-input"`
    /// かつ `data-part` が [`INCREMENT_TRIGGER_PART`]/[`DECREMENT_TRIGGER_PART`]
    /// の最寄り要素（トリガー要素）を返す（内側優先。イシュー #1962）。
    /// 見つからなければ `None`（fail-closed。トリガー以外の click は
    /// 一切 dispatch しない）。
    fn find_trigger_ancestor(root: &Element, start: &Element) -> Option<Element> {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if element.get_attribute("data-scope").as_deref() == Some(NUMBER_INPUT_SCOPE) {
                let part = element.get_attribute("data-part");
                if part.as_deref() == Some(INCREMENT_TRIGGER_PART)
                    || part.as_deref() == Some(DECREMENT_TRIGGER_PART)
                {
                    return Some(element);
                }
            }
            if !root.contains(Some(&element)) || element == *root {
                break;
            }
            current = element.parent_element();
        }
        None
    }

    /// `trigger` から祖先方向に最寄りの Control（無ければ Root）パーツを
    /// 探し、その配下の Input パーツ（`data-scope="number-input"`
    /// `data-part="input"`）を 1 件返す（イシュー #1962）。
    ///
    /// `aria-controls` 経由の `get_element_by_id` ではなく祖先 + 部分木
    /// 探索を主経路とする（`aria-controls` の対象 id は
    /// [`fandhe_frontend_headless_ui::number_input::input`] 呼び出し時の
    /// 任意引数であり欠落しうるため）。Control/Root いずれも見つからない、
    /// または配下に Input が無い場合は `None`（fail-closed）。
    fn find_input_within_control(root: &Element, trigger: &Element) -> Option<Element> {
        // Control/Root の探索範囲を最寄りの NumberInput Root 配下へ限定する
        // （イシュー #1962 codex-review P1 是正）。`root`（配線登録時の
        // ルート）をそのまま境界に使うと、Control を省略した NumberInput が
        // 別の NumberInput にネストされている場合に外側インスタンスの
        // Root/Control まで祖先探索が及び、内側トリガーの操作が外側
        // インスタンスの Input を誤って更新してしまう（PR #1982
        // codex-review 指摘）。最寄り Root が見つからない場合は
        // fail-closed（`None`）。
        let nearest_root = find_nearest_root(root, trigger)?;

        let mut current = Some(trigger.clone());
        let mut container: Option<Element> = None;
        while let Some(element) = current {
            if element.get_attribute("data-scope").as_deref() == Some(NUMBER_INPUT_SCOPE) {
                let part = element.get_attribute("data-part");
                if part.as_deref() == Some(CONTROL_PART) {
                    container = Some(element);
                    break;
                }
                if part.as_deref() == Some(ROOT_PART) {
                    container = Some(element.clone());
                    // Control が Root より内側に無い構成もあるため、
                    // 最寄り Root に到達しても即座に確定させず、より内側の
                    // Control を優先して探し続ける（見つからなければ Root を
                    // 使う）。ただし探索範囲は `nearest_root` で打ち切られる
                    // （下記境界判定）ため、他インスタンスの Root/Control へ
                    // は及ばない。
                }
            }
            if !nearest_root.contains(Some(&element)) || element == nearest_root {
                break;
            }
            current = element.parent_element();
        }
        let container = container?;
        // `container` が Control を省略した外側 Root そのものの場合、その
        // 部分木には別インスタンス（内側 NumberInput）がネストされ得る。
        // `query_selector`（単一マッチ）は部分木内の最初の一致だけを返し
        // 内側/外側の区別をしないため、`query_selector_all` で候補を
        // すべて列挙し、各候補の最寄り Root が `trigger` の
        // `nearest_root` と一致する最初の 1 件を選ぶ（PR #1982
        // codex-review P1 是正その 2: 先頭候補を検証して不一致なら
        // 即 `None` で打ち切る実装は、内側 Root/Input・外側 Input・外側
        // Trigger の順に配置されると `query_selector_all` の先頭が内側
        // Input になり、正しい外側 Input が存在しても探索が終了して
        // 外側トリガーが常に無反応になっていた。候補を最後まで走査し、
        // 一致するものが 1 件も無い場合だけ fail-closed で `None` を
        // 返す）。
        let candidates = container
            .query_selector_all(r#"[data-scope="number-input"][data-part="input"]"#)
            .ok()?;
        for index in 0..candidates.length() {
            let Some(node) = candidates.item(index) else {
                continue;
            };
            let Some(candidate) = node.dyn_ref::<Element>().cloned() else {
                continue;
            };
            if find_nearest_root(root, &candidate).as_ref() == Some(&nearest_root) {
                return Some(candidate);
            }
        }
        None
    }

    /// `start` から `root` まで祖先方向を辿り、最寄りの NumberInput Root
    /// （`data-scope="number-input"` `data-part="root"`）を返す（イシュー
    /// #1962 codex-review P1 是正）。見つからなければ `None`。
    ///
    /// [`find_input_within_control`] の探索範囲をこの最寄り Root 配下へ
    /// 限定するために使う。Control を省略した NumberInput が別の
    /// NumberInput にネストされている場合、祖先探索を最寄り Root で
    /// 打ち切らないと外側インスタンスの Root/Control まで遡ってしまい、
    /// 内側トリガーの操作が外側インスタンスの Input を誤って更新する
    /// （PR #1982 codex-review 指摘）。
    fn find_nearest_root(root: &Element, start: &Element) -> Option<Element> {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if element.get_attribute("data-scope").as_deref() == Some(NUMBER_INPUT_SCOPE)
                && element.get_attribute("data-part").as_deref() == Some(ROOT_PART)
            {
                return Some(element);
            }
            if !root.contains(Some(&element)) || element == *root {
                break;
            }
            current = element.parent_element();
        }
        None
    }

    /// click: IncrementTrigger/DecrementTrigger（`data-scope="number-input"`
    /// `data-part="increment-trigger"`/`"decrement-trigger"`）上でのみ反応
    /// する（[`action_for_trigger_part`]、モジュール冒頭 doc「dispatch と
    /// アクションの対応」節参照。イシュー #1962、親 #1961）。
    ///
    /// `event.target()` がテキストノード（ボタン内テキスト・SVG アイコン
    /// 等）の場合は [`Node::parent_element`] で直近の親要素まで遡ってから
    /// 祖先探索を始める（`crate::headless::wiring::wire_headless_events` と
    /// 同じ対策）。
    fn handle_click(
        root: &Element,
        event: &Event,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        let Some(target) = event.target() else {
            return;
        };
        let target_element: Element = match target.dyn_ref::<Element>() {
            Some(element) => element.clone(),
            None => {
                let Some(node) = target.dyn_ref::<Node>() else {
                    return;
                };
                let Some(parent) = node.parent_element() else {
                    return;
                };
                parent
            }
        };
        if !root.contains(Some(&target_element)) {
            return;
        }

        let Some(trigger) = find_trigger_ancestor(root, &target_element) else {
            return;
        };
        // ネイティブ `disabled` ボタン（headless-ui がクランプ到達時に
        // 出力）はブラウザが click 自体を発火させないため、本判定は
        // 多層防御の位置づけ（モジュール冒頭 doc「セキュリティ不変条件」
        // 節参照）。
        if has_noninteractive_ancestor(root, &trigger) {
            return;
        }
        let Some(part) = trigger.get_attribute("data-part") else {
            return;
        };
        let Some(key_action) = action_for_trigger_part(&part) else {
            return;
        };

        let Some(input_element) = find_input_within_control(root, &trigger) else {
            return;
        };
        // Trigger 自体・その祖先が非対話状態（`data-disabled`/
        // `data-readonly`）でなくても、公開 API で Input のみに
        // readonly/disabled を指定した構成では Input 側で非対話が成立する
        // （イシュー #1962 codex-review P1 是正）。keydown 経路と同様に
        // Input 解決後も改めて確認し、成立していれば dispatch しない
        // （fail-closed。モジュール冒頭 doc「セキュリティ不変条件」節参照）。
        if has_noninteractive_ancestor(root, &input_element) {
            return;
        }
        let raw_value = input_element
            .clone()
            .dyn_into::<HtmlInputElement>()
            .map(|input| input.value())
            .unwrap_or_default();

        // 解決できた場合のみ伝播を止める（`crate::headless::wiring::
        // wire_headless_events` と同じ根拠: 入れ子・複数の配線済み root で
        // 同一 click イベントが二重解決されるのを防ぐ）。`<button
        // type="button">` に既定動作はないため `prevent_default()` は
        // 呼ばない。
        event.stop_propagation();

        let source = ElementAttrSource(&input_element);
        for action_ref in resolve_dispatches(key_action, &source, &raw_value) {
            if let Ok(mut cb) = on_action.try_borrow_mut() {
                (cb)(action_ref);
            }
        }
    }

    /// [`wire_number_input_events`] の keydown dispatch を
    /// `fandhe_frontend_interactive::dispatch` へ接続し、成功時のみ
    /// `on_update` を呼ぶ利便関数（`crate::headless::wire_headless_component`/
    /// `headless_signature_pad::wire_signature_pad_component` と同型）。
    ///
    /// `on_update` は呼び出し側（`crate::lib::Runtime::wire_number_input`）が
    /// 束縛点更新（`BindingTable::apply_dirty`・keyed list 差し替え）を渡す
    /// 想定であり、本関数自体は DOM 反映を行わない。
    ///
    /// # Errors
    ///
    /// [`wire_number_input_events`]（`add_event_listener_with_callback`）の
    /// 失敗を伝播する。
    pub fn wire_number_input_component<C>(
        root: Element,
        component: std::rc::Rc<std::cell::RefCell<C>>,
        on_update: impl FnMut(&C, &Element) + 'static,
    ) -> Result<(), JsValue>
    where
        C: fandhe_frontend_interactive::Component + 'static,
    {
        let on_update = std::rc::Rc::new(std::cell::RefCell::new(on_update));
        let wired_root = root.clone();

        wire_number_input_events(root, move |action_ref: ActionRef| {
            let Ok(mut state) = component.try_borrow_mut() else {
                return;
            };
            let dispatched = fandhe_frontend_interactive::dispatch(
                &mut *state,
                &action_ref.action,
                &action_ref.payload,
            );
            if !dispatched {
                return;
            }
            if let Ok(mut cb) = on_update.try_borrow_mut() {
                (cb)(&state, &wired_root);
            }
        })
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{wire_number_input_component, wire_number_input_events};

#[cfg(test)]
mod tests {
    use super::*;

    fn mods() -> Modifiers {
        Modifiers::default()
    }

    fn mods_ctrl() -> Modifiers {
        Modifiers {
            ctrl: true,
            alt: false,
            meta: false,
        }
    }

    // --- action_for_key ---

    #[test]
    fn arrow_up_is_increment() {
        assert_eq!(
            action_for_key("ArrowUp", mods()),
            Some(KeyAction::Increment)
        );
    }

    #[test]
    fn arrow_down_is_decrement() {
        assert_eq!(
            action_for_key("ArrowDown", mods()),
            Some(KeyAction::Decrement)
        );
    }

    #[test]
    fn home_sets_to_min() {
        assert_eq!(action_for_key("Home", mods()), Some(KeyAction::Home));
    }

    #[test]
    fn end_sets_to_max() {
        assert_eq!(action_for_key("End", mods()), Some(KeyAction::End));
    }

    #[test]
    fn enter_commits_typed_value() {
        assert_eq!(action_for_key("Enter", mods()), Some(KeyAction::Set));
    }

    #[test]
    fn unknown_key_is_noop() {
        assert_eq!(action_for_key("a", mods()), None);
        assert_eq!(action_for_key("PageUp", mods()), None);
        assert_eq!(action_for_key("Tab", mods()), None);
        assert_eq!(action_for_key("ArrowLeft", mods()), None);
        assert_eq!(action_for_key("ArrowRight", mods()), None);
    }

    #[test]
    fn modifier_keys_are_noop() {
        assert_eq!(action_for_key("ArrowUp", mods_ctrl()), None);
        assert_eq!(action_for_key("Enter", mods_ctrl()), None);
        assert_eq!(
            action_for_key(
                "Home",
                Modifiers {
                    ctrl: false,
                    alt: true,
                    meta: false
                }
            ),
            None
        );
        assert_eq!(
            action_for_key(
                "End",
                Modifiers {
                    ctrl: false,
                    alt: false,
                    meta: true
                }
            ),
            None
        );
    }

    // --- action_for_trigger_part（イシュー #1962）---

    #[test]
    fn increment_trigger_part_is_increment() {
        assert_eq!(
            action_for_trigger_part(INCREMENT_TRIGGER_PART),
            Some(KeyAction::Increment)
        );
    }

    #[test]
    fn decrement_trigger_part_is_decrement() {
        assert_eq!(
            action_for_trigger_part(DECREMENT_TRIGGER_PART),
            Some(KeyAction::Decrement)
        );
    }

    #[test]
    fn non_trigger_parts_are_noop() {
        assert_eq!(action_for_trigger_part("input"), None);
        assert_eq!(action_for_trigger_part("root"), None);
        assert_eq!(action_for_trigger_part("control"), None);
        assert_eq!(action_for_trigger_part("label"), None);
        assert_eq!(action_for_trigger_part("value-text"), None);
        assert_eq!(action_for_trigger_part(""), None);
        assert_eq!(action_for_trigger_part("unknown-part"), None);
    }

    /// click 経路（[`action_for_trigger_part`]）が決定する [`KeyAction`] を
    /// [`resolve_dispatches`] へ渡した結果が、keydown 経路
    /// （`action_for_key("ArrowUp"/"ArrowDown", ...)`）と同一の dispatch 列
    /// （`"set"` 同期 + `name` payload 付き `"increment"`/`"decrement"`）に
    /// なることの回帰テスト（`resolve_dispatches` を両経路が共有する契約の
    /// 固定）。
    #[test]
    fn trigger_click_dispatches_match_keydown_dispatches() {
        let input = input_with(&[("name", "qty")]);

        let click_dispatches = resolve_dispatches(
            action_for_trigger_part(INCREMENT_TRIGGER_PART).unwrap(),
            &input,
            "5",
        );
        let keydown_dispatches =
            resolve_dispatches(action_for_key("ArrowUp", mods()).unwrap(), &input, "5");
        assert_eq!(click_dispatches, keydown_dispatches);

        let click_dispatches = resolve_dispatches(
            action_for_trigger_part(DECREMENT_TRIGGER_PART).unwrap(),
            &input,
            "5",
        );
        let keydown_dispatches =
            resolve_dispatches(action_for_key("ArrowDown", mods()).unwrap(), &input, "5");
        assert_eq!(click_dispatches, keydown_dispatches);
    }

    /// `data-action-input` 上書きが click 経路の同期 `"set"` にも効くこと
    /// （keydown 経路と同じ `resolve_dispatches` を共有するため当然成立
    /// するが、click 配線が別関数を経由しないことの回帰確認として固定
    /// する）。
    #[test]
    fn trigger_click_respects_data_action_input_override() {
        let input = input_with(&[("name", "price"), ("data-action-input", "price_set")]);

        assert_eq!(
            resolve_dispatches(
                action_for_trigger_part(INCREMENT_TRIGGER_PART).unwrap(),
                &input,
                "9.5"
            )[0],
            ActionRef {
                action: "price_set".to_string(),
                payload: "9.5".to_string(),
            }
        );
    }

    // --- KeyAction::action_name ---

    #[test]
    fn action_name_matches_decode_action_contract() {
        assert_eq!(KeyAction::Increment.action_name(), "increment");
        assert_eq!(KeyAction::Decrement.action_name(), "decrement");
        assert_eq!(KeyAction::Home.action_name(), "home");
        assert_eq!(KeyAction::End.action_name(), "end");
        assert_eq!(KeyAction::Set.action_name(), "set");
    }

    // --- ACTION_CLEAR（PR #1881 codex-review P1 是正その 2） ---

    /// [`ACTION_CLEAR`] は
    /// [`fandhe_frontend_headless_ui::number_input::NumberInput::decode_action`]
    /// が受理する `"clear"` と完全一致すること（配線層は文字列リテラルを
    /// 個別に書かず本定数のみを参照する契約の固定）。
    #[test]
    fn action_clear_matches_decode_action_contract() {
        assert_eq!(ACTION_CLEAR, "clear");
    }

    // --- resolve_dispatches（PR #1881 codex-review P1 是正: 複数
    // インスタンス識別）---

    /// native `cargo test` 用のテストダブル（`overlay.rs`/`events.rs` の
    /// `FakeElement` と同型）。
    struct FakeInput {
        attrs: std::collections::HashMap<&'static str, &'static str>,
    }

    impl AttrSource for FakeInput {
        fn attr(&self, name: &str) -> Option<String> {
            self.attrs.get(name).map(|v| v.to_string())
        }
    }

    fn input_with(attrs: &[(&'static str, &'static str)]) -> FakeInput {
        FakeInput {
            attrs: attrs.iter().copied().collect(),
        }
    }

    /// `data-action-input` も `name` も無い最小構成（単一インスタンス
    /// モード）: 固定アクション名のまま、増減/Home/End の payload は空文字
    /// （後方互換、`NumberInput::decode_action` が payload を無視する契約と
    /// 整合）。
    #[test]
    fn resolve_dispatches_single_instance_mode_uses_fixed_names_and_empty_payload() {
        let input = input_with(&[]);

        assert_eq!(
            resolve_dispatches(KeyAction::Increment, &input, "5"),
            vec![
                ActionRef {
                    action: "set".to_string(),
                    payload: "5".to_string(),
                },
                ActionRef {
                    action: "increment".to_string(),
                    payload: String::new(),
                },
            ]
        );
        assert_eq!(
            resolve_dispatches(KeyAction::Home, &input, ""),
            vec![ActionRef {
                action: "home".to_string(),
                payload: String::new(),
            }]
        );
        assert_eq!(
            resolve_dispatches(KeyAction::Set, &input, "5"),
            vec![ActionRef {
                action: "set".to_string(),
                payload: "5".to_string(),
            }]
        );
        assert_eq!(
            resolve_dispatches(KeyAction::Set, &input, "   "),
            vec![ActionRef {
                action: "clear".to_string(),
                payload: String::new(),
            }]
        );
    }

    /// `name` 属性がある（`data-action-input` は無い）構成: 固定アクション名
    /// は変わらないが、増減/Home/End/Clear の payload に `name` が載る
    /// （[`crate::splitter`] の trigger index と同型の識別方式）。
    #[test]
    fn resolve_dispatches_increment_decrement_home_end_clear_carry_name_in_payload() {
        let qty = input_with(&[("name", "qty")]);
        let price = input_with(&[("name", "price")]);

        assert_eq!(
            resolve_dispatches(KeyAction::Increment, &qty, "5")[1],
            ActionRef {
                action: "increment".to_string(),
                payload: "qty".to_string(),
            }
        );
        assert_eq!(
            resolve_dispatches(KeyAction::Increment, &price, "5")[1],
            ActionRef {
                action: "increment".to_string(),
                payload: "price".to_string(),
            }
        );
        assert_eq!(
            resolve_dispatches(KeyAction::Decrement, &price, "5")[1],
            ActionRef {
                action: "decrement".to_string(),
                payload: "price".to_string(),
            }
        );
        assert_eq!(
            resolve_dispatches(KeyAction::Home, &price, "")[0],
            ActionRef {
                action: "home".to_string(),
                payload: "price".to_string(),
            }
        );
        assert_eq!(
            resolve_dispatches(KeyAction::End, &price, "")[0],
            ActionRef {
                action: "end".to_string(),
                payload: "price".to_string(),
            }
        );
        assert_eq!(
            resolve_dispatches(KeyAction::Set, &price, "  ")[0],
            ActionRef {
                action: "clear".to_string(),
                payload: "price".to_string(),
            }
        );
    }

    /// `data-action-input` がある構成: `"set"`（Enter 確定・Arrow 前の
    /// 同期）のアクション名がその値へ上書きされる。payload は
    /// `input.value` のまま変わらない（`crate::events::ACTION_INPUT_ATTR`
    /// 契約の再利用）。
    #[test]
    fn resolve_dispatches_set_action_name_overridden_by_data_action_input() {
        let price = input_with(&[("name", "price"), ("data-action-input", "price_set")]);

        assert_eq!(
            resolve_dispatches(KeyAction::Set, &price, "9.5")[0],
            ActionRef {
                action: "price_set".to_string(),
                payload: "9.5".to_string(),
            }
        );
        // Increment の同期 "set" も上書き名を使う。
        assert_eq!(
            resolve_dispatches(KeyAction::Increment, &price, "9.5")[0],
            ActionRef {
                action: "price_set".to_string(),
                payload: "9.5".to_string(),
            }
        );
        // increment 自体のアクション名は変わらない（payload のみ name）。
        assert_eq!(
            resolve_dispatches(KeyAction::Increment, &price, "9.5")[1],
            ActionRef {
                action: "increment".to_string(),
                payload: "price".to_string(),
            }
        );
    }

    /// `data-action-input` があっても Enter 確定時の trim 後空文字は
    /// `"clear"`（固定名 + `name` payload）へ分岐し、`"set"` 上書き名は
    /// 使われない（PR #1881 codex-review P1 是正その 2 との整合。空欄
    /// 確定を上書きアクション名の `"set"` へ回すと `decode_action` 側で
    /// クリア専用の分岐が必要になり複数インスタンス契約が破綻するため）。
    #[test]
    fn resolve_dispatches_clear_ignores_data_action_input_override() {
        let price = input_with(&[("name", "price"), ("data-action-input", "price_set")]);

        assert_eq!(
            resolve_dispatches(KeyAction::Set, &price, ""),
            vec![ActionRef {
                action: "clear".to_string(),
                payload: "price".to_string(),
            }]
        );
    }

    /// 2 インスタンス（qty/price）で dispatch 列が完全に異なることを固定する
    /// 回帰テスト（PR #1881 codex-review P1 「片方の ArrowUp が同じ
    /// `(action, payload)` になり区別できない」の是正確認）。
    #[test]
    fn resolve_dispatches_distinguishes_two_instances_end_to_end() {
        let qty = input_with(&[("name", "qty"), ("data-action-input", "qty_set")]);
        let price = input_with(&[("name", "price"), ("data-action-input", "price_set")]);

        let qty_dispatches = resolve_dispatches(KeyAction::Increment, &qty, "5");
        let price_dispatches = resolve_dispatches(KeyAction::Increment, &price, "5");

        assert_ne!(qty_dispatches, price_dispatches);
        assert_eq!(
            qty_dispatches,
            vec![
                ActionRef {
                    action: "qty_set".to_string(),
                    payload: "5".to_string(),
                },
                ActionRef {
                    action: "increment".to_string(),
                    payload: "qty".to_string(),
                },
            ]
        );
        assert_eq!(
            price_dispatches,
            vec![
                ActionRef {
                    action: "price_set".to_string(),
                    payload: "5".to_string(),
                },
                ActionRef {
                    action: "increment".to_string(),
                    payload: "price".to_string(),
                },
            ]
        );
    }

    // --- resolve_dispatches（PR #1881 codex P1 / Bugbot Medium 是正:
    // "set" payload の前後空白除去）---

    /// 前後空白付きの値（`" 8"`）を Increment/Decrement/Set いずれで同期
    /// しても、`"set"` payload は trim 済み文字列（`"8"`）になること。
    /// trim せず `input.value` をそのまま渡すと
    /// `NumberInput::decode_action` の `parse::<f64>()` が前後空白を
    /// 拒否し no-op になる不具合の回帰テスト。
    #[test]
    fn resolve_dispatches_trims_leading_and_trailing_whitespace_for_set_payload() {
        let input = input_with(&[]);

        assert_eq!(
            resolve_dispatches(KeyAction::Increment, &input, " 8")[0],
            ActionRef {
                action: "set".to_string(),
                payload: "8".to_string(),
            }
        );
        assert_eq!(
            resolve_dispatches(KeyAction::Decrement, &input, " 8")[0],
            ActionRef {
                action: "set".to_string(),
                payload: "8".to_string(),
            }
        );
        assert_eq!(
            resolve_dispatches(KeyAction::Set, &input, " 8 ")[0],
            ActionRef {
                action: "set".to_string(),
                payload: "8".to_string(),
            }
        );
    }

    /// 空白のみの値（`"  "`）は trim 後空文字として扱われ、Increment/
    /// Decrement の同期 `"set"` payload も空文字になる（既存の
    /// `KeyAction::Set` 分岐の空欄判定〔`"clear"` へ分岐〕と同じ trim
    /// 済み値を共有していることの確認）。
    #[test]
    fn resolve_dispatches_whitespace_only_value_trims_to_empty_set_payload() {
        let input = input_with(&[]);

        assert_eq!(
            resolve_dispatches(KeyAction::Increment, &input, "  ")[0],
            ActionRef {
                action: "set".to_string(),
                payload: String::new(),
            }
        );
        assert_eq!(
            resolve_dispatches(KeyAction::Set, &input, "  "),
            vec![ActionRef {
                action: "clear".to_string(),
                payload: String::new(),
            }]
        );
    }
}
