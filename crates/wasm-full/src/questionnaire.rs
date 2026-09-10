//! Questionnaire（`fandhe-frontend-headless-ui` `questionnaire` モジュール）の
//! back / next / skip クリックから `data-state` 更新への配線
//! （イシュー #2118、親 #2117、祖父 #2116）。
//!
//! `crates/headless-ui/src/questionnaire.rs` は Root/Progress/Question/
//! Prompt/Description/Options/Freeform/Actions/Back/Next/Skip の 11 anatomy
//! パーツと、`count`/`step` から質問の 3 状態（`active`/`completed`/
//! `upcoming`）を導出する決定的状態機械
//! [`fandhe_frontend_headless_ui::questionnaire::Questionnaire`] を提供する
//! 一方、trigger click から dispatch への実配線は同モジュール冒頭 rustdoc
//! 「out-of-scope」節が明記するとおり本クレート（wasm 層）の後続スコープ
//! （#2118）とされていた。本モジュールがその配線を実装する。
//!
//! # 責務境界（`.claude/rules/coding-rust.md` §UI 部品の責務境界、規則 1）
//!
//! 回答値の保持・検証（必須判定）・送信・分岐（次にどの質問へ進むか）は
//! アプリケーション責務であり、本モジュールは持たない。`data-answered`/
//! `data-skipped`/`data-required`/`data-invalid`/`aria-invalid` は一切
//! 触らない。本モジュールが担うのは「アプリが指定した対象 question」の
//! 表示状態（`data-state`/`hidden`）・root/progress の位置表示・trigger の
//! 境界 `disabled` を切り替えるまでである。
//!
//! # DOM を Questionnaire の一時的な真として扱う（`headless_timer` 方式）
//!
//! click 対象から祖先方向へ辿って最寄りの
//! `[data-scope="questionnaire"][data-part="root"]`（インスタンス root）を
//! 解決し、その `data-step`/`data-orientation` と、そのインスタンスに属する
//! `question` 要素数（`count`）から
//! [`fandhe_frontend_headless_ui::questionnaire::Questionnaire::new`] を
//! 都度再構築 →
//! [`fandhe_frontend_interactive::dispatch`]（`"prev"`/`"next"`/`"skip"`）→
//! 変化があれば DOM へ書き戻す（[`write_questionnaire`]）。`Timer` と異なり
//! **リスナー登録先（`root`）は任意の祖先でよい**（インスタンスは click
//! 位置から解決するため）。[`crate::headless_timer`] が「`root` 自身が
//! Timer root でなければ no-op」という制約を持つのと対照的であり、
//! `Runtime::mount`（`set_inner_html` で子として流し込む構成）と
//! `Runtime::hydrate` の双方で機能する。
//!
//! 同一 `root` 配下の複数インスタンス・入れ子インスタンスは「click 位置
//! から最寄りの questionnaire root」で分離する（[`closest_matching`]）。
//! `question`/`progress`/`back`/`next`/`skip` の集計も「その要素の最寄り
//! questionnaire root がこのインスタンス root であるもの」に限定する
//! （[`belongs_to_instance`]）。
//!
//! # `headless::MAPPING_TABLE` へ登録しない理由
//!
//! `"next"`/`"prev"` は carousel / steps / pagination / tour / toolbar /
//! menubar / date-input / pin-input の各 `decode_action` と共有される語彙
//! であり、`(questionnaire, next) → "next"` の行を足すと
//! `wire_headless_component` 利用アプリで同一 click が二重解決・誤 dispatch
//! される（`sidebar` が `"toggle"` 共有を理由にオプトイン API へ倒した
//! 判断、および `headless_timer` が独自配線を持つ判断と同型）。代わりに
//! 本モジュール専用の click 委譲を `root` へ 1 個登録する
//! （`docs/design/wasm-full-architecture.md` §27 参照）。
//!
//! # アプリ状態 `C` への通知（`questionnaire:*`、`headless_timer` の
//! `timer:*` 先例と同型）
//!
//! 状態遷移が実際に起きた（before ≠ after）場合のみ、`on_action` 経由で
//! `C` へ [`ACTION_PREV`]/[`ACTION_NEXT`]/[`ACTION_SKIP`] を通知する。
//! payload は [`encode_notification_payload`]（`"{遷移前の step}|{instance
//! root の id 属性値（未設定時は空文字列）}"`）でエンコードし、アプリは
//! [`decode_notification_payload`] で分割する。`step` を先頭に置くのは
//! `step` が区切り文字 `|` を含み得ない `usize` の 10 進文字列であるのに
//! 対し `instance_id` はアプリが任意の文字列を設定できるため（イシュー
//! #2118 PR #2286 codex-review P1 指摘: 複数インスタンス時に通知だけでは
//! どのインスタンスの遷移か判別できなかった）。headless-ui は「どの質問を
//! スキップしたか」を保持しない設計のため、アプリはこの通知で
//! `QuestionProps::skipped`/`answered` を自身の状態へ記録できる。境界での
//! no-op click（例: 完了状態で next）は DOM も書かず通知もしない（アプリが
//! 「起きていないスキップ」を記録しないための fail-closed）。
//!
//! # 書き戻し対象と規則
//!
//! - インスタンス root: `data-step`（after 由来）、`data-complete`（存在
//!   属性、after 由来）。常に after から導出する。
//! - 各 `question`（`data-index` を `usize` パース）: `data-state`・
//!   `hidden`（非 active のみ）。常に after から導出する。パース不能な
//!   要素はその要素だけスキップする。
//! - `progress`: `aria-valuenow`/`aria-valuetext`/`data-complete`。
//!   [`crate::headless_timer::wiring::sync_area_aria_label`] と同型の
//!   before/after 比較を行い、現在の `aria-valuenow` が before 由来の値と
//!   一致する要素のみ更新する（利用者の独自値を壊さない fail-closed）。
//! - `back`: `disabled`/`data-disabled`。**境界条件（`step == 0`）が
//!   before/after で変化したときのみ**付与/除去する。
//! - `next`/`skip`: `disabled`/`data-disabled`。**境界条件
//!   （`step == count`）が before/after で変化したときのみ**付与/除去する。
//!
//! trigger の disabled をエッジ変化時にのみ触る理由: SSR は `step == 0` の
//! back に `disabled` を焼き込むため、wasm-full が再活性化しないと最初の
//! next 以降 back が永久に押せない。一方 `TRIGGER_RESERVED`
//! （`crates/headless-ui/src/questionnaire.rs`）により DOM 上ではアプリ
//! 由来（必須判定）の `disabled` と境界由来の `disabled` を区別できない。
//! 境界エッジ以外では一切触らないことで、非境界位置でアプリが付けた
//! `disabled` は保存される。**保存できない既知の限界**: 完了状態から back
//! で最終質問へ戻ったとき、next/skip の `disabled` は除去される（完了
//! 状態では必須判定が意味を持たないため）。アプリの必須判定は
//! `questionnaire:*` 通知後にアプリ自身の再描画で再適用する契約とする。
//!
//! # 分岐（どの質問へ進むか）についての立場
//!
//! 既定の遷移は headless-ui の状態機械どおり線形（`next`/`skip` は
//! `min(step+1, count)`、`prev` は `saturating_sub(1)`）。分岐が必要な
//! アプリは `questionnaire:*` 通知を受けて自身の状態で遷移先を決め、通常の
//! 再描画（dirty field → 束縛点更新 / 構造フォールバック）で対象 question
//! を active にする契約とする。`"goto"`（任意 step への直接移動）の DOM
//! 配線は本イシューのスコープ外（trigger パーツが無い）。
//!
//! # `Runtime` への統合
//!
//! [`wire_questionnaire_events`] は `crate::Runtime::mount`/
//! `Runtime::hydrate` の双方から `Self::wire_chart` の直後に組み込まれる
//! （`crate::Runtime::wire_questionnaire` 参照）。
//!
//! # fail-closed 契約（改ざん・欠損入力）
//!
//! - click 対象（またはその祖先、インスタンス root まで）に
//!   `data-disabled` または `disabled` 属性がある → no-op（ブラウザが
//!   disabled ボタンの合成 click を抑止することに依存せず本モジュール側で
//!   判定する、`sidebar::wiring::has_disabled_ancestor` と同型）。
//! - `data-step` が欠落・非数値 → no-op。`data-step > count` → no-op
//!   （[`Questionnaire::from_hydration_attrs`] の拒否と同じ判断。
//!   [`Questionnaire::new`] のクランプは使わず拒否する）。
//! - `data-orientation` が欠落・`horizontal`/`vertical` 以外 → no-op。
//! - `question` 要素が 0 個 → no-op。
//! - `data-index` が非数値の question → その要素のみスキップ（他は更新）。
//! - `try_borrow_mut` 失敗（再入）→ no-op。panic しない。
//! - questionnaire パーツを持たないアプリでは click 経路が常に早期
//!   return（副作用なし）。
//!
//! # セキュリティ不変条件
//!
//! - DOM 反映は `set_attribute`/`remove_attribute` のみで行い、HTML
//!   文字列を一切組み立てない（REQ-1）。属性名はすべて `&'static str`
//!   リテラル、値は数値整形済み文字列・固定書式（`"{percent}% complete"`）
//!   ・空文字のみ。書き込みは `set_dom_attribute`（`fw gate` の
//!   `url_validation_check` 契約）を経由する。
//! - `data-index`/`data-step` はクライアント改ざん可能な入力として
//!   `str::parse::<usize>` でのみ解釈し、HTML/セレクタとして解釈しない。
//! - リスナーは `root` あたり click 1 個・`Closure::forget` は 1 回のみ。
//!   1 click あたりの処理はインスタンス配下の question 数に比例する有界
//!   処理で、非同期予約のような継続副作用を持たない。
//! - 新規 `unsafe` コードは追加しない（`web-sys`/`js-sys` の safe API のみ
//!   使用）。
//!
//! # out-of-scope（イシュー #2118）
//!
//! - `data-answered`/`data-skipped`/`data-required`/`data-invalid` の DOM
//!   更新（アプリ責務、規則 1）。
//! - `"goto"`（任意 step への直接移動）の DOM 配線と分岐の部品内実装。
//! - 遷移により `back`/`next` が `disabled` になった際のフォーカス移動。
//! - `crates/pre-styled-ui` の recipe・golden・Themes ページ（兄弟イシュー
//!   #2119）。
//! - `steps` scope の同型配線（別イシュー対象）。

