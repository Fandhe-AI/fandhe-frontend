//! Toolbar（ボタン・セパレータ・ToggleGroup の横方向グループ化）headless
//! コンポーネント（イシュー #991、親 #932 Phase 8、祖父トラッキング #924）。
//!
//! `docs/design/component-coverage-map.md` §5 Part D・§9 で「実装対象・
//! 対応 mod なし」と確定していた Radix Primitives `Toolbar` 相当を埋める。
//! Root / Button / Link / Separator / ToggleGroup / ToggleItem の 6 anatomy
//! パーツと、roving tabindex（左右矢印でのフォーカス移動）の**状態機械と
//! 属性出力のみ**を提供する [`Toolbar`] を提供する。実 DOM のキー配線は
//! [`crate::carousel::Carousel`] と同様に `fandhe-frontend-wasm-full` の
//! 責務でありスコープ外（本モジュール doc 「スコープ外」節参照）。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`button`]/[`link`]/[`separator`]/
//! [`toggle_group`]/[`toggle_item`]、いずれも純粋関数で完結）を直接呼んで
//! 組み立てる。CSR/hydration は [`Toolbar`]（
//! [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"next"`/`"prev"`/`"first"`/`"last"`/`"focus"`）で focus index を遷移
//! する。`fandhe-frontend-pre-styled-ui` が本モジュールを呼んでスタイル済み
//! Toolbar を組み立てる想定である。
//!
//! # ToggleGroup / ToggleItem を再エクスポートしない理由
//!
//! [`crate::toggle_group`] の `data-scope="toggle-group"` をそのまま
//! toolbar 内へ持ち込むと、pre-styled-ui 側の `SlotRecipe::new("toolbar",
//! SLOTS)` が生成するセレクタ（`[data-scope="toolbar"][data-part="…"]`）から
//! 到達できなくなる。したがって toolbar 専用の `toggle-group`/`toggle-item`
//! anatomy パーツを新設しつつ、押下状態の語彙・状態機械は既存実装へ完全に
//! 委譲することで重複実装を避ける（[`crate::aria::aria_pressed`]/
//! [`crate::state::pressed_data_state`]/[`crate::data_attrs::data_pressed`]/
//! [`crate::data_attrs::data_disabled`] をそのまま使い、独自の `data-state`
//! 語彙を作らない）。押下管理そのものが必要な呼び出し側向けに
//! [`crate::toggle_group::ToggleGroup`]/[`crate::toggle_group::MultiToggleGroup`]
//! を本モジュールから再エクスポートする（新規状態機械は追加しない）。
//!
//! # roving tabindex の状態機械（[`crate::carousel::Carousel`] を雛形）
//!
//! [`Toolbar`] は `focused`（現在フォーカス対象の index）・`item_count`・
//! `loop_focus`・`orientation` の複合フィールドを持ち、
//! [`crate::carousel::Carousel`] の `normalize_index` と同型の fail-closed
//! 正規化を行う（[`normalize_focus`]）。disabled 項目もフォーカス順序から
//! 除外しない（WAI-ARIA APG の toolbar パターン推奨に従う意図的な設計
//! 判断。skip-disabled モードは本イシューのスコープ外、後述「スコープ外」
//! 節）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`role`/`aria-*`/`data-*`/`type`/`tabindex`）はすべて
//!   `&'static str` リテラルで固定しており、動的値が属性名スロットへ混入
//!   する経路はない（[`mod@crate::anatomy`]/[`crate::aria`]/
//!   [`crate::data_attrs`] の既存不変条件をそのまま継承する）。
//! - 動的値（`label`/`value`/`href`/呼び出し側 `attrs`/`children`）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する
//!   （REQ-1）。`raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - **呼び出し側 `tabindex` 偽装の除去**: [`drop_tabindex_attr`] が
//!   [`crate::skip_nav::content`] と同型のパターンで呼び出し側 `attrs` から
//!   `tabindex`（大文字小文字を無視）を除去してから
//!   `tabindex="0"`/`tabindex="-1"` を合成する。[`button`]/[`link`]/
//!   [`toggle_item`] のすべての focusable パーツへ適用し、roving tabindex が
//!   呼び出し側の偽装によって非決定にならないことを保証する。
//! - **`type="button"` の固定**: [`button`]/[`toggle_item`] はフォーム内
//!   配置時の意図しない submit を防ぐため `type="button"` を固定付与する
//!   （[`crate::action_bar::selection_trigger`] と同じ判断）。
//! - **reverse tabnabbing 対策**: [`link`] は [`crate::link::root`] の
//!   「`external` 時に `target="_blank"` と `rel="noopener noreferrer"` を
//!   不可分に付与する」実装へ完全委譲する（独自の付与ロジックを再導出
//!   しない）。
//! - `decode_action` は既知アクション名（`"next"`/`"prev"`/`"first"`/
//!   `"last"`/`"focus"`）以外を `None` にする（fail-closed）。`"focus"` の
//!   payload は `usize` の厳密パースで fail-closed（パース不能は `None`）。
//! - hydration 属性（`data-hydrate-focused`/`-item-count`/`-loop`/
//!   `-orientation`）はクライアント側で改ざんされうる入力として扱う。
//!   欠落は [`fandhe_frontend_interactive::HydrateError::MissingAttr`]、
//!   パース不能・範囲外 `focused`・不正な `loop`/`orientation` 語彙は
//!   [`fandhe_frontend_interactive::HydrateError::InvalidValue`] を返す
//!   （panic しない。[`crate::carousel::Carousel`] と同型の fail-closed
//!   契約）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - 矢印キー・Home/End の実 DOM 配線とフォーカス移動
//!   （`fandhe-frontend-wasm-full` の責務）。
//! - roving focus の skip-disabled モード（disabled 項目をフォーカス順序
//!   から除外する挙動）。本モジュールは APG 推奨の「disabled もフォーカス
//!   可能」のみを提供する。
//! - `loopFocus` の CSS/視覚表現、Toolbar 内オーバーフロー時のスクロール・
//!   折りたたみ。
//! - headless-ui 側への汎用 `separator` mod の切り出し（[`crate::action_bar`]/
//!   [`crate::menu`]/[`crate::steps`]/本モジュールが各自 scope でセパレータを
//!   持つ重複は横断リファクタとして別 Issue 相当）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_label, aria_orientation, aria_pressed, role};
use crate::data_attrs::{data_disabled, data_orientation, data_pressed, data_state, Orientation};
use crate::link;
use crate::state::pressed_data_state;
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

