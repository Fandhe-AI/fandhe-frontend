//! Steps（段階ナビゲーション、ウィザード）headless コンポーネント
//! （イシュー #752、`docs/api/headless-ui-api.md` §4b.3 の保留解除）。
//!
//! ark-ui の Steps
//!（`.claude/skills/ark-ui/references/components/collections/steps.md`）を
//! 参考に、Root / List / Item / Trigger / Indicator / Separator / Content /
//! CompletedContent / PrevTrigger / NextTrigger / Progress の 11 anatomy
//! パーツ（Progress はイシュー #1665 で新設、§参照突合参照）と、
//! Phase 1（#524）の [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 抽象へ直接乗る段階状態機械
//! [`Steps`] を提供する。
//!
//! # `data-state` 語彙について（[`crate::state::Disclosure`]/[`crate::state::SingleSelect`] を使わない理由）
//!
//! [`crate::state::Disclosure`]/[`crate::state::SingleSelect`] は
//! `"open"/"closed"` や選択 ID という語彙に固定されている
//! （[`crate::state::OpenState`]）。Steps の item は `count`/`step` から
//! 導出する 3 状態（complete/current/incomplete）を持ち、いずれの既存語彙
//! にも収まらないため、[`crate::progress::Progress`]/[`crate::slider::Slider`]/
//! [`crate::rating_group::RatingGroup`] と同じ判断で、本モジュールも
//! [`fandhe_frontend_interactive::Component`]/[`fandhe_frontend_interactive::Hydrate`]
//! を直接実装し、Phase 1 が確立した dispatch 契約（未知アクション no-op）・
//! fail-closed hydration という**統合様式**にのみ準拠する。
//!
//! # 状態モデル
//!
//! [`Steps`] は `count`（全 step 数、`>= 1`）と `step`（現在位置、
//! `0..=count`）を持つ。`step == count` は「全 step 完了」を表す
//! （ark-ui の `isCompleted` 相当。[`Steps::is_completed`]）。item の
//! インデックス `index`（`0..count`）に対する 3 状態は:
//!
//! - `index < step` → complete
//! - `index == step` → current（`step == count` のときは該当する item が
//!   存在しないため current な item はない）
//! - `index > step` → incomplete
//!
//! # 呼び出し文脈
//!
//! SSR は [`Steps::new`] で値を正規化してから各パーツメソッド（[`Steps::root`]/
//! [`Steps::list`]/[`Steps::item`]/[`Steps::trigger`]/[`Steps::indicator`]/
//! [`Steps::separator`]/[`Steps::content`]/[`Steps::completed_content`]/
//! [`Steps::prev_trigger`]/[`Steps::next_trigger`]/[`Steps::progress`]）を
//! 呼んで組み立てる。CSR/hydration は [`Steps`] を経由し、dispatch
//! （`"next"`/`"prev"`/`"goto"`）で状態遷移する。`fandhe-frontend-pre-styled-ui`
//! （#546〜）が本モジュールを呼んでスタイル済み Steps を組み立てる想定である。
//!
//! # 参照突合（イシュー #1665）
//!
//! ark-ui の Steps・Zag.js の `steps` machine（`steps.connect.ts`/
//! `steps.anatomy.ts`）・chakra-ui の Steps を参照して is/isn't を洗い出した。
//!
//! **是正した差分（本イシューで加算、いずれもシグネチャ非破壊）**:
//!
//! - [`Steps::prev_trigger`]/[`Steps::next_trigger`]: 境界時に native
//!   `disabled` に加えて `data-disabled` を出力する（本リポジトリの
//!   disabled 語彙統一。`fandhe-frontend-pre-styled-ui` 側は既に
//!   `[data-disabled]` セレクタを登録済みで、本変更により活性化する）。
//! - [`Steps::trigger`]/[`Steps::content`]/[`Steps::completed_content`]:
//!   `data-orientation` を追加（Zag.js が trigger/content に出力する属性。
//!   completed_content は content と対称にするための加算）。
//! - [`Steps::progress`] パーツを新設（Zag.js anatomy 10 パーツ中、本実装
//!   に唯一欠けていたパート。`role="progressbar"` は a11y 意味論であり
//!   §3.25 規則 2 の装飾・レイアウト計測には該当しない）。
//! - 全パーツへ [`drop_reserved`]（[`crate::toolbar::drop_reserved`]・
//!   [`crate::nav_list::drop_reserved`] と同型）を導入し、呼び出し側
//!   `attrs` が固定付与属性へなりすませないようにした（A05 対策）。
//!
//! **意図的に合わせなかった差分（回帰ガードは `tests/steps_reference_parity.rs`）**:
//!
//! - trigger の `data-state="open"|"closed"` は非採用（既存の
//!   `complete`/`current`/`incomplete` を維持。変更は
//!   `fandhe-frontend-pre-styled-ui` の golden CSS を壊す破壊的変更になる）。
//! - `aria-current="step"` は item ではなく trigger のみに付与（フォーカス
//!   可能な要素への付与が支援技術に読まれやすく、両方に付けると重複読み
//!   上げになる）。
//! - list の `role="tablist"`/`aria-owns`/`aria-orientation`、trigger の
//!   `role="tab"`/`aria-selected`/`aria-controls`、content の
//!   `role="tabpanel"`/`aria-labelledby`/`tabindex="0"` は非採用（id 相互
//!   参照の配管が必要でシグネチャ変更＝破壊的変更になる。後続イシュー候補）。
//! - indicator の `aria-hidden="true"` は非採用（trigger の子が indicator
//!   のみの構成でアクセシブルネームを消してしまうため）。
//! - root の style `--percent`（装飾用 CSS 変数）・`dir`（RTL）は非採用
//!   （リポジトリ横断で未採用の判断軸に合わせる）。
//! - item の `data-skippable`・`linear` 時の roving tabIndex 制御・
//!   `isStepValid`/`isStepSkippable` は非採用（アプリケーションロジック
//!   依存、§out-of-scope 参照）。
//!
//! **wasm-full 未配線の事実**: `fandhe-frontend-wasm-full` の
//! `headless::MAPPING_TABLE` に `"steps"` scope は登録されておらず、
//! trigger/prev-trigger/next-trigger の click・キー操作は配線されていない
//! （ネイティブ `button` の Tab/Enter/Space のみが機能する。実配線は別
//! イシュー候補、§out-of-scope 参照）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`）はすべて `&'static str` リテラルで
//!   固定しており、動的値が属性名スロットへ混入する経路はない
//!   （[`crate::anatomy`]/[`crate::data_attrs`]/[`crate::aria`] の既存不変条件を
//!   そのまま継承する）。
//! - 動的値（数値属性・呼び出し側 `attrs`・children テキスト）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-state` 値語彙（`"complete"`/`"current"`/`"incomplete"`）は本
//!   モジュール内で一元管理し（[`Steps::item_state`]）、パーツ関数間で
//!   分裂させない。
//! - `aria-current="step"` は current な item の trigger のみに付与し、
//!   任意文字列を受け付けない列挙値（[`crate::aria::AriaCurrent::Step`]、
//!   Breadcrumb/Pagination と共有する [`crate::aria::aria_current`]）
//!   経由で出力する。
//! - hydration 属性（`data-hydrate-count`/`data-hydrate-step`）はクライアント
//!   側で改ざんされうる入力として扱う。[`Steps`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は panic せず
//!   `HydrateError` を返す（パース不能・`count == 0`・`step > count` を
//!   すべて拒否する）。
//!
//! # out-of-scope（イシュー #752）
//!
//! - `linear`（順序強制。前の step を完了しないと次へ進めない制約）は
//!   ark-ui のオプション機能であり、本実装は持たない（呼び出し側が
//!   `NextTrigger` の描画有無で表現する余地を残す）。
//! - `isStepValid`/`isStepSkippable`（step 単位のバリデーション/スキップ
//!   許可）はアプリケーション固有のロジックであり、本モジュールのスコープ外。
//! - キーボード操作・roving focus・クリックの実配線は `fandhe-frontend-wasm-full`
//!   の keynav 層の責務（本モジュールは属性・状態機械のみを提供する）。
//! - `fandhe-frontend-wasm-full` への `"steps"` scope 追加（trigger click →
//!   `goto`、prev/next click → `prev`/`next` の dispatch 配線）はイシュー
//!   #1665 でも見送り、後続イシュー候補とした（§参照突合参照）。
//! - tabs 意味論（`role="tablist"/"tab"/"tabpanel"` + id 配管）と chakra-ui
//!   `Steps.Title`/`Description`/`Status`/`Number` 相当は、破壊的シグネチャ
//!   変更・Themes 層の関心のいずれかを理由にイシュー #1665 でも非採用と
//!   した（§参照突合参照）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_current, aria_hidden, role, AriaCurrent};
use crate::data_attrs::{
    data_complete, data_current, data_disabled, data_incomplete, data_orientation, data_state,
    Orientation,
};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// Steps の anatomy（`data-scope="steps"`）。
const ANATOMY: Anatomy = anatomy("steps");

