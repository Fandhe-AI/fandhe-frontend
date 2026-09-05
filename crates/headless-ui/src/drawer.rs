//! Drawer（画面端からスライドインするパネル）headless コンポーネント
//! （イシュー #758、親 #520）。
//!
//! ark-ui の Drawer（`.claude/skills/ark-ui/references/components/overlays/drawer.md`）・
//! chakra-ui の Drawer（`.claude/skills/chakra-ui/references/components/overlays/drawer.md`）
//! は WAI-ARIA 上 Dialog パターンの変種であり、開閉意味論そのものは通常の
//! ダイアログと同一である。このため本モジュールは**新規状態機械を作らず**、
//! [`crate::dialog`] の状態機械（内部は [`crate::state::Disclosure`]）を
//! [`Drawer`] 経由でそのまま再利用する。追加する要素は (1) `data-scope="drawer"`
//! の専用 anatomy（[`crate::dialog`] と同じ 8 パーツ構成）、(2) 画面のどの端
//! から出現するかを表す [`DrawerPlacement`] と、それを `root`/`positioner`/
//! `content` へ `data-placement` として出力する処理のみである。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（純粋関数で完結）を直接呼んで組み立てる。
//! CSR/hydration は [`Drawer`]（[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装、内部で [`crate::dialog::Dialog`]
//! へ全委譲）を経由し、dispatch（`"open"`/`"close"`/`"toggle"`）で状態遷移する。
//! `fandhe-frontend-pre-styled-ui` が本モジュールを呼んでスタイル済み Drawer を
//! 組み立てる想定である（`crates/pre-styled-ui/src/drawer.rs`）。
//!
//! # スコープ外（out-of-scope-tracking 対応）
//!
//! - フォーカストラップ・Escape キーでの閉鎖・外側クリックでの閉鎖・
//!   アニメーション対応の `open`/`visible` 分離は [`crate::dialog`] と同じく
//!   JS ランタイム側の責務であり本イシューのスコープ外（SSR/属性出力のみ）。
//! - **`fandhe-frontend-wasm-full` は drawer scope を一切配線していない
//!   （イシュー #1639 で判明した事実）**: `headless.rs` の
//!   `MAPPING_TABLE`（part → action 対応表）に `scope: "drawer"` の行が
//!   なくハイドレーション後も trigger/close-trigger の click が
//!   dispatch されない。`overlay.rs` の `OverlayKind::from_scope` は
//!   `"drawer"` を拒否し（Escape・外側クリック閉鎖なし）、
//!   `focus_trap.rs` の `should_trap` も `data-scope="dialog"` のみを
//!   対象とする（フォーカストラップ・フォーカス復帰なし）。つまり
//!   ハイドレーション後の Drawer は現状 **inert**（trigger 等が click
//!   に応答しない）である。「Escape・外側クリックのみ未対応」という
//!   従来の記述は過小であり、本節で正確に是正する。wasm-full 側の配線は
//!   別イシューで追跡する（fail-closed のため未配線でも安全側）。
//! - Grabber / snapPoints / draggable（ark-ui のドラッグ・スワイプ操作）は
//!   JS ランタイムの実行時計測・アニメーション関心であり、
//!   [`crate::dialog`] のスコープ外方針を継承する（
//!   `docs/policy/intentional-non-adoption.md` §3.25 規則 2）。
//!
//! # 参考サイトとの意図的な差分（イシュー #1639 で参照突合）
//!
//! ark-ui（zag `drawer.connect.ts`）・chakra-ui の Drawer と突合した結果、
//! anatomy パーツ・`data-*` 属性の増減は行っていない（Themes
//! （`fandhe-frontend-pre-styled-ui`）側への波及なし）。判定結果は以下の
//! 通り:
//!
//! - **是正**: [`content`] へ `tabindex="-1"` を固定付与した（zag
//!   `drawer.connect.ts`/WAI-ARIA dialog パターンの前提。詳細は [`content`]
//!   の rustdoc 参照）。
//! - **意図的に非採用**（維持）: `grabber`/`grabber-indicator`/
//!   `swipe-area`/`indent`/`indent-background` パーツ、
//!   `data-swipe-direction`/`data-swiping`/`data-dragging`/`data-expanded`/
//!   `data-nested-drawer-*` 属性はいずれもドラッグ操作・スタック積層の
//!   実行時計測に紐づく装飾関心であり headless 層へ持ち込まない。trigger
//!   の `data-ownedby`/`data-value`/`data-current`（zag 固有）も採用せず、
//!   既存の `aria-controls` による id 関連付けで代替する（[`crate::dialog`]
//!   と同判断）。`data-placement`（論理方向、chakra-ui の `placement`
//!   語彙・RTL 対応・pre-styled-ui recipe の依存元）は物理方向の
//!   `data-swipe-direction` へ置き換えない。
//! - **意図的な差分**（維持）: `root` パートは全部品共通の
//!   `data-state` 付与先として維持する。`backdrop` の `aria-hidden="true"`
//!   （zag は付けないが装飾層として読み上げ対象外にする既存方針）・
//!   `content` の `role="dialog"` 固定（zag は `alertdialog` も選べるが
//!   Drawer は確認/警告用途ではなく常設ナビ/フィルタ補助パネル用途に
//!   限定する既存判断）を維持する。chakra-ui の Header/Body/Footer/
//!   ActionTrigger は `fandhe-frontend-pre-styled-ui`（Themes 層）の関心で
//!   あり headless anatomy には持ち込まない。
//! - **キーボード操作**: Enter（trigger で開く）・Tab/Shift+Tab
//!   （フォーカストラップ）・Esc（閉じてフォーカス復帰）はいずれも
//!   ark-ui/chakra-ui では JS ランタイム（dismissable layer・focus trap）
//!   が担う。上記のとおり wasm-full は drawer scope を未配線のため、
//!   現時点でこれらのキー操作は一切動作しない。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`/`hidden`/`id`/`tabindex`）は
//!   すべて `&'static str` リテラルで固定しており、動的値が属性名スロットへ
//!   混入する経路はない（[`mod@crate::anatomy`]/[`crate::aria`]/
//!   [`crate::data_attrs`] の既存不変条件をそのまま継承する）。
//! - 動的値（`id`/`labelledby`/`describedby`/`controls`/呼び出し側 `attrs`/
//!   `children` テキスト）は [`fandhe_frontend_core::render`] の既定エスケープを
//!   必ず経由する。`raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-state` 値語彙（`"open"`/`"closed"`）は [`crate::state::OpenState`]
//!   に一元化する（[`crate::dialog`] と同一）。`data-placement` 値語彙は
//!   [`DrawerPlacement`] に一元化し、任意文字列は受け付けない。
//! - hydration 属性（`data-hydrate-state`）はクライアント側で改ざんされうる
//!   入力として扱う。[`Drawer`] の [`fandhe_frontend_interactive::Hydrate`]
//!   実装は [`crate::dialog::Dialog`] へ全委譲することで、panic せず
//!   `HydrateError` を返す既存保証をそのまま継承する。placement は描画設定
//!   （状態ではない）のため hydration 属性としては `data-hydrate-placement`
//!   を別途出力するが、未知値は状態と同じく `HydrateError::InvalidValue` で
//!   fail-closed に拒否する（クライアント改ざん入力扱い）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{
    aria_controls, aria_describedby, aria_expanded, aria_haspopup, aria_hidden, aria_labelledby,
    aria_modal, role, AriaPopup,
};
use crate::data_attrs::data_state;
use crate::dialog::{ContentIds, Dialog, DialogRole};
use crate::state::{DisclosureAction, OpenState};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// Drawer の anatomy（`data-scope="drawer"`）。[`crate::dialog`] と同じ
/// 8 パーツ構成（root/trigger/backdrop/positioner/content/title/description/
/// close-trigger）を持つが、`data-scope` 値のみ異なる（呼び出し側が dialog と
/// drawer を CSS セレクタで独立にスタイルできるようにするための分離）。
const ANATOMY: Anatomy = anatomy("drawer");

