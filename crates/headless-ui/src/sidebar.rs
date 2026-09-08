//! Sidebar（shadcn/ui `Sidebar` 相当）headless コンポーネント（イシュー
//! #2072、親 #2071「sidebar を Primitives / Themes へ追加する」、Phase 4、
//! 祖父トラッキング参照軸 #2001）。
//!
//! `docs/design/shadcn-inventory.md`・`docs/design/component-coverage-map.md`
//! のとおり、shadcn/ui 固有部品かつ Blocks（#2007、dashboard-01 /
//! sidebar-07 / sidebar-03）の前提部品であり、他 3 参照軸（ark-ui /
//! chakra-ui / Radix）には対応物がない（[`mod@crate::command`]/
//! [`mod@crate::item`] と同型の位置付け）。アプリシェル用サイドバーの
//! `provider` / `root` / `header` / `content` / `footer` / `separator` /
//! `input` / `group` / `group-label` / `group-content` / `group-action` /
//! `menu` / `menu-item` / `menu-button` / `menu-action` / `menu-badge` /
//! `menu-sub` / `menu-sub-item` / `menu-sub-button` / `rail` / `trigger` /
//! `inset` の 22 anatomy パーツを提供する。
//!
//! 新規の意味論は持ち込まず、[`mod@crate::nav_list`]（`menu`/`menu-item`/
//! `menu-button` の構造・`aria-current` 語彙）・[`mod@crate::collapsible`]
//! （`menu-sub` の開閉は呼び出し側が `collapsible::root`/`trigger`/`content`
//! で合成する）・[`mod@crate::tooltip`]（`menu-button` の `describedby` が
//! `aria-describedby` のみで tooltip 関連付けを表現する）の語彙・パターンを
//! 再利用する。**いずれも他 scope のパーツを内包しない**（下記「他 scope を
//! 内包しない」節参照）。
//!
//! # 他 scope を内包しない（`data-scope="sidebar"` のみを出力する）
//!
//! [`menu_sub`] は開閉状態を持たない静的な `ul` であり、`collapsible::root`
//! 等を内部で呼び出さない（shadcn/ui 自体も `SidebarMenuSub` は状態を持たず、
//! `Collapsible` は呼び出し側が `SidebarMenuItem` の外側から合成する構造）。
//! [`menu_button`] は `describedby: Option<&str>` を受け取り
//! `aria-describedby` を出力するのみで、`tooltip::trigger`（`button`
//! そのもの）を内部に入れ子にしない（`button` の入れ子は不正 HTML）。
//! Examples（`docs-site` 側の原稿）でこれらの合成例を示す想定であり、本
//! モジュール自体はテストで scope 契約
//! （`tests/primitive_showcase.rs::resolved_scope_matches_the_page_kebab_for_every_entry`
//! 相当）に反しないことを担保する。
//!
//! # `data-state`/`data-collapsible`/`data-variant`/`data-side`/`data-mobile`
//!
//! [`provider`]/[`root`] は状態・静的 props から導出する 5 種の `data-*` を
//! 固定出力する: `data-state`（[`SidebarState`]、`"expanded"`/`"collapsed"`）
//! / `data-collapsible`（[`SidebarCollapsible`]、`"offcanvas"`/`"icon"`/
//! `"none"`）/ `data-variant`（[`SidebarVariant`]、`"sidebar"`/`"floating"`/
//! `"inset"`）/ `data-side`（[`SidebarSide`]、`"left"`/`"right"`）/
//! `data-mobile`（`mobile` が `true` のときのみ存在属性）。
//!
//! `data-collapsible` は状態に関わらず常に出力する（shadcn/ui は collapsed
//! 時のみ付与するが、本モジュールは SSR 決定性と CSS セレクタの単純化の
//! ため常時出力する意図的差分として記録する）。
//!
//! # `menu-skeleton` は本モジュールに置かない（意図的な非採用）
//!
//! 親 #2071 の anatomy 定義に列挙される `menu-skeleton`（ローディング装飾）
//! は本モジュールへ持ち込まない。shadcn/ui の実装はランダム幅を持ち SSR
//! 決定性を壊すため、`docs/policy/intentional-non-adoption.md` §3.25
//! 規則 2（装飾・アニメーションは headless へ持ち込まず Themes 層の責務と
//! する）により `fandhe-frontend-pre-styled-ui` 側（#2073）の責務へ割り当てる。
//!
//! # `inset` を `main` にしない理由
//!
//! shadcn/ui の `SidebarInset` は `main` だが、`fandhe-frontend-docs-site`
//! のページ骨格が既に `<main class="docs-main">` を描画しており、Demo で
//! `main` を入れ子にすると「文書内の非 hidden `main` は 1 個」という HTML
//! 制約に反する。よって [`inset`] は `div` とし、ランドマークが必要な
//! 利用者は `attrs` で `role="main"` を渡すか呼び出し側で `main_tag` を使う
//! （[`inset`] 自体は `role` を予約しない）。
//!
//! # `root` を `nav` にする理由
//!
//! `div` への `aria-label` は ARIA 仕様上 `generic` ロールへは意味を持たず
//! 支援技術へ露出しない。ランドマークが必要なため [`mod@crate::nav_list`]
//! と同じく `label` を必須引数にして型でアクセシブルネームを強制する。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-pre-styled-ui`（後続イシュー #2073）が本モジュールを
//! 呼んでスタイル済み Sidebar を組み立てる想定。`fandhe-frontend-wasm-full`
//! の Cmd/Ctrl+B・モバイル drawer 切替・`menu-button` の tooltip hover 配線
//! は後続イシュー #2074 の責務（本イシューのスコープ外）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`role`/`aria-*`/`data-*`/`href`/`type`/`tabindex`）はすべて
//!   `&'static str` リテラルで固定しており、動的値が属性名スロットへ混入
//!   する経路はない。
//! - 動的値（`label`/`href`/`id`/`controls`/`describedby`/`labelledby`/
//!   呼び出し側 `attrs`/`children`）は [`fandhe_frontend_core::render`] の
//!   既定エスケープを必ず経由する（REQ-1）。`raw_html()` は使用せず、HTML
//!   文字列を直接組み立てない。
//! - 呼び出し側 `attrs` による予約キーなりすましは各パート別 `*_RESERVED`
//!   定数 + [`drop_reserved`]（ASCII 大文字小文字無視の完全一致）が除去する
//!   （[`mod@crate::nav_list`]/[`mod@crate::item`] と同型）。`data-scope`/
//!   `data-part` の偽装は [`crate::anatomy::Anatomy::part`] が別途除去する。
//! - `data-hydrate-state` はクライアント改ざん可能な入力として扱い、
//!   [`SidebarState::from_data_state`] が未知値を `None` にし
//!   [`Sidebar::from_hydration_attrs`] が `HydrateError::InvalidValue` を
//!   返す（panic しない）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `fandhe-frontend-pre-styled-ui` の recipe・golden・`site/themes/sidebar.md`・
//!   `--fandhe-sidebar-*` トークン・`menu-skeleton` の実装は #2073。
//! - `fandhe-frontend-wasm-full` の Cmd/Ctrl+B・モバイル drawer 切替・
//!   tooltip hover 配線は #2074。
//! - `/themes/sidebar/` の docs-site ページ・
//!   `docs/design/component-coverage-map.md` の「実装済み」化は #2075。
//! - `mobile` のメディアクエリ判定自体（本モジュールは静的 `bool` prop を
//!   `data-mobile` へ写すのみ）は #2074。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_current, aria_describedby, aria_expanded, aria_label, AriaCurrent};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{
    Component, DirtyTracked, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX,
};

