//! HoverCard（リンク先プレビュー等、hover / focus で開閉するオーバーレイ）
//! headless コンポーネント（イシュー #759、親トラッキング #726 Phase 5 #757）。
//!
//! ark-ui の HoverCard
//! （`.claude/skills/ark-ui/references/components/overlays/hover-card.md`）を
//! 参考に、Root / Trigger / Positioner / Content / Arrow / ArrowTip の
//! 6 anatomy パーツと、[`crate::state::Disclosure`] を埋め込んだ開閉状態機械
//! [`HoverCard`] を提供する。[`mod@tooltip`]（#533）と最も近い構造だが、
//! trigger がリンク先プレビュー用途の `a` 要素である点が異なる。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`trigger`]/[`positioner`]/
//! [`content`]/[`arrow`]/[`arrow_tip`]、純粋関数で完結）を直接呼んで
//! 組み立てる。CSR/hydration は [`HoverCard`]
//! （[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"open"`/`"close"`/`"toggle"`）で状態遷移する。`fandhe-frontend-pre-styled-ui`
//! が本モジュールを呼んでスタイル済み HoverCard を組み立てる想定である。
//!
//! # WAI-ARIA と `aria-expanded`/`aria-controls`/`aria-haspopup` を付与しない理由
//!
//! WAI-ARIA APG に hover card 専用パターンは存在しない。trigger は
//! リンク先プレビューを目的とした通常の `a` 要素であり、それ自体が
//! 展開可能なウィジェットとして振る舞うわけではないため
//! （[`mod@tooltip`] の `aria-describedby` と同じく、Disclosure 系
//! （[`crate::collapsible`] 等）が使う `aria-expanded`/`aria-controls` は
//! 使用しない）、`content` にも固定 role を付与しない。
//!
//! # 遅延設定値（openDelay/closeDelay）
//!
//! [`HoverCardDelays`] は ark-ui 既定（`open_ms: 600`/`close_ms: 300`）を
//! 保持する決定的な SSR 設定値であり、[`root`] が `data-open-delay`/
//! `data-close-delay`（10 進の ms 値のみ）として出力する。実際の hover /
//! focus タイマー駆動（`fandhe-frontend-wasm-full` が DOM からこの 2 属性を
//! 読んで発火する配線）は本イシューのスコープ外（下記 §スコープ外参照）。
//! `data-open-delay`/`data-close-delay` の読み取り側は fail-closed パース
//! （不正・欠落値は既定へフォールバックする等）を実装側で徹底する契約とする。
//!
//! `delays` はクライアント可変状態ではなく SSR 静的設定であるため、
//! [`HoverCard`] の hydration 属性（`data-hydrate-state`）には含めない。
//! [`HoverCard::from_hydration_attrs`] で復元した後の `delays` は常に
//! [`HoverCardDelays::default`] になる（正は DOM の `data-open-delay`/
//! `data-close-delay` 属性であり、hydration 経由では復元しない）。
//!
//! # positioning（#590）の再利用
//!
//! フローティング位置計算（Floating UI 相当の placement / CSS 変数出力）は
//! [`crate::positioning`]（イシュー #590）をそのまま再利用する。[`positioner`]/
//! [`arrow`]/[`arrow_tip`] は [`mod@tooltip`]/[`mod@popover`] と同型の
//! 「`attrs` 経由で `style`/`data-side`/`data-align` を受け取る薄いラッパー」
//! であり、計算自体は `fandhe-frontend-wasm-full`（`position` モジュール）が
//! [`crate::positioning::compute_position`] を呼び出して行う（本モジュール
//! 自体は `web-sys` 非依存を維持する）。
//!
//! # スコープ外（out-of-scope）
//!
//! - hover / focus のタイマー駆動（`openDelay`/`closeDelay` の実時間発火）と
//!   `data-open-delay`/`data-close-delay` の DOM 読み取り配線
//! - `interactive`（content 内へのポインタ移動時の open 維持）
//! - `fandhe-frontend-wasm-full` の `PositionedKind::from_scope` への
//!   `"hover-card"` 追加（位置再計算の対象化）
//!
//! いずれも `fandhe-frontend-wasm-full` の後続イシューのスコープとする
//! （`.claude/rules/out-of-scope-tracking.md` 準拠）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`hidden`/`href`/`id`）はすべて `&'static str`
//!   リテラルで固定しており、動的値が属性名スロットへ混入する経路はない
//!   （[`crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`] の既存不変
//!   条件をそのまま継承する）。
//! - 動的値（`href`/`id`/呼び出し側 `attrs`/`children` テキスト/遅延値の
//!   文字列化）は [`fandhe_frontend_core::render`] の既定エスケープを必ず
//!   経由する。`raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `href` の `javascript:` 等の危険スキームは `fandhe-frontend-core` の
//!   URL スキーム検証が除去する（[`crate::breadcrumb::link`] と同じ保証、
//!   本モジュールのテストで固定）。
//! - `data-state` 値語彙（`"open"`/`"closed"`）は [`crate::state::OpenState`]
//!   に一元化し、本モジュールで独自の値を作らない。
//! - hydration 属性（`data-hydrate-state`）はクライアント側で改ざんされうる
//!   入力として扱う。[`HoverCard`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は
//!   [`crate::state::Disclosure`] へ全委譲することで、panic せず
//!   `HydrateError` を返す既存保証をそのまま継承する。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::aria_hidden;
use crate::data_attrs::data_state;
use crate::state::{Disclosure, DisclosureAction, OpenState};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// HoverCard の anatomy（`data-scope="hover-card"`）。
const ANATOMY: Anatomy = anatomy("hover-card");

