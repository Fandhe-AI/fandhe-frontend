//! Menu（トリガー起点のオーバーレイ + アクション項目リスト）headless
//! コンポーネント（イシュー #540、親 #539、ルートトラッキング #520）。
//!
//! ark-ui の Menu
//!（`.claude/skills/ark-ui/references/components/collections/menu.md`）を
//! 参考に、Root / Trigger / Indicator / Positioner / Content / Arrow /
//! ArrowTip / Item / ItemGroup / ItemGroupLabel / Separator / TriggerItem /
//! ContextTrigger の 13 anatomy パーツと、Phase 1（#524）の
//! [`crate::state::Disclosure`] を埋め込んだ開閉状態機械 [`Menu`] を提供する。
//! **構造上最も近い先行例は [`crate::popover::Popover`]**（trigger 起点の
//! オーバーレイ + `Disclosure` 埋め込み）であり、本モジュールはそのパターンに
//! 完全準拠する。TriggerItem/ContextTrigger（イシュー #598、親 #596/#593）は
//! いずれも既存 [`Menu`] インスタンスをそのまま流用する合成であり、新しい
//! 状態機械は追加しない（サブメニューは「子 Menu インスタンス 1 個」、
//! ContextTrigger は「右クリックで開かれる Menu インスタンス自身」を指す）。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`trigger`]/[`indicator`]/
//! [`positioner`]/[`content`]/[`arrow`]/[`arrow_tip`]/[`item`]/
//! [`item_group`]/[`item_group_label`]/[`separator`]/[`trigger_item`]/
//! [`context_trigger`]、純粋関数で完結）を直接呼んで組み立てる。
//! サブメニューは「親 `Menu` インスタンスの `content` 内に子 `Menu`
//! インスタンス由来の `trigger_item`/`positioner`/`content` を入れ子で配置
//! する」ことで表現する（親子は別インスタンスであり、`aria-haspopup`
//! 連鎖 — 親 [`trigger`] と子 [`trigger_item`] の双方に
//! `aria-haspopup="menu"` — でネストしたメニューであることを示す）。
//! CSR/hydration は [`Menu`]
//!（[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"open"`/`"close"`/`"toggle"`）で状態遷移する。`fandhe-frontend-pre-styled-ui`
//! （#546〜、#551）が本モジュールを呼んでスタイル済み Menu を組み立てる想定である。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`/`hidden`/`disabled`/`id`）は
//!   すべて `&'static str` リテラルで固定しており、動的値が属性名スロットへ
//!   混入する経路はない（[`crate::anatomy`]/[`crate::aria`]/
//!   [`crate::data_attrs`] の既存不変条件をそのまま継承する）。
//! - 動的値（`value`/`controls`/`id`/`labelledby`/呼び出し側 `attrs`/
//!   `children` テキスト）は [`fandhe_frontend_core::render`] の既定エスケープを
//!   必ず経由する。`raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-state` 値語彙（`"open"`/`"closed"`）は [`crate::state::OpenState`]
//!   に一元化し、本モジュールで独自の値を作らない。
//! - hydration 属性（`data-hydrate-state`）はクライアント側で改ざんされうる
//!   入力として扱う。[`Menu`] の [`fandhe_frontend_interactive::Hydrate`]
//!   実装は [`crate::state::Disclosure`] へ全委譲することで、panic せず
//!   `HydrateError` を返す既存保証をそのまま継承する。
//!
//! # スコープ外（ark-ui Menu のうち本イシューで実装しないもの）
//!
//! - `CheckboxItem`/`RadioGroup`/`RadioItem`: form 系（Checkbox #534 系列）の
//!   checked 状態設計と整合させるべきであり、別イシュー化をユーザーへ提案する
//!   （`out-of-scope-tracking.md` 準拠、勝手に起票しない）。
//! - [`trigger_item`]/[`context_trigger`] の**クライアントイベント処理**
//!   （右クリック検知・ホバー/矢印キーでのサブメニュー展開・フォーカス管理）:
//!   本イシュー（#598）は SSR マークアップ（anatomy 表現）のみを対象とし、
//!   配線は wasm ランタイム側の将来イシュー（#580/PR #611 系列）のスコープ。
//! - `loopFocus`/`typeahead`/`closeOnSelect`/キーボード操作・portal・
//!   `lazyMount`: wasm クライアントランタイム側の将来イシューのスコープ
//!   （Popover/Tooltip と共通の判断）。
//!
//! 位置決めロジック（Floating UI 相当の placement / `sameWidth` / CSS 変数
//! 出力）は本イシュー（#540）時点ではスコープ外だったが、イシュー #590
//! （親 #588）で [`crate::positioning`] として実装済みである。詳細は
//! [`positioner`] の doc を参照。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{
    aria_controls, aria_disabled, aria_expanded, aria_haspopup, aria_labelledby, aria_orientation,
    role, AriaPopup,
};
use crate::data_attrs::{data_disabled, data_highlighted, data_state, Orientation};
use crate::state::{Disclosure, DisclosureAction, OpenState};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// Menu の anatomy（`data-scope="menu"`）。
const ANATOMY: Anatomy = anatomy("menu");

