//! Dialog（モーダルダイアログ）headless コンポーネント（イシュー #531、親 #530）。
//!
//! ark-ui の Dialog
//!（`.claude/skills/ark-ui/references/components/overlays/dialog.md`）を
//! 参考に、Root / Trigger / Backdrop / Positioner / Content / Title /
//! Description / CloseTrigger の 8 anatomy パーツと、Phase 1（#524）の
//! [`crate::state::Disclosure`] を埋め込んだ開閉状態機械 [`Dialog`] を提供する。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（純粋関数で完結）を直接呼んで組み立てる。
//! CSR/hydration は [`Dialog`]（[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"open"`/`"close"`/`"toggle"`）で状態遷移する。
//! `fandhe-frontend-pre-styled-ui`（#546〜）が本モジュールを呼んでスタイル済み
//! Dialog を組み立てる想定である。
//!
//! # スコープ外（out-of-scope-tracking 対応）
//!
//! アニメーション対応の `open`/`visible` 分離は JS ランタイム側の責務であり
//! 本モジュールのスコープ外（SSR/属性出力のみ）。ネイティブ `<dialog>` 要素は
//! core のタグ語彙（`crates/core/src/tags.rs`）に存在しないため採用しない
//! （採用検討はスコープ外として記録）。
//!
//! Escape キーでの閉鎖・外側クリックでの閉鎖・フォーカストラップ・閉鎖時の
//! trigger へのフォーカス復帰・click → dispatch 配線は本モジュールが属性を
//! 出力するのみで、実 DOM 配線は `fandhe-frontend-wasm-full`（
//! `overlay::close_on_escape_for`/`close_on_interact_outside_for`〔#585〕・
//! `focus_trap`〔#586〕・`headless` の part → action 対応表）が担う
//! （イシュー #1638 で確認・文書化）。オプトアウト/オプトインは [`content`]
//! の `attrs` 経由で以下の `data-*` を渡す（`"false"` リテラルのときのみ
//! 無効化し、それ以外は既定へフォールバックする fail-closed 規則）:
//!
//! - `data-close-on-escape="false"`: Escape キーでの閉鎖を無効化する。
//! - `data-close-on-interact-outside="false"`: 外側クリックでの閉鎖を
//!   無効化する（`role="alertdialog"` のときは既定で無効）。
//! - `data-autofocus`: フォーカストラップの初期フォーカス先を指定する。
//!
//! # 参考サイトとの意図的な差分（イシュー #1638 で参照突合）
//!
//! ark-ui（zag `dialog.connect.ts`）・Radix Primitives・chakra-ui と突合した
//! 結果、anatomy（8 パート）・`data-state` 語彙（`open`/`closed`）は一致して
//! おり、以下の差分は意図的に合わせない:
//!
//! - **DOM 上の `root` パート**: zag の `Dialog.Root` は context のみで DOM を
//!   持たないが、本リポジトリの全部品が `data-state` 付与先として root を
//!   DOM 要素に持つ規約のため維持する。
//! - **`positioner` の `data-state` + `hidden`**: zag は `pointer-events` の
//!   インラインスタイルで代替するが、headless-ui はスタイルを出力しないため
//!   JS なし SSR での閉状態表現として維持する（`crates/pre-styled-ui`
//!   の recipe が `positioner[hidden]` に依存する、PR #575 参照）。
//! - **trigger の `data-ownedby`/`data-value`/`data-current`**（zag の複数
//!   トリガー識別）: `aria-controls` による id 関連付けが同等の役割を担うため
//!   不採用。
//! - **content の `data-nested`/`data-has-nested`**（ark-ui のネストダイアログ
//!   実行時計測）: `docs/policy/intentional-non-adoption.md` §3.25 規則 2
//!   （レイアウト計測・実行時関心は headless へ持ち込まない）により不採用。
//!   `fandhe-frontend-wasm-full` の overlay スタックが実行時に担う。
//! - **Radix `Portal`**: DOM 配置の関心（§3.25 規則 2）のため不採用。
//! - **Radix AlertDialog の `Cancel`/`Action` パート**: `DialogRole::Alertdialog`
//!   と [`close_trigger`] と素の `button` で構成でき、ark-ui にも該当パートは
//!   無いため不採用（Themes 側 alert-dialog 設計イシュー #1675 へ申し送り）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`/`hidden`/`id`/`tabindex`）はすべて
//!   `&'static str` リテラルで固定しており、動的値が属性名スロットへ混入する
//!   経路はない（[`mod@crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`] の
//!   既存不変条件をそのまま継承する）。
//! - 動的値（`id`/`labelledby`/`describedby`/`controls`/呼び出し側 `attrs`/
//!   `children` テキスト）は [`fandhe_frontend_core::render`] の既定エスケープを
//!   必ず経由する。`raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-state` 値語彙（`"open"`/`"closed"`）は [`crate::state::OpenState`]
//!   に一元化し、本モジュールで独自の値を作らない。`role` 値語彙も
//!   [`DialogRole`] に一元化し、任意文字列は受け付けない。
//! - hydration 属性（`data-hydrate-state`）はクライアント側で改ざんされうる
//!   入力として扱う。[`Dialog`] の [`fandhe_frontend_interactive::Hydrate`]
//!   実装は [`crate::state::Disclosure`] へ全委譲することで、panic せず
//!   `HydrateError` を返す既存保証をそのまま継承する。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{
    aria_controls, aria_describedby, aria_expanded, aria_haspopup, aria_hidden, aria_labelledby,
    aria_modal, role, AriaPopup,
};
use crate::data_attrs::data_state;
use crate::state::{Disclosure, DisclosureAction, OpenState};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// Dialog の anatomy（`data-scope="dialog"`）。
const ANATOMY: Anatomy = anatomy("dialog");

