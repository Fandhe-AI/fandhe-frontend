//! Questionnaire（多段の質問提示、単一選択・複数選択・自由記述・スキップ可）
//! headless コンポーネント（イシュー #2117、親 #2116、祖父 #2057、
//! shadcn/ui `Questionnaire` 相当、参照軸 #2001）。
//!
//! Root / Progress / Question / Prompt / Description / Options / Freeform /
//! Actions / Back / Next / Skip の 11 anatomy パーツ（イシュータイトルの
//! 6 パーツに、親 #2116 が列挙する prompt / description / back / next / skip
//! を加えたもの。差分は「イシュータイトルとの差分」節参照）と、
//! [`crate::steps::Steps`] と同型に [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装する状態機械
//! [`Questionnaire`] を提供する。
//!
//! # 責務境界（`.claude/rules/coding-rust.md` §UI 部品の責務境界、規則 1）
//!
//! 回答値の保持・検証（必須判定）・分岐（次にどの質問へ進むか）・送信は
//! アプリケーション責務であり、本モジュールは持たない。部品が担うのは
//! 「現在位置（`step`）に応じて質問の表示状態を切り替え、前へ / 次へ /
//! スキップのトリガーを出す」までである。`next`/`skip` の `data-disabled`
//! は、必須判定の**結果（真偽）だけ**を呼び出し側から [`Questionnaire::next`]/
//! [`Questionnaire::skip`] の `disabled` 引数として受け取る。
//!
//! # イシュータイトルとの差分
//!
//! イシュータイトルは root / progress / question / options / freeform /
//! actions の 6 パーツのみを挙げるが、親 #2116 は back / next / skip も
//! anatomy として明記しており、prompt（質問文の `legend`）/ description
//! （補足文のスロット）も質問の構成上不可欠なため、本実装は 11 パーツを
//! 持つ（[`crate::attachment`] が「イシュータイトルとの差分」として
//! action パーツを追加したのと同型の判断）。
//!
//! # `data-state` 語彙について（[`crate::steps::Steps`] の語彙を再利用しない理由）
//!
//! [`crate::steps::Steps`] の item は `complete`/`current`/`incomplete` の
//! 3 状態を持つが、本モジュールの question は親 #2116 の指定語彙
//! `active`/`completed`/`upcoming` を持つ（値そのものが異なるため
//! 共有ヘルパを再利用できない）。状態機械としては `count`/`step` から
//! 3 状態を導出する構造が [`crate::steps::Steps`] と同型であるため、
//! [`crate::progress::Progress`]/[`crate::slider::Slider`]/
//! [`crate::steps::Steps`] と同じ判断で、本モジュールも
//! [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装し、Phase 1 が
//! 確立した dispatch 契約（未知アクション no-op）・fail-closed hydration
//! という**統合様式**にのみ準拠する。
//!
//! # 状態モデル
//!
//! [`Questionnaire`] は `count`（全質問数、`>= 1`）と `step`（現在位置、
//! `0..=count`）を持つ（[`crate::steps::Steps`] と同じ正規化規則、
//! `normalize`）。`step == count` は「全質問完了」を表す
//! （[`Questionnaire::is_completed`]）。質問のインデックス `index`
//! （`0..count`）に対する 3 状態は:
//!
//! - `index < step` → completed
//! - `index == step` → active（`step == count` のときは該当する質問が
//!   存在しないため active な質問はない）
//! - `index > step` → upcoming
//!
//! `Skip` は状態遷移としては `Next` と同一（[`QuestionnaireAction::Skip`]も
//! `update()` 内で `Next` と同じく `saturating_add(1).min(count)` に進む）であり、
//! 「どの質問をスキップしたか」は本モジュールに保持しない。dispatch 名を
//! 分けるのは、呼び出し側（`fandhe-frontend-wasm-full` 配線・イシュー
//! #2118 のスコープ）がスキップ操作を観測して自身の状態へ記録できるように
//! するためである（責務境界規則 1。#2118 実装者は本モジュールへ
//! `skipped` を持ち込まないこと）。
//!
//! # 呼び出し文脈
//!
//! SSR は [`Questionnaire::new`] で値を正規化してから各パーツメソッド
//! （[`Questionnaire::root`]/[`Questionnaire::progress`]/
//! [`Questionnaire::question`]/[`Questionnaire::prompt`]/
//! [`Questionnaire::description`]/[`Questionnaire::options`]/
//! [`Questionnaire::freeform`]/[`Questionnaire::actions`]/
//! [`Questionnaire::back`]/[`Questionnaire::next`]/[`Questionnaire::skip`]）を
//! 呼んで組み立てる。[`Questionnaire::options`]/[`Questionnaire::freeform`]
//! は純スロットであり、呼び出し側が [`crate::radio_group`]/
//! [`crate::checkbox_group`] のパーツ（選択肢）や [`crate::field::textarea`]
//! （自由記述）を入れ子にする契約とする（両者の `data-scope` は
//! `questionnaire` scope と独立して残る）。CSR/hydration は
//! [`Questionnaire`] を経由し、dispatch（`"next"`/`"prev"`/`"skip"`/
//! `"goto"`）で状態遷移する。`fandhe-frontend-wasm-full` への配線
//! （trigger click → dispatch）は本イシューのスコープ外（#2118）。
//!
//! # shadcn/ui 実 API との意図的差分
//!
//! - `QuestionnaireError`: 非採用。検証結果の表示は呼び出し側が
//!   [`crate::field::error_text`]/[`crate::fieldset`] を question 内へ
//!   入れ子にする（責務境界規則 1）。
//! - `QuestionnaireSubmit`: 非採用。送信はアプリ責務であり、送信ボタンは
//!   呼び出し側が [`Questionnaire::actions`] スロットへ通常の `button` を
//!   置く。
//! - `QuestionnaireChoice`/`QuestionnaireInput`/`shortcuts`（英数字キー
//!   割当）/`required`/`multiple` の回答ロジック: 非採用。選択肢は
//!   [`crate::radio_group`]/[`crate::checkbox_group`]（ネイティブ
//!   radio/checkbox のキーボード操作を継承）、自由記述は
//!   [`crate::field::textarea`] の再利用で賄う。
//! - `items` 配列からの一括描画・回答状態管理・バリデーション・失敗時の
//!   フォーカス移動: 非採用（責務境界規則 1 / DOM 操作はクライアント
//!   ランタイムの責務）。
//!
//! # キーボード操作（headless 層での意味）
//!
//! [`Questionnaire::back`]/[`Questionnaire::next`]/[`Questionnaire::skip`]
//! はネイティブ `button`（Tab/Enter/Space）。`disabled` はネイティブ属性で
//! 活性化を抑止し `data-disabled` を併出力する。非 active な question は
//! `hidden` 属性により Tab 到達不能。options/freeform のキー操作は入れ子
//! にした radio_group / checkbox_group / textarea のものを継承し、
//! 本モジュール自身は矢印キー等の独自ハンドリングを持たない（実配線は
//! #2118 のスコープ）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`）はすべて `&'static str` リテラルで
//!   固定しており、動的値が属性名スロットへ混入する経路はない
//!   （[`crate::anatomy`](mod@crate::anatomy)/[`crate::data_attrs`]/[`crate::aria`] の既存
//!   不変条件をそのまま継承する）。
//! - 動的値（数値属性・呼び出し側 `attrs`・children テキスト・`label`）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-state` 値語彙（`"active"`/`"completed"`/`"upcoming"`）は本
//!   モジュール内で一元管理し（`Questionnaire::question_state`）、
//!   パーツ関数間で分裂させない。
//! - 各パーツへ `drop_reserved`（`crate::steps::drop_reserved` と同型）
//!   を導入し、呼び出し側 `attrs` が固定付与属性へなりすませないように
//!   する（A05 対策）。
//! - hydration 属性（`data-hydrate-count`/`data-hydrate-step`/
//!   `data-hydrate-orientation`）はクライアント側で改ざんされうる入力
//!   として扱う。[`Questionnaire`] の [`fandhe_frontend_interactive::Hydrate`]
//!   実装は panic せず `HydrateError` を返す（パース不能・`count == 0`・
//!   `step > count`・未知の `orientation` 値をすべて拒否する）。
//! - `progress` の百分率計算は u128 へ拡張してから行う（[`crate::steps::Steps`]
//!   PR #1941 codex-review P1 指摘と同型の対策。`step * 100` の usize
//!   オーバーフロー・折り返しを防ぐ）。
//!
//! # out-of-scope（イシュー #2117）
//!
//! - `fandhe-frontend-wasm-full` の `headless::MAPPING_TABLE` への
//!   `"questionnaire"` scope 登録（back/next/skip の click → dispatch 配線）
//!   は後続イシュー #2118 のスコープ。
//! - `fandhe-frontend-pre-styled-ui` のスタイル済み recipe・golden テスト・
//!   Themes ページは後続イシュー #2119 のスコープ。
//! - 回答値の保持・検証・分岐・送信ロジックはアプリケーション責務
//!   （責務境界規則 1）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_invalid, role};
use crate::data_attrs::{
    data_answered, data_complete, data_disabled, data_invalid, data_orientation, data_required,
    data_skipped, data_state, Orientation,
};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// Questionnaire の anatomy（`data-scope="questionnaire"`）。
const ANATOMY: Anatomy = anatomy("questionnaire");