/// Sidebar の anatomy（`data-scope="sidebar"`）。
const ANATOMY: Anatomy = anatomy("sidebar");

/// `data-state` 属性値 "expanded"（[`SidebarState`] 参照）。
pub const DATA_STATE_EXPANDED: &str = "expanded";
/// `data-state` 属性値 "collapsed"。
pub const DATA_STATE_COLLAPSED: &str = "collapsed";

/// サイドバーの開閉状態。`data-state` 属性値（`"expanded"`/`"collapsed"`）と
/// 1:1 対応する。[`crate::state::OpenState`]（`"open"`/`"closed"`）とは語彙が
/// 異なるため独立した型として定義する（モジュール doc 参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarState {
    /// 展開している（SSR の状態なし初期描画に対応する既定値）。
    #[default]
    Expanded,
    /// 折りたたまれている。
    Collapsed,
}

impl SidebarState {
    /// `data-state`/`data-hydrate-state` 属性値へ変換する。
    #[must_use]
    pub fn as_data_state(self) -> &'static str {
        match self {
            SidebarState::Expanded => DATA_STATE_EXPANDED,
            SidebarState::Collapsed => DATA_STATE_COLLAPSED,
        }
    }

    /// `data-state`/`data-hydrate-state` 属性値から復元する。未知の値
    /// （改ざん・タイポ）は `None` を返す（安全側）。
    #[must_use]
    pub fn from_data_state(s: &str) -> Option<Self> {
        match s {
            DATA_STATE_EXPANDED => Some(SidebarState::Expanded),
            DATA_STATE_COLLAPSED => Some(SidebarState::Collapsed),
            _ => None,
        }
    }

    /// 開閉を反転した状態を返す。
    #[must_use]
    pub fn toggled(self) -> Self {
        match self {
            SidebarState::Expanded => SidebarState::Collapsed,
            SidebarState::Collapsed => SidebarState::Expanded,
        }
    }

    /// 展開しているかどうか。
    #[must_use]
    pub fn is_expanded(self) -> bool {
        matches!(self, SidebarState::Expanded)
    }
}

/// [`Sidebar`] に対する型付きアクション（[`crate::state::DisclosureAction`]
/// と同型、`payload` は使用しない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarAction {
    /// 展開する。
    Expand,
    /// 折りたたむ。
    Collapse,
    /// 展開/折りたたみを反転する。
    Toggle,
}

/// サイドバーの開閉状態機械（[`crate::state::Disclosure`] を手本に、
/// `"expanded"`/`"collapsed"` 語彙で独立実装する。モジュール doc参照）。
///
/// `dirty` は [`DirtyTracked::dirty_fields`] の実体であり、[`PartialEq`]/
/// [`Eq`] の比較対象から除外する（[`crate::state::Disclosure`] と同型）。
#[derive(Debug, Clone, Copy, Default)]
pub struct Sidebar {
    state: SidebarState,
    dirty: bool,
}