/// `data-state` 属性値 "complete"（`index < step` の item/trigger/indicator/separator）。
const DATA_STATE_COMPLETE: &str = "complete";
/// `data-state` 属性値 "current"（`index == step` の item/trigger/indicator/separator）。
const DATA_STATE_CURRENT: &str = "current";
/// `data-state` 属性値 "incomplete"（`index > step` の item/trigger/indicator/separator）。
const DATA_STATE_INCOMPLETE: &str = "incomplete";

/// Content パーツの `data-state` 値 "open"（現在 step の content）。
/// [`crate::state::OpenState`] を流用しない理由はモジュール doc を参照
/// （Steps は独自の値語彙 `complete`/`current`/`incomplete` を持つため、
/// content の開閉のみ別途 `open`/`closed` の 2 値を使う）。
const CONTENT_STATE_OPEN: &str = "open";
/// Content パーツの `data-state` 値 "closed"（非現在 step の content）。
const CONTENT_STATE_CLOSED: &str = "closed";

/// `trigger` パートが固定付与する属性名（呼び出し側 `attrs` からの
/// なりすまし・重複出力を防ぐ、[`crate::toolbar::drop_reserved`]・
/// [`crate::nav_list::drop_reserved`] と同型、イシュー #1665）。
const TRIGGER_RESERVED: &[&str] = &[
    "type",
    "data-state",
    "data-complete",
    "data-current",
    "data-incomplete",
    "data-orientation",
    "aria-current",
];

/// `content`/`completed-content` パートが固定付与する属性名。
const CONTENT_RESERVED: &[&str] = &["data-state", "data-orientation", "hidden"];

/// `indicator` パートが固定付与する属性名。
const INDICATOR_RESERVED: &[&str] = &[
    "data-state",
    "data-complete",
    "data-current",
    "data-incomplete",
];

/// `item` パートが固定付与する属性名。
const ITEM_RESERVED: &[&str] = &[
    "data-state",
    "data-orientation",
    "data-complete",
    "data-current",
    "data-incomplete",
];

/// `separator` パートが固定付与する属性名。
const SEPARATOR_RESERVED: &[&str] = &[
    "role",
    "aria-hidden",
    "data-state",
    "data-orientation",
    "data-complete",
    "data-current",
    "data-incomplete",
];

/// `root`/`list` パートが固定付与する属性名。
const ROOT_LIST_RESERVED: &[&str] = &["data-orientation"];

