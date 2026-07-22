//! 開閉系 headless コンポーネントが共有する状態機械（イシュー #524）。
//!
//! Dialog / Accordion / Tabs / Collapsible / Popover / Tooltip（Phase 2 の
//! #526〜#533）は「open/closed・selected」という同型の状態遷移を持つ。
//! これを各コンポーネントで個別実装すると、dispatch 契約（未知アクション
//! no-op）・`data-state` 整合・SSR/hydration 契約の実装が分散し、レビュー・
//! XSS 回帰の検証面が増える。本モジュールは
//! [`fandhe_frontend_interactive::Component`]/[`fandhe_frontend_interactive::Hydrate`]
//! にそのまま乗る形で、この 2 種の状態機械（[`Disclosure`]/[`SingleSelect`]）
//! を一度だけ実装し、Phase 2 の各コンポーネントがフィールドとして埋め込み
//! `decode_action`/`update` を委譲することで再利用する。
//!
//! signal/store は導入しない（`docs/policy/intentional-non-adoption.md`
//! §3.4 準拠。細粒度リアクティブは採用せず、単一状態機械＋明示的 dispatch
//! で統一する）。
//!
//! # 本モジュールの不変条件（`crates/interactive/src/lib.rs` の不変条件を継承）
//!
//! 1. `view()` の出力は [`fandhe_frontend_core::Node`] のみであり、
//!    `fandhe_frontend_core::render()` の既定エスケープを必ず経由する
//!    （`raw_html()`・HTML 文字列直接組み立ては不使用）。
//! 2. 未知アクション名の dispatch は no-op とし、状態を変更しない
//!    （[`fandhe_frontend_interactive::dispatch`] 経由で成立する安全側
//!    フォールバック）。
//! 3. `data-hydrate-*` 属性値はクライアント側で改ざんされうる入力として
//!    扱い、`from_hydration_attrs` は panic せず `Result` で失敗を返す。

use crate::data_attrs::data_state as data_state_attr;
use fandhe_frontend_core::{el, Node};
use fandhe_frontend_interactive::{codec, Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// `data-state` 属性値 "open"（#523 の [`crate::data_attrs::data_state`] が
/// 属性名 `"data-state"` 自体を一元管理するため、本モジュールは値のみを
/// 定数化する。属性名の重複定義はしない）。
pub const DATA_STATE_OPEN: &str = "open";
/// `data-state` 属性値 "closed"。[`DATA_STATE_OPEN`] 参照。
pub const DATA_STATE_CLOSED: &str = "closed";

/// 開閉状態。`data-state` 属性値（`"open"`/`"closed"`）と 1:1 対応する。
///
/// [`Disclosure`]/[`SingleSelect`] の両方から共有される値型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OpenState {
    /// 開いている。
    Open,
    /// 閉じている（SSR の状態なし初期描画に対応する既定値）。
    #[default]
    Closed,
}

impl OpenState {
    /// `data-state` 属性値へ変換する（`"open"`/`"closed"`）。
    #[must_use]
    pub fn as_data_state(self) -> &'static str {
        match self {
            OpenState::Open => DATA_STATE_OPEN,
            OpenState::Closed => DATA_STATE_CLOSED,
        }
    }

    /// `data-state`/`data-hydrate-state` 属性値から復元する。
    ///
    /// 未知の値（改ざん・タイポ）は `None` を返す（安全側、呼び出し元が
    /// [`HydrateError::InvalidValue`] 等へ変換する）。
    #[must_use]
    pub fn from_data_state(s: &str) -> Option<Self> {
        match s {
            DATA_STATE_OPEN => Some(OpenState::Open),
            DATA_STATE_CLOSED => Some(OpenState::Closed),
            _ => None,
        }
    }

    /// 開閉を反転した状態を返す。
    #[must_use]
    pub fn toggled(self) -> Self {
        match self {
            OpenState::Open => OpenState::Closed,
            OpenState::Closed => OpenState::Open,
        }
    }

    /// 開いているかどうか。
    #[must_use]
    pub fn is_open(self) -> bool {
        matches!(self, OpenState::Open)
    }
}

