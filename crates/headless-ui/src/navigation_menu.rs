//! Navigation Menu（トリガー起点で開閉するナビゲーションパネル）headless
//! コンポーネント（イシュー #993、親 #932 Phase 8、トラッキング #924）。
//!
//! Radix Primitives `Navigation Menu` / ark-ui `NavigationMenu`
//!（`docs/design/component-coverage-map.md` §5 Part D / §9）を参照し、
//! Root / List / Item / Trigger / ItemIndicator / Content / Link の 7 anatomy
//! パーツと、[`crate::state::SingleSelect`] を埋め込んだ「高々 1 項目が開く」
//! 状態機械 [`NavigationMenu`] を提供する（イシュー #1654 で ItemIndicator を
//! 新設し 6 → 7 パーツへ拡張）。
//!
//! # `nav_list` との使い分け
//!
//! [`mod@crate::nav_list`] は状態機械を一切持たない静的な文書ナビ
//! リンク集（見出し + リンクリストのみ、ディスクロージャなし）である。
//! 本モジュールは Trigger/Content によるディスクロージャ（クリックで
//! パネルが開閉する）と「高々 1 個の Trigger だけが開く」状態機械、
//! および `aria-expanded`/`aria-controls` によるトリガー・パネルの
//! 関連付けを持つ点で異なる。両者に共通するのは「`role` を明示付与しない」
//! 判断（下記参照）であり、使い分けの軸は role の有無ではなく
//! **ディスクロージャの有無**である。単なるリンク集は [`mod@crate::nav_list`]
//! を、開閉するナビゲーションパネルが必要な場合は本モジュールを使う。
//!
//! # `role` を明示付与しない
//!
//! - [`root`] は `<nav>` の暗黙 ARIA role（`navigation`）に依拠し、
//!   `role="navigation"` を明示付与しない（冗長 role のアンチパターン。
//!   [`mod@crate::nav_list`] が確立したハウススタイルを踏襲する）。
//! - **`role="menu"`/`role="menuitem"` を一切付与しない**。Radix
//!   NavigationMenu も意図的に menu role を避けている。文書ナビを
//!   操作メニューと誤伝達するとスクリーンリーダー利用者のアクセシビリティを
//!   毀損する（`docs/design/docs-site-styled-ui-adoption.md` §3.1 が
//!   [`mod@crate::nav_list`] 新設の理由として指摘した問題と同型であり、
//!   [`mod@crate::menubar`] の `role="menubar"`/`role="menuitem"` からの
//!   類推は誤り）。
//!
//! # `data-motion`・viewport 測定を実装しない
//!
//! Radix NavigationMenu が primitives 層に持ち込んでいる **viewport 寸法
//! 測定** と **`data-motion`（アニメーション方向の露出）** は、
//! `docs/policy/intentional-non-adoption.md` §3.25 規則 2（層の割り当て）
//! の対象であり、本モジュール（headless 層）には持ち込まない。装飾・
//! アニメーション・レイアウト計測の関心は上層の `fandhe-frontend-pre-styled-ui`
//! の責務として設計する。本モジュールが担うのは anatomy（構造）・
//! アクセシビリティ（WAI-ARIA）・表示状態（`data-*`）までである。
//!
//! # 状態機械は [`crate::state::SingleSelect`] を埋め込む
//!
//! Radix NavigationMenu の `value`（開いている項目の値。高々 1 個）は
//! [`crate::state::SingleSelect`] にそのまま写像できるため、
//! [`mod@crate::accordion`] の [`crate::accordion::Accordion`] と同型に
//! [`crate::state::SingleSelect`] を埋め込んで委譲する（独自の
//! `Component`/`Hydrate` 直接実装は行わない）。dispatch 文字列は
//! [`crate::state::SingleSelect`] の既存契約（`"select"`/`"toggle"`/
//! `"deselect"`）をそのまま継承する。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`list`]/[`item`]/[`trigger`]/
//! [`item_indicator`]/[`content`]/[`link`]、いずれも純粋関数で完結）を
//! 直接呼んで組み立てる。
//! 各パーツは項目ごとの [`crate::state::OpenState`] を引数で受け取るため
//! [`NavigationMenu`] を経由しない構成でも共用できる。CSR/hydration は
//! [`NavigationMenu`]（[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を使う。
//! `fandhe-frontend-pre-styled-ui`（イシュー #993）が本モジュールを呼んで
//! スタイル済み Navigation Menu を組み立てる想定である。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`type`/`hidden`/`disabled`/`id`/`href`）は
//!   すべて `&'static str` リテラルで固定しており、動的値が属性名スロットへ
//!   混入する経路はない（[`mod@crate::anatomy`]/[`crate::aria`]/
//!   [`crate::data_attrs`] の既存不変条件をそのまま継承する）。
//! - 動的値（`label`/`id`/`controls`/`labelled_by`/`href`/呼び出し側
//!   `attrs`/`children` テキスト）は [`fandhe_frontend_core::render`] の
//!   既定エスケープを必ず経由する。`raw_html()` は使用せず、HTML 文字列を
//!   直接組み立てない。
//! - [`link`] の `href` は [`fandhe_frontend_core`] の許可リスト方式に
//!   委ねる（危険 URL スキームは属性ごと拒否される。
//!   [`mod@crate::nav_list`] の `link` と同じ不変条件）。
//! - hydration 属性（`data-hydrate-selected`）はクライアント側で改ざんされ
//!   うる入力として扱う。[`NavigationMenu`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は
//!   [`crate::state::SingleSelect`] へ全委譲することで、panic せず
//!   `HydrateError` を返す既存保証をそのまま継承する。
//!
//! # 参考サイト（Radix Primitives / ark-ui）との意図的な差分（イシュー #1654）
//!
//! `docs/policy/intentional-non-adoption.md` §3.25 規則 2（層の割り当て）に
//! 基づき、以下は headless 層へ持ち込まない意図的な非採用である。
//!
//! - **Indicator（スライドバー）/ Viewport / ViewportPositioner / Arrow**:
//!   レイアウト計測を伴う装飾関心であり、必要なら上層 `pre-styled-ui` の
//!   責務として設計する。
//! - **Sub（入れ子ナビゲーション）**: 状態機械の入れ子は本イシューの範囲外。
//! - **hover/delay による自動 open・open-follows-focus・typeahead**:
//!   `crates/wasm-full/src/keynav.rs` のキーボード操作節（下記）参照。クリック
//!   起点の開閉のみをサポートし、ホバーでの自動展開は実装しない。
//! - **`data-trigger-proxy-id`（ark-ui）**: 実行時 proxy 要素向けの内部属性。
//! - **Radix の `data-active`（[`link`]）**: ark-ui 語彙の `data-current`
//!   （[`mod@crate::nav_list`] の `link` と同じ）へ統一する。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - **`data-motion`・viewport 寸法測定**: 上記のとおり §3.25 規則 2 により
//!   headless 層へ置かない。
//! - **Indicator / Viewport / Sub\* パーツ**: 上記「意図的な差分」参照。
//! - **キーボード操作の実 DOM 配線**（矢印キー・Escape・フォーカス移動）:
//!   `fandhe-frontend-wasm-full` の責務（[`mod@crate::menubar`]/
//!   [`mod@crate::toolbar`] と同じ扱い）。`crates/wasm-full/src/keynav.rs`
//!   （イシュー #1075）で実装済み。[`NavigationMenuProps::orientation`] が
//!   出力する `data-orientation`（SSR 静的属性）は同モジュールの
//!   `handle_navigation_menu_trigger_keydown` が矢印キーの向き判定に読む
//!   （[`mod@crate::accordion`] の同種記述と同型）。
//!
//! # キーボード操作（`crates/wasm-full/src/keynav.rs` 実装、APG Disclosure
//! Navigation Menu 準拠、イシュー #1075/#1654）
//!
//! | キー | 対象 | 挙動 |
//! |---|---|---|
//! | Enter / Space | Trigger | claim しない（ネイティブ `button` の click →
//!   `"toggle"` dispatch） |
//! | ArrowRight/ArrowLeft（horizontal）、ArrowDown/ArrowUp（vertical） |
//!   Trigger | Trigger 間のフォーカス移動（disabled をスキップ、非循環。
//!   `data-loop-focus="true"` で循環） |
//! | Home / End | Trigger | 先頭 / 末尾の非 disabled Trigger へ |
//! | ArrowDown/ArrowUp（horizontal）、ArrowRight/ArrowLeft（vertical） |
//!   Trigger | closed なら click 合成で開いて先頭/末尾リンクへ、open なら
//!   合成なしで先頭/末尾リンクへフォーカス |
//! | ArrowDown/ArrowUp/ArrowRight/ArrowLeft/Home/End | Content 内 Link |
//!   同一 content 内の非 disabled リンク間を非循環で移動 |
//! | Escape | open 中の Trigger / Content 内 Link | click 合成で閉じ、Trigger
//!   へフォーカスを戻す（closed の Trigger 上は no-op） |
//! | Tab / Shift+Tab | — | 配線なし（roving tabindex 不採用） |

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{
    aria_controls, aria_current, aria_expanded, aria_hidden, aria_label, aria_labelledby,
    AriaCurrent,
};
use crate::data_attrs::{data_current, data_disabled, data_orientation, data_state, Orientation};
use crate::state::{OpenState, SingleSelect, SingleSelectAction};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// Navigation Menu の anatomy（`data-scope="navigation-menu"`）。
const ANATOMY: Anatomy = anatomy("navigation-menu");