// 押下管理が必要な呼び出し側は既存の状態機械をそのまま使う（本モジュール
// doc「ToggleGroup / ToggleItem を再エクスポートしない理由」参照。ここで
// 再エクスポートするのは型のみであり、`toggle-group`/`toggle-item` の
// anatomy パーツ自体は本モジュールが独自に持つ）。
pub use crate::toggle_group::{MultiToggleGroup, ToggleGroup};

/// Toolbar の anatomy（`data-scope="toolbar"`）。
const ANATOMY: Anatomy = anatomy("toolbar");

/// 呼び出し側 `attrs` から `tabindex`（大文字小文字を無視）を除去する
/// （[`crate::skip_nav::content`] と同型のパターン）。focusable な
/// [`button`]/[`link`]/[`toggle_item`] が共通で使うヘルパ。
fn drop_tabindex_attr<'a>(attrs: Vec<(&'a str, &'a str)>) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("tabindex"))
        .collect()
}

/// focus 対象なら `tabindex="0"`、そうでなければ `tabindex="-1"` を返す
/// roving tabindex の共通ヘルパ。
fn roving_tabindex(focused: bool) -> (&'static str, &'static str) {
    if focused {
        ("tabindex", "0")
    } else {
        ("tabindex", "-1")
    }
}

/// Root パーツ（`div`）。`role="toolbar"` + `aria-orientation` +
/// `data-orientation` を固定出力する。`label` は動的値であり
/// [`fandhe_frontend_core::render`] の既定エスケープを経由して
/// `aria-label` へ出力する（空文字列のときは省略する）。
#[must_use]
pub fn root<'a>(
    orientation: Orientation,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("toolbar"),
        aria_orientation(orientation),
        data_orientation(orientation),
    ];
    if !label.is_empty() {
        merged.push(aria_label(label));
    }
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Button パーツ（`button`）。フォーム内配置時の意図しない submit を防ぐ
/// ため `type="button"` を固定で付与する。`focused` が `true` のとき
/// `tabindex="0"`、そうでなければ `tabindex="-1"`（roving tabindex）。
/// `disabled` は `data-disabled`/`aria-disabled` で表現し、ネイティブ
/// `disabled` 属性は付与しない（disabled 項目もフォーカス順序に残す設計、
/// モジュール doc「スコープ外」節参照）。
#[must_use]
pub fn button<'a>(
    focused: bool,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button"), roving_tabindex(focused)];
    if disabled {
        merged.push(("aria-disabled", "true"));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(drop_tabindex_attr(attrs));
    ANATOMY.part("button", "button", merged, children)
}

