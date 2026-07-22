//! anchor positioning の計測値注入・再計算配線（イシュー #590、親 #588）。
//!
//! `fandhe-frontend-headless-ui` の [`fandhe_frontend_headless_ui::positioning`]
//! （`compute_position`/`css_vars_style`、純粋関数・`web-sys` 非依存）へ、実
//! DOM 計測値（anchor 矩形・floating/viewport 寸法）を注入するのが本モジュール
//! の責務である。`events.rs`/`overlay.rs` と同じ 2 層構成を踏襲する:
//!
//! - 純粋ロジック層（本モジュールの `PositionedKind` / DOM 属性の
//!   fail-closed パース / [`resolve_position`]）は native の `cargo test` で
//!   検証できる（`web-sys` 型を引数に取らない）。
//! - `#[cfg(target_arch = "wasm32")]` の [`wiring::PositionController`] が
//!   実 DOM 計測（`getBoundingClientRect`）・スクロール/リサイズ契機の
//!   離散的な再計算呼び出しを担う。
//!
//! # 他モジュール・他クレートとの契約
//!
//! - [`PositionedKind::from_scope`] は `data-scope` 属性値
//!   （`"popover"`/`"tooltip"`/`"menu"`/`"select"`）と 1 対 1 対応する
//!   （[`crate::overlay::OverlayKind::from_scope`] と同型の fail-closed
//!   パターン。未知の scope 値は `None` とし、呼び出し側は対象外として
//!   無視する）。
//! - `positioner` 要素の `data-side`/`data-align` 属性は
//!   [`fandhe_frontend_headless_ui::Placement`] の語彙で読み書きする。
//!   DOM 上の値は改ざんされうるクライアント入力として扱い、未知値は既定
//!   （`bottom`/`center`）へフォールバックする（fail-closed）。
//! - `data-side`/`data-align` は flip 適用後の**確定** side/align（CSS
//!   セレクタ用に毎回上書きされる出力）であり、「希望 placement」の入力
//!   としては使わない。希望 placement は独立の `data-requested-side`/
//!   `data-requested-align` 属性（[`wiring::reposition_one`] が初回のみ
//!   書き込み、以後は上書きしない永続化領域）で保持する
//!   （[`resolve_requested_placement`] 参照。イシュー #622 レビュー指摘:
//!   従来 `data-side`/`data-align` 自体を希望として読み戻していたため、
//!   flip 後の side が新しい希望として扱われ、スペースが戻っても元の希望へ
//!   戻せなかった）。
//! - 実際の `"close"` dispatch・状態機械の更新は行わない。本モジュールは
//!   `positioner`/`arrow` 要素へ `style`/`data-side`/`data-align` 属性を
//!   直接 `set_attribute` するのみであり（ADR 第 4.4 節の経路とは別に、
//!   wasm 層は DOM API で直接属性を書き込む。SSR/CSR いずれの初期表示も
//!   `fandhe_frontend_core::render` の既定エスケープ経由だが、本モジュールの
//!   再計算は初期表示後の DOM 直接更新であり HTML 文字列を組み立てない
//!   ため既定エスケープ経路の対象外である点に注意）、開閉 dispatch との
//!   統合呼び出しはイシュー #580 統合層の責務とする（`overlay.rs` と同じ
//!   責務分離）。
//!
//! # セキュリティ不変条件
//!
//! - [`resolve_position`] が返す `style` 文字列は
//!   [`fandhe_frontend_headless_ui::css_vars_style`] が組み立てる内部生成の
//!   数値書式のみであり、ユーザー入力を含まない。
//! - DOM から読む `data-scope`/`data-side`/`data-align` はいずれも
//!   fail-closed（未知値は既定へフォールバック、`panic!`/`unwrap()` 不使用）。
//! - `wiring::PositionController` の `Closure::forget` は scroll/resize の
//!   2 個のみに限定し、無制限なリスナーリーク（A04 相当の DoS）を構造的に
//!   避ける（`events.rs` の既存判断を踏襲）。

use fandhe_frontend_headless_ui::{
    compute_position, css_vars_style, Align, Placement, PositioningConfig, Rect, Side, Size,
};

/// 位置決め対象のコンポーネント種別。
///
/// `data-scope` 属性値と 1 対 1 対応する（[`crate::overlay::OverlayKind`] と
/// 同型だが、position モジュールは overlay と異なりオーバーレイでない
/// Select も対象に含むため独立した enum とする）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionedKind {
    /// `data-scope="popover"`。
    Popover,
    /// `data-scope="tooltip"`。
    Tooltip,
    /// `data-scope="menu"`。
    Menu,
    /// `data-scope="select"`。
    Select,
}