/// [`Disclosure`] に対する型付きアクション。
///
/// WASM 境界の文字列 dispatch（`name`/`payload`）とは
/// [`Disclosure::decode_action`] で接続する（[`Component::decode_action`] 実装）。
/// payload は使用しない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisclosureAction {
    /// 開く。
    Open,
    /// 閉じる。
    Close,
    /// 開閉を反転する。
    Toggle,
}

/// 単一の open/closed を持つ開閉状態機械。
///
/// Dialog / Collapsible / Popover / Tooltip 等、単一のパネルを開閉する
/// headless コンポーネントが埋め込んで使う共通状態機械。`Default` は
/// [`OpenState::Closed`]（SSR の状態なし初期描画に対応する既定値）。
///
/// [`Component`]/[`Hydrate`] の `view()`/`hydration_attrs()` は
/// 「`data-state` 整合・hydration ルート」という共通契約のみを担う最小
/// 正準ビューであり、Phase 2 の具象コンポーネントは本型をフィールドとして
/// 埋め込み、`decode_action`/`update` を委譲したうえで独自の anatomy
/// （トリガー・パネル・オーバーレイ等）を別途組み立てる想定である。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Disclosure {
    state: OpenState,
}

impl Disclosure {
    /// `data-hydrate-state` 属性名のフィールド部分
    /// （`docs/api/hydration-state-format.md` の `<field>` 命名規約に従う）。
    pub const FIELD_STATE: &'static str = "state";

    /// 指定した初期状態で開閉状態機械を生成する。
    #[must_use]
    pub fn new(initial: OpenState) -> Self {
        Self { state: initial }
    }

    /// 現在の開閉状態。
    #[must_use]
    pub fn state(&self) -> OpenState {
        self.state
    }

    /// 現在の `data-state` 属性値（`"open"`/`"closed"`）。
    #[must_use]
    pub fn data_state(&self) -> &'static str {
        self.state.as_data_state()
    }
}

impl Component for Disclosure {
    type Action = DisclosureAction;

    fn update(&mut self, action: DisclosureAction) {
        self.state = match action {
            DisclosureAction::Open => OpenState::Open,
            DisclosureAction::Close => OpenState::Closed,
            DisclosureAction::Toggle => self.state.toggled(),
        };
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー。Phase 2 の具象コンポーネントは自身の anatomy を別途組み立て、
    /// 本メソッドの出力をそのまま公開 API として使うことは想定しない。
    fn view(&self) -> Node {
        el("div", vec![data_state_attr(self.data_state())], Vec::new())
    }

    fn decode_action(name: &str, _payload: &str) -> Option<DisclosureAction> {
        match name {
            "open" => Some(DisclosureAction::Open),
            "close" => Some(DisclosureAction::Close),
            "toggle" => Some(DisclosureAction::Toggle),
            _ => None,
        }
    }
}

impl Hydrate for Disclosure {
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
        let state = OpenState::from_data_state(raw).ok_or_else(|| HydrateError::InvalidValue {
            attr: attr_name.clone(),
            reason: "expected \"open\" or \"closed\"".to_string(),
        })?;
        Ok(Self { state })
    }
}

/// [`SingleSelect`] に対する型付きアクション。
///
/// `payload`（WASM 境界の `data-payload` 属性値、改ざんされうるクライアント
/// 入力）は項目値としてそのまま保持し、HTML として解釈しない
/// （呼び出し元の [`fandhe_frontend_core::render`] が既定エスケープする）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SingleSelectAction {
    /// 指定した項目値を選択する（他の選択は解除される）。
    Select(String),
    /// 選択を解除する（全項目 closed）。
    Deselect,
    /// 指定した項目値を選択/解除の間でトグルする（選択中の同値なら解除、
    /// それ以外は選択に切り替え、他の選択は解除される）。
    Toggle(String),
}