/// `openDelay`/`closeDelay`（ark-ui 用語）を保持する決定的な SSR 設定値。
///
/// [`root`] が `data-open-delay`/`data-close-delay`（10 進の ms 値）として
/// 出力する。タイマー駆動そのものは本クレートのスコープ外（モジュール doc
/// §スコープ外参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoverCardDelays {
    /// hover / focus 開始から `content` が開くまでの遅延（ミリ秒）。
    pub open_ms: u32,
    /// ポインタ / フォーカス離脱から `content` が閉じるまでの遅延（ミリ秒）。
    pub close_ms: u32,
}

impl Default for HoverCardDelays {
    /// ark-ui 既定値（`openDelay: 600`/`closeDelay: 300`）。
    fn default() -> Self {
        Self {
            open_ms: 600,
            close_ms: 300,
        }
    }
}

/// Root パーツ（`div`）。開閉状態を `data-state` へ、[`HoverCardDelays`] を
/// `data-open-delay`/`data-close-delay`（10 進の ms 値のみ）へ反映する。
#[must_use]
pub fn root<'a>(
    state: OpenState,
    delays: HoverCardDelays,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let open_ms = delays.open_ms.to_string();
    let close_ms = delays.close_ms.to_string();
    let mut merged = vec![
        data_state(state.as_data_state()),
        ("data-open-delay", open_ms.as_str()),
        ("data-close-delay", close_ms.as_str()),
    ];
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Trigger パーツ（`a`）。
///
/// ark-ui の「リンク先プレビュー」ユースケース準拠で `a` 要素とする
/// （[`crate::breadcrumb::link`] と同型）。`href` が `Some` のときのみ
/// `href` 属性を出力し、`javascript:` 等の危険スキームは
/// `fandhe-frontend-core` の URL スキーム検証が除去する（モジュール doc
/// §セキュリティ不変条件参照）。WAI-ARIA に hover card 専用パターンは
/// 存在しないため `aria-expanded`/`aria-controls`/`aria-haspopup` は
/// 付与しない（モジュール doc 参照）。
#[must_use]
pub fn trigger<'a>(
    state: OpenState,
    href: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    if let Some(href) = href {
        merged.push(("href", href));
    }
    merged.extend(attrs);
    ANATOMY.part("trigger", "a", merged, children)
}