impl PositionedKind {
    /// `data-scope` 属性値からのパース。未知の scope は `None`
    /// （fail-closed。[`crate::overlay::OverlayKind::from_scope`] と同じ
    /// 契約）。
    #[must_use]
    pub fn from_scope(scope: &str) -> Option<Self> {
        match scope {
            "popover" => Some(Self::Popover),
            "tooltip" => Some(Self::Tooltip),
            "menu" => Some(Self::Menu),
            "select" => Some(Self::Select),
            _ => None,
        }
    }

    /// arrow 座標計算の対象か（ADR §4.2: Select のみ arrow を持たない）。
    #[must_use]
    pub const fn has_arrow(self) -> bool {
        !matches!(self, Self::Select)
    }

    /// sameWidth（`--fandhe-reference-width` を anchor 幅に固定する）の
    /// kind 別既定。Select はドロップダウン幅をトリガー幅に一致させる
    /// 用途が主目的のため既定 `true`。Menu もトリガー幅への追随が自然な
    /// ユースケースが多いため `true` とする。Popover/Tooltip は任意サイズの
    /// コンテンツを想定するため既定 `false`（呼び出し側が
    /// [`PositioningRequest::same_width`] で上書き可能）。
    #[must_use]
    pub const fn same_width_default(self) -> bool {
        matches!(self, Self::Menu | Self::Select)
    }
}

/// `data-side` 属性値の fail-closed パース。欠落・未知値は既定
/// （[`Side::Bottom`]）へフォールバックする（DOM 属性はクライアント側で
/// 改ざんされうる入力のため）。
#[must_use]
pub fn parse_side_attr(value: Option<&str>) -> Side {
    value.and_then(Side::from_str).unwrap_or(Side::Bottom)
}

/// `data-align` 属性値の fail-closed パース。欠落・未知値は既定
/// （[`Align::Center`]）へフォールバックする。
#[must_use]
pub fn parse_align_attr(value: Option<&str>) -> Align {
    value.and_then(Align::from_str).unwrap_or(Align::Center)
}

/// 「希望 placement」（flip 適用前の入力、[`resolve_position`] の
/// `requested` 引数）を解決する純粋関数（native `cargo test` で検証可能。
/// [`wiring::reposition_one`] が実 DOM 属性から抽出した値を渡す）。
///
/// `persisted_side`/`persisted_align` は `data-requested-side`/
/// `data-requested-align`（[`wiring`] が一度だけ書き込み、以後は flip
/// 結果で上書きしない永続化領域）の現在値。存在すればそれを希望
/// placement として最優先で採用する。存在しない場合（初回の再計算・SSR
/// マークアップに元々なかった場合）は `fallback_side`/`fallback_align`
/// （`data-side`/`data-align` の現在値、SSR が出力した初期値でまだ flip
/// されていない）を希望 placement の初期値として採用する。
///
/// イシュー #622 レビュー指摘（High）: 従来 `reposition_one` は
/// `data-side`/`data-align` を「希望 placement」として直接読み取り、flip
/// 後の結果を同じ属性へ書き戻していた。そのため 2 回目以降の再計算では
/// 直前の flip 後の side が新しい希望として扱われてしまい、スペースが
/// 戻っても元の希望へ戻せず、大きすぎる floating 要素は再計算のたびに
/// 左右反転を繰り返しうる不具合があった。本関数と `data-requested-*` の
/// 永続化により、「希望 placement」（不変）と「確定 side/align」
/// （`data-side`/`data-align`、CSS セレクタ用に毎回更新される出力）を
/// 分離する。
#[must_use]
pub fn resolve_requested_placement(
    persisted_side: Option<&str>,
    persisted_align: Option<&str>,
    fallback_side: Option<&str>,
    fallback_align: Option<&str>,
) -> Placement {
    Placement::new(
        parse_side_attr(persisted_side.or(fallback_side)),
        parse_align_attr(persisted_align.or(fallback_align)),
    )
}

/// 実 DOM 計測値（[`wiring`] が `getBoundingClientRect`/`window` から
/// 組み立てる、native テストではテストダブルから組み立てる）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measurement {
    /// anchor（トリガー等）矩形。
    pub anchor: Rect,
    /// floating（positioner）要素の寸法。
    pub floating: Size,
    /// viewport 寸法。
    pub viewport: Size,
}