use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::questionnaire::Questionnaire;

/// Questionnaire の `data-scope` 属性値
/// （`fandhe_frontend_headless_ui::questionnaire` の `ANATOMY` と一致）。
const QUESTIONNAIRE_SCOPE: &str = "questionnaire";
/// Root パーツの `data-part` 属性値。wasm32 配線層専用の定数だが、native の
/// 非テストビルドでは未使用と誤検出される
/// （`headless_timer.rs::AREA_PART` と同じ理由の dead_code 抑制）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const ROOT_PART: &str = "root";
/// Question パーツの `data-part` 属性値（wasm32 配線層専用の定数だが、
/// native の非テストビルドでは未使用と誤検出される）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const QUESTION_PART: &str = "question";
/// Progress パーツの `data-part` 属性値（wasm32 配線層専用の定数だが、
/// native の非テストビルドでは未使用と誤検出される）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const PROGRESS_PART: &str = "progress";
/// Back パーツの `data-part` 属性値。
const BACK_PART: &str = "back";
/// Next パーツの `data-part` 属性値。
const NEXT_PART: &str = "next";
/// Skip パーツの `data-part` 属性値。
const SKIP_PART: &str = "skip";

/// `data-state` 属性値 "completed"（[`fandhe_frontend_headless_ui::questionnaire`]
/// の語彙と一致、[`crates/wasm-full/tests/questionnaire_native.rs`] 側の
/// ドリフト検知テストで突合する）。
const DATA_STATE_COMPLETED: &str = "completed";
/// `data-state` 属性値 "active"。
const DATA_STATE_ACTIVE: &str = "active";
/// `data-state` 属性値 "upcoming"。
const DATA_STATE_UPCOMING: &str = "upcoming";