/// Root レベルの共通プロパティ（ark-ui/Radix の Root `orientation` 相当、
/// イシュー #1654）。`root`/`list`/`item`/`content` へ通し `data-orientation`
/// を出力する（`trigger`/`link` には付与しない。参考サイトとも trigger には
/// `data-orientation` を付けない）。
///
/// `orientation` は SSR 静的マークアップ（`data-orientation` 属性）にのみ
/// 寄与し、実際のキーボード操作は `fandhe-frontend-wasm-full` の `keynav.rs`
/// が本属性を読んで解釈する（[`mod@crate::accordion`] の
/// [`crate::accordion::AccordionProps`] と同型の設計、本モジュールはキー
/// 入力を処理しない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavigationMenuProps {
    /// パーツの向き（既定 [`Orientation::Horizontal`]。ark-ui/Radix の
    /// NavigationMenu Root `orientation` 既定・`keynav.rs` の欠落時
    /// フォールバックと一致）。
    pub orientation: Orientation,
}

impl Default for NavigationMenuProps {
    fn default() -> Self {
        Self {
            orientation: Orientation::Horizontal,
        }
    }
}

/// [`root`] が固定付与するキー一覧。
const ROOT_RESERVED: &[&str] = &["aria-label", "data-orientation"];
/// [`list`] が固定付与するキー一覧。
const LIST_RESERVED: &[&str] = &["data-orientation"];
/// [`item`] が固定付与するキー一覧。
const ITEM_RESERVED: &[&str] = &[
    "data-state",
    "data-disabled",
    "data-orientation",
    "data-value",
];
/// [`trigger`] が固定付与するキー一覧。
const TRIGGER_RESERVED: &[&str] = &[
    "type",
    "aria-expanded",
    "data-state",
    "data-value",
    "data-disabled",
    "disabled",
];
/// [`item_indicator`] が固定付与するキー一覧。
const ITEM_INDICATOR_RESERVED: &[&str] = &[
    "aria-hidden",
    "data-state",
    "data-orientation",
    "data-value",
];
/// [`content`] が固定付与するキー一覧（`Option` 引数由来の `id`/
/// `aria-labelledby`/`hidden` は #1920/#1921 と同じ規則で reserved から
/// 除外する）。
const CONTENT_RESERVED: &[&str] = &["data-state", "data-orientation", "data-value"];
/// [`link`] が固定付与するキー一覧（`href` は必須引数のため reserved から
/// 除外する）。
const LINK_RESERVED: &[&str] = &["aria-current", "data-current"];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（`crates/headless-ui/src/accordion.rs::drop_reserved` 等と同型の
/// 重複実装。モジュール間の相互依存を避けるため個別に定義する）。呼び出し側
/// が `data-state`/`data-orientation`/`aria-expanded` 等を偽装しても
/// フレームワークが付与する値が常に優先されることを保証する（A05 対策）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// `root` パーツ（`nav`）。`label` は `aria-label` として付与し必須引数
/// （複数 `nav` ランドマークの区別のため、[`mod@crate::nav_list`] の
/// `root` と同じ判断）。状態非依存（各項目の開閉状態は [`item`] 側が持つ）。
/// `props.orientation` を `data-orientation` として出力する（イシュー #1654）。
#[must_use]
pub fn root<'a>(
    props: &NavigationMenuProps,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let mut merged: Vec<(&str, &str)> =
        vec![aria_label(label), data_orientation(props.orientation)];
    merged.extend(attrs);
    ANATOMY.part("root", "nav", merged, children)
}