/// 呼び出し側 `attrs` から `tabindex`（大文字小文字を無視）を除去する
/// （[`crate::dialog::drop_tabindex_attr`] と同型のパターン。クレート API
/// 表面を増やさないため再利用せずここへ複製する）。[`content`] が
/// `tabindex="-1"` を固定付与する前に呼ぶことで、呼び出し側が渡した
/// `tabindex` との重複出力（SSR は両方出力して先勝ち、wasm-client の
/// `set_attribute` は後勝ちになる描画経路間の不一致）を防ぐ。
fn drop_tabindex_attr<'a>(attrs: Vec<(&'a str, &'a str)>) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("tabindex"))
        .collect()
}

/// Drawer がどの画面端から出現するか（`data-placement` の固定語彙、chakra-ui
/// の `placement` prop 相当）。任意文字列は受け付けない。
///
/// `start`/`end` は論理方向（RTL 対応、[`crate::pre_styled_ui` 側で
/// `margin-inline-*` 等の論理プロパティに変換される契約]。物理方向の
/// `left`/`right` ではなく `start`/`end` を採用するのは
/// [`crate::positioning::Side`]（`top`/`bottom`/`left`/`right` の物理方向、
/// floating-ui 系の positioner 用）とは異なる語彙が必要なため
/// （drawer は画面端固定パネルであり、追従計算を行う floating-ui の
/// side/align とは無関係）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawerPlacement {
    /// 書字方向の開始側（LTR では左、RTL では右）。
    Start,
    /// 書字方向の終了側（LTR では右、RTL では左。chakra-ui の既定値）。
    End,
    /// 画面上端。
    Top,
    /// 画面下端。
    Bottom,
}