/// `data-state` 属性値 "active"（`index == step` の question）。
const DATA_STATE_ACTIVE: &str = "active";
/// `data-state` 属性値 "completed"（`index < step` の question）。
const DATA_STATE_COMPLETED: &str = "completed";
/// `data-state` 属性値 "upcoming"（`index > step` の question）。
const DATA_STATE_UPCOMING: &str = "upcoming";

/// `root` パートが固定付与する属性名。
const ROOT_RESERVED: &[&str] = &["data-orientation", "data-step", "data-complete"];

/// `progress` パートが固定付与する属性名。
const PROGRESS_RESERVED: &[&str] = &[
    "role",
    "aria-valuemin",
    "aria-valuemax",
    "aria-valuenow",
    "aria-valuetext",
    "data-complete",
    "aria-label",
];

/// `question` パートが固定付与する属性名。
const QUESTION_RESERVED: &[&str] = &[
    "data-index",
    "data-state",
    "data-answered",
    "data-skipped",
    "data-required",
    "data-invalid",
    "aria-invalid",
    "hidden",
];

/// `back`/`next`/`skip` パートが固定付与する属性名。
const TRIGGER_RESERVED: &[&str] = &["type", "disabled", "data-disabled"];

/// 呼び出し側 `attrs` から予約キー（本モジュールが固定付与する属性名）を
/// 除去する（ASCII 大文字小文字無視の完全一致、[`crate::steps::drop_reserved`]
/// と同型、イシュー #2117）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// `count`/`step` を fail-closed に正規化する（[`crate::steps::normalize`]
/// と同型の契約）。
fn normalize(count: usize, step: usize) -> (usize, usize) {
    let count = count.max(1);
    let step = step.min(count);
    (count, step)
}