impl PartialEq for Sidebar {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

impl Eq for Sidebar {}

impl Sidebar {
    /// `data-hydrate-state` 属性名のフィールド部分。
    pub const FIELD_STATE: &'static str = "state";

    /// 指定した初期状態でサイドバー状態機械を生成する。
    #[must_use]
    pub fn new(initial: SidebarState) -> Self {
        Self {
            state: initial,
            dirty: false,
        }
    }

    /// 現在の開閉状態。
    #[must_use]
    pub fn state(&self) -> SidebarState {
        self.state
    }

    /// 現在の `data-state` 属性値。
    #[must_use]
    pub fn data_state(&self) -> &'static str {
        self.state.as_data_state()
    }
}

impl Component for Sidebar {
    type Action = SidebarAction;

    fn update(&mut self, action: SidebarAction) {
        let next = match action {
            SidebarAction::Expand => SidebarState::Expanded,
            SidebarAction::Collapse => SidebarState::Collapsed,
            SidebarAction::Toggle => self.state.toggled(),
        };
        self.dirty = next != self.state;
        self.state = next;
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー。実際の anatomy は [`provider`]/[`root`] 等を別途組み立てる。
    fn view(&self) -> Node {
        provider(
            self,
            &SidebarProps::default(),
            Vec::new(),
            vec![root(
                self,
                &SidebarProps::default(),
                "Sidebar",
                None,
                Vec::new(),
                Vec::new(),
            )],
        )
    }

    fn decode_action(name: &str, _payload: &str) -> Option<SidebarAction> {
        match name {
            "expand" => Some(SidebarAction::Expand),
            "collapse" => Some(SidebarAction::Collapse),
            "toggle" => Some(SidebarAction::Toggle),
            _ => None,
        }
    }
}

impl Hydrate for Sidebar {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        vec![(
            format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STATE),
            self.data_state().to_string(),
        )]
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let attr_name = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STATE);
        let raw = attrs
            .iter()
            .find(|(k, _)| *k == attr_name)
            .map(|(_, v)| v.as_str())
            .ok_or_else(|| HydrateError::MissingAttr(attr_name.clone()))?;
        let state =
            SidebarState::from_data_state(raw).ok_or_else(|| HydrateError::InvalidValue {
                attr: attr_name.clone(),
                reason: "expected \"expanded\" or \"collapsed\"".to_string(),
            })?;
        Ok(Self {
            state,
            dirty: false,
        })
    }
}

impl DirtyTracked for Sidebar {
    fn dirty_fields(&self) -> &[&'static str] {
        if self.dirty {
            &[Self::FIELD_STATE]
        } else {
            &[]
        }
    }
}

/// [`provider`]/[`root`] の `data-collapsible` 語彙（shadcn/ui `Sidebar` の
/// `collapsible` prop に対応）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarCollapsible {
    /// 折りたたみ時に画面外へ退避する（既定）。
    #[default]
    Offcanvas,
    /// 折りたたみ時にアイコンのみの幅で残る。
    Icon,
    /// 折りたたみを行わない。
    None,
}

impl SidebarCollapsible {
    /// `data-collapsible` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Offcanvas => "offcanvas",
            Self::Icon => "icon",
            Self::None => "none",
        }
    }
}

/// [`provider`]/[`root`] の `data-variant` 語彙（shadcn/ui `Sidebar` の
/// `variant` prop に対応）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarVariant {
    /// 既定の見た目。
    #[default]
    Sidebar,
    /// 浮遊する見た目。
    Floating,
    /// 埋め込み（インセット）の見た目。
    Inset,
}

impl SidebarVariant {
    /// `data-variant` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sidebar => "sidebar",
            Self::Floating => "floating",
            Self::Inset => "inset",
        }
    }
}

/// [`provider`]/[`root`] の `data-side` 語彙（shadcn/ui `Sidebar` の `side`
/// prop に対応）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarSide {
    /// 画面左側に配置する（既定）。
    #[default]
    Left,
    /// 画面右側に配置する。
    Right,
}

impl SidebarSide {
    /// `data-side` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

/// [`provider`]/[`root`] の静的 props（[`crate::field::FieldProps`] と同型の
/// `Default` derive 構造体）。
#[derive(Debug, Clone, Copy, Default)]
pub struct SidebarProps {
    /// 折りたたみ方式。
    pub collapsible: SidebarCollapsible,
    /// 見た目バリアント。
    pub variant: SidebarVariant,
    /// 配置側。
    pub side: SidebarSide,
    /// モバイル表示中かどうか。`true` のとき `data-mobile` を出力する。
    pub mobile: bool,
}

/// [`menu_button`]/[`menu_sub_button`] のサイズバリアント（shadcn/ui
/// `SidebarMenuButton` の `size` prop に対応、[`crate::item::ItemSize`] と
/// 同型の `as_str()` パターン）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarMenuButtonSize {
    /// 既定のサイズ。
    #[default]
    Default,
    /// 小さいサイズ。
    Sm,
    /// 大きいサイズ。
    Lg,
}

impl SidebarMenuButtonSize {
    /// `data-size` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sm => "sm",
            Self::Lg => "lg",
        }
    }
}