impl Default for DrawerPlacement {
    /// chakra-ui の既定 placement（`end`）に合わせる。
    fn default() -> Self {
        Self::End
    }
}

impl DrawerPlacement {
    /// `data-placement` 属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }

    /// 属性値文字列からのパース。未知の値は `None`（fail-closed。
    /// hydration での復元・クライアント側改ざん入力の検証双方に使う契約は
    /// [`crate::positioning::Side::from_str`] と同型）。
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "start" => Some(Self::Start),
            "end" => Some(Self::End),
            "top" => Some(Self::Top),
            "bottom" => Some(Self::Bottom),
            _ => None,
        }
    }
}

/// `data-placement` 属性を組み立てる（内部ヘルパ）。値は [`DrawerPlacement`]
/// の固定語彙のみを受け付ける。
fn data_placement(placement: DrawerPlacement) -> (&'static str, &'static str) {
    ("data-placement", placement.as_str())
}

/// Root パーツ（`div`）。開閉状態・placement を `data-*` へ反映する。
#[must_use]
pub fn root<'a>(
    state: OpenState,
    placement: DrawerPlacement,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![data_state(state.as_data_state()), data_placement(placement)];
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Trigger パーツ（`button`）。[`crate::dialog::trigger`] と同一契約
/// （`type="button"` 固定・`aria-haspopup="dialog"`・`aria-expanded`・
/// `controls` が `Some` のとき `aria-controls`）。
#[must_use]
pub fn trigger<'a>(
    state: OpenState,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        aria_haspopup(AriaPopup::Dialog),
        aria_expanded(state.is_open()),
        data_state(state.as_data_state()),
    ];
    if let Some(id) = controls {
        merged.push(aria_controls(id));
    }
    merged.extend(attrs);
    ANATOMY.part("trigger", "button", merged, children)
}

/// Backdrop パーツ（`div`）。[`crate::dialog::backdrop`] と同一契約。
#[must_use]
pub fn backdrop<'a>(state: OpenState, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![data_state(state.as_data_state()), aria_hidden(true)];
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("backdrop", "div", merged, children)
}

/// Positioner パーツ（`div`）。[`content`] の配置用ラッパー。placement を
/// `data-*` へ反映する（styled 層が方向別のレイアウトを切り替える起点）。
#[must_use]
pub fn positioner<'a>(
    state: OpenState,
    placement: DrawerPlacement,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![data_state(state.as_data_state()), data_placement(placement)];
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("positioner", "div", merged, children)
}