/// `list` パーツ（`ul`）。`props.orientation` を `data-orientation` として
/// 出力する（イシュー #1654）。
#[must_use]
pub fn list(props: &NavigationMenuProps, attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, LIST_RESERVED);
    let mut merged: Vec<(&str, &str)> = vec![data_orientation(props.orientation)];
    merged.extend(attrs);
    ANATOMY.part("list", "ul", merged, children)
}

/// `item` パーツ（`li`）。項目 1 個の開閉状態・disabled 状態を `data-*` へ
/// 反映する（[`crate::accordion::item`] と同型）。`props.orientation` を
/// `data-orientation` として、`value` を `data-value` として出力する
/// （イシュー #1654、ark-ui `NavigationMenu.Item` の Data Attributes 表に
/// 合わせる）。
#[must_use]
pub fn item<'a>(
    state: OpenState,
    disabled: bool,
    props: &NavigationMenuProps,
    value: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        data_state(state.as_data_state()),
        data_orientation(props.orientation),
        ("data-value", value),
    ];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("item", "li", merged, children)
}

/// `trigger` パーツ（`button`）。
///
/// フォーム内配置時の意図しない submit を防ぐため `type="button"` を
/// 固定で付与する（A05 セキュリティ設定ミス対策。
/// [`crate::accordion::item_trigger`] と同じ判断を踏襲する）。`controls`
/// が `Some` のとき `aria-controls` で [`content`] と関連付ける。
/// `disabled` はネイティブ `disabled` 存在属性と `data-disabled` の両方へ
/// 反映する。`role` は付与しない（本モジュール冒頭の rustdoc「`role` を
/// 明示付与しない」参照）。`value` は `data-value` として出力し、
/// `crates/wasm-full` の `MAPPING_TABLE`（`("navigation-menu", "trigger")`
/// → `"toggle"`）が payload としてクリック起点のディスパッチに用いる
/// （[`crate::accordion::item_trigger`] の `data-value` と同型、イシュー
/// #1161）。
#[must_use]
pub fn trigger<'a>(
    state: OpenState,
    disabled: bool,
    value: &'a str,
    id: Option<&'a str>,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, TRIGGER_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        aria_expanded(state.is_open()),
        data_state(state.as_data_state()),
        ("data-value", value),
    ];
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if let Some(controls) = controls {
        merged.push(aria_controls(controls));
    }
    merged.extend(data_disabled(disabled));
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("trigger", "button", merged, children)
}