/// [`menu_button`] の見た目バリアント（shadcn/ui `SidebarMenuButton` の
/// `variant` prop に対応）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarMenuButtonVariant {
    /// 既定の見た目。
    #[default]
    Default,
    /// 枠線のみの見た目。
    Outline,
}

impl SidebarMenuButtonVariant {
    /// `data-variant` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Outline => "outline",
        }
    }
}

/// [`menu_sub_button`] のサイズバリアント（shadcn/ui `SidebarMenuSubButton`
/// の `size` prop に対応。既定バリアントは持たず `default` を持たない
/// 2 値のみ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarMenuSubButtonSize {
    /// 小さいサイズ（既定）。
    #[default]
    Sm,
    /// 中間サイズ。
    Md,
}

impl SidebarMenuSubButtonSize {
    /// `data-size` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
        }
    }
}

/// [`menu_button`] の描画引数（[`crate::item::ItemRootProps`] と同型の
/// ライフタイム付き props）。
#[derive(Debug, Clone, Copy, Default)]
pub struct SidebarMenuButtonProps<'a> {
    /// `Some` なら `a` として描画し `href` を固定付与する。`None` なら
    /// `button type="button"` として描画する。
    pub href: Option<&'a str>,
    /// `true` のとき `data-active` + （`a` のときのみ）`aria-current="page"`
    /// を付与する。
    pub active: bool,
    /// サイズバリアント。
    pub size: SidebarMenuButtonSize,
    /// 見た目バリアント。
    pub variant: SidebarMenuButtonVariant,
    /// `Some` のとき `aria-describedby` を付与する（tooltip 関連付け、
    /// モジュール doc「他 scope を内包しない」参照）。
    pub describedby: Option<&'a str>,
}

/// [`menu_sub_button`] の描画引数。
#[derive(Debug, Clone, Copy, Default)]
pub struct SidebarMenuSubButtonProps<'a> {
    /// `Some` なら `a`、`None` なら `button type="button"`。
    pub href: Option<&'a str>,
    /// `true` のとき `data-active` + （`a` のときのみ）`aria-current="page"`
    /// を付与する。
    pub active: bool,
    /// サイズバリアント。
    pub size: SidebarMenuSubButtonSize,
}

/// `data-active` 存在属性（[`crate::data_attrs`] を拡張せず本モジュール内
/// 私有ヘルパとする、[`mod@crate::drawer`] の `data_placement` と同型の
/// 判断）。
fn data_active(active: bool) -> Option<(&'static str, &'static str)> {
    active.then_some(("data-active", ""))
}

/// `data-mobile` 存在属性（本モジュール専用の私有ヘルパ）。
fn data_mobile(mobile: bool) -> Option<(&'static str, &'static str)> {
    mobile.then_some(("data-mobile", ""))
}

/// [`provider`]/[`root`] が共有する 5 種の `data-*`（モジュール doc
/// 「`data-state`/`data-collapsible`/`data-variant`/`data-side`/
/// `data-mobile`」参照）。
fn state_data_attrs(
    state: SidebarState,
    props: &SidebarProps,
) -> Vec<(&'static str, &'static str)> {
    let mut attrs = vec![
        ("data-state", state.as_data_state()),
        ("data-collapsible", props.collapsible.as_str()),
        ("data-variant", props.variant.as_str()),
        ("data-side", props.side.as_str()),
    ];
    attrs.extend(data_mobile(props.mobile));
    attrs
}

/// 呼び出し側 `attrs` から予約キー（本モジュールが固定付与する属性名）を
/// 除去する（ASCII 大文字小文字無視の完全一致、[`crate::nav_list::drop_reserved`]
/// と同型）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

const STATE_RESERVED: &[&str] = &[
    "data-state",
    "data-collapsible",
    "data-variant",
    "data-side",
    "data-mobile",
];
const ROOT_RESERVED: &[&str] = &[
    "aria-label",
    "id",
    "data-state",
    "data-collapsible",
    "data-variant",
    "data-side",
    "data-mobile",
];
const NO_RESERVED: &[&str] = &[];
const SEPARATOR_RESERVED: &[&str] = &["role"];
const GROUP_RESERVED: &[&str] = &["role", "aria-labelledby"];
const GROUP_LABEL_RESERVED: &[&str] = &["id"];
const GROUP_ACTION_RESERVED: &[&str] = &["type", "aria-label"];
const MENU_ACTION_RESERVED: &[&str] = &["type", "aria-label"];
const MENU_BUTTON_RESERVED: &[&str] = &[
    "href",
    "type",
    "data-active",
    "aria-current",
    "data-size",
    "data-variant",
    "aria-describedby",
];
const MENU_SUB_BUTTON_RESERVED: &[&str] =
    &["href", "type", "data-active", "aria-current", "data-size"];
const RAIL_RESERVED: &[&str] = &["type", "aria-label", "tabindex", "data-state"];
const TRIGGER_RESERVED: &[&str] = &[
    "type",
    "aria-label",
    "aria-expanded",
    "aria-controls",
    "data-state",
];

/// `provider` パーツ（`div`）。[`Sidebar`] の状態と [`SidebarProps`] から
/// 5 種の `data-*`（モジュール doc参照）を固定出力する。
#[must_use]
pub fn provider<'a>(
    state: &Sidebar,
    props: &SidebarProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = state_data_attrs(state.state(), props);
    merged.extend(attrs);
    ANATOMY.part("provider", "div", merged, children)
}

