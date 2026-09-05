//! Accordion（開閉可能な項目リスト）headless コンポーネント（イシュー #527、親 #526）。
//!
//! ark-ui の Accordion
//!（`.claude/skills/ark-ui/references/components/disclosure/accordion.md`）を
//! 参考に、Root / Item / ItemTrigger / ItemIndicator / ItemContent の 5
//! anatomy パーツと、Phase 1（#524）の [`crate::state::SingleSelect`] を
//! 埋め込んだ「高々 1 項目が開く」状態機械 [`Accordion`]、および
//! [`crate::state::MultiSelect`]（イシュー #594）を埋め込んだ「複数項目が
//! 同時に開く」状態機械 [`MultiAccordion`] を提供する。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`item`]/[`item_trigger`]/
//! [`item_indicator`]/[`item_content`]、いずれも純粋関数で完結）を直接呼んで
//! 組み立てる。各パーツは項目ごとの [`crate::state::OpenState`] を引数で
//! 受け取るため single/multiple のどちらのモードでも共用できる。
//!
//! CSR/hydration は用途に応じて [`Accordion`] または [`MultiAccordion`]
//! （いずれも [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を使い分ける。
//! [`Accordion`] は「高々 1 項目が開く」single モード
//! （dispatch: `"select"`/`"deselect"`/`"toggle"`、`"deselect"` は payload
//! なしで全解除）を、[`MultiAccordion`] は「複数項目が同時に開く」
//! multiple モード（dispatch: `"select"`/`"deselect"`/`"toggle"`、
//! `"deselect"` は項目値 payload 必須で当該項目のみ解除）を提供する。
//! `fandhe-frontend-pre-styled-ui`（#546〜）が本モジュールを呼んで
//! スタイル済み Accordion を組み立てる想定である。
//!
//! [`item_trigger`] は `data-value` を出力する（イシュー #1127）。これは
//! `fandhe-frontend-wasm-full` の `headless.rs::MAPPING_TABLE` が
//! `("accordion", "item-trigger")` クリックを `"toggle"` アクションへ写像
//! する際の payload 契約であり、単なる装飾属性ではない。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`type`/`role`/`hidden`/`disabled`/`id`）は
//!   すべて `&'static str` リテラルで固定しており、動的値が属性名スロットへ
//!   混入する経路はない（[`mod@crate::anatomy`]/[`crate::aria`]/
//!   [`crate::data_attrs`] の既存不変条件をそのまま継承する）。
//! - 動的値（`value`/`id`/`controls`/`labelled_by`/呼び出し側 `attrs`/
//!   `children` テキスト）は [`fandhe_frontend_core::render`] の既定
//!   エスケープを必ず経由する。`raw_html()` は使用せず、HTML 文字列を
//!   直接組み立てない（`id` の `format!` 利用は Tabs（#528）と同じく属性値
//!   という**データ**の組み立てであり、マークアップ自体の文字列化ではない）。
//! - `data-state` 値語彙（`"open"`/`"closed"`）は [`crate::state::OpenState`]
//!   に一元化し、本モジュールで独自の値を作らない。
//! - hydration 属性（`data-hydrate-selected`）はクライアント側で改ざんされ
//!   うる入力として扱う。[`Accordion`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は
//!   [`crate::state::SingleSelect`] へ全委譲することで、panic せず
//!   `HydrateError` を返す既存保証をそのまま継承する。
//! - `AccordionProps` は呼び出し側から渡される構造体だが、そこから読む
//!   `orientation`/`disabled` の 2 フィールドは属性値としてのみ使い
//!   （`orientation.as_data_state()` 相当の固定文字列 or bool）、属性名
//!   スロットへは一切混入しない。呼び出し側 `attrs` に固定付与キー
//!   （`data-orientation`/`aria-disabled`/`aria-hidden`）を混入させる
//!   なりすましは [`drop_reserved`] で除去する（#1903 と同型の対策）。
//!
//! # 参考サイト突合（イシュー #1636）
//!
//! ark-ui（一次参照）/Radix Primitives の Accordion と突合し、以下を是正した:
//!
//! - **`data-orientation`**（root・item・item-trigger・item-indicator・
//!   item-content の全パーツ）: ark-ui/Radix とも Root props の
//!   `orientation`（既定 vertical）を全パーツへ反映する。本モジュールは
//!   新設した [`AccordionProps`] を各パーツ関数へ通す形で対応する。
//! - **`item-indicator`/`item-content` の `data-disabled`**: ark-ui は
//!   indicator/content にも disabled 状態を反映する（Radix は content の
//!   み）。項目単位の `disabled` を両パーツへ伝播する。
//! - **`item-trigger` の `aria-disabled="true"`**（disabled 時のみ）:
//!   zag.js（ark-ui の実装基盤）の accordion trigger が付与する。ネイティブ
//!   `disabled` 属性のみでは支援技術によっては disabled 状態が伝わらない
//!   場合があるための補完。
//! - **`item-indicator` の `aria-hidden="true"`**: zag.js が indicator へ
//!   常時付与する（装飾用の視覚要素であり、支援技術には trigger の
//!   `aria-expanded` から状態が伝わるため indicator 自体は隠す）。
//! - **Root レベルの一括 `disabled`**: ark-ui/Radix とも Root に `disabled`
//!   props を持つ（全項目へ一括反映）。[`AccordionProps::disabled`] として
//!   採用し、実効 disabled は `props.disabled || 項目単位 disabled` とする。
//! - **キーボードナビゲーション**: SSR 静的マークアップ自体は変えず
//!   （orientation は `data-orientation` で表現済み）、`fandhe-frontend-wasm-full`
//!   の `keynav.rs` 側で horizontal（ArrowLeft/ArrowRight）対応を追加した
//!   （同 crate の変更履歴参照）。
//!
//! 以下は意図的に合わせなかった（`docs/policy/intentional-non-adoption.md`
//! §3.25 規則 2 に照らし、装飾・アニメーション・レイアウト計測の関心を
//! `headless-ui` へ持ち込まない判断を踏襲する）:
//!
//! - **`data-focus`**（ark-ui が item/trigger/indicator/content へ付与する
//!   フォーカスの一時的表現）: `data-focus-visible` 等と同じくクライアント
//!   ローカル状態であり、SSR には不要。他コンポーネント（toggle-group /
//!   radio-group / toggle）も同じ判断で不採用としている。
//! - **`data-controls`/`data-ownedby`**（ark-ui/zag.js が trigger へ付与する
//!   関連付け表現）: 本モジュールは `aria-controls` が既に同じ関連付けを
//!   担うため独自属性を追加しない。
//! - **Radix の `Header`（`<h3>` ラップ）パーツ**: 見出しレベルは呼び出し側
//!   の用途に依存するため `children` で自由に表現できる。専用パーツを追加
//!   すると `fandhe-frontend-pre-styled-ui` の `SLOTS`/CSS レシピ・
//!   Themes/Primitives Demo へ波及するため見送る。
//! - **`--height`/`--width` CSS 変数・`lazyMount`/`unmountOnExit`**:
//!   アニメーション対応・レイアウト計測の関心であり、必要なら上層
//!   （`pre-styled-ui`）の責務とする（§out-of-scope 参照）。
//! - **キーボードの循環（loop）**: APG は循環をオプションとするが、本実装は
//!   #582 の決定（`accordion_next_index`）を維持し非循環のままとする
//!   （意図的な差分として記録）。
//!
//! # out-of-scope（本イシュー #527/#594/#1636 のスコープ外）
//!
//! - **全項目一括 close（`MultiSelect` の payload なし deselect 相当）**:
//!   [`crate::state::MultiSelectAction::Deselect`] は項目単位（payload
//!   必須）のみを提供する。「どれを閉じるか」の指定なしに全解除する
//!   アクションはイシュー #594 の dispatch 契約に含まれないため未実装。
//! - **DOM キーボードイベント処理そのもの**: 実際のキー入力ハンドリング
//!   （focus 移動）は `fandhe-frontend-wasm-full` の `keynav.rs` の責務であり
//!   本モジュールは `data-orientation` の SSR 出力までを担う。
//! - **lazyMount / unmountOnExit / CSS 変数（`--height` 等）**: アニメーション
//!   対応はスコープ外（[`item_content`] は `hidden` 存在属性のみで closed を
//!   表現する）。
//! - **heading 要素でのラップ**: `<h3>` 等での [`item_trigger`] のラップは
//!   呼び出し側が `children` で自由に表現できるため、本モジュールは専用
//!   パーツを持たない。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{
    aria_controls, aria_disabled, aria_expanded, aria_hidden, aria_labelledby, role,
};
use crate::data_attrs::{data_disabled, data_orientation, data_state, Orientation};
use crate::state::{MultiSelect, MultiSelectAction, OpenState, SingleSelect, SingleSelectAction};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// Accordion の anatomy（`data-scope="accordion"`）。
const ANATOMY: Anatomy = anatomy("accordion");