/// [`content`] の `role` 属性値。通常のダイアログか、確認・警告用の
/// alertdialog かを選ぶ（WAI-ARIA の固定語彙のみを受け付け、任意文字列は
/// 受け付けない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogRole {
    /// 通常のダイアログ（`role="dialog"`）。
    Dialog,
    /// 確認・警告用ダイアログ（`role="alertdialog"`）。
    Alertdialog,
}

impl DialogRole {
    /// `role` 属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dialog => "dialog",
            Self::Alertdialog => "alertdialog",
        }
    }
}

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
/// 付与する（A05 セキュリティ設定ミス対策）。`aria-haspopup="dialog"` を
/// 常に付与し、`controls` が `Some` のとき `aria-controls` で [`content`]
/// と関連付ける。
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

/// Backdrop パーツ（`div`）。ダイアログ背面を覆う装飾層。
///
/// closed のとき `hidden` 存在属性を付与する。装飾層のため常に
/// `aria-hidden="true"` を付与する（スクリーンリーダーの読み上げ対象外）。
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

/// Positioner パーツ（`div`）。[`content`] の配置用ラッパー。
///
/// closed のとき `hidden` 存在属性を付与する。
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

/// [`content`] が受け取る id 関連付け 3 種（`id`/`aria-labelledby`/
/// `aria-describedby`）をまとめた引数グループ。
///
/// 個別の `Option<&str>` 3 引数を素朴に並べると `clippy::too_many_arguments`
/// に抵触する（`content` は `state`/`role_kind`/`modal`/`attrs`/`children`
/// と合わせて 8 引数になる）ため、[`title`]/[`description`] との対応関係が
/// 強い 3 つの id をひとまとまりの構造体として扱う。
#[derive(Debug, Clone, Copy, Default)]
pub struct ContentIds<'a> {
    /// `content` 自身の `id`（[`trigger`] の `controls` と対）。
    pub id: Option<&'a str>,
    /// `aria-labelledby`（[`title`] の `id` と対）。
    pub labelledby: Option<&'a str>,
    /// `aria-describedby`（[`description`] の `id` と対）。
    pub describedby: Option<&'a str>,
}