/// [`resolve_position`] の結果。呼び出し側（[`wiring`]）はこの `style` を
/// positioner 要素の `style` 属性へ、`side`/`data_align` を
/// `data-side`/`data-align` 属性へ、それぞれ `set_attribute` で反映する。
#[derive(Debug, Clone, PartialEq)]
pub struct RepositionResult {
    /// `style` 属性値（`--fandhe-*` CSS 変数、内部生成の数値書式のみ）。
    pub style: String,
    /// 確定 placement の主軸方向。
    pub side: Side,
    /// 確定 placement の交差軸整列。
    pub align: Align,
}

/// 計測値・希望 placement・kind から位置計算を実行し、DOM へ反映すべき
/// 属性値一式を組み立てる（`fandhe_frontend_headless_ui::compute_position` +
/// `css_vars_style` の呼び出しをまとめた本クレート側のエントリポイント）。
///
/// flip/shift は常に有効（ADR が定める既定挙動、opt-out API は本イシューの
/// スコープ外）。offset は `0.0` 固定（呼び出し側がギャップを持たせたい
/// 場合は将来イシューで `PositioningConfig` をそのまま公開する拡張余地を
/// 残す）。
#[must_use]
pub fn resolve_position(
    kind: PositionedKind,
    measurement: Measurement,
    requested: Placement,
) -> RepositionResult {
    let config = PositioningConfig {
        placement: requested,
        offset: 0.0,
        flip: true,
        shift: true,
        same_width: kind.same_width_default(),
    };
    let resolved = compute_position(
        measurement.anchor,
        measurement.floating,
        measurement.viewport,
        &config,
        kind.has_arrow(),
    );
    let style = css_vars_style(&resolved, measurement.anchor.width, config.same_width);
    RepositionResult {
        style,
        side: resolved.placement.side(),
        align: resolved.placement.align(),
    }
}