/// question のインデックスから見た 3 状態を表す（[`Questionnaire::question_state`]）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QuestionState {
    Completed,
    Active,
    Upcoming,
}

impl QuestionState {
    /// `data-state` 属性値。
    fn as_data_state(self) -> &'static str {
        match self {
            QuestionState::Completed => DATA_STATE_COMPLETED,
            QuestionState::Active => DATA_STATE_ACTIVE,
            QuestionState::Upcoming => DATA_STATE_UPCOMING,
        }
    }
}

/// [`Questionnaire::question`] へ渡す、単一質問の表示状態フラグ
/// （回答値そのものはアプリケーション責務のため持たない、責務境界規則 1）。
///
/// 全フィールドの既定値は `false`（[`Default`]）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct QuestionProps {
    /// 回答済みかどうか（`data-answered` 存在属性）。
    pub answered: bool,
    /// スキップ済みかどうか（`data-skipped` 存在属性）。「どの質問を
    /// スキップしたか」は呼び出し側が保持する（モジュール doc参照）。
    pub skipped: bool,
    /// 必須質問かどうか（`data-required` 存在属性）。
    pub required: bool,
    /// 入力値が不正かどうか（`data-invalid` 存在属性 + `aria-invalid="true"`）。
    pub invalid: bool,
}

/// Questionnaire の状態機械（[`crate::steps::Steps`] 同型）。
///
/// `step` は `0..=count` の範囲を取り、`step == count` は「全質問完了」を
/// 表す（[`Questionnaire::is_completed`]）。`Default` は `count=1, step=0`
/// （SSR の「未開始」初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Questionnaire {
    count: usize,
    step: usize,
    orientation: Orientation,
}

impl Default for Questionnaire {
    fn default() -> Self {
        Self::new(1, 0, Orientation::Horizontal)
    }
}

impl Questionnaire {
    /// `data-hydrate-count` 属性名のフィールド部分。
    pub const FIELD_COUNT: &'static str = "count";
    /// `data-hydrate-step` 属性名のフィールド部分。
    pub const FIELD_STEP: &'static str = "step";
    /// `data-hydrate-orientation` 属性名のフィールド部分。
    pub const FIELD_ORIENTATION: &'static str = "orientation";

    /// 指定した値で [`Questionnaire`] を生成する（`normalize` で
    /// fail-closed 正規化する。呼び出し側の不正な入力で panic しない）。
    #[must_use]
    pub fn new(count: usize, step: usize, orientation: Orientation) -> Self {
        let (count, step) = normalize(count, step);
        Self {
            count,
            step,
            orientation,
        }
    }

    /// 全質問数。
    #[must_use]
    pub fn count(&self) -> usize {
        self.count
    }

    /// 現在の質問位置（`0..=count`）。
    #[must_use]
    pub fn step(&self) -> usize {
        self.step
    }

    /// 現在の向き（`data-orientation`/hydration ラウンドトリップの対象）。
    #[must_use]
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// 全質問が完了しているか（`step == count`）。
    #[must_use]
    pub fn is_completed(&self) -> bool {
        self.step == self.count
    }

    /// 指定 `index`（`0..count`）の 3 状態を判定する。
    fn question_state(&self, index: usize) -> QuestionState {
        if index < self.step {
            QuestionState::Completed
        } else if index == self.step {
            QuestionState::Active
        } else {
            QuestionState::Upcoming
        }
    }