/// `root` パーツ（`nav`）。`label` は必須引数（モジュール doc「`root` を
/// `nav` にする理由」参照）。`id` は [`trigger`] の `aria-controls` の対に
/// 使う想定で `Some` のときのみ出力する。
#[must_use]
pub fn root<'a>(
    state: &Sidebar,
    props: &SidebarProps,
    label: &'a str,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![aria_label(label)];
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(state_data_attrs(state.state(), props));
    merged.extend(attrs);
    ANATOMY.part("root", "nav", merged, children)
}

/// `header` パーツ（`div`）。サイドバー上部スロット（ブランド・チーム切替
/// 等）。
#[must_use]
pub fn header<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("header", "div", attrs, children)
}

/// `content` パーツ（`div`）。スクロール可能な本文領域（`group` を並べる
/// 想定）。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("content", "div", attrs, children)
}

/// `footer` パーツ（`div`）。サイドバー下部スロット（ユーザーメニュー等）。
#[must_use]
pub fn footer<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("footer", "div", attrs, children)
}

/// `separator` パーツ（`hr`）。`hr` の暗黙 `separator` ロールに委ね、`role`
/// を明示付与しない（[`mod@crate::menu`] の `separator` と同型の判断）。
#[must_use]
pub fn separator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, SEPARATOR_RESERVED);
    ANATOMY.part("separator", "hr", attrs, children)
}

/// `input` パーツ（`input`、void 要素のため children を取らない）。
/// `type` は呼び出し側が `attrs` で指定する（shadcn `SidebarInput` は素の
/// Input）。
#[must_use]
pub fn input<'a>(attrs: Vec<(&'a str, &'a str)>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("input", "input", attrs, Vec::new())
}

/// `group` パーツ（`div`）。`role="group"` を固定出力し、`labelledby` が
/// `Some` のときのみ `aria-labelledby` を併記する（[`mod@crate::item`] の
/// `group` と同型の判断、[`group_label`] の `id` を参照する想定）。
#[must_use]
pub fn group<'a>(
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, GROUP_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![("role", "group")];
    if let Some(id) = labelledby {
        merged.push(("aria-labelledby", id));
    }
    merged.extend(attrs);
    ANATOMY.part("group", "div", merged, children)
}

/// `group-label` パーツ（`div`）。[`group`] の見出し。`id` は `Some` の
/// ときのみ出力し、[`group`] の `aria-labelledby` の対に使う。
#[must_use]
pub fn group_label<'a>(
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, GROUP_LABEL_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("group-label", "div", merged, children)
}

/// `group-content` パーツ（`div`）。[`group`] 内の本文（[`menu`] を置く
/// 想定）。
#[must_use]
pub fn group_content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("group-content", "div", attrs, children)
}

/// `group-action` パーツ（`button`）。`label` はアイコンのみの操作を想定した
/// 必須の `aria-label`（[`mod@crate::menu_action`] と同型）。
#[must_use]
pub fn group_action<'a>(
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, GROUP_ACTION_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button"), aria_label(label)];
    merged.extend(attrs);
    ANATOMY.part("group-action", "button", merged, children)
}

/// `menu` パーツ（`ul`）。暗黙の `list` ロールに委ね `role` を付与しない
/// （[`mod@crate::nav_list::list`] と同型）。
#[must_use]
pub fn menu<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("menu", "ul", attrs, children)
}

/// `menu-item` パーツ（`li`）。
#[must_use]
pub fn menu_item<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("menu-item", "li", attrs, children)
}

/// `menu-button` パーツ（`href` が `Some` なら `a`、`None` なら
/// `button type="button"`）。`active` が `true` のとき `data-active` を、
/// `a` かつ `active` のときのみ `aria-current="page"` を付与する
/// （[`mod@crate::nav_list::link`] の `current` 語彙を踏襲）。
#[must_use]
pub fn menu_button<'a>(
    props: &SidebarMenuButtonProps<'a>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, MENU_BUTTON_RESERVED);
    let is_link = props.href.is_some();
    let tag: &'static str = if is_link { "a" } else { "button" };
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(href) = props.href {
        merged.push(("href", href));
    } else {
        merged.push(("type", "button"));
    }
    merged.extend(data_active(props.active));
    if props.active && is_link {
        merged.push(aria_current(AriaCurrent::Page));
    }
    merged.push(("data-size", props.size.as_str()));
    merged.push(("data-variant", props.variant.as_str()));
    if let Some(describedby) = props.describedby {
        merged.push(aria_describedby(describedby));
    }
    merged.extend(attrs);
    ANATOMY.part("menu-button", tag, merged, children)
}

/// `menu-action` パーツ（`button`）。`label` は必須の `aria-label`
/// （[`group_action`] と同型）。
#[must_use]
pub fn menu_action<'a>(
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, MENU_ACTION_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button"), aria_label(label)];
    merged.extend(attrs);
    ANATOMY.part("menu-action", "button", merged, children)
}

/// `menu-badge` パーツ（`span`）。件数・状態表示等の装飾テキスト。
#[must_use]
pub fn menu_badge<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("menu-badge", "span", attrs, children)
}