// ---------------------------------------------------------------------
// 配線層: web-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`events.rs`/`overlay.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{resolve_position, resolve_requested_placement, Measurement, PositionedKind};
    use fandhe_frontend_headless_ui::{Rect, Size};
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, Window};

    /// `[data-part="positioner"][data-state="open"]` を document 全体から
    /// 走査し、開いている positioner のみ再計算する（閉じている
    /// positioner は非表示のため `getBoundingClientRect` が意味を持たず、
    /// 再計算する必要がない）。
    const OPEN_POSITIONER_SELECTOR: &str = "[data-part=\"positioner\"][data-state=\"open\"]";

    /// 希望 placement の永続化領域（[`super::resolve_requested_placement`]
    /// 参照）。`data-side`/`data-align`（flip 後の確定値で毎回上書きされる）
    /// とは独立した属性とし、flip 書き戻しの影響を受けない「希望」を保持する。
    const REQUESTED_SIDE_ATTR: &str = "data-requested-side";
    const REQUESTED_ALIGN_ATTR: &str = "data-requested-align";

    /// `element` の祖先方向へ anatomy の scope root（`data-part="root"`）を
    /// 探す。
    ///
    /// `anatomy::Anatomy::part` は `root` を含む**全ての**パーツへ
    /// `data-scope` を付与するため（`headless-ui` の `popover`/`tooltip`/
    /// `menu`/`select` いずれも `root`/`trigger`/`anchor`/`positioner`/
    /// `content` 等すべてが同じ `data-scope` 値を持つ）、単に
    /// `[data-scope]` で `closest` すると `positioner` 自身が自己マッチして
    /// しまい、真の scope root（`anchor`/`trigger` を子孫に持つ祖先）まで
    /// 辿り着けない（`closest` は呼び出し要素自身も候補に含む DOM 仕様の
    /// ため）。`data-part="root"` は 4 コンポーネントいずれも scope root
    /// にのみ付与される固有の part 名であるため、これを直接セレクタへ
    /// 指定することで自己マッチを避ける。
    fn find_scope_root(element: &Element) -> Option<Element> {
        element.closest("[data-part=\"root\"]").ok().flatten()
    }

    /// `selector` に一致する `scope_root` 配下の要素のうち、**`scope_root`
    /// 自身に属する**（ネストした子スコープの子孫ではない）最初の 1 件を
    /// 返す。
    ///
    /// `Element::query_selector` は子孫全体を対象にした単純な CSS セレクタ
    /// マッチのため、Menu のサブメニュー（親 scope root の `content` 配下に
    /// 子 `Menu` インスタンスの scope root がネストする構造）のように
    /// scope root が入れ子になる場合、親 scope root からの検索で子スコープ
    /// 内の同名パーツ（例: 子の `trigger-item`）まで拾ってしまう。各候補
    /// 要素について `closest("[data-part=\"root\"]")`（[`find_scope_root`]
    /// と同じ「最近傍の scope root」解決）が `scope_root` 自身と一致するかを
    /// 確認し、一致するものだけを採用することでこの誤検出を防ぐ。
    fn find_direct_scope_match(scope_root: &Element, selector: &str) -> Option<Element> {
        let list = scope_root.query_selector_all(selector).ok()?;
        for i in 0..list.length() {
            // `reposition_all`（`document.query_selector_all` を走査する
            // 呼び出し元）と同じ「取得できなかった要素はスキップして続行」
            // という fail-closed 方針に合わせる（`i < list.length()` のため
            // 通常 `None` にはならないが、想定外の `None` で探索全体を
            // 打ち切らないための防御）。
            let Some(node) = list.item(i) else {
                continue;
            };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            let belongs_to_scope_root = element
                .closest("[data-part=\"root\"]")
                .ok()
                .flatten()
                .is_some_and(|nearest_root| nearest_root.is_same_node(Some(scope_root.as_ref())));
            if belongs_to_scope_root {
                return Some(element);
            }
        }
        None
    }

    /// scope root 配下の anchor 要素を解決する。`[data-part="anchor"]` が
    /// あれば優先し、なければ `[data-part="trigger"]` を anchor として扱う
    /// （Popover の `anchor` パーツ、他コンポーネントの `trigger` パーツの
    /// いずれも実 DOM 上の参照要素として妥当という設計判断、ADR §4.1 の
    /// 「anchor（トリガー等）矩形」記述に対応）。Menu のコンテキストメニュー
    /// （`context-trigger`）・サブメニュー（`trigger-item`）は `trigger`/
    /// `anchor` パーツを持たず、これら固有の part 名がそれぞれの scope root
    /// 直下の参照要素であるため、フォールバック先へ追加する（イシュー #622
    /// レビュー指摘の回帰: 追加しないと `find_anchor` が `None` を返し
    /// `reposition_one` が no-op となって開いている positioner へ CSS 変数が
    /// 届かなくなる）。
    ///
    /// `context-trigger` を `trigger-item` より先に判定する
    /// （イシュー #622 Bugbot 指摘: サブメニューを含むコンテキストメニューで
    /// `query_selector` が入れ子の `trigger-item` を自身の `context-trigger`
    /// より先に拾ってしまい、開いている positioner が誤った要素を anchor に
    /// して座標計算されていた）。加えて全パーツの探索を
    /// [`find_direct_scope_match`] 経由にし、ネストした子スコープ配下の
    /// 同名パーツを拾わないようにする。
    fn find_anchor(scope_root: &Element) -> Option<Element> {
        find_direct_scope_match(scope_root, "[data-part=\"anchor\"]")
            .or_else(|| find_direct_scope_match(scope_root, "[data-part=\"trigger\"]"))
            .or_else(|| find_direct_scope_match(scope_root, "[data-part=\"context-trigger\"]"))
            .or_else(|| find_direct_scope_match(scope_root, "[data-part=\"trigger-item\"]"))
    }

    /// scope root 配下の arrow 要素（存在しないコンポーネント・マークアップ
    /// では `None`）。
    ///
    /// [`find_anchor`] と同じ理由（ネストした子スコープの子孫を誤って
    /// 拾わない）で [`find_direct_scope_match`] 経由にする。単純な
    /// `query_selector` のままだと、サブメニューを持つ Menu で自身に
    /// arrow が無くても入れ子のサブメニューの arrow を拾ってしまい、
    /// 外側 positioner の `style` へ誤った要素の座標が複製されうる。
    fn find_arrow(scope_root: &Element) -> Option<Element> {
        find_direct_scope_match(scope_root, "[data-part=\"arrow\"]")
    }

    /// `Element::get_bounding_client_rect` から [`Rect`] を組み立てる。
    fn measure_rect(element: &Element) -> Rect {
        let rect = element.get_bounding_client_rect();
        Rect {
            x: rect.x(),
            y: rect.y(),
            width: rect.width(),
            height: rect.height(),
        }
    }

    /// `window` の viewport 寸法。`window`/`inner_width`/`inner_height` の
    /// いずれかが取得できない場合は `0.0`（[`super::resolve_position`] が
    /// fail-closed で吸収する異常値）を返す。
    fn measure_viewport(window: &Window) -> Size {
        let width = window
            .inner_width()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let height = window
            .inner_height()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        Size { width, height }
    }

    /// 1 件の positioner を再計算し、DOM 属性へ反映する。
    ///
    /// anchor が見つからない・`data-scope` が未知の場合は no-op とする
    /// （fail-closed。マークアップが不完全でも panic しない）。
    /// `element.set_attribute(name, value)` の薄いガード付きラッパー
    /// （イシュー #401 の `fw gate` `url_validation_check` 契約に準拠、
    /// `.claude/rules/security.md`）。本モジュールが書き込む属性
    /// （`style`/`data-side`/`data-align`）はいずれも `&'static str`
    /// リテラルで固定された非 URL・非イベントハンドラ属性であり、`style`
    /// 値も内部生成の数値のみで実害はないが、`fandhe_frontend_core::url`
    /// のガード関数群（`is_event_handler_attr`/`is_url_attr`/
    /// `is_safe_url`/`is_safe_srcset`）を経由することで、将来 `name`/
    /// `value` が動的な入力から組み立てられるよう変更された場合の防御
    /// としても機能する（`keynav::set_dom_attribute` と同じガード方針）。
    fn set_dom_attribute(element: &Element, name: &str, value: &str) {
        if fandhe_frontend_core::is_event_handler_attr(name) {
            return;
        }
        if fandhe_frontend_core::is_url_attr(name) && !fandhe_frontend_core::is_safe_url(value) {
            return;
        }
        if name.eq_ignore_ascii_case("srcset") && !fandhe_frontend_core::is_safe_srcset(value) {
            return;
        }
        let _ = element.set_attribute(name, value);
    }

    fn reposition_one(positioner: &Element, window: &Window) {
        let Some(scope_root) = find_scope_root(positioner) else {
            return;
        };
        let Some(scope) = scope_root.get_attribute("data-scope") else {
            return;
        };
        let Some(kind) = PositionedKind::from_scope(&scope) else {
            return;
        };
        let Some(anchor_element) = find_anchor(&scope_root) else {
            return;
        };

        let anchor = measure_rect(&anchor_element);
        let floating_rect = positioner.get_bounding_client_rect();
        let floating = Size {
            width: floating_rect.width(),
            height: floating_rect.height(),
        };
        let viewport = measure_viewport(window);

        // 希望 placement は data-requested-side/data-requested-align
        // （永続化領域、flip 書き戻しの影響を受けない）を最優先で読み、
        // 未設定（初回）なら現在の data-side/data-align（SSR 初期値）へ
        // フォールバックする（[`super::resolve_requested_placement`] 参照）。
        let persisted_side = positioner.get_attribute(REQUESTED_SIDE_ATTR);
        let persisted_align = positioner.get_attribute(REQUESTED_ALIGN_ATTR);
        let requested = resolve_requested_placement(
            persisted_side.as_deref(),
            persisted_align.as_deref(),
            positioner.get_attribute("data-side").as_deref(),
            positioner.get_attribute("data-align").as_deref(),
        );

        // 初回（data-requested-* が未設定）は希望 placement をここで確定させ
        // 永続化する。以後の再計算は flip 後の data-side/data-align を
        // 希望として読み直さず、この永続化値のみを希望として扱い続ける。
        if persisted_side.is_none() {
            set_dom_attribute(positioner, REQUESTED_SIDE_ATTR, requested.side().as_str());
        }
        if persisted_align.is_none() {
            set_dom_attribute(positioner, REQUESTED_ALIGN_ATTR, requested.align().as_str());
        }

        let result = resolve_position(
            kind,
            Measurement {
                anchor,
                floating,
                viewport,
            },
            requested,
        );

        set_dom_attribute(positioner, "style", &result.style);
        set_dom_attribute(positioner, "data-side", result.side.as_str());
        set_dom_attribute(positioner, "data-align", result.align.as_str());

        if kind.has_arrow() {
            if let Some(arrow_element) = find_arrow(&scope_root) {
                // arrow の絶対座標は floating 相対のオフセットのため、
                // positioner が既に反映した `style` の `--fandhe-arrow-*`
                // をそのまま arrow 要素側にも複製する（arrow は positioner
                // の子として配置され、CSS 側で親の CSS 変数を継承する設計
                // だが、arrow 要素自身の `style` にも明示反映することで
                // pre-styled-ui 側のセレクタ設計を CSS 変数継承に限定しない
                // 柔軟性を残す）。
                set_dom_attribute(&arrow_element, "style", &result.style);
            }
        }
    }

    /// `document` 内の開いている positioner を全て走査し再計算する
    /// （公開 API。開閉 dispatch との統合呼び出しはイシュー #580 統合層の
    /// 責務、本関数は呼び出し側から明示的に呼ばれる契約）。
    pub fn reposition_all(window: &Window) {
        let Some(document) = window.document() else {
            return;
        };
        let Ok(list) = document.query_selector_all(OPEN_POSITIONER_SELECTOR) else {
            return;
        };
        for i in 0..list.length() {
            let Some(node) = list.item(i) else { continue };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            reposition_one(&element, window);
        }
    }

    /// scroll/resize イベントを契機に開いている positioner を再計算する
    /// 配線層（ADR §4.3・§6: `autoUpdate` 相当の連続監視は非採用、離散的な
    /// イベント駆動の再計算のみ）。
    ///
    /// `Closure::forget` は scroll/resize の 2 個のみに限定する
    /// （[`crate::events::wire_events`] と同じ「マウントがアプリ生存期間に
    /// 1 度」という前提でのリーク許容であり、無制限に増加しない）。
    ///
    /// scroll リスナーはキャプチャフェーズ（`useCapture: true`）で登録する
    /// （イシュー #622 レビュー指摘: `scroll` イベントはバブリングしない
    /// ため、`window` へバブリングフェーズのみで登録すると overflow
    /// コンテナ内側のスクロールを検知できず、スクロール可能なペイン内の
    /// menu/select/popover が古い座標のまま残ってしまう。キャプチャ
    /// フェーズは `window` から実イベントターゲットへ向けて先に通過する
    /// ため、`window` に登録しておけば任意の祖先要素上のスクロールも
    /// 捕捉できる、Floating UI 等が採用する既知のパターン）。`resize` は
    /// `window` 自身がターゲットのイベントであり capture/bubble の区別が
    /// 意味を持たないため既定（`false`、バブリングフェーズ）のままとする。
    pub struct PositionController {
        window: Window,
        scroll_closure: Closure<dyn FnMut(Event)>,
        resize_closure: Closure<dyn FnMut(Event)>,
    }

    impl PositionController {
        /// `window` へ `scroll`（キャプチャフェーズ）/`resize`（バブリング
        /// フェーズ）リスナーを登録する。
        ///
        /// # Errors
        ///
        /// `add_event_listener_with_callback_and_bool` が失敗した場合に
        /// `Err` を返す。
        pub fn new(window: &Window) -> Result<Self, JsValue> {
            let scroll_window = window.clone();
            let scroll_closure = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
                reposition_all(&scroll_window);
            });
            window.add_event_listener_with_callback_and_bool(
                "scroll",
                scroll_closure.as_ref().unchecked_ref(),
                true,
            )?;

            let resize_window = window.clone();
            let resize_closure = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
                reposition_all(&resize_window);
            });
            if let Err(err) = window
                .add_event_listener_with_callback("resize", resize_closure.as_ref().unchecked_ref())
            {
                let _ = window.remove_event_listener_with_callback_and_bool(
                    "scroll",
                    scroll_closure.as_ref().unchecked_ref(),
                    true,
                );
                return Err(err);
            }

            // `Closure::forget` はアプリ生存期間に 1 度だけ生成される想定の
            // 本コントローラに限り許容する（`events::wire_events` と同じ
            // 判断）。値は保持しつつリークさせるため、フィールドとして
            // 持ち続ける必要はないが、`Closure` を drop すると登録済み
            // リスナーが無効化される wasm-bindgen の仕様のため、
            // `forget()` せずフィールドに保持して `Self` の生存期間に
            // 結びつける（forget も選択肢だが、`Self` を明示的に破棄
            // できる設計を残すため保持を選ぶ）。
            Ok(Self {
                window: window.clone(),
                scroll_closure,
                resize_closure,
            })
        }

        /// 開いている positioner を即座に再計算する（マウント直後の初期
        /// 配置・呼び出し側が明示的にトリガーしたい場合に使う）。
        pub fn reposition_now(&self) {
            reposition_all(&self.window);
        }
    }

    impl Drop for PositionController {
        /// scroll/resize リスナーを対称的に解除する（`overlay::wiring` と
        /// 同じ、無制限リークを避ける設計）。`scroll` は登録時と同じ
        /// `useCapture: true` を指定する（DOM 仕様上 `removeEventListener`
        /// は capture フラグの一致でリスナーを識別するため、登録時と揃えない
        /// と解除が効かない）。
        fn drop(&mut self) {
            let _ = self.window.remove_event_listener_with_callback_and_bool(
                "scroll",
                self.scroll_closure.as_ref().unchecked_ref(),
                true,
            );
            let _ = self.window.remove_event_listener_with_callback(
                "resize",
                self.resize_closure.as_ref().unchecked_ref(),
            );
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::PositionController;

#[cfg(test)]
mod tests {
    use super::*;

    fn measurement() -> Measurement {
        Measurement {
            anchor: Rect {
                x: 100.0,
                y: 100.0,
                width: 50.0,
                height: 20.0,
            },
            floating: Size {
                width: 200.0,
                height: 80.0,
            },
            viewport: Size {
                width: 800.0,
                height: 600.0,
            },
        }
    }

    // --- PositionedKind::from_scope ---

    #[test]
    fn from_scope_recognizes_known_scopes() {
        assert_eq!(
            PositionedKind::from_scope("popover"),
            Some(PositionedKind::Popover)
        );
        assert_eq!(
            PositionedKind::from_scope("tooltip"),
            Some(PositionedKind::Tooltip)
        );
        assert_eq!(
            PositionedKind::from_scope("menu"),
            Some(PositionedKind::Menu)
        );
        assert_eq!(
            PositionedKind::from_scope("select"),
            Some(PositionedKind::Select)
        );
    }

    #[test]
    fn from_scope_rejects_unknown_scopes() {
        for bogus in ["dialog", "", "POPOVER", "popover "] {
            assert_eq!(PositionedKind::from_scope(bogus), None, "scope={bogus:?}");
        }
    }

    #[test]
    fn only_select_lacks_arrow() {
        assert!(PositionedKind::Popover.has_arrow());
        assert!(PositionedKind::Tooltip.has_arrow());
        assert!(PositionedKind::Menu.has_arrow());
        assert!(!PositionedKind::Select.has_arrow());
    }

    #[test]
    fn same_width_default_true_for_menu_and_select_only() {
        assert!(!PositionedKind::Popover.same_width_default());
        assert!(!PositionedKind::Tooltip.same_width_default());
        assert!(PositionedKind::Menu.same_width_default());
        assert!(PositionedKind::Select.same_width_default());
    }

    // --- data-side/data-align の fail-closed パース ---

    #[test]
    fn parse_side_attr_defaults_to_bottom_when_absent_or_invalid() {
        assert_eq!(parse_side_attr(None), Side::Bottom);
        assert_eq!(parse_side_attr(Some("diagonal")), Side::Bottom);
        assert_eq!(parse_side_attr(Some("top")), Side::Top);
    }

    #[test]
    fn parse_align_attr_defaults_to_center_when_absent_or_invalid() {
        assert_eq!(parse_align_attr(None), Align::Center);
        assert_eq!(parse_align_attr(Some("middle")), Align::Center);
        assert_eq!(parse_align_attr(Some("start")), Align::Start);
    }

    // --- resolve_position ---

    #[test]
    fn resolve_position_produces_css_vars_style_with_arrow_for_popover() {
        let result = resolve_position(
            PositionedKind::Popover,
            measurement(),
            Placement::new(Side::Bottom, Align::Center),
        );
        assert!(result.style.contains("--fandhe-x:"));
        assert!(result.style.contains("--fandhe-y:"));
        // Popover は same_width_default() == false のため
        // --fandhe-reference-width は出力されない（イシュー #622 レビュー
        // 指摘の回帰、`PositionedKind::same_width_default` 参照）。
        assert!(!result.style.contains("--fandhe-reference-width:"));
        assert!(result.style.contains("--fandhe-arrow-x:"));
        assert!(result.style.contains("--fandhe-arrow-y:"));
    }

    #[test]
    fn resolve_position_includes_reference_width_for_menu_and_select_only() {
        // same_width_default() が true の Menu/Select のみ
        // --fandhe-reference-width を出力し、false の Popover/Tooltip では
        // 出力しないことを resolve_position 経由（wasm-full 側）で固定する
        // （イシュー #622 レビュー指摘: same_width が実行時挙動に影響しない
        // 不具合の回帰。headless-ui 側の同種テストと二重化する）。
        for kind in [PositionedKind::Menu, PositionedKind::Select] {
            let result = resolve_position(
                kind,
                measurement(),
                Placement::new(Side::Bottom, Align::Center),
            );
            assert!(
                result.style.contains("--fandhe-reference-width:"),
                "kind={kind:?}"
            );
        }
        for kind in [PositionedKind::Popover, PositionedKind::Tooltip] {
            let result = resolve_position(
                kind,
                measurement(),
                Placement::new(Side::Bottom, Align::Center),
            );
            assert!(
                !result.style.contains("--fandhe-reference-width:"),
                "kind={kind:?}"
            );
        }
    }

    #[test]
    fn resolve_position_omits_arrow_vars_for_select() {
        let result = resolve_position(
            PositionedKind::Select,
            measurement(),
            Placement::new(Side::Bottom, Align::Center),
        );
        assert!(!result.style.contains("--fandhe-arrow-x:"));
        assert!(!result.style.contains("--fandhe-arrow-y:"));
    }

    #[test]
    fn resolve_position_flips_when_requested_placement_overflows() {
        let overflowing_measurement = Measurement {
            anchor: Rect {
                x: 100.0,
                y: 10.0,
                width: 50.0,
                height: 20.0,
            },
            ..measurement()
        };
        let result = resolve_position(
            PositionedKind::Tooltip,
            overflowing_measurement,
            Placement::new(Side::Top, Align::Center),
        );
        assert_eq!(result.side, Side::Bottom);
    }

    #[test]
    fn resolve_position_style_never_contains_quote_or_angle_bracket() {
        // XSS 回帰: 数値書式以外の文字（属性値エスケープの breakout に
        // 使われうる `"`/`<`/`>`）が混入しないことを wasm-full 側でも確認する
        // （headless-ui 側の同種テストと二重化して境界越えの回帰を防ぐ）。
        let result = resolve_position(
            PositionedKind::Menu,
            measurement(),
            Placement::new(Side::Right, Align::Start),
        );
        assert!(!result.style.contains('"'));
        assert!(!result.style.contains('<'));
        assert!(!result.style.contains('>'));
    }

    // --- resolve_requested_placement（イシュー #622 レビュー指摘: flip が
    // 希望 placement を上書きする不具合の回帰。`wiring::reposition_one` の
    // 決定ロジックを純粋関数として抽出し native `cargo test` で検証する） ---

    #[test]
    fn resolve_requested_placement_prefers_persisted_over_fallback() {
        // 永続化済みの希望（data-requested-*）があれば、現在の
        // data-side/data-align（flip 後の確定値かもしれない）より優先する。
        let placement =
            resolve_requested_placement(Some("top"), Some("start"), Some("bottom"), Some("center"));
        assert_eq!(placement.side(), Side::Top);
        assert_eq!(placement.align(), Align::Start);
    }

    #[test]
    fn resolve_requested_placement_falls_back_when_not_yet_persisted() {
        // 初回（data-requested-* 未設定）は現在の data-side/data-align を
        // 希望の初期値として採用する。
        let placement = resolve_requested_placement(None, None, Some("left"), Some("end"));
        assert_eq!(placement.side(), Side::Left);
        assert_eq!(placement.align(), Align::End);
    }

    #[test]
    fn resolve_requested_placement_defaults_when_nothing_present() {
        let placement = resolve_requested_placement(None, None, None, None);
        assert_eq!(placement.side(), Side::Bottom);
        assert_eq!(placement.align(), Align::Center);
    }

    #[test]
    fn resolve_requested_placement_ignores_unknown_values_fail_closed() {
        let placement = resolve_requested_placement(
            Some("diagonal"),
            Some("middle"),
            Some("top"),
            Some("start"),
        );
        assert_eq!(placement.side(), Side::Bottom);
        assert_eq!(placement.align(), Align::Center);
    }

    #[test]
    fn resolve_requested_placement_survives_repeated_flip_writebacks() {
        // 元の不具合の再現シナリオ: 希望は top/start のまま、flip により
        // data-side/data-align が bottom/start へ書き換わり続けたとしても、
        // data-requested-* が最初に "top"/"start" で永続化されていれば、
        // 何度目の呼び出しでも希望は top/start のまま変わらない。
        for flipped_side in ["bottom", "top", "bottom"] {
            let placement = resolve_requested_placement(
                Some("top"),
                Some("start"),
                Some(flipped_side),
                Some("start"),
            );
            assert_eq!(placement.side(), Side::Top, "flipped_side={flipped_side}");
            assert_eq!(placement.align(), Align::Start);
        }
    }
}