/// Content パーツ（`div`）。ダイアログパターン準拠のため `role="dialog"` を
/// 固定する（[`crate::dialog::DialogRole::Alertdialog`] 相当の変種は提供しない。
/// Drawer は確認・警告用途ではなく、常設ナビ/フィルタ等の補助パネルという
/// chakra-ui/ark-ui 双方の用途と一致させるため）。`aria-modal`・closed 時の
/// `hidden`・placement を付与する。`ids`（[`ContentIds`]、[`crate::dialog`]
/// を再利用）の各フィールドが `Some` のときのみ対応する属性を出力する。
///
/// `tabindex="-1"` を固定で付与する（zag `drawer.connect.ts`/WAI-ARIA
/// dialog パターンの前提と同じく、プログラム的フォーカスのみを許可する
/// ため。[`crate::dialog::content`] の同一是正（イシュー #1638）と揃える
/// 判断だが、**drawer scope は `fandhe-frontend-wasm-full` の
/// `focus_trap` 配線対象外**（モジュール冒頭「スコープ外」節参照）のため、
/// dialog のような「wasm-full 側の動的付与と SSR 出力の一致」根拠は
/// 成立しない。本関数の付与は SSR/静的属性としての正当性のみに基づく
/// （イシュー #1639）。呼び出し側 `attrs` に `tabindex` が含まれる場合は
/// [`drop_tabindex_attr`] で事前に除去してから固定値を合成するため、
/// 出力に重複した `tabindex` 属性は生じない（[`crate::dialog::content`]
/// と同一の対策）。
#[must_use]
pub fn content<'a>(
    state: OpenState,
    placement: DrawerPlacement,
    modal: bool,
    ids: ContentIds<'a>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role(DialogRole::Dialog.as_str()),
        aria_modal(modal),
        data_state(state.as_data_state()),
        data_placement(placement),
        ("tabindex", "-1"),
    ];
    if let Some(id) = ids.id {
        merged.push(("id", id));
    }
    if let Some(labelledby) = ids.labelledby {
        merged.push(aria_labelledby(labelledby));
    }
    if let Some(describedby) = ids.describedby {
        merged.push(aria_describedby(describedby));
    }
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(drop_tabindex_attr(attrs));
    ANATOMY.part("content", "div", merged, children)
}

/// Title パーツ（`h2`）。[`crate::dialog::title`] と同一契約。
#[must_use]
pub fn title<'a>(id: Option<&'a str>, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("title", "h2", merged, children)
}

/// Description パーツ（`p`）。[`crate::dialog::description`] と同一契約。
#[must_use]
pub fn description<'a>(
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("description", "p", merged, children)
}

/// CloseTrigger パーツ（`button`）。[`crate::dialog::close_trigger`] と
/// 同一契約（`type="button"` 固定、ラベルは呼び出し側が `attrs`/`children` で
/// 付与する）。
#[must_use]
pub fn close_trigger<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    merged.extend(attrs);
    ANATOMY.part("close-trigger", "button", merged, children)
}

/// [`crate::dialog::Dialog`] を埋め込んだ Drawer の状態機械。
///
/// 開閉状態機械は**新規に作らず** [`crate::dialog::Dialog`]（内部は
/// [`crate::state::Disclosure`]）へ全委譲する（モジュール冒頭 rustdoc 参照）。
/// `placement` は描画設定であり状態機械には持たせない（`Disclosure` の
/// 開閉状態と独立、hydration では別属性 `data-hydrate-placement` として
/// 往復する）。`Default` は `placement` = [`DrawerPlacement::End`]・
/// 状態 = [`OpenState::Closed`]。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Drawer {
    dialog: Dialog,
    placement: DrawerPlacement,
}

impl Drawer {
    /// 指定した初期状態・placement で Drawer を生成する。
    #[must_use]
    pub fn new(initial: OpenState, placement: DrawerPlacement) -> Self {
        Self {
            dialog: Dialog::new(initial),
            placement,
        }
    }

    /// 現在の開閉状態。
    #[must_use]
    pub fn state(&self) -> OpenState {
        self.dialog.state()
    }

    /// 現在の placement。
    #[must_use]
    pub fn placement(&self) -> DrawerPlacement {
        self.placement
    }