/// Link パーツ（`a`）。[`crate::link::root`] へ完全委譲する
/// （reverse tabnabbing 対策の不可分付与ロジックを再導出しない、モジュール
/// doc「セキュリティ不変条件」参照）。`focused` が `true` のとき
/// `tabindex="0"`、そうでなければ `tabindex="-1"`。
///
/// [`crate::link::root`] が組み立てた要素（`data-scope="link"`/
/// `data-part="root"` 付き）から属性・子ノードのみを引き継ぎ、
/// [`ANATOMY`] 側の `toolbar`/`link` セレクタへ再構成する（`href` の URL
/// スキーム検証・エスケープはすべて委譲先の既存経路がそのまま担う）。
#[must_use]
pub fn link<'a>(
    focused: bool,
    href: &'a str,
    external: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![roving_tabindex(focused)];
    merged.extend(drop_tabindex_attr(attrs));
    let inner = link::root(href, external, false, merged, children);
    match inner {
        Node::Element {
            tag,
            attrs,
            children,
        } => {
            let filtered: Vec<(String, String)> = attrs
                .into_iter()
                .filter(|(k, _)| k != "data-scope" && k != "data-part")
                .collect();
            let filtered: Vec<(&str, &str)> = filtered
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            ANATOMY.part("link", tag, filtered, children)
        }
        // link::root は必ず Node::Element を返す（危険な URL スキームが
        // 拒否される場合も href 属性のみが省略され、要素自体は Element の
        // ままである。crates/headless-ui/src/link.rs のテスト参照）。
        // 契約が破られた場合に備え、非 Element はそのまま返す fail-closed
        // フォールバックを残す。
        other => other,
    }
}

/// Separator パーツ（`div`）。`role="separator"` +
/// toolbar 自身の向きと**直交**する `aria-orientation` を固定出力する
/// （横向き toolbar のセパレータは縦線になるため `vertical`、
/// [`crate::action_bar::separator`] と同じ判断）。
#[must_use]
pub fn separator<'a>(
    toolbar_orientation: Orientation,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let orthogonal = match toolbar_orientation {
        Orientation::Horizontal => Orientation::Vertical,
        Orientation::Vertical => Orientation::Horizontal,
    };
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("separator"), aria_orientation(orthogonal)];
    merged.extend(attrs);
    ANATOMY.part("separator", "div", merged, children)
}

/// ToggleGroup パーツ（`div`）。`role="group"` のみを付与する
/// （`role="group"` には `aria-orientation` が許可されていないため付与
/// しない。[`crate::toggle_group::root`] の PR #791 Bugbot 指摘と同じ
/// 判断）。
#[must_use]
pub fn toggle_group<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("group")];
    merged.extend(attrs);
    ANATOMY.part("toggle-group", "div", merged, children)
}

/// ToggleItem パーツ（`button`）。押下状態の語彙は
/// [`crate::state::pressed_data_state`]/[`crate::aria::aria_pressed`]/
/// [`crate::data_attrs::data_pressed`] へ完全委譲し、本モジュールで独自の
/// `data-state` 語彙を作らない。`value` は動的値のまま `data-value` として
/// 出力し、[`fandhe_frontend_core::render`] の既定エスケープを必ず経由する
/// （REQ-1）。`focused` が `true` のとき `tabindex="0"`、そうでなければ
/// `tabindex="-1"`。
#[must_use]
pub fn toggle_item<'a>(
    pressed: bool,
    focused: bool,
    disabled: bool,
    value: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        roving_tabindex(focused),
        aria_pressed(pressed),
        data_state(pressed_data_state(pressed)),
        ("data-value", value),
    ];
    if disabled {
        merged.push(("aria-disabled", "true"));
    }
    merged.extend(data_pressed(pressed));
    merged.extend(data_disabled(disabled));
    merged.extend(drop_tabindex_attr(attrs));
    ANATOMY.part("toggle-item", "button", merged, children)
}

/// Toolbar のアクション（WASM 境界の文字列 dispatch と
/// [`Toolbar::decode_action`] で接続する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    /// 次の項目へフォーカスを進める（末尾かつ `loop_focus` 無効なら
    /// no-op）。
    Next,
    /// 前の項目へフォーカスを戻す（先頭かつ `loop_focus` 無効なら
    /// no-op）。
    Prev,
    /// 先頭項目へフォーカスを移動する。
    First,
    /// 末尾項目へフォーカスを移動する。
    Last,
    /// 指定した index の項目へ直接フォーカスを移動する（`index >=
    /// item_count` は no-op）。
    Focus(usize),
}

/// `focused >= item_count`（または `item_count == 0` で `focused != 0`）を
/// `0` へ fail-closed に正規化する（[`crate::carousel::Carousel`] の
/// `normalize_index` と同型のヘルパ、[`Toolbar::new`]/hydration 復元で
/// 使う）。
fn normalize_focus(focused: usize, item_count: usize) -> usize {
    if item_count == 0 || focused >= item_count {
        0
    } else {
        focused
    }
}

/// Toolbar の roving tabindex 状態機械（[`crate::carousel::Carousel`] を
/// 雛形とする index + count + loop + orientation の複合フィールド）。
///
/// `Default` は `focused=0, item_count=0, loop_focus=false,
/// orientation=Horizontal`（SSR の初期描画に対応する既定値。項目を持たない
/// 空 toolbar）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Toolbar {
    focused: usize,
    item_count: usize,
    loop_focus: bool,
    orientation: Orientation,
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new(0, 0, false, Orientation::Horizontal)
    }
}