/// Content パーツ（`div`）。ダイアログ本体。
///
/// `role`（[`DialogRole`]）・`aria-modal`・closed 時の `hidden` を付与する。
/// `ids`（[`ContentIds`]）の各フィールドが `Some` のときのみ対応する属性を
/// 出力し、[`title`]/[`description`] の `id` と対で `aria-labelledby`/
/// `aria-describedby` 関連付けを成立させる。
///
/// `tabindex="-1"` を固定で付与する（zag `dialog.connect.ts` の
/// `getContentProps` と同じく、プログラム的フォーカスのみを許可する
/// WAI-ARIA dialog パターンの前提）。`fandhe-frontend-wasm-full` の
/// `focus_trap::focus_content_itself` はハイドレーション後に tabbable な
/// 子孫が無い場合の代替フォーカス先として同属性を動的にも付与しており、
/// 本関数が SSR 時点から固定付与することで SSR 出力とハイドレーション後の
/// 出力が一致する（イシュー #1638）。
#[must_use]
pub fn content<'a>(
    state: OpenState,
    role_kind: DialogRole,
    modal: bool,
    ids: ContentIds<'a>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role(role_kind.as_str()),
        aria_modal(modal),
        data_state(state.as_data_state()),
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
    merged.extend(attrs);
    ANATOMY.part("content", "div", merged, children)
}

/// Title パーツ（`h2`）。`id` が `Some` のとき [`content`] の `labelledby`
/// と対にする。
#[must_use]
pub fn title<'a>(id: Option<&'a str>, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("title", "h2", merged, children)
}

/// Description パーツ（`p`）。`id` が `Some` のとき [`content`] の
/// `describedby` と対にする。
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

/// CloseTrigger パーツ（`button`）。ラベル（`aria-label`/children）は
/// 呼び出し側が `attrs`/`children` で付与する。
///
/// [`trigger`] と同じく `type="button"` を固定で付与する。
#[must_use]
pub fn close_trigger<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    merged.extend(attrs);
    ANATOMY.part("close-trigger", "button", merged, children)
}

/// [`Disclosure`]（#524）を埋め込んだ Dialog の状態機械。
///
/// `data-state` と実際の開閉状態の整合を型レベルで保証する入口として、
/// 各パーツ関数（[`root`]/[`trigger`]/[`backdrop`]/[`positioner`]/[`content`]）
/// へ `self.state()` を注入する利便メソッドを提供する。SSR での自由関数
/// 直接利用（本型を経由しない構成）も引き続き可能。`Default` は
/// [`OpenState::Closed`]（SSR の状態なし初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Dialog {
    disclosure: Disclosure,
}

impl Dialog {
    /// 指定した初期状態で Dialog を生成する。
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

    /// [`positioner`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn positioner<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        positioner(self.state(), attrs, children)
    }

    /// [`content`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn content<'a>(
        &self,
        role_kind: DialogRole,
        modal: bool,
        ids: ContentIds<'a>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        content(self.state(), role_kind, modal, ids, attrs, children)
    }
}

impl Component for Dialog {
    type Action = DisclosureAction;