/// Root レベルの共通プロパティ（ark-ui/Radix の Root `orientation`/`disabled`
/// 相当、イシュー #1636）。各パーツ関数へ通し `data-orientation`（全パーツ）
/// と実効 disabled（`disabled || 項目単位 disabled`、[`item_trigger`]/
/// [`item_indicator`]/[`item_content`] へ反映）を決定する。
///
/// `orientation` は SSR 静的マークアップ（`data-orientation` 属性）にのみ
/// 寄与し、実際のキーボード操作は `fandhe-frontend-wasm-full` の `keynav.rs`
/// が本属性を読んで解釈する（本モジュールはキー入力を処理しない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccordionProps {
    /// パーツの向き（既定 [`Orientation::Vertical`]。ark-ui/Radix の Root
    /// `orientation` 既定と同じ）。
    pub orientation: Orientation,
    /// 全項目を一括 disabled にするか（既定 `false`）。項目単位の
    /// `disabled` 引数と OR 合成され、いずれか true なら実効 disabled。
    pub disabled: bool,
}

impl Default for AccordionProps {
    fn default() -> Self {
        Self {
            orientation: Orientation::Vertical,
            disabled: false,
        }
    }
}

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（`crates/headless-ui/src/date_input.rs::drop_reserved` 等と同型の
/// 重複実装。モジュール間の相互依存を避けるため個別に定義する）。呼び出し側
/// が `data-orientation`/`aria-disabled`/`aria-hidden` を偽装してもフレーム
/// ワークが付与する値が常に優先されることを保証する（A05 対策）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// [`root`] が固定付与するキー一覧。
const ROOT_RESERVED: &[&str] = &["data-orientation"];
/// [`item`] が固定付与するキー一覧。
const ITEM_RESERVED: &[&str] = &["data-orientation"];
/// [`item_trigger`] が固定付与するキー一覧（[`ROOT_RESERVED`] に
/// `aria-disabled` を加えたもの）。
const ITEM_TRIGGER_RESERVED: &[&str] = &["data-orientation", "aria-disabled"];
/// [`item_indicator`] が固定付与するキー一覧（[`ROOT_RESERVED`] に
/// `aria-hidden` を加えたもの）。
const ITEM_INDICATOR_RESERVED: &[&str] = &["data-orientation", "aria-hidden"];
/// [`item_content`] が固定付与するキー一覧。
const ITEM_CONTENT_RESERVED: &[&str] = &["data-orientation"];