/// 高々 1 個の項目値が「開いている」状態機械。
///
/// Tabs / Accordion（single モード）等、複数項目のうち高々 1 個だけを
/// 開いた状態にする headless コンポーネントが埋め込んで使う共通状態機械。
/// `Default` は未選択（全項目 closed。SSR の状態なし初期描画に対応する
/// 既定値）。
///
/// 複数同時選択（Accordion multiple モード）は本型のスコープ外（イシュー
/// #524 では未実装。Phase 2 の #527 で別途 `MultiSelect` として判断する）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SingleSelect {
    selected: Option<String>,
}

impl SingleSelect {
    /// `data-hydrate-selected` 属性名のフィールド部分。
    pub const FIELD_SELECTED: &'static str = "selected";

    /// 現在選択中の項目値（未選択なら `None`）。
    #[must_use]
    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    /// 指定した項目値が選択中かどうか。
    #[must_use]
    pub fn is_selected(&self, value: &str) -> bool {
        self.selected.as_deref() == Some(value)
    }

    /// 項目 `value` ごとの `data-state` 値。選択中なら `"open"`、
    /// それ以外は `"closed"`。Phase 2 の各コンポーネントが項目ごとの
    /// anatomy（`data-state` 付与）を組む際に使う。
    #[must_use]
    pub fn item_data_state(&self, value: &str) -> &'static str {
        if self.is_selected(value) {
            DATA_STATE_OPEN
        } else {
            DATA_STATE_CLOSED
        }
    }

    /// ルート全体の `data-state` 値。いずれかの項目が選択中なら `"open"`、
    /// 未選択なら `"closed"`。
    fn root_data_state(&self) -> &'static str {
        if self.selected.is_some() {
            DATA_STATE_OPEN
        } else {
            DATA_STATE_CLOSED
        }
    }
}

impl Component for SingleSelect {
    type Action = SingleSelectAction;

    fn update(&mut self, action: SingleSelectAction) {
        match action {
            SingleSelectAction::Select(value) => self.selected = Some(value),
            SingleSelectAction::Deselect => self.selected = None,
            SingleSelectAction::Toggle(value) => {
                if self.is_selected(&value) {
                    self.selected = None;
                } else {
                    self.selected = Some(value);
                }
            }
        }
    }