/// `C` への通知アクション名（前へ）。モジュール冒頭「アプリ状態 `C` への
/// 通知」節参照。wasm32 配線層専用の定数だが、native の非テストビルドでは
/// 未使用と誤検出されるため、`headless_timer.rs::AREA_PART` と同じ理由で
/// dead_code を抑制する。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub const ACTION_PREV: &str = "questionnaire:prev";
/// `C` への通知アクション名（次へ）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub const ACTION_NEXT: &str = "questionnaire:next";
/// `C` への通知アクション名（スキップ）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub const ACTION_SKIP: &str = "questionnaire:skip";

// ---------------------------------------------------------------------
// 純粋ロジック層: web-sys 非依存。native の `cargo test --workspace` で
// 検証できる（`sidebar.rs`/`chart.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------

/// クリックターゲット（またはその祖先）の `data-scope`/`data-part` から
/// dispatch アクション名（`"prev"`/`"next"`/`"skip"`）を決定する allowlist
/// 変換（完全一致のみ）。`scope` が `"questionnaire"` でない、または `part`
/// が back/next/skip のいずれでもない場合は `None`（fail-closed）。
///
/// `has_explicit_action`（その要素自身が `data-action` 属性を持つか）が
/// `true` の場合も `None` を返す。`TRIGGER_RESERVED`
/// （`crates/headless-ui/src/questionnaire.rs`）は `"type"`/`"disabled"`/
/// `"data-disabled"` のみを予約し `"data-action"` を落とさないため、
/// アプリは back/next/skip へ `data-action` を明示的に付与して
/// `crate::events::wire_events` の汎用配線（`closest("[data-action]")` →
/// `C::decode_action`）に委ねることができる。この場合に本モジュールの
/// 自動配線も反応すると、`root` へ登録された 2 個のクリックリスナー
/// （`events::wire_events` と本モジュールの委譲リスナー）が同一クリックを
/// 二重に処理し、`C` 側の遷移と本モジュールの DOM 直書き遷移が別々に
/// 状態を進めて二重遷移になる（イシュー #2118 PR #2286 codex-review P1
/// 指摘）。`data-action` の存在はアプリが手動配線を明示的に選んだ合図と
/// みなし、本モジュールの自動配線はその要素に対しては早期に諦める
/// （継続して祖先を探索しない。back/next/skip 自身より外側の祖先に
/// 偶然 back/next/skip が存在することは想定しない）。
#[must_use]
pub fn trigger_action(
    scope: Option<&str>,
    part: Option<&str>,
    has_explicit_action: bool,
) -> Option<&'static str> {
    if scope != Some(QUESTIONNAIRE_SCOPE) {
        return None;
    }
    if has_explicit_action {
        return None;
    }
    match part {
        Some(p) if p == BACK_PART => Some("prev"),
        Some(p) if p == NEXT_PART => Some("next"),
        Some(p) if p == SKIP_PART => Some("skip"),
        _ => None,
    }
}

/// dispatch アクション名（`"prev"`/`"next"`/`"skip"`）を `C` への通知
/// アクション名（`"questionnaire:*"`）へ変換する。未知の名前は `None`。
#[must_use]
pub fn notification_action(action: &str) -> Option<&'static str> {
    match action {
        "prev" => Some(ACTION_PREV),
        "next" => Some(ACTION_NEXT),
        "skip" => Some(ACTION_SKIP),
        _ => None,
    }
}

/// 通知 payload の区切り文字。`step` は `usize` の 10 進文字列のため
/// 区切り文字を含み得ない（fail-closed に一意分割できる、
/// [`decode_notification_payload`] 参照）。
const NOTIFICATION_PAYLOAD_SEP: char = '|';

/// `C` への通知 payload をエンコードする（モジュール冒頭「アプリ状態 `C`
/// への通知」節参照）。`step` を先頭に置くのは、`step` が区切り文字を
/// 含み得ない `usize` の 10 進文字列であるのに対し `instance_id`
/// （インスタンス root の `id` 属性値、未設定時は空文字列）はアプリが
/// 任意の文字列を設定でき区切り文字を含み得るため（イシュー #2118
/// PR #2286 codex-review P1 指摘: 複数インスタンス時に通知からどの
/// インスタンスか判別できない）。`instance_id` を先頭に置く設計は
/// `id` に区切り文字が含まれた場合に `step` 側の分割が曖昧になり得る。
#[must_use]
pub fn encode_notification_payload(step: usize, instance_id: &str) -> String {
    format!("{step}{NOTIFICATION_PAYLOAD_SEP}{instance_id}")
}