/// Root パーツ（`div`）。開閉状態を `data-*` へ反映する。
#[must_use]
pub fn root<'a>(state: OpenState, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Trigger パーツ（`button`）。
///
/// フォーム内配置時の意図しない submit を防ぐため `type="button"` を固定で
/// 付与する（A05 セキュリティ設定ミス対策、Popover/Collapsible の `trigger`
/// と同判断）。`aria-haspopup="menu"` を固定付与し、`controls` が `Some` の
/// とき `aria-controls` で [`content`] と関連付ける。`disabled` はネイティブ
/// `disabled` 存在属性と `data-disabled` の両方へ反映する。
#[must_use]
pub fn trigger<'a>(
    state: OpenState,
    disabled: bool,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        aria_haspopup(AriaPopup::Menu),
        aria_expanded(state.is_open()),
        data_state(state.as_data_state()),
    ];
    if let Some(id) = controls {
        merged.push(aria_controls(id));
    }
    merged.extend(data_disabled(disabled));
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("trigger", "button", merged, children)
}

/// Indicator パーツ（`span`）。開閉状態のみを `data-state` へ反映する
/// 最小主義な装飾用パーツ（アイコン等は呼び出し側の `attrs`/`children` が担う。
/// Popover/Collapsible の `indicator` と同じ最小主義に揃える）。
#[must_use]
pub fn indicator<'a>(
    state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(attrs);
    ANATOMY.part("indicator", "span", merged, children)
}

/// Positioner パーツ（`div`）。位置決めロジックのコンテナ。開閉状態を
/// `data-*` へ反映する。placement 計算自体は
/// [`crate::positioning::compute_position`]（#590）が担い、算出された
/// `style`（`--fandhe-*` CSS 変数）・`data-side`/`data-align` は呼び出し側が
/// `attrs` 経由で渡す（`fandhe-frontend-wasm-full` の `position` モジュールが
/// 実 DOM 計測を行ったうえで計算する。本関数自体は `web-sys` 非依存の
/// ままである）。
///
/// anatomy 上 [`arrow`]/[`arrow_tip`] は [`content`] と並んで本パーツ内に
/// 配置される想定であり、closed のとき `hidden` 存在属性を本パーツへ付与
/// することで、[`content`] だけでなく arrow 等のポインタ層も含めて
/// SSR/no-JS マークアップから隠す（[`crate::popover::Popover`] の
/// `positioner` と同じ判断、イシュー #532 レビュー指摘を継承）。
#[must_use]
pub fn positioner<'a>(
    state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("positioner", "div", merged, children)
}

/// Content パーツ（`div`）。
///
/// `role="menu"` を固定付与する。`id`/`labelledby` が `Some` のとき
/// [`trigger`] の `id`/`controls` と対で関連付ける想定である。closed のとき
/// `hidden` 存在属性を付与し、JS なしの SSR でも閉状態を表現する
/// （Popover の `content` と同じ判断）。
#[must_use]
pub fn content<'a>(
    state: OpenState,
    id: Option<&'a str>,
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("menu"), data_state(state.as_data_state())];
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if let Some(labelledby) = labelledby {
        merged.push(aria_labelledby(labelledby));
    }
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("content", "div", merged, children)
}

/// Arrow パーツ（`div`）。視覚的なポインター要素。anatomy 属性のみを付与する
/// 装飾用パーツ。
#[must_use]
pub fn arrow<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("arrow", "div", attrs, children)
}

/// ArrowTip パーツ（`div`）。`data-part="arrow-tip"`（ark-ui 準拠の kebab-case）。
#[must_use]
pub fn arrow_tip<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("arrow-tip", "div", attrs, children)
}