/// Root パーツ（`div`）。状態非依存（項目の開閉状態は各 [`item`] 側が持つ）。
/// `props.orientation` を `data-orientation` として出力する（イシュー #1636）。
#[must_use]
pub fn root<'a>(
    props: &AccordionProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_orientation(props.orientation)];
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Item パーツ（`div`）。項目 1 個の開閉状態・disabled 状態を `data-*` へ反映する。
/// `disabled` は項目単位の値そのもの（`props.disabled` との OR 合成は
/// [`item_trigger`]/[`item_indicator`]/[`item_content`] 側で行う。`item` 自体は
/// 呼び出し側が渡した `disabled` をそのまま表示する既存契約を維持する）。
#[must_use]
pub fn item<'a>(
    state: OpenState,
    disabled: bool,
    props: &AccordionProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        data_state(state.as_data_state()),
        data_orientation(props.orientation),
    ];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("item", "div", merged, children)
}

/// ItemTrigger パーツ（`button`）。
///
/// フォーム内配置時の意図しない submit を防ぐため `type="button"` を固定で
/// 付与する（A05 セキュリティ設定ミス対策。Collapsible 実装（イシュー #529）
/// と同じ判断を踏襲する）。`controls` が `Some` のとき
/// `aria-controls` で [`item_content`] と関連付ける。実効 disabled
/// （`props.disabled || disabled`）はネイティブ `disabled` 存在属性・
/// `data-disabled`・`aria-disabled="true"`（disabled 時のみ、イシュー #1636。
/// zag.js の accordion trigger 実装に合わせる補完属性）の 3 つへ反映する。
///
/// イシュー #1127: `fandhe-frontend-wasm-full` の headless 配線基盤
/// （`wasm-full/src/headless.rs::MAPPING_TABLE`）が
/// `(data-scope, data-part) = ("accordion", "item-trigger")` クリックを
/// `"toggle"` アクション（[`SingleSelectAction::Toggle`]/
/// [`MultiSelectAction::Toggle`]、いずれも項目値 payload 必須）へ写像する
/// 際の payload 源として `value` を `data-value` へ出力する（Tabs の
/// `trigger`（#580）と同型の契約）。この出力を欠くと
/// `requires_value: true` 行は常に fail-closed（`None`）となりクリック
/// でもキーボード（Enter/Space、ネイティブ `<button>` click 発火経由）
/// でも開閉が no-op のままになる。
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn item_trigger<'a>(
    state: OpenState,
    disabled: bool,
    props: &AccordionProps,
    value: &'a str,
    id: Option<&'a str>,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_TRIGGER_RESERVED);
    let effective_disabled = props.disabled || disabled;
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        aria_expanded(state.is_open()),
        data_state(state.as_data_state()),
        data_orientation(props.orientation),
        ("data-value", value),
    ];
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if let Some(controls) = controls {
        merged.push(aria_controls(controls));
    }
    merged.extend(data_disabled(effective_disabled));
    if effective_disabled {
        merged.push(("disabled", ""));
        merged.push(aria_disabled(true));
    }
    merged.extend(attrs);
    ANATOMY.part("item-trigger", "button", merged, children)
}

/// ItemIndicator パーツ（`span`）。開閉状態のみを `data-state` へ反映する
/// 最小主義な装飾用パーツ（アイコン等は呼び出し側の `attrs`/`children` が
/// 担う。Collapsible の `indicator` と同じ最小主義に揃える）。実効 disabled
/// （`props.disabled || disabled`）を `data-disabled` へ反映し、常時
/// `aria-hidden="true"` を付与する（イシュー #1636。装飾用の視覚要素で
/// あり、支援技術へは [`item_trigger`] の `aria-expanded` から開閉状態が
/// 伝わるため indicator 自体は隠す。zag.js の accordion 実装に合わせる）。
#[must_use]
pub fn item_indicator<'a>(
    state: OpenState,
    disabled: bool,
    props: &AccordionProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_INDICATOR_RESERVED);
    let effective_disabled = props.disabled || disabled;
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        data_state(state.as_data_state()),
        data_orientation(props.orientation),
        aria_hidden(true),
    ];
    merged.extend(data_disabled(effective_disabled));
    merged.extend(attrs);
    ANATOMY.part("item-indicator", "span", merged, children)
}

/// ItemContent パーツ（`div`）。
///
/// closed のとき `hidden` 存在属性を付与し、JS なしの SSR でも閉状態を表現
/// する（アニメーション対応の CSS 変数出力等はスコープ外、モジュール doc
/// §out-of-scope 参照）。`id` が `Some` のとき [`item_trigger`] の `controls`
/// と対で `aria-controls` 関連付けを成立させる。`labelled_by` が `Some` の
/// ときのみ `role="region"` と `aria-labelledby` をセットで付与する
/// （名前なし region を作らないため、`labelled_by` が `None` の場合は
/// どちらも出力しない）。実効 disabled（`props.disabled || disabled`）を
/// `data-disabled` へ反映する（イシュー #1636。ark-ui の accordion content
/// が disabled 状態を反映する仕様に合わせる）。
#[must_use]
pub fn item_content<'a>(
    state: OpenState,
    disabled: bool,
    props: &AccordionProps,
    id: Option<&'a str>,
    labelled_by: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_CONTENT_RESERVED);
    let effective_disabled = props.disabled || disabled;
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        data_state(state.as_data_state()),
        data_orientation(props.orientation),
    ];
    merged.extend(data_disabled(effective_disabled));
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if let Some(labelled_by) = labelled_by {
        merged.push(role("region"));
        merged.push(aria_labelledby(labelled_by));
    }
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("item-content", "div", merged, children)
}