/// `prev-trigger`/`next-trigger` パートが固定付与する属性名。`disabled`/
/// `data-disabled` はここには含めない（呼び出し側がバリデーション結果に
/// 応じて Next を強制無効化する経路を残すため、状態機械が既に出力して
/// いる場合のみ [`Vec::extend`] 後の [`fandhe_frontend_core::render`] 側で
/// 重複したキーがそのまま両方出力される点に注意。ただし呼び出し側が
/// `disabled`/`data-disabled` を渡すのは境界の内側〔state 側が既に
/// 同じ属性を出す〕か外側〔state 側は出さない〕のいずれかであり、
/// 前者は同名属性が 2 回出るだけで意味論は変わらず、後者は呼び出し側の
/// 意図した強制無効化がそのまま反映される）。
const PREV_NEXT_RESERVED: &[&str] = &["type"];

/// `progress` パートが固定付与する属性名。
const PROGRESS_RESERVED: &[&str] = &[
    "role",
    "aria-valuemin",
    "aria-valuemax",
    "aria-valuenow",
    "aria-valuetext",
    "data-complete",
    "data-orientation",
];

/// 呼び出し側 `attrs` から予約キー（本モジュールが固定付与する属性名）を
/// 除去する（ASCII 大文字小文字無視の完全一致）。`fandhe_frontend_core::el`
/// は属性の重複除去をしないため、これを経由しない呼び出しは同名属性の
/// 重複出力・状態属性のなりすましを許してしまう
/// （[`crate::toolbar::drop_reserved`]・[`crate::nav_list::drop_reserved`]
/// と同型、イシュー #1665）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// item のインデックスから見た 3 状態を表す（[`Steps::item_state`]）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemState {
    Complete,
    Current,
    Incomplete,
}

impl ItemState {
    /// `data-state` 属性値。
    fn as_data_state(self) -> &'static str {
        match self {
            ItemState::Complete => DATA_STATE_COMPLETE,
            ItemState::Current => DATA_STATE_CURRENT,
            ItemState::Incomplete => DATA_STATE_INCOMPLETE,
        }
    }
}

/// `count`/`step` を fail-closed に正規化する。
///
/// - `count == 0` は無効な入力として `1` へフォールバックする（呼び出し側の
///   不正な入力で panic させない、ライブラリコードの panic 回避規約に従う
///   防御的実装）。
/// - `step` は正規化後の `count` に対して `0..=count` へ clamp する。
fn normalize(count: usize, step: usize) -> (usize, usize) {
    let count = count.max(1);
    let step = step.min(count);
    (count, step)
}

/// Steps の段階状態機械（ark-ui 準拠）。
///
/// `step` は `0..=count` の範囲を取り、`step == count` は「全 step 完了」を
/// 表す（[`Steps::is_completed`]）。`Default` は `count=1, step=0`
/// （SSR の「未開始」初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Steps {
    count: usize,
    step: usize,
    orientation: Orientation,
}

impl Default for Steps {
    fn default() -> Self {
        Self::new(1, 0, Orientation::Horizontal)
    }
}

impl Steps {
    /// `data-hydrate-count` 属性名のフィールド部分。
    pub const FIELD_COUNT: &'static str = "count";
    /// `data-hydrate-step` 属性名のフィールド部分。
    pub const FIELD_STEP: &'static str = "step";
    /// `data-hydrate-orientation` 属性名のフィールド部分。
    pub const FIELD_ORIENTATION: &'static str = "orientation";

    /// 指定した値で [`Steps`] を生成する（[`normalize`] で fail-closed
    /// 正規化する。呼び出し側の不正な入力で panic しない）。
    #[must_use]
    pub fn new(count: usize, step: usize, orientation: Orientation) -> Self {
        let (count, step) = normalize(count, step);
        Self {
            count,
            step,
            orientation,
        }
    }

    /// 全 step 数。
    #[must_use]
    pub fn count(&self) -> usize {
        self.count
    }

    /// 現在の step（`0..=count`）。
    #[must_use]
    pub fn step(&self) -> usize {
        self.step
    }

    /// 現在の向き（`data-orientation`/hydration ラウンドトリップの対象）。
    #[must_use]
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// 全 step が完了しているか（`step == count`、ark-ui の `isCompleted` 相当）。
    #[must_use]
    pub fn is_completed(&self) -> bool {
        self.step == self.count
    }

    /// 指定 `index`（`0..count`）の 3 状態を判定する。
    fn item_state(&self, index: usize) -> ItemState {
        if index < self.step {
            ItemState::Complete
        } else if index == self.step {
            ItemState::Current
        } else {
            ItemState::Incomplete
        }
    }