    fn update(&mut self, action: DisclosureAction) {
        self.disclosure.update(action);
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > trigger + positioner(content)、children 空・id なし）。
    /// [`Disclosure::view`] と同じ位置付けであり、公開 UI としての利用は
    /// 想定しない（実際の UI 構築は §パーツ関数群を呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        let state = self.state();
        self.root(
            Vec::new(),
            vec![
                trigger(state, None, Vec::new(), Vec::new()),
                positioner(
                    state,
                    Vec::new(),
                    vec![content(
                        state,
                        DialogRole::Dialog,
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
        Disclosure::decode_action(name, payload)
    }
}

impl Hydrate for Dialog {
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

    // --- 各パーツの data-scope/data-part/data-state 出力 ---

    #[test]
    fn root_outputs_scope_part_and_state() {
        let html = render(&root(OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="dialog""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="closed""#));
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
        let html = render(&trigger(
            OpenState::Closed,
            Some("dialog-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-controls="dialog-1""#));
    }

    #[test]
    fn trigger_controls_none_omits_aria_controls() {
        let html = render(&trigger(OpenState::Closed, None, vec![], vec![]));
        assert!(!html.contains("aria-controls"));
    }

    #[test]
    fn backdrop_open_has_no_hidden_and_always_aria_hidden() {
        let open = render(&backdrop(OpenState::Open, vec![], vec![]));
        assert!(!open.contains("hidden=\"\""));
        assert!(open.contains(r#"aria-hidden="true""#));
        assert!(open.contains(r#"data-state="open""#));

        let closed = render(&backdrop(OpenState::Closed, vec![], vec![]));
        assert!(closed.contains(r#"hidden="""#));
        assert!(closed.contains(r#"aria-hidden="true""#));
    }

    #[test]
    fn positioner_closed_has_hidden_open_does_not() {
        let closed = render(&positioner(OpenState::Closed, vec![], vec![]));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&positioner(OpenState::Open, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn content_role_dialog_and_alertdialog() {
        let dialog_html = render(&content(
            OpenState::Open,
            DialogRole::Dialog,
            true,
            ContentIds::default(),
            vec![],
            vec![],
        ));
        assert!(dialog_html.contains(r#"role="dialog""#));

        let alert_html = render(&content(
            OpenState::Open,
            DialogRole::Alertdialog,
            true,
            ContentIds::default(),
            vec![],
            vec![],
        ));
        assert!(alert_html.contains(r#"role="alertdialog""#));
    }

    #[test]
    fn content_aria_modal_reflects_argument() {
        let modal = render(&content(
            OpenState::Open,
            DialogRole::Dialog,
            true,
            ContentIds::default(),
            vec![],
            vec![],
        ));
        assert!(modal.contains(r#"aria-modal="true""#));

        let non_modal = render(&content(
            OpenState::Open,
            DialogRole::Dialog,
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
            DialogRole::Dialog,
            true,
            ContentIds::default(),
            vec![],
            vec![],
        ));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&content(
            OpenState::Open,
            DialogRole::Dialog,
            true,
            ContentIds::default(),
            vec![],
            vec![],
        ));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn content_has_tabindex_minus_one() {
        // zag `dialog.connect.ts` の `getContentProps` と同じく、content は
        // 開閉に関わらず `tabindex="-1"` を固定で持つ（イシュー #1638）。
        let closed = render(&content(
            OpenState::Closed,
            DialogRole::Dialog,
            true,
            ContentIds::default(),
            vec![],
            vec![],
        ));
        assert!(closed.contains(r#"tabindex="-1""#));

        let open = render(&content(
            OpenState::Open,
            DialogRole::Dialog,
            true,
            ContentIds::default(),
            vec![],
            vec![],
        ));
        assert!(open.contains(r#"tabindex="-1""#));
    }

    #[test]
    fn content_id_labelledby_describedby_none_omit_attrs() {
        let html = render(&content(
            OpenState::Open,
            DialogRole::Dialog,
            true,
            ContentIds::default(),
            vec![],
            vec![],
        ));
        assert!(!html.contains(" id="));
        assert!(!html.contains("aria-labelledby"));
        assert!(!html.contains("aria-describedby"));
    }

    #[test]
    fn content_id_labelledby_describedby_some_output_attrs() {
        let html = render(&content(
            OpenState::Open,
            DialogRole::Dialog,
            true,
            ContentIds {
                id: Some("d1"),
                labelledby: Some("d1-title"),
                describedby: Some("d1-desc"),
            },
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"id="d1""#));
        assert!(html.contains(r#"aria-labelledby="d1-title""#));
        assert!(html.contains(r#"aria-describedby="d1-desc""#));
    }

    #[test]
    fn title_and_description_id_some_output_id() {
        let title_html = render(&title(Some("d1-title"), vec![], vec![text("Confirm")]));
        assert!(title_html.contains(r#"<h2"#));
        assert!(title_html.contains(r#"id="d1-title""#));
        assert!(title_html.contains("Confirm"));

        let desc_html = render(&description(
            Some("d1-desc"),
            vec![],
            vec![text("Are you sure?")],
        ));
        assert!(desc_html.contains(r#"<p"#));
        assert!(desc_html.contains(r#"id="d1-desc""#));
    }

    #[test]
    fn title_and_description_id_none_omit_id() {
        let title_html = render(&title(None, vec![], vec![]));
        assert!(!title_html.contains(" id="));

        let desc_html = render(&description(None, vec![], vec![]));
        assert!(!desc_html.contains(" id="));
    }

    #[test]
    fn close_trigger_has_type_button() {
        let html = render(&close_trigger(vec![], vec![text("Close")]));
        assert!(html.contains(r#"<button"#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"data-part="close-trigger""#));
        assert!(html.contains("Close"));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            OpenState::Closed,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="dialog""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- trigger + content の aria-controls/id、title/description の
    // labelledby/describedby 対応 ---

    #[test]
    fn trigger_controls_and_content_id_correspond() {
        let trigger_html = render(&trigger(OpenState::Open, Some("d1"), vec![], vec![]));
        let content_html = render(&content(
            OpenState::Open,
            DialogRole::Dialog,
            true,
            ContentIds {
                id: Some("d1"),
                ..Default::default()
            },
            vec![],
            vec![],
        ));
        assert!(trigger_html.contains(r#"aria-controls="d1""#));
        assert!(content_html.contains(r#"id="d1""#));
    }

    #[test]
    fn title_id_and_content_labelledby_correspond() {
        let title_html = render(&title(Some("d1-title"), vec![], vec![]));
        let content_html = render(&content(
            OpenState::Open,
            DialogRole::Dialog,
            true,
            ContentIds {
                labelledby: Some("d1-title"),
                ..Default::default()
            },
            vec![],
            vec![],
        ));
        assert!(title_html.contains(r#"id="d1-title""#));
        assert!(content_html.contains(r#"aria-labelledby="d1-title""#));
    }

    // --- Dialog: dispatch 統合 ---

    #[test]
    fn dialog_default_is_closed() {
        assert_eq!(Dialog::default().state(), OpenState::Closed);
    }

    #[test]
    fn dialog_dispatch_toggle_changes_data_state() {
        let mut d = Dialog::default();
        assert!(render(&d.root(vec![], vec![])).contains(r#"data-state="closed""#));

        assert!(dispatch(&mut d, "toggle", ""));
        assert!(render(&d.root(vec![], vec![])).contains(r#"data-state="open""#));
        assert!(render(&d.backdrop(vec![], vec![])).contains(r#"data-state="open""#));
        assert!(!render(&d.backdrop(vec![], vec![])).contains("hidden=\"\""));
    }

    #[test]
    fn dialog_dispatch_open_and_close() {
        let mut d = Dialog::default();
        assert!(dispatch(&mut d, "open", ""));
        assert_eq!(d.state(), OpenState::Open);
        assert!(dispatch(&mut d, "close", ""));
        assert_eq!(d.state(), OpenState::Closed);
    }

    #[test]
    fn dialog_dispatch_ignores_unknown_action() {
        let mut d = Dialog::new(OpenState::Open);
        assert!(!dispatch(&mut d, "no_such_action", "x"));
        assert_eq!(d.state(), OpenState::Open);
    }

    // --- Dialog: SSR 状態なし初期描画 ---

    #[test]
    fn dialog_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Dialog::default().view());
        assert!(rendered.contains(r#"data-state="closed""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    #[test]
    fn dialog_view_root_is_element_for_render_for_hydration() {
        let node = Dialog::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }

    // --- Dialog: hydration 経路 ---

    #[test]
    fn dialog_hydration_round_trip() {
        let d = Dialog::new(OpenState::Open);
        let rendered = render(&render_for_hydration(&d));
        assert!(rendered.contains(r#"data-hydrate-state="open""#));

        let restored = Dialog::from_hydration_attrs(&d.hydration_attrs()).unwrap();
        assert_eq!(restored, d);
    }

    #[test]
    fn dialog_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Dialog::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-state".to_string())
        );
    }

    #[test]
    fn dialog_from_hydration_attrs_invalid_value_does_not_panic() {
        for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
            let attrs = vec![("data-hydrate-state".to_string(), bogus.to_string())];
            let err = Dialog::from_hydration_attrs(&attrs).unwrap_err();
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
            DialogRole::Dialog,
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
    fn title_and_description_id_payload_is_escaped_on_render() {
        let title_html = render(&title(Some(ATTR_BREAK_PAYLOAD), vec![], vec![]));
        assert!(!title_html.contains("onmouseover=\"alert(1)"));

        let desc_html = render(&description(Some(ATTR_BREAK_PAYLOAD), vec![], vec![]));
        assert!(!desc_html.contains("onmouseover=\"alert(1)"));
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
    fn children_text_is_escaped_on_render() {
        let html = render(&close_trigger(
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn dialog_xss_payload_in_hydration_state_is_rejected_not_rendered() {
        // data-hydrate-state はサーバーが state.as_data_state() から生成する
        // 固定語彙のみを出力するため攻撃者が任意値を注入する経路はないが、
        // クライアント改ざん入力の復元経路（from_hydration_attrs）が
        // 未知値を拒否することを Dialog 経由でも固定する。
        let attrs = vec![(
            "data-hydrate-state".to_string(),
            "<script>alert(1)</script>".to_string(),
        )];
        let err = Dialog::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}