/// [`SingleSelect`]（#524）を埋め込んだ Accordion（single モード）の状態機械。
///
/// 「高々 1 項目が開く」制約を型レベルで保証する入口として、[`Self::item_state`]
/// が各項目値の [`OpenState`] を決定し、各パーツ関数（[`item`]/
/// [`item_trigger`]/[`item_indicator`]/[`item_content`]）へ注入する利便
/// メソッドを提供する（[`root`] は状態非依存のため利便メソッドを持たない）。
/// SSR での自由関数直接利用（本型を経由しない構成。複数項目同時 open の
/// 表現を含む）も引き続き可能。`Default` は未選択（全項目 closed。SSR の
/// 状態なし初期描画に対応する既定値）。
///
/// collapsible な挙動（開いた項目を再クリックで閉じる）が必要な呼び出し側は
/// dispatch アクション名 `"toggle"`（[`SingleSelectAction::Toggle`]）を、
/// 常に何か 1 項目を開いたままにしたい場合は `"select"`
/// （[`SingleSelectAction::Select`]）を使い分ける。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Accordion {
    select: SingleSelect,
}

impl Accordion {
    /// 現在開いている項目値（未選択なら `None`）。
    #[must_use]
    pub fn expanded(&self) -> Option<&str> {
        self.select.selected()
    }

    /// 指定した項目値が開いているかどうか。
    #[must_use]
    pub fn is_open(&self, value: &str) -> bool {
        self.select.is_selected(value)
    }

    /// 項目 `value` の現在の [`OpenState`]。
    #[must_use]
    pub fn item_state(&self, value: &str) -> OpenState {
        if self.is_open(value) {
            OpenState::Open
        } else {
            OpenState::Closed
        }
    }

    /// [`item`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item<'a>(
        &self,
        value: &str,
        disabled: bool,
        props: &AccordionProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(self.item_state(value), disabled, props, attrs, children)
    }

    /// [`item_trigger`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn item_trigger<'a>(
        &self,
        value: &str,
        disabled: bool,
        props: &AccordionProps,
        id: Option<&'a str>,
        controls: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_trigger(
            self.item_state(value),
            disabled,
            props,
            value,
            id,
            controls,
            attrs,
            children,
        )
    }

    /// [`item_indicator`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item_indicator<'a>(
        &self,
        value: &str,
        disabled: bool,
        props: &AccordionProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_indicator(self.item_state(value), disabled, props, attrs, children)
    }

    /// [`item_content`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn item_content<'a>(
        &self,
        value: &str,
        disabled: bool,
        props: &AccordionProps,
        id: Option<&'a str>,
        labelled_by: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_content(
            self.item_state(value),
            disabled,
            props,
            id,
            labelled_by,
            attrs,
            children,
        )
    }
}

impl Component for Accordion {
    type Action = SingleSelectAction;

    fn update(&mut self, action: SingleSelectAction) {
        self.select.update(action);
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（root、children
    /// 空）。[`SingleSelect::view`] と同じ位置付けであり、公開 UI としての
    /// 利用は想定しない（実際の UI 構築は §パーツ関数群を呼び出し側が
    /// 組み合わせる）。
    fn view(&self) -> Node {
        root(&AccordionProps::default(), Vec::new(), Vec::new())
    }

    fn decode_action(name: &str, payload: &str) -> Option<SingleSelectAction> {
        SingleSelect::decode_action(name, payload)
    }
}

impl Hydrate for Accordion {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.select.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            select: SingleSelect::from_hydration_attrs(attrs)?,
        })
    }
}

/// [`MultiSelect`]（イシュー #594）を埋め込んだ Accordion（multiple モード）
/// の状態機械。
///
/// [`Accordion`]（single モード）と対称の API を提供する。「複数項目が同時に
/// 開く」ことを許すため [`Self::expanded`] は `&[String]` を返す。
/// [`Component::Action`] は関連型が 1 つのため、single/multiple 双方を 1 型で
/// 扱おうとすると dispatch 契約（`"deselect"` の payload 有無）が衝突する。
/// 型を分けることで hydration の解釈（2 件以上のリストを拒否/受理のどちらで
/// 扱うか）も静的に確定し、fail-closed 性を保つ（詳細は
/// `docs/design`（該当があれば）または本イシューの実装計画を参照）。
/// `Default` は全項目 closed（SSR の状態なし初期描画に対応する既定値）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MultiAccordion {
    select: MultiSelect,
}

impl MultiAccordion {
    /// 現在開いている項目値（選択順）。
    #[must_use]
    pub fn expanded(&self) -> &[String] {
        self.select.selected()
    }

    /// 指定した項目値が開いているかどうか。
    #[must_use]
    pub fn is_open(&self, value: &str) -> bool {
        self.select.is_selected(value)
    }

    /// 項目 `value` の現在の [`OpenState`]。
    #[must_use]
    pub fn item_state(&self, value: &str) -> OpenState {
        if self.is_open(value) {
            OpenState::Open
        } else {
            OpenState::Closed
        }
    }

    /// [`item`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item<'a>(
        &self,
        value: &str,
        disabled: bool,
        props: &AccordionProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(self.item_state(value), disabled, props, attrs, children)
    }

    /// [`item_trigger`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn item_trigger<'a>(
        &self,
        value: &str,
        disabled: bool,
        props: &AccordionProps,
        id: Option<&'a str>,
        controls: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_trigger(
            self.item_state(value),
            disabled,
            props,
            value,
            id,
            controls,
            attrs,
            children,
        )
    }

    /// [`item_indicator`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item_indicator<'a>(
        &self,
        value: &str,
        disabled: bool,
        props: &AccordionProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_indicator(self.item_state(value), disabled, props, attrs, children)
    }

    /// [`item_content`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn item_content<'a>(
        &self,
        value: &str,
        disabled: bool,
        props: &AccordionProps,
        id: Option<&'a str>,
        labelled_by: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_content(
            self.item_state(value),
            disabled,
            props,
            id,
            labelled_by,
            attrs,
            children,
        )
    }
}