    /// Root パーツ（`div`）。
    #[must_use]
    pub fn root<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let attrs = drop_reserved(attrs, ROOT_LIST_RESERVED);
        let mut merged: Vec<(&'a str, &'a str)> = vec![data_orientation(self.orientation)];
        merged.extend(attrs);
        ANATOMY.part("root", "div", merged, children)
    }

    /// List パーツ（`ol`。ark-ui は順序付きリストで item を並べる）。
    #[must_use]
    pub fn list<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let attrs = drop_reserved(attrs, ROOT_LIST_RESERVED);
        let mut merged: Vec<(&'a str, &'a str)> = vec![data_orientation(self.orientation)];
        merged.extend(attrs);
        ANATOMY.part("list", "ol", merged, children)
    }

    /// Item パーツ（`li`、`index` は `0..count`）。`data-orientation` も
    /// 併せて付与する（イシュー #752 pre-styled-ui レビュー指摘対応。
    /// `list`/`root` の `data-orientation` だけでは pre-styled-ui 側の
    /// `SlotRecipe`（`[data-part="item"]` 自身の属性のみを条件化できる）
    /// から垂直方向レイアウトを判定できないため、`separator` と同様に
    /// item 自身へも複製する）。
    #[must_use]
    pub fn item<'a>(
        &self,
        index: usize,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let attrs = drop_reserved(attrs, ITEM_RESERVED);
        let state = self.item_state(index);
        let mut merged: Vec<(&'a str, &'a str)> = vec![
            data_state(state.as_data_state()),
            data_orientation(self.orientation),
        ];
        merged.extend(data_complete(state == ItemState::Complete));
        merged.extend(data_current(state == ItemState::Current));
        merged.extend(data_incomplete(state == ItemState::Incomplete));
        merged.extend(attrs);
        ANATOMY.part("item", "li", merged, children)
    }

    /// Trigger パーツ（`button`、`index` は `0..count`）。current な item の
    /// trigger のみ `aria-current="step"` を付与する。`data-orientation` を
    /// 併せて付与する（イシュー #1665 参照突合。Zag.js の trigger 出力に
    /// 合わせた加算。呼び出し側 CSS が単独パートのみでレイアウト条件化
    /// できるようにする）。
    #[must_use]
    pub fn trigger<'a>(
        &self,
        index: usize,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let attrs = drop_reserved(attrs, TRIGGER_RESERVED);
        let state = self.item_state(index);
        let mut merged: Vec<(&'a str, &'a str)> = vec![
            ("type", "button"),
            data_state(state.as_data_state()),
            data_orientation(self.orientation),
        ];
        merged.extend(data_complete(state == ItemState::Complete));
        merged.extend(data_current(state == ItemState::Current));
        merged.extend(data_incomplete(state == ItemState::Incomplete));
        if state == ItemState::Current {
            merged.push(aria_current(AriaCurrent::Step));
        }
        merged.extend(attrs);
        ANATOMY.part("trigger", "button", merged, children)
    }

    /// Indicator パーツ（`div`、`index` は `0..count`）。参照側
    /// （Zag.js/ark-ui）の `aria-hidden` は意図的に付与しない
    /// （イシュー #1665 参照突合。trigger の子が indicator（数字）のみの
    /// 構成が多く、無条件付与は trigger のアクセシブルネームを消す。
    /// 必要な利用者は `attrs` で自前付与できる）。
    #[must_use]
    pub fn indicator<'a>(
        &self,
        index: usize,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let attrs = drop_reserved(attrs, INDICATOR_RESERVED);
        let state = self.item_state(index);
        let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
        merged.extend(data_complete(state == ItemState::Complete));
        merged.extend(data_current(state == ItemState::Current));
        merged.extend(data_incomplete(state == ItemState::Incomplete));
        merged.extend(attrs);
        ANATOMY.part("indicator", "div", merged, children)
    }

    /// Separator パーツ（`div`、装飾用。`index` は `0..count`、item 間の
    /// 区切り線を表す。`role="separator"` + `aria-hidden` で a11y ツリーから
    /// 除外する）。
    #[must_use]
    pub fn separator<'a>(
        &self,
        index: usize,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let attrs = drop_reserved(attrs, SEPARATOR_RESERVED);
        let state = self.item_state(index);
        let mut merged: Vec<(&'a str, &'a str)> = vec![
            role("separator"),
            aria_hidden(true),
            data_state(state.as_data_state()),
            data_orientation(self.orientation),
        ];
        merged.extend(data_complete(state == ItemState::Complete));
        merged.extend(data_current(state == ItemState::Current));
        merged.extend(data_incomplete(state == ItemState::Incomplete));
        merged.extend(attrs);
        ANATOMY.part("separator", "div", merged, children)
    }

    /// Content パーツ（`div`、`index` は `0..count`）。現在 step のみ
    /// `data-state="open"` で表示し、非現在 step は `data-state="closed"` +
    /// `hidden` 属性で隠す（[`crate::tabs`] の content と同型の契約）。
    /// `data-orientation` を併せて付与する（イシュー #1665 参照突合、
    /// [`Steps::trigger`] と同じ理由）。
    #[must_use]
    pub fn content<'a>(
        &self,
        index: usize,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let attrs = drop_reserved(attrs, CONTENT_RESERVED);
        let is_open = index == self.step;
        let mut merged: Vec<(&'a str, &'a str)> = vec![
            data_state(if is_open {
                CONTENT_STATE_OPEN
            } else {
                CONTENT_STATE_CLOSED
            }),
            data_orientation(self.orientation),
        ];
        if !is_open {
            merged.push(("hidden", ""));
        }
        merged.extend(attrs);
        ANATOMY.part("content", "div", merged, children)
    }

    /// CompletedContent パーツ（`div`）。[`Steps::is_completed`] が
    /// `true`（`step == count`）のときのみ表示し、それ以外は `hidden`
    /// 属性で隠す。`data-orientation` を併せて付与する（イシュー #1665
    /// 参照突合、[`Steps::content`] と対称にするための加算。content と
    /// 同型契約を保つ）。
    #[must_use]
    pub fn completed_content<'a>(
        &self,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let attrs = drop_reserved(attrs, CONTENT_RESERVED);
        let is_open = self.is_completed();
        let mut merged: Vec<(&'a str, &'a str)> = vec![
            data_state(if is_open {
                CONTENT_STATE_OPEN
            } else {
                CONTENT_STATE_CLOSED
            }),
            data_orientation(self.orientation),
        ];
        if !is_open {
            merged.push(("hidden", ""));
        }
        merged.extend(attrs);
        ANATOMY.part("completed-content", "div", merged, children)
    }

    /// PrevTrigger パーツ（`button`）。`step == 0` のとき `disabled` +
    /// `data-disabled` 属性を付与する（呼び出し側が無効化描画を自前で
    /// 判断しなくてよいように、状態機械側で境界を一元管理する）。
    /// `data-disabled` はイシュー #1665 参照突合での加算（本リポジトリの
    /// disabled 語彙統一。`fandhe-frontend-pre-styled-ui` 側は既に
    /// `[data-disabled]` セレクタを登録済みで、本変更により活性化する）。
    /// `disabled`/`data-disabled` は予約キーに含めない（呼び出し側が
    /// バリデーション結果に応じて Next を強制無効化する経路を残す。
    /// [`PREV_NEXT_RESERVED`] の doc コメント参照）。
    #[must_use]
    pub fn prev_trigger<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let attrs = drop_reserved(attrs, PREV_NEXT_RESERVED);
        let disabled = self.step == 0;
        let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
        if disabled {
            merged.push(("disabled", ""));
        }
        merged.extend(data_disabled(disabled));
        merged.extend(attrs);
        ANATOMY.part("prev-trigger", "button", merged, children)
    }

    /// NextTrigger パーツ（`button`）。`step == count` のとき `disabled` +
    /// `data-disabled` 属性を付与する（[`Steps::prev_trigger`] と同型の
    /// 境界一元管理・イシュー #1665 参照突合）。
    #[must_use]
    pub fn next_trigger<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let attrs = drop_reserved(attrs, PREV_NEXT_RESERVED);
        let disabled = self.step == self.count;
        let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
        if disabled {
            merged.push(("disabled", ""));
        }
        merged.extend(data_disabled(disabled));
        merged.extend(attrs);
        ANATOMY.part("next-trigger", "button", merged, children)
    }

    /// Progress パーツ（`div`、`role="progressbar"`）。ark-ui/Zag.js の
    /// anatomy に存在し本実装が欠いていた唯一のパート（イシュー #1665
    /// 参照突合で新設）。`percent`（`step * 100 / count` の整数、`0..=100`）
    /// を `aria-valuenow`/`aria-valuetext` へ出力する。`count >= 1` は
    /// [`normalize`] が保証するためゼロ除算は起きない。`data-complete` は
    /// `percent == 100`（`step == count`）のときのみ付与し、
    /// `data-orientation` は本実装の superset（Zag/ark-ui の progress は
    /// `data-complete` のみを持つ）として付与する。
    #[must_use]
    pub fn progress<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let attrs = drop_reserved(attrs, PROGRESS_RESERVED);
        // `step`/`count` は usize のためオーバーフローの恐れがある乗算を
        // u128 へ拡張してから行う（`step > usize::MAX / 100` で usize の
        // まま乗算すると debug では panic、release では折り返して誤った
        // 割合になる。イシュー #1665 PR #1941 codex-review P1 指摘）。
        // `count >= 1` は `normalize` が保証するためゼロ除算は起きない。
        let percent = (self.step as u128 * 100 / self.count as u128) as usize;
        let now = percent.to_string();
        let text = format!("{percent}% complete");
        let mut merged: Vec<(&str, &str)> = vec![
            role("progressbar"),
            ("aria-valuemin", "0"),
            ("aria-valuemax", "100"),
            data_orientation(self.orientation),
        ];
        merged.extend(data_complete(percent == 100));
        merged.push(("aria-valuenow", now.as_str()));
        merged.push(("aria-valuetext", text.as_str()));
        merged.extend(attrs);
        ANATOMY.part("progress", "div", merged, children)
    }
}