impl Toolbar {
    /// `data-hydrate-focused` 属性名のフィールド部分。
    pub const FIELD_FOCUSED: &'static str = "focused";
    /// `data-hydrate-item-count` 属性名のフィールド部分。
    pub const FIELD_ITEM_COUNT: &'static str = "item-count";
    /// `data-hydrate-loop` 属性名のフィールド部分。
    pub const FIELD_LOOP: &'static str = "loop";
    /// `data-hydrate-orientation` 属性名のフィールド部分。
    pub const FIELD_ORIENTATION: &'static str = "orientation";

    /// 指定した状態で [`Toolbar`] を生成する（[`normalize_focus`] で
    /// fail-closed 正規化する。呼び出し側の不正な `focused` で panic
    /// しない）。
    #[must_use]
    pub fn new(
        focused: usize,
        item_count: usize,
        loop_focus: bool,
        orientation: Orientation,
    ) -> Self {
        Self {
            focused: normalize_focus(focused, item_count),
            item_count,
            loop_focus,
            orientation,
        }
    }

    /// 現在フォーカス対象の index（`0`-origin）。
    #[must_use]
    pub fn focused(&self) -> usize {
        self.focused
    }

    /// 項目総数。
    #[must_use]
    pub fn item_count(&self) -> usize {
        self.item_count
    }

    /// 端で循環するかどうか。
    #[must_use]
    pub fn is_loop_focus(&self) -> bool {
        self.loop_focus
    }

    /// 現在の向き（`data-orientation`/hydration ラウンドトリップの対象）。
    #[must_use]
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// 指定 index が現在のフォーカス対象かどうか。
    #[must_use]
    pub fn is_focused(&self, index: usize) -> bool {
        self.item_count != 0 && index == self.focused
    }

    /// [`root`] へ現在の向きを注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        label: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.orientation, label, attrs, children)
    }

    /// [`button`] へ [`Self::is_focused`] の判定を注入する利便メソッド。
    #[must_use]
    pub fn button<'a>(
        &self,
        index: usize,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        button(self.is_focused(index), disabled, attrs, children)
    }

    /// [`link`] へ [`Self::is_focused`] の判定を注入する利便メソッド。
    #[must_use]
    pub fn link<'a>(
        &self,
        index: usize,
        href: &'a str,
        external: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        link(self.is_focused(index), href, external, attrs, children)
    }

    /// [`separator`] へ現在の向きを注入する利便メソッド。
    #[must_use]
    pub fn separator<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        separator(self.orientation, attrs, children)
    }

    /// [`toggle_item`] へ [`Self::is_focused`] の判定を注入する利便
    /// メソッド。押下状態は呼び出し側が明示的に渡す
    /// （[`ToggleGroup`]/[`MultiToggleGroup`] 等、押下管理を持つ状態機械の
    /// 判定結果を呼び出し側で解決してから渡す想定。モジュール doc
    /// 「ToggleGroup / ToggleItem を再エクスポートしない理由」参照）。
    #[must_use]
    pub fn toggle_item<'a>(
        &self,
        index: usize,
        pressed: bool,
        disabled: bool,
        value: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        toggle_item(
            pressed,
            self.is_focused(index),
            disabled,
            value,
            attrs,
            children,
        )
    }
}

impl Component for Toolbar {
    type Action = ToolbarAction;