/// `menu-sub` パーツ（`ul`）。開閉状態を持たない静的なサブメニュー（
/// モジュール doc「他 scope を内包しない」参照。開閉が必要な場合は呼び出し
/// 側で [`mod@crate::collapsible`] を合成する）。
#[must_use]
pub fn menu_sub<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("menu-sub", "ul", attrs, children)
}

/// `menu-sub-item` パーツ（`li`）。
#[must_use]
pub fn menu_sub_item<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("menu-sub-item", "li", attrs, children)
}

/// `menu-sub-button` パーツ（[`menu_button`] と同型の `a`/`button` 切替）。
#[must_use]
pub fn menu_sub_button<'a>(
    props: &SidebarMenuSubButtonProps<'a>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, MENU_SUB_BUTTON_RESERVED);
    let is_link = props.href.is_some();
    let tag: &'static str = if is_link { "a" } else { "button" };
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(href) = props.href {
        merged.push(("href", href));
    } else {
        merged.push(("type", "button"));
    }
    merged.extend(data_active(props.active));
    if props.active && is_link {
        merged.push(aria_current(AriaCurrent::Page));
    }
    merged.push(("data-size", props.size.as_str()));
    merged.extend(attrs);
    ANATOMY.part("menu-sub-button", tag, merged, children)
}

/// `rail` パーツ（`button`）。マウス専用のドラッグ/クリック領域
/// （shadcn/ui 準拠、`tabindex="-1"` でキーボードフォーカス対象から除外
/// する。キーボード経路は [`trigger`] が担う）。`label` は必須の
/// `aria-label`。
#[must_use]
pub fn rail<'a>(
    state: &Sidebar,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, RAIL_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        aria_label(label),
        ("tabindex", "-1"),
        ("data-state", state.data_state()),
    ];
    merged.extend(attrs);
    ANATOMY.part("rail", "button", merged, children)
}

/// `trigger` パーツ（`button`）。キーボード操作可能な開閉トリガー。`label`
/// は必須の `aria-label`。`controls` が `Some` のとき `aria-controls`
/// （[`root`] の `id` と対）を付与する。
#[must_use]
pub fn trigger<'a>(
    state: &Sidebar,
    label: &'a str,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, TRIGGER_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        aria_label(label),
        aria_expanded(state.state().is_expanded()),
    ];
    if let Some(id) = controls {
        merged.push(("aria-controls", id));
    }
    merged.push(("data-state", state.data_state()));
    merged.extend(attrs);
    ANATOMY.part("trigger", "button", merged, children)
}