    /// 共通契約（ルート `data-state` 整合・hydration ルート）のみを表す
    /// 最小正準ビュー。項目ごとの anatomy 構築（[`Self::item_data_state`]
    /// を使う）は Phase 2 の具象コンポーネントの責務。
    fn view(&self) -> Node {
        el(
            "div",
            vec![data_state_attr(self.root_data_state())],
            Vec::new(),
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<SingleSelectAction> {
        match name {
            "select" => Some(SingleSelectAction::Select(payload.to_string())),
            "deselect" => Some(SingleSelectAction::Deselect),
            "toggle" => Some(SingleSelectAction::Toggle(payload.to_string())),
            _ => None,
        }
    }
}

impl Hydrate for SingleSelect {
    /// [`codec::encode_list`] で選択値を運ぶ（0 件 = 未選択、1 件 = 選択値）。
    /// リスト方式を流用することで、空文字列値の選択と未選択を区別でき、
    /// 区切り文字・エスケープ文字を含む任意の項目値でもラウンドトリップが
    /// 成立する（codec の既存保証をそのまま利用し、再実装しない）。
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let items: Vec<String> = self.selected.iter().cloned().collect();
        vec![(
            format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_SELECTED),
            codec::encode_list(&items),
        )]
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let attr_name = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_SELECTED);
        let raw = attrs
            .iter()
            .find(|(k, _)| *k == attr_name)
            .map(|(_, v)| v.as_str())
            .ok_or_else(|| HydrateError::MissingAttr(attr_name.clone()))?;
        let items = codec::decode_list(raw);
        match items.len() {
            0 => Ok(Self { selected: None }),
            1 => Ok(Self {
                selected: items.into_iter().next(),
            }),
            // 2 件以上のリストは本型の不変条件（高々 1 個選択）に反する
            // 改ざん入力。panic せず InvalidValue を返す（不変条件 3）。
            _ => Err(HydrateError::InvalidValue {
                attr: attr_name.clone(),
                reason: "expected at most one selected item".to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- OpenState ---

    #[test]
    fn open_state_round_trips_through_data_state_string() {
        for state in [OpenState::Open, OpenState::Closed] {
            assert_eq!(
                OpenState::from_data_state(state.as_data_state()),
                Some(state)
            );
        }
    }

    #[test]
    fn open_state_from_data_state_rejects_unknown_value() {
        assert_eq!(OpenState::from_data_state("OPEN"), None);
        assert_eq!(OpenState::from_data_state(""), None);
        assert_eq!(OpenState::from_data_state("<script>"), None);
    }

    #[test]
    fn open_state_toggled_flips_between_open_and_closed() {
        assert_eq!(OpenState::Open.toggled(), OpenState::Closed);
        assert_eq!(OpenState::Closed.toggled(), OpenState::Open);
    }

    // --- Disclosure: dispatch 経由の遷移 ---

    #[test]
    fn disclosure_default_is_closed() {
        assert_eq!(Disclosure::default().state(), OpenState::Closed);
    }

    #[test]
    fn disclosure_dispatch_open_close_toggle() {
        let mut d = Disclosure::default();

        assert!(dispatch(&mut d, "open", ""));
        assert_eq!(d.state(), OpenState::Open);

        assert!(dispatch(&mut d, "close", ""));
        assert_eq!(d.state(), OpenState::Closed);

        assert!(dispatch(&mut d, "toggle", ""));
        assert_eq!(d.state(), OpenState::Open);
        assert!(dispatch(&mut d, "toggle", ""));
        assert_eq!(d.state(), OpenState::Closed);
    }

    #[test]
    fn disclosure_dispatch_ignores_unknown_action() {
        let mut d = Disclosure::new(OpenState::Open);
        assert!(!dispatch(&mut d, "no_such_action", "x"));
        assert_eq!(d.state(), OpenState::Open);
    }

    // --- Disclosure: data-state 整合 ---

    #[test]
    fn disclosure_view_data_state_matches_current_state() {
        let closed = Disclosure::new(OpenState::Closed);
        assert!(render(&closed.view()).contains(r#"data-state="closed""#));

        let open = Disclosure::new(OpenState::Open);
        assert!(render(&open.view()).contains(r#"data-state="open""#));
    }

    #[test]
    fn disclosure_view_data_state_matches_after_transition() {
        let mut d = Disclosure::default();
        dispatch(&mut d, "open", "");
        assert!(render(&d.view()).contains(r#"data-state="open""#));
    }

    // --- Disclosure: SSR 状態なし初期描画 ---

    #[test]
    fn disclosure_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Disclosure::default().view());
        assert!(rendered.contains(r#"data-state="closed""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- Disclosure: hydration 経路 ---

    #[test]
    fn disclosure_hydration_round_trip() {
        let d = Disclosure::new(OpenState::Open);
        let rendered = render(&render_for_hydration(&d));
        assert!(rendered.contains(r#"data-hydrate-state="open""#));

        let restored = Disclosure::from_hydration_attrs(&d.hydration_attrs()).unwrap();
        assert_eq!(restored, d);
    }

    // --- Disclosure: 改ざん耐性 ---

    #[test]
    fn disclosure_from_hydration_attrs_missing_attr() {
        let err = Disclosure::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-state".to_string())
        );
    }

    #[test]
    fn disclosure_from_hydration_attrs_invalid_value_does_not_panic() {
        for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
            let attrs = vec![("data-hydrate-state".to_string(), bogus.to_string())];
            let err = Disclosure::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- SingleSelect: dispatch 経由の遷移 ---

    #[test]
    fn single_select_default_is_unselected() {
        assert_eq!(SingleSelect::default().selected(), None);
    }

    #[test]
    fn single_select_dispatch_select_deselect_toggle() {
        let mut s = SingleSelect::default();

        assert!(dispatch(&mut s, "select", "tab-1"));
        assert_eq!(s.selected(), Some("tab-1"));

        assert!(dispatch(&mut s, "select", "tab-2"));
        assert_eq!(s.selected(), Some("tab-2"));

        assert!(dispatch(&mut s, "deselect", ""));
        assert_eq!(s.selected(), None);

        assert!(dispatch(&mut s, "toggle", "tab-1"));
        assert_eq!(s.selected(), Some("tab-1"));
        assert!(dispatch(&mut s, "toggle", "tab-1"));
        assert_eq!(s.selected(), None);
    }

    #[test]
    fn single_select_dispatch_ignores_unknown_action() {
        let mut s = SingleSelect::default();
        dispatch(&mut s, "select", "tab-1");
        assert!(!dispatch(&mut s, "no_such_action", "tab-2"));
        assert_eq!(s.selected(), Some("tab-1"));
    }

    // --- SingleSelect: data-state 整合 ---

    #[test]
    fn single_select_item_data_state_matches_selection() {
        let mut s = SingleSelect::default();
        assert_eq!(s.item_data_state("a"), DATA_STATE_CLOSED);
        assert_eq!(s.item_data_state("b"), DATA_STATE_CLOSED);

        dispatch(&mut s, "select", "a");
        assert_eq!(s.item_data_state("a"), DATA_STATE_OPEN);
        assert_eq!(s.item_data_state("b"), DATA_STATE_CLOSED);
    }

    #[test]
    fn single_select_root_view_data_state_reflects_selection() {
        let unselected = SingleSelect::default();
        assert!(render(&unselected.view()).contains(r#"data-state="closed""#));

        let mut selected = SingleSelect::default();
        dispatch(&mut selected, "select", "a");
        assert!(render(&selected.view()).contains(r#"data-state="open""#));
    }

    // --- SingleSelect: SSR 状態なし初期描画 ---

    #[test]
    fn single_select_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&SingleSelect::default().view());
        assert!(rendered.contains(r#"data-state="closed""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- SingleSelect: hydration 経路 ---

    #[test]
    fn single_select_hydration_round_trip_selected() {
        let mut s = SingleSelect::default();
        dispatch(&mut s, "select", "tab-1");
        let restored = SingleSelect::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }

    #[test]
    fn single_select_hydration_round_trip_unselected() {
        let s = SingleSelect::default();
        let restored = SingleSelect::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }

    #[test]
    fn single_select_hydration_round_trip_survives_separator_and_empty_string_values() {
        for value in ["", "with\u{1f}separator", "with\\backslash"] {
            let mut s = SingleSelect::default();
            dispatch(&mut s, "select", value);
            let restored = SingleSelect::from_hydration_attrs(&s.hydration_attrs()).unwrap();
            assert_eq!(restored, s);
            assert_eq!(restored.selected(), Some(value));
        }
    }

    // --- SingleSelect: 改ざん耐性 ---

    #[test]
    fn single_select_from_hydration_attrs_missing_attr() {
        let err = SingleSelect::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-selected".to_string())
        );
    }

    #[test]
    fn single_select_from_hydration_attrs_rejects_multiple_selected_without_panicking() {
        let bogus = codec::encode_list(&["a".to_string(), "b".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = SingleSelect::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- XSS 回帰: 選択値に攻撃者制御文字列が入っても既定エスケープが効く ---

    #[test]
    fn single_select_xss_payload_in_selected_value_is_escaped_on_render() {
        let mut s = SingleSelect::default();
        let payload = "\"><script>alert(1)</script>";
        assert!(dispatch(&mut s, "select", payload));

        let rendered = render(&render_for_hydration(&s));
        // 正の確認: data-hydrate-selected 属性が実際に出力へ載っていること
        // （素朴に不在アサーションのみだと、attr マージが壊れて属性ごと
        // 消えた場合にも「偽装文字列が含まれない」ため誤って合格しうる。
        // 属性値そのものにペイロードのエスケープ済み形跡が現れることまで
        // 確認し、エスケープが実際に効いたことを検証する）。
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(!rendered.contains(r#""><script"#));
    }

    #[test]
    fn disclosure_view_root_is_element_for_render_for_hydration() {
        // render_for_hydration はルートが Node::Element であることを前提に
        // 属性を合成する（`crates/interactive/src/lib.rs` 参照）。本型の
        // view() が常に Element を返すことを固定する回帰テスト。
        let node = Disclosure::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }
}