    /// Root パーツ（`div`）。`data-step` は動的な `usize` 値のため、
    /// [`Questionnaire::progress`] の `aria-valuenow` と同様にローカル
    /// `String` として保持してから借用する（`&'static str` を返せない
    /// ため `Box::leak` 等は用いない）。
    #[must_use]
    pub fn root<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let attrs = drop_reserved(attrs, ROOT_RESERVED);
        let step = self.step.to_string();
        let mut merged: Vec<(&str, &str)> = vec![
            data_orientation(self.orientation),
            ("data-step", step.as_str()),
        ];
        merged.extend(data_complete(self.is_completed()));
        merged.extend(attrs);
        ANATOMY.part("root", "div", merged, children)
    }

    /// Progress パーツ（`div`、`role="progressbar"`）。`percent`
    /// （`step * 100 / count` の整数、`0..=100`）を `aria-valuenow`/
    /// `aria-valuetext` へ出力する。`count >= 1` は `normalize` が保証
    /// するためゼロ除算は起きない。`data-complete` は `percent == 100`
    /// のときのみ付与する。`label` が空文字でないときのみ `aria-label`
    /// を付与する（shadcn「named progressbar」）。
    #[must_use]
    pub fn progress<'a>(
        &self,
        label: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let attrs = drop_reserved(attrs, PROGRESS_RESERVED);
        // `step`/`count` は usize のためオーバーフローの恐れがある乗算を
        // u128 へ拡張してから行う（`crate::steps::Steps::progress` と同型の
        // 対策、イシュー #1665 PR #1941 codex-review P1 指摘）。
        let percent = (self.step as u128 * 100 / self.count as u128) as usize;
        let now = percent.to_string();
        let text = format!("{percent}% complete");
        let mut merged: Vec<(&str, &str)> = vec![
            role("progressbar"),
            ("aria-valuemin", "0"),
            ("aria-valuemax", "100"),
        ];
        merged.extend(data_complete(percent == 100));
        merged.push(("aria-valuenow", now.as_str()));
        merged.push(("aria-valuetext", text.as_str()));
        if !label.is_empty() {
            merged.push(("aria-label", label));
        }
        merged.extend(attrs);
        ANATOMY.part("progress", "div", merged, children)
    }

    /// Question パーツ（`fieldset`、`index` は `0..count`）。`prompt`
    /// （[`Questionnaire::prompt`]、`legend`）を先頭子に置く呼び出し側契約
    /// とする（shadcn は fieldset/legend を採用。id 配管なしでネイティブな
    /// グループ名が付く）。非 active のとき `hidden` 属性で隠す
    /// （[`crate::steps::Steps::content`] と同型契約）。
    #[must_use]
    pub fn question<'a>(
        &self,
        index: usize,
        props: QuestionProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let attrs = drop_reserved(attrs, QUESTION_RESERVED);
        let state = self.question_state(index);
        let index_str = index.to_string();
        let mut merged: Vec<(&str, &str)> = vec![
            ("data-index", index_str.as_str()),
            data_state(state.as_data_state()),
        ];
        merged.extend(data_answered(props.answered));
        merged.extend(data_skipped(props.skipped));
        merged.extend(data_required(props.required));
        merged.extend(data_invalid(props.invalid));
        if props.invalid {
            merged.push(aria_invalid(true));
        }
        if state != QuestionState::Active {
            merged.push(("hidden", ""));
        }
        merged.extend(attrs);
        ANATOMY.part("question", "fieldset", merged, children)
    }

    /// Prompt パーツ（`legend`）。質問文のスロット。
    #[must_use]
    pub fn prompt<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        ANATOMY.part("prompt", "legend", attrs, children)
    }

    /// Description パーツ（`div`）。補足文のスロット。`aria-describedby`
    /// の関連付けは呼び出し側が `attrs` で `id` を渡し、[`Questionnaire::options`]
    /// 側の入れ子ルートへ `aria-describedby` を渡す契約とする。
    #[must_use]
    pub fn description<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        ANATOMY.part("description", "div", attrs, children)
    }

    /// Options パーツ（`div`）。純スロット。呼び出し側が
    /// [`crate::radio_group`]/[`crate::checkbox_group`] のパーツを入れ子に
    /// する（両 scope は独立して残る）。
    #[must_use]
    pub fn options<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        ANATOMY.part("options", "div", attrs, children)
    }

    /// Freeform パーツ（`div`）。純スロット。呼び出し側が
    /// [`crate::field::textarea`]（`field` パーツ群）を入れ子にする。
    #[must_use]
    pub fn freeform<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        ANATOMY.part("freeform", "div", attrs, children)
    }

    /// Actions パーツ（`div`）。back / next / skip を束ねるコンテナ。
    #[must_use]
    pub fn actions<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        ANATOMY.part("actions", "div", attrs, children)
    }

    /// Back パーツ（`button`）。`step == 0` のとき、または呼び出し側が
    /// 明示的に `disabled` を渡したときに `disabled` + `data-disabled`
    /// 属性を付与する。
    #[must_use]
    pub fn back<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let attrs = drop_reserved(attrs, TRIGGER_RESERVED);
        let disabled = disabled || self.step == 0;
        let mut merged: Vec<(&str, &str)> = vec![("type", "button")];
        if disabled {
            merged.push(("disabled", ""));
        }
        merged.extend(data_disabled(disabled));
        merged.extend(attrs);
        ANATOMY.part("back", "button", merged, children)
    }

    /// Next パーツ（`button`）。`step == count` のとき、または引数
    /// `disabled`（未回答かつ必須、といった判定結果をアプリが渡す）が
    /// `true` のときに `disabled` + `data-disabled` 属性を付与する
    /// （責務境界規則 1: 必須判定の結果〔真偽〕だけを受け取る）。
    #[must_use]
    pub fn next<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let attrs = drop_reserved(attrs, TRIGGER_RESERVED);
        let disabled = disabled || self.step == self.count;
        let mut merged: Vec<(&str, &str)> = vec![("type", "button")];
        if disabled {
            merged.push(("disabled", ""));
        }
        merged.extend(data_disabled(disabled));
        merged.extend(attrs);
        ANATOMY.part("next", "button", merged, children)
    }

    /// Skip パーツ（`button`）。`step == count` のとき、または引数
    /// `disabled`（スキップ不可＝必須、といった判定結果をアプリが渡す）が
    /// `true` のときに `disabled` + `data-disabled` 属性を付与する
    /// （[`Questionnaire::next`] と同型の責務境界）。
    #[must_use]
    pub fn skip<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let attrs = drop_reserved(attrs, TRIGGER_RESERVED);
        let disabled = disabled || self.step == self.count;
        let mut merged: Vec<(&str, &str)> = vec![("type", "button")];
        if disabled {
            merged.push(("disabled", ""));
        }
        merged.extend(data_disabled(disabled));
        merged.extend(attrs);
        ANATOMY.part("skip", "button", merged, children)
    }
}