/// Item パーツ（`div`）。個々のアクション項目。
///
/// `role="menuitem"` を固定付与する。`value` は動的値だが `render()` の
/// 既定エスケープを必ず経由する（`data-value` の属性名はリテラル固定・
/// 値のみ動的）。`disabled` は `aria-disabled="true"` と `data-disabled` の
/// 両方へ反映する（native `disabled` 存在属性を持たない `div` ベースの
/// ため、ネイティブ属性ではなく ARIA/`data-*` のみで無効状態を表現する）。
/// `highlighted`（キーボードナビゲーションのフォーカス位置）はクライアント
/// ランタイムの領域だが、SSR でも `data-highlighted` を出力できるよう
/// `bool` 引数として受ける（状態機械には持たせない。ark-ui でも highlight
/// は開閉状態と独立）。
#[must_use]
pub fn item<'a>(
    value: &'a str,
    disabled: bool,
    highlighted: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("menuitem"), ("data-value", value)];
    if disabled {
        merged.push(aria_disabled(true));
        merged.extend(data_disabled(true));
    }
    merged.extend(data_highlighted(highlighted));
    merged.extend(attrs);
    ANATOMY.part("item", "div", merged, children)
}

/// ItemGroup パーツ（`div`）。関連する [`item`] 群をまとめるコンテナ。
///
/// `role="group"` を固定付与する。`labelledby` が `Some` のとき
/// [`item_group_label`] の `id` と対で `aria-labelledby` 関連付けを成立させる。
/// `data-part="item-group"`（ark-ui 準拠の kebab-case）。
#[must_use]
pub fn item_group<'a>(
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("group")];
    if let Some(labelledby) = labelledby {
        merged.push(aria_labelledby(labelledby));
    }
    merged.extend(attrs);
    ANATOMY.part("item-group", "div", merged, children)
}

/// ItemGroupLabel パーツ（`div`）。[`item_group`] の見出し。
///
/// `id` が `Some` のとき [`item_group`] の `labelledby` と対で関連付ける。
/// `data-part="item-group-label"`（ark-ui 準拠の kebab-case）。
#[must_use]
pub fn item_group_label<'a>(
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("item-group-label", "div", merged, children)
}

/// Separator パーツ（`hr`）。項目群の視覚的な区切り。
///
/// `role="separator"`・`aria-orientation="horizontal"`
/// （[`crate::data_attrs::Orientation`] 経由で `data-orientation`/
/// `aria-orientation` の値語彙を共用する既存不変条件を継承）を固定付与する。
#[must_use]
pub fn separator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![role("separator"), aria_orientation(Orientation::Horizontal)];
    merged.extend(attrs);
    ANATOMY.part("separator", "hr", merged, children)
}