/// Steps のアクション（WASM 境界の文字列 dispatch と
/// [`Steps::decode_action`] で接続する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepsAction {
    /// 次の step へ進む（`step == count` のときは no-op、`update` 内で
    /// `min(step + 1, count)` に丸める）。
    Next,
    /// 前の step へ戻る（`step == 0` のときは no-op、`update` 内で
    /// `saturating_sub(1)` に丸める）。
    Prev,
    /// 指定 step へ直接移動する（`0..=count` の範囲内のみ有効）。
    Goto(usize),
}

impl Component for Steps {
    type Action = StepsAction;

    /// `StepsAction::Goto` は範囲外（`> count`）を fail-closed に無視する
    /// （no-op）。[`normalize`]/[`Steps::decode_action`] が課す
    /// 「`step` は `0..=count`」という本モジュールの不変条件を `update()`
    /// 単体でも維持するため（`decode_action` を経由しない直接
    /// `StepsAction::Goto` 構築・呼び出しからも同じ不変条件を守る）。
    fn update(&mut self, action: StepsAction) {
        match action {
            StepsAction::Next => {
                self.step = (self.step + 1).min(self.count);
            }
            StepsAction::Prev => {
                self.step = self.step.saturating_sub(1);
            }
            StepsAction::Goto(step) => {
                if step <= self.count {
                    self.step = step;
                }
            }
        }
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > list > item(0)）。公開 UI としての利用は想定しない
    /// （実際の UI 構築は §パーツメソッド群を呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        self.root(
            Vec::new(),
            vec![self.list(Vec::new(), vec![self.item(0, Vec::new(), Vec::new())])],
        )
    }

    /// `"next"`/`"prev"`: payload 不使用。`"goto"`: payload を
    /// `str::parse::<usize>()` でパースし、パース不能な場合は `None`
    /// （fail-closed、dispatch は false）。本メソッドは `&self` を取らない
    /// 静的メソッドのため `count` を参照できず、範囲外の数値（`count` 超過）
    /// はここでは弾けない。範囲チェックは [`Steps::update`] が担い、
    /// 範囲外の `Goto` は no-op にする（`dispatch` 自体は「認識された
    /// アクション」として true を返すが、状態は変わらない。
    /// [`crate::progress::Progress`] の `"set"` clamp と同型の契約）。
    fn decode_action(name: &str, payload: &str) -> Option<StepsAction> {
        match name {
            "next" => Some(StepsAction::Next),
            "prev" => Some(StepsAction::Prev),
            "goto" => payload.parse::<usize>().ok().map(StepsAction::Goto),
            _ => None,
        }
    }
}