/// Questionnaire のアクション（WASM 境界の文字列 dispatch と
/// [`Questionnaire::decode_action`] で接続する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestionnaireAction {
    /// 次の質問へ進む（`step == count` のときは no-op、`update` 内で
    /// `saturating_add(1).min(count)` に丸める。`step == count == usize::MAX`
    /// のときも `saturating_add` がオーバーフローを吸収するため panic しない）。
    Next,
    /// 前の質問へ戻る（`step == 0` のときは no-op、`update` 内で
    /// `saturating_sub(1)` に丸める）。
    Prev,
    /// 現在の質問をスキップして次へ進む。状態遷移としては [`Self::Next`]
    /// と同一（モジュール doc参照）。「どの質問をスキップしたか」は本
    /// 状態機械に保持しない。
    Skip,
    /// 指定 step へ直接移動する（`0..=count` の範囲内のみ有効）。
    Goto(usize),
}

impl Component for Questionnaire {
    type Action = QuestionnaireAction;

    /// `QuestionnaireAction::Goto` は範囲外（`> count`）を fail-closed に
    /// 無視する（no-op）。`normalize`/[`Questionnaire::decode_action`] が
    /// 課す「`step` は `0..=count`」という本モジュールの不変条件を
    /// `update()` 単体でも維持する（[`crate::steps::Steps::update`] と
    /// 同型の契約）。
    fn update(&mut self, action: QuestionnaireAction) {
        match action {
            QuestionnaireAction::Next | QuestionnaireAction::Skip => {
                self.step = self.step.saturating_add(1).min(self.count);
            }
            QuestionnaireAction::Prev => {
                self.step = self.step.saturating_sub(1);
            }
            QuestionnaireAction::Goto(step) => {
                if step <= self.count {
                    self.step = step;
                }
            }
        }
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > question(0)）。公開 UI としての利用は想定しない
    /// （実際の UI 構築は §パーツメソッド群を呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        self.root(
            Vec::new(),
            vec![self.question(0, QuestionProps::default(), Vec::new(), Vec::new())],
        )
    }

    /// `"next"`/`"prev"`/`"skip"`: payload 不使用。`"goto"`: payload を
    /// `str::parse::<usize>()` でパースし、パース不能な場合は `None`
    /// （fail-closed、dispatch は false）。範囲チェックは
    /// [`Questionnaire::update`] が担う（[`crate::steps::Steps::decode_action`]
    /// と同型の契約）。
    fn decode_action(name: &str, payload: &str) -> Option<QuestionnaireAction> {
        match name {
            "next" => Some(QuestionnaireAction::Next),
            "prev" => Some(QuestionnaireAction::Prev),
            "skip" => Some(QuestionnaireAction::Skip),
            "goto" => payload.parse::<usize>().ok().map(QuestionnaireAction::Goto),
            _ => None,
        }
    }
}