/// [`encode_notification_payload`] の逆変換。`(step, instance_id)` を
/// 返す。区切り文字が見つからない、または `step` 側が非数値の場合は
/// `None`（fail-closed。改ざんされうるクライアント入力ではなく本モジュール
/// 自身が生成した payload の形式検証だが、契約破りの入力を潜在的に受け
/// 得る任意の呼び出し元のため同じ規律を適用する）。
#[must_use]
pub fn decode_notification_payload(payload: &str) -> Option<(usize, &str)> {
    let (step, instance_id) = payload.split_once(NOTIFICATION_PAYLOAD_SEP)?;
    let step = step.parse::<usize>().ok()?;
    Some((step, instance_id))
}

/// インスタンス root の表示属性（`data-step`/`data-orientation`）と、
/// そのインスタンスに属する question 要素数（`count`）から
/// [`Questionnaire`] を再構築する純粋関数（モジュール冒頭「DOM を
/// Questionnaire の一時的な真として扱う」節参照）。改ざん・欠落・
/// 範囲外は `None`（fail-closed、[`Questionnaire::new`] のクランプは
/// 使わず拒否する）。
#[must_use]
pub fn questionnaire_from_display_attrs(
    step: Option<&str>,
    orientation: Option<&str>,
    count: usize,
) -> Option<Questionnaire> {
    if count == 0 {
        return None;
    }
    let step = step?.parse::<usize>().ok()?;
    if step > count {
        return None;
    }
    let orientation = match orientation? {
        "horizontal" => Orientation::Horizontal,
        "vertical" => Orientation::Vertical,
        _ => return None,
    };
    Some(Questionnaire::new(count, step, orientation))
}

/// `index`（`0..count`）の 3 状態を `step` から導出する
/// （[`fandhe_frontend_headless_ui::questionnaire::Questionnaire`] 内部の
/// 判定ロジックと同型。ドリフトは native ドリフト検知テストで固定する）。
#[must_use]
pub fn question_data_state(step: usize, index: usize) -> &'static str {
    if index < step {
        DATA_STATE_COMPLETED
    } else if index == step {
        DATA_STATE_ACTIVE
    } else {
        DATA_STATE_UPCOMING
    }
}

/// `index` の question が非 active（`hidden` を付与すべき）かどうか。
#[must_use]
pub fn is_question_hidden(step: usize, index: usize) -> bool {
    question_data_state(step, index) != DATA_STATE_ACTIVE
}

/// [`Questionnaire`] から `progress` パーツへ書き戻す 3 値
/// （`aria-valuenow` 文字列・`aria-valuetext` 文字列・`data-complete` 有無）
/// を計算する。`step`/`count` は usize のためオーバーフローの恐れがある
/// 乗算を u128 へ拡張してから行う
/// （`crates/headless-ui/src/questionnaire.rs::Questionnaire::progress` と
/// 同型の対策）。
#[must_use]
pub fn progress_values(q: &Questionnaire) -> (String, String, bool) {
    let percent = (q.step() as u128 * 100 / q.count() as u128) as usize;
    (
        percent.to_string(),
        format!("{percent}% complete"),
        percent == 100,
    )
}

/// `back`/`next`・`skip` trigger の境界条件が `before` → `after` で変化
/// したかどうかを表す（`None` = 変化なし＝触らない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TriggerBoundary {
    /// `back` の境界（`step == 0`）が変化した場合、変化後の disabled 値。
    pub back: Option<bool>,
    /// `next`/`skip` の境界（`step == count`）が変化した場合、変化後の
    /// disabled 値。
    pub next_skip: Option<bool>,
}

/// `before`/`after` 間で back・next/skip の境界条件（`step == 0`・
/// `step == count`）が変化したかどうかを判定する（モジュール冒頭「trigger
/// の disabled をエッジ変化時にのみ触る理由」節参照）。
#[must_use]
pub fn trigger_boundary_transition(
    before: &Questionnaire,
    after: &Questionnaire,
) -> TriggerBoundary {
    let before_back_edge = before.step() == 0;
    let after_back_edge = after.step() == 0;
    let back = (before_back_edge != after_back_edge).then_some(after_back_edge);

    let before_next_edge = before.step() == before.count();
    let after_next_edge = after.step() == after.count();
    let next_skip = (before_next_edge != after_next_edge).then_some(after_next_edge);

    TriggerBoundary { back, next_skip }
}