/// Positioner パーツ（`div`）。位置計算自体は本関数の責務ではなく
/// [`crate::positioning::compute_position`]（#590）が担う。本関数は
/// `data-scope`/`data-part` に加え、呼び出し側が `attrs` 経由で渡す
/// `style`（`--fandhe-*` CSS 変数）・`data-side`/`data-align` をそのまま
/// 透過させる薄いラッパーである（[`mod@tooltip::positioner`] と同型）。
///
/// `state` から `data-state` を出力する（`fandhe-frontend-wasm-full` の
/// `reposition_all` が使う `[data-part="positioner"][data-state="open"]`
/// セレクタにマッチさせるため、イシュー #622 の教訓を踏襲）。closed の
/// とき `hidden` 存在属性を付与し、arrow/arrow_tip が positioner 内に
/// ネストされる anatomy 構造上、closed 時にポインタ層を SSR/no-JS
/// マークアップへ表示させない（[`mod@popover::positioner`] と同じ判断）。
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
/// WAI-ARIA に hover card 専用パターンは存在しないため固定 role は付与
/// しない（モジュール doc 参照）。closed のとき `hidden` 存在属性を付与し、
/// JS なしの SSR でも閉状態を表現する。`id` が `Some` のとき呼び出し側が
/// 必要に応じて [`trigger`] との関連付けに使える（本モジュールは固定の
/// `aria-describedby` を配線しない。WAI-ARIA に hover card 専用パターンが
/// ないため）。
#[must_use]
pub fn content<'a>(
    state: OpenState,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("content", "div", merged, children)
}

/// Arrow パーツ（`div`）。装飾用のみであり、スクリーンリーダーへ読み上げ
/// させないため `aria-hidden="true"` を固定で付与する。
#[must_use]
pub fn arrow<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![aria_hidden(true)];
    merged.extend(attrs);
    ANATOMY.part("arrow", "div", merged, children)
}

/// ArrowTip パーツ（`div`）。[`arrow`] 同様に装飾用のみであり、
/// `aria-hidden="true"` を固定で付与する。
#[must_use]
pub fn arrow_tip<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![aria_hidden(true)];
    merged.extend(attrs);
    ANATOMY.part("arrow-tip", "div", merged, children)
}

/// [`Disclosure`]（#524）を埋め込んだ HoverCard の状態機械。
///
/// `data-state` と実際の開閉状態の整合を型レベルで保証する入口として、
/// 各パーツ関数（[`root`]/[`trigger`]/[`content`]/[`positioner`]）へ
/// `self.state()` を注入する利便メソッドを提供する（[`arrow`]/[`arrow_tip`]
/// は状態非依存のため利便メソッドを持たない）。SSR での自由関数直接利用
/// （本型を経由しない構成）も引き続き可能。`Default` は
/// [`OpenState::Closed`] + [`HoverCardDelays::default`]（SSR の状態なし
/// 初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HoverCard {
    disclosure: Disclosure,
    delays: HoverCardDelays,
}

impl HoverCard {
    /// 指定した初期状態・遅延設定で HoverCard を生成する。
    #[must_use]
    pub fn new(initial: OpenState, delays: HoverCardDelays) -> Self {
        Self {
            disclosure: Disclosure::new(initial),
            delays,
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

    /// 現在の遅延設定値。
    #[must_use]
    pub fn delays(&self) -> HoverCardDelays {
        self.delays
    }

    /// [`root`] へ現在の状態・遅延設定を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        root(self.state(), self.delays, attrs, children)
    }

    /// [`trigger`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn trigger<'a>(
        &self,
        href: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        trigger(self.state(), href, attrs, children)
    }

    /// [`content`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn content<'a>(
        &self,
        id: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        content(self.state(), id, attrs, children)
    }

    /// [`positioner`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn positioner<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        positioner(self.state(), attrs, children)
    }
}

impl Component for HoverCard {
    type Action = DisclosureAction;

    fn update(&mut self, action: DisclosureAction) {
        self.disclosure.update(action);
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > trigger + positioner > content、children 空・id
    /// なし・href なし）。[`Disclosure::view`] と同じ位置付けであり、公開
    /// UI としての利用は想定しない（実際の UI 構築は §パーツ関数群を
    /// 呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        let state = self.state();
        self.root(
            Vec::new(),
            vec![
                trigger(state, None, Vec::new(), Vec::new()),
                positioner(
                    state,
                    Vec::new(),
                    vec![content(state, None, Vec::new(), Vec::new())],
                ),
            ],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<DisclosureAction> {
        Disclosure::decode_action(name, payload)
    }
}

impl Hydrate for HoverCard {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.disclosure.hydration_attrs()
    }

    /// `delays` は hydration 属性に含まれない（モジュール doc §遅延設定値
    /// 参照）ため、復元後の `delays` は常に [`HoverCardDelays::default`]
    /// になる。正は DOM の `data-open-delay`/`data-close-delay` 属性であり、
    /// 復元は `disclosure` の既存 fail-closed 保証をそのまま継承する。
    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            disclosure: Disclosure::from_hydration_attrs(attrs)?,
            delays: HoverCardDelays::default(),
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
            same_width: false,
        };
        let resolved = compute_position(anchor, floating, viewport, &config, true);
        let style = css_vars_style(&resolved, anchor.width, config.same_width);
        let mut attrs: Vec<(&str, &str)> = vec![("style", &style)];
        attrs.extend(placement_attrs(resolved.placement));