/// TriggerItem パーツ（`div`）。サブメニューを開く項目（イシュー #598）。
///
/// [`item`] と同型（`div` ベース、native `disabled` を持たない）だが、
/// `role="menuitem"` に加えて `aria-haspopup="menu"` を固定付与し、親
/// [`trigger`] と同じ「haspopup 連鎖」でネストしたメニューであることを
/// 支援技術へ伝える。`aria-expanded`/`data-state` は**このトリガーが開閉する
/// サブメニュー側**の `sub_state` から導出する（呼び出し側は子 `Menu`
/// インスタンスの状態をここへ注入する。[`Menu::trigger_item`] は子
/// インスタンスの `self.state()` を自動注入する利便メソッド）。`controls` が
/// `Some` のときサブメニュー [`content`] の `id` と `aria-controls` で
/// 関連付ける。`disabled`/`highlighted` の扱いは [`item`] と同判断。
#[must_use]
pub fn trigger_item<'a>(
    sub_state: OpenState,
    disabled: bool,
    highlighted: bool,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("menuitem"),
        aria_haspopup(AriaPopup::Menu),
        aria_expanded(sub_state.is_open()),
        data_state(sub_state.as_data_state()),
    ];
    if let Some(id) = controls {
        merged.push(aria_controls(id));
    }
    if disabled {
        merged.push(aria_disabled(true));
        merged.extend(data_disabled(true));
    }
    if highlighted {
        merged.push(("data-highlighted", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("trigger-item", "div", merged, children)
}

/// ContextTrigger パーツ（`button`）。右クリックで Menu を開くトリガー
/// （イシュー #598）。
///
/// [`trigger`] と同じくフォーム内配置時の意図しない submit を防ぐため
/// `type="button"` を固定付与し、`data-state` で開閉状態を反映する。
/// **ARIA 属性は一切付与しない**（`aria-haspopup`/`aria-expanded` 等）。
/// 右クリックというジェスチャは SSR/no-JS では成立せず、ARIA を付けると
/// JS なしでは実現できない操作性を支援技術へ誤って約束することになるため、
/// 受け入れ条件どおり `data-*` フックのみで表現する（クライアント配線時に
/// wasm ランタイム側で ARIA 拡張するかどうかは #580/PR #611 系列で再検討）。
#[must_use]
pub fn context_trigger<'a>(
    state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![("type", "button"), data_state(state.as_data_state())];
    merged.extend(attrs);
    ANATOMY.part("context-trigger", "button", merged, children)
}

/// [`Disclosure`]（#524）を埋め込んだ Menu の状態機械。
///
/// `data-state` と実際の開閉状態の整合を型レベルで保証する入口として、
/// 状態を持つ各パーツ関数（[`root`]/[`trigger`]/[`indicator`]/
/// [`positioner`]/[`content`]）へ `self.state()` を注入する利便メソッドを
/// 提供する。状態を取らないパーツ（[`arrow`]/[`arrow_tip`]/[`item`]/
/// [`item_group`]/[`item_group_label`]/[`separator`]）は自由関数のみを
/// 提供し、`Menu` のメソッドとしては公開しない（[`crate::popover::Popover`]
/// と同じ切り分け）。SSR での自由関数直接利用（本型を経由しない構成）も
/// 引き続き可能。`Default` は [`OpenState::Closed`]（SSR の状態なし初期
/// 描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Menu {
    disclosure: Disclosure,
}

impl Menu {
    /// 指定した初期状態で Menu を生成する。
    #[must_use]
    pub fn new(initial: OpenState) -> Self {
        Self {
            disclosure: Disclosure::new(initial),
        }
    }

    /// 現在の開閉状態。
    #[must_use]
    pub fn state(&self) -> OpenState {
        self.disclosure.state()
    }

    /// 現在の `data-state` 属性値（`"open"`/`"closed"`）。
    #[must_use]
    pub fn data_state(&self) -> &'static str {
        self.disclosure.data_state()
    }

    /// 開いているかどうか。
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.disclosure.state().is_open()
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        root(self.state(), attrs, children)
    }

    /// [`trigger`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn trigger<'a>(
        &self,
        disabled: bool,
        controls: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        trigger(self.state(), disabled, controls, attrs, children)
    }

    /// [`indicator`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn indicator<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        indicator(self.state(), attrs, children)
    }

    /// [`positioner`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn positioner<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        positioner(self.state(), attrs, children)
    }

    /// [`content`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn content<'a>(
        &self,
        id: Option<&'a str>,
        labelledby: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        content(self.state(), id, labelledby, attrs, children)
    }

    /// [`trigger_item`] へ現在の状態を注入する利便メソッド。
    ///
    /// **親メニューではなく、このトリガーが開くサブメニュー側の `Menu`
    /// インスタンスから呼ぶ**（`aria-expanded`/`data-state` はサブメニュー
    /// 自身の開閉状態を反映するため。親 `Menu` から呼ぶと親の状態が誤って
    /// サブメニュートリガーへ反映されてしまう）。
    #[must_use]
    pub fn trigger_item<'a>(
        &self,
        disabled: bool,
        highlighted: bool,
        controls: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        trigger_item(
            self.state(),
            disabled,
            highlighted,
            controls,
            attrs,
            children,
        )
    }

    /// [`context_trigger`] へ現在の状態を注入する利便メソッド。
    ///
    /// 右クリックで開かれる Menu 自身のインスタンスから呼ぶ。
    #[must_use]
    pub fn context_trigger<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        context_trigger(self.state(), attrs, children)
    }
}

impl Component for Menu {
    type Action = DisclosureAction;