// ---------------------------------------------------------------------
// 配線層: web-sys/js-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`headless_timer.rs`/`sidebar.rs`/`chart.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        encode_notification_payload, is_question_hidden, notification_action, progress_values,
        question_data_state, questionnaire_from_display_attrs, trigger_action,
        trigger_boundary_transition, Questionnaire, BACK_PART, NEXT_PART, PROGRESS_PART,
        QUESTIONNAIRE_SCOPE, QUESTION_PART, ROOT_PART, SKIP_PART,
    };
    use crate::events::ActionRef;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event};

    /// `target` から `root`（含む）まで祖先方向へ辿り、`data-scope`/
    /// `data-part` が指定値と一致する最初の要素を返す
    /// （`crate::sidebar::wiring::closest_matching`/
    /// `crate::headless_timer::wiring::closest_matching` と同型）。
    fn closest_matching(
        root: &Element,
        start: &Element,
        scope: &str,
        part: &str,
    ) -> Option<Element> {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if !root.contains(Some(&element)) {
                break;
            }
            if element.get_attribute("data-scope").as_deref() == Some(scope)
                && element.get_attribute("data-part").as_deref() == Some(part)
            {
                return Some(element);
            }
            if element == *root {
                break;
            }
            current = element.parent_element();
        }
        None
    }

    /// `target` から `root`（含む）まで祖先方向へ辿り、back/next/skip の
    /// いずれかに一致する最初の要素と、対応する dispatch アクション名を
    /// 返す（[`super::trigger_action`] の allowlist 判定を各祖先へ適用）。
    /// 一致した要素が `data-action` を持つ場合は、アプリが手動配線を
    /// 明示的に選んだものとして自動配線を諦める（[`super::trigger_action`]
    /// rustdoc 参照。祖先探索は継続しない）。
    fn resolve_trigger(root: &Element, start: &Element) -> Option<(Element, &'static str)> {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if !root.contains(Some(&element)) {
                break;
            }
            let scope = element.get_attribute("data-scope");
            let part = element.get_attribute("data-part");
            let has_explicit_action = element.has_attribute("data-action");
            if let Some(action) =
                trigger_action(scope.as_deref(), part.as_deref(), has_explicit_action)
            {
                return Some((element, action));
            }
            if element == *root {
                break;
            }
            current = element.parent_element();
        }
        None
    }

    /// `start` から `boundary`（含む）まで祖先方向を辿り、`data-disabled`
    /// またはネイティブ `disabled` 属性を持つ要素が 1 つでもあれば `true`
    /// （`crate::sidebar::wiring::has_disabled_ancestor` と同型。ブラウザが
    /// disabled ボタンの合成 click を抑止することに依存せず本モジュール側で
    /// 判定する、モジュール冒頭「fail-closed 契約」節参照）。
    fn has_disabled_ancestor(boundary: &Element, start: &Element) -> bool {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if element.has_attribute("data-disabled") || element.has_attribute("disabled") {
                return true;
            }
            if !boundary.contains(Some(&element)) || element == *boundary {
                break;
            }
            current = element.parent_element();
        }
        false
    }

    /// `instance_root` 配下（`instance_root` 自身を除く）から `data-scope`/
    /// `data-part` が一致する要素を document 順に収集し、各要素の最寄りの
    /// questionnaire root が `instance_root` 自身であるものだけへ絞り込む
    /// （モジュール冒頭「同一 `root` 配下の複数インスタンス・入れ子
    /// インスタンス」節参照。入れ子インスタンスの子要素を親側の集計へ
    /// 取りこぼさない・混入させないための不変条件）。
    fn collect_scoped(instance_root: &Element, part: &str) -> Vec<Element> {
        let selector = format!(r#"[data-scope="{QUESTIONNAIRE_SCOPE}"][data-part="{part}"]"#);
        let Ok(node_list) = instance_root.query_selector_all(&selector) else {
            return Vec::new();
        };
        let len = node_list.length();
        let mut out = Vec::with_capacity(len as usize);
        for i in 0..len {
            let Some(node) = node_list.get(i) else {
                continue;
            };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            if closest_matching(instance_root, &element, QUESTIONNAIRE_SCOPE, ROOT_PART).as_ref()
                == Some(instance_root)
            {
                out.push(element);
            }
        }
        out
    }

    /// `instance_root` 配下の question 要素を収集する
    /// （[`collect_scoped`] の薄いエイリアス）。
    fn collect_questions(instance_root: &Element) -> Vec<Element> {
        collect_scoped(instance_root, QUESTION_PART)
    }

    /// `element.set_attribute(name, value)` の薄いガード付きラッパー
    /// （`crate::headless_timer::wiring::set_dom_attribute` と同型、
    /// イシュー #401 の `fw gate` `url_validation_check` 契約に準拠）。
    fn set_dom_attribute(element: &Element, name: &str, value: &str) -> Result<(), JsValue> {
        if fandhe_frontend_core::is_event_handler_attr(name) {
            return Ok(());
        }
        if fandhe_frontend_core::is_url_attr(name) && !fandhe_frontend_core::is_safe_url(value) {
            return Ok(());
        }
        if name.eq_ignore_ascii_case("srcset") && !fandhe_frontend_core::is_safe_srcset(value) {
            return Ok(());
        }
        element.set_attribute(name, value)
    }

    /// 存在属性（値は常に空文字）を `enabled` に応じて付与/除去する。
    fn set_existence_attr(element: &Element, name: &str, enabled: bool) -> Result<(), JsValue> {
        if enabled {
            set_dom_attribute(element, name, "")
        } else {
            element.remove_attribute(name)
        }
    }

    /// `instance_root` の表示属性と、そのインスタンスに属する question 数
    /// から [`Questionnaire`] を再構築する
    /// （[`super::questionnaire_from_display_attrs`] への薄い DOM 読み取り
    /// 委譲）。
    fn read_questionnaire(instance_root: &Element) -> Option<Questionnaire> {
        let count = collect_questions(instance_root).len();
        questionnaire_from_display_attrs(
            instance_root.get_attribute("data-step").as_deref(),
            instance_root.get_attribute("data-orientation").as_deref(),
            count,
        )
    }

    /// `before`（dispatch 前）から `after`（dispatch 後）への遷移を
    /// `instance_root`・各 question・`progress`・back/next/skip trigger へ
    /// 反映する（モジュール冒頭「書き戻し対象と規則」節参照）。
    ///
    /// # Errors
    ///
    /// `query_selector_all`/`set_attribute` の失敗を伝播する。
    fn write_questionnaire(
        instance_root: &Element,
        before: &Questionnaire,
        after: &Questionnaire,
    ) -> Result<(), JsValue> {
        set_dom_attribute(instance_root, "data-step", &after.step().to_string())?;
        set_existence_attr(instance_root, "data-complete", after.is_completed())?;

        for question in collect_questions(instance_root) {
            let Some(index) = question
                .get_attribute("data-index")
                .and_then(|s| s.parse::<usize>().ok())
            else {
                continue;
            };
            set_dom_attribute(
                &question,
                "data-state",
                question_data_state(after.step(), index),
            )?;
            set_existence_attr(&question, "hidden", is_question_hidden(after.step(), index))?;
        }

        let (before_now, _before_text, _before_complete) = progress_values(before);
        let (after_now, after_text, after_complete) = progress_values(after);
        for progress in collect_scoped(instance_root, PROGRESS_PART) {
            if progress.get_attribute("aria-valuenow").as_deref() != Some(before_now.as_str()) {
                continue;
            }
            set_dom_attribute(&progress, "aria-valuenow", &after_now)?;
            set_dom_attribute(&progress, "aria-valuetext", &after_text)?;
            set_existence_attr(&progress, "data-complete", after_complete)?;
        }

        let boundary = trigger_boundary_transition(before, after);
        if let Some(disabled) = boundary.back {
            for back in collect_scoped(instance_root, BACK_PART) {
                set_existence_attr(&back, "disabled", disabled)?;
                set_existence_attr(&back, "data-disabled", disabled)?;
            }
        }
        if let Some(disabled) = boundary.next_skip {
            for next in collect_scoped(instance_root, NEXT_PART) {
                set_existence_attr(&next, "disabled", disabled)?;
                set_existence_attr(&next, "data-disabled", disabled)?;
            }
            for skip in collect_scoped(instance_root, SKIP_PART) {
                set_existence_attr(&skip, "disabled", disabled)?;
                set_existence_attr(&skip, "data-disabled", disabled)?;
            }
        }

        Ok(())
    }

    /// `on_action` へ 1 アクションを通知する（`try_borrow_mut` 失敗＝再入は
    /// no-op、panic 回避。`crate::headless_timer::wiring::notify_action`
    /// と同型）。
    fn notify_action(
        action: &str,
        payload: &str,
        on_action: &Rc<RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        if let Ok(mut cb) = on_action.try_borrow_mut() {
            (cb)(ActionRef {
                action: action.to_string(),
                payload: payload.to_string(),
            });
        }
    }

    /// click イベント 1 件を処理する。back/next/skip 上のクリックのみ反応し
    /// （fail-closed、無関係要素上のクリックは無視）、disabled 祖先チェック
    /// → インスタンス解決 → dispatch → 実際に状態が変化した場合のみ DOM
    /// 反映と `C` への通知を行う（モジュール冒頭「fail-closed 契約」節
    /// 参照）。
    fn handle_click(
        root: &Element,
        event: &Event,
        on_action: &Rc<RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        let Some(target) = event.target() else {
            return;
        };
        let target_element: Element = match target.dyn_ref::<Element>() {
            Some(element) => element.clone(),
            None => {
                let Some(node) = target.dyn_ref::<web_sys::Node>() else {
                    return;
                };
                let Some(parent) = node.parent_element() else {
                    return;
                };
                parent
            }
        };

        let Some((trigger, action)) = resolve_trigger(root, &target_element) else {
            return;
        };

        let Some(instance_root) = closest_matching(root, &trigger, QUESTIONNAIRE_SCOPE, ROOT_PART)
        else {
            return;
        };

        if has_disabled_ancestor(&instance_root, &trigger) {
            return;
        }

        let Some(mut questionnaire) = read_questionnaire(&instance_root) else {
            return;
        };
        let before = questionnaire;
        if !fandhe_frontend_interactive::dispatch(&mut questionnaire, action, "") {
            return;
        }
        if before == questionnaire {
            // 境界（例: 完了状態での next）での no-op。DOM も書かず通知も
            // 抑止する（モジュール冒頭「アプリ状態 `C` への通知」節参照）。
            return;
        }

        let _ = write_questionnaire(&instance_root, &before, &questionnaire);
        if let Some(notif) = notification_action(action) {
            // インスタンス root の `id` 属性を安定なインスタンス識別子として
            // 通知へ含める（イシュー #2118 PR #2286 codex-review P1 指摘。
            // 未設定時は空文字列 — 単一インスタンスのアプリはこれまでどおり
            // 空の識別子で動作し、複数インスタンスを区別したいアプリは
            // root へ `id` を設定する契約とする。`id` はアプリが authoring
            // 時に付与する属性でありクライアント改ざん入力ではないため、
            // そのまま通知へ転記して構わない（HTML/セレクタとしては解釈
            // しない、不透明な文字列として扱う）。
            let instance_id = instance_root.get_attribute("id").unwrap_or_default();
            let payload = encode_notification_payload(before.step(), &instance_id);
            notify_action(notif, &payload, on_action);
        }
    }

    /// `root` 配下の Questionnaire back/next/skip クリックへ配線を 1 回だけ
    /// 登録する。
    ///
    /// click はバブリングするため、`root` への委譲リスナー 1 個で完結する
    /// （`crate::headless_timer::wiring::wire_timer_events` と同型）。
    /// `root` は任意の祖先でよい（モジュール冒頭「DOM を Questionnaire の
    /// 一時的な真として扱う」節参照。`Timer` と異なりリスナー登録先自身が
    /// インスタンス root である必要はない）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback` の失敗を伝播する。
    pub fn wire_questionnaire_events(
        root: Element,
        on_action: impl FnMut(ActionRef) + 'static,
    ) -> Result<(), JsValue> {
        let on_action = Rc::new(RefCell::new(on_action));

        let click_root = root.clone();
        let click_on_action = on_action.clone();
        let closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_click(&click_root, &event, &click_on_action);
        });
        root.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_questionnaire_events;

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    // --- trigger_action ---

    #[test]
    fn trigger_action_matches_back_next_skip_exact_scope() {
        assert_eq!(
            trigger_action(Some("questionnaire"), Some("back"), false),
            Some("prev")
        );
        assert_eq!(
            trigger_action(Some("questionnaire"), Some("next"), false),
            Some("next")
        );
        assert_eq!(
            trigger_action(Some("questionnaire"), Some("skip"), false),
            Some("skip")
        );
    }

    #[test]
    fn trigger_action_rejects_wrong_scope_or_unknown_part() {
        assert_eq!(trigger_action(Some("steps"), Some("next"), false), None);
        assert_eq!(
            trigger_action(Some("questionnaire"), Some("root"), false),
            None
        );
        assert_eq!(trigger_action(None, None, false), None);
    }

    #[test]
    fn trigger_action_yields_to_explicit_data_action() {
        // `data-action` を明示的に持つ back/next/skip は、アプリが手動配線
        // （`events::wire_events`）を選んだものとみなし自動配線を諦める
        // （イシュー #2118 PR #2286 codex-review P1 指摘、二重遷移回帰）。
        assert_eq!(
            trigger_action(Some("questionnaire"), Some("back"), true),
            None
        );
        assert_eq!(
            trigger_action(Some("questionnaire"), Some("next"), true),
            None
        );
        assert_eq!(
            trigger_action(Some("questionnaire"), Some("skip"), true),
            None
        );
    }

    // --- notification_action ---

    #[test]
    fn notification_action_maps_all_three() {
        assert_eq!(notification_action("prev"), Some(ACTION_PREV));
        assert_eq!(notification_action("next"), Some(ACTION_NEXT));
        assert_eq!(notification_action("skip"), Some(ACTION_SKIP));
        assert_eq!(notification_action("goto"), None);
        assert_eq!(notification_action("bogus"), None);
    }

    // --- encode_notification_payload / decode_notification_payload ---

    #[test]
    fn notification_payload_round_trips_with_instance_id() {
        let encoded = encode_notification_payload(2, "qn-instance-a");
        assert_eq!(encoded, "2|qn-instance-a");
        assert_eq!(
            decode_notification_payload(&encoded),
            Some((2, "qn-instance-a"))
        );
    }

    #[test]
    fn notification_payload_round_trips_with_empty_instance_id() {
        // 単一インスタンスのアプリは root へ `id` を設定しない想定
        // （後方互換: 空文字列の識別子として動作する）。
        let encoded = encode_notification_payload(0, "");
        assert_eq!(encoded, "0|");
        assert_eq!(decode_notification_payload(&encoded), Some((0, "")));
    }

    #[test]
    fn notification_payload_decode_rejects_missing_separator() {
        assert_eq!(decode_notification_payload("2"), None);
        assert_eq!(decode_notification_payload(""), None);
    }

    #[test]
    fn notification_payload_decode_rejects_non_numeric_step() {
        assert_eq!(decode_notification_payload("abc|qn-instance-a"), None);
    }

    #[test]
    fn notification_payload_instance_id_may_contain_separator() {
        // `step` を先頭に置く設計により、`instance_id` 自体に `|` が
        // 含まれていても `split_once` は最初の `|` で分割するため
        // `step` 側の解釈は曖昧にならない。
        let encoded = encode_notification_payload(1, "a|b");
        assert_eq!(decode_notification_payload(&encoded), Some((1, "a|b")));
    }

    // --- questionnaire_from_display_attrs（fail-closed） ---

    #[test]
    fn from_display_attrs_accepts_valid_input() {
        let q = questionnaire_from_display_attrs(Some("1"), Some("horizontal"), 3).unwrap();
        assert_eq!(q.count(), 3);
        assert_eq!(q.step(), 1);
        assert_eq!(q.orientation(), Orientation::Horizontal);
    }

    #[test]
    fn from_display_attrs_accepts_vertical() {
        let q = questionnaire_from_display_attrs(Some("0"), Some("vertical"), 2).unwrap();
        assert_eq!(q.orientation(), Orientation::Vertical);
    }

    #[test]
    fn from_display_attrs_rejects_missing_step() {
        assert!(questionnaire_from_display_attrs(None, Some("horizontal"), 3).is_none());
    }

    #[test]
    fn from_display_attrs_rejects_non_numeric_step() {
        assert!(questionnaire_from_display_attrs(Some("abc"), Some("horizontal"), 3).is_none());
    }

    #[test]
    fn from_display_attrs_rejects_step_greater_than_count() {
        assert!(questionnaire_from_display_attrs(Some("9"), Some("horizontal"), 3).is_none());
    }

    #[test]
    fn from_display_attrs_rejects_unknown_orientation() {
        assert!(questionnaire_from_display_attrs(Some("0"), Some("diagonal"), 3).is_none());
        assert!(questionnaire_from_display_attrs(Some("0"), None, 3).is_none());
    }

    #[test]
    fn from_display_attrs_rejects_zero_count() {
        assert!(questionnaire_from_display_attrs(Some("0"), Some("horizontal"), 0).is_none());
    }

    // --- question_data_state / is_question_hidden ---

    #[test]
    fn question_data_state_reflects_index_vs_step() {
        assert_eq!(question_data_state(1, 0), DATA_STATE_COMPLETED);
        assert_eq!(question_data_state(1, 1), DATA_STATE_ACTIVE);
        assert_eq!(question_data_state(1, 2), DATA_STATE_UPCOMING);
    }

    #[test]
    fn is_question_hidden_true_only_for_non_active() {
        assert!(is_question_hidden(1, 0));
        assert!(!is_question_hidden(1, 1));
        assert!(is_question_hidden(1, 2));
    }

    // --- progress_values ---

    #[test]
    fn progress_values_computes_percent_and_text() {
        let q = Questionnaire::new(3, 1, Orientation::Horizontal);
        let (now, text, complete) = progress_values(&q);
        assert_eq!(now, "33");
        assert_eq!(text, "33% complete");
        assert!(!complete);
    }

    #[test]
    fn progress_values_marks_complete_at_100_percent() {
        let q = Questionnaire::new(3, 3, Orientation::Horizontal);
        let (now, text, complete) = progress_values(&q);
        assert_eq!(now, "100");
        assert_eq!(text, "100% complete");
        assert!(complete);
    }

    #[test]
    fn progress_values_does_not_overflow_near_usize_max() {
        // u128 拡張により `step * 100` の usize オーバーフローが起きない
        // ことを大きめの値で確認する（headless-ui 側 PR #1941 P1 指摘と
        // 同型の境界確認）。
        let q = Questionnaire::new(usize::MAX, usize::MAX, Orientation::Horizontal);
        let (now, _text, complete) = progress_values(&q);
        assert_eq!(now, "100");
        assert!(complete);
    }

    // --- trigger_boundary_transition ---

    #[test]
    fn boundary_transition_detects_back_edge_leaving() {
        let before = Questionnaire::new(3, 0, Orientation::Horizontal);
        let after = Questionnaire::new(3, 1, Orientation::Horizontal);
        let boundary = trigger_boundary_transition(&before, &after);
        assert_eq!(boundary.back, Some(false));
        assert_eq!(boundary.next_skip, None);
    }

    #[test]
    fn boundary_transition_detects_back_edge_entering() {
        let before = Questionnaire::new(3, 1, Orientation::Horizontal);
        let after = Questionnaire::new(3, 0, Orientation::Horizontal);
        let boundary = trigger_boundary_transition(&before, &after);
        assert_eq!(boundary.back, Some(true));
        assert_eq!(boundary.next_skip, None);
    }

    #[test]
    fn boundary_transition_detects_next_edge_entering() {
        let before = Questionnaire::new(3, 2, Orientation::Horizontal);
        let after = Questionnaire::new(3, 3, Orientation::Horizontal);
        let boundary = trigger_boundary_transition(&before, &after);
        assert_eq!(boundary.back, None);
        assert_eq!(boundary.next_skip, Some(true));
    }

    #[test]
    fn boundary_transition_detects_next_edge_leaving() {
        let before = Questionnaire::new(3, 3, Orientation::Horizontal);
        let after = Questionnaire::new(3, 2, Orientation::Horizontal);
        let boundary = trigger_boundary_transition(&before, &after);
        assert_eq!(boundary.back, None);
        assert_eq!(boundary.next_skip, Some(false));
    }

    #[test]
    fn boundary_transition_none_on_non_edge_moves() {
        let before = Questionnaire::new(5, 1, Orientation::Horizontal);
        let after = Questionnaire::new(5, 2, Orientation::Horizontal);
        let boundary = trigger_boundary_transition(&before, &after);
        assert_eq!(boundary, TriggerBoundary::default());
    }

    // --- ドリフト検知: headless-ui 実出力との突合
    // （headless-ui は wasm-full の通常依存のため native から render 可能）。
    // ---

    #[test]
    fn drift_question_data_state_matches_headless_ui_render() {
        let q = Questionnaire::new(3, 1, Orientation::Horizontal);
        for index in 0..3 {
            let html = render(&q.question(
                index,
                fandhe_frontend_headless_ui::questionnaire::QuestionProps::default(),
                vec![],
                vec![],
            ));
            let expected = format!(r#"data-state="{}""#, question_data_state(1, index));
            assert!(html.contains(&expected), "html={html} expected={expected}");
            assert_eq!(html.contains("hidden"), is_question_hidden(1, index));
        }
    }

    #[test]
    fn drift_progress_values_matches_headless_ui_render() {
        let q = Questionnaire::new(3, 1, Orientation::Horizontal);
        let html = render(&q.progress("", vec![], vec![]));
        let (now, text, _complete) = progress_values(&q);
        assert!(html.contains(&format!(r#"aria-valuenow="{now}""#)));
        assert!(html.contains(&format!(r#"aria-valuetext="{text}""#)));
    }

    #[test]
    fn drift_completed_state_marks_root_and_progress_data_complete() {
        let q = Questionnaire::new(3, 3, Orientation::Horizontal);
        assert!(render(&q.root(vec![], vec![])).contains("data-complete"));
        assert!(render(&q.progress("", vec![], vec![])).contains("data-complete"));
        let (_now, _text, complete) = progress_values(&q);
        assert!(complete);
    }

    #[test]
    fn drift_boundary_triggers_carry_disabled_attrs() {
        let start = Questionnaire::new(3, 0, Orientation::Horizontal);
        assert!(render(&start.back(false, vec![], vec![])).contains("disabled"));
        assert!(render(&start.back(false, vec![], vec![])).contains("data-disabled"));

        let end = Questionnaire::new(3, 3, Orientation::Horizontal);
        assert!(render(&end.next(false, vec![], vec![])).contains("disabled"));
        assert!(render(&end.skip(false, vec![], vec![])).contains("disabled"));
    }

    #[test]
    fn drift_data_part_names_exist_in_rendered_output() {
        let q = Questionnaire::new(1, 0, Orientation::Horizontal);
        let root_html = render(&q.root(vec![], vec![]));
        assert!(root_html.contains(&format!(r#"data-part="{ROOT_PART}""#)));

        let question_html = render(&q.question(
            0,
            fandhe_frontend_headless_ui::questionnaire::QuestionProps::default(),
            vec![],
            vec![],
        ));
        assert!(question_html.contains(&format!(r#"data-part="{QUESTION_PART}""#)));

        let progress_html = render(&q.progress("", vec![], vec![]));
        assert!(progress_html.contains(&format!(r#"data-part="{PROGRESS_PART}""#)));

        assert!(
            render(&q.back(false, vec![], vec![])).contains(&format!(r#"data-part="{BACK_PART}""#))
        );
        assert!(
            render(&q.next(false, vec![], vec![])).contains(&format!(r#"data-part="{NEXT_PART}""#))
        );
        assert!(
            render(&q.skip(false, vec![], vec![])).contains(&format!(r#"data-part="{SKIP_PART}""#))
        );
    }
}