impl Hydrate for Questionnaire {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_COUNT),
                self.count.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STEP),
                self.step.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_ORIENTATION),
                self.orientation.as_str().to_string(),
            ),
        ]
    }

    /// クライアント改ざん入力として扱う。欠落は
    /// [`HydrateError::MissingAttr`]、パース不能・`count == 0`・
    /// `step > count`・未知の `orientation` 値は
    /// [`HydrateError::InvalidValue`]（panic しない、[`crate::steps::Steps`]
    /// と同型の契約）。
    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |field: &str| -> Result<&str, HydrateError> {
            let name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.as_str())
                .ok_or(HydrateError::MissingAttr(name))
        };

        let count_raw = find(Self::FIELD_COUNT)?;
        let step_raw = find(Self::FIELD_STEP)?;
        let orientation_raw = find(Self::FIELD_ORIENTATION)?;

        let attr_name_count = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_COUNT);
        let count = count_raw
            .parse::<usize>()
            .ok()
            .ok_or_else(|| HydrateError::InvalidValue {
                attr: attr_name_count.clone(),
                reason: "expected a non-negative integer".to_string(),
            })?;
        if count == 0 {
            return Err(HydrateError::InvalidValue {
                attr: attr_name_count,
                reason: "expected count >= 1".to_string(),
            });
        }

        let attr_name_step = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STEP);
        let step = step_raw
            .parse::<usize>()
            .ok()
            .ok_or_else(|| HydrateError::InvalidValue {
                attr: attr_name_step.clone(),
                reason: "expected a non-negative integer".to_string(),
            })?;
        if step > count {
            return Err(HydrateError::InvalidValue {
                attr: attr_name_step,
                reason: "expected step within [0, count]".to_string(),
            });
        }

        let attr_name_orientation = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_ORIENTATION);
        let orientation = match orientation_raw {
            "horizontal" => Orientation::Horizontal,
            "vertical" => Orientation::Vertical,
            _ => {
                return Err(HydrateError::InvalidValue {
                    attr: attr_name_orientation,
                    reason: "expected \"horizontal\" or \"vertical\"".to_string(),
                })
            }
        };

        Ok(Self {
            count,
            step,
            orientation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 正規化（fail-closed） ---

    #[test]
    fn new_clamps_zero_count_to_one() {
        let q = Questionnaire::new(0, 0, Orientation::Horizontal);
        assert_eq!(q.count(), 1);
        assert_eq!(q.step(), 0);
    }

    #[test]
    fn new_clamps_step_to_count() {
        let q = Questionnaire::new(3, 10, Orientation::Horizontal);
        assert_eq!(q.count(), 3);
        assert_eq!(q.step(), 3);
    }

    #[test]
    fn default_is_count_one_step_zero() {
        let q = Questionnaire::default();
        assert_eq!(q.count(), 1);
        assert_eq!(q.step(), 0);
        assert!(!q.is_completed());
    }

    // --- question 3 状態 ---

    #[test]
    fn question_state_reflects_index_vs_step() {
        let q = Questionnaire::new(3, 1, Orientation::Horizontal);
        let props = QuestionProps::default();
        assert!(render(&q.question(0, props, vec![], vec![])).contains(r#"data-state="completed""#));
        assert!(render(&q.question(1, props, vec![], vec![])).contains(r#"data-state="active""#));
        assert!(render(&q.question(2, props, vec![], vec![])).contains(r#"data-state="upcoming""#));
    }

    #[test]
    fn only_active_question_is_not_hidden() {
        let q = Questionnaire::new(3, 1, Orientation::Horizontal);
        let props = QuestionProps::default();
        assert!(render(&q.question(0, props, vec![], vec![])).contains("hidden"));
        assert!(!render(&q.question(1, props, vec![], vec![])).contains("hidden"));
        assert!(render(&q.question(2, props, vec![], vec![])).contains("hidden"));
    }

    #[test]
    fn is_completed_true_when_step_equals_count() {
        let q = Questionnaire::new(3, 3, Orientation::Horizontal);
        assert!(q.is_completed());
        let props = QuestionProps::default();
        for i in 0..3 {
            assert!(
                render(&q.question(i, props, vec![], vec![])).contains(r#"data-state="completed""#)
            );
        }
    }

    // --- QuestionProps 存在属性 ---

    #[test]
    fn question_props_output_existence_attrs() {
        let q = Questionnaire::new(3, 1, Orientation::Horizontal);
        let all_true = QuestionProps {
            answered: true,
            skipped: true,
            required: true,
            invalid: true,
        };
        let html = render(&q.question(1, all_true, vec![], vec![]));
        assert!(html.contains("data-answered"));
        assert!(html.contains("data-skipped"));
        assert!(html.contains("data-required"));
        assert!(html.contains("data-invalid"));
        assert!(html.contains(r#"aria-invalid="true""#));

        let all_false = QuestionProps::default();
        let html = render(&q.question(1, all_false, vec![], vec![]));
        assert!(!html.contains("data-answered"));
        assert!(!html.contains("data-skipped"));
        assert!(!html.contains("data-required"));
        assert!(!html.contains("data-invalid"));
        assert!(!html.contains("aria-invalid"));
    }

    #[test]
    fn question_has_data_index() {
        let q = Questionnaire::new(3, 0, Orientation::Horizontal);
        let html = render(&q.question(2, QuestionProps::default(), vec![], vec![]));
        assert!(html.contains(r#"data-index="2""#));
    }

    #[test]
    fn question_is_a_fieldset() {
        let q = Questionnaire::default();
        let html = render(&q.question(0, QuestionProps::default(), vec![], vec![]));
        assert!(html.starts_with("<fieldset"));
    }

    // --- anatomy / root / progress ---

    #[test]
    fn root_outputs_scope_part_orientation_step_and_complete() {
        let q = Questionnaire::new(3, 1, Orientation::Vertical);
        let html = render(&q.root(vec![], vec![]));
        assert!(html.contains(r#"data-scope="questionnaire""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-orientation="vertical""#));
        assert!(html.contains(r#"data-step="1""#));
        assert!(!html.contains("data-complete"));

        let done = Questionnaire::new(3, 3, Orientation::Horizontal);
        let done_html = render(&done.root(vec![], vec![]));
        assert!(done_html.contains("data-complete"));
    }

    #[test]
    fn progress_exposes_progressbar_semantics_and_percent() {
        let q = Questionnaire::new(3, 1, Orientation::Horizontal);
        let html = render(&q.progress("", vec![], vec![]));
        assert!(html.contains(r#"data-scope="questionnaire""#));
        assert!(html.contains(r#"data-part="progress""#));
        assert!(html.contains(r#"role="progressbar""#));
        assert!(html.contains(r#"aria-valuemin="0""#));
        assert!(html.contains(r#"aria-valuemax="100""#));
        assert!(html.contains(r#"aria-valuenow="33""#));
        assert!(html.contains(r#"aria-valuetext="33% complete""#));
        assert!(!html.contains("data-complete"));
        assert!(!html.contains("aria-label"));
    }

    #[test]
    fn progress_reflects_zero_and_full_percent() {
        let at_start = Questionnaire::new(3, 0, Orientation::Horizontal);
        let start_html = render(&at_start.progress("", vec![], vec![]));
        assert!(start_html.contains(r#"aria-valuenow="0""#));
        assert!(!start_html.contains("data-complete"));

        let done = Questionnaire::new(3, 3, Orientation::Horizontal);
        let done_html = render(&done.progress("", vec![], vec![]));
        assert!(done_html.contains(r#"aria-valuenow="100""#));
        assert!(done_html.contains("data-complete"));
    }

    #[test]
    fn progress_percent_does_not_overflow_for_large_count_and_step() {
        let max_step = usize::MAX;
        let q = Questionnaire::new(max_step, max_step, Orientation::Horizontal);
        let html = render(&q.progress("", vec![], vec![]));
        assert!(html.contains(r#"aria-valuenow="100""#));
        assert!(html.contains("data-complete"));

        let half = Questionnaire::new(usize::MAX, usize::MAX / 2, Orientation::Horizontal);
        let half_html = render(&half.progress("", vec![], vec![]));
        assert!(half_html.contains(r#"aria-valuenow="49""#));
        assert!(!half_html.contains("data-complete"));
    }

    #[test]
    fn progress_adds_aria_label_only_when_non_empty() {
        let q = Questionnaire::new(3, 1, Orientation::Horizontal);
        let html = render(&q.progress("Questionnaire progress", vec![], vec![]));
        assert!(html.contains(r#"aria-label="Questionnaire progress""#));
    }

    // --- prompt / description / options / freeform / actions（純スロット） ---

    #[test]
    fn slot_parts_output_scope_and_part_only() {
        let q = Questionnaire::default();
        let prompt_html = render(&q.prompt(vec![], vec![text("What is your name?")]));
        assert!(prompt_html.starts_with("<legend"));
        assert!(prompt_html.contains(r#"data-part="prompt""#));
        assert!(prompt_html.contains("What is your name?"));

        let description_html = render(&q.description(vec![], vec![]));
        assert!(description_html.contains(r#"data-part="description""#));

        let options_html = render(&q.options(vec![], vec![]));
        assert!(options_html.contains(r#"data-part="options""#));

        let freeform_html = render(&q.freeform(vec![], vec![]));
        assert!(freeform_html.contains(r#"data-part="freeform""#));

        let actions_html = render(&q.actions(vec![], vec![]));
        assert!(actions_html.contains(r#"data-part="actions""#));
    }

    // --- back / next / skip ---

    #[test]
    fn back_disabled_at_step_zero() {
        let at_start = Questionnaire::new(3, 0, Orientation::Horizontal);
        let html = render(&at_start.back(false, vec![], vec![]));
        assert!(html.contains("disabled"));
        assert!(html.contains("data-disabled"));

        let mid = Questionnaire::new(3, 1, Orientation::Horizontal);
        let mid_html = render(&mid.back(false, vec![], vec![]));
        assert!(!mid_html.contains("disabled"));
        assert!(!mid_html.contains("data-disabled"));
    }

    #[test]
    fn next_disabled_at_step_equals_count_or_by_argument() {
        let at_end = Questionnaire::new(3, 3, Orientation::Horizontal);
        let html = render(&at_end.next(false, vec![], vec![]));
        assert!(html.contains("disabled"));

        let mid = Questionnaire::new(3, 1, Orientation::Horizontal);
        let forced = render(&mid.next(true, vec![], vec![]));
        assert!(forced.contains("disabled"));
        assert!(forced.contains("data-disabled"));

        let free = render(&mid.next(false, vec![], vec![]));
        assert!(!free.contains("disabled"));
    }

    #[test]
    fn skip_disabled_at_step_equals_count_or_by_argument() {
        let at_end = Questionnaire::new(3, 3, Orientation::Horizontal);
        let html = render(&at_end.skip(false, vec![], vec![]));
        assert!(html.contains("disabled"));

        let mid = Questionnaire::new(3, 1, Orientation::Horizontal);
        let forced = render(&mid.skip(true, vec![], vec![]));
        assert!(forced.contains("disabled"));

        let free = render(&mid.skip(false, vec![], vec![]));
        assert!(!free.contains("disabled"));
    }

    #[test]
    fn triggers_are_type_button() {
        let q = Questionnaire::default();
        assert!(render(&q.back(false, vec![], vec![])).contains(r#"type="button""#));
        assert!(render(&q.next(false, vec![], vec![])).contains(r#"type="button""#));
        assert!(render(&q.skip(false, vec![], vec![])).contains(r#"type="button""#));
    }

    // --- Anatomy::part / drop_reserved fail-closed 回帰 ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let q = Questionnaire::default();
        let html = render(&q.root(
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="questionnaire""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn caller_cannot_spoof_reserved_attrs() {
        let q = Questionnaire::new(3, 1, Orientation::Horizontal);

        let root_html = render(&q.root(vec![("data-step", "attacker")], vec![]));
        assert_eq!(root_html.matches("data-step").count(), 1);
        assert!(root_html.contains(r#"data-step="1""#));

        let question_html = render(&q.question(
            1,
            QuestionProps::default(),
            vec![("data-state", "attacker")],
            vec![],
        ));
        assert_eq!(question_html.matches("data-state").count(), 1);
        assert!(question_html.contains(r#"data-state="active""#));

        let progress_html = render(&q.progress("", vec![("aria-valuenow", "attacker")], vec![]));
        assert_eq!(progress_html.matches("aria-valuenow").count(), 1);
        assert!(!progress_html.contains("attacker"));

        let mid = Questionnaire::new(3, 1, Orientation::Horizontal);
        let next_html = render(&mid.next(false, vec![("disabled", "attacker")], vec![]));
        assert!(!next_html.contains("attacker"));
        assert!(!next_html.contains("disabled"));
    }

    // --- dispatch 統合 ---

    #[test]
    fn dispatch_next_advances_and_stops_at_count() {
        let mut q = Questionnaire::new(3, 0, Orientation::Horizontal);
        assert!(dispatch(&mut q, "next", ""));
        assert_eq!(q.step(), 1);
        assert!(dispatch(&mut q, "next", ""));
        assert!(dispatch(&mut q, "next", ""));
        assert_eq!(q.step(), 3);
        assert!(dispatch(&mut q, "next", ""));
        assert_eq!(q.step(), 3);
    }

    #[test]
    fn dispatch_skip_advances_same_as_next() {
        let mut q = Questionnaire::new(3, 0, Orientation::Horizontal);
        assert!(dispatch(&mut q, "skip", ""));
        assert_eq!(q.step(), 1);
    }

    #[test]
    fn dispatch_prev_retreats_and_stops_at_zero() {
        let mut q = Questionnaire::new(3, 2, Orientation::Horizontal);
        assert!(dispatch(&mut q, "prev", ""));
        assert_eq!(q.step(), 1);
        assert!(dispatch(&mut q, "prev", ""));
        assert_eq!(q.step(), 0);
        assert!(dispatch(&mut q, "prev", ""));
        assert_eq!(q.step(), 0);
    }

    #[test]
    fn dispatch_goto_within_range() {
        let mut q = Questionnaire::new(5, 0, Orientation::Horizontal);
        assert!(dispatch(&mut q, "goto", "3"));
        assert_eq!(q.step(), 3);
        assert!(dispatch(&mut q, "goto", "0"));
        assert_eq!(q.step(), 0);
        assert!(dispatch(&mut q, "goto", "5"));
        assert_eq!(q.step(), 5);
    }

    #[test]
    fn dispatch_goto_rejects_invalid_payload() {
        let mut q = Questionnaire::new(3, 1, Orientation::Horizontal);
        for bogus in ["abc", "-1", ""] {
            assert!(!dispatch(&mut q, "goto", bogus));
            assert_eq!(q.step(), 1);
        }
    }

    #[test]
    fn dispatch_goto_out_of_range_is_recognized_but_no_op() {
        let mut q = Questionnaire::new(3, 1, Orientation::Horizontal);
        assert!(dispatch(&mut q, "goto", "4"));
        assert_eq!(q.step(), 1);
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut q = Questionnaire::new(3, 1, Orientation::Horizontal);
        assert!(!dispatch(&mut q, "no_such_action", "x"));
        assert_eq!(q.step(), 1);
    }

    #[test]
    fn update_rejects_out_of_range_goto_directly() {
        let mut q = Questionnaire::new(3, 1, Orientation::Horizontal);
        Component::update(&mut q, QuestionnaireAction::Goto(100));
        assert_eq!(q.step(), 1);
    }

    /// codex-review PR #2279 P1 回帰: `count == step == usize::MAX`
    /// （完了済み・最大値）の状態で `Next`/`Skip` を dispatch しても
    /// `step + 1` のオーバーフローで panic せず、公開契約どおり no-op
    /// （`step` は `count` のまま）であることを固定する。
    #[test]
    fn update_next_at_usize_max_does_not_overflow() {
        let mut q = Questionnaire::new(usize::MAX, usize::MAX, Orientation::Horizontal);
        Component::update(&mut q, QuestionnaireAction::Next);
        assert_eq!(q.step(), usize::MAX);
        assert_eq!(q.count(), usize::MAX);
    }

    #[test]
    fn update_skip_at_usize_max_does_not_overflow() {
        let mut q = Questionnaire::new(usize::MAX, usize::MAX, Orientation::Horizontal);
        Component::update(&mut q, QuestionnaireAction::Skip);
        assert_eq!(q.step(), usize::MAX);
        assert_eq!(q.count(), usize::MAX);
    }

    // --- SSR 状態なし初期描画 ---

    #[test]
    fn default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Questionnaire::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- hydration 経路 ---

    #[test]
    fn hydration_round_trip() {
        let q = Questionnaire::new(4, 2, Orientation::Vertical);
        let rendered = render(&render_for_hydration(&q));
        assert!(rendered.contains(r#"data-hydrate-count="4""#));
        assert!(rendered.contains(r#"data-hydrate-step="2""#));
        assert!(rendered.contains(r#"data-hydrate-orientation="vertical""#));

        let restored = Questionnaire::from_hydration_attrs(&q.hydration_attrs()).unwrap();
        assert_eq!(restored, q);
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Questionnaire::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-count".to_string())
        );
    }

    #[test]
    fn from_hydration_attrs_invalid_value_does_not_panic() {
        let bogus_sets: Vec<Vec<(String, String)>> = vec![
            vec![
                ("data-hydrate-count".to_string(), "abc".to_string()),
                ("data-hydrate-step".to_string(), "0".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            vec![
                ("data-hydrate-count".to_string(), "0".to_string()),
                ("data-hydrate-step".to_string(), "0".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            vec![
                ("data-hydrate-count".to_string(), "3".to_string()),
                ("data-hydrate-step".to_string(), "10".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            vec![
                ("data-hydrate-count".to_string(), "3".to_string()),
                (
                    "data-hydrate-step".to_string(),
                    "<script>alert(1)</script>".to_string(),
                ),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            vec![
                ("data-hydrate-count".to_string(), "3".to_string()),
                ("data-hydrate-step".to_string(), "1".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "diagonal".to_string(),
                ),
            ],
        ];
        for attrs in bogus_sets {
            let err = Questionnaire::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- render_for_hydration 経由時のみ data-hydrate- が出る ---

    #[test]
    fn direct_part_calls_do_not_emit_hydrate_attrs() {
        let q = Questionnaire::new(3, 1, Orientation::Horizontal);
        let html = render(&q.root(
            vec![],
            vec![q.question(1, QuestionProps::default(), vec![], vec![])],
        ));
        assert!(!html.contains("data-hydrate-"));
    }

    // --- XSS 回帰: 呼び出し側 attrs/children/label にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let q = Questionnaire::default();
        let html = render(&q.root(vec![("data-testid", ATTR_BREAK_PAYLOAD)], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let q = Questionnaire::default();
        let html = render(&q.prompt(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn progress_label_payload_is_escaped_on_render() {
        let q = Questionnaire::new(3, 1, Orientation::Horizontal);
        let html = render(&q.progress(ATTR_BREAK_PAYLOAD, vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }
}