    fn update(&mut self, action: DisclosureAction) {
        self.disclosure.update(action);
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > trigger + positioner > content、children 空・id なし）。
    /// [`Disclosure::view`]・Popover の `view` と同じ位置付けであり、
    /// 公開 UI としての利用は想定しない（実際の UI 構築は §パーツ関数群を
    /// 呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        let state = self.state();
        self.root(
            Vec::new(),
            vec![
                trigger(state, false, None, Vec::new(), Vec::new()),
                positioner(
                    state,
                    Vec::new(),
                    vec![content(state, None, None, Vec::new(), Vec::new())],
                ),
            ],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<DisclosureAction> {
        Disclosure::decode_action(name, payload)
    }
}

impl Hydrate for Menu {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.disclosure.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            disclosure: Disclosure::from_hydration_attrs(attrs)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- positioning（#590）接続 ---

    #[test]
    fn positioner_accepts_computed_style_and_placement_attrs_via_attrs() {
        use crate::positioning::{
            compute_position, css_vars_style, placement_attrs, Align, Placement, PositioningConfig,
            Rect, Side, Size,
        };

        let anchor = Rect {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 20.0,
        };
        let floating = Size {
            width: 200.0,
            height: 80.0,
        };
        let viewport = Size {
            width: 800.0,
            height: 600.0,
        };
        let config = PositioningConfig {
            placement: Placement::new(Side::Bottom, Align::Start),
            offset: 0.0,
            flip: true,
            shift: true,
            same_width: true,
        };
        let resolved = compute_position(anchor, floating, viewport, &config, true);
        let style = css_vars_style(&resolved, anchor.width);
        let mut attrs: Vec<(&str, &str)> = vec![("style", &style)];
        attrs.extend(placement_attrs(resolved.placement));

        let html = render(&positioner(OpenState::Open, attrs, vec![]));
        assert!(html.contains("--fandhe-reference-width:"));
        assert!(html.contains(r#"data-side="bottom""#));
        assert!(html.contains(r#"data-align="start""#));
    }

    // --- 各パーツの data-scope/data-part 出力 ---

    #[test]
    fn root_outputs_scope_part_and_state() {
        let html = render(&root(OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="menu""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="closed""#));
    }

    #[test]
    fn trigger_has_type_button_haspopup_menu_and_aria_expanded() {
        let html = render(&trigger(OpenState::Closed, false, None, vec![], vec![]));
        assert!(html.contains(r#"<button"#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-haspopup="menu""#));
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(!html.contains("aria-controls"));
        assert!(!html.contains("disabled"));

        let html_open = render(&trigger(OpenState::Open, false, None, vec![], vec![]));
        assert!(html_open.contains(r#"aria-expanded="true""#));
    }

    #[test]
    fn trigger_controls_some_outputs_aria_controls() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            Some("menu-content-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-controls="menu-content-1""#));
    }

    #[test]
    fn trigger_disabled_true_adds_native_and_data_disabled() {
        let html = render(&trigger(OpenState::Closed, true, None, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"disabled="""#));
    }

    #[test]
    fn trigger_disabled_false_omits_both_disabled_attrs() {
        let html = render(&trigger(OpenState::Closed, false, None, vec![], vec![]));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains(r#" disabled"#));
    }

    #[test]
    fn indicator_outputs_scope_part_and_state_only() {
        let html = render(&indicator(OpenState::Open, vec![], vec![text("+")]));
        assert!(html.contains(r#"data-scope="menu""#));
        assert!(html.contains(r#"data-part="indicator""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains('+'));
    }

    #[test]
    fn positioner_outputs_scope_part_and_state() {
        let html = render(&positioner(OpenState::Open, vec![], vec![]));
        assert!(html.contains(r#"data-part="positioner""#));
        assert!(html.contains(r#"data-state="open""#));
    }

    #[test]
    fn positioner_closed_has_hidden_attr_open_does_not() {
        let closed = render(&positioner(OpenState::Closed, vec![], vec![]));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&positioner(OpenState::Open, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn content_has_role_menu_and_state() {
        let html = render(&content(OpenState::Open, None, None, vec![], vec![]));
        assert!(html.contains(r#"role="menu""#));
        assert!(html.contains(r#"data-state="open""#));
    }

    #[test]
    fn content_closed_has_hidden_attr_open_does_not() {
        let closed = render(&content(OpenState::Closed, None, None, vec![], vec![]));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&content(OpenState::Open, None, None, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn content_id_labelledby_some_outputs_both() {
        let html = render(&content(
            OpenState::Open,
            Some("content-1"),
            Some("trigger-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"id="content-1""#));
        assert!(html.contains(r#"aria-labelledby="trigger-1""#));
    }

    #[test]
    fn arrow_outputs_scope_and_part_only() {
        let html = render(&arrow(vec![], vec![]));
        assert!(html.contains(r#"data-part="arrow""#));
    }

    #[test]
    fn arrow_tip_outputs_kebab_case_part() {
        let html = render(&arrow_tip(vec![], vec![]));
        assert!(html.contains(r#"data-part="arrow-tip""#));
    }

    #[test]
    fn item_has_role_menuitem_and_data_value() {
        let html = render(&item("item-1", false, false, vec![], vec![text("Item 1")]));
        assert!(html.contains(r#"role="menuitem""#));
        assert!(html.contains(r#"data-value="item-1""#));
        assert!(!html.contains("aria-disabled"));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("data-highlighted"));
    }

    #[test]
    fn item_disabled_true_adds_aria_disabled_and_data_disabled() {
        let html = render(&item("item-1", true, false, vec![], vec![]));
        assert!(html.contains(r#"aria-disabled="true""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_highlighted_true_adds_data_highlighted_false_omits() {
        let highlighted = render(&item("item-1", false, true, vec![], vec![]));
        assert!(highlighted.contains(r#"data-highlighted="""#));

        let not_highlighted = render(&item("item-1", false, false, vec![], vec![]));
        assert!(!not_highlighted.contains("data-highlighted"));
    }

    #[test]
    fn item_group_has_role_group_and_kebab_case_part() {
        let html = render(&item_group(None, vec![], vec![]));
        assert!(html.contains(r#"role="group""#));
        assert!(html.contains(r#"data-part="item-group""#));
        assert!(!html.contains("aria-labelledby"));
    }

    #[test]
    fn item_group_labelledby_some_outputs_aria_labelledby() {
        let html = render(&item_group(Some("label-1"), vec![], vec![]));
        assert!(html.contains(r#"aria-labelledby="label-1""#));
    }

    #[test]
    fn item_group_label_id_some_outputs_id_and_kebab_case_part() {
        let html = render(&item_group_label(
            Some("label-1"),
            vec![],
            vec![text("Group")],
        ));
        assert!(html.contains(r#"id="label-1""#));
        assert!(html.contains(r#"data-part="item-group-label""#));
    }

    #[test]
    fn item_group_and_label_correspond() {
        let group_html = render(&item_group(Some("g1"), vec![], vec![]));
        let label_html = render(&item_group_label(Some("g1"), vec![], vec![]));
        assert!(group_html.contains(r#"aria-labelledby="g1""#));
        assert!(label_html.contains(r#"id="g1""#));
    }

    #[test]
    fn separator_has_hr_tag_role_and_aria_orientation() {
        let html = render(&separator(vec![], vec![]));
        assert!(html.contains(r#"<hr"#));
        assert!(html.contains(r#"role="separator""#));
        assert!(html.contains(r#"aria-orientation="horizontal""#));
    }

    // --- trigger_item（サブメニュートリガー、イシュー #598） ---

    #[test]
    fn trigger_item_has_kebab_case_part_role_and_haspopup_menu() {
        let html = render(&trigger_item(
            OpenState::Closed,
            false,
            false,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-part="trigger-item""#));
        assert!(html.contains(r#"role="menuitem""#));
        assert!(html.contains(r#"aria-haspopup="menu""#));
    }

    #[test]
    fn trigger_item_aria_expanded_and_data_state_follow_sub_state() {
        let closed = render(&trigger_item(
            OpenState::Closed,
            false,
            false,
            None,
            vec![],
            vec![],
        ));
        assert!(closed.contains(r#"aria-expanded="false""#));
        assert!(closed.contains(r#"data-state="closed""#));

        let open = render(&trigger_item(
            OpenState::Open,
            false,
            false,
            None,
            vec![],
            vec![],
        ));
        assert!(open.contains(r#"aria-expanded="true""#));
        assert!(open.contains(r#"data-state="open""#));
    }

    #[test]
    fn trigger_item_controls_some_outputs_aria_controls_none_omits() {
        let with_controls = render(&trigger_item(
            OpenState::Closed,
            false,
            false,
            Some("sub-1"),
            vec![],
            vec![],
        ));
        assert!(with_controls.contains(r#"aria-controls="sub-1""#));

        let without_controls = render(&trigger_item(
            OpenState::Closed,
            false,
            false,
            None,
            vec![],
            vec![],
        ));
        assert!(!without_controls.contains("aria-controls"));
    }

    #[test]
    fn trigger_item_disabled_true_adds_aria_disabled_and_data_disabled_no_native() {
        let html = render(&trigger_item(
            OpenState::Closed,
            true,
            false,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-disabled="true""#));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(!html.contains(r#" disabled"#));

        let not_disabled = render(&trigger_item(
            OpenState::Closed,
            false,
            false,
            None,
            vec![],
            vec![],
        ));
        assert!(!not_disabled.contains("aria-disabled"));
        assert!(!not_disabled.contains("data-disabled"));
    }

    #[test]
    fn trigger_item_highlighted_true_adds_data_highlighted_false_omits() {
        let highlighted = render(&trigger_item(
            OpenState::Closed,
            false,
            true,
            None,
            vec![],
            vec![],
        ));
        assert!(highlighted.contains(r#"data-highlighted="""#));

        let not_highlighted = render(&trigger_item(
            OpenState::Closed,
            false,
            false,
            None,
            vec![],
            vec![],
        ));
        assert!(!not_highlighted.contains("data-highlighted"));
    }

    #[test]
    fn trigger_item_caller_supplied_scope_and_part_are_dropped() {
        let html = render(&trigger_item(
            OpenState::Closed,
            false,
            false,
            None,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="menu""#));
        assert!(html.contains(r#"data-part="trigger-item""#));
        assert!(!html.contains("attacker"));
    }

    // --- context_trigger（右クリックトリガー、イシュー #598） ---

    #[test]
    fn context_trigger_has_button_tag_type_button_and_kebab_case_part() {
        let html = render(&context_trigger(OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"<button"#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"data-part="context-trigger""#));
        assert!(html.contains(r#"data-state="closed""#));
    }

    #[test]
    fn context_trigger_has_no_aria_attributes() {
        let closed = render(&context_trigger(OpenState::Closed, vec![], vec![]));
        assert!(!closed.contains("aria-"));

        let open = render(&context_trigger(OpenState::Open, vec![], vec![]));
        assert!(!open.contains("aria-"));
        assert!(open.contains(r#"data-state="open""#));
    }

    #[test]
    fn context_trigger_caller_supplied_scope_and_part_are_dropped() {
        let html = render(&context_trigger(
            OpenState::Closed,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="menu""#));
        assert!(html.contains(r#"data-part="context-trigger""#));
        assert!(!html.contains("attacker"));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            OpenState::Closed,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="menu""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- trigger + content の aria-controls/id 対応 ---

    #[test]
    fn trigger_controls_and_content_id_correspond() {
        let trigger_html = render(&trigger(OpenState::Open, false, Some("c1"), vec![], vec![]));
        let content_html = render(&content(OpenState::Open, Some("c1"), None, vec![], vec![]));
        assert!(trigger_html.contains(r#"aria-controls="c1""#));
        assert!(content_html.contains(r#"id="c1""#));
    }

    // --- Menu: dispatch 統合 ---

    #[test]
    fn menu_default_is_closed() {
        assert_eq!(Menu::default().state(), OpenState::Closed);
    }

    #[test]
    fn menu_dispatch_toggle_changes_data_state() {
        let mut m = Menu::default();
        assert!(render(&m.root(vec![], vec![])).contains(r#"data-state="closed""#));

        assert!(dispatch(&mut m, "toggle", ""));
        assert!(render(&m.root(vec![], vec![])).contains(r#"data-state="open""#));
        assert!(render(&m.trigger(false, None, vec![], vec![])).contains(r#"aria-expanded="true""#));
        assert!(render(&m.positioner(vec![], vec![])).contains(r#"data-state="open""#));
        let content_html = render(&m.content(None, None, vec![], vec![]));
        assert!(content_html.contains(r#"data-state="open""#));
        assert!(!content_html.contains("hidden"));
        assert!(render(&m.indicator(vec![], vec![])).contains(r#"data-state="open""#));
    }

    #[test]
    fn menu_dispatch_open_and_close() {
        let mut m = Menu::default();
        assert!(dispatch(&mut m, "open", ""));
        assert_eq!(m.state(), OpenState::Open);
        assert!(dispatch(&mut m, "close", ""));
        assert_eq!(m.state(), OpenState::Closed);
    }

    #[test]
    fn menu_dispatch_ignores_unknown_action() {
        let mut m = Menu::new(OpenState::Open);
        assert!(!dispatch(&mut m, "no_such_action", "x"));
        assert_eq!(m.state(), OpenState::Open);
    }

    #[test]
    fn menu_trigger_item_and_context_trigger_reflect_own_state() {
        let mut sub = Menu::default();
        assert!(
            render(&sub.trigger_item(false, false, None, vec![], vec![]))
                .contains(r#"aria-expanded="false""#)
        );
        assert!(dispatch(&mut sub, "open", ""));
        assert!(
            render(&sub.trigger_item(false, false, None, vec![], vec![]))
                .contains(r#"aria-expanded="true""#)
        );

        let ctx = Menu::new(OpenState::Open);
        assert!(render(&ctx.context_trigger(vec![], vec![])).contains(r#"data-state="open""#));
    }

    // --- Menu: SSR 状態なし初期描画 ---

    #[test]
    fn menu_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Menu::default().view());
        assert!(rendered.contains(r#"data-state="closed""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- Menu: hydration 経路 ---

    #[test]
    fn menu_hydration_round_trip() {
        let m = Menu::new(OpenState::Open);
        let rendered = render(&render_for_hydration(&m));
        assert!(rendered.contains(r#"data-hydrate-state="open""#));

        let restored = Menu::from_hydration_attrs(&m.hydration_attrs()).unwrap();
        assert_eq!(restored, m);
    }

    #[test]
    fn menu_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Menu::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-state".to_string())
        );
    }

    #[test]
    fn menu_from_hydration_attrs_invalid_value_does_not_panic() {
        for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
            let attrs = vec![("data-hydrate-state".to_string(), bogus.to_string())];
            let err = Menu::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: value/controls/id/labelledby/呼び出し側 attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn item_value_payload_is_escaped_on_render() {
        let html = render(&item(ATTR_BREAK_PAYLOAD, false, false, vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn trigger_controls_payload_is_escaped_on_render() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn content_id_labelledby_payload_is_escaped_on_render() {
        let html = render(&content(
            OpenState::Open,
            Some(ATTR_BREAK_PAYLOAD),
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn item_group_labelledby_and_item_group_label_id_payload_is_escaped_on_render() {
        let group_html = render(&item_group(Some(ATTR_BREAK_PAYLOAD), vec![], vec![]));
        let label_html = render(&item_group_label(Some(ATTR_BREAK_PAYLOAD), vec![], vec![]));
        assert!(!group_html.contains("onmouseover=\"alert(1)"));
        assert!(!label_html.contains("onmouseover=\"alert(1)"));
        assert!(group_html.contains("&quot;"));
        assert!(label_html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            OpenState::Closed,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn trigger_item_controls_payload_is_escaped_on_render() {
        let html = render(&trigger_item(
            OpenState::Closed,
            false,
            false,
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn trigger_item_and_context_trigger_caller_attrs_payload_is_escaped_on_render() {
        let trigger_item_html = render(&trigger_item(
            OpenState::Closed,
            false,
            false,
            None,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        let context_trigger_html = render(&context_trigger(
            OpenState::Closed,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!trigger_item_html.contains("onmouseover=\"alert(1)"));
        assert!(!context_trigger_html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn trigger_item_and_context_trigger_children_text_is_escaped_on_render() {
        let trigger_item_html = render(&trigger_item(
            OpenState::Closed,
            false,
            false,
            None,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        let context_trigger_html = render(&context_trigger(
            OpenState::Closed,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!trigger_item_html.contains("<script>alert(1)</script>"));
        assert!(!context_trigger_html.contains("<script>alert(1)</script>"));
        assert!(trigger_item_html.contains("&lt;script&gt;"));
        assert!(context_trigger_html.contains("&lt;script&gt;"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&item(
            "item-1",
            false,
            false,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn menu_xss_payload_in_hydration_state_is_rejected_not_rendered() {
        // data-hydrate-state はサーバーが state.as_data_state() から生成する
        // 固定語彙のみを出力するため攻撃者が任意値を注入する経路はないが、
        // クライアント改ざん入力の復元経路（from_hydration_attrs）が
        // 未知値を拒否することを Menu 経由でも固定する。
        let attrs = vec![(
            "data-hydrate-state".to_string(),
            "<script>alert(1)</script>".to_string(),
        )];
        let err = Menu::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}