    /// `item_count == 0` はすべてのアクションを no-op にする
    /// （[`crate::carousel::Carousel::update`] と同型の判断）。
    fn update(&mut self, action: ToolbarAction) {
        if self.item_count == 0 {
            return;
        }
        match action {
            ToolbarAction::Next => {
                if self.focused + 1 < self.item_count {
                    self.focused += 1;
                } else if self.loop_focus {
                    self.focused = 0;
                }
            }
            ToolbarAction::Prev => {
                if self.focused > 0 {
                    self.focused -= 1;
                } else if self.loop_focus {
                    self.focused = self.item_count - 1;
                }
            }
            ToolbarAction::First => {
                self.focused = 0;
            }
            ToolbarAction::Last => {
                self.focused = self.item_count - 1;
            }
            ToolbarAction::Focus(i) => {
                if i < self.item_count {
                    self.focused = i;
                }
            }
        }
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（root >
    /// button、children 空）。公開 UI としての利用は想定しない
    /// （[`crate::carousel::Carousel::view`] と同じ位置付け）。
    fn view(&self) -> Node {
        self.root(
            "toolbar",
            Vec::new(),
            vec![self.button(0, false, Vec::new(), Vec::new())],
        )
    }

    /// `"next"`/`"prev"`/`"first"`/`"last"`: payload 不使用。`"focus"`:
    /// payload を `str::parse::<usize>()` でパースし、パース不能な場合は
    /// `None`（fail-closed、dispatch は no-op）。範囲外 index（`i >=
    /// item_count`）はここでは弾かず [`Toolbar::update`] 側の no-op に
    /// 委ねる（[`crate::carousel::Carousel::decode_action`] と同型）。
    fn decode_action(name: &str, payload: &str) -> Option<ToolbarAction> {
        match name {
            "next" => Some(ToolbarAction::Next),
            "prev" => Some(ToolbarAction::Prev),
            "first" => Some(ToolbarAction::First),
            "last" => Some(ToolbarAction::Last),
            "focus" => payload.parse::<usize>().ok().map(ToolbarAction::Focus),
            _ => None,
        }
    }
}

impl Hydrate for Toolbar {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_FOCUSED),
                self.focused.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_ITEM_COUNT),
                self.item_count.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_LOOP),
                self.loop_focus.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_ORIENTATION),
                self.orientation.as_str().to_string(),
            ),
        ]
    }

    /// クライアント改ざん入力として扱う。欠落は
    /// [`HydrateError::MissingAttr`]、パース不能・範囲外 `focused`・不正な
    /// `loop`/`orientation` 語彙は [`HydrateError::InvalidValue`]（panic
    /// しない。[`crate::carousel::Carousel`] と同型の fail-closed 契約）。
    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |field: &str| -> Result<&str, HydrateError> {
            let name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.as_str())
                .ok_or(HydrateError::MissingAttr(name))
        };

        let focused_raw = find(Self::FIELD_FOCUSED)?;
        let item_count_raw = find(Self::FIELD_ITEM_COUNT)?;
        let loop_raw = find(Self::FIELD_LOOP)?;
        let orientation_raw = find(Self::FIELD_ORIENTATION)?;

        let attr_name_item_count = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_ITEM_COUNT);
        let item_count =
            item_count_raw
                .parse::<usize>()
                .map_err(|_| HydrateError::InvalidValue {
                    attr: attr_name_item_count,
                    reason: "expected a non-negative integer".to_string(),
                })?;

        let attr_name_focused = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_FOCUSED);
        let focused = focused_raw
            .parse::<usize>()
            .map_err(|_| HydrateError::InvalidValue {
                attr: attr_name_focused.clone(),
                reason: "expected a non-negative integer".to_string(),
            })?;
        if item_count == 0 {
            if focused != 0 {
                return Err(HydrateError::InvalidValue {
                    attr: attr_name_focused,
                    reason: "expected focused == 0 when item_count == 0".to_string(),
                });
            }
        } else if focused >= item_count {
            return Err(HydrateError::InvalidValue {
                attr: attr_name_focused,
                reason: "expected focused within [0, item_count)".to_string(),
            });
        }

        let attr_name_loop = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_LOOP);
        let loop_focus = match loop_raw {
            "true" => true,
            "false" => false,
            _ => {
                return Err(HydrateError::InvalidValue {
                    attr: attr_name_loop,
                    reason: "expected \"true\" or \"false\"".to_string(),
                })
            }
        };

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
            focused,
            item_count,
            loop_focus,
            orientation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part/ARIA/data-* 出力 ---

    #[test]
    fn root_outputs_toolbar_role_and_orientation() {
        let html = render(&root(
            Orientation::Horizontal,
            "Text formatting",
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="toolbar""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="toolbar""#));
        assert!(html.contains(r#"aria-orientation="horizontal""#));
        assert!(html.contains(r#"data-orientation="horizontal""#));
        assert!(html.contains(r#"aria-label="Text formatting""#));
    }

    #[test]
    fn root_empty_label_omits_aria_label() {
        let html = render(&root(Orientation::Horizontal, "", vec![], vec![]));
        assert!(!html.contains("aria-label"));
    }

    #[test]
    fn root_vertical_outputs_vertical_orientation() {
        let html = render(&root(Orientation::Vertical, "Toolbar", vec![], vec![]));
        assert!(html.contains(r#"aria-orientation="vertical""#));
        assert!(html.contains(r#"data-orientation="vertical""#));
    }

    #[test]
    fn button_focused_true_outputs_tabindex_zero() {
        let html = render(&button(true, false, vec![], vec![text("Bold")]));
        assert!(html.contains(r#"data-part="button""#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(!html.contains("aria-disabled"));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn button_focused_false_outputs_tabindex_minus_one() {
        let html = render(&button(false, false, vec![], vec![]));
        assert!(html.contains(r#"tabindex="-1""#));
    }

    #[test]
    fn button_disabled_stays_focusable_with_aria_disabled() {
        let html = render(&button(false, true, vec![], vec![]));
        assert!(html.contains(r#"tabindex="-1""#));
        assert!(html.contains(r#"aria-disabled="true""#));
        assert!(html.contains(r#"data-disabled="""#));
        // ネイティブ disabled は付与しない（フォーカス順序から除外しない設計）。
        // `aria-disabled="true"` も末尾が `disabled="` を含むため、単純な
        // 部分文字列比較ではなく空値のネイティブ boolean 属性
        // （`" disabled=\"\""`）そのものの不在を確認する。
        assert!(!html.contains(r#" disabled="""#));
    }

    #[test]
    fn button_caller_tabindex_is_dropped() {
        let html = render(&button(true, false, vec![("tabindex", "5")], vec![]));
        assert_eq!(html.matches("tabindex=").count(), 1);
        assert!(html.contains(r#"tabindex="0""#));
        assert!(!html.contains(r#"tabindex="5""#));
    }

    #[test]
    fn link_delegates_to_link_root_for_external_attrs() {
        let html = render(&link(
            true,
            "https://example.com",
            true,
            vec![],
            vec![text("Docs")],
        ));
        assert!(html.starts_with("<a"));
        assert!(html.contains(r#"data-scope="toolbar""#));
        assert!(html.contains(r#"data-part="link""#));
        assert!(html.contains(r#"href="https://example.com""#));
        assert!(html.contains(r#"target="_blank""#));
        assert!(html.contains(r#"rel="noopener noreferrer""#));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(html.contains(">Docs<"));
    }

    #[test]
    fn link_not_external_omits_target_and_rel() {
        let html = render(&link(false, "/docs", false, vec![], vec![]));
        assert!(!html.contains("target="));
        assert!(!html.contains("rel="));
        assert!(html.contains(r#"tabindex="-1""#));
    }

    #[test]
    fn link_caller_tabindex_is_dropped() {
        let html = render(&link(true, "/docs", false, vec![("tabindex", "7")], vec![]));
        assert_eq!(html.matches("tabindex=").count(), 1);
        assert!(html.contains(r#"tabindex="0""#));
    }

    #[test]
    fn link_dangerous_url_scheme_is_rejected() {
        let html = render(&link(true, "javascript:alert(1)", false, vec![], vec![]));
        assert!(!html.contains("href="));
    }

    #[test]
    fn separator_horizontal_toolbar_outputs_vertical_orientation() {
        let html = render(&separator(Orientation::Horizontal, vec![], vec![]));
        assert!(html.contains(r#"data-part="separator""#));
        assert!(html.contains(r#"role="separator""#));
        assert!(html.contains(r#"aria-orientation="vertical""#));
    }

    #[test]
    fn separator_vertical_toolbar_outputs_horizontal_orientation() {
        let html = render(&separator(Orientation::Vertical, vec![], vec![]));
        assert!(html.contains(r#"aria-orientation="horizontal""#));
    }

    #[test]
    fn toggle_group_outputs_group_role_without_aria_orientation() {
        let html = render(&toggle_group(vec![], vec![]));
        assert!(html.contains(r#"data-part="toggle-group""#));
        assert!(html.contains(r#"role="group""#));
        assert!(!html.contains("aria-orientation"));
    }

    #[test]
    fn toggle_item_reflects_pressed_focused_disabled() {
        let pressed = render(&toggle_item(true, true, false, "bold", vec![], vec![]));
        assert!(pressed.contains(r#"data-part="toggle-item""#));
        assert!(pressed.contains(r#"aria-pressed="true""#));
        assert!(pressed.contains(r#"data-state="on""#));
        assert!(pressed.contains(r#"data-pressed="""#));
        assert!(pressed.contains(r#"data-value="bold""#));
        assert!(pressed.contains(r#"tabindex="0""#));

        let unpressed_unfocused =
            render(&toggle_item(false, false, true, "italic", vec![], vec![]));
        assert!(unpressed_unfocused.contains(r#"aria-pressed="false""#));
        assert!(unpressed_unfocused.contains(r#"data-state="off""#));
        assert!(!unpressed_unfocused.contains("data-pressed"));
        assert!(unpressed_unfocused.contains(r#"tabindex="-1""#));
        assert!(unpressed_unfocused.contains(r#"aria-disabled="true""#));
        assert!(unpressed_unfocused.contains(r#"data-disabled="""#));
        // button_disabled_stays_focusable_with_aria_disabled と同じ理由で
        // 空値のネイティブ boolean 属性そのものの不在を確認する。
        assert!(!unpressed_unfocused.contains(r#" disabled="""#));
    }

    #[test]
    fn toggle_item_caller_tabindex_is_dropped() {
        let html = render(&toggle_item(
            false,
            true,
            false,
            "bold",
            vec![("tabindex", "3")],
            vec![],
        ));
        assert_eq!(html.matches("tabindex=").count(), 1);
        assert!(html.contains(r#"tabindex="0""#));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            Orientation::Horizontal,
            "Toolbar",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="toolbar""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn link_caller_supplied_scope_and_part_are_dropped() {
        let html = render(&link(
            true,
            "/docs",
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="toolbar""#));
        assert!(html.contains(r#"data-part="link""#));
        assert!(!html.contains("attacker"));
    }

    // --- 正規化（fail-closed） ---

    #[test]
    fn new_normalizes_out_of_range_focused_to_zero() {
        let t = Toolbar::new(5, 3, false, Orientation::Horizontal);
        assert_eq!(t.focused(), 0);
    }

    #[test]
    fn new_zero_item_count_forces_focused_zero() {
        let t = Toolbar::new(2, 0, true, Orientation::Horizontal);
        assert_eq!(t.focused(), 0);
        assert_eq!(t.item_count(), 0);
    }

    #[test]
    fn default_is_empty_toolbar() {
        let t = Toolbar::default();
        assert_eq!(t.focused(), 0);
        assert_eq!(t.item_count(), 0);
        assert!(!t.is_loop_focus());
        assert_eq!(t.orientation(), Orientation::Horizontal);
        assert!(!t.is_focused(0));
    }

    // --- dispatch 統合: 決定的な遷移規則 ---

    #[test]
    fn dispatch_next_advances_and_stops_at_end_without_loop() {
        let mut t = Toolbar::new(0, 3, false, Orientation::Horizontal);
        assert!(dispatch(&mut t, "next", ""));
        assert_eq!(t.focused(), 1);
        assert!(dispatch(&mut t, "next", ""));
        assert_eq!(t.focused(), 2);
        assert!(dispatch(&mut t, "next", ""));
        assert_eq!(t.focused(), 2, "loop_focus 無効時は末尾で停止する");
    }

    #[test]
    fn dispatch_next_wraps_to_zero_at_end_with_loop() {
        let mut t = Toolbar::new(2, 3, true, Orientation::Horizontal);
        assert!(dispatch(&mut t, "next", ""));
        assert_eq!(t.focused(), 0);
    }

    #[test]
    fn dispatch_prev_retreats_and_stops_at_start_without_loop() {
        let mut t = Toolbar::new(2, 3, false, Orientation::Horizontal);
        assert!(dispatch(&mut t, "prev", ""));
        assert_eq!(t.focused(), 1);
        assert!(dispatch(&mut t, "prev", ""));
        assert_eq!(t.focused(), 0);
        assert!(dispatch(&mut t, "prev", ""));
        assert_eq!(t.focused(), 0, "loop_focus 無効時は先頭で停止する");
    }

    #[test]
    fn dispatch_prev_wraps_to_end_at_start_with_loop() {
        let mut t = Toolbar::new(0, 3, true, Orientation::Horizontal);
        assert!(dispatch(&mut t, "prev", ""));
        assert_eq!(t.focused(), 2);
    }

    #[test]
    fn dispatch_first_and_last() {
        let mut t = Toolbar::new(1, 4, false, Orientation::Horizontal);
        assert!(dispatch(&mut t, "last", ""));
        assert_eq!(t.focused(), 3);
        assert!(dispatch(&mut t, "first", ""));
        assert_eq!(t.focused(), 0);
    }

    #[test]
    fn dispatch_focus_moves_to_valid_index() {
        let mut t = Toolbar::new(0, 5, false, Orientation::Horizontal);
        assert!(dispatch(&mut t, "focus", "3"));
        assert_eq!(t.focused(), 3);
    }

    #[test]
    fn dispatch_focus_out_of_range_is_noop() {
        let mut t = Toolbar::new(1, 5, false, Orientation::Horizontal);
        assert!(dispatch(&mut t, "focus", "5"));
        assert_eq!(t.focused(), 1);
        assert!(dispatch(&mut t, "focus", "999"));
        assert_eq!(t.focused(), 1);
    }

    #[test]
    fn dispatch_focus_rejects_invalid_payload() {
        let mut t = Toolbar::new(1, 5, false, Orientation::Horizontal);
        for bogus in ["abc", "-1", "1.5", ""] {
            assert!(!dispatch(&mut t, "focus", bogus));
            assert_eq!(t.focused(), 1);
        }
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut t = Toolbar::new(1, 3, false, Orientation::Horizontal);
        assert!(!dispatch(&mut t, "no_such_action", "x"));
        assert_eq!(t.focused(), 1);
    }

    #[test]
    fn item_count_zero_makes_all_actions_noop() {
        let mut t = Toolbar::default();
        assert!(dispatch(&mut t, "next", ""));
        assert!(dispatch(&mut t, "prev", ""));
        assert!(dispatch(&mut t, "first", ""));
        assert!(dispatch(&mut t, "last", ""));
        assert!(dispatch(&mut t, "focus", "0"));
        assert_eq!(t.focused(), 0);
    }

    // --- 利便メソッド ---

    #[test]
    fn convenience_button_reflects_focused_state() {
        let t = Toolbar::new(1, 3, false, Orientation::Horizontal);
        let focused = render(&t.button(1, false, vec![], vec![]));
        assert!(focused.contains(r#"tabindex="0""#));
        let unfocused = render(&t.button(0, false, vec![], vec![]));
        assert!(unfocused.contains(r#"tabindex="-1""#));
    }

    #[test]
    fn convenience_link_reflects_focused_state() {
        let t = Toolbar::new(0, 3, false, Orientation::Horizontal);
        let focused = render(&t.link(0, "/a", false, vec![], vec![]));
        assert!(focused.contains(r#"tabindex="0""#));
    }

    #[test]
    fn convenience_separator_reflects_orientation() {
        let t = Toolbar::new(0, 2, false, Orientation::Vertical);
        let html = render(&t.separator(vec![], vec![]));
        assert!(html.contains(r#"aria-orientation="horizontal""#));
    }

    #[test]
    fn convenience_toggle_item_reflects_focused_state() {
        let t = Toolbar::new(1, 2, false, Orientation::Horizontal);
        let html = render(&t.toggle_item(1, true, false, "bold", vec![], vec![]));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(html.contains(r#"data-state="on""#));
    }

    // --- SSR 状態なし初期描画 ---

    #[test]
    fn default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Toolbar::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- hydration 経路 ---

    #[test]
    fn hydration_round_trip() {
        let t = Toolbar::new(2, 5, true, Orientation::Horizontal);
        let rendered = render(&render_for_hydration(&t));
        assert!(rendered.contains(r#"data-hydrate-focused="2""#));
        assert!(rendered.contains(r#"data-hydrate-item-count="5""#));
        assert!(rendered.contains(r#"data-hydrate-loop="true""#));
        assert!(rendered.contains(r#"data-hydrate-orientation="horizontal""#));

        let restored = Toolbar::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }

    #[test]
    fn hydration_round_trip_vertical_without_loop() {
        let t = Toolbar::new(0, 3, false, Orientation::Vertical);
        let restored = Toolbar::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
        assert_eq!(restored.orientation(), Orientation::Vertical);
        assert!(!restored.is_loop_focus());
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Toolbar::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-focused".to_string())
        );
    }

    #[test]
    fn from_hydration_attrs_invalid_value_does_not_panic() {
        let bogus_sets: Vec<Vec<(String, String)>> = vec![
            // focused が範囲外。
            vec![
                ("data-hydrate-focused".to_string(), "5".to_string()),
                ("data-hydrate-item-count".to_string(), "3".to_string()),
                ("data-hydrate-loop".to_string(), "false".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            // item_count がパース不能。
            vec![
                ("data-hydrate-focused".to_string(), "0".to_string()),
                ("data-hydrate-item-count".to_string(), "abc".to_string()),
                ("data-hydrate-loop".to_string(), "false".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            // loop が未知の値。
            vec![
                ("data-hydrate-focused".to_string(), "0".to_string()),
                ("data-hydrate-item-count".to_string(), "3".to_string()),
                ("data-hydrate-loop".to_string(), "yes".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            // orientation が未知の値。
            vec![
                ("data-hydrate-focused".to_string(), "0".to_string()),
                ("data-hydrate-item-count".to_string(), "3".to_string()),
                ("data-hydrate-loop".to_string(), "false".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "diagonal".to_string(),
                ),
            ],
            // item_count == 0 なのに focused != 0。
            vec![
                ("data-hydrate-focused".to_string(), "1".to_string()),
                ("data-hydrate-item-count".to_string(), "0".to_string()),
                ("data-hydrate-loop".to_string(), "false".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            // focused が XSS ペイロード。
            vec![
                (
                    "data-hydrate-focused".to_string(),
                    "<script>alert(1)</script>".to_string(),
                ),
                ("data-hydrate-item-count".to_string(), "3".to_string()),
                ("data-hydrate-loop".to_string(), "false".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
        ];
        for attrs in bogus_sets {
            let err = Toolbar::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: label/value/href/attrs/children/hydration にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn root_label_payload_is_escaped_on_render() {
        let html = render(&root(
            Orientation::Horizontal,
            ATTR_BREAK_PAYLOAD,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn toggle_item_value_payload_is_escaped_on_render() {
        let html = render(&toggle_item(
            false,
            false,
            false,
            ATTR_BREAK_PAYLOAD,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn link_href_payload_is_escaped_on_render() {
        let html = render(&link(
            false,
            "/docs\" onmouseover=\"alert(1)",
            false,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            Orientation::Horizontal,
            "Toolbar",
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&button(
            false,
            false,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn dispatch_focus_payload_is_escaped_on_render() {
        // "focus" の payload は usize の厳密パースのみを通すため、
        // スクリプトペイロードはそもそも decode_action で拒否される
        // （dispatch は false を返し状態は変化しない）。
        let mut t = Toolbar::new(0, 3, false, Orientation::Horizontal);
        assert!(!dispatch(&mut t, "focus", "\"><script>alert(1)</script>"));
        assert_eq!(t.focused(), 0);
    }

    #[test]
    fn hydration_xss_payload_in_focused_is_rejected_not_rendered() {
        let attrs = vec![
            (
                "data-hydrate-focused".to_string(),
                "<script>alert(1)</script>".to_string(),
            ),
            ("data-hydrate-item-count".to_string(), "3".to_string()),
            ("data-hydrate-loop".to_string(), "false".to_string()),
            (
                "data-hydrate-orientation".to_string(),
                "horizontal".to_string(),
            ),
        ];
        let err = Toolbar::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}