impl Component for MultiAccordion {
    type Action = MultiSelectAction;

    fn update(&mut self, action: MultiSelectAction) {
        self.select.update(action);
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（root、children
    /// 空）。[`MultiSelect::view`] と同じ位置付け。
    fn view(&self) -> Node {
        root(&AccordionProps::default(), Vec::new(), Vec::new())
    }

    fn decode_action(name: &str, payload: &str) -> Option<MultiSelectAction> {
        MultiSelect::decode_action(name, payload)
    }
}

impl Hydrate for MultiAccordion {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.select.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            select: MultiSelect::from_hydration_attrs(attrs)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    fn vertical() -> AccordionProps {
        AccordionProps::default()
    }

    fn horizontal() -> AccordionProps {
        AccordionProps {
            orientation: Orientation::Horizontal,
            disabled: false,
        }
    }

    // --- 各パーツの data-scope/data-part/data-state 出力 ---

    #[test]
    fn root_outputs_scope_part_and_orientation() {
        let html = render(&root(&vertical(), vec![], vec![]));
        assert_eq!(
            html,
            r#"<div data-scope="accordion" data-part="root" data-orientation="vertical"></div>"#
        );

        let html_h = render(&root(&horizontal(), vec![], vec![]));
        assert!(html_h.contains(r#"data-orientation="horizontal""#));
    }

    #[test]
    fn root_caller_supplied_data_orientation_is_dropped() {
        let html = render(&root(
            &vertical(),
            vec![("data-orientation", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-orientation="vertical""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn item_outputs_scope_part_state_and_orientation() {
        let html = render(&item(OpenState::Closed, false, &vertical(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="accordion""#));
        assert!(html.contains(r#"data-part="item""#));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(html.contains(r#"data-orientation="vertical""#));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn item_disabled_true_adds_data_disabled() {
        let html = render(&item(OpenState::Open, true, &vertical(), vec![], vec![]));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_trigger_has_type_button_and_aria_expanded() {
        let html = render(&item_trigger(
            OpenState::Closed,
            false,
            &vertical(),
            "a",
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains("<button"));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(html.contains(r#"data-orientation="vertical""#));
        assert!(!html.contains("aria-controls"));
        assert!(!html.contains(" id="));
        assert!(!html.contains("disabled"));

        let html_open = render(&item_trigger(
            OpenState::Open,
            false,
            &vertical(),
            "a",
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html_open.contains(r#"aria-expanded="true""#));
    }

    #[test]
    fn item_trigger_id_and_controls_some_outputs_both_attributes() {
        let html = render(&item_trigger(
            OpenState::Closed,
            false,
            &vertical(),
            "t-trigger-a",
            Some("t-trigger-a"),
            Some("t-content-a"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"id="t-trigger-a""#));
        assert!(html.contains(r#"aria-controls="t-content-a""#));
    }

    #[test]
    fn item_trigger_disabled_true_adds_native_data_and_aria_disabled() {
        let html = render(&item_trigger(
            OpenState::Closed,
            true,
            &vertical(),
            "a",
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"aria-disabled="true""#));
    }

    #[test]
    fn item_trigger_disabled_false_omits_all_disabled_attrs() {
        let html = render(&item_trigger(
            OpenState::Closed,
            false,
            &vertical(),
            "a",
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains(" disabled"));
        assert!(!html.contains("aria-disabled"));
    }

    #[test]
    fn item_trigger_root_disabled_propagates_to_effective_disabled() {
        let props = AccordionProps {
            orientation: Orientation::Vertical,
            disabled: true,
        };
        let html = render(&item_trigger(
            OpenState::Closed,
            false,
            &props,
            "a",
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"aria-disabled="true""#));
    }

    #[test]
    fn item_trigger_caller_supplied_reserved_keys_are_dropped() {
        let html = render(&item_trigger(
            OpenState::Closed,
            true,
            &horizontal(),
            "a",
            None,
            None,
            vec![("data-orientation", "attacker"), ("aria-disabled", "false")],
            vec![],
        ));
        assert!(html.contains(r#"data-orientation="horizontal""#));
        assert!(html.contains(r#"aria-disabled="true""#));
        assert!(!html.contains("attacker"));
    }

    // イシュー #1127: `data-value` は wasm-full `MAPPING_TABLE` の
    // `"toggle"` payload 契約であり、単なる装飾属性ではない。Tabs の
    // `trigger_outputs_data_value_matching_item_value`（#580）と同型の
    // 回帰テスト。
    #[test]
    fn item_trigger_outputs_data_value_matching_item_value() {
        let html_a = render(&item_trigger(
            OpenState::Closed,
            false,
            &vertical(),
            "a",
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html_a.contains(r#"data-value="a""#));

        let html_b = render(&item_trigger(
            OpenState::Open,
            false,
            &vertical(),
            "b",
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html_b.contains(r#"data-value="b""#));
    }

    // `data-value` は wasm-full 側で改ざんされうるクライアント入力（クリック時
    // に payload として再度読まれる）だが、SSR 出力自体は `ANATOMY.part` 経由
    // で既定エスケープを必ず経由することを固定する（REQ-1、Tabs の
    // `trigger_data_value_payload_is_escaped_on_render` と同型）。
    #[test]
    fn item_trigger_data_value_payload_is_escaped_on_render() {
        let payload = "\"><script>alert(1)</script>";
        let html = render(&item_trigger(
            OpenState::Closed,
            false,
            &vertical(),
            payload,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(!html.contains(r#""><script"#));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn item_indicator_outputs_scope_part_state_orientation_and_aria_hidden() {
        let html = render(&item_indicator(
            OpenState::Open,
            false,
            &vertical(),
            vec![],
            vec![text("+")],
        ));
        assert!(html.contains(r#"data-scope="accordion""#));
        assert!(html.contains(r#"data-part="item-indicator""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains(r#"data-orientation="vertical""#));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(html.contains('+'));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn item_indicator_disabled_true_adds_data_disabled() {
        let html = render(&item_indicator(
            OpenState::Open,
            true,
            &vertical(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_indicator_caller_supplied_aria_hidden_is_dropped() {
        let html = render(&item_indicator(
            OpenState::Open,
            false,
            &vertical(),
            vec![("aria-hidden", "false")],
            vec![],
        ));
        assert!(html.contains(r#"aria-hidden="true""#));
    }

    #[test]
    fn item_content_closed_has_hidden_attr_open_does_not() {
        let closed = render(&item_content(
            OpenState::Closed,
            false,
            &vertical(),
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&item_content(
            OpenState::Open,
            false,
            &vertical(),
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn item_content_disabled_true_adds_data_disabled() {
        let html = render(&item_content(
            OpenState::Open,
            true,
            &vertical(),
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_content_outputs_orientation() {
        let html = render(&item_content(
            OpenState::Open,
            false,
            &horizontal(),
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-orientation="horizontal""#));
    }

    #[test]
    fn item_content_id_some_outputs_id_attribute() {
        let html = render(&item_content(
            OpenState::Open,
            false,
            &vertical(),
            Some("t-content-a"),
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"id="t-content-a""#));
    }

    #[test]
    fn item_content_labelled_by_some_outputs_role_region_and_aria_labelledby_together() {
        let html = render(&item_content(
            OpenState::Open,
            false,
            &vertical(),
            None,
            Some("t-trigger-a"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"role="region""#));
        assert!(html.contains(r#"aria-labelledby="t-trigger-a""#));
    }

    #[test]
    fn item_content_labelled_by_none_omits_role_and_aria_labelledby() {
        let html = render(&item_content(
            OpenState::Open,
            false,
            &vertical(),
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(!html.contains("role="));
        assert!(!html.contains("aria-labelledby"));
    }

    // 意図的に非採用とした属性・パートが一切出力されないことを固定する
    // 回帰テスト（イシュー #1636、モジュール doc §参考サイト突合参照）。
    #[test]
    fn intentionally_omitted_attributes_are_absent() {
        let props = AccordionProps {
            orientation: Orientation::Horizontal,
            disabled: true,
        };
        let node = root(
            &props,
            vec![],
            vec![item(
                OpenState::Open,
                true,
                &props,
                vec![],
                vec![
                    item_trigger(
                        OpenState::Open,
                        true,
                        &props,
                        "a",
                        None,
                        None,
                        vec![],
                        vec![item_indicator(
                            OpenState::Open,
                            true,
                            &props,
                            vec![],
                            vec![],
                        )],
                    ),
                    item_content(OpenState::Open, true, &props, None, None, vec![], vec![]),
                ],
            )],
        );
        let html = render(&node);
        assert!(!html.contains("data-focus"));
        assert!(!html.contains("data-motion"));
        assert!(!html.contains("data-ownedby"));
        assert!(!html.contains("data-controls"));
        assert!(!html.contains("--height"));
        assert!(!html.contains("--width"));
        assert!(!html.contains(r#"data-part="header""#));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&item(
            OpenState::Closed,
            false,
            &vertical(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="accordion""#));
        assert!(html.contains(r#"data-part="item""#));
        assert!(!html.contains("attacker"));
    }

    // --- root > item > trigger(+indicator) + content の組み立てテスト（相互参照込みスナップショット） ---

    #[test]
    fn full_assembly_trigger_and_content_id_cross_reference() {
        let props = vertical();
        let node = root(
            &props,
            vec![],
            vec![item(
                OpenState::Open,
                false,
                &props,
                vec![],
                vec![
                    item_trigger(
                        OpenState::Open,
                        false,
                        &props,
                        "a",
                        Some("t-trigger-a"),
                        Some("t-content-a"),
                        vec![],
                        vec![item_indicator(
                            OpenState::Open,
                            false,
                            &props,
                            vec![],
                            vec![text("+")],
                        )],
                    ),
                    item_content(
                        OpenState::Open,
                        false,
                        &props,
                        Some("t-content-a"),
                        Some("t-trigger-a"),
                        vec![],
                        vec![text("panel A")],
                    ),
                ],
            )],
        );
        assert_eq!(
            render(&node),
            concat!(
                r#"<div data-scope="accordion" data-part="root" data-orientation="vertical">"#,
                r#"<div data-scope="accordion" data-part="item" data-state="open" data-orientation="vertical">"#,
                r#"<button data-scope="accordion" data-part="item-trigger" type="button" aria-expanded="true" data-state="open" data-orientation="vertical" data-value="a" id="t-trigger-a" aria-controls="t-content-a">"#,
                r#"<span data-scope="accordion" data-part="item-indicator" data-state="open" data-orientation="vertical" aria-hidden="true">+</span>"#,
                r#"</button>"#,
                r#"<div data-scope="accordion" data-part="item-content" data-state="open" data-orientation="vertical" id="t-content-a" role="region" aria-labelledby="t-trigger-a">panel A</div>"#,
                r#"</div>"#,
                r#"</div>"#,
            )
        );
    }

    // --- Accordion: dispatch 統合（single モード） ---

    #[test]
    fn accordion_default_is_all_closed() {
        let a = Accordion::default();
        assert_eq!(a.expanded(), None);
        assert!(!a.is_open("a"));
        assert!(!a.is_open("b"));
    }

    #[test]
    fn accordion_dispatch_select_opens_at_most_one_item() {
        let mut a = Accordion::default();
        assert!(dispatch(&mut a, "select", "a"));
        assert!(a.is_open("a"));
        assert!(!a.is_open("b"));

        assert!(dispatch(&mut a, "select", "b"));
        assert!(!a.is_open("a"));
        assert!(a.is_open("b"));
    }

    #[test]
    fn accordion_dispatch_toggle_opens_then_closes_collapsible_style() {
        let mut a = Accordion::default();
        assert!(dispatch(&mut a, "toggle", "a"));
        assert!(a.is_open("a"));

        assert!(dispatch(&mut a, "toggle", "a"));
        assert!(!a.is_open("a"));
        assert_eq!(a.expanded(), None);
    }

    #[test]
    fn accordion_dispatch_deselect_closes_all() {
        let mut a = Accordion::default();
        dispatch(&mut a, "select", "a");
        assert!(dispatch(&mut a, "deselect", ""));
        assert_eq!(a.expanded(), None);
    }

    #[test]
    fn accordion_dispatch_ignores_unknown_action() {
        let mut a = Accordion::default();
        dispatch(&mut a, "select", "a");
        assert!(!dispatch(&mut a, "no_such_action", "b"));
        assert!(a.is_open("a"));
    }

    // --- Accordion: 利便メソッド経由の描画が状態機械と一致 ---

    #[test]
    fn accordion_convenience_methods_reflect_state() {
        let mut a = Accordion::default();
        dispatch(&mut a, "select", "a");
        let props = vertical();

        let trigger_a = render(&a.item_trigger("a", false, &props, None, None, vec![], vec![]));
        assert!(trigger_a.contains(r#"aria-expanded="true""#));
        assert!(trigger_a.contains(r#"data-state="open""#));

        let trigger_b = render(&a.item_trigger("b", false, &props, None, None, vec![], vec![]));
        assert!(trigger_b.contains(r#"aria-expanded="false""#));
        assert!(trigger_b.contains(r#"data-state="closed""#));

        let content_a = render(&a.item_content("a", false, &props, None, None, vec![], vec![]));
        assert!(!content_a.contains("hidden"));

        let content_b = render(&a.item_content("b", false, &props, None, None, vec![], vec![]));
        assert!(content_b.contains(r#"hidden="""#));
    }

    // --- Accordion: SSR 状態なし初期描画 ---

    #[test]
    fn accordion_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Accordion::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- Accordion: hydration 経路 ---

    #[test]
    fn accordion_hydration_round_trip_selected() {
        let mut a = Accordion::default();
        dispatch(&mut a, "select", "tab-1");
        let rendered = render(&render_for_hydration(&a));
        // codec::encode_list は区切り文字を先頭に付与するエンコードのため、
        // 属性値は選択値そのままの文字列（"tab-1"）とは一致しない。属性が
        // 実際に出力され値に選択値が含まれることのみを確認する
        // （エンコード形式の詳細は `crate::state::SingleSelect` の責務）。
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("tab-1"));

        let restored = Accordion::from_hydration_attrs(&a.hydration_attrs()).unwrap();
        assert_eq!(restored, a);
    }

    #[test]
    fn accordion_hydration_round_trip_unselected() {
        let a = Accordion::default();
        let restored = Accordion::from_hydration_attrs(&a.hydration_attrs()).unwrap();
        assert_eq!(restored, a);
    }

    #[test]
    fn accordion_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Accordion::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-selected".to_string())
        );
    }

    #[test]
    fn accordion_from_hydration_attrs_invalid_value_does_not_panic() {
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["a".to_string(), "b".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = Accordion::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- XSS 回帰: value/id/controls/labelled_by/呼び出し側 attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn item_trigger_id_and_controls_payload_is_escaped_on_render() {
        let html = render(&item_trigger(
            OpenState::Closed,
            false,
            &vertical(),
            ATTR_BREAK_PAYLOAD,
            Some(ATTR_BREAK_PAYLOAD),
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn item_content_id_and_labelled_by_payload_is_escaped_on_render() {
        let html = render(&item_content(
            OpenState::Open,
            false,
            &vertical(),
            Some(ATTR_BREAK_PAYLOAD),
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            &vertical(),
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&item_indicator(
            OpenState::Open,
            false,
            &vertical(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn accordion_dispatch_select_payload_is_escaped_on_render() {
        let mut a = Accordion::default();
        let payload = "\"><script>alert(1)</script>";
        assert!(dispatch(&mut a, "select", payload));

        let rendered = render(&render_for_hydration(&a));
        // 正の確認: data-hydrate-selected 属性が実際に出力へ載っていること
        // （不在アサーションのみだと属性ごと消えた場合にも誤って合格しうる）。
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(!rendered.contains(r#""><script"#));
    }

    #[test]
    fn accordion_xss_payload_in_hydration_selected_is_rejected_not_rendered() {
        // 改ざん耐性: from_hydration_attrs は未知/不正な値を panic せず拒否する
        // （SingleSelect の既存保証を Accordion 経由でも固定する）。
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["<script>alert(1)</script>".to_string(), "b".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = Accordion::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn accordion_view_root_is_element_for_render_for_hydration() {
        // render_for_hydration はルートが Node::Element であることを前提に
        // 属性を合成する（`crates/interactive/src/lib.rs` 参照）。本型の
        // view() が常に Element を返すことを固定する回帰テスト。
        let node = Accordion::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }

    // --- MultiAccordion: dispatch 統合（multiple モード） ---

    #[test]
    fn multi_accordion_default_is_all_closed() {
        let a = MultiAccordion::default();
        assert_eq!(a.expanded(), &[] as &[String]);
        assert!(!a.is_open("a"));
        assert!(!a.is_open("b"));
    }

    #[test]
    fn multi_accordion_dispatch_select_opens_multiple_items_simultaneously() {
        let mut a = MultiAccordion::default();
        assert!(dispatch(&mut a, "select", "a"));
        assert!(a.is_open("a"));
        assert!(!a.is_open("b"));

        // 複数項目同時 open が本型の存在理由（Accordion は select 2 回目で
        // 前項目が閉じるが、MultiAccordion は両方 open のまま維持する）。
        assert!(dispatch(&mut a, "select", "b"));
        assert!(a.is_open("a"));
        assert!(a.is_open("b"));
    }

    #[test]
    fn multi_accordion_dispatch_toggle_and_deselect_close_only_target_item() {
        let mut a = MultiAccordion::default();
        dispatch(&mut a, "select", "a");
        dispatch(&mut a, "select", "b");

        assert!(dispatch(&mut a, "deselect", "a"));
        assert!(!a.is_open("a"));
        assert!(a.is_open("b"));

        assert!(dispatch(&mut a, "toggle", "b"));
        assert!(!a.is_open("b"));
        assert_eq!(a.expanded(), &[] as &[String]);
    }

    #[test]
    fn multi_accordion_dispatch_ignores_unknown_action() {
        let mut a = MultiAccordion::default();
        dispatch(&mut a, "select", "a");
        assert!(!dispatch(&mut a, "no_such_action", "b"));
        assert!(a.is_open("a"));
    }

    // --- MultiAccordion: 利便メソッド経由の描画が状態機械と一致（複数同時 open） ---

    #[test]
    fn multi_accordion_convenience_methods_reflect_state_for_two_open_items() {
        let mut a = MultiAccordion::default();
        dispatch(&mut a, "select", "a");
        dispatch(&mut a, "select", "b");
        let props = vertical();

        let trigger_a = render(&a.item_trigger("a", false, &props, None, None, vec![], vec![]));
        assert!(trigger_a.contains(r#"aria-expanded="true""#));
        let trigger_b = render(&a.item_trigger("b", false, &props, None, None, vec![], vec![]));
        assert!(trigger_b.contains(r#"aria-expanded="true""#));
        let trigger_c = render(&a.item_trigger("c", false, &props, None, None, vec![], vec![]));
        assert!(trigger_c.contains(r#"aria-expanded="false""#));

        let content_a = render(&a.item_content("a", false, &props, None, None, vec![], vec![]));
        assert!(!content_a.contains("hidden"));
        let content_b = render(&a.item_content("b", false, &props, None, None, vec![], vec![]));
        assert!(!content_b.contains("hidden"));
        let content_c = render(&a.item_content("c", false, &props, None, None, vec![], vec![]));
        assert!(content_c.contains(r#"hidden="""#));
    }

    // --- MultiAccordion: SSR 状態なし初期描画 ---

    #[test]
    fn multi_accordion_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&MultiAccordion::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- MultiAccordion: hydration 経路（複数同時 open のラウンドトリップ） ---

    #[test]
    fn multi_accordion_hydration_round_trip_multiple_selected() {
        let mut a = MultiAccordion::default();
        dispatch(&mut a, "select", "tab-1");
        dispatch(&mut a, "select", "tab-2");
        let rendered = render(&render_for_hydration(&a));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("tab-1"));
        assert!(rendered.contains("tab-2"));

        let restored = MultiAccordion::from_hydration_attrs(&a.hydration_attrs()).unwrap();
        assert_eq!(restored, a);
    }

    #[test]
    fn multi_accordion_hydration_round_trip_unselected() {
        let a = MultiAccordion::default();
        let restored = MultiAccordion::from_hydration_attrs(&a.hydration_attrs()).unwrap();
        assert_eq!(restored, a);
    }

    #[test]
    fn multi_accordion_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = MultiAccordion::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-selected".to_string())
        );
    }

    #[test]
    fn multi_accordion_from_hydration_attrs_duplicate_value_rejected_not_panicking() {
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["a".to_string(), "a".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = MultiAccordion::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- XSS 回帰: MultiAccordion の dispatch payload/hydration 経路 ---

    #[test]
    fn multi_accordion_dispatch_select_payload_is_escaped_on_render() {
        let mut a = MultiAccordion::default();
        let payload = "\"><script>alert(1)</script>";
        assert!(dispatch(&mut a, "select", payload));

        let rendered = render(&render_for_hydration(&a));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(!rendered.contains(r#""><script"#));
    }

    #[test]
    fn multi_accordion_xss_payload_in_hydration_selected_is_rejected_not_rendered() {
        // 改ざん耐性: from_hydration_attrs は不正な値（重複）を panic せず
        // 拒否する（MultiSelect の既存保証を MultiAccordion 経由でも固定）。
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&[
            "<script>alert(1)</script>".to_string(),
            "<script>alert(1)</script>".to_string(),
        ]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = MultiAccordion::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn multi_accordion_view_root_is_element_for_render_for_hydration() {
        // render_for_hydration はルートが Node::Element であることを前提に
        // 属性を合成する（`crates/interactive/src/lib.rs` 参照）。本型の
        // view() が常に Element を返すことを固定する回帰テスト。
        let node = MultiAccordion::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }
}