/// `item-indicator` パーツ（`span`）。イシュー #1654 で新設（ark-ui
/// `NavigationMenu.ItemIndicator` 相当、Radix 側には対応パーツがない）。
///
/// 装飾用の視覚要素であり、支援技術へは [`trigger`] の `aria-expanded` から
/// 開閉状態が既に伝わるため常時 `aria-hidden="true"` を固定付与する
/// （[`crate::accordion::item_indicator`] と同じ判断）。`props.orientation`
/// を `data-orientation` として、`value` を `data-value` として出力する
/// （ark-ui の ItemIndicator Data Attributes 表に合わせる）。
#[must_use]
pub fn item_indicator<'a>(
    state: OpenState,
    props: &NavigationMenuProps,
    value: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_INDICATOR_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        aria_hidden(true),
        data_state(state.as_data_state()),
        data_orientation(props.orientation),
        ("data-value", value),
    ];
    merged.extend(attrs);
    ANATOMY.part("item-indicator", "span", merged, children)
}

/// `content` パーツ（`div`）。
///
/// closed のとき `hidden` 存在属性を付与し、JS なしの SSR でも閉状態を
/// 表現する（[`crate::accordion::item_content`] と同型。viewport 測定・
/// `data-motion` はモジュール冒頭の rustdoc「`data-motion`・viewport 測定を
/// 実装しない」参照）。`id` が `Some` のとき [`trigger`] の `controls` と
/// 対で `aria-controls` 関連付けを成立させる。`labelled_by` が `Some` の
/// ときのみ `aria-labelledby` を付与する。`props.orientation` を
/// `data-orientation` として、`value` を `data-value` として出力する
/// （イシュー #1654、ark-ui `NavigationMenu.Content` の Data Attributes 表に
/// 合わせる）。
#[must_use]
pub fn content<'a>(
    state: OpenState,
    props: &NavigationMenuProps,
    value: &'a str,
    id: Option<&'a str>,
    labelled_by: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, CONTENT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        data_state(state.as_data_state()),
        data_orientation(props.orientation),
        ("data-value", value),
    ];
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if let Some(labelled_by) = labelled_by {
        merged.push(aria_labelledby(labelled_by));
    }
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("content", "div", merged, children)
}

/// `link` パーツ（`a`）。`current` が `true` のとき `aria-current="page"`
/// と `data-current` を付与する（[`mod@crate::nav_list`] の `link` と同じ
/// 語彙。Radix の `data-active` 語彙は採らず ark-ui 語彙で統一する、
/// モジュール冒頭「参考サイトとの意図的な差分」参照）。`role` は一切
/// 付与しない。
#[must_use]
pub fn link<'a>(
    href: &'a str,
    current: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, LINK_RESERVED);
    let mut merged: Vec<(&str, &str)> = vec![("href", href)];
    if current {
        merged.push(aria_current(AriaCurrent::Page));
        merged.extend(data_current(true));
    }
    merged.extend(attrs);
    ANATOMY.part("link", "a", merged, children)
}

/// [`crate::state::SingleSelect`] を埋め込んだ Navigation Menu の状態機械。
///
/// 「高々 1 個の Trigger だけが開く」制約を型レベルで保証する入口として、
/// [`Self::item_state`] が各項目値の [`OpenState`] を決定し、
/// [`item`]/[`trigger`]/[`item_indicator`]/[`content`] へ注入する利便
/// メソッドを提供する（[`root`]/[`list`]/[`link`] は状態非依存またはリンク
/// 単位のため利便メソッドを持たない）。SSR での自由関数直接利用も引き続き
/// 可能。
/// `Default` は未選択（全項目 closed。SSR の状態なし初期描画に対応する
/// 既定値）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NavigationMenu {
    select: SingleSelect,
}

impl NavigationMenu {
    /// 現在開いている項目値（未選択なら `None`）。
    #[must_use]
    pub fn open_value(&self) -> Option<&str> {
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
        value: &'a str,
        disabled: bool,
        props: &NavigationMenuProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(
            self.item_state(value),
            disabled,
            props,
            value,
            attrs,
            children,
        )
    }

    /// [`trigger`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn trigger<'a>(
        &self,
        value: &str,
        disabled: bool,
        id: Option<&'a str>,
        controls: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        trigger(
            self.item_state(value),
            disabled,
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
        value: &'a str,
        props: &NavigationMenuProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_indicator(self.item_state(value), props, value, attrs, children)
    }

    /// [`content`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn content<'a>(
        &self,
        value: &'a str,
        props: &NavigationMenuProps,
        id: Option<&'a str>,
        labelled_by: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        content(
            self.item_state(value),
            props,
            value,
            id,
            labelled_by,
            attrs,
            children,
        )
    }
}

impl Component for NavigationMenu {
    type Action = SingleSelectAction;

    fn update(&mut self, action: SingleSelectAction) {
        self.select.update(action);
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（root、children
    /// 空）。[`crate::state::SingleSelect::view`] と同じ位置付けであり、
    /// 公開 UI としての利用は想定しない（実際の UI 構築は §パーツ関数群を
    /// 呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        root(&NavigationMenuProps::default(), "", Vec::new(), Vec::new())
    }

    fn decode_action(name: &str, payload: &str) -> Option<SingleSelectAction> {
        SingleSelect::decode_action(name, payload)
    }
}