        let html = render(&positioner(OpenState::Open, attrs, vec![]));
        assert!(html.contains("--fandhe-arrow-x:"));
        assert!(html.contains(r#"data-side="bottom""#));
        assert!(html.contains(r#"data-align="start""#));
    }

    // --- 各パーツの data-scope/data-part/data-state 出力 ---

    #[test]
    fn root_outputs_scope_part_and_state() {
        let html = render(&root(
            OpenState::Closed,
            HoverCardDelays::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="hover-card""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="closed""#));
    }

    #[test]
    fn root_outputs_default_delays_as_decimal_ms() {
        let html = render(&root(
            OpenState::Closed,
            HoverCardDelays::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-open-delay="600""#));
        assert!(html.contains(r#"data-close-delay="300""#));
    }

    #[test]
    fn root_outputs_custom_delays_as_decimal_ms() {
        let html = render(&root(
            OpenState::Closed,
            HoverCardDelays {
                open_ms: 1000,
                close_ms: 0,
            },
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-open-delay="1000""#));
        assert!(html.contains(r#"data-close-delay="0""#));
    }

    #[test]
    fn trigger_is_anchor_element_with_no_disclosure_aria() {
        let html = render(&trigger(OpenState::Closed, None, vec![], vec![]));
        assert!(html.contains("<a"));
        assert!(html.contains(r#"data-state="closed""#));
        // WAI-ARIA に hover card 専用パターンは存在しないため付与しない
        // （モジュール doc 参照）。
        assert!(!html.contains("aria-expanded"));
        assert!(!html.contains("aria-controls"));
        assert!(!html.contains("aria-haspopup"));
        assert!(!html.contains("href"));
    }

    #[test]
    fn trigger_href_some_outputs_href_attribute() {
        let html = render(&trigger(
            OpenState::Closed,
            Some("https://example.com"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"href="https://example.com""#));
    }

    #[test]
    fn trigger_href_none_omits_href_attribute() {
        let html = render(&trigger(OpenState::Closed, None, vec![], vec![]));
        assert!(!html.contains("href"));
    }

    #[test]
    fn trigger_javascript_scheme_href_is_dropped_by_core_url_validation() {
        let html = render(&trigger(
            OpenState::Closed,
            Some("javascript:alert(1)"),
            vec![],
            vec![],
        ));
        assert!(!html.contains("javascript:"));
        assert!(!html.contains("href="));
    }

    #[test]
    fn positioner_outputs_scope_part_and_state() {
        let html = render(&positioner(OpenState::Open, vec![], vec![]));
        assert!(html.contains(r#"data-scope="hover-card""#));
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
    fn content_has_no_fixed_role() {
        let html = render(&content(OpenState::Open, None, vec![], vec![]));
        assert!(!html.contains("role="));
    }

    #[test]
    fn content_closed_has_hidden_attr_open_does_not() {
        let closed = render(&content(OpenState::Closed, None, vec![], vec![]));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&content(OpenState::Open, None, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn content_id_some_outputs_id_attribute() {
        let html = render(&content(
            OpenState::Open,
            Some("hover-card-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"id="hover-card-1""#));
    }

    #[test]
    fn arrow_has_aria_hidden_true() {
        let html = render(&arrow(vec![], vec![]));
        assert!(html.contains(r#"data-part="arrow""#));
        assert!(html.contains(r#"aria-hidden="true""#));
    }

    #[test]
    fn arrow_tip_has_kebab_case_part_and_aria_hidden_true() {
        let html = render(&arrow_tip(vec![], vec![]));
        assert!(html.contains(r#"data-part="arrow-tip""#));
        assert!(html.contains(r#"aria-hidden="true""#));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            OpenState::Closed,
            HoverCardDelays::default(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="hover-card""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- HoverCard: dispatch 統合 ---

    #[test]
    fn hover_card_default_is_closed() {
        assert_eq!(HoverCard::default().state(), OpenState::Closed);
    }

    #[test]
    fn hover_card_default_has_default_delays() {
        assert_eq!(HoverCard::default().delays(), HoverCardDelays::default());
    }

    #[test]
    fn hover_card_dispatch_toggle_changes_data_state() {
        let mut hc = HoverCard::default();
        assert!(render(&hc.root(vec![], vec![])).contains(r#"data-state="closed""#));

        assert!(dispatch(&mut hc, "toggle", ""));
        assert!(render(&hc.root(vec![], vec![])).contains(r#"data-state="open""#));
        assert!(render(&hc.content(None, vec![], vec![])).contains(r#"data-state="open""#));
        assert!(!render(&hc.content(None, vec![], vec![])).contains("hidden"));
    }

    #[test]
    fn hover_card_dispatch_open_and_close() {
        let mut hc = HoverCard::default();
        assert!(dispatch(&mut hc, "open", ""));
        assert_eq!(hc.state(), OpenState::Open);
        assert!(dispatch(&mut hc, "close", ""));
        assert_eq!(hc.state(), OpenState::Closed);
    }

    #[test]
    fn hover_card_dispatch_ignores_unknown_action() {
        let mut hc = HoverCard::new(OpenState::Open, HoverCardDelays::default());
        assert!(!dispatch(&mut hc, "no_such_action", "x"));
        assert_eq!(hc.state(), OpenState::Open);
    }

    // --- HoverCard: SSR 状態なし初期描画 ---

    #[test]
    fn hover_card_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&HoverCard::default().view());
        assert!(rendered.contains(r#"data-state="closed""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- HoverCard: hydration 経路 ---

    #[test]
    fn hover_card_hydration_round_trip() {
        let hc = HoverCard::new(OpenState::Open, HoverCardDelays::default());
        let rendered = render(&render_for_hydration(&hc));
        assert!(rendered.contains(r#"data-hydrate-state="open""#));

        let restored = HoverCard::from_hydration_attrs(&hc.hydration_attrs()).unwrap();
        assert_eq!(restored, hc);
    }

    #[test]
    fn hover_card_hydration_restores_default_delays_regardless_of_original() {
        // delays は hydration 属性へ含めない設計であるため（モジュール doc
        // §遅延設定値参照）、非既定値で生成しても復元後は常に既定値へ戻る。
        let hc = HoverCard::new(
            OpenState::Open,
            HoverCardDelays {
                open_ms: 1000,
                close_ms: 0,
            },
        );
        let restored = HoverCard::from_hydration_attrs(&hc.hydration_attrs()).unwrap();
        assert_eq!(restored.delays(), HoverCardDelays::default());
    }

    #[test]
    fn hover_card_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = HoverCard::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-state".to_string())
        );
    }

    #[test]
    fn hover_card_from_hydration_attrs_invalid_value_does_not_panic() {
        for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
            let attrs = vec![("data-hydrate-state".to_string(), bogus.to_string())];
            let err = HoverCard::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: href/id/呼び出し側 attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn trigger_href_payload_is_escaped_on_render() {
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
    fn content_id_payload_is_escaped_on_render() {
        let html = render(&content(
            OpenState::Open,
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
            OpenState::Closed,
            HoverCardDelays::default(),
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&content(
            OpenState::Open,
            None,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn hover_card_xss_payload_in_hydration_state_is_rejected_not_rendered() {
        // data-hydrate-state はサーバーが state.as_data_state() から生成する
        // 固定語彙のみを出力するため攻撃者が任意値を注入する経路はないが、
        // クライアント改ざん入力の復元経路（from_hydration_attrs）が
        // 未知値を拒否することを HoverCard 経由でも固定する。
        let attrs = vec![(
            "data-hydrate-state".to_string(),
            "<script>alert(1)</script>".to_string(),
        )];
        let err = HoverCard::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}