    /// 現在の `data-state` 属性値（`"open"`/`"closed"`）。
    #[must_use]
    pub fn data_state(&self) -> &'static str {
        self.dialog.data_state()
    }

    /// 開いているかどうか。
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.dialog.is_open()
    }

    /// [`root`] へ現在の状態・placement を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        root(self.state(), self.placement, attrs, children)
    }

    /// [`trigger`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn trigger<'a>(
        &self,
        controls: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        trigger(self.state(), controls, attrs, children)
    }

    /// [`backdrop`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn backdrop<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        backdrop(self.state(), attrs, children)
    }

    /// [`positioner`] へ現在の状態・placement を注入する利便メソッド。
    #[must_use]
    pub fn positioner<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        positioner(self.state(), self.placement, attrs, children)
    }

    /// [`content`] へ現在の状態・placement を注入する利便メソッド。
    #[must_use]
    pub fn content<'a>(
        &self,
        modal: bool,
        ids: ContentIds<'a>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        content(self.state(), self.placement, modal, ids, attrs, children)
    }
}

impl Component for Drawer {
    type Action = DisclosureAction;

    /// 開閉アクション（`"open"`/`"close"`/`"toggle"`）は
    /// [`crate::dialog::Dialog`] へそのまま委譲する。placement を変更する
    /// アクションは提供しない（描画設定は再構築で切り替える契約、
    /// `fandhe-frontend-interactive` の `Component` は同一 `Action` 型のまま
    /// 状態のみを更新する設計と整合させる）。
    fn update(&mut self, action: DisclosureAction) {
        self.dialog.update(action);
    }