impl Hydrate for NavigationMenu {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.select.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            select: SingleSelect::from_hydration_attrs(attrs)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    fn h() -> NavigationMenuProps {
        NavigationMenuProps::default()
    }

    fn v() -> NavigationMenuProps {
        NavigationMenuProps {
            orientation: Orientation::Vertical,
        }
    }

    // --- 各パーツの data-scope/data-part/data-state 出力・role 非出力 ---

    #[test]
    fn root_outputs_nav_with_aria_label_and_no_role() {
        let html = render(&root(&h(), "Main", vec![], vec![]));
        assert!(html.starts_with("<nav"));
        assert!(html.contains(r#"aria-label="Main""#));
        assert!(html.contains(r#"data-scope="navigation-menu""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("role="));
    }

    // --- イシュー #1654: data-orientation（root/list/item/content） ---

    #[test]
    fn root_list_item_content_output_data_orientation_default_horizontal() {
        let root_html = render(&root(&h(), "Main", vec![], vec![]));
        let list_html = render(&list(&h(), vec![], vec![]));
        let item_html = render(&item(OpenState::Closed, false, &h(), "a", vec![], vec![]));
        let content_html = render(&content(
            OpenState::Open,
            &h(),
            "a",
            None,
            None,
            vec![],
            vec![],
        ));
        for html in [root_html, list_html, item_html, content_html] {
            assert!(html.contains(r#"data-orientation="horizontal""#));
        }
    }

    #[test]
    fn root_list_item_content_output_data_orientation_vertical() {
        let root_html = render(&root(&v(), "Main", vec![], vec![]));
        let list_html = render(&list(&v(), vec![], vec![]));
        let item_html = render(&item(OpenState::Closed, false, &v(), "a", vec![], vec![]));
        let content_html = render(&content(
            OpenState::Open,
            &v(),
            "a",
            None,
            None,
            vec![],
            vec![],
        ));
        for html in [root_html, list_html, item_html, content_html] {
            assert!(html.contains(r#"data-orientation="vertical""#));
        }
    }

    #[test]
    fn trigger_and_link_do_not_output_data_orientation() {
        let trigger_html = render(&trigger(
            OpenState::Closed,
            false,
            "a",
            None,
            None,
            vec![],
            vec![],
        ));
        let link_html = render(&link("/docs", false, vec![], vec![]));
        assert!(!trigger_html.contains("data-orientation"));
        assert!(!link_html.contains("data-orientation"));
    }

    #[test]
    fn root_list_item_content_caller_supplied_data_orientation_is_dropped() {
        let root_html = render(&root(
            &h(),
            "Main",
            vec![("data-orientation", "vertical")],
            vec![],
        ));
        let list_html = render(&list(&h(), vec![("data-orientation", "vertical")], vec![]));
        let item_html = render(&item(
            OpenState::Closed,
            false,
            &h(),
            "a",
            vec![("data-orientation", "vertical")],
            vec![],
        ));
        let content_html = render(&content(
            OpenState::Open,
            &h(),
            "a",
            None,
            None,
            vec![("data-orientation", "vertical")],
            vec![],
        ));
        for html in [root_html, list_html, item_html, content_html] {
            assert!(html.contains(r#"data-orientation="horizontal""#));
            assert!(!html.contains(r#"data-orientation="vertical""#));
        }
    }

    // --- イシュー #1654: data-value（item/content） ---

    #[test]
    fn item_and_content_output_data_value() {
        let item_html = render(&item(
            OpenState::Closed,
            false,
            &h(),
            "products",
            vec![],
            vec![],
        ));
        assert!(item_html.contains(r#"data-value="products""#));

        let content_html = render(&content(
            OpenState::Open,
            &h(),
            "products",
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(content_html.contains(r#"data-value="products""#));
    }

    #[test]
    fn item_and_content_caller_supplied_data_value_is_dropped() {
        let item_html = render(&item(
            OpenState::Closed,
            false,
            &h(),
            "products",
            vec![("data-value", "attacker")],
            vec![],
        ));
        assert!(item_html.contains(r#"data-value="products""#));
        assert!(!item_html.contains("attacker"));

        let content_html = render(&content(
            OpenState::Open,
            &h(),
            "products",
            None,
            None,
            vec![("data-value", "attacker")],
            vec![],
        ));
        assert!(content_html.contains(r#"data-value="products""#));
        assert!(!content_html.contains("attacker"));
    }

    #[test]
    fn list_and_item_output_expected_tags_without_role() {
        let list_html = render(&list(
            &h(),
            vec![],
            vec![item(OpenState::Closed, false, &h(), "a", vec![], vec![])],
        ));
        assert!(list_html.starts_with("<ul"));
        assert!(list_html.contains("<li"));
        assert!(!list_html.contains("role="));

        let item_html = render(&item(OpenState::Open, false, &h(), "a", vec![], vec![]));
        assert!(item_html.contains(r#"data-state="open""#));
        assert!(!item_html.contains("data-disabled"));
    }

    #[test]
    fn item_disabled_true_adds_data_disabled() {
        let html = render(&item(OpenState::Closed, true, &h(), "a", vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn trigger_has_type_button_aria_expanded_and_no_role() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            "a",
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains("<button"));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(html.contains(r#"data-value="a""#));
        assert!(!html.contains("aria-controls"));
        assert!(!html.contains(" id="));
        assert!(!html.contains("disabled"));
        assert!(!html.contains("role="));

        let open_html = render(&trigger(
            OpenState::Open,
            false,
            "a",
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(open_html.contains(r#"aria-expanded="true""#));
        assert!(open_html.contains(r#"data-state="open""#));
    }

    #[test]
    fn trigger_id_and_controls_some_outputs_both_attributes() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            "a",
            Some("t-trigger-a"),
            Some("t-content-a"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"id="t-trigger-a""#));
        assert!(html.contains(r#"aria-controls="t-content-a""#));
    }

    #[test]
    fn trigger_disabled_reflects_native_and_data_disabled() {
        let disabled_html = render(&trigger(
            OpenState::Closed,
            true,
            "a",
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(disabled_html.contains(r#"data-disabled="""#));
        assert!(disabled_html.contains(r#"disabled="""#));

        let enabled_html = render(&trigger(
            OpenState::Closed,
            false,
            "a",
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(!enabled_html.contains("data-disabled"));
        assert!(!enabled_html.contains(" disabled"));
    }

    #[test]
    fn trigger_caller_supplied_reserved_keys_are_dropped() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            "a",
            None,
            None,
            vec![
                ("type", "submit"),
                ("aria-expanded", "true"),
                ("data-state", "open"),
                ("data-value", "attacker"),
                ("data-disabled", ""),
                ("disabled", ""),
            ],
            vec![],
        ));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(html.contains(r#"data-value="a""#));
        assert!(!html.contains("attacker"));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains(" disabled"));
    }

    // --- イシュー #1654: item-indicator パーツ新設 ---

    #[test]
    fn item_indicator_outputs_scope_part_state_orientation_value_and_aria_hidden() {
        let html = render(&item_indicator(
            OpenState::Open,
            &v(),
            "products",
            vec![],
            vec![text("▾")],
        ));
        assert!(html.starts_with("<span"));
        assert!(html.contains(r#"data-scope="navigation-menu""#));
        assert!(html.contains(r#"data-part="item-indicator""#));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains(r#"data-orientation="vertical""#));
        assert!(html.contains(r#"data-value="products""#));
        assert!(!html.contains("role="));
    }

    #[test]
    fn item_indicator_closed_state_reflects_data_state() {
        let html = render(&item_indicator(
            OpenState::Closed,
            &h(),
            "products",
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-state="closed""#));
    }

    #[test]
    fn item_indicator_caller_supplied_reserved_keys_are_dropped() {
        let html = render(&item_indicator(
            OpenState::Open,
            &h(),
            "products",
            vec![
                ("aria-hidden", "false"),
                ("data-state", "closed"),
                ("data-orientation", "vertical"),
                ("data-value", "attacker"),
            ],
            vec![],
        ));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains(r#"data-orientation="horizontal""#));
        assert!(html.contains(r#"data-value="products""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn content_closed_has_hidden_open_does_not() {
        let closed = render(&content(
            OpenState::Closed,
            &h(),
            "a",
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&content(
            OpenState::Open,
            &h(),
            "a",
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn content_id_and_labelled_by_are_optional_and_independent() {
        let with_id = render(&content(
            OpenState::Open,
            &h(),
            "a",
            Some("t-content-a"),
            None,
            vec![],
            vec![],
        ));
        assert!(with_id.contains(r#"id="t-content-a""#));
        assert!(!with_id.contains("aria-labelledby"));

        let with_labelled_by = render(&content(
            OpenState::Open,
            &h(),
            "a",
            None,
            Some("t-trigger-a"),
            vec![],
            vec![],
        ));
        assert!(with_labelled_by.contains(r#"aria-labelledby="t-trigger-a""#));
        assert!(!with_labelled_by.contains(" id="));
    }

    #[test]
    fn content_has_no_role_attribute() {
        let html = render(&content(
            OpenState::Open,
            &h(),
            "a",
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(!html.contains("role="));
    }

    #[test]
    fn link_current_true_adds_aria_current_and_data_current_without_role() {
        let html = render(&link("/docs/intro", true, vec![], vec![text("Intro")]));
        assert!(html.contains(r#"aria-current="page""#));
        assert!(html.contains("data-current"));
        assert!(!html.contains("role="));
    }

    #[test]
    fn link_current_false_omits_aria_current_and_data_current() {
        let html = render(&link("/docs/intro", false, vec![], vec![]));
        assert!(!html.contains("aria-current"));
        assert!(!html.contains("data-current"));
    }

    #[test]
    fn link_caller_supplied_aria_current_and_data_current_are_dropped() {
        let html = render(&link(
            "/docs/intro",
            false,
            vec![("aria-current", "page"), ("data-current", "")],
            vec![],
        ));
        assert!(!html.contains("aria-current"));
        assert!(!html.contains("data-current"));
    }

    // --- §3.25 規則 2: data-motion を一切出力しない機械的固定 ---

    #[test]
    fn no_part_outputs_data_motion() {
        let root_html = render(&root(&h(), "Main", vec![], vec![]));
        let item_html = render(&item(OpenState::Open, false, &h(), "a", vec![], vec![]));
        let trigger_html = render(&trigger(
            OpenState::Open,
            false,
            "a",
            None,
            None,
            vec![],
            vec![],
        ));
        let item_indicator_html =
            render(&item_indicator(OpenState::Open, &h(), "a", vec![], vec![]));
        let content_html = render(&content(
            OpenState::Open,
            &h(),
            "a",
            None,
            None,
            vec![],
            vec![],
        ));
        let link_html = render(&link("/docs", true, vec![], vec![]));
        for html in [
            root_html,
            item_html,
            trigger_html,
            item_indicator_html,
            content_html,
            link_html,
        ] {
            assert!(!html.contains("data-motion"));
        }
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&item(
            OpenState::Closed,
            false,
            &h(),
            "a",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="navigation-menu""#));
        assert!(html.contains(r#"data-part="item""#));
        assert!(!html.contains("attacker"));
    }

    // --- root > list > item > trigger + item-indicator + content(link) の組み立てテスト ---

    #[test]
    fn full_assembly_trigger_and_content_id_cross_reference() {
        let node = root(
            &h(),
            "Main",
            vec![],
            vec![list(
                &h(),
                vec![],
                vec![item(
                    OpenState::Open,
                    false,
                    &h(),
                    "products",
                    vec![],
                    vec![
                        trigger(
                            OpenState::Open,
                            false,
                            "products",
                            Some("t-trigger-a"),
                            Some("t-content-a"),
                            vec![],
                            vec![text("Products")],
                        ),
                        content(
                            OpenState::Open,
                            &h(),
                            "products",
                            Some("t-content-a"),
                            Some("t-trigger-a"),
                            vec![],
                            vec![link("/products/a", false, vec![], vec![text("A")])],
                        ),
                    ],
                )],
            )],
        );
        assert_eq!(
            render(&node),
            concat!(
                r#"<nav data-scope="navigation-menu" data-part="root" aria-label="Main" data-orientation="horizontal">"#,
                r#"<ul data-scope="navigation-menu" data-part="list" data-orientation="horizontal">"#,
                r#"<li data-scope="navigation-menu" data-part="item" data-state="open" data-orientation="horizontal" data-value="products">"#,
                r#"<button data-scope="navigation-menu" data-part="trigger" type="button" aria-expanded="true" data-state="open" data-value="products" id="t-trigger-a" aria-controls="t-content-a">Products</button>"#,
                r#"<div data-scope="navigation-menu" data-part="content" data-state="open" data-orientation="horizontal" data-value="products" id="t-content-a" aria-labelledby="t-trigger-a">"#,
                r#"<a data-scope="navigation-menu" data-part="link" href="/products/a">A</a>"#,
                r#"</div>"#,
                r#"</li>"#,
                r#"</ul>"#,
                r#"</nav>"#,
            )
        );
    }

    // --- NavigationMenu: dispatch 統合 ---

    #[test]
    fn navigation_menu_default_is_all_closed() {
        let m = NavigationMenu::default();
        assert_eq!(m.open_value(), None);
        assert!(!m.is_open("products"));
    }

    #[test]
    fn navigation_menu_dispatch_select_opens_at_most_one_item() {
        let mut m = NavigationMenu::default();
        assert!(dispatch(&mut m, "select", "products"));
        assert!(m.is_open("products"));
        assert!(!m.is_open("solutions"));

        assert!(dispatch(&mut m, "select", "solutions"));
        assert!(!m.is_open("products"));
        assert!(m.is_open("solutions"));
    }

    #[test]
    fn navigation_menu_dispatch_toggle_opens_then_closes() {
        let mut m = NavigationMenu::default();
        assert!(dispatch(&mut m, "toggle", "products"));
        assert!(m.is_open("products"));

        assert!(dispatch(&mut m, "toggle", "products"));
        assert!(!m.is_open("products"));
        assert_eq!(m.open_value(), None);
    }

    #[test]
    fn navigation_menu_dispatch_deselect_closes_all() {
        let mut m = NavigationMenu::default();
        dispatch(&mut m, "select", "products");
        assert!(dispatch(&mut m, "deselect", ""));
        assert_eq!(m.open_value(), None);
    }

    #[test]
    fn navigation_menu_dispatch_ignores_unknown_action() {
        let mut m = NavigationMenu::default();
        dispatch(&mut m, "select", "products");
        assert!(!dispatch(&mut m, "no_such_action", "solutions"));
        assert!(m.is_open("products"));
    }

    // --- NavigationMenu: 利便メソッド経由の描画が状態機械と一致 ---

    #[test]
    fn navigation_menu_convenience_methods_reflect_state() {
        let mut m = NavigationMenu::default();
        dispatch(&mut m, "select", "products");
        let props = h();

        let trigger_products = render(&m.trigger("products", false, None, None, vec![], vec![]));
        assert!(trigger_products.contains(r#"aria-expanded="true""#));
        assert!(trigger_products.contains(r#"data-state="open""#));

        let trigger_solutions = render(&m.trigger("solutions", false, None, None, vec![], vec![]));
        assert!(trigger_solutions.contains(r#"aria-expanded="false""#));

        let item_products = render(&m.item("products", false, &props, vec![], vec![]));
        assert!(item_products.contains(r#"data-state="open""#));
        assert!(item_products.contains(r#"data-value="products""#));

        let item_indicator_products = render(&m.item_indicator("products", &props, vec![], vec![]));
        assert!(item_indicator_products.contains(r#"data-state="open""#));

        let item_indicator_solutions =
            render(&m.item_indicator("solutions", &props, vec![], vec![]));
        assert!(item_indicator_solutions.contains(r#"data-state="closed""#));

        let content_products = render(&m.content("products", &props, None, None, vec![], vec![]));
        assert!(!content_products.contains("hidden"));

        let content_solutions = render(&m.content("solutions", &props, None, None, vec![], vec![]));
        assert!(content_solutions.contains(r#"hidden="""#));
    }

    // --- NavigationMenu: SSR 状態なし初期描画 ---

    #[test]
    fn navigation_menu_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&NavigationMenu::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- NavigationMenu: hydration 経路 ---

    #[test]
    fn navigation_menu_hydration_round_trip_selected() {
        let mut m = NavigationMenu::default();
        dispatch(&mut m, "select", "products");
        let rendered = render(&render_for_hydration(&m));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("products"));

        let restored = NavigationMenu::from_hydration_attrs(&m.hydration_attrs()).unwrap();
        assert_eq!(restored, m);
    }

    #[test]
    fn navigation_menu_hydration_round_trip_unselected() {
        let m = NavigationMenu::default();
        let restored = NavigationMenu::from_hydration_attrs(&m.hydration_attrs()).unwrap();
        assert_eq!(restored, m);
    }

    #[test]
    fn navigation_menu_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = NavigationMenu::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-selected".to_string())
        );
    }

    #[test]
    fn navigation_menu_from_hydration_attrs_invalid_value_does_not_panic() {
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["a".to_string(), "b".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = NavigationMenu::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn navigation_menu_view_root_is_element_for_render_for_hydration() {
        // render_for_hydration はルートが Node::Element であることを前提に
        // 属性を合成する（`crates/interactive/src/lib.rs` 参照）。本型の
        // view() が常に Element を返すことを固定する回帰テスト。
        let node = NavigationMenu::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }

    // --- XSS 回帰: label/id/controls/labelled_by/href/value/呼び出し側 attrs/children ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn root_label_payload_is_escaped_on_render() {
        let html = render(&root(&h(), ATTR_BREAK_PAYLOAD, vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn item_and_content_value_payload_is_escaped_on_render() {
        let item_html = render(&item(
            OpenState::Closed,
            false,
            &h(),
            ATTR_BREAK_PAYLOAD,
            vec![],
            vec![],
        ));
        assert!(!item_html.contains("onmouseover=\"alert(1)"));
        assert!(item_html.contains("&quot;"));

        let content_html = render(&content(
            OpenState::Open,
            &h(),
            ATTR_BREAK_PAYLOAD,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(!content_html.contains("onmouseover=\"alert(1)"));
        assert!(content_html.contains("&quot;"));
    }

    #[test]
    fn item_indicator_value_payload_is_escaped_on_render() {
        let html = render(&item_indicator(
            OpenState::Open,
            &h(),
            ATTR_BREAK_PAYLOAD,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn trigger_id_and_controls_payload_is_escaped_on_render() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            ATTR_BREAK_PAYLOAD,
            Some(ATTR_BREAK_PAYLOAD),
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    /// `data-value`（MAPPING_TABLE payload 源）に渡した攻撃ペイロードが
    /// `render()` の既定エスケープを経由してエスケープ済みで出力されること
    /// を固定する（`crate::accordion::item_trigger` の同名テストと同型、
    /// イシュー #1161）。
    #[test]
    fn trigger_data_value_payload_is_escaped_on_render() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            ATTR_BREAK_PAYLOAD,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn content_id_and_labelled_by_payload_is_escaped_on_render() {
        let html = render(&content(
            OpenState::Open,
            &h(),
            "a",
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
            &h(),
            "Main",
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&link(
            "/docs",
            false,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn link_dangerous_url_schemes_are_rejected() {
        let dangerous_urls = [
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "data:text/html;base64,PHNjcmlwdD4=",
            "vbscript:msgbox(1)",
        ];
        for url in dangerous_urls {
            let html = render(&link(url, false, vec![], vec![]));
            assert!(
                !html.contains("href="),
                "危険な URL スキームなのに href 属性が出力されている: url={url:?}, html={html}"
            );
        }
    }

    #[test]
    fn link_href_attribute_breakout_payload_is_escaped() {
        let html = render(&link(
            "/docs\" onmouseover=\"alert(1)",
            false,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
    }

    #[test]
    fn navigation_menu_dispatch_select_payload_is_escaped_on_render() {
        let mut m = NavigationMenu::default();
        let payload = "\"><script>alert(1)</script>";
        assert!(dispatch(&mut m, "select", payload));

        let rendered = render(&render_for_hydration(&m));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(!rendered.contains(r#""><script"#));
    }

    #[test]
    fn navigation_menu_xss_payload_in_hydration_selected_is_rejected_not_rendered() {
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["<script>alert(1)</script>".to_string(), "b".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = NavigationMenu::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}