impl Hydrate for Steps {
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
    /// [`HydrateError::InvalidValue`]（panic しない）。
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
        let s = Steps::new(0, 0, Orientation::Horizontal);
        assert_eq!(s.count(), 1);
        assert_eq!(s.step(), 0);
    }

    #[test]
    fn new_clamps_step_to_count() {
        let s = Steps::new(3, 10, Orientation::Horizontal);
        assert_eq!(s.count(), 3);
        assert_eq!(s.step(), 3);
    }

    #[test]
    fn default_is_count_one_step_zero() {
        let s = Steps::default();
        assert_eq!(s.count(), 1);
        assert_eq!(s.step(), 0);
        assert!(!s.is_completed());
    }

    // --- item 3 状態 ---

    #[test]
    fn item_state_reflects_index_vs_step() {
        let s = Steps::new(3, 1, Orientation::Horizontal);
        assert!(render(&s.item(0, vec![], vec![])).contains(r#"data-state="complete""#));
        assert!(render(&s.item(1, vec![], vec![])).contains(r#"data-state="current""#));
        assert!(render(&s.item(2, vec![], vec![])).contains(r#"data-state="incomplete""#));
    }

    #[test]
    fn item_existence_attrs_match_data_state() {
        let s = Steps::new(3, 1, Orientation::Horizontal);
        let complete_html = render(&s.item(0, vec![], vec![]));
        assert!(complete_html.contains("data-complete"));
        assert!(!complete_html.contains("data-current"));
        assert!(!complete_html.contains("data-incomplete"));

        let current_html = render(&s.item(1, vec![], vec![]));
        assert!(!current_html.contains("data-complete"));
        assert!(current_html.contains("data-current"));
        assert!(!current_html.contains("data-incomplete"));

        let incomplete_html = render(&s.item(2, vec![], vec![]));
        assert!(!incomplete_html.contains("data-complete"));
        assert!(!incomplete_html.contains("data-current"));
        assert!(incomplete_html.contains("data-incomplete"));
    }

    #[test]
    fn is_completed_true_when_step_equals_count() {
        let s = Steps::new(3, 3, Orientation::Horizontal);
        assert!(s.is_completed());
        // step == count のとき全 item が complete（current な item は存在しない）。
        for i in 0..3 {
            assert!(render(&s.item(i, vec![], vec![])).contains(r#"data-state="complete""#));
        }
    }

    // --- anatomy / ARIA / data-* ---

    #[test]
    fn root_and_list_output_scope_part_and_orientation() {
        let s = Steps::new(3, 1, Orientation::Vertical);
        let root_html = render(&s.root(vec![], vec![]));
        assert!(root_html.contains(r#"data-scope="steps""#));
        assert!(root_html.contains(r#"data-part="root""#));
        assert!(root_html.contains(r#"data-orientation="vertical""#));

        let list_html = render(&s.list(vec![], vec![]));
        assert!(list_html.starts_with("<ol"));
        assert!(list_html.contains(r#"data-part="list""#));
        assert!(list_html.contains(r#"data-orientation="vertical""#));
    }

    #[test]
    fn trigger_has_aria_current_step_only_when_current() {
        let s = Steps::new(3, 1, Orientation::Horizontal);
        let current_html = render(&s.trigger(1, vec![], vec![]));
        assert!(current_html.contains(r#"aria-current="step""#));

        let complete_html = render(&s.trigger(0, vec![], vec![]));
        assert!(!complete_html.contains("aria-current"));

        let incomplete_html = render(&s.trigger(2, vec![], vec![]));
        assert!(!incomplete_html.contains("aria-current"));
    }

    #[test]
    fn trigger_is_a_button_with_type_button() {
        let s = Steps::default();
        let html = render(&s.trigger(0, vec![], vec![]));
        assert!(html.starts_with("<button"));
        assert!(html.contains(r#"type="button""#));
    }

    #[test]
    fn trigger_has_data_orientation() {
        // イシュー #1665 参照突合: Zag.js の trigger 出力に合わせて
        // `data-orientation` を加算する。
        let s = Steps::new(3, 1, Orientation::Vertical);
        let html = render(&s.trigger(1, vec![], vec![]));
        assert!(html.contains(r#"data-orientation="vertical""#));
    }

    #[test]
    fn indicator_outputs_scope_part_and_state() {
        let s = Steps::new(3, 1, Orientation::Horizontal);
        let html = render(&s.indicator(1, vec![], vec![]));
        assert!(html.contains(r#"data-scope="steps""#));
        assert!(html.contains(r#"data-part="indicator""#));
        assert!(html.contains(r#"data-state="current""#));
        assert!(html.contains("data-current"));
    }

    #[test]
    fn separator_has_role_separator_and_aria_hidden() {
        let s = Steps::new(3, 1, Orientation::Vertical);
        let html = render(&s.separator(0, vec![], vec![]));
        assert!(html.contains(r#"role="separator""#));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(html.contains(r#"data-orientation="vertical""#));
        assert!(html.contains("data-complete"));
    }

    #[test]
    fn content_open_for_current_step_only() {
        let s = Steps::new(3, 1, Orientation::Vertical);
        let current_html = render(&s.content(1, vec![], vec![text("body")]));
        assert!(current_html.contains(r#"data-state="open""#));
        assert!(!current_html.contains("hidden"));
        assert!(current_html.contains(r#"data-orientation="vertical""#));

        let other_html = render(&s.content(0, vec![], vec![text("body")]));
        assert!(other_html.contains(r#"data-state="closed""#));
        assert!(other_html.contains("hidden"));
        assert!(other_html.contains(r#"data-orientation="vertical""#));
    }

    #[test]
    fn completed_content_open_only_when_all_steps_done() {
        let not_done = Steps::new(3, 1, Orientation::Vertical);
        let html = render(&not_done.completed_content(vec![], vec![]));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(html.contains("hidden"));
        assert!(html.contains(r#"data-orientation="vertical""#));

        let done = Steps::new(3, 3, Orientation::Horizontal);
        let html = render(&done.completed_content(vec![], vec![]));
        assert!(html.contains(r#"data-state="open""#));
        assert!(!html.contains("hidden"));
    }

    #[test]
    fn prev_trigger_disabled_at_step_zero() {
        // イシュー #1665 参照突合: 境界時に native `disabled` に加えて
        // `data-disabled` も出力する（本リポジトリの disabled 語彙統一。
        // `fandhe-frontend-pre-styled-ui` 側は既に `[data-disabled]`
        // セレクタを登録済みで、本変更により活性化する）。
        let at_start = Steps::new(3, 0, Orientation::Horizontal);
        let start_html = render(&at_start.prev_trigger(vec![], vec![]));
        assert!(start_html.contains("disabled"));
        assert!(start_html.contains("data-disabled"));

        let mid = Steps::new(3, 1, Orientation::Horizontal);
        let mid_html = render(&mid.prev_trigger(vec![], vec![]));
        assert!(!mid_html.contains("disabled"));
        assert!(!mid_html.contains("data-disabled"));
    }

    #[test]
    fn next_trigger_disabled_at_step_equals_count() {
        let at_end = Steps::new(3, 3, Orientation::Horizontal);
        let end_html = render(&at_end.next_trigger(vec![], vec![]));
        assert!(end_html.contains("disabled"));
        assert!(end_html.contains("data-disabled"));

        let mid = Steps::new(3, 1, Orientation::Horizontal);
        let mid_html = render(&mid.next_trigger(vec![], vec![]));
        assert!(!mid_html.contains("disabled"));
        assert!(!mid_html.contains("data-disabled"));
    }

    // --- Progress パーツ（イシュー #1665 新設） ---

    #[test]
    fn progress_exposes_progressbar_semantics_and_percent() {
        let s = Steps::new(3, 1, Orientation::Horizontal);
        let html = render(&s.progress(vec![], vec![]));
        assert!(html.contains(r#"data-scope="steps""#));
        assert!(html.contains(r#"data-part="progress""#));
        assert!(html.contains(r#"role="progressbar""#));
        assert!(html.contains(r#"aria-valuemin="0""#));
        assert!(html.contains(r#"aria-valuemax="100""#));
        assert!(html.contains(r#"aria-valuenow="33""#));
        assert!(html.contains(r#"aria-valuetext="33% complete""#));
        assert!(!html.contains("data-complete"));
    }

    #[test]
    fn progress_reflects_zero_and_full_percent() {
        let at_start = Steps::new(3, 0, Orientation::Horizontal);
        let start_html = render(&at_start.progress(vec![], vec![]));
        assert!(start_html.contains(r#"aria-valuenow="0""#));
        assert!(!start_html.contains("data-complete"));

        let done = Steps::new(3, 3, Orientation::Horizontal);
        let done_html = render(&done.progress(vec![], vec![]));
        assert!(done_html.contains(r#"aria-valuenow="100""#));
        assert!(done_html.contains("data-complete"));
    }

    #[test]
    fn progress_does_not_add_hydration_attrs() {
        // 状態機械の hydration 属性契約（[`Steps::hydration_attrs`]）を
        // progress パーツの追加が拡張しないことの回帰。
        let s = Steps::new(4, 2, Orientation::Vertical);
        assert_eq!(s.hydration_attrs().len(), 3);
        let _ = render(&s.progress(vec![], vec![]));
        assert_eq!(s.hydration_attrs().len(), 3);
    }

    #[test]
    fn progress_percent_does_not_overflow_for_large_count_and_step() {
        // イシュー #1665 PR #1941 codex-review P1 回帰: `step > usize::MAX / 100`
        // では `step * 100` が usize のまま計算するとオーバーフローする
        // （debug では panic、release では折り返して誤った割合になる）。
        // u128 へ拡張した計算が全入力域で正しい割合を返すことを確認する。
        let max_step = usize::MAX;
        let s = Steps::new(max_step, max_step, Orientation::Horizontal);
        let html = render(&s.progress(vec![], vec![]));
        assert!(html.contains(r#"aria-valuenow="100""#));
        assert!(html.contains(r#"aria-valuetext="100% complete""#));
        assert!(html.contains("data-complete"));

        // step が count 未満でも usize::MAX 近傍で正しい割合（50%）になる。
        let half = Steps::new(usize::MAX, usize::MAX / 2, Orientation::Horizontal);
        let half_html = render(&half.progress(vec![], vec![]));
        assert!(half_html.contains(r#"aria-valuenow="49""#));
        assert!(!half_html.contains("data-complete"));
    }

    // --- Anatomy::part fail-closed 回帰 ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let s = Steps::default();
        let html = render(&s.root(
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="steps""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn caller_cannot_spoof_reserved_attrs() {
        // イシュー #1665: `drop_reserved`（toolbar/nav_list と同型）を
        // 導入し、呼び出し側 `attrs` が固定付与属性へなりすませないことを
        // 各パートで固定する（ASCII 大小無視の完全一致除去）。
        let s = Steps::new(3, 1, Orientation::Horizontal);

        let trigger_html = render(&s.trigger(1, vec![("data-state", "attacker")], vec![]));
        assert_eq!(trigger_html.matches("data-state").count(), 1);
        assert!(trigger_html.contains(r#"data-state="current""#));

        let content_html = render(&s.content(1, vec![("DATA-ORIENTATION", "attacker")], vec![]));
        assert_eq!(content_html.matches("data-orientation").count(), 1);
        assert!(content_html.contains(r#"data-orientation="horizontal""#));

        let indicator_html = render(&s.indicator(1, vec![("data-complete", "attacker")], vec![]));
        assert!(!indicator_html.contains("attacker"));

        let separator_html = render(&s.separator(1, vec![("role", "attacker")], vec![]));
        assert_eq!(separator_html.matches("role=").count(), 1);
        assert!(separator_html.contains(r#"role="separator""#));

        let progress_html = render(&s.progress(vec![("aria-valuenow", "attacker")], vec![]));
        assert_eq!(progress_html.matches("aria-valuenow").count(), 1);
        assert!(!progress_html.contains("attacker"));

        // prev-trigger/next-trigger の `disabled`/`data-disabled` は予約
        // しない（呼び出し側がバリデーション結果で Next を強制無効化する
        // 経路を残す設計。境界外〔mid〕でも呼び出し側が明示付与すれば
        // そのまま出力される）。
        let mid = Steps::new(3, 1, Orientation::Horizontal);
        let forced_html = render(&mid.next_trigger(vec![("disabled", "")], vec![]));
        assert!(forced_html.contains("disabled"));
    }

    // --- Steps: dispatch 統合 ---

    #[test]
    fn dispatch_next_advances_and_stops_at_count() {
        let mut s = Steps::new(3, 0, Orientation::Horizontal);
        assert!(dispatch(&mut s, "next", ""));
        assert_eq!(s.step(), 1);
        assert!(dispatch(&mut s, "next", ""));
        assert!(dispatch(&mut s, "next", ""));
        assert_eq!(s.step(), 3);
        // count に達したあとも呼べる（no-op のまま 3 に留まる）が、
        // dispatch 自体は「認識されたアクション」として true を返す。
        assert!(dispatch(&mut s, "next", ""));
        assert_eq!(s.step(), 3);
    }

    #[test]
    fn dispatch_prev_retreats_and_stops_at_zero() {
        let mut s = Steps::new(3, 2, Orientation::Horizontal);
        assert!(dispatch(&mut s, "prev", ""));
        assert_eq!(s.step(), 1);
        assert!(dispatch(&mut s, "prev", ""));
        assert_eq!(s.step(), 0);
        assert!(dispatch(&mut s, "prev", ""));
        assert_eq!(s.step(), 0);
    }

    #[test]
    fn dispatch_goto_within_range() {
        let mut s = Steps::new(5, 0, Orientation::Horizontal);
        assert!(dispatch(&mut s, "goto", "3"));
        assert_eq!(s.step(), 3);
        assert!(dispatch(&mut s, "goto", "0"));
        assert_eq!(s.step(), 0);
        assert!(dispatch(&mut s, "goto", "5"));
        assert_eq!(s.step(), 5);
    }

    #[test]
    fn dispatch_goto_rejects_invalid_payload() {
        // decode_action は `count` を知らない静的メソッドのため、パース不能
        // ペイロード（非数値・負数・空文字）のみを None（dispatch は false）で
        // 拒否する。
        let mut s = Steps::new(3, 1, Orientation::Horizontal);
        for bogus in ["abc", "-1", ""] {
            assert!(!dispatch(&mut s, "goto", bogus));
            assert_eq!(s.step(), 1);
        }
    }

    #[test]
    fn dispatch_goto_out_of_range_is_recognized_but_no_op() {
        // 範囲外の数値（`count` 超過）はパース自体は成功するため
        // dispatch は true を返すが、`update()` が範囲チェックして無視する
        // （[`Steps::update`] の不変条件、[`crate::progress::Progress`] の
        // `dispatch_set` clamp と同型の「decode 成功 = dispatch true」契約）。
        let mut s = Steps::new(3, 1, Orientation::Horizontal);
        assert!(dispatch(&mut s, "goto", "4"));
        assert_eq!(s.step(), 1);
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut s = Steps::new(3, 1, Orientation::Horizontal);
        assert!(!dispatch(&mut s, "no_such_action", "x"));
        assert_eq!(s.step(), 1);
    }

    /// レビュー観点回帰: `decode_action` を経由せず `StepsAction::Goto`
    /// を直接構築して `update()` を呼んでも、範囲外の `step` が混入しない
    /// （「`step` は `0..=count`」不変条件を `update()` 単体でも維持する、
    /// [`crate::progress::Progress`] の同型回帰テストに倣う）。
    #[test]
    fn update_rejects_out_of_range_goto_directly() {
        let mut s = Steps::new(3, 1, Orientation::Horizontal);
        Component::update(&mut s, StepsAction::Goto(100));
        assert_eq!(s.step(), 1);
    }

    // --- Steps: SSR 状態なし初期描画 ---

    #[test]
    fn default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Steps::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- Steps: hydration 経路 ---

    #[test]
    fn hydration_round_trip() {
        let s = Steps::new(4, 2, Orientation::Vertical);
        let rendered = render(&render_for_hydration(&s));
        assert!(rendered.contains(r#"data-hydrate-count="4""#));
        assert!(rendered.contains(r#"data-hydrate-step="2""#));
        assert!(rendered.contains(r#"data-hydrate-orientation="vertical""#));

        let restored = Steps::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Steps::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-count".to_string())
        );
    }

    #[test]
    fn from_hydration_attrs_invalid_value_does_not_panic() {
        let bogus_sets: Vec<Vec<(String, String)>> = vec![
            // count が非数値。
            vec![
                ("data-hydrate-count".to_string(), "abc".to_string()),
                ("data-hydrate-step".to_string(), "0".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            // count == 0。
            vec![
                ("data-hydrate-count".to_string(), "0".to_string()),
                ("data-hydrate-step".to_string(), "0".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            // step > count。
            vec![
                ("data-hydrate-count".to_string(), "3".to_string()),
                ("data-hydrate-step".to_string(), "10".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            // step が XSS ペイロード。
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
            // orientation が未知の語彙。
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
            let err = Steps::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: 呼び出し側 attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let s = Steps::default();
        let html = render(&s.root(vec![("data-testid", ATTR_BREAK_PAYLOAD)], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let s = Steps::default();
        let html = render(&s.item(0, vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