    /// 共通契約（`data-state`/`data-placement` 整合・hydration ルート）のみを
    /// 表す最小正準ビュー（[`crate::dialog::Dialog::view`] と同じ位置付け。
    /// 公開 UI としての利用は想定しない）。
    fn view(&self) -> Node {
        self.root(
            Vec::new(),
            vec![
                trigger(self.state(), None, Vec::new(), Vec::new()),
                positioner(
                    self.state(),
                    self.placement,
                    Vec::new(),
                    vec![content(
                        self.state(),
                        self.placement,
                        true,
                        ContentIds::default(),
                        Vec::new(),
                        Vec::new(),
                    )],
                ),
            ],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<DisclosureAction> {
        Dialog::decode_action(name, payload)
    }
}

/// hydration 属性名（placement 用）。`data-hydrate-state` は
/// [`crate::dialog::Dialog`]（内部の [`crate::state::Disclosure`]）が既に
/// 出力するため、本モジュールは placement 専用の属性のみを追加する。
const HYDRATE_PLACEMENT_ATTR: &str = "data-hydrate-placement";

impl Hydrate for Drawer {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let mut attrs = self.dialog.hydration_attrs();
        attrs.push((
            HYDRATE_PLACEMENT_ATTR.to_string(),
            self.placement.as_str().to_string(),
        ));
        attrs
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let dialog = Dialog::from_hydration_attrs(attrs)?;
        let raw = attrs
            .iter()
            .find(|(k, _)| k == HYDRATE_PLACEMENT_ATTR)
            .map(|(_, v)| v.as_str())
            .ok_or_else(|| HydrateError::MissingAttr(HYDRATE_PLACEMENT_ATTR.to_string()))?;
        let placement =
            DrawerPlacement::from_str(raw).ok_or_else(|| HydrateError::InvalidValue {
                attr: HYDRATE_PLACEMENT_ATTR.to_string(),
                reason: "expected \"start\"/\"end\"/\"top\"/\"bottom\"".to_string(),
            })?;
        Ok(Self { dialog, placement })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part/data-state/data-placement 出力 ---

    #[test]
    fn root_outputs_scope_part_state_and_placement() {
        let html = render(&root(
            OpenState::Closed,
            DrawerPlacement::Start,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="drawer""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(html.contains(r#"data-placement="start""#));
    }

    #[test]
    fn all_four_placements_output_expected_data_placement() {
        for (placement, expected) in [
            (DrawerPlacement::Start, "start"),
            (DrawerPlacement::End, "end"),
            (DrawerPlacement::Top, "top"),
            (DrawerPlacement::Bottom, "bottom"),
        ] {
            let html = render(&root(OpenState::Open, placement, vec![], vec![]));
            assert!(html.contains(&format!(r#"data-placement="{expected}""#)));
        }
    }

    #[test]
    fn drawer_placement_default_is_end() {
        assert_eq!(DrawerPlacement::default(), DrawerPlacement::End);
    }

    #[test]
    fn drawer_placement_from_str_round_trips_and_rejects_unknown() {
        for placement in [
            DrawerPlacement::Start,
            DrawerPlacement::End,
            DrawerPlacement::Top,
            DrawerPlacement::Bottom,
        ] {
            assert_eq!(
                DrawerPlacement::from_str(placement.as_str()),
                Some(placement)
            );
        }
        assert_eq!(DrawerPlacement::from_str("left"), None);
        assert_eq!(DrawerPlacement::from_str(""), None);
        assert_eq!(DrawerPlacement::from_str("<script>alert(1)</script>"), None);
    }

    #[test]
    fn trigger_has_type_button_haspopup_and_aria_expanded() {
        let html = render(&trigger(OpenState::Closed, None, vec![], vec![]));
        assert!(html.contains(r#"<button"#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-haspopup="dialog""#));
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(!html.contains("aria-controls"));

        let html_open = render(&trigger(OpenState::Open, None, vec![], vec![]));
        assert!(html_open.contains(r#"aria-expanded="true""#));
    }

    #[test]
    fn trigger_controls_some_outputs_aria_controls() {
        let html = render(&trigger(OpenState::Closed, Some("dw-1"), vec![], vec![]));
        assert!(html.contains(r#"aria-controls="dw-1""#));
    }

    #[test]
    fn backdrop_open_has_no_hidden_and_always_aria_hidden() {
        let open = render(&backdrop(OpenState::Open, vec![], vec![]));
        assert!(!open.contains("hidden=\"\""));
        assert!(open.contains(r#"aria-hidden="true""#));

        let closed = render(&backdrop(OpenState::Closed, vec![], vec![]));
        assert!(closed.contains(r#"hidden="""#));
    }

    #[test]
    fn positioner_closed_has_hidden_open_does_not_and_carries_placement() {
        let closed = render(&positioner(
            OpenState::Closed,
            DrawerPlacement::Bottom,
            vec![],
            vec![],
        ));
        assert!(closed.contains(r#"hidden="""#));
        assert!(closed.contains(r#"data-placement="bottom""#));

        let open = render(&positioner(
            OpenState::Open,
            DrawerPlacement::Bottom,
            vec![],
            vec![],
        ));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn content_role_is_dialog_fixed_and_aria_modal_reflects_argument() {
        let html = render(&content(
            OpenState::Open,
            DrawerPlacement::End,
            true,
            ContentIds::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"role="dialog""#));
        assert!(html.contains(r#"aria-modal="true""#));

        let non_modal = render(&content(
            OpenState::Open,
            DrawerPlacement::End,
            false,
            ContentIds::default(),
            vec![],
            vec![],
        ));
        assert!(non_modal.contains(r#"aria-modal="false""#));
    }

    #[test]
    fn content_closed_has_hidden_open_does_not() {
        let closed = render(&content(
            OpenState::Closed,
            DrawerPlacement::End,
            true,
            ContentIds::default(),
            vec![],
            vec![],
        ));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&content(
            OpenState::Open,
            DrawerPlacement::End,
            true,
            ContentIds::default(),
            vec![],
            vec![],
        ));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn content_id_labelledby_describedby_some_output_attrs() {
        let html = render(&content(
            OpenState::Open,
            DrawerPlacement::End,
            true,
            ContentIds {
                id: Some("dw1"),
                labelledby: Some("dw1-title"),
                describedby: Some("dw1-desc"),
            },
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"id="dw1""#));
        assert!(html.contains(r#"aria-labelledby="dw1-title""#));
        assert!(html.contains(r#"aria-describedby="dw1-desc""#));
    }

    #[test]
    fn title_and_description_id_some_output_id() {
        let title_html = render(&title(Some("dw1-title"), vec![], vec![text("Menu")]));
        assert!(title_html.contains(r#"<h2"#));
        assert!(title_html.contains(r#"id="dw1-title""#));

        let desc_html = render(&description(
            Some("dw1-desc"),
            vec![],
            vec![text("Navigation")],
        ));
        assert!(desc_html.contains(r#"<p"#));
        assert!(desc_html.contains(r#"id="dw1-desc""#));
    }

    #[test]
    fn close_trigger_has_type_button() {
        let html = render(&close_trigger(vec![], vec![text("Close")]));
        assert!(html.contains(r#"<button"#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"data-part="close-trigger""#));
    }

    #[test]
    fn content_has_tabindex_minus_one() {
        // zag `drawer.connect.ts`/WAI-ARIA dialog パターンと同じく、content は
        // 開閉に関わらず `tabindex="-1"` を固定で持つ（イシュー #1639）。
        let closed = render(&content(
            OpenState::Closed,
            DrawerPlacement::End,
            true,
            ContentIds::default(),
            vec![],
            vec![],
        ));
        assert!(closed.contains(r#"tabindex="-1""#));

        let open = render(&content(
            OpenState::Open,
            DrawerPlacement::End,
            true,
            ContentIds::default(),
            vec![],
            vec![],
        ));
        assert!(open.contains(r#"tabindex="-1""#));
    }

    #[test]
    fn content_drops_caller_tabindex_to_keep_fixed_minus_one() {
        // 呼び出し側 attrs に tabindex（大文字小文字違い含む）を渡しても
        // 固定の `tabindex="-1"` のみが 1 つだけ出力される（PR #1911
        // codex-review/Cursor Bugbot 指摘: 除去しないと SSR は重複属性を
        // 出力し、wasm-client の set_attribute は後勝ちで呼び出し側の値が
        // 有効になり描画経路間で結果が食い違う。crate::dialog::content と
        // 同一の対策、イシュー #1639）。
        let rendered = render(&content(
            OpenState::Open,
            DrawerPlacement::End,
            true,
            ContentIds::default(),
            vec![("TabIndex", "0")],
            vec![],
        ));
        assert_eq!(rendered.matches("tabindex").count(), 1);
        assert!(rendered.contains(r#"tabindex="-1""#));
        assert!(!rendered.contains(r#"tabindex="0""#));
    }

    #[test]
    fn no_part_outputs_drag_or_swipe_vocabulary() {
        // ark-ui（zag drawer.connect.ts）のドラッグ・スワイプ・ネスト計測
        // 語彙（grabber/swipe-area パート・data-swipe-direction 等）は
        // 意図的非採用（イシュー #1639、モジュール冒頭「参考サイトとの
        // 意図的な差分」節）。全 8 パートを open/closed 双方で描画しても
        // これらの語彙が一切出力されないことを固定する。
        let ids = ContentIds {
            id: Some("dw1"),
            labelledby: Some("dw1-title"),
            describedby: Some("dw1-desc"),
        };
        for state in [OpenState::Open, OpenState::Closed] {
            let htmls = [
                render(&root(state, DrawerPlacement::End, vec![], vec![])),
                render(&trigger(state, Some("dw1"), vec![], vec![])),
                render(&backdrop(state, vec![], vec![])),
                render(&positioner(state, DrawerPlacement::End, vec![], vec![])),
                render(&content(
                    state,
                    DrawerPlacement::End,
                    true,
                    ids,
                    vec![],
                    vec![],
                )),
                render(&title(Some("dw1-title"), vec![], vec![text("Menu")])),
                render(&description(
                    Some("dw1-desc"),
                    vec![],
                    vec![text("Navigation")],
                )),
                render(&close_trigger(vec![], vec![text("Close")])),
            ];
            for html in htmls {
                assert!(!html.contains("data-swipe-direction"));
                assert!(!html.contains("data-swiping"));
                assert!(!html.contains("data-dragging"));
                assert!(!html.contains("data-expanded"));
                assert!(!html.contains("data-nested-drawer"));
                assert!(!html.contains(r#"data-part="grabber""#));
                assert!(!html.contains(r#"data-part="grabber-indicator""#));
                assert!(!html.contains(r#"data-part="swipe-area""#));
                assert!(!html.contains(r#"data-part="indent""#));
                assert!(!html.contains(r#"data-part="indent-background""#));
            }
        }
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            OpenState::Closed,
            DrawerPlacement::End,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="drawer""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- Drawer: dispatch 統合（dialog 状態機械への委譲） ---

    #[test]
    fn drawer_default_is_closed_and_placement_end() {
        let d = Drawer::default();
        assert_eq!(d.state(), OpenState::Closed);
        assert_eq!(d.placement(), DrawerPlacement::End);
    }

    #[test]
    fn drawer_dispatch_toggle_open_close_delegate_to_dialog() {
        let mut d = Drawer::default();
        assert!(render(&d.root(vec![], vec![])).contains(r#"data-state="closed""#));

        assert!(dispatch(&mut d, "toggle", ""));
        assert!(render(&d.root(vec![], vec![])).contains(r#"data-state="open""#));

        assert!(dispatch(&mut d, "close", ""));
        assert!(!d.is_open());

        assert!(dispatch(&mut d, "open", ""));
        assert!(d.is_open());
    }

    #[test]
    fn drawer_dispatch_ignores_unknown_action() {
        let mut d = Drawer::new(OpenState::Open, DrawerPlacement::Top);
        assert!(!dispatch(&mut d, "no_such_action", "x"));
        assert_eq!(d.state(), OpenState::Open);
        assert_eq!(d.placement(), DrawerPlacement::Top);
    }

    // --- Drawer: SSR 状態なし初期描画 ---

    #[test]
    fn drawer_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Drawer::default().view());
        assert!(rendered.contains(r#"data-state="closed""#));
        assert!(rendered.contains(r#"data-placement="end""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    #[test]
    fn drawer_view_root_is_element_for_render_for_hydration() {
        let node = Drawer::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }

    // --- Drawer: hydration 経路（状態 + placement の往復） ---

    #[test]
    fn drawer_hydration_round_trip_carries_state_and_placement() {
        let d = Drawer::new(OpenState::Open, DrawerPlacement::Bottom);
        let rendered = render(&render_for_hydration(&d));
        assert!(rendered.contains(r#"data-hydrate-state="open""#));
        assert!(rendered.contains(r#"data-hydrate-placement="bottom""#));

        let restored = Drawer::from_hydration_attrs(&d.hydration_attrs()).unwrap();
        assert_eq!(restored, d);
    }

    #[test]
    fn drawer_from_hydration_attrs_missing_state_does_not_panic() {
        let err = Drawer::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-state".to_string())
        );
    }

    #[test]
    fn drawer_from_hydration_attrs_missing_placement_does_not_panic() {
        let attrs = vec![("data-hydrate-state".to_string(), "open".to_string())];
        let err = Drawer::from_hydration_attrs(&attrs).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr(HYDRATE_PLACEMENT_ATTR.to_string())
        );
    }

    #[test]
    fn drawer_from_hydration_attrs_invalid_placement_is_rejected_not_rendered() {
        let attrs = vec![
            ("data-hydrate-state".to_string(), "open".to_string()),
            (
                HYDRATE_PLACEMENT_ATTR.to_string(),
                "<script>alert(1)</script>".to_string(),
            ),
        ];
        let err = Drawer::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn drawer_from_hydration_attrs_invalid_state_does_not_panic() {
        for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
            let attrs = vec![
                ("data-hydrate-state".to_string(), bogus.to_string()),
                (HYDRATE_PLACEMENT_ATTR.to_string(), "end".to_string()),
            ];
            let err = Drawer::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: id/labelledby/describedby/controls/attrs/children に
    // ペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn trigger_controls_payload_is_escaped_on_render() {
        let html = render(&trigger(
            OpenState::Closed,
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn content_id_labelledby_describedby_payload_is_escaped_on_render() {
        let html = render(&content(
            OpenState::Open,
            DrawerPlacement::End,
            true,
            ContentIds {
                id: Some(ATTR_BREAK_PAYLOAD),
                labelledby: Some(ATTR_BREAK_PAYLOAD),
                describedby: Some(ATTR_BREAK_PAYLOAD),
            },
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            OpenState::Closed,
            DrawerPlacement::End,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&close_trigger(
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