/// `inset` パーツ（`div`）。サイドバー外側の主コンテンツ領域
/// （モジュール doc「`inset` を `main` にしない理由」参照。`role` は予約
/// しないため呼び出し側が `attrs` で `role="main"` を渡せる）。
#[must_use]
pub fn inset<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("inset", "div", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    fn expanded() -> Sidebar {
        Sidebar::new(SidebarState::Expanded)
    }

    fn collapsed() -> Sidebar {
        Sidebar::new(SidebarState::Collapsed)
    }

    #[test]
    fn provider_outputs_five_data_attrs_for_default_props() {
        let html = render(&provider(
            &expanded(),
            &SidebarProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.starts_with("<div"));
        assert!(html.contains(r#"data-scope="sidebar""#));
        assert!(html.contains(r#"data-part="provider""#));
        assert!(html.contains(r#"data-state="expanded""#));
        assert!(html.contains(r#"data-collapsible="offcanvas""#));
        assert!(html.contains(r#"data-variant="sidebar""#));
        assert!(html.contains(r#"data-side="left""#));
        assert!(!html.contains("data-mobile"));
    }

    #[test]
    fn provider_collapsed_and_mobile_reflect_state() {
        let props = SidebarProps {
            collapsible: SidebarCollapsible::Icon,
            variant: SidebarVariant::Floating,
            side: SidebarSide::Right,
            mobile: true,
        };
        let html = render(&provider(&collapsed(), &props, vec![], vec![]));
        assert!(html.contains(r#"data-state="collapsed""#));
        assert!(html.contains(r#"data-collapsible="icon""#));
        assert!(html.contains(r#"data-variant="floating""#));
        assert!(html.contains(r#"data-side="right""#));
        assert!(html.contains("data-mobile"));
    }

    #[test]
    fn root_is_nav_with_required_label_and_optional_id() {
        let html = render(&root(
            &expanded(),
            &SidebarProps::default(),
            "Main navigation",
            Some("app-sidebar"),
            vec![],
            vec![],
        ));
        assert!(html.starts_with("<nav"));
        assert!(html.contains(r#"aria-label="Main navigation""#));
        assert!(html.contains(r#"id="app-sidebar""#));
        assert!(html.contains(r#"data-state="expanded""#));
        assert!(!html.contains("role="));
    }

    #[test]
    fn root_without_id_omits_id_attribute() {
        let html = render(&root(
            &expanded(),
            &SidebarProps::default(),
            "Main navigation",
            None,
            vec![],
            vec![],
        ));
        assert!(!html.contains("id="));
    }

    #[test]
    fn header_content_footer_are_plain_divs() {
        for html in [
            render(&header(vec![], vec![])),
            render(&content(vec![], vec![])),
            render(&footer(vec![], vec![])),
        ] {
            assert!(html.starts_with("<div"));
            assert!(!html.contains("role="));
        }
    }

    #[test]
    fn separator_is_hr_without_explicit_role() {
        let html = render(&separator(vec![], vec![]));
        assert!(html.starts_with("<hr"));
        assert!(!html.contains("role="));
    }

    #[test]
    fn input_is_void_element_without_children() {
        let html = render(&input(vec![("type", "search")]));
        assert!(html.starts_with("<input"));
        assert!(!html.contains("</input>"));
        assert!(html.contains(r#"type="search""#));
    }

    #[test]
    fn group_outputs_role_group_and_optional_labelledby() {
        let with_label = render(&group(Some("g1-label"), vec![], vec![]));
        assert!(with_label.contains(r#"role="group""#));
        assert!(with_label.contains(r#"aria-labelledby="g1-label""#));

        let without_label = render(&group(None, vec![], vec![]));
        assert!(without_label.contains(r#"role="group""#));
        assert!(!without_label.contains("aria-labelledby"));
    }

    #[test]
    fn group_label_outputs_optional_id() {
        let html = render(&group_label(Some("g1-label"), vec![], vec![text("Team")]));
        assert!(html.contains(r#"id="g1-label""#));
    }

    #[test]
    fn group_action_requires_aria_label_and_type_button() {
        let html = render(&group_action("Add", vec![], vec![]));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-label="Add""#));
    }

    #[test]
    fn menu_button_href_renders_anchor_with_aria_current_when_active() {
        let props = SidebarMenuButtonProps {
            href: Some("/dashboard"),
            active: true,
            ..Default::default()
        };
        let html = render(&menu_button(&props, vec![], vec![text("Dashboard")]));
        assert!(html.starts_with("<a"));
        assert!(html.contains(r#"href="/dashboard""#));
        assert!(html.contains("data-active"));
        assert!(html.contains(r#"aria-current="page""#));
        assert!(!html.contains("type=\"button\""));
    }

    #[test]
    fn menu_button_without_href_renders_button_type_button_without_aria_current() {
        let props = SidebarMenuButtonProps {
            href: None,
            active: true,
            ..Default::default()
        };
        let html = render(&menu_button(&props, vec![], vec![]));
        assert!(html.starts_with("<button"));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains("data-active"));
        assert!(!html.contains("aria-current"));
        assert!(!html.contains("href="));
    }

    #[test]
    fn menu_button_inactive_omits_data_active_and_aria_current() {
        let props = SidebarMenuButtonProps {
            href: Some("/settings"),
            active: false,
            ..Default::default()
        };
        let html = render(&menu_button(&props, vec![], vec![]));
        assert!(!html.contains("data-active"));
        assert!(!html.contains("aria-current"));
    }

    #[test]
    fn menu_button_size_variant_describedby_are_output() {
        let props = SidebarMenuButtonProps {
            href: None,
            active: false,
            size: SidebarMenuButtonSize::Lg,
            variant: SidebarMenuButtonVariant::Outline,
            describedby: Some("tip-1"),
        };
        let html = render(&menu_button(&props, vec![], vec![]));
        assert!(html.contains(r#"data-size="lg""#));
        assert!(html.contains(r#"data-variant="outline""#));
        assert!(html.contains(r#"aria-describedby="tip-1""#));
    }

    #[test]
    fn menu_sub_button_mirrors_menu_button_link_switch() {
        let link_props = SidebarMenuSubButtonProps {
            href: Some("/a"),
            active: true,
            size: SidebarMenuSubButtonSize::Md,
        };
        let html = render(&menu_sub_button(&link_props, vec![], vec![]));
        assert!(html.starts_with("<a"));
        assert!(html.contains(r#"aria-current="page""#));
        assert!(html.contains(r#"data-size="md""#));

        let button_props = SidebarMenuSubButtonProps {
            href: None,
            active: false,
            size: SidebarMenuSubButtonSize::Sm,
        };
        let html = render(&menu_sub_button(&button_props, vec![], vec![]));
        assert!(html.starts_with("<button"));
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("aria-current"));
    }

    #[test]
    fn rail_has_tabindex_minus_one_and_data_state() {
        let html = render(&rail(&expanded(), "Toggle sidebar rail", vec![], vec![]));
        assert!(html.starts_with("<button"));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-label="Toggle sidebar rail""#));
        assert!(html.contains(r#"tabindex="-1""#));
        assert!(html.contains(r#"data-state="expanded""#));
    }

    #[test]
    fn trigger_reflects_aria_expanded_and_controls() {
        let html = render(&trigger(
            &expanded(),
            "Toggle sidebar",
            Some("app-sidebar"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-expanded="true""#));
        assert!(html.contains(r#"aria-controls="app-sidebar""#));
        assert!(html.contains(r#"data-state="expanded""#));

        let html_collapsed = render(&trigger(
            &collapsed(),
            "Toggle sidebar",
            None,
            vec![],
            vec![],
        ));
        assert!(html_collapsed.contains(r#"aria-expanded="false""#));
        assert!(!html_collapsed.contains("aria-controls"));
    }

    #[test]
    fn inset_is_div_not_main_and_role_is_not_reserved() {
        let html = render(&inset(vec![("role", "main")], vec![]));
        assert!(html.starts_with("<div"));
        assert!(html.contains(r#"role="main""#));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&menu(
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="sidebar""#));
        assert!(html.contains(r#"data-part="menu""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn menu_button_reserved_key_spoofing_is_dropped() {
        let props = SidebarMenuButtonProps {
            href: Some("/real"),
            active: false,
            ..Default::default()
        };
        let html = render(&menu_button(
            &props,
            vec![
                ("href", "javascript:alert(1)"),
                ("type", "submit"),
                ("data-active", ""),
                ("aria-current", "page"),
                ("data-size", "lg"),
                ("data-variant", "outline"),
                ("aria-describedby", "attacker"),
            ],
            vec![],
        ));
        assert_eq!(html.matches("href=").count(), 1);
        assert!(html.contains(r#"href="/real""#));
        assert!(!html.contains("submit"));
        assert!(!html.contains("data-active"));
        assert!(!html.contains("aria-current"));
        assert!(html.contains(r#"data-size="default""#));
        assert!(html.contains(r#"data-variant="default""#));
        assert!(!html.contains("aria-describedby"));
    }

    #[test]
    fn trigger_reserved_key_spoofing_is_dropped() {
        let html = render(&trigger(
            &collapsed(),
            "Toggle sidebar",
            None,
            vec![
                ("aria-expanded", "true"),
                ("aria-controls", "attacker"),
                ("data-state", "expanded"),
                ("type", "submit"),
                ("aria-label", "attacker"),
            ],
            vec![],
        ));
        assert_eq!(html.matches("aria-expanded").count(), 1);
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(!html.contains("aria-controls"));
        assert!(html.contains(r#"data-state="collapsed""#));
        assert!(!html.contains("submit"));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn menu_button_dangerous_url_schemes_are_rejected() {
        let dangerous_urls = [
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "data:text/html;base64,PHNjcmlwdD4=",
            "vbscript:msgbox(1)",
        ];
        for url in dangerous_urls {
            let props = SidebarMenuButtonProps {
                href: Some(url),
                ..Default::default()
            };
            let html = render(&menu_button(&props, vec![], vec![]));
            assert!(
                !html.contains("href="),
                "危険な URL スキームなのに href 属性が出力されている: url={url:?}, html={html}"
            );
        }
    }

    #[test]
    fn sidebar_state_default_is_expanded() {
        assert_eq!(Sidebar::default().state(), SidebarState::Expanded);
        assert_eq!(SidebarState::default(), SidebarState::Expanded);
    }

    #[test]
    fn sidebar_dispatch_expand_collapse_toggle() {
        let mut s = Sidebar::new(SidebarState::Expanded);
        s.update(SidebarAction::Collapse);
        assert_eq!(s.state(), SidebarState::Collapsed);
        assert_eq!(s.dirty_fields(), &[Sidebar::FIELD_STATE]);

        s.update(SidebarAction::Expand);
        assert_eq!(s.state(), SidebarState::Expanded);

        s.update(SidebarAction::Toggle);
        assert_eq!(s.state(), SidebarState::Collapsed);

        // no-op transition (same state) should not mark dirty.
        let mut s2 = Sidebar::new(SidebarState::Expanded);
        s2.update(SidebarAction::Expand);
        assert!(s2.dirty_fields().is_empty());
    }

    #[test]
    fn sidebar_decode_action_unknown_is_none() {
        assert_eq!(
            Sidebar::decode_action("expand", ""),
            Some(SidebarAction::Expand)
        );
        assert_eq!(
            Sidebar::decode_action("collapse", ""),
            Some(SidebarAction::Collapse)
        );
        assert_eq!(
            Sidebar::decode_action("toggle", ""),
            Some(SidebarAction::Toggle)
        );
        assert_eq!(Sidebar::decode_action("OPEN", ""), None);
        assert_eq!(Sidebar::decode_action("<script>", ""), None);
        assert_eq!(Sidebar::decode_action("", ""), None);
    }

    #[test]
    fn sidebar_hydration_round_trip() {
        let s = Sidebar::new(SidebarState::Collapsed);
        let attrs: Vec<(String, String)> = s.hydration_attrs();
        let restored = Sidebar::from_hydration_attrs(&attrs).expect("should restore");
        assert_eq!(restored.state(), SidebarState::Collapsed);
        assert!(restored.dirty_fields().is_empty());
    }

    #[test]
    fn sidebar_hydration_missing_attr_errors() {
        let err = Sidebar::from_hydration_attrs(&[]).unwrap_err();
        assert!(matches!(err, HydrateError::MissingAttr(_)));
    }

    #[test]
    fn sidebar_hydration_invalid_value_does_not_panic() {
        for bad in ["OPEN", "open", "<script>alert(1)</script>", ""] {
            let attrs = vec![(format!("{HYDRATE_ATTR_PREFIX}state"), bad.to_string())];
            let err = Sidebar::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    #[test]
    fn sidebar_view_produces_provider_root_composition() {
        let html = render(&Sidebar::new(SidebarState::Expanded).view());
        assert!(html.contains(r#"data-part="provider""#));
        assert!(html.contains(r#"data-part="root""#));
    }
}
